// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt

//! End-to-end mod-tooling regression test. For each "all-opaque"
//! format (`.pastage`, `.paseq`, `.paseqc`), sample N files, find the
//! FIRST LP-string in each, replace it with a longer value, re-parse,
//! and verify the new string is present and the file is still valid.
//!
//! `.paschedule` is INTENTIONALLY excluded — its 21-byte structured
//! header sits BEFORE the name CString, and the generic
//! `walk_u32_prefixed_strings` may match `u32` patterns inside the
//! header bytes (hash, version, etc.) before reaching the real name.
//! Mod authors editing `.paschedule` should use the JSON path
//! (parse → edit `name` field → serialize) instead of the generic
//! walker. Same applies to `.paatt`, which uses `u8` length prefixes
//! for its string tables.

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paseq::{
    replace_cstring_at, walk_u32_prefixed_strings, TypedPaseqFile,
};
use dmm_parser::binary::paseqc::TypedPaseqcFile;
use dmm_parser::binary::pastage::TypedPastageFile;
use dmm_parser::binary::paz;
use std::path::Path;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";
const PER_FORMAT_LIMIT: usize = 50;

#[derive(Default)]
struct Stats {
    samples: usize,
    pass: usize,
    fail: usize,
    skip_no_strings: usize,
    first_failures: Vec<(String, String)>,
}

fn record_failure(stats: &mut Stats, label: String, reason: String) {
    stats.fail += 1;
    if stats.first_failures.len() < 3 {
        stats.first_failures.push((label, reason));
    }
}

fn validate_edit(stats: &mut Stats, label: &str, bytes: &[u8],
    parse_check: impl Fn(&[u8]) -> bool)
{
    stats.samples += 1;
    let strings = walk_u32_prefixed_strings(bytes, 0);
    if strings.is_empty() {
        stats.skip_no_strings += 1;
        return;
    }
    // Replace the first string with a longer value.
    let (offset, original) = &strings[0];
    let new_value = format!("MOD_{}_TEST", original);
    let modified = match replace_cstring_at(bytes, *offset, Some(original), &new_value) {
        Ok(m) => m,
        Err(e) => {
            record_failure(stats, label.to_string(), format!("replace failed: {}", e));
            return;
        }
    };
    // Verify parse still works.
    if !parse_check(&modified) {
        record_failure(stats, label.to_string(),
            "modified file failed to re-parse".to_string());
        return;
    }
    // Verify new string appears.
    let strings2 = walk_u32_prefixed_strings(&modified, 0);
    let found_new = strings2.iter().any(|(_, s)| s == &new_value);
    if !found_new {
        record_failure(stats, label.to_string(),
            format!("new value {:?} not found after replace", new_value));
        return;
    }
    stats.pass += 1;
}

fn run() {
    let game_dir = Path::new(GAME_DIR);
    if !game_dir.exists() {
        eprintln!("Game install not found at {}", GAME_DIR);
        std::process::exit(1);
    }
    let mut groups: Vec<String> = std::fs::read_dir(game_dir)
        .expect("read game dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.len() == 4 && n.chars().all(|c| c.is_ascii_digit()))
        .collect();
    groups.sort();

    let mut pastage = Stats::default();
    let mut paseq = Stats::default();
    let mut paseqc = Stats::default();

    for group_name in &groups {
        let group_dir = game_dir.join(group_name);
        let pamt_data = match std::fs::read(group_dir.join("0.pamt")) { Ok(d) => d, Err(_) => continue };
        let pamt = match PackMeta::parse(&pamt_data, None) { Ok(p) => p, Err(_) => continue };
        let encrypt_info = pamt.header.encrypt_info.encrypt_info;
        for dir in &pamt.directories {
            for f in &dir.files {
                let lower = f.name.to_ascii_lowercase();
                let bytes = match paz::extract_file(&group_dir, f, &dir.path, &encrypt_info) {
                    Ok(b) => b, Err(_) => continue,
                };
                let label = format!("{}/{}/{}", group_name, dir.path, f.name);
                if lower.ends_with(".pastage") && pastage.samples < PER_FORMAT_LIMIT {
                    validate_edit(&mut pastage, &label, &bytes, |b| {
                        TypedPastageFile::parse(b).is_ok()
                    });
                } else if lower.ends_with(".paseq") && paseq.samples < PER_FORMAT_LIMIT {
                    validate_edit(&mut paseq, &label, &bytes, |b| {
                        TypedPaseqFile::parse(b).is_ok()
                    });
                } else if lower.ends_with(".paseqc") && paseqc.samples < PER_FORMAT_LIMIT {
                    validate_edit(&mut paseqc, &label, &bytes, |b| {
                        TypedPaseqcFile::parse(b).is_ok()
                    });
                }
            }
        }
        if pastage.samples >= PER_FORMAT_LIMIT && paseq.samples >= PER_FORMAT_LIMIT
            && paseqc.samples >= PER_FORMAT_LIMIT
        {
            break;
        }
    }

    let report = |name: &str, s: &Stats| {
        let total = s.pass + s.fail + s.skip_no_strings;
        let pass_pct = if s.pass + s.fail > 0 {
            s.pass as f64 / (s.pass + s.fail) as f64 * 100.0
        } else { 0.0 };
        println!("{:18} samples={:>3} pass={:>3} fail={:>3} skip={:>3} ({:.0}% pass of attempted)",
            name, total, s.pass, s.fail, s.skip_no_strings, pass_pct);
        if !s.first_failures.is_empty() {
            for (path, reason) in &s.first_failures {
                println!("    FAIL: {}\n          {}", path, reason);
            }
        }
    };

    println!("=== Mod-Tooling End-to-End Test ===");
    println!("(parse → walk_lp_strings → replace first → re-parse → verify new string present)");
    println!("Note: .paschedule and .paatt require structured-edit (JSON path), not generic walker.\n");
    report(".pastage", &pastage);
    report(".paseq", &paseq);
    report(".paseqc", &paseqc);

    let total_attempted = pastage.pass + pastage.fail + paseq.pass + paseq.fail
        + paseqc.pass + paseqc.fail;
    let total_pass = pastage.pass + paseq.pass + paseqc.pass;
    let total_fail = pastage.fail + paseq.fail + paseqc.fail;
    println!("\nTOTAL: {} attempted, {} pass, {} fail ({:.1}%)",
        total_attempted, total_pass, total_fail,
        if total_attempted > 0 { total_pass as f64 / total_attempted as f64 * 100.0 } else { 0.0 });
    if total_fail > 0 { std::process::exit(2); }
}

fn main() { run(); }
