# v3.1 Alias Verification Against NattKh Schema

Cross-reference of dmm-parser's mechanically-generated v3.1 aliases
against the canonical Pearl Abyss field names in NattKh's schema
(extracted from Korean error strings in CrimsonDesert.exe).

Schema source: https://github.com/NattKh/CrimsonDesertModdingTools

## Summary

- Tables with aliases in dmm-parser: **126**
- Tables present in NattKh schema:    **109**
- Tables missing from schema:         **17**
- Total field aliases verified:       **1161**
- Total mechanical-rule mismatches:   **0**
- Total schema fields not decoded:    **548**

## Tables not in NattKh schema

These tables exist in dmm-parser but the schema doesn't have a
matching PascalCase entry. Possible reasons: schema gap, table
name divergence, or dmm-parser table doesn't go through the
Korean-error-string parser path.

- `ai_dialog_string_info` (expected schema key: `AiDialogStringInfo`)
- `aiaction_attribute_info` (expected schema key: `AiactionAttributeInfo`)
- `aidialog_type_info` (expected schema key: `AidialogTypeInfo`)
- `aievent_table_info` (expected schema key: `AieventTableInfo`)
- `aimemory_info` (expected schema key: `AimemoryInfo`)
- `aimove_speed_info` (expected schema key: `AimoveSpeedInfo`)
- `equip_slot_info` (expected schema key: `EquipSlotInfo`)
- `faction_waypoint_info` (expected schema key: `FactionWaypointInfo`)
- `house_info` (expected schema key: `HouseInfo`)
- `mercenary_group_info` (expected schema key: `MercenaryGroupInfo`)
- `paac` (expected schema key: `Paac`)
- `paatt` (expected schema key: `Paatt`)
- `pamhc` (expected schema key: `Pamhc`)
- `pappt` (expected schema key: `Pappt`)
- `ui_social_action_info` (expected schema key: `UiSocialActionInfo`)
- `uifilter_group_info` (expected schema key: `UifilterGroupInfo`)
- `uimap_texture_info` (expected schema key: `UimapTextureInfo`)

## Per-table details

