// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Pull a couple of small .pastage files from vanilla, decompress, and walk
//! the bytes printing every plausible u32 + length-prefixed string. Goal:
//! understand the basic layout (header → stage path → record list?) before
//! committing to a parser shape.

use dmm_parser::binary::pamt::{Compression, PackMeta};
use lz4_flex::block::decompress;
use std::path::Path;

const TARGETS: &[&str] = &[
    "cd_seq_spawn_auto_animal_bush_bird.pastage",
    "cd_item_fishrod_base_main_only.pastage",
    "cd_gimmick_minigame_new_milkingcow_base.pastage",
];

fn main() {
    let game = Path::new(r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert");

    for target in TARGETS {
        let mut found: Option<(String, String, Vec<u8>)> = None;
        for entry in std::fs::read_dir(game).unwrap().filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.len() != 4 || !name.chars().all(|c| c.is_ascii_digit()) { continue; }
            let pamt_p = entry.path().join("0.pamt");
            let paz_p = entry.path().join("0.paz");
            if !pamt_p.exists() || !paz_p.exists() { continue; }
            let Ok(pamt_data) = std::fs::read(&pamt_p) else { continue };
            let Ok(meta) = PackMeta::parse(&pamt_data, None) else { continue };
            for d in &meta.directories {
                for f in &d.files {
                    if f.name == *target {
                        let paz_data = std::fs::read(&paz_p).unwrap();
                        let off = f.file.chunk_offset as usize;
                        let comp = f.file.compressed_size as usize;
                        let comp_bytes = &paz_data[off..off + comp];
                        let body = match f.file.compression {
                            Compression::Lz4 => decompress(comp_bytes, f.file.uncompressed_size as usize).unwrap(),
                            Compression::None => comp_bytes.to_vec(),
                            other => panic!("compression {:?}", other),
                        };
                        found = Some((name.clone(), format!("{}/{}", d.path, f.name), body));
                    }
                }
            }
        }

        let Some((group, path, body)) = found else {
            println!("=== {} NOT FOUND ===\n", target);
            continue;
        };

        println!("=== {} (group {}, {} bytes) ===", path, group, body.len());

        // Header dump (first 96 bytes)
        for chunk_start in (0..96.min(body.len())).step_by(16) {
            let end = (chunk_start + 16).min(body.len());
            print!("{:04X}: ", chunk_start);
            for i in chunk_start..end {
                print!("{:02X} ", body[i]);
            }
            for _ in end..chunk_start + 16 {
                print!("   ");
            }
            print!(" ");
            for i in chunk_start..end {
                let b = body[i];
                print!("{}", if b.is_ascii_graphic() { b as char } else { '.' });
            }
            println!();
        }
        println!();

        // Walk the file looking for length-prefixed strings (length 1..=128, ascii).
        // Print each one we find with its offset.
        println!("--- Plausible length-prefixed strings ---");
        let mut p = 0usize;
        let mut printed = 0usize;
        while p + 4 <= body.len() && printed < 40 {
            let len = u32::from_le_bytes(body[p..p + 4].try_into().unwrap());
            if len > 0 && len <= 128 && p + 4 + len as usize <= body.len() {
                let slice = &body[p + 4..p + 4 + len as usize];
                let printable = slice.iter().filter(|b| b.is_ascii_graphic() || **b == b'/' || **b == b'_' || **b == b'.' || **b == b' ').count();
                if printable >= (len as usize).saturating_sub(1) && len >= 3 {
                    let s = String::from_utf8_lossy(slice);
                    println!("  0x{:04X} (decimal {}): len={} \"{}\"", p, p, len, s);
                    p += 4 + len as usize;
                    printed += 1;
                    continue;
                }
            }
            p += 1;
        }
        if printed >= 40 {
            println!("  ... (truncated at 40)");
        }
        println!();

        // Trailer dump (last 64 bytes)
        let tail_start = body.len().saturating_sub(64);
        println!("--- Trailer (last {} bytes from 0x{:04X}) ---", body.len() - tail_start, tail_start);
        for chunk_start in (tail_start..body.len()).step_by(16) {
            let end = (chunk_start + 16).min(body.len());
            print!("{:04X}: ", chunk_start);
            for i in chunk_start..end {
                print!("{:02X} ", body[i]);
            }
            for _ in end..chunk_start + 16 {
                print!("   ");
            }
            print!(" ");
            for i in chunk_start..end {
                let b = body[i];
                print!("{}", if b.is_ascii_graphic() { b as char } else { '.' });
            }
            println!();
        }
        println!();
    }
}
