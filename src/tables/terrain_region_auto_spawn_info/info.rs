//! Hand-corrected: IDA-derived parser for `TerrainRegionAutoSpawnInfo.pabgb`.
//!
//! Per IDA sub_1410FA5B0: 24 fields. _spawnList (field 11) is polymorphic
//! CArray via sub_1411092E0 with deeply-nested element structure. The
//! polymorphic block is captured as raw bytes by probing trailing-field
//! validity from candidate boundaries.

use crate::binary::variant::find_variant_boundary;
use crate::binary::*;
use std::io::{self, Write};

#[derive(Debug)]
pub struct TerrainRegionAutoSpawnInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub possible_list: CArray<u8>,
    pub auto_spawn_spline_name: CArray<CString<'a>>,
    pub auto_spawn_spline_except_name: CArray<CString<'a>>,
    pub region_info_list: CArray<u16>,
    pub not_spawn_region_info_list: CArray<u16>,
    pub spawn_region_tag_list: CArray<u32>,
    pub not_spawn_region_tag_list: CArray<u32>,
    pub spawn_list: Vec<u8>,
    pub voxel_type: u32,
    pub road_group_type: u8,
    pub is_only_summon_data: u8,
    pub is_only_check_data: u8,
    pub stage_category: u8,
    pub tag_list: CArray<CString<'a>>,
    pub is_default_activated: u8,
    pub all_terrain_region: u8,
    pub bitmap_position_info: u32,
    pub bitmap_color_list_for_spawn: CArray<u8>,
    pub spawn_at_height_field_landscape: u8,
    pub fish_summon_time_frequency_type: u8,
    pub spawn_reason_list: CArray<u32>,
}

/// Try to read the tail starting at `probe`. Returns Some(bytes_consumed) on
/// success. Used by the variant boundary probe.
fn try_read_tail(data: &[u8], probe: usize, end: usize) -> Option<usize> {
    let mut cursor = probe;
    // 4 + 1 + 1 + 1 + 1
    if cursor + 8 > end { return None; }
    let _voxel_type = u32::read_from(data, &mut cursor).ok()?;
    let _road_group = u8::read_from(data, &mut cursor).ok()?;
    let _only_summon = u8::read_from(data, &mut cursor).ok()?;
    let _only_check = u8::read_from(data, &mut cursor).ok()?;
    let _stage_cat = u8::read_from(data, &mut cursor).ok()?;
    // CArray<CString>
    let _tag_list = CArray::<CString>::read_from(data, &mut cursor).ok()?;
    if cursor > end { return None; }
    // 1 + 1 + 4
    if cursor + 6 > end { return None; }
    let _is_default = u8::read_from(data, &mut cursor).ok()?;
    let _all_terrain = u8::read_from(data, &mut cursor).ok()?;
    let _bitmap_pos = u32::read_from(data, &mut cursor).ok()?;
    let _bitmap_color = CArray::<u8>::read_from(data, &mut cursor).ok()?;
    if cursor > end { return None; }
    if cursor + 2 > end { return None; }
    let _spawn_at_height = u8::read_from(data, &mut cursor).ok()?;
    let _fish_freq = u8::read_from(data, &mut cursor).ok()?;
    let _reason_list = CArray::<u32>::read_from(data, &mut cursor).ok()?;
    if cursor != end { return None; }
    Some(cursor - probe)
}

impl<'a> TerrainRegionAutoSpawnInfo<'a> {
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
    ) -> io::Result<Self> {
        let entry_start = *offset;
        let entry_end = entry_start + entry_size;

        let key = u32::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;
        let possible_list = CArray::<u8>::read_from(data, offset)?;
        let auto_spawn_spline_name = CArray::<CString>::read_from(data, offset)?;
        let auto_spawn_spline_except_name = CArray::<CString>::read_from(data, offset)?;
        let region_info_list = CArray::<u16>::read_from(data, offset)?;
        let not_spawn_region_info_list = CArray::<u16>::read_from(data, offset)?;
        let spawn_region_tag_list = CArray::<u32>::read_from(data, offset)?;
        let not_spawn_region_tag_list = CArray::<u32>::read_from(data, offset)?;

        let post_pre = *offset;
        let variant_size = find_variant_boundary(data, post_pre, entry_end, 4, |probe| {
            try_read_tail(data, probe, entry_end)
        })?;

        let spawn_list = data[post_pre..post_pre + variant_size].to_vec();
        *offset = post_pre + variant_size;

        let voxel_type = u32::read_from(data, offset)?;
        let road_group_type = u8::read_from(data, offset)?;
        let is_only_summon_data = u8::read_from(data, offset)?;
        let is_only_check_data = u8::read_from(data, offset)?;
        let stage_category = u8::read_from(data, offset)?;
        let tag_list = CArray::<CString>::read_from(data, offset)?;
        let is_default_activated = u8::read_from(data, offset)?;
        let all_terrain_region = u8::read_from(data, offset)?;
        let bitmap_position_info = u32::read_from(data, offset)?;
        let bitmap_color_list_for_spawn = CArray::<u8>::read_from(data, offset)?;
        let spawn_at_height_field_landscape = u8::read_from(data, offset)?;
        let fish_summon_time_frequency_type = u8::read_from(data, offset)?;
        let spawn_reason_list = CArray::<u32>::read_from(data, offset)?;

        Ok(Self {
            key, string_key, is_blocked, possible_list,
            auto_spawn_spline_name, auto_spawn_spline_except_name,
            region_info_list, not_spawn_region_info_list,
            spawn_region_tag_list, not_spawn_region_tag_list,
            spawn_list, voxel_type, road_group_type,
            is_only_summon_data, is_only_check_data, stage_category,
            tag_list, is_default_activated, all_terrain_region,
            bitmap_position_info, bitmap_color_list_for_spawn,
            spawn_at_height_field_landscape, fish_summon_time_frequency_type,
            spawn_reason_list,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.possible_list.write_to(w)?;
        self.auto_spawn_spline_name.write_to(w)?;
        self.auto_spawn_spline_except_name.write_to(w)?;
        self.region_info_list.write_to(w)?;
        self.not_spawn_region_info_list.write_to(w)?;
        self.spawn_region_tag_list.write_to(w)?;
        self.not_spawn_region_tag_list.write_to(w)?;
        w.write_all(&self.spawn_list)?;
        self.voxel_type.write_to(w)?;
        self.road_group_type.write_to(w)?;
        self.is_only_summon_data.write_to(w)?;
        self.is_only_check_data.write_to(w)?;
        self.stage_category.write_to(w)?;
        self.tag_list.write_to(w)?;
        self.is_default_activated.write_to(w)?;
        self.all_terrain_region.write_to(w)?;
        self.bitmap_position_info.write_to(w)?;
        self.bitmap_color_list_for_spawn.write_to(w)?;
        self.spawn_at_height_field_landscape.write_to(w)?;
        self.fish_summon_time_frequency_type.write_to(w)?;
        self.spawn_reason_list.write_to(w)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\terrainregionautospawninfo.pabgb";
    const PABGH_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\terrainregionautospawninfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
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
}
