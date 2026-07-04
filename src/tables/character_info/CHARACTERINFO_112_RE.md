# CharacterInfo 1.12 RE (B-3)

Reader = **sub_101FAFE34** (0x15bc, 5564B). **187 fields** total (Korean error strings
0x1076114d7..0x107614a10). Authoritative field->reader map in CHARINFO_FIELDMAP.json
(nesting-depth paired, drift-proof — same method as stageinfo FIELDMAP_FINAL.json; 186
guards/187 errs so ~1-off drift past one missed guard — validate by tail-shrink).

## State 2026-06-24 (B-3 start)
- Current info.rs struct types fields 1-94 (stops at `interaction_info_list`), then tail_blob.
- LIVE 1.12: 7089 records, fallback=0, but ALL have tail (avg 3218B, max 12714B) = fields
  95-187 untyped (~93 fields, MANY complex nested lists).
- NEXT: extend struct from field 95 (_interactionDistance) in read order per the map.
  Decompile each reader-sub for wire type. Complex list readers seen: sub_101FCBA88 (skill
  lists, CArray<SkillPair>), sub_101FDABC0 (_inspectDataList), sub_10138917C
  (_characterGroupInfoList), sub_101727770 (CArray<u32>, farm lists), + ~25 more list/struct
  readers across reward/inventory/docking/farm/level/elemental data. Add chunks, rebuild,
  drive tail->0. Validate on butcher 3410.

## Field map (95+, the untyped region):
95 _interactionDistance sub_101623904
96 _defaultActionActionIndex sub_101A611A8
97 _defaultShareValueIndex sub_100D392D8
98 _characterWeight sub_101378554
99 _battleOrderType sub_101378504
100 _characterType sub_101F766DC
101 _uiMapTextureInfo sub_101378564
102 _mapIconDisplayType sub_100DCD97C
103 _knowledgeInfo sub_101A733C4
104 _knowledgeObtainType sub_101FD756C
105 _inspectDataList sub_101FDABC0
106 _characterGroupInfoList sub_10138917C
107 _visioningData sub_101FAAE1C
108 _detectInfo sub_100D39238
109 _maxAggroCount sub_101A72E34
110 _personalityType sub_101378480
111 _characterTier sub_101638358
112 _characterRegionInfoList INLINE/1
113 _characterAge sub_100D39448
114 _characterWeaponType sub_101D7C614
115 _dialogVoiceInfo sub_101FB1524
116 _interactionCategoryGroupInfo sub_101EA4114
117 _detectReactionInfo sub_101FA18A0
118 _allyGroupInfo sub_101A730E4
119 _characterPauseType sub_101378514
120 _ownerFollowType sub_10185DCAC
121 _farmDropInfoList sub_101727770
122 _farmBreedingTargetList sub_101727770
123 _farmBreedingResultList sub_100D39278
124 _farmBreedingCoolTime sub_101FE71E4
125 _characterRewardDataList sub_100D391B8
126 _isRewardDropRollByCreateActor sub_10185DCAC
127 _mercenaryDropInfoList sub_101FCFFB4
128 _equipItemInfoList sub_101FB4980
129 _minigameSeedList sub_101FD3B98
130 _priceList sub_101FD3B98
131 _wantedPriceList sub_101FB161C
132 _terrainRegionAutoSpawnInfo sub_100D39278
133 _terrainRegionSpawnPerCount sub_100DCD78C
134 _convertItemInfo sub_101378420
135 _pathTrailType sub_101FE73F0
136 _inventoryInfoList sub_100D39278
137 _pathFindTableName sub_101FE76A8
138 _dockingChildDataList sub_101FE79A0
139 _dockingChildEventList sub_101E03750
140 _bagDockingData sub_101AB3118
141 _characterInteractionOverrideDataList sub_101378594
142 _characterCollisionType sub_100D39278
143 _bumpTypeHash sub_101FE7B98
144 _characterFriendlyItemDataList sub_101FB1714
145 _characterThreatDialogInfo sub_101FE7E28
146 _aiDialogOverrideList sub_101FAED78
147 _trapFoodData sub_100D392D8
148 _weatherWeight sub_100D391B8
149 _useHideCameraOverlap sub_101A733B4
150 _forceFieldTargetType sub_101FE81D0
151 _additionalPartsDataList sub_100D39278
152 _attackByCollisionInfoListKey INLINE/1
153 _interactionUIDistanceLv sub_101FE8334
154 _detectReactionOverrideList sub_101D905F4
155 _stageInfoForNpcShopList sub_101FA1998
156 _gamePlayObjectShareData sub_100D392D8
157 _characterScale sub_101FA17A8
158 _breakableObjectInfo sub_101FD8074
159 _weakPointEffectDataList sub_1013786EC
160 _miniGameParam sub_10137866C
161 _bulletItem sub_101F8B5DC
162 _jobInfo sub_100DCDC64
163 _callVehicleGimmickInfo sub_101FAF2F0
164 _campGuestData sub_101F89C3C
165 _talkTreeInfo sub_100D39278
166 _baseMaterialKeyOverride sub_100D391B8
167 _isFarmAnimal sub_100D391B8
168 _useLevelInheritance sub_101FAFC28
169 _catchSpawnData sub_101FE852C
170 _grownTargetKeyList sub_100D39258
171 _grownLevel sub_100D39298
172 _defaultFriendlyValue sub_101FB180C
173 _gameDifficultyBuffLevelList sub_101F96C40
174 _gameDifficultyBuffInfo sub_100D3A1C8
175 _empoweredOverlayColor sub_100D392D8
176 _empoweredOverlayColorRatio sub_101FE86DC
177 _buffOverlayColorDataMap sub_100D391B8
178 _wallHitRebound sub_100D39258
179 _balanceDifficultyLevel sub_100D391B8
180 _isApplyStatControlData sub_101FB185C
181 _applyStatBalaceData sub_101FB185C
182 _applyMaxStatBalaceData sub_101F9AA64
183 _statusGroupInfo sub_101FE89BC
184 _characterLevelDataList sub_10131FF98
185 _detectableGimmickTagNameHashList sub_10131FF98
186 _mercenaryDetectableGimmickTagHashList sub_101FBC528


