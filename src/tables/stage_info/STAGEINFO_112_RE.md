# StageInfo 1.12 wire layout — RE from Mac binary (CrimsonDesert_Steam, 1.12.02)

## ✅ SOLVED (2026-06-23) — 51441/51441 typed, byte-exact roundtrip
Fix: **truncated `StageInfo` to the IDA-verified stable prefix (fields 1-14) +
`tail_blob`** in `info.rs`. Fields 1-14 are byte-identical 1.06↔1.12; everything from
field 15 (the 1.08+ additions) is captured verbatim by the tail. Result on LIVE 1.12.02
stageinfo.pabgb: `total=51441 typed=51441 fallback=0`, pabgb+pabgh roundtrip byte-exact,
AND a typed edit of `sequencer_desc.position` (the funcnpc world coords) persists +
re-serializes byte-stable. `sequencer_desc` (field 7) exposes scene name + prefab +
position — the placement data the additive-NPC tool needs. Built from dmm-parser-src via
`cargo build --release` (PYO3_PYTHON=3.14), pyd copied to crimson_rs/ + dmm_parser/.
To type more 1.12 fields, extend the struct from field 15 using the map below (read order
is reliable; resolve element wire-sizes via roundtrip iteration).

---


Reader function: **`sub_101F78FF8`** (size 0xa70). Found via xrefs from the Korean
field error strings `"StageInfo의 _X를 읽어들이는데 실패했다."` (0x1075ce6d7…0x1075cfe2e,
sequential in field order). 89 fields total. The 1.06-era parser (`info.rs`) modeled a
MUCH smaller struct — StageInfo grew massively by 1.08 (which is why all 51441 records
blob-fall-back on 1.08/1.10/1.12; 1.06 parsed 50717/50789).

## Authoritative field order (89, from error-string address order)
1 _key · 2 _stringKey · 3 _isBlocked · 4 _name · 5 _stageDesc · 6 _completeLog ·
7 _sequencerDesc · 8 _spawnFactionSpawnDataInfo · 9 _spawnFactionNodeInfo ·
10 _disableFactionSpawnPartyNameHashList · 11 _stageCategory · 12 _closeFilter ·
13 _closeFilterByGroup · 14 _globalFilterCharacterList · 15 _questType · 16 _stageDataType ·
17 _parentQuest · 18 _parentStage · 19 _ownerMissionInfo · 20 _childStageList ·
21 _executorMissionList · 22 _executorStageList · 23 _executeTargetStageList ·
24 **_logoutMercenaryGroupInfoList (NEW vs 1.06)** · 25 _hideMercenaryGroupInfoList ·
26 _playCondition · 27 _closeCondition · 28 _fieldInfo · 29 _startPlayerList ·
30 _forbiddenCharacterList · 31 _rematchStageDesc · 32 _platformCharacter ·
33 _platformDockingTagHash · 34 _platformSocketName · 35 _isIgnoreDistance ·
36 _isFactionSequencer · 37 _factionSequencerSpawnTagHash · 38 _resetSecond ·
39 _randomSpawnCount · 40 _randomPercent · 41 _randomRepeatTime · 42 _completeCount ·
43 _subTimelineBreakDescList · 44 _scheduleCompleteCondition ·
45 _scheduleStageCompleteAIEventList · 46 _itemConditionAndRemoveArray ·
47 _rewardDropSetInfoList · 48 _levelName · 49 _globalEffectData · 50 _guideEffectName ·
51 _fieldReviveInfo · 52 _stageIconPath · 53 _stageTextIconPath · 54 _stageImagePath ·
55 _completeImagePath · 56 _npcShopCharacterInfo · 57 _closeDialogSpeakerCharacter ·
58 _closeDialogString · 59 _closeDialogSoundEventName · 60 _updatePriority ·
61 _completeAlertType · 62 _stageKnowledge · 63 _stageGameEventDataList ·
64 _spawnBlockTypeFlag · 65 _weatherInfo · 66 _gameLevelInfoForValidation ·
67 _gameLevelDataNameForValidation · 68 _weatherStartBlendTime · 69 _weatherEndBlendTime ·
70 _weatherIngTime · 71 _beginTime · 72 _endTime · 73 _changeTime · 74 _useCommute ·
75 _showStageIcon · 76 _isSave · 77 _saveSchedule · 78 _hasDynamicActor ·
79 _isForceSpawnAfterRetreat · 80 _isForceSpawnNearDistance · 81 _isForceSpawnAllActor ·
82 _disableGiveUp · 83 _reviveInPlaceHardDifficulty · 84 _evadeProjectile ·
85 _followParentReaction · 86 _isPlayableOnWanted · 87 _allowAccompany ·
88 _useRevivePointForDead · 89 _ignoreFactionClose

