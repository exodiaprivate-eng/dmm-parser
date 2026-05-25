//! Tier 1 — fully typed (no _tail_b64).
//!
//! Reader (Tier IDA verified 2026-05-19 vs CrimsonDesert.exe md5
//! 3d614280…): `sub_1410BE2C0` — ElementalMaterialInfo deserializer
//! (via "ElementalMaterialInfo" class block at 0x144AEC6D0+). (Cited
//! `sub_1410DC8F0` stale.) The struct definition below is correct and
//! fully matches the reader; the OLD prose field-list further down in
//! this header was imprecise (it omitted `unk_new_u32_a/_b` and
//! mislabeled the fuel block) — trust the struct, not that list.
//! Verified mem map:
//!   a2+0  u32 key (hashed sub_141BF0180)   a2+8  CString string_key
//!   a2+16 u8 is_blocked                    a2+17 u8 system_type
//!   a2+24 CString elemental_material_key
//!   a2+32 u32 unk_new_u32_a                a2+36 u32 unk_new_u32_b
//!   a2+40 u32 total_fuel_amount + a2+44 u32 fuel_standard_obb_size
//!         (engine batches these two as one 8-byte read)
//!   a2+48/52/56/60  4× u32 fuel_end_passive/active key+level
//!   a2+64 u8 use_temperature_transfer_margin
//!   a2+72  CArray state_data_list   a2+88/104 CArray min/max_stat_list
//!   a2+120 CArray parent_material_key_list (elem = u32 raw + u32 hash)
//!   a2+136..164  flag_0..flag_7 (8× u32, via `for j in 0..8` loop —
//!         confirms the 1-to-8 `_flag` wrapper)
//!   a2+168 u8 is_system_type   a2+176 CArray stat_data_list
//!   a2+192 u8 scene_object_spawnable_type
//! All ID-ref fields read wire-width per the struct; roundtrip + this
//! map agree.
//!
//! ─── v3.1 closure analysis (iter 76) ────────────────────────────────────
//! Cross-check via `sub_1410A8FA0` (typeinfo→record-reader path per the
//! iter 58 typeinfo registry). The crucial wire pattern:
//!
//!   for j in 0..8 {
//!       read 4 bytes at a2 + 4*(j+34)   → offsets 136, 140, …, 164
//!   }
//!
//! Eight consecutive u32 reads at offsets 136..164 (8 × u32 = 32 bytes).
//! This pairs exactly with the eight already-unrolled rust fields:
//!
//!   flag_0, flag_1, flag_2, flag_3, flag_4, flag_5, flag_6, flag_7  (u32)
//!
//! NattKh schema lists `_flag` as a single canonical at this position.
//! Wire-level confirmation: **`_flag` is a pure 1-to-8 wrapper** around
//! the eight `flag_*` rust fields. Closure path: 1-to-N alias entry.
//! No new decoder work needed.
//! Inner readers (decoded for full Tier 1):
//!   - sub_1411166F0: outer CArray<ElementalMaterialStateData>
//!     (24 mem bytes per outer = u32 lookup + nested CArray)
//!   - sub_1411168A0: nested CArray<ElementalMaterialStateInner>
//!     (152 mem bytes per element via sub_1410DC480)
//!   - sub_1410DC480: 19 wire fields including 5× CString-hash +
//!     2× ElementalMaterialStateInfo (sub_1410DC310, 9 fields)
//!   - sub_1410DC310: 9-field flat struct (mix of u32 lookup +
//!     u32 raw)
//!   - sub_141102B30: CArray<{u32 + u64}> = 12 wire bytes/elem
//!   - sub_1410DC7F0: 4-field struct (u32 + u32-hash + u32 + u8)
//!     = 13 wire bytes/elem
//!
//! Wire reads, in order (canonical names from
//! `docs/449_TABLE_CATALOG.md` ElementalMaterialInfo section, 20 fields):
//!   1. u32 key                                   (_key, hashed via
//!      sub_141BF6840 → on-disk wire is u32)
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. u8 elemental_material_system_type
//!   5. CString elemental_material_key
//!   6. u32 total_fuel_amount                     (raw)
//!   7. u32 fuel_standard_obb_size                (raw)
//!   8. u32 fuel_end_passive_skill_key            (raw, hashed mem)
//!   9. u32 fuel_end_passive_skill_level          (raw)
//!  10. u32 fuel_end_active_skill_key             (raw, hashed mem)
//!  11. u32 fuel_end_active_skill_level           (raw)
//!  12. u8 use_temperature_transfer_margin
//!  13. CArray<ElementalMaterialStateData> elemental_material_state_data_list
//!  14. CArray<ElementalMaterialU32U64Pair> min_stat_list
//!  15. CArray<ElementalMaterialU32U64Pair> max_stat_list
//!  16. CArray<ElementalMaterialParentEntry> parent_material_key_list_deprecated_xxx
//!  17. [u32; 8] flag                              (8 raw u32s inline)
//!  18. u8 is_system_type
//!  19. CArray<ElementalMaterialEntry> elemental_material_stat_data_list
//!  20. u8 scene_object_spawnable_type

