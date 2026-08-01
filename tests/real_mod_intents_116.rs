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
/// Emit a corrected Equip All: drop any intent whose prefab_data_list references a
/// hash that does not exist on v16. Those dead refs are what crash the engine.
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
    // The full v16 prefab-name hash universe + the field set a WELL-FORMED element
    // has, so a mod-only element can be judged instead of trusted.
    let mut van_prefabs: std::collections::HashSet<u64> = Default::default();
    let mut van_fields: Vec<String> = Vec::new();
    for r in &arr {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        if let Some(p) = o.get("prefab_data_list") {
            for e in p.as_array().into_iter().flatten() {
                if van_fields.is_empty() {
                    if let Some(eo) = e.as_object() { van_fields = eo.keys().cloned().collect(); }
                }
                for h in e["prefab_names"].as_array().into_iter().flatten() {
                    if let Some(h) = h.as_u64() { van_prefabs.insert(h); }
                }
            }
            van.insert(k, p.clone());
        }
    }
    eprintln!("v16 vanilla: {} distinct prefab hashes; element fields = {van_fields:?}",
        van_prefabs.len());
    let (mut dropped, mut partial, mut dangling_ct) = (0usize, 0usize, 0usize);
    // Read the RELEASE SOURCE, not the installed folder — the mods dir gets
    // replaced as versions ship, and v7.2 is no longer installed there.
    let p = PathBuf::from(r"C:\Users\justi\Desktop\MyMods\mod_sources\Equip Everything V7.2.json");
    let mut raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&p).expect("v7.2 source")).expect("json");
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
                    // ★ Take the mod's grant lists VERBATIM. Do NOT union them with
                    // vanilla's. The mod's whole mechanism is that an EMPTY
                    // tribe_gender_list = unrestricted; a NON-empty one RESTRICTS.
                    // Unioning re-adds vanilla's gate entries and re-locks the item —
                    // that is what broke Damiane armor in v7.3. Every other field
                    // still comes from v16 vanilla, so patch additions survive.
                    for f in ["tribe_gender_list", "equip_slot_list"] {
                        if !mpd[f].is_null() { e[f] = mpd[f].clone(); merged += 1; }
                    }
                }
                out.push(e);
            }
            // ★ DO NOT append elements the mod has that v16 vanilla does not.
            // v7.4 did, and it CTD'd the game on LAUNCH. Two independent defects:
            //   1. The mod's elements use the OLD sparse capture schema — they carry
            //      only equip_slot_list/is_craft_material/prefab_names/tribe_gender_list
            //      and are MISSING animation_path_list, prefab_data_type, scale and
            //      use_gimmick_prefab. Writing one back produces a malformed record.
            //   2. Their prefab_names are v15-era hashes; see the dangling report below.
            // v7.3 shipped without these 32 and did not crash, so dropping them is the
            // known-good structure. Grants above are what actually makes the mod work.
            let vkeys: Vec<String> = vlist.iter().map(|v| key_of(v)).collect();
            for pd in i["new"].as_array().into_iter().flatten() {
                if vkeys.contains(&key_of(pd)) { continue; }
                dropped += 1;
                let missing: Vec<&str> = van_fields.iter()
                    .filter(|f| pd.get(*f).is_none()).map(|s| s.as_str()).collect();
                let dangling: Vec<u64> = pd["prefab_names"].as_array().into_iter().flatten()
                    .filter_map(|x| x.as_u64()).filter(|h| !van_prefabs.contains(h)).collect();
                if !missing.is_empty() { partial += 1; }
                if !dangling.is_empty() { dangling_ct += 1; }
                eprintln!("DROP key={k} {} missing={missing:?} dangling_prefabs={dangling:?}",
                    i["entry"].as_str().unwrap_or(""));
            }
            i["new"] = serde_json::Value::Array(out);
            rebuilt += 1;
        }
    }
    raw["modinfo"]["title"] = serde_json::json!("Equip All V7.4");
    raw["modinfo"]["version"] = serde_json::json!("7.4");
    let out_dir = PathBuf::from(MODS).join("Equip Everything V7.4");
    std::fs::create_dir_all(&out_dir).unwrap();
    std::fs::write(out_dir.join("Equip Everything V7.4.json"),
        serde_json::to_vec_pretty(&raw).unwrap()).unwrap();
    eprintln!("rebuilt {rebuilt} item intents from v16 vanilla; {merged} grant values merged; {nokey} item(s) not in v16");
    eprintln!("RESULT mod-only elements dropped={dropped} (schema-incomplete={partial}, with dangling prefab hashes={dangling_ct})");
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
    let mut van_prefabs: std::collections::HashSet<u64> = Default::default();
    let mut van_fields: Vec<String> = Vec::new();
    for r in &arr {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        let pl = o.get("prefab_data_list").and_then(|x| x.as_array());
        for e in pl.into_iter().flatten() {
            if van_fields.is_empty() {
                if let Some(eo) = e.as_object() { van_fields = eo.keys().cloned().collect(); }
            }
            for h in e["prefab_names"].as_array().into_iter().flatten() {
                if let Some(h) = h.as_u64() { van_prefabs.insert(h); }
            }
        }
        vlen.insert(k, pl.map(|a| a.len()).unwrap_or(0));
    }
    let raw: serde_json::Value = serde_json::from_slice(&std::fs::read(&modpath).unwrap()).unwrap();
    let (mut shorter, mut partial, mut dangling) = (0usize, 0usize, 0usize);
    for t in raw["targets"].as_array().into_iter().flatten() {
        if !t["file"].as_str().unwrap_or("").contains("iteminfo") { continue; }
        for i in t["intents"].as_array().into_iter().flatten() {
            let (Some(k), Some(list)) = (i["key"].as_u64(), i["new"].as_array()) else { continue };
            if let Some(v) = vlen.get(&k) { if list.len() < *v { shorter += 1; } }
            // ★ Every element the mod writes must be STRUCTURALLY COMPLETE and must
            // reference prefab hashes this build actually has. v7.4 shipped 32
            // elements captured under the old sparse schema (no scale /
            // prefab_data_type / animation_path_list / use_gimmick_prefab) carrying
            // v15 prefab hashes, and the game CTD'd on LAUNCH. Bytes round-tripped
            // and every intent applied, so only this check sees it.
            for e in list {
                let miss: Vec<&str> = van_fields.iter()
                    .filter(|f| e.get(*f).is_none()).map(|s| s.as_str()).collect();
                if !miss.is_empty() {
                    partial += 1;
                    if partial <= 10 {
                        eprintln!("RESULT   PARTIAL key={k} {} missing={miss:?}",
                            i["entry"].as_str().unwrap_or(""));
                    }
                }
                for h in e["prefab_names"].as_array().into_iter().flatten() {
                    let Some(h) = h.as_u64() else { continue };
                    if !van_prefabs.contains(&h) {
                        dangling += 1;
                        if dangling <= 10 {
                            eprintln!("RESULT   DANGLING key={k} {} prefab={h}",
                                i["entry"].as_str().unwrap_or(""));
                        }
                    }
                }
            }
        }
    }
    eprintln!("RESULT prefab lists SHORTER than v16 vanilla: {shorter}");
    eprintln!("RESULT schema-incomplete elements: {partial}");
    eprintln!("RESULT dangling prefab refs: {dangling}");

    let (mut applied, mut skipped, mut unstable) = (0usize, 0usize, 0usize);
    for (target, intents) in doc.flatten_targets() {
        let Some(fx) = find_fixture(&dir, &target) else { continue };
        let b = std::fs::read(&fx).unwrap();
        let h = std::fs::read(fx.with_extension("pabgh")).ok();
        let (nb, nh, out) = apply_intents_to_table_body(&target, &b, h.as_deref(), &intents).expect("apply");
        applied += out.iter().filter(|o| matches!(o.status, ApplyStatus::Applied)).count();
        skipped += out.iter().filter(|o| !matches!(o.status, ApplyStatus::Applied)).count();
        for o in &out {
            if let ApplyStatus::Skipped(why) = &o.status {
                eprintln!("RESULT   SKIP [{}] {}", o.op, why);
            }
        }
        match apply_intents_to_table_body(&target, &nb, nh.as_deref(), &[]) {
            Ok((again, _, _)) if again == nb => {}
            _ => unstable += 1,
        }
        eprintln!("RESULT {target}: {} B -> {} B", b.len(), nb.len());
    }
    eprintln!("RESULT applied={applied} skipped={skipped} unstable_tables={unstable}");
    assert_eq!(shorter, 0, "mod would truncate v16 prefab lists");
    assert_eq!(partial, 0, "mod writes structurally incomplete prefab elements (v7.4 launch-CTD class)");
    assert_eq!(dangling, 0, "mod references prefab hashes absent from this build");
    assert_eq!(skipped, 0);
    assert_eq!(unstable, 0);
}

