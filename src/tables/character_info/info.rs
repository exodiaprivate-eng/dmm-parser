//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410D7480` in CrimsonDesert.exe (Win build). Massive
//! 8616-byte function — largest pabgb reader in the binary. Reader
//! string xref via " CharacterInfo" (with leading space) at 0x144ae12e0.
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u32 key                       (_key)
//!   2. CString string_key            (_stringKey)
//!   3. u8 is_blocked                 (_isBlocked)
//!   4. LocalizableString name        (_characterName)
//!   5. LocalizableString desc        (_characterDesc)
//!   6. u32 ui_icon_path              (_uiIconPath, read_u32_lookup_DA30)
//!   7. u32 category                  (_category, read_u32_lookup_DA30)
//!   8. CString character_edit_name   (_characterEditName)
//!   9. u8 spawn_actor_type           (_spawnActorType)
//!  10. u8 none_player_sub_type       (_nonePlayerSubType)
//!  11. u32 equip_info                (_equipInfo, inline → qword_145F0EF30)
//!  12. u32 npc_info                  (_npcInfo, inline → qword_145F15060)
//!  13. sub_1411007B0 → struct +110 (unknown helper)
//!      ← TAIL STARTS HERE
//!  14. (body, in tail blob) _vehicleInfo, _callMercenaryCoolTime,
//!      _callMercenarySpawnDuration, _mercenaryCoolTimeType,
//!      _childVehicleList, _factionInfo, _upperActionChartPackageGroupName,
//!      _lowerActionChartPackageGroupName, _characterGamePlayDataName,
//!      _appearanceName, _characterPrefabPath, _skeletonName, …
//!      (~100 more body fields)
//!
//! Steps 1-12 are typed (12 fields). Body has 100+ wire reads.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

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
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = CharacterInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: {}", i, key, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "characterinfo roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else {
            eprintln!("SKIP: missing pabgh fixture {}", PABGH_PATH);
            return;
        };
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
