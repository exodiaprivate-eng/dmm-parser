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
            parse_typed_blob_table_to_json_with_pabgh(
                pabgb,
                ph,
                |data, offset, size| {
                    <$ty>::read_with_size(data, offset, size).map(|t| t.to_json_dict())
                },
                // No partial salvage for this table — a failed record keeps only
                // its key and `_blob_b64`, as before. Opt in with `pp!`.
                |_data, _start, _size| ::serde_json::Map::new(),
            )?
        }};
    }

    /// Like `p!` but SALVAGES the typed prefix when a record fails to decode.
    ///
    /// Use for tables where a late unmodelled field would otherwise hide fields
    /// that parse perfectly. characterinfo is the motivating case: a break at
    /// field 153 buried 232 records — every pet cat and dog, and the kakapo —
    /// whose name/desc/lookup_22/f38 live at fields 3/4/24/42.
    ///
    /// Requires `read_partial_json` on the type (emitted by the blob-table
    /// macros). The salvaged fields are READ-ONLY: `_blob_fallback` still routes
    /// the record to the verbatim blob writer, so roundtrip is untouched.
    macro_rules! pp {
        ($ty:path) => {{
            let ph = pabgh.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput,
                format!("table '{}' requires a pabgh file", table_name)))?;
            parse_typed_blob_table_to_json_with_pabgh(
                pabgb,
                ph,
                |data, offset, size| {
                    <$ty>::read_with_size(data, offset, size).map(|t| t.to_json_dict())
                },
                |data, start, size| <$ty>::read_partial_json(data, start, size),
            )?
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
        "character_info"                 => pp!(crate::tables::character_info::CharacterInfo),
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
        "stage_info"                     => pp!(crate::tables::stage_info::StageInfo),
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
        "game_start_info"                => s!(crate::tables::game_start_info::GameStartInfo),
        "zone_info"                      => s!(crate::tables::zone_info::ZoneInfo),
        "contents_phase_info"            => s!(crate::tables::contents_phase_info::ContentsPhaseInfo),
        "faction_reblockading_info"      => s!(crate::tables::faction_reblockading_info::FactionReblockadingInfo),
        "quick_slot_info"                => s!(crate::tables::quick_slot_info::QuickSlotInfo),
        "bank_info"                      => s!(crate::tables::bank_info::BankInfo),
        "talk_tree_info"                 => s!(crate::tables::talk_tree_info::TalkTreeInfo),
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
        // 1.12: Tier-1.5 (typed prefix + opaque tail) — pabgh-bounded so the
        // record size is known. See npc_activity_info::info for the rationale.
        "npc_activity_info"              => p!(crate::tables::npc_activity_info::NpcActivityInfo),
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
            serialize_typed_blob_table_from_json(
                json_items,
                |w, map| <$ty>::write_from_json_dict(w, map),
                |_w, _map, _n| Err(io::Error::new(
                    io::ErrorKind::Unsupported, "no partial writer for this table")),
            )?
        };
    }

    /// Like `d!` but lets an edit to a SALVAGED prefix field land on a
    /// blob-fallback record (see `pp!` on the parse side). Opt-in per table.
    macro_rules! dp {
        ($ty:path) => {
            serialize_typed_blob_table_from_json(
                json_items,
                |w, map| <$ty>::write_from_json_dict(w, map),
                |w, map, n| <$ty>::write_partial_prefix(w, map, n),
            )?
        };
    }

    let table_name = normalize_target_name(table_name).unwrap_or(table_name);

    Ok(match table_name {
        // ── pabgh-bounded tables ──────────────────────────────────────────
        "ai_dialog_string_info"          => d!(crate::tables::ai_dialog_string_info::AIDialogStringInfo),
        "bitmap_position_info"           => d!(crate::tables::bitmap_position_info::BitmapPositionInfo),
        "buff_info"                      => d!(crate::tables::buff_info::BuffInfo),
        "character_change_info"          => d!(crate::tables::character_change_info::CharacterChangeInfo),
        "character_info"                 => dp!(crate::tables::character_info::CharacterInfo),
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
        "stage_info"                     => dp!(crate::tables::stage_info::StageInfo),
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
        "game_start_info"                => d!(crate::tables::game_start_info::GameStartInfo),
        "zone_info"                      => d!(crate::tables::zone_info::ZoneInfo),
        "contents_phase_info"            => d!(crate::tables::contents_phase_info::ContentsPhaseInfo),
        "faction_reblockading_info"      => d!(crate::tables::faction_reblockading_info::FactionReblockadingInfo),
        "quick_slot_info"                => d!(crate::tables::quick_slot_info::QuickSlotInfo),
        "bank_info"                      => d!(crate::tables::bank_info::BankInfo),
        "talk_tree_info"                 => d!(crate::tables::talk_tree_info::TalkTreeInfo),
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
        "game_version_data_info"         => d!(crate::tables::game_version_data_info::GameVersionDataInfo),
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
        "npc_activity_group_info"        => d!(crate::tables::npc_activity_group_info::NpcActivityGroupInfo),
        "npc_activity_info"              => d!(crate::tables::npc_activity_info::NpcActivityInfo),
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
    // Normalize third-party-exporter field-name aliases (e.g. CrimsonGameMods
    // DropSets `drops` → `list`) so their intents resolve instead of silently
    // dropping. Snake-named intents and tables without community aliases pass
    // through unchanged, so this is a no-op for every existing mod.
    let intents_norm: Vec<crate::intents::Intent> = intents
        .iter()
        .map(|i| {
            let mut c = i.clone();
            crate::intents::normalize_intent_community(&mut c, table_name);
            c
        })
        .collect();
    let outcomes = crate::intents::apply_resolved_intents(&mut records, &intents_norm)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("apply: {}", e)))?;

    let (new_body, new_pabgh) = if let Some(pabgh_bytes) = pabgh {
        match serialize_table_from_json_with_pabgh(table_name, &records, pabgh_bytes) {
            Ok((new_body, new_pabgh)) => (new_body, Some(new_pabgh)),
            // Self-bounded (count-record) tables — e.g. wanted_info, equip_type_info —
            // have a sister .pabgh on disk but don't use it for record bounds, so the
            // tracked serializer has no rebuild path and errors. A caller that reads
            // the pabgh purely because the file exists (DMM's mount path) shouldn't be
            // penalized: serialize sequentially and pass the pabgh back unchanged (a
            // same-length field edit leaves any offsets it holds valid).
            //
            // That reasoning holds only for a table whose PARSE ignores the pabgh —
            // a truly sequential table can grow freely, because nothing reads the
            // index for record bounds. It does NOT hold for a table that parses
            // through the pabgh and merely lacks a tracked serializer: there, this
            // arm returns a resized body with a stale index and an Ok, so the caller
            // cannot tell. That is exactly how action_point_info / field_info /
            // npc_activity_info silently swallowed an added record — all three parse
            // via `p!` but were missing from serialize_table_from_json_tracked.
            // Registering them fixed those three; the guard below is what stops the
            // next omission being silent instead of loud.
            Err(e)
                if e.kind() == io::ErrorKind::InvalidInput
                    && e.to_string().contains("not pabgh-bounded") =>
            {
                let new_body = serialize_table_from_json(table_name, &records)?;
                if new_body.len() != body.len() && table_requires_pabgh(table_name) {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, format!(
                        "table '{}' is read through its .pabgh but has no entry in \
                         serialize_table_from_json_tracked, and this edit resized the body \
                         ({} -> {} bytes). Returning the original index would leave every \
                         offset after the edit pointing into the wrong record. Register \
                         '{}' in serialize_table_from_json_tracked.",
                        table_name, body.len(), new_body.len(), table_name)));
                }
                (new_body, Some(pabgh_bytes.to_vec()))
            }
            Err(e) => return Err(e),
        }
    } else {
        (serialize_table_from_json(table_name, &records)?, None)
    };

    // ── Post-condition for record-CREATING ops ────────────────────────────
    // `clone_record` / `new_record` promise a record under a caller-chosen key.
    // Nothing on the write path enforces that promise: the key lands in a typed
    // field, and an over-wide value is CLAMPED there (json_traits `[V3_CLAMP]`,
    // deliberate — one bad set-value must not drop every other mod sharing the
    // overlay). Harmless for an ordinary field edit; silently wrong for a KEY,
    // because the record is then created under a different identity than the
    // manifest asked for — 65535 or 255 — which on most tables already belongs
    // to a real record. Measured across the 1.18 fixtures: 31 tables clamp a
    // 990001-style custom key, every one of them reporting "Applied".
    //
    // So read the answer back off the bytes we just produced. This is the only
    // check that sees the wire truth rather than the JSON we hoped to write, and
    // it catches the whole family at once — clamped key, key collision, stale
    // index, record swallowed by its predecessor. It only runs when a creating
    // op is present, so ordinary field mods pay nothing.
    let created: Vec<i64> = intents_norm
        .iter()
        .filter(|i| matches!(i.op.as_deref(),
                             Some("clone_record") | Some("new_record") | Some("add_entry")))
        .filter_map(|i| i.new_key)
        .collect();
    if !created.is_empty() {
        verify_created_keys_landed(table_name, &new_body, new_pabgh.as_deref(), &created)?;
    }

    Ok((new_body, new_pabgh, outcomes))
}

