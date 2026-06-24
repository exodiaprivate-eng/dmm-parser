//! Hand-corrected: IDA-derived parser for `DyeColorGroupInfo.pabgb`.
//!
//! Per IDA sub_1410DA7C9: dye_color_data_list element is
//! { u32 raw + u32 sub_1410FF430-hash } = 8 bytes total.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::DyeColorGroupInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  6
// Decoded by dmm-parser:   6
// Missing in this struct:  0
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _dyeColorDataList
// ✅ _isBlocked (direct_13B, stream=13)
// ✅ _iconPath (reader_4B, stream=4)
// ✅ _dyeColorGroupName (reader_8B, stream=8)
// ✅ _stringKey
// ✅ _key

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct DyeColorEntry {
        pub raw_color: u32,
        pub texture_lookup: u32,
    }
}

py_binary_struct! {
    pub struct DyeColorGroupInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub dye_color_data_list: CArray<DyeColorEntry>,
        pub dye_color_group_name: LocalizableString<'a>,
        pub icon_path: u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("dyecolorgroupinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(DyeColorGroupInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "dyecolorgroupinfo roundtrip bytes mismatch");
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
            items.push(DyeColorGroupInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            DyeColorGroupInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
