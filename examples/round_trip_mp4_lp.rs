// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! MP4 round-trip via lp_token_stream. Cutscene videos are best
//! handled as drop-in replacements; tokenizer-level round-trip is
//! enough to ship file replacement reliably.

use dmm_parser::binary::lp_token_stream::{LpTokenFile, Token};
use dmm_parser::binary::pamt::PackMeta;
use std::path::Path;

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
    let mut fail: usize = 0;
    let mut total_lp: u64 = 0;
    let mut total_raw: u64 = 0;
    let mut total_bytes: u64 = 0;

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
                if ext != "mp4" { continue; }
                let cid = f.file.chunk_id;
                let paz = paz_cache.entry(cid).or_insert_with(|| {
                    std::fs::read(group_dir.join(format!("{}.paz", cid))).unwrap_or_default()
                });
                if paz.is_empty() { continue; }
                let off = f.file.chunk_offset as usize;
                let csz = f.file.compressed_size as usize;
                if off + csz > paz.len() { continue; }
                let stored = &paz[off..off + csz];
                total += 1;
                total_bytes += stored.len() as u64;

                let parsed = match LpTokenFile::parse(stored) { Ok(p) => p, Err(_) => { fail += 1; continue; } };
                let written = match parsed.to_bytes() { Ok(b) => b, Err(_) => { fail += 1; continue; } };
                if written == stored {
                    pass += 1;
                    for t in &parsed.tokens {
                        match t {
                            Token::LpString(b) => total_lp += b.len() as u64,
                            Token::RawBytes(b) => total_raw += b.len() as u64,
                        }
                    }
                } else {
                    fail += 1;
                }
            }
        }
    }

    println!("=== MP4 Tier 1.5 round-trip ===");
    println!("Total .mp4 files:  {}", total);
    println!("Round-trip PASS:   {} ({:.2}%)", pass, 100.0 * pass as f64 / total.max(1) as f64);
    println!("Round-trip FAIL:   {}", fail);
    let body = (total_lp + total_raw) as f64;
    println!("LpString bytes:    {} ({:.1}%)", total_lp, 100.0 * total_lp as f64 / body.max(1.0));
    println!("Total bytes:       {} ({:.2} GB)", total_bytes, total_bytes as f64 / (1024.0 * 1024.0 * 1024.0));
}
