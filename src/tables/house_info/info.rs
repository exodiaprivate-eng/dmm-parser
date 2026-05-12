//! IDA-derived parser for `HouseInfo.pabgb`.
//!
//! Field layout extracted from Hex-Rays decompile of the parse function
//! in the current Win exe (CrimsonDesert.exe). Field NAMES paired with
//! Mac binary __cstring declaration order. Round-trip-validated against
//! the vanilla pabgb dump from the live game install.
//!
//! **T0-V verification (iter 2 of T0 verification loop, IDA Win 1.06):**
//! HouseInfo is NOT in NattKh's pabgb_complete_schema.json. IDA cross-
//! references the rust struct against the in-binary metaobject at
//! 0x144afbcd0+. **6/6 top-level fields verified canonical:**
//!
//! | rust field | canonical PA name | IDA address |
//! |---|---|---|
//! | `key` | `_key` | 0x144afc014 ✓ |
//! | `string_key` | `_stringKey` | 0x144afbe1c ✓ |
//! | `is_blocked` | `_isBlocked` | 0x144afbe5c ✓ |
//! | `house_name` | `_houseName` | 0x144afbe9c ✓ |
//! | `unlock_condition_info` | `_unlockConditionInfo` | 0x144afbedc ✓ |
//! | `house_region_data_list` | `_houseRegionDataList` | 0x144afbcdc ✓ |
//!
//! Renamed nested `HouseRegionPhase` → `HouseRegionData` (canonical
//! class name from IDA at 0x144afbf20+; "Phase" was a mechanical guess).
//! The 3 nested-struct fields (`phase_id`, `region_hash`, `texture_path`)
//! are not directly readable from the metaobject (pointer-table format);
//! their canonical names need decompile of the parser's nested-record
//! reader. Field semantics (positional decode) verified by the existing
//! roundtrip test against 4 vanilla entries.
//!
//! DO NOT EDIT BY HAND - regenerate via tools/ida_extract.py.

use crate::binary::*;
use crate::py_binary_struct;

// Renamed iter 2 of T0 verification loop: canonical class name is
// `HouseRegionData` (verified in IDA at 0x144afbf20+). Old name
// `HouseRegionPhase` was a mechanical guess from `_houseRegionDataList`
// container field name.
py_binary_struct! {
    pub struct HouseRegionData<'a> {
        pub phase_id: u16,
        pub region_hash: u32,
        pub texture_path: CString<'a>,
    }
}

py_binary_struct! {
    pub struct HouseInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub house_name: LocalizableString<'a>,
        pub unlock_condition_info: u32,
        pub house_region_data_list: CArray<HouseRegionData<'a>>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/houseinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(HouseInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "houseinfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(HouseInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            HouseInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
