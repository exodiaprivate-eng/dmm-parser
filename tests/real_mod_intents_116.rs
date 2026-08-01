//! End-to-end check: do the user's REAL mod files apply cleanly against the
//! current (game v16) tables?
//!
//! The V3 coverage harness only proves parse→serialize is byte-perfect; a table
//! can round-trip through the blob fallback while still being unable to RESOLVE
//! a field path, which is exactly what "N intents skipped" in the mount log means.
//! This drives the same entry point DMM does (`apply_intents_to_table_body`) with
//! the actual mod JSON, and reports every skip reason.
//!
//! Run:
//!   DMM_PARSER_PABGB_DIR=<1.16 dump> cargo test --test real_mod_intents_116 -- --ignored --nocapture

use dmm_parser::dispatch::apply_intents_to_table_body;
use dmm_parser::intents::{ApplyStatus, IntentDoc};
use std::path::{Path, PathBuf};

const MODS: &str = r"C:\Users\justi\Desktop\DMM\mods";

fn fixture_dir() -> PathBuf {
    std::env::var("DMM_PARSER_PABGB_DIR")
        .unwrap_or_else(|_| r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-8-1".into())
        .into()
}

/// Fixture files use the compact name (`characterinfo.pabgb`), which is already
/// what the mod `target` / `file` field carries — but a few tables drop the
/// trailing "info", so try the obvious candidates.
fn find_fixture(dir: &Path, target: &str) -> Option<PathBuf> {
    let stem = target.trim_end_matches(".pabgb");
    for cand in [stem.to_string(), stem.trim_end_matches("info").to_string()] {
        let p = dir.join(format!("{cand}.pabgb"));
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

#[test]
#[ignore]
fn real_mods_apply_on_v16() {
    let dir = fixture_dir();
    eprintln!("fixtures: {}\n", dir.display());

    let mods: [(&str, &str); 4] = [
        ("Female Oongka", r"Female Oongka\Female Oongka.field.json"),
        ("Ultimate Female Animations (V3)", r"Female Weapon Master\Ultimate Female Animations V3.json"),
        ("World Mod Pack", r"World Mod Pack\World Mod Pack.json"),
        ("I Like Space", r"I Like Space\I Like Space.json"),
    ];

    let mut total_applied = 0usize;
    let mut total_skipped = 0usize;

    for (name, rel) in mods {
        let path = PathBuf::from(MODS).join(rel);
        let Ok(bytes) = std::fs::read(&path) else {
            eprintln!("{name}: MOD FILE NOT FOUND at {}", path.display());
            continue;
        };
        let doc = match IntentDoc::from_slice(&bytes) {
            Ok(d) => d,
            Err(e) => { eprintln!("{name}: doc parse failed: {e}"); continue; }
        };

        for (target, intents) in doc.flatten_targets() {
            let Some(fx) = find_fixture(&dir, &target) else {
                eprintln!("{name} -> {target}: no fixture, SKIPPED CHECK");
                continue;
            };
            let body = std::fs::read(&fx).expect("read body");
            let pabgh = std::fs::read(fx.with_extension("pabgh")).ok();

            match apply_intents_to_table_body(&target, &body, pabgh.as_deref(), &intents) {
                Ok((_new_body, _new_pabgh, outcomes)) => {
                    let applied = outcomes.iter()
                        .filter(|o| matches!(o.status, ApplyStatus::Applied)).count();
                    let skipped = outcomes.len() - applied;
                    total_applied += applied;
                    total_skipped += skipped;
                    let flag = if skipped == 0 { "OK  " } else { "SKIP" };
                    eprintln!("{flag} {name} -> {target}: {applied} applied / {skipped} skipped  (of {})",
                        intents.len());
                    for o in &outcomes {
                        if let ApplyStatus::Skipped(why) = &o.status {
                            eprintln!("        [{}] {}", o.op, why);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("ERR  {name} -> {target}: apply failed: {e}");
                    total_skipped += intents.len();
                }
            }
        }
    }

    eprintln!("\n==== TOTAL: {total_applied} applied / {total_skipped} skipped ====");
    assert_eq!(total_skipped, 0, "{total_skipped} intents still skipped");
}

/// What entry names does inventory.pabgb actually expose on v16? "I Like Space"
/// targets an entry called `Character` that the apply path can no longer find,
/// so either it was renamed or it is gone.
#[test]
#[ignore]
fn dump_inventory_entries() {
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("inventory.pabgb")).expect("body");
    let pabgh = std::fs::read(dir.join("inventory.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("inventory.pabgb", &body, pabgh.as_deref())
        .expect("parse");
    eprintln!("inventory records: {}", arr.len());
    for r in &arr {
        let obj = match r.as_object() { Some(o) => o, None => continue };
        // print whatever string-ish identifier the record carries
        let ident: Vec<String> = obj.iter()
            .filter(|(_, v)| v.is_string())
            .map(|(k, v)| format!("{k}={}", v.as_str().unwrap_or("")))
            .take(3)
            .collect();
        let key = obj.get("__key__").or_else(|| obj.get("key"));
        eprintln!("   key={:?}  {}", key, ident.join("  "));
    }
}

/// Female Oongka borrows appearance/prefab/skeleton HASHES from the playable
/// donors (Kliff=1, Yahn=2, Damian=4) and writes them onto Oongka (key 6).
/// If v16 changed the donor values, those hashes now dangle -> null deref on any
/// screen that builds a character preview. Print the donors' live v16 values.
#[test]
#[ignore]
fn dump_playable_character_hashes() {
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("characterinfo.pabgb")).expect("body");
    let pabgh = std::fs::read(dir.join("characterinfo.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("characterinfo.pabgb", &body, pabgh.as_deref())
        .expect("parse");
    let fields = ["appearance_name", "character_prefab_path", "skeleton_name",
                  "default_action_action_index", "lookup_24", "lookup_25", "f36"];
    // What Female Oongka writes onto key 6:
    let wants: [(&str, u64); 7] = [
        ("appearance_name", 1767116530), ("character_prefab_path", 3755051597),
        ("skeleton_name", 3000129643), ("default_action_action_index", 1287066785),
        ("lookup_24", 2831867940), ("lookup_25", 3511542393), ("f36", 2),
    ];
    for r in &arr {
        let o = match r.as_object() { Some(o) => o, None => continue };
        let key = o.get("__key__").or_else(|| o.get("key"))
            .and_then(|k| k.as_u64()).unwrap_or(u64::MAX);
        if ![1u64, 2, 4, 6].contains(&key) { continue; }
        eprintln!("--- key={key}");
        for f in fields {
            let v = o.get(f).and_then(|x| x.as_u64());
            let mark = wants.iter().find(|(n, w)| *n == f && Some(*w) == v)
                .map(|_| "   <== matches what Female Oongka writes").unwrap_or("");
            eprintln!("      {:<30} {:?}{}", f, v, mark);
        }
    }
    eprintln!("\nFemale Oongka writes onto key 6:");
    for (n, w) in wants { eprintln!("      {:<30} {}", n, w); }
}

/// ★ A byte-exact ROUNDTRIP does not validate field PLACEMENT: if a field sits at
/// the wrong offset, read and write stay symmetric so the table still round-trips,
/// but every field after it is misparsed and any mod EDIT writes corruption.
/// So: apply the real mods, then RE-PARSE the applied output and re-serialize it.
/// If the result is not stable, the applied table is malformed and the game will
/// choke on it even though every gate above was green.
#[test]
#[ignore]
fn applied_output_reparses() {
    let dir = fixture_dir();
    let mods: [(&str, &str); 6] = [
        ("Equip All V7.2", r"Equip_Everything_V7_2571_7.2_2026-07-19T02-47Z_XOtFiw9Pm"),
        ("World Mod Pack", r"World Mod Pack"),
        ("I Like Space", r"I Like Space"),
        ("QOL Mod", r"QOL Mod"),
        ("Super Guard", r"Super Guard"),
        ("Store Refresh", r"Store Refresh 1 Day + Restock x10"),
    ];
    let mut bad = 0;
    for (name, folder) in mods {
        let base = PathBuf::from(MODS).join(folder);
        let Ok(rd) = std::fs::read_dir(&base) else { eprintln!("{name}: folder missing"); continue };
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) != Some("json") { continue; }
            let Ok(bytes) = std::fs::read(&p) else { continue };
            let Ok(doc) = IntentDoc::from_slice(&bytes) else { continue };
            for (target, intents) in doc.flatten_targets() {
                let Some(fx) = find_fixture(&dir, &target) else { continue };
                let body = std::fs::read(&fx).unwrap();
                let pabgh = std::fs::read(fx.with_extension("pabgh")).ok();
                let Ok((new_body, new_pabgh, _)) =
                    apply_intents_to_table_body(&target, &body, pabgh.as_deref(), &intents)
                else { continue };
                // Re-parse the APPLIED bytes and re-serialize with zero intents.
                match apply_intents_to_table_body(&target, &new_body, new_pabgh.as_deref(), &[]) {
                    Ok((again, _, _)) if again == new_body =>
                        eprintln!("STABLE  {name} -> {target}  ({} B)", new_body.len()),
                    Ok((again, _, _)) => {
                        bad += 1;
                        eprintln!("UNSTABLE {name} -> {target}: re-serialize differs ({} vs {} B)",
                            again.len(), new_body.len());
                    }
                    Err(e) => { bad += 1; eprintln!("REPARSE-FAIL {name} -> {target}: {e}"); }
                }
            }
        }
    }
    eprintln!("\n==== {bad} table(s) malformed after apply ====");
    assert_eq!(bad, 0, "applied output does not re-parse cleanly");
}

/// Sanity-check equipslotinfo entry VALUES (not bytes). If `slot_name_hash_116`
/// is at the wrong offset, every field after it is misparsed — which bytes can
/// never reveal, because read/write stay symmetric. Flags must be 0/1 and
/// slot_index must be a small ordinal; garbage there means the layout is wrong.
#[test]
#[ignore]
fn equipslot_value_sanity() {
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("equipslotinfo.pabgb")).expect("body");
    let pabgh = std::fs::read(dir.join("equipslotinfo.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("equipslotinfo.pabgb", &body, pabgh.as_deref())
        .expect("parse");
    let mut bad = 0usize;
    let mut checked = 0usize;
    for r in &arr {
        let o = match r.as_object() { Some(o) => o, None => continue };
        let key = o.get("__key__").or_else(|| o.get("key")).and_then(|k| k.as_u64());
        let Some(entries) = o.get("entries").and_then(|e| e.as_array()) else { continue };
        for (i, e) in entries.iter().enumerate() {
            let Some(eo) = e.as_object() else { continue };
            checked += 1;
            let g = |n: &str| eo.get(n).and_then(|x| x.as_u64());
            let slot = g("slot_index");
            let flags: Vec<(&str, Option<u64>)> = ["is_hide_equip_in_dyeing_process_113",
                "tail_byte_6", "tail_byte_7", "tail_byte_8", "tail_byte_9", "tail_byte_10"]
                .iter().map(|n| (*n, g(n))).collect();
            let slot_bad = slot.map(|s| s > 64).unwrap_or(true);
            let flag_bad: Vec<&str> = flags.iter()
                .filter(|(_, v)| v.map(|x| x > 1).unwrap_or(false))
                .map(|(n, _)| *n).collect();
            if slot_bad || !flag_bad.is_empty() {
                bad += 1;
                if bad <= 8 {
                    eprintln!("SUSPECT k={:?} entries[{}] slot_index={:?} bad_flags={:?} slot_name_hash_116={:?} dyeing_a={:?}",
                        key, i, slot, flag_bad, g("slot_name_hash_116"), g("dyeing_field_a_113"));
                }
            }
        }
    }
    eprintln!("\nchecked {checked} entries, {bad} with out-of-range values");
}

/// ★ Value-sanity the table AFTER the mod writes to it — the bytes that actually
/// ship to the engine. Sanity-checking only the VANILLA table (as an earlier
/// version of this test did) proves nothing about the modded output.
#[test]
#[ignore]
fn equipslot_value_sanity_after_apply() {
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("equipslotinfo.pabgb")).expect("body");
    let pabgh = std::fs::read(dir.join("equipslotinfo.pabgh")).ok();
    let p = PathBuf::from(MODS)
        .join(r"Equip_Everything_V7_2571_7.2_2026-07-19T02-47Z_XOtFiw9Pm\Equip Everything V7.2.json");
    let doc = IntentDoc::from_slice(&std::fs::read(&p).expect("mod")).expect("doc");
    let intents: Vec<_> = doc.flatten_targets().into_iter()
        .filter(|(t, _)| t.contains("equipslot")).flat_map(|(_, v)| v).collect();
    eprintln!("applying {} equipslotinfo intents", intents.len());
    let (new_body, new_pabgh, _) =
        apply_intents_to_table_body("equipslotinfo.pabgb", &body, pabgh.as_deref(), &intents)
            .expect("apply");
    eprintln!("vanilla {} B -> applied {} B (delta {:+})",
        body.len(), new_body.len(), new_body.len() as i64 - body.len() as i64);

    for (label, b, h) in [("VANILLA", &body, pabgh.clone()), ("APPLIED", &new_body, new_pabgh)] {
        let arr = match dmm_parser::dispatch::parse_table_to_json("equipslotinfo.pabgb", b, h.as_deref()) {
            Ok(a) => a,
            Err(e) => { eprintln!("{label}: PARSE FAILED: {e}"); continue; }
        };
        let (mut bad, mut checked, mut hashes) = (0usize, 0usize, 0usize);
        for r in &arr {
            let Some(o) = r.as_object() else { continue };
            let Some(entries) = o.get("entries").and_then(|e| e.as_array()) else { continue };
            for e in entries {
                let Some(eo) = e.as_object() else { continue };
                checked += 1;
                hashes += eo.get("etl_hashes").and_then(|x| x.as_array()).map(|a| a.len()).unwrap_or(0);
                let g = |n: &str| eo.get(n).and_then(|x| x.as_u64());
                let slot_bad = g("slot_index").map(|s| s > 64).unwrap_or(true);
                let flag_bad = ["is_hide_equip_in_dyeing_process_113", "tail_byte_6",
                    "tail_byte_7", "tail_byte_8", "tail_byte_9", "tail_byte_10"]
                    .iter().any(|n| g(n).map(|x| x > 1).unwrap_or(false));
                if slot_bad || flag_bad { bad += 1; }
            }
        }
        eprintln!("{label}: {checked} entries, {bad} insane, {hashes} total etl hashes");
    }
}

/// Does Equip All write prefab_names / etl hashes that no longer EXIST on v16?
/// The mod was authored on v15 and stores captured hashes; v16 remapped keys
/// wholesale (confirmed in item_use_info). A dangling reference is a null deref
/// at the moment the engine builds the item — which no byte-level gate can see.
#[test]
#[ignore]
fn equipall_hashes_exist_on_v16() {
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("iteminfo.pabgb")).expect("iteminfo");
    let pabgh = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &body, pabgh.as_deref())
        .expect("parse iteminfo");
    // every prefab_name hash that EXISTS in vanilla v16
    let mut live = std::collections::HashSet::new();
    for r in &arr {
        let Some(o) = r.as_object() else { continue };
        let Some(pdl) = o.get("prefab_data_list").and_then(|x| x.as_array()) else { continue };
        for pd in pdl {
            let Some(po) = pd.as_object() else { continue };
            for k in ["prefab_names", "animation_path_list", "tribe_gender_list"] {
                if let Some(a) = po.get(k).and_then(|x| x.as_array()) {
                    for v in a { if let Some(n) = v.as_u64() { live.insert(n); } }
                }
            }
        }
    }
    eprintln!("vanilla v16 distinct prefab/anim/tribe hashes: {}", live.len());

    let p = PathBuf::from(MODS)
        .join(r"Equip_Everything_V7_2571_7.2_2026-07-19T02-47Z_XOtFiw9Pm\Equip Everything V7.2.json");
    let raw: serde_json::Value = serde_json::from_slice(&std::fs::read(&p).expect("mod")).expect("json");
    let (mut total, mut missing) = (0usize, 0usize);
    let mut sample = Vec::new();
    for t in raw["targets"].as_array().into_iter().flatten() {
        if !t["file"].as_str().unwrap_or("").contains("iteminfo") { continue; }
        for i in t["intents"].as_array().into_iter().flatten() {
            for pd in i["new"].as_array().into_iter().flatten() {
                for k in ["prefab_names", "animation_path_list", "tribe_gender_list"] {
                    for v in pd[k].as_array().into_iter().flatten() {
                        let Some(n) = v.as_u64() else { continue };
                        total += 1;
                        if !live.contains(&n) {
                            missing += 1;
                            if sample.len() < 10 {
                                sample.push(format!("{} (item {})", n, i["entry"].as_str().unwrap_or("?")));
                            }
                        }
                    }
                }
            }
        }
    }
    eprintln!("Equip All writes {total} hash refs; {missing} DO NOT EXIST in vanilla v16");
    for s in &sample { eprintln!("   missing: {s}"); }
}

/// Emit a corrected Equip All: drop any intent whose prefab_data_list references a
/// hash that does not exist on v16. Those dead refs are what crash the engine.
/// Everything else applies unchanged.
#[test]
#[ignore]
fn write_fixed_equipall() {
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("iteminfo.pabgb")).expect("iteminfo");
    let pabgh = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &body, pabgh.as_deref())
        .expect("parse");
    let mut live = std::collections::HashSet::new();
    for r in &arr {
        let Some(o) = r.as_object() else { continue };
        for pd in o.get("prefab_data_list").and_then(|x| x.as_array()).into_iter().flatten() {
            let Some(po) = pd.as_object() else { continue };
            for k in ["prefab_names", "animation_path_list", "tribe_gender_list"] {
                for v in po.get(k).and_then(|x| x.as_array()).into_iter().flatten() {
                    if let Some(n) = v.as_u64() { live.insert(n); }
                }
            }
        }
    }
    let p = PathBuf::from(MODS)
        .join(r"Equip_Everything_V7_2571_7.2_2026-07-19T02-47Z_XOtFiw9Pm\Equip Everything V7.2.json");
    let mut raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&p).expect("mod")).expect("json");
    let mut dropped: Vec<String> = Vec::new();
    for t in raw["targets"].as_array_mut().into_iter().flatten() {
        if !t["file"].as_str().unwrap_or("").contains("iteminfo") { continue; }
        let name = |i: &serde_json::Value| i["entry"].as_str().unwrap_or("?").to_string();
        let keep: Vec<serde_json::Value> = t["intents"].as_array().unwrap().iter().filter(|i| {
            let mut ok = true;
            for pd in i["new"].as_array().into_iter().flatten() {
                for k in ["prefab_names", "animation_path_list", "tribe_gender_list"] {
                    for v in pd[k].as_array().into_iter().flatten() {
                        if let Some(n) = v.as_u64() { if !live.contains(&n) { ok = false; } }
                    }
                }
            }
            if !ok { dropped.push(name(i)); }
            ok
        }).cloned().collect();
        t["intents"] = serde_json::Value::Array(keep);
    }
    raw["modinfo"]["title"] = serde_json::json!("ZZ TEST EquipAll v16-FIXED");
    let out_dir = PathBuf::from(MODS).join("ZZ TEST EquipAll v16-FIXED");
    std::fs::create_dir_all(&out_dir).unwrap();
    std::fs::write(out_dir.join("ZZ TEST EquipAll v16-FIXED.json"),
        serde_json::to_vec_pretty(&raw).unwrap()).unwrap();
    dropped.sort(); dropped.dedup();
    eprintln!("DROPPED {} item intent(s) with dead v16 refs:", dropped.len());
    for d in &dropped { eprintln!("   {d}"); }
    eprintln!("wrote {}", out_dir.display());
}

