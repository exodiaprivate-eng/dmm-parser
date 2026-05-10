// Diagnostic: apply Universal Proficiency mod against vanilla iteminfo
// and report per-intent outcomes. Reproduces what DMM's
// apply_v3_to_typed_table_body would do for this specific mod, in
// isolation from the rest of the mount pipeline.
//
// Reports:
//   - parse_iteminfo_to_json record count
//   - per-intent applied / unresolved_entry / unresolved_field counts
//   - body delta (vanilla vs new)
//   - re-parse round-trip check (catches schema drift between
//     write_from_json and the parser)
//
// If this passes (all 2494 intents apply, body delta non-zero,
// re-parse succeeds), the regression is downstream of dmm-parser —
// most likely DMM-side mount orchestration or PAPGT registration.

use dmm_parser::item_info::{parse_iteminfo_to_json, serialize_iteminfo_from_json};
use dmm_parser::intents::{IntentDoc, apply_resolved_intents};
use std::path::Path;

fn main() {
    let vanilla_path = std::env::var("VANILLA_ITEMINFO_PATH")
        .unwrap_or_else(|_| {
            r"C:\Users\corin\Desktop\DMM 1.3.5\backups\iteminfo_pabgb_clean.bin".to_string()
        });
    let mod_path = std::env::var("MOD_PATH")
        .unwrap_or_else(|_| r"C:\Users\corin\Desktop\ZIPS\Universal_Proficiency.json".to_string());

    let vanilla = std::fs::read(Path::new(&vanilla_path))
        .expect("read vanilla iteminfo body");
    println!("vanilla iteminfo body: {} bytes", vanilla.len());

    let mod_bytes = std::fs::read(Path::new(&mod_path)).expect("read mod json");
    let doc = IntentDoc::from_slice(&mod_bytes).expect("parse mod json");
    let targets = doc.flatten_targets();
    println!("mod has {} target(s)", targets.len());
    for (t, intents) in &targets {
        println!("  target={:?} intents={}", t, intents.len());
    }

    let mut records = parse_iteminfo_to_json(&vanilla).expect("parse vanilla");
    println!("parsed {} records from vanilla", records.len());

    let mut total_applied = 0usize;
    let mut total_skipped = 0usize;
    let mut sample_skips: Vec<String> = Vec::new();
    for (target, intents) in &targets {
        if !target.starts_with("iteminfo") {
            println!("SKIP target {:?} (not iteminfo)", target);
            continue;
        }
        match apply_resolved_intents(&mut records, intents) {
            Ok(outcomes) => {
                for o in &outcomes {
                    match &o.status {
                        dmm_parser::intents::ApplyStatus::Applied => total_applied += 1,
                        dmm_parser::intents::ApplyStatus::Skipped(reason) => {
                            total_skipped += 1;
                            if sample_skips.len() < 5 {
                                sample_skips.push(reason.clone());
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("HARD ERROR applying intents: {}", e);
                std::process::exit(1);
            }
        }
    }
    println!(
        "apply: {} applied, {} skipped",
        total_applied, total_skipped
    );
    for s in &sample_skips {
        println!("  sample skip: {}", s);
    }

    // Serialize back to bytes.
    let new_body = match serialize_iteminfo_from_json(&records) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("SERIALIZE FAILED: {}", e);
            std::process::exit(1);
        }
    };
    let delta = new_body.len() as isize - vanilla.len() as isize;
    println!(
        "serialize OK: {} bytes (Δ {:+}B vs vanilla)",
        new_body.len(),
        delta
    );

    // Re-parse to catch schema drift between write_from_json and
    // the parser (the silent-no-op class of bug — body changes but
    // re-parse fails / records flip back).
    match parse_iteminfo_to_json(&new_body) {
        Ok(re) => {
            println!("re-parse OK: {} records (vanilla had {})", re.len(), records.len());
            if re.len() != records.len() {
                eprintln!("WARN: record count changed across round-trip");
            }
        }
        Err(e) => {
            eprintln!("RE-PARSE FAILED: {}", e);
            std::process::exit(1);
        }
    }

    // Spot-check: the first intent's target record.
    if let Some(first_intent) = targets.first().and_then(|(_, i)| i.first()) {
        if let Some(key) = first_intent.key {
            let target_idx = records.iter().position(|r| {
                r.get("key").and_then(|k| k.get("value")).and_then(|v| v.as_u64()) == Some(key as u64)
                    || r.get("key").and_then(|v| v.as_u64()) == Some(key as u64)
            });
            match target_idx {
                Some(i) => {
                    let pdl = records[i].get("prefab_data_list");
                    println!(
                        "spot-check: record key={} idx={} prefab_data_list = {} elements",
                        key, i,
                        pdl.and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0)
                    );
                }
                None => println!("spot-check: record key={} NOT FOUND", key),
            }
        }
    }

    println!("\n=== SUMMARY ===");
    println!("applied: {} / {}", total_applied, total_applied + total_skipped);
    println!("body delta: {:+} bytes", delta);
    if total_skipped > 0 {
        println!("STATUS: regression — {} intents skipped", total_skipped);
    } else if delta == 0 {
        println!("STATUS: regression — body byte-identical to vanilla despite all intents applying");
    } else {
        println!("STATUS: dmm-parser side OK — regression is downstream (DMM mount / PAPGT / overlay)");
    }
}
