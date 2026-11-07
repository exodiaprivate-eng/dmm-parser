// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Parser for `npcnameinfo` (dlc_01 only, 10 records on game 2.04.08:
//! NpcName_Human_South/_North/_Desert, _Troll_Scholar, _Troll_Noral, _Goblin,
//! _Dwarf, _Orc, _Shay, _Padus).
//!
//! Decoded from the 2.04.08 game data (Steam build 26480954), not from the exe:
//! the exe holds no static-table field names, so every field name is a proposal.
//! Byte layout: confirmed. Every record round-trips byte for byte, and through
//! JSON (tests below).
//!
//! Comment tags: `ref` = every non-zero value is a key of the named table;
//! `inferred` = meaning read from the values; `unk_` = meaning unknown (width
//! confirmed by the round-trip, meaning not).
//!
//! Name pools for generated NPCs, by race and region. Three name lists per
//! record. Their roles are inferred: name_list_0 = given names (gender 1/2),
//! name_list_1 = family names, only on the human records whose job_list_1 is
//! Job_Noble + Job_NobleRider, name_list_2 = a third pool (120 names each for
//! goblin, dwarf and orc).
//!
//! Header: u16 count + N x {u32 key, u32 offset} (8-byte pabgh entries). Body
//! records are contiguous in offset order, so the table reads sequentially.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct NpcNameData<'a> {
        // 1 or 2 in name_list_0 (0 on 37 entries), 0 in the other lists (inferred: gender)
        pub gender: u8,
        // category 0x37. LocalizableString: index = (record key << 32) | (n << 18) |
        // 0x370, n = running number across name_list_0, _1 and _2; default = that
        // index in decimal; the text itself is in the localisation file
        pub name: LocalizableString<'a>,
        // = position in its list
        pub index: u32,
    }
}

py_binary_struct! {
    pub struct NpcNameInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u32,
        pub is_blocked: u8,
        // hash; the same on the 3 human records and on the 2 troll records
        pub unk_hash: u32,
        // = 0..9 in record order
        pub unk_index: u32,
        // ref -> regioninfo (South = Akapen, Delpheon, Brondell; North = Kweiden;
        // Desert = Crimson_Desert)
        pub region_list: CArray<u16>,
        // ref -> jobinfo (Troll_Scholar -> Job_Scholar)
        pub job_list_0: CArray<u16>,
        // 0 in every record
        pub unk_u32_a: u32,
        pub name_list_0: CArray<NpcNameData<'a>>,
        // ref -> jobinfo (human records -> Job_Noble, Job_NobleRider)
        pub job_list_1: CArray<u16>,
        pub name_list_1: CArray<NpcNameData<'a>>,
        // 0 in every record
        pub unk_u32_b: u32,
        pub name_list_2: CArray<NpcNameData<'a>>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A missing fixture is skipped.
    const FIXTURES: &[&str] = &[
        "npcnameinfo.pabgb",
        "npcnameinfo.dlc_01.pabgb",
    ];

    fn read_all(data: &[u8]) -> Vec<NpcNameInfo<'_>> {
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(NpcNameInfo::read_from(data, &mut offset).unwrap());
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
                NpcNameInfo::write_from_json_dict(&mut from_json, &dict)
                    .unwrap_or_else(|e| panic!("{name} entry {i} key={}: {e}", item.key));
                assert_eq!(from_json, from_typed,
                    "{name} entry {i} key={}: JSON round-trip diverges from typed write", item.key);
            }
        }
    }

    /// The .pabgh lists every record of the body: u16 count, then
    /// {key (4 bytes), u32 offset} per record, in body order.
    #[test]
    fn pabgh_matches_body() {
        const KEY_WIDTH: usize = 4;
        for name in FIXTURES {
            let ph = name.replace(".pabgb", ".pabgh");
            let (Ok(data), Ok(h)) = (std::fs::read(crate::testenv::resolve(name)),
                                     std::fs::read(crate::testenv::resolve(&ph))) else {
                eprintln!("SKIP: fixture {name} or {ph} not found");
                continue;
            };
            let count = u16::from_le_bytes([h[0], h[1]]) as usize;
            assert_eq!(h.len(), 2 + count * (KEY_WIDTH + 4), "{ph}: size");
            let mut offset = 0;
            for i in 0..count {
                let e = 2 + i * (KEY_WIDTH + 4);
                let mut k = [0u8; 8];
                k[..KEY_WIDTH].copy_from_slice(&h[e..e + KEY_WIDTH]);
                let at = u32::from_le_bytes(h[e + KEY_WIDTH..e + KEY_WIDTH + 4].try_into().unwrap());
                assert_eq!(at as usize, offset, "{ph} entry {i}: offset");
                let item = NpcNameInfo::read_from(&data, &mut offset).unwrap();
                assert_eq!(u64::from_le_bytes(k), item.key as u64, "{ph} entry {i}: key");
            }
            assert_eq!(offset, data.len(), "{ph}: count covers the body");
        }
    }
}
