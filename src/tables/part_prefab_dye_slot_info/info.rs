//! Hand-corrected: IDA-derived parser for `PartPrefabDyeSlotInfo.pabgb`.
//!
//! Per IDA sub_1410EF0B0 (outer): u32 key, CString string_key, u8 is_blocked,
//! CArray<DyeSlotEntry> sub_mesh_list, CString mesh_file_name.
//!
//! Per IDA sub_14110C970 (CArray reader) + sub_1410EEE40 (element reader):
//! DyeSlotEntry = CString + 3 u8 + 3 sub_1410A9D40-CString + 3 u8.
//! sub_1410A9D40 reads a length-prefixed CString from disk and resolves
//! it to a u32 hash in memory, but on disk it remains a CString — we
//! preserve the CString form for round-trip.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::PartPrefabDyeSlotInfo
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
// ✅ _stringKey
// ✅ _key
// ✅ _subMeshList
// ✅ _isBlocked (direct_u8, stream=1)
// ✅ _meshFileName

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    /// PartPrefabDyeTextureSet — 3 texture-path CStrings + 4 flag bytes. The game
    /// reads this same layout for both a sub-mesh's `_default` set AND every element
    /// of its `_modelPropertyDyeInfoList` (IDA readers sub_14117A1D0 / sub_1411937A0,
    /// each = 3× CString + 3× u8 + 1× u8).
    pub struct DyeTextureSet<'a> {
        pub texture_a: CString<'a>,
        pub texture_b: CString<'a>,
        pub texture_c: CString<'a>,
        pub flag_d: u8,
        pub flag_e: u8,
        pub flag_f: u8,
        // ── 2.01.00: `_useDyeGrime` grew from 3 bytes to 12 (Mac reader sub_102064620 reads
        // twelve u8 in one chain). flag_d/e/f are the first three, kept under their old
        // names because Dye Fix and friends address them by name; the nine new ones follow.
        pub use_dye_grime_3: u8, pub use_dye_grime_4: u8, pub use_dye_grime_5: u8,
        pub use_dye_grime_6: u8, pub use_dye_grime_7: u8, pub use_dye_grime_8: u8,
        pub use_dye_grime_9: u8, pub use_dye_grime_10: u8, pub use_dye_grime_11: u8,
        pub flag_g: u8,   // `_modelPropertyIndex`
    }
}

py_binary_struct! {
    // PartSubMeshDyeData (IDA reader sub_141193270): subMeshName + 3 bytes +
    // _default(DyeTextureSet) + _modelPropertyDyeInfoList(CArray<DyeTextureSet>).
    pub struct DyeSlotEntry<'a> {
        pub name: CString<'a>,
        pub flag_a: u8,
        pub flag_b: u8,
        pub flag_c: u8,
        // _default PartPrefabDyeTextureSet, inlined (3 CString + 4 u8).
        pub texture_a: CString<'a>,
        pub texture_b: CString<'a>,
        pub texture_c: CString<'a>,
        pub flag_d: u8,
        pub flag_e: u8,
        pub flag_f: u8,
        // ── 2.01.00: `_useDyeGrime` grew from 3 bytes to 12 (Mac reader sub_102064620 reads
        // twelve u8 in one chain). flag_d/e/f are the first three, kept under their old
        // names because Dye Fix and friends address them by name; the nine new ones follow.
        pub use_dye_grime_3: u8, pub use_dye_grime_4: u8, pub use_dye_grime_5: u8,
        pub use_dye_grime_6: u8, pub use_dye_grime_7: u8, pub use_dye_grime_8: u8,
        pub use_dye_grime_9: u8, pub use_dye_grime_10: u8, pub use_dye_grime_11: u8,
        pub flag_g_112: u8,   // `_modelPropertyIndex`
        // 1.13.00 FIX: this was mis-modeled as a plain u32 (`unk_u32_112`). It is
        // actually the `_modelPropertyDyeInfoList` — a CArray whose elements share
        // the _default's 3-CString + 4-u8 layout. Vanilla 1.12.2 records all had
        // count 0, so the old u32 read matched by luck; 1.13.00's dyeable
        // weapons/disguises populate it (>0) → the old parser desynced (record
        // key=1414745672 had "cloth"+"leather" entries). Now decoded typed.
        pub model_property_list: CArray<DyeTextureSet<'a>>,
    }
}

py_binary_struct! {
    pub struct PartPrefabDyeSlotInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub sub_mesh_list: CArray<DyeSlotEntry<'a>>,
        pub mesh_file_name: CString<'a>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("partprefabdyeslotinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(PartPrefabDyeSlotInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "partprefabdyeslotinfo roundtrip bytes mismatch");
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
            items.push(PartPrefabDyeSlotInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            PartPrefabDyeSlotInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
