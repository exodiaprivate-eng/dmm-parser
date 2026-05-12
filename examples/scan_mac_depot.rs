use dmm_parser::binary::papgt::PackGroupTreeMeta;
use dmm_parser::binary::pamt::PackMeta;
use std::path::Path;
use std::collections::HashMap;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\content\app_3321460\depot_3321462\CrimsonDesert_Steam.app\Contents\Resources\packages";
const PAPGT_PATH: &str = r"C:\Program Files (x86)\Steam\steamapps\content\app_3321460\depot_3321462\CrimsonDesert_Steam.app\Contents\Resources\packages\meta\0.papgt";

fn main() {
    let game_dir = Path::new(GAME_DIR);
    let papgt_data = std::fs::read(PAPGT_PATH).expect("read PAPGT");
    let papgt = PackGroupTreeMeta::parse(&papgt_data).expect("parse PAPGT");
    let mut by_ext: HashMap<String, (usize, u64)> = HashMap::new(); // ext -> (count, total_size)
    for entry in &papgt.entries {
        let group_dir = game_dir.join(&entry.group_name);
        let pamt_path = group_dir.join("0.pamt");
        let pamt_data = match std::fs::read(&pamt_path) { Ok(d) => d, Err(_) => continue };
        let pamt = match PackMeta::parse(&pamt_data, None) { Ok(p) => p, Err(_) => continue };
        for dir in &pamt.directories {
            for f in &dir.files {
                let name = f.name.to_ascii_lowercase();
                let ext = name.rsplit('.').next().unwrap_or("").to_string();
                let e = by_ext.entry(ext).or_insert((0, 0));
                e.0 += 1;
                e.1 += f.file.uncompressed_size as u64;
            }
        }
    }
    let mut v: Vec<_> = by_ext.into_iter().collect();
    v.sort_by_key(|(_, (_, sz))| std::cmp::Reverse(*sz));
    println!("Mac 1.06.01 depot — files by extension:");
    println!("{:>10}  {:>10}  {:>15}  ext", "count", "MB", "bytes");
    for (ext, (count, size)) in &v {
        if *size < 1_000_000 { continue; }
        println!("{:>10}  {:>10.1}  {:>15}  .{}", count, *size as f64 / 1024.0 / 1024.0, size, ext);
    }
}
