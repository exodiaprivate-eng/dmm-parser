//! Hand-corrected: IDA-derived parser for `FactionNodeSpawnInfo.pabgb`.
//!
//! Per IDA sub_1410DED80: 6 fields. _patrolAISplineDataList is a CArray
//! with COptional<sub_1413F8A20> inner elements (variable per-element size).
//! Captured as raw byte-blob; no tail.

use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
use std::io::{self, Write};

#[derive(Debug)]
pub struct FactionNodeSpawnInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub faction_node_info: u32,
    /// 3D bounding box (Vec3 min, Vec3 max) — 24 wire bytes total.
    pub boundary_box_min: [f32; 3],
    pub boundary_box_max: [f32; 3],
    pub patrol_ai_spline_data_list: Vec<u8>,
}

impl<'a> FactionNodeSpawnInfo<'a> {
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
        let faction_node_info = u32::read_from(data, offset)?;
        let boundary_box_min = <[f32; 3]>::read_from(data, offset)?;
        let boundary_box_max = <[f32; 3]>::read_from(data, offset)?;

        let patrol_ai_spline_data_list = data[*offset..entry_end].to_vec();
        *offset = entry_end;

        Ok(Self {
            key, string_key, is_blocked, faction_node_info,
            boundary_box_min, boundary_box_max, patrol_ai_spline_data_list,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.faction_node_info.write_to(w)?;
        self.boundary_box_min.write_to(w)?;
        self.boundary_box_max.write_to(w)?;
        w.write_all(&self.patrol_ai_spline_data_list)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("faction_node_info".to_string(), self.faction_node_info.to_json_value());
        m.insert("boundary_box_min".to_string(), self.boundary_box_min.to_json_value());
        m.insert("boundary_box_max".to_string(), self.boundary_box_max.to_json_value());
        m.insert("_patrol_ai_spline_data_list_b64".to_string(),
            Value::String(B64.encode(&self.patrol_ai_spline_data_list)));
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "faction_node_info")?)?;
        <[f32; 3] as WriteJsonValue>::write_from_json(w, json_get_field(obj, "boundary_box_min")?)?;
        <[f32; 3] as WriteJsonValue>::write_from_json(w, json_get_field(obj, "boundary_box_max")?)?;
        let b64 = json_get_field(obj, "_patrol_ai_spline_data_list_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "FactionNodeSpawnInfo: _patrol_ai_spline_data_list_b64 must be a base64 string"))?;
        let bytes = B64.decode(b64).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
            format!("FactionNodeSpawnInfo: _patrol_ai_spline_data_list_b64 invalid base64: {}", e)))?;
        w.extend_from_slice(&bytes);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\factionnodespawninfo.pabgb";
    const PABGH_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\factionnodespawninfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = FactionNodeSpawnInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end-start, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "factionnodespawninfo roundtrip bytes mismatch");
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
            let item = FactionNodeSpawnInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            FactionNodeSpawnInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
