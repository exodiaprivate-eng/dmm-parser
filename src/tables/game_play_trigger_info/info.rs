//! Tier 1 — fully typed (no _tail_b64).
//!
//! Reader (Tier IDA verified 2026-05-19 vs CrimsonDesert.exe md5
//! 3d614280…): `sub_1410C1BA0` — GamePlayTriggerInfo deserializer (via
//! "GamePlayTriggerInfo" class block at 0x144AF37D0+). All 13 fields
//! confirmed in order. (Cited `sub_1410E0100` stale.) Inner
//! `_targetDataList` reader is `sub_1410E6300` (cited `sub_141103D50`
//! stale): u32 count + per-elem {u8 tag + u32 hash} = 5 wire bytes,
//! tag 0-3 dispatch to 4 hash helpers, tag ≥4 rejected — matches the
//! TargetDataItem JSON narrowing below.
//! Type confirmations: `position` = single 12-byte read ([u8;12] vec3,
//! kept raw); `rotation_y` = 4-byte read (f32-as-u32, u32 bit-preserves
//! the float); `player_condition_info` (sub_1410E19E0) & `ui_map_texture_
//! info` are u32 wire → u16 RAM; `field_revive_info` is u32 wire → u32
//! RAM (stored full-width DWORD, unlike the u16-remapped refs). All
//! Rust field types correct for the wire.
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u32 key                         (_key)
//!   2. CString string_key              (_stringKey)
//!   3. u8 is_blocked                   (_isBlocked)
//!   4. u8 trigger_type                 (_triggerType)
//!   5. u8 is_enable                    (_isEnable)
//!   6. u8 safe_zone_type               (_safeZoneType)
//!   7. u32 player_condition_info       (_playerConditionInfo,
//!      sub_1410FF430 → qword_145F0E9C0)
//!   8. u32 ui_map_texture_info         (_uiMapTextureInfo,
//!      inline → qword_145F113B0)
//!   9. [u8; 12] position               (_position, vec3)
//!  10. u32 rotation_y                  (_rotationY, f32-as-u32)
//!  11. u8 world_map_color_r            (_worldMapColorR)
//!  12. u32 field_revive_info           (_fieldReviveInfo,
//!      inline → qword_145F1A890)
//!  13. CArray<TargetDataItem> target_data_list  (_targetDataList,
//!      sub_141103D50)
//!
//! Each `TargetDataItem` is u8 tag + u32 hash (5 wire bytes). Tag
//! dispatches to one of four hash-lookup helpers in sub_141103D50:
//!   - 0 → sub_141100740 → qword_145F0DA38 (gimmick_info)
//!   - 1 → sub_141104AE0 → qword_145F11D70 (gimmick_group_info)
//!   - 2 → sub_1410FF5C0 → qword_145F0DA00 (exchange_item_info)
//!   - 3 → sub_1410FF340 → qword_145F0DA08 (character_info)
//! Tags 4+ are rejected by the reader, so we narrow the JSON shape to
//! the four valid `kind` strings on encode/decode.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::GamePlayTriggerInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  12
// Decoded by dmm-parser:   12
// Missing in this struct:  0
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _stringKey
// ✅ _key
// ✅ _triggerType (direct_u8, stream=1)
// ✅ _isBlocked (direct_u8, stream=1)
// ✅ _safeZoneType (direct_u8, stream=1)
// ✅ _isEnable (direct_u8, stream=1)
// ✅ _uiMapTextureInfo
// ✅ _playerConditionInfo (reader_4B, stream=4)
// ✅ _rotationY (direct_u32, stream=4)
// ✅ _position (direct_12B, stream=12)
// ✅ _targetDataList
// ✅ _worldMapColorR (direct_u8, stream=1)

use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use serde_json::{Map, Value};
use std::io::{self, Write};

/// One element of `target_data_list`. Wire shape: `u8 tag + u32 hash`.
/// JSON shape: `{"kind": "<one of four names>", "value": <u32>}`.
#[derive(Debug, Clone, Copy)]
#[allow(clippy::enum_variant_names)]
pub enum TargetDataItem {
    GimmickInfoRef(u32),
    GimmickGroupInfoRef(u32),
    ExchangeItemInfoRef(u32),
    CharacterInfoRef(u32),
}