## 2026-06-24 iter2 — EXTRACTION FIXED + fields 95-104 typed
KEY: characterinfo uses TWO guard forms — `(subX & 1)!=0` AND `(unsigned int)subX(a1,...)`
(lookup-temp fields). Extracting only the first gave 186/187 (1-off drift). Combined
extractor (both forms, source order) → **187/187 aligned** = CHARINFO_FIELDMAP.json (corrected,
byte-walk validated on butcher 3410: f95 f32=2.0, f96 u32, f97 CArray count=2, f98=0, f99 u8=3).
Added fields 95-104: interaction_distance u32(f32), default_action_action_index u32,
default_share_value_index CArray<u32>, character_weight u32(f32), battle_order_type u8,
character_type u8, ui_map_texture_info u32, map_icon_display_type u8, knowledge_info u32,
knowledge_obtain_type u8. fallback=0, byte-exact, tail 3218→3186B avg.
NEXT: field 105 _inspectDataList = sub_101FD756C = CArray<96B struct> (element sub_101FA79A0 —
decompile for the 96-byte wire). The remaining tail BULK is in the big lists (inspect 96B,
characterGroupInfoList, reward/inventory/docking/farm) — cracking those drops the tail fast.
Continue per CHARINFO_FIELDMAP.json reader order, decompiling each complex element reader.

## 2026-06-24 iter3 — fields 105-106 (InspectData decoded)
- 105 _inspectDataList = CArray<InspectData> (reader sub_101FA79A0, 20 fields): item_info/
  gimmick_info/character_info u32 keys, spawn_reason_hash u32, socket_name CString,
  speak_character_info u32, inspect_target_tag u32, reward_own_knowledge u8, reward_knowledge_info
  u32, item_desc LocalizableString, board_key u32, inspect_action_type u8, gimmick_state_name_hash
  u32, target_page_index u32, is_left_page u8, target_page_related_knowledge_info u32,
  enable_read_after_reward u8, refer_to_left_page u8, inspect_effect/complete_effect u32.
  KEY RULE: lookup wire width = BYREF var type (int=u32, unsigned __int16=u16).
