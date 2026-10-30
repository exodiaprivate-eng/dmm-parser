//! Tier 1 (with Decoded|Raw fallback) — typed prefix plus a per-entry
//! enum that exposes the full body. As of 2026-04-30 with the
//! Mac-IDA tag 54/214 recipe fixes (`5fa0b06`), **all 363/363 vanilla
//! entries route through Decoded** (100% Decoded, down from 57 Raw at
//! the start of recipe-verification). The Raw fallback path is kept
//! for byte-perfect resilience if a future build introduces
//! unresolved ConditionPair variants.
//!
//! Reader: `sub_1410DFBA0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. u8 interaction_type
//!   5. u8 interaction_show_ui_type
//!   6. u8 preemption_type
//!   7. LocalizableString interaction_name
//!   8. u8 pivot_selection_target
//!   9. CArray<InteractionPivotData> interaction_pivot_list
//!     ← typed prefix ends here; tail follows
//!  10. ConditionPairCArray cond_data_list (sub_141114DD0)
//!  11. u8 auto_interaction_type
//!  12. u16 category_info        (sub_141103CA0 → qword_145F290B0)
//!  13. u32 input_key_map_name   (read_u32_lookup_DA30)
//!  14. u8 button_click_type
//!  15. u8 keyboard_click_type
//!  16-19. 4× u8 unknown_flags
//!  20. SequencerStageChartDescPartial sequencer_desc (sub_141D8C6D0)
//!  21. u32 raw_a
//!  22. u32 some_name            (read_u32_lookup_DA30)
//!  23. u32 lookup_b             (sub_141100370 → qword_145F113C8)
//!  24. CArray<FactionAdjacencyMobItem> mob_list (sub_141100E90)
//!  25. u32 lookup_c             (sub_141103C30 → qword_145F1A720)
//!  26. CArray<u32> list_a       (sub_1410FEF40 → qword_145F0DA30)
//!  27. CString cstring_a
//!  28. CString cstring_b
//!  29-38. 10× u8 trailing flags
//!
//! **All 363 / 363 vanilla entries (100%)** route through `Decoded`.
//! The Raw fallback path is preserved for byte-perfect resilience
//! against future builds. Tag 54 (`CheckCurrentEquipType_OrTag54`,
//! TwoU32BodyPayload) and tag 214 (`CheckExistStealItem`,
//! `ConditionData_CheckExistStealItemPayload`) — formerly the only
//! anti-disassembly blockers — were resolved via Mac-IDA in
//! `5fa0b06`.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::InteractionInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  37
// Decoded by dmm-parser:   8
// Missing in this struct:  29
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ⏳ _isPlayerInterruptable (direct_u8, stream=1)
// ⏳ _isCatchInteractionForEditor (direct_u8, stream=1)
// ⏳ _fixRotationWhileInteraction (direct_u8, stream=1)
// ⏳ _useActionGotoOffset (direct_u8, stream=1)
// ⏳ _subInteraction (direct_u8, stream=1)
// ⏳ _cancelOnMoveFail (direct_u8, stream=1)
// ⏳ _waitForInteraction (direct_u8, stream=1)
// ⏳ _dialogSetInfo (reader_4B, stream=4)
// ⏳ _interactionPopItemList
// ⏳ _showMainMenuPanelName
// ⏳ _interactionTag (reader_4B, stream=4)
// ⏳ _useFacingGotoTransform (direct_u8, stream=1)
// ⏳ _showMainMenuEventName
// ⏳ _allowInteractionWhileInteraction (direct_u8, stream=1)
// ⏳ _showWhenTargeted (direct_u8, stream=1)
// ⏳ _checkObjectOnTop (direct_u8, stream=1)
// ⏳ _autoMovingStopDistance (direct_u8, stream=1)
// ⏳ _showUIAtPivotSocket (direct_u8, stream=1)
// ⏳ _enableOnDockingOrCatch (direct_u8, stream=1)
// ⏳ _onPreemptionSuccessAiEventKey (direct_u32, stream=4)
// ⏳ _sequencerStageChartDesc (array_or_complex, stream=12)
// ⏳ _rewardDropSetInfo (reader_4B, stream=4)
// ⏳ _uiKeyTriggerSoundName (reader_4B, stream=4)
// ✅ _pivotSelectionTarget (direct_u8, stream=1)
// ✅ _interactionName (reader_8B, stream=8)
// ⏳ _interactionConditionDataList
// ✅ _interactionPivotList
// ⏳ _categoryInfo (reader_2B, stream=2)
// ⏳ _autoInteractionType (direct_u8, stream=1)
// ⏳ _buttonClickType (direct_u8, stream=1)
// ⏳ _inputKeyMapName (reader_4B, stream=4)
// ✅ _stringKey
// ✅ _key (direct_u8, stream=1)
// ✅ _interactionType (direct_u8, stream=1)
// ✅ _isBlocked (direct_u8, stream=1)
// ✅ _preemptionType (direct_u8, stream=1)
// ⏳ _interactionShowUIType (direct_u8, stream=1)