| Table | Aliases | Verified | Mismatches | Missing in dmm-parser |
|---|---|---|---|---|
| `action_point_info` | 4 | 4 | 0 | 2 |
| `action_restriction_order_info` | 18 | 18 | 0 | 0 |
| `ally_group_info` | 11 | 11 | 0 | 1 |
| `auto_spawn_filter_info` | 4 | 4 | 0 | 0 |
| `bitmap_position_info` | 10 | 10 | 0 | 0 |
| `board_info` | 5 | 5 | 0 | 0 |
| `breakable_object_info` | 8 | 8 | 0 | 0 |
| `buff_info` | 13 | 13 | 0 | 0 |
| `category_group_info` | 4 | 4 | 0 | 0 |
| `category_info` | 6 | 6 | 0 | 0 |
| `character_appearance_index_info` | 7 | 7 | 0 | 0 |
| `character_change_info` | 3 | 3 | 0 | 1 |
| `character_group_info` | 15 | 15 | 0 | 0 |
| `character_info` | 18 | 18 | 0 | 146 |
| `condition_info` | 6 | 6 | 0 | 0 |
| `craft_tool_group_info` | 6 | 6 | 0 | 0 |
| `craft_tool_info` | 6 | 6 | 0 | 0 |
| `detect_detail_info` | 4 | 4 | 0 | 0 |
| `detect_info` | 7 | 7 | 0 | 0 |
| `detect_reaction_info` | 6 | 6 | 0 | 1 |
| `dialog_voice_info` | 13 | 13 | 0 | 0 |
| `drop_set_info` | 12 | 12 | 0 | 0 |
| `dye_color_group_info` | 6 | 6 | 0 | 0 |
| `effect_info` | 8 | 8 | 0 | 0 |
| `elemental_material_info` | 19 | 19 | 0 | 1 |
| `equip_info` | 7 | 7 | 0 | 0 |
| `equip_type_info` | 19 | 19 | 0 | 1 |
| `faction_group_info` | 8 | 8 | 0 | 0 |
| `faction_info` | 19 | 19 | 0 | 0 |
| `faction_node_info` | 17 | 17 | 0 | 14 |
| `faction_node_spawn_info` | 5 | 5 | 0 | 1 |
| `faction_relation_group_info` | 3 | 3 | 0 | 1 |
| `faction_spawn_data_info` | 7 | 7 | 0 | 0 |
| `fail_message_info` | 4 | 4 | 0 | 0 |
| `field_info` | 2 | 2 | 0 | 22 |
| `field_level_name_table_info` | 5 | 5 | 0 | 0 |
| `field_revive_info` | 10 | 10 | 0 | 0 |
| `formation_info` | 6 | 6 | 0 | 0 |
| `frame_event_attr_group_info` | 4 | 4 | 0 | 0 |
| `game_advice_group_info` | 8 | 8 | 0 | 0 |
| `game_advice_info` | 15 | 15 | 0 | 0 |
| `game_event_handler_info` | 9 | 9 | 0 | 0 |
| `game_global_effect_info` | 17 | 17 | 0 | 0 |
| `game_level_info` | 6 | 6 | 0 | 0 |
| `game_play_trigger_info` | 12 | 12 | 0 | 0 |
| `game_play_variable_info` | 5 | 5 | 0 | 0 |
| `gimmick_event_table_info` | 7 | 7 | 0 | 0 |
| `gimmick_gate_connection_info` | 9 | 9 | 0 | 0 |
| `gimmick_gate_info` | 6 | 6 | 0 | 0 |
| `gimmick_group_info` | 25 | 25 | 0 | 45 |
| `gimmick_info` | 6 | 6 | 0 | 153 |
| `global_game_event_group_info` | 5 | 5 | 0 | 0 |
| `global_game_event_info` | 5 | 5 | 0 | 3 |
| `global_stage_sequencer_info` | 14 | 14 | 0 | 0 |
| `interaction_info` | 9 | 9 | 0 | 28 |
| `inventory_info` | 11 | 11 | 0 | 0 |
| `item_group_info` | 12 | 12 | 0 | 0 |
| `item_use_info` | 4 | 4 | 0 | 0 |
| `job_info` | 7 | 7 | 0 | 0 |
| `key_map_setting_list_info` | 4 | 4 | 0 | 0 |
| `knowledge_group_info` | 15 | 15 | 0 | 0 |
| `knowledge_info` | 30 | 30 | 0 | 0 |
| `level_action_point_info` | 5 | 5 | 0 | 0 |
| `level_gimmick_scene_object_info` | 23 | 23 | 0 | 1 |
| `local_string_info` | 5 | 5 | 0 | 0 |
| `material_blood_decal_info` | 5 | 5 | 0 | 0 |
| `material_match_info` | 8 | 8 | 0 | 0 |
| `material_relation_info` | 4 | 4 | 0 | 0 |
| `mercenary_info` | 15 | 15 | 0 | 0 |
| `mini_game_data_info` | 16 | 16 | 0 | 0 |
| `mission_info` | 15 | 15 | 0 | 25 |
| `multi_change_info` | 25 | 25 | 0 | 0 |
| `npc_info` | 15 | 15 | 0 | 0 |
| `part_prefab_dye_slot_info` | 5 | 5 | 0 | 0 |
| `part_prefab_dye_texture_pallete_info` | 5 | 5 | 0 | 0 |
| `pattern_description_info` | 10 | 10 | 0 | 0 |
| `platform_achievement_info` | 10 | 10 | 0 | 0 |
| `platform_entitlement_info` | 9 | 9 | 0 | 0 |
| `quest_gauge_info` | 12 | 12 | 0 | 0 |
| `quest_group_info` | 15 | 15 | 0 | 0 |
| `quest_info` | 34 | 34 | 0 | 0 |
| `quick_time_event_info` | 4 | 4 | 0 | 0 |
| `region_info` | 23 | 23 | 0 | 0 |
| `relation_info` | 11 | 11 | 0 | 0 |
| `reserve_slot_info` | 17 | 17 | 0 | 0 |
| `royal_supply_info` | 5 | 5 | 0 | 1 |
| `sequencer_spawn_info` | 8 | 8 | 0 | 0 |
| `skill_group_info` | 5 | 5 | 0 | 0 |
| `skill_info` | 34 | 34 | 0 | 0 |
| `skill_tree_group_info` | 9 | 9 | 0 | 0 |
| `skill_tree_info` | 15 | 15 | 0 | 0 |
| `socket_group_info` | 4 | 4 | 0 | 0 |
| `socket_info` | 5 | 5 | 0 | 0 |
| `spawning_pool_auto_spawn_info` | 15 | 15 | 0 | 0 |
| `special_mode_info` | 24 | 24 | 0 | 0 |
| `stage_info` | 10 | 10 | 0 | 72 |
| `status_group_info` | 8 | 8 | 0 | 0 |
| `status_info` | 34 | 34 | 0 | 0 |
| `store_info` | 21 | 21 | 0 | 0 |
| `string_info` | 4 | 4 | 0 | 0 |
| `sub_level_info` | 22 | 22 | 0 | 1 |
| `terrain_region_auto_spawn_info` | 24 | 24 | 0 | 0 |
| `terrain_region_navi_info` | 4 | 4 | 0 | 0 |
| `tribe_info` | 3 | 3 | 0 | 26 |
| `trigger_region_info` | 4 | 4 | 0 | 0 |
| `valid_schedule_action_info` | 6 | 6 | 0 | 0 |
| `vehicle_info` | 19 | 19 | 0 | 2 |
| `vibrate_pattern_info` | 6 | 6 | 0 | 0 |
| `wanted_info` | 5 | 5 | 0 | 0 |

