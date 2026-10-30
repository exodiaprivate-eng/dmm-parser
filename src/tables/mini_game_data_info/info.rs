//! MiniGameDataInfo, 2.04.00 (Mac reader `sub_10203FB2C`, names from its Korean asserts).
//!
//! 2.04 re-sequenced the record and added equipItemUsable, recoveryHP, completeMissionInfo,
//! difficultyBuffInfo, uiTextureName, tag, difficultyBuffLevelList and name. The two
//! leader-board sounds became StringInfoKeys (were CStrings). `_spawnDataList` is no longer
//! the last field, and every ConditionData in it now decodes, so it is read typed (the
//! pre-2.04 Decoded|Raw fallback and its tail probe are gone).

use crate::binary::*;
use crate::binary::variants::sequencer_stage_chart_desc::SequencerStageSpawnData;
use crate::py_binary_struct;
use std::io;

py_binary_struct! {
    /// `_playerDataList` / `_npcDataList` element (Mac sub_10203F648). Names kept from the
    /// pre-2.04 model; game names in comments.
    pub struct MiniGameParticipantData {
        pub key_hash: u32,          // _itemInfo
        pub flag_qword: u64,        // _count
        pub flag_a: u8,             // _isEquip
        pub flag_b: u8,             // _isRetrieve
        pub value_dword: u32,       // _unequipSlotNameKey
        pub spawn_keys: CArray<u32>, // _fixedWeaponTypeList
    }
}

py_binary_struct! {
    /// `_entranceFeeList` element (Mac sub_101A71520): u32 + 3x u64.
    pub struct EntranceFee {
        pub count_or_flag: u32,
        pub value_a: u64,
        pub value_b: u64,
        pub value_c: u64,
    }
}

py_binary_struct! {
    /// `_difficultyBuffLevelList` (Mac sub_10203D1D0): three fixed u32 reads.
    pub struct DifficultyBuffLevelList {
        pub level_0: u32,
        pub level_1: u32,
        pub level_2: u32,
    }
}

py_binary_struct! {
    pub struct MiniGameDataInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u16,
        pub is_blocked: u8,
        pub need_change_character_scale: u8,
        pub use_deactive_result: u8,
        pub equip_item_usable: u8,
        pub recovery_hp: u8,
        pub default_reward_drop_set_info: u32,
        pub game_event_handler_info: u16,
        pub complete_mission_info: u32,
        pub knowledge_info: u32,
        pub difficulty_buff_info: u32,
        pub ui_texture_name: u32,                 // StringInfoKey
        pub ui_view_id: u32,                      // StringInfoKey
        pub phase_panel_tag_name: u32,            // StringInfoKey
        pub script_name: u32,                     // StringInfoKey
        pub ui_leader_board_modal_title_local_string_info: u32,
        pub tag: CString<'a>,
        pub difficulty_buff_level_list: DifficultyBuffLevelList,
        pub name: LocalizableString<'a>,
        pub ui_leader_board_sound: u32,           // StringInfoKey
        pub ui_leader_board_modal_sound: u32,     // StringInfoKey
        pub entrance_fee_list: CArray<EntranceFee>,
        pub player_data_list: CArray<MiniGameParticipantData>,
        pub npc_data_list: CArray<MiniGameParticipantData>,
        pub spawn_data_list: CArray<CArray<SequencerStageSpawnData<'a>>>,
        pub game_advice_info_list: CArray<u32>,
    }
}

impl<'a> MiniGameDataInfo<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        if *offset - start != entry_size {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!(
                "MiniGameDataInfo: consumed {} of {} bytes", *offset - start, entry_size)));
        }
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("minigamedatainfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP: fixture not found"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP: pabgh not found"); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = MiniGameDataInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end-start, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "minigamedatainfo roundtrip bytes mismatch");
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
            let item = MiniGameDataInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            MiniGameDataInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
