//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410F9670` in CrimsonDesert.exe (Win build). Prior IDA
//! work decoded the full layout (see git history + the
//! `mac_extract/skill_character_decode_progress.json` artifact); 28/31
//! entries round-tripped with the fully-typed parser, but 3 entries
//! diverged inside one of the StatNode sub-arrays. Promoting to Tier
//! 1.5 captures the 10 simple prefix fields and leaves the
//! polymorphic body (2× CArrays of StatNode/SkillNode + trailing
//! scalars) as a tail blob for byte-perfect round-trip.
//!
//! Wire reads (typed prefix):
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. u32 character_info       (lookup)
//!   5. u32 faction_info         (lookup)
//!   6. u32 item_info            (lookup)
//!   7. u32 ui_grid_size_x
//!   8. u32 ui_grid_size_y
//!   9. u32 ui_texture_icon_path (lookup)
//!  10. LocalizableString ui_page_name
//!      ← TAIL STARTS HERE
//!  11. (tail) CArray<StatNode> via sub_141109630 (152-byte elements,
//!      sub_1410F92A0 reader)
//!  12. (tail) CArray<SkillNode> inline u32-count loop, sub_1410F9520,
//!      45-byte stride
//!  13. (tail) u32 ui_root_node_lookup, u32 trailing_a, u64 trailing_b,
//!      u64 trailing_c

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct SkillTreeInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub character_info: u32,
        pub faction_info: u32,
        pub item_info: u32,
        pub ui_grid_size_x: u32,
        pub ui_grid_size_y: u32,
        pub ui_texture_icon_path: u32,
        pub ui_page_name: LocalizableString<'a>,
    }
    tail: tail_blob;
}

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
            items.push(
                SkillTreeInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "skilltreeinfo roundtrip mismatch");
    }
}
