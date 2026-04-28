//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410E4450` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. u32 lookup_18 (sub_141104AE0 → qword_145F11D70)
//!   5. [u8; 8] raw_24
//!   6. [u8; 8] raw_32
//!   7. sub_141113BF0 → struct +40 (unknown helper)
//!      ← TAIL STARTS HERE
//!   8. (After tail) sub_141101AB0, sub_141102990, CString, 2× u8,
//!      [u8;12], 2× u8 trailing, sub_141113A50 ×2, sub_141104540,
//!      sub_1411138C0, sub_1411135E0, u32, u8, sub_141101960,
//!      sub_14106BAC0, sub_141113410, [u8;8], u32, u8, ~50 more u8s
//!      and lookups, sub_141C79D00 etc.
//!
//! Steps 1-6 are typed (6 fields). Body has 100+ wire reads.
//!
//! New helper: `sub_141104AE0` = u32 lookup at qword_145F11D70.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct GimmickGroupInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub lookup_18: u32,
        pub raw_24: [u8; 8],
        pub raw_32: [u8; 8],
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gimmickgroupinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gimmickgroupinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                GimmickGroupInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "gimmickgroupinfo roundtrip mismatch");
    }
}
