//! Generic resolve-layer validator. For a given table, auto-builds the
//! resolution context from all dumped target tables (those named by
//! reference_fields / nested_reference_fields), annotates, and reports
//! companions per field + byte-roundtrip. Replaces the per-table
//! verify_*_resolve throwaways.
//!
//! Usage: cargo run --example resolve_validate -- <stem>   (e.g. skill, knowledgeinfo, npcinfo, storeinfo, characterinfo)
//! With no arg, validates all wired tables.

use dmm_parser::dispatch::{normalize_target_name, parse_table_to_json_shaped, serialize_table_from_json_shaped};
use dmm_parser::json_shape::JsonShape;
use dmm_parser::resolve::{annotate, extract_key_name_index, reference_fields, nested_reference_fields, NameIndex};
use dmm_parser::item_info::parse_iteminfo_to_json;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const DIRS: &[&str] = &[
    r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\live_full",
    r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser\fixtures_skill",
];

/// Build canonical-name -> pabgb path map by scanning the dump dirs.
fn build_file_map() -> HashMap<String, PathBuf> {
    let mut m = HashMap::new();
    for d in DIRS {
        let Ok(rd) = std::fs::read_dir(Path::new(d)) else { continue };
        for e in rd.filter_map(|e| e.ok()) {
            let p = e.path();
            if p.extension().map(|x| x == "pabgb").unwrap_or(false) {
                let stem = p.file_stem().unwrap().to_string_lossy().to_string();
                if let Some(c) = normalize_target_name(&stem).or_else(|| normalize_target_name(&format!("{}info", stem))) {
                    m.entry(c.to_string()).or_insert(p);
                }
            }
        }
    }
    m
}

fn parse_table(canon: &str, files: &HashMap<String, PathBuf>) -> Option<(Vec<Value>, Vec<u8>)> {
    let p = files.get(canon)?;
    let data = std::fs::read(p).ok()?;
    let h = p.with_extension("pabgh");
    let hd = std::fs::read(&h).ok();
    let items = parse_table_to_json_shaped(canon, &data, hd.as_deref(), JsonShape::from_str("").unwrap()).ok()?;
    Some((items, data))
}

/// Index for a target table: string_info uses `buffer`, item_info via the
/// legacy parser, everything else uses `string_key`.
fn build_index(canon: &str, files: &HashMap<String, PathBuf>) -> Option<HashMap<u32, String>> {
    if canon == "item_info" {
        // iteminfo normalizes to canonical "iteminfo" (legacy), not "item_info".
        let p = files.get("item_info").or_else(|| files.get("iteminfo"))?;
        let items = parse_iteminfo_to_json(&std::fs::read(p).ok()?).ok()?;
        return Some(extract_key_name_index(&items, "string_key"));
    }
    let (items, _) = parse_table(canon, files)?;
    let name_field = if canon == "string_info" { "buffer" } else { "string_key" };
    Some(extract_key_name_index(&items, name_field))
}

fn validate(canon: &str, files: &HashMap<String, PathBuf>) {
    let Some((mut items, orig)) = parse_table(canon, files) else {
        println!("{}: SKIP (no dump)", canon); return;
    };
    // Collect targets from the resolve config.
    let mut targets: Vec<&str> = reference_fields(canon).iter().map(|r| r.target).collect();
    targets.extend(nested_reference_fields(canon).iter().map(|n| n.target));
    targets.sort(); targets.dedup();

    let mut idx: NameIndex = HashMap::new();
    for t in &targets {
        if let Some(m) = build_index(t, files) { idx.insert(t.to_string(), m); }
    }
    let added = annotate(canon, &mut items, &idx);
    let re = serialize_table_from_json_shaped(canon, &items, JsonShape::from_str("").unwrap()).unwrap();
    println!("{:<18} recs={:<6} targets={:<2} companions={:<6} roundtrip={}",
        canon, items.len(), targets.len(), added, re == orig);
    // Per-field: resolved count vs how many records carry a nonzero/non-empty
    // value. 0 resolved but >0 present ⇒ WRONG target. 0 present ⇒ all-zero
    // data (mapping fine, just nothing to resolve).
    let count_key = |k: &str| items.iter().filter(|it| it.as_object().map(|o| o.contains_key(k)).unwrap_or(false)).count();
    for rf in reference_fields(canon) {
        let key = if rf.is_array { format!("{}_names", rf.field) } else { format!("{}_name", rf.field) };
        let resolved = count_key(&key);
        let present = items.iter().filter(|it| {
            let Some(v) = it.get(rf.field) else { return false };
            if rf.is_array { v.as_array().map(|a| a.iter().any(|e| e.as_u64() != Some(0))).unwrap_or(false) }
            else { v.as_u64().map(|n| n != 0).unwrap_or(false) }
        }).count();
        let flag = if resolved == 0 && present > 0 { " !! WRONG TARGET" } else { "" };
        println!("      {} -> {}: {}/{} present{}", rf.field, rf.target, resolved, present, flag);
    }
}

fn main() {
    let files = build_file_map();
    let wired = ["skill_info", "knowledge_info", "npc_info", "store_info", "character_info", "mission_info", "quest_info", "buff_info"];
    match std::env::args().nth(1) {
        Some(stem) => {
            let canon = normalize_target_name(&stem).or_else(|| normalize_target_name(&format!("{}info", stem)))
                .unwrap_or_else(|| panic!("unknown table {:?}", stem));
            validate(canon, &files);
        }
        None => for t in wired { validate(t, &files); }
    }
}