/// CORRECTION to `equipall_hashes_exist_on_v16`: prefab_names are StringInfoKey,
/// which resolve against STRINGINFO -- not against "some other vanilla item's
/// prefab list". Equip All exists precisely to grant gear no vanilla item
/// references, so the old test flagged valid hashes as dead. Check the real table.
#[test]
#[ignore]
fn equipall_hashes_vs_stringinfo() {
    let dir = fixture_dir();
    let mut live = std::collections::HashSet::new();
    for tbl in ["stringinfo", "iteminfo"] {
        let Ok(body) = std::fs::read(dir.join(format!("{tbl}.pabgb"))) else { continue };
        let pabgh = std::fs::read(dir.join(format!("{tbl}.pabgh"))).ok();
        let Ok(arr) = dmm_parser::dispatch::parse_table_to_json(
            &format!("{tbl}.pabgb"), &body, pabgh.as_deref()) else { continue };
        let mut n = 0usize;
        // collect EVERY integer that appears anywhere in the table
        fn walk(v: &serde_json::Value, out: &mut std::collections::HashSet<u64>, n: &mut usize) {
            match v {
                serde_json::Value::Number(x) => { if let Some(u) = x.as_u64() { out.insert(u); *n += 1; } }
                serde_json::Value::Array(a) => for e in a { walk(e, out, n) },
                serde_json::Value::Object(o) => for (_, e) in o { walk(e, out, n) },
                _ => {}
            }
        }
        for r in &arr { walk(r, &mut live, &mut n); }
        eprintln!("{tbl}: {} records, {} numeric values", arr.len(), n);
    }
    eprintln!("live numeric universe: {}", live.len());

    let p = PathBuf::from(MODS)
        .join(r"Equip_Everything_V7_2571_7.2_2026-07-19T02-47Z_XOtFiw9Pm\Equip Everything V7.2.json");
    let raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&p).expect("mod")).expect("json");
    let (mut total, mut missing) = (0usize, 0usize);
    let mut miss: Vec<(u64, String)> = Vec::new();
    for t in raw["targets"].as_array().into_iter().flatten() {
        if !t["file"].as_str().unwrap_or("").contains("iteminfo") { continue; }
        for i in t["intents"].as_array().into_iter().flatten() {
            for pd in i["new"].as_array().into_iter().flatten() {
                for k in ["prefab_names", "animation_path_list", "tribe_gender_list"] {
                    for v in pd[k].as_array().into_iter().flatten() {
                        let Some(n) = v.as_u64() else { continue };
                        total += 1;
                        if !live.contains(&n) {
                            missing += 1;
                            miss.push((n, i["entry"].as_str().unwrap_or("?").into()));
                        }
                    }
                }
            }
        }
    }
    eprintln!("\nEquip All writes {total} refs; {missing} absent from stringinfo+iteminfo entirely");
    miss.sort(); miss.dedup();
    for (h, item) in miss.iter().take(40) { eprintln!("   {h} ({item})"); }
}

