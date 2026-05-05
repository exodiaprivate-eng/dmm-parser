// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Bulk WEM round-trip: parse + serialize every vanilla .wem byte-exact.
//! Reads raw .paz bytes (per the survey, all .wem are stored
//! uncompressed and unencrypted).

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::wem::WemFile;
use std::path::Path;

const MAX_FAILURE_REPORTS: usize = 5;

fn main() {
    let game = Path::new(r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert");
    let mut groups: Vec<String> = std::fs::read_dir(game).unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.len() == 4 && n.chars().all(|c| c.is_ascii_digit()))
        .collect();
    groups.sort();

    let mut total: usize = 0;
    let mut pass: usize = 0;
    let mut fail_parse: usize = 0;
    let mut fail_roundtrip: usize = 0;
    let mut failures_reported: usize = 0;
    let mut total_chunks: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut largest = (0usize, String::new());

    let mut json_total: usize = 0;
    let mut json_pass: usize = 0;

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

                let cid = f.file.chunk_id;
                let paz = paz_cache.entry(cid).or_insert_with(|| {
                    std::fs::read(group_dir.join(format!("{}.paz", cid))).unwrap_or_default()
                });
                if paz.is_empty() { continue; }
                let off = f.file.chunk_offset as usize;
                let csz = f.file.compressed_size as usize;
                if off + csz > paz.len() { continue; }
                let stored = &paz[off..off + csz];

                if stored.len() > largest.0 {
                    largest = (stored.len(), format!("{} {}/{}", g, d.path, f.name));
                }

                let parsed = match WemFile::parse(stored) {
                    Ok(p) => p,
                    Err(e) => {
                        fail_parse += 1;
                        if failures_reported < MAX_FAILURE_REPORTS {
                            println!("PARSE-ERR {}/{}: {}", d.path, f.name, e);
                            failures_reported += 1;
                        }
                        continue;
                    }
                };

                let written = match parsed.to_bytes() {
                    Ok(b) => b,
                    Err(e) => {
                        fail_roundtrip += 1;
                        if failures_reported < MAX_FAILURE_REPORTS {
                            println!("WRITE-ERR {}/{}: {}", d.path, f.name, e);
                            failures_reported += 1;
                        }
                        continue;
                    }
                };

                if written == stored {
                    pass += 1;
                    total_chunks += parsed.chunks.len() as u64;
                    total_bytes += stored.len() as u64;
                    if json_total < 50 {
                        json_total += 1;
                        let json = parsed.to_json();
                        if let Ok(rebuilt) = WemFile::from_json(&json) {
                            if let Ok(rb) = rebuilt.to_bytes() {
                                if rb == stored { json_pass += 1; }
                            }
                        }
                    }
                } else {
                    fail_roundtrip += 1;
                    if failures_reported < MAX_FAILURE_REPORTS {
                        let first_diff = (0..stored.len().min(written.len()))
                            .find(|&i| stored[i] != written[i])
                            .unwrap_or_else(|| stored.len().min(written.len()));
                        println!("MISMATCH {}/{}: in={} out={} first_diff=0x{:X}",
                            d.path, f.name, stored.len(), written.len(), first_diff);
                        failures_reported += 1;
                    }
                }
            }
        }
    }

    println!("\n=== WEM round-trip summary ===");
    println!("Total .wem files:    {}", total);
    println!("Round-trip PASS:     {} ({:.2}%)", pass, 100.0 * pass as f64 / total.max(1) as f64);
    println!("Parse failed:        {}", fail_parse);
    println!("Round-trip failed:   {}", fail_roundtrip);
    if pass > 0 {
        println!("Avg chunks / file:   {:.1}", total_chunks as f64 / pass as f64);
        println!("Total bytes:         {} ({:.2} GB)", total_bytes, total_bytes as f64 / (1024.0 * 1024.0 * 1024.0));
    }
    println!("Largest:             {} bytes  ({})", largest.0, largest.1);
    println!("\nJSON round-trip (first 50 successful parses): {}/{}", json_pass, json_total);
}
