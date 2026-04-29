//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader (Mac CrimsonDesert_Steam): `sub_10186D874` at 0x10186D874.
//! No on-disk pabgb dump — table is runtime/conditional, so the
//! roundtrip test SKIPs. Typed prefix is documentation + tooling
//! support only.
//!
//! Wire reads, in order:
//!   1. u32 key                       (sub_100EFBC60, width 4)
//!   2. CString string_key            (sub_1006B3F50, struct +8)
//!   3. u8 is_blocked                 (sub_1006B3CC0, struct +16)
//!   4. u16 attacked_material_slot_no (sub_1006B3D20, struct +18)
//!      ← TAIL STARTS HERE
//!   5. (tail) _list                  (sub_10189F34C, struct +24,
//!      stride 16) — CArray, likely polymorphic equipment item
//!      descriptors
//!   6. (tail) _radgollEquipTableGroupDataList (sub_10189F718,
//!      struct +40, stride 16) — CArray
//!   7. (tail) _uiComponentName       (sub_100C93428, struct +56)
//!
//! Stop at field 4 because helpers sub_10189F34C/sub_10189F718 are
//! unknown CArray readers (not in the cheat-sheet) and likely
//! polymorphic.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct EquipInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub attacked_material_slot_no: u16,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    // No on-disk pabgb dump for this table; test SKIPs.
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\equipinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\equipinfo.pabgh";
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