- 106 _characterGroupInfoList = CArray<u16> (CharacterGroupKey, u16 wire — sub_101FDABC0 allocs 2*count).
- fallback=0, byte-exact, tail 3186→3155B. Fields 1-106 typed.
NEXT: 107 _visioningData = sub_10138917C = CONDITIONAL {u8 type; if type==0 then u32 EffectKey} —
needs manual BinaryRead impl (not py_binary_struct). 108 _detectInfo = u16 (DetectKey, sub_101FAAE1C).
Then 109 _maxAggroCount u16, 110 _personalityType, 111 _characterTier u8, 112 _characterRegionInfoList...
per CHARINFO_FIELDMAP.json. The tail BULK is later big lists (reward/inventory/docking/level) — keep going in order.

## 2026-06-24 iter4 — fields 107-114
- 107 visioning_data = VisioningData manual impl {u8 type; if type==0 then u32 EffectKey}.
- 108 detect_info u16 (DetectKey), 109 max_aggro_count u16, 110 personality_type u8,
  111 character_tier u8, 112 character_region_info_list CArray<u16> (RegionKey),
  113 character_age u8, 114 character_weapon_type CString.
- fallback=0, byte-exact, tail 3155→3126B. Fields 1-114 typed (73 remain).
NEXT: 115 _dialogVoiceInfo (sub_101D7C614 — decompile), 116 _interactionCategoryGroupInfo
(sub_101FB1524), 117 _detectReactionInfo (sub_101EA4114), 118 _allyGroupInfo (sub_101FA18A0),
119 _characterPauseType (sub_101A730E4), 120 _ownerFollowType (sub_101378514 u8),
121-123 farm lists (sub_10185DCAC, sub_101727770×2), 124 farm_breeding_cool_time u32,
125 _characterRewardDataList (sub_101FE71E4 — BIG list, holds tail bulk), 127 _mercenaryDropInfoList,
128 _equipItemInfoList (sub_101FCFFB4). Tail bulk is 125+ (reward/equip/inventory/docking) — reaching them drops tail fast.

## 2026-06-24 iter5 — fields 115-124
115 dialog_voice_info u16, 116 interaction_category_group_info u16, 117 detect_reaction_info u32,
118 ally_group_info u32, 119 character_pause_type u32(INLINE4), 120 owner_follow_type u8,
121 farm_drop_info_list CArray<u32>(DropSetKey, sub_10185DCAC BYREF int→u32), 122/123
farm_breeding_target/result_list CArray<u32>(sub_101727770), 124 farm_breeding_cool_time u32.
fallback=0, byte-exact, tail 3126→3093B. Fields 1-124 typed (63 remain).
NEXT: 125 _characterRewardDataList = sub_101FE71E4 = BIG nested list (decompile element for wire) —
holds tail bulk. 126 is_reward_drop_roll u8, 127 _mercenaryDropInfoList CArray<u32>(sub_10185DCAC),
128 _equipItemInfoList sub_101FCFFB4 (decompile). Reward/equip/inventory/docking lists 125+ hold the bulk.

## 2026-06-24 iter6 — fields 125-128 (reward + equip lists, tail bulk cracking)
125 character_reward_data_list = CArray<CharacterRewardData{drop_set_info u32, reward_tag_type_flag u32,
repeat_count u32}> (12B, sub_101FAFA94). 126 is_reward_drop_roll_by_create_actor u8. 127
mercenary_drop_info_list CArray<u32>(DropSetKey). 128 equip_item_info_list = CArray<CharacterEquipmentData>
(64B = u32 ItemKey + u32 DropSetKey + 7× u64 percents; REUSED existing struct, sub_101FAF024 confirmed shape).
fallback=0, byte-exact, tail 3093→2906B (-187, bulk cracking). Fields 1-128 typed (59 remain).
NEXT: 129 _minigameSeedList sub_101FB4980 (CArray<32B> — decompile element), 130/131 _priceList/_wantedPriceList
sub_101FD3B98 (decompile), 132 _terrainRegionAutoSpawnInfo sub_101FB161C, 133 u32, 134 _convertItemInfo
sub_100DCD78C, 135 _pathTrailType sub_101378420(u8), 136 _inventoryInfoList sub_101FE73F0 (BIG), 137 u32,
138 _dockingChildDataList sub_101FE76A8 (BIG), 139 _dockingChildEventList sub_101FE79A0, 140 _bagDockingData
sub_101E03750, 141 _characterInteractionOverrideDataList sub_101AB3118 (already a variant import?). Keep cracking the big lists.

