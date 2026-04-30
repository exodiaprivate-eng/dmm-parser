//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410D7480` in CrimsonDesert.exe (Win build). Massive
//! 8616-byte function — largest pabgb reader in the binary. Reader
//! string xref via " CharacterInfo" (with leading space) at 0x144ae12e0.
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. LocalizableString name
//!   5. LocalizableString desc
//!   6. u32 ui_icon_path                   (read_u32_lookup_DA30 wire u32)
//!   7. u32 category                       (read_u32_lookup_DA30 wire u32)
//!   8. CString character_edit_name
//!   9. u8 spawn_actor_type
//!  10. u8 none_player_sub_type
//!  11. u32 equip_info                     (inline → qword_145F0EF30)
//!  12. u32 npc_info                       (inline → qword_145F15060)
//!  13. u16 vehicle_info                   (sub_1411007B0 wire u16)
//!  14. u64 call_mercenary_cool_time       (8 raw bytes)
//!  15. u64 call_mercenary_spawn_duration  (8 raw bytes)
//!  16. u8 mercenary_cool_time_type
//!  17. CharacterActionChartEntry upper_chart  (u32 group + u16 package)
//!  18. CharacterActionChartEntry lower_chart  (u32 group + u16 package)
//!  19. u32 character_game_play_data_name  (sub_141100860 wire u32)
//!  20. u32 appearance_name                (read_u32_lookup_DA30 wire u32)
//!  21. u32 character_prefab_path
//!  22. u32 skeleton_name
//!  23. u32 lookup_22
//!  24. u32 lookup_23
//!  25. u32 lookup_24
//!  26. u32 lookup_25
//!  27. u32 raw_a                          (4 raw bytes at +156)
//!  28. u32 lookup_27                      (read_u32_lookup_DA30)
//!  29. u32 lookup_28                      (read_u32_lookup_DA30)
//!  30. u32 lookup_29                      (sub_1411008D0 wire u32)
//!  31. u32 raw_b                          (at +168)
//!  32. u32 lookup_31                      (read_u32_lookup_DA30)
//!  33. u32 raw_c                          (at +176)
//!  34. u32 raw_d                          (at +180)
//!  35. u8 flag_a                          (at +184)
//!  36. u8 flag_b
//!  37. u8 flag_c
//!  38. u8 flag_d                          (sub_141100950 wire u8)
//!  39. LocalizableString label_a          (at +192, 32 mem bytes)
//!  40. u32 lookup_36                      (sub_1410FF340 wire u32, at +224)
//!  41. u8 flag_e                          (at +226)
//!  42. u16 raw_e                          (at +228, 2 raw bytes)
//!  43. CharacterFourFlags four_flags      (sub_1410E0380, 4× u8 at +230)
//!  44-82. 39× u8 flags                    (at +234 through +272)
//!  83. u32 raw_f                          (at +276)
//!  84. u32 lookup_77                      (read_u32_lookup_DA30, at +280)
//!  85. u32 lookup_78                      (at +282)
//!  86. CArray<u64> list_a                 (sub_141100A00 — per element
//!      u32 lookup (sub_1410FEBE0) + u32 raw)
//!  87. CArray<u64> list_b                 (sub_141100A00)
//!  88. CArray<u64> list_c                 (sub_141100A00)
//!  89. CArray<u64> list_d                 (sub_141100A00)
//!  90. CArray<u32> list_e                 (sub_141100B10 → qword_145F0DA78)
//!  91. u32 raw_g                          (at +368)
//!  92. u32 lookup_84                      (sub_141BF63C0 wire u32)
//!  93. CArray<u32> list_f                 (sub_141F8F830 — per element 4
//!      wire bytes hashed via sub_141BF61C0)
//!      ← TAIL STARTS HERE
//!  94. (body) ~50 more wire reads — _factionInfo continued plus
//!      _appearanceName, _characterGamePlayDataName, _characterPrefabPath,
//!      _skeletonName, more u8 flags, CArrays, polymorphic descriptors.
//!
//! Steps 1-93 are typed (93 of ~150 wire fields).

