//! Tier 1 — fully typed parser. All 174 wire fields editable; tail blob
//! is empty for every vanilla entry (see `roundtrip` test which prints
//! `0 nonempty tails`). The `pabgh_typed_blob_table!` macro now omits
//! `_tail_b64` from JSON output when the tail Vec is empty, and the
//! `json_roundtrip` test asserts the field never leaks.
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
//!  94. u32 raw_h                          (4 raw bytes at +392)
//!  95. u8 flag_85                         (at +396)
//!  96. u8 flag_86                         (at +397)
//!  97. u32 lookup_87                      (inline u32 → qword_145F113B0
//!      hash lookup → u16 at +398)
//!  98. u8 flag_88                         (at +400)
//!  99. u32 lookup_89                      (sub_1411006D0 wire u32, +402)
//! 100. u8 flag_90                         (at +404)
//! 101. CArray<CharacterMercenaryEntry> mercenary_list (sub_141118980 →
//!      sub_1410D9880; 96 mem bytes / 20 wire fields per entry incl.
//!      CString hash, LocalizableString, 4 lookups, 11 raw u8/u32)
//! 102. CArray<u16> list_g                 (sub_1410FF0C0 wire u16)
//! 103. u8 flag_91                         (at +440)
//!      ← TAIL STARTS HERE
//! 104. (tail, conditional) when flag_91 == 0: sub_141105AC0 reads u32
//!      wire / u16 mem at +442. When flag_91 != 0, this read is SKIPPED.
//!      Vanilla distribution: flag_91 == 0 (2035 entries), flag_91 == 2
//!      (4931 entries). Promotion past field 103 requires manual impl
//!      to dispatch on flag_91 — pabgh_typed_blob_table macro can't
//!      express conditional reads.
//! 105. (tail) sub_141100C20 — u16 wire / u16 mem at +444
//! 106. (tail) 2 raw bytes (u16) at +446
//! 107. (tail) u8 at +448
//! 108. (tail) u8 at +449
//! 109. (tail) sub_1410FFAC0 — CArray<u16> at +456
//! 110. (tail) u8 at +472
//! 111. (tail) CString at +480
//! 112. (tail) sub_1410FEE90 — u16 wire / u16 mem at +488
//! 113. (tail) inline u16 wire / u16 mem at +490 (qword_145F290C0 lookup)
//! 114. (tail) sub_141100C90 — u32 wire / u16 mem at +492
//! 115. (tail) sub_141100D00 — u32 wire / u16 mem at +494 (post-process
//!      via sub_141BF6720)
//! 116. (tail) 4 raw bytes (u32) at +496
//! 117. (tail) u8 at +500
//! 118. (tail) sub_141100510 — CArray<u32> at +504
//! 119. (tail) sub_1410FF890 — CArray<u32> at +520
//! 120. (tail) sub_1410FF890 — CArray<u32> at +536
//! 121. (tail) 4 raw bytes (u32) at +552
//! 122. (tail) sub_1411187E0 — CArray of 12-wire-byte items (u16 lookup
//!      sub_141100370 + u32 raw + u32 raw) at +560
//! 123. (tail) u8 at +576
//! 124. (tail) sub_141100510 — CArray<u32> at +584
//! 125. (tail) sub_141100D80 — CArray of 64-byte items via sub_1410D7170
//!      (per element: 2 u32 lookups + 7× u64 = 64 wire bytes) at +600
//! 126. (tail) sub_141100E90 — CArray<FactionAdjacencyMobItem> at +616
//! 127-128. (tail) 2× sub_141118620 — CArray of 24-byte items per element
//!      (u32 lookup + u64 raw + u32 raw + u32 lookup = 20 wire bytes)
//!      at +632 and +648
//!
//! Steps 1-172 typed via incremental walk through sub_1410D7480
//! (8616-byte function decompiled to a tool-results file then
//! grep-traversed). Reusable per-element substructs:
//!
//!   CharacterField136Entry  — 32-byte stride (CArray<u32> + 3 lookups
//!                             + u64), used by sub_1411181F0 at +760.
//!   CharacterField137Entry  — 8-byte stride (2× u32 lookup), used by
//!                             sub_141101380 at +784.
//!   CharacterField141Entry  — 24-byte stride (2× u32 lookup +
//!                             CArray<u32>), used by sub_141118000.
//!   CharacterField144Entry  — 12-byte stride (2× u32 + u8 + u32),
//!                             used by sub_1411014B0 at +864.
//!   CharacterInline147       — 14 wire bytes (3× u32 + 2× u8) inline
//!                             via sub_1410D7370 at +896 (NOT a CArray).
//!   CharacterField150Entry  — 32-byte stride (u32 + CString-hash +
//!                             2× Vec3), used by sub_141101710.
//!   CharacterField169Entry  — 16-byte stride (u32 lookup + u64), one
//!                             entry of the 5-iter loop at +1032.
//!   CharacterFiveU64        — 5× u64 (40 wire bytes), used by
//!                             sub_1411001A0 inner.
//!   CharacterField171Entry  — 128-byte stride for sub_141117EC0
//!                             (u32 + 2× u64 + [u32; 4] + u32 + 5
//!                             nested CArrays).
//!
//! Field 173 (sub_141101AB0 CArray<u32>) and field 174 (sub_141101B80
//! CArray<{u32, u32}>) close out the wire layout. The earlier "field_173
//! breaks the test" symptom was caused by a missing 3-iter raw u32 loop
//! at mem +1008-+1019 (the loop that runs between raw_165 and lookup_166
//! in sub_1410D7480). Adding raw_165a/b/c restored byte alignment and
//! cleared the cascade.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::CharacterInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  164
// Decoded by dmm-parser:   18
// Missing in this struct:  146
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ⏳ _ownedMercenaryCharacterInfo (reader_4B, stream=4)
// ⏳ _mercenaryHireMessage (reader_8B, stream=8)
// ⏳ _playerIndex (direct_u16, stream=2)
// ⏳ _spawnFixType (direct_u8, stream=1)
// ⏳ _isRemoteCatchable (direct_u8, stream=1)
// ⏳ _isCatchable (direct_u8, stream=1)
// ⏳ _isAttackable (direct_u8, stream=1)
// ⏳ _isAttackThrowable (direct_u8, stream=1)
// ⏳ _tribeEffectHash (direct_u32, stream=4)
// ⏳ _tribeInfoWrapper
// ⏳ _aiScriptPathHash (direct_u32, stream=4)
// ⏳ _characterTribeAndGenderString (reader_4B, stream=4)
// ⏳ _playerTargetableType (direct_u8, stream=1)
// ⏳ _aiScriptPathFocusHash (direct_u32, stream=4)
// ⏳ _mercenaryInfo (reader_1B, stream=1)
// ⏳ _gender (direct_u8, stream=1)
// ✅ _appearanceName (reader_4B, stream=4)
// ✅ _characterGamePlayDataName (reader_4B, stream=4)
// ✅ _skeletonName (reader_4B, stream=4)
// ✅ _characterPrefabPath (reader_4B, stream=4)
// ⏳ _shareValueNameHash (direct_u32, stream=4)
// ⏳ _skeletonVariationName (reader_4B, stream=4)
// ⏳ _memo (reader_4B, stream=4)
// ⏳ _projectileInfoPackage (reader_4B, stream=4)
// ✅ _callMercenaryCoolTime (direct_u64, stream=8)
// ✅ _vehicleInfo (reader_2B, stream=2)
// ✅ _mercenaryCoolTimeType (direct_u8, stream=1)
// ✅ _callMercenarySpawnDuration (direct_u64, stream=8)
// ⏳ _factionInfo (reader_4B, stream=4)
// ⏳ _childVehicleList
// ⏳ _lowerActionChartPackageGroupName (reader_4B, stream=4)
// ⏳ _upperActionChartPackageGroupName (reader_4B, stream=4)
// ✅ _uiIconPath (reader_4B, stream=4)
// ⏳ _characterDesc (reader_8B, stream=8)
// ✅ _characterEditName
// ✅ _category (reader_4B, stream=4)
// ✅ _nonePlayerSubType (direct_u8, stream=1)
// ✅ _spawnActorType (direct_u8, stream=1)
// ✅ _npcInfo
// ✅ _equipInfo
// ✅ _stringKey
// ✅ _key
// ⏳ _characterName (reader_8B, stream=8)
// ✅ _isBlocked (direct_u8, stream=1)
// ⏳ _detectInfo (reader_2B, stream=2)
// ⏳ _visioningData
// ⏳ _personalityType (direct_u8, stream=1)
// ⏳ _maxAggroCount (direct_u16, stream=2)
// ⏳ _characterRegionInfoList (reader_2B, stream=2)
// ⏳ _characterTier (direct_u8, stream=1)
// ⏳ _characterWeaponType
// ⏳ _characterAge (direct_u8, stream=1)
// ⏳ _characterType (direct_u8, stream=1)
// ⏳ _battleOrderType (direct_u8, stream=1)
// ⏳ _mapIconDisplayType (direct_u8, stream=1)
// ⏳ _uiMapTextureInfo
// ⏳ _knowledgeObtainType (direct_u8, stream=1)
// ⏳ _knowledgeInfo (reader_4B, stream=4)
// ⏳ _characterGroupInfoList (reader_2B, stream=2)
// ⏳ _inspectDataList
// ⏳ _aliveSkillInfoList (reader_4B, stream=4)
// ⏳ _skillInfoByReviveList (reader_4B, stream=4)
// ⏳ _interactionInfoList (reader_4B, stream=4)
// ⏳ _playerSkillInfoList (reader_4B, stream=4)
// ⏳ _defaultActionActionIndex (reader_4B, stream=4)
// ⏳ _interactionDistance (direct_u32, stream=4)
// ⏳ _characterWeight (direct_u32, stream=4)
// ⏳ _defaultShareValueIndex
// ⏳ _ignoreWaterFall (direct_u8, stream=1)
// ⏳ _refillHPWhenCooltimeEnd (direct_u8, stream=1)
// ⏳ _enableDockingGimmickAutoWallUp (direct_u8, stream=1)
// ⏳ _isCreatableDetectIcon (direct_u8, stream=1)
// ⏳ _uiPortraitPath (reader_4B, stream=4)
// ⏳ _vanishTickCount (direct_u32, stream=4)
// ⏳ _skillInfoBySpawnList (reader_4B, stream=4)
// ⏳ _symbolImage (reader_4B, stream=4)
// ⏳ _isTerrainCharacter (direct_u8, stream=1)
// ⏳ _ignoreTriggerRegion (direct_u8, stream=1)
// ⏳ _isWallSwingable (direct_u8, stream=1)
// ⏳ _isMapIconAlwaysShow (direct_u8, stream=1)
// ⏳ _isClimbable (direct_u8, stream=1)
// ⏳ _isItemSocketContents (direct_u8, stream=1)
// ⏳ _allowFarAttackTarget (direct_u8, stream=1)
// ⏳ _isEnableFriendly (direct_u8, stream=1)
// ⏳ _isEquipDropable (direct_u8, stream=1)
// ⏳ _invincibility (direct_u8, stream=1)
// ⏳ _isEditorUsableAppearance (direct_u8, stream=1)
// ⏳ _isEditorUsable (direct_u8, stream=1)
// ⏳ _isVisibleWhenDetectModeOnly (direct_u8, stream=1)
// ⏳ _disableFootStepOptimize (direct_u8, stream=1)
// ⏳ _isGhost (direct_u8, stream=1)
// ⏳ _obstacleDisableByDead (direct_u8, stream=1)
// ⏳ _isRandomAppearance_IgnoreScale (direct_u8, stream=1)
// ⏳ _isRandomAppearance (direct_u8, stream=1)
// ⏳ _isHirable (direct_u8, stream=1)
// ⏳ _isRandomCharacter (direct_u8, stream=1)
// ⏳ _sendKillEventOnDead (direct_u8, stream=1)
// ⏳ _useLargeSplineCurve (direct_u8, stream=1)
// ⏳ _isHudHpEnabled (direct_u8, stream=1)
// ⏳ _isShowHpWhenFocusActor (direct_u8, stream=1)
// ⏳ _isUnique (direct_u8, stream=1)
// ⏳ _isAggroTargetable (direct_u8, stream=1)
// ⏳ _isLookable (direct_u8, stream=1)
// ⏳ _isPushable (direct_u8, stream=1)
// ⏳ _isUseScheduleInfo_Dev (direct_u8, stream=1)
// ⏳ _isLogoutAtLooted (direct_u8, stream=1)
// ⏳ _isSealable (direct_u8, stream=1)
// ⏳ _isGlobalSchedule (direct_u8, stream=1)
// ⏳ _grownTargetKeyList (reader_4B, stream=4)
// ⏳ _catchSpawnData
// ⏳ _statusGroupInfo (reader_4B, stream=4)
// ⏳ _grownLevel (direct_u32, stream=4)
// ⏳ _detectableGimmickTagNameHashList (reader_4B, stream=4)
// ⏳ _characterLevelDataList
// ⏳ _elementalMaterialInfoList (reader_4B, stream=4)
// ⏳ _mercenaryDetectableGimmickTagHashList (reader_4B, stream=4)
// ⏳ _breakableObjectInfo
// ⏳ _characterScale (direct_u32, stream=4)
// ⏳ _miniGameParam
// ⏳ _weakPointEffectDataList
// ⏳ _jobInfo (reader_2B, stream=2)
// ⏳ _bulletItem (reader_4B, stream=4)
// ⏳ _isFarmAnimal (direct_u8, stream=1)
// ⏳ _baseMaterialKeyOverride (direct_u32, stream=4)
// ⏳ _forceFieldTargetType (direct_u8, stream=1)
// ⏳ _weatherWeight (direct_u32, stream=4)
// ⏳ _attackByCollisionInfoListKey (direct_u32, stream=4)
// ⏳ _additionalPartsDataList
// ⏳ _detectReactionOverrideList (reader_14B, stream=14)
// ⏳ _interactionUIDistanceLv (direct_u8, stream=1)
// ⏳ _gamePlayObjectShareData (array_or_complex, stream=4)
// ⏳ _stageInfoForNpcShopList (reader_4B, stream=4)
// ⏳ _characterInteractionOverrideDataList
// ⏳ _dockingChildEventList (reader_14B, stream=14)
// ⏳ _bumpTypeHash (direct_u32, stream=4)
// ⏳ _characterCollisionType (direct_u8, stream=1)
// ⏳ _characterThreatDialogInfo
// ⏳ _characterFriendlyItemDataList
// ⏳ _trapFoodData
// ⏳ _aiDialogOverrideList (reader_4B, stream=4)
// ⏳ _terrainRegionAutoSpawnInfo
// ⏳ _wantedPriceList (array_or_complex, stream=8)
// ⏳ _convertItemInfo (reader_4B, stream=4)
// ⏳ _terrainRegionSpawnPerCount (direct_u32, stream=4)
// ⏳ _inventoryInfoList (reader_2B, stream=2)
// ⏳ _pathTrailType (direct_u8, stream=1)
// ⏳ _dockingChildDataList
// ⏳ _pathFindTableName (direct_u32, stream=4)
// ⏳ _farmBreedingCoolTime (direct_u32, stream=4)
// ⏳ _farmBreedingResultList (reader_4B, stream=4)
// ⏳ _isRewardDropRollByCreateActor (direct_u8, stream=1)
// ⏳ _characterRewardDataList
// ⏳ _equipItemInfoList
// ⏳ _mercenaryDropInfoList (reader_4B, stream=4)
// ⏳ _priceList (array_or_complex, stream=8)
// ⏳ _minigameSeedList
// ⏳ _interactionCategoryGroupInfo
// ⏳ _dialogVoiceInfo (reader_2B, stream=2)
// ⏳ _allyGroupInfo
// ⏳ _detectReactionInfo (reader_4B, stream=4)
// ⏳ _ownerFollowType (direct_u8, stream=1)
// ⏳ _characterPauseType (direct_u32, stream=4)
// ⏳ _farmBreedingTargetList (reader_4B, stream=4)
// ⏳ _farmDropInfoList (reader_4B, stream=4)

