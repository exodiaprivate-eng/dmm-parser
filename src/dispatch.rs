// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Generic table dispatch — pure-Rust API.
//!
//! This module exposes the same 122-table dispatch as `python.rs::dispatch_parse`
//! / `dispatch_serialize_bytes`, but with `io::Result<...>` instead of `PyResult<...>`
//! so Rust callers (DMM, CLI tools) can use it without the PyO3 dependency.
//!
//! The Python wrappers in `python.rs` call these functions and convert
//! `io::Error` to `PyValueError`. This keeps a single source of truth for the
//! match arms — adding a new table = one entry here, the Python side picks it
//! up automatically.
//!
//! ## Public API
//!
//! ```ignore
//! use dmm_parser::dispatch::{parse_table_to_json, serialize_table_from_json};
//!
//! let items: Vec<serde_json::Value> = parse_table_to_json(
//!     "gimmick_info",
//!     &pabgb_bytes,
//!     Some(&pabgh_bytes),
//! )?;
//!
//! let raw: Vec<u8> = serialize_table_from_json("gimmick_info", &items)?;
//! ```
//!
//! See `FIELD_JSON_V3_1_SPEC.md` (in CrimsonGameMods repo) for the format these
//! functions support.

use std::io;

/// Parse a `.pabgb` body to a list of typed JSON dicts.
///
/// `table_name`: snake_case identifier matching `src/tables/<name>/`.
/// E.g. `"gimmick_info"`, `"condition_info"`, `"vehicle_info"`.
///
/// `pabgb`: the raw `.pabgb` bytes.
///
/// `pabgh`: optional `.pabgh` companion bytes. Required for ~47 tables that
/// use the pabgh_blob_table layout (entries are size-delimited via offsets in
/// the .pabgh file). Ignored for ~73 sequential tables (which self-delimit).
///
/// Returns one `serde_json::Value::Object` per record.
///
/// Errors:
/// - `InvalidInput`: `table_name` is unknown
/// - `InvalidInput`: pabgh missing for a pabgh-bounded table
/// - `InvalidData`: parse error mid-record (table-specific message)
pub fn parse_table_to_json(
    table_name: &str,
    pabgb: &[u8],
    pabgh: Option<&[u8]>,
) -> io::Result<Vec<serde_json::Value>> {
    use crate::binary::BinaryRead;
    use crate::tables::blob_runtime::parse_typed_blob_table_to_json_with_pabgh;

    macro_rules! p {
        ($ty:path) => {{
            let ph = pabgh.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput,
                format!("table '{}' requires a pabgh file", table_name)))?;
            parse_typed_blob_table_to_json_with_pabgh(pabgb, ph, |data, offset, size| {
                <$ty>::read_with_size(data, offset, size).map(|t| t.to_json_dict())
            })?
        }};
    }

    macro_rules! s {
        ($ty:path) => {{
            let mut offset = 0usize;
            let mut out: Vec<serde_json::Value> = Vec::new();
            while offset < pabgb.len() {
                let item = <$ty>::read_from(pabgb, &mut offset)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
                        format!("offset 0x{:08x}: {}", offset, e)))?;
                out.push(serde_json::Value::Object(item.to_json_dict()));
            }
            out
        }};
    }

    let table_name = normalize_target_name(table_name).unwrap_or(table_name);

    Ok(match table_name {
        // ── pabgh-bounded tables ──────────────────────────────────────────
        "ai_dialog_string_info"          => p!(crate::tables::ai_dialog_string_info::AIDialogStringInfo),
        "bitmap_position_info"           => p!(crate::tables::bitmap_position_info::BitmapPositionInfo),
        "buff_info"                      => p!(crate::tables::buff_info::BuffInfo),
        "character_change_info"          => p!(crate::tables::character_change_info::CharacterChangeInfo),
        "character_info"                 => p!(crate::tables::character_info::CharacterInfo),
        "condition_info"                 => p!(crate::tables::condition_info::ConditionInfo),
        "drop_set_info"                  => p!(crate::tables::drop_set_info::DropSetInfo),
        "effect_info"                    => p!(crate::tables::effect_info::EffectInfo),
        "elemental_material_info"        => p!(crate::tables::elemental_material_info::ElementalMaterialInfo),
        "equip_info"                     => p!(crate::tables::equip_info::EquipInfo),
        "equip_slot_info"                => {
            let ph = pabgh.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput,
                "table 'equip_slot_info' requires a pabgh file"))?;
            crate::tables::equip_slot_info::parse_equip_slot_info_to_json_with_pabgh(pabgb, ph)?
        },
        "faction_info"                   => p!(crate::tables::faction_info::FactionInfo),
        "faction_node_info"              => p!(crate::tables::faction_node_info::FactionNodeInfo),
        "faction_node_spawn_info"        => p!(crate::tables::faction_node_spawn_info::FactionNodeSpawnInfo),
        "faction_spawn_data_info"        => p!(crate::tables::faction_spawn_data_info::FactionSpawnDataInfo),
        "field_revive_info"              => p!(crate::tables::field_revive_info::FieldReviveInfo),
        "frame_event_attr_group_info"    => p!(crate::tables::frame_event_attr_group_info::FrameEventAttrGroupInfo),
        "game_event_handler_info"        => p!(crate::tables::game_event_handler_info::GameEventHandlerInfo),
        "game_global_effect_info"        => p!(crate::tables::game_global_effect_info::GameGlobalEffectInfo),
        "game_level_info"                => p!(crate::tables::game_level_info::GameLevelInfo),
        "game_play_trigger_info"         => p!(crate::tables::game_play_trigger_info::GamePlayTriggerInfo),
        "gimmick_group_info"             => p!(crate::tables::gimmick_group_info::GimmickGroupInfo),
        "gimmick_info"                   => p!(crate::tables::gimmick_info::GimmickInfo),
        "global_game_event_info"         => p!(crate::tables::global_game_event_info::GlobalGameEventInfo),
        "global_stage_sequencer_info"    => p!(crate::tables::global_stage_sequencer_info::GlobalStageSequencerInfo),
        "interaction_info"               => p!(crate::tables::interaction_info::InteractionInfo),
        "inventory_info"                 => p!(crate::tables::inventory_info::InventoryInfo),
        "item_use_info"                  => p!(crate::tables::item_use_info::ItemUseInfo),
        "knowledge_info"                 => p!(crate::tables::knowledge_info::KnowledgeInfo),
        "level_gimmick_scene_object_info" => p!(crate::tables::level_gimmick_scene_object_info::LevelGimmickSceneObjectInfo),
        "mini_game_data_info"            => p!(crate::tables::mini_game_data_info::MiniGameDataInfo),
        "mission_info"                   => p!(crate::tables::mission_info::MissionInfo),
        "multi_change_info"              => p!(crate::tables::multi_change_info::MultiChangeInfo),
        "npc_info"                       => p!(crate::tables::npc_info::NpcInfo),
        "platform_entitlement_info"      => p!(crate::tables::platform_entitlement_info::PlatformEntitlementInfo),
        "quest_info"                     => p!(crate::tables::quest_info::QuestInfo),
        "region_info"                    => p!(crate::tables::region_info::RegionInfo),
        "royal_supply_info"              => p!(crate::tables::royal_supply_info::RoyalSupplyInfo),
        "sequencer_spawn_info"           => p!(crate::tables::sequencer_spawn_info::SequencerSpawnInfo),
        "skill_info"                     => p!(crate::tables::skill_info::SkillInfo),
        "spawning_pool_auto_spawn_info"  => p!(crate::tables::spawning_pool_auto_spawn_info::SpawningPoolAutoSpawnInfo),
        "special_mode_info"              => p!(crate::tables::special_mode_info::SpecialModeInfo),
        "stage_info"                     => p!(crate::tables::stage_info::StageInfo),
        "store_info"                     => p!(crate::tables::store_info::StoreInfo),
        "sub_level_info"                 => p!(crate::tables::sub_level_info::SubLevelInfo),
        "terrain_region_auto_spawn_info" => p!(crate::tables::terrain_region_auto_spawn_info::TerrainRegionAutoSpawnInfo),

        // ── localization (paloc) ──────────────────────────────────────────
        // Self-delimiting via trailing u32 count, not pabgh-bounded or strict
        // sequential. Special-cased like skill_info / equip_slot_info.
        "paloc" | "paloc.pamt" | "localizationstring" => {
            crate::binary::paloc::parse_paloc_to_json(pabgb)?
        },

        // ── sequential tables ─────────────────────────────────────────────
        "action_point_info"              => p!(crate::tables::action_point_info::ActionPointInfo),
        "action_restriction_order_info"  => s!(crate::tables::action_restriction_order_info::ActionRestrictionOrderInfo),
        "aiaction_attribute_info"        => s!(crate::tables::aiaction_attribute_info::AIActionAttributeInfo),
        "aidialog_type_info"             => s!(crate::tables::aidialog_type_info::AIDialogTypeInfo),
        "aievent_table_info"             => s!(crate::tables::aievent_table_info::AIEventTableInfo),
        "aimemory_info"                  => s!(crate::tables::aimemory_info::AIMemoryInfo),
        "aimove_speed_info"              => s!(crate::tables::aimove_speed_info::AIMoveSpeedInfo),
        "ally_group_info"                => s!(crate::tables::ally_group_info::AllyGroupInfo),
        "auto_spawn_filter_info"         => s!(crate::tables::auto_spawn_filter_info::AutoSpawnFilterInfo),
        "board_info"                     => s!(crate::tables::board_info::BoardInfo),
        "breakable_object_info"          => s!(crate::tables::breakable_object_info::BreakableObjectInfo),
        "category_group_info"            => s!(crate::tables::category_group_info::CategoryGroupInfo),
        "category_info"                  => s!(crate::tables::category_info::CategoryInfo),
        "character_appearance_index_info" => s!(crate::tables::character_appearance_index_info::CharacterAppearanceIndexInfo),
        "character_group_info"           => s!(crate::tables::character_group_info::CharacterGroupInfo),
        "craft_tool_group_info"          => s!(crate::tables::craft_tool_group_info::CraftToolGroupInfo),
        "craft_tool_info"                => s!(crate::tables::craft_tool_info::CraftToolInfo),
        "detect_detail_info"             => s!(crate::tables::detect_detail_info::DetectDetailInfo),
        "detect_info"                    => s!(crate::tables::detect_info::DetectInfo),
        "detect_reaction_info"           => s!(crate::tables::detect_reaction_info::DetectReactionInfo),
        "dialog_voice_info"              => s!(crate::tables::dialog_voice_info::DialogVoiceInfo),
        "dye_color_group_info"           => s!(crate::tables::dye_color_group_info::DyeColorGroupInfo),
        "equip_type_info"                => s!(crate::tables::equip_type_info::EquipTypeInfo),
        "faction_group_info"             => s!(crate::tables::faction_group_info::FactionGroupInfo),
        "faction_relation_group_info"    => s!(crate::tables::faction_relation_group_info::FactionRelationGroupInfo),
        "faction_waypoint_info"          => s!(crate::tables::faction_waypoint_info::FactionWaypointInfo),
        "fail_message_info"              => s!(crate::tables::fail_message_info::FailMessageInfo),
        "field_info"                     => p!(crate::tables::field_info::FieldInfo),
        "field_level_name_table_info"    => s!(crate::tables::field_level_name_table_info::FieldLevelNameTableInfo),
        "formation_info"                 => s!(crate::tables::formation_info::FormationInfo),
        "game_advice_group_info"         => s!(crate::tables::game_advice_group_info::GameAdviceGroupInfo),
        "game_advice_info"               => s!(crate::tables::game_advice_info::GameAdviceInfo),
        "game_play_variable_info"        => s!(crate::tables::game_play_variable_info::GamePlayVariableInfo),
        "game_version_data_info"         => s!(crate::tables::game_version_data_info::GameVersionDataInfo),
        "gimmick_event_table_info"       => s!(crate::tables::gimmick_event_table_info::GimmickEventTableInfo),
        "gimmick_gate_connection_info"   => s!(crate::tables::gimmick_gate_connection_info::GimmickGateConnectionInfo),
        "gimmick_gate_info"              => s!(crate::tables::gimmick_gate_info::GimmickGateInfo),
        "global_game_event_group_info"   => s!(crate::tables::global_game_event_group_info::GlobalGameEventGroupInfo),
        "house_info"                     => s!(crate::tables::house_info::HouseInfo),
        "item_group_info"                => s!(crate::tables::item_group_info::ItemGroupInfo),
        "job_info"                       => s!(crate::tables::job_info::JobInfo),
        "key_map_setting_list_info"      => s!(crate::tables::key_map_setting_list_info::KeyMapSettingListInfo),
        "knowledge_group_info"           => s!(crate::tables::knowledge_group_info::KnowledgeGroupInfo),
        "level_action_point_info"        => s!(crate::tables::level_action_point_info::LevelActionPointInfo),
        "local_string_info"              => s!(crate::tables::local_string_info::LocalStringInfo),
        "material_blood_decal_info"      => s!(crate::tables::material_blood_decal_info::MaterialBloodDecalInfo),
        "material_match_info"            => s!(crate::tables::material_match_info::MaterialMatchInfo),
        "material_relation_info"         => s!(crate::tables::material_relation_info::MaterialRelationInfo),
        "mercenary_group_info"           => s!(crate::tables::mercenary_group_info::MercenaryGroupInfo),
        "mercenary_info"                 => s!(crate::tables::mercenary_info::MercenaryInfo),
        "npc_activity_group_info"        => s!(crate::tables::npc_activity_group_info::NpcActivityGroupInfo),
        "npc_activity_info"              => s!(crate::tables::npc_activity_info::NpcActivityInfo),
        "part_prefab_dye_slot_info"      => s!(crate::tables::part_prefab_dye_slot_info::PartPrefabDyeSlotInfo),
        "part_prefab_dye_texture_pallete_info" => s!(crate::tables::part_prefab_dye_texture_pallete_info::PartPrefabDyeTexturePalleteInfo),
        "pattern_description_info"       => s!(crate::tables::pattern_description_info::PatternDescriptionInfo),
        "platform_achievement_info"      => s!(crate::tables::platform_achievement_info::PlatformAchievementInfo),
        "quest_gauge_info"               => s!(crate::tables::quest_gauge_info::QuestGaugeInfo),
        "quest_group_info"               => s!(crate::tables::quest_group_info::QuestGroupInfo),
        "quick_time_event_info"          => s!(crate::tables::quick_time_event_info::QuickTimeEventInfo),
        "relation_info"                  => s!(crate::tables::relation_info::RelationInfo),
        "reserve_slot_info"              => s!(crate::tables::reserve_slot_info::ReserveSlotInfo),
        "skill_group_info"               => s!(crate::tables::skill_group_info::SkillGroupInfo),
        "skill_tree_group_info"          => s!(crate::tables::skill_tree_group_info::SkillTreeGroupInfo),
        "skill_tree_info"                => s!(crate::tables::skill_tree_info::SkillTreeInfo),
        "socket_group_info"              => s!(crate::tables::socket_group_info::SocketGroupInfo),
        "socket_info"                    => s!(crate::tables::socket_info::SocketInfo),
        "status_group_info"              => s!(crate::tables::status_group_info::StatusGroupInfo),
        "status_info"                    => s!(crate::tables::status_info::StatusInfo),
        "string_info"                    => s!(crate::tables::string_info::StringInfo),
        "terrain_region_navi_info"       => s!(crate::tables::terrain_region_navi_info::TerrainRegionNaviInfo),
        "tribe_info"                     => s!(crate::tables::tribe_info::TribeInfo),
        "trigger_region_info"            => s!(crate::tables::trigger_region_info::TriggerRegionInfo),
        "ui_social_action_info"          => s!(crate::tables::ui_social_action_info::UISocialActionInfo),
        "uifilter_group_info"            => s!(crate::tables::uifilter_group_info::UIFilterGroupInfo),
        "uimap_texture_info"             => s!(crate::tables::uimap_texture_info::UIMapTextureInfo),
        "valid_schedule_action_info"     => s!(crate::tables::valid_schedule_action_info::ValidScheduleActionInfo),
        "vehicle_info"                   => s!(crate::tables::vehicle_info::VehicleInfo),
        "vibrate_pattern_info"           => s!(crate::tables::vibrate_pattern_info::VibratePatternInfo),
        "wanted_info"                    => s!(crate::tables::wanted_info::WantedInfo),
        "iteminfo"                       => s!(crate::item_info::ItemInfo),

        // ── file-format tables (non-pabgb) ────────────────────────────────
        // Parsers ported from Workbench fork. Each returns a 1-element
        // Vec<Value> where the single element carries the entire file
        // shape plus `key: 0` / `string_key: ""` so v3 intent dispatch
        // can find it. v3 field paths address nested arrays directly:
        //   primary[5].key_a            (pappt)
        //   sections.section_a[3]       (pamhc, when wired)
        //   states[7].condition_id      (paac, when wired)
        "pappt" => crate::tables::pappt::parse_pappt_to_json(pabgb)?,
        "pamhc" => crate::tables::pamhc::parse_pamhc_to_json(pabgb)?,
        "paatt" => crate::tables::paatt::parse_paatt_to_json(pabgb)?,
        "paac" => crate::tables::paac::parse_paac_to_json(pabgb)?,

        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput,
            format!("unknown table: '{}'", table_name))),
    })
}

