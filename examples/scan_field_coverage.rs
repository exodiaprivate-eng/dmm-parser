//! Field-level coverage scanner. Parses every dumped table and counts
//! records that fall into an opaque branch (a `*_b64` tail field with
//! non-empty value, or a `_kind":"Raw"` polymorphic fallback). Output is
//! the data-driven "not 100% field level" worklist, sorted worst-first.
//!
//! Byte round-trip is already 100% everywhere; this measures the *other*
//! axis — how many records still carry undecoded bytes.

use dmm_parser::dispatch::{normalize_target_name, parse_table_to_json_shaped, serialize_table_from_json_shaped};
use dmm_parser::json_shape::JsonShape;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

const DUMP: &str = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\live_full";

/// Count opaque markers in one record (recursively).
fn opaque_in(v: &Value) -> usize {
    match v {
        Value::Object(m) => {
            let mut n = 0;
            for (k, val) in m {
                let is_b64_tail = (k.ends_with("_b64") || k.contains("tail_b64"))
                    && val.as_str().map(|s| !s.is_empty()).unwrap_or(false);
                let is_raw = k == "_kind" && val.as_str() == Some("Raw");
                if is_b64_tail || is_raw { n += 1; }
                n += opaque_in(val);
            }
            n
        }
        Value::Array(a) => a.iter().map(opaque_in).sum(),
        _ => 0,
    }
}

fn main() {
    let shape = JsonShape::from_str("").unwrap();
    let dir = Path::new(DUMP);
    let mut entries: Vec<_> = std::fs::read_dir(dir).expect("read dump dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "pabgb").unwrap_or(false))
        .collect();
    entries.sort();

    let mut clean: Vec<String> = Vec::new();
    let mut gaps: BTreeMap<String, (usize, usize, bool, usize)> = BTreeMap::new(); // (opaque, total, roundtrips, drift_recs)
    let mut unsupported: Vec<String> = Vec::new();
    let mut parse_fail: Vec<String> = Vec::new();
    let mut roundtrip_fail: Vec<String> = Vec::new();

    for pb in &entries {
        let stem = pb.file_stem().unwrap().to_string_lossy().to_string();
        // Dump filenames often drop the "info" suffix (skill.pabgb →
        // skill_info, royalsupply.pabgb → royal_supply_info). Try the bare
        // stem, then stem+"info", through the canonical normalizer.
        let canon = normalize_target_name(&stem)
            .or_else(|| normalize_target_name(&format!("{}info", stem)));
        let Some(canon) = canon else { unsupported.push(stem); continue; };
        let data = std::fs::read(pb).unwrap();
        let ph = pb.with_extension("pabgh");
        let ph_data = std::fs::read(&ph).ok();
        let parsed = parse_table_to_json_shaped(canon, &data, ph_data.as_deref(), shape);
        match parsed {
            Ok(items) => {
                let total = items.len();
                // Distinguish whole-record fallback (top-level _blob_b64 ⇒ the
                // record FAILED typed parse ⇒ drift) from a nested undecoded
                // sub-field (genuine tail). A record can be RT-clean either way.
                let drift_recs = items.iter().filter(|it|
                    it.get("_blob_b64").and_then(Value::as_str).map(|s| !s.is_empty()).unwrap_or(false)
                ).count();
                let with_opaque = items.iter().filter(|it| opaque_in(it) > 0).count();
                // Byte-roundtrip on live data.
                let rt = match serialize_table_from_json_shaped(canon, &items, shape) {
                    Ok(out) => out == data,
                    Err(_) => false,
                };
                if !rt { roundtrip_fail.push(format!("{} ({} opaque/{} rec)", canon, with_opaque, total)); }
                if with_opaque == 0 { clean.push(canon.to_string()); }
                else { gaps.insert(canon.to_string(), (with_opaque, total, rt, drift_recs)); }
            }
            Err(e) => parse_fail.push(format!("{} ({}): {}", stem, canon, e)),
        }
    }

    println!("=== FIELD-LEVEL COVERAGE ({} pabgb dumped) ===", entries.len());
    println!("\nFully field-level (0 opaque records): {}", clean.len());
    println!("Tables with opaque records: {}", gaps.len());
    println!("Unsupported (no dispatch entry): {}", unsupported.len());
    println!("Parse failures: {}", parse_fail.len());

    println!("Roundtrip failures (live data): {}", roundtrip_fail.len());

    println!("\n--- GAPS (worst first: opaque records | RT=byte-roundtrip) ---");
    println!("    RT=false ⇒ 1.07 DRIFT (parser realignment); RT=true ⇒ genuine tail (decode work)");
    let mut sorted: Vec<_> = gaps.iter().collect();
    sorted.sort_by(|a, b| b.1.0.cmp(&a.1.0));
    for (t, (op, tot, rt, drift)) in sorted {
        // Whole-record fallback (drift>0) ⇒ DRIFT regardless of RT; else genuine tail.
        let class = if *drift > 0 { "DRIFT (blob fallback)" } else { "genuine tail" };
        println!("  {:<30} {:>5}/{:<6} opaque ({:>5.1}%)  RT={:<5} blob_recs={:<5} [{}]",
            t, op, tot, 100.0 * *op as f64 / *tot as f64, rt, drift, class);
    }

    if !roundtrip_fail.is_empty() {
        println!("\n--- ROUNDTRIP FAILURES (drift — incl. tables with 0 opaque) ---");
        for f in &roundtrip_fail { println!("  {}", f); }
    }

    if !parse_fail.is_empty() {
        println!("\n--- PARSE FAILURES ---");
        for f in &parse_fail { println!("  {}", f); }
    }
    if !unsupported.is_empty() {
        println!("\n--- UNSUPPORTED ({}) ---", unsupported.len());
        println!("  {}", unsupported.join(", "));
    }
}