/// Equip All SETS prefab_data_list wholesale from v15-captured content, so any v16
/// change to those items is REVERTED. Compare, per item, what the mod writes vs
/// what v16 vanilla actually has -- element COUNT and equip_slot values. A reverted
/// list or an out-of-range slot is exactly what would fault while the save-select
/// screen builds each slot's character portrait.
#[test]
#[ignore]
fn equipall_vs_vanilla_prefab_lists() {
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("iteminfo.pabgb")).expect("iteminfo");
    let pabgh = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &body, pabgh.as_deref())
        .expect("parse");
    let mut van: std::collections::HashMap<u64, &serde_json::Value> = Default::default();
    let mut slots_live = std::collections::HashSet::new();
    for r in &arr {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        if let Some(p) = o.get("prefab_data_list") { van.insert(k, p); }
        for pd in o.get("prefab_data_list").and_then(|x| x.as_array()).into_iter().flatten() {
            for v in pd["equip_slot_list"].as_array().into_iter().flatten() {
                if let Some(n) = v.as_u64() { slots_live.insert(n); }
            }
        }
    }
    let mut ls: Vec<_> = slots_live.iter().copied().collect(); ls.sort();
    eprintln!("v16 vanilla equip_slot values in use: {:?}", ls);

    let p = PathBuf::from(MODS)
        .join(r"Equip_Everything_V7_2571_7.2_2026-07-19T02-47Z_XOtFiw9Pm\Equip Everything V7.2.json");
    let raw: serde_json::Value = serde_json::from_slice(&std::fs::read(&p).expect("mod")).expect("json");
    let (mut same_len, mut diff_len, mut bad_slot) = (0usize, 0usize, 0usize);
    let mut examples = Vec::new();
    for t in raw["targets"].as_array().into_iter().flatten() {
        if !t["file"].as_str().unwrap_or("").contains("iteminfo") { continue; }
        for i in t["intents"].as_array().into_iter().flatten() {
            let Some(k) = i["key"].as_u64() else { continue };
            let modl = i["new"].as_array().map(|a| a.len()).unwrap_or(0);
            let vanl = van.get(&k).and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0);
            if modl == vanl { same_len += 1 } else {
                diff_len += 1;
                if examples.len() < 8 {
                    examples.push(format!("{} (key {k}): mod {modl} elems vs vanilla {vanl}",
                        i["entry"].as_str().unwrap_or("?")));
                }
            }
            for pd in i["new"].as_array().into_iter().flatten() {
                for v in pd["equip_slot_list"].as_array().into_iter().flatten() {
                    if let Some(n) = v.as_u64() { if !slots_live.contains(&n) { bad_slot += 1; } }
                }
            }
        }
    }
    eprintln!("items where mod list length == vanilla: {same_len}");
    eprintln!("items where mod list length DIFFERS  : {diff_len}");
    for e in &examples { eprintln!("   {e}"); }
    eprintln!("equip_slot values written that v16 never uses: {bad_slot}");
}

