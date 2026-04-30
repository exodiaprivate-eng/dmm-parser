//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410DA3D0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings
//! / `docs/449_TABLE_CATALOG.md` FactionInfo section):
//!   1. u32 key                              (_key)
//!   2. CString string_key                   (_stringKey)
//!   3. u8 is_blocked                        (_isBlocked)
//!   4. CString memo                         (_memo)
//!   5. u8 category_type                     (_categoryType)
//!   6. u32 flag_component_name              (_flagComponentName,
//!      read_u32_lookup_DA30)
//!   7. u32 knowledge_info                   (_knowledgeInfo,
//!      sub_1411006D0 → qword_145F0DA28)
//!   8. u32 contribution_sub_level_info      (_contributionSubLevelInfo,
//!      inline → qword_145F0EF10)
//!   9. u32 contribution_worker_info         (_contributionWorkerInfo,
//!      inline → qword_145F0EF10)
//!  10. u32 trade_reward_dropset_info        (_tradeRewardDropsetInfo,
//!      inline → qword_145F0DA08)
//!  11. u32 faction_relation_group_info      (_factionRelationGroupInfo,
//!      inline → qword_145F0DA08)
//!  12. u32 faction_group_info               (_factionGroupInfo,
//!      sub_141100370 → qword_145F113C8 — wire u32, mem u16)
//!  13. u16 represent_faction_info_lookup_a  (sub_141102410 →
//!      qword_145F0EEF0 — wire u16, mem u16; unnamed catalog sub-field)
//!  14. u16 represent_faction_info_lookup_b  (sub_1411024C0 →
//!      qword_145F24D10 — wire u16; unnamed catalog sub-field)
//!  15. u32 represent_faction_info           (_representFactionInfo,
//!      sub_141100860 → qword_145F0DA48 — wire u32, mem u16)
//!  16. u32 stage_icon_path                  (_stageIconPath,
//!      read_u32_lookup_DA30)
//!  17. CArray<FactionUiCardEntry> faction_ui_card_list
//!      (_factionUiCardList, sub_141117AC0; per element: u32 lookup
//!      via sub_1411006D0 + nested CArray<u32 raw> = 24 mem bytes)
//!  18. CArray<StealthOption> stealth_option_list
//!      (_stealthOptionList, sub_141117920; per element:
//!      CString tag + u32 lookup via sub_1410FF430 + u8 — 8 mem bytes)
//!      ← TAIL STARTS HERE
//!  19. (tail) _factionEventDataList — 13× sub_141128990 (each a
//!      CArray of 288-byte composites via sub_1410DD2A0 + sub_1410DD420;
//!      sub_1410DD420 alone has 30+ wire reads + sub_1410DD140 unknown).
//!      Hard blocker for full Tier 1.
//!  20. (tail) u8 _isEmptyMisc                (a2+296)
//!  21. (tail) u32 _factionColor              (a2+300)
//!
//! Steps 1-18 are typed (18 of 19 catalog fields surfaced + 2 sub-field
//! lookups). Reopens cleanly for the final 3 tail fields once the
//! `_factionEventDataList` 13-slot composite is decoded.

use crate::binary::*;
use crate::pabgh_typed_blob_table;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct FactionUiCardEntry {
        pub knowledge_info: u32,
        pub list: CArray<u32>,
    }
}

py_binary_struct! {
    pub struct StealthOption<'a> {
        pub tag: CString<'a>,
        pub condition_logic: u32,
        pub flag: u8,
    }
}

pabgh_typed_blob_table! {
    pub struct FactionInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub memo: CString<'a>,
        pub category_type: u8,
        pub flag_component_name: u32,
        pub knowledge_info: u32,
        pub contribution_sub_level_info: u32,
        pub contribution_worker_info: u32,
        pub trade_reward_dropset_info: u32,
        pub faction_relation_group_info: u32,
        pub faction_group_info: u32,
        pub represent_faction_info_lookup_a: u16,
        pub represent_faction_info_lookup_b: u16,
        pub represent_faction_info: u32,
        pub stage_icon_path: u32,
        pub faction_ui_card_list: CArray<FactionUiCardEntry>,
        pub stealth_option_list: CArray<StealthOption<'a>>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\faction.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\faction.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                FactionInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "faction roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
        let Ok(data) = std::fs::read(PABGB) else {
            eprintln!("SKIP: missing fixture {}", PABGB);
            return;
        };
        let Some(entries) = load_pabgh_offsets(PABGH) else {
            eprintln!("SKIP: missing pabgh fixture {}", PABGH);
            return;
        };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = FactionInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            FactionInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
