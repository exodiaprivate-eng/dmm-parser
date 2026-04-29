//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410E36C0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u32 key                       (_key)
//!   2. CString string_key            (_stringKey)
//!   3. u8 is_blocked                 (_isBlocked)
//!   4. u32 ui_texture_name           (_uiTextureName, read_u32_lookup_DA30)
//!   5. u8 is_default                 (_isDefault)
//!   6. u8 expand_mercenary_type      (_expandMercenaryType)
//!   7. u32 faction_info              (_factionInfo, sub_141100860 →
//!      qword_145F0DA48)
//!   8. u32 faction_node_info         (_factionNodeInfo, sub_141101D50
//!      → qword_145F0EEE8)
//!   9. u32 skill_info                (_skillInfo, sub_1410FEBE0 →
//!      qword_145F0DA68)
//!  10. CArray<u32> character_info_list (_characterInfoList,
//!      sub_1410FF890 → qword_145F0DA08)
//!  11. CArray<u32> gimmick_info_list   (_gimmickInfoList,
//!      sub_141104540 → qword_145F0DA38)
//!  12. CArray<u16> region_info_list    (_regionInfoList,
//!      sub_1410FFAC0 → qword_145F0DA80)
//!  13. sub_141101610 → struct +80 (unknown helper) ← TAIL STARTS HERE
//!  14. (body) _stageInfoList, _isShowUI, _isShowUIAlert, …
//!
//! Steps 1-12 are typed. The knowledge body has many more fields
//! interleaved with unknowns; reopens cleanly.
//!
//! Helpers: `sub_141100860` = u32 lookup at qword_145F0DA48;
//! `sub_1410FF890` = CArray<u32> at qword_145F0DA08;
//! `sub_141104540` = CArray<u32> at qword_145F0DA38.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
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
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\knowledgeinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\knowledgeinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                KnowledgeInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "knowledgeinfo roundtrip mismatch");
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
            let item = KnowledgeInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
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
}
