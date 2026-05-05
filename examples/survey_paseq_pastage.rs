// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Inventory every .paseq and .pastage in vanilla. For each, print size +
//! first 16 magic bytes after decompression so we can spot the magic / shape
//! of the format. Group by file kind. Used as a starting point for round-trip
//! parsers.

use dmm_parser::binary::pamt::{Compression, PackMeta};
use lz4_flex::block::decompress;
use std::collections::BTreeMap;
use std::path::Path;

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

    // Per-extension summary: count, min/max size, distinct magics seen
    #[derive(Default)]
    struct Stat {
        count: usize,
        total_bytes: u64,
        min_size: u64,
        max_size: u64,
        magic_counts: BTreeMap<[u8; 4], usize>,
    }
    let mut stats: BTreeMap<String, Stat> = BTreeMap::new();
    let mut samples_per_ext: BTreeMap<String, Vec<(String, String, u32, [u8; 16])>> = BTreeMap::new();

    let target_exts: &[&str] = &["paseq", "pastage", "pastag"];

    for g in &groups {
        let pamt_p = game.join(g).join("0.pamt");
        let paz_p = game.join(g).join("0.paz");
        if !pamt_p.exists() { continue; }
        let Ok(pamt_data) = std::fs::read(&pamt_p) else { continue };
        let Ok(meta) = PackMeta::parse(&pamt_data, None) else { continue };
        let paz_data = std::fs::read(&paz_p).ok();

        for d in &meta.directories {
            for f in &d.files {
                let ext = f.name.rsplit('.').next().unwrap_or("").to_lowercase();
                if !target_exts.contains(&ext.as_str()) { continue; }

                let st = stats.entry(ext.clone()).or_default();
                st.count += 1;
                st.total_bytes += f.file.uncompressed_size as u64;
                if st.min_size == 0 || (f.file.uncompressed_size as u64) < st.min_size {
                    st.min_size = f.file.uncompressed_size as u64;
                }
                if (f.file.uncompressed_size as u64) > st.max_size {
                    st.max_size = f.file.uncompressed_size as u64;
                }

                // Decompress + read first 16 bytes for magic
                let mut head = [0u8; 16];
                let mut got_head = false;
                if let Some(paz) = &paz_data {
                    let off = f.file.chunk_offset as usize;
                    let comp = f.file.compressed_size as usize;
                    if off + comp <= paz.len() {
                        let comp_bytes = &paz[off..off + comp];
                        let body_res: Result<Vec<u8>, _> = match f.file.compression {
                            Compression::Lz4 => decompress(comp_bytes, f.file.uncompressed_size as usize)
                                .map_err(|e| format!("{:?}", e)),
                            Compression::None => Ok(comp_bytes.to_vec()),
                            other => Err(format!("compression {:?}", other)),
                        };
                        if let Ok(body) = body_res {
                            let n = body.len().min(16);
                            head[..n].copy_from_slice(&body[..n]);
                            got_head = true;
                            if body.len() >= 4 {
                                let mut magic = [0u8; 4];
                                magic.copy_from_slice(&body[..4]);
                                *st.magic_counts.entry(magic).or_insert(0) += 1;
                            }
                        }
                    }
                }

                if got_head {
                    let samples = samples_per_ext.entry(ext.clone()).or_default();
                    if samples.len() < 8 {
                        samples.push((g.clone(), format!("{}/{}", d.path, f.name), f.file.uncompressed_size, head));
                    }
                }
            }
        }
    }

    println!("=== Per-extension summary ===");
    for (ext, st) in &stats {
        println!("\n.{} : {} files, {} total bytes (min {}, max {})",
            ext, st.count, st.total_bytes, st.min_size, st.max_size);
        println!("  Distinct first-4-byte magics:");
        let mut sorted: Vec<_> = st.magic_counts.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        for (magic, count) in sorted.iter().take(8) {
            let ascii: String = magic.iter()
                .map(|b| if b.is_ascii_graphic() { *b as char } else { '.' })
                .collect();
            println!("    {:02X} {:02X} {:02X} {:02X} ({:>4}) [{}]",
                magic[0], magic[1], magic[2], magic[3], ascii, count);
        }
    }

    println!("\n=== Sample files per extension (first 16 bytes) ===");
    for (ext, samples) in &samples_per_ext {
        println!("\n.{}", ext);
        for (group, path, size, head) in samples {
            println!("  group {} | {:>10} bytes | {} | {} | {}",
                group, size, hex16(head), ascii16(head), path);
        }
    }
}
