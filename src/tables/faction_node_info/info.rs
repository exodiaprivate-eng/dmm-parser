//! Tier 2 — typed prefix (fields 1–17) + raw `tail_blob` (fields 18+).
//!
//! 1.10 layout: `_operationSpawnRadius` (f32) was added between
//! `_worldPosition` and `_nodeRadius`. Fields 1–17 are typed and
//! JSON-addressable; `_factionScheduleInfoList` onward stays in `tail_blob`
//! because the FactionSchedule element decoder still over-consumes on the
//! 359 records with populated schedule/adjacency lists. See the struct
//! definition's truncation note for the full rationale.
//!
//! Reader: `sub_1410DE7A0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings
//! / `docs/449_TABLE_CATALOG.md` FactionNodeInfo section):
//!   1. u32 key                                      (_key)
//!   2. CString string_key                           (_stringKey)
//!   3. u8 is_blocked                                (_isBlocked)
//!   4. u32 knowledge_info                           (_knowledgeInfo,
//!      sub_1411006D0 wire u32)
//!   5. u32 skill_tree_info                          (_skillTreeInfo,
//!      sub_1411035A0 wire u32)
//!   6. u32 connect_research_node_info               (_connectResearchNodeInfo,
//!      sub_141101D50 wire u32)
//!   7. u16 store_info                               (_storeInfo,
//!      sub_141103610 — wire u16)
//!   8. u16 royal_supply_info                        (_royalSupplyInfo,
//!      sub_1411036C0 — wire u16)
//!   9. CString memo                                 (_memo)
//!  10. CArray<u32> child_faction_info_list          (_childFactionInfoList,
//!      sub_141102FF0 wire u32)
//!  11. CArray<u32> node_line_main_faction_info_list (_nodeLineMainFactionInfoList,
//!      sub_141102FF0)
//!  12. [f32; 3] world_position                      (_worldPosition, Vec3)
//!  13. f32 node_radius                              (_nodeRadius)
//!  14. CArray<ApplySkillData> apply_skill_data_list (_applySkillDataList,
//!      sub_141115FD0; per element: u32 lookup + u64 raw +
//!      FactionNodeRawDataExt — 40 mem bytes)
//!  15. CArray<ResourceItemData> resource_item_list  (_resourceItemList,
//!      sub_141115D90; per element: u32 + u32 + u64 + u64 +
//!      FactionNodeRawDataExt — 48 mem bytes)
//!  16. CArray<u32> revival_stage_info_list          (_revivalStageInfoList,
//!      sub_141101610 wire u32)
//!  17. CArray<WayPointDeprData> way_point_data_list_deprecated
//!      (_wayPointDataList_deprecated, sub_141115BA0; per element:
//!      u16 + u16 + CArray<[f32; 3]> — 24 mem bytes)
//!  18. CArray<FactionSchedule> faction_schedule_list (sub_1410DDE60
//!      via inline CArray; per element 280 mem bytes / 31 wire fields)
//!  19. u8 unknown_a                                (mem a2+168)
//!  20. CString key_str_after                       (sub_1410A9D40 wire
//!      CString — mem a2+172 stores u32 hash)
//!  21. u8 unknown_b                                (mem a2+176)
//!  22. u32 lookup_after                            (sub_1410FF430 wire
//!      u32 — mem a2+178)
//!  23. u8 unknown_c                                (mem a2+180)
//!  24. u8 unknown_d                                (mem a2+181)
//!  25. CArray<FactionAdjacencyEntry> adjacency_list (sub_141115A30 →
//!      sub_1410DE350; per element 144 mem bytes / 10+ wire fields:
//!      u32 raw_a + FactionAdjacencyData inner with 2 LocalizableStrings,
//!      3 u32 lookups, FactionScheduleU64Triple list, u32 lookup+raw
//!      pair list, FactionAdjacencyMobItem list, trailing u32)
//!  26. FactionNodeBigCompositeSlots big_composite_slots (13 fixed
//!      CArray<FactionNodeBigComposite> slots via sub_141128990 →
//!      sub_1410DD2A0 (24-byte/10-field header) + sub_1410DD420
//!      (296-byte/37-field body with FactionNodeDD140Inner inner)
//!  27. u8 flag_after_slots                           (mem a2+408)
//!  28. FactionNodeDE690 de690_data                   (sub_1410DE690 —
//!      28-byte fixed: Vec3 + 4× u32, mem a2+412)
//!  29. u32 raw_after_de690                           (mem a2+440)
//!  30. CArray<u32> final_list_u32                    (sub_141100510 wire
//!      u32 — mem a2+448)
//!  31. CArray<u16> final_list_u16                    (sub_1410FFAC0 wire
//!      u16 — mem a2+464)
//!  32. u32 final_lookup                              (sub_141103770 wire
//!      u32 — mem a2+480)
//!
//! All 32 wire fields typed. JSON-addressable for full mod-editing.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::FactionNodeInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  31
// Decoded by dmm-parser:   16
// Missing in this struct:  15
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ⏳ _observeData (array_or_complex, stream=4)
// ⏳ _useCustomWayPointforDev (direct_15B, stream=15)
// ⏳ _religionBlockCostList (reader_4B, stream=4)
// ⏳ _religionMaxBlockDay (direct_u32, stream=4)
// ⏳ _religionSubLevelInfo (reader_4B, stream=4)
// ⏳ _religionEffectRegionInfoList (reader_2B, stream=2)
// ⏳ _factionType (direct_15B, stream=15)
// ⏳ _factionScheduleInfoList
// ⏳ _workerCount (direct_15B, stream=15)
// ⏳ _subInnerTypeString
// ⏳ _bitMapColorKey
// ⏳ _knockDownCondition (reader_4B, stream=4)
// ⏳ _factionEventDataList (reader_4B, stream=4)
// ⏳ _researchDataList
// ✅ _nodeLineMainFactionInfoList (reader_4B, stream=4)
// ✅ _childFactionInfoList (reader_4B, stream=4)
// ✅ _nodeRadius (direct_u32, stream=4)
// ✅ _worldPosition (direct_12B, stream=12)
// ✅ _resourceItemList
// ✅ _applySkillDataList
// ⏳ _wayPointDataList_deprecated
// ✅ _revivalStageInfoList (reader_4B, stream=4)
// ✅ _isBlocked (direct_15B, stream=15)
// ✅ _stringKey
// ✅ _skillTreeInfo (reader_4B, stream=4)
// ✅ _knowledgeInfo (reader_4B, stream=4)
// ✅ _storeInfo (reader_2B, stream=2)
// ✅ _connectResearchNodeInfo (reader_4B, stream=4)
// ✅ _memo
// ✅ _royalSupplyInfo (reader_2B, stream=2)
// ✅ _key (direct_u32, stream=4)

