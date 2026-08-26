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
// Canonical fields re-derived from current Win exe (sub_1410E14A0, 2026-05-29).
// Mac-binary catalog was stale — _isSellable and _isGrowable no longer exist;
// _sharedSummonCountTag and _feedFromGimmickInfo are new; _summonOwnerOption
// and _parentMercenaryGroupInfo changed wire width; _setNewMercenaryIsMain added.
//
// ✅ _key (direct_u8, stream=1)
// ✅ _stringKey
// ✅ _isBlocked (direct_u8, stream=1)
// ✅ _defaultLimitSummonCount (direct_u32, stream=4)
// ✅ _defaultLimitHireCount (direct_u32, stream=4)
// ✅ _maxLimitHireCount (direct_u32, stream=4)
// ✅ _mercenaryType (direct_u8, stream=1)
// ✅ _farFromLeaderOption (direct_u8, stream=1)
// ✅ _isControllable (direct_u8, stream=1)
// ✅ _isPlayable (direct_u8, stream=1)
// ✅ _summonAfterRegist (direct_u8, stream=1)
// ✅ _mainMercenaryPerTribe (direct_u8, stream=1)
// ✅ _isForceStackable (direct_u8, stream=1)
// ✅ _useCampLevel (direct_u8, stream=1)
// ✅ _applyEquipItemStat (direct_u8, stream=1)
// ✅ _checkItemNoOnPushToItem (direct_u8, stream=1)
// ✅ _allowExceedLimitHireCount (direct_u8, stream=1)
// ✅ _isSelectMercenarySpawn (direct_u8, stream=1)
// ✅ _unspawnOnFocusActorChanged (direct_u8, stream=1)
// ✅ _isMainDischargeable (direct_u8, stream=1)
// ✅ _spawnPositionType (direct_u8, stream=1)
// ✅ _summonOwnerOption (direct_u8, stream=1)   — was u32, now u8 in-block
// ✅ _parentMercenaryGroupInfo (lookup_u8, stream=1) — was u32, now 1B
// ✅ _sharedSummonCountTag (lookup_u32, stream=4)   — re-added
// ✅ _feedFromGimmickInfo (lookup_u32, stream=4)    — new
// ✅ _hiredSkillInfoList
// ✅ _setNewMercenaryIsMain (direct_u8, stream=1)   — new trailing byte

use crate::binary::*;
use crate::py_binary_struct;

// NOTE: 1.10 removed hired_skill_info_list from MercenaryInfo, so the former
// HiredSkillData element struct ({u32 skill_lookup_key + u32 level}, IDA
// sub_141100A00) is no longer used and was removed. If a future version
// re-adds the list, restore it here.

