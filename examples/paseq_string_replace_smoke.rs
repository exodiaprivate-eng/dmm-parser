// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt

//! Smoke test for `replace_cstring_at`: parse a .paseq, find the
//! `_sequencerName` value (or first value-section string), replace
//! it, re-parse, verify the new value sticks. Runs end-to-end on
//! a single sample to validate the edit primitive.

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paseq::{TypedPaseqFile, replace_cstring_at};
use dmm_parser::binary::paz;
use std::path::Path;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";
const TARGET: &str = "cd_seq_ui_empty.paseq";

fn main() {
    let game_dir = Path::new(GAME_DIR);
    let mut groups: Vec<String> = std::fs::read_dir(game_dir)
        .expect("read game dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.len() == 4 && n.chars().all(|c| c.is_ascii_digit()))
        .collect();
    groups.sort();

    'outer: for group_name in &groups {
        let group_dir = game_dir.join(group_name);
        let pamt_data = match std::fs::read(group_dir.join("0.pamt")) { Ok(d) => d, Err(_) => continue };
        let pamt = match PackMeta::parse(&pamt_data, None) { Ok(p) => p, Err(_) => continue };
        let encrypt_info = pamt.header.encrypt_info.encrypt_info;
        for dir in &pamt.directories {
            for f in &dir.files {
                if f.name != TARGET { continue; }
                let bytes = paz::extract_file(&group_dir, f, &dir.path, &encrypt_info).unwrap();
                println!("Original file: {} ({} bytes)", TARGET, bytes.len());

                let typed = TypedPaseqFile::parse(&bytes).unwrap();
                let strings = typed.value_section_strings().unwrap();
                println!("\nValue-section strings found: {}", strings.len());
                for (off, s) in &strings {
                    println!("  offset 0x{:04x}  {:?}", off, s);
                }

                if strings.is_empty() {
                    println!("No strings to replace; nothing to test.");
                    return;
                }

                // Replace the first string with a longer value.
                let (offset, original) = strings[0].clone();
                let new_value = format!("{} (modded by Session 20)", original);
                println!("\nReplacing {:?} at offset 0x{:x} with {:?}",
                    original, offset, new_value);

                let modified = replace_cstring_at(
                    &bytes, offset, Some(&original), &new_value,
                ).unwrap();
                println!("Modified file: {} bytes (delta: {:+})",
                    modified.len(),
                    modified.len() as i64 - bytes.len() as i64);

                // Re-parse and verify.
                let typed2 = TypedPaseqFile::parse(&modified)
                    .expect("modified file must still parse");
                let strings2 = typed2.value_section_strings().unwrap();
                println!("\nRe-parsed: found {} strings", strings2.len());

                let found_new = strings2.iter().any(|(_, s)| s == &new_value);
                let found_old = strings2.iter().any(|(_, s)| s == &original);
                if found_new && !found_old {
                    println!("✓ Replacement succeeded — new value present, old gone");
                } else {
                    println!("✗ Replacement FAILED");
                    println!("  found_new: {}", found_new);
                    println!("  found_old: {}", found_old);
                    std::process::exit(2);
                }

                // Round-trip: new file should also work via parse → to_bytes
                let rt = typed2.to_bytes().unwrap();
                if rt == modified {
                    println!("✓ Modified file round-trips through parse → to_bytes");
                } else {
                    println!("✗ Round-trip mismatch on modified file");
                    std::process::exit(2);
                }
                break 'outer;
            }
        }
    }
}
