// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt

//! Walk a sample of `.paseq` and `.paseqc` files in the game install,
//! parse their outer class field directory, and report:
//!
//!   - Total samples successfully field-parsed
//!   - Distinct (field_name, type_name) pairs encountered, with
//!     counts. Vanilla data has consistent schemas across all files
//!     of a format, so the output should show one stable schema per
//!     format.
//!
//! This validates the new `parse_outer_fields()` accessor and gives
//! mod authors a printable reference of the .paseq / .paseqc field
//! lists.

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paseq::TypedPaseqFile;
use dmm_parser::binary::paseqc::TypedPaseqcFile;
use dmm_parser::binary::paz;
use std::collections::BTreeMap;
use std::path::Path;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";

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

    // (field_name, type_name) -> count
    let mut paseq_fields: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut paseqc_fields: BTreeMap<(String, String), usize> = BTreeMap::new();
    let mut paseq_count = 0usize;
    let mut paseqc_count = 0usize;
    let mut paseq_failed = 0usize;
    let mut paseqc_failed = 0usize;

    for group_name in &groups {
        let group_dir = game_dir.join(group_name);
        let pamt_data = match std::fs::read(group_dir.join("0.pamt")) { Ok(d) => d, Err(_) => continue };
        let pamt = match PackMeta::parse(&pamt_data, None) { Ok(p) => p, Err(_) => continue };
        let encrypt_info = pamt.header.encrypt_info.encrypt_info;
        for dir in &pamt.directories {
            for f in &dir.files {
                let lower = f.name.to_ascii_lowercase();
                let bytes = match paz::extract_file(&group_dir, f, &dir.path, &encrypt_info) {
                    Ok(b) => b, Err(_) => continue,
                };
                if lower.ends_with(".paseq") {
                    paseq_count += 1;
                    match TypedPaseqFile::parse(&bytes).and_then(|t| t.outer_fields()) {
                        Ok(fields) => {
                            for f in fields {
                                *paseq_fields.entry((f.field_name, f.type_name)).or_insert(0) += 1;
                            }
                        }
                        Err(_) => paseq_failed += 1,
                    }
                } else if lower.ends_with(".paseqc") {
                    paseqc_count += 1;
                    match TypedPaseqcFile::parse(&bytes).and_then(|t| t.outer_fields()) {
                        Ok(fields) => {
                            for f in fields {
                                *paseqc_fields.entry((f.field_name, f.type_name)).or_insert(0) += 1;
                            }
                        }
                        Err(_) => paseqc_failed += 1,
                    }
                }
            }
        }
    }

    println!("=== .paseq outer field directory ===");
    println!("Files: {} (failed: {})", paseq_count, paseq_failed);
    println!("Distinct (field_name, type_name) pairs: {}\n", paseq_fields.len());
    for ((name, ty), count) in &paseq_fields {
        println!("  {:>5}  {:32}  {}", count, name, ty);
    }

    println!("\n=== .paseqc outer field directory ===");
    println!("Files: {} (failed: {})", paseqc_count, paseqc_failed);
    println!("Distinct (field_name, type_name) pairs: {}\n", paseqc_fields.len());
    for ((name, ty), count) in &paseqc_fields {
        println!("  {:>5}  {:32}  {}", count, name, ty);
    }
}
