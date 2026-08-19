//! Adding a record must survive the write.
//!
//! `clone_record` / `new_record` are the basis of custom-item mods, and their
//! failures are quiet ones: the manifest reports "Applied", the body grows, and
//! the record is either indexed at the wrong offset or created under a key
//! nobody asked for. Both shapes shipped before — see the notes on
//! `verify_created_keys_landed` and on the `not pabgh-bounded` arm in
//! `dispatch::apply_intents_to_table_body`.
//!
//! So this sweeps the whole registry rather than one table: for every fixture
//! present, clone the first record under a fresh key, re-read the exact
//! (body, pabgh) pair the caller would ship, and require the clone to be there
//! — with no record that used to decode falling back to a blob.
//!
//! Fixtures live outside the repo, so the sweep skips cleanly when they are
//! absent. Point `DMM_PARSER_FIXTURE_DIR` at a patch-day dump to run it.

use dmm_parser::dispatch::{apply_intents_to_table_body, normalize_target_name,
                           parse_table_to_json};
use dmm_parser::intents::{Intent, Patch};
use serde_json::{json, Value};

fn fixture_dir() -> Option<String> {
    let candidates = [
        std::env::var("DMM_PARSER_FIXTURE_DIR").ok(),
        Some(r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-8-17".into()),
        Some("/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-8-17".into()),
    ];
    candidates.into_iter().flatten().find(|p| std::path::Path::new(p).is_dir())
}

/// A record the parser could not decode, kept as an opaque blob or a salvaged
/// prefix. Cloning a blob donor legitimately produces one more blob; anything
/// beyond that means the write moved a record out from under its index.
fn blobbed(r: &Value) -> bool {
    r.get("_partial_fields").is_some() || r.get("_blob_fallback").is_some()
}

fn key_of(v: &Value) -> Option<i64> {
    v.get("key").and_then(|k| k.get("value").or(Some(k))).and_then(|x| x.as_i64())
}

#[test]
fn cloned_record_survives_the_round_trip_on_every_table() {
    let Some(dir) = fixture_dir() else {
        eprintln!("SKIP cloned_record_survives_the_round_trip_on_every_table: no fixture dir");
        return;
    };
    let Ok(entries) = std::fs::read_dir(&dir) else { return };
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".pabgb"))
        .collect();
    names.sort();

    let mut clean = 0usize;
    let mut narrow_key = 0usize;
    let mut skipped = 0usize;
    let mut failures: Vec<String> = Vec::new();

    for file in &names {
        let stem = file.trim_end_matches(".pabgb");
        let Some(table) = normalize_target_name(file).or_else(|| normalize_target_name(stem))
        else { continue };
        let (Ok(body), Ok(pabgh)) = (
            std::fs::read(format!("{}/{}", dir, file)),
            std::fs::read(format!("{}/{}.pabgh", dir, stem)),
        ) else { continue };

        let before = match parse_table_to_json(table, &body, Some(&pabgh)) {
            Ok(r) if !r.is_empty() => r,
            _ => { skipped += 1; continue }
        };
        let Some(donor) = key_of(&before[0]) else { skipped += 1; continue };
        let used: std::collections::HashSet<i64> = before.iter().filter_map(key_of).collect();

        // Prefer a wide custom key, the shape a real custom-item mod uses. Where
        // the table's key field is too narrow to hold one the parser now refuses
        // instead of clamping, so fall back to the widest key that table can
        // actually express — one past its own highest.
        let mut fitted = used.iter().copied().max().unwrap_or(0) + 1;
        while used.contains(&fitted) { fitted += 1; }
        let candidates: Vec<i64> =
            if fitted == 990_001 { vec![990_001] } else { vec![990_001, fitted] };

        let mut applied = None;
        let mut last_err = String::new();
        for (attempt, new_key) in candidates.iter().enumerate() {
            let intent = Intent {
                op: Some("clone_record".into()),
                source_key: Some(donor),
                new_key: Some(*new_key),
                patches: Some(vec![Patch {
                    path: "string_key".into(), op: None, new: json!("DMM_Clone_Probe"),
                }]),
                ..Default::default()
            };
            match apply_intents_to_table_body(table, &body, Some(&pabgh), &[intent]) {
                Ok(v) => {
                    if attempt > 0 { narrow_key += 1; }
                    applied = Some((v, *new_key));
                    break;
                }
                Err(e) => last_err = e.to_string(),
            }
        }
        let Some(((new_body, new_pabgh, _outcomes), new_key)) = applied else {
            // A table with no pabgh rebuild path and no record-op support is out
            // of scope here, not a regression.
            if last_err.contains("not pabgh-bounded") { skipped += 1; }
            else { failures.push(format!("{}: apply refused every key — {}", table, last_err)); }
            continue;
        };

        let after = match parse_table_to_json(table, &new_body, new_pabgh.as_deref()) {
            Ok(a) => a,
            Err(e) => { failures.push(format!("{}: re-read failed — {}", table, e)); continue }
        };

        let donor_was_blob = before.iter().find(|r| key_of(r) == Some(donor)).is_some_and(blobbed);
        let budget = before.iter().filter(|r| blobbed(r)).count()
            + usize::from(donor_was_blob);
        let now = after.iter().filter(|r| blobbed(r)).count();

        if after.len() != before.len() + 1 {
            failures.push(format!("{}: record count {} -> {}, expected +1",
                                  table, before.len(), after.len()));
        } else if now > budget {
            failures.push(format!("{}: {} records fell back to blob (was {}) — the added \
                                   record moved the ones after it out from under the index",
                                  table, now, budget));
        } else if !after.iter().any(|r| key_of(r) == Some(new_key)) {
            failures.push(format!("{}: clone is not readable back at key {}", table, new_key));
        } else {
            clean += 1;
        }
    }

    eprintln!("clone_record sweep: {} tables clean ({} needed a key narrow enough for their \
               key field), {} skipped, {} failed",
              clean, narrow_key, skipped, failures.len());
    assert!(clean > 100, "expected the sweep to cover >100 tables, got {}", clean);
    assert!(failures.is_empty(), "clone_record round-trip failed:\n  {}", failures.join("\n  "));
}

