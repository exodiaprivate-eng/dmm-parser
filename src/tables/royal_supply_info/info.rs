//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410F64D0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u16 key (pabgh format 2)
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. sub_14110A270 → struct +24 (CArray composite with sub_14110A0E0
//!      + sub_140562FD0; sub_14110A0E0 itself is a CArray of items
//!      using 4 sub-helpers including sub_141102CB0/sub_141102D90)
//!      ← TAIL STARTS HERE
//!   5. sub_14110A270 → struct +56 (same composite)
//!   6. sub_14110A0E0 → struct +88 (CArray of 4-helper sub-items)
//!   7. sub_141102D20 → struct +104 (single u32 lookup at qword_145F0EF38)
//!
//! Steps 1-3 are typed; everything from step 4 lives in `tail_blob`.
//! `sub_14110A270` is a complex composite reader using helpers we
//! haven't traced (sub_140562FD0, sub_141102CB0, sub_141102D90); it
//! exceeds the 3-call budget. Reopens cleanly when those are decoded.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct RoyalSupplyInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\royalsupply.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\royalsupply.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                RoyalSupplyInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "royalsupply roundtrip mismatch");
    }
}
