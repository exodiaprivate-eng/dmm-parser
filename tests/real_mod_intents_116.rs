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

/// Read the iteminfo overlay the GAME ACTUALLY LOADED (not the mod json, not a
/// fixture) and report the grant shape. Method rule 1: verify against the artifact
/// the runtime consumes. Point DMM_LIVE_OVERLAY at e.g.
/// "D:\SteamLibrary\steamapps\common\Crimson Desert\dmmv3_iteminfo".
#[test]
#[ignore]
fn inspect_live_overlay() {
    use dmm_parser::binary::{pamt::PackMeta, paz};
    let dirp = std::env::var("DMM_LIVE_OVERLAY")
        .unwrap_or_else(|_| r"D:\SteamLibrary\steamapps\common\Crimson Desert\dmmv3_iteminfo".into());
    let group = PathBuf::from(&dirp);
    let pamt = PackMeta::parse(&std::fs::read(group.join("0.pamt")).expect("0.pamt"), None)
        .expect("pamt");
    let mut body: Option<Vec<u8>> = None;
    let mut pabgh: Option<Vec<u8>> = None;
    for d in &pamt.directories {
        for f in &d.files {
            let raw = paz::extract_file(&group, f, &d.path, &pamt.header.encrypt_info.encrypt_info)
                .expect("extract");
            eprintln!("LIVE file: {}/{}  {} B", d.path, f.name, raw.len());
            if f.name.ends_with(".pabgb") { body = Some(raw); }
            else if f.name.ends_with(".pabgh") { pabgh = Some(raw); }
        }
    }
    let body = body.expect("no .pabgb in overlay");
    let arr = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &body, pabgh.as_deref())
        .expect("parse live overlay");

    // vanilla for comparison
    let dir = fixture_dir();
    let vb = std::fs::read(dir.join("iteminfo.pabgb")).unwrap();
    let vh = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let varr = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &vb, vh.as_deref()).unwrap();

    let shape = |a: &Vec<serde_json::Value>| {
        let (mut elems, mut open, mut prefabs) = (0usize, 0usize, std::collections::HashSet::new());
        for r in a {
            for e in r.get("prefab_data_list").and_then(|x| x.as_array()).into_iter().flatten() {
                elems += 1;
                if e["tribe_gender_list"].as_array().map(|l| l.is_empty()).unwrap_or(false) { open += 1; }
                for h in e["prefab_names"].as_array().into_iter().flatten() {
                    if let Some(h) = h.as_u64() { prefabs.insert(h); }
                }
            }
        }
        (a.len(), elems, open, prefabs)
    };
    let (vr, ve, vo, vp) = shape(&varr);
    let (lr, le, lo, lp) = shape(&arr);
    eprintln!("VANILLA v16 : records={vr} elements={ve} open(empty tribe_gender)={vo} distinct_prefabs={}", vp.len());
    eprintln!("LIVE overlay: records={lr} elements={le} open(empty tribe_gender)={lo} distinct_prefabs={}", lp.len());
    let dangling: Vec<u64> = lp.difference(&vp).copied().collect();
    eprintln!("LIVE prefab hashes NOT in v16 vanilla: {} {:?}", dangling.len(),
        &dangling.iter().take(8).collect::<Vec<_>>());
    eprintln!("=> extra elements vs vanilla: {}", le as i64 - ve as i64);
}

/// Dump v16 vanilla + the LIVE overlay prefab_data_list side by side to JSON so the
/// exact set of elements the mod OPENS can be characterised without re-parsing.
#[test]
#[ignore]
fn dump_prefab_shapes() {
    use dmm_parser::binary::{pamt::PackMeta, paz};
    let dir = fixture_dir();
    let grab = |body: &[u8], h: Option<&[u8]>| {
        dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", body, h).expect("parse")
    };
    let vb = std::fs::read(dir.join("iteminfo.pabgb")).unwrap();
    let vh = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let varr = grab(&vb, vh.as_deref());

    let group = PathBuf::from(std::env::var("DMM_LIVE_OVERLAY")
        .unwrap_or_else(|_| r"D:\SteamLibrary\steamapps\common\Crimson Desert\dmmv3_iteminfo".into()));
    let pamt = PackMeta::parse(&std::fs::read(group.join("0.pamt")).unwrap(), None).unwrap();
    let (mut lb, mut lh) = (None, None);
    for d in &pamt.directories { for f in &d.files {
        let raw = paz::extract_file(&group, f, &d.path, &pamt.header.encrypt_info.encrypt_info).unwrap();
        if f.name.ends_with(".pabgb") { lb = Some(raw) } else if f.name.ends_with(".pabgh") { lh = Some(raw) }
    }}
    let larr = grab(&lb.unwrap(), lh.as_deref());

    let slim = |a: &Vec<serde_json::Value>| -> serde_json::Value {
        let mut out = serde_json::Map::new();
        for r in a {
            let Some(o) = r.as_object() else { continue };
            let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
            let els: Vec<serde_json::Value> = o.get("prefab_data_list").and_then(|x| x.as_array())
                .into_iter().flatten().map(|e| serde_json::json!({
                    "p": e["prefab_names"], "t": e["tribe_gender_list"],
                    "s": e["equip_slot_list"], "dt": e["prefab_data_type"],
                })).collect();
            out.insert(k.to_string(), serde_json::json!({
                "name": o.get("string_key").cloned().unwrap_or(serde_json::Value::Null), "e": els }));
        }
        serde_json::Value::Object(out)
    };
    let dst = PathBuf::from(std::env::var("DMM_DUMP_DIR").unwrap_or_else(|_| ".".into()));
    std::fs::write(dst.join("vanilla_prefabs.json"), serde_json::to_vec(&slim(&varr)).unwrap()).unwrap();
    std::fs::write(dst.join("live_prefabs.json"), serde_json::to_vec(&slim(&larr)).unwrap()).unwrap();
    eprintln!("wrote vanilla_prefabs.json + live_prefabs.json to {}", dst.display());
}

