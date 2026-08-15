//! Fully typed parser for `LevelGimmickSceneObjectInfo.pabgb` (165 records).
//!
//! THE additive-NPC spawn lever — record 1000011 "Shop" holds ~500 town shop
//! placements in `_levelGimmickSceneObjectDataList`. Reverse-engineered from the
//! 1.12.02 Mac binary (ida-pro-mcp): record reader `sub_101F76270`, element
//! reader `sub_101F75C18`. Field names from the Korean error strings, wire types
//! from the per-field byte-count readers. Element wire = 221B for the Hernand
//! butcher (matches the hand-decoded byte layout exactly). See
//! `src/tables/LEVELGIMMICK_112_RE.md` for the full map.

use crate::binary::*;
use crate::py_binary_struct;
use crate::pabgh_typed_blob_table;

// `LevelGimmickSceneObject_LinkedCompleteGimmick` element (sub_101FB3AA8 body):
// 16B UUID + u32. Wire 20B/elem.
py_binary_struct! {
    pub struct LinkedCompleteGimmick {
        pub scene_object_uuid: [u32; 4],
        pub complete_gimmick_index: u32,
    }
}

// `_worldTransform` / `_teleportOffsetTransform` (sub_100D39CD4) — 40 wire bytes,
// read order: Vec3 + [u32;4] + Vec3.
py_binary_struct! {
    pub struct WorldTransform {
        pub vec_a: [f32; 3],
        pub raw: [u32; 4],
        pub vec_b: [f32; 3],
    }
}

// `LevelGimmickSceneObjectData` — one placement (element reader sub_101F75C18).
py_binary_struct! {
    pub struct LevelGimmickSceneObjectData<'a> {
        pub level_gimmick_scene_object_info: u32,   // LevelGimmickSceneObjectInfoKey
        pub gimmick_info: u32,                      // GimmickInfoKey
        pub item_info: u32,                         // ItemKey
        pub parent_spawning_pool_auto_spawn_info: u32,
        // ── 1.18.00: `_prefabPath`, 4 bytes per ELEMENT, right after
        // parentSpawningPoolAutoSpawnInfo (oracle idx 3).
        // Per-record deltas are all multiples of 4 (0 x14, 4 x22, 8 x13, 12 x10, …)
        // = 4 bytes per list element — and the 14 zero-delta records are exactly
        // the ones that still decoded, which is what pinned it to the element.
        //
        // Modelled as CString rather than u32: the observed bytes are
        // `00 00 00 00` everywhere, so an empty string and a zero hash are
        // indistinguishable HERE — but if it turns out to be a hash, a non-zero
        // value makes CString fail LOUDLY (absurd length -> blob fallback) instead
        // of a u32 silently desyncing the rest of the element. This table already
        // uses CString for `level_name` / `gimmick_alias_name`.
        pub prefab_path: CString<'a>,
        pub level_name: CString<'a>,                // placement/area key (Shop_Hernand_0001_Phase00_05_sub_1_0)
        pub related_game_level_info: u32,           // GameLevelKey
        pub level_name_controlled_by_game_level_info: u32, // StringInfoKey
        pub scene_object_uuid: [u32; 4],            // 16B UUID
        pub root_gimmick_scene_object_uuid: [u32; 4],
        pub spawn_reason: u32,
        pub gimmick_alias_name: CString<'a>,        // scene/shop ref (Shop_Butcher_Hernand)
        pub world_transform: WorldTransform,        // position lives here
        pub teleport_offset_transform: WorldTransform,
        pub guide_effect_offset_position: [f32; 3],
        pub fog_reveal_bitmap_color_r: u8,
        pub linked_complete_gimmick_list: CArray<LinkedCompleteGimmick>,
    }
}

pabgh_typed_blob_table! {
    pub struct LevelGimmickSceneObjectInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub level_name: CString<'a>,
        // the placements (THE spawn list)
        pub level_gimmick_scene_object_data_list: CArray<LevelGimmickSceneObjectData<'a>>,
        // map/fog/discover scalars (all key-lookups are u32 wire)
        pub map_icon_texture_info: u32,
        pub discover_near_fog: u8,
        pub fog_map_icon_texture_info: u32,
        pub fog_distance: u32,           // sub_100D392D8 = u32 (4B), NOT u64
        pub over_abyss_icon_texture_info: u32,
        pub over_abyss_fog_map_icon_texture_info: u32,
        pub over_abyss_fog_distance: u32,
        pub discover_distance: u32,
        pub show_icon_condition_type: u8,
        pub use_teleport: u8,
        pub use_guide_effect: u8,
        pub is_sub_inner_gimmick: u8,
        pub check_game_level_load_state: u8,
        pub use_gimmick_knowledge_for_ui: u8,
        pub check_block_condition: u8,
        pub is_restore_stock_target_item: u8,
        pub completed_discover_map_icon_texture_info: u32,
        pub over_abyss_completed_discover_map_icon_texture_info: u32,
        pub guide_effect_socket_name: CString<'a>,
        pub ore_vein_index: u32,
        pub contents_phase_info_for_move_point: u16,  // ContentsPhaseKey reader = u16 wire
        pub discover_type: u32,
        pub ignore_same_gimmick_discover_distance: u32,
        pub discover_gimmick_state_hash: u32,
        pub is_empty_info: u8,
    }
    tail: tail_blob;
}
