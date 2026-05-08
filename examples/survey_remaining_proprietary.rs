// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Bulk round-trip on the remaining proprietary formats not yet
//! covered. lp_token_stream is byte-exact for any input, but the
//! string-density column tells us how field-addressable the format
//! actually is at Tier 1.5 — anything below ~5% likely needs a typed
//! parser to be useful for mods.

use dmm_parser::binary::lp_token_stream::{LpTokenFile, Token};
use dmm_parser::binary::pamt::{Compression, PackMeta};
use lz4_flex::block::decompress;
use std::collections::BTreeMap;
use std::path::Path;

const TARGET_EXTS: &[&str] = &[
    "paa", "paa_metabin", "pac", "pam", "meshinfo", "padxil",
    "road", "roadsector", "paaa", "paat", "paadgi", "paef",
    "paphys", "ragdoll", "papb", "paaa_metabin",
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
    sample: Option<(String, String, u32)>,
}

fn main() {
    let game = Path::new(r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert");
    let mut groups: Vec<String> = std::fs::read_dir(game).unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.len() == 4 && n.chars().all(|c| c.is_ascii_digit()))
        .collect();
    groups.sort();

    // Discovery pass: also list any other extension we haven't classified
    // anywhere yet, so we know what's left after this batch.
    let known: std::collections::HashSet<&'static str> = [
        "paseq", "pastage", "paseqc", "paseqh", "paschedule", "paschedulepath",
        "binarygimmick", "prefab", "pami", "paac", "pabc", "pae", "pat",
        "paem", "palevel", "levelinfo", "motionblending", "pbd", "pab",
        "paccd", "material", "pamlod", "pampg",
        "paa", "paa_metabin", "pac", "pam", "meshinfo", "padxil",
        "pabgb", "pabgh", "paloc", "paatt",
        "dds", "wem", "bnk", "hkx", "mp4",
        "xml", "css", "html", "pac_xml", "app_xml", "prefabdata_xml",
        "paz", "pamt", "papgt", "save", "txt", "ini", "cfg", "json",
        "log", "bak", "tmp",
    ].iter().copied().collect();
    let mut unknown_counts: BTreeMap<String, usize> = BTreeMap::new();

    let mut stats: BTreeMap<String, ExtStats> = BTreeMap::new();
    for ext in TARGET_EXTS {
        stats.insert((*ext).into(), ExtStats::default());
    }

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
                if !known.contains(ext.as_str()) {
                    *unknown_counts.entry(ext.clone()).or_insert(0) += 1;
                }
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
                if st.sample.is_none() {
                    st.sample = Some((g.clone(), format!("{}/{}", d.path, f.name), f.file.uncompressed_size));
                }

                st.total += 1;
                let parsed = match LpTokenFile::parse(&body) {
                    Ok(p) => p,
                    Err(_) => { st.fail += 1; continue; }
                };
                let written = match parsed.to_bytes() {
                    Ok(b) => b,
                    Err(_) => { st.fail += 1; continue; }
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
    println!("{:<18} {:>8} {:>8} {:>8} {:>10} {:>10}",
        "extension", "files", "pass", "fail", "tokens/f", "string %");
    let mut grand_total = 0;
    let mut grand_pass = 0;
    for ext in TARGET_EXTS {
        let st = stats.get(*ext).unwrap();
        if st.total == 0 { continue; }
        let avg_tokens = if st.pass > 0 { st.total_tokens as f64 / st.pass as f64 } else { 0.0 };
        let total_body = (st.total_lp_strings + st.total_raw_bytes) as f64;
        let lp_pct = if total_body > 0.0 { 100.0 * st.total_lp_strings as f64 / total_body } else { 0.0 };
        println!("{:<18} {:>8} {:>8} {:>8} {:>10.1} {:>9.1}%",
            ext, st.total, st.pass, st.fail, avg_tokens, lp_pct);
        grand_total += st.total;
        grand_pass += st.pass;
    }
    println!("{:<18} {:>8} {:>8}", "TOTAL", grand_total, grand_pass);

    println!("\n=== Magics + first sample ===");
    for ext in TARGET_EXTS {
        let st = stats.get(*ext).unwrap();
        if st.total == 0 { continue; }
        println!("\n.{} : {} files (pass {}, fail {})", ext, st.total, st.pass, st.fail);
        let mut sorted: Vec<_> = st.magics.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        for (magic, count) in sorted.iter().take(3) {
            let ascii: String = magic.iter()
                .map(|b| if b.is_ascii_graphic() { *b as char } else { '.' })
                .collect();
            println!("  magic {:02X} {:02X} {:02X} {:02X} ({}) [{}]",
                magic[0], magic[1], magic[2], magic[3], ascii, count);
        }
        if let Some((g, p, sz)) = &st.sample {
            println!("  first: group {} | {} bytes | {}", g, sz, p);
        }
    }

    println!("\n=== Other unclassified extensions in vanilla ===");
    let mut sorted: Vec<_> = unknown_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    for (ext, count) in sorted.iter().take(40) {
        println!("  .{} : {} files", ext, count);
    }
}
