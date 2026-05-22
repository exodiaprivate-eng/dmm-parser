//! Tier 1 — fully typed (no _tail_b64).
//!
//! Reader (Tier IDA verified 2026-05-20 vs CrimsonDesert.exe md5
//! 3d614280…) — 100% field-level, top to bottom. Table reader is
//! `sub_1410BC9F0` (the cited `sub_1410DB040` was STALE → lands inside
//! the element reader `sub_1410DAE40`). Full reader tree:
//!
//!   EquipInfo            sub_1410BC9F0  — 7 fields:
//!     a2+0   u32 key
//!     a2+8   CString string_key                 (sub_14108B300)
//!     a2+16  u8 is_blocked
//!     a2+18  u16 attacked_material_slot_no
//!     a2+24  CArray<EquipListItem> list         (sub_1410FA190)
//!     a2+40  CArray<Ragdoll…> ragdoll_list       (sub_1410F9F80)
//!     a2+56  u32 ui_component_name              (sub_1410E1350,
//!            4 wire bytes → u16 RAM)
//!
//!   EquipListItem        sub_1410BC660  — 20 fields, all confirmed:
//!     list_u32 CArray<u32> (sub_1410E4B20: u32 wire→u16 RAM elems),
//!     raw_a u32, raw_b u32, lookup_a (sub_1410E1350 u32→u16),
//!     raw_c u16, raw_d u64, lookup_b (sub_1410E1350), raw_e..h u32×4,
//!     label LocalizableString (sub_140F48220), flag_a/b u8,
//!     lookup_c (sub_1410E4C30 u32→u16), flag_c..g u8×5.
//!
//!   RagdollEquipTableGroupData  sub_1410F9F80 elem (24 mem bytes):
//!     raw_a u32 + triples CArray (sub_1410FA320).
//!   RagdollGroupTriple   sub_1410FA320  — 12 wire bytes:
//!     lookup (sub_1410E19E0 u32→u16) + raw_a u32 + raw_b u32.
//!
//! ⚠ **No on-disk pabgb dump in ANY version — runtime/conditional
//! table, never serialized to disk.** Confirmed: `equipinfo.pabgb` is
//! absent from both the 1.0.4 dump and the current 1.07 dump (only
//! `equipslotinfo`/`equiptypeinfo` ship). Byte-roundtrip is therefore
//! N/A by nature (the test SKIPs); the IDA reader-tree trace above IS
//! the complete field-level validation — the strongest claim possible
//! for a table with no on-disk form. ID-ref fields read wider on the
//! wire than RAM, as elsewhere; Rust models the wire.
//! (Original Mac note: `sub_10186D874`.)
//!
//! Wire reads, in order:
//!   1. u32 key                          (a2+0)
//!   2. CString string_key               (a2+8)
//!   3. u8 is_blocked                    (a2+16)
//!   4. u16 attacked_material_slot_no    (a2+18)
//!   5. CArray<EquipListItem> list       (sub_141117600 → sub_1410DACB0;
//!      112 mem bytes / 20 wire fields per element)
//!   6. CArray<RagdollEquipTableGroupData> ragdoll_list
//!      (sub_1411173F0 → sub_141117790 inner per element; 24 mem bytes
//!      with inline CArray of 12-wire-byte triples)
//!   7. u32 ui_component_name            (read_u32_lookup_DA30 wire u32)

use crate::binary::*;
use crate::py_binary_struct;

// sub_141117790 inner — 12-wire-byte triple per element.
py_binary_struct! {
    pub struct RagdollGroupTriple {
        pub lookup: u32,    // sub_1410FF430 wire u32
        pub raw_a: u32,
        pub raw_b: u32,
    }
}

// sub_1411173F0 inner — 24 mem bytes per element. Wire: u32 raw + CArray
// of 3-element groups.
py_binary_struct! {
    pub struct RagdollEquipTableGroupData {
        pub raw_a: u32,
        pub triples: CArray<RagdollGroupTriple>,
    }
}

// sub_1410DACB0 inner — 112 mem bytes / 20 wire fields per element.
py_binary_struct! {
    pub struct EquipListItem<'a> {
        pub list_u32: CArray<u32>,           // sub_141102570 wire u32 / u16 mem
        pub raw_a: u32,
        pub raw_b: u32,
        pub lookup_a: u32,                   // read_u32_lookup_DA30 wire u32
        pub raw_c: u16,
        pub raw_d: u64,
        pub lookup_b: u32,                   // read_u32_lookup_DA30 wire u32
        pub raw_e: u32,
        pub raw_f: u32,
        pub raw_g: u32,
        pub raw_h: u32,
        pub label: LocalizableString<'a>,
        pub flag_a: u8,
        pub flag_b: u8,
        pub lookup_c: u32,                   // sub_141102680 wire u32 / u16 mem
        pub flag_c: u8,
        pub flag_d: u8,
        pub flag_e: u8,
        pub flag_f: u8,
        pub flag_g: u8,
    }
}

py_binary_struct! {
    pub struct EquipInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub attacked_material_slot_no: u16,
        pub list: CArray<EquipListItem<'a>>,
        pub ragdoll_list: CArray<RagdollEquipTableGroupData>,
        pub ui_component_name: u32,
    }
}

impl<'a> EquipInfo<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> std::io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        let consumed = *offset - start;
        if consumed != entry_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("EquipInfo: consumed {} bytes, expected {}", consumed, entry_size),
            ));
        }
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    // No on-disk pabgb dump for this table; test SKIPs.
    const PABGB: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/equipinfo.pabgb";
    const PABGH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/equipinfo.pabgh";
    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                EquipInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "equipinfo roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
        let Ok(data) = std::fs::read(PABGB) else {
            eprintln!("SKIP: missing fixture {}", PABGB);
            return;
        };
        let Some(entries) = load_pabgh_offsets(PABGH) else {
            eprintln!("SKIP: missing pabgh fixture {}", PABGH);
            return;
        };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = EquipInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            EquipInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
