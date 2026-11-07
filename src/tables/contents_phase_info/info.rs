//! Parser for `contentsphaseinfo.pabgb` (base 11 records on game 2.04.08,
//! Steam build 26480954; the dlc_01 header holds 0 records and there is no
//! dlc_01 body).
//!
//! 2.04 layout, decoded from the game data, not from the exe (the exe holds no
//! static-table field names). Byte layout: confirmed, every record round-trips
//! byte for byte (tests below). Before 2.04 the record began with the u16 key;
//! it now begins with the string key, like most tables, and the field order
//! after the key changed, so the 1.16 struct failed with "not enough data".
//!
//! Field names: names that existed before are kept only where the 2.04 values
//! back them (parser field names are a MOD CONTRACT; a name on the wrong bytes
//! would silently edit the wrong field). Every other field is `unk_<type>_<N>`,
//! N = byte offset from `is_blocked` (offset 0). Tags: `checked` = values
//! name-checked against the target table; `inferred` = meaning read from the
//! values; candidates from the 1.16 field list are given where the values fit.
//!
//! Header: u16 count + N x {u16 key, u32 offset}. Records are contiguous in
//! header order, so the table reads sequentially.
use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct ContentsPhaseInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u16,
        // 0 on every record.
        pub is_blocked: u8,
        // Ten u8 flags/enums. 1.16 candidates for this run: _contentsPhaseType,
        // _activeCursorMode, _playGlobalStageSequencer, _hidePlayer,
        // _checkActivatedGamePlayTrigger, _useChangeHouse (+2 new in 2.04).
        // unk_u8_1: 1 on the three Housing phases only.
        pub unk_u8_1: u8,
        pub unk_u8_2: u8,
        // unk_u8_3: 1 on FactionManagement*, Pond; 0 elsewhere.
        pub unk_u8_3: u8,
        pub unk_u8_4: u8,
        // inferred: 1 on exactly the phases with housing_placement_limit > 0
        // (Housing*, FactionManagement and its 4 shops), 0 on
        // FactionManagementWithoutHousing, Pond, InsectStorage.
        pub use_housing_placement: u8,
        pub unk_u8_6: u8,
        // unk_u8_7: category-like: 1 Housing*, 2 FactionManagement*, 0 Pond and
        // InsectStorage (candidate: _contentsPhaseType).
        pub unk_u8_7: u8,
        pub unk_u8_8: u8,
        // unk_u8_9: 1 on Housing_Castle only.
        pub unk_u8_9: u8,
        // Name hash; 0xEAC5E173 (null hash) on FactionManagementWithoutHousing,
        // Pond, InsectStorage.
        pub unk_u32_10: u32,
        // 0xFF on 9 records; 0x45 Pond, 0x54 InsectStorage (the two item-list
        // storages; candidate: _uiItemListMercenaryInfo, the 1.16 tail_a u8).
        pub unk_u8_14: u8,
        // unk_u32_15 .. unk_u32_39: name/string hashes. 19 = 0x68FD694B on all
        // records; 15 = the same on Housing* and FactionManagement* (null hash
        // on the other 3); 27..39 are non-null only on Pond and InsectStorage
        // (candidates: the 1.16 UI title / icon / list-empty / list-exceed
        // strings).
        pub unk_u32_15: u32,
        pub unk_u32_19: u32,
        pub unk_u32_23: u32,
        pub unk_u32_27: u32,
        pub unk_u32_31: u32,
        pub unk_u32_35: u32,
        pub unk_u32_39: u32,
        // checked: ref -> levelgimmicksceneobjectinfo, 0 = none. Housing_Island
        // -> HousingIslandMovePoint, InsectStorage -> InsectStorageMovePoint
        // (= _cameraPivotTargetLevelGimmickSceneObjectInfo, the 1.16 tail_e_116;
        // name kept: MOD CONTRACT).
        pub tail_e_116: u32,
        // 0x3F490FDB (pi/4 as f32) on Housing_Castle, 0xFFFFFFFF elsewhere
        // (kept u32: the 0xFFFFFFFF default is a NaN as f32, which JSON cannot carry).
        pub unk_u32_47: u32,
        // 0xFFFFFFFF on every record.
        pub unk_u32_51: u32,
        // inferred: per-phase hash right before the ratio, the same pair order
        // as 1.16 (Housing, FactionManagement* and WithoutHousing share one value).
        pub camera_preset_name_hash: u32,
        // inferred: 0.055 / 0.025 / 0.01 / 0.6 / 0.25.
        pub camera_pivot_offset_y_ratio: f32,
        // Hash on Housing* and FactionManagement* (0x0ACA072B), 0 elsewhere.
        pub unk_u32_63: u32,
        // 8.0 on every record except Housing_Castle (20.0).
        pub unk_f32_67: f32,
        // inferred: 500 Housing / Housing_Island / FactionManagement*, 1000
        // Housing_Castle, 0 on the three phases without housing placement.
        pub housing_placement_limit: u32,
        // Hash on Housing* and FactionManagement* (0xBBEA14E2), 0 elsewhere.
        pub unk_u32_75: u32,
        // New in 2.04: list of name hashes, 0-2 entries.
        pub unk_list_79: CArray<u32>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The dlc_01 header has 0 records and no body; a missing fixture is skipped.
    const FIXTURES: &[&str] = &["contentsphaseinfo.pabgb", "contentsphaseinfo.dlc_01.pabgb"];

    fn read_all(data: &[u8]) -> Vec<ContentsPhaseInfo<'_>> {
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(ContentsPhaseInfo::read_from(data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        items
    }

    #[test]
    fn roundtrip() {
        for name in FIXTURES {
            let Ok(data) = std::fs::read(crate::testenv::resolve(name)) else {
                eprintln!("SKIP: fixture {name} not found");
                continue;
            };
            let items = read_all(&data);
            let mut out = Vec::with_capacity(data.len());
            for item in &items {
                item.write_to(&mut out).unwrap();
            }
            assert_eq!(out, data, "{name}: roundtrip bytes mismatch");
            eprintln!("OK: {name} {} records", items.len());
        }
    }

    #[test]
    fn json_roundtrip() {
        for name in FIXTURES {
            let Ok(data) = std::fs::read(crate::testenv::resolve(name)) else {
                eprintln!("SKIP: fixture {name} not found");
                continue;
            };
            for (i, item) in read_all(&data).iter().enumerate() {
                let dict = item.to_json_dict();
                let mut from_typed = Vec::new();
                item.write_to(&mut from_typed).unwrap();
                let mut from_json = Vec::new();
                ContentsPhaseInfo::write_from_json_dict(&mut from_json, &dict)
                    .unwrap_or_else(|e| panic!("{name} entry {i} key={}: {e}", item.key));
                assert_eq!(from_json, from_typed,
                    "{name} entry {i} key={}: JSON round-trip diverges from typed write", item.key);
            }
        }
    }
}
