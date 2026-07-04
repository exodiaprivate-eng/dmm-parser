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
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use serde_json::{Map, Value};
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
    pub spawn_list: CArray<AutoSpawnEntry>,
    /// Inter-element bytes between consecutive `spawn_list` elements (N-1 for N
    /// elements). Previously assumed to always be `0xff` and hardcoded on write,
    /// but real game data uses `0x00` for some entries — hardcoding `0xff`
    /// flipped those bytes and broke byte round-trip (and shipped a corrupted
    /// overlay for affected regions). We now preserve the actual bytes verbatim.
    pub spawn_list_separators: Vec<u8>,
    pub voxel_type: u32,
    pub road_group_type: u8,
    pub is_only_summon_data: u8,
    pub is_only_check_data: u8,
    pub stage_category: u8,
    pub spawn_mode_type: u8,
    pub tag_list: CArray<CBytes<'a>>,
    pub is_default_activated: u8,
    pub all_terrain_region: u8,
    pub bitmap_position_info: u32,
    pub bitmap_color_list_for_spawn: CArray<u8>,
    pub spawn_at_height_field_landscape: u8,
    pub fish_summon_time_frequency_type: u8,
    pub spawn_reason_list: CArray<u32>,
    // 1.13.00: new trailing u32 appended to every record (always 0 in vanilla;
    // likely _spawnLimitCount or _attachTerrainCheckHeight per reflection). Byte-diff
    // decisive: all records = 1.12.2 bytes + `00 00 00 00` at the tail.
    pub trailing_u32_113: u32,
}

