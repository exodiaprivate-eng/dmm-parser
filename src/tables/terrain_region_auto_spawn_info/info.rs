//! Tier 1 — fully typed parser for `TerrainRegionAutoSpawnInfo.pabgb`.
//!
//! Per IDA sub_1410FA5B0: 24 fields (old binary). New binary adds `spawn_mode_type` (u8) after
//! `stage_category`, giving 25 fields. `_spawnList` is
//! `CArray<AutoSpawnEntry>` via sub_1411092E0 + sub_1410FA2A0 (shared with
//! SpawningPoolAutoSpawnInfo). Per-element wire layout reverse-engineered
//! and lives in `crate::binary::variants::auto_spawn_entry`. Despite the original
//! "polymorphic" docstring, sub_1410FA2A0 is fixed-shape.
//!
//! **spawn_list inter-element separator**: `CArray<AutoSpawnEntry>` encodes a single
//! 0xff byte between consecutive elements (N elements → N-1 separators). Entries with
//! one ASE carry no separator; entries with N≥2 ASEs carry one 0xff byte after each
//! non-final element. Handled explicitly in `read_with_size`/`write_to`/`write_from_json_dict`.

use crate::binary::*;
use crate::binary::variants::auto_spawn_entry::AutoSpawnEntry;
use crate::py_binary_struct;
use std::io;

py_binary_struct! {
    // 2.04.00 order (Mac reader sub_10206D788). The spawn list is a plain CArray again: the
    // pre-2.04 "N-1 separator bytes" between AutoSpawnEntry elements were each element's
    // `_indoorType`, which AutoSpawnEntry now reads itself. New: spawnAtFlatPlace,
    // spawnablePhysicsMaterialList. `spawn_mode_type` is the game's `_autoSpawnVoxelLevelType`,
    // `trailing_u32_113` its `_attachTerrainCheckHeight` (f32).
    pub struct TerrainRegionAutoSpawnInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u32,
        pub is_blocked: u8,
        pub is_default_activated: u8,
        pub fish_summon_time_frequency_type: u8,
        pub spawn_at_height_field_landscape: u8,
        pub spawn_at_flat_place: u8,
        pub all_terrain_region: u8,
        pub is_only_check_data: u8,
        pub spawn_mode_type: u8,
        pub stage_category: u8,
        pub is_only_summon_data: u8,
        pub road_group_type: u8,
        pub bitmap_position_info: u32,
        pub trailing_u32_113: u32,
        pub voxel_type: u32,
        pub possible_list: CArray<u8>,
        pub auto_spawn_spline_name: CArray<CString<'a>>,
        pub auto_spawn_spline_except_name: CArray<CString<'a>>,
        pub region_info_list: CArray<u16>,
        pub not_spawn_region_info_list: CArray<u16>,
        pub spawn_region_tag_list: CArray<u32>,
        pub not_spawn_region_tag_list: CArray<u32>,
        pub spawn_list: CArray<AutoSpawnEntry>,
        pub tag_list: CArray<CBytes<'a>>,
        pub bitmap_color_list_for_spawn: CArray<u8>,
        pub spawnable_physics_material_list: CArray<CString<'a>>,
        pub spawn_reason_list: CArray<u32>,
    }
}

impl<'a> TerrainRegionAutoSpawnInfo<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        if *offset - start != entry_size {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!(
                "TerrainRegionAutoSpawnInfo: consumed {} of {} bytes", *offset - start, entry_size)));
        }
        Ok(item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("terrainregionautospawninfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP: fixture not found"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP: pabgh not found"); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = TerrainRegionAutoSpawnInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end-start, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "terrainregionautospawninfo roundtrip bytes mismatch");
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
            let mut cursor = *start;
            let item = TerrainRegionAutoSpawnInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            TerrainRegionAutoSpawnInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
