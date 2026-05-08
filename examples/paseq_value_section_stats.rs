// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt

//! Run the value-section accessor across every `.paseq` and `.paseqc`
//! and report:
//!   - Files where the schema/value boundary was located
//!   - Distribution of value section sizes
//!   - Average ratio (value_section_bytes / opaque_body_bytes)

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paseq::TypedPaseqFile;
use dmm_parser::binary::paseqc::TypedPaseqcFile;
use dmm_parser::binary::paz;
use std::path::Path;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";

#[derive(Default)]
struct Stats {
    files: usize,
    succeeded: usize,
    /// Sum of (value_section_size, opaque_body_size).
    value_total: usize,
    body_total: usize,
    min_value: Option<usize>,
    max_value: Option<usize>,
}

fn main() {
    let game_dir = Path::new(GAME_DIR);
    if !game_dir.exists() { eprintln!("Game install not found"); std::process::exit(1); }
    let mut groups: Vec<String> = std::fs::read_dir(game_dir)
        .expect("read game dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.len() == 4 && n.chars().all(|c| c.is_ascii_digit()))
        .collect();
    groups.sort();

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
                if lower.ends_with(".paseq") {
                    paseq.files += 1;
                    if let Ok(t) = TypedPaseqFile::parse(&bytes) {
                        if let Ok(values) = t.value_section() {
                            paseq.succeeded += 1;
                            let v = values.len();
                            paseq.value_total += v;
                            paseq.body_total += t.opaque_body.len();
                            paseq.min_value = Some(paseq.min_value.map_or(v, |m| m.min(v)));
                            paseq.max_value = Some(paseq.max_value.map_or(v, |m| m.max(v)));
                        }
                    }
                } else if lower.ends_with(".paseqc") {
                    paseqc.files += 1;
                    if let Ok(t) = TypedPaseqcFile::parse(&bytes) {
                        if let Ok(values) = t.value_section() {
                            paseqc.succeeded += 1;
                            let v = values.len();
                            paseqc.value_total += v;
                            paseqc.body_total += t.opaque_body.len();
                            paseqc.min_value = Some(paseqc.min_value.map_or(v, |m| m.min(v)));
                            paseqc.max_value = Some(paseqc.max_value.map_or(v, |m| m.max(v)));
                        }
                    }
                }
            }
        }
    }

    let report = |name: &str, s: &Stats| {
        println!("\n=== {} value section stats ===", name);
        println!("Files: {} (boundary located: {})", s.files, s.succeeded);
        if s.succeeded > 0 {
            println!("Value section: min={} max={} avg={:.0} bytes",
                s.min_value.unwrap_or(0),
                s.max_value.unwrap_or(0),
                s.value_total as f64 / s.succeeded as f64);
            let pct = s.value_total as f64 / s.body_total as f64 * 100.0;
            println!("Value/opaque_body ratio: {:.1}% values, {:.1}% schema",
                pct, 100.0 - pct);
        }
    };

    report(".paseq", &paseq);
    report(".paseqc", &paseqc);
}
