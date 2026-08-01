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


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::BuffInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  13
// Decoded by dmm-parser:   13
// Missing in this struct:  0
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _stringKey
// ✅ _key (direct_u32, stream=4)
// ✅ _buffDataList (direct_u32, stream=4)
// ✅ _isBlocked (direct_15B, stream=15)
// ✅ _maxLevel (direct_u32, stream=4)
// ✅ _minLevel (direct_u32, stream=4)
// ✅ _buffLevelCalculateType (direct_15B, stream=15)
// ✅ _sequencerFileName
// ✅ _uiComponentName (reader_4B, stream=4)
// ✅ _uiTemplateName (reader_4B, stream=4)
// ✅ _isUseSkillInfoPatternDescription (direct_15B, stream=15)
// ✅ _elementalStatusInfo (reader_4B, stream=4)
// ✅ _useCountingByGlobalTimer (direct_15B, stream=15)

use crate::binary::variants::buff_data::BuffData;
use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
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

impl<'a> ToJsonValue for BuffDataEntry<'a> {
    fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("leading_lookup".into(), self.leading_lookup.to_json_value());
        m.insert("absent_flag".into(), self.absent_flag.to_json_value());
        m.insert(
            "data".into(),
            match &self.data {
                Some(d) => Value::Object(d.to_json_dict()),
                None => Value::Null,
            },
        );
        Value::Object(m)
    }
}

impl<'a> WriteJsonValue for BuffDataEntry<'a> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "BuffDataEntry: expected object",
        ))?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "leading_lookup")?)?;
        let absent_v = json_get_field(obj, "absent_flag")?;
        let absent = absent_v.as_u64().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "BuffDataEntry.absent_flag: expected u8",
        ))?;
        if absent > u8::MAX as u64 {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("BuffDataEntry.absent_flag: {} out of u8 range", absent)));
        }
        w.push(absent as u8);
        if absent == 0 {
            let data_v = json_get_field(obj, "data")?;
            let data_obj = data_v.as_object().ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "BuffDataEntry.data: expected object when absent_flag==0",
            ))?;
            BuffData::write_from_json_dict(w, data_obj)?;
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

    pub fn read_tracked_with_size(
        data: &'a [u8],
        offset: &mut usize,
        _entry_size: usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        let key = track_read_field::<u32>(data, offset, path, ranges, "key", "u32")?;
        let string_key = track_read_field::<CString<'a>>(data, offset, path, ranges, "string_key", "CString")?;
        let is_blocked = track_read_field::<u8>(data, offset, path, ranges, "is_blocked", "u8")?;
        let buff_data_list = track_read_field::<CArray<BuffDataEntry<'a>>>(data, offset, path, ranges, "buff_data_list", "CArray<BuffDataEntry>")?;
        let min_level = track_read_field::<u32>(data, offset, path, ranges, "min_level", "u32")?;
        let max_level = track_read_field::<u32>(data, offset, path, ranges, "max_level", "u32")?;
        let sequencer_file_name = track_read_field::<CString<'a>>(data, offset, path, ranges, "sequencer_file_name", "CString")?;
        let buff_level_calculate_type = track_read_field::<u8>(data, offset, path, ranges, "buff_level_calculate_type", "u8")?;
        let ui_template_name = track_read_field::<u32>(data, offset, path, ranges, "ui_template_name", "u32")?;
        let ui_component_name = track_read_field::<u32>(data, offset, path, ranges, "ui_component_name", "u32")?;
        let elemental_status_info = track_read_field::<u32>(data, offset, path, ranges, "elemental_status_info", "u32")?;
        let is_use_skill_info_pattern_description = track_read_field::<u8>(data, offset, path, ranges, "is_use_skill_info_pattern_description", "u8")?;
        let use_counting_by_global_timer = track_read_field::<u8>(data, offset, path, ranges, "use_counting_by_global_timer", "u8")?;
        Ok(Self {
            key, string_key, is_blocked, buff_data_list,
            min_level, max_level, sequencer_file_name,
            buff_level_calculate_type, ui_template_name, ui_component_name,
            elemental_status_info,
            is_use_skill_info_pattern_description, use_counting_by_global_timer,
        })
    }

    /// Convert this BuffInfo record to a JSON dict. `buff_data_list` is
    /// a fully typed CArray of BuffDataEntry; each BuffData drills into
    /// the typed BuffDataBase (28 fields) plus per-variant body via the
    /// 120-variant BuffDataVariant ToJsonValue/WriteJsonValue pipeline.
    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("buff_data_list".to_string(), self.buff_data_list.to_json_value());
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

    /// Inverse of `to_json_dict`. Reads typed fields by name including
    /// the recursive `buff_data_list` (CArray of BuffDataEntry).
    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        <CArray<BuffDataEntry> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "buff_data_list")?)?;
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