/// Controlled A/B of two mod versions at the OVERLAY level: apply each mod's
/// intents to v16 vanilla, then diff the resulting prefab_data_list element states.
/// The mod json diff is misleading because `op:set` replaces whole lists and the
/// mod only covers a subset of items — this compares what the ENGINE would read.
/// DMM_MOD_A / DMM_MOD_B.
#[test]
#[ignore]
fn ab_overlay_diff() {
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("iteminfo.pabgb")).unwrap();
    let ph = std::fs::read(dir.join("iteminfo.pabgh")).ok();

    let build = |p: &str| -> Vec<serde_json::Value> {
        let doc = IntentDoc::from_slice(&std::fs::read(p).expect("mod")).expect("doc");
        let mut b = body.clone();
        let mut h = ph.clone();
        for (target, intents) in doc.flatten_targets() {
            if !target.contains("iteminfo") { continue }
            let (nb, nh, _) = apply_intents_to_table_body(&target, &b, h.as_deref(), &intents)
                .expect("apply");
            b = nb; h = nh;
        }
        dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &b, h.as_deref()).expect("parse")
    };
    let pa = std::env::var("DMM_MOD_A").expect("DMM_MOD_A");
    let pb = std::env::var("DMM_MOD_B").expect("DMM_MOD_B");
    // "VANILLA" means: apply nothing, i.e. compare a mod against the stock table.
    let vanilla_json = || dmm_parser::dispatch::parse_table_to_json(
        "iteminfo.pabgb", &body, ph.as_deref()).expect("parse");
    let aa = if pa == "VANILLA" { vanilla_json() } else { build(&pa) };
    let bb = if pb == "VANILLA" { vanilla_json() } else { build(&pb) };
    let van = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &body, ph.as_deref()).unwrap();

    let idx = |a: &Vec<serde_json::Value>| {
        let mut m: std::collections::HashMap<u64, Vec<bool>> = Default::default();
        let mut n: std::collections::HashMap<u64, String> = Default::default();
        for r in a {
            let Some(o) = r.as_object() else { continue };
            let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
            n.insert(k, o.get("string_key").and_then(|x| x.as_str()).unwrap_or("").to_string());
            m.insert(k, o.get("prefab_data_list").and_then(|x| x.as_array()).into_iter().flatten()
                .map(|e| e["tribe_gender_list"].as_array().map(|l| l.is_empty()).unwrap_or(false))
                .collect());
        }
        (m, n)
    };
    let ((ma, names), (mb, _), (mv, _)) = (idx(&aa), idx(&bb), idx(&van));

    let (mut a_open_b_shut, mut b_open_a_shut) = (0usize, 0usize);
    let mut starved: Vec<(u64, String, usize)> = Vec::new();   // B has ZERO open, A has >=1
    let mut starved_vs_vanilla = 0usize;
    for (k, va) in &ma {
        let (Some(vb), Some(vv)) = (mb.get(k), mv.get(k)) else { continue };
        if va.len() != vb.len() { continue }
        for (x, y) in va.iter().zip(vb.iter()) {
            if *x && !*y { a_open_b_shut += 1 }
            if *y && !*x { b_open_a_shut += 1 }
        }
        let (oa, ob, ov) = (va.iter().filter(|b| **b).count(),
                            vb.iter().filter(|b| **b).count(),
                            vv.iter().filter(|b| **b).count());
        if ob == 0 && oa > 0 { starved.push((*k, names[k].clone(), va.len())); }
        if ob == 0 && ov > 0 { starved_vs_vanilla += 1; }
    }
    eprintln!("A = {pa}\nB = {pb}");
    eprintln!("elements open in A but RESTRICTED in B: {a_open_b_shut}");
    eprintln!("elements open in B but RESTRICTED in A: {b_open_a_shut}");
    eprintln!("★ items with ZERO open elements in B but >0 in A      : {}", starved.len());
    eprintln!("★ items with ZERO open elements in B but >0 in VANILLA: {starved_vs_vanilla}");
    for s in starved.iter().take(15) { eprintln!("    starved: {} (key={} elems={})", s.1, s.0, s.2); }
}

/// Audit the equipslotinfo half of an Equip All build against the VALIDATED-SAFE
/// recipe: a slot_index-matched etl_hashes union over ONLY the 3 player classes
/// (Kliff 1, Damiane 4, Oongka 6). Unioning across the NPC/BOSS classes is what
/// broke the mod historically -- their slot_index means something different, so
/// armor/offhand types leak into weapon slots. DMM_VERIFY_MOD.
#[test]
#[ignore]
fn audit_equipslot_intents() {
    const PLAYERS: [u64; 3] = [1, 4, 6];
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("equipslotinfo.pabgb")).unwrap();
    let ph = std::fs::read(dir.join("equipslotinfo.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("equipslotinfo.pabgb", &body, ph.as_deref())
        .expect("parse");
    // (class_key, slot_position) -> etl set ; and per position the union over players
    let mut van: std::collections::HashMap<(u64, usize), Vec<u64>> = Default::default();
    let mut player_pos: std::collections::HashMap<usize, std::collections::HashSet<u64>> = Default::default();
    let mut classes: Vec<u64> = Vec::new();
    for r in &arr {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        classes.push(k);
        for (i, e) in o.get("entries").and_then(|x| x.as_array()).into_iter().flatten().enumerate() {
            let set: Vec<u64> = e["etl_hashes"].as_array().into_iter().flatten()
                .filter_map(|x| x.as_u64()).collect();
            if PLAYERS.contains(&k) { player_pos.entry(i).or_default().extend(set.iter().copied()); }
            van.insert((k, i), set);
        }
    }
    classes.sort();
    eprintln!("equipslotinfo classes on v16: {classes:?}");

    let modpath = std::env::var("DMM_VERIFY_MOD").expect("DMM_VERIFY_MOD");
    let raw: serde_json::Value = serde_json::from_slice(&std::fs::read(&modpath).unwrap()).unwrap();
    let (mut n, mut nonplayer, mut removes, mut foreign) = (0usize, 0usize, 0usize, 0usize);
    for t in raw["targets"].as_array().into_iter().flatten() {
        if !t["file"].as_str().unwrap_or("").contains("equipslot") { continue }
        for i in t["intents"].as_array().into_iter().flatten() {
            n += 1;
            let key = i["key"].as_u64().unwrap_or(u64::MAX);
            let field = i["field"].as_str().unwrap_or("");
            let pos: usize = field.trim_start_matches("entries[").split(']').next()
                .and_then(|s| s.parse().ok()).unwrap_or(usize::MAX);
            let new: Vec<u64> = i["new"].as_array().into_iter().flatten()
                .filter_map(|x| x.as_u64()).collect();
            let v = van.get(&(key, pos)).cloned().unwrap_or_default();
            let added: Vec<u64> = new.iter().filter(|h| !v.contains(h)).copied().collect();
            let dropped: Vec<u64> = v.iter().filter(|h| !new.contains(h)).copied().collect();
            let empty = std::collections::HashSet::new();
            let allowed = player_pos.get(&pos).unwrap_or(&empty);
            let leak: Vec<u64> = added.iter().filter(|h| !allowed.contains(h)).copied().collect();
            if !PLAYERS.contains(&key) { nonplayer += 1 }
            if !dropped.is_empty() { removes += 1 }
            if !leak.is_empty() { foreign += 1 }
            eprintln!("RESULT class={key} slot_pos={pos}: vanilla={} -> new={} added={:?}{}{}",
                v.len(), new.len(), added,
                if dropped.is_empty() { String::new() } else { format!(" DROPPED={dropped:?}") },
                if leak.is_empty() { String::new() } else { format!("  ★FOREIGN(not in any player class at this slot)={leak:?}") });
        }
    }
    eprintln!("RESULT equipslot intents={n} targeting_non_player_class={nonplayer} removing_vanilla_types={removes} foreign_type_leaks={foreign}");
}

/// Repair the equipslotinfo half for v16: keep the SAME (class, slot) targets the
/// mod already ships -- the validated 3-player slot-matched set -- but union each
/// with v16 vanilla so a stale `op:set` cannot DELETE an equip type this patch
/// added. Minimal by construction: the result is a superset of both the mod's
/// current grants and vanilla, so nothing the mod granted is lost and nothing
/// vanilla has is dropped. DMM_VERIFY_MOD in, DMM_MOD_OUT out.
#[test]
#[ignore]
fn rebuild_equipslot_for_v16() {
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("equipslotinfo.pabgb")).unwrap();
    let ph = std::fs::read(dir.join("equipslotinfo.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("equipslotinfo.pabgb", &body, ph.as_deref())
        .expect("parse");
    let mut van: std::collections::HashMap<(u64, usize), Vec<u64>> = Default::default();
    for r in &arr {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        for (i, e) in o.get("entries").and_then(|x| x.as_array()).into_iter().flatten().enumerate() {
            van.insert((k, i), e["etl_hashes"].as_array().into_iter().flatten()
                .filter_map(|x| x.as_u64()).collect());
        }
    }
    let modpath = std::env::var("DMM_VERIFY_MOD").expect("DMM_VERIFY_MOD");
    let mut raw: serde_json::Value = serde_json::from_slice(&std::fs::read(&modpath).unwrap()).unwrap();
    let mut restored = 0usize;
    for t in raw["targets"].as_array_mut().into_iter().flatten() {
        if !t["file"].as_str().unwrap_or("").contains("equipslot") { continue }
        for i in t["intents"].as_array_mut().into_iter().flatten() {
            let key = i["key"].as_u64().unwrap_or(u64::MAX);
            let pos: usize = i["field"].as_str().unwrap_or("").trim_start_matches("entries[")
                .split(']').next().and_then(|s| s.parse().ok()).unwrap_or(usize::MAX);
            let mut new: Vec<u64> = i["new"].as_array().into_iter().flatten()
                .filter_map(|x| x.as_u64()).collect();
            for h in van.get(&(key, pos)).into_iter().flatten() {
                if !new.contains(h) { new.push(*h); restored += 1; }
            }
            i["new"] = serde_json::Value::Array(
                new.into_iter().map(|h| serde_json::json!(h)).collect());
        }
    }
    let out = std::env::var("DMM_MOD_OUT").expect("DMM_MOD_OUT");
    std::fs::write(&out, serde_json::to_vec_pretty(&raw).unwrap()).unwrap();
    eprintln!("RESULT restored {restored} vanilla equip type(s) the stale op:set would have deleted");
    eprintln!("wrote {out}");
}

/// Print iteminfo records in PARSE ORDER. AutoLoot's gimmick_block.h indexes its
/// tables by the engine's manager array index, and the generator assumes parse
/// order == manager index. Verify that assumption differentially against the build
/// the probes were generated on before regenerating anything.
#[test]
#[ignore]
fn dump_item_order() {
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("iteminfo.pabgb")).unwrap();
    let ph = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &body, ph.as_deref())
        .expect("parse");
    eprintln!("RESULT dir={} records={}", dir.display(), arr.len());
    for (i, r) in arr.iter().take(6).enumerate() {
        eprintln!("RESULT   idx {i}: {}", r.get("string_key").and_then(|x| x.as_str()).unwrap_or("?"));
    }
}