/// Serialize a list of typed JSON dicts back to `.pabgb` bytes.
///
/// `table_name`: same set as `parse_table_to_json`.
///
/// `json_items`: typically the output of `parse_table_to_json` after edits,
/// but can also be hand-constructed JSON values matching the table's schema.
///
/// Returns the raw `.pabgb` bytes. The caller is responsible for rebuilding
/// the companion `.pabgh` (offsets file) for pabgh-bounded tables — see
/// `dmm_parser::binary::variant::build_pabgh_for_modified_body` (TODO: rename)
/// or DMM's `iteminfo/v3_overlay::build_pabgh_for_modified_body`.
pub fn serialize_table_from_json(
    table_name: &str,
    json_items: &[serde_json::Value],
) -> io::Result<Vec<u8>> {
    use crate::tables::blob_runtime::serialize_typed_blob_table_from_json;

    macro_rules! d {
        ($ty:path) => {
            serialize_typed_blob_table_from_json(json_items, |w, map| {
                <$ty>::write_from_json_dict(w, map)
            })?
        };
    }

    Ok(match table_name {
        // ── pabgh-bounded tables ──────────────────────────────────────────
        "ai_dialog_string_info"          => d!(crate::tables::ai_dialog_string_info::AIDialogStringInfo),
        "bitmap_position_info"           => d!(crate::tables::bitmap_position_info::BitmapPositionInfo),
        "buff_info"                      => d!(crate::tables::buff_info::BuffInfo),
        "character_change_info"          => d!(crate::tables::character_change_info::CharacterChangeInfo),
        "character_info"                 => d!(crate::tables::character_info::CharacterInfo),
        "condition_info"                 => d!(crate::tables::condition_info::ConditionInfo),
        "drop_set_info"                  => d!(crate::tables::drop_set_info::DropSetInfo),
        "effect_info"                    => d!(crate::tables::effect_info::EffectInfo),
        "elemental_material_info"        => d!(crate::tables::elemental_material_info::ElementalMaterialInfo),
        "equip_info"                     => d!(crate::tables::equip_info::EquipInfo),
        "equip_slot_info"                => {
            crate::tables::equip_slot_info::serialize_equip_slot_info_from_json(json_items)?
        },
        "faction_info"                   => d!(crate::tables::faction_info::FactionInfo),
        "faction_node_info"              => d!(crate::tables::faction_node_info::FactionNodeInfo),
        "faction_node_spawn_info"        => d!(crate::tables::faction_node_spawn_info::FactionNodeSpawnInfo),
        "faction_spawn_data_info"        => d!(crate::tables::faction_spawn_data_info::FactionSpawnDataInfo),
        "field_revive_info"              => d!(crate::tables::field_revive_info::FieldReviveInfo),
        "frame_event_attr_group_info"    => d!(crate::tables::frame_event_attr_group_info::FrameEventAttrGroupInfo),
        "game_event_handler_info"        => d!(crate::tables::game_event_handler_info::GameEventHandlerInfo),
        "game_global_effect_info"        => d!(crate::tables::game_global_effect_info::GameGlobalEffectInfo),
        "game_level_info"                => d!(crate::tables::game_level_info::GameLevelInfo),
        "game_play_trigger_info"         => d!(crate::tables::game_play_trigger_info::GamePlayTriggerInfo),
        "gimmick_group_info"             => d!(crate::tables::gimmick_group_info::GimmickGroupInfo),
        "gimmick_info"                   => d!(crate::tables::gimmick_info::GimmickInfo),
        "global_game_event_info"         => d!(crate::tables::global_game_event_info::GlobalGameEventInfo),
        "global_stage_sequencer_info"    => d!(crate::tables::global_stage_sequencer_info::GlobalStageSequencerInfo),
        "interaction_info"               => d!(crate::tables::interaction_info::InteractionInfo),
        "inventory_info"                 => d!(crate::tables::inventory_info::InventoryInfo),
        "item_use_info"                  => d!(crate::tables::item_use_info::ItemUseInfo),
        "knowledge_info"                 => d!(crate::tables::knowledge_info::KnowledgeInfo),
        "level_gimmick_scene_object_info" => d!(crate::tables::level_gimmick_scene_object_info::LevelGimmickSceneObjectInfo),
        "mini_game_data_info"            => d!(crate::tables::mini_game_data_info::MiniGameDataInfo),
        "mission_info"                   => d!(crate::tables::mission_info::MissionInfo),
        "multi_change_info"              => d!(crate::tables::multi_change_info::MultiChangeInfo),
        "npc_info"                       => d!(crate::tables::npc_info::NpcInfo),
        "platform_entitlement_info"      => d!(crate::tables::platform_entitlement_info::PlatformEntitlementInfo),
        "quest_info"                     => d!(crate::tables::quest_info::QuestInfo),
        "region_info"                    => d!(crate::tables::region_info::RegionInfo),
        "royal_supply_info"              => d!(crate::tables::royal_supply_info::RoyalSupplyInfo),
        "sequencer_spawn_info"           => d!(crate::tables::sequencer_spawn_info::SequencerSpawnInfo),
        "skill_info"                     => d!(crate::tables::skill_info::SkillInfo),
        "spawning_pool_auto_spawn_info"  => d!(crate::tables::spawning_pool_auto_spawn_info::SpawningPoolAutoSpawnInfo),
        "special_mode_info"              => d!(crate::tables::special_mode_info::SpecialModeInfo),
        "stage_info"                     => d!(crate::tables::stage_info::StageInfo),
        "store_info"                     => d!(crate::tables::store_info::StoreInfo),
        "sub_level_info"                 => d!(crate::tables::sub_level_info::SubLevelInfo),
        "terrain_region_auto_spawn_info" => d!(crate::tables::terrain_region_auto_spawn_info::TerrainRegionAutoSpawnInfo),

        // ── localization (paloc) ──────────────────────────────────────────
        "paloc" | "paloc.pamt" | "localizationstring" => {
            crate::binary::paloc::serialize_paloc_from_json(json_items)?
        },

        // ── sequential tables ─────────────────────────────────────────────
        "action_point_info"              => d!(crate::tables::action_point_info::ActionPointInfo),
        "action_restriction_order_info"  => d!(crate::tables::action_restriction_order_info::ActionRestrictionOrderInfo),
        "aiaction_attribute_info"        => d!(crate::tables::aiaction_attribute_info::AIActionAttributeInfo),
        "aidialog_type_info"             => d!(crate::tables::aidialog_type_info::AIDialogTypeInfo),
        "aievent_table_info"             => d!(crate::tables::aievent_table_info::AIEventTableInfo),
        "aimemory_info"                  => d!(crate::tables::aimemory_info::AIMemoryInfo),
        "aimove_speed_info"              => d!(crate::tables::aimove_speed_info::AIMoveSpeedInfo),
        "ally_group_info"                => d!(crate::tables::ally_group_info::AllyGroupInfo),
        "auto_spawn_filter_info"         => d!(crate::tables::auto_spawn_filter_info::AutoSpawnFilterInfo),
        "board_info"                     => d!(crate::tables::board_info::BoardInfo),
        "breakable_object_info"          => d!(crate::tables::breakable_object_info::BreakableObjectInfo),
        "category_group_info"            => d!(crate::tables::category_group_info::CategoryGroupInfo),
        "category_info"                  => d!(crate::tables::category_info::CategoryInfo),
        "character_appearance_index_info" => d!(crate::tables::character_appearance_index_info::CharacterAppearanceIndexInfo),
        "character_group_info"           => d!(crate::tables::character_group_info::CharacterGroupInfo),
        "craft_tool_group_info"          => d!(crate::tables::craft_tool_group_info::CraftToolGroupInfo),
        "craft_tool_info"                => d!(crate::tables::craft_tool_info::CraftToolInfo),
        "detect_detail_info"             => d!(crate::tables::detect_detail_info::DetectDetailInfo),
        "detect_info"                    => d!(crate::tables::detect_info::DetectInfo),
        "detect_reaction_info"           => d!(crate::tables::detect_reaction_info::DetectReactionInfo),
        "dialog_voice_info"              => d!(crate::tables::dialog_voice_info::DialogVoiceInfo),
        "dye_color_group_info"           => d!(crate::tables::dye_color_group_info::DyeColorGroupInfo),
        "equip_type_info"                => d!(crate::tables::equip_type_info::EquipTypeInfo),
        "faction_group_info"             => d!(crate::tables::faction_group_info::FactionGroupInfo),
        "faction_relation_group_info"    => d!(crate::tables::faction_relation_group_info::FactionRelationGroupInfo),
        "faction_waypoint_info"          => d!(crate::tables::faction_waypoint_info::FactionWaypointInfo),
        "fail_message_info"              => d!(crate::tables::fail_message_info::FailMessageInfo),
        "field_info"                     => d!(crate::tables::field_info::FieldInfo),
        "field_level_name_table_info"    => d!(crate::tables::field_level_name_table_info::FieldLevelNameTableInfo),
        "formation_info"                 => d!(crate::tables::formation_info::FormationInfo),
        "game_advice_group_info"         => d!(crate::tables::game_advice_group_info::GameAdviceGroupInfo),
        "game_advice_info"               => d!(crate::tables::game_advice_info::GameAdviceInfo),
        "game_play_variable_info"        => d!(crate::tables::game_play_variable_info::GamePlayVariableInfo),
        "gimmick_event_table_info"       => d!(crate::tables::gimmick_event_table_info::GimmickEventTableInfo),
        "gimmick_gate_connection_info"   => d!(crate::tables::gimmick_gate_connection_info::GimmickGateConnectionInfo),
        "gimmick_gate_info"              => d!(crate::tables::gimmick_gate_info::GimmickGateInfo),
        "global_game_event_group_info"   => d!(crate::tables::global_game_event_group_info::GlobalGameEventGroupInfo),
        "house_info"                     => d!(crate::tables::house_info::HouseInfo),
        "item_group_info"                => d!(crate::tables::item_group_info::ItemGroupInfo),
        "job_info"                       => d!(crate::tables::job_info::JobInfo),
        "key_map_setting_list_info"      => d!(crate::tables::key_map_setting_list_info::KeyMapSettingListInfo),
        "knowledge_group_info"           => d!(crate::tables::knowledge_group_info::KnowledgeGroupInfo),
        "level_action_point_info"        => d!(crate::tables::level_action_point_info::LevelActionPointInfo),
        "local_string_info"              => d!(crate::tables::local_string_info::LocalStringInfo),
        "material_blood_decal_info"      => d!(crate::tables::material_blood_decal_info::MaterialBloodDecalInfo),
        "material_match_info"            => d!(crate::tables::material_match_info::MaterialMatchInfo),
        "material_relation_info"         => d!(crate::tables::material_relation_info::MaterialRelationInfo),
        "mercenary_group_info"           => d!(crate::tables::mercenary_group_info::MercenaryGroupInfo),
        "mercenary_info"                 => d!(crate::tables::mercenary_info::MercenaryInfo),
        "part_prefab_dye_slot_info"      => d!(crate::tables::part_prefab_dye_slot_info::PartPrefabDyeSlotInfo),
        "part_prefab_dye_texture_pallete_info" => d!(crate::tables::part_prefab_dye_texture_pallete_info::PartPrefabDyeTexturePalleteInfo),
        "pattern_description_info"       => d!(crate::tables::pattern_description_info::PatternDescriptionInfo),
        "platform_achievement_info"      => d!(crate::tables::platform_achievement_info::PlatformAchievementInfo),
        "quest_gauge_info"               => d!(crate::tables::quest_gauge_info::QuestGaugeInfo),
        "quest_group_info"               => d!(crate::tables::quest_group_info::QuestGroupInfo),
        "quick_time_event_info"          => d!(crate::tables::quick_time_event_info::QuickTimeEventInfo),
        "relation_info"                  => d!(crate::tables::relation_info::RelationInfo),
        "reserve_slot_info"              => d!(crate::tables::reserve_slot_info::ReserveSlotInfo),
        "skill_group_info"               => d!(crate::tables::skill_group_info::SkillGroupInfo),
        "skill_tree_group_info"          => d!(crate::tables::skill_tree_group_info::SkillTreeGroupInfo),
        "skill_tree_info"                => d!(crate::tables::skill_tree_info::SkillTreeInfo),
        "socket_group_info"              => d!(crate::tables::socket_group_info::SocketGroupInfo),
        "socket_info"                    => d!(crate::tables::socket_info::SocketInfo),
        "status_group_info"              => d!(crate::tables::status_group_info::StatusGroupInfo),
        "status_info"                    => d!(crate::tables::status_info::StatusInfo),
        "string_info"                    => d!(crate::tables::string_info::StringInfo),
        "terrain_region_navi_info"       => d!(crate::tables::terrain_region_navi_info::TerrainRegionNaviInfo),
        "tribe_info"                     => d!(crate::tables::tribe_info::TribeInfo),
        "trigger_region_info"            => d!(crate::tables::trigger_region_info::TriggerRegionInfo),
        "ui_social_action_info"          => d!(crate::tables::ui_social_action_info::UISocialActionInfo),
        "uifilter_group_info"            => d!(crate::tables::uifilter_group_info::UIFilterGroupInfo),
        "uimap_texture_info"             => d!(crate::tables::uimap_texture_info::UIMapTextureInfo),
        "valid_schedule_action_info"     => d!(crate::tables::valid_schedule_action_info::ValidScheduleActionInfo),
        "vehicle_info"                   => d!(crate::tables::vehicle_info::VehicleInfo),
        "vibrate_pattern_info"           => d!(crate::tables::vibrate_pattern_info::VibratePatternInfo),
        "wanted_info"                    => d!(crate::tables::wanted_info::WantedInfo),
        "iteminfo"                       => d!(crate::item_info::ItemInfo),

        // ── file-format tables (non-pabgb) ────────────────────────────────
        "pappt" => crate::tables::pappt::serialize_pappt_from_json(json_items)?,
        "pamhc" => crate::tables::pamhc::serialize_pamhc_from_json(json_items)?,
        "paatt" => crate::tables::paatt::serialize_paatt_from_json(json_items)?,
        "paac" => crate::tables::paac::serialize_paac_from_json(json_items)?,

        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput,
            format!("unknown table: '{}'", table_name))),
    })
}