use crate::binary::*;
use crate::binary::variants::gimmick_interaction_override::GimmickInteractionOverrideCArray;
use crate::json_traits::{ToJsonValue, WriteJsonValue};
use crate::pabgh_typed_blob_table;
use crate::py_binary_struct;
use serde_json::Value;
use std::io::{self, Write};

/// Conditional u32 wire field that's only present when the IMMEDIATELY
/// PRECEDING byte (flag_91) is 0. Reverse-engineered from sub_1410D7480
/// where:
/// ```c
/// if ( !*(_BYTE *)(a2 + 440) && !sub_141105AC0(a1, a2 + 442) )
/// ```
/// Vanilla distribution (6966 entries): flag_91 == 0 (2035 entries, has
/// the conditional u32) vs flag_91 == 2 (4931 entries, no conditional).
///
/// Implemented as a wire-trait wrapper around `Option<u32>` so the
/// `pabgh_typed_blob_table!` macro can keep generating CharacterInfo's
/// 100+ fields straightline. read_from peeks at `data[offset - 1]` to
/// dispatch — relies on the macro reading flag_91 immediately before.
#[derive(Debug)]
pub struct Conditional92(pub Option<u32>);

impl<'a> BinaryRead<'a> for Conditional92 {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        // Peek at the byte just consumed by flag_91 (the previous field).
        if *offset == 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                "Conditional92: cannot read at offset 0 (no preceding flag_91 byte)"));
        }
        let flag_91 = data[*offset - 1];
        if flag_91 == 0 {
            Ok(Self(Some(u32::read_from(data, offset)?)))
        } else {
            Ok(Self(None))
        }
    }
}

