// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Parser for `housingobjectgroupinfo` (base 32 records, dlc_01 19 records
//! on game 2.04.03).
//!
//! Decoded from the 2.04.03 game data (Steam build 26345091), not from the exe:
//! the exe holds no static-table field names, so every field name is a proposal.
//! Byte layout: confirmed. Every base and dlc_01 record round-trips byte for byte
//! (tests below,).
//!
//! Comment tags: `ref` = every value is a key of the named table (or the stated
//! "none" value); `ref, both ways` = the target points back; `inferred` = meaning
//! read from the values; `unk_` = meaning unknown.
//!
//! Header: u16 count + N x {u16 key, u32 offset} (6-byte pabgh entries). Body
//! records are contiguous in offset order, so the table reads sequentially.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct HousingObjectGroupInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u16,
        pub is_blocked: u8,
        // ref -> stringinfo; 0xEAC5E173 = none (that value is a key in no table)
        pub string_info: u32,
        pub group_name: LocalizableString<'a>,
        // ref, both ways -> housingobjectgroupinfo; 0 = top level
        pub parent_housing_object_group_info: u16,
        // ref, both ways -> housingobjectgroupinfo (each child names this group as parent)
        pub child_housing_object_group_info_list: CArray<u16>,
        // ref, both ways -> housingobjectinfo (each object names this group)
        pub housing_object_info_list: CArray<u16>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Base and dlc_01 bodies share this layout; a missing fixture is skipped.
    const FIXTURES: &[&str] = &[
        "housingobjectgroupinfo.pabgb",
        "housingobjectgroupinfo.dlc_01.pabgb",
    ];

    fn read_all(data: &[u8]) -> Vec<HousingObjectGroupInfo<'_>> {
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(HousingObjectGroupInfo::read_from(data, &mut offset).unwrap());
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
                HousingObjectGroupInfo::write_from_json_dict(&mut from_json, &dict)
                    .unwrap_or_else(|e| panic!("{name} entry {i} key={}: {e}", item.key));
                assert_eq!(from_json, from_typed,
                    "{name} entry {i} key={}: JSON round-trip diverges from typed write", item.key);
            }
        }
    }
}