/// Apply Field-JSON v3.x intents end-to-end against a `.pabgb` body.
///
/// Pipeline: `parse_table_to_json` → `apply_resolved_intents` →
/// `serialize_table_from_json` (or the tracked variant for pabgh-bounded
/// tables). Returns the new body, an optional rebuilt pabgh sister file
/// (populated when the input had a pabgh — i.e., for pabgh-bounded
/// tables), and the per-intent outcome list.
///
/// Routing rules:
///
/// - `"iteminfo"` / `"iteminfo.pabgb"` — special-cased to
///   [`crate::intents::apply_intents_to_iteminfo`]. Sequential, no pabgh.
/// - Sequential tables in the dispatch table — full support, returns
///   `(new_body, None, outcomes)`.
/// - `paloc` / `paloc.pamt` / `localizationstring` — full support; no
///   pabgh.
/// - Pabgh-bounded tables — full support via
///   [`serialize_table_from_json_with_pabgh`]. Returns
///   `(new_body, Some(new_pabgh), outcomes)` with the pabgh rebuilt in
///   the same on-disk format as the input.
///
/// Errors:
///   - `InvalidInput`: unknown `table_name` (passed through from
///     `parse_table_to_json`)
///   - `InvalidData`: parse / apply / serialize / pabgh-rebuild failure
pub fn apply_intents_to_table_body(
    table_name: &str,
    body: &[u8],
    pabgh: Option<&[u8]>,
    intents: &[crate::intents::Intent],
) -> io::Result<(Vec<u8>, Option<Vec<u8>>, Vec<crate::intents::ApplyOutcome>)> {
    // Resolve aliases (`characterinfo.pabgb` → `character_info`, etc.).
    // Fall back to the raw input so unknown names produce a clean
    // dispatch-level error from `parse_table_to_json` instead of a
    // misleading "alias not found" here.
    let table_name = normalize_target_name(table_name).unwrap_or(table_name);

    // Iteminfo is exposed via the dedicated item_info module rather than
    // this dispatcher's match table, so route it explicitly.
    if matches!(table_name, "iteminfo") {
        let (new_body, outcomes) =
            crate::intents::apply_intents_to_iteminfo(body, intents)?;
        return Ok((new_body, None, outcomes));
    }

    let mut records = parse_table_to_json(table_name, body, pabgh)?;
    let outcomes = crate::intents::apply_resolved_intents(&mut records, intents)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("apply: {}", e)))?;

    if let Some(pabgh_bytes) = pabgh {
        let (new_body, new_pabgh) =
            serialize_table_from_json_with_pabgh(table_name, &records, pabgh_bytes)?;
        Ok((new_body, Some(new_pabgh), outcomes))
    } else {
        let new_body = serialize_table_from_json(table_name, &records)?;
        Ok((new_body, None, outcomes))
    }
}