use crate::binary::*;
use crate::pabgh_typed_blob_table;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct FactionNodeRawDataExt {
        pub raw_bytes: CArray<u8>,
        pub lookup_a: u32,
        pub lookup_b: u32,
        pub flag: u8,
    }
}

py_binary_struct! {
    pub struct ApplySkillData {
        pub lookup: u32,
        pub raw: u64,
        pub ext: FactionNodeRawDataExt,
    }
}

py_binary_struct! {
    pub struct ResourceItemData {
        pub lookup_a: u32,
        pub lookup_b: u32,
        pub raw_a: u64,
        pub raw_b: u64,
        pub ext: FactionNodeRawDataExt,
    }
}

py_binary_struct! {
    pub struct WayPointTriplet {
        pub xyz: [f32; 3],
    }
}

py_binary_struct! {
    // 2.04.00: from/to node keys are 4 bytes each (Mac reader sub_10206DAE0).
    pub struct WayPointDeprData {
        pub lookup_a: u32,
        pub lookup_b: u32,
        pub points: CArray<WayPointTriplet>,
    }
}

// sub_1410DDD40 — FactionScheduleEntry48 inner: 48 mem / 6 wire fields.
// Trailing CArray<u64> per sub_141103200: each u64 is u32 lookup
// (sub_1410FF340 wire u32) + u32 raw, packed LE so wire matches u64.
py_binary_struct! {
    pub struct FactionScheduleEntry48 {
        pub flag_a: u8,
        pub raw_a: u32,
        pub raw_b: u32,
        pub vec3: [f32; 3],
        pub flag_b: u8,
        pub list_u64: CArray<u64>,
    }
}

