//! PAMT-level diff between Win 1.06 install and Mac 1.06.01 depot.
//! Compares per-file uncompressed size + name across all packages.
//! Same uncompressed size + name = likely identical content.
//! Size mismatch or missing name = changed/added/removed file.

use dmm_parser::binary::papgt::PackGroupTreeMeta;
use dmm_parser::binary::pamt::PackMeta;
use std::collections::HashMap;
use std::path::Path;

const WIN_BASE: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";
const WIN_PAPGT: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert\meta\0.papgt";
const MAC_BASE: &str = r"C:\Program Files (x86)\Steam\steamapps\content\app_3321460\depot_3321462\CrimsonDesert_Steam.app\Contents\Resources\packages";
const MAC_PAPGT: &str = r"C:\Program Files (x86)\Steam\steamapps\content\app_3321460\depot_3321462\CrimsonDesert_Steam.app\Contents\Resources\packages\meta\0.papgt";

fn collect(label: &str, base: &str, papgt_path: &str) -> HashMap<String, u32> {
    let mut out = HashMap::new();
    let game_dir = Path::new(base);
    let papgt_data = std::fs::read(papgt_path).expect("read papgt");
    let papgt = PackGroupTreeMeta::parse(&papgt_data).expect("parse papgt");
    for entry in &papgt.entries {
        let group_dir = game_dir.join(&entry.group_name);
        let pamt_data = match std::fs::read(group_dir.join("0.pamt")) { Ok(d) => d, Err(_) => continue };
        let pamt = match PackMeta::parse(&pamt_data, None) { Ok(p) => p, Err(_) => continue };
        for dir in &pamt.directories {
            for f in &dir.files {
                let path = if dir.path.is_empty() { f.name.clone() } else { format!("{}/{}", dir.path, f.name) };
                out.insert(path.to_ascii_lowercase(), f.file.uncompressed_size);
            }
        }
    }
    println!("{}: {} files indexed", label, out.len());
    out
}

fn main() {
    let win = collect("Win 1.06 install", WIN_BASE, WIN_PAPGT);
    let mac = collect("Mac 1.06.01 depot", MAC_BASE, MAC_PAPGT);

    let mut added = Vec::new();      // in mac, not in win
    let mut removed = Vec::new();    // in win, not in mac
    let mut changed = Vec::new();    // both, different uncompressed_size
    let mut unchanged = 0usize;

    let mut all: std::collections::BTreeSet<&String> = std::collections::BTreeSet::new();
    for k in win.keys() { all.insert(k); }
    for k in mac.keys() { all.insert(k); }

    for path in &all {
        match (win.get(*path), mac.get(*path)) {
            (Some(&ws), Some(&ms)) if ws == ms => unchanged += 1,
            (Some(&ws), Some(&ms)) => changed.push(((*path).clone(), ws, ms)),
            (Some(&ws), None) => removed.push(((*path).clone(), ws)),
            (None, Some(&ms)) => added.push(((*path).clone(), ms)),
            (None, None) => {}
        }
    }

    println!("\n=== DIFF SUMMARY ===");
    println!("Unchanged files: {}", unchanged);
    println!("Changed files:   {}", changed.len());
    println!("Added (Mac only):{}", added.len());
    println!("Removed (Win only):{}", removed.len());

    let ch_bytes: u64 = changed.iter().map(|(_, _, m)| *m as u64).sum();
    let ad_bytes: u64 = added.iter().map(|(_, m)| *m as u64).sum();
    let rm_bytes: u64 = removed.iter().map(|(_, m)| *m as u64).sum();
    println!("\nChanged total uncompressed: {:.1} MB", ch_bytes as f64 / 1024.0 / 1024.0);
    println!("Added total uncompressed:   {:.1} MB", ad_bytes as f64 / 1024.0 / 1024.0);
    println!("Removed total uncompressed: {:.1} MB", rm_bytes as f64 / 1024.0 / 1024.0);

    println!("\n=== CHANGED files by extension ===");
    let mut by_ext: HashMap<String, (usize, u64)> = HashMap::new();
    for (p, _, m) in &changed {
        let ext = p.rsplit('.').next().unwrap_or("").to_string();
        let e = by_ext.entry(ext).or_insert((0, 0));
        e.0 += 1;
        e.1 += *m as u64;
    }
    let mut v: Vec<_> = by_ext.into_iter().collect();
    v.sort_by_key(|(_, (_, sz))| std::cmp::Reverse(*sz));
    for (ext, (count, size)) in v.iter().take(20) {
        println!("  {:>10}  {:>8.1} MB  .{}", count, *size as f64 / 1024.0 / 1024.0, ext);
    }

    println!("\n=== ADDED files (top 20 by size) ===");
    let mut added_sorted = added.clone();
    added_sorted.sort_by_key(|(_, s)| std::cmp::Reverse(*s));
    for (p, s) in added_sorted.iter().take(20) {
        println!("  +{:>8.1} KB  {}", *s as f64 / 1024.0, p);
    }

    println!("\n=== REMOVED files (top 10 by size) ===");
    let mut removed_sorted = removed.clone();
    removed_sorted.sort_by_key(|(_, s)| std::cmp::Reverse(*s));
    for (p, s) in removed_sorted.iter().take(10) {
        println!("  -{:>8.1} KB  {}", *s as f64 / 1024.0, p);
    }

    println!("\n=== CHANGED .pabgb tables (the game data) ===");
    let mut pabgb_changed: Vec<_> = changed.iter().filter(|(p, _, _)| p.ends_with(".pabgb")).collect();
    pabgb_changed.sort_by_key(|(_, w, m)| std::cmp::Reverse((*m as i64 - *w as i64).abs()));
    for (p, w, m) in pabgb_changed.iter().take(30) {
        let delta = *m as i64 - *w as i64;
        println!("  {} bytes Δ {:+}  {}", m, delta, p);
    }
}
