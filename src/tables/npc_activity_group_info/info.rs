//! Parser for `npcactivitygroupinfo.pabgb` (base 56 records, dlc_01 1 record
//! on game 2.04.08, Steam build 26480954).
//! Pre-2.04 reader: `sub_1410E4660` (CrimsonDesert.exe 1.0.8).
//!
//! 2.04 layout, decoded from the game data. Byte layout: confirmed, every base
//! and dlc_01 record round-trips byte for byte (tests below). Before 2.04 the
//! record began with the u32 key; it now begins with the string key (like most
//! tables) and the fields after `is_blocked` are reordered, so the 1.16 struct
//! failed with "not enough data". Same 7 fields as the binary's 7-field list;
//! field names unchanged (parser field names are a MOD CONTRACT).
//!
//! Header: u16 count + N x {u32 key, u32 offset}. Records are contiguous in
//! header order, so the table reads sequentially.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct NpcActivityGroupInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u32,
        // 0 on every 2.04.08 record.
        pub is_blocked: u8,
        // inferred: the only u8 of the 7-field list left after is_blocked
        // (_useContentsOnly). 1 on 19 of 56 base records (e.g. Marry_NoCatch,
        // SpecialActionPoint, GroupAttack, several Management_* groups).
        pub use_contents_only: u8,
        // checked: ref -> conditioninfo, 57/57 records (base + dlc_01), e.g.
        // DecoInsect_Life -> "IsHiredMercenary() && CheckMercenaryKey(Insect) &&
        // IsDecoMercenary()", 1000531 = "CheckNone()".
        // This u32 IS the binary's `_selfConditionInfo` (name kept: MOD CONTRACT).
        pub activity_info_key: u32,
        // f32, 0.0-101.0. Not the list length (e.g. 24.0 with 6 entries).
        pub total_rate: f32,
        // checked: ref -> npcactivityinfo, 198/198 entries (base + dlc_01).
        pub npc_activity_info_list: CArray<u32>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Base and dlc_01 bodies share this layout; a missing fixture is skipped.
    const FIXTURES: &[&str] = &[
        "npcactivitygroupinfo.pabgb",
        "npcactivitygroupinfo.dlc_01.pabgb",
    ];

    fn read_all(data: &[u8]) -> Vec<NpcActivityGroupInfo<'_>> {
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(NpcActivityGroupInfo::read_from(data, &mut offset).unwrap());
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
                NpcActivityGroupInfo::write_from_json_dict(&mut from_json, &dict)
                    .unwrap_or_else(|e| panic!("{name} entry {i} key={}: {e}", item.key));
                assert_eq!(from_json, from_typed,
                    "{name} entry {i} key={}: JSON round-trip diverges from typed write", item.key);
            }
        }
    }
}
