// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Long-tail proprietary formats not yet covered: imp, uianiminit, parg,
//! seqmt, pabv, pcg, spline2d, nav, spline, impostor, pasg, ies, thtml,
//! mi, pashv, papr, technique, paproj, pasound, ani, pai, pas, roadidx,
//! and the singletons.

use dmm_parser::binary::lp_token_stream::{LpTokenFile, Token};
use dmm_parser::binary::pamt::{Compression, PackMeta};
use lz4_flex::block::decompress;
use std::collections::BTreeMap;
use std::path::Path;

const TARGET_EXTS: &[&str] = &[
    "imp", "uianiminit", "parg", "seqmt", "pabv", "pcg", "spline2d",
    "nav", "spline", "impostor", "pasg", "ies", "thtml", "mi",
    "pashv", "papr", "technique", "paproj", "pasound", "ani", "pai",
    "pas", "roadidx",
    "binarygimmickcacheddata", "binarygimmickframeevent", "binarystring",
    "linkedsceneobject", "paacdesc", "paasmt", "pamhc", "pappt",
    "paprojdesc", "paschedulectx",
];

#[derive(Default, Clone)]
struct ExtStats {
    total: usize,
    pass: usize,
    fail: usize,
    total_tokens: u64,
    total_lp_strings: u64,
    total_raw_bytes: u64,
    magics: BTreeMap<[u8; 4], usize>,
}

fn main() {
    let game = Path::new(r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert");
    let mut groups: Vec<String> = std::fs::read_dir(game).unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.len() == 4 && n.chars().all(|c| c.is_ascii_digit()))
        .collect();
    groups.sort();

    let mut stats: BTreeMap<String, ExtStats> = BTreeMap::new();
    for ext in TARGET_EXTS { stats.insert((*ext).into(), ExtStats::default()); }

    for g in &groups {
        let pamt_p = game.join(g).join("0.pamt");
        let paz_p = game.join(g).join("0.paz");
        if !pamt_p.exists() || !paz_p.exists() { continue; }
        let Ok(pamt_data) = std::fs::read(&pamt_p) else { continue };
        let Ok(meta) = PackMeta::parse(&pamt_data, None) else { continue };
        let Ok(paz_data) = std::fs::read(&paz_p) else { continue };

        for d in &meta.directories {
            for f in &d.files {
                let ext = f.name.rsplit('.').next().unwrap_or("").to_lowercase();
                let Some(st) = stats.get_mut(&ext) else { continue };

                let off = f.file.chunk_offset as usize;
                let comp = f.file.compressed_size as usize;
                if off + comp > paz_data.len() { continue; }
                let comp_bytes = &paz_data[off..off + comp];
                let body = match f.file.compression {
                    Compression::Lz4 => match decompress(comp_bytes, f.file.uncompressed_size as usize) {
                        Ok(b) => b,
                        Err(_) => continue,
                    },
                    Compression::None => comp_bytes.to_vec(),
                    _ => continue,
                };
                if body.len() >= 4 {
                    let mut magic = [0u8; 4];
                    magic.copy_from_slice(&body[..4]);
                    *st.magics.entry(magic).or_insert(0) += 1;
                }
                st.total += 1;
                let parsed = match LpTokenFile::parse(&body) {
                    Ok(p) => p, Err(_) => { st.fail += 1; continue; }
                };
                let written = match parsed.to_bytes() {
                    Ok(b) => b, Err(_) => { st.fail += 1; continue; }
                };
                if written == body {
                    st.pass += 1;
                    st.total_tokens += parsed.tokens.len() as u64;
                    for t in &parsed.tokens {
                        match t {
                            Token::LpString(b) => st.total_lp_strings += b.len() as u64,
                            Token::RawBytes(b) => st.total_raw_bytes += b.len() as u64,
                        }
                    }
                } else {
                    st.fail += 1;
                }
            }
        }
    }

    println!("=== Round-trip summary ===");
    println!("{:<28} {:>8} {:>8} {:>8} {:>10} {:>10}",
        "extension", "files", "pass", "fail", "tokens/f", "string %");
    let mut grand_total = 0;
    let mut grand_pass = 0;
    for ext in TARGET_EXTS {
        let st = stats.get(*ext).unwrap();
        if st.total == 0 { continue; }
        let avg_tokens = if st.pass > 0 { st.total_tokens as f64 / st.pass as f64 } else { 0.0 };
        let total_body = (st.total_lp_strings + st.total_raw_bytes) as f64;
        let lp_pct = if total_body > 0.0 { 100.0 * st.total_lp_strings as f64 / total_body } else { 0.0 };
        println!("{:<28} {:>8} {:>8} {:>8} {:>10.1} {:>9.1}%",
            ext, st.total, st.pass, st.fail, avg_tokens, lp_pct);
        grand_total += st.total;
        grand_pass += st.pass;
    }
    println!("{:<28} {:>8} {:>8}", "TOTAL", grand_total, grand_pass);
}
