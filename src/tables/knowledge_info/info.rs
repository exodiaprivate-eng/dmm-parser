//! Tier 1 — fully typed (no _tail_b64).
//!
//! Reader (Tier IDA verified 2026-05-19 vs CrimsonDesert.exe md5
//! 3d614280…): `sub_1410C51C0` — the KnowledgeInfo deserializer (found
//! via xref to the "KnowledgeInfo" class block at 0x144AF8110+). All
//! fields confirmed in order. (Cited `sub_1410E36C0` was stale — it is
//! an inner reader SHARED with another large table `sub_1410B8D40`,
//! 800+ byte struct, NOT KnowledgeInfo.)
//! Type confirmations (the kind byte-roundtrip can't prove):
//!   - region_info_list (sub_1410E2070): u32 count + **u16** elements
//!     (2-byte wire read + hash remap). Rust `CArray<u16>` correct —
//!     genuinely 2-byte, not the usual u32-ref.
//!   - knowledge_from_list element: u8 flag + u64 value = 9 wire bytes
//!     (read 1 + read 8). Matches `KnowledgeFromItem`.
//!   - learning_position: a single **12-byte** raw read. Rust `[f32;3]`
//!     has the right width; the float interpretation is a modeling
//!     choice (3-component position) the reader neither confirms nor
//!     denies — bits are preserved either way.
//!   - u32-wire ID refs via sub_1410E1350/E2EE0/E2D50/E1B70/E1190 etc.
//!     (4-byte wire → u16 RAM). Rust `u32` models the wire.
//!
//! ─── v3.1 closure analysis (iter 77) ────────────────────────────────────
//! Cross-check via `sub_1410AFE20` (typeinfo→record-reader path per the
//! iter 47 typeinfo registry). All wire reads pair to rust fields except
//! one — the FINAL CArray read at offset 248 (via `sub_1410D1660`):
//!
//!   Last wire read:   sub_1410D1660 → CArray<U32U32Pair>
//!   Rust field name:  `level_gimmick_scene_object_data_list`
//!   Canonical (PA):   `_linkKnowledgeNodeList`  (only unmapped schema entry)
//!
//! The rust name `level_gimmick_scene_object_data_list` was an early
//! guess. PA's canonical is `_linkKnowledgeNodeList` — semantically
//! "list of links to other knowledge-graph nodes" (each pair = source
//! node key + target node key). This is the single missing canonical.
//!
//! Closure (shipped): tuple-scoped MANUAL_OVERRIDES entry maps
//! `(knowledge_info, level_gimmick_scene_object_data_list)` →
//! `_linkKnowledgeNodeList`. Safety-checked: the literal canonical
//! `_levelGimmickSceneObjectDataList` is owned by a DIFFERENT table
//! (LevelGimmickSceneObjectInfo), so the tuple-scoping prevents
//! collisions. Schema verifier confirms knowledge_info is now 30/30
//! verified, 0 missing canonicals (down from 1).
//! KnowledgeLevelData inner reader: `sub_1410E3300` (232 byte struct).
//! KnowledgeMeditationData inner reader: `sub_1410E3170` (88 byte struct).
//! KnowledgeLearnDialog inner reader: `sub_141114070` body block.
//!
//! Wire reads, in order (canonical names from
//! `docs/449_TABLE_CATALOG.md` KnowledgeInfo section, 29 fields):
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. u32 ui_texture_name              (read_u32_lookup_DA30)
//!   5. u8 is_default
//!   6. u8 expand_mercenary_type
//!   7. u32 faction_info                 (sub_141100860 → qword_145F0DA48)
//!   8. u32 faction_node_info            (sub_141101D50 → qword_145F0EEE8)
//!   9. u32 skill_info                   (sub_1410FEBE0 → qword_145F0DA68)
//!  10. CArray<u32> character_info_list  (sub_1410FF890 → qword_145F0DA08)
//!  11. CArray<u32> gimmick_info_list    (sub_141104540 → qword_145F0DA38)
//!  12. CArray<u16> region_info_list     (sub_1410FFAC0 → qword_145F0DA80;
//!      wire is u16 per element — keeps the existing Tier 1.5 width)
//!  13. CArray<u32> stage_info_list      (sub_141101610 → qword_145F0EF38)
//!  14. u8 is_show_ui
//!  15. u8 is_show_ui_alert
//!  16. u8 is_legendary_animal
//!  17. u32 ui_component_name            (read_u32_lookup_DA30)
//!  18. CArray<KnowledgeFromItem> knowledge_from_list
//!      (inline u32 count + per element: u8 flag + u64 value — 9 wire bytes)
//!  19. CArray<u32> knowledge_group_list (sub_141104650 → qword_145F15030)
//!  20. CArray<KnowledgeLevelData> knowledge_level_data_list
//!      (sub_141113F00 wraps sub_1410E3300; 232 byte mem stride)
//!  21. CArray<u32> meditation_resource_list (sub_141104760 → qword_145F0DA28)
//!  22. u32 shared_level_main_knowledge_info (sub_1411006D0 → qword_145F0DA28)
//!  23. CArray<u32> shared_level_knowledge_info_list (sub_141104760)
//!  24. CArray<KnowledgeAliasMapEntry> knowledge_alias_map
//!      (sub_141113D80; per element: CString key + CString value)
//!  25. u32 item_info                    (sub_1410FF5C0 → qword_145F0DA00)
//!  26. u8 bitmap_color_r
//!  27. [f32; 3] learning_position       (12 wire bytes inline)
//!  28. u32 learning_stage_info          (sub_141102D20 → qword_145F0EF38)
//!  29. u32 learn_apply_skill_info       (sub_1410FEBE0)
//!
//! KnowledgeLevelData (sub_1410E3300, 232 mem bytes, 12 fields):
//!   1. u32 level
//!   2. LocalizableString name
//!   3. LocalizableString description
//!   4. u32 ui_lookup                    (qword_145F23858 hash → u16)
//!   5. CArray<u32> related_lookup_list  (qword_145F113B0)
//!   6. u32 icon                         (read_u32_lookup_DA30)
//!   7. CArray<U32U32Pair> learnable_data_list (sub_1411043B0 wraps
//!      sub_141104AE0 → qword_145F11D70 + raw u32)
//!   8. CArray<KnowledgeMeditationData> meditation_data_list
//!      (88-byte memory stride via sub_1410E3170)
//!   9. CArray<U32U32Pair> meditation_resource_list_2
//!      (sub_141104230 wraps sub_1411006D0 + raw u32)
//!  10. CArray<KnowledgeLearnDialog> learn_dialog_list
//!      (104-byte memory stride via sub_141114070 body)
//!  11. LocalizableString tail_localizable_a
//!  12. LocalizableString tail_localizable_b
//!
//! KnowledgeMeditationData (sub_1410E3170, 88 mem bytes, 10 fields):
//!   1. CArray<MeditationLabelEntry> label_list
//!      (sub_1411142E0; u32 lookup + LocalizableString)
//!   2. CArray<MeditationItemEntry> item_lookup_list
//!      (sub_141103310; u32 lookup + 8 raw bytes)
//!   3. CArray<U32U32Pair> material_list_a (sub_141104230)
//!   4. CArray<U32U32Pair> material_list_b (sub_141104230)
//!   5. u32 group_leader_info            (sub_141104340 → qword_145F0E9B8)
//!   6. u8 flag1
//!   7. u8 flag2
//!   8. u32 raw_a
//!   9. u32 raw_b
//!  10. u32 extra_lookup                 (sub_1410FF050 → qword_145F0DA60)
//!
//! KnowledgeLearnDialog (sub_141114070 inner, 104 mem bytes, 6 fields):
//!   1. LocalizableString text_a
//!   2. LocalizableString text_b
//!   3. LocalizableString text_c
//!   4. u32 lookup_a                     (read_u32_lookup_DA30)
//!   5. u32 lookup_b                     (read_u32_lookup_DA30)
//!   6. u32 extra_lookup                 (sub_1410FF050)


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::KnowledgeInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  30
// Decoded by dmm-parser:   27
// Missing in this struct:  3
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _key
// ✅ _isBlocked (direct_15B, stream=15)
// ✅ _stringKey
// ✅ _isDefault (direct_15B, stream=15)
// ✅ _uiTextureName (reader_4B, stream=4)
// ✅ _learnApplySkillInfo (reader_4B, stream=4)
// ✅ _sharedLevelMainKnowledgeInfo (reader_4B, stream=4)
// ✅ _meditationResourceList (reader_4B, stream=4)
// ✅ _knowledgeAliasMap
// ✅ _sharedLevelKnowledgeInfoList (reader_4B, stream=4)
// ✅ _bitmapColorR (direct_15B, stream=15)
// ✅ _itemInfo (reader_4B, stream=4)
// ✅ _learningStageInfo (reader_4B, stream=4)
// ✅ _learningPosition (direct_12B, stream=12)
// ⏳ _isShowUIAlert (direct_15B, stream=15)
// ⏳ _isShowUI (direct_15B, stream=15)
// ✅ _uiComponentName (reader_4B, stream=4)
// ✅ _isLegendaryAnimal (direct_15B, stream=15)
// ✅ _knowledgeGroupList (reader_4B, stream=4)
// ✅ _knowledgeFromList (reader_4B, stream=4)
// ⏳ _linkKnowledgeNodeList (reader_4B, stream=4)
// ✅ _knowledgeLevelDataList
// ✅ _factionInfo (reader_4B, stream=4)
// ✅ _expandMercenaryType (direct_15B, stream=15)
// ✅ _skillInfo (reader_4B, stream=4)
// ✅ _factionNodeInfo (reader_4B, stream=4)
// ✅ _gimmickInfoList (reader_4B, stream=4)
// ✅ _characterInfoList (reader_4B, stream=4)
// ✅ _stageInfoList (reader_4B, stream=4)
// ✅ _regionInfoList (reader_2B, stream=2)

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct KnowledgeFromItem {
        pub flag: u8,
        pub value: u64,
    }
}

