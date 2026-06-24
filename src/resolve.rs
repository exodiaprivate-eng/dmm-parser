// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Opt-in cross-table hash → name resolution.
//!
//! Many table fields hold a `u32` that is really a **foreign record key**:
//! the game's reader maps it through a per-table runtime registry to an
//! index (`wire u32 → registry → u16 index`). The wire value is the
//! authoritative foreign key, so a modder targeting it must currently
//! supply a raw hash — guesswork.
//!
//! This module annotates already-parsed table JSON with readable companion
//! fields (`<field>_name` for scalars, `<field>_names` for arrays) by
//! looking each key up in a caller-supplied [`NameIndex`]. The raw hash
//! stays authoritative; companions are advisory and ignored on
//! serialize, so round-trip is unaffected.
//!
//! The field → target-table map is derived from IDA analysis of each
//! table reader's lookup sub-calls (which registry a field consults
//! identifies the table its foreign key belongs to). All confirmed
//! against live data:
//!   - `learn_knowledge_info → knowledge_info` (452/453 match real keys)
//!   - `icon_path`, `video_path → string_info` (311/311 and 242/243 match;
//!     the StringInfo `buffer` field holds the readable icon name /
//!     `ui/media/*.mp4` path — these were never path hashes, they are
//!     StringInfo foreign keys via registry `0x145F2A368`, the "stringinfo"
//!     table).
//!
//! Because the readable name lives in different fields per table
//! (`string_key` for most, `buffer` for `string_info`), the caller picks
//! the field when building each index — see [`extract_key_name_index`].

use serde_json::Value;
use std::collections::HashMap;

/// `table_name → (record key → readable name)`. Built by the caller from
/// the referenced tables' own parsed records (see [`extract_key_name_index`]).
pub type NameIndex = HashMap<String, HashMap<u32, String>>;

/// A resolvable reference field: `(json_field, target_table, is_array)`.
pub struct RefField {
    pub field: &'static str,
    pub target: &'static str,
    pub is_array: bool,
}

const fn rf(field: &'static str, target: &'static str, is_array: bool) -> RefField {
    RefField { field, target, is_array }
}

/// Reference fields for a canonical table name. Empty slice ⇒ no known
/// resolvable references (resolution is a no-op).
///
/// `skill_info` mappings (IDA reader `sub_1410DAE40` lookup sub-calls):
///   - `parent_skill`              `sub_1410E1190` → skill_info (self)
///   - `learn_knowledge_info`      `sub_1410E2D50` → knowledge_info ✅ verified
///   - `need_upgrade_item_info`    `sub_1410E1B70` → item_info
///   - `faction_info`              `sub_1410E4300` → faction_info
///   - `icon_path`                 `sub_1410E1350` → string_info (buffer)
///   - `video_path`                `sub_1410E1350` → string_info (buffer)
///   - `usable_character_info_list``sub_1410E1E40` → character_info (array)
///   - `skill_group_key_list`      `sub_1410E1040` → skill_group_info (array)
///   - `reserve_slot_info_list`    `sub_1410EA320` → reserve_slot_info (array)
pub fn reference_fields(table: &str) -> &'static [RefField] {
    match table {
        "skill_info" => SKILL_INFO_REFS,
        "knowledge_info" => KNOWLEDGE_INFO_REFS,
        "npc_info" => NPC_INFO_REFS,
        "store_info" => STORE_INFO_REFS,
        "character_info" => CHARACTER_INFO_REFS,
        "mission_info" => MISSION_INFO_REFS,
        "quest_info" => QUEST_INFO_REFS,
        "buff_info" => BUFF_INFO_REFS,
        _ => &[],
    }
}

/// `buff_info` references. `elemental_status_info`→status_info validated
/// (8/8). (`ui_template_name`/`ui_component_name` are NOT string_info —
/// 0/283, a different UI registry — so omitted. Most buff content is in the
/// nested buff_data_list/BuffData family, not top-level.)
static BUFF_INFO_REFS: &[RefField] = &[
    rf("elemental_status_info", "status_info", false),
];