## Mismatch / missing-field detail

### `action_point_info` (schema key: `ActionPointInfo`)

**Schema fields not in dmm-parser** (2):

- `_actionPosition`
- `_actionYaw`

### `ally_group_info` (schema key: `AllyGroupInfo`)

**Schema fields not in dmm-parser** (1):

- `_relationTypeList`

### `character_change_info` (schema key: `CharacterChangeInfo`)

**Schema fields not in dmm-parser** (1):

- `_characterChangeFilter`

### `character_info` (schema key: `CharacterInfo`)

**Schema fields not in dmm-parser** (146):

- `_additionalPartsDataList`
- `_aiDialogOverrideList`
- `_aiScriptPathFocusHash`
- `_aiScriptPathHash`
- `_aliveSkillInfoList`
- `_allowFarAttackTarget`
- `_allyGroupInfo`
- `_attackByCollisionInfoListKey`
- `_baseMaterialKeyOverride`
- `_battleOrderType`
- `_breakableObjectInfo`
- `_bulletItem`
- `_bumpTypeHash`
- `_catchSpawnData`
- `_characterAge`
- `_characterCollisionType`
- `_characterDesc`
- `_characterFriendlyItemDataList`
- `_characterGroupInfoList`
- `_characterInteractionOverrideDataList`
- `_characterLevelDataList`
- `_characterName`
- `_characterPauseType`
- `_characterRegionInfoList`
- `_characterRewardDataList`
- `_characterScale`
- `_characterThreatDialogInfo`
- `_characterTier`
- `_characterTribeAndGenderString`
- `_characterType`
- `_characterWeaponType`
- `_characterWeight`
- `_childVehicleList`
- `_convertItemInfo`
- `_defaultActionActionIndex`
- `_defaultShareValueIndex`
- `_detectInfo`
- `_detectReactionInfo`
- `_detectReactionOverrideList`
- `_detectableGimmickTagNameHashList`
- `_dialogVoiceInfo`
- `_disableFootStepOptimize`
- `_dockingChildDataList`
- `_dockingChildEventList`
- `_elementalMaterialInfoList`
- `_enableDockingGimmickAutoWallUp`
- `_equipItemInfoList`
- `_factionInfo`
- `_farmBreedingCoolTime`
- `_farmBreedingResultList`
- `_farmBreedingTargetList`
- `_farmDropInfoList`
- `_forceFieldTargetType`
- `_gamePlayObjectShareData`
- `_gender`
- `_grownLevel`
- `_grownTargetKeyList`
- `_ignoreTriggerRegion`
- `_ignoreWaterFall`
- `_inspectDataList`
- `_interactionCategoryGroupInfo`
- `_interactionDistance`
- `_interactionInfoList`
- `_interactionUIDistanceLv`
- `_inventoryInfoList`
- `_invincibility`
- `_isAggroTargetable`
- `_isAttackThrowable`
- `_isAttackable`
- `_isCatchable`
- `_isClimbable`
- `_isCreatableDetectIcon`
- `_isEditorUsable`
- `_isEditorUsableAppearance`
- `_isEnableFriendly`
- `_isEquipDropable`
- `_isFarmAnimal`
- `_isGhost`
- `_isGlobalSchedule`
- `_isHirable`
- `_isHudHpEnabled`
- `_isItemSocketContents`
- `_isLogoutAtLooted`
- `_isLookable`
- `_isMapIconAlwaysShow`
- `_isPushable`
- `_isRandomAppearance`
- `_isRandomAppearance_IgnoreScale`
- `_isRandomCharacter`
- `_isRemoteCatchable`
- `_isRewardDropRollByCreateActor`
- `_isSealable`
- `_isShowHpWhenFocusActor`
- `_isTerrainCharacter`
- `_isUnique`
- `_isUseScheduleInfo_Dev`
- `_isVisibleWhenDetectModeOnly`
- `_isWallSwingable`
- `_jobInfo`
- `_knowledgeInfo`
- `_knowledgeObtainType`
- `_lowerActionChartPackageGroupName`
- `_mapIconDisplayType`
- `_maxAggroCount`
- `_memo`
- `_mercenaryDetectableGimmickTagHashList`
- `_mercenaryDropInfoList`
- `_mercenaryHireMessage`
- `_mercenaryInfo`
- `_miniGameParam`
- `_minigameSeedList`
- `_obstacleDisableByDead`
- `_ownedMercenaryCharacterInfo`
- `_ownerFollowType`
- `_pathFindTableName`
- `_pathTrailType`
- `_personalityType`
- `_playerIndex`
- `_playerSkillInfoList`
- `_playerTargetableType`
- `_priceList`
- `_projectileInfoPackage`
- `_refillHPWhenCooltimeEnd`
- `_sendKillEventOnDead`
- `_shareValueNameHash`
- `_skeletonVariationName`
- `_skillInfoByReviveList`
- `_skillInfoBySpawnList`
- `_spawnFixType`
- `_stageInfoForNpcShopList`
- `_statusGroupInfo`
- `_symbolImage`
- `_terrainRegionAutoSpawnInfo`
- `_terrainRegionSpawnPerCount`
- `_trapFoodData`
- `_tribeEffectHash`
- `_tribeInfoWrapper`
- `_uiMapTextureInfo`
- `_uiPortraitPath`
- `_upperActionChartPackageGroupName`
- `_useLargeSplineCurve`
- `_vanishTickCount`
- `_visioningData`
- `_wantedPriceList`
- `_weakPointEffectDataList`
- `_weatherWeight`