py_binary_struct! {
    pub struct U32U32Pair {
        pub a: u32,
        pub b: u32,
    }
}

py_binary_struct! {
    pub struct KnowledgeAliasMapEntry<'a> {
        pub key: CString<'a>,
        pub value: CString<'a>,
    }
}

py_binary_struct! {
    pub struct MeditationLabelEntry<'a> {
        pub lookup: u32,
        pub label: LocalizableString<'a>,
    }
}

py_binary_struct! {
    pub struct MeditationItemEntry {
        pub lookup: u32,
        pub raw: u64,
    }
}

py_binary_struct! {
    pub struct KnowledgeMeditationData<'a> {
        pub label_list: CArray<MeditationLabelEntry<'a>>,
        pub item_lookup_list: CArray<MeditationItemEntry>,
        pub material_list_a: CArray<U32U32Pair>,
        pub material_list_b: CArray<U32U32Pair>,
        pub group_leader_info: u32,
        pub flag1: u8,
        pub flag2: u8,
        pub raw_a: u32,
        pub raw_b: u32,
        pub extra_lookup: u32,
    }
}

py_binary_struct! {
    pub struct KnowledgeLearnDialog<'a> {
        pub text_a: LocalizableString<'a>,
        pub text_b: LocalizableString<'a>,
        pub text_c: LocalizableString<'a>,
        pub lookup_a: u32,
        pub lookup_b: u32,
        pub extra_lookup: u32,
    }
}

