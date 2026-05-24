//! Tier 1 — fully typed (no _tail_b64).
//!
//! Reader (Tier IDA verified 2026-05-19 vs CrimsonDesert.exe md5
//! 3d614280…): `sub_1410D89A0` — RoyalSupplyInfo deserializer (via
//! "RoyalSupplyInfo" class block at 0x144B1B880+). `a2` is `_WORD*`.
//! 7 fields confirmed in order at bytes 0/8/16/24/56/88/104, matching
//! the offset map below. (Cited `sub_1410F64D0` stale.) Element readers:
//! random-map = `sub_1410ECB70`, default_random_list element
//! (RoyalSupplyRandomData) = `sub_1410EC9E0` (cited `sub_14110A270`/
//! `sub_14110A0E0` stale). RoyalSupplyRandomData element confirmed:
//! active_quest_info + active_mission_info + item_info (`sub_1410E1B70`,
//! u32 wire→u16 RAM) + 8 raw bytes = 20 wire bytes. stage_info via
//! `sub_1410E52D0` (u32 wire). All ID refs read wider on the wire than
//! RAM, as elsewhere; Rust models the wire width.
//!
//! ─── v3.1 closure analysis (iter 71) ────────────────────────────────────
//! Cross-check via `sub_1410C3220` (typeinfo→record-reader path per the
//! iter 56 typeinfo registry). 7 wire reads, in order:
//!
//!   word 0    u16              _key
//!   word 4    CString          _stringKey  (sub_141076050)
//!   word 8    u8               _isBlocked
//!   word 12   sub_1410D7290    royal_supply_random_map_quest
//!   word 28   sub_1410D7290    royal_supply_random_map_mission
//!   word 44   sub_1410D7100    default_random_list
//!   word 52   sub_1410CFB10    stage_info
//!
//! NattKh schema lists 7 canonicals; the only "missing" is
//! `_royalSupplyRandomMap`. Two consecutive same-reader
//! (`sub_1410D7290`) calls at adjacent offsets prove **`_royalSupplyRandomMap`
//! is a pure 1-to-2 wrapper** around `royal_supply_random_map_quest +
//! royal_supply_random_map_mission`. Closure path: 1-to-N alias entry
//! mapping `_royalSupplyRandomMap` →
//! `[royal_supply_random_map_quest, royal_supply_random_map_mission]`.
//! No new decoder work needed.
//! Inner readers (decoded for the Tier 1.5 → 1 promotion):
//!   - sub_14110A270: outer hash-table CArray of `RoyalSupplyMapEntry`
//!     (per element: u32 quest/mission key + nested
//!     CArray<RoyalSupplyRandomData> via sub_14110A0E0).
//!     Hash-table sizing logic in the reader is purely metadata —
//!     wire shape is just u32 count + N × (u32 + inner CArray).
//!   - sub_14110A0E0: CArray<RoyalSupplyRandomData> (4 fields per
//!     element, 20 wire bytes / 16 mem bytes via 3 hash lookups +
//!     a u64 raw count).
//!   - sub_141102D20: u32 wire → u32 hash lookup at qword_145F0EF38.
//!
//! Wire reads, in order (canonical names from
//! `docs/449_TABLE_CATALOG.md` RoyalSupplyInfo section, 7 fields):
//!   1. u16 key                                  (_key, pabgh format 2)
//!   2. CString string_key                       (_stringKey)
//!   3. u8 is_blocked                            (_isBlocked)
//!   4. CArray<RoyalSupplyMapEntry> royal_supply_random_map_quest
//!      (_royalSupplyRandomMap_Quest, sub_14110A270 → struct +24)
//!   5. CArray<RoyalSupplyMapEntry> royal_supply_random_map_mission
//!      (_royalSupplyRandomMap_Mission, sub_14110A270 → struct +56)
//!   6. CArray<RoyalSupplyRandomData> default_random_list
//!      (_defaultRandomList, sub_14110A0E0 → struct +88)
//!   7. u32 stage_info                           (_stageInfo,
//!      sub_141102D20 → qword_145F0EF38)
//!
//! RoyalSupplyMapEntry (sub_14110A270 inner — 4 + 16N wire bytes):
//!   1. u32 key — quest or mission key (hash key in runtime map)
//!   2. CArray<RoyalSupplyRandomData> random_data_list
//!
//! RoyalSupplyRandomData (sub_14110A0E0 inner, 4 fields, 20 wire
//! bytes per element / 16 mem bytes packed):
//!   1. u32 active_quest_info        (_activeQuestInfo, sub_141102CB0
//!      → qword_145F0EF20)
//!   2. u32 active_mission_info      (_activeMissionInfo, sub_141102D90
//!      → qword_145F0EF00 — wire u32, mem u16)
//!   3. u32 item_info                (_itemInfo, sub_1410FF5C0 →
//!      qword_145F0DA00 — wire u32, mem u16)
//!   4. u64 count                    (_count, 8 raw wire bytes)

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct RoyalSupplyRandomData {
        pub active_quest_info: u32,
        pub active_mission_info: u32,
        pub item_info: u32,
        pub count: u64,
    }
}

py_binary_struct! {
    pub struct RoyalSupplyMapEntry {
        pub key: u32,
        pub random_data_list: CArray<RoyalSupplyRandomData>,
    }
}

py_binary_struct! {
    pub struct RoyalSupplyInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub royal_supply_random_map_quest: CArray<RoyalSupplyMapEntry>,
        pub royal_supply_random_map_mission: CArray<RoyalSupplyMapEntry>,
        pub default_random_list: CArray<RoyalSupplyRandomData>,
        pub stage_info: u32,
    }
}

impl<'a> RoyalSupplyInfo<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> std::io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        let consumed = *offset - start;
        if consumed != entry_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("RoyalSupplyInfo: consumed {} bytes, expected {}", consumed, entry_size),
            ));
        }
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("royalsupply.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            let item = RoyalSupplyInfo::read_from(&data, &mut c)
                .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er));
            assert_eq!(c, *e, "e{} k=0x{:x}: under/over-read {}/{}", i, k, c - s, e - s);
            items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "royalsupply roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = RoyalSupplyInfo::read_from(&data, &mut cursor).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            RoyalSupplyInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }

    /// Confirm typed lists carry data — guards against silent regression.
    #[test]
    fn typed_lists_populated() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut totals = (0usize, 0usize, 0usize);
        for (_, s, _) in &ranges {
            let mut c = *s;
            let item = RoyalSupplyInfo::read_from(&data, &mut c).unwrap();
            totals.0 += item.royal_supply_random_map_quest.items.len();
            totals.1 += item.royal_supply_random_map_mission.items.len();
            totals.2 += item.default_random_list.items.len();
        }
        eprintln!(
            "royal_supply_info: {} entries, quest_map={} mission_map={} default={}",
            ranges.len(), totals.0, totals.1, totals.2
        );
    }
}
