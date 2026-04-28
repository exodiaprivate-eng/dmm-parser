//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410E0100` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. u8 byte_17
//!   5. u8 byte_18
//!   6. u8 byte_19
//!   7. u32 lookup_20 (sub_1410FF430 → qword_145F0E9C0)
//!   8. u32 lookup_22 (inline → qword_145F113B0)
//!   9. [u8; 12] raw_24
//!  10. u32 u32_36
//!  11. u8 u8_40
//!  12. u32 lookup_44 (inline → qword_145F1A890)
//!  13. sub_141103D50 → struct +48 (CArray of TAGGED VARIANT items:
//!      u8 tag + 4-byte lookup via case-dispatched helper
//!      sub_141104AE0/sub_1410FF5C0/sub_1410FF340/sub_141100740)
//!      ← TAIL STARTS HERE
//!
//! Steps 1-12 are typed; the tag-dispatched variant CArray lives in
//! tail. Reopens cleanly when each tag's helper is decoded.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct GamePlayTriggerInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub byte_17: u8,
        pub byte_18: u8,
        pub byte_19: u8,
        pub lookup_20: u32,
        pub lookup_22: u32,
        pub raw_24: [u8; 12],
        pub u32_36: u32,
        pub u8_40: u8,
        pub lookup_44: u32,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gameplaytrigger.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gameplaytrigger.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                GamePlayTriggerInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "gameplaytrigger roundtrip mismatch");
    }
}
