// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Parser for `npcfriendlyinfo` (dlc_01 only, 153 records on game 2.04.08).
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
//! One record per pair of characters. Every record is 22 bytes and string_key
//! is empty in all of them, so a mod addresses a record by key.
//! The Mac catalog lists one data field, `_npcFriendlyList`; these 9 bytes are
//! probably its element (inferred).
//!
//! **The key is a u64**: key = character_info | (friendly_character_info << 32)
//! in all 153 records. Header: u16 count + N x {u64 key, u32 offset}
//! (12-byte pabgh entries), so a pabgh rebuild for this table needs an 8-byte
//! key width. Body records are contiguous in offset order, so the table reads
//! sequentially.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct NpcFriendlyInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u64,
        pub is_blocked: u8,
        // ref -> characterinfo (125 of 125 distinct values resolve; e.g. Lauren 439,
        // Arlan 440, Grundir 500, Barden 620). Which side is the "friendly" one is
        // inferred
        pub character_info: u32,
        // ref -> characterinfo (40 of 40 distinct values resolve)
        pub friendly_character_info: u32,
        // 1 in 121 records, 0 in 32
        pub unk_flag: u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A missing fixture is skipped.
    const FIXTURES: &[&str] = &[
        "npcfriendlyinfo.pabgb",
        "npcfriendlyinfo.dlc_01.pabgb",
    ];

    fn read_all(data: &[u8]) -> Vec<NpcFriendlyInfo<'_>> {
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(NpcFriendlyInfo::read_from(data, &mut offset).unwrap());
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
                NpcFriendlyInfo::write_from_json_dict(&mut from_json, &dict)
                    .unwrap_or_else(|e| panic!("{name} entry {i} key={}: {e}", item.key));
                assert_eq!(from_json, from_typed,
                    "{name} entry {i} key={}: JSON round-trip diverges from typed write", item.key);
            }
        }
    }

    /// The .pabgh lists every record of the body: u16 count, then
    /// {key (8 bytes), u32 offset} per record, in body order.
    #[test]
    fn pabgh_matches_body() {
        const KEY_WIDTH: usize = 8;
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
                let item = NpcFriendlyInfo::read_from(&data, &mut offset).unwrap();
                assert_eq!(u64::from_le_bytes(k), item.key as u64, "{ph} entry {i}: key");
            }
            assert_eq!(offset, data.len(), "{ph}: count covers the body");
        }
    }
}
