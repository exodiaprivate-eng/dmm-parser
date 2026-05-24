//! IDA-derived parser for `QuestGroupInfo.pabgb`.
//!
//! Field layout extracted from Hex-Rays decompile of the parse function
//! in the current Win exe (CrimsonDesert.exe). Field NAMES paired with
//! Mac binary __cstring declaration order. Round-trip-validated against
//! the vanilla pabgb dump from the live game install.
//!
//! DO NOT EDIT BY HAND - regenerate via tools/ida_extract.py.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::QuestGroupInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  15
// Decoded by dmm-parser:   15
// Missing in this struct:  0
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _isSave (direct_u8, stream=1)
// ✅ _factionGroupInfo (reader_2B, stream=2)
// ✅ _isAutoSave (direct_u8, stream=1)
// ✅ _isDev (direct_u8, stream=1)
// ✅ _name (reader_8B, stream=8)
// ✅ _questType (direct_u8, stream=1)
// ✅ _questList (reader_4B, stream=4)
// ✅ _questGroupDesc (reader_8B, stream=8)
// ✅ _stageIconPath (reader_4B, stream=4)
// ✅ _debugColor (direct_u32, stream=4)
// ✅ _stageImagePath (reader_4B, stream=4)
// ✅ _stageTextIconPath (reader_4B, stream=4)
// ✅ _key (direct_u8, stream=1)
// ✅ _isBlocked (direct_u8, stream=1)
// ✅ _stringKey

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct QuestGroupInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub quest_type: u8,
        pub name: LocalizableString<'a>,
        pub quest_group_desc: LocalizableString<'a>,
        pub quest_list: CArray<u32>,
        pub debug_color: u32,
        pub stage_icon_path: u32,
        pub stage_text_icon_path: u32,
        pub stage_image_path: u32,
        pub faction_group_info: u16,
        pub is_save: u8,
        pub is_dev: u8,
        pub is_show_quest_list: u8,
        pub is_auto_save: u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("questgroupinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(QuestGroupInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "questgroupinfo roundtrip bytes mismatch");
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
            items.push(QuestGroupInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let _ = &item;
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            QuestGroupInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
