//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410F60E0` in CrimsonDesert.exe (Win build).
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u32 key                              (_key)
//!   2. CString string_key                   (_stringKey)
//!   3. u8 is_blocked                        (_isBlocked)
//!   4. u8 type_                             (_type, Rust keyword suffix)
//!   5. u32 active_condition_info            (_activeConditionInfo)
//!   6. CString post_process_sequencer_name  (_postProcessSequencerName)
//!   7. u32 time_scale                       (_timeScale, f32-as-u32)
//!   8. u32 player_time_scale                (_playerTimeScale)
//!   9. u32 mode_radius                      (_modeRadius)
//!  10. u32 passive_skill                    (_passiveSkill, sub_1410FEBE0
//!      → qword_145F0DA68)
//!  11. u32 skill_level                      (_skillLevel)
//!  12. u32 input_key_hash                   (_inputKeyHash)
//!  13. u32 cancel_input_key_hash            (_cancelInputKeyHash)
//!  14. u8 has_near_by_target_option         (_hasNearByTargetOption)
//!  15. u8 is_high_priority                  (_isHighPriority)
//!  16. u8 exclusive_with_detect             (_exclusiveWithDetect)
//!  17. u8 disable_occlusion_culling         (_disableOcclusionCulling)
//!  18. u8 disable_player_targetable         (_disablePlayerTargetable)
//!  19. u8 change_minimap_scale              (_changeMinimapScale)
//!  20. u8 is_minimap_zoom_out               (_isMinimapZoomOut)
//!  21. u8 is_allow_dialog                   (_isAllowDialog)
//!  22. _optionList (24× sub_141128AF0(struct +72+16*i) CArray-like
//!      24-iter loop reading 16-byte items) ← TAIL STARTS HERE
//!  23. (body) _detectModeAreaData, _playerActionLimitDesc
//!
//! Steps 1-21 are typed; step 22 onward lives in `tail_blob`. Reopens
//! cleanly when the 16-byte item helper sub_141128AF0 is decoded.

use crate::binary::*;
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct SpecialModeInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub type_: u8,
        pub active_condition_info: u32,
        pub post_process_sequencer_name: CString<'a>,
        pub time_scale: u32,
        pub player_time_scale: u32,
        pub mode_radius: u32,
        pub passive_skill: u32,
        pub skill_level: u32,
        pub input_key_hash: u32,
        pub cancel_input_key_hash: u32,
        pub has_near_by_target_option: u8,
        pub is_high_priority: u8,
        pub exclusive_with_detect: u8,
        pub disable_occlusion_culling: u8,
        pub disable_player_targetable: u8,
        pub change_minimap_scale: u8,
        pub is_minimap_zoom_out: u8,
        pub is_allow_dialog: u8,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\specialmode.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\specialmode.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
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
        let Ok(data) = std::fs::read(PABGB) else {
            eprintln!("SKIP: missing fixture {}", PABGB);
            return;
        };
        let Some(entries) = load_pabgh_offsets(PABGH) else {
            eprintln!("SKIP: missing pabgh fixture {}", PABGH);
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
