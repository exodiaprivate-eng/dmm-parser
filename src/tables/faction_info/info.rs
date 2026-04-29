//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410DA3D0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
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
//!  12. _factionGroupInfo (sub_141100370 → struct +46) ← TAIL STARTS HERE
//!  13. (body) _representFactionInfo, _stageIconPath, _factionUiCardList,
//!      …
//!
//! Steps 1-11 are typed (11 fields). The faction body has many more
//! reads but several unknown helpers. Reopens cleanly when decoded.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

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
}
