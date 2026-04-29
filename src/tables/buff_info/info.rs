//! Hand-corrected: IDA-derived parser for `BuffInfo.pabgb`.
//!
//! Per IDA sub_1410D6510: 13 fields matching mac binary __cstring order.
//! _buffDataList is a CArray<BuffDataEntry>. Each entry is:
//!   - u32 leading_lookup
//!   - u8 absent_flag (1=absent, 0=present)  ← INVERTED COptional
//!   - if !absent: typed BuffData (see binary::variants::buff_data)
//!
//! BuffData is a 120-variant polymorphic family, fully field-decoded
//! from Mac-binary symbols + Win-binary parser introspection.
//! All 48 variant tags observed in vanilla data validate cleanly.

use crate::binary::variants::buff_data::BuffData;
use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
use std::io::{self, Write};

#[derive(Debug)]
pub struct BuffDataEntry<'a> {
    pub leading_lookup: u32,
    pub absent_flag: u8,
    pub data: Option<BuffData<'a>>,
}

impl<'a> BinaryRead<'a> for BuffDataEntry<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let leading_lookup = u32::read_from(data, offset)?;
        let absent_flag = u8::read_from(data, offset)?;
        let payload = if absent_flag == 0 {
            Some(BuffData::read_from(data, offset)?)
        } else {
            None
        };
        Ok(Self { leading_lookup, absent_flag, data: payload })
    }
}

impl<'a> BinaryWrite for BuffDataEntry<'a> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.leading_lookup.write_to(w)?;
        self.absent_flag.write_to(w)?;
        if let Some(d) = &self.data {
            d.write_to(w)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct BuffInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub buff_data_list: CArray<BuffDataEntry<'a>>,
    pub min_level: u32,
    pub max_level: u32,
    pub sequencer_file_name: CString<'a>,
    pub buff_level_calculate_type: u8,
    pub ui_template_name: u32,
    pub ui_component_name: u32,
    pub elemental_status_info: u32,
    pub is_use_skill_info_pattern_description: u8,
    pub use_counting_by_global_timer: u8,
}

impl<'a> BuffInfo<'a> {
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        _entry_size: usize,
    ) -> io::Result<Self> {
        let key = u32::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;
        let buff_data_list = CArray::<BuffDataEntry>::read_from(data, offset)?;
        let min_level = u32::read_from(data, offset)?;
        let max_level = u32::read_from(data, offset)?;
        let sequencer_file_name = CString::read_from(data, offset)?;
        let buff_level_calculate_type = u8::read_from(data, offset)?;
        let ui_template_name = u32::read_from(data, offset)?;
        let ui_component_name = u32::read_from(data, offset)?;
        let elemental_status_info = u32::read_from(data, offset)?;
        let is_use_skill_info_pattern_description = u8::read_from(data, offset)?;
        let use_counting_by_global_timer = u8::read_from(data, offset)?;

        Ok(Self {
            key, string_key, is_blocked, buff_data_list,
            min_level, max_level, sequencer_file_name,
            buff_level_calculate_type, ui_template_name, ui_component_name,
            elemental_status_info,
            is_use_skill_info_pattern_description, use_counting_by_global_timer,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.buff_data_list.write_to(w)?;
        self.min_level.write_to(w)?;
        self.max_level.write_to(w)?;
        self.sequencer_file_name.write_to(w)?;
        self.buff_level_calculate_type.write_to(w)?;
        self.ui_template_name.write_to(w)?;
        self.ui_component_name.write_to(w)?;
        self.elemental_status_info.write_to(w)?;
        self.is_use_skill_info_pattern_description.write_to(w)?;
        self.use_counting_by_global_timer.write_to(w)?;
        Ok(())
    }

    /// Convert this BuffInfo record to a JSON dict with every typed prefix
    /// field by name. The typed-but-polymorphic `buff_data_list` (CArray of
    /// BuffData variants) round-trips as base64 under `_buff_data_list_b64`
    /// — clone-perfect across mods until BuffData gets its own JSON shim.
    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        let mut bd_bytes = Vec::new();
        let _ = self.buff_data_list.write_to(&mut bd_bytes);
        m.insert("_buff_data_list_b64".to_string(), Value::String(B64.encode(&bd_bytes)));
        m.insert("min_level".to_string(), self.min_level.to_json_value());
        m.insert("max_level".to_string(), self.max_level.to_json_value());
        m.insert("sequencer_file_name".to_string(), self.sequencer_file_name.to_json_value());
        m.insert("buff_level_calculate_type".to_string(), self.buff_level_calculate_type.to_json_value());
        m.insert("ui_template_name".to_string(), self.ui_template_name.to_json_value());
        m.insert("ui_component_name".to_string(), self.ui_component_name.to_json_value());
        m.insert("elemental_status_info".to_string(), self.elemental_status_info.to_json_value());
        m.insert("is_use_skill_info_pattern_description".to_string(), self.is_use_skill_info_pattern_description.to_json_value());
        m.insert("use_counting_by_global_timer".to_string(), self.use_counting_by_global_timer.to_json_value());
        m
    }

    /// Inverse of `to_json_dict`. Reads typed fields by name; the
    /// polymorphic buff_data_list is base64-decoded from
    /// `_buff_data_list_b64` and written verbatim.
    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        let bd_b64 = json_get_field(obj, "_buff_data_list_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "BuffInfo: _buff_data_list_b64 must be a base64 string"))?;
        let bd_bytes = B64.decode(bd_b64).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("BuffInfo: _buff_data_list_b64 invalid base64: {}", e)))?;
        w.extend_from_slice(&bd_bytes);
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "min_level")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "max_level")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "sequencer_file_name")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "buff_level_calculate_type")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "ui_template_name")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "ui_component_name")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "elemental_status_info")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_use_skill_info_pattern_description")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "use_counting_by_global_timer")?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB_PATH: &str =
        r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\buffinfo.pabgb";
    const PABGH_PATH: &str =
        r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\buffinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing pabgb fixture {}", PABGB_PATH);
            return;
        };
        let Some(entries) = load_pabgh_offsets(PABGH_PATH) else {
            eprintln!("SKIP: missing/unparseable pabgh fixture {}", PABGH_PATH);
            return;
        };
        let ranges = entry_ranges(&entries, data.len());

        let mut items = Vec::with_capacity(ranges.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = BuffInfo::read_with_size(&data, &mut cursor, end - start)
                .unwrap_or_else(|e| {
                    panic!(
                        "parse failed at entry {} (key=0x{:x}, offset 0x{:x}, size {}): {}",
                        i, key, start, end - start, e
                    )
                });
            assert_eq!(
                cursor, *end,
                "entry {} (key 0x{:x}) under/over-consumed: read {} bytes, expected {}",
                i, key, cursor - start, end - start
            );
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out.len(), data.len(), "buffinfo roundtrip size mismatch");
        assert_eq!(out, data, "buffinfo roundtrip bytes mismatch");
    }
}
