// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Walk every PAMT in the game install, extract every `.pastage` file,
//! run it through `TypedPastageFile::parse` + `to_bytes`, and report
//! how many round-trip byte-perfect.
//!
//! This validates the Tier 1.5 → Tier 1 promotion hypothesis: that
//! `.pastage` = `[CString stage_path][SequencerStageChartDescPartial body]`.
//!
//! Usage:
//!   cargo run --release --example pastage_roundtrip
//!
//! Reads from: C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert
//! Writes to:  (none — purely a validation pass)
//!
//! Output: counts and the first 10 failures with reason. On success
//! every sample round-trips byte-perfect; the loop graduates `.pastage`
//! to Tier 1 in the dispatcher and ships.

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::pastage::{PastageFileSafe, TypedPastageFile};
use dmm_parser::binary::paz;
use std::path::Path;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";

#[derive(Default)]
struct Stats {
    extracted: usize,
    typed_decoded_byte_perfect: usize,
    typed_decoded_byte_diff: usize,
    typed_failed: usize,
    safe_decoded_byte_perfect: usize,
    safe_raw_byte_perfect: usize,
    safe_byte_diff: usize,
    extract_failed: usize,
    first_typed_failures: Vec<(String, String)>,
    first_byte_diffs: Vec<(String, usize, usize)>,
}

fn main() {
    let game_dir = Path::new(GAME_DIR);
    if !game_dir.exists() {
        eprintln!("Game install not found at {}", GAME_DIR);
        eprintln!("Edit GAME_DIR at the top of examples/pastage_roundtrip.rs");
        std::process::exit(1);
    }

    // Walk groups directly from the filesystem instead of via PAPGT.
    // PAPGT checksum can fail when DMM has mounted overlays; PAMT
    // parsing with `None` skips its own checksum which is what we want.
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
        let pamt_path = group_dir.join("0.pamt");

        let pamt_data = match std::fs::read(&pamt_path) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let pamt = match PackMeta::parse(&pamt_data, None) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[{:2}/{}] {}: PAMT parse failed: {}",
                    gi + 1, groups.len(), group_name, e);
                continue;
            }
        };

        let encrypt_info = pamt.header.encrypt_info.encrypt_info;
        let mut group_pastage_count = 0usize;

        for dir in &pamt.directories {
            for f in &dir.files {
                if !f.name.to_ascii_lowercase().ends_with(".pastage") {
                    continue;
                }
                group_pastage_count += 1;
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

                // ── Tier 1: typed reader ──
                match TypedPastageFile::parse(&bytes) {
                    Ok(typed) => {
                        match typed.to_bytes() {
                            Ok(written) => {
                                if written == bytes {
                                    stats.typed_decoded_byte_perfect += 1;
                                } else {
                                    stats.typed_decoded_byte_diff += 1;
                                    if stats.first_byte_diffs.len() < 10 {
                                        stats.first_byte_diffs.push((
                                            label.clone(),
                                            bytes.len(),
                                            written.len(),
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
                        }
                    }
                    Err(e) => {
                        stats.typed_failed += 1;
                        if stats.first_typed_failures.len() < 10 {
                            stats.first_typed_failures.push((label.clone(), format!("parse: {}", e)));
                        }
                    }
                }

                // ── Safe wrapper (must always round-trip) ──
                match PastageFileSafe::parse(&bytes) {
                    Ok(safe) => match safe.to_bytes() {
                        Ok(rt) => {
                            if rt == bytes {
                                match safe {
                                    PastageFileSafe::Decoded(_) => stats.safe_decoded_byte_perfect += 1,
                                    PastageFileSafe::Raw { .. } => stats.safe_raw_byte_perfect += 1,
                                }
                            } else {
                                stats.safe_byte_diff += 1;
                            }
                        }
                        Err(_) => stats.safe_byte_diff += 1,
                    },
                    Err(_) => stats.safe_byte_diff += 1,
                }
            }
        }
        if group_pastage_count > 0 {
            println!(
                "[{:2}/{}] {}: {} pastage files",
                gi + 1, groups.len(), group_name, group_pastage_count,
            );
        }
    }

    println!("\n=== Summary ===");
    println!("Extracted (and decoded): {}", stats.extracted);
    println!();
    println!("Typed (Tier 1) reader:");
    println!("  Byte-perfect round-trip:  {}", stats.typed_decoded_byte_perfect);
    println!("  Decoded but bytes differ: {}", stats.typed_decoded_byte_diff);
    println!("  Parse/write failed:       {}", stats.typed_failed);
    println!();
    println!("Safe wrapper (must always round-trip):");
    println!("  Decoded arm, byte-perfect: {}", stats.safe_decoded_byte_perfect);
    println!("  Raw arm, byte-perfect:     {}", stats.safe_raw_byte_perfect);
    println!("  Byte mismatch (FATAL):     {}", stats.safe_byte_diff);
    println!();
    println!("Extract failures: {}", stats.extract_failed);

    if !stats.first_typed_failures.is_empty() {
        println!("\nFirst typed failures (up to 10):");
        for (path, reason) in &stats.first_typed_failures {
            println!("  {}", path);
            println!("    {}", reason);
        }
    }

    if !stats.first_byte_diffs.is_empty() {
        println!("\nFirst byte mismatches (up to 10):");
        for (path, orig_len, rt_len) in &stats.first_byte_diffs {
            println!("  {}: orig={} bytes, round-trip={} bytes", path, orig_len, rt_len);
        }
    }

    let total = stats.typed_decoded_byte_perfect
        + stats.typed_decoded_byte_diff
        + stats.typed_failed;
    if total > 0 {
        let pct = (stats.typed_decoded_byte_perfect as f64 / total as f64) * 100.0;
        println!("\nTier 1 (typed) byte-perfect rate: {:.1}% ({}/{})",
            pct, stats.typed_decoded_byte_perfect, total);
    }

    let safe_total = stats.safe_decoded_byte_perfect
        + stats.safe_raw_byte_perfect
        + stats.safe_byte_diff;
    if safe_total > 0 {
        let pct = ((stats.safe_decoded_byte_perfect + stats.safe_raw_byte_perfect) as f64
            / safe_total as f64) * 100.0;
        println!("Safe wrapper byte-perfect rate: {:.1}% ({}/{})",
            pct, stats.safe_decoded_byte_perfect + stats.safe_raw_byte_perfect, safe_total);
    }

    if stats.safe_byte_diff > 0 {
        eprintln!("\nFATAL: safe wrapper failed byte-perfect on {} samples — investigate",
            stats.safe_byte_diff);
        std::process::exit(2);
    }
}