// sub_141103310 inner — 16 mem bytes / 2 wire fields packed as u128.
// Wire: sub_1410FF5C0 (u32 lookup) + u64 raw = 12 wire bytes per element.
py_binary_struct! {
    pub struct FactionScheduleU128Pair {
        pub lookup: u32,
        pub raw: u64,
    }
}

// sub_141103420 → sub_1410ED8F0 inner — 32 mem / 6 wire fields = 30 wire.
py_binary_struct! {
    pub struct FactionScheduleSlotInner {
        pub lookup_a: u32, // sub_1410FF5C0 wire u32
        pub lookup_b: u32, // sub_141100740 wire u32
        pub lookup_c: u32, // sub_1410FF340 wire u32
        pub raw_a: u64,
        pub raw_b: u64,
        pub raw_c: u16,
    }
}

// sub_141116370 inner — 13 wire bytes per element.
py_binary_struct! {
    pub struct FactionScheduleU64Triple {
        pub lookup_a: u32,  // sub_1410FF430 wire u32
        pub flag: u8,
        pub lookup_b: u32,  // sub_1410FF050 wire u32
        pub lookup_c: u32,  // sub_1410FF050 wire u32
    }
}

// sub_1411161C0 inner — 12 wire bytes per element.
py_binary_struct! {
    pub struct FactionScheduleU32Triple {
        pub lookup: u32,  // sub_1411006D0 wire u32
        pub raw_a: u32,
        pub raw_b: u32,
    }
}

// sub_141100E90 inner — 32 mem bytes / 4 wire fields = 28 wire bytes.
// Wire: f32 + 8 raw bytes + 8 raw bytes + 8 raw bytes (assembled into
// 32-byte mem with 4-byte gap after the f32). Used by FactionAdjacencyData
// at slot +112 (and similarly elsewhere — gimmick_info field 7,
// interaction_info field 21).
py_binary_struct! {
    pub struct FactionAdjacencyMobItem {
        pub raw_a: u32,
        pub raw_b: u64,
        pub raw_c: u64,
        pub raw_d: u64,
    }
}

// sub_1410DE350 — FactionAdjacencyData inner, 128 mem bytes / 10 wire fields.
// Inner CArray<u64> at +96: per element u32 lookup (sub_1410FF430) + u32 raw
// = 8 wire bytes packed LE.
py_binary_struct! {
    pub struct FactionAdjacencyData<'a> {
        pub raw_a: u32,
        pub label_a: LocalizableString<'a>,
        pub label_b: LocalizableString<'a>,
        pub lookup_a: u32,                     // read_u32_lookup_DA30 wire u32
        pub lookup_b: u32,                     // sub_141100370 wire u32
        pub lookup_c: u32,                     // sub_141101D50 wire u32
        pub triple_u64_list: CArray<FactionScheduleU64Triple>,
        pub lookup_raw_pair_list: CArray<u64>, // u32 lookup + u32 raw per element
        pub mob_item_list: CArray<FactionAdjacencyMobItem>,
        pub trailing_raw: u32,
    }
}

// sub_141115A30 inner — FactionAdjacencyEntry, 144 mem bytes.
// Wire: u32 raw_a (4 wire) + FactionAdjacencyData (variable wire).
py_binary_struct! {
    pub struct FactionAdjacencyEntry<'a> {
        pub raw_a: u32,
        pub data: FactionAdjacencyData<'a>,
    }
}

// sub_1410DD2A0 — FactionNodeBigCompositeHeader, 24 mem / 31 wire bytes.
py_binary_struct! {
    pub struct FactionNodeBigCompositeHeader {
        pub flag_a: u8,
        pub lookup_a: u32,    // sub_1410FF430 wire u32
        pub lookup_b: u32,    // sub_141102CB0 wire u32
        pub lookup_c: u32,    // sub_141102D20 wire u32
        pub lookup_d: u32,    // sub_141102D90 wire u32
        pub lookup_e: u32,    // sub_141101D50 wire u32
        pub lookup_f: u32,    // sub_141100860 wire u32
        pub lookup_g: u32,    // sub_141102E00 wire u32
        pub flag_b: u8,
        pub flag_c: u8,
    }
}

