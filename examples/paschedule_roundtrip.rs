// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt

//! Walk every PAMT in the game install, extract every `.paschedule`
//! file (excluding `.paschedulepath`), and validate round-trip via
//! `TypedPascheduleFile`.

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paschedule::{PascheduleFileSafe, TypedPascheduleFile};
use dmm_parser::binary::paz;
use std::path::Path;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";

#[derive(Default)]
struct Stats {
    extracted: usize,
    typed_byte_perfect: usize,
    typed_failed: usize,
    safe_decoded_byte_perfect: usize,
    safe_raw_byte_perfect: usize,
    safe_byte_diff: usize,
    extract_failed: usize,
    first_typed_failures: Vec<(String, String)>,
}

fn main() {
    let game_dir = Path::new(GAME_DIR);
    if !game_dir.exists() {
        eprintln!("Game install not found at {}", GAME_DIR);
        std::process::exit(1);
    }
    let mut groups: Vec<String> = std::fs::read_dir(game_dir)
        .expect("read game dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.len() == 4 && n.chars().all(|c| c.is_ascii_digit()))
        .collect();
    groups.sort();
    println!("Found {} group directories", groups.len());

    let mut stats = Stats::default();
    for (gi, group_name) in groups.iter().enumerate() {
        let group_dir = game_dir.join(group_name);
        let pamt_data = match std::fs::read(group_dir.join("0.pamt")) {
            Ok(d) => d, Err(_) => continue,
        };
        let pamt = match PackMeta::parse(&pamt_data, None) { Ok(p) => p, Err(_) => continue };
        let encrypt_info = pamt.header.encrypt_info.encrypt_info;
        let mut group_count = 0usize;

        for dir in &pamt.directories {
            for f in &dir.files {
                let lower = f.name.to_ascii_lowercase();
                // Exclude .paschedulepath (separate format).
                if !lower.ends_with(".paschedule") || lower.ends_with(".paschedulepath") {
                    continue;
                }
                group_count += 1;
                let label = format!("{}/{}/{}", group_name, dir.path, f.name);
                let bytes = match paz::extract_file(&group_dir, f, &dir.path, &encrypt_info) {
                    Ok(b) => b,
                    Err(e) => {
                        stats.extract_failed += 1;
                        if stats.first_typed_failures.len() < 10 {
                            stats.first_typed_failures.push((label.clone(), format!("extract: {}", e)));
                        }
                        continue;
                    }
                };
                stats.extracted += 1;

                match TypedPascheduleFile::parse(&bytes) {
                    Ok(typed) => match typed.to_bytes() {
                        Ok(written) => {
                            if written == bytes {
                                stats.typed_byte_perfect += 1;
                            } else {
                                stats.typed_failed += 1;
                                if stats.first_typed_failures.len() < 10 {
                                    stats.first_typed_failures.push((
                                        label.clone(),
                                        format!("byte diff: orig={} round={}", bytes.len(), written.len()),
                                    ));
                                }
                            }
                        }
                        Err(e) => {
                            stats.typed_failed += 1;
                            if stats.first_typed_failures.len() < 10 {
                                stats.first_typed_failures.push((label.clone(), format!("write: {}", e)));
                            }
                        }
                    },
                    Err(e) => {
                        stats.typed_failed += 1;
                        if stats.first_typed_failures.len() < 10 {
                            stats.first_typed_failures.push((label.clone(), format!("parse: {}", e)));
                        }
                    }
                }

                match PascheduleFileSafe::parse(&bytes) {
                    Ok(safe) => match safe.to_bytes() {
                        Ok(rt) => if rt == bytes {
                            match safe {
                                PascheduleFileSafe::Decoded(_) => stats.safe_decoded_byte_perfect += 1,
                                PascheduleFileSafe::Raw { .. } => stats.safe_raw_byte_perfect += 1,
                            }
                        } else { stats.safe_byte_diff += 1; },
                        Err(_) => stats.safe_byte_diff += 1,
                    },
                    Err(_) => stats.safe_byte_diff += 1,
                }
            }
        }
        if group_count > 0 {
            println!("[{:2}/{}] {}: {} paschedule files", gi + 1, groups.len(), group_name, group_count);
        }
    }

    println!("\n=== Summary ===");
    println!("Extracted: {}", stats.extracted);
    println!();
    println!("Typed (Tier 1) reader:");
    println!("  Byte-perfect: {}", stats.typed_byte_perfect);
    println!("  Failed:       {}", stats.typed_failed);
    println!();
    println!("Safe wrapper:");
    println!("  Decoded byte-perfect: {}", stats.safe_decoded_byte_perfect);
    println!("  Raw byte-perfect:     {}", stats.safe_raw_byte_perfect);
    println!("  Mismatch (FATAL):     {}", stats.safe_byte_diff);
    println!();
    println!("Extract failures: {}", stats.extract_failed);

    if !stats.first_typed_failures.is_empty() {
        println!("\nFirst typed failures:");
        for (path, reason) in &stats.first_typed_failures {
            println!("  {}\n    {}", path, reason);
        }
    }

    let total = stats.typed_byte_perfect + stats.typed_failed;
    if total > 0 {
        let pct = (stats.typed_byte_perfect as f64 / total as f64) * 100.0;
        println!("\nTier 1 byte-perfect rate: {:.1}% ({}/{})",
            pct, stats.typed_byte_perfect, total);
    }
    if stats.safe_byte_diff > 0 {
        eprintln!("\nFATAL: safe wrapper failed on {} samples", stats.safe_byte_diff);
        std::process::exit(2);
    }
}
