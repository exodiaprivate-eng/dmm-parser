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
//!   5. CArray<u32> sub_mission_list     (_subMissionList, sub_1411049D0
//!      → qword_145F0EF00)
//!   6. CArray<MissionBranchData> branch_mission_list
//!      (sub_1411068C0 → sub_1410F3380; per element: u32 lookup +
//!      u32 lookup + 2× u32 raw + 3× u8 — 19 wire bytes / 20 mem)
//!   7. CArray<MissionExecuteStage> execute_stage_list
//!      (inline CArray of 16-byte items via sub_1410ED7D0; per element:
//!      u32 lookup + u32 lookup + 2× u32 raw + 2× u8 — 18 wire bytes)
//!      ← TAIL STARTS HERE
//!   8. (body) _startPlayerList, _fieldReviveList, _giveUpFieldReviveList,
//!      _triggerVolumeData, _rewardList, _resultDataList,
//!      _rewardInventoryKey, _uiDesc, … 30+ more wire reads.
//!
//! Steps 1-7 are typed. Body has many helpers; reopens cleanly when each
//! is decoded.

use crate::binary::*;
use crate::pabgh_typed_blob_table;
use crate::py_binary_struct;

// sub_1410F3380 inner — 20 mem bytes / 7 wire fields.
py_binary_struct! {
    pub struct MissionBranchData {
        pub lookup_a: u32,    // sub_141102D20
        pub lookup_b: u32,    // sub_1410FF430
        pub raw_a: u32,
        pub raw_b: u32,
        pub flag_a: u8,
        pub flag_b: u8,
        pub flag_c: u8,
    }
}

// sub_1410ED7D0 inner — 16 mem bytes / 6 wire fields.
py_binary_struct! {
    pub struct MissionExecuteStage {
        pub lookup_a: u32,    // sub_141102D90
        pub lookup_b: u32,    // sub_1410FF430
        pub raw_a: u32,
        pub raw_b: u32,
        pub flag_a: u8,
        pub flag_b: u8,
    }
}

pabgh_typed_blob_table! {
    pub struct MissionInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub parent_quest: u32,
        pub sub_mission_list: CArray<u32>,
        pub branch_mission_list: CArray<MissionBranchData>,
        pub execute_stage_list: CArray<MissionExecuteStage>,
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
            let item = MissionInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            MissionInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
