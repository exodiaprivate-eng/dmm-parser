//! Tier 1.5 — typed prefix (now including the full SequencerStageChartDesc)
//! plus tail blob for the remaining ~70 trailing fields.
//!
//! Reader: `sub_1410FA990` in CrimsonDesert.exe (Win build) — confirmed via
//! Win-IDA decompile this session. 25 MB pabgb / largest table in the set.
//!
//! Wire reads, in order:
//!   1. u32 key
//!   2. CString string_key
//!   3. u8 is_blocked
//!   4. LocalizableString name
//!   5. LocalizableString stage_desc
//!   6. LocalizableString complete_log
//!   7. SequencerStageChartDescPartial sequencer_desc (sub_141D8C6D0,
//!      INLINE single-instance — distinct from global_stage_sequencer_info
//!      which has a CArray of these)
//!      ← TAIL STARTS HERE
//!   8+. ~70 trailing fields including u32 lookups, CArrays, u8 flags,
//!       LocalizableString, CString-hash, etc. All fields are decodable
//!       from sub_1410FA990 — just mechanical work.
//!
//! Promotion note: the previous Tier 1.5 cut stopped at field 6 because
//! field 7 was an opaque polymorphic SequencerStageChartDesc. Now that
//! the desc has a complete decoder, it joins the typed prefix.

use crate::binary::*;
use crate::binary::sequencer_stage_chart_desc::SequencerStageChartDescPartial;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
use std::io::{self, Write};

#[derive(Debug)]
pub struct StageInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub name: LocalizableString<'a>,
    pub stage_desc: LocalizableString<'a>,
    pub complete_log: LocalizableString<'a>,
    pub sequencer_desc: SequencerStageChartDescPartial<'a>,
    pub tail_blob: Vec<u8>,
}

impl<'a> StageInfo<'a> {
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
        let name = LocalizableString::read_from(data, offset)?;
        let stage_desc = LocalizableString::read_from(data, offset)?;
        let complete_log = LocalizableString::read_from(data, offset)?;
        let sequencer_desc = SequencerStageChartDescPartial::read_from(data, offset)?;

        if *offset > entry_end {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "StageInfo: typed prefix overran entry ({} > {})",
                    *offset, entry_end
                ),
            ));
        }
        let tail_blob = data[*offset..entry_end].to_vec();
        *offset = entry_end;

        Ok(Self {
            key, string_key, is_blocked, name, stage_desc, complete_log,
            sequencer_desc, tail_blob,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.name.write_to(w)?;
        self.stage_desc.write_to(w)?;
        self.complete_log.write_to(w)?;
        self.sequencer_desc.write_to(w)?;
        w.write_all(&self.tail_blob)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("name".to_string(), self.name.to_json_value());
        m.insert("stage_desc".to_string(), self.stage_desc.to_json_value());
        m.insert("complete_log".to_string(), self.complete_log.to_json_value());
        m.insert("sequencer_desc".to_string(), self.sequencer_desc.to_json_value());
        m.insert("_tail_blob_b64".to_string(), Value::String(B64.encode(&self.tail_blob)));
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "name")?)?;
        <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "stage_desc")?)?;
        <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "complete_log")?)?;
        <SequencerStageChartDescPartial as WriteJsonValue>::write_from_json(w, json_get_field(obj, "sequencer_desc")?)?;
        let b64 = json_get_field(obj, "_tail_blob_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "StageInfo: _tail_blob_b64 must be a base64 string"))?;
        let bytes = B64.decode(b64).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
            format!("StageInfo: _tail_blob_b64 invalid base64: {}", e)))?;
        w.extend_from_slice(&bytes);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\stageinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\stageinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                StageInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "stageinfo roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = StageInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            StageInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
