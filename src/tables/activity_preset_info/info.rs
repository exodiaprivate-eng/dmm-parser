// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Parser for `activitypresetinfo` (11 records on game 2.04.08; the dlc_01
//! header holds 0 records).
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
//! Each record (RanchedAnimal, HouseAnimal, Default, Troll, Sailor, SailorNPC,
//! Married, Castle, Management_Customer, Management_Worker, Insect) picks the
//! NPC activity groups an NPC of that kind rolls from. The record holds a FIXED
//! 10 slots: no count on the wire, every record is 10 slots long (36 of the 110
//! slots hold groups on 2.04.08). What the 10 positions stand for is not known
//! (time-of-day phases is a guess), so they are named by position.
//!
//! Header: u16 count + N x {u32 key, u32 offset} (8-byte pabgh entries). Body
//! records are contiguous in offset order, so the table reads sequentially.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct ActivityPresetSlot {
        // ref -> conditioninfo, 0 = none (8 of 110 slots set), e.g. 1012111
        // '!CheckFocusActorMountedVehicleType(Ship)' on Sailor, 1013756
        // '!HasSummonTag(Worker_*)' on Management_Worker
        pub condition_info: u32,
        // ref -> conditioninfo, 0 = none (10 of 110 slots set), e.g. 1011683
        // 'IsGimmickPlacedByHousing()' on Married and Castle. Name inferred
        // (the old TalkTreeInfo pairs condition_info with target_condition_info)
        pub target_condition_info: u32,
        // ref -> npcactivitygroupinfo (30 of 30 distinct values resolve, e.g.
        // RanchedAnimal -> ActivityGroup_Life_Animal, 1000002 ActivityGroup_Rest)
        pub npc_activity_group_list: CArray<u32>,
        // one weight per entry of npc_activity_group_list (same length in every slot)
        pub rate_list: CArray<f32>,
        // = sum(rate_list) in every slot on 2.04.08
        pub total_rate: f32,
    }
}

py_binary_struct! {
    pub struct ActivityPresetInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u32,
        pub is_blocked: u8,
        pub slot_0: ActivityPresetSlot,
        pub slot_1: ActivityPresetSlot,
        pub slot_2: ActivityPresetSlot,
        pub slot_3: ActivityPresetSlot,
        pub slot_4: ActivityPresetSlot,
        pub slot_5: ActivityPresetSlot,
        pub slot_6: ActivityPresetSlot,
        pub slot_7: ActivityPresetSlot,
        pub slot_8: ActivityPresetSlot,
        pub slot_9: ActivityPresetSlot,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A missing fixture is skipped.
    const FIXTURES: &[&str] = &[
        "activitypresetinfo.pabgb",
        "activitypresetinfo.dlc_01.pabgb",
    ];

    fn read_all(data: &[u8]) -> Vec<ActivityPresetInfo<'_>> {
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(ActivityPresetInfo::read_from(data, &mut offset).unwrap());
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
                ActivityPresetInfo::write_from_json_dict(&mut from_json, &dict)
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
                let item = ActivityPresetInfo::read_from(&data, &mut offset).unwrap();
                assert_eq!(u64::from_le_bytes(k), item.key as u64, "{ph} entry {i}: key");
            }
            assert_eq!(offset, data.len(), "{ph}: count covers the body");
        }
    }
}
