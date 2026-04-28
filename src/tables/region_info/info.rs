//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410F5140` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u16 key                          (_key, pabgh format 2)
//!   2. CString string_key               (_stringKey)
//!   3. u8 is_blocked                    (_isBlocked)
//!   4. LocalizableString display_region_name (_displayRegionName)
//!   5. u32 knowledge_info               (_knowledgeInfo, sub_1411006D0)
//!   6. CArray<RegionSubA> region_enter_knowledge_info_list
//!      (_regionEnterknowledgeInfoList, sub_141104230: each item =
//!      u32 lookup via sub_1411006D0 + u32 = 8 wire bytes)
//!   7. u16 parent_region_info           (_parentRegionInfo,
//!      sub_1410FF220 → qword_145F0DA80)
//!   8. CArray<u16> child_region_info_list (_childRegionInfoList,
//!      sub_1410FFAC0 → qword_145F0DA80)
//!   9. u8 bitmap_color                  (_bitmapColor)
//!  10. u8 overrided_max_height          (_overriedMaxHeight — game
//!      typo'd "overrided" not "overridden")
//!  11. u32 region_type                  (_regionType)
//!  12. u8 fog_clear_condition           (_fogClearCondition)
//!  13. u32 limit_vehicle_run            (_limitVehicleRun, sub_1410FF430)
//!  14. u8 is_town                       (_isTown)
//!  15. u8 is_wild                       (_isWild)
//!  16. u8 is_ui_map_disable             (_isUIMapDisable)
//!  17. u8 is_housing_region             (_isHousingRegion)
//!  18. u8 is_none_play_zone             (_isNonePlayZone)
//!  19. u8 vehicle_mercenary_allow_type  (_vehicleMercenaryAllowType)
//!  20. u8 is_world_map_road_path_findable (_isWorldMapRoadPathFindable)
//!  21. u8 u8_123 (no matching Korean string — possibly a padding
//!      byte or off-by-one in the original Win-build analysis;
//!      preserved as-is for round-trip)
//!  22. sub_1411043B0 → struct +128 (_gimmickAliasPointerList? unknown)
//!      ← TAIL STARTS HERE
//!  23. (body) _domainFactionList, _tagList, … and 3 more unknown helpers
//!
//! Steps 1-21 are typed. Tail captures 3 unknown body helpers.
//!
//! Helpers: `sub_1410FF220` = single u16 lookup at qword_145F0DA80
//! (wire 2); `sub_1410FFAC0` = CArray<u16> hash-keyed at same dict;
//! `sub_141104230` = CArray<{u32 lookup + u32}=8B>.

use crate::binary::*;
use crate::pabgh_typed_blob_table;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct RegionSubA {
        pub lookup_a: u32,
        pub raw_b: u32,
    }
}

pabgh_typed_blob_table! {
    pub struct RegionInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub display_region_name: LocalizableString<'a>,
        pub knowledge_info: u32,
        pub region_enter_knowledge_info_list: CArray<RegionSubA>,
        pub parent_region_info: u16,
        pub child_region_info_list: CArray<u16>,
        pub bitmap_color: u8,
        pub overrided_max_height: u8,
        pub region_type: u32,
        pub fog_clear_condition: u8,
        pub limit_vehicle_run: u32,
        pub is_town: u8,
        pub is_wild: u8,
        pub is_ui_map_disable: u8,
        pub is_housing_region: u8,
        pub is_none_play_zone: u8,
        pub vehicle_mercenary_allow_type: u8,
        pub is_world_map_road_path_findable: u8,
        pub u8_123: u8,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\regioninfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\regioninfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                RegionInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "regioninfo roundtrip mismatch");
    }
}