### `detect_reaction_info` (schema key: `DetectReactionInfo`)

**Schema fields not in dmm-parser** (1):

- `_reactionTable`

### `elemental_material_info` (schema key: `ElementalMaterialInfo`)

**Schema fields not in dmm-parser** (1):

- `_flag`

### `equip_type_info` (schema key: `EquipTypeInfo`)

**Schema fields not in dmm-parser** (1):

- `_destroyedAiEvent`

### `faction_node_info` (schema key: `FactionNodeInfo`)

**Schema fields not in dmm-parser** (14):

- `_bitMapColorKey`
- `_factionEventDataList`
- `_factionScheduleInfoList`
- `_factionType`
- `_knockDownCondition`
- `_observeData`
- `_religionBlockCostList`
- `_religionEffectRegionInfoList`
- `_religionMaxBlockDay`
- `_religionSubLevelInfo`
- `_researchDataList`
- `_subInnerTypeString`
- `_useCustomWayPointforDev`
- `_workerCount`

### `faction_node_spawn_info` (schema key: `FactionNodeSpawnInfo`)

**Schema fields not in dmm-parser** (1):

- `_boundaryBox`

### `faction_relation_group_info` (schema key: `FactionRelationGroupInfo`)

**Schema fields not in dmm-parser** (1):

