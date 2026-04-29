//! Hand-corrected: IDA-derived parser for `EffectInfo.pabgb`.
//!
//! Per IDA sub_1410DBFC0: 8 fields. Two polymorphic CArrays
//! (_effectDataList via sub_141116A70 with 376-byte stride,
//! _meshEffectDataList via custom CArray with 48-byte stride) followed by
//! 3 trailing bytes. Captured both data lists as one combined byte-blob;
//! tail bytes parsed individually.

use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
use std::io::{self, Write};

#[derive(Debug)]
pub struct EffectInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    /// _effectDataList + _meshEffectDataList captured as one blob (both are
    /// CArrays of polymorphic data with internal nested variants).
    pub data_lists_blob: Vec<u8>,
    pub has_equip_type: u8,
    pub has_preset: u8,
    pub target_color_lerp_type: u8,
}

const TAIL_SIZE: usize = 3;

impl<'a> EffectInfo<'a> {
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

        if *offset + TAIL_SIZE > entry_end {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                "effectinfo: entry too small for tail"));
        }
        let blob_end = entry_end - TAIL_SIZE;
        let data_lists_blob = data[*offset..blob_end].to_vec();
        *offset = blob_end;

        let has_equip_type = u8::read_from(data, offset)?;
        let has_preset = u8::read_from(data, offset)?;
        let target_color_lerp_type = u8::read_from(data, offset)?;

        Ok(Self {
            key, string_key, is_blocked, data_lists_blob,
            has_equip_type, has_preset, target_color_lerp_type,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        w.write_all(&self.data_lists_blob)?;
        self.has_equip_type.write_to(w)?;
        self.has_preset.write_to(w)?;
        self.target_color_lerp_type.write_to(w)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("_data_lists_blob_b64".to_string(), Value::String(B64.encode(&self.data_lists_blob)));
        m.insert("has_equip_type".to_string(), self.has_equip_type.to_json_value());
        m.insert("has_preset".to_string(), self.has_preset.to_json_value());
        m.insert("target_color_lerp_type".to_string(), self.target_color_lerp_type.to_json_value());
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        let b64 = json_get_field(obj, "_data_lists_blob_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "EffectInfo: _data_lists_blob_b64 must be a base64 string"))?;
        let bytes = B64.decode(b64).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
            format!("EffectInfo: _data_lists_blob_b64 invalid base64: {}", e)))?;
        w.extend_from_slice(&bytes);
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "has_equip_type")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "has_preset")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "target_color_lerp_type")?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\effectinfo.pabgb";
    const PABGH_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\effectinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = EffectInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end-start, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "effectinfo roundtrip bytes mismatch");
    }
}
