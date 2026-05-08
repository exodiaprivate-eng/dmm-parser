// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt

//! Extract a few `.paseqc` files for hex inspection. Prints the first
//! 256 bytes of each sample with ASCII column.

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paz;
use std::path::Path;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";
const OUT_DIR: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser\target\paseqc_samples";
const SAMPLE_LIMIT: usize = 5;

fn main() {
    let game_dir = Path::new(GAME_DIR);
    std::fs::create_dir_all(OUT_DIR).unwrap();
    let mut groups: Vec<String> = std::fs::read_dir(game_dir)
        .expect("read game dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.len() == 4 && n.chars().all(|c| c.is_ascii_digit()))
        .collect();
    groups.sort();

    let mut extracted = 0usize;
    'outer: for group_name in &groups {
        let group_dir = game_dir.join(group_name);
        let pamt_data = match std::fs::read(group_dir.join("0.pamt")) { Ok(d) => d, Err(_) => continue };
        let pamt = match PackMeta::parse(&pamt_data, None) { Ok(p) => p, Err(_) => continue };
        let encrypt_info = pamt.header.encrypt_info.encrypt_info;
        for dir in &pamt.directories {
            for f in &dir.files {
                if !f.name.to_ascii_lowercase().ends_with(".paseqc") {
                    continue;
                }
                let bytes = match paz::extract_file(&group_dir, f, &dir.path, &encrypt_info) {
                    Ok(b) => b,
                    Err(_) => continue,
                };
                let bin = format!("{}/{:02}_{}", OUT_DIR, extracted, f.name);
                let hex = format!("{}.hex", bin);
                std::fs::write(&bin, &bytes).unwrap();
                let mut h = String::new();
                let limit = bytes.len().min(if bytes.len() < 2048 { bytes.len() } else { 4096 });
                for (i, chunk) in bytes[..limit].chunks(16).enumerate() {
                    let off = i * 16;
                    h.push_str(&format!("{:08x}  ", off));
                    for b in chunk { h.push_str(&format!("{:02x} ", b)); }
                    for _ in chunk.len()..16 { h.push_str("   "); }
                    h.push_str(" ");
                    for b in chunk {
                        h.push(if (0x20..=0x7e).contains(b) { *b as char } else { '.' });
                    }
                    h.push('\n');
                }
                std::fs::write(&hex, &h).unwrap();
                println!("[{}] {} -> {} ({} bytes)", extracted, f.name, bin, bytes.len());
                extracted += 1;
                if extracted >= SAMPLE_LIMIT { break 'outer; }
            }
        }
    }
    println!("\nExtracted {} samples to {}", extracted, OUT_DIR);
}