/// GENERIC patch-day audit: for every `op:set` whose value is a LIST, compare the
/// length the mod writes against the length the CURRENT vanilla table has at the
/// same path. Shorter == the mod deletes what this patch added == the Equip All
/// V7.2 defect class. Set DMM_AUDIT_MOD to a mod json, or leave unset to sweep
/// every mod under the mods folder.
#[test]
#[ignore]
fn audit_wholesale_set_lists() {
    use dmm_parser::intents::get_value_at_path;
    let dir = fixture_dir();
    let mut cache: std::collections::HashMap<String, Vec<serde_json::Value>> = Default::default();

    let mut files: Vec<PathBuf> = Vec::new();
    if let Ok(one) = std::env::var("DMM_AUDIT_MOD") {
        files.push(one.into());
    } else {
        fn walk(d: &Path, out: &mut Vec<PathBuf>) {
            let Ok(rd) = std::fs::read_dir(d) else { return };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() { walk(&p, out) }
                else if p.extension().and_then(|x| x.to_str()) == Some("json") { out.push(p) }
            }
        }
        walk(Path::new(MODS), &mut files);
        walk(Path::new(r"C:\Users\justi\Desktop\MyMods\mod_sources"), &mut files);
    }

    let mut findings: Vec<String> = Vec::new();
    let (mut scanned, mut unreadable, mut compared) = (0usize, 0usize, 0usize);
    for f in files {
        // ★ NEVER skip silently: a sweep that quietly reads nothing reports
        //   "all clear" while having audited nothing at all.
        let bytes = match std::fs::read(&f) {
            Ok(b) => b,
            Err(e) => { unreadable += 1; eprintln!("UNREADABLE {}: {e}", f.display()); continue }
        };
        let Ok(doc) = serde_json::from_slice::<serde_json::Value>(&bytes) else { continue };
        if doc.get("format").and_then(|x| x.as_u64()) != Some(3) { continue }
        let title = doc["modinfo"]["title"].as_str().unwrap_or("?").to_string();
        scanned += 1;
        let Ok(parsed) = IntentDoc::from_slice(&bytes) else { continue };
        for (target, intents) in parsed.flatten_targets() {
            let Some(fx) = find_fixture(&dir, &target) else { continue };
            let recs = cache.entry(target.clone()).or_insert_with(|| {
                let b = std::fs::read(&fx).unwrap_or_default();
                let h = std::fs::read(fx.with_extension("pabgh")).ok();
                dmm_parser::dispatch::parse_table_to_json(&target, &b, h.as_deref()).unwrap_or_default()
            });
            let mut short = 0usize;
            let mut sample = String::new();
            for i in &intents {
                if i.op.as_deref().unwrap_or("set") != "set" { continue }
                let Some(newv) = i.new.as_ref().and_then(|v| v.as_array()) else { continue };
                let Some(field) = i.field.as_deref() else { continue };
                let Some(k) = i.key else { continue };
                let Some(rec) = recs.iter().find(|r| r.get("__key__").or_else(|| r.get("key"))
                    .and_then(|x| x.as_i64()) == Some(k)) else { continue };
                let Ok(cur) = get_value_at_path(rec, field) else { continue };
                let Some(curl) = cur.as_array().map(|a| a.len()) else { continue };
                compared += 1;
                if newv.len() < curl {
                    short += 1;
                    if sample.is_empty() {
                        sample = format!("{field} key={k}: mod {} vs vanilla {curl}", newv.len());
                    }
                }
            }
            if short > 0 {
                findings.push(format!("{title} -> {target}: {short} list(s) SHORTER than v16   [{sample}]"));
            }
        }
    }
    eprintln!("AUDIT COVERAGE: {scanned} mod doc(s), {compared} list(s) compared, {unreadable} unreadable");
    assert!(compared > 0, "audited nothing -- coverage bug, not a clean result");
    if findings.is_empty() {
        eprintln!("AUDIT: no mod writes a list shorter than v16 vanilla");
    } else {
        eprintln!("AUDIT FINDINGS ({}):", findings.len());
        for f in &findings { eprintln!("   {f}"); }
    }
}