// ─── 2026-05-29 IDA re-decode against current Win exe ────────────────────
// Decompiled reader: sub_1410E14A0 (CrimsonDesert.exe 1.07+).
// 16 consecutive u8 fields at struct offsets +32..+47, then three
// helper reads (1B, 4B, 4B), then CArray, then 1 trailing u8.
//
// Changes from the previous Mac-canonical struct:
//   REMOVED: _isSellable, _isGrowable (fields dropped from game)
//   MOVED:   summon_owner_option — u32 post-block → u8 as 16th in-block field
//   CHANGED: parent_mercenary_group_info u32 → u8 (sub_1410FD230 reads 1B)
//   RE-ADDED: shared_summon_count_tag u32 (sub_1410F5B30 reads 4B; was
//             removed in 1.0.8 comment but present again in current binary)
//   ADDED:   feed_from_gimmick_info u32 (sub_1410F75A0 reads 4B; new field)
//   ADDED:   set_new_mercenary_is_main u8 (trailing byte; canonical name
//            from error string table — was listed ✅ in catalog but omitted)
//
// IDA error-string order confirming wire sequence (Korean: 읽어들이는데 실패했다):
//   _key → _stringKey → _isBlocked → _defaultLimitSummonCount →
//   _defaultLimitHireCount → _maxLimitHireCount → _mercenaryType →
//   _farFromLeaderOption → _isControllable → _isPlayable →
//   _summonAfterRegist → _mainMercenaryPerTribe → _isForceStackable →
//   _useCampLevel → _applyEquipItemStat → _checkItemNoOnPushToItem →
//   _allowExceedLimitHireCount → _isSelectMercenarySpawn →
//   _unspawnOnFocusActorChanged → _isMainDischargeable →
//   _spawnPositionType → _summonOwnerOption (u8, 16th in block) →
//   _parentMercenaryGroupInfo (u8, sub_1410FD230) →
//   _sharedSummonCountTag (u32, sub_1410F5B30) →
//   _feedFromGimmickInfo (u32, sub_1410F75A0) →
//   _hiredSkillInfoList → _setNewMercenaryIsMain
py_binary_struct! {
    pub struct MercenaryInfo<'a> {
        pub key: u8,                              // _key
        pub string_key: CString<'a>,             // _stringKey
        pub is_blocked: u8,                      // _isBlocked
        pub default_limit_summon_count: u32,     // _defaultLimitSummonCount
        pub default_limit_hire_count: u32,       // _defaultLimitHireCount
        pub max_limit_hire_count: u32,           // _maxLimitHireCount
        // 16 u8 fields (struct offsets +32..+47 in IDA, wire bytes sequential)
        pub mercenary_type: u8,                  // _mercenaryType
        pub far_from_leader_option: u8,          // _farFromLeaderOption
        pub is_controllable: u8,                 // _isControllable
        pub is_playable: u8,                     // _isPlayable
        pub summon_after_regist: u8,             // _summonAfterRegist
        pub main_mercenary_per_tribe: u8,        // _mainMercenaryPerTribe
        pub is_force_stackable: u8,              // _isForceStackable
        pub use_camp_level: u8,                  // _useCampLevel
        pub apply_equip_item_stat: u8,           // _applyEquipItemStat
        pub check_item_no_on_push_to_item: u8,   // _checkItemNoOnPushToItem
        pub allow_exceed_limit_hire_count: u8,   // _allowExceedLimitHireCount
        pub is_select_mercenary_spawn: u8,       // _isSelectMercenarySpawn
        pub unspawn_on_focus_actor_changed: u8,  // _unspawnOnFocusActorChanged
        pub is_main_dischargeable: u8,           // _isMainDischargeable
        // ── 1.18.00: `_isLossInventoryItem`, one u8 in the u8 block. Position
        // is exact, not a zero-run guess: in keys 0x4E and 0x42 the inserted
        // byte is `01` while the old byte at that offset is `00`, so it cannot
        // slide — it lands between is_main_dischargeable and spawn_position_type.
        pub is_loss_inventory_item: u8,          // _isLossInventoryItem
        // ── 2.00.00: fifteen new fields ─────────────────────────────
        // The companion overhaul in the Enhanced update (patch notes: companion
        // damage, flying-mount control, breeding). Recovered from the reader
        // rather than guessed — `field_order.py MercenaryInfo` gives the read
        // order, and the offsets in sub_10203C630 line up exactly:
        //
        //   +34..+51  eighteen 1-byte reads  = indices 8..25
        //   +56       _statusUIList          = index 26
        //   +72..+80  nine 1-byte reads      = indices 27..35
        //
        // Widths are the vtable call's third argument (sub_100E4BBE4 -> 1LL),
        // not inferred from the data.
        pub use_wagon_road: u8,                  // _useWagonRoad
        pub is_show_ddd_status_detail: u8,       // _isShowDDDStatusDetail
        pub is_show_horse_status: u8,            // _isShowHorseStatus
        pub is_require_login: u8,                // _isRequireLogin
        pub is_summon_recovery_item: u8,         // _isSummonRecoveryItem
        pub status_ui_list: CArray<u32>,         // _statusUIList
        pub is_check_grown_target_list: u8,      // _isCheckGrownTargetList
        pub is_main_mercenary_info: u8,          // _isMainMercenaryInfo
        pub can_have_breeding_targets: u8,       // _canHaveBreedingTargets
        pub is_use_inventory: u8,                // _isUseInventory
        pub is_focus_changeable: u8,             // _isFocusChangeable
        pub is_connection_required: u8,          // _isConnectionRequired
        pub is_buff_level_changeable: u8,        // _isBuffLevelChangeable
        pub is_min_hp_makeable: u8,              // _isMinHPMakeable
        pub is_fireable: u8,                     // _isFireable
        pub spawn_position_type: u8,             // _spawnPositionType
        pub summon_owner_option: u8,             // _summonOwnerOption (was u32, now u8 in-block)
        pub parent_mercenary_group_info: u8,     // _parentMercenaryGroupInfo (sub_1410FD230: 1B)
        // 1.10: tail RESTRUCTURED. shared_summon_count_tag (u32),
        // hired_skill_info_list (CArray), and set_new_mercenary_is_main (u8)
        // are GONE; the post-block tail is now u8 + u32 + u32 (fixed).
        // The trailing u8 mirrors the record key in vanilla; the first u32 is
        // a name-hash (0xEAC5E173 sentinel when empty); the second u32 is a
        // key reference. Verified via wire-walker: all 18 records byte-exact.
        pub tail_u8_110: u8,
        // 1.11: one new u8 inserted between tail_u8_110 and the name-hash (small
        // enum/flag, 0x40/0x41 observed), and _hiredSkillInfoList re-added as the
        // trailing CArray (16-byte elements; was removed in 1.10). Verified via
        // wire-walker against 1.11 pabgh boundaries: all 18 records byte-exact
        // (empty list in 17 records, 8 entries in the "Pet" record).
        pub tail_u8b_111: u8,
        pub feed_from_gimmick_info: u32,         // _feedFromGimmickInfo (name-hash)
        pub tail_u32_110: u32,
        pub hired_skill_info_list: CArray<HiredSkillData>,  // _hiredSkillInfoList (1.11 re-add)
        // ── 1.18.00: `_cameraPresetHash`, one u32 appended after the hired
        // skill list. Present in all 21 records; 5 carry real hashes
        // (0xBEE0C77B, 0x221089D6, 0x54473E0C, 0x5CB92E95, 0x70ED6590), the
        // rest are 0. A name-hash, consistent with the canonical name.
        pub camera_preset_hash: u32,             // _cameraPresetHash
    }
}

py_binary_struct! {
    /// 1.11 _hiredSkillInfoList element (16 wire bytes). Field 2 is the skill
    /// lookup key, field 3 the level; the two zero u32s are unknown padding /
    /// reserved (always 0 in vanilla). Kept as u32 for bit-exact roundtrip.
    pub struct HiredSkillData {
        pub unk0: u32,
        pub skill_lookup_key: u32,
        pub level: u32,
        pub unk1: u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("mercenaryinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
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
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
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
