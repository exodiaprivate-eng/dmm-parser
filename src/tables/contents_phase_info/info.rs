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
        // 1.13.00: new u16 inserted here (+2). 1.16.00 resolves it: these 2 bytes
        // are `_hidePlayer` (u8) + `_checkActivatedGamePlayTrigger` (u8). Kept as
        // one u16 — byte-identical, and parser field names are a MOD CONTRACT.
        pub unk_u16_113: u16,
        // 1.16.00: three fields inserted here by the housing/trading overhaul.
        // Verified on all 3 vanilla records: limit = 500 (Housing) / 100
        // (FactionManagement) / 0 (Pond).
        pub use_housing_placement: u8,
        pub use_change_house: u8,
        pub housing_placement_limit: u32,
        // = the binary's `_enableButtonUITemplateName` (name kept: MOD CONTRACT).
        pub button_ui_template_name: u32,
        // 1.16.00: the matching "disable" template, which 1.13 lacked.
        pub disable_button_ui_template_name: u32,
        pub enable_button_ui_component_name: u32,
        pub disable_button_ui_component_name: u32,
        // Trailing fixed run, 21 B in 1.16 (was 17). Per the binary's field list
        // these are _uiItemListMercenaryInfo, _uiTitleLocalString,
        // _uiTextureIconPath, _uiListEmptyLocalString,
        // _uiListExceedWarningLocalString, _cameraPivotTargetLevelGimmickSceneObjectInfo
        // — i.e. u8 + 5x u32. The existing a/b/c/d split is byte-equivalent to
        // that (d = the two LocalString hashes as one u64) and is preserved
        // verbatim; only `tail_e_116` is new. The split is pinned, not guessed:
        // the 0xEAC5E173 null-hash sentinel falls exactly on the tail_c boundary
        // in records 1 and 2, which no other alignment reproduces.
        pub tail_a_113: u8,
        pub tail_b_113: u32,
        pub tail_c_113: u32,
        pub tail_d_113: u64,
        pub tail_e_116: u32,
    }
}