## Reader-sub → wire type (decoded so far)
- `sub_100D391B8` = **u8** (`reader(a1,a2,1)`); `sub_100D39278` = **u32** (`,4`); `sub_100D392B8` = **u64** (`,8`)
- `sub_100D39448` = **CString** (_stringKey); `sub_100D5D6D8` = **LocalizableString** (_name/_stageDesc/_completeLog…)
- `sub_1017AB1D4` = **SequencerStageChartDesc** (mem 120→352 = 232 bytes; old parser already has this type)
- Key-lookups, all **u32 wire → u16/u32 mem** (read u32 key, hash-lookup an index):
  `sub_101F79A68`=FactionSpawnDataKey · `sub_101D84EC8`=FactionNodeKey · `sub_101F79B60`=QuestKey(u32 mem) ·
  `sub_101D7C8FC`=MissionKey · `sub_100D95C18`=StageKey(u32 mem) · `sub_1013631B4`=ConditionKey ·
  `sub_1013633A4`=StringInfoKey · `sub_100DCD884`=CharacterKey · `sub_101F79C58`=FieldInfoKey
- CArray builders (u32 count + count×element):
  `sub_101727770`=CArray\<CharacterKey u32\> · `sub_101D905F4`=CArray\<StageKey u32\> ·
  `sub_101FB6CCC`=CArray\<MercenaryGroupKey u16-mem, u32? wire\> · `sub_10131FF98`=CArray\<u32\> (disable list)
- Nested struct readers (own error strings, own sub-fields):
  `sub_101F78F38` = **RematchStageDesc** {_startSubTimelineName CString, _endSubTimelineNameList, _stageInfoList CArray\<StageKey\>}
- `sub_100D395EC` = CString variant (binarystring build)

## STATUS / remaining
- Field↔reader pairing is ~exact for fields 1-15; a few middle guards (multi-line in the
  decompile) still need precise pairing — guard count comes out 87/89 with the naive regex,
  causing ±2 drift past the middle. Need to pin those by xref'ing each error string's code
  site to the guard immediately before it.
- Still to type: `sub_101A730B4`(u8), `sub_101FB687C`, `sub_101FB6AD4`, `sub_101FB6F24`,
  `sub_101FB70E0`, `sub_101F79D50`, `sub_101F79DB0`, `sub_101FB738C`, `sub_101F79E10`,
  `sub_1017A7128`, `sub_100DCD97C`, `sub_101FB7464`, `sub_101F79F08`, `sub_101F76178`,
  `sub_100D399C0`, `sub_100D39298`, `sub_100D39238` — several are CArray-of-struct
  (`_rewardDropSetInfoList`, `_stageGameEventDataList`, `_scheduleStageCompleteAIEventList`,
  `_itemConditionAndRemoveArray`, `_fieldReviveInfo`, `_globalEffectData`,
  `_subTimelineBreakDescList`) needing their element structs RE'd.
- Then: rewrite `info.rs` struct (89 fields + new element structs), rebuild, prove byte-exact
  roundtrip on all 51441 records. This is the standard but it's a sizeable rewrite, not a patch.

## PROGRESS 2026-06-24 (B-2 loop)
- Fields 1-14: typed (truncation fix). **Fields 15-24 now ALSO typed** (restored old 15-23 = byte-identical 1.06↔1.12, + NEW field 24 `logout_mercenary_group_info_list` CArray<u8>). Validated: HerStore parent_quest=1000765 matches independent byte-walk; 51441/51441 no fallback; byte-exact; tail avg 348→242 B.
- REMAINING: fields 25-89 still in tail_blob. Next: type them in read order (the reader-sub list from sub_101F78FF8; NAME↔reader pairing drifts so trust READ ORDER + byte-walk validation, not the error-string name order). Complex element structs to RE: StageBranchData(~20B, reader sub_101F91D90), SubTimelineBreakDesc(sub_101F7A198), the schedule/reward/globalEffect CArrays. Iterate rebuild→check tail shrinks→fix over/under-read.

- 2026-06-24 iter2: **fields 25-30 typed** (hide_mercenary CArr<u8>, play_condition u32, close_condition u32, field_info u32, start_player_list CArr<u32>, forbidden_character_list CArr<u32>). fallback=0, byte-exact, HerStore tail 242→218B, field_info=1 validated. NEXT: field 31 = rematch_stage_desc (RematchStageDesc struct, reader sub_101F78F38 = {startSubTimelineName CString, endSubTimelineNameList CArray, stageInfoList CArray<StageKey>}), then 32 platform_character (CArray<CString>), 33+ . KEY LEARNING: byte-exact+no-fallback does NOT prove correctness while tail>0 (self-consistent wrong-size fields are masked); only tail=0 everywhere proves it. Validate via byte-walk + known values (parent_quest=1000765, field_info=1). Element sizes matter, not perfect names — both StageFilterEntry & StageBranchData are 19B so field 23 is structurally fine.

