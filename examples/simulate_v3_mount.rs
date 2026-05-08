//! Simulate mounting a v3 / v3.1 multi-target mod against the LIVE game install.
//! Pure dry-run: no overlay written, no game files touched. Reports per-target
//! intents applied / unresolved / type mismatch and post-apply byte delta.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use dmm_parser::binary::pamt::PackMeta;
use dmm_parser::binary::papgt::PackGroupTreeMeta;
use dmm_parser::binary::paz;
use dmm_parser::dispatch::{parse_table_to_json, serialize_table_from_json, supported_tables};
use dmm_parser::item_info::{parse_iteminfo_to_json, serialize_iteminfo_from_json};
use dmm_parser::tables::blob_runtime::{
    parse_blob_table_to_json_with_pabgh, serialize_blob_table_from_json,
};

use serde_json::Value;

const GAME_DIR: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert";
const PAPGT_PATH: &str = r"C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert\meta\0.papgt";
const MOD_PATH: &str = r"C:\Users\corin\Desktop\DMM 1.3.3 Release\mods\Merged_Stack.field.json";

struct GroupCtx {
    group_dir: PathBuf,
    encrypt_info: [u8; 3],
    pamt: PackMeta,
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
        let pamt_data = match std::fs::read(&pamt_path) { Ok(d) => d, Err(_) => continue };
        let pamt = match PackMeta::parse(&pamt_data, None) { Ok(p) => p, Err(_) => continue };
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

fn extract(groups: &[GroupCtx], loc: (usize, usize, usize)) -> Option<Vec<u8>> {
    let (g, d, f) = loc;
    paz::extract_file(&groups[g].group_dir, &groups[g].pamt.directories[d].files[f],
                      &groups[g].pamt.directories[d].path, &groups[g].encrypt_info).ok()
}

/// Map a pabgb filename (as written in mod JSON) to a dispatch table name.
fn dispatch_name_from_filename(fname: &str) -> Option<&'static str> {
    let stem = fname.strip_suffix(".pabgb").unwrap_or(fname).to_ascii_lowercase();
    let alias = match stem.as_str() {
        "reviepointinfo" => Some("field_revive_info"),
        "levelinfo"      => Some("game_level_info"),
        "keymap"         => Some("key_map_setting_list_info"),
        "entitlementinfo" => Some("platform_entitlement_info"),
        _ => None,
    };
    if let Some(a) = alias { return Some(a); }
    for &name in supported_tables() {
        let candidate1 = name.replace('_', "");
        let candidate2 = candidate1.strip_suffix("info").unwrap_or(&candidate1).to_string();
        if stem == candidate1 || stem == candidate2 { return Some(name); }
    }
    None
}

#[derive(Default, Debug)]
struct ApplyReport {
    applied: usize,
    unresolved_entry: usize,
    unresolved_field: usize,
    type_mismatch: usize,
    unsupported_op: usize,
}

/// Tokenize a field path like `spawn_list[0].spline_list[2].block` into
/// alternating Field/Index steps. Handles `name`, `name.next`, `name[N]`,
/// `name[N].next`, `name[N][M]`, etc.
enum PathStep<'a> { Field(&'a str), Index(usize) }

fn tokenize_path(path: &str) -> Result<Vec<PathStep<'_>>, &'static str> {
    let mut out = Vec::new();
    let bytes = path.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'.' { i += 1; continue; }
        if bytes[i] == b'[' {
            let close = bytes[i + 1..].iter().position(|&b| b == b']').ok_or("bad_path")? + i + 1;
            let n: usize = std::str::from_utf8(&bytes[i + 1..close]).map_err(|_| "bad_path")?
                .parse().map_err(|_| "bad_path")?;
            out.push(PathStep::Index(n));
            i = close + 1;
        } else {
            let start = i;
            while i < bytes.len() && bytes[i] != b'.' && bytes[i] != b'[' { i += 1; }
            out.push(PathStep::Field(std::str::from_utf8(&bytes[start..i]).map_err(|_| "bad_path")?));
        }
    }
    Ok(out)
}

