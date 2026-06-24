//! Parser for `ContentsPhaseInfo.pabgb` (new in 1.12). Layout from game deser
//! sub_101F9AB5C, cross-checked against the vanilla 3-record dump.
//! key(u16) + stringKey + isBlocked(u8) + contentsPhaseType(u8) +
//! cameraPresetNameHash(u32) + cameraPivotOffsetYRatio(f32) + activeCursorMode(u8)
//! + playGlobalStageSequencer(u8) + 3x name-hash(u32, StringInfo hash resolved to u16).
use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct ContentsPhaseInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub contents_phase_type: u8,
        pub camera_preset_name_hash: u32,
        pub camera_pivot_offset_y_ratio: f32,
        pub active_cursor_mode: u8,
        pub play_global_stage_sequencer: u8,
        pub button_ui_template_name: u32,
        pub enable_button_ui_component_name: u32,
        pub disable_button_ui_component_name: u32,
    }
}
