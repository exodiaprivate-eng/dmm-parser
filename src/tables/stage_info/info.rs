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
//!   8. u32 spawn_faction_spawn_data_info (mem +352, qword_145F0EF08)
//!   9. u32 spawn_faction_node_info       (sub_141101D50, qword_145F0EEE8)
//!  10. CArray<u32> disable_faction_spawn_party_name_hash_list
//!                                        (sub_141101AB0, mem +360)
//!  11. u64 raw_a                         (mem +376)
//!  12. u64 raw_b                         (mem +384)
//!  13. u64 raw_c                         (mem +392)
//!  14. CArray<u32> list_a                (sub_1410FF890, qword_145F0DA08
//!                                         hash, mem +400)
//!  15. u8 flag_a                         (mem +416)
//!  16. u8 flag_b                         (mem +417)
//!  17. u32 lookup_c                      (sub_141102CB0, qword_145F0EF20)
//!  18. u32 lookup_d                      (sub_141102D20, qword_145F0EF38)
//!  19. u32 lookup_e                      (sub_141102D90)
//!  20. CArray<u32> close_filter_a        (sub_141101610, qword_145F0EF38)
//!  21. CArray<u32> close_filter_b        (sub_1411049D0, qword_145F0EF00)
//!  22. CArray<u32> close_filter_c        (sub_141101610, qword_145F0EF38)
//!  23. CArray<StageFilterEntry> filter_entry_list
//!                                        (sub_1411068C0 → sub_1410F3380)
//!  24. u32 lookup_f                      (sub_1410FF430, qword_145F0E9C0)
//!  25. u32 lookup_g                      (sub_1410FF430)
//!  26. u32 lookup_h                      (qword_145F11398 hash)
//!  27. CArray<u32> list_b                (sub_1410FF890, qword_145F0DA08)
//!  28. CArray<u32> list_c                (sub_1410FF890, qword_145F0DA08)
//!  29. u32 lookup_i                      (sub_1410FF340)
//!  30. u32 raw_d
//!  31. CString cstring_a                 (sub_1410A9D40 — wire CString)
//!  32. u8 flag_c
//!  33. u8 flag_d
//!  34. u32 raw_e
//!  35. u32 raw_f
//!  36. u32 pair_a, u32 pair_b            (sub_1410AA070 — 2 raw u32s)
//!  37. u64 raw_g
//!  38. u32 raw_h
//!  39. u16 raw_i
//!      ← TAIL STARTS HERE
//!  40+. ~40 trailing fields. Several unknown helpers (sub_141108F70,
//!       sub_141108DE0, sub_141108C30, sub_141107B30/C70, sub_141103530)
//!       gate further extension — needs IDA per helper.
//!
//! Promotion note: the previous Tier 1.5 cut stopped at field 6 because
//! field 7 was an opaque polymorphic SequencerStageChartDesc. Now that
//! the desc has a complete decoder, it joins the typed prefix.

use crate::binary::*;
use crate::binary::sequencer_stage_chart_desc::SequencerStageChartDescPartial;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use crate::py_binary_struct;
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
use std::io::{self, Write};

py_binary_struct! {
    /// `sub_1410F3380` per-element of stage_info field 23's outer
    /// CArray (sub_1411068C0). 19 wire bytes / 20 mem bytes.
    pub struct StageFilterEntry {
        pub lookup_a: u32,    // sub_141102D20 → qword_145F0EF38
        pub lookup_b: u32,    // sub_1410FF430 → qword_145F0E9C0
        pub raw_a: u32,
        pub raw_b: u32,
        pub flag_a: u8,
        pub flag_b: u8,
        pub flag_c: u8,
    }
}

