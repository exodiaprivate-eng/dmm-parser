//! Round-trip every dispatch-supported pabgb against the LIVE game install.
//! Pure diagnostic: no side effects, no file writes. Reports PASS / FAIL per
//! table so we know which parsers still byte-clean against the current patch.

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::papgt::PackGroupTreeMeta;
use dmm_parser::binary::paz;
use dmm_parser::dispatch::{
    parse_table_to_json, serialize_table_from_json, supported_tables,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";
const PAPGT_PATH: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert\meta\0.papgt";

struct GroupCtx {
    group_dir: PathBuf,
    encrypt_info: [u8; 3],
    pamt: PackMeta,
}

/// Map dispatch name to candidate pabgb stems. Game's filename convention is
/// inconsistent: most tables compress underscores out, some drop the `_info`
/// suffix, a few are renamed entirely. We try in order and take the first hit.
fn pabgb_stem_candidates(name: &str) -> Vec<String> {
    // Explicit aliases for tables whose pabgb filename doesn't follow the
    // mechanical strip-underscores rule. Determined by inspecting the
    // actual file names in the live PAZ archives.
    let alias: Option<&str> = match name {
        "field_revive_info"          => Some("reviepointinfo"),
        "game_level_info"            => Some("levelinfo"),
        "key_map_setting_list_info"  => Some("keymap"),
        "platform_entitlement_info"  => Some("entitlementinfo"),
        _ => None,
    };
    if let Some(a) = alias {
        return vec![a.to_string()];
    }

    let stripped = name.replace('_', "");
    // Try with underscores compressed first (e.g. action_point_info → actionpointinfo).
    // Then try with `_info` suffix dropped (e.g. faction_info → faction).
    let without_info = stripped.strip_suffix("info").unwrap_or(&stripped).to_string();
    let mut out = vec![stripped];
    if !out.contains(&without_info) {
        out.push(without_info);
    }
    out
}

fn build_index() -> (Vec<GroupCtx>, BTreeMap<String, (usize, usize, usize)>) {
    let papgt_data = std::fs::read(PAPGT_PATH).expect("read PAPGT");
    let papgt = PackGroupTreeMeta::parse(&papgt_data).expect("parse PAPGT");
    let game_dir = Path::new(GAME_DIR);

    let mut groups: Vec<GroupCtx> = Vec::new();
    let mut index: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();

    for entry in &papgt.entries {
        let group_dir = game_dir.join(&entry.group_name);
        let pamt_path = group_dir.join("0.pamt");
        let pamt_data = match std::fs::read(&pamt_path) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let pamt = match PackMeta::parse(&pamt_data, None) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let encrypt_info = pamt.header.encrypt_info.encrypt_info;
        let g_idx = groups.len();
        for (d_idx, dir) in pamt.directories.iter().enumerate() {
            for (f_idx, f) in dir.files.iter().enumerate() {
                let lower = f.name.to_ascii_lowercase();
                if lower.ends_with(".pabgb") || lower.ends_with(".pabgh") {
                    // Last writer wins (later PAPGT entries shadow earlier).
                    index.insert(lower, (g_idx, d_idx, f_idx));
                }
            }
        }
        groups.push(GroupCtx {
            group_dir,
            encrypt_info,
            pamt,
        });
    }
    (groups, index)
}

fn extract(groups: &[GroupCtx], loc: (usize, usize, usize)) -> Option<Vec<u8>> {
    let (g, d, f) = loc;
    let ctx = &groups[g];
    let dir = &ctx.pamt.directories[d];
    let file = &dir.files[f];
    paz::extract_file(&ctx.group_dir, file, &dir.path, &ctx.encrypt_info).ok()
}

fn main() {
    println!("=== Round-trip matrix vs LIVE game install ===");
    println!("Source: {}\n", GAME_DIR);

    let (groups, index) = build_index();
    println!(
        "Indexed {} pabgb/pabgh entries across {} groups\n",
        index.len(),
        groups.len()
    );

    let mut pass: usize = 0;
    let mut fail_parse: Vec<&str> = Vec::new();
    let mut fail_serialize: Vec<&str> = Vec::new();
    let mut fail_byteclean: Vec<(&str, isize)> = Vec::new();
    let mut missing: Vec<&str> = Vec::new();

    for &table in supported_tables() {
        if matches!(table, "paloc" | "paloc.pamt" | "localizationstring") {
            continue;
        }
        let candidates = pabgb_stem_candidates(table);
        let mut found: Option<(String, (usize, usize, usize))> = None;
        for stem in &candidates {
            let key = format!("{}.pabgb", stem);
            if let Some(&loc) = index.get(&key) {
                found = Some((stem.clone(), loc));
                break;
            }
        }
        let Some((stem, pabgb_loc)) = found else {
            println!("MISS  {} (tried {:?})", table, candidates);
            missing.push(table);
            continue;
        };
        let Some(pabgb) = extract(&groups, pabgb_loc) else {
            println!("EX!   {} extract fail", table);
            continue;
        };
        let pabgh_name = format!("{}.pabgh", stem);
        let pabgh = index.get(&pabgh_name).copied().and_then(|loc| extract(&groups, loc));

        let parsed = match parse_table_to_json(table, &pabgb, pabgh.as_deref()) {
            Ok(v) => v,
            Err(e) => {
                println!("PARSE {} ({} bytes): {}", table, pabgb.len(), e);
                fail_parse.push(table);
                continue;
            }
        };
        let out = match serialize_table_from_json(table, &parsed) {
            Ok(b) => b,
            Err(e) => {
                println!("SER   {} ({} entries): {}", table, parsed.len(), e);
                fail_serialize.push(table);
                continue;
            }
        };
        if out == pabgb {
            println!("PASS  {} ({} entries, {} bytes)", table, parsed.len(), pabgb.len());
            pass += 1;
        } else {
            let diff_at = out
                .iter()
                .zip(pabgb.iter())
                .position(|(a, b)| a != b)
                .unwrap_or(out.len().min(pabgb.len()));
            let delta = out.len() as isize - pabgb.len() as isize;
            println!(
                "DIFF  {} byte 0x{:X} (out={}, vanilla={}, Δ {:+})",
                table,
                diff_at,
                out.len(),
                pabgb.len(),
                delta
            );
            fail_byteclean.push((table, delta));
        }
    }

    println!("\n=== Summary ===");
    println!("PASS clean round-trip:  {}", pass);
    println!("PARSE failed:           {}", fail_parse.len());
    if !fail_parse.is_empty() {
        for t in &fail_parse {
            println!("    - {}", t);
        }
    }
    println!("SERIALIZE failed:       {}", fail_serialize.len());
    if !fail_serialize.is_empty() {
        for t in &fail_serialize {
            println!("    - {}", t);
        }
    }
    println!("Byte mismatch:          {}", fail_byteclean.len());
    if !fail_byteclean.is_empty() {
        for (t, d) in &fail_byteclean {
            println!("    - {} (Δ {:+})", t, d);
        }
    }
    println!("Missing from PAZ:       {}", missing.len());
    if !missing.is_empty() {
        for t in &missing {
            println!("    - {}", t);
        }
    }
}
