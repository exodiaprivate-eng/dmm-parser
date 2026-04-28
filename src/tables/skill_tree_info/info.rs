//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader (Mac CrimsonDesert_Steam): `sub_101857BC0` at 0x101857BC0.
//! 28 KB pabgb / 31 records.
//!
//! ## Why still Tier 1.5
//!
//! A prior session attempted full Tier 1; 28/31 entries round-tripped
//! but 3 entries (e.g. key=0x32 with 47 stat nodes) diverged by ~30
//! bytes. Recent Mac-IDB analysis (post-Tier-2-grind, 2026-04-28)
//! identified TWO distinct bugs in that attempt:
//!
//!   1. **CArray order was swapped**: prior comment claimed
//!      `CArray<StatNode>` then `CArray<SkillNode>`. Mac error
//!      strings prove the wire reads `_skillNodeList` FIRST, then
//!      `_statNodeList`. The 152-byte vs 45-byte stride numbers in
//!      the prior comment were attached to the wrong type.
//!   2. **StatNode element is VARIABLE-SIZED, not fixed**: it
//!      contains a CString `_uiCommand` field (sub_1006B40F4 wraps
//!      `[u32 length][length bytes]`). The 3 diverging entries had
//!      non-empty `_uiCommand` strings totaling ~30 extra bytes.
//!
//! ## Wire reads (canonical names from Mac Korean error strings)
//!
//! TYPED PREFIX:
//!   1. u32 key                       (_key, sub_100EF935C)
//!   2. CString string_key            (_stringKey, sub_1006B3F50)
//!   3. u8 is_blocked                 (_isBlocked, sub_1006B3CC0)
//!   4. u32 character_info            (_characterInfo, inline u32 hash
//!      → CharacterKey u16 lookup)
//!   5. u32 faction_info              (_factionInfo, inline → FactionNodeKey)
//!   6. u32 item_info                 (_itemInfo, inline → ItemKey)
//!   7. u32 ui_grid_size_x            (_uiGridSizeX, sub_1006B3D80)
//!   8. u32 ui_grid_size_y            (_uiGridSizeY)
//!   9. u32 ui_texture_icon_path      (_uiTextureIconPath, sub_100C93428)
//!  10. LocalizableString ui_page_name (_uiPageName, sub_1006D8484)
//!      ← TAIL STARTS HERE
//!
//! BODY (typed in Tier 1, currently in tail_blob):
//!  11. CArray<SkillNode> _skillNodeList (sub_101887858, struct +72)
//!      — VARIABLE-SIZE elements (each has 17 fields including
//!      multiple CArrays + CString-like helpers).
//!  12. CArray<StatNode> _statNodeList (sub_101887C48, struct +88)
//!      — VARIABLE-SIZE elements due to `_uiCommand` CString.
//!  13. u32 first_focus_skill_info   (_firstFocusSkillInfo, sub_10074B660)
//!  14. u32 first_focus_zoom         (_firstFocusZoom, sub_1006B3DE0,
//!      f32-as-u32)
//!  15. [u8;8] first_focus_position  (_firstFocusPosition, sub_1006B4C60
//!      vtable[2] width=8)
//!  16. [u8;8] skill_tree_area       (_skillTreeArea, sub_1006B4C60)
//!
//! ## StatNode layout (decoded — ready for future Tier 1 wiring)
//!
//! Element reader: `sub_10185791C` (Mac). 8 wire fields:
//!   1. u32 _id                (sub_1006B3D80, struct +0)
//!   2. u32 _itemInfoWrapper   (inline u32 hash → ItemKey lookup,
//!      stored as u16 at struct +4, wire 4)
//!   3. u32 _subLevelInfoWrapper (inline → SubLevelKey, stored as
//!      u16 at struct +6, wire 4)
//!   4. CString _uiCommand     (sub_1006B40F4, wire 4 + len bytes —
//!      THIS IS THE VARIABLE-SIZE FIELD)
//!   5. [u8;8] _uiPosition     (sub_1006B4C60, vec2)
//!   6. u8 _nodeType           (vtable[2] width=1, struct +20)
//!   7. u32 _decoLineNodeId    (sub_1006B3D80, struct +24)
//!   8. [u8;16] _color         (sub_1005FFBFC = LinearColor: 4× f32
//!      RGBA)
//!
//! Min wire size = 4+4+4+4+8+1+4+16 = 45 bytes (when uiCommand empty).
//! With non-empty uiCommand, add the string length + 4 length bytes
//! were already counted, so just add string length bytes.
//!
//! ## SkillNode layout (FULLY decoded as of 2026-04-28)
//!
//! Element reader: `sub_1018574CC` (Mac). 17 fields:
//!   1. u32 _id (sub_1006B3D80, wire 4)
//!   2. u32 _skillInfoWrapper (inline → SkillKey, stored u16, wire 4)
//!   3. u32 _skillLevel (sub_1006B3D60 = u32 read, wire 4)
//!   4. u32 _knowledgeInfo (inline → KnowledgeKey, wire 4)
//!   5. u32 _conditionInfo (inline → ConditionKey, wire 4)
//!   6. u32 _uiPositionX (sub_1006B3D80, wire 4)
//!   7. u32 _uiPositionY (sub_1006B3D80, wire 4)
//!   8. u32 _decoLineNodeId (sub_1006B3D80, wire 4)
//!   9. [u8;8] _uiPosition (sub_1006B4C60, wire 8)
//!  10. u32 _parentId (sub_1006B3D80, wire 4)
//!  11. CArray<u32> _childIdList (sub_100C52EA4 = CArray<u32>: u32
//!      count + N×u32, wire 4+4N)
//!  12. _uiParentDataList (sub_101887514 = HASH MAP: u32 count +
//!      N×(u32 key + sub_1018876B0 value). The value-reader
//!      sub_1018876B0 is the LAST UNDECODED HELPER for full Tier 1)
//!  13. CArray<u32> _uiChildIdForGuideline (sub_100C52EA4)
//!  14. u8 _nodeType (vtable[2] width=1, wire 1)
//!  15. CArray<u32> _uiLearnNeedNodeList (sub_100C52EA4)
//!  16. [u8;16] _color (sub_1005FFBFC LinearColor, wire 16)
//!  17. u32 _factionResearchKey (sub_100F2893C = u32 read, wire 4)
//!
//! Wire size = 4+4+4+4+4+4+4+4+8+4 + (4+4*childCount) +
//!   (4+(4+sub_1018876B0_size)*parentDataCount) +
//!   (4+4*guidelineCount) + 1 + (4+4*learnNeedCount) + 16 + 4
//! = 71 + variable CArray contents + parentDataMap. Per-element
//! is fully variable.
//!
//! ## Roadmap to Tier 1
//!
//! 1. Decompile `sub_1018876B0` (the last unknown — value-reader for
//!    the `_uiParentDataList` hash-map). Likely a small struct
//!    (CString + u32 + ...).
//! 2. Implement a custom CHashMap<u32, T> reader in `binary/mod.rs`
//!    or a one-off helper here. Wire = u32 count + N×(u32 key + V).
//! 3. Add `SkillTreeStatNode` py_binary_struct! type with the 8
//!    fields documented above (uses CString, so variable-size).
//! 4. Add `SkillTreeSkillNode` type with the 17 fields above (uses
//!    multiple CArray<u32> + the hash-map + LinearColor).
//! 5. Replace `tail_blob: Vec<u8>` with
//!    `skill_node_list: CArray<SkillTreeSkillNode>`,
//!    `stat_node_list: CArray<SkillTreeStatNode>`,
//!    `first_focus_skill_info: u32`, `first_focus_zoom: u32`,
//!    `first_focus_position: [u8;8]`, `skill_tree_area: [u8;8]`.
//! 6. Run `cargo test --lib --no-default-features
//!    skill_tree_info::info::tests::roundtrip` — should be 31/31
//!    byte-perfect (with the CString uiCommand bug now fixed).

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
