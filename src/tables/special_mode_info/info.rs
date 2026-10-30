//! Tier 1 — fully typed (no _tail_b64).
//!
//! 2.04.00 reader: Mac `sub_102044268` (names from its Korean assert strings). 2.04 moved
//! stringKey first, gathered the flags after isBlocked, added `_contentsPhaseInfo` (u16) and
//! grew `_optionList` from 24 to 25 slots. The pre-2.04 `use_world_lighting` / `new_112_flag`
//! bytes were width fits; there are no such fields in 2.04.
//!
//! SpecialModeOption: Mac element reader `sub_102043814`. DetectModeAreaData
//! (`sub_102043E3C`) and PlayerActionLimitDesc (`sub_102037CD4`) are unchanged.

use crate::binary::*;
use crate::py_binary_struct;

// `_subInnerGuideEffectList` element (Mac sub_102082788) — 24 mem bytes / 3 wire fields.
py_binary_struct! {
    pub struct SpecialModeOptionMacroEntry<'a> {
        pub key_str: CString<'a>,
        pub name: CString<'a>,
        pub flag: u8,
    }
}

// Mac sub_102043814. Fields past `bokeh_kernel_size` have no assert string; they are named
// by type and position.
py_binary_struct! {
    pub struct SpecialModeOption<'a> {
        pub type_: u8,
        pub player_condition_info: u32,
        pub target_condition_info: u32,
        pub custom_render_value_name: CString<'a>,
        pub effect_key: u32,
        pub param_name: CString<'a>,
        pub param_uint32: u32,
        pub on_event_name: u32,                   // StringInfoKey
        pub off_event_name: u32,                  // StringInfoKey
        pub repeat_event_name: u32,               // StringInfoKey
        pub gimmick_event_param_string_hash: u32,
        pub repeat_interval_random_min: u64,
        pub repeat_interval_random_max: u64,
        pub visual_equip_item_info: u32,
        pub path_trail_type: u8,
        pub allow_interaction_info_list: CArray<u32>,
        pub sub_inner_guide_effect_list: CArray<SpecialModeOptionMacroEntry<'a>>,
        pub level_gimmick_scene_object_info: u32,
        pub guide_effect_name: CString<'a>,
        pub guide_effect_type: u8,
        pub guide_effect_immediately_kill: u8,
        pub force_send_gimmick_event: u8,
        pub guide_effect_distance_limit: f32,
        pub check_distance_by_horizontal: u8,
        pub illusion_effect_color: u32,
        pub illusion_effect_time: f32,
        pub lens_flare_scale_value: f32,
        pub use_depth_of_field: u8,
        pub focus_distance: f32,
        pub focal_length: f32,
        pub aperture: f32,
        pub bokeh_kernel_size: f32,
        pub f32_156: f32,
        pub f32_160: f32,
        pub key_list_168: CArray<u32>,
        pub name_184: CString<'a>,
        pub u64_188: u64,
        pub u64_196: u64,
        pub u8_204: u8,
        // sub_101A2F048: 16 single-byte reads.
        pub trailing_00: u8, pub trailing_01: u8, pub trailing_02: u8, pub trailing_03: u8,
        pub trailing_04: u8, pub trailing_05: u8, pub trailing_06: u8, pub trailing_07: u8,
        pub trailing_08: u8, pub trailing_09: u8, pub trailing_10: u8, pub trailing_11: u8,
        pub trailing_12: u8, pub trailing_13: u8, pub trailing_14: u8, pub trailing_15: u8,
    }
}

// `_optionList`: 25 fixed CArray<SpecialModeOption> slots (Mac sub_102082AA0).
py_binary_struct! {
    pub struct SpecialModeOptionSlots<'a> {
        pub slot_00: CArray<SpecialModeOption<'a>>,
        pub slot_01: CArray<SpecialModeOption<'a>>,
        pub slot_02: CArray<SpecialModeOption<'a>>,
        pub slot_03: CArray<SpecialModeOption<'a>>,
        pub slot_04: CArray<SpecialModeOption<'a>>,
        pub slot_05: CArray<SpecialModeOption<'a>>,
        pub slot_06: CArray<SpecialModeOption<'a>>,
        pub slot_07: CArray<SpecialModeOption<'a>>,
        pub slot_08: CArray<SpecialModeOption<'a>>,
        pub slot_09: CArray<SpecialModeOption<'a>>,
        pub slot_10: CArray<SpecialModeOption<'a>>,
        pub slot_11: CArray<SpecialModeOption<'a>>,
        pub slot_12: CArray<SpecialModeOption<'a>>,
        pub slot_13: CArray<SpecialModeOption<'a>>,
        pub slot_14: CArray<SpecialModeOption<'a>>,
        pub slot_15: CArray<SpecialModeOption<'a>>,
        pub slot_16: CArray<SpecialModeOption<'a>>,
        pub slot_17: CArray<SpecialModeOption<'a>>,
        pub slot_18: CArray<SpecialModeOption<'a>>,
        pub slot_19: CArray<SpecialModeOption<'a>>,
        pub slot_20: CArray<SpecialModeOption<'a>>,
        pub slot_21: CArray<SpecialModeOption<'a>>,
        pub slot_22: CArray<SpecialModeOption<'a>>,
        pub slot_23: CArray<SpecialModeOption<'a>>,
        pub slot_24: CArray<SpecialModeOption<'a>>,
    }
}

// Mac sub_102043E3C — DetectModeAreaData, 64 mem bytes / 8 wire fields.
py_binary_struct! {
    pub struct DetectModeAreaData<'a> {
        pub flag_a: u8,
        pub name_a: CString<'a>,
        pub name_b: CString<'a>,
        pub vec_a: [f32; 3],
        pub vec_b: [f32; 3],
        pub raw_a: u32,
        pub raw_b: u32,
        pub flag_b: u8,
    }
}

// Mac sub_102037CD4 — PlayerActionLimitDesc.
py_binary_struct! {
    pub struct PlayerActionLimitDesc {
        pub flag_a: u8,
        pub flag_b: u8,
        pub flag_c: u8,
        pub flag_d: u8,
        pub flag_e: u8,
        pub flag_f: u8,
        pub list_a: CArray<u16>,
        pub list_b: CArray<u16>,
    }
}

py_binary_struct! {
    pub struct SpecialModeInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u32,
        pub is_blocked: u8,
        pub exclusive_with_detect: u8,
        pub disable_occlusion_culling: u8,
        pub has_near_by_target_option: u8,
        pub is_high_priority: u8,
        pub disable_player_targetable: u8,
        pub is_allow_dialog: u8,
        pub type_: u8,
        pub change_minimap_scale: u8,
        pub is_minimap_zoom_out: u8,
        pub active_condition_info: u32,
        pub passive_skill: u32,
        pub contents_phase_info: u16,
        pub time_scale: f32,
        pub cancel_input_key_hash: u32,
        pub player_time_scale: f32,
        pub input_key_hash: u32,
        pub skill_level: u32,
        pub mode_radius: f32,
        pub post_process_sequencer_name: CString<'a>,
        pub option_slots: SpecialModeOptionSlots<'a>,
        pub detect_mode_area_data: DetectModeAreaData<'a>,
        pub player_action_limit_desc: PlayerActionLimitDesc,
    }
}

impl<'a> SpecialModeInfo<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> std::io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        let consumed = *offset - start;
        if consumed != entry_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("SpecialModeInfo: consumed {} bytes, expected {}", consumed, entry_size),
            ));
        }
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("specialmode.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                SpecialModeInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "specialmode roundtrip mismatch");
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
            let item = SpecialModeInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            SpecialModeInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