pub fn parse_buffinfo_to_json_with_pabgh(data: &[u8], pabgh: &[u8]) -> io::Result<Vec<Value>> {
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets_from_bytes};
    let entries = load_pabgh_offsets_from_bytes(pabgh).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "pabgh parse failed")
    })?;
    let ranges = entry_ranges(&entries, data.len());
    let mut items = Vec::with_capacity(ranges.len());
    for (_k, s, e) in ranges {
        let mut c = s;
        let item = BuffInfo::read_with_size(data, &mut c, e - s)?;
        if c != e {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("BuffInfo entry under/over-consumed: {}/{}", c - s, e - s),
            ));
        }
        items.push(Value::Object(item.to_json_dict()));
    }
    Ok(items)
}

pub fn serialize_buffinfo_from_json(items: &[Value]) -> io::Result<Vec<u8>> {
    let mut out = Vec::with_capacity(items.len() * 64);
    for (i, v) in items.iter().enumerate() {
        let obj = v.as_object().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, format!("buff[{}]: not an object", i))
        })?;
        BuffInfo::write_from_json_dict(&mut out, obj)
            .map_err(|e| io::Error::new(e.kind(), format!("buff[{}]: {}", i, e)))?;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("buffinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: fixture not found");
            return;
        };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else {
            eprintln!("SKIP: pabgh not found");
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

    /// JSON dict round-trip — typed write_to bytes must match
    /// write_from_json_dict bytes for every entry. Validates the typed
    /// BuffData base + variant_payload_b64 JSON shape preserves bytes.
    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else {
            eprintln!("SKIP: missing pabgb fixture");
            return;
        };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else {
            eprintln!("SKIP: missing pabgh fixture");
            return;
        };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = BuffInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            BuffInfo::write_from_json_dict(&mut from_json, &dict).unwrap_or_else(|e| {
                panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e)
            });
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, key,
            );
        }
    }
}

#[cfg(test)]
mod diag_1_16 {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    /// Per-BuffData-element trace for one record: prints each element's
    /// discriminant and byte span so a payload-size drift can be attributed to a
    /// specific variant instead of guessed at.
    #[test]
    #[ignore = "diagnostic; DIAG_BUFF_KEY=<hex>"]
    fn diag_element_spans() {
        let want = u32::from_str_radix(
            &std::env::var("DIAG_BUFF_KEY").unwrap_or_else(|_| "f4251".into()), 16).unwrap();
        let p = crate::testenv::resolve("buffinfo.pabgb");
        let Ok(data) = std::fs::read(&p) else { return };
        let Some(entries) = load_pabgh_offsets(&p.with_extension("pabgh").to_string_lossy())
        else { return };
        for (k, s, e) in entry_ranges(&entries, data.len()) {
            if k as u32 != want { continue; }
            let mut c = s;
            let _key = u32::read_from(&data, &mut c).unwrap();
            let _sk = CString::read_from(&data, &mut c).unwrap();
            let _ib = u8::read_from(&data, &mut c).unwrap();
            let n = u32::read_from(&data, &mut c).unwrap();
            eprintln!("k=0x{:x} span [{}..{}) len={}  buff_data_list count={}", k, s, e, e - s, n);
            for i in 0..n {
                let st = c;
                let _ll = u32::read_from(&data, &mut c).unwrap();
                let absent = u8::read_from(&data, &mut c).unwrap();
                let disc = if absent == 0 { data.get(c).copied() } else { None };
                match BuffData::read_from(&data, &mut c) {
                    Ok(_) => eprintln!("  [{:2}] absent={} disc={:?} rel {}..{} ({} B)",
                        i, absent, disc, st - s, c - s, c - st),
                    Err(err) => {
                        eprintln!("  [{:2}] absent={} disc={:?} rel {} ERR: {}", i, absent, disc, st - s, err);
                        return;
                    }
                }
            }
            eprintln!("  consumed to rel {} of {}", c - s, e - s);
        }
    }
}
