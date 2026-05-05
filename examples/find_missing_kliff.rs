//! Diagnostic: cross-check the English Kliff mod's expected paths against
//! the live game's PAPGT/PAMT index. Tells us whether the missing 951
//! files are genuinely absent from the install or if DMM's lookup is wrong.

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::papgt::PackGroupTreeMeta;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";
const PAPGT_PATH: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert\meta\0.papgt";
const MOD_DIR: &str = r"C:\Users\corin\Desktop\CD JSON Mod Manager\mods\kliff_female_voice_english\files";

fn build_game_index() -> BTreeSet<String> {
    let papgt_data = std::fs::read(PAPGT_PATH).expect("read PAPGT");
    let papgt = PackGroupTreeMeta::parse(&papgt_data).expect("parse PAPGT");
    let game_dir = Path::new(GAME_DIR);
    let mut paths = BTreeSet::new();
    for entry in &papgt.entries {
        let group_dir = game_dir.join(&entry.group_name);
        let pamt_path = group_dir.join("0.pamt");
        let Ok(data) = std::fs::read(&pamt_path) else { continue };
        let Ok(pamt) = PackMeta::parse(&data, None) else { continue };
        for dir in &pamt.directories {
            for f in &dir.files {
                let key = if dir.path.is_empty() {
                    f.name.to_lowercase()
                } else {
                    format!("{}/{}", dir.path.to_lowercase(), f.name.to_lowercase())
                };
                paths.insert(key);
            }
        }
    }
    paths
}

fn walk_mod_files(base: &Path, group: &str, prefix: &str, out: &mut Vec<String>) {
    let dir = base.join(group).join(prefix);
    if !dir.is_dir() { return; }
    walk_recursive(&dir, &dir, out);
}

fn walk_recursive(root: &Path, current: &Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(current) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk_recursive(root, &p, out);
        } else if let Ok(rel) = p.strip_prefix(root) {
            out.push(rel.to_string_lossy().replace('\\', "/").to_lowercase());
        }
    }
}

fn main() {
    let game_index = build_game_index();
    println!("Game has {} indexed file paths in PAZ archives", game_index.len());

    // Mod's files/0004/sound/... structure → in-game it lives at sound/...
    let mod_base = Path::new(MOD_DIR);
    let mut mod_paths_relative: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(mod_base) {
        for e in entries.flatten() {
            if !e.path().is_dir() { continue; }
            let group = e.file_name().to_string_lossy().to_string();
            // Walk this group's files/sound/... and build relative paths
            let group_dir = mod_base.join(&group);
            let mut paths: Vec<PathBuf> = Vec::new();
            collect_files(&group_dir, &mut paths);
            for p in paths {
                if let Ok(rel) = p.strip_prefix(&group_dir) {
                    mod_paths_relative.push(rel.to_string_lossy().replace('\\', "/").to_lowercase());
                }
            }
        }
    }

    println!("Mod ships {} file paths\n", mod_paths_relative.len());

    let mut found = 0usize;
    let mut missing = 0usize;
    let mut sample_missing: Vec<&String> = Vec::new();
    let mut sample_found: Vec<&String> = Vec::new();
    for path in &mod_paths_relative {
        // Try multiple game-side path variants because PAMT dir paths might
        // include different prefixes than what the mod assumed.
        let exists = game_index.contains(path);
        if exists { found += 1; if sample_found.len() < 3 { sample_found.push(path); } }
        else { missing += 1; if sample_missing.len() < 5 { sample_missing.push(path); } }
    }
    println!("===== Direct path match =====");
    println!("Found in game:  {}", found);
    println!("Missing:        {}", missing);
    if !sample_found.is_empty() {
        println!("\nSample FOUND:");
        for p in &sample_found { println!("  {}", p); }
    }
    if !sample_missing.is_empty() {
        println!("\nSample MISSING:");
        for p in &sample_missing { println!("  {}", p); }
    }

    // Now check: does ANY entry in the game index contain the .wem hash from a missing path?
    // This tells us if the file just lives at a different path than the mod expected.
    println!("\n===== Searching for missing files by basename =====");
    let mut basename_found = 0usize;
    let mut basename_paths: Vec<(String, String)> = Vec::new();
    for missing_path in sample_missing.iter().take(5) {
        let basename = missing_path.rsplit('/').next().unwrap_or(missing_path);
        let mut hits = Vec::new();
        for game_path in &game_index {
            if game_path.ends_with(basename) {
                hits.push(game_path.clone());
            }
        }
        if !hits.is_empty() {
            basename_found += 1;
            for hit in hits.iter().take(3) {
                basename_paths.push((basename.to_string(), hit.clone()));
            }
        } else {
            println!("  {} → NOT in game at any path", basename);
        }
    }
    println!("\nOf 5 sample missing files, {} found at different paths:", basename_found);
    for (bn, hit) in &basename_paths {
        println!("  {} found at: {}", bn, hit);
    }

    // Check what english(us) directories exist in the game
    println!("\n===== Game's english(us) sound directories =====");
    let mut english_dirs: BTreeSet<String> = BTreeSet::new();
    for path in &game_index {
        if path.contains("english") {
            if let Some(slash_pos) = path.rfind('/') {
                english_dirs.insert(path[..slash_pos].to_string());
            }
        }
    }
    for d in english_dirs.iter().take(20) {
        println!("  {}", d);
    }
    if english_dirs.len() > 20 {
        println!("  ... +{} more", english_dirs.len() - 20);
    }
    println!("\nTotal english(us)-containing dirs: {}", english_dirs.len());
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            collect_files(&p, out);
        } else {
            out.push(p);
        }
    }
}