/// Serialize a list of typed JSON records back to `.pabgb` bytes AND
/// rebuild the sister `.pabgh` index, preserving the input pabgh's
/// on-disk format (`U16CountU32Key` / `U16CountU16Key` /
/// `U32CountU32Key`).
///
/// Use this for pabgh-bounded tables when records are added, removed,
/// or change size — anything that shifts entry offsets. The rebuilt
/// pabgh contains one `(key, offset)` pair per record in the input
/// list, in the same order.
///
/// If no records mutated and the input was the original parse output,
/// the output is byte-perfect against the input pabgb / pabgh pair.
pub fn serialize_table_from_json_with_pabgh(
    table_name: &str,
    items: &[serde_json::Value],
    original_pabgh: &[u8],
) -> io::Result<(Vec<u8>, Vec<u8>)> {
    use crate::binary::pabgh::{Pabgh, PabghEntry, PabghFormat};

    let original = Pabgh::parse(original_pabgh)?;
    let format = original.format;

    let (new_body, offsets) = serialize_table_from_json_tracked(table_name, items)?;

    // Format 2 (U16CountU16Key) caps keys at u16::MAX. Catch overflow
    // before it silently truncates.
    if format == PabghFormat::U16CountU16Key {
        for (k, _) in &offsets {
            if *k > u16::MAX as u32 {
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                    format!("pabgh rebuild: key 0x{:x} doesn't fit u16 (table uses pabgh format U16CountU16Key)", k)));
            }
        }
    }

    let entries: Vec<PabghEntry> = offsets
        .into_iter()
        .map(|(key, offset)| PabghEntry { key, offset, extra_bytes: Vec::new() })
        .collect();
    let new_pabgh = Pabgh { format, entries }.to_bytes()?;
    Ok((new_body, new_pabgh))
}