use crate::binary::*;
use crate::pabgh_typed_blob_table;
use crate::py_binary_struct;

// 2-iter loop body in sub_1410D7480: u32 lookup + u16 lookup per entry.
py_binary_struct! {
    pub struct CharacterActionChartEntry {
        pub group_lookup: u32,    // sub_1410FF340 wire u32
        pub package_lookup: u16,  // sub_1411003E0 wire u16
    }
}

// sub_1410E0380 inner: 4 u8s = 4 wire bytes.
py_binary_struct! {
    pub struct CharacterFourFlags {
        pub flag_a: u8,
        pub flag_b: u8,
        pub flag_c: u8,
        pub flag_d: u8,
    }
}

pabgh_typed_blob_table! {
    pub struct CharacterInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub name: LocalizableString<'a>,
        pub desc: LocalizableString<'a>,
        pub ui_icon_path: u32,
        pub category: u32,
        pub character_edit_name: CString<'a>,
        pub spawn_actor_type: u8,
        pub none_player_sub_type: u8,
        pub equip_info: u32,
        pub npc_info: u32,
        pub vehicle_info: u16,
        pub call_mercenary_cool_time: u64,
        pub call_mercenary_spawn_duration: u64,
        pub mercenary_cool_time_type: u8,
        pub upper_chart: CharacterActionChartEntry,
        pub lower_chart: CharacterActionChartEntry,
        pub character_game_play_data_name: u32,
        pub appearance_name: u32,
        pub character_prefab_path: u32,
        pub skeleton_name: u32,
        pub lookup_22: u32,
        pub lookup_23: u32,
        pub lookup_24: u32,
        pub lookup_25: u32,
        pub raw_a: u32,
        pub lookup_27: u32,
        pub lookup_28: u32,
        pub lookup_29: u32,
        pub raw_b: u32,
        pub lookup_31: u32,
        pub raw_c: u32,
        pub raw_d: u32,
        pub flag_a: u8,
        pub flag_b: u8,
        pub flag_c: u8,
        pub flag_d: u8,
        pub label_a: LocalizableString<'a>,
        pub lookup_36: u32,
        pub flag_e: u8,
        pub raw_e: u16,
        pub four_flags: CharacterFourFlags,
        pub flag_38: u8,
        pub flag_39: u8,
        pub flag_40: u8,
        pub flag_41: u8,
        pub flag_42: u8,
        pub flag_43: u8,
        pub flag_44: u8,
        pub flag_45: u8,
        pub flag_46: u8,
        pub flag_47: u8,
        pub flag_48: u8,
        pub flag_49: u8,
        pub flag_50: u8,
        pub flag_51: u8,
        pub flag_52: u8,
        pub flag_53: u8,
        pub flag_54: u8,
        pub flag_55: u8,
        pub flag_56: u8,
        pub flag_57: u8,
        pub flag_58: u8,
        pub flag_59: u8,
        pub flag_60: u8,
        pub flag_61: u8,
        pub flag_62: u8,
        pub flag_63: u8,
        pub flag_64: u8,
        pub flag_65: u8,
        pub flag_66: u8,
        pub flag_67: u8,
        pub flag_68: u8,
        pub flag_69: u8,
        pub flag_70: u8,
        pub flag_71: u8,
        pub flag_72: u8,
        pub flag_73: u8,
        pub flag_74: u8,
        pub flag_75: u8,
        pub flag_76: u8,
        pub raw_f: u32,
        pub lookup_77: u32,
        pub lookup_78: u32,
        pub list_a: CArray<u64>,
        pub list_b: CArray<u64>,
        pub list_c: CArray<u64>,
        pub list_d: CArray<u64>,
        pub list_e: CArray<u32>,
        pub raw_g: u32,
        pub lookup_84: u32,
        pub list_f: CArray<u32>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB_PATH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\characterinfo.pabgb";
    const PABGH_PATH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\characterinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            let item = CharacterInfo::read_with_size(&data, &mut c, e - s).unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er));
            assert_eq!(c, *e);
            items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "characterinfo roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = CharacterInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            CharacterInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
