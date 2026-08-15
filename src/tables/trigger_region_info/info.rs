//! Hand-corrected: IDA-derived parser for `TriggerRegionInfo.pabgb`.
//!
//! Per IDA sub_14057C900 (outer): u32 key, CString string_key, u8 is_blocked,
//! CArray<PresetEntry> preset_list (sub_1411084A0).
//!
//! Per IDA sub_141429BE0 (PresetEntry reader): each element on disk is
//! 1 CString (sub_1410A9D40) + 7 u32 + 8 (u32-hash + u32) pairs + 2 u32
//! + 1 u32 tail + 2 u8 trailing bytes (in disk order: u32, byte_a, byte_b).
//! Memory stride = 128 bytes (sub_1411084A0 << 7), but disk format is
//! length-prefixed CString followed by all u32/u8 fields in read order.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::TriggerRegionInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  4
// Decoded by dmm-parser:   4
// Missing in this struct:  0
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _key
// ✅ _isBlocked (direct_u8, stream=1)
// ✅ _stringKey
// ✅ _presetList

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct PresetEntry<'a> {
        pub key_name: CString<'a>,
        pub field_b: u32,
        pub field_c: u32,
        pub field_d: u32,
        pub field_e: u32,
        pub field_f: u32,
        pub field_g: u32,
        pub field_h: u32,
        pub hash_a1: u32,
        pub val_a1: u32,
        pub hash_a2: u32,
        pub val_a2: u32,
        pub hash_a3: u32,
        pub val_a3: u32,
        pub hash_a4: u32,
        pub val_a4: u32,
        pub hash_b1: u32,
        pub val_b1: u32,
        pub hash_b2: u32,
        pub val_b2: u32,
        pub hash_b3: u32,
        pub val_b3: u32,
        pub hash_b4: u32,
        pub val_b4: u32,
        pub hash_c1: u32,
        pub val_c1: u32,
        pub hash_c2: u32,
        pub val_c2: u32,
        pub field_x: u32,
        pub field_y: u32,
        pub tail_u32: u32,
        pub tail_byte_a: u8,
        pub tail_byte_b: u8,
    }
}

py_binary_struct! {
    pub struct TriggerRegionInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        // ── 1.18.00: `_physicsMaterialKey`, one u32 between is_blocked and the
        // preset list. Unambiguous: the inserted bytes sit between a CString's
        // terminator region and the preset count, and one record carries a real
        // value (0x6963DF0E) while the other twelve hold the 0xFFFFFFFF unset
        // sentinel — so it is a key, not padding.
        pub physics_material_key: u32,
        pub preset_list: CArray<PresetEntry<'a>>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("triggerregioninfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(TriggerRegionInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "triggerregioninfo roundtrip bytes mismatch");
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
            items.push(TriggerRegionInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let _ = &item;
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            TriggerRegionInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
