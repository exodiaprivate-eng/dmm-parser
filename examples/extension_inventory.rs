//! Inventory every distinct file extension referenced in the live game's
//! PAZ archives via PAMT enumeration. Used to size the field-level
//! roadmap for binary asset formats.

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::papgt::PackGroupTreeMeta;
use std::collections::BTreeMap;
use std::path::Path;

const PAPGT_PATH: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert\meta\0.papgt";
const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";

fn main() {
    let papgt_data = std::fs::read(PAPGT_PATH).unwrap();
    let papgt = PackGroupTreeMeta::parse(&papgt_data).unwrap();
    let mut ext_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut ext_total_bytes: BTreeMap<String, u64> = BTreeMap::new();
    let mut total_files = 0u64;

    for entry in &papgt.entries {
        let group_dir = Path::new(GAME_DIR).join(&entry.group_name);
        let pamt_path = group_dir.join("0.pamt");
        let Ok(data) = std::fs::read(&pamt_path) else { continue };
        let Ok(pamt) = PackMeta::parse(&data, None) else { continue };
        for dir in &pamt.directories {
            for f in &dir.files {
                total_files += 1;
                let lower = f.name.to_ascii_lowercase();
                let ext = lower.rsplit('.').next().unwrap_or("");
                let ext = if lower.contains('.') { ext.to_string() } else { "(no-ext)".to_string() };
                *ext_counts.entry(ext.clone()).or_insert(0) += 1;
                *ext_total_bytes.entry(ext).or_insert(0) += f.file.uncompressed_size as u64;
            }
        }
    }

    let mut by_count: Vec<(String, u64, u64)> = ext_counts
        .iter()
        .map(|(k, v)| (k.clone(), *v, ext_total_bytes.get(k).copied().unwrap_or(0)))
        .collect();
    by_count.sort_by(|a, b| b.1.cmp(&a.1));

    println!("Total files in PAZ archives: {}\n", total_files);
    println!("{:<20} {:>10} {:>16}", "extension", "count", "total uncomp MB");
    println!("{}", "-".repeat(50));
    for (ext, n, bytes) in &by_count {
        let mb = (*bytes as f64) / 1_048_576.0;
        println!("{:<20} {:>10} {:>16.1}", ext, n, mb);
    }
}
