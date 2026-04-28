//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410D6420` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. CArray<BoardSubItem> sub_list (sub_141118F10: each item is u32
//!      raw + u32 lookup sub_1410FF5C0 + u32 lookup sub_1410FF430,
//!      total 12 wire bytes per element)
//!   5. sub_141118D60 → struct +40 (CArray of 72-byte items via
//!      sub_1410D62F0 + helpers; nested structure exceeds budget)
//!      ← TAIL STARTS HERE
//!
//! Steps 1-4 are typed; step 5 lives in `tail_blob`.

use crate::binary::*;
use crate::pabgh_typed_blob_table;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct BoardSubItem {
        pub u32_a: u32,
        pub lookup_b: u32,
        pub lookup_c: u32,
    }
}

pabgh_typed_blob_table! {
    pub struct BoardInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub sub_list: CArray<BoardSubItem>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\board.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\board.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                BoardInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "board roundtrip mismatch");
    }
}
