// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt

//! Extract a few `.paseq` files from the game install for manual
//! byte-level analysis. Writes raw bytes + hex dumps to
//! `target/paseq_samples/`. Walks every group's PAMT to find samples.

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paz;
use std::path::Path;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";
const OUT_DIR: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser\target\paseq_samples";
const SAMPLE_LIMIT: usize = 5;
/// If set, only extract samples whose file name matches this needle.
const NAME_NEEDLE: &str = "cd_ui_hud_questmessage_complete";

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
        let pamt_data = match std::fs::read(group_dir.join("0.pamt")) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let pamt = match PackMeta::parse(&pamt_data, None) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let encrypt_info = pamt.header.encrypt_info.encrypt_info;
        for dir in &pamt.directories {
            for f in &dir.files {
                if !f.name.to_ascii_lowercase().ends_with(".paseq") {
                    continue;
                }
                if !NAME_NEEDLE.is_empty() && !f.name.contains(NAME_NEEDLE) {
                    continue;
                }
                let bytes = match paz::extract_file(&group_dir, f, &dir.path, &encrypt_info) {
                    Ok(b) => b,
                    Err(_) => continue,
                };
                let bin_path = format!("{}/{:02}_{}", OUT_DIR, extracted, f.name);
                let hex_path = format!("{}.hex", bin_path);
                std::fs::write(&bin_path, &bytes).unwrap();

                let mut hex = String::new();
                // Full hex for the smallest samples (need to walk
                // value-section). Cap larger ones at 4KB to keep file
                // small but still useful.
                let limit = bytes.len().min(if bytes.len() < 2048 { bytes.len() } else { 4096 });
                for (i, chunk) in bytes[..limit].chunks(16).enumerate() {
                    let off = i * 16;
                    hex.push_str(&format!("{:08x}  ", off));
                    for b in chunk { hex.push_str(&format!("{:02x} ", b)); }
                    for _ in chunk.len()..16 { hex.push_str("   "); }
                    hex.push_str(" ");
                    for b in chunk {
                        hex.push(if (0x20..=0x7e).contains(b) { *b as char } else { '.' });
                    }
                    hex.push('\n');
                }
                std::fs::write(&hex_path, &hex).unwrap();
                println!("[{}] {} -> {} ({} bytes)", extracted, f.name, bin_path, bytes.len());
                extracted += 1;
                if extracted >= SAMPLE_LIMIT {
                    break 'outer;
                }
            }
        }
    }
    println!("\nExtracted {} samples to {}", extracted, OUT_DIR);
}
