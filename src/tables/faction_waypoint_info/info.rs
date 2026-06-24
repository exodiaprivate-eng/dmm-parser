//! Tier 1 — fully typed parser.
//!
//! Reader (Mac CrimsonDesert_Steam): `sub_10186AF88` at 0x10186AF88.
//! Wire layout:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. FactionWayPointData way_point_data (sub_10186AD6C):
//!      - u32 from_node_info  (FactionNodeKey lookup, wire 4)
//!      - u32 to_node_info    (FactionNodeKey lookup, wire 4)
//!      - CArray<[u8;12]> way_point_list  (CArray of vec3s via
//!        sub_1013AC340 → sub_1006B48A8 12-byte element reader)
//!
//! **T0-V verification (iter 3 of T0 verification loop, IDA Win 1.06):**
//! FactionWayPointInfo is NOT in NattKh's pabgb_complete_schema.json.
//! IDA cross-references against the in-binary metaobject at
//! 0x144af0d80+. **7/7 fields verified canonical** (full match):
//!
//! Top-level `FactionWaypointInfo` (canonical: `FactionWayPointInfo`):
//! | rust field | canonical PA name | IDA address |
//! |---|---|---|
//! | `key` | `_key` | 0x144af0fc6 ✓ |
//! | `string_key` | `_stringKey` | 0x144af0d96 ✓ |
//! | `is_blocked` | `_isBlocked` | 0x144af0de6 ✓ |
//! | `way_point_data` | `_wayPointData` | 0x144af0e36 ✓ |
//!
//! Nested `FactionWayPointData`:
//! | rust field | canonical PA name | IDA address |
//! |---|---|---|
//! | `from_node_info` | `_fromNodeInfo` | 0x144af0ed6 ✓ |
//! | `to_node_info` | `_toNodeInfo` | 0x144af0f26 ✓ |
//! | `way_point_list` | `_wayPointList` | 0x144af0f76 ✓ |
//!
//! Note: rust uses `Waypoint` (one word) in struct names; canonical PA
//! uses `WayPoint` (two words / camelCase). Functionally identical.
//! Status: **T0-V FULL** — all fields verified canonical.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    /// `way_point_list` is a CArray of Vec3 waypoint coordinates per the
    /// per-element reader (sub_1006B48A8 reads 12 bytes = 3 × f32).
    pub struct FactionWayPointData {
        pub from_node_info: u32,
        pub to_node_info: u32,
        pub way_point_list: CArray<[f32; 3]>,
    }
}

py_binary_struct! {
    pub struct FactionWaypointInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub way_point_data: FactionWayPointData,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("factionwaypoint.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            let item = FactionWaypointInfo::read_from(&data, &mut c)
                .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er));
            assert_eq!(c, *e, "entry {} k=0x{:x} consumed {} of {} bytes", i, k, c - s, e - s);
            items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "factionwaypoint roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else {
            eprintln!("SKIP: pabgh not found");
            return;
        };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut c = *start;
            let item = FactionWaypointInfo::read_from(&data, &mut c).unwrap();
            assert_eq!(c, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            FactionWaypointInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
