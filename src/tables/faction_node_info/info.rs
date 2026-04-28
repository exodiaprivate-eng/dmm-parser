//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410DE7A0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. u32 lookup_18 (sub_1411006D0 → qword_145F0DA28)
//!   5. u32 lookup_20 (sub_1411035A0 → qword_145F1A740)
//!   6. u32 lookup_22 (sub_141101D50 → qword_145F0EEE8)
//!   7. u16 lookup_24 (sub_141103610 → qword_145F15038)
//!   8. u16 lookup_26 (sub_1411036C0 → qword_145F113A0)
//!   9. CString second_string
//!  10. CArray<u32> list_a (sub_141102FF0 → qword_145F0EEE8)
//!  11. CArray<u32> list_b (sub_141102FF0)
//!  12. [u8; 12] raw_72
//!  13. u32 u32_84
//!  14. sub_141115FD0 → struct +88 (unknown helper) ← TAIL STARTS HERE
//!  15. (After tail) sub_141115D90, sub_141101610, sub_141115BA0,
//!      inline CArray<280-byte items> via sub_1410DDE60, lots more.
//!
//! Steps 1-13 are typed. The body has many more typed-shaped fields
//! interleaved with unknowns; reopens cleanly when the helper family
//! is decoded.
//!
//! New helpers: `sub_1411035A0` = u32 lookup at qword_145F1A740;
//! `sub_141101D50` = u32 lookup at qword_145F0EEE8; `sub_1411036C0` =
//! u16 lookup at qword_145F113A0; `sub_141102FF0` = CArray<u32>
//! hash-keyed at qword_145F0EEE8.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct FactionNodeInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub lookup_18: u32,
        pub lookup_20: u32,
        pub lookup_22: u32,
        pub lookup_24: u16,
        pub lookup_26: u16,
        pub second_string: CString<'a>,
        pub list_a: CArray<u32>,
        pub list_b: CArray<u32>,
        pub raw_72: [u8; 12],
        pub u32_84: u32,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\factionnode.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\factionnode.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                FactionNodeInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "factionnode roundtrip mismatch");
    }
}