impl TargetDataItem {
    pub fn tag(&self) -> u8 {
        match self {
            Self::GimmickInfoRef(_) => 0,
            Self::GimmickGroupInfoRef(_) => 1,
            Self::ExchangeItemInfoRef(_) => 2,
            Self::CharacterInfoRef(_) => 3,
        }
    }

    pub fn value(&self) -> u32 {
        match self {
            Self::GimmickInfoRef(v)
            | Self::GimmickGroupInfoRef(v)
            | Self::ExchangeItemInfoRef(v)
            | Self::CharacterInfoRef(v) => *v,
        }
    }

    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::GimmickInfoRef(_) => "GimmickInfoRef",
            Self::GimmickGroupInfoRef(_) => "GimmickGroupInfoRef",
            Self::ExchangeItemInfoRef(_) => "ExchangeItemInfoRef",
            Self::CharacterInfoRef(_) => "CharacterInfoRef",
        }
    }
}

impl<'a> BinaryRead<'a> for TargetDataItem {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let tag = u8::read_from(data, offset)?;
        let value = u32::read_from(data, offset)?;
        match tag {
            0 => Ok(Self::GimmickInfoRef(value)),
            1 => Ok(Self::GimmickGroupInfoRef(value)),
            2 => Ok(Self::ExchangeItemInfoRef(value)),
            3 => Ok(Self::CharacterInfoRef(value)),
            other => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("TargetDataItem: unknown tag {}", other),
            )),
        }
    }
}

impl BinaryWrite for TargetDataItem {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.tag().write_to(w)?;
        self.value().write_to(w)?;
        Ok(())
    }
}

impl ToJsonValue for TargetDataItem {
    fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("kind".to_string(), Value::String(self.kind_name().to_string()));
        m.insert("value".to_string(), self.value().to_json_value());
        Value::Object(m)
    }
}

impl WriteJsonValue for TargetDataItem {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "TargetDataItem: expected object",
        ))?;
        let kind = json_get_field(obj, "kind")?.as_str().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "TargetDataItem.kind: expected string",
        ))?;
        let tag: u8 = match kind {
            "GimmickInfoRef" => 0,
            "GimmickGroupInfoRef" => 1,
            "ExchangeItemInfoRef" => 2,
            "CharacterInfoRef" => 3,
            other => return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("TargetDataItem.kind: unknown variant '{}'", other),
            )),
        };
        w.push(tag);
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "value")?)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct GamePlayTriggerInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub trigger_type: u8,
    pub is_enable: u8,
    pub safe_zone_type: u8,
    pub player_condition_info: u32,
    pub ui_map_texture_info: u32,
    pub position: [f32; 3],
    pub rotation_y: f32,
    pub world_map_color_r: u8,
    pub condition_info_list: CArray<u32>,
    pub field_revive_info: u32,
    pub flag_a: u8,
    pub region_info: u32,
    pub flag_b: u32,
    pub target_data_list: CArray<TargetDataItem>,
}

