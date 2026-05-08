// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.
//
// Diagnostic for the BagSpace regression. Extracts vanilla
// `inventory.pabgb`, runs the MaxInventoryStorage mod's signature scan
// + byte patches against it, and reports whether each patch landed on
// vanilla bytes that match the mod's declared `original`. Tells us
// whether the regression is in the byte-patch logic or further down
// (overlay write / PAPGT registration / engine read).

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paz::extract_file;
use std::path::Path;

const PATCHES: &[(u64, &str, &str, &str)] = &[
    (3218, "backpack 50->600",          "3200",     "5802"),
    (3220, "backpack max 240->700",     "f000",     "bc02"),
    (4108, "private 240->1200",         "f000e803", "b004b004"),
    (4736, "private max 1000->1400",    "f000f000", "78057805"),
    (5497, "kuku pot 240->600",         "f000",     "5802"),
    (5499, "kuku pot max 240->700",     "f000",     "bc02"),
    (5826, "wardrobe 1000->1400",       "0A00E803", "78057805"),
    (6045, "kuku cooler 1000->1400",    "0A00E803", "78057805"),
    (6482, "collectibles 1000->1400",   "0A00E803", "78057805"),
    (15142,"gatherables 1000->1400",    "0A00E803", "78057805"),
];

const SIGNATURE_HEX: &str = "02000900000043686172616374657200";

fn hexdec(s: &str) -> Vec<u8> {
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap()).collect()
}