impl BinaryWrite for Conditional92 {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        if let Some(v) = &self.0 {
            v.write_to(w)?;
        }
        Ok(())
    }
}

impl<'a> BinaryReadTracked<'a> for Conditional92 {
    fn read_tracked(
        data: &'a [u8],
        offset: &mut usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        if item.0.is_some() {
            ranges.push(FieldRange {
                path: path.clone(),
                start,
                end: *offset,
                ty: "Conditional92",
            });
        }
        Ok(item)
    }
}

impl ToJsonValue for Conditional92 {
    fn to_json_value(&self) -> Value {
        match &self.0 {
            Some(v) => v.to_json_value(),
            None => Value::Null,
        }
    }
}

impl WriteJsonValue for Conditional92 {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() {
            // No bytes written — the wire skips the conditional read.
            return Ok(());
        }
        <u32 as WriteJsonValue>::write_from_json(w, v)
    }
}

// 2-iter loop body in sub_1410D7480: u32 lookup + u16 lookup per entry.
py_binary_struct! {
    // S29: trailing hashed-name field in CharacterInfo tail (wire u8 + u64 + u32 + CString).
    pub struct CharHashedString<'a> {
        pub flag: u8,
        pub hash: u64,
        pub count: u32,
        pub text: CString<'a>,
    }
}

py_binary_struct! {
    // S31: overridable-flags element — {key u32, value u32} (8B).
    pub struct CharU32Pair {
        pub a: u32,
        pub b: u32,
    }
}

py_binary_struct! {
    // S32: CharacterRewardData (reader sub_101FAFA94) — 12B wire.
    pub struct CharRewardData {
        pub drop_set_info: u32,        // DropSetKey
        pub reward_tag_type_flag: u32,
        pub repeat_count: u32,
    }
}

py_binary_struct! {
    // S33: CharacterEquipmentData (reader sub_101FCFFB4 elem) — 64B wire.
    // 2 keys + 7 percent f64s (modeled u64 for guaranteed byte-exact roundtrip).
    pub struct CharacterEquipmentData {
        pub equip_item_info: u32,       // ItemKey
        pub equip_drop_set_info: u32,   // DropSetKey
        pub p0: u64, pub p1: u64, pub p2: u64,
        pub p3: u64, pub p4: u64, pub p5: u64, pub p6: u64,
    }
}

py_binary_struct! {
    pub struct CharacterActionChartEntry {
        pub group_lookup: u32,    // sub_1410FF340 wire u32
        pub package_lookup: u16,  // sub_1411003E0 wire u16
    }
}

// sub_10188F438: CArray<u64> (_aliveSkillInfoList via sub_1018BF738)
// then 4× u8 (_invincibility, _isAttackable, _isAggroTargetable, _isValid).
// Mac IDA confirms: sub_1018BF738 reads u32 count + count×u64 elements,
// followed by 4 individual u8 reads at struct offsets +16..+19.
py_binary_struct! {
    pub struct CharacterFourFlags {
        pub alive_skill_info_list: CArray<u64>,
        pub flag_a: u8,
        pub flag_b: u8,
        pub flag_c: u8,
        pub flag_d: u8,
    }
}

// sub_1411187E0 inner — 12-wire-byte items.
py_binary_struct! {
    pub struct CharacterPropEntry {
        pub lookup: u32,    // sub_141100370 wire u32 / u16 mem
        pub raw_a: u32,
        pub raw_b: u32,
    }
}

// sub_1410D7170 inner — 64-wire-byte items.
py_binary_struct! {
    pub struct CharacterMobEntry {
        pub lookup_a: u32,    // sub_1410FF5C0 wire u32
        pub lookup_b: u32,    // sub_141100370 wire u32
        pub raw_a: u64,
        pub raw_b: u64,
        pub raw_c: u64,
        pub raw_d: u64,
        pub raw_e: u64,
        pub raw_f: u64,
        pub raw_g: u64,
    }
}

// sub_141118620 inner — 20-wire-byte items.
py_binary_struct! {
    pub struct CharacterTagEntry {
        pub lookup_a: u32,    // sub_1410FF5C0 wire u32
        pub raw_a: u64,
        pub raw_b: u32,
        pub lookup_b: u32,    // sub_1410FF5C0 wire u32
    }
}

// sub_141101010 inner — 4-wire-byte items (u16 lookup + u16 raw).
py_binary_struct! {
    pub struct CharacterShortPair {
        pub lookup: u16,    // sub_141103F00 wire u16
        pub raw: u16,
    }
}

// sub_1411181F0 inner — 32 mem bytes per element.
// Wire: CArray<u32> + u32 lookup + u16 lookup + u32 lookup + u64 raw
// (per sub_1411181F0 IDA decompile).
py_binary_struct! {
    pub struct CharacterField136Entry {
        pub list: CArray<u32>,            // sub_141100510 (qword_145F113C8)
        pub lookup_a: u32,                // sub_1410FF5C0 (qword_145F0DA00)
        pub lookup_b: u16,                // sub_141100620 (qword_145F0DA20)
        pub lookup_c: u32,                // sub_1411006D0 (qword_145F0DA28)
        pub raw: u64,
    }
}