## 2026-06-24 iter7 — fields 129-135
129 minigame_seed_list CArray<MinigameSeed{item_info u32, 3× u64}>(28B, sub_101874E7C). 130/131
price_list/wanted_price_list CArray<PriceListEntry{item_info u32, price u64, sym_no u32,
item_info_wrapper u32}>(20B, sub_101FD3DA4→PriceFloor sub_101F98F9C) — validated on merchant chars.
132 terrain_region_auto_spawn_info u32 (manager <...,unsigned int>=u32 mem), 133
terrain_region_spawn_per_count u32, 134 convert_item_info u32(ItemKey), 135 path_trail_type u8.
fallback=0, byte-exact, tail 2906→2875B. Fields 1-135 typed (52 remain).
NEXT: 136 _inventoryInfoList sub_101FE73F0 (BIG — decompile element), 137 _pathFindTableName u32,
138 _dockingChildDataList sub_101FE76A8 (BIG), 139 _dockingChildEventList sub_101FE79A0, 140
_bagDockingData sub_101E03750, 141 _characterInteractionOverrideDataList sub_101AB3118
(GimmickInteractionOverrideCArray already imported — check reuse). Inventory/docking lists hold remaining bulk.

## 2026-06-24 iter8 — fields 136-138 (DockingChildData decoded)
136 inventory_info_list CArray<u32>(sub_101FE73F0). 137 path_find_table_name u32. 138
docking_child_data_list CArray<DockingChildData>(reader sub_101E03750, 28 fields, 76B mem; wire=read
order: 3 keys u32 + 2 CString + 4 u32 + u16(sub_100D39218) + u32(spawn_dist sub_100D39258) + 3 u8 +
u8(docking_type sub_101A732D4) + 2 u8 + u32(is_npc_only_condition sub_1013631B4) + 6 u8 + CString +
u8 + 4 u32(summon_tag sub_101348BF4) + u8). fallback=0, byte-exact, tail 2875→2838B. Fields 1-138 typed (49 remain).
READER WIDTHS: sub_100D39218=u16, sub_101A732D4=u8, sub_101348BF4=4×u32, sub_1013631B4=ConditionKey(u32 wire/u16 mem).
NEXT: 139 _dockingChildEventList sub_101FE79A0 = CArray<20B = 5× u32 in WIRE-READ order (sub_101E03D04:
@0,@12,@8,@4,@16; CONFIRM sub_101A732E4 width @12 — likely u32)>. 140 _bagDockingData = SINGLE DockingChildData
(same reader sub_101E03750, not a list). 141 _characterInteractionOverrideDataList sub_101AB3118
(GimmickInteractionOverrideCArray imported — check reuse). 142 _characterCollisionType u8, 143 u32, 144+ ...

## 2026-06-24 iter9 — fields 139-143
139 docking_child_event_list CArray<DockingChildEvent{u32,u8,u32,u32,u32}>(17B wire read-order,
sub_101E03D04; sub_101A732E4=u8). 140 bag_docking_data = SINGLE DockingChildData (reused). 141
character_interaction_override_data_list = GimmickInteractionOverrideCArray (REUSED existing import,
sub_101AB3118 — works!). 142 character_collision_type u8. 143 bump_type_hash u32.
fallback=0, byte-exact, tail 2838→2740B (-98). Fields 1-143 typed (44 remain).
NEXT: 144 _characterFriendlyItemDataList sub_101FE7B98 = CArray<32B element sub_101FAF58C (has heap
ptr/CString — decompile)>, 145 _characterThreatDialogInfo u32(AIDialogStringKey, manager unsigned int=u32),
146 _aiDialogOverrideList sub_101FE7E28, 147 _trapFoodData sub_101FAED78, 148 _weatherWeight u64(sub_100D392D8),
149 _useHideCameraOverlap u8, 150 _forceFieldTargetType sub_101A733B4, 151 _additionalPartsDataList sub_101FE81D0,
152 u32, 153 _interactionUIDistanceLv INLINE u8, 154 _detectReactionOverrideList sub_101FE8334,
155 _stageInfoForNpcShopList sub_101D905F4 (CArray<u32>), 156+. Keep cracking.

