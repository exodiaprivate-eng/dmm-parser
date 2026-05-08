// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt

//! Walk the value section of every `.paseq` and `.paseqc` and report
//! the most common embedded string values across the corpus. These
//! are the staticstringA values + asset path references that mod
//! authors most commonly want to edit.

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paseq::TypedPaseqFile;
use dmm_parser::binary::paseqc::TypedPaseqcFile;
use dmm_parser::binary::paz;
use std::collections::BTreeMap;
use std::path::Path;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";

#[derive(Default)]
struct Stats {
    files: usize,
    total_strings: usize,
    /// Strings by length bucket
    by_length: BTreeMap<usize, usize>,
    /// Most common (capped to top 50)
    sample_strings: Vec<String>,
    string_counts: BTreeMap<String, usize>,
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
                        if let Ok(strings) = t.value_section_strings() {
                            paseq.total_strings += strings.len();
                            for (_off, s) in &strings {
                                paseq.by_length.entry(s.len() / 16).and_modify(|c| *c += 1).or_insert(1);
                                *paseq.string_counts.entry(s.clone()).or_insert(0) += 1;
                                if paseq.sample_strings.len() < 30 && paseq.files <= 5 {
                                    paseq.sample_strings.push(format!("{} (file {})", s, f.name));
                                }
                            }
                        }
                    }
                } else if lower.ends_with(".paseqc") {
                    paseqc.files += 1;
                    if let Ok(t) = TypedPaseqcFile::parse(&bytes) {
                        if let Ok(strings) = t.value_section_strings() {
                            paseqc.total_strings += strings.len();
                            for (_off, s) in &strings {
                                paseqc.by_length.entry(s.len() / 16).and_modify(|c| *c += 1).or_insert(1);
                                *paseqc.string_counts.entry(s.clone()).or_insert(0) += 1;
                                if paseqc.sample_strings.len() < 30 && paseqc.files <= 5 {
                                    paseqc.sample_strings.push(format!("{} (file {})", s, f.name));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let report = |name: &str, s: &Stats| {
        println!("\n=== {} value-section strings ===", name);
        println!("Files: {}, total strings extracted: {}", s.files, s.total_strings);
        println!("Avg strings per file: {:.1}", s.total_strings as f64 / s.files.max(1) as f64);
        println!("Distinct strings: {}", s.string_counts.len());
        println!("\nFirst 30 strings from first 5 files:");
        for s in s.sample_strings.iter().take(30) {
            println!("  {}", s);
        }
        println!("\nTop 20 most-frequent strings:");
        let mut counts: Vec<_> = s.string_counts.iter().collect();
        counts.sort_by(|a, b| b.1.cmp(a.1));
        for (val, count) in counts.iter().take(20) {
            let display = if val.len() > 80 { format!("{}...", &val[..77]) } else { (*val).clone() };
            println!("  {:>5}  {}", count, display);
        }
    };

    report(".paseq", &paseq);
    report(".paseqc", &paseqc);
}