/// Re-read `body` and confirm every key a record-creating intent asked for is
/// present, exactly once.
///
/// Reports a diagnosis rather than a bare "not found" — the caller is a mod
/// manager relaying to a mod author, and "that key is wider than this table's
/// key field" is the answer for 31 of the 124 tables.
fn verify_created_keys_landed(
    table_name: &str,
    body: &[u8],
    pabgh: Option<&[u8]>,
    created: &[i64],
) -> io::Result<()> {
    let records = parse_table_to_json(table_name, body, pabgh).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, format!(
            "table '{}': a record-creating intent produced a body this parser can no \
             longer read ({}). Refusing to hand back a table we cannot verify.",
            table_name, e))
    })?;
    for want in created {
        let n = records.iter().filter(|r| record_key_of(r) == Some(*want)).count();
        if n == 1 {
            continue;
        }
        let hint = if n > 1 {
            " — the key resolves to more than one record; it collides with an existing one."
        } else if *want > u16::MAX as i64 {
            " — the key is too wide for this table's key field and was clamped on write \
             (to 65535 or 255), so the record exists under a different identity. Choose a \
             new_key that fits the table's key width."
        } else {
            " — the record was written but does not read back at that key."
        };
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!(
            "table '{}': record-creating intent asked for key {}, but the written body \
             contains it {} time(s){}",
            table_name, want, n, hint)));
    }
    Ok(())
}

