//! IDA-derived parser for `gameversiondatainfo.pabgb`.
//!
//! Reader: `sub_1410DB310` (CrimsonDesert.exe 1.0.8).
//!
//! Wire layout:
//!   u16  key                              (_key, GameVersionDataKey)
//!   CString string_key                    (_stringKey)
//!   u8   is_blocked                       (_isBlocked)
//!   CArray<RestoreItemData>               (_restoreItemDataByGameVersion)
//!     per element (1.10): u32 item_key + u64 raw + u8 (NEW) + u32 target_key
//!   u32  version_flag                     (last field, name unknown)
//!
//! 1.10 change: RestoreItemData grew 16→17 bytes (a u8 inserted between
//! raw_data and target_key). Verified against live 1.10 pabgb.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct RestoreItemData {
        pub item_key: u32,
        pub raw_data: u64,
        // 1.10: u8 inserted here. Element grew 16→17 bytes; observed 0x00.
        pub unknown_flag_110: u8,
        pub target_key: u32,
    }
}

py_binary_struct! {
    pub struct GameVersionDataInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub restore_item_data_by_game_version: CArray<RestoreItemData>,
        pub version_flag: u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("gameversiondatainfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(GameVersionDataInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            let item = GameVersionDataInfo::read_from(&data, &mut offset).unwrap();
            items.push(item.to_json_dict());
        }
        let mut out = Vec::with_capacity(data.len());
        for map in &items {
            GameVersionDataInfo::write_from_json_dict(&mut out, map).unwrap();
        }
        assert_eq!(out, data, "json roundtrip bytes mismatch");
    }
}