// sub_1410DD140 — FactionNodeDD140Inner, 32 mem / 8 wire fields = 23 wire.
py_binary_struct! {
    pub struct FactionNodeDD140Inner {
        pub lookup_a: u32,             // sub_141100860 wire u32
        pub lookup_b: u32,             // sub_141101D50 wire u32
        pub flag_a: u8,
        pub flag_b: u8,
        pub flag_c: u8,
        pub raw_a: u32,
        pub raw_b: u32,
        pub list_lookup: CArray<u32>,  // sub_1410FFC20 wire u32 per element
    }
}

// sub_1410DD420 — FactionNodeBigCompositeBody, 296 mem / 37 wire fields.
py_binary_struct! {
    pub struct FactionNodeBigCompositeBody<'a> {
        pub flag_a: u8,
        pub lookup_a: u32,                     // sub_141100860 wire u32
        pub lookup_b: u32,                     // sub_141101D50 wire u32
        pub lookup_c: u32,                     // sub_1410FF340 wire u32
        pub lookup_d: u32,                     // sub_141101D50 wire u32
        pub lookup_e: u32,                     // sub_141100860 wire u32
        pub flag_b: u8,
        pub raw_a: u32,
        pub lookup_f: u16,                     // sub_141102410 wire u16
        pub lookup_g: u32,                     // sub_141101D50 wire u32
        pub flag_c: u8,
        pub flag_d: u8,
        pub dd140_inner: FactionNodeDD140Inner,
        pub flag_e: u8,
        pub lookup_h: u32,                     // sub_1410FF5C0 wire u32
        pub raw_b: u64,
        pub raw_c: u32,
        pub raw_d: u64,
        pub list_u32: CArray<u32>,             // sub_141102EF0 wire u32
        pub lookup_i: u32,                     // sub_141102D20 wire u32
        pub list_u32_b: CArray<u32>,           // sub_1410FFC20 wire u32
        pub raw_e: u64,
        pub list_u32_c: CArray<u32>,           // sub_141102FF0 wire u32
        pub list_u16: CArray<u16>,             // sub_1410FFAC0 wire u16
        pub vec3: [f32; 3],
        pub flag_f: u8,
        pub flag_g: u8,
        pub flag_h: u8,
        pub flag_i: u8,
        pub flag_j: u8,
        pub raw_f: u32,
        pub flag_k: u8,
        pub lookup_j: u32,                     // read_u32_lookup_DA30 wire u32
        pub lookup_k: u32,                     // read_u32_lookup_DA30 wire u32
        pub label: LocalizableString<'a>,
        pub name: CString<'a>,
        pub trailing_raw: u32,
    }
}

// sub_141128990 inner — FactionNodeBigComposite, 288 mem bytes.
// Wire: FactionNodeBigCompositeHeader + FactionNodeBigCompositeBody.
py_binary_struct! {
    pub struct FactionNodeBigComposite<'a> {
        pub header: FactionNodeBigCompositeHeader,
        pub body: FactionNodeBigCompositeBody<'a>,
    }
}

// 13 fixed CArray slots used by faction_node_info field 26 (a2+200..408
// stride 16 mem). Each is a CArray<FactionNodeBigComposite>.
py_binary_struct! {
    pub struct FactionNodeBigCompositeSlots<'a> {
        pub slot_00: CArray<FactionNodeBigComposite<'a>>,
        pub slot_01: CArray<FactionNodeBigComposite<'a>>,
        pub slot_02: CArray<FactionNodeBigComposite<'a>>,
        pub slot_03: CArray<FactionNodeBigComposite<'a>>,
        pub slot_04: CArray<FactionNodeBigComposite<'a>>,
        pub slot_05: CArray<FactionNodeBigComposite<'a>>,
        pub slot_06: CArray<FactionNodeBigComposite<'a>>,
        pub slot_07: CArray<FactionNodeBigComposite<'a>>,
        pub slot_08: CArray<FactionNodeBigComposite<'a>>,
        pub slot_09: CArray<FactionNodeBigComposite<'a>>,
        pub slot_10: CArray<FactionNodeBigComposite<'a>>,
        pub slot_11: CArray<FactionNodeBigComposite<'a>>,
        pub slot_12: CArray<FactionNodeBigComposite<'a>>,
    }
}

