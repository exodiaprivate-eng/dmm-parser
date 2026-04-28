//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410ED0E0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u32 key                          (_key)
//!   2. CString string_key               (_stringKey)
//!   3. u8 is_blocked                    (_isBlocked)
//!   4. u32 parent_quest                 (_parentQuest, sub_141102CB0
//!      → qword_145F0EF20 lookup)
//!   5. _subMissionList (sub_1411049D0 → struct +24, unknown 16-byte
//!      slot helper) ← TAIL STARTS HERE
//!   6. (body) _executeStageList, _branchMissionList, _startPlayerList,
//!      _fieldReviveList, _giveUpFieldReviveList, _triggerVolumeData,
//!      _rewardList, _resultDataList, _rewardInventoryKey, _uiDesc, …
//!      50+ wire reads in body.
//!
//! Steps 1-4 are typed. The mission body has many helpers; reopens
//! cleanly when each is decoded.
//!
//! Helper: `sub_141102CB0` = u32 lookup at qword_145F0EF20.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct MissionInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub parent_quest: u32,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\missioninfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\missioninfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                MissionInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "missioninfo roundtrip mismatch");
    }
}
