// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Find a partial-compression .dds in vanilla, read the raw bytes
//! straight from the .paz, and print the head + tail. We want to know
//! if "partial" means "stored-as-is" (just a flag) or if there's a
//! header / chunk layout we need to skip.

use dmm_parser::binary::pamt::PackMeta;
use std::path::Path;

fn dump_window(label: &str, bytes: &[u8], start: usize, len: usize) {
    let end = (start + len).min(bytes.len());
    println!("--- {} (offset 0x{:X} .. 0x{:X}) ---", label, start, end);
    let slice = &bytes[start..end];
    for chunk in slice.chunks(16) {
        for b in chunk { print!("{:02X} ", b); }
        for _ in chunk.len()..16 { print!("   "); }
        print!(" ");
        for b in chunk { print!("{}", if b.is_ascii_graphic() { *b as char } else { '.' }); }
        println!();
    }
}

fn main() {
    let game = Path::new(r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert");

    for entry in std::fs::read_dir(game).unwrap().filter_map(|e| e.ok()) {
        let g = entry.file_name().to_string_lossy().to_string();
        if g.len() != 4 || !g.chars().all(|c| c.is_ascii_digit()) { continue; }
        let group_dir = entry.path();
        let pamt_p = group_dir.join("0.pamt");
        if !pamt_p.exists() { continue; }
        let Ok(pamt_data) = std::fs::read(&pamt_p) else { continue };
        let Ok(meta) = PackMeta::parse(&pamt_data, None) else { continue };

        for d in &meta.directories {
            for f in &d.files {
                let ext = f.name.rsplit('.').next().unwrap_or("").to_lowercase();
                if ext != "dds" { continue; }
                if !f.file.is_partial { continue; }

                let paz_path = group_dir.join(format!("{}.paz", f.file.chunk_id));
                let Ok(paz_data) = std::fs::read(&paz_path) else { continue };
                let off = f.file.chunk_offset as usize;
                let csz = f.file.compressed_size as usize;
                let usz = f.file.uncompressed_size as usize;
                if off + csz > paz_data.len() { continue; }

                println!("=== {}/{} ===", d.path, f.name);
                println!("group:             {}", g);
                println!("chunk_offset:      0x{:X} ({})", off, off);
                println!("compressed_size:   {} bytes", csz);
                println!("uncompressed_size: {} bytes", usz);
                println!("flags:             0x{:02X}", f.file.flags);
                println!("is_partial:        {}", f.file.is_partial);
                println!("compression enum:  {:?}", f.file.compression);
                println!("crypto enum:       {:?}", f.file.crypto);
                println!();

                let raw = &paz_data[off..off + csz];
                dump_window("HEAD (first 64 bytes of stored chunk)", raw, 0, 64);
                println!();

                if csz > 64 {
                    let mid = csz / 2;
                    dump_window("MID (32 bytes around midpoint)", raw, mid.saturating_sub(16), 32);
                    println!();
                }

                if csz > 32 {
                    dump_window("TAIL (last 32 bytes)", raw, csz.saturating_sub(32), 32);
                }

                // Sanity check: does the chunk contain "DDS " anywhere?
                let dds_magic = [b'D', b'D', b'S', b' '];
                if let Some(pos) = raw.windows(4).position(|w| w == dds_magic) {
                    println!("\nDDS magic found at offset 0x{:X} ({}) of stored chunk", pos, pos);
                } else {
                    println!("\nNO DDS magic anywhere in stored chunk");
                }

                // Stop after one sample to keep output small.
                return;
            }
        }
    }
    println!("No partial DDS files found.");
}