// sub_1410DE690 — 28-byte fixed struct: Vec3 + 4× u32.
py_binary_struct! {
    pub struct FactionNodeDE690 {
        pub vec3: [f32; 3],
        pub raw_a: u32,
        pub raw_b: u32,
        pub raw_c: u32,
        pub raw_d: u32,
    }
}

// sub_1410DDE60 — FactionSchedule, 280 mem bytes / 31 wire fields.
py_binary_struct! {
    pub struct FactionSchedule<'a> {
        pub flag_a: u8,
        pub schedule_entries: CArray<FactionScheduleEntry48>,
        pub raw_data_ext: FactionNodeRawDataExt,
        pub list_u128: CArray<FactionScheduleU128Pair>,
        pub player_list: CArray<u32>,        // sub_1410FF890 wire u32
        pub raw_a: u32,
        pub raw_b: u32,
        pub raw_c: u32,
        pub flag_b: u8,
        pub flag_c: u8,
        pub raw_d: u32,
        pub memo: CString<'a>,
        pub label: LocalizableString<'a>,
        pub slot_inner_list: CArray<FactionScheduleSlotInner>,
        pub lookup_a: u32,                   // sub_141100370 wire u32
        pub lookup_b: u32,                   // sub_141100370 wire u32
        pub lookup_c: u32,                   // sub_1410FF5C0 wire u32
        pub lookup_d: u32,                   // sub_141103530 wire u32
        pub name: CString<'a>,
        pub raw_e: u32,
        pub raw_f: u32,
        pub raw_g: u32,
        pub raw_h: u32,
        pub key_str: CString<'a>,            // sub_1410A9D40 wire CString
        pub lookup_e: u32,                   // read_u32_lookup_DA10 wire u32
        pub lookup_f: u32,                   // sub_1410FEBE0 wire u32
        pub raw_i: u64,
        pub vec3: [f32; 3],
        pub triple_u64_list: CArray<FactionScheduleU64Triple>,
        pub triple_u32_list: CArray<FactionScheduleU32Triple>,
        pub flag_d: u8,
    }
}

py_binary_struct! {
    /// 2.04.00 `_observeData` (Mac reader sub_10204F5A0).
    pub struct FactionNodeObserveData {
        pub observe_position: [f32; 3],
        pub start_yaw: f32,
        pub yaw_min: f32,
        pub yaw_max: f32,
        pub observe_camera_preset_hash: u32,
    }
}

// ─────────────────────────────────────────────────────────────────────────
// 2.04 TAIL (game 2.04.08, build 26480954, FileVersion 1.0.0.3576)
// ─────────────────────────────────────────────────────────────────────────
// Everything after `_wayPointDataList_deprecated` used to stay in
// `tail_blob`. The types below type it completely: all 1,120 base + 206
// dlc_01 records read to the end of their pabgh range (tail_blob empty) and
// write back byte-exact. Wire order and names come from the Win exe reader
// (inline after the waypoint list) and its Korean error strings
// ("<Type>의 <_field>를 읽어들이는데 실패했다"). Fields named
// raw_*/flag_*/value_*/stat_* have no matching error string; their meaning
// is not claimed. A `u32` marked "lookup" is a key on the wire that the game
// maps to an index (u16/u32) in memory.

py_binary_struct! {
    // FactionSchedulePlayInfo: one phase of a dispatch operation (always 3 per
    // schedule on 2.04.08; complete types 1, 2, 1). Phase 2 carries the work
    // amount (_operationProgress 40..720; 0 in phases 1 and 3).
    pub struct FactionSchedulePlayInfo {
        pub schedule_complete_type: u8,
        pub schedule_file_name_hash: u32,
        pub operation_progress: u32,
        pub spawn_position: [f32; 3],
        pub spawn_radius: u32,
        pub skip_schedule: u8,
        pub additional_list: CArray<FactionU32Pair>,
    }
}

py_binary_struct! {
    pub struct FactionU32Pair {
        pub a: u32,
        pub b: u32,
    }
}

