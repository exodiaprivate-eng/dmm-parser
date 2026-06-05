//! IDA-derived parser for `QuestGaugeInfo.pabgb`.
//!
//! Field layout extracted from Hex-Rays decompile of the parse function
//! in the current Win exe (CrimsonDesert.exe). Field NAMES paired with
//! Mac binary __cstring declaration order. Round-trip-validated against
//! the vanilla pabgb dump from the live game install.
//!
//! DO NOT EDIT BY HAND - regenerate via tools/ida_extract.py.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::QuestGaugeInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  12
// Decoded by dmm-parser:   12
// Missing in this struct:  0
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _completeEventData
// ✅ _startEventData
// ✅ _targetMissionInfoList (reader_4B, stream=4)
// ✅ _questInfoList (reader_4B, stream=4)
// ✅ _factionInfoList (reader_4B, stream=4)
// ✅ _excludeStageInfoList (reader_4B, stream=4)
// ✅ _percent
// ✅ _factionNodeInfoList (reader_4B, stream=4)
// ✅ _stringKey
// ✅ _key
// ✅ _parentQuest (reader_4B, stream=4)
// ✅ _isBlocked (direct_u8, stream=1)

use crate::binary::*;
use crate::py_binary_struct;

// EventData: 13-byte composite per empirical sweep.
// Layout: u32 type (small enum, 0x0c/0x6a observed) + u32 (always 0) +
// u32 (varies, often 0 or 0xffe51c00) + u8 trailing (0x00 or 0xff).
py_binary_struct! {
    pub struct QuestGaugeEventData {
        pub event_type: u32,
        pub field_b: u32,
        pub field_c: u32,
        pub trailing: u8,
    }
}

py_binary_struct! {
    pub struct QuestGaugeInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub parent_quest: u32,
        pub start_event_data: QuestGaugeEventData,
        pub complete_event_data: QuestGaugeEventData,
        pub quest_info_list: CArray<u32>,
        pub target_mission_info_list: CArray<u32>,
        pub exclude_stage_info_list: CArray<u32>,
        pub faction_info_list: CArray<u32>,
        pub faction_node_info_list: CArray<u32>,
        pub percent: u64,
        // 1.10: +5 trailing bytes after percent (u32 + u8, observed all-zero).
        // Verified via wire-walker: reconciles all 509 records (byte-exact).
        pub trailing_u32_110: u32,
        pub trailing_u8_110: u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("questgaugeinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(QuestGaugeInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "questgaugeinfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(QuestGaugeInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let _ = &item;
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            QuestGaugeInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
