// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Inventory .hkx files. Havok binary format. Two known wire formats:
//!   - Tagfile: starts with 0x05CA0010 magic
//!   - Packfile: starts with 0x57E0E057 magic
//! Find what crimson uses.

use dmm_parser::binary::pamt::PackMeta;
use std::collections::BTreeMap;
use std::path::Path;

fn main() {
    let game = Path::new(r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert");
    let mut groups: Vec<String> = std::fs::read_dir(game).unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.len() == 4 && n.chars().all(|c| c.is_ascii_digit()))
        .collect();
    groups.sort();

    let mut total: usize = 0;
    let mut total_bytes: u64 = 0;
    let mut crypto_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut compression_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut magic_counts: BTreeMap<u32, usize> = BTreeMap::new();
    let mut samples: Vec<(String, [u8; 32])> = Vec::new();

    for g in &groups {
        let group_dir = game.join(g);
        let pamt_p = group_dir.join("0.pamt");
        if !pamt_p.exists() { continue; }
        let Ok(pamt_data) = std::fs::read(&pamt_p) else { continue };
        let Ok(meta) = PackMeta::parse(&pamt_data, None) else { continue };

        let mut paz_cache: std::collections::HashMap<u16, Vec<u8>> = std::collections::HashMap::new();
        for d in &meta.directories {
            for f in &d.files {
                let ext = f.name.rsplit('.').next().unwrap_or("").to_lowercase();
                if ext != "hkx" { continue; }
                total += 1;
                total_bytes += f.file.uncompressed_size as u64;
                *crypto_counts.entry(format!("{:?}", f.file.crypto)).or_insert(0) += 1;
                *compression_counts.entry(format!("{:?}", f.file.compression)).or_insert(0) += 1;

                let cid = f.file.chunk_id;
                let paz = paz_cache.entry(cid).or_insert_with(|| {
                    std::fs::read(group_dir.join(format!("{}.paz", cid))).unwrap_or_default()
                });
                if paz.is_empty() { continue; }
                let off = f.file.chunk_offset as usize;
                let csz = f.file.compressed_size as usize;
                if off + csz > paz.len() || csz < 4 { continue; }
                let stored = &paz[off..off + csz];

                let m = u32::from_le_bytes(stored[..4].try_into().unwrap());
                *magic_counts.entry(m).or_insert(0) += 1;

                if samples.len() < 5 && stored.len() >= 32 {
                    let mut head = [0u8; 32];
                    head.copy_from_slice(&stored[..32]);
                    samples.push((format!("{} {}/{}", g, d.path, f.name), head));
                }
            }
        }
    }

    println!("=== HKX inventory ===");
    println!("Total .hkx files: {}", total);
    println!("Total bytes:      {} ({:.2} GB)", total_bytes, total_bytes as f64 / (1024.0 * 1024.0 * 1024.0));
    println!("\n=== Crypto / compression ===");
    for (k, v) in &crypto_counts { println!("  crypto      {:>15} : {}", k, v); }
    for (k, v) in &compression_counts { println!("  compression {:>15} : {}", k, v); }
    println!("\n=== First-4-byte magic (top 10) ===");
    let mut m: Vec<_> = magic_counts.iter().collect();
    m.sort_by(|a, b| b.1.cmp(a.1));
    for (k, v) in m.iter().take(10) {
        let bytes = k.to_le_bytes();
        let ascii: String = bytes.iter().map(|b| if (0x20..=0x7E).contains(b) { *b as char } else { '?' }).collect();
        println!("  0x{:08X} ({}) : {}", k, ascii, v);
    }
    println!("\n=== Sample heads (32 bytes) ===");
    for (name, head) in &samples {
        print!("  {}\n  ", name);
        for b in head { print!("{:02X} ", b); }
        println!();
    }
}