/// Dump v16 gimmick/puzzle item string_keys (item_type == 102) so AutoLoot's
/// gimmick_block.h can be diffed against the live build. The Visione_Chip_ half of
/// the filter is a prefix test and needs no regeneration; this explicit list does.
#[test]
#[ignore]
fn dump_gimmick_keys() {
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("iteminfo.pabgb")).unwrap();
    let ph = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &body, ph.as_deref()).unwrap();
    if let Some(o) = arr.first().and_then(|r| r.as_object()) {
        let mut ks: Vec<&String> = o.keys().filter(|k| k.contains("type") || k.contains("item")).collect();
        ks.sort();
        eprintln!("candidate type fields: {ks:?}");
    }
    let field = std::env::var("DMM_TYPE_FIELD").unwrap_or_else(|_| "item_type".into());
    let mut out: Vec<String> = Vec::new();
    let mut vis = 0usize;
    for r in &arr {
        let Some(o) = r.as_object() else { continue };
        let sk = o.get("string_key").and_then(|x| x.as_str()).unwrap_or("");
        if sk.starts_with("Visione_Chip_") { vis += 1; }
        if o.get(&field).and_then(|x| x.as_u64()) == Some(102) { out.push(sk.to_string()); }
    }
    out.sort();
    eprintln!("RESULT v16 item_type==102 keys: {}", out.len());
    eprintln!("RESULT v16 Visione_Chip_* items: {vis}");
    let dst = PathBuf::from(std::env::var("DMM_DUMP_DIR").unwrap_or_else(|_| ".".into()))
        .join("v16_gimmick_keys.json");
    std::fs::write(&dst, serde_json::to_vec(&out).unwrap()).unwrap();
    eprintln!("wrote {}", dst.display());
}

/// Diagnose SPECIFIC items the user reports as "Unequippable" in-game: dump their
/// equip_type, their per-element tribe_gender lists, and which player class/slot
/// whitelists that equip_type on v16. DMM_ITEMS="Matana,Ashad,Golden_Greed".
#[test]
#[ignore]
fn diagnose_unequippable() {
    const PLAYERS: [u64; 3] = [1, 4, 6];
    let dir = fixture_dir();
    let ib = std::fs::read(dir.join("iteminfo.pabgb")).unwrap();
    let ih = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let items = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &ib, ih.as_deref()).unwrap();
    let eb = std::fs::read(dir.join("equipslotinfo.pabgb")).unwrap();
    let eh = std::fs::read(dir.join("equipslotinfo.pabgh")).ok();
    let slots = dmm_parser::dispatch::parse_table_to_json("equipslotinfo.pabgb", &eb, eh.as_deref()).unwrap();

    // equip_type -> list of (class, slot_pos) that whitelist it
    let mut where_ok: std::collections::HashMap<u64, Vec<(u64, usize)>> = Default::default();
    for r in &slots {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        for (i, e) in o.get("entries").and_then(|x| x.as_array()).into_iter().flatten().enumerate() {
            for h in e["etl_hashes"].as_array().into_iter().flatten().filter_map(|x| x.as_u64()) {
                where_ok.entry(h).or_default().push((k, i));
            }
        }
    }
    let pats: Vec<String> = std::env::var("DMM_ITEMS").unwrap_or_default()
        .split(',').map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty()).collect();
    for r in &items {
        let Some(o) = r.as_object() else { continue };
        let sk = o.get("string_key").and_then(|x| x.as_str()).unwrap_or("");
        let lo = sk.to_lowercase();
        if !pats.iter().any(|p| lo.contains(p)) { continue }
        let et = o.get("equip_type_info").and_then(|x| x.as_u64());
        eprintln!("RESULT === {sk}  key={:?} equip_type={et:?}",
            o.get("__key__").or_else(|| o.get("key")));
        for (i, e) in o.get("prefab_data_list").and_then(|x| x.as_array()).into_iter().flatten().enumerate() {
            eprintln!("RESULT     elem[{i}] tribe_gender={} equip_slot={} prefab={}",
                e["tribe_gender_list"], e["equip_slot_list"], e["prefab_names"]);
        }
        match et.and_then(|t| where_ok.get(&t)) {
            None => eprintln!("RESULT     equip_type whitelisted by: NOBODY"),
            Some(v) => {
                let players: Vec<String> = v.iter().filter(|(c, _)| PLAYERS.contains(c))
                    .map(|(c, s)| format!("class{c}/slot{s}")).collect();
                eprintln!("RESULT     equip_type whitelisted by {} entries; PLAYER classes: {:?}",
                    v.len(), players);
            }
        }
    }
}

