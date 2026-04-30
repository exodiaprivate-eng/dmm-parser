//! Hand-corrected: IDA-derived parser for `DropSetInfo.pabgb`.
//!
//! Per IDA sub_1410DB650: 12 fields. _list is a CArray<polymorphic> via
//! sub_141102760 (each element via sub_141D03AA0 — same deeply polymorphic
//! reader as ItemUseInfo's RandomBox variant). Captured as raw byte-blob
//! using a tail-fields probe.

use crate::binary::variant::find_variant_boundary;
use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
use std::io::{self, Write};

#[derive(Debug)]
pub struct DropSetInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub drop_roll_type: u8,
    pub drop_roll_count: u32,
    pub drop_condition_string: CString<'a>,
    pub drop_tag_name_hash: u32,
    /// Polymorphic CArray captured as raw bytes.
    pub list: Vec<u8>,
    pub nee_slot_count: u16,
    /// Wire u64 — fixed-point weight value (vanilla data shows values
    /// always within u32 range with high u32 zero, suggesting fixed-point
    /// scaling rather than f64).
    pub need_weight: u64,
    /// Wire u64 — fixed-point drop rate value. Vanilla samples show
    /// values like 1000000 (= 100% × 10000 PPM) supporting fixed-point
    /// interpretation. Treat as opaque u64 for round-trip safety; the
    /// game side knows the scaling factor.
    pub total_drop_rate: u64,
    pub original_string: CString<'a>,
}

impl<'a> DropSetInfo<'a> {
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
        let drop_roll_type = u8::read_from(data, offset)?;
        let drop_roll_count = u32::read_from(data, offset)?;
        let drop_condition_string = CString::read_from(data, offset)?;
        let drop_tag_name_hash = u32::read_from(data, offset)?;

        // Tail: u16 + [u8;8] + [u8;8] + CString = 18 + (4 + N)
        const TAIL_FIXED: usize = 22;
        let post_pre = *offset;
        let variant_size = find_variant_boundary(data, post_pre, entry_end, 4, |probe| {
            if probe + TAIL_FIXED > entry_end {
                return None;
            }
            let cs_off = probe + 18;
            let cs_len_bytes = data.get(cs_off..cs_off + 4)?.try_into().ok()?;
            let cs_len = u32::from_le_bytes(cs_len_bytes) as usize;
            let total = TAIL_FIXED + cs_len;
            if probe + total != entry_end {
                return None;
            }
            std::str::from_utf8(data.get(cs_off + 4..cs_off + 4 + cs_len)?).ok()?;
            Some(total)
        })?;

        let list = data[post_pre..post_pre + variant_size].to_vec();
        *offset = post_pre + variant_size;

        let nee_slot_count = u16::read_from(data, offset)?;
        let need_weight = u64::read_from(data, offset)?;
        let total_drop_rate = u64::read_from(data, offset)?;
        let original_string = CString::read_from(data, offset)?;

        Ok(Self {
            key, string_key, is_blocked, drop_roll_type, drop_roll_count,
            drop_condition_string, drop_tag_name_hash, list, nee_slot_count,
            need_weight, total_drop_rate, original_string,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.drop_roll_type.write_to(w)?;
        self.drop_roll_count.write_to(w)?;
        self.drop_condition_string.write_to(w)?;
        self.drop_tag_name_hash.write_to(w)?;
        w.write_all(&self.list)?;
        self.nee_slot_count.write_to(w)?;
        self.need_weight.write_to(w)?;
        self.total_drop_rate.write_to(w)?;
        self.original_string.write_to(w)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("drop_roll_type".to_string(), self.drop_roll_type.to_json_value());
        m.insert("drop_roll_count".to_string(), self.drop_roll_count.to_json_value());
        m.insert("drop_condition_string".to_string(), self.drop_condition_string.to_json_value());
        m.insert("drop_tag_name_hash".to_string(), self.drop_tag_name_hash.to_json_value());
        m.insert("_list_b64".to_string(), Value::String(B64.encode(&self.list)));
        m.insert("nee_slot_count".to_string(), self.nee_slot_count.to_json_value());
        m.insert("need_weight".to_string(), self.need_weight.to_json_value());
        m.insert("total_drop_rate".to_string(), self.total_drop_rate.to_json_value());
        m.insert("original_string".to_string(), self.original_string.to_json_value());
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "drop_roll_type")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "drop_roll_count")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "drop_condition_string")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "drop_tag_name_hash")?)?;
        let b64 = json_get_field(obj, "_list_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "DropSetInfo: _list_b64 must be a base64 string"))?;
        let bytes = B64.decode(b64).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
            format!("DropSetInfo: _list_b64 invalid base64: {}", e)))?;
        w.extend_from_slice(&bytes);
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "nee_slot_count")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "need_weight")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "total_drop_rate")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "original_string")?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB_PATH: &str =
        r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\dropsetinfo.pabgb";
    const PABGH_PATH: &str =
        r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\dropsetinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else { eprintln!("SKIP: {}", PABGB_PATH); return; };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else { eprintln!("SKIP: {}", PABGH_PATH); return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = DropSetInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x} off=0x{:x} size={}: {}", i, key, start, end-start, e));
            assert_eq!(cursor, *end);
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items { item.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "dropsetinfo roundtrip bytes mismatch");
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
            let item = DropSetInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            DropSetInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