- `_relationGroupList`

### `field_info` (schema key: `FieldInfo`)

**Schema fields not in dmm-parser** (22):

- `_addFieldStyle`
- `_alwaysCallVehicle_dev`
- `_boundaryPositionMax`
- `_boundaryPositionMin`
- `_crimeRegionBitmapPositionInfo`
- `_detectInfo`
- `_endSectorIndex`
- `_fieldRegistType`
- `_fixedFieldTime`
- `_isBlocked`
- `_isEnableAutoSave`
- `_levelName`
- `_maxPlayerCount`
- `_natureRegionBitmapPositionInfo`
- `_readOnly`
- `_regionBitmapPositionInfo`
- `_returnPosition`
- `_sceneLevelPath`
- `_sequencerSpawnKey`
- `_spawnPath`
- `_startSectorIndex`
- `_useFixedFieldTime`

### `gimmick_group_info` (schema key: `GimmickGroupInfo`)

**Schema fields not in dmm-parser** (45):

- `_combinationAliasDataList`
- `_combinationLinkDataList`
- `_elementalMaterialInfoList`
- `_elementalStatusInitialStatList`
- `_excludeSequencerBoundary`
- `_gimmickNodeData`
- `_interactionUIDistanceLv`
- `_isAnchorEdgeDisable`
- `_isAttackByCollisionDocking`
- `_isAttackByCollisionDynamic`
- `_isAttackByCollisionKeyFrame`
- `_isAutoPartialBreak`
- `_isBreakMainPartOnBreak`
- `_isBuyable`
- `_isDefaultSpawnDistanceLevel`
- `_isDockingCombinationKeyFrame`
- `_isEditorUseable`
- `_isGetKnowledgeWhenGetItem`
- `_isHousingGimmick`
- `_isIsolatedAnchorBreakable`
- `_isKeepAnchor`
- `_isLinkDecoGimmick`
- `_isMacroGimmick`
- `_isPiercedAllyProjectile`
- `_isScaleable`
- `_isSpawnComponentInLevel`
- `_isSpawnedOnPlatformKeyFrame`
- `_isSpreadBreakInCombination`
- `_isSubPart`
- `_isTargetable`
- `_isUseConstrainSound`
- `_isWild`
- `_pushObjectSocketList`
- `_remoteCatchPullOutUseAction`
- `_saveLevelData`
- `_spawnDistanceLevel`
- `_stickToObjectSocketList`
- `_stickToObjectType`
- `_targetableRange`
- `_unlockableIDataList`
- `_useBuoyancyRestoringCenterOfMass`
- `_useConstraintAchorEdge`
- `_useParentGimmickPoint`
- `_useRemoteCatchFishing`
- `_useSlidingMotionProperty`

### `gimmick_info` (schema key: `GimmickInfo`)

**Schema fields not in dmm-parser** (153):