/// Find which characterinfo field carries the tribe_gender hash, by looking for the
/// values observed on Damiane-locked vs Kliff-locked gear inside the player records.
#[test]
#[ignore]
fn find_tribe_gender_source() {
    const DAM: [u64; 5] = [650024735, 590304724, 4184612308, 2348049478, 2885474193];
    const OTH: [u64; 5] = [4234598676, 2278589063, 3215062603, 335227758, 3500335599];
    let dir = fixture_dir();
    let b = std::fs::read(dir.join("characterinfo.pabgb")).unwrap();
    let h = std::fs::read(dir.join("characterinfo.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("characterinfo.pabgb", &b, h.as_deref()).unwrap();
    for r in &arr {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        if ![1u64, 2, 4, 6].contains(&k) { continue }
        let mut hits: Vec<String> = Vec::new();
        for (f, v) in o {
            if let Some(n) = v.as_u64() {
                if DAM.contains(&n) { hits.push(format!("{f}={n} <DAMIANE-set>")); }
                if OTH.contains(&n) { hits.push(format!("{f}={n} <OTHER-set>")); }
            }
        }
        eprintln!("RESULT char key={k}: {:?}", hits);
    }
    // Which items carry each set on elem[0]? Count them.
    let ib = std::fs::read(dir.join("iteminfo.pabgb")).unwrap();
    let ih = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let items = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &ib, ih.as_deref()).unwrap();
    let (mut dam, mut oth, mut open, mut other_sets) = (0usize, 0usize, 0usize, 0usize);
    for r in &items {
        let Some(e0) = r.get("prefab_data_list").and_then(|x| x.as_array()).and_then(|a| a.first()) else { continue };
        let l: Vec<u64> = e0["tribe_gender_list"].as_array().into_iter().flatten()
            .filter_map(|x| x.as_u64()).collect();
        if l.is_empty() { open += 1 }
        else if l.iter().any(|h| DAM.contains(h)) { dam += 1 }
        else if l.iter().any(|h| OTH.contains(h)) { oth += 1 }
        else { other_sets += 1 }
    }
    eprintln!("RESULT elem[0] classification: open={open} damiane_set={dam} other_set={oth} neither={other_sets}");
}

/// V7.2's format worked on 1.15, so the question is what v16 changed. Compare the
/// per-item distribution of FULLY OPEN (empty tribe_gender_list) elements between
/// vanilla and a mod build: a state vanilla never produces is the prime suspect.
#[test]
#[ignore]
fn open_element_distribution() {
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("iteminfo.pabgb")).unwrap();
    let ph = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let count = |a: &Vec<serde_json::Value>, label: &str| {
        let mut hist: std::collections::BTreeMap<usize, usize> = Default::default();
        let mut multi: Vec<String> = Vec::new();
        for r in a {
            let els = r.get("prefab_data_list").and_then(|x| x.as_array());
            let n = els.into_iter().flatten()
                .filter(|e| e["tribe_gender_list"].as_array().map(|l| l.is_empty()).unwrap_or(false))
                .count();
            *hist.entry(n).or_default() += 1;
            if n >= 2 && multi.len() < 6 {
                multi.push(format!("{} (open={n}/{})",
                    r.get("string_key").and_then(|x| x.as_str()).unwrap_or("?"),
                    els.map(|v| v.len()).unwrap_or(0)));
            }
        }
        eprintln!("RESULT {label}: items by #fully-open elements {hist:?}");
        if !multi.is_empty() { eprintln!("RESULT {label}   e.g. {multi:?}"); }
    };
    let van = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &body, ph.as_deref()).unwrap();
    count(&van, "VANILLA v16");
    if let Ok(p) = std::env::var("DMM_VERIFY_MOD") {
        let doc = IntentDoc::from_slice(&std::fs::read(&p).unwrap()).unwrap();
        let (mut b, mut h) = (body.clone(), ph.clone());
        for (target, intents) in doc.flatten_targets() {
            if !target.contains("iteminfo") { continue }
            let (nb, nh, _) = apply_intents_to_table_body(&target, &b, h.as_deref(), &intents).unwrap();
            b = nb; h = nh;
        }
        let m = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &b, h.as_deref()).unwrap();
        count(&m, "MOD        ");
    }
}

