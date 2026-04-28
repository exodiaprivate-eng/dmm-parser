//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410EDA00` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. u16 lookup_26 (sub_141105A10 → qword_145F15028)
//!   5. u8 byte_28
//!   6. sub_141114A80 → struct +32 (unknown helper)
//!      ← TAIL STARTS HERE
//!   7. (After tail) sub_1411006D0, sub_1410A9D40, 6× u8,
//!      sub_141103420, sub_14110D970, read_u32_lookup_DA10,
//!      sub_14106BAC0 (CArray<CString>), 2× LocalizableString,
//!      3× sub_1410FF050, LocalizableString, 2× sub_141100510.
//!
//! Steps 1-5 are typed. The body has many more fields with unknown
//! helpers; reopens cleanly when decoded.
//!
//! New helper: `sub_141105A10` = u16 lookup at qword_145F15028.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct MultiChangeInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub lookup_26: u16,
        pub byte_28: u8,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\multichangeinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\multichangeinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                MultiChangeInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "multichangeinfo roundtrip mismatch");
    }
}