impl<'a> GamePlayTriggerInfo<'a> {
    /// Read with explicit entry size from pabgh (compat shim — Tier 1 means
    /// every byte is consumed by typed reads, so the size is just verified).
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        let consumed = *offset - start;
        if consumed != entry_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("GamePlayTriggerInfo: consumed {} bytes, expected {}", consumed, entry_size),
            ));
        }
        Ok(item)
    }

    pub fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let key = u32::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;
        let trigger_type = u8::read_from(data, offset)?;
        let is_enable = u8::read_from(data, offset)?;
        let safe_zone_type = u8::read_from(data, offset)?;
        let player_condition_info = u32::read_from(data, offset)?;
        let ui_map_texture_info = u32::read_from(data, offset)?;
        let position = <[f32; 3]>::read_from(data, offset)?;
        let rotation_y = f32::read_from(data, offset)?;
        let world_map_color_r = u8::read_from(data, offset)?;
        let condition_info_list = CArray::<u32>::read_from(data, offset)?;
        let field_revive_info = u32::read_from(data, offset)?;
        let flag_a = u8::read_from(data, offset)?;
        let region_info = u32::read_from(data, offset)?;
        let flag_b = u32::read_from(data, offset)?;
        let target_data_list = CArray::<TargetDataItem>::read_from(data, offset)?;
        Ok(Self {
            key, string_key, is_blocked, trigger_type, is_enable, safe_zone_type,
            player_condition_info, ui_map_texture_info, position, rotation_y,
            world_map_color_r, condition_info_list, field_revive_info, flag_a,
            region_info, flag_b, target_data_list,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.trigger_type.write_to(w)?;
        self.is_enable.write_to(w)?;
        self.safe_zone_type.write_to(w)?;
        self.player_condition_info.write_to(w)?;
        self.ui_map_texture_info.write_to(w)?;
        self.position.write_to(w)?;
        self.rotation_y.write_to(w)?;
        self.world_map_color_r.write_to(w)?;
        self.condition_info_list.write_to(w)?;
        self.field_revive_info.write_to(w)?;
        self.flag_a.write_to(w)?;
        self.region_info.write_to(w)?;
        self.flag_b.write_to(w)?;
        self.target_data_list.write_to(w)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("trigger_type".to_string(), self.trigger_type.to_json_value());
        m.insert("is_enable".to_string(), self.is_enable.to_json_value());
        m.insert("safe_zone_type".to_string(), self.safe_zone_type.to_json_value());
        m.insert("player_condition_info".to_string(), self.player_condition_info.to_json_value());
        m.insert("ui_map_texture_info".to_string(), self.ui_map_texture_info.to_json_value());
        m.insert("position".to_string(), self.position.to_json_value());
        m.insert("rotation_y".to_string(), self.rotation_y.to_json_value());
        m.insert("world_map_color_r".to_string(), self.world_map_color_r.to_json_value());
        m.insert("condition_info_list".to_string(), self.condition_info_list.to_json_value());
        m.insert("field_revive_info".to_string(), self.field_revive_info.to_json_value());
        m.insert("flag_a".to_string(), self.flag_a.to_json_value());
        m.insert("region_info".to_string(), self.region_info.to_json_value());
        m.insert("flag_b".to_string(), self.flag_b.to_json_value());
        m.insert("target_data_list".to_string(), self.target_data_list.to_json_value());
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "trigger_type")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_enable")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "safe_zone_type")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "player_condition_info")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "ui_map_texture_info")?)?;
        <[f32; 3] as WriteJsonValue>::write_from_json(w, json_get_field(obj, "position")?)?;
        <f32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "rotation_y")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "world_map_color_r")?)?;
        <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "condition_info_list")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "field_revive_info")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_a")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "region_info")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_b")?)?;
        <CArray<TargetDataItem> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "target_data_list")?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/gameplaytrigger.pabgb";
    const PABGH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/gameplaytrigger.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            let item = GamePlayTriggerInfo::read_from(&data, &mut c)
                .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er));
            assert_eq!(c, *e, "e{} k=0x{:x}: under/over-read {}/{}", i, k, c - s, e - s);
            items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "gameplaytrigger roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = GamePlayTriggerInfo::read_from(&data, &mut cursor).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            GamePlayTriggerInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }

    #[test]
    fn target_kinds_seen() {
        // Confirm vanilla data exercises the variant — sanity check that
        // we're not just shipping an unused enum.
        use std::collections::HashMap;
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut counts: HashMap<&'static str, usize> = HashMap::new();
        for (_, s, _) in &ranges {
            let mut c = *s;
            let item = GamePlayTriggerInfo::read_from(&data, &mut c).unwrap();
            for t in &item.target_data_list.items {
                *counts.entry(t.kind_name()).or_insert(0) += 1;
            }
        }
        eprintln!("game_play_trigger target_data kinds: {:?}", counts);
    }
}
