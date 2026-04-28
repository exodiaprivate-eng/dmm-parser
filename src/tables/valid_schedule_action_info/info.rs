//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410FB840` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u32 key (via sub_141BF6600 wrapper)
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. CArray<u32> ref_list_a (sub_141101AB0: u32 count + N×u32)
//!   5. u8 byte_40
//!   6. CArray<CString> string_list (sub_14106BAC0: u32 count + N×CString)
//!   7. sub_1410FB680 (composite: 2× CArray<CString>, sub_1410FF9A0,
//!      sub_1410FFD30, sub_141102570, sub_1410FFC20, [u8;16],
//!      sub_141108AE0 — 4 unknown helpers exceed budget) ← TAIL STARTS HERE
//!
//! Steps 1-6 are typed; everything from step 7 lives in `tail_blob`.
//! Reopens cleanly when the unknown helper family is decoded.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct ValidScheduleActionInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub ref_list_a: CArray<u32>,
        pub byte_40: u8,
        pub string_list: CArray<CString<'a>>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\validscheduleaction.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\validscheduleaction.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                ValidScheduleActionInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "validscheduleaction roundtrip mismatch");
    }
}
