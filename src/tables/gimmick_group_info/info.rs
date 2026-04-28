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
//!   5. [u8; 8] battery_init_capacity            (_batteryInitCapacity)
//!   6. [u8; 8] battery_total_capacity           (_batteryTotalCapacity)
//!      ← TAIL STARTS HERE
//!   7. _linkSignalGroupList, _propertyList, _gimmickTagList,
//!      _gimmickChartPath, _gimmickType, _gimmickPlacementStyle,
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
//! Steps 1-6 are typed (6 fields). Body has 100+ wire reads.
//!
//! Helper: `sub_141104AE0` = u32 lookup at qword_145F11D70.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct GimmickGroupInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub main_gimmick_group_info_of_combination: u32,
        pub battery_init_capacity: [u8; 8],
        pub battery_total_capacity: [u8; 8],
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
}
