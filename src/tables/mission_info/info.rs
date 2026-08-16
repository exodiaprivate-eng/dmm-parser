//! Tier 1 — fully typed (no _tail_b64). FULLY DECODED — no tail remains.
//!
//! Reader (Tier IDA verified 2026-05-19 vs CrimsonDesert.exe md5
//! 3d614280…): `sub_1410CF190` — MissionInfo deserializer (via
//! "MissionInfo" class block at 0x144B0B720+). (Cited `sub_1410ED0E0`
//! stale. The "TAIL STARTS HERE / Steps 1-7 typed / 15 of 40" notes
//! that USED to be here were stale too — the struct has since been
//! fully decoded and matches the reader field-for-field.)
//!
//! All 34 struct fields confirmed against the reader, in order:
//!   a2+0   u32 key            a2+8   CString string_key
//!   a2+16  u8 is_blocked      a2+20  u32 parent_quest (sub_1410E5260)
//!   a2+24  CArray sub_mission_list      a2+40  CArray branch_mission_list
//!   a2+56  CArray execute_stage_list (loop, 16B elems via sub_1410CF880)
//!   a2+72  CArray start_player_list     a2+88  CArray field_revive_list
//!   a2+104 CArray give_up_field_revive_list
//!   a2+120 COptional trigger_volume_data (sub_1410E93E0 — formerly the
//!          "hard blocker", now fully decoded)
//!   a2+128 CArray reward_list           a2+144 CArray result_data_list
//!   a2+160 u16 reward_inventory_key     a2+168 MissionUIDesc ui_desc
//!   a2+248/280/312/344  LocalizableString label_a..d
//!   a2+376 u32 result_data_2_lookup     a2+384 CArray result_data_list_2
//!   a2+400 CArray mission_stage_list    a2+416 u32 category_info
//!   a2+418 u16 raw_418   a2+420 u16 raw_420   a2+424 u32 raw_424
//!   a2+428..440  flag_428..flag_440 (13× u8, consecutive 1-byte reads)
//!   a2+444 u32 trailing_u32
//! No discrepancy, no missing fields. ID-ref fields read wire-width →
//! u16 RAM as elsewhere; Rust models the wire.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::MissionInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  40
// Decoded by dmm-parser:   15
// Missing in this struct:  25
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ⏳ _showMiniMap (direct_15B, stream=15)
// ⏳ _checkCompleteCountAtOnce (direct_15B, stream=15)
// ⏳ _ignoreRepeatOnDead (direct_15B, stream=15)
// ⏳ _isOperationMission (direct_15B, stream=15)
// ⏳ _targetQuestDialogKey (reader_4B, stream=4)
// ⏳ _isShowAlertPlaying (direct_15B, stream=15)
// ⏳ _checkOverlapType (direct_15B, stream=15)
// ⏳ _existStart (direct_15B, stream=15)
// ⏳ _optional (direct_15B, stream=15)
// ⏳ _existComplete (direct_15B, stream=15)
// ⏳ _existHaveCount (direct_15B, stream=15)
// ⏳ _preCheck (direct_15B, stream=15)
// ⏳ _existFail (direct_15B, stream=15)
// ⏳ _missionFunctionList
// ⏳ _parentMissionInfo (reader_4B, stream=4)
// ⏳ _repeatCondition (reader_4B, stream=4)
// ⏳ _challengeEventList
// ⏳ _completeTime (direct_u16, stream=2)
// ⏳ _limitTime (direct_u16, stream=2)
// ⏳ _completeType (direct_15B, stream=15)
// ⏳ _completeCount (direct_u32, stream=4)
// ✅ _resultDataList
// ✅ _rewardList (reader_4B, stream=4)
// ✅ _uiDesc
// ✅ _rewardInventoryKey (direct_u16, stream=2)
// ⏳ _completeName (reader_8B, stream=8)
// ⏳ _name (reader_8B, stream=8)
// ⏳ _completeLog (reader_8B, stream=8)
// ⏳ _desc (reader_8B, stream=8)
// ✅ _subMissionList (reader_4B, stream=4)
// ✅ _parentQuest (reader_4B, stream=4)
// ✅ _branchMissionList (reader_4B, stream=4)
// ✅ _executeStageList
// ✅ _fieldReviveList (reader_4B, stream=4)
// ✅ _startPlayerList (reader_4B, stream=4)
// ✅ _triggerVolumeData (reader_1B, stream=1)
// ✅ _giveUpFieldReviveList (reader_4B, stream=4)
// ✅ _key
// ✅ _isBlocked (direct_15B, stream=15)
// ✅ _stringKey

