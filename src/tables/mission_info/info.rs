//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410ED0E0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. u32 lookup_20 (sub_141102CB0 → qword_145F0EF20)
//!   5. sub_1411049D0 → struct +24 (unknown 16-byte slot helper)
//!      ← TAIL STARTS HERE
//!   6. (After tail) sub_1411068C0, inline CArray<sub_1410ED7D0
//!      16-byte items>, sub_1410FF890, 2× sub_1411069E0,
//!      sub_141106AE0, sub_141100510, sub_14110DE30, u16, sub_1410EC8B0
//!      (80-byte!), 4× LocalizableString, sub_141102D90, sub_14110DCE0,
//!      sub_14110DB10, sub_1410FF430, 2× u16, u32, 14× u8,
//!      sub_141BD4120 (4-byte). 50+ wire reads in body.
//!
//! Steps 1-4 are typed. The mission body has many helpers; reopens
//! cleanly when each is decoded.
//!
//! New helper: `sub_141102CB0` = u32 lookup at qword_145F0EF20.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct MissionInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub lookup_20: u32,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\missioninfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\missioninfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                MissionInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "missioninfo roundtrip mismatch");
    }
}
