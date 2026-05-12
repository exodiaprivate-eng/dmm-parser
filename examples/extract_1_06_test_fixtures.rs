//! One-shot extractor: pull mercenaryinfo, dialogvoiceinfo, reserveslot
//! .pabgb + .pabgh from the LIVE 1.06 game install for fixture testing.
//! Outputs to ./fixtures_1_06/ (gitignored).

use dmm_parser::binary::papgt::PackGroupTreeMeta;
use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::paz;
use std::path::{Path, PathBuf};

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";
const PAPGT_PATH: &str =
    r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert\meta\0.papgt";
const OUT_DIR: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser\fixtures_1_06";

const WANTED: &[&str] = &[
    "mercenaryinfo.pabgb",
    "mercenaryinfo.pabgh",
    "dialogvoiceinfo.pabgb",
    "dialogvoiceinfo.pabgh",
    "reserveslot.pabgb",
    "reserveslot.pabgh",
];

fn main() {
    let out_dir = PathBuf::from(OUT_DIR);
    std::fs::create_dir_all(&out_dir).expect("create out dir");

    let game_dir = Path::new(GAME_DIR);
    let papgt_data = std::fs::read(PAPGT_PATH).expect("read PAPGT");
    let papgt = PackGroupTreeMeta::parse(&papgt_data).expect("parse PAPGT");

    let mut extracted = 0usize;

    for entry in &papgt.entries {
        let group_name = &entry.group_name;
        let group_dir = game_dir.join(group_name);
        let pamt_path = group_dir.join("0.pamt");
        let pamt_data = match std::fs::read(&pamt_path) { Ok(d) => d, Err(_) => continue };
        let pamt = match PackMeta::parse(&pamt_data, None) { Ok(p) => p, Err(_) => continue };
        let encrypt_info = pamt.header.encrypt_info.encrypt_info;

        for dir in &pamt.directories {
            for f in &dir.files {
                let name_lower = f.name.to_ascii_lowercase();
                if !WANTED.iter().any(|w| name_lower == *w) { continue; }

                let out_path = out_dir.join(&name_lower);
                match paz::extract_file(&group_dir, f, &dir.path, &encrypt_info) {
                    Ok(bytes) => {
                        std::fs::write(&out_path, &bytes).expect("write output");
                        println!("  + {} ({} bytes) ← group={}", name_lower, bytes.len(), group_name);
                        extracted += 1;
                    }
                    Err(e) => eprintln!("EXTRACT {}: {}", f.name, e),
                }
            }
        }
    }

    println!("\nTotal extracted: {} files to {}", extracted, OUT_DIR);
}
