// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt

//! Enhanced BaseData layout inference for `.paatt`.
//!
//! For every version, groups consecutive bytes into likely field boundaries by:
//!   1. 4-byte group correlation: if the group has far fewer distinct u32
//!      values than the product of per-byte distincts, the bytes move together
//!      (one field).  Otherwise they are independent.
//!   2. IEEE-float detection: for 4-byte groups, what fraction of samples
//!      have a float-like exponent (e.g. covers ±[1e-5, 65536]).
//!   3. Constant / near-constant detection.
//!
//! Output is a compact layout table per version that can be copy-pasted into
//! the BaseData struct implementation.

use dmm_parser::binary::paatt::PaattFile;
use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paz;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

const GAME_DIR: &str = "/mnt/d/SteamLibrary/steamapps/common/Crimson Desert";

// ── per-byte stats (same as entropy tool) ─────────────────────────────────

#[derive(Default)]
struct ByteStats {
    dist: HashMap<u8, u32>,
    total: u32,
}
impl ByteStats {
    fn add(&mut self, b: u8) { *self.dist.entry(b).or_insert(0) += 1; self.total += 1; }
    fn distinct(&self) -> usize { self.dist.len() }
    fn mode(&self) -> (u8, u32) {
        self.dist.iter().max_by_key(|&(_, &c)| c).map(|(&v, &c)| (v, c)).unwrap_or((0, 0))
    }
    fn always_zero(&self) -> bool { self.distinct() == 1 && self.mode().0 == 0 }
    fn always_const(&self) -> bool { self.distinct() == 1 }
}

// ── 4-byte group stats ────────────────────────────────────────────────────

#[derive(Default)]
struct GroupStats {
    dist: HashMap<u32, u32>,
    total: u32,
}
impl GroupStats {
    fn add(&mut self, bytes: [u8; 4]) {
        let v = u32::from_le_bytes(bytes);
        *self.dist.entry(v).or_insert(0) += 1;
        self.total += 1;
    }
    fn distinct(&self) -> usize { self.dist.len() }
    fn mode(&self) -> (u32, u32) {
        self.dist.iter().max_by_key(|&(_, &c)| c).map(|(&v, &c)| (v, c)).unwrap_or((0, 0))
    }

    /// % of samples whose LE-float exponent suggests a real game float
    /// (covers roughly ±[1e-7 .. 1e7] and zero).
    fn float_fraction(&self) -> f64 {
        let float_like: u32 = self.dist.iter().filter(|&(&v, _)| {
            let b = v.to_le_bytes();
            let exp_sign = b[3];
            let exp = exp_sign & 0x7f;
            // exponent 0 = ±zero/subnormal, 0x3e..0x4e ≈ 0.25..8M, 0x00 = zero
            exp == 0 || (exp >= 0x35 && exp <= 0x50)
        }).map(|(_, &c)| c).sum();
        float_like as f64 / self.total as f64
    }

    /// "Correlation ratio": group_distinct / (byte_product_distinct).
    /// Near 1.0 = bytes independent; much < 1.0 = bytes move together.
    fn correlation_ratio(&self, byte_stats: &[ByteStats; 4]) -> f64 {
        let prod: usize = byte_stats.iter().map(|b| b.distinct().max(1)).product();
        self.distinct() as f64 / prod as f64
    }
}

// ── main ──────────────────────────────────────────────────────────────────