py_binary_struct! {
    // _resourceDataList element (operation cost), 30 wire bytes.
    pub struct FactionResourceData {
        pub resource_type: u32, // 11..15 = resource kinds, otherwise an item key
        pub lookup_b: u32,
        pub lookup_c: u32,
        pub amount: u64,
        pub raw_b: u64,
        pub raw_c: u16,
    }
}

py_binary_struct! {
    pub struct FactionScheduleStatData {
        pub stat_a: u32,
        pub stat_b: u32,
        pub stat_c: u64,
    }
}

py_binary_struct! {
    // _conditionOptionList element (schedules and research).
    pub struct FactionConditionOption {
        pub condition: u32, // lookup
        pub flag: u8,
        pub value_a: u32,
        pub value_b: u32,
    }
}

py_binary_struct! {
    // FactionScheduleInfo = one dispatch operation (1,112 on 2.04.08).
    pub struct FactionScheduleInfo<'a> {
        pub flag_a: u8,                                   // always 2 on 2.04.08
        pub schedule_play_info_list: CArray<FactionSchedulePlayInfo>,
        pub state: FactionNodeRawDataExt,                 // FactionStateData, same wire shape
        pub pairs: CArray<FactionScheduleU128Pair>,
        pub list_u32: CArray<u32>,
        pub raw_a: u32,
        pub raw_b: u32,
        pub raw_c: u32,
        pub flag_b: u8,
        pub flag_c: u8,
        pub operation_key: u32,
        pub operation_name: LocalizableString<'a>,
        pub operation_group_info: u16,
        pub resource_data_list: CArray<FactionResourceData>,
        pub reward_drop_set_info: u32,                    // lookup
        pub reward_drop_money: u32,                       // lookup (a key, not an amount)
        pub equip_item_info: u32,                         // lookup
        pub target_level_info: u32,                       // lookup
        pub operation_day: u32,
        pub need_operator_min_count: u32,
        pub need_operator_max_count: u32,
        pub max_combat_power: u32,
        pub operation_type_string: CString<'a>,           // Combat, Building, Farming, ...
        pub stat_data: FactionScheduleStatData,
        pub schedule_spawn_pivot: [f32; 3],
        pub condition_option_list: CArray<FactionConditionOption>,
        pub operation_additional_data_list: CArray<FactionScheduleU32Triple>,
        pub is_repeatable: u8,
        pub is_unlock_operation: u8,
        pub need_request_for_complete: u8,
        pub target_sub_inner_type: CString<'a>,
        pub gimmick_alias_name: CString<'a>,
        pub reward_housing_save_preset_name: CString<'a>,
        pub operation_string_key: CString<'a>,
    }
}

py_binary_struct! {
    pub struct FactionResearchProgress {
        pub progress_info: u32, // lookup
        pub value: u32,
    }
}

py_binary_struct! {
    pub struct FactionResearchData<'a> {
        pub key: u32,
        pub research_name: LocalizableString<'a>,
        pub research_desc: LocalizableString<'a>,
        pub ui_texture_name: u32,                            // hash
        pub drop_set_info: u32,                              // lookup
        pub connect_faction_node_info: u32,                  // lookup
        pub condition_option_list: CArray<FactionConditionOption>,
        pub progress_data_list: CArray<FactionResearchProgress>,
        pub cost_item_list: CArray<FactionAdjacencyMobItem>, // u32 item + 3x u64, same wire shape
        pub research_total_progress: u32,
    }
}

py_binary_struct! {
    // _researchDataList is a map on the wire: u32 count, then per entry
    // u32 map key + FactionResearchData (same bytes as a CArray of this).
    pub struct FactionResearchEntry<'a> {
        pub map_key: u32,
        pub data: FactionResearchData<'a>,
    }
}

py_binary_struct! {
    // FactionEventData_ExecuteCondition (element header).
    pub struct FactionEventExecuteCondition {
        pub event_type: u8,              // = slot index (3, 4, 6 used on 2.04.08)
        pub player_condition: u32,       // lookup
        pub quest_info: u32,             // lookup
        pub stage_info: u32,             // lookup
        pub mission_info: u32,           // lookup
        pub faction_node_info: u32,      // lookup
        pub faction_info: u32,           // lookup
        pub game_variable_info: u32,     // lookup
        pub close_stage_on_end_type: u8,
        pub sub_inner_complete: u8,
    }
}