/// Tracked variant of [`serialize_table_from_json`] for pabgh-bounded
/// tables — emits the body plus a `Vec<(key, byte_offset)>` of one
/// pair per record. Sequential tables and paloc don't need this entry
/// point (their offset maps are either irrelevant or computed from the
/// body itself).
fn serialize_table_from_json_tracked(
    table_name: &str,
    items: &[serde_json::Value],
) -> io::Result<(Vec<u8>, Vec<(u32, u32)>)> {
    use crate::tables::blob_runtime::serialize_typed_blob_table_from_json_tracked;

    macro_rules! dt {
        ($ty:path) => {
            serialize_typed_blob_table_from_json_tracked(items, |w, map| {
                <$ty>::write_from_json_dict(w, map)
            })?
        };
    }

    Ok(match table_name {
        "ai_dialog_string_info"          => dt!(crate::tables::ai_dialog_string_info::AIDialogStringInfo),
        "bitmap_position_info"           => dt!(crate::tables::bitmap_position_info::BitmapPositionInfo),
        "buff_info"                      => dt!(crate::tables::buff_info::BuffInfo),
        "character_change_info"          => dt!(crate::tables::character_change_info::CharacterChangeInfo),
        "character_info"                 => dt!(crate::tables::character_info::CharacterInfo),
        "condition_info"                 => dt!(crate::tables::condition_info::ConditionInfo),
        "drop_set_info"                  => dt!(crate::tables::drop_set_info::DropSetInfo),
        "effect_info"                    => dt!(crate::tables::effect_info::EffectInfo),
        "elemental_material_info"        => dt!(crate::tables::elemental_material_info::ElementalMaterialInfo),
        "equip_info"                     => dt!(crate::tables::equip_info::EquipInfo),
        "faction_info"                   => dt!(crate::tables::faction_info::FactionInfo),
        "faction_node_info"              => dt!(crate::tables::faction_node_info::FactionNodeInfo),
        "faction_node_spawn_info"        => dt!(crate::tables::faction_node_spawn_info::FactionNodeSpawnInfo),
        "faction_spawn_data_info"        => dt!(crate::tables::faction_spawn_data_info::FactionSpawnDataInfo),
        "field_revive_info"              => dt!(crate::tables::field_revive_info::FieldReviveInfo),
        "frame_event_attr_group_info"    => dt!(crate::tables::frame_event_attr_group_info::FrameEventAttrGroupInfo),
        "game_event_handler_info"        => dt!(crate::tables::game_event_handler_info::GameEventHandlerInfo),
        "game_global_effect_info"        => dt!(crate::tables::game_global_effect_info::GameGlobalEffectInfo),
        "game_level_info"                => dt!(crate::tables::game_level_info::GameLevelInfo),
        "game_play_trigger_info"         => dt!(crate::tables::game_play_trigger_info::GamePlayTriggerInfo),
        "gimmick_group_info"             => dt!(crate::tables::gimmick_group_info::GimmickGroupInfo),
        "gimmick_info"                   => dt!(crate::tables::gimmick_info::GimmickInfo),
        "global_game_event_info"         => dt!(crate::tables::global_game_event_info::GlobalGameEventInfo),
        "global_stage_sequencer_info"    => dt!(crate::tables::global_stage_sequencer_info::GlobalStageSequencerInfo),
        "interaction_info"               => dt!(crate::tables::interaction_info::InteractionInfo),
        "inventory_info"                 => dt!(crate::tables::inventory_info::InventoryInfo),
        "item_use_info"                  => dt!(crate::tables::item_use_info::ItemUseInfo),
        "knowledge_info"                 => dt!(crate::tables::knowledge_info::KnowledgeInfo),
        "level_gimmick_scene_object_info" => dt!(crate::tables::level_gimmick_scene_object_info::LevelGimmickSceneObjectInfo),
        "mini_game_data_info"            => dt!(crate::tables::mini_game_data_info::MiniGameDataInfo),
        "mission_info"                   => dt!(crate::tables::mission_info::MissionInfo),
        "multi_change_info"              => dt!(crate::tables::multi_change_info::MultiChangeInfo),
        "npc_info"                       => dt!(crate::tables::npc_info::NpcInfo),
        "platform_entitlement_info"      => dt!(crate::tables::platform_entitlement_info::PlatformEntitlementInfo),
        "quest_info"                     => dt!(crate::tables::quest_info::QuestInfo),
        "region_info"                    => dt!(crate::tables::region_info::RegionInfo),
        "royal_supply_info"              => dt!(crate::tables::royal_supply_info::RoyalSupplyInfo),
        "sequencer_spawn_info"           => dt!(crate::tables::sequencer_spawn_info::SequencerSpawnInfo),
        "spawning_pool_auto_spawn_info"  => dt!(crate::tables::spawning_pool_auto_spawn_info::SpawningPoolAutoSpawnInfo),
        "special_mode_info"              => dt!(crate::tables::special_mode_info::SpecialModeInfo),
        "stage_info"                     => dt!(crate::tables::stage_info::StageInfo),
        "store_info"                     => dt!(crate::tables::store_info::StoreInfo),
        "sub_level_info"                 => dt!(crate::tables::sub_level_info::SubLevelInfo),
        "terrain_region_auto_spawn_info" => dt!(crate::tables::terrain_region_auto_spawn_info::TerrainRegionAutoSpawnInfo),

        "skill_info" => dt!(crate::tables::skill_info::SkillInfo),
        "equip_slot_info" => {
            let mut out = Vec::with_capacity(items.len() * 1024);
            let mut offsets = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                let key = item.get("key").and_then(|v| v.as_u64()).ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("equip_slot_info[{}]: missing 'key' field for pabgh rebuild", i)))? as u32;
                offsets.push((key, out.len() as u32));
                crate::tables::equip_slot_info::info::write_equip_slot_info_record(&mut out, item)
                    .map_err(|e| io::Error::new(e.kind(), format!("equip_slot_info[{}]: {}", i, e)))?;
            }
            (out, offsets)
        }

        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput,
            format!("table '{}' is not pabgh-bounded (no pabgh rebuild needed)", table_name))),
    })
}

