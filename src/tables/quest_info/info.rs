//! Tier 1.5 (extended) — typed prefix through field 28, tail blob from
//! `_questDialogFilterDataList` onward.
//!
//! Reader (Mac CrimsonDesert_Steam): entry-level `sub_1018545F0` at
//! 0x1018545F0. 35 wire fields total.
//!
//! Fields 1-28 are fully typed (this file). Fields 29-35 stay in
//! `tail_blob` because field 29 (`_questDialogFilterDataList`) is a
//! tagged-variant CArray (`FilterCondition` has 11 variants discriminated
//! by a u8 tag with 0-8 byte payloads) and decoding it requires the
//! tagged-variant family work tracked in task #66. Once that lands,
//! fields 30-35 (which are simple CArrays / scalars / hash lookups)
//! reopen for free.
//!
//! Wire layout (in order; canonical names from Mac Korean error strings):
//!   1.  u32 key                       (sub_100F133BC, QuestKey, template
//!                                      `<...,unsigned int>`)
//!   2.  CString string_key
//!   3.  u8 is_blocked
//!   4.  u8 quest_type                 (sub_10136CA5C = vtable[2] width 1)
//!   5.  u8 quest_category             (vtable[2] width 1)
//!   6.  LocalizableString name
//!   7.  LocalizableString desc
//!   8.  u16 quest_group_info          (sub_10183DCC8, QuestGroupKey wire u16,
//!                                      template `<...,unsigned short>`)
//!   9.  u32 faction_info              (FactionKey wire u32 hash, runtime u16)
//!  10. FactionStateData faction_state_data (sub_101848C10, fixed 4-field
//!       struct, see below)
//!  11. BranchData branch_data         (sub_101652724, fixed 6-field struct,
//!       see below)
//!  12. CArray<u32> start_player_list  (CharacterKey hash list)
//!  13. CArray<BranchData> branch_data_list  (sub_101885280)
//!  14. CArray<u32> executor_quest_list  (sub_10186F2C4, QuestKey hash)
//!  15. CArray<u32> gauge_list         (sub_101885460, QuestGaugeKey wire u32)
//!  16. CArray<u32> mission_list       (sub_10186E494, MissionKey hash)
//!  17. CArray<u32> stage_list         (sub_101667390, StageKey hash)
//!  18. u32 start_mission              (MissionKey hash)
//!  19. u32 start_stage                (StageKey hash)
//!  20. u32 stage_icon_path            (StringInfoKey hash → u16 lookup)
//!  21. u32 stage_text_icon_path       (StringInfoKey)
//!  22. u32 stage_image_path           (StringInfoKey)
//!  23. u32 playable_mission_count
//!  24. u32 playable_stage_count
//!  25. CString test_tag
//!  26. u32 game_start_stage           (StageKey hash)
//!  27. u32 game_start_sub_timeline    (sub_1006B40F4, u32 wire + hash compute)
//!  28. CString memo
//!  --- TAIL STARTS HERE ---
//!  29. _questDialogFilterDataList (CArray of 144-byte QuestDialog_FilterData
//!       — contains tagged-variant FilterCondition; needs task #66)
//!  30. CArray<u32> _dialogMustMissionInfoList (would be tractable post-#66)
//!  31. u32 _npcDialogMustCondition
//!  32. u8 _isSave
//!  33. u8 _isContinuousMission
//!  34. u8 _isRepeatable
//!  35. u32 _debugColor (sub_1006B4CD0 = vtable[2] width 4)
//!
//! `FactionStateData` (sub_101848C10, struct stride 24 bytes):
//!   - CArray<u8> activate_faction_state_list (sub_101879D70, element u8)
//!   - u32 player_condition_info     (ConditionKey hash)
//!   - u32 relation_target_faction_info (FactionKey hash)
//!   - u8 relation_type
//!
//! `BranchData` (sub_101652724, struct stride 16 bytes, wire 18 bytes):
//!   - u32 quest_key                  (QuestKey hash)
//!   - u32 condition_key              (ConditionKey hash)
//!   - u8 byte_a
//!   - u8 byte_b
//!   - u32 u32_a
//!   - u32 u32_b

use crate::binary::*;
use crate::pabgh_typed_blob_table;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct FactionStateData {
        pub activate_faction_state_list: CArray<u8>,
        pub player_condition_info: u32,
        pub relation_target_faction_info: u32,
        pub relation_type: u8,
    }
}

py_binary_struct! {
    pub struct BranchData {
        pub quest_key: u32,
        pub condition_key: u32,
        pub byte_a: u8,
        pub byte_b: u8,
        pub u32_a: u32,
        pub u32_b: u32,
    }
}

pabgh_typed_blob_table! {
    pub struct QuestInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub quest_type: u8,
        pub quest_category: u8,
        pub name: LocalizableString<'a>,
        pub desc: LocalizableString<'a>,
        pub quest_group_info: u16,
        pub faction_info: u32,
        pub faction_state_data: FactionStateData,
        pub branch_data: BranchData,
        pub start_player_list: CArray<u32>,
        pub branch_data_list: CArray<BranchData>,
        pub executor_quest_list: CArray<u32>,
        pub gauge_list: CArray<u32>,
        pub mission_list: CArray<u32>,
        pub stage_list: CArray<u32>,
        pub start_mission: u32,
        pub start_stage: u32,
        pub stage_icon_path: u32,
        pub stage_text_icon_path: u32,
        pub stage_image_path: u32,
        pub playable_mission_count: u32,
        pub playable_stage_count: u32,
        pub test_tag: CString<'a>,
        pub game_start_stage: u32,
        // _gameStartSubTimeline (sub_1006B40F4): wire is `u32 length + N
        // raw bytes` (CString-shaped); runtime hashes to a u32 stored at
        // struct +268. We round-trip the raw wire to preserve the string.
        pub game_start_sub_timeline: CString<'a>,
        pub memo: CString<'a>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\questinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\questinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                QuestInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "questinfo roundtrip mismatch");
    }
}