- `_additionalHeightOnCatched`
- `_allyGroupInfo`
- `_applyGimmickStateToItem`
- `_applyOffsetByScreenSpaceCasting`
- `_attackImpulseCompleteData`
- `_autoSpawnEnviornmentDetailEffect`
- `_batteryInitCapacity`
- `_batteryTotalCapacity`
- `_blockNavigation`
- `_boardKey`
- `_bodyMass`
- `_breakDropOffsetDistance`
- `_breakTypeFromParent`
- `_buoyancySubmersionRatio`
- `_buyableDropItem`
- `_canDisassemble`
- `_centerOfMass`
- `_characterStepHeight`
- `_checkAllyToBreak`
- `_checkAllyToBreakUseGimmickInfo`
- `_cogWheelSawToothCount`
- `_cogWheelTriggerScale`
- `_collectFilter_Dev`
- `_collisionBodyData`
- `_collisionGroupLayer`
- `_constraintSpeedLevel`
- `_controlMaterialParamValueList`
- `_convertItemInfo`
- `_craftToolData`
- `_customVolumeGroupDataList`
- `_defaultAliasName`
- `_defaultSpawnReasonData`
- `_defaultSpawnReasonHash`
- `_detectCustomRenderIndex`
- `_devMemo`
- `_dialogDataList`
- `_dropInfoDataList`
- `_dropOffsetSocketName`
- `_dropRollCount`
- `_dropSetInfoList`
- `_elementalAreaDataList`
- `_elementalAreaWithMaterial`
- `_elementalMaterialInfoList`
- `_elementalReceiverColliderGroupDataList`
- `_elementalStatusInitialStatList`
- `_emojiTextureID`
- `_equipDockingSpawnDistanceLevel`
- `_eventKeyGuideList`
- `_excludeSequencerBoundary`
- `_factionStructure`
- `_fertilizerIntakeAmount`
- `_forceCursorAimTargetable`
- `_forceFieldTargetType`
- `_gamePlayObjectShareData`
- `_generateEffectData`
- `_gimmickAttachTargetDataList`
- `_gimmickChartParameterList`
- `_gimmickFactionInoMode`
- `_gimmickInteractionOverrideDataList`
- `_gimmickName`
- `_gimmickNameHash`
- `_gimmickNodeData`
- `_gimmickOnTimeGroupDataList`
- `_gimmickTagList`
- `_growthDataList`
- `_hasObstacleUseType`
- `_housingGimmickSpecialType`
- `_housingItemPlacementTypeFlag`
- `_housingStackableTypeFlag`
- `_housingSupportPlaneScale`
- `_hoveringData`
- `_impulseSurroundingDistance`
- `_initScale`
- `_initialNavigation`
- `_inspectDataList`
- `_installOriginGimmickInfo`
- `_interactionUIDistanceLv`
- `_isAlwaysSave`
- `_isAttachTargetOfOtherGimmick`
- `_isBlockRoadSpawnStageObstacle`
- `_isBlockSpawnOnAwayFromOriginTransform`
- `_isBuyable`
- `_isCollectOnlyGimmick`
- `_isHandCatchable`
- `_isHousingGimmick`
- `_isInstallable`
- `_isLevelGimmickQuickRespawn`
- `_isPuzzleGimmick`
- `_isSavePresetTarget`
- `_isShowInteractionByTrigger`
- `_isTargetable`
- `_isTwoHandsRemoteCatch`
- `_isUnique`
- `_isWild`
- `_jamReactionType`
- `_jammedLogoutEffectName`
- `_keepClimbPointWhenBroken`
- `_knowledgeExtractType`
- `_knowledgeInfo`
- `_makeNaviVoxelSpecial`
- `_massLevel`
- `_maxFertilizerAmount`
- `_miniGameDataList`
- `_motionTypeAsPlatform`
- `_movableNavigation`
- `_pageGimmickInfo`
- `_pendulumData`
- `_physicsBreakingDeltaVelocityThreashold`
- `_physicsContactEventDeltaVelocityThreashold`
- `_physicsQualityPreset`
- `_physicsTriggerDataList`
- `_propagateSkillFromParentActor`
- `_propertyConditionStringListForDebug`
- `_propertyList`
- `_pushObjectSocketList`
- `_pushableDirection`
- `_registerAsPlatformOfSummonee`
- `_remoteCatchPullInDurationTime`
- `_respawnTimeSeconds`
- `_saveLevelData`
- `_saveOption`
- `_sealCompleteCount`
- `_sealData`
- `_shaderMaterialEffectType`
- `_snapDialData`
- `_snowRatio`
- `_spawnDistanceLevel`
- `_spawnableVisibleOnly`
- `_stickToObjectSocketList`
- `_stickToObjectType`
- `_summonCharacterDataList`
- `_summonGimmickDataList`
- `_summonItemDataList`
- `_summonRandomDataList`
- `_targetSealPartGimmickInfoList`
- `_targetableRange`
- `_timerRandomInterval`
- `_trafficBoxDataList`
- `_transformSetList`
- `_transmutationMaterialGimmickList`
- `_transmutationMaterialItemGroupList`
- `_transmutationMaterialItemList`
- `_triggerCheckTargetDataList`
- `_triggerCheckTargetType`
- `_triggerVolumeGroupDataList`
- `_uiMapTextureInfo`
- `_useGroupingRemoteCatch`
- `_useInteractionUISocket`
- `_useOnDemandCombination`
- `_useRemoteCatchFishing`
- `_useSubPartForInteraction`
- `_vehicleInfo`
- `_weakPointEffectDataList`

