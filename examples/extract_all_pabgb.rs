// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Extract every `.pabgb` + matching `.pabgh` from the game install
//! that we don't already have in vanilla_dumps. Walks PAPGT → PAMTs.
//! Saves everything (or just the missing) to vanilla_dumps/.

use dmm_parser::binary::papgt::PackGroupTreeMeta;
use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paz;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

// Patch-day fixture extractor.
//
//   DMM_GAME_DIR=<install> DMM_DUMP_DIR=<out> cargo run --release --example extract_all_pabgb
//
// These were three hardcoded constants pointing at a Program Files install and a
// dump directory on someone else's machine, so every patch started by editing this
// file (and it failed with PermissionDenied if you forgot). Defaults are the live
// paths; override per patch with the env vars.
// ⚠ The game must be UNMOUNTED — a mounted install yields INJECTED bytes, not
// vanilla, which silently invalidates every downstream byte comparison.
const GAME_DIR_DEFAULT: &str = r"D:\SteamLibrary\steamapps\common\Crimson Desert";
const DUMP_DIR_DEFAULT: &str = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\latest";

fn game_dir_path() -> PathBuf {
    PathBuf::from(std::env::var("DMM_GAME_DIR").unwrap_or_else(|_| GAME_DIR_DEFAULT.into()))
}

fn dump_dir_path() -> PathBuf {
    PathBuf::from(std::env::var("DMM_DUMP_DIR").unwrap_or_else(|_| DUMP_DIR_DEFAULT.into()))
}

fn existing_dump_basenames(dump_dir: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    if let Ok(rd) = std::fs::read_dir(dump_dir) {
        for e in rd.flatten() {
            if let Some(stem) = e.path().file_stem().and_then(|s| s.to_str()) {
                out.insert(stem.to_string());
            }
        }
    }
    out
}

fn main() {
    let dump_dir = dump_dir_path();
    std::fs::create_dir_all(&dump_dir).expect("create dump dir");
    let existing = existing_dump_basenames(&dump_dir);
    println!("Already in vanilla_dumps: {} files (~{} tables)",
        existing.len(), existing.len() / 2);

    let game_owned = game_dir_path();
    let game_dir = game_owned.as_path();
    println!("game: {}\ndump: {}", game_dir.display(), dump_dir.display());
    let papgt_data =
        std::fs::read(game_dir.join("meta").join("0.papgt")).expect("read PAPGT");
    let papgt = PackGroupTreeMeta::parse(&papgt_data).expect("parse PAPGT");

    let mut found_pabgb = 0usize;
    let mut new_extracted = 0usize;
    let mut already_have = 0usize;
    let mut paths_seen: BTreeSet<String> = BTreeSet::new();

    for entry in &papgt.entries {
        let group_name = &entry.group_name;
        let group_dir = game_dir.join(group_name);
        let pamt_path = group_dir.join("0.pamt");
        let pamt_data = match std::fs::read(&pamt_path) { Ok(d) => d, Err(_) => continue };
        let pamt = match PackMeta::parse(&pamt_data, None) { Ok(p) => p, Err(_) => continue };
        let encrypt_info = pamt.header.encrypt_info.encrypt_info;

        for dir in &pamt.directories {
            for f in &dir.files {
                let lower = f.name.to_ascii_lowercase();
                let is_pabgb = lower.ends_with(".pabgb");
                let is_pabgh = lower.ends_with(".pabgh");
                if !is_pabgb && !is_pabgh { continue; }

                if is_pabgb { found_pabgb += 1; }

                let base = lower.trim_end_matches(".pabgb").trim_end_matches(".pabgh").to_string();
                paths_seen.insert(base.clone());

                if existing.contains(&base) {
                    already_have += 1;
                    continue;
                }

                let out_path = dump_dir.join(&f.name.to_ascii_lowercase());
                if out_path.exists() {
                    already_have += 1;
                    continue;
                }
                match paz::extract_file(&group_dir, f, &dir.path, &encrypt_info) {
                    Ok(bytes) => {
                        if let Err(e) = std::fs::write(&out_path, &bytes) {
                            eprintln!("WRITE {}: {}", out_path.display(), e);
                        } else {
                            new_extracted += 1;
                            println!("  + {} ({} bytes)", f.name, bytes.len());
                        }
                    }
                    Err(e) => eprintln!("EXTRACT {}: {}", f.name, e),
                }
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Total .pabgb references found: {}", found_pabgb);
    println!("Unique tables seen in archives: {}", paths_seen.len());
    println!("New extractions:               {}", new_extracted);
    println!("Already had:                   {}", already_have);
}