// sub_141101380 inner — 8 wire bytes per element.
// Wire: u32 lookup (sub_1410FF2D0 → qword_145F115E8) + u32 lookup
// (qword_145F1A550). Both are u32 on the wire even though mem stores
// u16 hashes after the lookup.
py_binary_struct! {
    pub struct CharacterField137Entry {
        pub lookup_a: u32,
        pub lookup_b: u32,
    }
}

// sub_141118000 inner — 24 mem bytes per element.
// Wire: u32 lookup (sub_1410FF430) + u32 lookup (read_u32_lookup_DA30)
// + CArray<u32> (sub_1410FEF40 → qword_145F0DA30).
py_binary_struct! {
    pub struct CharacterField141Entry {
        pub lookup_a: u32,
        pub lookup_b: u32,
        pub list: CArray<u32>,
    }
}

// sub_1411014B0 inner — 13 wire bytes / 12 mem bytes per element.
// Wire: u32 lookup (sub_1410FF430) + u32 lookup (sub_1410FF430) + u8 + u32.
py_binary_struct! {
    pub struct CharacterField144Entry {
        pub lookup_a: u32,
        pub lookup_b: u32,
        pub flag: u8,
        pub raw: u32,
    }
}

// sub_1410D7370 inline (NOT a CArray — single 14-wire-byte struct)
// at a2+896. Wire: 3× u32 + 2× u8.
py_binary_struct! {
    pub struct CharacterInline147 {
        pub raw_a: u32,
        pub raw_b: u32,
        pub raw_c: u32,
        pub flag_a: u8,
        pub flag_b: u8,
    }
}

// sub_141101710 inner — 32 mem bytes per element.
// Wire: u32 raw + CString-hash (sub_1410A9D40) + Vec3 + Vec3.
py_binary_struct! {
    pub struct CharacterField150Entry<'a> {
        pub raw: u32,
        pub key_hash: CString<'a>,    // sub_1410A9D40 — wire CString
        pub vec_a: [f32; 3],
        pub vec_b: [f32; 3],
    }
}

// One entry of the 5-iteration loop at a2+1032..+1112. Each iteration:
// read_u32_lookup_DA10 (u32 wire / u16 mem) + 8 raw bytes (u64).
py_binary_struct! {
    pub struct CharacterField169Entry {
        pub lookup: u32,
        pub raw: u64,
    }
}

// sub_1410FC090 — 40-byte struct, 5× u64 raw.
py_binary_struct! {
    pub struct CharacterFiveU64 {
        pub a: u64,
        pub b: u64,
        pub c: u64,
        pub d: u64,
        pub e: u64,
    }
}

// sub_141101B80 inner — 8-byte items per element.
// Wire: u32 raw + u32 raw (post-processed via sub_141BF6840).
py_binary_struct! {
    pub struct CharacterField174Entry {
        pub raw_a: u32,
        pub raw_b: u32,
    }
}

// 1.0.8: new CArray element between flag_e and the flag block.
// Replaces the old raw_e(u16) + CharacterFourFlags + post_override_list.
// 8 wire bytes per element: u32 + u32.
py_binary_struct! {
    pub struct CharacterNewListEntry {
        pub key: u32,
        pub value: u32,
    }
}

// sub_1410D6F70 — 128-byte per-element of sub_141117EC0.
// Wire: u32 + u64 + u64 + 4× u32 (loop) + u32 + 5 nested CArrays.
py_binary_struct! {
    pub struct CharacterField171Entry {
        pub raw_a: u32,
        pub raw_b: u64,
        pub raw_c: u64,
        pub block: [u32; 4],
        pub raw_d: u32,
        pub list_a: CArray<u32>,                 // sub_141100090 (qword_145F0DA68 hash)
        pub list_b: CArray<u32>,                 // sub_141100090
        pub quint_list_a: CArray<CharacterFiveU64>, // sub_1411001A0 → sub_1410FC090
        pub quint_list_b: CArray<CharacterFiveU64>, // sub_1411001A0
        pub byte_list: CArray<u8>,                // sub_1411002A0
    }
}

// sub_141B536F0 inner — 76 mem bytes, ~78 wire bytes (with empty
// strings) / 24 wire fields. Wire ORDER (not mem order):
py_binary_struct! {
    pub struct CharacterChartEntry<'a> {
        pub lookup_a: u32,             // inline u32 → qword_145F0DA38
        pub lookup_b: u32,             // inline u32 → qword_145F0DA08
        pub lookup_c: u32,             // inline u32 → qword_145F0DA00
        pub key_a: CString<'a>,        // sub_1410A9D40 wire CString
        pub key_b: CString<'a>,        // sub_1410A9D40
        // raw_block_a was [u8;16] — sub_141107700 IDA-confirmed as
        // `for i in 0..4 { read_u32() }`, so split into 4 named u32s
        // per the field-level rule (lane-c, 2026-04-30).
        pub block_a_dword_0: u32,
        pub block_a_dword_1: u32,
        pub block_a_dword_2: u32,
        pub block_a_dword_3: u32,
        pub raw_a: u16,
        pub raw_b: u32,
        pub flag_a: u8,
        pub flag_b: u8,
        pub flag_c: u8,
        pub flag_d: u8,
        pub flag_e: u8,
        pub flag_f: u8,
        pub lookup_d: u32,             // sub_1410FF430 wire u32 (inserted mid-sequence)
        pub flag_g: u8,
        pub flag_h: u8,
        pub flag_i: u8,
        pub flag_j: u8,
        pub flag_k: u8,
        pub flag_l: u8,
        pub key_c: CString<'a>,        // sub_1410A9D40
        pub flag_m: u8,
        // raw_block_b: same 4× u32 split (sub_141107700).
        pub block_b_dword_0: u32,
        pub block_b_dword_1: u32,
        pub block_b_dword_2: u32,
        pub block_b_dword_3: u32,
    }
}

// sub_141101210 inner — 20 mem bytes / 5 wire fields = 17 wire bytes.
// Wire reads in stack-stuffing order: u32 raw + u8 flag + u32 raw +
// u32 raw + u32 raw.
py_binary_struct! {
    pub struct CharacterFiveTuple {
        pub raw_a: u32,
        pub flag: u8,
        pub raw_b: u32,
        pub raw_c: u32,
        pub raw_d: u32,
    }
}

// sub_141100E90 inner — 32 mem bytes / 28 wire bytes (f32 + 3× 8 bytes).
// Same shape as faction_node_info::FactionAdjacencyMobItem.
py_binary_struct! {
    pub struct CharacterAdjacencyMobItem {
        pub raw_a: u32,
        pub raw_b: u64,
        pub raw_c: u64,
        pub raw_d: u64,
    }
}

// sub_1410D9880 inner — 96 mem bytes / 20 wire fields, CArray element of
// sub_141118980 (CharacterInfo's _hireableMercenaryList).
py_binary_struct! {
    pub struct CharacterMercenaryEntry<'a> {
        pub lookup_a: u32,                      // sub_1410FF5C0 wire u32
        pub lookup_b: u32,                      // sub_141100740 wire u32
        pub lookup_c: u32,                      // sub_1410FF340 wire u32
        pub raw_a: u32,
        pub key_str: CString<'a>,               // sub_1410A9D40 wire CString
        pub lookup_d: u32,                      // sub_1410FF340 wire u32
        pub raw_b: u32,
        pub flag_a: u8,
        pub lookup_e: u32,                      // sub_1411006D0 wire u32
        pub label: LocalizableString<'a>,
        pub raw_c: u32,
        pub flag_b: u8,
        pub raw_d: u32,
        pub raw_e: u32,
        pub flag_c: u8,
        pub lookup_f: u32,                      // sub_1411006D0 wire u32
        pub flag_d: u8,
        pub flag_e: u8,
        pub raw_f: u32,
        pub raw_g: u32,
    }
}