py_binary_struct! {
    // FactionBlockadingData (FactionEventData._blockData).
    pub struct FactionEventBlockadingData {
        pub blockading_faction_info: u32,                  // lookup
        pub blockading_line_start_node_info: u32,          // lookup
        pub use_block_day: u8,
        pub is_block_by_player: u8,
        pub block_sub_type: u8,
        pub block_end_day: u32,
        pub block_event_key: u32,
        pub sub_blockading_faction_info_list: CArray<u32>, // lookups
    }
}

py_binary_struct! {
    pub struct FactionDailyDeliveryItemPair {
        pub item: u32,   // hashed key
        pub count: u64,
    }
}

py_binary_struct! {
    // FactionEventData (element body).
    pub struct FactionEventData<'a> {
        pub event_data_type: u8,
        pub faction_info: u32,                     // lookup
        pub faction_node_info: u32,                // lookup
        pub leader_character_info: u32,            // lookup
        pub target_faction_node_info: u32,         // lookup
        pub target_faction_info: u32,              // lookup
        pub faction_relation: u8,
        pub due_date_in_seconds: u32,
        pub relation_group_info: u16,              // wire u16
        pub conquered_node_info: u32,              // lookup
        pub is_conquer_enable: u8,
        pub is_capital: u8,
        pub block_data: FactionEventBlockadingData,
        pub is_block_enable: u8,
        pub daily_delivery_item_pair: FactionDailyDeliveryItemPair,
        pub range_value: u32,
        pub spawn_rate: u64,
        pub target_quest_list: CArray<u32>,        // lookups
        pub target_stage: u32,                     // lookup
        pub target_faction_list: CArray<u32>,      // lookups
        pub apply_skill_data: u64,
        pub target_faction_node_list: CArray<u32>, // lookups
        pub target_region_list: CArray<u16>,       // wire u16
        pub target_position: [f32; 3],
        pub is_skill_enable: u8,
        pub is_revive_enable: u8,
        pub is_save_enable: u8,
        pub is_sub_inner_enable: u8,
        pub is_node_enable: u8,
        pub fire_arm_range_type: u32,
        pub target_faction_type: u8,
        pub level_name: u32,                       // lookup (hash)
        pub alias_name: u32,                       // lookup (hash)
        pub desc: LocalizableString<'a>,
        pub event_data_file_line_number: u32,
        pub event_data_file_name: CString<'a>,
    }
}

py_binary_struct! {
    pub struct FactionEventDataEntry<'a> {
        pub execute_condition: FactionEventExecuteCondition,
        pub event_data: FactionEventData<'a>,
    }
}

py_binary_struct! {
    // _factionEventDataList: 13 fixed lists, index = _eventType.
    pub struct FactionEventDataSlots<'a> {
        pub slot_00: CArray<FactionEventDataEntry<'a>>,
        pub slot_01: CArray<FactionEventDataEntry<'a>>,
        pub slot_02: CArray<FactionEventDataEntry<'a>>,
        pub slot_03: CArray<FactionEventDataEntry<'a>>,
        pub slot_04: CArray<FactionEventDataEntry<'a>>,
        pub slot_05: CArray<FactionEventDataEntry<'a>>,
        pub slot_06: CArray<FactionEventDataEntry<'a>>,
        pub slot_07: CArray<FactionEventDataEntry<'a>>,
        pub slot_08: CArray<FactionEventDataEntry<'a>>,
        pub slot_09: CArray<FactionEventDataEntry<'a>>,
        pub slot_10: CArray<FactionEventDataEntry<'a>>,
        pub slot_11: CArray<FactionEventDataEntry<'a>>,
        pub slot_12: CArray<FactionEventDataEntry<'a>>,
    }
}

py_binary_struct! {
    // FactionBlockConditionData (_factionBlockadingDataList element).
    pub struct FactionBlockConditionData {
        pub blockading_condition_info: u32,                // lookup
        pub blockading_faction_info: u32,                  // lookup
        pub blockading_line_start_node_info: u32,          // lookup
        pub use_block_day: u8,
        pub is_block_by_player: u8,
        pub block_sub_type: u8,
        pub block_end_day: u32,
        pub block_event_key: u32,
        pub sub_blockading_faction_info_list: CArray<u32>,
        pub block_quest: u32,                              // lookup
        pub block_mission: u32,                            // lookup
    }
}