/// The failure this replaces was silent: a key too wide for the table's key
/// field was clamped to 65535 on write, the record was created under that
/// identity — usually colliding with a real one — and the apply still reported
/// success. Refusing is the whole point, so pin it.
#[test]
fn an_over_wide_new_key_is_refused_rather_than_clamped() {
    let Some(dir) = fixture_dir() else {
        eprintln!("SKIP an_over_wide_new_key_is_refused_rather_than_clamped: no fixture dir");
        return;
    };
    // bank_info's key field is u16, so 990001 cannot be represented in it.
    let (Ok(body), Ok(pabgh)) = (
        std::fs::read(format!("{}/bankinfo.pabgb", dir)),
        std::fs::read(format!("{}/bankinfo.pabgh", dir)),
    ) else { eprintln!("SKIP: no bankinfo fixture"); return };

    let before = parse_table_to_json("bank_info", &body, Some(&pabgh)).expect("parse bank_info");
    let donor = key_of(&before[0]).expect("donor key");

    let wide = Intent {
        op: Some("clone_record".into()),
        source_key: Some(donor), new_key: Some(990_001),
        ..Default::default()
    };
    let err = apply_intents_to_table_body("bank_info", &body, Some(&pabgh), &[wide])
        .expect_err("a key too wide for the key field must be refused, not clamped");
    let msg = err.to_string();
    assert!(msg.contains("990001"), "the error must name the key asked for: {}", msg);
    assert!(msg.contains("too wide"), "the error must say why: {}", msg);

    // ...and a key the field can hold still works.
    let used: std::collections::HashSet<i64> = before.iter().filter_map(key_of).collect();
    let mut fits = 60_000i64;
    while used.contains(&fits) { fits += 1; }
    let narrow = Intent {
        op: Some("clone_record".into()),
        source_key: Some(donor), new_key: Some(fits),
        ..Default::default()
    };
    let (nb, np, _) = apply_intents_to_table_body("bank_info", &body, Some(&pabgh), &[narrow])
        .expect("a key that fits the field must apply");
    let after = parse_table_to_json("bank_info", &nb, np.as_deref()).expect("re-read");
    assert_eq!(after.len(), before.len() + 1);
    assert!(after.iter().any(|r| key_of(r) == Some(fits)),
            "clone should be readable back at key {}", fits);
}
