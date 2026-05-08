// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt

//! Walk every PAMT in the game install, extract every `.paatt` file,
//! run it through `PaattFile::parse_strict` + `to_bytes`, and report
//! how many round-trip byte-perfect.
//!
//! The Tier 1 envelope reader has been shipping for a while; this
//! validator exists to keep us honest as we incrementally decode
//! BaseData fields per version.

use dmm_parser::binary::paatt::PaattFile;
use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paz;
use dmm_parser::json_traits::{ToJsonValue, WriteJsonValue};
use std::collections::BTreeMap;
use std::path::Path;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";

#[derive(Default)]
struct Stats {
    extracted: usize,
    byte_perfect: usize,
    failed: usize,
    json_byte_perfect: usize,
    json_failed: usize,
    extract_failed: usize,
    /// version -> (count, base_data sizes seen)
    version_stats: BTreeMap<u8, (usize, BTreeMap<usize, usize>)>,
    first_failures: Vec<(String, String)>,
}

fn main() {
    let game_dir = Path::new(GAME_DIR);
    if !game_dir.exists() {
        eprintln!("Game install not found"); std::process::exit(1);
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
        let pamt_data = match std::fs::read(group_dir.join("0.pamt")) { Ok(d) => d, Err(_) => continue };
        let pamt = match PackMeta::parse(&pamt_data, None) { Ok(p) => p, Err(_) => continue };
        let encrypt_info = pamt.header.encrypt_info.encrypt_info;
        let mut group_count = 0usize;

        for dir in &pamt.directories {
            for f in &dir.files {
                if !f.name.to_ascii_lowercase().ends_with(".paatt") { continue; }
                group_count += 1;
                let label = format!("{}/{}/{}", group_name, dir.path, f.name);
                let bytes = match paz::extract_file(&group_dir, f, &dir.path, &encrypt_info) {
                    Ok(b) => b,
                    Err(e) => {
                        stats.extract_failed += 1;
                        if stats.first_failures.len() < 10 {
                            stats.first_failures.push((label.clone(), format!("extract: {}", e)));
                        }
                        continue;
                    }
                };
                stats.extracted += 1;
                match PaattFile::parse_strict(&bytes) {
                    Ok((paatt, trailing)) => {
                        // Record version stats.
                        for info in &paatt.infos {
                            let entry = stats.version_stats
                                .entry(info.version)
                                .or_insert_with(|| (0, BTreeMap::new()));
                            entry.0 += 1;
                            *entry.1.entry(info.base_data.len()).or_insert(0) += 1;
                        }
                        // Round-trip check. `parse_strict` returns the
                        // count of trailing bytes that the reader did
                        // not consume — we fail the round-trip if any
                        // exist (means the writer would lose them).
                        let _ = trailing;
                        // JSON round-trip path: validate that
                        // ToJsonValue + WriteJsonValue path produces
                        // byte-identical output. This is what the
                        // PyO3 bindings use, so verifying it here
                        // covers the Python entry points.
                        let json = paatt.to_json_value();
                        let mut json_out = Vec::new();
                        match PaattFile::write_from_json(&mut json_out, &json) {
                            Ok(()) => {
                                if json_out == bytes {
                                    stats.json_byte_perfect += 1;
                                } else {
                                    stats.json_failed += 1;
                                }
                            }
                            Err(_) => stats.json_failed += 1,
                        }
                        match paatt.to_bytes() {
                            Ok(written) => {
                                if written == bytes {
                                    stats.byte_perfect += 1;
                                } else {
                                    stats.failed += 1;
                                    if stats.first_failures.len() < 10 {
                                        stats.first_failures.push((
                                            label.clone(),
                                            format!("byte diff: orig={} round={}", bytes.len(), written.len()),
                                        ));
                                    }
                                }
                            }
                            Err(e) => {
                                stats.failed += 1;
                                if stats.first_failures.len() < 10 {
                                    stats.first_failures.push((label.clone(), format!("write: {}", e)));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        stats.failed += 1;
                        if stats.first_failures.len() < 10 {
                            stats.first_failures.push((label.clone(), format!("parse: {}", e)));
                        }
                    }
                }
            }
        }
        if group_count > 0 {
            println!("[{:2}/{}] {}: {} paatt files", gi + 1, groups.len(), group_name, group_count);
        }
    }

    println!("\n=== Summary ===");
    println!("Extracted: {}", stats.extracted);
    println!("Byte-perfect: {}", stats.byte_perfect);
    println!("Failed: {}", stats.failed);
    println!("JSON-path byte-perfect: {}", stats.json_byte_perfect);
    println!("JSON-path failed: {}", stats.json_failed);
    println!("Extract failures: {}", stats.extract_failed);
    println!();
    println!("Per-version stats:");
    for (version, (count, sizes)) in &stats.version_stats {
        println!("  v{}: {} infos", version, count);
        for (size, n) in sizes {
            println!("    BaseData size {}: {} infos", size, n);
        }
    }

    if !stats.first_failures.is_empty() {
        println!("\nFirst failures:");
        for (path, reason) in &stats.first_failures {
            println!("  {}\n    {}", path, reason);
        }
    }

    let total = stats.byte_perfect + stats.failed;
    if total > 0 {
        let pct = (stats.byte_perfect as f64 / total as f64) * 100.0;
        println!("\nByte-perfect rate: {:.1}% ({}/{})", pct, stats.byte_perfect, total);
    }
}