use crate::binary::*;
use crate::py_binary_struct;

// sub_1410AA0D0 inner — Quaternion ([f32; 4], 16 wire bytes).
py_binary_struct! {
    pub struct Quaternion {
        pub x: f32,
        pub y: f32,
        pub z: f32,
        pub w: f32,
    }
}

// sub_1410AA1B0 — Transform: Vec3 (wire first) + Quaternion + Vec3.
// Total 40 wire bytes. Mem offsets are out-of-order but wire shape
// is just sequential reads.
py_binary_struct! {
    pub struct TriggerVolumeTransform {
        pub vec3_a: [f32; 3],
        pub rotation: Quaternion,
        pub vec3_b: [f32; 3],
    }
}

// sub_141D7FE40 — TriggerVolumeData (88 mem bytes / 9 wire fields).
py_binary_struct! {
    pub struct TriggerVolumeData<'a> {
        pub flag_a: u8,
        pub transform: TriggerVolumeTransform,
        pub tag: CString<'a>,             // sub_1410A9D40 (wire = CString)
        pub name: CString<'a>,
        pub flag_b: u8,
        pub vec_a: [f32; 3],
        pub vec_b: [f32; 3],
        pub flag_c: u8,
        pub flag_d: u8,
    }
}

// sub_1410ECC50 — sub_14110DCE0 inner, 104 mem bytes / 12 wire fields.
py_binary_struct! {
    pub struct MissionResultData2<'a> {
        pub flag_a: u8,
        pub name_a: CString<'a>,
        pub name_b: CString<'a>,
        pub region_lookup: u16,             // sub_1410FF220 (wire u16)
        pub lookup_a: u16,                  // sub_141106810 (wire u16)
        pub flag_b: u8,
        pub name_c: CString<'a>,
        pub list_a: CArray<u32>,            // sub_1410FFC20
        pub list_b: CArray<u32>,            // sub_141102FF0
        pub list_c: CArray<u32>,            // sub_141102EF0
        pub raw: u64,
        pub flag_c: u8,
    }
}

// sub_1410ECFD0 — sub_14110DB10 inner, 32 mem bytes / 9 wire fields.
// First 4 fields are sub_1410E2030 (u8 flag + 3× u32 lookup).
py_binary_struct! {
    pub struct MissionStageData {
        pub flag_a: u8,
        pub lookup_a: u32,                  // sub_1410FF430
        pub lookup_b: u32,                  // sub_1410FF430
        pub lookup_c: u32,                  // sub_1410FF430
        pub list: CArray<u32>,              // sub_141101610
        pub lookup_d: u32,                  // sub_1410FF5C0
        pub lookup_e: u16,                  // sub_141100620 (wire u16)
        pub flag_b: u8,
        pub flag_c: u8,
    }
}

