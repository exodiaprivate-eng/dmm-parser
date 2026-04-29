//! Hand-corrected: IDA-derived parser for `LevelGimmickSceneObjectInfo.pabgb`.
//!
//! Per IDA sub_1410EB480: 25 fields. _levelGimmickSceneObjectDataList is a
//! 160-byte stride polymorphic CArray (sub_14110ECD0 / sub_1410EB270).
//! Captured as raw byte-blob via tail-fields probe. Tail = 15 fixed fields +
//! CString guide_effect_socket_name + 4 trailing u32s.

use crate::binary::variant::find_variant_boundary;
use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
use std::io::{self, Write};

#[derive(Debug)]
pub struct LevelGimmickSceneObjectInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub level_name: CString<'a>,
    /// Polymorphic CArray captured as raw bytes.
    pub data_list: Vec<u8>,
    pub map_icon_texture_info: u32,
    pub discover_near_fog: u8,
    pub fog_map_icon_texture_info: u32,
    pub fog_distance: u32,
    pub over_abyss_icon_texture_info: u32,
    pub over_abyss_fog_map_icon_texture_info: u32,
    pub over_abyss_fog_distance: u32,
    pub discover_distance: u32,
    pub show_icon_condition_type: u8,
    pub use_teleport: u8,
    pub use_guide_effect: u8,
    pub is_sub_inner_gimmick: u8,
    pub check_game_level_load_state: u8,
    pub completed_discover_map_icon_texture_info: u32,
    pub over_abyss_completed_discover_map_icon_texture_info: u32,
    pub guide_effect_socket_name: CString<'a>,
    pub ore_vein_index: u32,
    pub discover_type: u32,
    pub ignore_same_gimmick_discover_distance: u32,
    pub discover_gimmick_state_hash: u32,
}

const TAIL_FIXED_BEFORE_CSTRING: usize = 42;
const TAIL_FIXED_AFTER_CSTRING: usize = 16;
const TAIL_FIXED: usize = TAIL_FIXED_BEFORE_CSTRING + TAIL_FIXED_AFTER_CSTRING + 4;

impl<'a> LevelGimmickSceneObjectInfo<'a> {
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
        let level_name = CString::read_from(data, offset)?;

        let post_pre = *offset;
        let variant_size = find_variant_boundary(data, post_pre, entry_end, 4, |probe| {
            if probe + TAIL_FIXED > entry_end {
                return None;
            }
            let cs_off = probe + TAIL_FIXED_BEFORE_CSTRING;
            let cs_len_bytes = data.get(cs_off..cs_off + 4)?.try_into().ok()?;
            let cs_len = u32::from_le_bytes(cs_len_bytes) as usize;
            let total = TAIL_FIXED + cs_len;
            if probe + total != entry_end {
                return None;
            }
            std::str::from_utf8(data.get(cs_off + 4..cs_off + 4 + cs_len)?).ok()?;
            Some(total)
        })?;

        let data_list = data[post_pre..post_pre + variant_size].to_vec();
        *offset = post_pre + variant_size;

        let map_icon_texture_info = u32::read_from(data, offset)?;
        let discover_near_fog = u8::read_from(data, offset)?;
        let fog_map_icon_texture_info = u32::read_from(data, offset)?;
        let fog_distance = u32::read_from(data, offset)?;
        let over_abyss_icon_texture_info = u32::read_from(data, offset)?;
        let over_abyss_fog_map_icon_texture_info = u32::read_from(data, offset)?;
        let over_abyss_fog_distance = u32::read_from(data, offset)?;
        let discover_distance = u32::read_from(data, offset)?;
        let show_icon_condition_type = u8::read_from(data, offset)?;
        let use_teleport = u8::read_from(data, offset)?;
        let use_guide_effect = u8::read_from(data, offset)?;
        let is_sub_inner_gimmick = u8::read_from(data, offset)?;
        let check_game_level_load_state = u8::read_from(data, offset)?;
        let completed_discover_map_icon_texture_info = u32::read_from(data, offset)?;
        let over_abyss_completed_discover_map_icon_texture_info = u32::read_from(data, offset)?;
        let guide_effect_socket_name = CString::read_from(data, offset)?;
        let ore_vein_index = u32::read_from(data, offset)?;
        let discover_type = u32::read_from(data, offset)?;
        let ignore_same_gimmick_discover_distance = u32::read_from(data, offset)?;
        let discover_gimmick_state_hash = u32::read_from(data, offset)?;

