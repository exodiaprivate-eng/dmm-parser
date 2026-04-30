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
//!      sub_1411006D0 → qword_145F0DA28)
//!   5. u32 skill_tree_info                          (_skillTreeInfo,
//!      sub_1411035A0 → qword_145F1A740)
//!   6. u32 connect_research_node_info               (_connectResearchNodeInfo,
//!      sub_141101D50 → qword_145F0EEE8)
//!   7. u16 store_info                               (_storeInfo,
//!      sub_141103610 → qword_145F15038 — wire u16)
//!   8. u16 royal_supply_info                        (_royalSupplyInfo,
//!      sub_1411036C0 → qword_145F113A0 — wire u16)
//!   9. CString memo                                 (_memo)
//!  10. CArray<u32> child_faction_info_list          (_childFactionInfoList,
//!      sub_141102FF0 → qword_145F0EEE8)
//!  11. CArray<u32> node_line_main_faction_info_list (_nodeLineMainFactionInfoList,
//!      sub_141102FF0)
//!  12. [f32; 3] world_position                      (_worldPosition, Vec3)
//!  13. f32 node_radius                              (_nodeRadius)
//!  14. CArray<ApplySkillData> apply_skill_data_list (_applySkillDataList,
//!      sub_141115FD0; per element: u32 lookup + u64 raw +
//!      CArray<u8> + u32 + u32 + u8 — 40 mem bytes)
//!  15. CArray<ResourceItemData> resource_item_list  (_resourceItemList,
//!      sub_141115D90; per element: u32 + u32 + 8 raw + 8 raw +
//!      CArray<u8> + u32 + u32 + u8 — 48 mem bytes)
//!  16. CArray<u32> revival_stage_info_list          (_revivalStageInfoList,
//!      sub_141101610 → qword_145F0EF38)
//!  17. CArray<WayPointDeprData> way_point_data_list_deprecated
//!      (_wayPointDataList_deprecated, sub_141115BA0; per element:
//!      u16 + u16 + CArray<[u8; 12]> — 24 mem bytes)
//!      ← TAIL STARTS HERE
//!  18. (tail) inline CArray of 280-byte items via sub_1410DDE60 →
//!      a2+152, has multiple unknown sub-helpers (sub_141103420,
//!      sub_141103530, sub_1410DD140); stop here.
//!  19. (tail) u8 + sub_1410A9D40 + u8 + u32 lookup + u8 + u8 +
//!      sub_141115A30 (CArray of 144-byte items via sub_1410DE350,
//!      itself a 128-byte composite — DEEP)
//!  20. (tail) 13× sub_141128990 — CArray of 288-byte items via
//!      sub_1410DD2A0+sub_1410DD420 (HARD BLOCKER)
//!  21. (tail) u8 + sub_1410DE690 + 4 raw + sub_141100510 (CArray<u32>)
//!      + sub_1410FFAC0 (CArray<u16>) + sub_141103770
//!
//! Steps 1-17 are typed (17 of 31 catalog fields surfaced). Reopens
//! cleanly when the tail composite readers are decoded.
//!
//! New helpers from this promotion: `sub_141115FD0` = CArray<40-byte
//! ApplySkillData>, `sub_141115D90` = CArray<48-byte ResourceItemData>,
//! `sub_141115BA0` = CArray<24-byte WayPointDeprData>, `sub_1410DDBC0` =
//! shared CArray<u8> + u32 lookup + u32 lookup + u8 sub-shape used by
//! both ApplySkillData and ResourceItemData.

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
        // Promoted from [u8;12] to [f32;3] — name "Triplet" + 12 bytes is
        // canonically a Vec3. 0 vanilla entries (deprecated path) so no
        // NaN risk.
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