/// How broken is interaction_info on v16, and do the records "Fast Pickup" targets
/// (Gimmick_PickUp key 1000004, Gimmick_Collect key 10028) parse or blob-fall-back?
#[test]
#[ignore]
fn interactioninfo_blob_census() {
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("interactioninfo.pabgb")).expect("body");
    let pabgh = std::fs::read(dir.join("interactioninfo.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("interactioninfo.pabgb", &body, pabgh.as_deref())
        .expect("parse");
    let (mut blob, mut ok) = (0usize, 0usize);
    for r in &arr {
        let Some(o) = r.as_object() else { continue };
        if o.contains_key("_blob_b64") { blob += 1 } else { ok += 1 }
    }
    eprintln!("CENSUS interactioninfo: {} records, {} typed, {} BLOB ({:.1}% blob)",
        arr.len(), ok, blob, 100.0 * blob as f64 / arr.len().max(1) as f64);
    for want in [1000004u64, 10028] {
        let hit = arr.iter().find(|r| r.as_object()
            .and_then(|o| o.get("__key__").or_else(|| o.get("key")))
            .and_then(|k| k.as_u64()) == Some(want));
        match hit {
            Some(r) => {
                let o = r.as_object().unwrap();
                let sk = o.get("string_key").and_then(|x| x.as_str()).unwrap_or("?");
                eprintln!("CENSUS key {want}: {}  string_key={sk}",
                    if o.contains_key("_blob_b64") { "BLOB  <-- intents cannot resolve" } else { "typed OK" });
            }
            None => eprintln!("CENSUS key {want}: NOT FOUND in v16"),
        }
    }
}

/// ★ BLOB CENSUS — the gap the V3 harness structurally cannot see.
/// A blob-fallback record round-trips BYTE-PERFECTLY, so `v3_all_tables` reports
/// OK while the table resolves NO field paths and silently drops every intent.
/// That is how interaction_info sat 100% broken with a green harness.
/// Compare typed/blob rates OLD vs NEW: a table whose blob rate JUMPS is a
/// regression that will silently break any mod touching it.
/// Run: DMM_CENSUS_OLD=<old dir> cargo test --test real_mod_intents_116 blob_census_all -- --ignored --nocapture
#[test]
#[ignore]
fn blob_census_all() {
    let newdir = fixture_dir();
    let olddir: PathBuf = std::env::var("DMM_CENSUS_OLD")
        .unwrap_or_else(|_| r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-7-16".into()).into();

    let rate = |dir: &Path, file: &str| -> Option<(usize, usize)> {
        let body = std::fs::read(dir.join(file)).ok()?;
        let pabgh = std::fs::read(dir.join(file).with_extension("pabgh")).ok();
        let arr = dmm_parser::dispatch::parse_table_to_json(file, &body, pabgh.as_deref()).ok()?;
        let blob = arr.iter().filter(|r| r.as_object()
            .map(|o| o.contains_key("_blob_b64")).unwrap_or(false)).count();
        Some((arr.len(), blob))
    };

    let mut files: Vec<String> = std::fs::read_dir(&newdir).expect("fixture dir")
        .flatten()
        .filter_map(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            if n.ends_with(".pabgb") { Some(n) } else { None }
        })
        .collect();
    files.sort();

    let mut worse = Vec::new();
    let mut allblob = Vec::new();
    for f in &files {
        let (Some((no, bo)), Some((nn, bn))) = (rate(&olddir, f), rate(&newdir, f)) else { continue };
        if no == 0 || nn == 0 { continue }
        let po = 100.0 * bo as f64 / no as f64;
        let pn = 100.0 * bn as f64 / nn as f64;
        if pn > po + 5.0 {
            worse.push(format!("{f:<44} blob {po:>5.1}% -> {pn:>5.1}%   ({bn}/{nn} records)"));
        } else if pn > 95.0 {
            allblob.push(format!("{f:<44} blob {pn:>5.1}% on BOTH builds (pre-existing)"));
        }
    }
    eprintln!("\n=== REGRESSED (blob rate jumped on the new build) ===");
    if worse.is_empty() { eprintln!("   none"); }
    for w in &worse { eprintln!("   {w}"); }
    eprintln!("\n=== PRE-EXISTING (already blob before the patch) ===");
    for a in &allblob { eprintln!("   {a}"); }
}

/// itemuseinfo is 43.8% blob on v16. Its records are a variant family keyed by a
/// `disc` byte, so census the BLOB rate PER DISCRIMINATOR: the failing variants
/// name themselves, and the passing ones bound the drift.
#[test]
#[ignore]
fn itemuse_disc_census() {
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("itemuseinfo.pabgb")).expect("body");
    let pabgh = std::fs::read(dir.join("itemuseinfo.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("itemuseinfo.pabgb", &body, pabgh.as_deref())
        .expect("parse");
    // For blob records the disc is not in the JSON, so read it off the raw bytes:
    // key u32 + CString(len u32 + bytes) + is_blocked u8 + disc u8.
    use std::collections::BTreeMap;
    let mut tally: BTreeMap<u8, (usize, usize)> = BTreeMap::new();
    for r in &arr {
        let Some(o) = r.as_object() else { continue };
        let isblob = o.contains_key("_blob_b64");
        let disc = if let Some(d) = o.get("disc").and_then(|x| x.as_u64()) {
            d as u8
        } else if let Some(b64) = o.get("_blob_b64").and_then(|x| x.as_str()) {
            // decode just enough of the blob to reach `disc`
            let raw = b64_decode(b64);
            if raw.len() < 10 { continue }
            let ln = u32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]) as usize;
            let p = 8 + ln;
            if p + 1 >= raw.len() { continue }
            raw[p + 1]
        } else { continue };
        let e = tally.entry(disc).or_insert((0, 0));
        if isblob { e.1 += 1 } else { e.0 += 1 }
    }
    eprintln!("{:<6} {:>8} {:>8}  {}", "disc", "typed", "BLOB", "verdict");
    for (d, (ok, blob)) in &tally {
        let v = if *blob == 0 { "ok" } else if *ok == 0 { "*** ALL BLOB ***" } else { "mixed" };
        eprintln!("{:<6} {:>8} {:>8}  {}", d, ok, blob, v);
    }
}

fn b64_decode(s: &str) -> Vec<u8> {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = Vec::new();
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in s.bytes() {
        if c == b'=' { break }
        let Some(v) = T.iter().position(|&t| t == c) else { continue };
        acc = (acc << 6) | v as u32;
        bits += 6;
        if bits >= 8 { bits -= 8; out.push((acc >> bits) as u8); }
    }
    out
}