/// `quest_info` candidate references — pruned by resolve_validate to those
/// that actually resolve names on live data.
static QUEST_INFO_REFS: &[RefField] = &[
    rf("faction_info", "faction_info", false),
    rf("quest_group_info", "quest_group_info", false),
    rf("start_mission", "mission_info", false),
    rf("start_stage", "stage_info", false),
    rf("game_start_stage", "stage_info", false),
    rf("stage_icon_path", "string_info", false),
    rf("stage_text_icon_path", "string_info", false),
    rf("stage_image_path", "string_info", false),
    rf("start_player_list", "character_info", true),
    rf("executor_quest_list", "quest_info", true),
    rf("mission_list", "mission_info", true),
    rf("stage_list", "stage_info", true),
    rf("dialog_must_mission_info_list", "mission_info", true),
];

/// `mission_info` references. `sub_mission_list`→mission_info (self, 2143)
/// and `start_player_list`→character_info (proven player-list pattern) are
/// validated. (`reward_inventory_key`→inventory_info was tried but matched
/// 0 keys — wrong target — so it is omitted.)
static MISSION_INFO_REFS: &[RefField] = &[
    rf("sub_mission_list", "mission_info", true),
    rf("start_player_list", "character_info", true),
];

/// `character_info` references. `vehicle_info`→vehicle_info yields readable
/// names (436/436). (`npc_info` keys do reference npc_info records but
/// those have empty `string_key` → no names, so it's omitted as a no-op.
/// `equip_info` omitted: unvalidated, equip_info parse yielded 0 keys.
/// `ui_icon_path` uses read_u32_lookup_DA30, not string_info.)
static CHARACTER_INFO_REFS: &[RefField] = &[
    rf("vehicle_info", "vehicle_info", false),
];

/// `store_info` (shop) item references (IDA `sub_1410DB.../sub_1410E1B70`
/// item-lookup reader — same as npc `coupon_item_info`): the buy item and
/// the list of sellable items.
static STORE_INFO_REFS: &[RefField] = &[
    rf("exchange_item_info_for_buy", "item_info", false),
    rf("exchange_item_info_list_for_sell", "item_info", true),
];

/// `npc_info` (shop-NPC table) references. `store_info`→store_info is
/// validated on live data (378/378); `coupon_item_info`→item_info is
/// IDA-documented (sub_1410E1B70). NOTE: `icon_path` uses a DIFFERENT
/// registry (read_u32_lookup_DA30 / qword_145F0DA30), NOT string_info —
/// confirmed by 0/398 string_info matches — so it is intentionally absent.
static NPC_INFO_REFS: &[RefField] = &[
    rf("store_info", "store_info", false),
    rf("coupon_item_info", "item_info", false),
];

/// `knowledge_info` references (IDA reader `sub_1410C51C0` + live-data
/// validation). `skill_info`/`learn_apply_skill_info`→skill_info,
/// `faction_info`→faction_info are 100%-confirmed; `ui_texture_name`→
/// string_info (buffer) is the proven icon-path pattern. The rest are
/// IDA-documented (each field is named after its target table). Resolution
/// is opt-in + advisory, so only tables supplied in `context` annotate.
static KNOWLEDGE_INFO_REFS: &[RefField] = &[
    rf("skill_info", "skill_info", false),
    rf("learn_apply_skill_info", "skill_info", false),
    rf("faction_info", "faction_info", false),
    rf("faction_node_info", "faction_node_info", false),
    rf("item_info", "item_info", false),
    rf("ui_texture_name", "string_info", false),
    rf("learning_stage_info", "stage_info", false),
    rf("shared_level_main_knowledge_info", "knowledge_info", false),
    rf("character_info_list", "character_info", true),
    rf("gimmick_info_list", "gimmick_info", true),
    rf("knowledge_group_list", "knowledge_group_info", true),
    rf("stage_info_list", "stage_info", true),
    rf("shared_level_knowledge_info_list", "knowledge_info", true),
];

static SKILL_INFO_REFS: &[RefField] = &[
    rf("parent_skill", "skill_info", false),
    rf("learn_knowledge_info", "knowledge_info", false),
    rf("need_upgrade_item_info", "item_info", false),
    rf("faction_info", "faction_info", false),
    rf("icon_path", "string_info", false),
    rf("video_path", "string_info", false),
    rf("usable_character_info_list", "character_info", true),
    rf("skill_group_key_list", "skill_group_info", true),
    rf("reserve_slot_info_list", "reserve_slot_info", true),
];

