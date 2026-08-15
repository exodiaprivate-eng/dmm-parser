// TEMP — dump native equip_buffs of vanilla rings to confirm which buffs are
// "ring-legal" (safe to use on a buff-carrier ring without the infinite-load
// bug). Delete after.
use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::papgt::PackGroupTreeMeta;
use dmm_parser::binary::paz;
use dmm_parser::dispatch::parse_table_to_json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn main() {
    let game = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "D:/SteamLibrary/steamapps/common/Crimson Desert".to_string());
    let game_dir = Path::new(&game);
    let papgt =
        PackGroupTreeMeta::parse(&fs::read(game_dir.join("meta").join("0.papgt")).unwrap()).unwrap();

    let mut body: Option<Vec<u8>> = None;
    for entry in &papgt.entries {
        if body.is_some() {
            break;
        }
        let group_dir = game_dir.join(&entry.group_name);
        let Ok(pamt_data) = fs::read(group_dir.join("0.pamt")) else {
            continue;
        };
        let Ok(pamt) = PackMeta::parse(&pamt_data, None) else {
            continue;
        };
        let ei = pamt.header.encrypt_info.encrypt_info;
        for dir in &pamt.directories {
            for f in &dir.files {
                if f.name.eq_ignore_ascii_case("iteminfo.pabgb") {
                    if let Ok(b) = paz::extract_file(&group_dir, f, &dir.path, &ei) {
                        body = Some(b);
                    }
                }
            }
        }
    }
    let body = body.expect("iteminfo.pabgb not found");
    let recs = parse_table_to_json("iteminfo", &body, None).unwrap();
    println!("iteminfo records: {}", recs.len());

    // Ring keys of interest: legendaries + commons.
    let rings: &[(u64, &str)] = &[
        (8501, "White Horn's Ring"),
        (8502, "Ring of Lightning (Legendary_Titan)"),
        (8505, "Ogre's Ring (Legendary_Ogre)"),
        (8506, "Witch's Ring"),
        (620002, "Ancient Shell Ring (Pattern_Silver)"),
        (620004, "Crude Blue Ring (Pattern_Copper) <-- ONE_ring base"),
        (620010, "Rough Bluestone Ring (Red_Ring)"),
        (620011, "Finely Crafted Gold Ring (Detail_Gold)"),
        (795052, "Worn Ring (Old_Ring)"),
    ];
    let want: std::collections::HashSet<u64> = rings.iter().map(|(k, _)| *k).collect();

    let mut by_key: HashMap<u64, &serde_json::Value> = HashMap::new();
    for r in &recs {
        if let Some(k) = r.get("key").and_then(|v| v.as_u64()) {
            if want.contains(&k) {
                by_key.insert(k, r);
            }
        }
    }

    // Collect the union of all buff ids used across these rings, to resolve names.
    let mut all_buff_ids: std::collections::BTreeSet<u64> = Default::default();

    for (k, label) in rings {
        println!("\n=== {k}  {label} ===");
        let Some(rec) = by_key.get(k) else {
            println!("  (not present in iteminfo)");
            continue;
        };
        let Some(list) = rec.get("enchant_data_list").and_then(|v| v.as_array()) else {
            println!("  (no enchant_data_list)");
            continue;
        };
        // Rings usually repeat the same buff set per enchant level; show level 0
        // and note if higher levels differ.
        let mut seen_sets: Vec<String> = Vec::new();
        for (i, ed) in list.iter().enumerate() {
            let buffs = ed.get("equip_buffs").and_then(|v| v.as_array());
            let repr = match buffs {
                Some(arr) => {
                    let mut parts = Vec::new();
                    for b in arr {
                        let id = b.get("buff").and_then(|v| v.as_u64()).unwrap_or(0);
                        let lvl = b.get("level").and_then(|v| v.as_i64()).unwrap_or(0);
                        all_buff_ids.insert(id);
                        parts.push(format!("{id}@{lvl}"));
                    }
                    parts.join(", ")
                }
                None => "(none)".to_string(),
            };
            if i == 0 || !seen_sets.contains(&repr) {
                println!("  enchant[{i}].equip_buffs: [{repr}]");
                seen_sets.push(repr);
            }
        }
    }

    // Resolve buff ids -> names via crimson_data.db if present.
    let db = "C:/Users/justi/Desktop/Project/CRIMSON-DESERT-SAVE-EDITOR-AND-GAME-MODS/CrimsonGameMods/crimson_data.db";
    println!("\n=== buff id -> name (union across the rings above) ===");
    println!("ids: {:?}", all_buff_ids);
    println!("(resolve with the DB separately)");
    let _ = db;
}