/// V7.6: keep V7.2's PROVEN format (merge the mod's grants onto v16 vanilla
/// structure) but never push an item into a state vanilla itself never produces.
/// v16 vanilla tops out at 3 fully-open prefab elements per item; V7.4 produced 51
/// items with 4..10 -- all multi-variant pet/mount/ring gear whose elements are
/// per-species variants. Any item whose open-count would exceed vanilla's own count
/// for that item is left ENTIRELY at vanilla. DMM_MOD_SRC in, DMM_MOD_OUT out.
#[test]
#[ignore]
fn rebuild_equipall_v76() {
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("iteminfo.pabgb")).unwrap();
    let ph = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &body, ph.as_deref()).unwrap();
    let mut van: std::collections::HashMap<u64, serde_json::Value> = Default::default();
    let mut name: std::collections::HashMap<u64, String> = Default::default();
    for r in &arr {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        name.insert(k, o.get("string_key").and_then(|x| x.as_str()).unwrap_or("").into());
        if let Some(p) = o.get("prefab_data_list") { van.insert(k, p.clone()); }
    }
    let openc = |l: &[serde_json::Value]| l.iter()
        .filter(|e| e["tribe_gender_list"].as_array().map(|a| a.is_empty()).unwrap_or(false)).count();

    // ★ SCOPE: an item whose equip_type no PLAYER class whitelists can never be worn
    // by the player, so editing it is pure damage. For mount/pet gear the
    // tribe_gender list encodes SPECIES (one element per breed, each with its own
    // mesh) -- blanking it makes every breed's armor match every animal. V7.2 was a
    // blanket "empty every tribe_gender_list" sweep and caught 109 such items.
    const PLAYERS: [u64; 3] = [1, 4, 6];
    let eb = std::fs::read(dir.join("equipslotinfo.pabgb")).unwrap();
    let eh = std::fs::read(dir.join("equipslotinfo.pabgh")).ok();
    let slots = dmm_parser::dispatch::parse_table_to_json("equipslotinfo.pabgb", &eb, eh.as_deref()).unwrap();
    let mut player_types: std::collections::HashSet<u64> = Default::default();
    for r in &slots {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        if !PLAYERS.contains(&k) { continue }
        for e in o.get("entries").and_then(|x| x.as_array()).into_iter().flatten() {
            for h in e["etl_hashes"].as_array().into_iter().flatten().filter_map(|x| x.as_u64()) {
                player_types.insert(h);
            }
        }
    }
    let mut item_type: std::collections::HashMap<u64, u64> = Default::default();
    for r in &arr {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        item_type.insert(k, o.get("equip_type_info").and_then(|x| x.as_u64()).unwrap_or(0));
    }
    let mut skipped_scope = 0usize;

    let src = std::env::var("DMM_MOD_SRC").expect("DMM_MOD_SRC");
    let mut raw: serde_json::Value = serde_json::from_slice(&std::fs::read(&src).unwrap()).unwrap();
    let key_of = |pd: &serde_json::Value| -> String {
        pd["prefab_names"].as_array().map(|a| a.iter().filter_map(|x| x.as_u64())
            .map(|x| x.to_string()).collect::<Vec<_>>().join(",")).unwrap_or_default()
    };
    let (mut rebuilt, mut held, mut merged) = (0usize, 0usize, 0usize);
    for t in raw["targets"].as_array_mut().into_iter().flatten() {
        if !t["file"].as_str().unwrap_or("").contains("iteminfo") { continue }
        for i in t["intents"].as_array_mut().into_iter().flatten() {
            let Some(k) = i["key"].as_u64() else { continue };
            let Some(vlist) = van.get(&k).and_then(|v| v.as_array()).cloned() else { continue };
            // Out-of-scope: DROP the intent entirely rather than writing vanilla
            // back. A no-op `op:set` on a nested list is still a patch-day liability
            // -- next patch it would silently revert whatever that patch added.
            if !item_type.get(&k).map(|t| player_types.contains(t)).unwrap_or(false) {
                skipped_scope += 1;
                i["__drop__"] = serde_json::json!(true);
                continue;
            }
            let mut modmap: std::collections::HashMap<String, serde_json::Value> = Default::default();
            for pd in i["new"].as_array().into_iter().flatten() { modmap.insert(key_of(pd), pd.clone()); }
            let mut out = Vec::with_capacity(vlist.len());
            for vpd in &vlist {
                let mut e = vpd.clone();
                if let Some(mpd) = modmap.get(&key_of(vpd)) {
                    for f in ["tribe_gender_list", "equip_slot_list"] {
                        if !mpd[f].is_null() { e[f] = mpd[f].clone(); merged += 1; }
                    }
                }
                out.push(e);
            }
            // ★ Envelope guard: never exceed vanilla's own open-element count.
            if openc(&out) > openc(&vlist) && openc(&vlist) < out.len() && openc(&out) > 3 {
                eprintln!("RESULT HOLD {} ({}): would open {} of {} (vanilla opens {})",
                    name.get(&k).cloned().unwrap_or_default(), k,
                    openc(&out), out.len(), openc(&vlist));
                out = vlist.clone();
                held += 1;
            }
            i["new"] = serde_json::Value::Array(out);
            rebuilt += 1;
        }
    }
    for t in raw["targets"].as_array_mut().into_iter().flatten() {
        if let Some(v) = t["intents"].as_array_mut() {
            v.retain(|i| !i.get("__drop__").and_then(|x| x.as_bool()).unwrap_or(false));
        }
    }
    let ver = std::env::var("DMM_MOD_VER").unwrap_or_else(|_| "7.7".into());
    raw["modinfo"]["title"] = serde_json::json!(format!("Equip All V{ver}"));
    raw["modinfo"]["version"] = serde_json::json!(ver);
    let out = std::env::var("DMM_MOD_OUT").expect("DMM_MOD_OUT");
    std::fs::write(&out, serde_json::to_vec_pretty(&raw).unwrap()).unwrap();
    eprintln!("RESULT rebuilt={rebuilt} merged_grants={merged} items_held_at_vanilla={held} not_player_equippable_skipped={skipped_scope}");
    eprintln!("wrote {out}");
}

/// How much of the mod's item scope is NOT player-equippable at all? An item whose
/// equip_type no PLAYER class (Kliff 1 / Damiane 4 / Oongka 6) whitelists cannot be
/// worn by the player under any edit -- for mount/pet gear the tribe_gender list
/// encodes SPECIES, so blanking it destroys breed selection. DMM_VERIFY_MOD.
#[test]
#[ignore]
fn audit_mod_scope() {
    const PLAYERS: [u64; 3] = [1, 4, 6];
    let dir = fixture_dir();
    let ib = std::fs::read(dir.join("iteminfo.pabgb")).unwrap();
    let ih = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let items = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &ib, ih.as_deref()).unwrap();
    let eb = std::fs::read(dir.join("equipslotinfo.pabgb")).unwrap();
    let eh = std::fs::read(dir.join("equipslotinfo.pabgh")).ok();
    let slots = dmm_parser::dispatch::parse_table_to_json("equipslotinfo.pabgb", &eb, eh.as_deref()).unwrap();
    let mut player_types: std::collections::HashSet<u64> = Default::default();
    for r in &slots {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        if !PLAYERS.contains(&k) { continue }
        for e in o.get("entries").and_then(|x| x.as_array()).into_iter().flatten() {
            for h in e["etl_hashes"].as_array().into_iter().flatten().filter_map(|x| x.as_u64()) {
                player_types.insert(h);
            }
        }
    }
    let mut et: std::collections::HashMap<u64, (u64, String)> = Default::default();
    for r in &items {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        et.insert(k, (o.get("equip_type_info").and_then(|x| x.as_u64()).unwrap_or(0),
                      o.get("string_key").and_then(|x| x.as_str()).unwrap_or("").into()));
    }
    let raw: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("DMM_VERIFY_MOD").expect("DMM_VERIFY_MOD")).unwrap()).unwrap();
    let (mut total, mut out_of_scope) = (0usize, 0usize);
    let mut examples: Vec<String> = Vec::new();
    for t in raw["targets"].as_array().into_iter().flatten() {
        if !t["file"].as_str().unwrap_or("").contains("iteminfo") { continue }
        for i in t["intents"].as_array().into_iter().flatten() {
            let Some(k) = i["key"].as_u64() else { continue };
            total += 1;
            if let Some((ty, nm)) = et.get(&k) {
                if !player_types.contains(ty) {
                    out_of_scope += 1;
                    if examples.len() < 12 { examples.push(nm.clone()); }
                }
            }
        }
    }
    eprintln!("RESULT mod touches {total} items; NOT player-equippable: {out_of_scope}");
    eprintln!("RESULT e.g. {examples:?}");
}