- 2026-06-24 iter3: tried fields 31 (rematch_stage_desc) + 32 (platform_character CArray<CString>) → 16 FALLBACKS on Platform_Sequencer train records (keys 0xf56c5 etc, string_key Delesyia_Train_Passenger_Schedule_Child*). REVERTED to clean fields 1-30 (0 fallback). RematchStageDesc struct defined (sub_101F78F38 = {start_sub_timeline_name CString, end_sub_timeline_name_list CArray<CString>, stage_info_list CArray<u32>}); for the platform record rematch IS empty (12B = 3 zero counts). BUT after rematch@12: u32=2103, u32=199818120, THEN platform CString len21 'Platform_Sequencer_00' @tail-offset20. So 8 mystery bytes (2103, 199818120) sit between rematch and the platform string → field 32 is NOT a bare CArray<CString>; likely an intermediate field (platform_character may be a CArray-of-struct, or there's a field between 31 and 32). After the platform string come 0xFFFF sentinel keys (offset 48+). NEXT: decompile the EXACT readers for fields 31-34 (the platform cluster) via the corrected guard order, OR byte-walk 0xf56c5 from field 7's known end. Until 31-32 are right, keep 1-30.

- 2026-06-24 iter4 (DRIFT RESOLVED): built the AUTHORITATIVE 89-field map (FIELDMAP_FINAL.json) by nesting-depth pairing — guards in source order (backward balanced-walk from each '& 1) != 0', comments stripped) = field order; errors source order = reverse field order; pair guard[i]↔reversed_err[i]. Fixed platform regression: field 32 _platformCharacter = **u32 CharacterKey** (sub_100DCD884), NOT CArray<CString>! Implemented fields 31-44: rematch(31), platform_character u32(32), platform_docking_tag_hash u32(33), platform_socket_name CString(34), is_ignore_distance/is_faction_sequencer u8(35-36), faction_sequencer_spawn_tag_hash u32(37), reset_second u32(38), random_spawn_count [u32;2](39), random_percent u64(40), random_repeat_time u32(41), complete_count u16(42), sub_timeline_break_desc_list CArr<SubTimelineBreakDesc{u8,3×u32}>(43), schedule_complete_condition u32(44). **fallback=0, byte-exact, tail avg 218→154B.** REMAINING fields 45-89 (per FIELDMAP_FINAL.json): mostly simple u32-keys/u8-flags, EXCEPT complex CArrays 45 _scheduleStageCompleteAIEventList(sub_101FB70E0=u32+CString elem), 46 _itemConditionAndRemoveArray(sub_101F79D50), 47 _rewardDropSetInfoList(sub_101F79DB0=4×DropSetKey-list), 49 _globalEffectData(sub_101FB738C=opt struct), 63 _stageGameEventDataList(sub_101FB7464=16B-struct). Fields 48,50-62,64-89 are simple (u32 hashes/keys + 16 u8 flags 74-89). Continue in order; decompile the 5 complex element readers, add chunks, drive tail→0.

- 2026-06-24 iter5 (NEAR-COMPLETE): implemented ALL 89 fields per FIELDMAP_FINAL.json. Structs added: ScheduleAiEvent(45), StageQuadList(46/47 placeholder), StageGlobalEffect(49). fallback=0, byte-exact. **51316/51441 records now tail=0 (fully typed, 99.76%).** ONLY 125 records still have tail (max 72B): 114 abyss-weather stages populate global_effect_data (field 49) + 11 populate stage_game_event_data_list (field 63). REMAINING FIX: (a) field 49 StageGlobalEffect inner is bigger than {COptional<u32> trigger, u16 effect_info, u32 priority, u32 blending} — the trigger volume (sub_101AF7F08, 0x58=88B mem) is likely PRESENT on these records; decompile sub_101AF7F08 for the exact trigger-volume wire. (b) field 63 element (sub_101D75F78 = u32 + u8 presence + optional via sub_101F85EAC) — currently CArray<u32> placeholder under-reads. Byte-walk an abyss weather record (e.g. cd_seq_abyss_0014_weather) + a gev63 record to nail both. Then tail=0 everywhere → stageinfo ✅ 100%.

- 2026-06-24 iter6 (COMPLETE ✅): fixed field 49 trigger volume (StageTriggerVolume = u8 + 40B transform + 2×CString + u8 + 2×Vec3 + 2×u8; all sub_101A7xxxx readers = u8) + field 63 (StageGameEventData = {u32, COptional<GameEventExecuteData{u8 type + 3× ConditionKey u32}>}). **stageinfo 100% TYPED: 51441/51441 tail=0, fallback=0, byte-exact roundtrip on live 1.12.02.** B-2 DONE.
