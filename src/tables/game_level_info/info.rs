//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader (Mac CrimsonDesert_Steam): `sub_101852C78` at 0x101852C78
//! (size 0x260). Pabgb dump path is `levelinfo.pabgb` (14 KB).
//!
//! The reader has INLINE hash-lookup loops for fields 4 and 5 (no
//! clean wrapper helpers — the lookup logic is open-coded against
//! StaticInfoWrapper<StringInfoKey,...> and
//! StaticInfoWrapper<RegionKey,...> manager structs).
//!
//! Wire reads, in order:
//!   1. u32 key                    (sub_100F12348, width 4)
//!   2. CString string_key         (sub_1006B3F50, struct +8)
//!   3. u8 is_blocked              (sub_1006B3CC0, struct +16)
//!   4. u32 default_level_data_name (inline: sub_100F051D8 init v26
//!      + sub_100F055C0 read u32 from stream → StringInfoKey hash
//!      lookup → u16 index at struct +18, wire 4)
//!   5. u16 update_region_info     (inline: sub_100F015A0 init v28
//!      + sub_100F01988 read u16 from stream → RegionKey hash lookup
//!      → u16 index at struct +20, wire 2)
//!      ← TAIL STARTS HERE
//!   6. (tail) _levelDataList      (sub_101882B74, struct +24,
//!      unknown CArray-like helper)

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct GameLevelInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub default_level_data_name: u32,
        pub update_region_info: u16,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\levelinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\levelinfo.pabgh";
    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                GameLevelInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "gamelevelinfo (levelinfo.pabgb) roundtrip mismatch");
    }
}