impl<'a> TerrainRegionAutoSpawnInfo<'a> {
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
    ) -> io::Result<Self> {
        let _ = entry_size; // typed reader is byte-perfect; size is informational

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

        // spawn_list: CArray<AutoSpawnEntry> with N-1 inter-element separator bytes
        // between consecutive elements (value 0xff).  For N elements, N-1 separator
        // bytes are inserted between them.  Entries with a single ASE have no
        // separator; entries with two or more ASEs each contribute one separator byte.
        let (spawn_list, spawn_list_separators) = {
            let count = u32::read_from(data, offset)? as usize;
            let remaining = data.len().saturating_sub(*offset);
            if count > remaining {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData,
                    format!("CArray count {} exceeds remaining bytes {} at offset {}",
                        count, remaining, *offset)));
            }
            let mut items = Vec::with_capacity(count.min(1 << 20));
            let mut separators: Vec<u8> = Vec::with_capacity(count.saturating_sub(1));
            for i in 0..count {
                items.push(AutoSpawnEntry::read_from(data, offset)?);
                if i + 1 < count {
                    // preserve the inter-element separator byte (0x00 or 0xff)
                    separators.push(u8::read_from(data, offset)?);
                }
            }
            (CArray { items }, separators)
        };

        let voxel_type = u32::read_from(data, offset)?;
        let road_group_type = u8::read_from(data, offset)?;
        let is_only_summon_data = u8::read_from(data, offset)?;
        let is_only_check_data = u8::read_from(data, offset)?;
        let stage_category = u8::read_from(data, offset)?;
        let spawn_mode_type = u8::read_from(data, offset)?;
        let tag_list = CArray::<CBytes>::read_from(data, offset)?;
        let is_default_activated = u8::read_from(data, offset)?;
        let all_terrain_region = u8::read_from(data, offset)?;
        let bitmap_position_info = u32::read_from(data, offset)?;
        let bitmap_color_list_for_spawn = CArray::<u8>::read_from(data, offset)?;
        let spawn_at_height_field_landscape = u8::read_from(data, offset)?;
        let fish_summon_time_frequency_type = u8::read_from(data, offset)?;
        let spawn_reason_list = CArray::<u32>::read_from(data, offset)?;
        let trailing_u32_113 = u32::read_from(data, offset)?;

        Ok(Self {
            key, string_key, is_blocked, possible_list,
            auto_spawn_spline_name, auto_spawn_spline_except_name,
            region_info_list, not_spawn_region_info_list,
            spawn_region_tag_list, not_spawn_region_tag_list,
            spawn_list, spawn_list_separators, voxel_type, road_group_type,
            is_only_summon_data, is_only_check_data, stage_category, spawn_mode_type,
            tag_list, is_default_activated, all_terrain_region,
            bitmap_position_info, bitmap_color_list_for_spawn,
            spawn_at_height_field_landscape, fish_summon_time_frequency_type,
            spawn_reason_list, trailing_u32_113,
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
        // spawn_list: write count then elements with the PRESERVED N-1 separator
        // bytes between them. Fall back to 0xff only if a separator is missing
        // (e.g. spawn_list was grown by a mod beyond the captured separator count).
        (self.spawn_list.items.len() as u32).write_to(w)?;
        for (i, entry) in self.spawn_list.items.iter().enumerate() {
            entry.write_to(w)?;
            if i + 1 < self.spawn_list.items.len() {
                let sep = self.spawn_list_separators.get(i).copied().unwrap_or(0xff);
                w.write_all(&[sep])?;
            }
        }
        self.voxel_type.write_to(w)?;
        self.road_group_type.write_to(w)?;
        self.is_only_summon_data.write_to(w)?;
        self.is_only_check_data.write_to(w)?;
        self.stage_category.write_to(w)?;
        self.spawn_mode_type.write_to(w)?;
        self.tag_list.write_to(w)?;
        self.is_default_activated.write_to(w)?;
        self.all_terrain_region.write_to(w)?;
        self.bitmap_position_info.write_to(w)?;
        self.bitmap_color_list_for_spawn.write_to(w)?;
        self.spawn_at_height_field_landscape.write_to(w)?;
        self.fish_summon_time_frequency_type.write_to(w)?;
        self.spawn_reason_list.write_to(w)?;
        self.trailing_u32_113.write_to(w)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("possible_list".to_string(), self.possible_list.to_json_value());
        m.insert("auto_spawn_spline_name".to_string(), self.auto_spawn_spline_name.to_json_value());
        m.insert("auto_spawn_spline_except_name".to_string(), self.auto_spawn_spline_except_name.to_json_value());
        m.insert("region_info_list".to_string(), self.region_info_list.to_json_value());
        m.insert("not_spawn_region_info_list".to_string(), self.not_spawn_region_info_list.to_json_value());
        m.insert("spawn_region_tag_list".to_string(), self.spawn_region_tag_list.to_json_value());
        m.insert("not_spawn_region_tag_list".to_string(), self.not_spawn_region_tag_list.to_json_value());
        m.insert("spawn_list".to_string(), self.spawn_list.to_json_value());
        m.insert("spawn_list_separators".to_string(),
            Value::Array(self.spawn_list_separators.iter().map(|b| Value::from(*b)).collect()));
        m.insert("voxel_type".to_string(), self.voxel_type.to_json_value());
        m.insert("road_group_type".to_string(), self.road_group_type.to_json_value());
        m.insert("is_only_summon_data".to_string(), self.is_only_summon_data.to_json_value());
        m.insert("is_only_check_data".to_string(), self.is_only_check_data.to_json_value());
        m.insert("stage_category".to_string(), self.stage_category.to_json_value());
        m.insert("spawn_mode_type".to_string(), self.spawn_mode_type.to_json_value());
        m.insert("tag_list".to_string(), self.tag_list.to_json_value());
        m.insert("is_default_activated".to_string(), self.is_default_activated.to_json_value());
        m.insert("all_terrain_region".to_string(), self.all_terrain_region.to_json_value());
        m.insert("bitmap_position_info".to_string(), self.bitmap_position_info.to_json_value());
        m.insert("bitmap_color_list_for_spawn".to_string(), self.bitmap_color_list_for_spawn.to_json_value());
        m.insert("spawn_at_height_field_landscape".to_string(), self.spawn_at_height_field_landscape.to_json_value());
        m.insert("fish_summon_time_frequency_type".to_string(), self.fish_summon_time_frequency_type.to_json_value());
        m.insert("spawn_reason_list".to_string(), self.spawn_reason_list.to_json_value());
        m.insert("trailing_u32_113".to_string(), self.trailing_u32_113.to_json_value());
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        <CArray<u8> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "possible_list")?)?;
        <CArray<CString> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "auto_spawn_spline_name")?)?;
        <CArray<CString> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "auto_spawn_spline_except_name")?)?;
        <CArray<u16> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "region_info_list")?)?;
        <CArray<u16> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "not_spawn_region_info_list")?)?;
        <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "spawn_region_tag_list")?)?;
        <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "not_spawn_region_tag_list")?)?;
        // spawn_list: custom write — N-1 inter-element separator bytes (0xff) between elements
        {
            let v = json_get_field(obj, "spawn_list")?;
            let arr = v.as_array().ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::InvalidData, "spawn_list is not a JSON array"))?;
            w.extend_from_slice(&(arr.len() as u32).to_le_bytes());
            // Preserved separators (optional for backward-compat with JSON that
            // predates this field; default 0xff if absent or short).
            let seps = obj.get("spawn_list_separators").and_then(|v| v.as_array());
            for (i, item) in arr.iter().enumerate() {
                <AutoSpawnEntry as WriteJsonValue>::write_from_json(w, item)?;
                if i + 1 < arr.len() {
                    let sep = seps
                        .and_then(|s| s.get(i))
                        .and_then(|v| v.as_u64())
                        .map(|n| n as u8)
                        .unwrap_or(0xff);
                    w.push(sep);
                }
            }
        }
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "voxel_type")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "road_group_type")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_only_summon_data")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_only_check_data")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "stage_category")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "spawn_mode_type")?)?;
        <CArray<CBytes> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "tag_list")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_default_activated")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "all_terrain_region")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "bitmap_position_info")?)?;
        <CArray<u8> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "bitmap_color_list_for_spawn")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "spawn_at_height_field_landscape")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "fish_summon_time_frequency_type")?)?;
        <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "spawn_reason_list")?)?;
        // null-safe: old mods (pre-1.13.00) omit trailing_u32_113 → default 0.
        <u32 as WriteJsonValue>::write_from_json(w, obj.get("trailing_u32_113").unwrap_or(&Value::Null))?;
        Ok(())
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
