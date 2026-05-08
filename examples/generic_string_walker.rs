// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt

//! Validate the generic `walk_u32_prefixed_strings` function across
//! ALL Tier 1 formats. For each format, walk one sample file and
//! report how many LP-strings the generic walker discovers vs. what
//! we'd expect from the typed reader's structural fields.

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paseq::walk_u32_prefixed_strings;
use dmm_parser::binary::paz;
use std::path::Path;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";

fn first_sample(extension: &str) -> Option<(String, Vec<u8>)> {
    let game_dir = Path::new(GAME_DIR);
    let mut groups: Vec<String> = std::fs::read_dir(game_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.len() == 4 && n.chars().all(|c| c.is_ascii_digit()))
        .collect();
    groups.sort();
    for group_name in &groups {
        let group_dir = game_dir.join(group_name);
        let pamt_data = std::fs::read(group_dir.join("0.pamt")).ok()?;
        let pamt = PackMeta::parse(&pamt_data, None).ok()?;
        let encrypt_info = pamt.header.encrypt_info.encrypt_info;
        for dir in &pamt.directories {
            for f in &dir.files {
                if f.name.to_ascii_lowercase().ends_with(extension) {
                    if let Ok(bytes) = paz::extract_file(&group_dir, f, &dir.path, &encrypt_info) {
                        return Some((f.name.clone(), bytes));
                    }
                }
            }
        }
    }
    None
}

fn report(label: &str, ext: &str) {
    if let Some((name, bytes)) = first_sample(ext) {
        let strings = walk_u32_prefixed_strings(&bytes, 0);
        println!("\n=== {} sample: {} ({} bytes) ===", label, name, bytes.len());
        println!("LP-strings discovered: {}", strings.len());
        if strings.len() <= 30 {
            for (off, s) in &strings {
                let display = if s.len() > 60 {
                    format!("{}...", &s[..57])
                } else {
                    s.clone()
                };
                println!("  0x{:04x}  {:?}", off, display);
            }
        } else {
            // Show first 5 + last 5
            for (off, s) in strings.iter().take(5) {
                let display = if s.len() > 60 { format!("{}...", &s[..57]) } else { s.clone() };
                println!("  0x{:04x}  {:?}", off, display);
            }
            println!("  ... {} more ...", strings.len() - 10);
            for (off, s) in strings.iter().rev().take(5).collect::<Vec<_>>().iter().rev() {
                let display = if s.len() > 60 { format!("{}...", &s[..57]) } else { s.clone() };
                println!("  0x{:04x}  {:?}", off, display);
            }
        }
    } else {
        println!("\n=== {} sample: NOT FOUND ===", label);
    }
}

fn main() {
    println!("Generic walk_u32_prefixed_strings test across all Tier 1 formats");
    println!("============================================================");

    report(".pastage", ".pastage");
    report(".paseq", ".paseq");
    report(".paseqc", ".paseqc");
    report(".paschedule", ".paschedule");
    report(".paschedulepath", ".paschedulepath");
    report(".paatt", ".paatt");
}
