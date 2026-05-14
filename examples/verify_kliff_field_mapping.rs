// Empirical test: load vanilla characterinfo's Kliff entry, modify a single
// field at a time, serialize, and report which byte offset within the file
// changed. Cross-references the byte-patch label offsets from
// Kliff_Damiane_Runtimepackages.json to determine the TRUE field mapping.

use std::fs;
use std::path::Path;
use dmm_parser::{parse_table_to_json, serialize_table_from_json};
use dmm_parser::binary::pamt::PackMeta;

fn extract_vanilla_file(game: &Path, target: &str) -> Option<Vec<u8>> {
    // Search every 0.pamt for `target` basename; return decrypted/decompressed bytes.
    let mut groups: Vec<String> = fs::read_dir(game).ok()?.filter_map(|e| e.ok())
        .filter(|e| { let n=e.file_name().to_string_lossy().to_string(); n.len()==4 && n.chars().all(|c| c.is_ascii_digit()) && e.path().is_dir() })
        .map(|e| e.file_name().to_string_lossy().to_string()).collect();
    groups.sort();
    for g in &groups {
        let pamt_path = game.join(g).join("0.pamt");
        if !pamt_path.exists() { continue; }
        let data = fs::read(&pamt_path).ok()?;
        let pamt = PackMeta::parse(&data, None).ok()?;
        for dir in &pamt.directories {
            for f in &dir.files {
                if f.name.eq_ignore_ascii_case(target) {
                    let paz_path = game.join(g).join(format!("{}.paz", f.file.chunk_id));
                    let paz = fs::read(&paz_path).ok()?;
                    let off = f.file.chunk_offset as usize;
                    let csz = f.file.compressed_size as usize;
                    if off + csz > paz.len() { return None; }
                    let raw = &paz[off..off+csz];
                    // For simplicity assume flags=0x0001 (raw stored, no PAZ-level compression).
                    // characterinfo.pabgb / .pabgh in vanilla are large; let's check the flag.
                    // flags & 0x0F: 0=none, 1=raw-stored-partial, 2=LZ4, 3=Zlib
                    let compression = f.file.flags & 0x0F;
                    match compression {
                        0 | 1 => return Some(raw.to_vec()),
                        2 => {
                            let dec = f.file.uncompressed_size as usize;
                            match lz4_flex::block::decompress(raw, dec) {
                                Ok(d) => return Some(d),
                                Err(e) => { eprintln!("LZ4 decode '{}': {}", target, e); return None; }
                            }
                        }
                        _ => { eprintln!("  '{}' compression={} unsupported", target, compression); return None; }
                    }
                }
            }
        }
    }
    None
}

