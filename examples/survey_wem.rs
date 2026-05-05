// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Inventory .wem files in vanilla. Confirm RIFF magic distribution
//! and which chunk IDs appear so we know what the typed parser needs
//! to handle. Reads raw .paz bytes (DDS pattern: stored uncompressed).

use dmm_parser::binary::pamt::PackMeta;
use std::collections::BTreeMap;
use std::path::Path;

fn fourcc_str(v: [u8; 4]) -> String {
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
    let mut form_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut chunk_id_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut sample_paths: Vec<String> = Vec::new();
    let mut largest = (0u64, String::new());

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
                if ext != "wem" { continue; }
                total += 1;
                total_bytes += f.file.uncompressed_size as u64;
                if f.file.uncompressed_size as u64 > largest.0 {
                    largest = (f.file.uncompressed_size as u64, format!("{} {}/{}", g, d.path, f.name));
                }
                *crypto_counts.entry(format!("{:?}", f.file.crypto)).or_insert(0) += 1;
                *compression_counts.entry(format!("{:?}", f.file.compression)).or_insert(0) += 1;

                let cid = f.file.chunk_id;
                let paz = paz_cache.entry(cid).or_insert_with(|| {
                    std::fs::read(group_dir.join(format!("{}.paz", cid))).unwrap_or_default()
                });
                if paz.is_empty() { continue; }
                let off = f.file.chunk_offset as usize;
                let csz = f.file.compressed_size as usize;
                if off + csz > paz.len() || csz < 12 { continue; }
                let stored = &paz[off..off + csz];

                let mut magic = [0u8; 4];
                magic.copy_from_slice(&stored[..4]);
                *magic_counts.entry(fourcc_str(magic)).or_insert(0) += 1;

                if &magic == b"RIFF" || &magic == b"RIFX" {
                    let mut form = [0u8; 4];
                    form.copy_from_slice(&stored[8..12]);
                    *form_counts.entry(fourcc_str(form)).or_insert(0) += 1;

                    // Walk chunks (only first ~5 to keep work bounded)
                    let mut p = 12usize;
                    let mut walked = 0;
                    while p + 8 <= stored.len() && walked < 5 {
                        let mut id = [0u8; 4];
                        id.copy_from_slice(&stored[p..p + 4]);
                        let chunk_size = u32::from_le_bytes(stored[p + 4..p + 8].try_into().unwrap());
                        *chunk_id_counts.entry(fourcc_str(id)).or_insert(0) += 1;
                        // RIFF chunks pad to even length
                        let aligned = (chunk_size as usize + 1) & !1;
                        p = p.saturating_add(8).saturating_add(aligned);
                        walked += 1;
                        if chunk_size == 0 { break; }
                    }
                }

                if sample_paths.len() < 5 {
                    sample_paths.push(format!("{} bytes  {} {}/{}",
                        stored.len(), g, d.path, f.name));
                }
            }
        }
    }

    println!("=== WEM inventory ===");
    println!("Total .wem files:    {}", total);
    println!("Total bytes:         {} ({:.2} GB)", total_bytes, total_bytes as f64 / (1024.0 * 1024.0 * 1024.0));
    println!("Largest:             {} bytes  {}", largest.0, largest.1);
    println!("\n=== Crypto / compression ===");
    for (k, v) in &crypto_counts { println!("  crypto      {:>15} : {}", k, v); }
    for (k, v) in &compression_counts { println!("  compression {:>15} : {}", k, v); }
    println!("\n=== First-4-byte magic ===");
    let mut m: Vec<_> = magic_counts.iter().collect();
    m.sort_by(|a, b| b.1.cmp(a.1));
    for (k, v) in m.iter().take(10) {
        println!("  {:>10} : {}", format!("\"{}\"", k), v);
    }
    println!("\n=== RIFF form type (bytes 8..12) ===");
    let mut f: Vec<_> = form_counts.iter().collect();
    f.sort_by(|a, b| b.1.cmp(a.1));
    for (k, v) in &f { println!("  {:>10} : {}", format!("\"{}\"", k), v); }
    println!("\n=== Top chunk IDs (across first 5 chunks of each file) ===");
    let mut c: Vec<_> = chunk_id_counts.iter().collect();
    c.sort_by(|a, b| b.1.cmp(a.1));
    for (k, v) in c.iter().take(15) {
        println!("  {:>10} : {}", format!("\"{}\"", k), v);
    }
    println!("\n=== Samples ===");
    for s in &sample_paths { println!("  {}", s); }
}
