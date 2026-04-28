//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410F4620` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. u8 byte_17
//!   5. u8 byte_18
//!   6. LocalizableString name
//!   7. LocalizableString desc
//!   8. sub_141106F50 (thunk → sub_14B99E300, deep) → struct +88
//!      ← TAIL STARTS HERE
//!
//! Steps 1-7 are typed (7 fields). Tail captures sub_141106F50 onward
//! with many more body fields (sub_141100860, sub_1410DDBC0,
//! sub_141D8EF30, sub_1410FF890, sub_141107270, sub_141102EF0, inline
//! CArray<u16> at qword_145F12678, sub_1411049D0, sub_141101610,
//! sub_141102D90, 2× sub_141102D20, 3× read_u32_lookup_DA30, 2× u32,
//! CString, sub_141102D20, sub_1410A9D40, CString, sub_14110AD30,
//! sub_1411049D0, sub_1410FF430, 3× u8, u32).

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct QuestInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub byte_17: u8,
        pub byte_18: u8,
        pub name: LocalizableString<'a>,
        pub desc: LocalizableString<'a>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\questinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\questinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                QuestInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "questinfo roundtrip mismatch");
    }
}
