// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt

//! Cross-format Tier 1 validator. Walks every PAMT in the game install,
//! extracts every file matching one of the six promoted Tier 1 formats,
//! and validates round-trip via BOTH the typed reader AND the JSON path
//! (the path the PyO3 bindings use). Prints a unified summary.
//!
//! This is the canonical "is the Tier 1 work shipping correctly?"
//! regression check.

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paatt::PaattFile;
use dmm_parser::binary::paschedule::TypedPascheduleFile;
use dmm_parser::binary::paschedulepath::TypedPaschedulePathFile;
use dmm_parser::binary::paseq::TypedPaseqFile;
use dmm_parser::binary::paseqc::TypedPaseqcFile;
use dmm_parser::binary::pastage::TypedPastageFile;
use dmm_parser::binary::paz;
use dmm_parser::json_traits::{ToJsonValue, WriteJsonValue};
use std::path::Path;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";

#[derive(Default, Clone)]
struct FormatStats {
    extracted: usize,
    typed_byte_perfect: usize,
    json_byte_perfect: usize,
    typed_failed: usize,
    json_failed: usize,
}

impl FormatStats {
    fn print_row(&self, name: &str) {
        let typed_pct = if self.extracted > 0 {
            self.typed_byte_perfect as f64 / self.extracted as f64 * 100.0
        } else { 0.0 };
        let json_pct = if self.extracted > 0 {
            self.json_byte_perfect as f64 / self.extracted as f64 * 100.0
        } else { 0.0 };
        println!(
            "{:18} {:>8}  typed={:>6}/{:<6} ({:>5.1}%)  json={:>6}/{:<6} ({:>5.1}%)",
            name,
            self.extracted,
            self.typed_byte_perfect, self.extracted, typed_pct,
            self.json_byte_perfect, self.extracted, json_pct,
        );
    }
}

