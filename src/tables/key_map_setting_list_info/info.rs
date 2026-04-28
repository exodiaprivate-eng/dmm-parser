//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader (Mac CrimsonDesert_Steam): `sub_101851F00` at 0x101851F00
//! (size 0xdc, tiny). Pabgb dump path is `keymap.pabgb` (822 B).
//!
//! Wire reads, in order:
//!   1. u32 key                  (sub_100F1AC74, width 4)
//!   2. CString string_key       (sub_1006B3F50, struct +8)
//!   3. u8 is_blocked            (sub_1006B3CC0, struct +16)
//!      ← TAIL STARTS HERE
//!   4. (tail) _keyMapSettingList (sub_101881CD4, struct +24, unknown
//!      CArray-like helper)
//!
//! Stop at field 3 because the tail helper is unknown (likely
//! CArray<KeyMapSetting> with polymorphic or composite element type).

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct KeyMapSettingListInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\keymap.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\keymap.pabgh";
    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                KeyMapSettingListInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "keymapsettinglistinfo (keymap.pabgb) roundtrip mismatch");
    }
}
