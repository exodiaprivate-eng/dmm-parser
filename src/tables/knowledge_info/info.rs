//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410E36C0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. u32 lookup_18 (read_u32_lookup_DA30)
//!   5. u8 byte_20
//!   6. u8 byte_21
//!   7. u32 lookup_22 (sub_141100860 → qword_145F0DA48)
//!   8. u32 lookup_24 (sub_141101D50 → qword_145F0EEE8)
//!   9. u32 lookup_26 (sub_1410FEBE0 → qword_145F0DA68)
//!  10. CArray<u32> list_32 (sub_1410FF890 → qword_145F0DA08)
//!  11. CArray<u32> list_48 (sub_141104540 → qword_145F0DA38)
//!  12. CArray<u16> list_64 (sub_1410FFAC0 → qword_145F0DA80)
//!  13. sub_141101610 → struct +80 (unknown helper) ← TAIL STARTS HERE
//!  14. (After tail) 3× u8, read_u32_lookup_DA30, inline CArray of
//!      9-byte items, sub_141104650, sub_141113F00, sub_141104760,
//!      sub_1411006D0, sub_141104760, sub_141113D80, sub_1410FF5C0,
//!      u8, [u8;12], sub_141102D20, sub_1410FEBE0.
//!
//! Steps 1-12 are typed. The knowledge body has many more fields
//! interleaved with unknowns; reopens cleanly.
//!
//! New helpers: `sub_141100860` = u32 lookup at qword_145F0DA48;
//! `sub_1410FF890` = CArray<u32> at qword_145F0DA08;
//! `sub_141104540` = CArray<u32> at qword_145F0DA38.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct KnowledgeInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub lookup_18: u32,
        pub byte_20: u8,
        pub byte_21: u8,
        pub lookup_22: u32,
        pub lookup_24: u32,
        pub lookup_26: u32,
        pub list_32: CArray<u32>,
        pub list_48: CArray<u32>,
        pub list_64: CArray<u16>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\knowledgeinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\knowledgeinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                KnowledgeInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "knowledgeinfo roundtrip mismatch");
    }
}