use crate::binary::*;
use crate::binary::optional_game_condition::OptionalGameCondition;
use crate::binary::variants::condition_pair::ConditionPairCArray;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use crate::py_binary_struct;
use crate::tables::faction_node_info::info::FactionAdjacencyMobItem;
use serde_json::{Map, Value};
use std::io::{self, Write};

py_binary_struct! {
    pub struct InteractionPivotData<'a> {
        // game v16: the leading u32 `key` hash was REPLACED by the pivot's key
        // NAME as a CString (e.g. "gimmick_collect", "TempPivotKey_<X>_On_1").
        // Byte-decisive on k=10028: OLD `0d 3a 9c 6a` (4 B) -> NEW `0f 00 00 00`
        // + "gimmick_collect" (19 B), and the record delta is exactly +15.
        pub pivot_key_name_116: CString<'a>,
        pub tag_a: CString<'a>,
        pub vec_a: [f32; 3],
        pub tag_b: CString<'a>,
        pub vec_b: [f32; 3],
        pub tag_c: CString<'a>,
        pub vec_c: [f32; 3],
        // ── 1.18.00: `_targetGotoOffsetRadius`, an f32 immediately after the
        // goto-offset vec3. From the Mac 1.18 reader sub_101FE8638
        // (`InteractionPivotOverrideData`): … targetGotoOffset (12 B vec3) @+20,
        // **targetGotoOffsetRadius @+32**, interactionUpperHeight @+36,
        // interactionLowerHeight @+40, interactionDistance @+44 — i.e. a single
        // 4-byte float wedged between the vec3 and the three height/distance
        // floats.
        // Anchor chosen empirically: of the five vec3s in this element, only
        // placing it after vec_c lets the table parse (the CString/CArray length
        // prefixes downstream reject every other position).
        // 2.04.00: 8 bytes on the wire (Mac reader, inlined in sub_101A5C758; the same change
        // as InteractionPivotOverrideData's radius). Held raw.
        pub target_goto_offset_radius: u64,
        pub tag_d: CString<'a>,
        pub vec_d: [f32; 3],
        pub raw_a: u32,
        pub raw_b: u32,
        pub raw_c: u32,
        pub vec_e: [f32; 3],
        pub raw_e_3: u32,
        pub raw_f: u32,
        pub raw_f_1: u32,
        pub raw_g: u32,
        pub raw_g_1: u32,
        pub raw_h: u32,
        pub raw_i: u32,
        pub name: CString<'a>,
        pub vec_f: [f32; 3],
        pub raw_j: u32,
        pub raw_k: u64,
        pub raw_l: u64,
        pub faction_group_info: u32,
    }
}

py_binary_struct! {
    /// 2.04.00 `InteractionNameSelectionData` (Mac reader sub_10208E2E8, behind a presence byte).
    pub struct InteractionNameSelectionData<'a> {
        pub condition_logic: OptionalGameCondition<'a>,
        pub target_condition_logic: OptionalGameCondition<'a>,
        pub interaction_name: LocalizableString<'a>,
    }
}