/// Resolve a Field-JSON target name to its canonical dispatch identifier.
///
/// Field-JSON manifests in the wild use several spellings for the same
/// table. SuperMod-style mods use the compact lower-case-no-underscores
/// form (`characterinfo.pabgb`); legacy v3 specs use the snake_case
/// (`character_info`); some use the bare extension (`character_info.pabgb`).
/// This function maps any of those onto the canonical name used by
/// [`parse_table_to_json`] / [`serialize_table_from_json`].
///
/// Recognized inputs:
///
/// - Canonical snake_case (`character_info`)
/// - Snake_case + extension (`character_info.pabgb`)
/// - Compact lowercase (`characterinfo`)
/// - Compact + extension (`characterinfo.pabgb`)
/// - Iteminfo aliases (`iteminfo`, `iteminfo.pabgb`)
/// - Paloc aliases (`paloc`, `paloc.pamt`, `localizationstring`)
///
/// Returns `None` if the input doesn't resolve to any known table. Use
/// [`is_supported_table`] for a yes/no answer or pass the result of this
/// function to the apply / parse / serialize entry points.
pub fn normalize_target_name(input: &str) -> Option<&'static str> {
    // Strip recognized extensions.
    let stripped = input
        .strip_suffix(".pabgb")
        .or_else(|| input.strip_suffix(".pamt"))
        .unwrap_or(input);

    // Iteminfo lives outside dispatch's table list — handle it explicitly.
    if matches!(stripped, "iteminfo") {
        return Some("iteminfo");
    }

    // Paloc has multiple legitimate aliases.
    if matches!(stripped, "paloc" | "localizationstring") {
        return Some("paloc");
    }

    // Direct match against canonical names.
    for &canonical in supported_tables() {
        if stripped == canonical {
            return Some(canonical);
        }
    }

    // Compact form: canonical with underscores stripped (e.g.
    // "characterinfo" → "character_info"). Done O(N) — the table list is
    // ~120 entries; if this becomes hot we can lazy-init a HashMap.
    for &canonical in supported_tables() {
        if !canonical.contains('_') {
            continue;
        }
        let compact_len = canonical.len() - canonical.matches('_').count();
        if stripped.len() != compact_len {
            continue;
        }
        // Compare canonical (skipping '_') against stripped char-by-char.
        let mut cs = canonical.bytes().filter(|&b| b != b'_');
        let mut ss = stripped.bytes();
        let matches = std::iter::from_fn(|| match (cs.next(), ss.next()) {
            (Some(a), Some(b)) => Some(a == b),
            (None, None) => None,
            _ => Some(false),
        }).all(|m| m);
        if matches {
            return Some(canonical);
        }
    }

    None
}

/// True if `table_name` is supported by `parse_table_to_json`/`serialize_table_from_json`.
///
/// Useful for callers that want to detect supported targets without triggering
/// a parse error. Cheaper than a real parse: just dispatches the match arm.
///
/// For file-format tables (paac, paatt, pamhc, pappt) this still returns
/// true even though the JSON layer isn't wired yet — callers needing to
/// know whether field-level apply works should attempt the parse and
/// catch the `Unsupported` ErrorKind.
pub fn is_supported_table(table_name: &str) -> bool {
    // Cheapest test: try a parse on empty bytes. If the table is recognized
    // but pabgh is missing, we get InvalidInput("requires a pabgh"). If the
    // table is unknown we get InvalidInput("unknown table"). Distinguish via
    // the message. Both errors are no-ops (no allocation, no real parse).
    match parse_table_to_json(table_name, &[], None) {
        Ok(_) => true,  // empty body parsed OK (sequential tables)
        Err(e) => !e.to_string().starts_with("unknown table:"),
    }
}

/// True if the table is a file-format table (paac/paatt/pamhc/pappt) — i.e.
/// recognized by the parser but JSON layer not yet implemented. Callers can
/// use this to skip the v3 field-level apply path and route to byte-level
/// passthrough instead. Returns false for pabgb tables and unknown names.
pub fn is_file_format_table(table_name: &str) -> bool {
    matches!(table_name, "paac" | "paatt" | "pamhc" | "pappt")
}

/// List every table_name supported by this dispatcher.
///
/// Order: `pabgh-bounded` first (alphabetical), then `sequential` (alphabetical).
/// Stable across releases — adding a new table appends to one of the two
/// halves. Useful for tooling/UI dropdowns.
pub fn supported_tables() -> &'static [&'static str] {
    &[
        // pabgh-bounded
        "ai_dialog_string_info", "bitmap_position_info", "buff_info",
        "character_change_info", "character_info", "condition_info",
        "drop_set_info", "effect_info", "elemental_material_info",
        "equip_info", "equip_slot_info", "faction_info", "faction_node_info",
        "faction_node_spawn_info", "faction_spawn_data_info",
        "field_revive_info", "frame_event_attr_group_info",
        "game_event_handler_info", "game_global_effect_info",
        "game_level_info", "game_play_trigger_info", "gimmick_group_info",
        "gimmick_info", "global_game_event_info", "global_stage_sequencer_info",
        "interaction_info", "inventory_info", "item_use_info",
        "knowledge_info", "level_gimmick_scene_object_info",
        "mini_game_data_info", "mission_info", "multi_change_info",
        "npc_info", "platform_entitlement_info", "quest_info", "region_info",
        "royal_supply_info", "sequencer_spawn_info", "skill_info",
        "spawning_pool_auto_spawn_info", "special_mode_info", "stage_info",
        "store_info", "sub_level_info", "terrain_region_auto_spawn_info",
        // localization
        "paloc", "paloc.pamt", "localizationstring",
        // sequential
        "action_point_info", "action_restriction_order_info",
        "aiaction_attribute_info", "aidialog_type_info", "aievent_table_info",
        "aimemory_info", "aimove_speed_info", "ally_group_info",
        "auto_spawn_filter_info", "board_info", "breakable_object_info",
        "category_group_info", "category_info", "character_appearance_index_info",
        "character_group_info", "craft_tool_group_info", "craft_tool_info",
        "detect_detail_info", "detect_info", "detect_reaction_info",
        "dialog_voice_info", "dye_color_group_info", "equip_type_info",
        "faction_group_info", "faction_relation_group_info",
        "faction_waypoint_info", "fail_message_info", "field_info",
        "field_level_name_table_info", "formation_info",
        "game_advice_group_info", "game_advice_info", "game_play_variable_info",
        "game_version_data_info",
        "gimmick_event_table_info", "gimmick_gate_connection_info",
        "gimmick_gate_info", "global_game_event_group_info", "house_info",
        "item_group_info", "job_info", "key_map_setting_list_info",
        "knowledge_group_info", "level_action_point_info", "local_string_info",
        "material_blood_decal_info", "material_match_info",
        "material_relation_info", "mercenary_group_info", "mercenary_info",
        "npc_activity_group_info", "npc_activity_info",
        "part_prefab_dye_slot_info", "part_prefab_dye_texture_pallete_info",
        "pattern_description_info", "platform_achievement_info",
        "quest_gauge_info", "quest_group_info", "quick_time_event_info",
        "relation_info", "reserve_slot_info", "skill_group_info",
        "skill_tree_group_info", "skill_tree_info", "socket_group_info",
        "socket_info", "status_group_info", "status_info", "string_info",
        "terrain_region_navi_info", "tribe_info", "trigger_region_info",
        "ui_social_action_info", "uifilter_group_info", "uimap_texture_info",
        "valid_schedule_action_info", "vehicle_info", "vibrate_pattern_info",
        "wanted_info", "iteminfo",
        // file-format tables (Phase 1: parsers ported, JSON layer pending)
        "paac", "paatt", "pamhc", "pappt",
    ]
}

