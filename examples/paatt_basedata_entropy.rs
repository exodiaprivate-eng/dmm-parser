// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt

//! Differential analysis tool — runs across every vanilla `.paatt`
//! BaseData blob, computes per-byte and per-u32 value distributions,
//! and emits a byte-by-byte structure map. Pairs with the field
//! directory in `docs/PAATT_BASEDATA_FIELDS.md` to surface likely
//! field boundaries inside BaseData.
//!
//! Output (per version):
//!   - sample count
//!   - per-byte value count + most common value
//!   - 4-byte interpretations: hint at f32 / u32 / pointer / count
//!   - byte alignment guesses based on always-zero bytes

use dmm_parser::binary::paatt::PaattFile;
use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paz;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";

#[derive(Default)]
struct ByteStats {
    /// value -> count
    distribution: HashMap<u8, u32>,
    total: u32,
}

impl ByteStats {
    fn add(&mut self, b: u8) {
        *self.distribution.entry(b).or_insert(0) += 1;
        self.total += 1;
    }

    fn distinct(&self) -> usize { self.distribution.len() }

    /// Most common value + how many times it appears.
    fn mode(&self) -> (u8, u32) {
        self.distribution.iter()
            .max_by_key(|&(_, c)| *c)
            .map(|(&v, &c)| (v, c))
            .unwrap_or((0, 0))
    }

    /// Hint: "always-zero", "binary 0/1", "low-cardinality", "high-entropy"
    fn classify(&self) -> &'static str {
        let (mode_val, mode_count) = self.mode();
        let frac = mode_count as f64 / self.total as f64;
        if mode_val == 0 && frac > 0.99 { return "always-0"; }
        if self.distinct() == 1 { return "always-const"; }
        if self.distinct() == 2 && self.distribution.contains_key(&0) && self.distribution.contains_key(&1) {
            return "bool";
        }
        if self.distinct() <= 16 { return "low-card (enum?)"; }
        if frac > 0.95 { return "near-const"; }
        "high-entropy"
    }
}

fn main() {
    let game_dir = Path::new(GAME_DIR);
    if !game_dir.exists() { eprintln!("Game install not found"); std::process::exit(1); }
    let mut groups: Vec<String> = std::fs::read_dir(game_dir)
        .expect("read game dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.len() == 4 && n.chars().all(|c| c.is_ascii_digit()))
        .collect();
    groups.sort();

    // version -> (sample_count, per_byte_stats)
    let mut by_version: BTreeMap<u8, (usize, Vec<ByteStats>)> = BTreeMap::new();

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
                        .or_insert_with(|| (0, Vec::new()));
                    entry.0 += 1;
                    if entry.1.is_empty() {
                        entry.1.resize_with(info.base_data.len(), ByteStats::default);
                    }
                    if entry.1.len() != info.base_data.len() {
                        // Skip mis-sized records.
                        continue;
                    }
                    for (i, &b) in info.base_data.iter().enumerate() {
                        entry.1[i].add(b);
                    }
                }
            }
        }
    }

    for (version, (count, stats)) in &by_version {
        println!("\n========================================");
        println!("Version {}: {} infos, BaseData = {} bytes", version, count, stats.len());
        println!("========================================");
        println!("offset  cls                      distinct  mode  mode%");
        for (i, s) in stats.iter().enumerate() {
            let (m_val, m_cnt) = s.mode();
            let pct = m_cnt as f64 / s.total as f64 * 100.0;
            println!("0x{:04x}  {:25}  {:>8}  0x{:02x}  {:>5.1}%",
                i, s.classify(), s.distinct(), m_val, pct);
        }
    }
}
