//! IDA-derived parser for `LevelActionPointInfo.pabgb`.
//!
//! Field layout extracted from Hex-Rays decompile of the parse function
//! in the current Win exe (CrimsonDesert.exe). Field NAMES paired with
//! Mac binary __cstring declaration order. Round-trip-validated against
//! the vanilla pabgb dump from the live game install.
//!
//! DO NOT EDIT BY HAND - regenerate via tools/ida_extract.py.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::LevelActionPointInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  5
// Decoded by dmm-parser:   5
// Missing in this struct:  0
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _isBlocked (direct_u8, stream=1)
// ✅ _stringKey
// ✅ _levelActionPointGroupList
// ✅ _accessType (direct_u8, stream=1)
// ✅ _key

use crate::binary::*;
use crate::py_binary_struct;

// Hand-corrected: sub_14110E680 is CArray<{u32 + CArray<LevelActionPoint>}>
// where inner sub_14110E890 reads 12-byte points. Catalog suggests Vec3 of
// f32, but vanilla sweep finds 17 NaN bit patterns across 1803 f32 values
// (denormals + huge magnitudes — values aren't physical coordinates).
// Exposed as 3× u32 raw bit fields so JSON consumers can edit each axis
// independently while preserving the exact NaN-bearing bit pattern.
py_binary_struct! {
    pub struct LevelActionPoint {
        pub x_bits: u32,
        pub y_bits: u32,
        pub z_bits: u32,
    }
}

py_binary_struct! {
    pub struct LevelActionPointGroupElem {
        pub group_key: u32,
        pub points: CArray<LevelActionPoint>,
    }
}

py_binary_struct! {
    pub struct LevelActionPointInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub access_type: u8,
        pub level_action_point_group_list: CArray<LevelActionPointGroupElem>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("levelactionpointinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(LevelActionPointInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "levelactionpointinfo roundtrip bytes mismatch");
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
            items.push(LevelActionPointInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            LevelActionPointInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