fn main() {
    let game = Path::new("C:/Program Files (x86)/Steam/steamapps/common/Crimson Desert");
    println!("Extracting vanilla characterinfo.pabgb + .pabgh from game ...");
    let pabgb_bytes = match extract_vanilla_file(game, "characterinfo.pabgb") {
        Some(b) => b,
        None => { eprintln!("Could not extract characterinfo.pabgb"); return; }
    };
    let pabgh_bytes = match extract_vanilla_file(game, "characterinfo.pabgh") {
        Some(b) => b,
        None => { eprintln!("Could not extract characterinfo.pabgh"); return; }
    };
    println!("pabgb size: {}", pabgb_bytes.len());
    println!("pabgh size: {}", pabgh_bytes.len());

    let items: Vec<serde_json::Value> =
        parse_table_to_json("character_info", &pabgb_bytes, Some(&pabgh_bytes))
            .expect("parse");
    println!("Parsed {} entries", items.len());

    // Print top-level keys of first entry
    if let Some(first) = items.first().and_then(|v| v.as_object()) {
        let keys: Vec<&String> = first.keys().collect();
        println!("\nTop-level fields ({}): {:?}\n", keys.len(), keys);
    }

    // Locate Kliff entry. The `name` field is a LocalizableString — look at .default
    // (which may be the literal string OR a numeric hash). Try both.
    let mut kliff_idx: Option<usize> = None;
    for (i, e) in items.iter().enumerate() {
        let name_default = e.get("name").and_then(|n| n.get("default"));
        let name_str = name_default.and_then(|v| v.as_str()).unwrap_or("");
        let name_path = e.get("name_path").and_then(|n| n.get("default")).and_then(|v| v.as_str()).unwrap_or("");
        let label_a = e.get("label_a").and_then(|n| n.get("default")).and_then(|v| v.as_str()).unwrap_or("");
        let string_key = e.get("string_key").and_then(|v| v.as_str()).unwrap_or("");
        if i < 5 {
            println!("  entry[{}] name.default={:?} name_path.default={:?} label_a.default={:?} string_key={:?}",
                i, name_str, name_path, label_a, string_key);
        }
        if name_str == "Kliff" || name_path == "Kliff" || label_a == "Kliff" || string_key == "Kliff" {
            kliff_idx = Some(i);
            let key = e.get("key").and_then(|k| k.as_u64()).unwrap_or(0);
            let appn = e.get("appearance_name").and_then(|v| v.as_u64()).unwrap_or(0);
            let skn = e.get("skeleton_name").and_then(|v| v.as_u64()).unwrap_or(0);
            let cgpd = e.get("character_game_play_data_name").and_then(|v| v.as_u64()).unwrap_or(0);
            let cpp = e.get("character_prefab_path").and_then(|v| v.as_u64()).unwrap_or(0);
            let uc_g = e.get("upper_chart").and_then(|c| c.get("group_lookup")).and_then(|v| v.as_u64()).unwrap_or(0);
            let lc_g = e.get("lower_chart").and_then(|c| c.get("group_lookup")).and_then(|v| v.as_u64()).unwrap_or(0);
            println!("Kliff @ idx={} key={} appearance_name={} skeleton_name={} char_gp_data={} char_prefab_path={} upper_chart.group={} lower_chart.group={}",
                i, key, appn, skn, cgpd, cpp, uc_g, lc_g);
            break;
        }
    }
    let kliff_idx = match kliff_idx {
        Some(i) => i,
        None => {
            eprintln!("Kliff entry not found");
            // Print first 5 entry names for diag
            for (i, e) in items.iter().take(10).enumerate() {
                eprintln!("  [{}] {:?}", i, e.get("name"));
            }
            return;
        }
    };
    println!("Kliff entry index: {}", kliff_idx);

    // Re-serialize unchanged to get a baseline
    let baseline = serialize_table_from_json("character_info", &items).expect("baseline serialize");
    println!("Baseline reserialized size: {} (vs original {})", baseline.len(), pabgb_bytes.len());
    if baseline != pabgb_bytes {
        println!("WARNING: baseline doesn't roundtrip exactly (parser drift). Continuing anyway.");
    }

    // The byte-patch's first Kliff change is at FILE offset 185:
    //   _upperActionChartPackageGroupName: 8914B28B → F20E5469 (Player_PHW = 1767116530)
    // We probe each candidate field by setting Kliff.<field> = 1767116530 and seeing
    // which byte offset in the serialized output changes to F2 0E 54 69.
    let target_value = serde_json::json!(1767116530u32);
    let expected_le: [u8; 4] = [0xF2, 0x0E, 0x54, 0x69];

    let candidates = [
        // Top-level fields per dmm-parser CharacterInfo struct
        "character_game_play_data_name",
        "appearance_name",
        "character_prefab_path",
        "skeleton_name",
        "lookup_22",
        "lookup_23",
        "lookup_24",
        "lookup_25",
        "raw_a",
        "lookup_27",
        "lookup_28",
        "lookup_29",
        // Nested chart fields
        "upper_chart.group_lookup",
        "lower_chart.group_lookup",
        // Flag/gender candidates (single-byte u8 fields)
        "flag_a",
        "flag_b",
        "flag_c",
        "flag_d",
        "flag_e",
    ];

    println!("\n--- Probing each candidate. Target: Kliff.X = 1767116530 (bytes F2 0E 54 69) ---");
    println!("File offset 185 is what _upperActionChartPackageGroupName byte-patch targets.\n");

    for field in &candidates {
        // u8 fields take 2, others take 1767116530
        let v: serde_json::Value = if field.starts_with("flag_") || field == &"none_player_sub_type" || field == &"mercenary_cool_time_type" {
            serde_json::json!(2u8)
        } else {
            target_value.clone()
        };
        let mut mutated = items.clone();
        let mut applied = false;
        if let Some(entry) = mutated.get_mut(kliff_idx).and_then(|e| e.as_object_mut()) {
            if let Some((parent, child)) = field.split_once('.') {
                if let Some(p) = entry.get_mut(parent).and_then(|p| p.as_object_mut()) {
                    p.insert(child.into(), v.clone());
                    applied = true;
                }
            } else if entry.contains_key(*field) {
                entry.insert((*field).into(), v.clone());
                applied = true;
            }
        }
        if !applied {
            println!("  {:36} ❌ field not found", field);
            continue;
        }

        let new_bytes = match serialize_table_from_json("character_info", &mutated) {
            Ok(b) => b,
            Err(e) => {
                println!("  {:36} ❌ serialize failed: {}", field, e);
                continue;
            }
        };

        // Find ALL byte offsets where the new differs from baseline
        let diffs: Vec<usize> = baseline.iter().zip(new_bytes.iter()).enumerate()
            .filter_map(|(i, (x, y))| if x != y { Some(i) } else { None })
            .collect();

        // Does the diff include offset 185..189 with the target pattern?
        let hit_185 = new_bytes.len() >= 189 && &new_bytes[185..189] == expected_le;
        // Also report which contiguous diff range we see
        let first = diffs.first().copied();
        let last = diffs.last().copied();
        println!("  {:36} diffs={:3} range=[{:?}..{:?}] hit@185={}",
            field, diffs.len(), first, last, hit_185);
        if diffs.len() < 12 {
            print!("      offsets:");
            for d in &diffs { print!(" {}", d); }
            println!();
        }
    }
}
