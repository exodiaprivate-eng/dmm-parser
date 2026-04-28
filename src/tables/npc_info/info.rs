//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410EBEB0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. u32 lookup_18 (read_u32_lookup_DA30)
//!   5. u16 lookup_20 (sub_141103610 → qword_145F15038)
//!   6. u32 lookup_22 (sub_1410FF5C0 → qword_145F0DA00)
//!   7. u32 u32_24
//!   8. u32 u32_28
//!   9. u32 u32_32
//!  10. u16 u16_36
//!  11. LocalizableString name
//!  12. LocalizableString desc
//!  13. LocalizableString extra
//!  14. sub_14110E340 → struct +136 (unknown helper) ← TAIL STARTS HERE
//!  15. (After tail) inline CArray<{u16+u32}=6B> at struct +152
//!
//! Steps 1-13 are typed. The unknown helper sub_14110E340 wraps the
//! NpcInfo body proper; reopens cleanly when decoded.
//!
//! New helper mapped: `sub_141103610` = single u16 lookup at
//! qword_145F15038 (wire 2).

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct NpcInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub lookup_18: u32,
        pub lookup_20: u16,
        pub lookup_22: u32,
        pub u32_24: u32,
        pub u32_28: u32,
        pub u32_32: u32,
        pub u16_36: u16,
        pub name: LocalizableString<'a>,
        pub desc: LocalizableString<'a>,
        pub extra: LocalizableString<'a>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\npcinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\npcinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                NpcInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "npcinfo roundtrip mismatch");
    }
}
