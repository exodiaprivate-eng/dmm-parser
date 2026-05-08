// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Inventory + Tier 1.5 round-trip for the next batch of proprietary
//! formats (binarygimmick / prefab / pami / paac / pabc / pae / pat /
//! paem / palevel / levelinfo / motionblending / pbd / pab / paccd /
//! material / pamlod / pampg). For each: per-magic counts, sample
//! head bytes, lp_token_stream parse + serialize round-trip totals,
//! and a JSON spot-check.

use dmm_parser::binary::lp_token_stream::{LpTokenFile, Token};
use dmm_parser::binary::pamt::{Compression, PackMeta};
use lz4_flex::block::decompress;
use std::collections::BTreeMap;
use std::path::Path;

const TARGET_EXTS: &[&str] = &[
    "binarygimmick", "prefab", "pami", "paac", "pabc", "pae", "pat",
    "paem", "palevel", "levelinfo", "motionblending", "pbd", "pab",
    "paccd", "material", "pamlod", "pampg",
];

const MAX_FAILURE_REPORTS_PER_EXT: usize = 3;
const SAMPLE_HEAD_BYTES: usize = 16;

#[derive(Default, Clone)]
struct ExtStats {
    total: usize,
    pass: usize,
    fail: usize,
    failures_reported: usize,
    total_tokens: u64,
    total_lp_strings: u64,
    total_raw_bytes: u64,
    largest: (usize, String),
    smallest: (usize, String),
    magics: BTreeMap<[u8; 4], usize>,
    samples: Vec<(String, String, u32, [u8; 16])>,
    json_total: usize,
    json_pass: usize,
}

fn hex16(bytes: &[u8]) -> String {
    let mut out = String::new();
    for b in bytes.iter().take(16) {
        out.push_str(&format!("{:02X} ", b));
    }
    out
}

fn ascii16(bytes: &[u8]) -> String {
    let mut out = String::new();
    for b in bytes.iter().take(16) {
        out.push(if b.is_ascii_graphic() { *b as char } else { '.' });
    }
    out
}