// ─────────────────────────────────────────────────────────────────────────
// Shape-aware wrappers (v3 / v3.1 surface selector)
// ─────────────────────────────────────────────────────────────────────────

/// Parse a `.pabgb` body to typed JSON, then project field names according
/// to the requested `JsonShape`. `JsonShape::V3` is identity (snake_case
/// names emitted by the Rust struct's `to_json_dict` pass through). For
/// `JsonShape::V3_1`, every field name covered by the table's
/// `FIELD_ALIASES_V3_1` table is renamed to its canonical Pearl Abyss
/// `_camelCase` identifier.
///
/// Tables with no v3.1 alias entry (or entries the generator skipped because
/// the field name is a placeholder like `field_a` / `_unkXXXX`) round-trip
/// unchanged regardless of shape.
pub fn parse_table_to_json_shaped(
    table_name: &str,
    pabgb: &[u8],
    pabgh: Option<&[u8]>,
    shape: crate::json_shape::JsonShape,
) -> io::Result<Vec<serde_json::Value>> {
    let mut items = parse_table_to_json(table_name, pabgb, pabgh)?;
    if matches!(shape, crate::json_shape::JsonShape::V3_1) {
        if let Some(aliases) = crate::json_shape::lookup_table_aliases_v3_1(table_name) {
            for item in items.iter_mut() {
                if let serde_json::Value::Object(map) = item {
                    crate::json_shape::apply_v3_1_aliases(map, aliases);
                }
            }
        }
    }
    Ok(items)
}