/// Exactly WHICH fields does a build change vs vanilla, and at which element index?
/// No hypothesis -- just the full per-field, per-index delta. DMM_VERIFY_MOD.
#[test]
#[ignore]
fn delta_vs_vanilla() {
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("iteminfo.pabgb")).unwrap();
    let ph = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let van = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &body, ph.as_deref()).unwrap();
    let doc = IntentDoc::from_slice(&std::fs::read(std::env::var("DMM_VERIFY_MOD").unwrap()).unwrap()).unwrap();
    let (mut b, mut h) = (body.clone(), ph.clone());
    for (t, ints) in doc.flatten_targets() {
        if !t.contains("iteminfo") { continue }
        let (nb, nh, _) = apply_intents_to_table_body(&t, &b, h.as_deref(), &ints).unwrap();
        b = nb; h = nh;
    }
    let m = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &b, h.as_deref()).unwrap();
    let mut byfield: std::collections::BTreeMap<String, usize> = Default::default();
    let mut byidx: std::collections::BTreeMap<usize, usize> = Default::default();
    let mut slot_emptied = 0usize;
    let mut ex: Vec<String> = Vec::new();
    for (rv, rm) in van.iter().zip(m.iter()) {
        let (Some(av), Some(am)) = (rv.get("prefab_data_list").and_then(|x| x.as_array()),
                                    rm.get("prefab_data_list").and_then(|x| x.as_array())) else { continue };
        if av.len() != am.len() { *byfield.entry("<LIST LENGTH>".into()).or_default() += 1; continue }
        for (i, (ev, em)) in av.iter().zip(am.iter()).enumerate() {
            let (Some(ov), Some(om)) = (ev.as_object(), em.as_object()) else { continue };
            for (f, v) in ov {
                if om.get(f) != Some(v) {
                    *byfield.entry(f.clone()).or_default() += 1;
                    *byidx.entry(i).or_default() += 1;
                    if f == "equip_slot_list" {
                        let was = v.as_array().map(|a| a.len()).unwrap_or(0);
                        let now = om.get(f).and_then(|x| x.as_array()).map(|a| a.len()).unwrap_or(0);
                        if was > 0 && now == 0 {
                            slot_emptied += 1;
                            if ex.len() < 8 { ex.push(format!("{} elem[{i}] {v} -> {}",
                                rv.get("string_key").and_then(|x| x.as_str()).unwrap_or("?"),
                                om.get(f).unwrap())); }
                        }
                    }
                }
            }
        }
    }
    eprintln!("RESULT changed elements by FIELD: {byfield:?}");
    eprintln!("RESULT changed elements by INDEX: {byidx:?}");
    eprintln!("RESULT ★ equip_slot_list EMPTIED (had a slot, now none): {slot_emptied}");
    for e in &ex { eprintln!("RESULT    {e}"); }
}

/// BISECTION build: apply the mod's tribe_gender grants to a RESTRICTED RANGE of
/// prefab element indices only, leaving every other element byte-identical to
/// vanilla. equip_slot_list is never touched (V7.7 emptied 3 items' slot bindings,
/// which is pure risk and unrelated to the mod's purpose). Scope + envelope guards
/// still apply. DMM_IDX_MIN / DMM_IDX_MAX (inclusive), DMM_MOD_SRC, DMM_MOD_OUT,
/// DMM_MOD_VER.
#[test]
#[ignore]
fn build_bisect() {
    const PLAYERS: [u64; 3] = [1, 4, 6];
    let lo: usize = std::env::var("DMM_IDX_MIN").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    let hi: usize = std::env::var("DMM_IDX_MAX").ok().and_then(|s| s.parse().ok()).unwrap_or(usize::MAX);
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("iteminfo.pabgb")).unwrap();
    let ph = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &body, ph.as_deref()).unwrap();
    let eb = std::fs::read(dir.join("equipslotinfo.pabgb")).unwrap();
    let eh = std::fs::read(dir.join("equipslotinfo.pabgh")).ok();
    let slots = dmm_parser::dispatch::parse_table_to_json("equipslotinfo.pabgb", &eb, eh.as_deref()).unwrap();
    let mut player_types: std::collections::HashSet<u64> = Default::default();
    for r in &slots {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        if !PLAYERS.contains(&k) { continue }
        for e in o.get("entries").and_then(|x| x.as_array()).into_iter().flatten() {
            for h in e["etl_hashes"].as_array().into_iter().flatten().filter_map(|x| x.as_u64()) {
                player_types.insert(h);
            }
        }
    }
    let mut van: std::collections::HashMap<u64, serde_json::Value> = Default::default();
    let mut ity: std::collections::HashMap<u64, u64> = Default::default();
    for r in &arr {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        ity.insert(k, o.get("equip_type_info").and_then(|x| x.as_u64()).unwrap_or(0));
        if let Some(p) = o.get("prefab_data_list") { van.insert(k, p.clone()); }
    }
    let openc = |l: &[serde_json::Value]| l.iter()
        .filter(|e| e["tribe_gender_list"].as_array().map(|a| a.is_empty()).unwrap_or(false)).count();
    let key_of = |pd: &serde_json::Value| -> String {
        pd["prefab_names"].as_array().map(|a| a.iter().filter_map(|x| x.as_u64())
            .map(|x| x.to_string()).collect::<Vec<_>>().join(",")).unwrap_or_default()
    };
    let mut raw: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("DMM_MOD_SRC").unwrap()).unwrap()).unwrap();
    let (mut changed, mut held, mut scope) = (0usize, 0usize, 0usize);
    for t in raw["targets"].as_array_mut().into_iter().flatten() {
        if !t["file"].as_str().unwrap_or("").contains("iteminfo") { continue }
        for i in t["intents"].as_array_mut().into_iter().flatten() {
            let Some(k) = i["key"].as_u64() else { continue };
            let Some(vlist) = van.get(&k).and_then(|v| v.as_array()).cloned() else { continue };
            if !ity.get(&k).map(|t| player_types.contains(t)).unwrap_or(false) {
                scope += 1; i["__drop__"] = serde_json::json!(true); continue;
            }
            let mut modmap: std::collections::HashMap<String, serde_json::Value> = Default::default();
            for pd in i["new"].as_array().into_iter().flatten() { modmap.insert(key_of(pd), pd.clone()); }
            let mut out = Vec::with_capacity(vlist.len());
            for (n, vpd) in vlist.iter().enumerate() {
                let mut e = vpd.clone();
                if n >= lo && n <= hi {
                    if let Some(mpd) = modmap.get(&key_of(vpd)) {
                        // tribe_gender ONLY -- never equip_slot_list
                        if !mpd["tribe_gender_list"].is_null()
                            && e["tribe_gender_list"] != mpd["tribe_gender_list"] {
                            e["tribe_gender_list"] = mpd["tribe_gender_list"].clone();
                            changed += 1;
                        }
                    }
                }
                out.push(e);
            }
            if openc(&out) > openc(&vlist) && openc(&out) > 3 { out = vlist.clone(); held += 1; }
            if out == vlist { i["__drop__"] = serde_json::json!(true); continue; }
            i["new"] = serde_json::Value::Array(out);
        }
    }
    for t in raw["targets"].as_array_mut().into_iter().flatten() {
        if let Some(v) = t["intents"].as_array_mut() {
            v.retain(|i| !i.get("__drop__").and_then(|x| x.as_bool()).unwrap_or(false));
        }
    }
    let ver = std::env::var("DMM_MOD_VER").unwrap_or_else(|_| "7.8".into());
    raw["modinfo"]["title"] = serde_json::json!(format!("Equip All V{ver}"));
    raw["modinfo"]["version"] = serde_json::json!(ver);
    let counts: Vec<String> = raw["targets"].as_array().unwrap().iter()
        .map(|t| format!("{}={}", t["file"].as_str().unwrap_or("?"),
             t["intents"].as_array().map(|a| a.len()).unwrap_or(0))).collect();
    let out = std::env::var("DMM_MOD_OUT").unwrap();
    std::fs::write(&out, serde_json::to_vec_pretty(&raw).unwrap()).unwrap();
    eprintln!("RESULT idx[{lo}..={}] elements_changed={changed} held={held} out_of_scope_dropped={scope} intents {counts:?}",
        if hi == usize::MAX { "end".into() } else { hi.to_string() });
    eprintln!("wrote {out}");
}