use crate::binary::*;
use crate::py_binary_struct;

// sub_1410DC310 — 36 mem bytes / 9 wire fields.
py_binary_struct! {
    pub struct ElementalMaterialStateInfo {
        pub lookup_a: u32,    // sub_1410FEBE0
        pub raw_a: u32,
        pub lookup_b: u32,    // sub_1410FEBE0
        pub raw_b: u32,
        pub lookup_c: u32,    // sub_1410FEBE0
        pub raw_c: u32,
        pub lookup_d: u32,    // sub_1410FEBE0
        pub raw_d: u32,
        pub raw_e: u32,
    }
}

// sub_1410DC480 — 152 mem bytes / 19 wire fields.
py_binary_struct! {
    pub struct ElementalMaterialStateInner<'a> {
        pub lookup_a: u32,    // read_u32_lookup_DA10
        pub raw_a: u64,
        pub raw_b: u64,
        pub icon: u32,        // read_u32_lookup_DA30
        pub name: CString<'a>,
        pub flag_a: u8,
        pub tag_a: CString<'a>,   // sub_1410A9D40
        pub tag_b: CString<'a>,   // sub_1410A9D40
        pub tag_c: CString<'a>,   // sub_1410A9D40
        pub tag_d: CString<'a>,   // sub_1410A9D40
        pub tag_e: CString<'a>,   // sub_1410A9D40
        pub state_a: ElementalMaterialStateInfo,
        pub state_b: ElementalMaterialStateInfo,
        pub raw_c: u32,
        pub raw_d: u32,
        pub flag_b: u8,
        pub flag_c: u8,
        pub flag_d: u8,
        pub flag_e: u8,
    }
}

// sub_1411166F0 outer element — u32 lookup + nested CArray.
py_binary_struct! {
    pub struct ElementalMaterialStateData<'a> {
        pub state_lookup: u32,
        pub data_list: CArray<ElementalMaterialStateInner<'a>>,
    }
}

// sub_141102B30 inner — 12 wire bytes (u32 + u64).
py_binary_struct! {
    pub struct ElementalMaterialU32U64Pair {
        pub a: u32,
        pub raw: u64,
    }
}

// Inline CArray entry at a2+112 — u32 raw + u32 (hashed via sub_141BF6840).
py_binary_struct! {
    pub struct ElementalMaterialParentEntry {
        pub raw: u32,
        pub key_hash: u32,
    }
}

// sub_1410DC7F0 inner — 13 wire bytes (u32 + u32-hash + u32 + u8).
py_binary_struct! {
    pub struct ElementalMaterialEntry {
        pub raw_a: u32,
        pub key_hash: u32,
        pub raw_b: u32,
        pub flag: u8,
    }
}

py_binary_struct! {
    pub struct ElementalMaterialInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub elemental_material_system_type: u8,
        pub elemental_material_key: CString<'a>,
        pub unk_new_u32_a: u32,
        pub unk_new_u32_b: u32,
        pub total_fuel_amount: u32,
        pub fuel_standard_obb_size: u32,
        pub fuel_end_passive_skill_key: u32,
        pub fuel_end_passive_skill_level: u32,
        pub fuel_end_active_skill_key: u32,
        pub fuel_end_active_skill_level: u32,
        pub use_temperature_transfer_margin: u8,
        pub elemental_material_state_data_list: CArray<ElementalMaterialStateData<'a>>,
        pub min_stat_list: CArray<ElementalMaterialU32U64Pair>,
        pub max_stat_list: CArray<ElementalMaterialU32U64Pair>,
        pub parent_material_key_list_deprecated_xxx: CArray<ElementalMaterialParentEntry>,
        pub flag_0: u32,
        pub flag_1: u32,
        pub flag_2: u32,
        pub flag_3: u32,
        pub flag_4: u32,
        pub flag_5: u32,
        pub flag_6: u32,
        pub flag_7: u32,
        pub is_system_type: u8,
        pub elemental_material_stat_data_list: CArray<ElementalMaterialEntry>,
        pub scene_object_spawnable_type: u8,
    }
}

impl<'a> ElementalMaterialInfo<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> std::io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        let consumed = *offset - start;
        if consumed != entry_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("ElementalMaterialInfo: consumed {} bytes, expected {}", consumed, entry_size),
            ));
        }
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("elementalmaterialinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            let item = ElementalMaterialInfo::read_from(&data, &mut c)
                .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er));
            assert_eq!(c, *e, "e{} k=0x{:x}: under/over-read {}/{}", i, k, c - s, e - s);
            items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "elementalmaterialinfo roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = ElementalMaterialInfo::read_from(&data, &mut cursor).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            ElementalMaterialInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