py_binary_struct! {
    // S41: skill-list element (reader sub_101FCBC40) — SkillKey u32 + u32 value (8B wire).
    pub struct SkillPair {
        pub skill: u32,
        pub value: u32,
    }
}

// Field 105 element `InspectData` (reader sub_101FA79A0). All key-lookups are u32 wire
// (BYREF `int`). Order = wire read order.
py_binary_struct! {
    pub struct InspectData<'a> {
        pub item_info: u32,                         // ItemKey
        pub gimmick_info: u32,                      // GimmickInfoKey
        pub character_info: u32,                    // CharacterKey
        pub spawn_reason_hash: u32,
        pub socket_name: CString<'a>,
        pub speak_character_info: u32,              // CharacterKey
        pub inspect_target_tag: u32,
        pub reward_own_knowledge: u8,
        pub reward_knowledge_info: u32,             // KnowledgeKey
        pub item_desc: LocalizableString<'a>,
        pub board_key: u32,
        pub inspect_action_type: u8,
        pub gimmick_state_name_hash: u32,
        pub target_page_index: u32,
        pub is_left_page: u8,
        pub target_page_related_knowledge_info: u32,// KnowledgeKey
        pub enable_read_after_reward: u8,
        pub refer_to_left_page_inspect_data: u8,
        pub inspect_effect_info_key: u32,           // EffectKey
        pub inspect_complete_effect_info_key: u32,  // EffectKey
    }
}

// Field 125 element `CharacterRewardData` (reader sub_101FAFA94): DropSetKey u32 +
// rewardTagTypeFlag u32 + repeatCount u32 (12B wire).
py_binary_struct! {
    pub struct CharacterRewardData {
        pub drop_set_info: u32,
        pub reward_tag_type_flag: u32,
        pub repeat_count: u32,
    }
}

// Field 128 element reuses the existing `CharacterEquipmentData` (defined above) —
// confirmed identical shape (u32 + u32 + 7× u64 = 64B) by reader sub_101FAF024.

// Field 129 element `MinigameSeed` (reader sub_101874E7C): ItemKey u32 + 3× u64 (28B wire).
py_binary_struct! {
    pub struct MinigameSeed {
        pub item_info: u32,
        pub val_a: u64,
        pub val_b: u64,
        pub val_c: u64,
    }
}

// Fields 130/131 element `PriceListEntry` (reader sub_101FD3DA4 → PriceFloor sub_101F98F9C):
// ItemKey u32 + PriceFloor{price u64, symNo u32, itemInfoWrapper u32} = 20B wire.
py_binary_struct! {
    pub struct PriceListEntry {
        pub item_info: u32,
        pub price: u64,
        pub sym_no: u32,
        pub item_info_wrapper: u32,
    }
}

// Field 138/140 element `DockingChildData` (reader sub_101E03750). Wire = read order
// (mem offsets are scattered; only read order matters for byte-exactness).
py_binary_struct! {
    pub struct DockingChildData<'a> {
        pub gimmick_info_key: u32,                   // GimmickInfoKey
        pub character_key: u32,                      // CharacterKey
        pub item_key: u32,                           // ItemKey
        pub attach_parent_socket_name: CString<'a>,
        pub attach_child_socket_name: CString<'a>,
        pub docking_tag_name_hash: u32,
        pub docking_tag_hash_b: u32,
        pub docking_tag_hash_c: u32,
        pub docking_tag_hash_d: u32,
        pub docking_equip_slot_no: u16,              // sub_100D39218 (2B)
        pub spawn_distance_level: u32,               // sub_100D39258
        pub is_item_equip_docking_gimmick: u8,
        pub send_damage_to_parent: u8,
        pub is_body_part: u8,
        pub docking_type: u8,                        // sub_101A732D4 (1B)
        pub is_summoner_team: u8,
        pub is_player_only: u8,
        pub is_npc_only_condition: u32,              // sub_1013631B4 ConditionKey u32 wire
        pub is_sync_break_parent: u8,
        pub hit_part: u8,
        pub detected_by_npc: u8,
        pub is_bag_docking: u8,
        pub enable_collision: u8,
        pub disable_collision_with_other_gimmick: u8,
        pub docking_slot_key: CString<'a>,
        pub inherit_summoner: u8,
        pub summon_tag_hash_a: u32,                  // sub_101348BF4 = 4× u32
        pub summon_tag_hash_b: u32,
        pub summon_tag_hash_c: u32,
        pub summon_tag_hash_d: u32,
    }
}

// Field 144 element `CharacterFriendlyItemData` (reader sub_101FAF58C): nested
// DropSet CArray + 3 key u32 + u64 reward.
py_binary_struct! {
    pub struct CharacterFriendlyItemData {
        pub friendly_item_reward_drop_set_info_list: CArray<u32>, // DropSetKey list
        pub item_info_to_deliver: u32,        // ItemKey
        pub item_group_info_to_deliver: u16,  // ItemGroupKey (sub_101600BC0 reads 2B!)
        pub knowledge_info: u32,              // KnowledgeKey
        pub reward_friendly: u64,
    }
}

// Field 146 element `AiDialogOverride` (reader sub_101FE7FE4): AIDialogTypeKey u32 +
// AIDialogStringKey u32 (manager <...,unsigned int>).
py_binary_struct! {
    pub struct AiDialogOverride {
        pub dialog_type: u32,
        pub dialog_string: u32,
    }
}

// Field 147 `_trapFoodData` (reader sub_101FAED78): like-food ItemKey list + 2× u64.
py_binary_struct! {
    pub struct TrapFoodData {
        pub like_food_info_list: CArray<u32>, // ItemKey list (sub_101727A70)
        pub default_chance_rate: u64,
        pub like_food_append_rate: u64,
    }
}

// Fields 181/182 element `StatBalance` (sub_101FE88B8): StatusKey u32 (sub_1016154FC=4B) + u64.
py_binary_struct! {
    pub struct StatBalance {
        pub status_key: u32,
        pub value: u64,
    }
}
// Fields 181/182 `_applyStatBalaceData`/`_applyMaxStatBalaceData` (sub_101FB185C): 5× StatBalance.
py_binary_struct! {
    pub struct ApplyStatBalance {
        pub a: StatBalance,
        pub b: StatBalance,
        pub c: StatBalance,
        pub d: StatBalance,
        pub e: StatBalance,
    }
}
// Field 184 stat-list element `DataDefinedDefaultStatData` (sub_101F96718): 5× u64.
py_binary_struct! {
    pub struct StatEntry40 {
        pub max_stat: u64,
        pub min_stat: u64,
        pub regen_stat: u64,
        pub initial_stat: u64,
        pub red_zone_stat: u64,
    }
}
// Field 184 element `CharacterLevelData` (sub_101FAEE38), wire read order.
py_binary_struct! {
    pub struct CharacterLevelData {
        pub level: u32,
        pub experience: u64,
        pub drop_experience: u64,
        pub stat_data_level_a: u32,
        pub stat_data_level_b: u32,
        pub stat_data_level_c: u32,
        pub stat_data_level_d: u32,
        pub frame_event_attr_group_info_name: u32,
        pub learn_skill_list: CArray<u32>,            // SkillKey
        pub hidden_skill_list: CArray<u32>,           // SkillKey
        pub stat_list_data_defined_static: CArray<StatEntry40>,
        pub stat_list_data_defined_regenerate: CArray<StatEntry40>,
        pub stat_list_static_stat_level: CArray<u8>,
        pub show_level_up_ui: u8,
        pub show_exp_up_ui: u8,
    }
}
// Field 187 element `ElementalMaterialInfo` (sub_101FBC6E4): StatusKey u32 + ElementalMaterialKey u32.
py_binary_struct! {
    pub struct ElementalMaterialInfo {
        pub status_key: u32,
        pub elemental_material_key: u32,
    }
}

