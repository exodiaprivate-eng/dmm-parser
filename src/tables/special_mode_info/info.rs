//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410F60E0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. u8 byte_17
//!   5. u32 lookup_18 (sub_1410FF430 → qword_145F0E9C0)
//!   6. CString second_string
//!   7. u32 u32_32
//!   8. u32 u32_36
//!   9. u32 u32_40
//!  10. u32 lookup_44 (sub_1410FEBE0 → qword_145F0DA68)
//!  11. u32 u32_48
//!  12. u32 u32_52
//!  13. u32 u32_56
//!  14-21. 8× u8 (u8_60..u8_67)
//!  22. 24× sub_141128AF0(struct +72+16*i) (CArray-like 24-iter loop
//!      reading 16-byte items via unknown helper) ← TAIL STARTS HERE
//!  23. sub_1410F5F80(struct +456) (unknown)
//!  24. sub_1410D4540(struct +520) (unknown)
//!
//! Steps 1-21 are typed; step 22 onward lives in `tail_blob`. Reopens
//! cleanly when the 16-byte item helper sub_141128AF0 is decoded.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct SpecialModeInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub byte_17: u8,
        pub lookup_18: u32,
        pub second_string: CString<'a>,
        pub u32_32: u32,
        pub u32_36: u32,
        pub u32_40: u32,
        pub lookup_44: u32,
        pub u32_48: u32,
        pub u32_52: u32,
        pub u32_56: u32,
        pub u8_60: u8,
        pub u8_61: u8,
        pub u8_62: u8,
        pub u8_63: u8,
        pub u8_64: u8,
        pub u8_65: u8,
        pub u8_66: u8,
        pub u8_67: u8,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\specialmode.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\specialmode.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                SpecialModeInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "specialmode roundtrip mismatch");
    }
}
