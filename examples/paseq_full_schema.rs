// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt

//! Run the linearly-following nested-class-block walker on every
//! `.paseq` and `.paseqc` and report the per-class statistics. The
//! walker stops when it encounters non-CString-shaped data (the start
//! of the value section).

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paseq::TypedPaseqFile;
use dmm_parser::binary::paseqc::TypedPaseqcFile;
use dmm_parser::binary::paz;
use std::collections::BTreeMap;
use std::path::Path;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";

#[derive(Default)]
struct Stats {
    files_processed: usize,
    files_succeeded: usize,
    total_class_blocks: usize,
    /// Distribution: blocks_per_file -> count_of_files
    blocks_per_file: BTreeMap<usize, usize>,
    /// Class name -> appearances
    classes_by_name: BTreeMap<String, usize>,
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
                    paseq.files_processed += 1;
                    if let Ok(t) = TypedPaseqFile::parse(&bytes) {
                        if let Ok(blocks) = t.all_class_blocks() {
                            paseq.files_succeeded += 1;
                            paseq.total_class_blocks += blocks.len();
                            *paseq.blocks_per_file.entry(blocks.len()).or_insert(0) += 1;
                            for block in &blocks {
                                *paseq.classes_by_name.entry(block.class_name.clone()).or_insert(0) += 1;
                            }
                        }
                    }
                } else if lower.ends_with(".paseqc") {
                    paseqc.files_processed += 1;
                    if let Ok(t) = TypedPaseqcFile::parse(&bytes) {
                        if let Ok(blocks) = t.all_class_blocks() {
                            paseqc.files_succeeded += 1;
                            paseqc.total_class_blocks += blocks.len();
                            *paseqc.blocks_per_file.entry(blocks.len()).or_insert(0) += 1;
                            for block in &blocks {
                                *paseqc.classes_by_name.entry(block.class_name.clone()).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    let report = |name: &str, s: &Stats| {
        println!("\n=== {} all_class_blocks() ===", name);
        println!("Files processed: {} (walker succeeded: {})",
            s.files_processed, s.files_succeeded);
        println!("Total class blocks parsed: {}", s.total_class_blocks);
        println!("Avg blocks/file: {:.1}",
            s.total_class_blocks as f64 / s.files_succeeded.max(1) as f64);
        println!("\nBlocks-per-file distribution (top 20):");
        let mut dist: Vec<_> = s.blocks_per_file.iter().collect();
        dist.sort_by(|a, b| b.1.cmp(a.1));
        for (n_blocks, count) in dist.iter().take(20) {
            println!("  {} blocks: {} files", n_blocks, count);
        }
        println!("\nDistinct class names: {}", s.classes_by_name.len());
        println!("Top 30 by frequency:");
        let mut classes: Vec<_> = s.classes_by_name.iter().collect();
        classes.sort_by(|a, b| b.1.cmp(a.1));
        for (cname, count) in classes.iter().take(30) {
            println!("  {:>6}  {}", count, cname);
        }
        if classes.len() > 30 {
            println!("  ... and {} more class names", classes.len() - 30);
        }
    };

    report(".paseq", &paseq);
    report(".paseqc", &paseqc);
}
