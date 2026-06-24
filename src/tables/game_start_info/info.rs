//! Parser for `GameStartInfo.pabgb` (new in game patch 1.12).
//!
//! The "New Game" definition: one record per selectable game-start profile
//! (vanilla 1.12 ships a single `Main`). Field layout + names extracted from
//! the Hex-Rays decompile of the GameStartInfo deserializer `sub_101F99838`
//! in the Mac binary `CrimsonDesert_Steam` (1.12). Wire reads in order:
//!
//!   key                    u8                      (sub_101618E3C)
//!   string_key             CString                 (sub_100D39448)  _stringKey
//!   is_blocked             u8                       (sub_100D391B8)  _isBlocked
//!   name                   CString                 (sub_100D39448)  _name
//!   field_info             u32 (FieldInfoKey)       (sub_1015EF180)  _fieldInfo
//!   start_quest_info       u32 (QuestKey)           (sub_101609A58)  _startQuestInfo
//!   use_instance_save_slot u8                       (sub_100D391B8)  _useInstanceSaveSlot
//!   character_spawn_data_map  CArray (count u32)    (sub_101FD01A8)  _characterSpawnDataMap
//!
//! Vanilla `Main` = 27 bytes: key=1, string_key="Main", is_blocked=0,
//! name="", field_info=1, start_quest_info=1, use_instance_save_slot=0,
//! character_spawn_data_map=[] (empty). Verified byte-perfect round-trip.
//!
//! NOTE: `character_spawn_data_map` is a runtime *map* whose per-entry layout
//! is unknown (vanilla ships it empty). It is modelled here as `CArray<u8>`
//! purely so the empty (count=0) case round-trips exactly; a non-empty map
//! would need its entry struct reverse-engineered before this is correct.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct GameStartInfo<'a> {
        pub key: u8,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub name: CString<'a>,
        pub field_info: u32,
        pub start_quest_info: u32,
        pub use_instance_save_slot: u8,
        pub character_spawn_data_map: CArray<u8>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("gamestartinfo.pabgb") }

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(GameStartInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "gamestartinfo roundtrip bytes mismatch");
    }
}