// Field 169 `_catchSpawnData` (sub_101FAFC28): hash u32 + GimmickInfoKey u32 + DropSetKey u32.
py_binary_struct! {
    pub struct CatchSpawnData {
        pub catch_preset_name_hash: u32,
        pub gimmick_info: u32,
        pub catch_drop_set_info: u32,
    }
}
// Field 173 `_gameDifficultyBuffLevelList` (sub_101FB180C): 3× u32 (sub_100D39258).
py_binary_struct! {
    pub struct GameDifficultyBuffLevel {
        pub a: u32,
        pub b: u32,
        pub c: u32,
    }
}
// Field 177 element `BuffOverlayColorEntry` (sub_101FE86DC map): StringInfoKey u32 + color u32 + ratio u32.
py_binary_struct! {
    pub struct BuffOverlayColorEntry {
        pub key: u32,
        pub color: u32,
        pub ratio: u32,
    }
}

// Field 161 `_bulletItem` (sub_10137866C): u32 (sub_101611BA4=4B) + ItemKey u32.
py_binary_struct! {
    pub struct BulletItem {
        pub bullet_item_group: u32,
        pub bullet_item: u32,
    }
}
// Field 164 `_campGuestData` (sub_101FAF2F0): u8 isValid + visit-tag u32 list (sub_10131FF98).
py_binary_struct! {
    pub struct CampGuestData {
        pub is_valid: u8,
        pub visit_tag_list: CArray<u32>,
    }
}

// Field 151 element (sub_101FAF870): ConditionKey u32 + StringInfoKey u32 (sub_1015FBC3C=4B)
// + random parts StringInfoKey list.
py_binary_struct! {
    pub struct CharacterAdditionalPartsData {
        pub parts_condition: u32,
        pub parts_file_name: u32,
        pub random_parts_name_list: CArray<u32>,
    }
}
// Field 154 element (sub_1017A1A24): 2× ConditionKey u32 + u8 + u32.
py_binary_struct! {
    pub struct DetectReactionOverride {
        pub condition_a: u32,
        pub condition_b: u32,
        pub flag: u8,
        pub value: u32,
    }
}
// Field 156 `_gamePlayObjectShareData` (sub_101FA1998): 3× key-color u32 (sub_100D3A1C8=4B) + 2 u8.
py_binary_struct! {
    pub struct GamePlayObjectShareData {
        pub main_key_color: u32,
        pub projectile_key_color: u32,
        pub summon_key_color: u32,
        pub use_key_color: u8,
        pub is_kill_jammed_target: u8,
    }
}
// Field 159 element (sub_10137860C): EffectKey u32 + CString socket + 2× Vec3.
py_binary_struct! {
    pub struct WeakPointEffectData<'a> {
        pub effect_key: u32,
        pub socket_name: CString<'a>,
        pub vec_a: [f32; 3],
        pub vec_b: [f32; 3],
    }
}
// Field 160 `_miniGameParam` (sub_1013786EC): min/max (sub_100D392D8=4B).
py_binary_struct! {
    pub struct MiniGameParam {
        pub min: u32,
        pub max: u32,
    }
}

// Field 139 element `DockingChildEvent` (reader sub_101E03D04): wire read order =
// u32 @0, u8 @12 (sub_101A732E4), u32 @8, u32 @4, u32 @16 (17B wire).
py_binary_struct! {
    pub struct DockingChildEvent {
        pub e0: u32,
        pub e1: u8,
        pub e2: u32,
        pub e3: u32,
        pub e4: u32,
    }
}

// Field 107 `_visioningData` (reader sub_10138917C): u8 type; ONLY if type==0 a u32
// EffectKey follows (BYREF int). Value-dependent → manual impl.
#[derive(Debug)]
pub struct VisioningData {
    pub visioning_type: u8,
    pub effect_info: Option<u32>,
}
impl<'a> BinaryRead<'a> for VisioningData {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let t = u8::read_from(data, offset)?;
        let effect_info = if t == 0 { Some(u32::read_from(data, offset)?) } else { None };
        Ok(Self { visioning_type: t, effect_info })
    }
}
impl BinaryWrite for VisioningData {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.visioning_type.write_to(w)?;
        if let Some(e) = self.effect_info { e.write_to(w)?; }
        Ok(())
    }
}
impl ToJsonValue for VisioningData {
    fn to_json_value(&self) -> Value {
        let mut m = serde_json::Map::new();
        m.insert("visioning_type".into(), self.visioning_type.to_json_value());
        m.insert("effect_info".into(), match self.effect_info { Some(e) => e.to_json_value(), None => Value::Null });
        Value::Object(m)
    }
}
impl WriteJsonValue for VisioningData {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "VisioningData: expected object"))?;
        let t = obj.get("visioning_type").and_then(|x| x.as_u64()).unwrap_or(0) as u8;
        (t as u8).write_to(&mut *w)?;
        if t == 0 {
            let e = obj.get("effect_info").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
            e.write_to(w)?;
        }
        Ok(())
    }
}
impl<'a> BinaryReadTracked<'a> for VisioningData {
    fn read_tracked(data: &'a [u8], offset: &mut usize, path: &mut String, ranges: &mut Vec<FieldRange>) -> io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        ranges.push(FieldRange { path: path.clone(), start, end: *offset, ty: "VisioningData" });
        Ok(item)
    }
}

