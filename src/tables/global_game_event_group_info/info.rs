//! Full Tier 1 — every wire read decoded.
//!
//! 2.04.00 reader: Mac `sub_1020735AC` (names from its Korean asserts). Wire order:
//!   stringKey, key (u16), isBlocked, updateType (u8), activeConditionInfo (u32),
//!   fieldInfo (u32), defaultOption, globalGameEventUpdateOption (list of the same option
//!   struct), globalGameEventInfoList (u16 keys).
//! The option struct (Mac sub_102073254) gained `_maxUpdateTime` in 2.04. The pre-2.04
//! `unk_*` / `sub_list` / `events` / `tail_optional` names were positional.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    /// GlobalGameEventUpdateOption (Mac sub_102073254).
    pub struct GlobalGameEventUpdateOption {
        pub mode_key: u32,
        pub execute_percent: u64,
        pub min_update_time: u64,
        pub max_update_time: u64,
        pub max_count: u32,
    }
}

py_binary_struct! {
    pub struct GlobalGameEventGroupInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u16,
        pub is_blocked: u8,
        pub update_type: u8,
        pub active_condition_info: u32,
        pub field_info: u32,
        pub default_option: GlobalGameEventUpdateOption,
        pub global_game_event_update_option: CArray<GlobalGameEventUpdateOption>,
        pub global_game_event_info_list: CArray<u16>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("globalgameeventgroup.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                GlobalGameEventGroupInfo::read_from(&data, &mut c)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e, "entry {} key=0x{:x}: cursor at {} expected {}", i, k, c, e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "globalgameeventgroup roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else {
            eprintln!("SKIP: pabgh not found");
            return;
        };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut c = *start;
            let item = GlobalGameEventGroupInfo::read_from(&data, &mut c).unwrap();
            assert_eq!(c, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            GlobalGameEventGroupInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