fn main() {
    let game_dir = Path::new(GAME_DIR);
    if !game_dir.exists() { eprintln!("Game install not found at {}", GAME_DIR); std::process::exit(1); }
    let mut groups: Vec<String> = std::fs::read_dir(game_dir)
        .expect("read game dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.len() == 4 && n.chars().all(|c| c.is_ascii_digit()))
        .collect();
    groups.sort();

    // version -> (sample_count, per_byte_stats[N], per_4byte_group_stats[N/4])
    let mut by_version: BTreeMap<u8, (usize, Vec<ByteStats>, Vec<GroupStats>)> = BTreeMap::new();

    for group_name in &groups {
        let group_dir = game_dir.join(group_name);
        let pamt_data = match std::fs::read(group_dir.join("0.pamt")) { Ok(d) => d, Err(_) => continue };
        let pamt = match PackMeta::parse(&pamt_data, None) { Ok(p) => p, Err(_) => continue };
        let encrypt_info = pamt.header.encrypt_info.encrypt_info;
        for dir in &pamt.directories {
            for f in &dir.files {
                if !f.name.to_ascii_lowercase().ends_with(".paatt") { continue; }
                let bytes = match paz::extract_file(&group_dir, f, &dir.path, &encrypt_info) {
                    Ok(b) => b, Err(_) => continue,
                };
                let paatt = match PaattFile::parse(&bytes) { Ok(p) => p, Err(_) => continue };
                for info in &paatt.infos {
                    let entry = by_version.entry(info.version)
                        .or_insert_with(|| (0, Vec::new(), Vec::new()));
                    entry.0 += 1;
                    let n = info.base_data.len();
                    if entry.1.is_empty() {
                        entry.1 = (0..n).map(|_| ByteStats::default()).collect();
                        entry.2 = (0..n/4).map(|_| GroupStats::default()).collect();
                    }
                    if entry.1.len() != n { continue; }
                    for (i, &b) in info.base_data.iter().enumerate() {
                        entry.1[i].add(b);
                    }
                    for g in 0..n/4 {
                        let base = g * 4;
                        let quad: [u8; 4] = info.base_data[base..base+4].try_into().unwrap();
                        entry.2[g].add(quad);
                    }
                }
            }
        }
    }

    for (version, (count, byte_stats, group_stats)) in &by_version {
        let n = byte_stats.len();
        println!("\n════════════════════════════════════════════════════════");
        println!("Version {} — {} infos, {} bytes BaseData", version, count, n);
        println!("════════════════════════════════════════════════════════");
        println!("{:<8} {:<12} {:<8} {:<8} {:<10} {:<12} {:<28} {}",
            "offset", "4b_class", "4b_dist", "corr%", "float%", "mode_u32_hex", "mode_f32", "byte_summary");

        for g in 0..n/4 {
            let base = g * 4;
            let gs = &group_stats[g];
            let bs: &[ByteStats; 4] = (&byte_stats[base..base+4]).try_into().unwrap();

            let corr = gs.correlation_ratio(bs);
            let ff = gs.float_fraction();
            let (mode_u32, mode_count) = gs.mode();
            let mode_frac = mode_count as f64 / gs.total as f64;
            let mode_f32 = f32::from_le_bytes(mode_u32.to_le_bytes());

            // Classify the group
            let class = if gs.distinct() == 1 {
                if mode_u32 == 0 { "ZERO" } else { "CONST" }
            } else if mode_frac > 0.999 {
                "NEAR-CONST"
            } else if ff > 0.85 && corr < 0.15 {
                "f32?"
            } else if corr < 0.08 && gs.distinct() > 100 {
                "u32?"
            } else if corr < 0.3 && gs.distinct() > 20 {
                "u32/f32?"
            } else {
                "bytes"
            };

            // Byte-level summary
            let byte_summary: String = bs.iter().enumerate().map(|(i, b)| {
                let (mv, mc) = b.mode();
                let pct = mc as f64 / b.total as f64 * 100.0;
                let tag = if b.always_zero() { "Z" }
                    else if b.always_const() { "C" }
                    else if b.distinct() == 2 && b.dist.contains_key(&0) && b.dist.contains_key(&1) { "B" }
                    else if b.distinct() <= 8 { "e" }
                    else { "v" };
                format!("[{i}]{tag}{:02x}@{:.0}%", mv, pct)
            }).collect::<Vec<_>>().join(" ");

            println!("0x{:04x}  {:<12} {:<8} {:<8} {:<10} {:08x}({:>5.1}%) {:>12.4}    {}",
                base, class, gs.distinct(),
                format!("{:.1}%", corr * 100.0),
                format!("{:.1}%", ff * 100.0),
                mode_u32, mode_frac * 100.0,
                mode_f32,
                byte_summary,
            );
        }

        // Print remaining odd bytes (if size not divisible by 4)
        let rem = n % 4;
        if rem > 0 {
            print!("0x{:04x}  [tail {} bytes]", n - rem, rem);
            for i in n-rem..n {
                let (mv, mc) = byte_stats[i].mode();
                let pct = mc as f64 / byte_stats[i].total as f64 * 100.0;
                print!("  {:02x}@{:.0}%", mv, pct);
            }
            println!();
        }
    }
}
