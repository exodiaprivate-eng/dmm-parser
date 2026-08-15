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
        // 1.13.00: new u16 inserted between phase_id and region_hash (+2 per
        // element). Byte-diff decisive: region_hash (a stable hash) is unchanged
        // and shifts +2 in each list element; the 2 bytes before it grew to 4.
        pub phase_extra_113: u16,
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
        // 1.12: trailing f32 (observed constant 4.0 = `00 00 80 40`) appended
        // to each HouseInfo record after the region list. Byte-diff decisive:
        // +4B once per record at the prior record-end offset (146/292/438).
        // ⓘ The 1.18 field-name oracle names this one `_floorHeight` (it is the
        // 7th and last pre-1.18 field, and 4.0 is a plausible floor height).
        // NOT renamed: parser field names are the mod contract, and a rename
        // would silently drop `unk_f32_112` out of any mod already keyed on it.
        pub unk_f32_112: f32,
        // ── 1.18.00: three fields appended after the float, +9 bytes total,
        // in this wire order: u8, u32, u32.
        // The last one holds 9 / 15 / 24 in the eight new records and
        // 0xFFFFFFFF in the four pre-existing ones — an index with an unset
        // sentinel, which is what `…SceneObjectInfo` should look like.
        // ⚠ The first two are 0 in every record, so the bytes cannot say which
        // of the two names takes the u8. Assigned by convention: `…Type` is a
        // u8 enum, `…Flag` a u32 bitfield. Round-trips either way.
        pub placement_system_type: u8,
        pub usable_placement_type_flag: u32,
        pub housing_pivot_level_gimmick_scene_object_info: u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("houseinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
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
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
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
