// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt

//! Extract one specific `.pastage` file from the game install for
//! manual byte-level analysis. Writes both the raw bytes and a hex
//! dump to `target/pastage_samples/`.

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paz;
use std::path::Path;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";
const TARGET_GROUP: &str = "0014";
const TARGET_FILES: &[&str] = &[
    "cd_seq_spawn_auto_animal_bush_bird.pastage",
    "cd_seq_spawn_doc_animal_fish_jump_00.pastage",
    "cd_minigame_armwrest_base.pastage",
];
const OUT_DIR: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser\target\pastage_samples";

fn main() {
    let group_dir = Path::new(GAME_DIR).join(TARGET_GROUP);
    let pamt = PackMeta::parse(&std::fs::read(group_dir.join("0.pamt")).unwrap(), None).unwrap();
    let encrypt_info = pamt.header.encrypt_info.encrypt_info;

    std::fs::create_dir_all(OUT_DIR).unwrap();

    for dir in &pamt.directories {
        for f in &dir.files {
            if !TARGET_FILES.iter().any(|t| f.name == *t) { continue; }
            let bytes = paz::extract_file(&group_dir, f, &dir.path, &encrypt_info).unwrap();
            let bin_path = format!("{}/{}", OUT_DIR, f.name);
            let hex_path = format!("{}/{}.hex", OUT_DIR, f.name);
            std::fs::write(&bin_path, &bytes).unwrap();

            // Hex dump: 16 bytes per line with ASCII column
            let mut hex = String::new();
            for (i, chunk) in bytes.chunks(16).enumerate() {
                let off = i * 16;
                hex.push_str(&format!("{:08x}  ", off));
                for b in chunk { hex.push_str(&format!("{:02x} ", b)); }
                for _ in chunk.len()..16 { hex.push_str("   "); }
                hex.push_str(" ");
                for b in chunk {
                    hex.push(if (0x20..=0x7e).contains(b) { *b as char } else { '.' });
                }
                hex.push('\n');
            }
            std::fs::write(&hex_path, &hex).unwrap();
            println!("{} -> {} ({} bytes)", f.name, bin_path, bytes.len());
        }
    }
}
