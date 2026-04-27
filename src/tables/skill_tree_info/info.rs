//! `SkillTreeInfo.pabgb` — IDA-derived parser, 90% complete (28/31 entries
//! round-trip), reverted to blob-tail until the remaining 3 entries' divergence
//! is isolated.
//!
//! Per IDA sub_1410F9670, full layout extracted (see `RECIPE_NEXT.md` and
//! `dmm-pabgb-aio/mac_extract/skill_character_decode_progress.json`):
//!   u32 key, CString string_key, u8 is_blocked,
//!   u32 character_info, u32 faction_info, u32 item_info,
//!   u32 ui_grid_size_x, u32 ui_grid_size_y, u32 ui_texture_icon_path,
//!   LocalizableString ui_page_name,
//!   CArray<StatNode>          (sub_141109630 → 152-byte elements via sub_1410F92A0),
//!   CArray<SkillNode>         (inline u32-count loop using sub_1410F9520, 45-byte stride),
//!   u32 ui_root_node_lookup, u32 trailing_a, u64 trailing_b, u64 trailing_c.
//!
//! All 13 sub-readers fully decompiled and analyzed. 28/31 entries round-trip
//! byte-perfect with the typed parser; 3 entries (e.g. key=0x32 with 47 stat
//! nodes) are short by ~30 bytes — likely an unrecognized extra CString field
//! inside one of the StatNode sub-arrays. Investigation is preserved in the
//! decode_progress.json artifact.

use crate::pabgh_blob_table;
pabgh_blob_table! { pub struct SkillTreeInfo<'a> { key: u32, blob_field: data_blob, } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\skilltreeinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\skilltreeinfo.pabgh";
    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(SkillTreeInfo::read_with_size(&data, &mut c, e - s).unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)));
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data);
    }
}