### `global_game_event_info` (schema key: `GlobalGameEventInfo`)

**Schema fields not in dmm-parser** (3):

- `_eventDesc`
- `_targetRegionInfoList`
- `_uiIconPath`

### `interaction_info` (schema key: `InteractionInfo`)

**Schema fields not in dmm-parser** (28):

- `_allowInteractionWhileInteraction`
- `_autoInteractionType`
- `_autoMovingStopDistance`
- `_buttonClickType`
- `_cancelOnMoveFail`
- `_categoryInfo`
- `_checkObjectOnTop`
- `_dialogSetInfo`
- `_enableOnDockingOrCatch`
- `_fixRotationWhileInteraction`
- `_inputKeyMapName`
- `_interactionConditionDataList`
- `_interactionPopItemList`
- `_interactionTag`
- `_isCatchInteractionForEditor`
- `_isPlayerInterruptable`
- `_onPreemptionSuccessAiEventKey`
- `_rewardDropSetInfo`
- `_sequencerStageChartDesc`
- `_showMainMenuEventName`
- `_showMainMenuPanelName`
- `_showUIAtPivotSocket`
- `_showWhenTargeted`
- `_subInteraction`
- `_uiKeyTriggerSoundName`
- `_useActionGotoOffset`
- `_useFacingGotoTransform`
- `_waitForInteraction`

### `level_gimmick_scene_object_info` (schema key: `LevelGimmickSceneObjectInfo`)

**Schema fields not in dmm-parser** (1):

- `_onDiscoverOnlyEnable`

### `mission_info` (schema key: `MissionInfo`)

**Schema fields not in dmm-parser** (25):

- `_challengeEventList`
- `_checkCompleteCountAtOnce`
- `_checkOverlapType`
- `_completeCount`
- `_completeLog`
- `_completeName`
- `_completeTime`
- `_completeType`
- `_desc`
- `_existComplete`
- `_existFail`
- `_existHaveCount`
- `_existStart`
- `_ignoreRepeatOnDead`
- `_isOperationMission`
- `_isShowAlertPlaying`
- `_limitTime`
- `_missionFunctionList`
- `_name`
- `_optional`
- `_parentMissionInfo`
- `_preCheck`
- `_repeatCondition`
- `_showMiniMap`
- `_targetQuestDialogKey`

