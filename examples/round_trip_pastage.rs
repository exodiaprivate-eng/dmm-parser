// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Walk every vanilla group, decompress every .pastage file, run the
//! Tier 1.5 tokenizer parse + serialize round-trip. Report PASS/FAIL
//! totals, the distribution of token counts and string ratios, and the
//! first N failures with byte deltas so we can iterate the parser.

use dmm_parser::binary::pamt::{Compression, PackMeta};
use dmm_parser::binary::pastage::{PastageFile, Token};
use lz4_flex::block::decompress;
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

    let mut total = 0usize;
    let mut pass = 0usize;
    let mut fail = 0usize;
    let mut failures_reported = 0usize;

    // Token-count + string-ratio histograms
    let mut total_tokens: u64 = 0;
    let mut total_lp_strings: u64 = 0;
    let mut total_raw_bytes: u64 = 0;
    let mut largest_file = (0usize, String::new());
    let mut smallest_file = (usize::MAX, String::new());

    // Roundtrip JSON spot-check (first 50 files)
    let mut json_total = 0usize;
    let mut json_pass = 0usize;

    for g in &groups {
        let pamt_p = game.join(g).join("0.pamt");
        let paz_p = game.join(g).join("0.paz");
        if !pamt_p.exists() || !paz_p.exists() { continue; }
        let Ok(pamt_data) = std::fs::read(&pamt_p) else { continue };
        let Ok(meta) = PackMeta::parse(&pamt_data, None) else { continue };
        let Ok(paz_data) = std::fs::read(&paz_p) else { continue };

        for d in &meta.directories {
            for f in &d.files {
                if !f.name.to_lowercase().ends_with(".pastage") { continue; }
                let off = f.file.chunk_offset as usize;
                let comp = f.file.compressed_size as usize;
                if off + comp > paz_data.len() { continue; }
                let comp_bytes = &paz_data[off..off + comp];
                let body = match f.file.compression {
                    Compression::Lz4 => match decompress(comp_bytes, f.file.uncompressed_size as usize) {
                        Ok(b) => b,
                        Err(e) => {
                            println!("decomp failed for {}/{}: {:?}", d.path, f.name, e);
                            continue;
                        }
                    },
                    Compression::None => comp_bytes.to_vec(),
                    other => {
                        println!("unsupported compression {:?} for {}/{}", other, d.path, f.name);
                        continue;
                    }
                };

                total += 1;
                if body.len() > largest_file.0 {
                    largest_file = (body.len(), format!("{} {}/{}", g, d.path, f.name));
                }
                if body.len() < smallest_file.0 {
                    smallest_file = (body.len(), format!("{} {}/{}", g, d.path, f.name));
                }

                let parsed = match PastageFile::parse(&body) {
                    Ok(p) => p,
                    Err(e) => {
                        fail += 1;
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
                        fail += 1;
                        if failures_reported < MAX_FAILURE_REPORTS {
                            println!("WRITE-ERR {}/{}: {}", d.path, f.name, e);
                            failures_reported += 1;
                        }
                        continue;
                    }
                };

                if written == body {
                    pass += 1;
                    total_tokens += parsed.tokens.len() as u64;
                    let mut lp = 0u64;
                    let mut raw = 0u64;
                    for t in &parsed.tokens {
                        match t {
                            Token::LpString(b) => lp += b.len() as u64,
                            Token::RawBytes(b) => raw += b.len() as u64,
                        }
                    }
                    total_lp_strings += lp;
                    total_raw_bytes += raw;

                    // JSON spot-check on the first 50 files
                    if json_total < 50 {
                        json_total += 1;
                        let json = parsed.to_json();
                        match PastageFile::from_json(&json) {
                            Ok(rebuilt) => match rebuilt.to_bytes() {
                                Ok(b) if b == body => json_pass += 1,
                                Ok(_) => println!("JSON-MISMATCH (bytes differ): {}/{}", d.path, f.name),
                                Err(e) => println!("JSON-WRITE-ERR {}/{}: {}", d.path, f.name, e),
                            },
                            Err(e) => println!("JSON-PARSE-ERR {}/{}: {}", d.path, f.name, e),
                        }
                    }
                } else {
                    fail += 1;
                    if failures_reported < MAX_FAILURE_REPORTS {
                        let first_diff = (0..body.len().min(written.len()))
                            .find(|&i| body[i] != written[i])
                            .unwrap_or_else(|| body.len().min(written.len()));
                        println!("MISMATCH {} {}/{}: in_size={} out_size={} first_diff=0x{:X}",
                            g, d.path, f.name, body.len(), written.len(), first_diff);
                        let win_start = first_diff.saturating_sub(8);
                        let win_end = (first_diff + 16).min(body.len()).min(written.len());
                        print!("  in : ");
                        for i in win_start..win_end {
                            print!("{:02X} ", body[i]);
                        }
                        println!();
                        print!("  out: ");
                        for i in win_start..win_end {
                            print!("{:02X} ", written[i]);
                        }
                        println!();
                        failures_reported += 1;
                    }
                }
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Total .pastage files: {}", total);
    println!("Round-trip PASS:      {} ({:.2}%)", pass, 100.0 * pass as f64 / total.max(1) as f64);
    println!("Round-trip FAIL:      {}", fail);
    if pass > 0 {
        let avg_tokens = total_tokens as f64 / pass as f64;
        let total_body = (total_lp_strings + total_raw_bytes) as f64;
        let lp_pct = 100.0 * total_lp_strings as f64 / total_body.max(1.0);
        println!("Avg tokens / file:    {:.1}", avg_tokens);
        println!("LpString body bytes:  {} ({:.1}% of body)", total_lp_strings, lp_pct);
        println!("RawBytes body bytes:  {}", total_raw_bytes);
    }
    println!("Largest file:         {} bytes ({})", largest_file.0, largest_file.1);
    if smallest_file.0 != usize::MAX {
        println!("Smallest file:        {} bytes ({})", smallest_file.0, smallest_file.1);
    }
    println!("\nJSON round-trip (first 50 successful parses): {}/{}", json_pass, json_total);
}