fn apply_intent_to_record(record: &mut Value, field_path: &str, new_val: &Value) -> Result<(), &'static str> {
    let steps = tokenize_path(field_path)?;
    if steps.is_empty() { return Err("unresolved_field"); }
    let mut cur: &mut Value = record;
    for (i, step) in steps.iter().enumerate() {
        let is_last = i == steps.len() - 1;
        match (step, &mut *cur) {
            (PathStep::Field(name), Value::Object(map)) => {
                if is_last {
                    if let Some(existing) = map.get(*name) {
                        let same_kind = matches!(
                            (existing, new_val),
                            (Value::Number(_), Value::Number(_))
                                | (Value::String(_), Value::String(_))
                                | (Value::Bool(_), Value::Bool(_))
                                | (Value::Array(_), Value::Array(_))
                                | (Value::Object(_), Value::Object(_))
                                | (Value::Null, _) | (_, Value::Null)
                        );
                        if !same_kind { return Err("type_mismatch"); }
                    } else {
                        return Err("unresolved_field");
                    }
                    map.insert(name.to_string(), new_val.clone());
                    return Ok(());
                }
                cur = map.get_mut(*name).ok_or("unresolved_field")?;
            }
            (PathStep::Index(idx), Value::Array(arr)) => {
                if *idx >= arr.len() { return Err("unresolved_field"); }
                if is_last {
                    let same_kind = matches!(
                        (&arr[*idx], new_val),
                        (Value::Number(_), Value::Number(_))
                            | (Value::String(_), Value::String(_))
                            | (Value::Bool(_), Value::Bool(_))
                            | (Value::Array(_), Value::Array(_))
                            | (Value::Object(_), Value::Object(_))
                            | (Value::Null, _) | (_, Value::Null)
                    );
                    if !same_kind { return Err("type_mismatch"); }
                    arr[*idx] = new_val.clone();
                    return Ok(());
                }
                cur = &mut arr[*idx];
            }
            _ => return Err("unresolved_field"),
        }
    }
    Err("unresolved_field")
}

fn index_records(records: &[Value]) -> (HashMap<String, usize>, HashMap<i64, usize>) {
    let mut by_name = HashMap::new();
    let mut by_key = HashMap::new();
    for (i, r) in records.iter().enumerate() {
        if let Some(obj) = r.as_object() {
            if let Some(s) = obj.get("string_key").and_then(|v| v.as_str()) {
                if !s.is_empty() { by_name.insert(s.to_string(), i); }
            }
            if let Some(k) = obj.get("key").and_then(|v| v.as_i64()) {
                by_key.insert(k, i);
            }
        }
    }
    (by_name, by_key)
}

fn apply_intents(records: &mut [Value], intents: &[Value]) -> ApplyReport {
    let (by_name, by_key) = index_records(records);
    let mut report = ApplyReport::default();
    for intent in intents {
        let entry = intent.get("entry").and_then(|v| v.as_str()).unwrap_or("");
        let key = intent.get("key").and_then(|v| v.as_i64());
        let op = intent.get("op").and_then(|v| v.as_str()).unwrap_or("set");
        let field = intent.get("field").and_then(|v| v.as_str());
        let new_val = intent.get("new");

        if op != "set" { report.unsupported_op += 1; continue; }
        let Some(field_path) = field else { report.unresolved_field += 1; continue; };
        let Some(nv) = new_val else { report.unresolved_field += 1; continue; };

        let idx = (!entry.is_empty()).then(|| by_name.get(entry).copied()).flatten()
            .or_else(|| key.and_then(|k| by_key.get(&k).copied()));
        let Some(idx) = idx else { report.unresolved_entry += 1; continue; };

        match apply_intent_to_record(&mut records[idx], field_path, nv) {
            Ok(()) => report.applied += 1,
            Err("unresolved_field") => report.unresolved_field += 1,
            Err("type_mismatch") => report.type_mismatch += 1,
            Err(_) => report.unresolved_field += 1,
        }
    }
    report
}

