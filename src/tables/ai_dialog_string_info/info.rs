//! Hand-corrected: IDA-derived parser for `AIDialogStringInfo.pabgb`.
//!
//! Per IDA sub_1410D5210: 11 fields. The latter half contains a hashmap
//! (sub_141119210) and a COptional<polymorphic> (sub_141119080). Captured
//! the trailing fields as one byte-blob.

use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
use std::io::{self, Write};

#[derive(Debug)]
pub struct AIDialogStringInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub flag_a: u8,
    pub flag_b: u8,
    pub list_a: CArray<u16>,
    pub lookup_a: u16,  // 2 bytes on disk via sub_1410FF220
    pub lookup_b: u32,  // 4 bytes on disk via sub_1410FF2D0
    /// Hashmap + trailing fields captured as one byte-blob (variable-size
    /// internal structures defy linear-probe boundary detection).
    pub trailing_blob: Vec<u8>,
}

impl<'a> AIDialogStringInfo<'a> {
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
        let flag_a = u8::read_from(data, offset)?;
        let flag_b = u8::read_from(data, offset)?;
        let list_a = CArray::<u16>::read_from(data, offset)?;
        let lookup_a = u16::read_from(data, offset)?;
        let lookup_b = u32::read_from(data, offset)?;

        let trailing_blob = data[*offset..entry_end].to_vec();
        *offset = entry_end;

        Ok(Self {
            key, string_key, is_blocked, flag_a, flag_b,
            list_a, lookup_a, lookup_b, trailing_blob,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.flag_a.write_to(w)?;
        self.flag_b.write_to(w)?;
        self.list_a.write_to(w)?;
        self.lookup_a.write_to(w)?;
        self.lookup_b.write_to(w)?;
        w.write_all(&self.trailing_blob)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("flag_a".to_string(), self.flag_a.to_json_value());
        m.insert("flag_b".to_string(), self.flag_b.to_json_value());
        m.insert("list_a".to_string(), self.list_a.to_json_value());
        m.insert("lookup_a".to_string(), self.lookup_a.to_json_value());
        m.insert("lookup_b".to_string(), self.lookup_b.to_json_value());
        m.insert("_trailing_blob_b64".to_string(), Value::String(B64.encode(&self.trailing_blob)));
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_a")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_b")?)?;
        <CArray<u16> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "list_a")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_a")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_b")?)?;
        let b64 = json_get_field(obj, "_trailing_blob_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "AIDialogStringInfo: _trailing_blob_b64 must be a base64 string"))?;
        let bytes = B64.decode(b64).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
            format!("AIDialogStringInfo: _trailing_blob_b64 invalid base64: {}", e)))?;
        w.extend_from_slice(&bytes);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\aidialogstringinfo.pabgb";
    const PABGH_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\aidialogstringinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = AIDialogStringInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end-start, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "aidialogstringinfo roundtrip bytes mismatch");
    }
}
