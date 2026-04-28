//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410FCD20` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u16 key (pabgh format 2)
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. u32 lookup_18 (sub_1410FF5C0 → qword_145F0DA00)
//!   5. CArray<u32> sub_a_list (sub_1410FFF10 → qword_145F0DA00)
//!   6. [u8; 8] raw_40
//!   7. u8 u8_48
//!   8. CArray<u64> u64_list at struct +56 (inline u32 count +
//!      N×u64)
//!   9. u32 lookup_72 (sub_1410FF430 → qword_145F0E9C0)
//!  10. u32 u32_76
//!  11. u32 u32_80
//!  12. u32 u32_84
//!  13. u32 u32_88
//!  14. u8 u8_92
//!  15. inline CArray of 88-byte items via sub_1410FC8F0 → struct +96
//!      ← TAIL STARTS HERE
//!  16. sub_1411002A0 (16-byte slot, twice at +112 and +128)
//!  17. u32, 3× u8 trailing
//!
//! Steps 1-14 are typed; everything from step 15 is in `tail_blob`.
//!
//! New helper: `sub_1410FFF10` = CArray<u32> hash-keyed at
//! qword_145F0DA00.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct StoreInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub lookup_18: u32,
        pub sub_a_list: CArray<u32>,
        pub raw_40: [u8; 8],
        pub u8_48: u8,
        pub u64_list: CArray<u64>,
        pub lookup_72: u32,
        pub u32_76: u32,
        pub u32_80: u32,
        pub u32_84: u32,
        pub u32_88: u32,
        pub u8_92: u8,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\storeinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\storeinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                StoreInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "storeinfo roundtrip mismatch");
    }
}
