//! Tier 1.5 — typed prefix + tail blob.
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
//!      ← TAIL STARTS HERE
//!  19. (tail) u8 + sub_1410A9D40 (CString hash) + u8 + u32 lookup
//!      (sub_1410FF430) + u8 + u8 + sub_141115A30 (CArray of 144-byte
//!      items via sub_1410DE350 — 128-byte composite, blocked)
//!  20. (tail) 13× sub_141128990 — CArray of 288-byte items via
//!      sub_1410DD2A0 + sub_1410DD420 (HARD BLOCKER)
//!  21. (tail) u8 + sub_1410DE690 (28-byte target) + u32 raw +
//!      sub_141100510 (CArray<u32>) + sub_1410FFAC0 (CArray<u16>) +
//!      sub_141103770 (u16 lookup)
//!
//! Steps 1-18 typed (18 of 32 catalog fields surfaced). FactionSchedule
//! depends on 4 inner sub-structs (FactionScheduleEntry48,
//! FactionScheduleU128Pair, FactionScheduleSlotInner,
//! FactionScheduleU64Triple, FactionScheduleU32Triple) plus the shared
//! FactionNodeRawDataExt. Reopens cleanly when sub_141115A30,
//! sub_141128990, sub_1410DE690 are decoded.

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
    pub struct WayPointDeprData {
        pub lookup_a: u16,
        pub lookup_b: u16,
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

pabgh_typed_blob_table! {
    pub struct FactionNodeInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub knowledge_info: u32,
        pub skill_tree_info: u32,
        pub connect_research_node_info: u32,
        pub store_info: u16,
        pub royal_supply_info: u16,
        pub memo: CString<'a>,
        pub child_faction_info_list: CArray<u32>,
        pub node_line_main_faction_info_list: CArray<u32>,
        pub world_position: [f32; 3],
        pub node_radius: f32,
        pub apply_skill_data_list: CArray<ApplySkillData>,
        pub resource_item_list: CArray<ResourceItemData>,
        pub revival_stage_info_list: CArray<u32>,
        pub way_point_data_list_deprecated: CArray<WayPointDeprData>,
        pub faction_schedule_list: CArray<FactionSchedule<'a>>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\factionnode.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\factionnode.pabgh";


    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
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
    fn json_roundtrip() {
        use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
        let Ok(data) = std::fs::read(PABGB) else {
            eprintln!("SKIP: missing fixture {}", PABGB);
            return;
        };
        let Some(entries) = load_pabgh_offsets(PABGH) else {
            eprintln!("SKIP: missing pabgh fixture {}", PABGH);
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