fn main() {
    println!("=== v3.1 mount simulation ===");
    println!("Mod:  {}", MOD_PATH);
    println!("Game: {}\n", GAME_DIR);

    let raw = std::fs::read_to_string(MOD_PATH).expect("read mod");
    let mod_v: Value = serde_json::from_str(&raw).expect("parse mod JSON");
    let format = mod_v.get("format").and_then(|v| v.as_u64()).unwrap_or(0);
    let format_minor = mod_v.get("format_minor").and_then(|v| v.as_u64()).unwrap_or(0);
    let title = mod_v.get("modinfo").and_then(|v| v.get("title")).and_then(|v| v.as_str()).unwrap_or("(untitled)");
    let targets = mod_v.get("targets").and_then(|v| v.as_array()).expect("targets[] missing");
    println!("Mod: {} | format {}.{} | {} target(s)\n", title, format, format_minor, targets.len());

    let (groups, index) = build_index();
    println!("Indexed {} pabgb/pabgh entries from PAPGT\n", index.len());

    let mut total_applied = 0usize;
    let mut total_intents = 0usize;
    let mut targets_passed = 0usize;
    let mut targets_failed = 0usize;
    let empty_intents: Vec<Value> = Vec::new();

    for target in targets {
        let target_file = target.get("file").and_then(|v| v.as_str()).unwrap_or("(unknown)");
        let intents = target.get("intents").and_then(|v| v.as_array()).unwrap_or(&empty_intents);
        let intents_n = intents.len();
        total_intents += intents_n;
        let lower = target_file.to_ascii_lowercase();

        // Iteminfo path uses dedicated parser
        if lower == "iteminfo.pabgb" {
            let Some(&pabgb_loc) = index.get(&lower) else {
                println!("[{:32}] MISS in PAZ ({} intents)", target_file, intents_n);
                targets_failed += 1; continue;
            };
            let Some(pabgb) = extract(&groups, pabgb_loc) else {
                println!("[{:32}] EXTRACT FAIL", target_file); targets_failed += 1; continue;
            };
            let mut records = match parse_iteminfo_to_json(&pabgb) {
                Ok(r) => r,
                Err(e) => { println!("[{:32}] PARSE FAIL: {}", target_file, e); targets_failed += 1; continue; }
            };
            let report = apply_intents(&mut records, intents);
            let new_body = match serialize_iteminfo_from_json(&records) {
                Ok(b) => b,
                Err(e) => {
                    println!("[{:32}] SERIALIZE FAIL after {}/{} applied: {}", target_file, report.applied, intents_n, e);
                    targets_failed += 1; continue;
                }
            };
            let delta = new_body.len() as isize - pabgb.len() as isize;
            println!("[{:32}] {:>5}/{:<5} applied | unres-entry={:>4} unres-field={:>4} type-mismatch={:>3} | Δ {:+}B  ITEMINFO",
                target_file, report.applied, intents_n,
                report.unresolved_entry, report.unresolved_field, report.type_mismatch, delta);
            total_applied += report.applied;
            if report.applied > 0 { targets_passed += 1; } else { targets_failed += 1; }
            continue;
        }

        // Generic typed table path
        let Some(dispatch) = dispatch_name_from_filename(target_file) else {
            println!("[{:32}] NO DISPATCH for filename ({} intents)", target_file, intents_n);
            targets_failed += 1; continue;
        };
        let Some(&pabgb_loc) = index.get(&lower) else {
            println!("[{:32}] MISS in PAZ ({} intents) [dispatch={}]", target_file, intents_n, dispatch);
            targets_failed += 1; continue;
        };
        let Some(pabgb) = extract(&groups, pabgb_loc) else {
            println!("[{:32}] EXTRACT FAIL", target_file); targets_failed += 1; continue;
        };
        let pabgh_name = lower.replace(".pabgb", ".pabgh");
        let pabgh = index.get(&pabgh_name).copied().and_then(|loc| extract(&groups, loc));

        // Layer 1 — DMM all_blob shortcut. If every intent uses one of the
        // four blob-shape fields, parse via blob_table runtime (which exposes
        // `_blob_b64`/`is_blocked`/`string_key`/`key`) and apply there. This
        // mirrors apply_v3_for_target's first-layer fast path in DMM-BETA.
        let all_blob = !intents.is_empty() && intents.iter().all(|i| {
            let f = i.get("field").and_then(|v| v.as_str());
            matches!(f, Some("_blob_b64") | Some("is_blocked") | Some("string_key") | Some("key"))
        });
        if all_blob {
            let Some(p) = pabgh.as_deref() else {
                println!("[{:32}] BLOB requires pabgh ({} intents) [{}]", target_file, intents_n, dispatch);
                targets_failed += 1; continue;
            };
            let mut records = match parse_blob_table_to_json_with_pabgh(&pabgb, p) {
                Ok(r) => r,
                Err(e) => {
                    println!("[{:32}] BLOB PARSE FAIL [{}]: {}", target_file, dispatch, e);
                    targets_failed += 1; continue;
                }
            };
            let report = apply_intents(&mut records, intents);
            let new_body = match serialize_blob_table_from_json(&records) {
                Ok(b) => b,
                Err(e) => {
                    println!("[{:32}] BLOB SERIALIZE FAIL after {}/{} [{}]: {}", target_file, report.applied, intents_n, dispatch, e);
                    targets_failed += 1; continue;
                }
            };
            let delta = new_body.len() as isize - pabgb.len() as isize;
            println!("[{:32}] {:>5}/{:<5} applied | unres-entry={:>4} unres-field={:>4} type-mismatch={:>3} | Δ {:+}B  BLOB({})",
                target_file, report.applied, intents_n,
                report.unresolved_entry, report.unresolved_field, report.type_mismatch, delta, dispatch);
            total_applied += report.applied;
            if report.applied > 0 { targets_passed += 1; } else { targets_failed += 1; }
            continue;
        }

        let mut records = match parse_table_to_json(dispatch, &pabgb, pabgh.as_deref()) {
            Ok(r) => r,
            Err(e) => {
                println!("[{:32}] PARSE FAIL [{}]: {}", target_file, dispatch, e); targets_failed += 1; continue;
            }
        };
        let report = apply_intents(&mut records, intents);
        let new_body = match serialize_table_from_json(dispatch, &records) {
            Ok(b) => b,
            Err(e) => {
                println!("[{:32}] SERIALIZE FAIL after {}/{} applied [{}]: {}",
                    target_file, report.applied, intents_n, dispatch, e);
                targets_failed += 1; continue;
            }
        };
        let delta = new_body.len() as isize - pabgb.len() as isize;
        println!("[{:32}] {:>5}/{:<5} applied | unres-entry={:>4} unres-field={:>4} type-mismatch={:>3} | Δ {:+}B  [{}]",
            target_file, report.applied, intents_n,
            report.unresolved_entry, report.unresolved_field, report.type_mismatch, delta, dispatch);
        total_applied += report.applied;
        if report.applied > 0 { targets_passed += 1; } else { targets_failed += 1; }
    }

    println!("\n=== Summary ===");
    println!("Targets passed (>0 applied):  {} / {}", targets_passed, targets_passed + targets_failed);
    println!("Total intents applied:        {} / {} ({:.1}%)",
        total_applied, total_intents,
        100.0 * total_applied as f64 / total_intents.max(1) as f64);
}
