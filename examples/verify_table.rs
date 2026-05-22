//! Fast single-table validation: opaque(_b64) count + byte roundtrip on the
//! live dump. Usage: cargo run --example verify_table -- <filename-stem>
//! e.g. `verify_table minigamedatainfo` (the .pabgb stem in live_full).

use dmm_parser::dispatch::{normalize_target_name, parse_table_to_json_shaped, serialize_table_from_json_shaped};
use dmm_parser::json_shape::JsonShape;
use serde_json::Value;
use std::path::PathBuf;

const DIR: &str = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\live_full";

fn has_b64(v: &Value) -> bool {
    match v {
        Value::Object(m) => m.iter().any(|(k, val)|
            (k.ends_with("_b64") && val.as_str().map(|s| !s.is_empty()).unwrap_or(false)) || has_b64(val)),
        Value::Array(a) => a.iter().any(has_b64),
        _ => false,
    }
}

fn main() {
    let stem = std::env::args().nth(1).expect("usage: verify_table <stem>");
    let canon = normalize_target_name(&stem)
        .or_else(|| normalize_target_name(&format!("{}info", stem)))
        .unwrap_or_else(|| panic!("unknown table for stem {:?}", stem));
    let b = PathBuf::from(DIR).join(format!("{}.pabgb", stem));
    let h = PathBuf::from(DIR).join(format!("{}.pabgh", stem));
    let data = std::fs::read(&b).unwrap();
    let hd = std::fs::read(&h).ok();
    let shape = JsonShape::from_str("").unwrap();
    let items = parse_table_to_json_shaped(canon, &data, hd.as_deref(), shape).unwrap();
    let opaque = items.iter().filter(|i| has_b64(i)).count();
    let re = serialize_table_from_json_shaped(canon, &items, shape).unwrap();
    println!("{} ({}): {} records, opaque(_b64)={}, byte-roundtrip={}",
        canon, stem, items.len(), opaque, re == data);
}