/// A reference field *inside* the elements of a list field:
/// `<list_field>[].<elem_field>` resolves against `target`. Each element
/// object gains a `<elem_field>_name` companion when its key resolves.
pub struct NestedRefField {
    pub list_field: &'static str,
    pub elem_field: &'static str,
    pub target: &'static str,
}

const fn nrf(list_field: &'static str, elem_field: &'static str, target: &'static str) -> NestedRefField {
    NestedRefField { list_field, elem_field, target }
}

/// Nested (list-element) reference fields for a canonical table name.
///
/// `skill_info` resource costs: each `ResourceStat` carries `lookup_b`
/// (the consumed resource — Stamina/Mp/Fury), `lookup_e`/`lookup_f` (the
/// increase/decrease-rate modifier stats), all StatusInfo keys; each
/// `ResourceItem` carries an ItemInfo `lookup`. Verified on live data
/// (e.g. `use_resource_stat_list[].lookup_b == 0xf425a` → "Stamina").
pub fn nested_reference_fields(table: &str) -> &'static [NestedRefField] {
    match table {
        "skill_info" => SKILL_INFO_NESTED,
        "npc_info" => NPC_INFO_NESTED,
        _ => &[],
    }
}

static NPC_INFO_NESTED: &[NestedRefField] = &[
    nrf("dye_color_group_data_list", "dye_color_group_key", "dye_color_group_info"),
];

static SKILL_INFO_NESTED: &[NestedRefField] = &[
    nrf("use_resource_stat_list", "lookup_b", "status_info"),
    nrf("use_resource_stat_list", "lookup_e", "status_info"),
    nrf("use_resource_stat_list", "lookup_f", "status_info"),
    nrf("use_driver_resource_stat_list", "lookup_b", "status_info"),
    nrf("use_driver_resource_stat_list", "lookup_e", "status_info"),
    nrf("use_driver_resource_stat_list", "lookup_f", "status_info"),
    nrf("use_resource_item_list", "lookup", "item_info"),
];

/// Build a `key → name` map from a parsed table's JSON records.
///
/// Pulls the `"key"` (u32) and `name_field` (string) of each record.
/// Most tables use `"string_key"` for the canonical readable name.
/// Records missing either field, or with a zero key, are skipped.
pub fn extract_key_name_index(items: &[Value], name_field: &str) -> HashMap<u32, String> {
    let mut map = HashMap::new();
    for it in items {
        let Some(obj) = it.as_object() else { continue };
        let Some(key) = obj.get("key").and_then(Value::as_u64) else { continue };
        if key == 0 || key > u32::MAX as u64 { continue; }
        if let Some(name) = obj.get(name_field).and_then(Value::as_str) {
            if !name.is_empty() {
                map.insert(key as u32, name.to_string());
            }
        }
    }
    map
}

