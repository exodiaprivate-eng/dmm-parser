//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410D7480` in CrimsonDesert.exe (Win build). Massive
//! 8616-byte function — largest pabgb reader in the binary. Reader
//! string xref via " CharacterInfo" (with leading space) at 0x144ae12e0.
//!
//! Wire reads, in order:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. LocalizableString name
//!   5. LocalizableString desc
//!   6. u32 lookup_88 (read_u32_lookup_DA30)
//!   7. u32 lookup_90 (read_u32_lookup_DA30)
//!   8. CString cstring_96
//!   9. u8 byte_104
//!  10. u8 byte_105
//!  11. u32 lookup_106 (inline → qword_145F0EF30)
//!  12. u32 lookup_108 (inline → qword_145F15060)
//!  13. sub_1411007B0 → struct +110 (unknown helper)
//!      ← TAIL STARTS HERE
//!  14. (After tail) 8 raw + 8 raw, u8, 2-iter loop with
//!      sub_1410FF340 + sub_1411003E0, sub_141100860, 7×
//!      read_u32_lookup_DA30, u32, 2× read_u32_lookup_DA30,
//!      sub_1411008D0, u32, read_u32_lookup_DA30, 2× u32, 3× u8,
//!      sub_141100950, LocalizableString, sub_1410FF340, u8, u16,
//!      sub_1410E0380, ~80 trailing u8s, more.
//!
//! Steps 1-12 are typed (12 fields). Body has 100+ wire reads.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct CharacterInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub name: LocalizableString<'a>,
        pub desc: LocalizableString<'a>,
        pub lookup_88: u32,
        pub lookup_90: u32,
        pub cstring_96: CString<'a>,
        pub byte_104: u8,
        pub byte_105: u8,
        pub lookup_106: u32,
        pub lookup_108: u32,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB_PATH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\characterinfo.pabgb";
    const PABGH_PATH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\characterinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = CharacterInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: {}", i, key, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "characterinfo roundtrip mismatch");
    }
}