/// Common helper: validate the JSON round-trip path for any
/// `ToJsonValue + WriteJsonValue` typed reader. The direct path is
/// inlined per-format below since the borrowing readers' lifetime
/// bounds defeat a generic `parse` signature.
fn validate_json<T: ToJsonValue + WriteJsonValue>(
    typed: &T,
    bytes: &[u8],
    stats: &mut FormatStats,
) {
    let json = typed.to_json_value();
    let mut json_out = Vec::new();
    match T::write_from_json(&mut json_out, &json) {
        Ok(()) if json_out == bytes => stats.json_byte_perfect += 1,
        _ => stats.json_failed += 1,
    }
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
    println!("Cross-format Tier 1 validator");
    println!("Game: {}", GAME_DIR);
    println!("Group dirs: {}\n", groups.len());

    let mut pastage = FormatStats::default();
    let mut paseq = FormatStats::default();
    let mut paseqc = FormatStats::default();
    let mut paschedule = FormatStats::default();
    let mut paschedulepath = FormatStats::default();
    let mut paatt = FormatStats::default();

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

                if lower.ends_with(".pastage") {
                    pastage.extracted += 1;
                    match TypedPastageFile::parse(&bytes) {
                        Ok(t) => {
                            match t.to_bytes() {
                                Ok(out) if out == bytes => pastage.typed_byte_perfect += 1,
                                _ => pastage.typed_failed += 1,
                            }
                            validate_json(&t, &bytes, &mut pastage);
                        }
                        Err(_) => { pastage.typed_failed += 1; pastage.json_failed += 1; }
                    }
                } else if lower.ends_with(".paseq") {
                    paseq.extracted += 1;
                    match TypedPaseqFile::parse(&bytes) {
                        Ok(t) => {
                            match t.to_bytes() {
                                Ok(out) if out == bytes => paseq.typed_byte_perfect += 1,
                                _ => paseq.typed_failed += 1,
                            }
                            validate_json(&t, &bytes, &mut paseq);
                        }
                        Err(_) => { paseq.typed_failed += 1; paseq.json_failed += 1; }
                    }
                } else if lower.ends_with(".paseqc") {
                    paseqc.extracted += 1;
                    match TypedPaseqcFile::parse(&bytes) {
                        Ok(t) => {
                            match t.to_bytes() {
                                Ok(out) if out == bytes => paseqc.typed_byte_perfect += 1,
                                _ => paseqc.typed_failed += 1,
                            }
                            validate_json(&t, &bytes, &mut paseqc);
                        }
                        Err(_) => { paseqc.typed_failed += 1; paseqc.json_failed += 1; }
                    }
                } else if lower.ends_with(".paschedulepath") {
                    paschedulepath.extracted += 1;
                    match TypedPaschedulePathFile::parse(&bytes) {
                        Ok(t) => {
                            match t.to_bytes() {
                                Ok(out) if out == bytes => paschedulepath.typed_byte_perfect += 1,
                                _ => paschedulepath.typed_failed += 1,
                            }
                            validate_json(&t, &bytes, &mut paschedulepath);
                        }
                        Err(_) => { paschedulepath.typed_failed += 1; paschedulepath.json_failed += 1; }
                    }
                } else if lower.ends_with(".paschedule") {
                    paschedule.extracted += 1;
                    match TypedPascheduleFile::parse(&bytes) {
                        Ok(t) => {
                            match t.to_bytes() {
                                Ok(out) if out == bytes => paschedule.typed_byte_perfect += 1,
                                _ => paschedule.typed_failed += 1,
                            }
                            validate_json(&t, &bytes, &mut paschedule);
                        }
                        Err(_) => { paschedule.typed_failed += 1; paschedule.json_failed += 1; }
                    }
                } else if lower.ends_with(".paatt") {
                    paatt.extracted += 1;
                    match PaattFile::parse(&bytes) {
                        Ok(t) => {
                            match t.to_bytes() {
                                Ok(out) if out == bytes => paatt.typed_byte_perfect += 1,
                                _ => paatt.typed_failed += 1,
                            }
                            validate_json(&t, &bytes, &mut paatt);
                        }
                        Err(_) => { paatt.typed_failed += 1; paatt.json_failed += 1; }
                    }
                }
            }
        }
    }

    println!("=== Tier 1 Cross-Format Round-Trip Summary ===\n");
    println!("{:18} {:>8}  {:>22}  {:>22}",
        "format", "samples", "direct round-trip", "JSON path (PyO3)");
    println!("{}", "-".repeat(80));
    pastage.print_row(".pastage");
    paseq.print_row(".paseq");
    paseqc.print_row(".paseqc");
    paschedule.print_row(".paschedule");
    paschedulepath.print_row(".paschedulepath");
    paatt.print_row(".paatt");

    let total_extracted = pastage.extracted + paseq.extracted + paseqc.extracted
        + paschedule.extracted + paschedulepath.extracted + paatt.extracted;
    let total_typed_pass = pastage.typed_byte_perfect + paseq.typed_byte_perfect
        + paseqc.typed_byte_perfect + paschedule.typed_byte_perfect
        + paschedulepath.typed_byte_perfect + paatt.typed_byte_perfect;
    let total_json_pass = pastage.json_byte_perfect + paseq.json_byte_perfect
        + paseqc.json_byte_perfect + paschedule.json_byte_perfect
        + paschedulepath.json_byte_perfect + paatt.json_byte_perfect;
    let total_typed_fail = pastage.typed_failed + paseq.typed_failed + paseqc.typed_failed
        + paschedule.typed_failed + paschedulepath.typed_failed + paatt.typed_failed;
    let total_json_fail = pastage.json_failed + paseq.json_failed + paseqc.json_failed
        + paschedule.json_failed + paschedulepath.json_failed + paatt.json_failed;

    println!("{}", "-".repeat(80));
    println!("\nTOTALS");
    println!("  Files validated:        {}", total_extracted);
    println!("  Direct round-trip pass: {} ({:.2}%)",
        total_typed_pass,
        if total_extracted > 0 { total_typed_pass as f64 / total_extracted as f64 * 100.0 } else { 0.0 });
    println!("  JSON path pass:         {} ({:.2}%)",
        total_json_pass,
        if total_extracted > 0 { total_json_pass as f64 / total_extracted as f64 * 100.0 } else { 0.0 });

    let any_fail = total_typed_fail > 0 || total_json_fail > 0;
    if any_fail {
        eprintln!("\nFATAL: {} direct + {} JSON failures detected",
            total_typed_fail, total_json_fail);
        std::process::exit(2);
    } else {
        println!("\nAll Tier 1 formats round-trip byte-perfect across both paths.");
    }
}