/// Annotate parsed `items` (records of `table`) in place with readable
/// companion fields. Scalars get `<field>_name` (a string) when the key
/// resolves; arrays get `<field>_names` (parallel array, `null` per
/// unresolved element). Zero keys and absent fields are left untouched.
///
/// Returns the number of companion fields added (for diagnostics).
pub fn annotate(table: &str, items: &mut [Value], idx: &NameIndex) -> usize {
    let fields = reference_fields(table);
    let nested = nested_reference_fields(table);
    if fields.is_empty() && nested.is_empty() {
        return 0;
    }
    let mut added = 0;
    for it in items.iter_mut() {
        let Some(obj) = it.as_object_mut() else { continue };
        // Nested list-element lookups (e.g. use_resource_stat_list[].lookup_b
        // → status_info "Stamina"). Done first; uses get_mut on the list.
        for nf in nested {
            let Some(tbl) = idx.get(nf.target) else { continue };
            let Some(arr) = obj.get_mut(nf.list_field).and_then(Value::as_array_mut) else { continue };
            for elem in arr.iter_mut() {
                let Some(eo) = elem.as_object_mut() else { continue };
                let Some(k) = eo.get(nf.elem_field).and_then(Value::as_u64) else { continue };
                if k == 0 || k > u32::MAX as u64 { continue; }
                if let Some(name) = tbl.get(&(k as u32)) {
                    eo.insert(format!("{}_name", nf.elem_field), Value::String(name.clone()));
                    added += 1;
                }
            }
        }
        for rf in fields {
            let Some(tbl) = idx.get(rf.target) else { continue };
            if rf.is_array {
                let Some(arr) = obj.get(rf.field).and_then(Value::as_array) else { continue };
                let names: Vec<Value> = arr.iter().map(|e| {
                    e.as_u64()
                        .and_then(|k| if k == 0 { None } else { tbl.get(&(k as u32)) })
                        .map(|n| Value::String(n.clone()))
                        .unwrap_or(Value::Null)
                }).collect();
                if names.iter().any(|n| !n.is_null()) {
                    added += names.iter().filter(|n| !n.is_null()).count();
                    obj.insert(format!("{}_names", rf.field), Value::Array(names));
                }
            } else {
                let Some(k) = obj.get(rf.field).and_then(Value::as_u64) else { continue };
                if k == 0 { continue; }
                if let Some(name) = tbl.get(&(k as u32)) {
                    obj.insert(format!("{}_name", rf.field), Value::String(name.clone()));
                    added += 1;
                }
            }
        }
    }
    added
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn annotates_scalar_and_array() {
        let mut idx: NameIndex = HashMap::new();
        idx.insert("knowledge_info".into(),
            [(40001u32, "Knowledge_Foo".to_string())].into_iter().collect());
        idx.insert("character_info".into(),
            [(1u32, "Char_A".to_string()), (4u32, "Char_B".to_string())].into_iter().collect());

        let mut items = vec![json!({
            "key": 7,
            "learn_knowledge_info": 40001,
            "faction_info": 0,
            "usable_character_info_list": [1, 4, 999],
        })];

        let added = annotate("skill_info", &mut items, &idx);
        assert_eq!(added, 3); // 1 knowledge + 2 characters
        let o = items[0].as_object().unwrap();
        assert_eq!(o["learn_knowledge_info_name"], json!("Knowledge_Foo"));
        // faction_info==0 → no companion
        assert!(o.get("faction_info_name").is_none());
        assert_eq!(o["usable_character_info_list_names"], json!(["Char_A", "Char_B", null]));
    }

    #[test]
    fn annotates_nested_resource_stat() {
        let mut idx: NameIndex = HashMap::new();
        idx.insert("status_info".into(),
            [(0xf425au32, "Stamina".to_string()), (0xf425b, "Mp".to_string())].into_iter().collect());

        let mut items = vec![json!({
            "key": 7,
            "use_resource_stat_list": [
                {"a": 3, "lookup_b": 0xf425a, "c": 1, "d": 100000, "lookup_e": 0, "lookup_f": 0},
                {"a": 3, "lookup_b": 0xf425b, "c": 1, "d": 50000, "lookup_e": 0, "lookup_f": 0},
            ],
        })];

        annotate("skill_info", &mut items, &idx);
        let arr = items[0]["use_resource_stat_list"].as_array().unwrap();
        assert_eq!(arr[0]["lookup_b_name"], json!("Stamina"));
        assert_eq!(arr[1]["lookup_b_name"], json!("Mp"));
        // zero lookup_e gets no companion
        assert!(arr[0].get("lookup_e_name").is_none());
    }

    #[test]
    fn extract_skips_zero_and_missing() {
        let items = vec![
            json!({"key": 5, "string_key": "Five"}),
            json!({"key": 0, "string_key": "Zero"}),
            json!({"key": 9}),
        ];
        let m = extract_key_name_index(&items, "string_key");
        assert_eq!(m.len(), 1);
        assert_eq!(m[&5], "Five");
    }

    /// End-to-end on live data: resolve skill_info.learn_knowledge_info
    /// against real KnowledgeInfo records. Skips if fixtures are absent.
    #[test]
    fn resolves_learn_knowledge_on_real_data() {
        use crate::dispatch::parse_table_to_json_shaped;
        use crate::json_shape::JsonShape;

        const SKILL_B: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser\fixtures_skill\skill.pabgb";
        const SKILL_H: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser\fixtures_skill\skill.pabgh";
        const KNOW_B: &str = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-5-1\knowledgeinfo.pabgb";
        const KNOW_H: &str = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-5-1\knowledgeinfo.pabgh";

        let (Ok(skill_b), Ok(skill_h), Ok(know_b), Ok(know_h)) = (
            std::fs::read(SKILL_B), std::fs::read(SKILL_H),
            std::fs::read(KNOW_B), std::fs::read(KNOW_H),
        ) else { eprintln!("SKIP: fixtures absent"); return; };

        let shape = JsonShape::from_str("").unwrap();
        let know = parse_table_to_json_shaped("knowledge_info", &know_b, Some(&know_h), shape).unwrap();
        let mut skills = parse_table_to_json_shaped("skill_info", &skill_b, Some(&skill_h), shape).unwrap();

        let mut idx: NameIndex = HashMap::new();
        idx.insert("knowledge_info".into(), extract_key_name_index(&know, "string_key"));

        let added = annotate("skill_info", &mut skills, &idx);
        let resolved = skills.iter()
            .filter(|s| s.get("learn_knowledge_info_name").and_then(Value::as_str).is_some())
            .count();
        println!("annotated {} companions; {} skills got learn_knowledge_info_name", added, resolved);
        assert!(resolved > 400,
            "expected >400 learn_knowledge_info_name resolutions, got {}", resolved);

        // Spot-check: a resolved name must be a real knowledge string_key.
        let know_names: std::collections::HashSet<&str> = idx["knowledge_info"]
            .values().map(|s| s.as_str()).collect();
        for s in &skills {
            if let Some(n) = s.get("learn_knowledge_info_name").and_then(Value::as_str) {
                assert!(know_names.contains(n), "resolved name {:?} not a knowledge string_key", n);
            }
        }

        // Round-trip safety: serializing the ANNOTATED records (with the
        // advisory *_name companions present) must reproduce the original
        // bytes exactly — companions are ignored on write.
        let reser = crate::dispatch::serialize_table_from_json_shaped(
            "skill_info", &skills, shape).unwrap();
        assert_eq!(reser, skill_b,
            "annotated skill_info did not round-trip to identical bytes");
    }

    /// icon_path / video_path resolve via the StringInfo `buffer` field.
    /// Skips if fixtures are absent.
    #[test]
    fn resolves_icon_video_via_stringinfo() {
        use crate::dispatch::parse_table_to_json_shaped;
        use crate::json_shape::JsonShape;

        const SKILL_B: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser\fixtures_skill\skill.pabgb";
        const SKILL_H: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser\fixtures_skill\skill.pabgh";
        const STR_B: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser\fixtures_skill\stringinfo.pabgb";

        let (Ok(skill_b), Ok(skill_h), Ok(str_b)) = (
            std::fs::read(SKILL_B), std::fs::read(SKILL_H), std::fs::read(STR_B),
        ) else { eprintln!("SKIP: fixtures absent"); return; };

        let shape = JsonShape::from_str("").unwrap();
        // string_info is sequential (no pabgh).
        let strings = parse_table_to_json_shaped("string_info", &str_b, None, shape).unwrap();
        let mut skills = parse_table_to_json_shaped("skill_info", &skill_b, Some(&skill_h), shape).unwrap();

        let mut idx: NameIndex = HashMap::new();
        idx.insert("string_info".into(), extract_key_name_index(&strings, "buffer"));

        annotate("skill_info", &mut skills, &idx);
        let icon = skills.iter().filter(|s| s.get("icon_path_name").is_some()).count();
        let video = skills.iter().filter(|s| s.get("video_path_name").is_some()).count();
        println!("icon_path_name: {}, video_path_name: {}", icon, video);
        assert!(icon > 1500, "expected most icon_path resolved, got {}", icon);
        assert!(video > 200, "expected video_path resolved, got {}", video);

        // A resolved icon name looks like "Skill_*" / "cd_icon_*"; video like ".mp4".
        let some_video = skills.iter()
            .find_map(|s| s.get("video_path_name").and_then(Value::as_str))
            .unwrap();
        assert!(some_video.contains('.') || some_video.contains('/'),
            "video_path_name {:?} doesn't look like a path", some_video);
    }
}
