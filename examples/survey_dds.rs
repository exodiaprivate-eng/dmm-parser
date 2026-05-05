// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Inventory every .dds file in vanilla AFTER PAZ decrypt + decompress.
//! Records width, height, mip_count, fourcc, optional DXGI_FORMAT.

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paz::extract_file;
use std::collections::BTreeMap;
use std::path::Path;

fn fourcc_to_str(v: u32) -> String {
    let bytes = v.to_le_bytes();
    let s: String = bytes.iter()
        .map(|b| if b.is_ascii_graphic() { *b as char } else { '?' })
        .collect();
    format!("{:>8} (0x{:08X})", s, v)
}

fn main() {
    let game = Path::new(r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert");
    let mut groups: Vec<String> = std::fs::read_dir(game).unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.len() == 4 && n.chars().all(|c| c.is_ascii_digit()))
        .collect();
    groups.sort();

    let mut total: usize = 0;
    let mut bad_magic: usize = 0;
    let mut wrong_size: usize = 0;
    let mut extract_err: usize = 0;
    let mut total_bytes: u64 = 0;

    let mut fourcc_counts: BTreeMap<u32, usize> = BTreeMap::new();
    let mut dxgi_counts: BTreeMap<u32, usize> = BTreeMap::new();
    let mut mip_counts: BTreeMap<u32, usize> = BTreeMap::new();
    let mut crypto_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut compression_counts: BTreeMap<String, usize> = BTreeMap::new();

    for g in &groups {
        let group_dir = game.join(g);
        let pamt_p = group_dir.join("0.pamt");
        if !pamt_p.exists() { continue; }
        let Ok(pamt_data) = std::fs::read(&pamt_p) else { continue };
        let Ok(meta) = PackMeta::parse(&pamt_data, None) else { continue };
        let encrypt_info = meta.header.encrypt_info.encrypt_info;

        for d in &meta.directories {
            for f in &d.files {
                let ext = f.name.rsplit('.').next().unwrap_or("").to_lowercase();
                if ext != "dds" { continue; }
                total += 1;
                total_bytes += f.file.uncompressed_size as u64;

                *crypto_counts.entry(format!("{:?}", f.file.crypto)).or_insert(0) += 1;
                *compression_counts.entry(format!("{:?}", f.file.compression)).or_insert(0) += 1;

                let body = match extract_file(&group_dir, f, &d.path, &encrypt_info) {
                    Ok(b) => b,
                    Err(_) => { extract_err += 1; continue; }
                };

                if body.len() < 128 {
                    wrong_size += 1;
                    continue;
                }
                let magic = u32::from_le_bytes(body[0..4].try_into().unwrap());
                if magic != 0x2053_4444 {
                    bad_magic += 1;
                    continue;
                }
                let _height = u32::from_le_bytes(body[12..16].try_into().unwrap());
                let _width = u32::from_le_bytes(body[16..20].try_into().unwrap());
                let mip_count = u32::from_le_bytes(body[28..32].try_into().unwrap());
                let pf_fourcc = u32::from_le_bytes(body[84..88].try_into().unwrap());

                *fourcc_counts.entry(pf_fourcc).or_insert(0) += 1;
                *mip_counts.entry(mip_count).or_insert(0) += 1;
                if pf_fourcc == 0x3031_5844 && body.len() >= 148 {
                    let dxgi_format = u32::from_le_bytes(body[128..132].try_into().unwrap());
                    *dxgi_counts.entry(dxgi_format).or_insert(0) += 1;
                }
            }
        }
    }

    println!("=== DDS inventory (after decrypt + decompress) ===");
    println!("Total .dds files:     {}", total);
    println!("Total uncompressed:   {} bytes ({:.2} GB)", total_bytes, total_bytes as f64 / (1024.0 * 1024.0 * 1024.0));
    println!("Extract failed:       {}", extract_err);
    println!("Bad / missing magic:  {}", bad_magic);
    println!("Smaller than header:  {}", wrong_size);
    let parsed = total - extract_err - bad_magic - wrong_size;
    println!("Successfully parsed:  {} ({:.2}%)", parsed, 100.0 * parsed as f64 / total.max(1) as f64);

    println!("\n=== Crypto distribution (PAMT entries) ===");
    for (k, v) in &crypto_counts { println!("  {:>30} : {}", k, v); }
    println!("\n=== Compression distribution (PAMT entries) ===");
    for (k, v) in &compression_counts { println!("  {:>30} : {}", k, v); }

    println!("\n=== Pixel-format fourCC (top 15) ===");
    let mut fc: Vec<_> = fourcc_counts.iter().collect();
    fc.sort_by(|a, b| b.1.cmp(a.1));
    for (code, count) in fc.iter().take(15) {
        println!("  {:>6} files  {}", count, fourcc_to_str(**code));
    }

    println!("\n=== DXGI_FORMAT (DX10 ext, top 15) ===");
    let mut dx: Vec<_> = dxgi_counts.iter().collect();
    dx.sort_by(|a, b| b.1.cmp(a.1));
    for (fmt, count) in dx.iter().take(15) {
        println!("  {:>6} files  DXGI_FORMAT = {}", count, fmt);
    }

    println!("\n=== Mip-count distribution (top 12) ===");
    let mut mc: Vec<_> = mip_counts.iter().collect();
    mc.sort_by(|a, b| b.1.cmp(a.1));
    for (n, count) in mc.iter().take(12) {
        println!("  {:>6} files  mip_count = {}", count, n);
    }
}
