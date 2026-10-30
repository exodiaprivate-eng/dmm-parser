//! Tier 1 — fully typed parser for `SpawningPoolAutoSpawnInfo.pabgb`.
//!
//! Per IDA sub_1410F9B80: 16 fields. `_spawnList` is `CArray<AutoSpawnEntry>`
//! via sub_1411092E0 + sub_1410FA2A0 (shared with
//! TerrainRegionAutoSpawnInfo; element layout in
//! `crate::binary::variants::auto_spawn_entry`).
//!
//! **spawn_list inter-element separator**: same N-1 pattern as
//! TerrainRegionAutoSpawnInfo — a single `0xff` byte between consecutive
//! elements. Additionally, a mandatory trailing `0xff` byte is written after the
//! last element regardless of count (F4 in the 16-field wire layout). This is
//! decoded as a separate `terminator: u8` field so the roundtrip is lossless and
//! the field remains visible/patchable in JSON.

use crate::binary::*;
use crate::binary::variants::auto_spawn_entry::AutoSpawnEntry;
use crate::py_binary_struct;
use std::io;

py_binary_struct! {
    // 2.04.00 order (Mac reader sub_10206A8A8), with the game's names. The pre-2.04 model was
    // a width fit: its `terminator` byte after the spawn list (and the N-1 "separators"
    // between entries) were each AutoSpawnEntry's `_indoorType`, which the entry reads itself
    // now.
    pub struct SpawningPoolAutoSpawnInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u32,
        pub is_blocked: u8,
        pub use_random_rotation: u8,
        pub collect_filter_dev: u8,
        pub check_forbidden_area: u8,
        pub is_exist_indoor_type: u8,
        pub attach_to_socket: u8,
        pub type_: u8,
        pub level_action_point_info: u32,
        pub attach_terrain_check_height: f32,
        pub near_inner_radius: f32,
        pub spawn_safety_distance: f32,
        pub near_outer_radius: f32,
        pub mesh_group_spawn_offset: [f32; 3],
        pub spawn_list: CArray<AutoSpawnEntry>,
        pub mesh_name_list: CArray<u32>,          // StringInfoKeys
        pub spawning_pool_data: u32,              // StringInfoKey
    }
}

impl<'a> SpawningPoolAutoSpawnInfo<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        if *offset - start != entry_size {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!(
                "SpawningPoolAutoSpawnInfo: consumed {} of {} bytes", *offset - start, entry_size)));
        }
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("spawningpoolautospawninfo.pabgb") }

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP: fixture not found"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP: pabgh not found"); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = SpawningPoolAutoSpawnInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end - start, e));
            assert_eq!(cursor, *end,
                "entry {} key=0x{:x}: cursor {} != end {} (under/over-read by {} bytes)",
                i, key, cursor, end, cursor as i64 - *end as i64);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "spawningpoolautospawninfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
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
            let mut cursor = *start;
            let item = SpawningPoolAutoSpawnInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end - start, e));
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            SpawningPoolAutoSpawnInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