## 2026-06-24 iter10 — fields 144-150 + WIRE-WIDTH RULE CORRECTION
144 character_friendly_item_data_list CArray<CharacterFriendlyItemData{drop_set CArray<u32>,
item_info u32, item_group u16, knowledge u32, reward_friendly u64}>(sub_101FAF58C). 145
character_threat_dialog_info u32. 146 ai_dialog_override_list CArray<AiDialogOverride{u32,u32}>. 147
trap_food_data {like_food CArray<u32>, default_chance_rate u64, like_food_append_rate u64}. 148
weather_weight u32(sub_100D392D8 CONFIRMED 4B). 149 use_hide_camera_overlap u8. 150 force_field_target_type u8.
fallback=0, byte-exact, tail 2740→2563B. Fields 1-150 typed (37 remain).
**CORRECTED RULE**: wire width = the (a1,a2,N) inside the ACTUAL reader sub, NOT the caller BYREF type.
ItemGroupKey sub_101600BC0 reads (a1,&v,2) = u16 wire despite caller BYREF int (caused 5437 fallback when
typed u32). For ambiguous lookups DECOMPILE the reader sub (its inner (a1,a2,N) or (...&v,N) call gives bytes).
NEXT: 151 _additionalPartsDataList sub_101FE81D0, 152 _attackByCollisionInfoListKey u32, 153
_interactionUIDistanceLv INLINE u8, 154 _detectReactionOverrideList sub_101FE8334, 155 _stageInfoForNpcShopList
sub_101D905F4 CArray<u32>, 156 _gamePlayObjectShareData sub_101FA1998, 157 _characterScale u32, 158
_breakableObjectInfo sub_101FA17A8, 159 _weakPointEffectDataList sub_101FD8074, 160 _miniGameParam sub_1013786EC, 161+.

## 2026-06-24 iter11 — fields 151-160
151 additional_parts_data_list CArray<CharacterAdditionalPartsData{parts_condition u32, parts_file_name
u32(StringInfoKey sub_1015FBC3C=4B), random_parts_name_list CArray<u32>}>. 152 attack_by_collision_info_list_key
u32. 153 interaction_ui_distance_lv u8. 154 detect_reaction_override_list CArray<DetectReactionOverride{u32,u32,u8,u32}>.
155 stage_info_for_npc_shop_list CArray<u32>. 156 game_play_object_share_data {3×u32 colors(sub_100D3A1C8=4B), 2×u8}.
157 character_scale u32. 158 breakable_object_info u16. 159 weak_point_effect_data_list
CArray<WeakPointEffectData{effect_key u32, socket CString, 2×[f32;3]}>. 160 mini_game_param {min u32, max u32}.
fallback=0, byte-exact, tail 2563→2433B. Fields 1-160 typed (27 remain).
NEXT: 161+ per CHARINFO_FIELDMAP.json (161 _bulletItem, 162 _jobInfo, 163 _callVehicleGimmickInfo, 164
_campGuestData, 165 _talkTreeInfo, ... 187 _elementalMaterialInfoList). Continue decompiling each reader.

