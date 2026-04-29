//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410DE7A0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u32 key                                  (_key)
//!   2. CString string_key                       (_stringKey)
//!   3. u8 is_blocked                            (_isBlocked)
//!   4. u32 knowledge_info                       (_knowledgeInfo)
//!   5. u32 skill_tree_info                      (_skillTreeInfo)
//!   6. u32 connect_research_node_info           (_connectResearchNodeInfo)
//!   7. u16 store_info                           (_storeInfo)
//!   8. u16 royal_supply_info                    (_royalSupplyInfo)
//!   9. CString memo                             (_memo)
//!  10. CArray<u32> child_faction_info_list      (_childFactionInfoList)
//!  11. CArray<u32> node_line_main_faction_info_list (_nodeLineMainFactionInfoList)
//!  12. [u8; 12] world_position                  (_worldPosition, vec3)
//!  13. u32 node_radius                          (_nodeRadius, f32-as-u32)
//!  14. _applySkillDataList (sub_141115FD0 → struct +88) ← TAIL STARTS HERE
//!  15. (body) _resourceItemList, …
//!
//! Steps 1-13 are typed. The body has many more typed-shaped fields
//! interleaved with unknowns; reopens cleanly when the helper family
//! is decoded.
//!
//! New helpers: `sub_1411035A0` = u32 lookup at qword_145F1A740;
//! `sub_141101D50` = u32 lookup at qword_145F0EEE8; `sub_1411036C0` =
//! u16 lookup at qword_145F113A0; `sub_141102FF0` = CArray<u32>
//! hash-keyed at qword_145F0EEE8.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct FactionNodeInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub knowledge_info: u32,
        pub skill_tree_info: u32,
        pub connect_research_node_info: u32,
        pub store_info: u16,
        pub royal_supply_info: u16,
        pub memo: CString<'a>,
        pub child_faction_info_list: CArray<u32>,
        pub node_line_main_faction_info_list: CArray<u32>,
        pub world_position: [u8; 12],
        pub node_radius: u32,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\factionnode.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\factionnode.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                FactionNodeInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "factionnode roundtrip mismatch");
    }
}
