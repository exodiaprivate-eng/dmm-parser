//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410DFBA0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. u8 byte_17
//!   5. u8 byte_18
//!   6. u8 byte_19
//!   7. LocalizableString name
//!   8. u8 u8_56
//!   9. sub_141E2BEB0 → struct +64 (CArray of 168-byte composite
//!      items with 30+ helpers, way over budget) ← TAIL STARTS HERE
//!  10. sub_141114DD0 (CArray with conditional alloc + nested helper)
//!  11. ... (many more body fields including sub_141D8C6D0 polymorphic
//!      GameCondition tree at struct +112)
//!
//! Steps 1-8 are typed. Step 9 onward is tail. The InteractionInfo
//! body is unusually deep (40+ wire reads, 5+ unknown helpers) and
//! includes the sub_141D8C6D0 polymorphic GameCondition descriptor.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct InteractionInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub byte_17: u8,
        pub byte_18: u8,
        pub byte_19: u8,
        pub name: LocalizableString<'a>,
        pub u8_56: u8,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB_PATH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\interactioninfo.pabgb";
    const PABGH_PATH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\interactioninfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            let item = InteractionInfo::read_with_size(&data, &mut c, e - s).unwrap_or_else(|er| panic!("entry {} k=0x{:x}: {}", i, k, er));
            assert_eq!(c, *e);
            items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "interactioninfo roundtrip mismatch");
    }
}