py_binary_struct! {
    // 2.04.00: fully typed from the Mac reader (sub_10204BB60), which names every read. Before
    // 2.04 the record ended in an opaque `tail` (conditions + a few typed fields + a byte blob);
    // the field names that tail used are kept (cond_data_list, auto_interaction_type,
    // category_info, input_key_map_name, button_click_type, keyboard_click_type,
    // pivot_selection_target).
    pub struct InteractionInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u32,
        pub is_blocked: u8,
        pub show_when_targeted: u8,
        pub allow_interaction_while_interaction: u8,
        pub is_catch_interaction_for_editor: u8,
        pub show_ui_at_pivot_socket: u8,
        pub check_lootable_field: u8,
        pub use_facing_goto_transform: u8,
        pub cancel_on_move_fail: u8,
        pub sub_interaction: u8,
        pub wait_for_interaction: u8,
        pub is_player_interruptable: u8,
        pub use_action_goto_offset: u8,
        pub fix_rotation_while_interaction: u8,
        pub enable_on_docking_or_catch: u8,
        pub show_ui_intimacy: u8,
        pub pivot_selection_target: u8,            // _pivotSelectionType
        pub auto_interaction_type: u8,
        pub button_click_type: u8,
        pub interaction_type: u8,
        pub interaction_show_ui_type: u8,
        pub preemption_type: u8,
        pub keyboard_click_type: u8,
        pub auto_moving_stop_distance: u8,
        pub check_object_on_top: u8,
        pub reward_drop_set_info: u32,
        pub ui_key_trigger_sound_name: u32,         // StringInfoKey
        pub activity_group_info: u32,
        pub category_info: u16,
        pub input_key_map_name: u32,                // StringInfoKey
        pub on_preemption_success_ai_event_key: u32,
        pub dialog_set_info: u32,
        pub interaction_name: LocalizableString<'a>,
        pub interaction_pivot_list: CArray<InteractionPivotData<'a>>,
        pub cond_data_list: ConditionPairCArray<'a>, // _interactionConditionDataList
        pub interaction_name_selection_data_list: CArray<COptional<InteractionNameSelectionData<'a>>>,
        pub interaction_approach_type: u8,
        pub wall_check_type: u8,
        pub interaction_pop_item_list: CArray<FactionAdjacencyMobItem>,
        pub interaction_tag: CArray<u32>,           // StringInfoKeys
        pub child_interaction_info_list: CArray<u32>,
        pub show_main_menu_panel_name: u32,         // StringInfoKey
        pub show_main_menu_event_name: u32,         // StringInfoKey
    }
}

impl<'a> InteractionInfo<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        if *offset - start != entry_size {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!(
                "InteractionInfo: consumed {} of {} bytes", *offset - start, entry_size)));
        }
        Ok(item)
    }

    pub fn read_tracked_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        let start = *offset;
        let item = <Self as BinaryReadTracked>::read_tracked(data, offset, path, ranges)?;
        if *offset - start != entry_size {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!(
                "InteractionInfo: consumed {} of {} bytes", *offset - start, entry_size)));
        }
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("interactioninfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP: fixture not found"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP: pabgh not found"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            let item = match InteractionInfo::read_with_size(&data, &mut c, e - s) {
                Ok(it) => it,
                Err(er) => {
                    let tag = crate::binary::variants::condition_data::LAST_ATTEMPTED_TAG.with(|x| x.get());
                    let trail = crate::binary::variants::condition_data::TAG_TRAIL.with(|t| t.borrow().clone());
                    let trail_strs: Vec<String> = trail.iter().map(|(t, off)| format!("tag={} after_body_abs={}", t, off)).collect();
                    panic!("entry {} k=0x{:x}: {} (LAST_ATTEMPTED_TAG = {:?}, TRAIL = [{}])", i, k, er, tag, trail_strs.join(" | "));
                }
            };
            assert_eq!(c, *e);
            items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        if out != data {
            let first = out.iter().zip(data.iter()).position(|(a, b)| a != b).unwrap_or(out.len().min(data.len()));
            let which = ranges.iter().enumerate().find(|(_, (_, s, e))| first >= *s && first < *e);
            let (idx, key, start) = match which {
                Some((i, (k, s, _))) => (i as i64, *k, *s),
                None => (-1, 0, 0),
            };
            let lo = first.saturating_sub(8);
            let hi = (first + 24).min(data.len()).min(out.len());
            panic!(
                "interactioninfo roundtrip mismatch: out.len={} data.len={} first diff at 0x{:x} (entry {} key=0x{:x} +{})
  data: {:02x?}
  out:  {:02x?}",
                out.len(), data.len(), first, idx, key, first - start, &data[lo..hi], &out[lo..hi]
            );
        }
    }


    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP: fixture not found"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP: pabgh not found"); return; };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = InteractionInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            InteractionInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
