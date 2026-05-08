// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt

//! JSON-path mod tooling end-to-end test for the structured-header
//! formats (`.paschedule`, `.paatt`). Sister to `mod_tooling_e2e.rs`
//! which validates the walk + replace pattern for all-opaque formats.
//!
//! For each format:
//!   1. Parse → JSON
//!   2. Edit a named field
//!   3. Serialize back to bytes
//!   4. Re-parse the modified bytes
//!   5. Verify the edit stuck
//!
//! This is the canonical regression test for the
//! `parse_<format>_from_file` / `write_<format>_to_file` workflow.

use dmm_parser::binary::paatt::PaattFile;
use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paschedule::TypedPascheduleFile;
use dmm_parser::binary::paz;
use dmm_parser::json_traits::{ToJsonValue, WriteJsonValue};
use std::path::Path;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";
const PER_FORMAT_LIMIT: usize = 30;

#[derive(Default)]
struct Stats {
    samples: usize,
    pass: usize,
    fail: usize,
    first_failures: Vec<(String, String)>,
}

fn record_fail(stats: &mut Stats, label: String, reason: String) {
    stats.fail += 1;
    if stats.first_failures.len() < 3 {
        stats.first_failures.push((label, reason));
    }
}

fn test_paschedule(stats: &mut Stats, label: &str, bytes: &[u8]) {
    stats.samples += 1;
    let typed = match TypedPascheduleFile::parse(bytes) {
        Ok(t) => t,
        Err(e) => return record_fail(stats, label.to_string(), format!("parse: {}", e)),
    };
    let mut json = typed.to_json_value();
    let original_name = match json.get("name").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return record_fail(stats, label.to_string(), "missing 'name' field".into()),
    };
    let new_name = format!("MOD_{}_TEST", original_name);
    json["name"] = serde_json::Value::String(new_name.clone());

    let mut out = Vec::new();
    if let Err(e) = TypedPascheduleFile::write_from_json(&mut out, &json) {
        return record_fail(stats, label.to_string(), format!("write_from_json: {}", e));
    }
    let typed2 = match TypedPascheduleFile::parse(&out) {
        Ok(t) => t,
        Err(e) => return record_fail(stats, label.to_string(), format!("re-parse: {}", e)),
    };
    if typed2.name.data == new_name {
        stats.pass += 1;
    } else {
        record_fail(stats, label.to_string(),
            format!("edit lost: expected {:?} got {:?}", new_name, typed2.name.data));
    }
}

fn test_paatt(stats: &mut Stats, label: &str, bytes: &[u8]) {
    stats.samples += 1;
    let paatt = match PaattFile::parse(bytes) {
        Ok(p) => p,
        Err(e) => return record_fail(stats, label.to_string(), format!("parse: {}", e)),
    };
    if paatt.effect_name_table.is_empty() {
        // No table to edit; skip rather than fail.
        return;
    }
    let mut json = paatt.to_json_value();
    let new_value = "MOD_TEST_EFFECT".to_string();
    json["effect_name_table"][0] = serde_json::Value::String(new_value.clone());

    let mut out = Vec::new();
    if let Err(e) = PaattFile::write_from_json(&mut out, &json) {
        return record_fail(stats, label.to_string(), format!("write_from_json: {}", e));
    }
    let paatt2 = match PaattFile::parse(&out) {
        Ok(p) => p,
        Err(e) => return record_fail(stats, label.to_string(), format!("re-parse: {}", e)),
    };
    if paatt2.effect_name_table.first().map(|s| s.as_str()) == Some(new_value.as_str()) {
        stats.pass += 1;
    } else {
        record_fail(stats, label.to_string(),
            format!("edit lost: effect_name_table[0] = {:?}",
                paatt2.effect_name_table.first()));
    }
}

fn main() {
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

    let mut paschedule = Stats::default();
    let mut paatt = Stats::default();

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
                if lower.ends_with(".paschedule")
                    && !lower.ends_with(".paschedulepath")
                    && paschedule.samples < PER_FORMAT_LIMIT
                {
                    test_paschedule(&mut paschedule, &label, &bytes);
                } else if lower.ends_with(".paatt") && paatt.samples < PER_FORMAT_LIMIT {
                    test_paatt(&mut paatt, &label, &bytes);
                }
            }
        }
        if paschedule.samples >= PER_FORMAT_LIMIT && paatt.samples >= PER_FORMAT_LIMIT {
            break;
        }
    }

    let report = |name: &str, s: &Stats| {
        let attempted = s.pass + s.fail;
        println!("{:14} samples={:>3} pass={:>3} fail={:>3}", name, s.samples, s.pass, s.fail);
        for (path, reason) in &s.first_failures {
            println!("    FAIL: {}\n          {}", path, reason);
        }
        attempted
    };

    println!("=== JSON-Path Mod-Tooling End-to-End Test ===");
    println!("(parse → edit named field → serialize → re-parse → verify)\n");
    let p1 = report(".paschedule", &paschedule);
    let p2 = report(".paatt", &paatt);
    let total_pass = paschedule.pass + paatt.pass;
    let total_fail = paschedule.fail + paatt.fail;
    let total = p1 + p2;
    println!("\nTOTAL: {} attempted, {} pass, {} fail ({:.1}%)",
        total, total_pass, total_fail,
        if total > 0 { total_pass as f64 / total as f64 * 100.0 } else { 0.0 });
    if total_fail > 0 { std::process::exit(2); }
}
