//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410DA3D0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. CString second_string
//!   5. u8 byte_32
//!   6. u32 lookup_34 (read_u32_lookup_DA30)
//!   7. u32 lookup_36 (sub_1411006D0 → qword_145F0DA28)
//!   8-9. 2× u32 inline lookups at qword_145F0EF10 (lookup_a, lookup_b)
//!  10-11. 2× u32 inline lookups at qword_145F0DA08 (lookup_c, lookup_d)
//!  12. sub_141100370 → struct +46 (unknown helper) ← TAIL STARTS HERE
//!  13. (After tail) sub_141102410, sub_1411024C0, sub_141100860,
//!      read_u32_lookup_DA30, sub_141117AC0, sub_141117920,
//!      13× sub_141128990 loop, u8, u32.
//!
//! Steps 1-11 are typed (11 fields). The faction body has many more
//! reads but several unknown helpers. Reopens cleanly when decoded.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct FactionInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub second_string: CString<'a>,
        pub byte_32: u8,
        pub lookup_34: u32,
        pub lookup_36: u32,
        pub lookup_a: u32,
        pub lookup_b: u32,
        pub lookup_c: u32,
        pub lookup_d: u32,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\faction.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\faction.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                FactionInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "faction roundtrip mismatch");
    }
}
