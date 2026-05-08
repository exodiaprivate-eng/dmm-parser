// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Prefab format survey. 34k files, ReflectObject-based per roadmap.
//! Need to understand magic + header + structure before designing the
//! typed parser.

use dmm_parser::binary::pamt::PackMeta;
use std::collections::BTreeMap;
use std::path::Path;

fn fourcc(v: [u8; 4]) -> String {
    v.iter().map(|b| if (0x20..=0x7E).contains(b) { *b as char } else { '?' }).collect()
}

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
    let mut magic_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut size_buckets: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut samples: Vec<(String, usize, [u8; 64])> = Vec::new();

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
                if ext != "prefab" { continue; }
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
                if off + csz > paz.len() || csz < 16 { continue; }
                let stored = &paz[off..off + csz];

                let mut m = [0u8; 4];
                m.copy_from_slice(&stored[..4]);
                *magic_counts.entry(fourcc(m)).or_insert(0) += 1;

                let bucket = match stored.len() {
                    0..=512 => "0-512",
                    513..=2048 => "512-2KB",
                    2049..=8192 => "2-8KB",
                    8193..=32768 => "8-32KB",
                    _ => "32KB+",
                };
                *size_buckets.entry(bucket).or_insert(0) += 1;

                if samples.len() < 5 && stored.len() >= 64 {
                    let mut head = [0u8; 64];
                    head.copy_from_slice(&stored[..64]);
                    samples.push((format!("{} {}/{}", g, d.path, f.name), stored.len(), head));
                }
            }
        }
    }

    println!("=== Prefab inventory ===");
    println!("Total .prefab files: {}", total);
    println!("Total bytes: {} ({:.2} MB)", total_bytes, total_bytes as f64 / (1024.0 * 1024.0));

    println!("\n=== Crypto / compression ===");
    for (k, v) in &crypto_counts { println!("  crypto      {:>15} : {}", k, v); }
    for (k, v) in &compression_counts { println!("  compression {:>15} : {}", k, v); }

    println!("\n=== First-4-byte magic (top 10) ===");
    let mut mag: Vec<_> = magic_counts.iter().collect();
    mag.sort_by(|a, b| b.1.cmp(a.1));
    for (k, v) in mag.iter().take(10) {
        println!("  \"{}\" : {}", k, v);
    }

    println!("\n=== Size buckets ===");
    for (k, v) in &size_buckets { println!("  {:>10} : {}", k, v); }

    println!("\n=== Sample heads (64 bytes) ===");
    for (name, size, head) in &samples {
        println!("\n  {} ({} bytes)", name, size);
        for chunk in head.chunks(16) {
            print!("    ");
            for b in chunk { print!("{:02X} ", b); }
            print!(" ");
            for b in chunk { print!("{}", if b.is_ascii_graphic() { *b as char } else { '.' }); }
            println!();
        }
    }
}