fn main() {
    let game = Path::new(r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert");
    let mut groups: Vec<String> = std::fs::read_dir(game).unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.len() == 4 && n.chars().all(|c| c.is_ascii_digit()))
        .collect();
    groups.sort();

    let mut stats: BTreeMap<String, ExtStats> = BTreeMap::new();
    for ext in TARGET_EXTS {
        let mut s = ExtStats::default();
        s.smallest.0 = usize::MAX;
        stats.insert((*ext).into(), s);
    }

    for g in &groups {
        let pamt_p = game.join(g).join("0.pamt");
        let paz_p = game.join(g).join("0.paz");
        if !pamt_p.exists() || !paz_p.exists() { continue; }
        let Ok(pamt_data) = std::fs::read(&pamt_p) else { continue };
        let Ok(meta) = PackMeta::parse(&pamt_data, None) else { continue };
        let Ok(paz_data) = std::fs::read(&paz_p) else { continue };

        for d in &meta.directories {
            for f in &d.files {
                let ext = f.name.rsplit('.').next().unwrap_or("").to_lowercase();
                let Some(st) = stats.get_mut(&ext) else { continue };

                let off = f.file.chunk_offset as usize;
                let comp = f.file.compressed_size as usize;
                if off + comp > paz_data.len() { continue; }
                let comp_bytes = &paz_data[off..off + comp];
                let body = match f.file.compression {
                    Compression::Lz4 => match decompress(comp_bytes, f.file.uncompressed_size as usize) {
                        Ok(b) => b,
                        Err(_) => continue,
                    },
                    Compression::None => comp_bytes.to_vec(),
                    _ => continue,
                };

                if body.len() >= 4 {
                    let mut magic = [0u8; 4];
                    magic.copy_from_slice(&body[..4]);
                    *st.magics.entry(magic).or_insert(0) += 1;
                }
                if st.samples.len() < 4 {
                    let mut head = [0u8; 16];
                    let n = body.len().min(SAMPLE_HEAD_BYTES);
                    head[..n].copy_from_slice(&body[..n]);
                    st.samples.push((g.clone(), format!("{}/{}", d.path, f.name), f.file.uncompressed_size, head));
                }

                st.total += 1;
                if body.len() > st.largest.0 {
                    st.largest = (body.len(), format!("{} {}/{}", g, d.path, f.name));
                }
                if body.len() < st.smallest.0 {
                    st.smallest = (body.len(), format!("{} {}/{}", g, d.path, f.name));
                }

                let parsed = match LpTokenFile::parse(&body) {
                    Ok(p) => p,
                    Err(e) => {
                        st.fail += 1;
                        if st.failures_reported < MAX_FAILURE_REPORTS_PER_EXT {
                            println!(".{} PARSE-ERR {}/{}: {}", ext, d.path, f.name, e);
                            st.failures_reported += 1;
                        }
                        continue;
                    }
                };
                let written = match parsed.to_bytes() {
                    Ok(b) => b,
                    Err(e) => {
                        st.fail += 1;
                        if st.failures_reported < MAX_FAILURE_REPORTS_PER_EXT {
                            println!(".{} WRITE-ERR {}/{}: {}", ext, d.path, f.name, e);
                            st.failures_reported += 1;
                        }
                        continue;
                    }
                };
                if written == body {
                    st.pass += 1;
                    st.total_tokens += parsed.tokens.len() as u64;
                    for t in &parsed.tokens {
                        match t {
                            Token::LpString(b) => st.total_lp_strings += b.len() as u64,
                            Token::RawBytes(b) => st.total_raw_bytes += b.len() as u64,
                        }
                    }
                    if st.json_total < 25 {
                        st.json_total += 1;
                        let json = parsed.to_json();
                        if let Ok(rebuilt) = LpTokenFile::from_json(&json) {
                            if let Ok(rb) = rebuilt.to_bytes() {
                                if rb == body { st.json_pass += 1; }
                            }
                        }
                    }
                } else {
                    st.fail += 1;
                    if st.failures_reported < MAX_FAILURE_REPORTS_PER_EXT {
                        let first_diff = (0..body.len().min(written.len()))
                            .find(|&i| body[i] != written[i])
                            .unwrap_or_else(|| body.len().min(written.len()));
                        println!(".{} MISMATCH {}/{}: in={} out={} first_diff=0x{:X}",
                            ext, d.path, f.name, body.len(), written.len(), first_diff);
                        st.failures_reported += 1;
                    }
                }
            }
        }
    }

    println!("\n=== Bulk round-trip summary ===");
    println!("{:<18} {:>8} {:>8} {:>8} {:>10} {:>10} {:>8}",
        "extension", "files", "pass", "fail", "tokens/f", "string %", "json/25");
    let mut grand_total = 0;
    let mut grand_pass = 0;
    for ext in TARGET_EXTS {
        let st = stats.get(*ext).unwrap();
        let avg_tokens = if st.pass > 0 { st.total_tokens as f64 / st.pass as f64 } else { 0.0 };
        let total_body = (st.total_lp_strings + st.total_raw_bytes) as f64;
        let lp_pct = if total_body > 0.0 { 100.0 * st.total_lp_strings as f64 / total_body } else { 0.0 };
        println!("{:<18} {:>8} {:>8} {:>8} {:>10.1} {:>9.1}% {:>4}/{:<4}",
            ext, st.total, st.pass, st.fail, avg_tokens, lp_pct, st.json_pass, st.json_total);
        grand_total += st.total;
        grand_pass += st.pass;
    }
    println!("{:<18} {:>8} {:>8}", "TOTAL", grand_total, grand_pass);

    println!("\n=== Magics + samples per extension ===");
    for ext in TARGET_EXTS {
        let st = stats.get(*ext).unwrap();
        if st.total == 0 {
            println!("\n.{} : (no files found)", ext);
            continue;
        }
        println!("\n.{} : {} files (pass {}, fail {})",
            ext, st.total, st.pass, st.fail);
        let mut sorted: Vec<_> = st.magics.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        for (magic, count) in sorted.iter().take(5) {
            let ascii: String = magic.iter()
                .map(|b| if b.is_ascii_graphic() { *b as char } else { '.' })
                .collect();
            println!("  magic {:02X} {:02X} {:02X} {:02X} ({}) [{}]",
                magic[0], magic[1], magic[2], magic[3], ascii, count);
        }
        for (g, path, size, head) in &st.samples {
            println!("  group {} | {:>10} bytes | {} | {} | {}",
                g, size, hex16(head), ascii16(head), path);
        }
    }
}
