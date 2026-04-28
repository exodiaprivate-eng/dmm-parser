//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410DC8F0` in CrimsonDesert.exe (Win build).
//! Wire reads, in order:
//!   1. u32 key (via `sub_141BF6840` writer wrapper)
//!   2. CString string_key
//!   3. u8 byte_16
//!   4. u8 byte_17
//!   5. CString second_string
//!   6. u32 u32_32, u32_36, u32_40, u32_44, u32_48, u32_52
//!   7. u8 u8_56
//!   8. sub_1411166F0 (CArray of 24-byte elements via sub_1411168A0
//!      sub-helper) ← TAIL STARTS HERE
//!   9. 2× sub_141102B30 (unknown helpers)
//!  10. inline CArray (u32 count + N×(u32+u32) at struct +112)
//!  11. 8-iteration loop reading u32 each (struct +128..+156)
//!  12. u8 (struct +160)
//!  13. CArray<sub_1410DC7F0> at struct +168 (16-byte elements)
//!  14. u8 (struct +184)
//!
//! Steps 1-7 are typed; everything from step 8 lives in `tail_blob`.
//! `sub_1411166F0` exceeds the 3-IDA-call budget (nested
//! `sub_1411168A0`); reopens cleanly when the array helper family is
//! decoded.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct ElementalMaterialInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub byte_16: u8,
        pub byte_17: u8,
        pub second_string: CString<'a>,
        pub u32_32: u32,
        pub u32_36: u32,
        pub u32_40: u32,
        pub u32_44: u32,
        pub u32_48: u32,
        pub u32_52: u32,
        pub u8_56: u8,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\elementalmaterialinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\elementalmaterialinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                ElementalMaterialInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "elementalmaterialinfo roundtrip mismatch");
    }
}