/// Rebuild Equip All against v16 vanilla: for every item, START from the LIVE v16
/// prefab_data_list (so nothing v16 added is reverted) and merge in only the
/// cross-character grants the mod adds (extra tribe_gender_list entries, matched
/// per element by prefab_names). This preserves the mod's purpose without writing
/// stale v15 structure back over v16 content.
#[test]
#[ignore]
fn rebuild_equipall_for_v16() {
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("iteminfo.pabgb")).expect("iteminfo");
    let pabgh = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &body, pabgh.as_deref())
        .expect("parse");
    let mut van: std::collections::HashMap<u64, serde_json::Value> = Default::default();
    for r in &arr {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        if let Some(p) = o.get("prefab_data_list") { van.insert(k, p.clone()); }
    }
    let p = PathBuf::from(MODS)
        .join(r"Equip_Everything_V7_2571_7.2_2026-07-19T02-47Z_XOtFiw9Pm\Equip Everything V7.2.json");
    let mut raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&p).expect("mod")).expect("json");
    let key_of = |pd: &serde_json::Value| -> String {
        pd["prefab_names"].as_array().map(|a| a.iter()
            .filter_map(|x| x.as_u64()).map(|x| x.to_string()).collect::<Vec<_>>().join(","))
            .unwrap_or_default()
    };
    let (mut rebuilt, mut merged, mut nokey) = (0usize, 0usize, 0usize);
    for t in raw["targets"].as_array_mut().into_iter().flatten() {
        if !t["file"].as_str().unwrap_or("").contains("iteminfo") { continue; }
        for i in t["intents"].as_array_mut().into_iter().flatten() {
            let Some(k) = i["key"].as_u64() else { continue };
            let Some(vlist) = van.get(&k).and_then(|v| v.as_array()).cloned() else { nokey += 1; continue };
            // index the mod's elements by prefab_names
            let mut modmap: std::collections::HashMap<String, serde_json::Value> = Default::default();
            for pd in i["new"].as_array().into_iter().flatten() {
                modmap.insert(key_of(pd), pd.clone());
            }
            let mut out = Vec::with_capacity(vlist.len());
            for vpd in &vlist {
                let mut e = vpd.clone();
                if let Some(mpd) = modmap.get(&key_of(vpd)) {
                    // union the grant lists, keep every other v16 field as-is
                    for f in ["tribe_gender_list", "equip_slot_list"] {
                        let mut set: Vec<u64> = e[f].as_array().into_iter().flatten()
                            .filter_map(|x| x.as_u64()).collect();
                        for x in mpd[f].as_array().into_iter().flatten().filter_map(|x| x.as_u64()) {
                            if !set.contains(&x) { set.push(x); merged += 1; }
                        }
                        e[f] = serde_json::json!(set);
                    }
                }
                out.push(e);
            }
            i["new"] = serde_json::Value::Array(out);
            rebuilt += 1;
        }
    }
    raw["modinfo"]["title"] = serde_json::json!("ZZ TEST EquipAll v16-REBUILT");
    raw["modinfo"]["version"] = serde_json::json!("7.3-v16");
    let out_dir = PathBuf::from(MODS).join("ZZ TEST EquipAll v16-REBUILT");
    std::fs::create_dir_all(&out_dir).unwrap();
    std::fs::write(out_dir.join("ZZ TEST EquipAll v16-REBUILT.json"),
        serde_json::to_vec_pretty(&raw).unwrap()).unwrap();
    eprintln!("rebuilt {rebuilt} item intents from v16 vanilla; {merged} grant values merged; {nokey} item(s) not in v16");
    eprintln!("wrote {}", out_dir.display());
}

