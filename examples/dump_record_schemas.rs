//! Dump first-record JSON schema keys for the 6 tables that the Merged Stack
//! mod failed to apply on. Used to diff workbench's expected field names
//! against dmm-parser's current schema and produce an alias map.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::papgt::PackGroupTreeMeta;
use dmm_parser::binary::paz;
use dmm_parser::dispatch::parse_table_to_json;
use serde_json::Value;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";
const PAPGT_PATH: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert\meta\0.papgt";

const TARGETS: &[(&str, &str)] = &[
    ("character_info",                     "characterinfo.pabgb"),
    ("reserve_slot_info",                  "reserveslot.pabgb"),
    ("equip_slot_info",                    "equipslotinfo.pabgb"),
    ("skill_info",                         "skill.pabgb"),
    ("spawning_pool_auto_spawn_info",      "spawningpoolautospawninfo.pabgb"),
    ("terrain_region_auto_spawn_info",     "terrainregionautospawninfo.pabgb"),
];

struct GroupCtx {
    group_dir: PathBuf,
    encrypt_info: [u8; 3],
    pamt: PackMeta,
}

fn build_index() -> (Vec<GroupCtx>, BTreeMap<String, (usize, usize, usize)>) {
    let papgt_data = std::fs::read(PAPGT_PATH).unwrap();
    let papgt = PackGroupTreeMeta::parse(&papgt_data).unwrap();
    let game_dir = Path::new(GAME_DIR);
    let mut groups: Vec<GroupCtx> = Vec::new();
    let mut index: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();
    for entry in &papgt.entries {
        let group_dir = game_dir.join(&entry.group_name);
        let pamt_path = group_dir.join("0.pamt");
        let Ok(pamt_data) = std::fs::read(&pamt_path) else { continue };
        let Ok(pamt) = PackMeta::parse(&pamt_data, None) else { continue };
        let encrypt_info = pamt.header.encrypt_info.encrypt_info;
        let g_idx = groups.len();
        for (d_idx, dir) in pamt.directories.iter().enumerate() {
            for (f_idx, f) in dir.files.iter().enumerate() {
                let lower = f.name.to_ascii_lowercase();
                if lower.ends_with(".pabgb") || lower.ends_with(".pabgh") {
                    index.insert(lower, (g_idx, d_idx, f_idx));
                }
            }
        }
        groups.push(GroupCtx { group_dir, encrypt_info, pamt });
    }
    (groups, index)
}

fn extract(g: &[GroupCtx], loc: (usize, usize, usize)) -> Option<Vec<u8>> {
    let (gi, di, fi) = loc;
    paz::extract_file(&g[gi].group_dir, &g[gi].pamt.directories[di].files[fi],
                      &g[gi].pamt.directories[di].path, &g[gi].encrypt_info).ok()
}

fn print_keys(prefix: &str, v: &Value, depth: usize) {
    if depth > 4 { return; }
    let indent = "  ".repeat(depth);
    match v {
        Value::Object(map) => {
            for (k, vv) in map {
                let display_key = if prefix.is_empty() { k.clone() } else { format!("{}.{}", prefix, k) };
                let kind = match vv {
                    Value::Null => "null",
                    Value::Bool(_) => "bool",
                    Value::Number(_) => "num",
                    Value::String(_) => "str",
                    Value::Array(arr) => {
                        println!("{}{}: array[{}]", indent, display_key, arr.len());
                        if let Some(first) = arr.first() {
                            print_keys(&format!("{}[0]", display_key), first, depth + 1);
                        }
                        continue;
                    },
                    Value::Object(_) => {
                        println!("{}{}: object", indent, display_key);
                        print_keys(&display_key, vv, depth + 1);
                        continue;
                    }
                };
                println!("{}{}: {}", indent, display_key, kind);
            }
        }
        _ => {}
    }
}

fn main() {
    let (groups, index) = build_index();
    for (dispatch, fname) in TARGETS {
        println!("\n========== {} ({}) ==========", dispatch, fname);
        let lower = fname.to_lowercase();
        let Some(&loc) = index.get(&lower) else { println!("MISS"); continue; };
        let Some(pabgb) = extract(&groups, loc) else { println!("EXTRACT FAIL"); continue; };
        let pabgh_name = lower.replace(".pabgb", ".pabgh");
        let pabgh = index.get(&pabgh_name).copied().and_then(|l| extract(&groups, l));
        match parse_table_to_json(dispatch, &pabgb, pabgh.as_deref()) {
            Ok(records) => {
                println!("{} records parsed; sampling first record:", records.len());
                if let Some(r) = records.first() {
                    print_keys("", r, 0);
                }
            }
            Err(e) => println!("PARSE FAIL: {}", e),
        }
    }
}