// sub_1410EC8B0 — MissionUIDesc, 80 mem bytes / 18 wire fields.
py_binary_struct! {
    pub struct MissionUIDesc {
        pub icon_a: u32,           // read_u32_lookup_DA30
        pub icon_b: u32,           // read_u32_lookup_DA30
        pub icon_c: u32,           // read_u32_lookup_DA30
        pub lookup_a: u32,         // sub_1410FF050
        pub lookup_b: u32,         // sub_1410FF430
        pub lookup_c: u32,         // sub_141102D90
        pub list_a: CArray<u32>,   // sub_141101610 → qword_145F0EF38
        pub list_b: CArray<u32>,   // inline CArray<u32 raw>
        pub lookup_d: u32,         // sub_141101D50
        pub vec3: [f32; 3],
        pub raw_a: u32,
        pub unk_new_u32: u32,
        pub flag_a: u8,
        pub flag_b: u8,
        pub flag_c: u8,
        pub flag_d: u8,
        pub flag_e: u8,
        pub flag_f: u8,
        pub trailing: u16,         // sub_141106760 (wire u16)
        // 1.18 — 4 bytes that were never modelled, so EVERY record fell to
        // blob-fallback. Proven from the wire, not guessed: in record 0 the
        // label_a LocalizableString has index 0x5FE5F346_00000101 and a default
        // string of "6910196685243154689" — the same number in decimal. That
        // pins label_a's start exactly, and it is 4 bytes later than the old
        // struct put it. Candidate canonical name: _targetQuestDialogKey
        // (the catalog lists it as reader_4B, stream=4, still undecoded).
        pub unk_after_trailing: u32,
    }
}

// sub_1410ECE20 inner — 48 mem bytes / 11 wire fields.
py_binary_struct! {
    pub struct MissionResultData {
        pub flag_a: u8,
        pub list: CArray<u32>,           // sub_1410FEF40
        pub lookup_a: u32,                // sub_141100370
        pub lookup_b: u32,                // sub_1410FF5C0
        pub raw_a: u32,
        pub lookup_c: u32,                // read_u32_lookup_DA30
        pub lookup_d: u32,                // read_u32_lookup_DA30
        pub raw_b: u32,
        pub lookup_e: u32,                // sub_141102D90
        pub flag_b: u8,
        pub flag_c: u8,
    }
}

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

py_binary_struct! {
    pub struct MissionInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub parent_quest: u32,
        pub sub_mission_list: CArray<u32>,
        pub branch_mission_list: CArray<MissionBranchData>,
        pub execute_stage_list: CArray<MissionExecuteStage>,
        pub start_player_list: CArray<u32>,
        pub field_revive_list: CArray<u32>,
        pub give_up_field_revive_list: CArray<u32>,
        pub trigger_volume_data: COptional<TriggerVolumeData<'a>>,
        pub reward_list: CArray<u32>,
        pub result_data_list: CArray<MissionResultData>,
        pub reward_inventory_key: u16,
        pub ui_desc: MissionUIDesc,
        pub label_a: LocalizableString<'a>,
        pub label_b: LocalizableString<'a>,
        pub label_c: LocalizableString<'a>,
        pub label_d: LocalizableString<'a>,
        pub result_data_2_lookup: u32,       // sub_141102D90 (KNOWN)
        pub result_data_list_2: CArray<MissionResultData2<'a>>,
        pub mission_stage_list: CArray<MissionStageData>,
        pub category_info: u32,             // sub_1410FF430
        pub raw_418: u16,
        pub raw_420: u16,
        pub raw_424: u32,
        pub flag_428: u8,
        pub flag_429: u8,
        pub flag_430: u8,
        pub flag_431: u8,
        pub flag_432: u8,
        pub flag_433: u8,
        pub flag_434: u8,
        pub flag_435: u8,
        pub flag_436: u8,
        pub flag_437: u8,
        pub flag_438: u8,
        pub flag_439: u8,
        pub flag_440: u8,
        // 1.18 — a 14TH flag. Proven by end-alignment over all 6822 records:
        // counting BACK from each record's end, byte -5 is 0x01 in 100% of them
        // (a constant boolean), and bytes -4..-1 carry the variable u32. That
        // makes the flag run exactly bytes -18..-5 = 14 bytes, not 13.
        // (Reading it the other way — u32 then a trailing u8 — gives a u32 with
        // 555 wild values AND a varying last byte, i.e. misalignment.)
        pub flag_441: u8,
        pub trailing_u32: u32,              // sub_141BD4120 (raw u32)
    }
}

impl<'a> MissionInfo<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> std::io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        let consumed = *offset - start;
        if consumed != entry_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("MissionInfo: consumed {} bytes, expected {}", consumed, entry_size),
            ));
        }
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("missioninfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
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