        Ok(Self {
            key, string_key, is_blocked, level_name, data_list,
            map_icon_texture_info, discover_near_fog, fog_map_icon_texture_info,
            fog_distance, over_abyss_icon_texture_info, over_abyss_fog_map_icon_texture_info,
            over_abyss_fog_distance, discover_distance,
            show_icon_condition_type, use_teleport, use_guide_effect,
            is_sub_inner_gimmick, check_game_level_load_state,
            completed_discover_map_icon_texture_info, over_abyss_completed_discover_map_icon_texture_info,
            guide_effect_socket_name, ore_vein_index, discover_type,
            ignore_same_gimmick_discover_distance, discover_gimmick_state_hash,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.level_name.write_to(w)?;
        w.write_all(&self.data_list)?;
        self.map_icon_texture_info.write_to(w)?;
        self.discover_near_fog.write_to(w)?;
        self.fog_map_icon_texture_info.write_to(w)?;
        self.fog_distance.write_to(w)?;
        self.over_abyss_icon_texture_info.write_to(w)?;
        self.over_abyss_fog_map_icon_texture_info.write_to(w)?;
        self.over_abyss_fog_distance.write_to(w)?;
        self.discover_distance.write_to(w)?;
        self.show_icon_condition_type.write_to(w)?;
        self.use_teleport.write_to(w)?;
        self.use_guide_effect.write_to(w)?;
        self.is_sub_inner_gimmick.write_to(w)?;
        self.check_game_level_load_state.write_to(w)?;
        self.completed_discover_map_icon_texture_info.write_to(w)?;
        self.over_abyss_completed_discover_map_icon_texture_info.write_to(w)?;
        self.guide_effect_socket_name.write_to(w)?;
        self.ore_vein_index.write_to(w)?;
        self.discover_type.write_to(w)?;
        self.ignore_same_gimmick_discover_distance.write_to(w)?;
        self.discover_gimmick_state_hash.write_to(w)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("level_name".to_string(), self.level_name.to_json_value());
        m.insert("_data_list_b64".to_string(), Value::String(B64.encode(&self.data_list)));
        m.insert("map_icon_texture_info".to_string(), self.map_icon_texture_info.to_json_value());
        m.insert("discover_near_fog".to_string(), self.discover_near_fog.to_json_value());
        m.insert("fog_map_icon_texture_info".to_string(), self.fog_map_icon_texture_info.to_json_value());
        m.insert("fog_distance".to_string(), self.fog_distance.to_json_value());
        m.insert("over_abyss_icon_texture_info".to_string(), self.over_abyss_icon_texture_info.to_json_value());
        m.insert("over_abyss_fog_map_icon_texture_info".to_string(), self.over_abyss_fog_map_icon_texture_info.to_json_value());
        m.insert("over_abyss_fog_distance".to_string(), self.over_abyss_fog_distance.to_json_value());
        m.insert("discover_distance".to_string(), self.discover_distance.to_json_value());
        m.insert("show_icon_condition_type".to_string(), self.show_icon_condition_type.to_json_value());
        m.insert("use_teleport".to_string(), self.use_teleport.to_json_value());
        m.insert("use_guide_effect".to_string(), self.use_guide_effect.to_json_value());
        m.insert("is_sub_inner_gimmick".to_string(), self.is_sub_inner_gimmick.to_json_value());
        m.insert("check_game_level_load_state".to_string(), self.check_game_level_load_state.to_json_value());
        m.insert("completed_discover_map_icon_texture_info".to_string(), self.completed_discover_map_icon_texture_info.to_json_value());
        m.insert("over_abyss_completed_discover_map_icon_texture_info".to_string(), self.over_abyss_completed_discover_map_icon_texture_info.to_json_value());
        m.insert("guide_effect_socket_name".to_string(), self.guide_effect_socket_name.to_json_value());
        m.insert("ore_vein_index".to_string(), self.ore_vein_index.to_json_value());
        m.insert("discover_type".to_string(), self.discover_type.to_json_value());
        m.insert("ignore_same_gimmick_discover_distance".to_string(), self.ignore_same_gimmick_discover_distance.to_json_value());
        m.insert("discover_gimmick_state_hash".to_string(), self.discover_gimmick_state_hash.to_json_value());
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "level_name")?)?;
        let b64 = json_get_field(obj, "_data_list_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "LevelGimmickSceneObjectInfo: _data_list_b64 must be a base64 string"))?;
        let bytes = B64.decode(b64).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
            format!("LevelGimmickSceneObjectInfo: _data_list_b64 invalid base64: {}", e)))?;
        w.extend_from_slice(&bytes);
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "map_icon_texture_info")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "discover_near_fog")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "fog_map_icon_texture_info")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "fog_distance")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "over_abyss_icon_texture_info")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "over_abyss_fog_map_icon_texture_info")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "over_abyss_fog_distance")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "discover_distance")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "show_icon_condition_type")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "use_teleport")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "use_guide_effect")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_sub_inner_gimmick")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "check_game_level_load_state")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "completed_discover_map_icon_texture_info")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "over_abyss_completed_discover_map_icon_texture_info")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "guide_effect_socket_name")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "ore_vein_index")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "discover_type")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "ignore_same_gimmick_discover_distance")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "discover_gimmick_state_hash")?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\levelgimmicksceneobjectinfo.pabgb";
    const PABGH_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\levelgimmicksceneobjectinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = LevelGimmickSceneObjectInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end-start, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "levelgimmicksceneobjectinfo roundtrip bytes mismatch");
    }
}