/// What is prefab_data_type, and does the equip gate only care about ONE type?
/// Cross-tabulate type against "is the tribe_gender list empty" -- if the open
/// elements are overwhelmingly one type and the restricted ones another, then the
/// two are different KINDS of element and must not be treated interchangeably.
#[test]
#[ignore]
fn prefab_type_census() {
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("iteminfo.pabgb")).unwrap();
    let ph = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &body, ph.as_deref()).unwrap();
    let mut cross: std::collections::BTreeMap<(u64, bool), usize> = Default::default();
    let mut byidx: std::collections::BTreeMap<(usize, u64), usize> = Default::default();
    for r in &arr {
        for (i, e) in r.get("prefab_data_list").and_then(|x| x.as_array()).into_iter().flatten().enumerate() {
            let t = e["prefab_data_type"].as_u64().unwrap_or(999);
            let open = e["tribe_gender_list"].as_array().map(|a| a.is_empty()).unwrap_or(false);
            *cross.entry((t, open)).or_default() += 1;
            if i < 3 { *byidx.entry((i, t)).or_default() += 1; }
        }
    }
    eprintln!("RESULT (prefab_data_type, tribe_gender_EMPTY) -> count:");
    for ((t, o), n) in &cross { eprintln!("RESULT    type={t} empty={o}: {n}"); }
    eprintln!("RESULT (element index, type) -> count:");
    for ((i, t), n) in &byidx { eprintln!("RESULT    idx={i} type={t}: {n}"); }

    // Restrict the access question to type-0 elements only.
    const F31: [(&str, u64); 3] = [("Kliff", 4234598676), ("Damiane", 650024735), ("Oongka", 2278589063)];
    for (who, h) in F31 {
        let (mut none, mut one, mut many) = (0usize, 0usize, 0usize);
        for r in &arr {
            let n = r.get("prefab_data_list").and_then(|x| x.as_array()).into_iter().flatten()
                .filter(|e| e["prefab_data_type"].as_u64() == Some(0))
                .filter(|e| {
                    let l = e["tribe_gender_list"].as_array();
                    l.map(|a| a.is_empty() || a.iter().any(|x| x.as_u64() == Some(h))).unwrap_or(false)
                }).count();
            match n { 0 => none += 1, 1 => one += 1, _ => many += 1 }
        }
        eprintln!("RESULT {who:8} type-0 elements matching: none={none} exactly_one={one} multiple={many}");
    }
}

/// ★★★ Replicate the ENGINE'S OWN load-time validation. game_launcher.log:
///   [characterinfo(4)]: 착용할 수 없는 아이템이 세팅됐습니다! ItemKey(Tynion_Giant_TwoHandGiantBastard)
///   checkValid 중 실패했습니다. InfoManagerType : Character
///   -> StaticInfoGroup LoadXml 실패 -> 게임데이터 로딩 실패  (hard CTD at launch)
/// Every item preset in characterinfo.equip_item_info_list MUST remain equippable by
/// that character: its tribe_gender (characterinfo.f31) must match a type-0
/// prefab element. A mod that REVOKES access to a preset item bricks the game load.
/// DMM_VERIFY_MOD (omit to validate vanilla itself).
#[test]
#[ignore]
fn engine_checkvalid_gate() {
    let dir = fixture_dir();
    let ib = std::fs::read(dir.join("iteminfo.pabgb")).unwrap();
    let ih = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let (mut b, mut h) = (ib.clone(), ih.clone());
    if let Ok(p) = std::env::var("DMM_VERIFY_MOD") {
        let doc = IntentDoc::from_slice(&std::fs::read(&p).unwrap()).unwrap();
        for (t, ints) in doc.flatten_targets() {
            if !t.contains("iteminfo") { continue }
            let (nb, nh, _) = apply_intents_to_table_body(&t, &b, h.as_deref(), &ints).unwrap();
            b = nb; h = nh;
        }
    }
    let items = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &b, h.as_deref()).unwrap();
    let mut by_key: std::collections::HashMap<u64, (String, Vec<serde_json::Value>)> = Default::default();
    for r in &items {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        by_key.insert(k, (o.get("string_key").and_then(|x| x.as_str()).unwrap_or("").into(),
            o.get("prefab_data_list").and_then(|x| x.as_array()).cloned().unwrap_or_default()));
    }
    let cb = std::fs::read(dir.join("characterinfo.pabgb")).unwrap();
    let ch = std::fs::read(dir.join("characterinfo.pabgh")).ok();
    let chars = dmm_parser::dispatch::parse_table_to_json("characterinfo.pabgb", &cb, ch.as_deref()).unwrap();

    let mut violations = 0usize;
    let mut checked = 0usize;
    for r in &chars {
        let Some(o) = r.as_object() else { continue };
        let Some(ck) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        let Some(tg) = o.get("f31").and_then(|x| x.as_u64()) else { continue };
        for e in o.get("equip_item_info_list").and_then(|x| x.as_array()).into_iter().flatten() {
            let Some(ik) = e.get("equip_item_info").and_then(|x| x.as_u64()) else { continue };
            if ik == 0 { continue }
            let Some((nm, els)) = by_key.get(&ik) else { continue };
            if els.is_empty() { continue }
            checked += 1;
            let ok = els.iter()
                .filter(|el| el["prefab_data_type"].as_u64() == Some(0))
                .any(|el| el["tribe_gender_list"].as_array()
                    .map(|a| a.is_empty() || a.iter().any(|x| x.as_u64() == Some(tg)))
                    .unwrap_or(false));
            if !ok {
                violations += 1;
                if violations <= 15 {
                    eprintln!("RESULT ✗ characterinfo({ck}) presets {nm} (key={ik}) but tribe_gender {tg} matches NO type-0 element");
                }
            }
        }
    }
    eprintln!("RESULT engine checkValid: {checked} preset items checked, {violations} VIOLATIONS");
    assert_eq!(violations, 0, "mod revokes access to a characterinfo preset item -> game data load FAILS at launch");
}

