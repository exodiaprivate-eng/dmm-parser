//! Hand-corrected: IDA-derived parser for `MiniGameDataInfo.pabgb`.
//!
//! Per IDA sub_1410EC670: 16 fields. Two polymorphic CArrays (player/npc data
//! lists via sub_14110E180) plus an unknown-polymorphic spawn data list
//! (sub_14110E010) captured as one combined byte-blob. Tail probed via
//! u16 + u32 + CArray<u32>.

use crate::binary::variant::find_variant_boundary;
use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
use std::io::{self, Write};

#[derive(Debug)]
pub struct EntranceFee(pub [u8; 28]);

#[derive(Debug)]
pub struct MiniGameDataInfo<'a> {
    pub key: u16,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub script_name: u32,
    pub phase_panel_tag_name: u32,
    pub ui_view_id: u32,
    pub use_deactive_result: u8,
    pub need_change_character_scale: u8,
    pub entrance_fee_list: Vec<EntranceFee>,
    pub default_reward_drop_set_info: u32,
    /// _playerDataList + _npcDataList + _spawnDataList combined byte-blob.
    pub data_lists_blob: Vec<u8>,
    pub game_event_handler_info: u16,
    pub knowledge_info: u32,
    pub game_advice_info_list: CArray<u32>,
}

fn try_read_tail(data: &[u8], probe: usize, end: usize) -> Option<usize> {
    let mut cursor = probe;
    if cursor + 2 + 4 + 4 > end { return None; }
    let _geh = u16::read_from(data, &mut cursor).ok()?;
    let _ki = u32::read_from(data, &mut cursor).ok()?;
    let _gail = CArray::<u32>::read_from(data, &mut cursor).ok()?;
    if cursor != end { return None; }
    Some(cursor - probe)
}

impl<'a> MiniGameDataInfo<'a> {
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
    ) -> io::Result<Self> {
        let entry_start = *offset;
        let entry_end = entry_start + entry_size;

        let key = u16::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;
        let script_name = u32::read_from(data, offset)?;
        let phase_panel_tag_name = u32::read_from(data, offset)?;
        let ui_view_id = u32::read_from(data, offset)?;
        let use_deactive_result = u8::read_from(data, offset)?;
        let need_change_character_scale = u8::read_from(data, offset)?;

        // Read entrance_fee_list as count + N×28-byte entries.
        let entrance_count = u32::read_from(data, offset)? as usize;
        let mut entrance_fee_list = Vec::with_capacity(entrance_count);
        for _ in 0..entrance_count {
            if *offset + 28 > data.len() {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "entrance fee truncated"));
            }
            let mut bytes = [0u8; 28];
            bytes.copy_from_slice(&data[*offset..*offset + 28]);
            *offset += 28;
            entrance_fee_list.push(EntranceFee(bytes));
        }

        let default_reward_drop_set_info = u32::read_from(data, offset)?;

        let post_pre = *offset;
        let variant_size = find_variant_boundary(data, post_pre, entry_end, 4, |probe| {
            try_read_tail(data, probe, entry_end)
        })?;
        let data_lists_blob = data[post_pre..post_pre + variant_size].to_vec();
        *offset = post_pre + variant_size;

        let game_event_handler_info = u16::read_from(data, offset)?;
        let knowledge_info = u32::read_from(data, offset)?;
        let game_advice_info_list = CArray::<u32>::read_from(data, offset)?;

        Ok(Self {
            key, string_key, is_blocked, script_name, phase_panel_tag_name,
            ui_view_id, use_deactive_result, need_change_character_scale,
            entrance_fee_list, default_reward_drop_set_info, data_lists_blob,
            game_event_handler_info, knowledge_info, game_advice_info_list,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.script_name.write_to(w)?;
        self.phase_panel_tag_name.write_to(w)?;
        self.ui_view_id.write_to(w)?;
        self.use_deactive_result.write_to(w)?;
        self.need_change_character_scale.write_to(w)?;
        (self.entrance_fee_list.len() as u32).write_to(w)?;
        for fee in &self.entrance_fee_list {
            w.write_all(&fee.0)?;
        }
        self.default_reward_drop_set_info.write_to(w)?;
        w.write_all(&self.data_lists_blob)?;
        self.game_event_handler_info.write_to(w)?;
        self.knowledge_info.write_to(w)?;
        self.game_advice_info_list.write_to(w)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("script_name".to_string(), self.script_name.to_json_value());
        m.insert("phase_panel_tag_name".to_string(), self.phase_panel_tag_name.to_json_value());
        m.insert("ui_view_id".to_string(), self.ui_view_id.to_json_value());
        m.insert("use_deactive_result".to_string(), self.use_deactive_result.to_json_value());
        m.insert("need_change_character_scale".to_string(), self.need_change_character_scale.to_json_value());
        m.insert("entrance_fee_list".to_string(),
            Value::Array(self.entrance_fee_list.iter().map(|f| f.0.to_json_value()).collect()));
        m.insert("default_reward_drop_set_info".to_string(), self.default_reward_drop_set_info.to_json_value());
        m.insert("_data_lists_blob_b64".to_string(), Value::String(B64.encode(&self.data_lists_blob)));
        m.insert("game_event_handler_info".to_string(), self.game_event_handler_info.to_json_value());
        m.insert("knowledge_info".to_string(), self.knowledge_info.to_json_value());
        m.insert("game_advice_info_list".to_string(), self.game_advice_info_list.to_json_value());
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "script_name")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "phase_panel_tag_name")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "ui_view_id")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "use_deactive_result")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "need_change_character_scale")?)?;
        let fees = json_get_field(obj, "entrance_fee_list")?
            .as_array()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "MiniGameDataInfo: entrance_fee_list must be a JSON array"))?;
        (fees.len() as u32).write_to(w)?;
        for f in fees {
            <[u8; 28] as WriteJsonValue>::write_from_json(w, f)?;
        }
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "default_reward_drop_set_info")?)?;
        let b64 = json_get_field(obj, "_data_lists_blob_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "MiniGameDataInfo: _data_lists_blob_b64 must be a base64 string"))?;
        let bytes = B64.decode(b64).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
            format!("MiniGameDataInfo: _data_lists_blob_b64 invalid base64: {}", e)))?;
        w.extend_from_slice(&bytes);
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "game_event_handler_info")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "knowledge_info")?)?;
        <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "game_advice_info_list")?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\minigamedatainfo.pabgb";
    const PABGH_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\minigamedatainfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = MiniGameDataInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end-start, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "minigamedatainfo roundtrip bytes mismatch");
    }
}
