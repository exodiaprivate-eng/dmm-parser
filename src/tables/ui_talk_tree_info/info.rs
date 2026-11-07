// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Parser for `uitalktreeinfo` (175 records on game 2.04.08, Talk_Friendly,
//! Talk_Friendly_A, Talk_Friendly_A_1 ...; the dlc_01 header holds 0 records).
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
//! A tree of talk buttons. Field names of the old `talk_tree_info` are reused
//! where the shape matches (root/parent/child keys, button_text).
//!
//! The seven `unk_*` u32 fields are 0 in every record, so only their total
//! width is confirmed, not where one field ends and the next starts. Some may
//! be the old TalkTreeInfo condition_info / target_condition_info /
//! interaction_info, and one could be a list count (an empty CArray also reads
//! as a 0 u32). Treat them as read-only until a record sets one.
//!
//! Header: u16 count + N x {u16 key, u32 offset} (6-byte pabgh entries). Body
//! records are contiguous in offset order, so the table reads sequentially.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct UITalkTreeInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u16,
        pub is_blocked: u8,
        pub unk_u32_0: u32,
        // ref -> npcactivityinfo, 0 = none (140 of 175 set; the 10 distinct
        // values are all Activity_Talk_* records)
        pub npc_activity_info: u32,
        pub unk_u32_8: u32,
        pub unk_u32_12: u32,
        // ref -> uitalktreeinfo (this table)
        pub root_talk_tree_info: u16,
        // ref -> uitalktreeinfo, 0 = none
        pub parent_talk_tree_info: u16,
        pub unk_u32_20: u32,
        // ref -> uitalktreeinfo. All 513 root/parent/child references resolve
        // in this table
        pub child_talk_tree_info_list: CArray<u16>,
        // category 0x32. LocalizableString: index = (record key << 32) | 0x320, default = that
        // index in decimal; the text itself is in the localisation file
        pub button_text: LocalizableString<'a>,
        pub unk_tail_0: u32,
        pub unk_tail_1: u32,
        pub unk_tail_2: u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A missing fixture is skipped.
    const FIXTURES: &[&str] = &[
        "uitalktreeinfo.pabgb",
        "uitalktreeinfo.dlc_01.pabgb",
    ];

    fn read_all(data: &[u8]) -> Vec<UITalkTreeInfo<'_>> {
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(UITalkTreeInfo::read_from(data, &mut offset).unwrap());
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
                UITalkTreeInfo::write_from_json_dict(&mut from_json, &dict)
                    .unwrap_or_else(|e| panic!("{name} entry {i} key={}: {e}", item.key));
                assert_eq!(from_json, from_typed,
                    "{name} entry {i} key={}: JSON round-trip diverges from typed write", item.key);
            }
        }
    }

    /// The .pabgh lists every record of the body: u16 count, then
    /// {key (2 bytes), u32 offset} per record, in body order.
    #[test]
    fn pabgh_matches_body() {
        const KEY_WIDTH: usize = 2;
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
                let item = UITalkTreeInfo::read_from(&data, &mut offset).unwrap();
                assert_eq!(u64::from_le_bytes(k), item.key as u64, "{ph} entry {i}: key");
            }
            assert_eq!(offset, data.len(), "{ph}: count covers the body");
        }
    }
}
