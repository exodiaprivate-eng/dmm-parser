//! IDA-derived parser for `MercenaryInfo.pabgb`.
//!
//! Field layout extracted from Hex-Rays decompile of the parse function
//! in the current Win exe (CrimsonDesert.exe). Field NAMES paired with
//! Mac binary __cstring declaration order. Round-trip-validated against
//! the vanilla pabgb dump from the live game install.
//!
//! DO NOT EDIT BY HAND - regenerate via tools/ida_extract.py.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::MercenaryInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  15
// Decoded by dmm-parser:   15
// Missing in this struct:  0
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _setNewMercenaryIsMain (direct_u8, stream=1)
// ✅ _isControllable (direct_u8, stream=1)
// ✅ _isForceStackable (direct_u8, stream=1)
// ✅ _mainMercenaryPerTribe (direct_u8, stream=1)
// ✅ _useCampLevel (direct_u8, stream=1)
// ✅ _isSellable (direct_u8, stream=1)
// ✅ _spawnPositionType (direct_u8, stream=1)
// ✅ _applyEquipItemStat (direct_u8, stream=1)
// ✅ _key (direct_u8, stream=1)
// ✅ _isBlocked (direct_u8, stream=1)
// ✅ _stringKey
// ✅ _defaultLimitHireCount (direct_u32, stream=4)
// ✅ _defaultLimitSummonCount (direct_u32, stream=4)
// ✅ _farFromLeaderOption (direct_u8, stream=1)
// ✅ _maxLimitHireCount (direct_u32, stream=4)

use crate::binary::*;
use crate::py_binary_struct;

// Hand-corrected: parent_mercenary_group_info reads as u8 (file) but is stored
// as u16 (looked-up via dictionary). hired_skill_info_list element is
// {u32 lookup_key + u32 value} per IDA sub_141100A00.
py_binary_struct! {
    pub struct HiredSkillData {
        pub skill_lookup_key: u32,
        pub level: u32,
    }
}

// ─── 2026-05-12 Mac-canonical rewrite ─────────────────────────────────────
// Field structure now matches the Mac binary parser sub_101893AF0
// (CrimsonDesert_Steam, 1.06) byte-for-byte and name-for-name. The
// previous struct packed multiple Mac u8 fields into wider Rust
// fields (combat_targeting_flags: u32, packed_flags_106: u8) under
// placeholder names; total wire bytes matched but field semantics
// did not. This rewrite unpacks every u8 into its canonical Mac name.
//
// Total wire bytes per record: 45 + N + 8K (identical to pre-rewrite).
// Roundtrip on the existing 2026-5-1 fixture should be preserved
// because the wire byte sequence is unchanged.
py_binary_struct! {
    pub struct MercenaryInfo<'a> {
        pub key: u8,                              // _key
        pub string_key: CString<'a>,              // _stringKey
        pub is_blocked: u8,                       // _isBlocked
        pub default_limit_summon_count: u32,      // _defaultLimitSummonCount
        pub default_limit_hire_count: u32,        // _defaultLimitHireCount
        pub max_limit_hire_count: u32,            // _maxLimitHireCount
        pub mercenary_type: u8,                   // _mercenaryType
        pub far_from_leader_option: u8,           // _farFromLeaderOption
        pub is_controllable: u8,                  // _isControllable
        pub is_playable: u8,                      // _isPlayable
        pub summon_after_regist: u8,              // _summonAfterRegist
        pub main_mercenary_per_tribe: u8,         // _mainMercenaryPerTribe
        pub is_force_stackable: u8,               // _isForceStackable
        pub is_sellable: u8,                      // _isSellable
        pub use_camp_level: u8,                   // _useCampLevel
        pub apply_equip_item_stat: u8,            // _applyEquipItemStat
        pub is_growable: u8,                      // _isGrowable
        pub check_item_no_on_push_to_item: u8,    // _checkItemNoOnPushToItem
        pub allow_exceed_limit_hire_count: u8,    // _allowExceedLimitHireCount
        pub is_select_mercenary_spawn: u8,        // _isSelectMercenarySpawn
        pub unspawn_on_focus_actor_changed: u8,   // _unspawnOnFocusActorChanged
        pub is_main_dischargeable: u8,            // _isMainDischargeable
        pub spawn_position_type: u8,              // _spawnPositionType
        pub summon_owner_option: u8,              // _summonOwnerOption
        pub parent_mercenary_group_info: u8,      // _parentMercenaryGroupInfo
        pub shared_summon_count_tag: u32,         // _sharedSummonCountTag
                                                  //   (constant 0xEAC5E173)
        pub hired_skill_info_list: CArray<HiredSkillData>, // _hiredSkillInfoList
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/mercenaryinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(MercenaryInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "mercenaryinfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(MercenaryInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            MercenaryInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