#[derive(Debug)]
pub struct StageInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub name: LocalizableString<'a>,
    pub stage_desc: LocalizableString<'a>,
    pub complete_log: LocalizableString<'a>,
    pub sequencer_desc: SequencerStageChartDescPartial<'a>,
    pub spawn_faction_spawn_data_info: u32,
    pub spawn_faction_node_info: u32,
    pub disable_faction_spawn_party_name_hash_list: CArray<u32>,
    pub raw_a: u64,
    pub raw_b: u64,
    pub raw_c: u64,
    pub list_a: CArray<u32>,
    pub flag_a: u8,
    pub flag_b: u8,
    pub lookup_c: u32,
    pub lookup_d: u32,
    pub lookup_e: u32,
    pub close_filter_a: CArray<u32>,
    pub close_filter_b: CArray<u32>,
    pub close_filter_c: CArray<u32>,
    pub filter_entry_list: CArray<StageFilterEntry>,
    pub lookup_f: u32,
    pub lookup_g: u32,
    pub lookup_h: u32,
    pub list_b: CArray<u32>,
    pub list_c: CArray<u32>,
    pub lookup_i: u32,
    pub raw_d: u32,
    pub cstring_a: CString<'a>,
    pub flag_c: u8,
    pub flag_d: u8,
    pub raw_e: u32,
    pub raw_f: u32,
    pub pair_a: u32,
    pub pair_b: u32,
    pub raw_g: u64,
    pub raw_h: u32,
    pub raw_i: u16,
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
        let spawn_faction_spawn_data_info = u32::read_from(data, offset)?;
        let spawn_faction_node_info = u32::read_from(data, offset)?;
        let disable_faction_spawn_party_name_hash_list = CArray::<u32>::read_from(data, offset)?;
        let raw_a = u64::read_from(data, offset)?;
        let raw_b = u64::read_from(data, offset)?;
        let raw_c = u64::read_from(data, offset)?;
        let list_a = CArray::<u32>::read_from(data, offset)?;
        let flag_a = u8::read_from(data, offset)?;
        let flag_b = u8::read_from(data, offset)?;
        let lookup_c = u32::read_from(data, offset)?;
        let lookup_d = u32::read_from(data, offset)?;
        let lookup_e = u32::read_from(data, offset)?;
        let close_filter_a = CArray::<u32>::read_from(data, offset)?;
        let close_filter_b = CArray::<u32>::read_from(data, offset)?;
        let close_filter_c = CArray::<u32>::read_from(data, offset)?;
        let filter_entry_list = CArray::<StageFilterEntry>::read_from(data, offset)?;
        let lookup_f = u32::read_from(data, offset)?;
        let lookup_g = u32::read_from(data, offset)?;
        let lookup_h = u32::read_from(data, offset)?;
        let list_b = CArray::<u32>::read_from(data, offset)?;
        let list_c = CArray::<u32>::read_from(data, offset)?;
        let lookup_i = u32::read_from(data, offset)?;
        let raw_d = u32::read_from(data, offset)?;
        let cstring_a = CString::read_from(data, offset)?;
        let flag_c = u8::read_from(data, offset)?;
        let flag_d = u8::read_from(data, offset)?;
        let raw_e = u32::read_from(data, offset)?;
        let raw_f = u32::read_from(data, offset)?;
        let pair_a = u32::read_from(data, offset)?;
        let pair_b = u32::read_from(data, offset)?;
        let raw_g = u64::read_from(data, offset)?;
        let raw_h = u32::read_from(data, offset)?;
        let raw_i = u16::read_from(data, offset)?;

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
            sequencer_desc, spawn_faction_spawn_data_info, spawn_faction_node_info,
            disable_faction_spawn_party_name_hash_list, raw_a, raw_b, raw_c,
            list_a, flag_a, flag_b, lookup_c, lookup_d, lookup_e,
            close_filter_a, close_filter_b, close_filter_c, filter_entry_list,
            lookup_f, lookup_g, lookup_h, list_b, list_c, lookup_i, raw_d,
            cstring_a, flag_c, flag_d, raw_e, raw_f, pair_a, pair_b,
            raw_g, raw_h, raw_i,
            tail_blob,
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
        self.spawn_faction_spawn_data_info.write_to(w)?;
        self.spawn_faction_node_info.write_to(w)?;
        self.disable_faction_spawn_party_name_hash_list.write_to(w)?;
        self.raw_a.write_to(w)?;
        self.raw_b.write_to(w)?;
        self.raw_c.write_to(w)?;
        self.list_a.write_to(w)?;
        self.flag_a.write_to(w)?;
        self.flag_b.write_to(w)?;
        self.lookup_c.write_to(w)?;
        self.lookup_d.write_to(w)?;
        self.lookup_e.write_to(w)?;
        self.close_filter_a.write_to(w)?;
        self.close_filter_b.write_to(w)?;
        self.close_filter_c.write_to(w)?;
        self.filter_entry_list.write_to(w)?;
        self.lookup_f.write_to(w)?;
        self.lookup_g.write_to(w)?;
        self.lookup_h.write_to(w)?;
        self.list_b.write_to(w)?;
        self.list_c.write_to(w)?;
        self.lookup_i.write_to(w)?;
        self.raw_d.write_to(w)?;
        self.cstring_a.write_to(w)?;
        self.flag_c.write_to(w)?;
        self.flag_d.write_to(w)?;
        self.raw_e.write_to(w)?;
        self.raw_f.write_to(w)?;
        self.pair_a.write_to(w)?;
        self.pair_b.write_to(w)?;
        self.raw_g.write_to(w)?;
        self.raw_h.write_to(w)?;
        self.raw_i.write_to(w)?;
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
        m.insert("spawn_faction_spawn_data_info".to_string(), self.spawn_faction_spawn_data_info.to_json_value());
        m.insert("spawn_faction_node_info".to_string(), self.spawn_faction_node_info.to_json_value());
        m.insert("disable_faction_spawn_party_name_hash_list".to_string(), self.disable_faction_spawn_party_name_hash_list.to_json_value());
        m.insert("raw_a".to_string(), self.raw_a.to_json_value());
        m.insert("raw_b".to_string(), self.raw_b.to_json_value());
        m.insert("raw_c".to_string(), self.raw_c.to_json_value());
        m.insert("list_a".to_string(), self.list_a.to_json_value());
        m.insert("flag_a".to_string(), self.flag_a.to_json_value());
        m.insert("flag_b".to_string(), self.flag_b.to_json_value());
        m.insert("lookup_c".to_string(), self.lookup_c.to_json_value());
        m.insert("lookup_d".to_string(), self.lookup_d.to_json_value());
        m.insert("lookup_e".to_string(), self.lookup_e.to_json_value());
        m.insert("close_filter_a".to_string(), self.close_filter_a.to_json_value());
        m.insert("close_filter_b".to_string(), self.close_filter_b.to_json_value());
        m.insert("close_filter_c".to_string(), self.close_filter_c.to_json_value());
        m.insert("filter_entry_list".to_string(), self.filter_entry_list.to_json_value());
        m.insert("lookup_f".to_string(), self.lookup_f.to_json_value());
        m.insert("lookup_g".to_string(), self.lookup_g.to_json_value());
        m.insert("lookup_h".to_string(), self.lookup_h.to_json_value());
        m.insert("list_b".to_string(), self.list_b.to_json_value());
        m.insert("list_c".to_string(), self.list_c.to_json_value());
        m.insert("lookup_i".to_string(), self.lookup_i.to_json_value());
        m.insert("raw_d".to_string(), self.raw_d.to_json_value());
        m.insert("cstring_a".to_string(), self.cstring_a.to_json_value());
        m.insert("flag_c".to_string(), self.flag_c.to_json_value());
        m.insert("flag_d".to_string(), self.flag_d.to_json_value());
        m.insert("raw_e".to_string(), self.raw_e.to_json_value());
        m.insert("raw_f".to_string(), self.raw_f.to_json_value());
        m.insert("pair_a".to_string(), self.pair_a.to_json_value());
        m.insert("pair_b".to_string(), self.pair_b.to_json_value());
        m.insert("raw_g".to_string(), self.raw_g.to_json_value());
        m.insert("raw_h".to_string(), self.raw_h.to_json_value());
        m.insert("raw_i".to_string(), self.raw_i.to_json_value());
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
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "spawn_faction_spawn_data_info")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "spawn_faction_node_info")?)?;
        <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "disable_faction_spawn_party_name_hash_list")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_a")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_b")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_c")?)?;
        <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "list_a")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_a")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_b")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_c")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_d")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_e")?)?;
        <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "close_filter_a")?)?;
        <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "close_filter_b")?)?;
        <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "close_filter_c")?)?;
        <CArray<StageFilterEntry> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "filter_entry_list")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_f")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_g")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_h")?)?;
        <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "list_b")?)?;
        <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "list_c")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_i")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_d")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "cstring_a")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_c")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_d")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_e")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_f")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "pair_a")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "pair_b")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_g")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_h")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_i")?)?;
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