/// Serialize typed JSON back to `.pabgb` bytes. Shape-tolerant on input —
/// each item's keys are normalized from `_camelCase` (v3.1) to snake_case
/// (the Rust struct's read form) before per-field deserialization, so a
/// caller can submit either shape regardless of what `shape` they declare.
/// The `shape` argument is currently advisory; both inputs accepted, output
/// matches what the underlying serializer produces.
pub fn serialize_table_from_json_shaped(
    table_name: &str,
    json_items: &[serde_json::Value],
    _shape: crate::json_shape::JsonShape,
) -> io::Result<Vec<u8>> {
    if let Some(aliases) = crate::json_shape::lookup_table_aliases_v3_1(table_name) {
        let mut normalized: Vec<serde_json::Value> =
            json_items.iter().cloned().collect();
        for item in normalized.iter_mut() {
            crate::json_shape::normalize_input_aliases_v3_1(item, aliases);
        }
        serialize_table_from_json(table_name, &normalized)
    } else {
        serialize_table_from_json(table_name, json_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Resolve a sister-pabgh fixture path for a pabgh-bounded fixture
    /// candidate. Searches the same dirs as the lib.rs tests do.
    fn fixture_pair(name: &str) -> Option<(Vec<u8>, Vec<u8>)> {
        let env_var = format!("DMM_PARSER_{}_PATH", name.to_uppercase());
        let mut candidates: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(p) = std::env::var(&env_var) {
            candidates.push(std::path::PathBuf::from(p));
        }
        for base in [
            "/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1",
            r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-5-1",
            r"C:\Users\corin\Desktop\CD JSON Mod Manager\Unpacked\0008\gamedata",
        ] {
            candidates.push(std::path::PathBuf::from(base).join(format!("{}.pabgb", name)));
        }
        for pabgb_path in &candidates {
            let pabgh_path = pabgb_path.with_extension("pabgh");
            let (Ok(pabgb), Ok(pabgh)) = (std::fs::read(pabgb_path), std::fs::read(&pabgh_path)) else {
                continue;
            };
            return Some((pabgb, pabgh));
        }
        None
    }

    /// Round-trip preservation on a real pabgh-bounded table. Parse, apply
    /// no intents, serialize-with-pabgh, and assert byte-perfect output
    /// against the originals. Strong sanity check that the tracked
    /// serializer + pabgh rebuild don't lose data when nothing changed.
    #[test]
    fn buff_info_apply_empty_intents_byte_perfect() {
        let Some((body, pabgh)) = fixture_pair("buff_info") else {
            eprintln!("SKIP buff_info_apply_empty_intents_byte_perfect: no fixture");
            return;
        };
        let (new_body, new_pabgh, outcomes) =
            apply_intents_to_table_body("buff_info", &body, Some(&pabgh), &[])
                .expect("apply");
        assert!(outcomes.is_empty());
        assert_eq!(new_body, body, "body diverged");
        assert_eq!(new_pabgh.expect("pabgh returned"), pabgh, "pabgh diverged");
    }

    /// Same round-trip preservation on the generic blob-fallback path.
    /// equip_info has a stable layout and is widely deployed in the
    /// pabgb dumps used for testing.
    #[test]
    fn equip_info_apply_empty_intents_byte_perfect() {
        let Some((body, pabgh)) = fixture_pair("equip_info") else {
            eprintln!("SKIP equip_info_apply_empty_intents_byte_perfect: no fixture");
            return;
        };
        let (new_body, new_pabgh, outcomes) =
            apply_intents_to_table_body("equip_info", &body, Some(&pabgh), &[])
                .expect("apply");
        assert!(outcomes.is_empty());
        assert_eq!(new_body, body, "body diverged");
        assert_eq!(new_pabgh.expect("pabgh returned"), pabgh, "pabgh diverged");
    }

    /// Set-by-key on a real pabgh-bounded table should re-emit a clean
    /// pabgh whose offsets still match the body. We compare structurally
    /// (parse the new pabgh, verify each (key, offset) points to a valid
    /// entry start) rather than byte-comparing because a string-length
    /// change shifts downstream offsets.
    #[test]
    fn buff_info_set_is_blocked_then_pabgh_offsets_align() {
        let Some((body, pabgh)) = fixture_pair("buff_info") else {
            eprintln!("SKIP buff_info_set_is_blocked_then_pabgh_offsets_align: no fixture");
            return;
        };
        let mut records = parse_table_to_json("buff_info", &body, Some(&pabgh)).expect("parse");
        let target_idx = 0usize;
        let target_key = records[target_idx]
            .get("key")
            .and_then(|k| k.as_u64())
            .expect("buff_info record has scalar u32 key");

        let intent = crate::intents::Intent {
            entry: None,
            key: Some(target_key as i64),
            field: Some("is_blocked".into()),
            new: Some(serde_json::json!(1)),
            ..Default::default()
        };

        let (new_body, new_pabgh, outcomes) =
            apply_intents_to_table_body("buff_info", &body, Some(&pabgh), &[intent])
                .expect("apply");
        assert_eq!(outcomes.len(), 1);
        assert!(matches!(outcomes[0].status, crate::intents::ApplyStatus::Applied));

        // Same record count, byte size unchanged (is_blocked is u8).
        assert_eq!(new_body.len(), body.len());
        let new_pabgh = new_pabgh.expect("pabgh returned");
        assert_eq!(new_pabgh.len(), pabgh.len(), "pabgh size changed");

        // Re-parse using the fresh pabgh — must produce the same record
        // count and the targeted record now has is_blocked=1.
        let reparsed = parse_table_to_json("buff_info", &new_body, Some(&new_pabgh)).expect("re-parse");
        assert_eq!(reparsed.len(), records.len());
        let target_after = reparsed.iter()
            .find(|r| r.get("key").and_then(|k| k.as_u64()) == Some(target_key))
            .expect("target buff still present after apply");
        assert_eq!(target_after.get("is_blocked").and_then(|v| v.as_u64()), Some(1));
        // Touch records to suppress unused-mut warning.
        let _ = &mut records;
    }

    #[test]
    fn normalize_target_name_canonical() {
        assert_eq!(normalize_target_name("character_info"), Some("character_info"));
        assert_eq!(normalize_target_name("character_info.pabgb"), Some("character_info"));
        assert_eq!(normalize_target_name("buff_info"), Some("buff_info"));
    }

    #[test]
    fn normalize_target_name_compact() {
        // SuperMod-style compact-with-extension names.
        assert_eq!(normalize_target_name("characterinfo.pabgb"), Some("character_info"));
        assert_eq!(normalize_target_name("regioninfo.pabgb"), Some("region_info"));
        assert_eq!(
            normalize_target_name("spawningpoolautospawninfo.pabgb"),
            Some("spawning_pool_auto_spawn_info"),
        );
        assert_eq!(
            normalize_target_name("terrainregionautospawninfo.pabgb"),
            Some("terrain_region_auto_spawn_info"),
        );
    }

    #[test]
    fn normalize_target_name_iteminfo_aliases() {
        assert_eq!(normalize_target_name("iteminfo"), Some("iteminfo"));
        assert_eq!(normalize_target_name("iteminfo.pabgb"), Some("iteminfo"));
    }

    #[test]
    fn normalize_target_name_paloc_aliases() {
        assert_eq!(normalize_target_name("paloc"), Some("paloc"));
        assert_eq!(normalize_target_name("paloc.pamt"), Some("paloc"));
        assert_eq!(normalize_target_name("localizationstring"), Some("paloc"));
    }

    #[test]
    fn normalize_target_name_unknown() {
        assert_eq!(normalize_target_name("does_not_exist"), None);
        assert_eq!(normalize_target_name("doesnotexist.pabgb"), None);
    }

    /// Parse the user-supplied SuperMod fixture (12,358 intents) and
    /// verify every target resolves to a canonical name + every intent
    /// is well-formed. Doc-level only — does not require pabgb fixtures.
    #[test]
    fn supermod_manifest_parses_cleanly() {
        let candidates = [
            std::env::var("DMM_PARSER_SUPERMOD_PATH").ok(),
            Some(r"C:\Users\corin\Desktop\ZIPS\SuperMod (4).json".into()),
            Some(r"/mnt/c/Users/corin/Desktop/ZIPS/SuperMod (4).json".into()),
        ];
        let bytes = candidates.iter().flatten().find_map(|p| std::fs::read(p).ok());
        let Some(bytes) = bytes else {
            eprintln!("SKIP supermod_manifest_parses_cleanly: no fixture");
            return;
        };
        let doc = crate::intents::IntentDoc::from_slice(&bytes).expect("parse manifest");
        assert_eq!(doc.format, 3);
        assert_eq!(doc.format_minor, Some(1));
        assert!(doc.is_field_json_v3());

        let flat = doc.flatten_targets();
        assert!(!flat.is_empty(), "expected at least one target");
        let mut total_intents = 0usize;
        for (name, intents) in &flat {
            // Every target name must resolve.
            assert!(
                normalize_target_name(name).is_some(),
                "target '{}' did not resolve to a canonical table name",
                name,
            );
            // Every intent's op must be recognized.
            for intent in intents {
                intent.resolve_op().unwrap_or_else(|e| {
                    panic!("target '{}': intent failed to resolve: {}", name, e)
                });
            }
            total_intents += intents.len();
        }
        assert!(
            total_intents > 1000,
            "expected SuperMod-class manifest, got only {} intents",
            total_intents,
        );
        eprintln!("supermod: {} targets, {} intents — all resolve", flat.len(), total_intents);
    }

    /// Load `samples/04_custom_item/mod.field.json` and verify it parses,
    /// resolves all three targets, and resolves the custom-item
    /// clone_record intent. Catches schema drift between the docs and
    /// the parser.
    #[test]
    fn samples_04_custom_item_manifest_round_trip() {
        // Search relative to common cwd hosts so this works whether
        // cargo is run from the repo root or the dmm-parser dir.
        let candidates = [
            "samples/04_custom_item/mod.field.json",
            "dmm-parser/samples/04_custom_item/mod.field.json",
        ];
        let bytes = candidates
            .iter()
            .find_map(|p| std::fs::read(p).ok())
            .expect("samples/04_custom_item/mod.field.json must be readable");
        let doc = crate::intents::IntentDoc::from_slice(&bytes).expect("parse manifest");
        assert_eq!(doc.format, 3);
        assert_eq!(doc.format_minor, Some(1));
        assert!(doc.is_field_json_v3());

        let flat = doc.flatten_targets();
        assert_eq!(flat.len(), 3, "expected three targets: iteminfo, paloc, asset");

        // Iteminfo target with a clone_record intent.
        let iteminfo = flat.iter().find(|(n, _)| n == "iteminfo.pabgb").expect("iteminfo target");
        assert_eq!(iteminfo.1.len(), 1);
        let intent = &iteminfo.1[0];
        assert_eq!(intent.op.as_deref(), Some("clone_record"));
        assert_eq!(intent.source_key, Some(12345));
        assert_eq!(intent.new_key, Some(999_001));
        let resolved = intent.resolve_op().expect("resolve clone_record");
        assert!(matches!(resolved, crate::intents::ResolvedIntentOp::CloneRecord { .. }));

        // Verify the paloc indices in the manifest match what
        // item_paloc_indices(999001) computes — catches authoring errors
        // where the doc and the formula drift.
        let (name_idx, desc_idx) = crate::intents::item_paloc_indices(999_001);
        let patches = intent.patches.as_ref().expect("clone_record has patches");
        let name_patch = patches
            .iter()
            .find(|p| p.path == "item_name.index")
            .expect("item_name.index patch");
        let desc_patch = patches
            .iter()
            .find(|p| p.path == "item_desc.index")
            .expect("item_desc.index patch");
        assert_eq!(name_patch.new.as_u64(), Some(name_idx));
        assert_eq!(desc_patch.new.as_u64(), Some(desc_idx));
    }

    /// v3.1 shape projects snake_case → _camelCase for any aliased table.
    /// The `skill_info` table has known aliases (`cooltime` → `_cooltime`,
    /// `buff_level_list` → `_buffLevelList`, etc.) so confirm the lookup
    /// returns a non-empty table and the rename direction is correct.
    #[test]
    fn v3_1_alias_lookup_returns_skill_info_aliases() {
        let aliases = crate::json_shape::lookup_table_aliases_v3_1("skill_info")
            .expect("skill_info should be indexed");
        assert!(!aliases.is_empty(), "skill_info aliases should be non-empty");
        let cooltime = aliases.iter().find(|(s, _)| *s == "cooltime");
        assert_eq!(cooltime, Some(&("cooltime", "_cooltime")));
        let buff = aliases.iter().find(|(s, _)| *s == "buff_level_list");
        assert_eq!(buff, Some(&("buff_level_list", "_buffLevelList")));
    }

    /// Unknown table → no aliases. Caller treats as "no rename needed".
    #[test]
    fn v3_1_alias_lookup_returns_none_for_unknown_table() {
        assert!(crate::json_shape::lookup_table_aliases_v3_1("not_a_table").is_none());
    }

    /// Sequential tables don't get a pabgh back — verify the contract.
    #[test]
    fn sequential_table_returns_no_pabgh() {
        // Use a tiny synthetic body for a sequential table. action_point_info
        // was switched to pabgh_blob_table in 1.0.8; use action_restriction_order_info
        // instead — it is a fully-typed sequential table.
        let (new_body, new_pabgh, outcomes) =
            apply_intents_to_table_body("action_restriction_order_info", &[], None, &[])
                .expect("apply");
        assert!(new_body.is_empty());
        assert!(new_pabgh.is_none());
        assert!(outcomes.is_empty());
    }
}