## 2026-06-24 iter12 — fields 161-168
161 bullet_item BulletItem{bullet_item_group u32, bullet_item u32 ItemKey}(sub_10137866C). 162 job_info
u16(JobKey). 163 call_vehicle_gimmick_info u32(GimmickInfoKey). 164 camp_guest_data CampGuestData{is_valid u8,
visit_tag_list CArray<u32>(sub_10131FF98)}. 165 talk_tree_info u16(TalkTreeKey). 166 base_material_key_override u32.
167 is_farm_animal u8. 168 use_level_inheritance u8. sub_10131FF98=CArray<u32> (also fields 185/186).
fallback=0, byte-exact, tail 2433→2406B. Fields 1-168 typed (19 remain).
NEXT (LAST 19): 169 _catchSpawnData sub_101FAFC28, 170 _grownTargetKeyList sub_101FE852C, 171 _grownLevel
u32(sub_100D39258), 172 _defaultFriendlyValue u64(sub_100D39298), 173 _gameDifficultyBuffLevelList sub_101FB180C,
174 _gameDifficultyBuffInfo sub_101F96C40, 175 _empoweredOverlayColor u32, 176 _empoweredOverlayColorRatio u32,
177 _buffOverlayColorDataMap sub_101FE86DC, 178 _wallHitRebound u8, 179 _balanceDifficultyLevel u32(sub_100D39258),
180 _isApplyStatControlData u8, 181/182 _applyStatBalaceData/_applyMaxStatBalaceData sub_101FB185C, 183
_statusGroupInfo sub_101F9AA64, 184 _characterLevelDataList sub_101FE89BC (likely BIG), 185/186
detectable tag lists CArray<u32>(sub_10131FF98), 187 _elementalMaterialInfoList sub_101FBC528. Decompile each, finish to tail=0.

## 2026-06-24 iter13 — fields 169-180 + SYSTEMIC u16/u32 AUDIT
169 catch_spawn_data CatchSpawnData{3×u32}. 170 grown_target_key_list CArray<u32>. 171 grown_level u32.
172 default_friendly_value u64. 173 game_difficulty_buff_level_list GameDifficultyBuffLevel{3×u32}. 174
game_difficulty_buff_info **u32** (BuffKey sub_1015F0E74 inner reads 4B — NOT u16!). 175 empowered_overlay_color u32.
176 empowered_overlay_color_ratio u32. 177 buff_overlay_color_data_map CArray<BuffOverlayColorEntry{key u32,
color u32, ratio u32}>. 178 wall_hit_rebound u8. 179 balance_difficulty_level u32. 180 is_apply_stat_control_data u8.
fallback=0, byte-exact, tail 2410→2342B. Fields 1-180 typed (7 remain).
BUG FOUND: field 174 typed u16 → 986 fallback (empowered/buffed records; tail[0]=0x00010000 = real count 0x0001
shifted 2B). AUDITED all u16 lookups by decompiling INNER readers: detect(108)/dialog_voice(115)/category(116)/
breakable(158)/job(162)/talk_tree(165) inner=2B → genuinely u16 ✓. BuffKey(174) + StatusGroupKey(183) inner=4B → u32.
LESSON REINFORCED: BYREF u16 storage ≠ u16 wire; always check inner (a1,&v,N).
NEXT (LAST 7): 181/182 apply_stat_balace/max sub_101FB185C (=5× sub_101FE88B8 16B struct — decompile sub_101FE88B8),
183 status_group_info u32, 184 _characterLevelDataList sub_101FE89BC CArray<136B element sub_101FAEE38 — decompile>,
185/186 detectable tag lists CArray<u32>(sub_10131FF98), 187 _elementalMaterialInfoList sub_101FBC528 CArray<8B
element sub_101FBC6E4 — decompile>. Finish to tail=0 → characterinfo 100% DONE.

## 2026-06-24 iter14 (COMPLETE ✅) — fields 181-187, characterinfo 100%
181/182 apply_stat_balace/max ApplyStatBalance{5× StatBalance{status_key u32(StatusKey sub_1016154FC=4B), value u64}}.
183 status_group_info u32(StatusGroupKey, inner 4B). 184 character_level_data_list CArray<CharacterLevelData{level u32,
experience u64, drop_experience u64, 4× stat_data_level u32, frame_event_attr u32, learn/hidden_skill_list CArray<u32>,
2× stat_list CArray<StatEntry40{5× u64}>, static_stat_level CArray<u8>, 2× u8}>. 185/186 detectable tag lists CArray<u32>.
187 elemental_material_info_list CArray<ElementalMaterialInfo{status_key u32, elemental_material_key u32}>.
**characterinfo 100% TYPED: 7089/7089 records tail=0, fallback=0, byte-exact roundtrip on live 1.12.02. ALL 187 FIELDS. B-3 DONE.**