/// Ship-gate for a single mod file given by DMM_VERIFY_MOD: every intent must
/// apply, the applied table must re-parse, and no prefab_data_list may be SHORTER
/// than v16 vanilla (the v7.2 defect that crashed the save-select screen).
#[test]
#[ignore]
fn verify_one_mod() {
    let Ok(modpath) = std::env::var("DMM_VERIFY_MOD") else { eprintln!("set DMM_VERIFY_MOD"); return };
    let dir = fixture_dir();
    let doc = IntentDoc::from_slice(&std::fs::read(&modpath).expect("mod")).expect("doc");

    // v16 vanilla prefab_data_list lengths
    let body = std::fs::read(dir.join("iteminfo.pabgb")).unwrap();
    let ph = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &body, ph.as_deref()).unwrap();
    let mut vlen: std::collections::HashMap<u64, usize> = Default::default();
    for r in &arr {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        vlen.insert(k, o.get("prefab_data_list").and_then(|x| x.as_array()).map(|a| a.len()).unwrap_or(0));
    }
    let raw: serde_json::Value = serde_json::from_slice(&std::fs::read(&modpath).unwrap()).unwrap();
    let mut shorter = 0usize;
    for t in raw["targets"].as_array().into_iter().flatten() {
        if !t["file"].as_str().unwrap_or("").contains("iteminfo") { continue; }
        for i in t["intents"].as_array().into_iter().flatten() {
            let (Some(k), Some(n)) = (i["key"].as_u64(), i["new"].as_array().map(|a| a.len())) else { continue };
            if let Some(v) = vlen.get(&k) { if n < *v { shorter += 1; } }
        }
    }
    eprintln!("RESULT prefab lists SHORTER than v16 vanilla: {shorter}");

    let (mut applied, mut skipped, mut unstable) = (0usize, 0usize, 0usize);
    for (target, intents) in doc.flatten_targets() {
        let Some(fx) = find_fixture(&dir, &target) else { continue };
        let b = std::fs::read(&fx).unwrap();
        let h = std::fs::read(fx.with_extension("pabgh")).ok();
        let (nb, nh, out) = apply_intents_to_table_body(&target, &b, h.as_deref(), &intents).expect("apply");
        applied += out.iter().filter(|o| matches!(o.status, ApplyStatus::Applied)).count();
        skipped += out.iter().filter(|o| !matches!(o.status, ApplyStatus::Applied)).count();
        match apply_intents_to_table_body(&target, &nb, nh.as_deref(), &[]) {
            Ok((again, _, _)) if again == nb => {}
            _ => unstable += 1,
        }
        eprintln!("RESULT {target}: {} B -> {} B", b.len(), nb.len());
    }
    eprintln!("RESULT applied={applied} skipped={skipped} unstable_tables={unstable}");
    assert_eq!(shorter, 0, "mod would truncate v16 prefab lists");
    assert_eq!(skipped, 0);
    assert_eq!(unstable, 0);
}
