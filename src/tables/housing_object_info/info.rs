// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Parser for `housingobjectinfo` (base 718 records, dlc_01 590 records on
//! game 2.04.03).
//!
//! Decoded from the 2.04.03 game data (Steam build 26345091), not from the exe:
//! the exe holds no static-table field names, so every field name is a proposal.
//! Byte layout: confirmed. Every base and dlc_01 record round-trips byte for byte
//! (tests below,).
//!
//! Comment tags: `ref` = every value is a key of the named table (or the stated
//! "none" value); `ref, both ways` = the target points back; `inferred` = meaning
//! read from the values; `unk_` = meaning unknown. Offsets in `unk_*_N` names
//! count from `is_blocked` (offset 0).
//!
//! Header: u16 count + N x {u16 key, u32 offset} (6-byte pabgh entries). Body
//! records are contiguous in offset order, so the table reads sequentially.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    // Item + quantity. On every 2.04.03 record unk_count_b == count and
    // unk_count_c == count + 1 (meaning unknown). The management tables use the
    // same 28-byte shape.
    pub struct ItemCountData {
        // ref -> iteminfo (housing costs use items 11, 13, 14 and 15)
        pub item_info: u32,
        // inferred: the quantity
        pub count: u64,
        pub unk_count_b: u64,
        pub unk_count_c: u64,
    }
}

py_binary_struct! {
    pub struct HousingObjectCostData {
        // inferred: identifies the cost entry; cost_hash_list repeats these hashes
        pub unk_hash: u32,
        pub item_list: CArray<ItemCountData>,
    }
}

py_binary_struct! {
    pub struct HousingObjectInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u16,
        pub is_blocked: u8,
        // inferred: object kind; 7 on most records, also 1, 2, 4, 5, 6 and 14
        pub object_type: u8,
        // rarely non-zero
        pub unk_u8_2: u8,
        pub unk_u8_3: u8,
        pub unk_u8_4: u8,
        pub unk_u8_5: u8,
        pub unk_u8_6: u8,
        pub unk_u8_7: u8,
        // ref -> knowledgeinfo; 0 = none
        pub knowledge_info: u32,
        // ref -> stringinfo; 0xEAC5E173 = none
        pub string_info: u32,
        // ref -> gimmickinfo
        pub gimmick_info: u32,
        // one non-zero value (18789) on 2.04.03
        pub unk_u16_20: u16,
        // ref, both ways -> housingobjectgroupinfo (the group lists this object);
        // 0 = none (190 dlc_01 building pieces such as Floor and DefaultTiledWall)
        pub housing_object_group_info: u16,
        // inferred: bit field; bits 0x1, 0x2, 0x4, 0x8 and 0x10 seen
        pub unk_flags_24: u32,
        pub unk_u32_28: u32,
        pub unk_u32_32: u32,
        pub unk_f32_36: f32,
        pub unk_u32_40: u32,
        pub prefab_path: CString<'a>,
        // inferred: build cost (materials)
        pub cost_list: CArray<HousingObjectCostData>,
        // inferred: the same 3-5 hashes as cost_list[].unk_hash
        pub cost_hash_list: CArray<u32>,
        // usually empty (category 0x35)
        pub unk_text_1: LocalizableString<'a>,
        pub unk_text_2: LocalizableString<'a>,
        pub unk_text_3: LocalizableString<'a>,
        pub unk_u32_t0: u32,
        pub unk_u32_t1: u32,
        pub unk_u32_t2: u32,
        pub unk_u32_t3: u32,
        pub unk_path_2: CString<'a>,
        pub unk_path_3: CString<'a>,
        pub unk_u32_t4: u32,
        pub unk_u32_t5: u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Base and dlc_01 bodies share this layout; a missing fixture is skipped.
    const FIXTURES: &[&str] = &[
        "housingobjectinfo.pabgb",
        "housingobjectinfo.dlc_01.pabgb",
    ];

    fn read_all(data: &[u8]) -> Vec<HousingObjectInfo<'_>> {
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(HousingObjectInfo::read_from(data, &mut offset).unwrap());
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
                HousingObjectInfo::write_from_json_dict(&mut from_json, &dict)
                    .unwrap_or_else(|e| panic!("{name} entry {i} key={}: {e}", item.key));
                assert_eq!(from_json, from_typed,
                    "{name} entry {i} key={}: JSON round-trip diverges from typed write", item.key);
            }
        }
    }
}
