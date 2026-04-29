//! Full Tier 1 — every wire read decoded.
//!
//! Reader: `sub_14111AA70` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u16 key (note: u16, not u32; pabgh format 2)
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. CArray<u16> rel_0 (via sub_141125F30: u32 count + N×u16)
//!   5. CArray<u16> rel_1
//!   6. CArray<u16> rel_2
//!   7. CArray<u16> rel_3
//!
//! `sub_141125F30` runtime-looks-up each u16 in a hash table at
//! `qword_145F0EEF0`, but the round-trip stores the raw wire u16. No
//! polymorphic helpers.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct FactionRelationGroupInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub rel_0: CArray<u16>,
        pub rel_1: CArray<u16>,
        pub rel_2: CArray<u16>,
        pub rel_3: CArray<u16>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\factionrelationgroup.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\factionrelationgroup.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                FactionRelationGroupInfo::read_from(&data, &mut c)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e, "entry {} key=0x{:x}: cursor at {} expected {}", i, k, c, e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "factionrelationgroup roundtrip mismatch");
    }
}
