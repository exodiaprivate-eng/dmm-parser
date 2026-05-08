//! Full Tier 1 — every wire read decoded.
//!
//! Reader: `sub_1410DDA70` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings —
//! 1.3.5 audit re-mapped placeholder names like `name`, `ref_list`,
//! `lookup_b/c/d` to canonical):
//!   1. u16 key (pabgh format 2)                 (_key)
//!   2. CString string_key                       (_stringKey)
//!   3. u8 is_blocked                            (_isBlocked)
//!   4. LocalizableString faction_group_name     (_factionGroupName)
//!   5. CArray<u32> faction_info_list            (_factionInfoList,
//!      sub_1410FFC20 hash-keyed at qword_145F0DA48)
//!   6. u32 knowledge_info                       (_knowledgeInfo,
//!      sub_1411006D0 single-shot lookup at qword_145F0DA28)
//!   7. u32 ui_icon_path                         (_uiIconPath,
//!      read_u32_lookup_DA30)
//!   8. u32 ui_daily_quest_image_path            (_uiDailyQuestImagePath,
//!      read_u32_lookup_DA30)
//!
//! All helpers consult runtime hash dictionaries; raw wire u32 round-trips.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct FactionGroupInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub faction_group_name: LocalizableString<'a>,
        pub faction_info_list: CArray<u32>,
        pub knowledge_info: u32,
        pub ui_icon_path: u32,
        pub ui_daily_quest_image_path: u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/factiongroup.pabgb";
    const PABGH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/factiongroup.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                FactionGroupInfo::read_from(&data, &mut c)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e, "entry {} key=0x{:x}: cursor at {} expected {}", i, k, c, e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "factiongroup roundtrip mismatch");
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
            let mut c = *start;
            let item = FactionGroupInfo::read_from(&data, &mut c).unwrap();
            assert_eq!(c, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            FactionGroupInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
