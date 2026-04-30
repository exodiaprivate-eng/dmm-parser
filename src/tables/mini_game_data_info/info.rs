//! Hand-corrected: IDA-derived parser for `MiniGameDataInfo.pabgb`.
//!
//! Per IDA sub_1410EC670: 16 fields. Player/NPC data lists are typed CArrays
//! of `MiniGameParticipantData` (per sub_14110E180 → sub_1410EC410). Spawn
//! data list (sub_14110E010 → sub_14110BCC0 → sub_1410F3220) has nested
//! polymorphic readers that cross into anti-disassembly-protected territory;
//! captured byte-perfect as `spawn_data_list_blob` (boundary recovered via
//! tail probe). Tail: u16 + u32 + CArray<u32>.
//!
//! ## sub_1410EC410 wire shape per participant element (player and NPC):
//!   1. u32 hash → u16 stored (sub_1410FF5C0, qword_DA00 lookup)
//!   2. u64 (8 bytes — flag/state)
//!   3. u8
//!   4. u8
//!   5. u32 (numeric value at +20)
//!   6. CArray<u32> (a2+24/+32 — list of u32 keys)

use crate::binary::variant::find_variant_boundary;
use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use crate::py_binary_struct;
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
use std::io::{self, Write};

py_binary_struct! {
    /// One element of `player_data_list` or `npc_data_list`. Per
    /// sub_1410EC410: 22 fixed bytes + 4×count from the trailing
    /// CArray<u32>. `key_hash` is a u32 wire hash key (qword_DA00 family);
    /// runtime resolves to u16 (sentinel 0xFFFF when not found).
    pub struct MiniGameParticipantData {
        pub key_hash: u32,
        pub flag_qword: u64,
        pub flag_a: u8,
        pub flag_b: u8,
        pub value_dword: u32,
        pub spawn_keys: CArray<u32>,
    }
}

/// Per-element of `entrance_fee_list`. Empirical structure across 5
/// vanilla samples: u32 count_or_flag + 3 × u64 values where high u32
/// of each u64 is always 0 (so payload fits comfortably in u32 range
/// at runtime). Promoted from [u8;28] for field-level JSON access.
#[derive(Debug)]
pub struct EntranceFee {
    pub count_or_flag: u32,
    pub value_a: u64,
    pub value_b: u64,
    pub value_c: u64,
}

impl<'a> BinaryRead<'a> for EntranceFee {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let count_or_flag = u32::read_from(data, offset)?;
        let value_a = u64::read_from(data, offset)?;
        let value_b = u64::read_from(data, offset)?;
        let value_c = u64::read_from(data, offset)?;
        Ok(Self { count_or_flag, value_a, value_b, value_c })
    }
}

impl BinaryWrite for EntranceFee {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.count_or_flag.write_to(w)?;
        self.value_a.write_to(w)?;
        self.value_b.write_to(w)?;
        self.value_c.write_to(w)?;
        Ok(())
    }
}

impl ToJsonValue for EntranceFee {
    fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("count_or_flag".into(), self.count_or_flag.to_json_value());
        m.insert("value_a".into(), self.value_a.to_json_value());
        m.insert("value_b".into(), self.value_b.to_json_value());
        m.insert("value_c".into(), self.value_c.to_json_value());
        Value::Object(m)
    }
}

impl WriteJsonValue for EntranceFee {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "EntranceFee: expected object"))?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "count_or_flag")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "value_a")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "value_b")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "value_c")?)?;
        Ok(())
    }
}

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
    /// _playerDataList — typed via sub_14110E180 → sub_1410EC410.
    pub player_data_list: CArray<MiniGameParticipantData>,
    /// _npcDataList — same element shape as player_data_list.
    pub npc_data_list: CArray<MiniGameParticipantData>,
    /// _spawnDataList — sub_14110E010 → sub_14110BCC0 → sub_1410F3220
    /// has multiple nested polymorphic readers (some anti-disassembly).
    /// Captured byte-perfect; boundary recovered via tail probe.
    pub spawn_data_list_blob: Vec<u8>,
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

        // Read entrance_fee_list as count + N×typed EntranceFee entries.
        let entrance_count = u32::read_from(data, offset)? as usize;
        let mut entrance_fee_list = Vec::with_capacity(entrance_count);
        for _ in 0..entrance_count {
            entrance_fee_list.push(EntranceFee::read_from(data, offset)?);
        }

        let default_reward_drop_set_info = u32::read_from(data, offset)?;

        // _playerDataList + _npcDataList — both are CArray<MiniGameParticipantData>.
        let player_data_list = CArray::<MiniGameParticipantData>::read_from(data, offset)?;
        let npc_data_list = CArray::<MiniGameParticipantData>::read_from(data, offset)?;

        // _spawnDataList — captured byte-perfect; boundary by tail probe.
        let post_npc = *offset;
        let spawn_size = find_variant_boundary(data, post_npc, entry_end, 4, |probe| {
            try_read_tail(data, probe, entry_end)
        })?;
        let spawn_data_list_blob = data[post_npc..post_npc + spawn_size].to_vec();
        *offset = post_npc + spawn_size;

        let game_event_handler_info = u16::read_from(data, offset)?;
        let knowledge_info = u32::read_from(data, offset)?;
        let game_advice_info_list = CArray::<u32>::read_from(data, offset)?;

        Ok(Self {
            key, string_key, is_blocked, script_name, phase_panel_tag_name,
            ui_view_id, use_deactive_result, need_change_character_scale,
            entrance_fee_list, default_reward_drop_set_info,
            player_data_list, npc_data_list, spawn_data_list_blob,
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
            fee.write_to(w)?;
        }
        self.default_reward_drop_set_info.write_to(w)?;
        self.player_data_list.write_to(w)?;
        self.npc_data_list.write_to(w)?;
        w.write_all(&self.spawn_data_list_blob)?;
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
            Value::Array(self.entrance_fee_list.iter().map(|f| f.to_json_value()).collect()));
        m.insert("default_reward_drop_set_info".to_string(), self.default_reward_drop_set_info.to_json_value());
        m.insert("player_data_list".to_string(), self.player_data_list.to_json_value());
        m.insert("npc_data_list".to_string(), self.npc_data_list.to_json_value());
        m.insert(
            "_spawn_data_list_blob_b64".to_string(),
            Value::String(B64.encode(&self.spawn_data_list_blob)),
        );
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
            EntranceFee::write_from_json(w, f)?;
        }
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "default_reward_drop_set_info")?)?;
        <CArray<MiniGameParticipantData> as WriteJsonValue>::write_from_json(
            w,
            json_get_field(obj, "player_data_list")?,
        )?;
        <CArray<MiniGameParticipantData> as WriteJsonValue>::write_from_json(
            w,
            json_get_field(obj, "npc_data_list")?,
        )?;
        let b64 = json_get_field(obj, "_spawn_data_list_blob_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "MiniGameDataInfo: _spawn_data_list_blob_b64 must be a base64 string"))?;
        let bytes = B64.decode(b64).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
            format!("MiniGameDataInfo: _spawn_data_list_blob_b64 invalid base64: {}", e)))?;
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

    #[test]
    fn json_roundtrip() {
        use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else {
            eprintln!("SKIP: missing pabgh fixture {}", PABGH_PATH);
            return;
        };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = MiniGameDataInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            MiniGameDataInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