pabgh_typed_blob_table! {
    // S41 (1.12): fully wire-RE'd via charinfo_refparser.py (Mac deser sub_101FAFE34).
    // All 7089 records walk byte-exact through interaction_info_list (idx93).
    // overridable_flags structs inlined (CArray<SkillPair> + 4 u8). Field names f<N>
    // are generic for un-semantically-named reads; widths are wire-accurate.
    pub struct CharacterInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub name: LocalizableString<'a>,
        pub desc: LocalizableString<'a>,
        // Wire fields 5–26: canonical PA names (from Korean error strings in
        // CrimsonDesert.exe, mirrored in field_aliases_v3_1.rs). The 1.12 RE
        // regenerated these as generic f<N>; restored here byte-neutrally so the
        // app's transmog (appearance_name/skeleton_name/lookup_22/24/25 +
        // character_prefab_path) and Mounts-Everywhere (call_mercenary_*) keep
        // working. Wire order/widths unchanged — pure rename.
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
        pub f16_cv0_char: u32,
        pub f16_cv0_group: u16,
        pub f16_cv1_char: u32,
        pub f16_cv1_group: u16,
        pub character_game_play_data_name: u32,
        pub appearance_name: u32,
        pub character_prefab_path: u32,
        pub skeleton_name: u32,
        pub lookup_22: u32,
        pub lookup_23: u32,
        pub lookup_24: u32,
        pub lookup_25: u32,
        pub f25: u32,
        pub f26: u32,
        pub f27: u32,
        pub f28: u32,
        pub f29: u32,
        pub f30: u32,
        pub f31: u32,
        pub f32: u32,
        pub f33: u32,
        pub f34: u8,
        pub f35: u8,
        pub f36: u8,
        pub personality_info: u16,
        pub f37: u8,
        pub f38: LocalizableString<'a>,
        pub f39: LocalizableString<'a>,
        pub f40: u32,
        pub f41: u8,
        pub f42: u16,
        pub of_origin_ally_group: u32,
        pub of_origin_detect: u16,
        pub of_origin_skills: CArray<SkillPair>,
        pub of_origin_invincibility: u8,
        pub of_origin_is_attackable: u8,
        pub of_origin_is_aggro_targetable: u8,
        pub of_origin_is_valid: u8,
        pub of_mercenary_ally_group: u32,
        pub of_mercenary_detect: u16,
        pub of_mercenary_skills: CArray<SkillPair>,
        pub of_mercenary_invincibility: u8,
        pub of_mercenary_is_attackable: u8,
        pub of_mercenary_is_aggro_targetable: u8,
        pub of_mercenary_is_valid: u8,
        pub f45: u8,
        pub f46: u8,
        pub f47: u8,
        pub f48: u8,
        pub f49: u8,
        pub f50: u8,
        pub f51: u8,
        pub f52: u8,
        pub f53: u8,
        pub f54: u8,
        pub f55: u8,
        pub f56: u8,
        pub f57: u8,
        pub f58: u8,
        pub f59: u8,
        pub f61: u8,
        pub f62: u8,
        pub f63: u8,
        pub f64: u8,
        pub f65: u8,
        pub f66: u8,
        pub f67: u8,
        pub f68: u8,
        pub f69: u8,
        pub f70: u8,
        pub f71: u8,
        pub f72: u8,
        pub f73: u8,
        pub f74: u8,
        pub f75: u8,
        pub f76: u8,
        pub f77: u8,
        pub f78: u8,
        pub f79: u8,
        pub f80: u8,
        pub f81: u8,
        pub f82: u8,
        pub f83: u8,
        pub f84: u8,
        pub f85: u8,
        pub f86: u32,
        pub f87: u32,
        pub f88: u32,
        pub f89_skills: CArray<SkillPair>,
        pub f90_skills: CArray<SkillPair>,
        pub f91_skills: CArray<SkillPair>,
        pub f92_skills: CArray<SkillPair>,
        pub interaction_info_list: CArray<u32>,
        // === Fields 95-104 (authoritative map CHARINFO_FIELDMAP.json, byte-walk validated) ===
        pub interaction_distance: u32,               // 95 f32 (2.0) — sub_100D392D8
        pub default_action_action_index: u32,        // 96 sub_101623904
        pub default_share_value_index: CArray<u32>,  // 97 sub_101A611A8
        pub character_weight: u32,                   // 98 f32 — sub_100D392D8
        pub battle_order_type: u8,                   // 99 sub_101378554
        pub character_type: u8,                      // 100 sub_101378504
        pub ui_map_texture_info: u32,                // 101 lookup — sub_101F766DC
        pub map_icon_display_type: u8,               // 102 sub_101378564
        pub knowledge_info: u32,                     // 103 KnowledgeKey — sub_100DCD97C
        pub knowledge_obtain_type: u8,               // 104 sub_101A733C4
        pub inspect_data_list: CArray<InspectData<'a>>, // 105 sub_101FD756C
        pub character_group_info_list: CArray<u16>,  // 106 CharacterGroupKey u16 — sub_101FDABC0
        pub visioning_data: VisioningData,           // 107 conditional — sub_10138917C
        pub max_aggro_count: u16,                    // 109 sub_100D39238
        pub personality_type: u8,                    // 110 sub_101A72E34
        pub character_tier: u8,                      // 111 sub_101378480
        pub character_region_info_list: CArray<u16>, // 112 RegionKey u16 — sub_101638358
        pub character_age: u8,                       // 113 sub INLINE
        pub character_weapon_type: CString<'a>,      // 114 sub_100D39448
        pub dialog_voice_info: u16,                  // 115 DialogVoiceKey — sub_101D7C614
        pub interaction_category_group_info: u16,    // 116 CategoryGroupKey — sub_101FB1524
        pub detect_reaction_info: u32,               // 117 DetectReactionKey — sub_101EA4114
        pub character_pause_type: u32,               // 119 sub_101A730E4 (INLINE 4)
        pub owner_follow_type: u8,                   // 120 sub_101378514
        pub farm_drop_info_list: CArray<u32>,        // 121 DropSetKey — sub_10185DCAC
        pub farm_breeding_target_list: CArray<u32>,  // 122 sub_101727770
        pub farm_breeding_result_list: CArray<u32>,  // 123 sub_101727770
        pub farm_breeding_cool_time: u32,            // 124 sub_100D39278
        pub character_reward_data_list: CArray<CharacterRewardData>, // 125 sub_101FE71E4
        pub is_reward_drop_roll_by_create_actor: u8, // 126 sub_100D391B8
        pub mercenary_drop_info_list: CArray<u32>,   // 127 DropSetKey — sub_10185DCAC
        pub equip_item_info_list: CArray<CharacterEquipmentData>, // 128 sub_101FCFFB4
        pub minigame_seed_list: CArray<MinigameSeed>, // 129 sub_101FB4980
        pub price_list: CArray<PriceListEntry>,       // 130 sub_101FD3B98
        pub wanted_price_list: CArray<PriceListEntry>,// 131 sub_101FD3B98
        pub terrain_region_auto_spawn_info: u32,      // 132 u32 key — sub_101FB161C
        pub terrain_region_spawn_per_count: u32,      // 133 sub_100D39278
        pub convert_item_info: u32,                   // 134 ItemKey — sub_100DCD78C
        pub path_trail_type: u8,                      // 135 sub_101378420
        pub inventory_info_list: CArray<u32>,         // 136 sub_101FE73F0
        pub path_find_table_name: u32,                // 137 sub_100D39278
        pub docking_child_data_list: CArray<DockingChildData<'a>>, // 138 sub_101FE76A8
        pub docking_child_event_list: CArray<DockingChildEvent>, // 139 sub_101FE79A0
        // 140 — game v16: was an unconditional DockingChildData; the engine now
        // writes a u8 presence flag and only emits the body when set. The
        // element's old trailing `has_bag_docking_data` byte is gone with it
        // (docking_child_data_list elements shrank by 1 each), which is why the
        // record delta is exactly `-len(docking_child_data_list) + 19`.
        pub bag_docking_data: COptional<DockingChildData<'a>>,
        pub character_interaction_override_data_list: GimmickInteractionOverrideCArray<'a>, // 141 sub_101AB3118
        pub character_collision_type: u8,            // 142 sub_101378594
        pub bump_type_hash: u32,                     // 143 sub_100D39278
        pub character_friendly_item_data_list: CArray<CharacterFriendlyItemData>, // 144 sub_101FE7B98
        pub character_threat_dialog_info: u32,       // 145 AIDialogStringKey — sub_101FB1714
        pub ai_dialog_override_list: CArray<AiDialogOverride>, // 146 sub_101FE7E28
        pub trap_food_data: TrapFoodData,            // 147 sub_101FAED78
        pub weather_weight: u32,                     // 148 sub_100D392D8 (f32)
        pub use_hide_camera_overlap: u8,             // 149 sub_100D391B8
        pub force_field_target_type: u8,             // 150 sub_101A733B4
        pub additional_parts_data_list: CArray<CharacterAdditionalPartsData>, // 151 sub_101FE81D0
        pub attack_by_collision_info_list_key: u32,  // 152 sub_100D39278
        pub interaction_ui_distance_lv: u8,          // 153 INLINE
        pub detect_reaction_override_list: CArray<DetectReactionOverride>, // 154 sub_101FE8334
        pub stage_info_for_npc_shop_list: CArray<u32>, // 155 sub_101D905F4
        pub game_play_object_share_data: GamePlayObjectShareData, // 156 sub_101FA1998
        pub character_scale: u32,                    // 157 sub_100D392D8 (f32)
        pub breakable_object_info: u16,              // 158 BreakableObjectInfoKey — sub_101FA17A8
        pub weak_point_effect_data_list: CArray<WeakPointEffectData<'a>>, // 159 sub_101FD8074
        pub mini_game_param: MiniGameParam,          // 160 sub_1013786EC
        pub bullet_item: BulletItem,                 // 161 sub_10137866C
        pub job_info: u16,                           // 162 JobKey — sub_101F8B5DC
        pub call_vehicle_gimmick_info: u32,          // 163 GimmickInfoKey — sub_100DCDC64
        pub camp_guest_data: CampGuestData,          // 164 sub_101FAF2F0
        pub talk_tree_info: u16,                     // 165 TalkTreeKey — sub_101F89C3C
        pub base_material_key_override: u32,         // 166 sub_100D39278
        pub armor_material_key_override: u32,        // game v16 — _armorMaterialKeyOverride (sub_100DDB838, 4B)
        pub is_farm_animal: u8,                      // 167 sub_100D391B8
        pub use_level_inheritance: u8,               // 168 sub_100D391B8
        pub catch_spawn_data: CatchSpawnData,        // 169 sub_101FAFC28
        pub grown_target_key_list: CArray<u32>,      // 170 CharacterKey — sub_101FE852C
        pub grown_level: u32,                        // 171 sub_100D39258
        pub default_friendly_value: u64,             // 172 sub_100D39298
        pub game_difficulty_buff_level_list: GameDifficultyBuffLevel, // 173 sub_101FB180C
        pub game_difficulty_buff_info: u32,          // 174 BuffKey — sub_101F96C40 (inner reads 4B!)
        // game 2.00.00 — the Korean field oracle puts _skillPointOwnerType here,
        // immediately after _gameDifficultyBuffInfo. ONE field, but at index 173 of
        // 191, so everything after it shifted and the whole table fell to 100%
        // blob fallback (2.4% -> 100%). u8, matching knowledge_group_info, which
        // gained the same field and is small enough to pin the width on.
        pub skill_point_owner_type: u8,
        pub empowered_overlay_color: u32,            // 175 sub_100D3A1C8
        pub empowered_overlay_color_ratio: u32,      // 176 sub_100D392D8
        pub buff_overlay_color_data_map: CArray<BuffOverlayColorEntry>, // 177 sub_101FE86DC
        pub wall_hit_rebound: u8,                    // 178 sub_100D391B8
        // game v16 — four new fields between _wallHitRebound and
        // _balanceDifficultyLevel (Korean field oracle + reader widths):
        pub activity_preset_info: u32,               // _activityPresetInfo   sub_101FEA2BC (int)
        pub daily_routine_info: u32,                 // _dailyRoutineInfo     sub_1020001F0 (int)
        // ── 1.18.00: `_zoneInfo` (u16 ZoneKey) was REMOVED and replaced here by
        // `_gamePlayTriggerZone`, a CArray<u32>. The whole `zoneinfo` TABLE was
        // deleted in 1.18 — which is also why `zoneinfo.pabgb` disappeared from
        // the fixture set — so the key it referenced no longer exists.
        //
        // Proven by arithmetic over all 7,226 shared records: every per-record
        // delta fits `-2 + 4 + 4n` (= 2 + 4n) exactly —
        //   +2  x1800 (empty list)   +6 x5349 (1)   +10 (2)
        //   +34 (8)   +50 (12)   +70 (17)
        // i.e. lose the u16, gain a count plus n u32 entries. No other shape fits.
        //
        // ⚠ This one break made EVERY characterinfo record fall to blob-fallback
        // on 1.18 (7226/7226 opaque, vs 232 on 1.17), which silently killed every
        // characterinfo mod — Random Boss Encounters Reborn applied 4 of 1061
        // intents. Byte round-trip stayed true throughout, so nothing corrupted;
        // the records just stopped being addressable.
        pub game_play_trigger_zone: CArray<u32>,     // _gamePlayTriggerZone  (1.18)
        pub sub_zone_tag_list: CArray<u32>,          // _subZoneTagList       sub_10111CDFC (count + 4*n)
        pub balance_difficulty_level: u32,           // 179 sub_100D39258
        pub is_apply_stat_control_data: u8,          // 180 sub_100D391B8
        pub apply_stat_balace_data: ApplyStatBalance,     // 181 sub_101FB185C
        pub apply_max_stat_balace_data: ApplyStatBalance, // 182 sub_101FB185C
        pub status_group_info: u32,                  // 183 StatusGroupKey — sub_101F9AA64 (inner 4B)
        pub character_level_data_list: CArray<CharacterLevelData>, // 184 sub_101FE89BC
        pub detectable_gimmick_tag_name_hash_list: CArray<u32>,    // 185 sub_10131FF98
        pub mercenary_detectable_gimmick_tag_hash_list: CArray<u32>, // 186 sub_10131FF98
        pub elemental_material_info_list: CArray<ElementalMaterialInfo>, // 187 sub_101FBC528
        // ✅ ALL 187 FIELDS TYPED.
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("characterinfo.pabgb") }

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP: fixture not found"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP: pabgh not found"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        let mut max_tail = 0usize;
        let mut nonempty_tails = 0usize;
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            let item = CharacterInfo::read_with_size(&data, &mut c, e - s).unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er));
            assert_eq!(c, *e);
            let t = item.tail_blob.len();
            if t > 0 { nonempty_tails += 1; max_tail = max_tail.max(t); }
            items.push(item);
        }
        eprintln!("characterinfo: {} entries, {} nonempty tails (max={} bytes)",
            ranges.len(), nonempty_tails, max_tail);
        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "characterinfo roundtrip mismatch");
    }

    /// Diagnostic: name the field a failing record dies on. `read_tracked_with_size`
    /// fills `ranges` as it walks, and the caller keeps them when it returns Err,
    /// so the tail of that vec is the last field read before the failure.
    #[test]
    #[ignore]
    fn diag_first_failure() {
        let Ok(data) = std::fs::read(pabgb_path()) else { return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { return; };
        let ranges_all = entry_ranges(&entries, data.len());
        let mut shown = 0;
        for (i, (k, s, e)) in ranges_all.iter().enumerate() {
            let mut c = *s;
            if CharacterInfo::read_with_size(&data, &mut c, e - s).is_ok() { continue; }
            let mut cur = *s;
            let mut path = String::new();
            let mut fr = Vec::new();
            let err = CharacterInfo::read_tracked_with_size(&data, &mut cur, e - s, &mut path, &mut fr)
                .err().map(|x| x.to_string()).unwrap_or_default();
            eprintln!("--- e{} k=0x{:x} size={} err={}", i, k, e - s, err);
            for r in fr.iter().rev().take(12).rev() {
                eprintln!("      {:>6}..{:<6} {}", r.start, r.end, r.path);
            }
            shown += 1;
            if shown >= 3 { break; }
        }
        let mut fails = 0usize;
        let mut first = None;
        for (i, (k, s, e)) in ranges_all.iter().enumerate() {
            let mut c = *s;
            if CharacterInfo::read_with_size(&data, &mut c, e - s).is_err() {
                fails += 1;
                if first.is_none() { first = Some((i, *k)); }
            }
        }
        eprintln!("(showed {} failing records) TOTAL {} / {} fail; first={:?}",
            shown, fails, ranges_all.len(), first);
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP: fixture not found"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP: pabgh not found"); return; };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = CharacterInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            // 1.0.8: tail blob expected — sub-structs after short_pair_list
            // changed and are not yet fully typed.
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
