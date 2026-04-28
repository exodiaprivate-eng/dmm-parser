//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410E05E0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u16 key (pabgh format 2)
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. CArray<InventorySubA> sub_a_list (sub_141103FB0: each item =
//!      u16 lookup via sub_141100620 at qword_145F0DA20 + u8 = 3 wire
//!      bytes)
//!   5. CArray<InventorySubA> sub_b_list (same shape)
//!   6. sub_141114720 → struct +56 (CArray of 160-byte items via
//!      sub_1410E0460 + many sub-helpers; exceeds budget)
//!      ← TAIL STARTS HERE
//!   7. (After tail) u16, u16, 2× LocalizableString, u32 lookup,
//!      u8, u32, u32, u8, u8, u8, sub_141103310 CArray<u32+u64>

use crate::binary::*;
use crate::pabgh_typed_blob_table;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct InventorySubA {
        pub lookup: u16,
        pub byte_2: u8,
    }
}

pabgh_typed_blob_table! {
    pub struct InventoryInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub sub_a_list: CArray<InventorySubA>,
        pub sub_b_list: CArray<InventorySubA>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\inventory.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\inventory.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                InventoryInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "inventory roundtrip mismatch");
    }
}