/// V7.9 -- the merge rule the engine's checkValid demands.
/// Per element, given vanilla's tribe_gender list V and the mod's M:
///   M empty     => write []      (the mod's "anyone may equip this" grant)
///   M non-empty => write V ∪ M   (widen; NEVER revoke a character's access)
/// Writing M verbatim (V7.4/7.6/7.7/7.8a) revoked access to items characterinfo
/// presets on a character -- e.g. Tynion_Giant_TwoHandGiantBastard went from
/// [650024735(Damiane),590304724,4184612308] to [4234598676(Kliff)], so
/// characterinfo(4)'s preset became unequippable and the game aborted its data load.
/// Unioning BOTH cases (V7.3) never revokes but also never grants, which is why
/// Damiane's armor stayed Unequippable.
/// Also: out-of-scope (non-player-equippable) items dropped; equip_slot_list never
/// touched; vanilla open-element envelope respected.
#[test]
#[ignore]
fn rebuild_equipall_v79() {
    const PLAYERS: [u64; 3] = [1, 4, 6];
    let dir = fixture_dir();
    let body = std::fs::read(dir.join("iteminfo.pabgb")).unwrap();
    let ph = std::fs::read(dir.join("iteminfo.pabgh")).ok();
    let arr = dmm_parser::dispatch::parse_table_to_json("iteminfo.pabgb", &body, ph.as_deref()).unwrap();
    let eb = std::fs::read(dir.join("equipslotinfo.pabgb")).unwrap();
    let eh = std::fs::read(dir.join("equipslotinfo.pabgh")).ok();
    let slots = dmm_parser::dispatch::parse_table_to_json("equipslotinfo.pabgb", &eb, eh.as_deref()).unwrap();
    let mut player_types: std::collections::HashSet<u64> = Default::default();
    for r in &slots {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        if !PLAYERS.contains(&k) { continue }
        for e in o.get("entries").and_then(|x| x.as_array()).into_iter().flatten() {
            for h in e["etl_hashes"].as_array().into_iter().flatten().filter_map(|x| x.as_u64()) {
                player_types.insert(h);
            }
        }
    }
    let mut van: std::collections::HashMap<u64, serde_json::Value> = Default::default();
    let mut ity: std::collections::HashMap<u64, u64> = Default::default();
    for r in &arr {
        let Some(o) = r.as_object() else { continue };
        let Some(k) = o.get("__key__").or_else(|| o.get("key")).and_then(|x| x.as_u64()) else { continue };
        ity.insert(k, o.get("equip_type_info").and_then(|x| x.as_u64()).unwrap_or(0));
        if let Some(p) = o.get("prefab_data_list") { van.insert(k, p.clone()); }
    }
    let openc = |l: &[serde_json::Value]| l.iter()
        .filter(|e| e["tribe_gender_list"].as_array().map(|a| a.is_empty()).unwrap_or(false)).count();
    let key_of = |pd: &serde_json::Value| -> String {
        pd["prefab_names"].as_array().map(|a| a.iter().filter_map(|x| x.as_u64())
            .map(|x| x.to_string()).collect::<Vec<_>>().join(",")).unwrap_or_default()
    };
    let mut raw: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("DMM_MOD_SRC").unwrap()).unwrap()).unwrap();
    let (mut granted, mut widened, mut scope, mut held) = (0usize, 0usize, 0usize, 0usize);
    for t in raw["targets"].as_array_mut().into_iter().flatten() {
        if !t["file"].as_str().unwrap_or("").contains("iteminfo") { continue }
        for i in t["intents"].as_array_mut().into_iter().flatten() {
            let Some(k) = i["key"].as_u64() else { continue };
            let Some(vlist) = van.get(&k).and_then(|v| v.as_array()).cloned() else { continue };
            if !ity.get(&k).map(|t| player_types.contains(t)).unwrap_or(false) {
                scope += 1; i["__drop__"] = serde_json::json!(true); continue;
            }
            let mut modmap: std::collections::HashMap<String, serde_json::Value> = Default::default();
            for pd in i["new"].as_array().into_iter().flatten() { modmap.insert(key_of(pd), pd.clone()); }
            let mut out = Vec::with_capacity(vlist.len());
            for vpd in &vlist {
                let mut e = vpd.clone();
                if let Some(mpd) = modmap.get(&key_of(vpd)) {
                    let m: Vec<u64> = mpd["tribe_gender_list"].as_array().into_iter().flatten()
                        .filter_map(|x| x.as_u64()).collect();
                    let v: Vec<u64> = vpd["tribe_gender_list"].as_array().into_iter().flatten()
                        .filter_map(|x| x.as_u64()).collect();
                    if mpd["tribe_gender_list"].is_null() {
                        // no opinion
                    } else if m.is_empty() {
                        if !v.is_empty() { granted += 1; }
                        e["tribe_gender_list"] = serde_json::json!([]);
                    } else if !v.is_empty() {
                        let mut u = v.clone();
                        for h in &m { if !u.contains(h) { u.push(*h); } }
                        if u.len() != v.len() { widened += 1; }
                        e["tribe_gender_list"] = serde_json::json!(u);
                    }
                    // v empty + m non-empty => leave OPEN (never narrow)
                }
                out.push(e);
            }
            if openc(&out) > openc(&vlist) && openc(&out) > 3 { out = vlist.clone(); held += 1; }
            if out == vlist { i["__drop__"] = serde_json::json!(true); continue; }
            i["new"] = serde_json::Value::Array(out);
        }
    }
    for t in raw["targets"].as_array_mut().into_iter().flatten() {
        if let Some(v) = t["intents"].as_array_mut() {
            v.retain(|i| !i.get("__drop__").and_then(|x| x.as_bool()).unwrap_or(false));
        }
    }
    let ver = std::env::var("DMM_MOD_VER").unwrap_or_else(|_| "7.9".into());
    raw["modinfo"]["title"] = serde_json::json!(format!("Equip All V{ver}"));
    raw["modinfo"]["version"] = serde_json::json!(ver);
    let counts: Vec<String> = raw["targets"].as_array().unwrap().iter()
        .map(|t| format!("{}={}", t["file"].as_str().unwrap_or("?"),
             t["intents"].as_array().map(|a| a.len()).unwrap_or(0))).collect();
    let out = std::env::var("DMM_MOD_OUT").unwrap();
    std::fs::write(&out, serde_json::to_vec_pretty(&raw).unwrap()).unwrap();
    eprintln!("RESULT granted_open={granted} widened_union={widened} scope_dropped={scope} envelope_held={held} intents {counts:?}");
}

/// List the vpaths inside a standalone-overlay mod's PAZ group. DMM's crash
/// attribution scans mod folders for FILE NAMES, so a mod that ships its content
/// inside a .paz (group dir + 0.pamt/0.paz) contributes nothing and can never be
/// blamed. DMM_PAMT_DIR = the group dir (e.g. "...\Cloak Remover\0036").
#[test]
#[ignore]
fn list_paz_group_contents() {
    use dmm_parser::binary::pamt::PackMeta;
    let dir = PathBuf::from(std::env::var("DMM_PAMT_DIR").expect("DMM_PAMT_DIR"));
    let pamt = PackMeta::parse(&std::fs::read(dir.join("0.pamt")).expect("0.pamt"), None)
        .expect("parse pamt");
    let mut n = 0usize;
    for d in &pamt.directories {
        for f in &d.files {
            n += 1;
            eprintln!("RESULT {}/{}", d.path, f.name);
        }
    }
    eprintln!("RESULT total files in group: {n}");
}