### `royal_supply_info` (schema key: `RoyalSupplyInfo`)

**Schema fields not in dmm-parser** (1):

- `_royalSupplyRandomMap`

### `stage_info` (schema key: `StageInfo`)

**Schema fields not in dmm-parser** (72):

- `_beginTime`
- `_changeTime`
- `_childStageList`
- `_closeCondition`
- `_closeDialogSoundEventName`
- `_closeDialogSpeakerCharacter`
- `_closeDialogString`
- `_closeFilter`
- `_closeFilterByGroup`
- `_completeAlertType`
- `_completeCount`
- `_completeImagePath`
- `_disableGiveUp`
- `_endTime`
- `_evadeProjectile`
- `_executeTargetStageList`
- `_executorMissionList`
- `_executorStageList`
- `_factionSequencerSpawnTagHash`
- `_fieldInfo`
- `_fieldReviveInfo`
- `_followParentReaction`
- `_forbiddenCharacterList`
- `_globalEffectData`
- `_globalFilterCharacterList`
- `_guideEffectName`
- `_hasDynamicActor`
- `_ignoreFactionClose`
- `_isFactionSequencer`
- `_isForceSpawnAfterRetreat`
- `_isForceSpawnAllActor`
- `_isForceSpawnNearDistance`
- `_isIgnoreDistance`
- `_isPlayableOnWanted`
- `_isSave`
- `_itemConditionAndRemoveArray`
- `_levelNameList`
- `_npcShopCharacterInfo`
- `_ownerMissionInfo`
- `_parentQuest`
- `_parentStage`
- `_platformCharacter`
- `_platformDockingTagHash`
- `_platformSocketName`
- `_playCondition`
- `_questType`
- `_randomPercent`
- `_randomRepeatTime`
- `_randomSpawnCount`
- `_resetSecond`
- `_rewardDropSetInfoList`
- `_saveSchedule`
- `_scheduleCompleteCondition`
- `_scheduleStageCompleteAIEventList`
- `_showStageIcon`
- `_spawnBlockTypeFlag`
- `_stageCategory`
- `_stageDataType`
- `_stageGameEventDataList`
- `_stageIconPath`
- `_stageImagePath`
- `_stageKnowledge`
- `_stageTextIconPath`
- `_startPlayerList`
- `_subTimelineBreakDescList`
- `_updatePriority`
- `_useMercenaryLogout`
- `_useRevivePointForDead`
- `_weatherEndBlendTime`
- `_weatherInfo`
- `_weatherIngTime`
- `_weatherStartBlendTime`

### `sub_level_info` (schema key: `SubLevelInfo`)

**Schema fields not in dmm-parser** (1):

- `_exp`

### `tribe_info` (schema key: `TribeInfo`)

**Schema fields not in dmm-parser** (26):

- `_activityWaterDepth`
- `_armorMaterialKey`
- `_baseMaterialKey`
- `_bumpTypeHash`
- `_characterPauseType`
- `_detectModeShowEnemy`
- `_detourMaxDegree`
- `_detourOnRoad`
- `_escapePlatform`
- `_footMaterialKey`
- `_footStepTypeEffectName`
- `_hasChild`
- `_ignoreOverlapPush`
- `_ignoreWaterFall`
- `_ignoredReactionInSafeZoneFlag`
- `_interactionUIDistanceLv`
- `_isBird`
- `_isDeathByDrowning`
- `_isHumanoid`
- `_parentTribeInfo`
- `_tamedSkillList`
- `_tribeMassLevel`
- `_tribeNameForEditor`
- `_velocityDampSpeed`
- `_wantedCrimeType`
- `_weaponMaterialKey`

### `vehicle_info` (schema key: `VehicleInfo`)

**Schema fields not in dmm-parser** (2):

- `_parentLinkAttachDataList`
- `_vehicleSeatDataList`
