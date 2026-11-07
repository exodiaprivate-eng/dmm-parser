// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Parser for `managementneedssynergyinfo` (dlc_01 only, 30 records on game
//! 2.04.03).
//!
//! Decoded from the 2.04.03 game data (Steam build 26345091), not from the exe:
//! the exe holds no static-table field names, so every field name is a proposal.
//! Byte layout: confirmed. Every record round-trips byte for byte (tests below,
//!).
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
    pub struct ManagementNeedsSynergyLevelData {
        // inferred: how many needs of this kind must be met to reach the level
        pub required_count: u32,
        // always 1 on 2.04.03
        pub unk_u32: u32,
        // ref -> managementbuffinfo (this synergy's Lv1-3 buffs, 107 of 107 resolve).
        // 89 of those buffs name this synergy back in effect.management_needs_synergy_info;
        // the 18 of Traditional, Delicacy, Refined, Classic, Luxury and Simple name none
        pub management_buff_info: u16,
    }
}

py_binary_struct! {
    pub struct ManagementNeedsSynergyInfo<'a> {
        // e.g. Meat, Seafood, Grain
        pub string_key: CString<'a>,
        pub key: u16,
        pub is_blocked: u8,
        // ref -> stringinfo; 0xEAC5E173 = none
        pub string_info: u32,
        pub synergy_name: LocalizableString<'a>,
        pub synergy_desc: LocalizableString<'a>,
        pub level_list: CArray<ManagementNeedsSynergyLevelData>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The table ships in dlc_01 only; a missing fixture is skipped.
    const FIXTURES: &[&str] = &[
        "managementneedssynergyinfo.pabgb",
        "managementneedssynergyinfo.dlc_01.pabgb",
    ];

    fn read_all(data: &[u8]) -> Vec<ManagementNeedsSynergyInfo<'_>> {
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(ManagementNeedsSynergyInfo::read_from(data, &mut offset).unwrap());
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
                ManagementNeedsSynergyInfo::write_from_json_dict(&mut from_json, &dict)
                    .unwrap_or_else(|e| panic!("{name} entry {i} key={}: {e}", item.key));
                assert_eq!(from_json, from_typed,
                    "{name} entry {i} key={}: JSON round-trip diverges from typed write", item.key);
            }
        }
    }
}