/// Does this table's PARSE need the sister `.pabgh` for record boundaries?
///
/// Answered by asking the dispatcher itself rather than by keeping a third
/// hand-maintained list beside `parse_table_to_json`'s `p!` arms and
/// `serialize_table_from_json_tracked`'s match — two lists have already drifted
/// apart once, and a third would only widen the gap. Every pabgh-bounded arm
/// resolves its pabgh argument before it looks at the body, so an empty body
/// with no pabgh separates the two families in O(1): pabgh-bounded tables fail
/// with "requires a pabgh file", sequential ones read zero records and succeed.
fn table_requires_pabgh(table_name: &str) -> bool {
    matches!(parse_table_to_json(table_name, &[], None),
             Err(e) if e.kind() == io::ErrorKind::InvalidInput)
}

/// Read a record's key, unwrapping iteminfo's `{"value": N}` shape.
fn record_key_of(v: &serde_json::Value) -> Option<i64> {
    v.get("key")
        .and_then(|k| k.get("value").or(Some(k)))
        .and_then(|x| x.as_i64())
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

    // Serialize FIRST, parse the index second. A sequential table has a sister
    // .pabgh on disk that nothing reads for record bounds, and callers pass it
    // simply because the file exists. Parsing it up front made such a table fail
    // on a file it does not use: gamestartinfo.pabgh is 6 bytes and matches none
    // of the six known layouts, so every edit to game_start_info died with
    // "pabgh size 6 doesn't match any known layout" instead of taking the
    // sequential path. Dispatching first lets the honest "not pabgh-bounded"
    // answer reach the caller, which knows what to do with it.
    let (new_body, offsets) = serialize_table_from_json_tracked(table_name, items)?;

    let original = Pabgh::parse(original_pabgh)?;
    let format = original.format;

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
            serialize_typed_blob_table_from_json_tracked(
                items,
                |w, map| <$ty>::write_from_json_dict(w, map),
                |_w, _map, _n| Err(io::Error::new(
                    io::ErrorKind::Unsupported, "no partial writer for this table")),
            )?
        };
    }

    /// Tracked sister of `dp!` — THIS is the path the v3 apply pipeline uses.
    macro_rules! dtp {
        ($ty:path) => {
            serialize_typed_blob_table_from_json_tracked(
                items,
                |w, map| <$ty>::write_from_json_dict(w, map),
                |w, map, n| <$ty>::write_partial_prefix(w, map, n),
            )?
        };
    }

    Ok(match table_name {
        // ★ These three parse through the pabgh (`p!`) but were missing here,
        // so `serialize_table_from_json_with_pabgh` bailed with "not
        // pabgh-bounded" and the caller's fallback arm shipped the body with
        // the VANILLA index. Any record-count or record-size change then left
        // every offset after the edit pointing at the wrong bytes — an added
        // record was simply swallowed by whichever record preceded it. See the
        // registration gate `every_pabgh_parsed_table_has_a_tracked_serializer`.
        "action_point_info"              => dt!(crate::tables::action_point_info::ActionPointInfo),
        "field_info"                     => dt!(crate::tables::field_info::FieldInfo),
        "npc_activity_info"              => dt!(crate::tables::npc_activity_info::NpcActivityInfo),
        "ai_dialog_string_info"          => dt!(crate::tables::ai_dialog_string_info::AIDialogStringInfo),
        "bitmap_position_info"           => dt!(crate::tables::bitmap_position_info::BitmapPositionInfo),
        "buff_info"                      => dt!(crate::tables::buff_info::BuffInfo),
        "character_change_info"          => dt!(crate::tables::character_change_info::CharacterChangeInfo),
        "character_info"                 => dtp!(crate::tables::character_info::CharacterInfo),
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
        "stage_info"                     => dtp!(crate::tables::stage_info::StageInfo),
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

    // Skillinfo is named Skill in v1.0.8
    if matches!(stripped, "skill") {
        return Some("skill_info");
    }

    // FactionNodeInfo is named factionnode in v1.0.8
    if matches!(stripped, "factionnode") {
        return Some("faction_node_info");
    }

    // FactionGroupInfo is named factiongroup in v1.0.8
    if matches!(stripped, "factiongroup") {
        return Some("faction_group_info");
    }

    // A handful of game filenames abbreviate the canonical name beyond a
    // simple "_info"-drop (the generic rule below handles the common case),
    // so map these onto their parser explicitly. Verified by parsing the
    // live 0008 client tables: entitlementinfo→platform_entitlement_info,
    // keymap→key_map_setting_list_info, levelinfo→game_level_info,
    // reviepointinfo→field_revive_info.
    match stripped {
        "entitlementinfo" => return Some("platform_entitlement_info"),
        "keymap" => return Some("key_map_setting_list_info"),
        "levelinfo" => return Some("game_level_info"),
        "reviepointinfo" => return Some("field_revive_info"),
        _ => {}
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
    // "characterinfo" → "character_info"). Most 1.0.8+ client filenames also
    // drop the trailing "_info" word (e.g. "gameplaytrigger" →
    // "game_play_trigger_info", "inventory" → "inventory_info"), so try both
    // the full canonical and the canonical-without-"_info", each compared
    // ignoring underscores. Done O(N) — the table list is ~120 entries; if
    // this becomes hot we can lazy-init a HashMap. Verified collision-free
    // across all canonical names. (This also subsumes the skill/factionnode/
    // factiongroup special-cases above, kept for explicitness.)
    for &canonical in supported_tables() {
        if eq_ignoring_underscores(stripped, canonical) {
            return Some(canonical);
        }
        let info_dropped_match = canonical
            .strip_suffix("_info")
            .is_some_and(|base| eq_ignoring_underscores(stripped, base));
        if info_dropped_match {
            return Some(canonical);
        }
    }

    None
}

/// Compare two byte strings for equality after dropping every `_`.
/// Used by [`normalize_target_name`] to match a compact game filename
/// (`gameplaytrigger`) against a snake_case canonical (`game_play_trigger`).
fn eq_ignoring_underscores(a: &str, b: &str) -> bool {
    let mut ai = a.bytes().filter(|&c| c != b'_');
    let mut bi = b.bytes().filter(|&c| c != b'_');
    loop {
        match (ai.next(), bi.next()) {
            (Some(x), Some(y)) if x == y => continue,
            (None, None) => return true,
            _ => return false,
        }
    }
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
        "action_point_info", "ai_dialog_string_info", "bitmap_position_info",
        "buff_info",
        "character_change_info", "character_info", "condition_info",
        "drop_set_info", "effect_info", "elemental_material_info",
        "equip_info", "equip_slot_info", "faction_info", "faction_node_info",
        "faction_node_spawn_info", "faction_spawn_data_info",
        "field_info", "field_revive_info", "frame_event_attr_group_info",
        "game_event_handler_info", "game_global_effect_info",
        "game_level_info", "game_play_trigger_info", "gimmick_group_info",
        "gimmick_info", "global_game_event_info", "global_stage_sequencer_info",
        "interaction_info", "inventory_info", "item_use_info",
        "knowledge_info", "level_gimmick_scene_object_info",
        "mini_game_data_info", "mission_info", "multi_change_info",
        "npc_activity_info", "npc_info", "platform_entitlement_info",
        "quest_info", "region_info",
        "royal_supply_info", "sequencer_spawn_info", "skill_info",
        "spawning_pool_auto_spawn_info", "special_mode_info", "stage_info",
        "store_info", "sub_level_info", "terrain_region_auto_spawn_info",
        // localization
        "paloc", "paloc.pamt", "localizationstring",
        // sequential
        "action_restriction_order_info",
        "aiaction_attribute_info", "aidialog_type_info", "aievent_table_info",
        "aimemory_info", "aimove_speed_info", "ally_group_info",
        "game_start_info", "zone_info", "contents_phase_info",
        "faction_reblockading_info", "quick_slot_info", "bank_info", "talk_tree_info",
        "auto_spawn_filter_info", "board_info", "breakable_object_info",
        "category_group_info", "category_info", "character_appearance_index_info",
        "character_group_info", "craft_tool_group_info", "craft_tool_info",
        "detect_detail_info", "detect_info", "detect_reaction_info",
        "dialog_voice_info", "dye_color_group_info", "equip_type_info",
        "faction_group_info", "faction_relation_group_info",
        "faction_waypoint_info", "fail_message_info",
        "field_level_name_table_info", "formation_info",
        "game_advice_group_info", "game_advice_info", "game_play_variable_info",
        "game_version_data_info",
        "gimmick_event_table_info", "gimmick_gate_connection_info",
        "gimmick_gate_info", "global_game_event_group_info", "house_info",
        "item_group_info", "job_info", "key_map_setting_list_info",
        "knowledge_group_info", "level_action_point_info", "local_string_info",
        "material_blood_decal_info", "material_match_info",
        "material_relation_info", "mercenary_group_info", "mercenary_info",
        "npc_activity_group_info",
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

    /// AUTHORITATIVE V3 coverage check: run the real V3 entry point
    /// (`apply_intents_to_table_body` with empty intents = parse → serialize)
    /// over EVERY supported table against a fixture dir, and report which
    /// tables fail to parse or don't round-trip byte-perfect. Set
    /// DMM_PARSER_V3_DIR to the fixture dir (e.g. the 1.11 extraction).
    /// Fixtures are named compactly (buff_info → buffinfo.pabgb).
    #[test]
    #[ignore]
    fn v3_all_tables_against_fixture_dir() {
        let dir = std::env::var("DMM_PARSER_V3_DIR")
            .unwrap_or_else(|_| r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-6-11".into());
        let base = std::path::PathBuf::from(&dir);
        let mut ok = Vec::new();
        let mut fail = Vec::new();
        let mut skip = Vec::new();

        // Unique canonical tables (skip paloc aliases + file-format JSON-pending).
        let mut names: Vec<&str> = supported_tables().iter().copied()
            .filter(|n| !matches!(*n, "paloc.pamt" | "localizationstring"
                | "paac" | "paatt" | "pamhc" | "pappt"))
            .collect();
        names.push("iteminfo");
        names.sort(); names.dedup();

        for name in names {
            // The game names fixtures with short forms that vary: most use the
            // compact (underscores stripped) name, but some drop a trailing
            // "info" (skill_info → skill, faction_info → faction). Try candidates
            // in order and take the first present.
            let compact = name.replace('_', "");
            let mut cands = vec![compact.clone()];
            if let Some(stem) = compact.strip_suffix("info") {
                cands.push(stem.to_string());
            }
            // A couple of heavily-truncated game names.
            cands.push(name.split('_').next().unwrap_or(name).to_string());
            // Explicit aliases where the game's short fixture name doesn't follow
            // either rule above (verified against the 0008 bin listing).
            for (canon, stem) in [
                ("key_map_setting_list_info", "keymap"),
                ("game_level_info", "levelinfo"),
                ("platform_entitlement_info", "entitlementinfo"),
            ] {
                if name == canon { cands.insert(0, stem.to_string()); }
            }
            // No client 0008 .pabgb exists for these (can't be V3-modded); paloc
            // is the localization path tested elsewhere. Don't report as gaps.
            if matches!(name, "equip_info" | "field_revive_info" | "paloc") {
                continue;
            }
            let found = cands.iter().find_map(|c| {
                let p = base.join(format!("{}.pabgb", c));
                std::fs::read(&p).ok().map(|b| (c.clone(), b))
            });
            let Some((stem, body)) = found else { skip.push(name); continue; };
            let pabgh = std::fs::read(base.join(format!("{}.pabgh", stem))).ok();
            // The real V3 read path. parse_table_to_json uses pabgh only for
            // pabgh-bounded tables; sequential tables ignore it. iteminfo is
            // routed through apply_intents_to_table_body's special case.
            let parsed = if name == "iteminfo" {
                crate::intents::apply_intents_to_iteminfo(&body, &[]).map(|_| ())
            } else {
                parse_table_to_json(name, &body, pabgh.as_deref()).map(|_| ())
            };
            match parsed {
                Ok(()) => ok.push(name),
                Err(e) => fail.push((name, e.to_string())),
            }
        }
        eprintln!("\n=== V3 coverage vs {} ===", dir);
        eprintln!("OK: {}  FAIL: {}  SKIP(no fixture): {}", ok.len(), fail.len(), skip.len());
        if !skip.is_empty() { eprintln!("SKIP: {:?}", skip); }
        eprintln!("--- FAILURES ---");
        for (n, e) in &fail { eprintln!("  {:<34} {}", n, e); }
        assert!(fail.is_empty(), "{} V3 tables broken on this build", fail.len());
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
    fn normalize_target_name_info_dropped() {
        // 1.0.8+ client filenames drop the trailing "_info" word. Verified
        // against the live 0008 client table list (126 .pabgb files). The
        // first three also have explicit special-cases; assert the generic
        // rule agrees so removing those cases later can't regress silently.
        for (filename, canonical) in [
            ("skill", "skill_info"),
            ("factionnode", "faction_node_info"),
            ("factiongroup", "faction_group_info"),
            ("bitmapposition", "bitmap_position_info"),
            ("board", "board_info"),
            ("characterchange", "character_change_info"),
            ("faction", "faction_info"),
            ("factionrelationgroup", "faction_relation_group_info"),
            ("factionwaypoint", "faction_waypoint_info"),
            ("gameeventhandler", "game_event_handler_info"),
            ("gameplaytrigger", "game_play_trigger_info"),
            ("gimmickgateconnection", "gimmick_gate_connection_info"),
            ("globalgameevent", "global_game_event_info"),
            ("globalgameeventgroup", "global_game_event_group_info"),
            ("inventory", "inventory_info"),
            ("reserveslot", "reserve_slot_info"),
            ("royalsupply", "royal_supply_info"),
            ("specialmode", "special_mode_info"),
            ("uisocialaction", "ui_social_action_info"),
            ("validscheduleaction", "valid_schedule_action_info"),
        ] {
            assert_eq!(
                normalize_target_name(&format!("{filename}.pabgb")),
                Some(canonical),
                "filename '{filename}' should resolve to '{canonical}'",
            );
        }
    }

    #[test]
    fn normalize_target_name_abbreviated_aliases() {
        // Filenames that abbreviate beyond a simple "_info"-drop; verified by
        // parsing the live 0008 client tables.
        assert_eq!(normalize_target_name("entitlementinfo.pabgb"), Some("platform_entitlement_info"));
        assert_eq!(normalize_target_name("keymap.pabgb"), Some("key_map_setting_list_info"));
        assert_eq!(normalize_target_name("levelinfo.pabgb"), Some("game_level_info"));
        assert_eq!(normalize_target_name("reviepointinfo.pabgb"), Some("field_revive_info"));
        // factionoperationgroupinfo has no parser — must stay unresolved.
        assert_eq!(normalize_target_name("factionoperationgroupinfo.pabgb"), None);
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
    /// Every table read through its `.pabgh` must also be registered in
    /// `serialize_table_from_json_tracked`.
    ///
    /// The two live in separate match statements and drifted apart once:
    /// action_point_info, field_info and npc_activity_info parsed via `p!` but
    /// had no tracked serializer, so `apply_intents_to_table_body` fell into the
    /// "not pabgh-bounded" arm and handed back a resized body paired with the
    /// VANILLA index. An added record was then swallowed by whichever record
    /// preceded it — no error, no warning, the mod simply did nothing.
    ///
    /// Probing rather than listing: a third hand-kept list would be a third
    /// thing to drift. An empty body with no pabgh separates the families,
    /// because every pabgh-bounded arm resolves its pabgh before it reads the
    /// body; a two-byte zero-count pabgh is the smallest valid index, and the
    /// tracked serializer either recognises the table or says it does not.
    #[test]
    fn every_pabgh_parsed_table_has_a_tracked_serializer() {
        const EMPTY_PABGH: [u8; 2] = [0, 0]; // u16 count = 0
        let mut unregistered = Vec::new();
        for &table in crate::dispatch::supported_tables() {
            let needs_pabgh = matches!(
                crate::dispatch::parse_table_to_json(table, &[], None),
                Err(ref e) if e.kind() == std::io::ErrorKind::InvalidInput);
            if !needs_pabgh {
                continue;
            }
            if let Err(e) = crate::dispatch::serialize_table_from_json_with_pabgh(
                table, &[], &EMPTY_PABGH)
                && e.to_string().contains("not pabgh-bounded")
            {
                unregistered.push(table);
            }
        }
        assert!(unregistered.is_empty(),
            "these tables are read through their .pabgh but have no tracked serializer, \
             so any size-changing edit would ship a stale index: {:?}. Add them to \
             serialize_table_from_json_tracked.", unregistered);
    }

    /// The mirror of the gate above: a table advertised as pabgh-bounded in
    /// `supported_tables()` must actually be one. The halves of that list are
    /// what callers use to decide whether to hand us a pabgh at all.
    #[test]
    fn supported_tables_pabgh_half_matches_reality() {
        // The pabgh-bounded half runs from the start of the list up to "paloc".
        let all = crate::dispatch::supported_tables();
        let split = all.iter().position(|&t| t == "paloc").expect("paloc marks the boundary");
        let mut misfiled = Vec::new();
        for (i, &table) in all.iter().enumerate() {
            let needs_pabgh = matches!(
                crate::dispatch::parse_table_to_json(table, &[], None),
                Err(ref e) if e.kind() == std::io::ErrorKind::InvalidInput);
            // equip_slot_info is pabgh-bounded through its own hand-written
            // parser rather than a `p!` arm, so it is correctly in the first
            // half even though the probe cannot see it there.
            if table == "equip_slot_info" {
                continue;
            }
            if needs_pabgh != (i < split) {
                misfiled.push((table, needs_pabgh));
            }
        }
        assert!(misfiled.is_empty(),
            "supported_tables() files these on the wrong side of the pabgh boundary \
             (name, actually_needs_pabgh): {:?}", misfiled);
    }

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

    #[test]
    fn probe_modded_overlays_reparse_clean() {
        let d12 = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-6-19";
        let mods = r"C:\Users\justi\Desktop\DMM\mods";
        let cases = [
            ("regioninfo", "regioninfo.pabgb", format!(r"{}\Mounts Everywhere.json", mods)),
            ("inventory",  "inventory.pabgb",  format!(r"{}\I like Space V3.json", mods)),
            ("buff_info",  "buffinfo.pabgb",   format!(r"{}\Ez_Trust_Dark_Fog_Lantern\Ez_Trust_DarkFogLantern.json", mods)),
        ];
        for (tname, file, mjpath) in cases {
            let Ok(raw) = std::fs::read(&mjpath) else { eprintln!("[{}] SKIP (no mod json {})", tname, mjpath); continue; };
            let doc: serde_json::Value = serde_json::from_slice(&raw).unwrap();
            // collect intents for this file (v3.1 targets[] or v3.0 target+intents)
            let mut intent_vals: Vec<serde_json::Value> = vec![];
            if let Some(ts) = doc.get("targets").and_then(|v| v.as_array()) {
                for t in ts {
                    if t.get("file").and_then(|v| v.as_str()).map(|s| s.eq_ignore_ascii_case(file)).unwrap_or(false) {
                        if let Some(arr) = t.get("intents").and_then(|v| v.as_array()) { intent_vals.extend(arr.iter().cloned()); }
                    }
                }
            } else if let Some(arr) = doc.get("intents").and_then(|v| v.as_array()) {
                intent_vals.extend(arr.iter().cloned());
            }
            let intents: Vec<crate::intents::Intent> = intent_vals.iter()
                .map(|v| crate::intents::Intent::from_value(v).unwrap()).collect();

            let body = std::fs::read(format!(r"{}\{}", d12, file)).unwrap();
            let ph = std::fs::read(format!(r"{}\{}", d12, file).replace(".pabgb", ".pabgh")).unwrap();
            let vanilla_n = parse_table_to_json(tname, &body, Some(&ph)).unwrap().len();

            match apply_intents_to_table_body(file, &body, Some(&ph), &intents) {
                Ok((nb, np, _outs)) => {
                    // RE-PARSE the modded output (this is what the game must read)
                    match parse_table_to_json(tname, &nb, np.as_deref()) {
                        Ok(recs) => {
                            let blob = recs.iter().filter(|r| r.as_object()
                                .map(|o| o.contains_key("_blob_fallback") || o.contains_key("_blob_b64") || o.contains_key("_tail_b64"))
                                .unwrap_or(false)).count();
                            eprintln!("[{}] {} intents → modded {}B: re-parse {} recs (vanilla {}), blob_fallback={} {}",
                                tname, intents.len(), nb.len(), recs.len(), vanilla_n, blob,
                                if recs.len()==vanilla_n && blob==0 {"OK"} else {"*** CORRUPT ***"});
                        }
                        Err(e) => eprintln!("[{}] modded body FAILS RE-PARSE *** CORRUPT ***: {}", tname, e),
                    }
                }
                Err(e) => eprintln!("[{}] apply ERROR: {}", tname, e),
            }
        }
    }
}
