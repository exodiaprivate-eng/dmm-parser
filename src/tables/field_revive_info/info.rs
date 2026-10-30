//! Tier 1 — every wire byte typed.
//!
//! 2.04.00 reader: Mac `sub_10203C23C`, names from its Korean asserts. The game file is
//! `fieldreviveinfo.pabgb` from 2.04 (it shipped as `reviepointinfo.pabgb` before; dispatch
//! still maps that name). 2.04 moved stringKey first and the per-point position, rotation and
//! knowledge into `_fieldReviveDataList`, and added regionHeight, regionInfoList and the two
//! exclusion lists. Wire order:
//!
//!   stringKey, key, isBlocked, useDefaultRevive, reviveWithMercenary, fieldInfoKey,
//!   regionHeight, fieldReviveDataList, regionInfoList, exceptGamePlayTriggerInfoList,
//!   exceptLevelGimmickSceneObjectInfoList, sequencerStageChartDesc

use crate::binary::*;
use crate::binary::variants::sequencer_stage_chart_desc::SequencerStageChartDescPartial;
use crate::py_binary_struct;
use std::io;

py_binary_struct! {
    /// `_fieldReviveDataList` element (Mac sub_10203BEA4).
    pub struct FieldReviveData {
        pub position: [f32; 3],
        pub rotation_y: f32,
        pub knowledge_info: u32,
        pub knowledge_level: u32,
    }
}

py_binary_struct! {
    pub struct FieldReviveInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u32,
        pub is_blocked: u8,
        pub use_default_revive: u8,
        pub revive_with_mercenary: u8,
        pub field_info_key: u32,
        pub region_height: f32,
        pub field_revive_data_list: CArray<FieldReviveData>,
        pub region_info_list: CArray<u16>,
        pub except_game_play_trigger_info_list: CArray<u32>,
        pub except_level_gimmick_scene_object_info_list: CArray<u32>,
        pub sequencer_stage_chart_desc: SequencerStageChartDescPartial<'a>,
    }
}

impl<'a> FieldReviveInfo<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        if *offset - start != entry_size {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!(
                "FieldReviveInfo: consumed {} of {} bytes", *offset - start, entry_size)));
        }
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("fieldreviveinfo.pabgb") }

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP: fixture not found"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP: pabgh not found"); return; };
        let mut out = Vec::with_capacity(data.len());
        for (i, (k, s, e)) in entry_ranges(&entries, data.len()).iter().enumerate() {
            let mut c = *s;
            let it = FieldReviveInfo::read_with_size(&data, &mut c, e - s)
                .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er));
            it.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "fieldreviveinfo roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP: fixture not found"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP: pabgh not found"); return; };
        for (i, (key, start, end)) in entry_ranges(&entries, data.len()).iter().enumerate() {
            let mut cursor = *start;
            let item = FieldReviveInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            FieldReviveInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(from_json, from_typed, "entry {} key=0x{:x}: JSON round-trip diverges", i, key);
        }
    }
}
