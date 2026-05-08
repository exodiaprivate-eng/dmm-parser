use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::papgt::PackGroupTreeMeta;
use std::path::Path;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";
const PAPGT_PATH: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert\meta\0.papgt";

fn main() {
    let papgt_data = std::fs::read(PAPGT_PATH).unwrap();
    let papgt = PackGroupTreeMeta::parse(&papgt_data).unwrap();
    let game_dir = Path::new(GAME_DIR);

    let mut count = 0usize;
    let mut samples: Vec<String> = Vec::new();
    for entry in &papgt.entries {
        let group_dir = game_dir.join(&entry.group_name);
        let pamt_path = group_dir.join("0.pamt");
        let Ok(data) = std::fs::read(&pamt_path) else { continue };
        let Ok(pamt) = PackMeta::parse(&data, None) else { continue };
        for dir in &pamt.directories {
            if dir.path.eq_ignore_ascii_case("sound/windows/english(us)") {
                for f in &dir.files {
                    count += 1;
                    if samples.len() < 15 {
                        samples.push(format!("[{}] {}", entry.group_name, f.name));
                    }
                }
            }
        }
    }
    println!("Game has {} files in sound/windows/english(us)/", count);
    println!("\nFirst 15:");
    for s in &samples { println!("  {}", s); }
}
