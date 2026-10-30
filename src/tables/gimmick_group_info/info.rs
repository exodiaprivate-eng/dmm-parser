//! Tier 1 — fully typed (no _tail_b64).
//!
//! 2.04.00 reader: Mac `sub_102045548`, names from its Korean assert strings. 2.04 moved
//! stringKey first and gathered the 41 one-byte fields (bools and small enums) right after
//! isBlocked; `_gimmickChartPath` became a StringInfoKey (was a CString),
//! `_gimmickRemoteCatchableData` gained a presence byte, `_applyFurnitureObstacleInHousing`
//! and `_equipDockingGimmickType` are new, and `_levelAliasNameList[].activeExpression` is
//! the 2.04 flattened GameExpression (`GameExpression204`).
//!
//! The pre-2.04 model named about half the record by position (`flag_28_4`, `raw_340`, ...);
//! nothing in DMM addressed it, so the game's names are used throughout.

use crate::binary::*;
use crate::binary::variants::game_expression_204::GameExpression204;
use crate::py_binary_struct;

py_binary_struct! {
    /// `_linkSignalGroupList` element (Mac sub_1020442BC).
    pub struct GimmickLinkSignalGroupData<'a> {
        pub link_signal_group_name: CString<'a>,
        pub type_: u8,
        pub complete_count: u32,
        pub counter_filter_type: u8,
        pub link_signal_on_event_key: u32,        // StringInfoKey
        pub link_signal_off_event_key: u32,       // StringInfoKey
        pub random_delay_min: u64,
        pub random_delay_max: u64,
    }
}

py_binary_struct! {
    /// `_gimmickRemoteCatchableData` body (Mac sub_102083714, inside a presence byte).
    pub struct GimmickRemoteCatchableData {
        pub control_base_direction: [f32; 3],
        pub gimmick_remote_catch_type: u8,
        pub use_cross_hair_position: u8,
    }
}

py_binary_struct! {
    /// `_autoTargetingConstraintDataList` / `_gimmickConstraintDataList` element
    /// (Mac sub_102044420).
    pub struct GimmickConstraintData<'a> {
        pub constraint_name: CString<'a>,
        pub constraint_type: u8,
        pub target_type: u8,
        pub use_trigger_auto_target: u8,
        pub disable_collide_with_target: u8,
        pub failable: u8,
        pub breaking_threshold: f32,
        pub pivot_position: [f32; 3],
        pub pivot_rotation: [f32; 3],
        pub self_socket_name: CString<'a>,
        pub target_socket_name_list: CArray<CString<'a>>,
    }
}

py_binary_struct! {
    /// `_gameEventHandlerList` element (Mac sub_102044BBC).
    pub struct GimmickGameEventHandlerData {
        pub game_event_type: u8,
        pub condition_info: u32,
        pub gimmick_event_key: u32,
    }
}

py_binary_struct! {
    /// `_unlockableIDataList` element (Mac sub_102084124).
    pub struct UnlockableData {
        pub key: u32,
        pub unlockable_item_info_list: CArray<u32>,
        pub unlockable_mission_info_list: CArray<u32>,
        pub default_locked: u8,
    }
}

py_binary_struct! {
    /// `_levelAliasNameList` element (Mac sub_102084748).
    pub struct GimmickLevelAliasData<'a> {
        pub level_name: CString<'a>,
        pub level_alias_name: CString<'a>,
        pub active_expression: GameExpression204<'a>,
    }
}

py_binary_struct! {
    /// `_uiMapTextureEnableDataList` element (Mac sub_1020831B0): a GameEventExecuteData
    /// (sub_10203D6C4) then a u8.
    pub struct UiMapTextureEnableData {
        pub game_event_type: u8,
        pub player_condition: u32,
        pub target_condition: u32,
        pub event_condition: u32,
        pub enable: u8,
    }
}

py_binary_struct! {
    /// `_gimmickAliasDataList` element (Mac sub_1020448A0).
    pub struct GimmickAliasData {
        pub self_spawn_reason: u32,
        pub target_spawn_reason: u32,
        pub need_knowledge_info: u32,
        pub condition_info: u32,
        pub need_knowledge_level: u32,
        pub target_game_play_trigger_info: u32,
        pub ui_map_texture_enable_data_list: CArray<UiMapTextureEnableData>,
    }
}

py_binary_struct! {
    /// `_gimmickNodeData` (Mac sub_101EC910C).
    pub struct GimmickNodeData<'a> {
        pub name_a: CString<'a>,
        pub name_b: CString<'a>,
        pub u32_16: u32,
        pub u32_20: u32,
        pub u32_24: u32,
    }
}

py_binary_struct! {
    /// `_combinationAliasDataList` element (Mac sub_102084E14 inline).
    pub struct GimmickCombinationAliasData<'a> {
        pub combination_alias_name: CString<'a>,
        pub collide_with_parent: u8,
        pub parent_socket_name: CString<'a>,
        pub child_socket_name: CString<'a>,
    }
}

