//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410E4450` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u32 key                                  (_key)
//!   2. CString string_key                       (_stringKey)
//!   3. u8 is_blocked                            (_isBlocked)
//!   4. u32 main_gimmick_group_info_of_combination
//!      (_mainGimmickGroupInfoOfCombination, sub_141104AE0 →
//!      qword_145F11D70 lookup)
//!   5. u64 battery_init_capacity            (_batteryInitCapacity)
//!   6. u64 battery_total_capacity           (_batteryTotalCapacity)
//!   7. CArray<GimmickProperty> link_signal_group_list
//!      (_linkSignalGroupList, sub_141113BF0 wraps sub_1410E3D20;
//!      per element: CString name + u8 + u32 + u8 + u32 + u32 +
//!      u64 + u64 — 30 + variable wire bytes)
//!   8. CArray<u32> property_list                (_propertyList,
//!      sub_141101AB0 — wire u32 per element, mem u32)
//!   9. CArray<CString> gimmick_tag_list         (_gimmickTagList,
//!      sub_141102990 — runtime hashes each tag to u32 via
//!      sub_1410A9D40, wire is CString)
//!      ← TAIL STARTS HERE
//!  10. _gimmickChartPath, _gimmickType, _gimmickPlacementStyle,
//!      _gimmickInterfaceType, _gimmickRemoteCatchableData,
//!      _autoTargetingConstraintDataList, _gimmickConstraintDataList,
//!      _gimmickInfoList, _gameEventHandlerList,
//!      _unlockableIDataList, _defaultSpawnReasonHash,
//!      _initialBodyMotionType,
//!      _sequencerLevelAllowGimmickEventKeyList,
//!      _sequencerLevelConnectAliasNameList, _gimmickAliasDataList,
//!      _logoutTimeAfterBreak, _attackByCollisionInfoListKey,
//!      _useSlidingMotionProperty, _isEditorUseable,
//!      _isGetKnowledgeWhenGetItem, _isUseConstrainSound, …
//!
//! Steps 1-9 are typed (9 fields). Body has 80+ wire reads with
//! several deep composites; reopens cleanly when those are decoded.
//!
//! Helper: `sub_141104AE0` = u32 lookup at qword_145F11D70.
//! `sub_141113BF0` = CArray<GimmickProperty> (48 mem bytes/element
//! via sub_1410E3D20).
//! `sub_1410E3D20` = inner GimmickProperty reader (8 wire fields).
//! `sub_141101AB0` = CArray<u32> (4 wire bytes/element).
//! `sub_141102990` = CArray<CString> via sub_1410A9D40
//! (CString-hash; wire bytes are u32 length + N raw bytes).

use crate::binary::*;
use crate::pabgh_typed_blob_table;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct GimmickProperty<'a> {
        pub name: CString<'a>,
        pub flag_a: u8,
        pub raw_a: u32,
        pub flag_b: u8,
        pub raw_b: u32,
        pub raw_c: u32,
        pub raw_d: u64,
        pub raw_e: u64,
    }
}

pabgh_typed_blob_table! {
    pub struct GimmickGroupInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub main_gimmick_group_info_of_combination: u32,
        pub battery_init_capacity: u64,
        pub battery_total_capacity: u64,
        pub link_signal_group_list: CArray<GimmickProperty<'a>>,
        pub property_list: CArray<u32>,
        pub gimmick_tag_list: CArray<CString<'a>>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gimmickgroupinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gimmickgroupinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                GimmickGroupInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "gimmickgroupinfo roundtrip mismatch");
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
            let item = GimmickGroupInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            GimmickGroupInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
