//! Hand-corrected: IDA-derived parser for `SpawningPoolAutoSpawnInfo.pabgb`.
//!
//! Per IDA sub_1410F9B80: 16 fields. _spawnList is polymorphic CArray via
//! sub_1411092E0 (same as TerrainRegionAutoSpawnInfo). Tail probe reads
//! CArray<u32> + CString + 4 u32 + 5 u8.

use crate::binary::variant::find_variant_boundary;
use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
use std::io::{self, Write};

#[derive(Debug)]
pub struct SpawningPoolAutoSpawnInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub spawn_list: Vec<u8>,
    pub mesh_name_list: CArray<u32>,
    pub spawning_pool_data: CString<'a>,
    pub type_: u8,
    pub level_action_point_info: u32,
    pub near_inner_radius: u32,
    pub near_outer_radius: u32,
    pub spawn_safety_distance: u32,
    pub use_random_rotation: u8,
    pub check_forbidden_area: u8,
    pub attach_to_socket: u8,
    pub is_exist_indoor_type: u8,
    pub collect_filter_dev: u8,
}

fn try_read_tail(data: &[u8], probe: usize, end: usize) -> Option<usize> {
    let mut cursor = probe;
    let _mesh = CArray::<u32>::read_from(data, &mut cursor).ok()?;
    if cursor > end { return None; }
    let _pool = CString::read_from(data, &mut cursor).ok()?;
    if cursor > end { return None; }
    if cursor + 1 + 4 + 4 + 4 + 4 + 5 > end { return None; }
    let _t = u8::read_from(data, &mut cursor).ok()?;
    let _lap = u32::read_from(data, &mut cursor).ok()?;
    let _nir = u32::read_from(data, &mut cursor).ok()?;
    let _nor = u32::read_from(data, &mut cursor).ok()?;
    let _sd = u32::read_from(data, &mut cursor).ok()?;
    let _urr = u8::read_from(data, &mut cursor).ok()?;
    let _cfa = u8::read_from(data, &mut cursor).ok()?;
    let _ats = u8::read_from(data, &mut cursor).ok()?;
    let _eit = u8::read_from(data, &mut cursor).ok()?;
    let _cfd = u8::read_from(data, &mut cursor).ok()?;
    if cursor != end { return None; }
    Some(cursor - probe)
}

impl<'a> SpawningPoolAutoSpawnInfo<'a> {
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

        let post_pre = *offset;
        let variant_size = find_variant_boundary(data, post_pre, entry_end, 4, |probe| {
            try_read_tail(data, probe, entry_end)
        })?;

        let spawn_list = data[post_pre..post_pre + variant_size].to_vec();
        *offset = post_pre + variant_size;

        let mesh_name_list = CArray::<u32>::read_from(data, offset)?;
        let spawning_pool_data = CString::read_from(data, offset)?;
        let type_ = u8::read_from(data, offset)?;
        let level_action_point_info = u32::read_from(data, offset)?;
        let near_inner_radius = u32::read_from(data, offset)?;
        let near_outer_radius = u32::read_from(data, offset)?;
        let spawn_safety_distance = u32::read_from(data, offset)?;
        let use_random_rotation = u8::read_from(data, offset)?;
        let check_forbidden_area = u8::read_from(data, offset)?;
        let attach_to_socket = u8::read_from(data, offset)?;
        let is_exist_indoor_type = u8::read_from(data, offset)?;
        let collect_filter_dev = u8::read_from(data, offset)?;

        Ok(Self {
            key, string_key, is_blocked, spawn_list, mesh_name_list,
            spawning_pool_data, type_, level_action_point_info,
            near_inner_radius, near_outer_radius, spawn_safety_distance,
            use_random_rotation, check_forbidden_area, attach_to_socket,
            is_exist_indoor_type, collect_filter_dev,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        w.write_all(&self.spawn_list)?;
        self.mesh_name_list.write_to(w)?;
        self.spawning_pool_data.write_to(w)?;
        self.type_.write_to(w)?;
        self.level_action_point_info.write_to(w)?;
        self.near_inner_radius.write_to(w)?;
        self.near_outer_radius.write_to(w)?;
        self.spawn_safety_distance.write_to(w)?;
        self.use_random_rotation.write_to(w)?;
        self.check_forbidden_area.write_to(w)?;
        self.attach_to_socket.write_to(w)?;
        self.is_exist_indoor_type.write_to(w)?;
        self.collect_filter_dev.write_to(w)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("_spawn_list_b64".to_string(), Value::String(B64.encode(&self.spawn_list)));
        m.insert("mesh_name_list".to_string(), self.mesh_name_list.to_json_value());
        m.insert("spawning_pool_data".to_string(), self.spawning_pool_data.to_json_value());
        m.insert("type_".to_string(), self.type_.to_json_value());
        m.insert("level_action_point_info".to_string(), self.level_action_point_info.to_json_value());
        m.insert("near_inner_radius".to_string(), self.near_inner_radius.to_json_value());
        m.insert("near_outer_radius".to_string(), self.near_outer_radius.to_json_value());
        m.insert("spawn_safety_distance".to_string(), self.spawn_safety_distance.to_json_value());
        m.insert("use_random_rotation".to_string(), self.use_random_rotation.to_json_value());
        m.insert("check_forbidden_area".to_string(), self.check_forbidden_area.to_json_value());
        m.insert("attach_to_socket".to_string(), self.attach_to_socket.to_json_value());
        m.insert("is_exist_indoor_type".to_string(), self.is_exist_indoor_type.to_json_value());
        m.insert("collect_filter_dev".to_string(), self.collect_filter_dev.to_json_value());
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        let b64 = json_get_field(obj, "_spawn_list_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "SpawningPoolAutoSpawnInfo: _spawn_list_b64 must be a base64 string"))?;
        let bytes = B64.decode(b64).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
            format!("SpawningPoolAutoSpawnInfo: _spawn_list_b64 invalid base64: {}", e)))?;
        w.extend_from_slice(&bytes);
        <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "mesh_name_list")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "spawning_pool_data")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "type_")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "level_action_point_info")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "near_inner_radius")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "near_outer_radius")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "spawn_safety_distance")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "use_random_rotation")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "check_forbidden_area")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "attach_to_socket")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_exist_indoor_type")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "collect_filter_dev")?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\spawningpoolautospawninfo.pabgb";
    const PABGH_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\spawningpoolautospawninfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = SpawningPoolAutoSpawnInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end-start, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "spawningpoolautospawninfo roundtrip bytes mismatch");
    }
}
