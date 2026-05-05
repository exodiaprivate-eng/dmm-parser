// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Inventory paseqc / paschedule / paschedulepath files: count, size,
//! magic, sample paths. Used to confirm whether the same Tier 1.5
//! tokenizer (LpString + RawBytes) round-trips them.

use dmm_parser::binary::pamt::{Compression, PackMeta};
use lz4_flex::block::decompress;
use std::collections::BTreeMap;
use std::path::Path;

fn hex16(bytes: &[u8]) -> String {
    let mut out = String::new();
    for b in bytes.iter().take(16) {
        out.push_str(&format!("{:02X} ", b));
    }
    out
}

fn ascii16(bytes: &[u8]) -> String {
    let mut out = String::new();
    for b in bytes.iter().take(16) {
        out.push(if b.is_ascii_graphic() { *b as char } else { '.' });
    }
    out
}

fn main() {
    let game = Path::new(r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert");
    let mut groups: Vec<String> = std::fs::read_dir(game).unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.len() == 4 && n.chars().all(|c| c.is_ascii_digit()))
        .collect();
    groups.sort();

    #[derive(Default)]
    struct Stat {
        count: usize,
        total_bytes: u64,
        magics: BTreeMap<[u8; 4], usize>,
        samples: Vec<(String, String, u32, [u8; 16])>,
    }
    let mut stats: BTreeMap<String, Stat> = BTreeMap::new();

    let target_exts: &[&str] = &["paseqc", "paseqh", "paschedule", "paschedulepath", "pastagedata"];

    for g in &groups {
        let pamt_p = game.join(g).join("0.pamt");
        let paz_p = game.join(g).join("0.paz");
        if !pamt_p.exists() || !paz_p.exists() { continue; }
        let Ok(pamt_data) = std::fs::read(&pamt_p) else { continue };
        let Ok(meta) = PackMeta::parse(&pamt_data, None) else { continue };
        let paz = std::fs::read(&paz_p).ok();

        for d in &meta.directories {
            for f in &d.files {
                let ext = f.name.rsplit('.').next().unwrap_or("").to_lowercase();
                if !target_exts.contains(&ext.as_str()) { continue; }
                let st = stats.entry(ext.clone()).or_default();
                st.count += 1;
                st.total_bytes += f.file.uncompressed_size as u64;

                if let Some(paz) = &paz {
                    let off = f.file.chunk_offset as usize;
                    let comp = f.file.compressed_size as usize;
                    if off + comp > paz.len() { continue; }
                    let comp_bytes = &paz[off..off + comp];
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
                    if st.samples.len() < 4 {
                        let mut head = [0u8; 16];
                        let n = body.len().min(16);
                        head[..n].copy_from_slice(&body[..n]);
                        st.samples.push((g.clone(), format!("{}/{}", d.path, f.name), f.file.uncompressed_size, head));
                    }
                }
            }
        }
    }

    for (ext, st) in &stats {
        println!("\n.{} : {} files, {} total bytes", ext, st.count, st.total_bytes);
        println!("  Distinct magics:");
        let mut sorted: Vec<_> = st.magics.iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(a.1));
        for (magic, count) in sorted.iter().take(6) {
            let ascii: String = magic.iter()
                .map(|b| if b.is_ascii_graphic() { *b as char } else { '.' })
                .collect();
            println!("    {:02X} {:02X} {:02X} {:02X} ({}) [{}]",
                magic[0], magic[1], magic[2], magic[3], ascii, count);
        }
        for (g, path, size, head) in &st.samples {
            println!("  group {} | {:>10} bytes | {} | {} | {}",
                g, size, hex16(head), ascii16(head), path);
        }
    }
}