py_binary_struct! {
    pub struct KnowledgeLevelData<'a> {
        pub level: u32,
        pub name: LocalizableString<'a>,
        pub description: LocalizableString<'a>,
        pub ui_lookup: u32,
        pub related_lookup_list: CArray<u32>,
        pub icon: u32,
        pub learnable_data_list: CArray<U32U32Pair>,
        pub meditation_data_list: CArray<KnowledgeMeditationData<'a>>,
        pub meditation_resource_list_2: CArray<U32U32Pair>,
        pub learn_dialog_list: CArray<KnowledgeLearnDialog<'a>>,
        pub tail_localizable_a: LocalizableString<'a>,
        pub tail_localizable_b: LocalizableString<'a>,
    }
}

py_binary_struct! {
    pub struct KnowledgeInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub ui_texture_name: u32,
        pub is_default: u8,
        pub expand_mercenary_type: u8,
        pub faction_info: u32,
        pub faction_node_info: u32,
        pub skill_info: u32,
        pub character_info_list: CArray<u32>,
        pub gimmick_info_list: CArray<u32>,
        pub region_info_list: CArray<u16>,
        pub stage_info_list: CArray<u32>,
        pub is_show_ui: u8,
        pub is_show_ui_alert: u8,
        pub is_legendary_animal: u8,
        pub ui_component_name: u32,
        pub knowledge_from_list: CArray<KnowledgeFromItem>,
        pub knowledge_group_list: CArray<u32>,
        pub knowledge_level_data_list: CArray<KnowledgeLevelData<'a>>,
        pub meditation_resource_list: CArray<u32>,
        pub shared_level_main_knowledge_info: u32,
        pub shared_level_knowledge_info_list: CArray<u32>,
        pub knowledge_alias_map: CArray<KnowledgeAliasMapEntry<'a>>,
        pub item_info: u32,
        pub bitmap_color_r: u8,
        pub learning_position: [f32; 3],
        pub learning_stage_info: u32,
        pub learn_apply_skill_info: u32,
        pub level_gimmick_scene_object_data_list: CArray<U32U32Pair>,
    }
}

impl<'a> KnowledgeInfo<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> std::io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        let consumed = *offset - start;
        if consumed != entry_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("KnowledgeInfo: consumed {} bytes, expected {}", consumed, entry_size),
            ));
        }
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser\pabgb-dumps-1.07\knowledgeinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser\pabgb-dumps-1.07\knowledgeinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            let item = KnowledgeInfo::read_from(&data, &mut c)
                .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er));
            assert_eq!(c, *e, "e{} k=0x{:x}: under/over-read {}/{}", i, k, c - s, e - s);
            items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "knowledgeinfo roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = KnowledgeInfo::read_from(&data, &mut cursor).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            KnowledgeInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }

    /// Confirm typed lists carry data — guards against silent regression.
    #[test]
    fn typed_lists_populated() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut totals = (0usize, 0usize, 0usize, 0usize);
        for (_, s, _) in &ranges {
            let mut c = *s;
            let item = KnowledgeInfo::read_from(&data, &mut c).unwrap();
            totals.0 += item.knowledge_from_list.items.len();
            totals.1 += item.knowledge_level_data_list.items.len();
            totals.2 += item.knowledge_alias_map.items.len();
            totals.3 += item
                .knowledge_level_data_list
                .items
                .iter()
                .map(|l| l.meditation_data_list.items.len())
                .sum::<usize>();
        }
        eprintln!(
            "knowledge_info: {} entries, knowledge_from={} levels={} alias_map={} meditations={}",
            ranges.len(), totals.0, totals.1, totals.2, totals.3
        );
    }
}
