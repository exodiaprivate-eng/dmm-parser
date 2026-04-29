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

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct FactionWayPointData {
        pub from_node_info: u32,
        pub to_node_info: u32,
        pub way_point_list: CArray<[u8; 12]>,
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
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\factionwaypoint.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\factionwaypoint.pabgh";
    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
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
}
