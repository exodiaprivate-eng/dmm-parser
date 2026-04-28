//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410D6120` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. u8 byte_17
//!   5. sub_14112DE30 → struct +24 (variable-length flagged
//!      sub-reader: 4+4+1+1+1 fixed bytes plus 0/1/2/3 nested
//!      32-byte allocations via sub_140F5B8B0; wire size depends on
//!      runtime flags) ← TAIL STARTS HERE
//!   6. (After tail) 8 bytes ×3, 2× u32, u8 trailing
//!
//! Steps 1-4 are typed. Step 5 has variable wire length so the rest
//! of the entry must live in the tail blob.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct BitmapPositionInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub byte_17: u8,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\bitmapposition.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\bitmapposition.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                BitmapPositionInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "bitmapposition roundtrip mismatch");
    }
}
