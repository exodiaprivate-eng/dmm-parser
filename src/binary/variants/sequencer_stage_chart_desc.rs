// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! SequencerStageChartDesc, 2.04.00 (Mac reader `sub_10202EA68`, names from its Korean
//! assert strings).
//!
//! 2.04 restructured the desc: the region/gimmick spawn fields moved into an optional
//! `_spawnDesc` (Mac `sub_10202F110`) and the track-change / spawn-data block into an
//! optional `_overrideDesc` (Mac `sub_101D997AC`). Wire layout:
//!
//!   1. CString sequencer_name
//!   2. u32 sequencer_spawn_key
//!   3. CString stage_chart_name
//!   4. COptional<SequencerStageSpawnDesc> spawn_desc
//!   5. COptional<SequencerStageOverrideDesc> override_desc
//!   6. [f32; 3] spawn_position
//!   7. f32 spawn_rotation_y
//!   8. u8 spawn_style
//!   9-13. 5x u8 flags (hideMainMenuOnStart .. useRevivePointForDead)
//!
//! The pre-2.04 model's names (`name`, `prefab_path`, `flag_a..i`, `list_a..f`, ...) were
//! positional; nothing in DMM addressed them, so the game's names are used throughout.

use crate::binary::optional_game_condition::OptionalGameCondition;
use crate::binary::*;
use crate::py_binary_struct;
use std::io;

py_binary_struct! {
    /// `_overrideDesc` string pair element: wire is 2x CString.
    pub struct StringPair<'a> {
        pub key: CString<'a>,
        pub value: CString<'a>,
    }
}

py_binary_struct! {
    /// `SequencerStageTrackChangeData_Character` element (Mac sub_10202F5D0). 2.04 added the
    /// two mercenary keys and `_priority`.
    pub struct TrackChangeCharacter<'a> {
        pub game_condition: OptionalGameCondition<'a>,
        pub spawn_rate: u64,
        pub character_info: u32,
        pub change_character_info: u32,
        pub change_character_group_info: u16,
        pub change_mercenary_info: u8,
        pub change_mercenary_group_info: u8,
        pub priority: u32,
        pub mini_game_play_character_parameter: COptional<u64>,
    }
}

py_binary_struct! {
    /// `SequencerStageTrackChangeData_Gimmick` element (Mac sub_10202FE08).
    pub struct TrackChangeGimmick<'a> {
        pub game_condition: OptionalGameCondition<'a>,
        pub spawn_rate: u64,
        pub gimmick_info: u32,
        pub change_gimmick_info: u32,
    }
}

py_binary_struct! {
    /// `SequencerStageTrackChangeData_Item` element (Mac sub_1020301E4).
    pub struct TrackChangeItem<'a> {
        pub game_condition: OptionalGameCondition<'a>,
        pub spawn_rate: u64,
        pub item_info: u32,
        pub change_item_info: u32,
    }
}

py_binary_struct! {
    /// `_characterTrackChangePriorityIndexList` element (Mac sub_102075548): two u32 reads.
    pub struct TrackChangePriorityIndex {
        pub a: u32,
        pub b: u32,
    }
}

py_binary_struct! {
    /// Track-change element (Mac sub_10203060C). 2.04 added the priority index list.
    pub struct ChartTrackChangeElement<'a> {
        pub game_condition: OptionalGameCondition<'a>,
        pub character_track_change_list: CArray<TrackChangeCharacter<'a>>,
        pub character_track_change_priority_index_list: CArray<TrackChangePriorityIndex>,
        pub gimmick_track_change_list: CArray<TrackChangeGimmick<'a>>,
        pub item_track_change_list: CArray<TrackChangeItem<'a>>,
    }
}

py_binary_struct! {
    /// `_miniGameSpawnDesc` (Mac sub_102075F0C, inside a presence byte). 2.04 added the u8.
    pub struct MiniGameSpawnDesc {
        pub list: CArray<u64>,
        pub raw: u32,
        pub flag: u8,
    }
}

py_binary_struct! {
    /// `SequencerStageSpawnData` element (Mac sub_102030A30). 2.04 added `_uiTextureName`.
    pub struct SequencerStageSpawnData<'a> {
        pub game_condition: OptionalGameCondition<'a>,
        pub spawn_rate: u64,
        pub spawn_character_info: u32,
        pub spawn_character_group_info: u16,
        pub spawn_rider_character_info: u32,
        pub spawn_rider_character_group_info: u16,
        pub reward_drop_set_info: u32,
        pub ui_texture_name: u32,                 // StringInfoKey
        pub is_must: u8,
        pub mini_game_spawn_desc: COptional<MiniGameSpawnDesc>,
    }
}

py_binary_struct! {
    /// `_spawnDesc` (Mac sub_10202F110).
    pub struct SequencerStageSpawnDesc<'a> {
        pub spawn_region_info: CArray<u16>,
        pub not_spawn_region_info: CArray<u16>,
        pub spawn_region_tag_list: CArray<u32>,
        pub not_spawn_region_tag_list: CArray<u32>,
        pub spawn_faction_node_info: CArray<u32>,
        pub not_spawn_faction_node_info: CArray<u32>,
        pub voxel_type: u32,
        pub level_gimmick_scene_object_info: u32,
        pub gimmick_alias_name: CString<'a>,
        pub gimmick_level_name: CString<'a>,
        pub spawn_road_group: u8,
        pub spawn_direction: u8,
        pub spawn_distance: u8,
    }
}

py_binary_struct! {
    /// `_overrideDesc` (Mac sub_101D997AC). Its reader has no assert strings, so the fields
    /// are named by what they hold.
    pub struct SequencerStageOverrideDesc<'a> {
        pub name: CString<'a>,
        pub game_condition: OptionalGameCondition<'a>,
        pub string_pair_list: CArray<StringPair<'a>>,
        pub track_change_list: CArray<ChartTrackChangeElement<'a>>,
        pub spawn_data_lists: CArray<CArray<SequencerStageSpawnData<'a>>>,
    }
}

py_binary_struct! {
    pub struct SequencerStageChartDescPartial<'a> {
        pub sequencer_name: CString<'a>,
        pub sequencer_spawn_key: u32,
        pub stage_chart_name: CString<'a>,
        pub spawn_desc: COptional<SequencerStageSpawnDesc<'a>>,
        pub override_desc: COptional<SequencerStageOverrideDesc<'a>>,
        pub spawn_position: [f32; 3],
        pub spawn_rotation_y: f32,
        pub spawn_style: u8,
        pub hide_main_menu_on_start: u8,
        pub show_main_menu_on_stop: u8,
        pub reserve_exclusive: u8,
        pub use_faction_character_change: u8,
        pub use_revive_point_for_dead: u8,
    }
}

impl<'a> SequencerStageChartDescPartial<'a> {
    /// Read a desc that must fill exactly `total_size` bytes (callers that size it from the
    /// owning record's fixed trailer).
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, total_size: usize) -> io::Result<Self> {
        let start = *offset;
        let item = <Self as BinaryRead>::read_from(data, offset)?;
        if *offset - start != total_size {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!(
                "SequencerStageChartDescPartial: consumed {} of {} bytes", *offset - start, total_size)));
        }
        Ok(item)
    }
}
