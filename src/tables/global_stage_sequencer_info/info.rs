//! Tier 1 — fully typed.
//!
//! 2.04.00 reader: Mac `sub_102052418` (names from its Korean assert strings). 2.04 moved
//! stringKey first, added `_useTemporalUniqueCharacter`, made `_groupName` a StringInfoKey
//! (was a CString) and `_contentsPhaseInfo` a u16 lookup (the pre-2.04 model read it as
//! `contents_phase_type` + `trailing_u8_112`).

use crate::binary::*;
use crate::binary::variants::sequencer_stage_chart_desc::SequencerStageChartDescPartial;
use crate::py_binary_struct;
use std::io;

py_binary_struct! {
    /// `_loadingTargetInfo` body (Mac sub_102093DE4, inside a presence byte).
    pub struct LoadingTargetInner<'a> {
        pub sequencer_desc: SequencerStageChartDescPartial<'a>,
        pub init_sub_timeline: CString<'a>,
    }
}

py_binary_struct! {
    /// `GameEventExecuteData` (Mac sub_10203D6C4, inside a presence byte).
    pub struct GameEventExecuteData {
        pub game_event_type: u8,
        pub player_condition: u32,
        pub target_condition: u32,
        pub event_condition: u32,
    }
}

/// The optional `GameEventExecuteData`; StageInfo embeds the same field.
pub type PlayerBehaviorOptional = COptional<GameEventExecuteData>;

py_binary_struct! {
    pub struct GlobalStageSequencerInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u32,
        pub is_blocked: u8,
        pub use_reserve: u8,
        pub ignore_player_state: u8,
        pub use_temporal_unique_character: u8,
        pub play_condition: u32,
        pub contents_phase_info: u16,
        pub group_leader_info: u32,
        pub player_behavior_space_check_offset_y: f32,
        pub player_behavior_space_radius: f32,
        pub player_behavior_floor_check_distance: f32,
        pub group_name: u32,                      // StringInfoKey
        pub loading_target_info: COptional<LoadingTargetInner<'a>>,
        pub game_event_execute_data: COptional<GameEventExecuteData>,
        pub sequencer_desc_list: CArray<SequencerStageChartDescPartial<'a>>,
    }
}

impl<'a> GlobalStageSequencerInfo<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        if *offset - start != entry_size {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!(
                "GlobalStageSequencerInfo: consumed {} of {} bytes", *offset - start, entry_size)));
        }
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("globalstagesequencerinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                GlobalStageSequencerInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "globalstagesequencerinfo roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = GlobalStageSequencerInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            GlobalStageSequencerInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