fn main() {
    let game = Path::new(r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert");

    // Find inventory.pabgb in the live game.
    let mut found: Option<Vec<u8>> = None;
    for entry in std::fs::read_dir(game).unwrap().filter_map(|e| e.ok()) {
        let g = entry.file_name().to_string_lossy().to_string();
        if g.len() != 4 || !g.chars().all(|c| c.is_ascii_digit()) { continue; }
        let group_dir = entry.path();
        let pamt = match std::fs::read(group_dir.join("0.pamt")) { Ok(d) => d, Err(_) => continue };
        let meta = match PackMeta::parse(&pamt, None) { Ok(m) => m, Err(_) => continue };
        let encrypt_info = meta.header.encrypt_info.encrypt_info;
        for d in &meta.directories {
            for f in &d.files {
                if f.name.eq_ignore_ascii_case("inventory.pabgb") {
                    if let Ok(body) = extract_file(&group_dir, f, &d.path, &encrypt_info) {
                        println!("Found inventory.pabgb in group {} ({} bytes)", g, body.len());
                        found = Some(body);
                    }
                }
            }
        }
        if found.is_some() { break; }
    }
    let vanilla = found.expect("inventory.pabgb not found");

    // Find the mod signature in the vanilla body.
    let sig = hexdec(SIGNATURE_HEX);
    let mut sig_pos: Option<usize> = None;
    for i in 0..vanilla.len().saturating_sub(sig.len()) {
        if vanilla[i..i + sig.len()] == sig[..] {
            sig_pos = Some(i);
            break;
        }
    }
    let base = sig_pos.expect("signature not found in vanilla");
    println!("Signature found at offset 0x{:X} ({})\n", base, base);

    // Score anchor (offsets relative to sig) vs absolute (offsets are absolute).
    let mut score_anchor = 0usize;
    let mut score_absolute = 0usize;
    for (off, _, orig, _) in PATCHES {
        let orig_bytes = hexdec(orig);
        let off_anchor = base + *off as usize;
        let off_absolute = *off as usize;
        if off_anchor + orig_bytes.len() <= vanilla.len()
            && vanilla[off_anchor..off_anchor + orig_bytes.len()] == orig_bytes[..]
        {
            score_anchor += 1;
        }
        if off_absolute + orig_bytes.len() <= vanilla.len()
            && vanilla[off_absolute..off_absolute + orig_bytes.len()] == orig_bytes[..]
        {
            score_absolute += 1;
        }
    }
    let use_absolute = score_absolute > score_anchor;
    println!(
        "Convention scoring: anchor={} absolute={}  → using {}\n",
        score_anchor, score_absolute,
        if use_absolute { "ABSOLUTE" } else { "ANCHOR (sig + off)" }
    );

    // Walk each patch and report.
    let mut applied = 0;
    let mut already = 0;
    let mut stale = 0;
    let mut patched_body = vanilla.clone();
    for (off, label, orig, new) in PATCHES {
        let orig_bytes = hexdec(orig);
        let new_bytes = hexdec(new);
        let resolved = if use_absolute { *off as usize } else { base + *off as usize };
        if resolved + orig_bytes.len() > vanilla.len() {
            println!("  0x{:04X} {:>32}: OOB", off, label);
            continue;
        }
        let actual = &vanilla[resolved..resolved + orig_bytes.len()];
        if actual == orig_bytes.as_slice() {
            patched_body[resolved..resolved + new_bytes.len()].copy_from_slice(&new_bytes);
            applied += 1;
            println!(
                "  0x{:04X} (resolved 0x{:X}) {:>32}: APPLIED  vanilla={}  new={}",
                off, resolved, label, orig, new,
            );
        } else if actual == new_bytes.as_slice() {
            already += 1;
            println!(
                "  0x{:04X} (resolved 0x{:X}) {:>32}: ALREADY-PATCHED",
                off, resolved, label,
            );
        } else {
            stale += 1;
            let actual_hex: String = actual.iter().map(|b| format!("{:02X}", b)).collect();
            println!(
                "  0x{:04X} (resolved 0x{:X}) {:>32}: STALE  expected={}  actual={}",
                off, resolved, label, orig, actual_hex,
            );
        }
    }

    println!(
        "\n=== Summary ===\nApplied: {}\nAlready patched: {}\nStale (vanilla doesn't match expected): {}",
        applied, already, stale
    );

    // Spot-check that our parser still round-trips this body
    use dmm_parser::dispatch::{parse_table_to_json, serialize_table_from_json};
    println!("\n=== Parser sanity: vanilla inventory_info round-trip ===");
    // Extract vanilla pabgh
    let mut pabgh_data: Option<Vec<u8>> = None;
    for entry in std::fs::read_dir(game).unwrap().filter_map(|e| e.ok()) {
        let g = entry.file_name().to_string_lossy().to_string();
        if g.len() != 4 || !g.chars().all(|c| c.is_ascii_digit()) { continue; }
        let group_dir = entry.path();
        let pamt = match std::fs::read(group_dir.join("0.pamt")) { Ok(d) => d, Err(_) => continue };
        let meta = match PackMeta::parse(&pamt, None) { Ok(m) => m, Err(_) => continue };
        let encrypt_info = meta.header.encrypt_info.encrypt_info;
        for d in &meta.directories {
            for f in &d.files {
                if f.name.eq_ignore_ascii_case("inventory.pabgh") {
                    if let Ok(body) = extract_file(&group_dir, f, &d.path, &encrypt_info) {
                        pabgh_data = Some(body);
                    }
                }
            }
        }
        if pabgh_data.is_some() { break; }
    }
    let pabgh = pabgh_data.expect("inventory.pabgh not found");

    let items = parse_table_to_json("inventory_info", &vanilla, Some(&pabgh))
        .expect("parse inventory_info failed");
    println!("Parsed {} inventory records", items.len());
    let written = serialize_table_from_json("inventory_info", &items)
        .expect("serialize inventory_info failed");
    if written == vanilla {
        println!("Parser round-trip: PASS (byte-exact)");
    } else {
        let first_diff = (0..vanilla.len().min(written.len())).find(|&i| vanilla[i] != written[i]);
        println!(
            "Parser round-trip: FAIL  in_size={} out_size={} first_diff={:?}",
            vanilla.len(), written.len(), first_diff,
        );
    }

    // Show the first few records' bag-space-relevant fields.
    println!("\n=== First 6 records (default_slot_count + max_slot_count) ===");
    for (i, item) in items.iter().take(6).enumerate() {
        let key = item.get("key").and_then(|v| v.as_u64()).unwrap_or(0);
        let sk = item.get("string_key").and_then(|v| v.as_str()).unwrap_or("?");
        let ds = item.get("default_slot_count").and_then(|v| v.as_u64()).unwrap_or(0);
        let ms = item.get("max_slot_count").and_then(|v| v.as_u64()).unwrap_or(0);
        println!("  [{}] key={:>5} string_key={:<32}  default={:>4}  max={:>4}",
            i, key, sk, ds, ms);
    }
}
