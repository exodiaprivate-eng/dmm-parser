//! IDA-derived parser for `ItemGroupInfo.pabgb`.
//!
//! Field layout extracted from Hex-Rays decompile of the parse function
//! in the current Win exe (CrimsonDesert.exe). Field NAMES paired with
//! Mac binary __cstring declaration order. Round-trip-validated against
//! the vanilla pabgb dump from the live game install.
//!
//! DO NOT EDIT BY HAND - regenerate via tools/ida_extract.py.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::ItemGroupInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  12
// Decoded by dmm-parser:   12
// Missing in this struct:  0
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _orderIndex (direct_u16, stream=2)
// ✅ _itemInfoList (reader_4B, stream=4)
// ✅ _iconPath (reader_4B, stream=4)
// ✅ _itemCageType (direct_u8, stream=1)
// ✅ _isMonsterOnlyEquip (direct_u8, stream=1)
// ✅ _isShowCategoryString (direct_u8, stream=1)
// ✅ _isAlwaysFoldItemGroup (direct_u8, stream=1)
// ✅ _key (direct_u8, stream=1)
// ✅ _isBlocked (direct_u8, stream=1)
// ✅ _stringKey
// ✅ _itemGroupInfoList (reader_2B, stream=2)
// ✅ _groupName (reader_8B, stream=8)

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct ItemGroupInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub group_name: LocalizableString<'a>,
        pub item_group_info_list: CArray<u16>,
        pub item_info_list: CArray<u32>,
        pub category_type_list: CArray<u8>,
        pub order_index: u16,
        pub item_cage_type: u8,
        pub icon_path: u32,
        pub is_show_category_string: u8,
        pub is_group_item_lockable: u8,
        pub is_monster_only_equip: u8,
        pub is_always_fold_item_group: u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("itemgroupinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(ItemGroupInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "itemgroupinfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(ItemGroupInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let _ = &item;
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            ItemGroupInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