py_binary_struct! {
    pub struct FactionConquerData {
        pub conqueror_faction_info: u32,   // lookup
        pub conquer_condition_info: u32,   // lookup
        pub is_capital: u8,
    }
}

pabgh_typed_blob_table! {
    // 2.04.00 order (Mac reader sub_10204FA84). The reader moved most one-byte and key fields
    // to the front, so the typed prefix now runs through `_wayPointDataList_deprecated` with
    // ten more fields typed than before; `_factionScheduleInfoList` onward stays in tail_blob.
    pub struct FactionNodeInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u32,
        pub is_blocked: u8,
        pub is_reblockading_target: u8,
        pub worker_count: u8,
        pub use_custom_way_point_for_dev: u8,
        pub faction_type: u8,
        pub knock_down_condition: u32,
        pub bit_map_color_key_r: u8,
        pub bit_map_color_key_g: u8,
        pub knowledge_info: u32,
        pub religion_sub_level_info: u32,
        pub field_info: u32,
        pub skill_tree_info: u32,
        pub store_info: u16,
        pub royal_supply_info: u16,
        pub connect_research_node_info: u32,
        pub religion_max_block_day: u32,
        pub operation_spawn_radius: f32,
        pub node_radius: f32,
        pub world_position: [f32; 3],
        pub observe_data: FactionNodeObserveData,
        pub memo: CString<'a>,
        pub child_faction_info_list: CArray<u32>,
        pub node_line_main_faction_info_list: CArray<u32>,
        pub apply_skill_data_list: CArray<ApplySkillData>,
        pub resource_item_list: CArray<ResourceItemData>,
        pub revival_stage_info_list: CArray<u32>,
        pub way_point_data_list_deprecated: CArray<WayPointDeprData>,
        pub faction_schedule_info_list: CArray<FactionScheduleInfo<'a>>,
        pub research_data_list: CArray<FactionResearchEntry<'a>>,
        pub faction_event_data_list: FactionEventDataSlots<'a>,
        pub religion_block_cost_list: CArray<u32>,
        pub religion_effect_region_info_list: CArray<u16>,   // wire u16
        pub faction_blockading_data_list: CArray<FactionBlockConditionData>,
        pub faction_conquer_data_list: CArray<FactionConquerData>,
    }
    // 2.04 TAIL TYPED (see the "2.04 TAIL" block above): fields 18+ are typed
    // with the 2.04.08 reader layout; tail_blob is empty on every 2.04.08
    // record (test tail_fully_typed) and only keeps unknown future bytes.
    // The 1.10 FactionSchedule/FactionAdjacency*/FactionNodeBigComposite*
    // structs above are no longer used by this table.
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("factionnode.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                FactionNodeInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "factionnode roundtrip mismatch");
    }

    #[test]
    fn tail_fully_typed() {
        for name in ["factionnode.pabgb", "factionnode.dlc_01.pabgb"] {
            let p = crate::testenv::resolve(name);
            let Ok(data) = std::fs::read(&p) else { eprintln!("SKIP {}", name); continue; };
            let Some(entries) = load_pabgh_offsets(&p.with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP {}", name); continue; };
            let ranges = entry_ranges(&entries, data.len());
            let mut out = Vec::with_capacity(data.len());
            for (i, (k, s, e)) in ranges.iter().enumerate() {
                let mut c = *s;
                let it = FactionNodeInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("{} e{} k=0x{:x}: {}", name, i, k, er));
                assert!(it.tail_blob.is_empty(), "{} e{} k=0x{:x}: {} tail bytes left", name, i, k, it.tail_blob.len());
                it.write_to(&mut out).unwrap();
            }
            assert_eq!(out, data, "{} roundtrip mismatch", name);
        }
    }

    #[test]
    fn json_roundtrip() {
        use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else {
            eprintln!("SKIP: pabgh not found");
            return;
        };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = FactionNodeInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            FactionNodeInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