py_binary_struct! {
    /// `_combinationLinkDataList` element (Mac sub_1020445C0).
    pub struct GimmickCombinationLinkData<'a> {
        pub from_gimmick_group_info: u32,
        pub from_gimmick_group_index: u32,
        pub from_gimmick_alias_name: CString<'a>,
        pub is_from_self: u8,
        pub to_gimmick_group_info: u32,
        pub to_gimmick_group_index: u32,
        pub to_gimmick_alias_name: CString<'a>,
        pub is_to_self: u8,
        pub link_signal_group_name: CString<'a>,
        pub from_state_hash: u32,
    }
}

py_binary_struct! {
    /// `_elementalMaterialInfoList` element (Mac sub_102085624): two 4-byte keys.
    pub struct GimmickElementalMaterialEntry {
        pub key_a: u32,
        pub key_b: u32,
    }
}

py_binary_struct! {
    /// `_elementalStatusInitialStatList` element (Mac sub_102085724): key + f32.
    pub struct GimmickElementalStatusInitialStat {
        pub stat: u32,
        pub value: f32,
    }
}

py_binary_struct! {
    pub struct GimmickGroupInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u32,
        pub is_blocked: u8,
        pub is_default_spawn_distance_level: u8,
        pub is_macro_gimmick: u8,
        pub is_pierced_ally_projectile: u8,
        pub use_parent_gimmick_point: u8,
        pub is_spawn_component_in_level: u8,
        pub is_spread_break_in_combination: u8,
        pub targetable_range: u8,
        pub remote_catch_pull_out_use_action: u8,
        pub is_scaleable: u8,
        pub save_level_data: u8,
        pub is_docking_combination_key_frame: u8,
        pub exclude_sequencer_boundary: u8,
        pub is_buyable: u8,
        pub apply_furniture_obstacle_in_housing: u8,
        pub interaction_ui_distance_lv: u8,
        pub stick_to_object_type: u8,
        pub use_remote_catch_fishing: u8,
        pub is_spawned_on_platform_key_frame: u8,
        pub is_housing_gimmick: u8,
        pub is_wild: u8,
        pub is_link_deco_gimmick: u8,
        pub is_attack_by_collision_dynamic: u8,
        pub use_sliding_motion_property: u8,
        pub initial_body_motion_type: u8,
        pub is_editor_usable: u8,
        pub is_use_constrain_sound: u8,
        pub is_get_knowledge_when_get_item: u8,
        pub gimmick_type: u8,
        pub is_attack_by_collision_docking: u8,
        pub equip_docking_gimmick_type: u8,
        pub gimmick_interface_type: u8,
        pub gimmick_placement_style: u8,
        pub is_sub_part: u8,
        pub is_keep_anchor: u8,
        pub use_constraint_achor_edge: u8,
        pub is_isolated_anchor_breakable: u8,
        pub is_break_main_part_on_break: u8,
        pub is_auto_partial_break: u8,
        pub is_targetable: u8,
        pub is_attack_by_collision_key_frame: u8,
        pub is_anchor_edge_disable: u8,
        pub main_gimmick_group_info_of_combination: u32,
        pub default_spawn_reason_hash: u32,
        pub spawn_distance_level: u32,
        pub attack_by_collision_info_list_key: u32,
        pub battery_init_capacity: u64,
        pub battery_total_capacity: u64,
        pub link_signal_group_list: CArray<GimmickLinkSignalGroupData<'a>>,
        pub property_list: CArray<u32>,
        pub gimmick_tag_list: CArray<CString<'a>>,
        pub gimmick_chart_path: u32,              // StringInfoKey
        pub gimmick_remote_catchable_data: COptional<GimmickRemoteCatchableData>,
        pub auto_targeting_constraint_data_list: CArray<GimmickConstraintData<'a>>,
        pub gimmick_constraint_data_list: CArray<GimmickConstraintData<'a>>,
        pub gimmick_info_list: CArray<u32>,
        pub game_event_handler_list: CArray<GimmickGameEventHandlerData>,
        pub unlockable_data_list: CArray<UnlockableData>,
        pub sequencer_level_allow_gimmick_event_key_list: CArray<u32>,
        pub level_alias_name_list: CArray<GimmickLevelAliasData<'a>>,
        pub child_scene_object_alias_name_list: CArray<CString<'a>>,
        pub gimmick_alias_data_list: CArray<GimmickAliasData>,
        pub logout_time_after_break: u64,
        pub gimmick_node_data: GimmickNodeData<'a>,
        pub stick_to_object_socket_list: CArray<CString<'a>>,
        pub push_object_socket_list: CArray<CString<'a>>,
        pub combination_alias_data_list: CArray<GimmickCombinationAliasData<'a>>,
        pub combination_link_data_list: CArray<GimmickCombinationLinkData<'a>>,
        pub elemental_material_info_list: CArray<GimmickElementalMaterialEntry>,
        pub elemental_status_initial_stat_list: CArray<GimmickElementalStatusInitialStat>,
    }
}

impl<'a> GimmickGroupInfo<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> std::io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        let consumed = *offset - start;
        if consumed != entry_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("GimmickGroupInfo: consumed {} bytes, expected {}", consumed, entry_size),
            ));
        }
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("gimmickgroupinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                GimmickGroupInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "gimmickgroupinfo roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else {
            eprintln!("SKIP: pabgh not found");
            return;
        };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = GimmickGroupInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            GimmickGroupInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
