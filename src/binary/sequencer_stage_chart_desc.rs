//! Partial typed wrapper for SequencerStageChartDesc (sub_141D8C6D0).
//!
//! The full per-element reader has 26 wire fields / 232 mem bytes.
//! Field 19 introduces a polymorphic family (SequencerStageTrackChange-
//! Data via sub_14110C270) that needs its own family-decoder pass, so
//! the typed prefix tops out at field 15 for now. The first 15 wire
//! fields each have a deterministic length:
//!
//!   1. CString name
//!   2. u32 raw
//!   3. CString prefab_path
//!   4. [f32; 3] position (Vec3)
//!   5. u32 raw
//!   6-13. 8× u8 flag
//!  14. u32 lookup_a (sub_141106210, qword_145F113B8 hash)
//!  15. OptionalGameCondition cond_a (sub_141103B30 — u8 presence +
//!      optional GameCondition tree + 3 footer bytes)
//!  16. CString cstring_a
//!  17. CString cstring_b
//!  18. CArray<(CString, CString)> string_pair_list
//!
//! `SequencerStageChartDescPartial` reads those 18 fields explicitly
//! and stores everything after as `opaque_tail: Vec<u8>`. For consumers
//! that own a SequencerStageChartDesc bounded by entry-size arithmetic
//! (e.g. `field_revive_info`'s single-instance case), this gives users
//! field-level edit access to the prefix without losing round-trip on
//! the unfinished tail.

use crate::binary::optional_game_condition::OptionalGameCondition;
use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use crate::py_binary_struct;
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
use std::io::{self, Write};

py_binary_struct! {
    /// Inner element of `string_pair_list` — wire is 2× CString,
    /// 16-byte mem stride per element (sub_141D8C6D0's loop body).
    pub struct StringPair<'a> {
        pub key: CString<'a>,
        pub value: CString<'a>,
    }
}

#[derive(Debug)]
pub struct SequencerStageChartDescPartial<'a> {
    pub name: CString<'a>,
    pub raw_a: u32,
    pub prefab_path: CString<'a>,
    pub position: [f32; 3],
    pub raw_b: u32,
    pub flag_a: u8,
    pub flag_b: u8,
    pub flag_c: u8,
    pub flag_d: u8,
    pub flag_e: u8,
    pub flag_f: u8,
    pub flag_g: u8,
    pub flag_h: u8,
    /// u32 wire / u16 mem hash (sub_141106210 → qword_145F113B8).
    pub lookup_a: u32,
    /// `OptionalGameCondition` (sub_141103B30 — u8 presence + optional
    /// recursive GameCondition tree + 3 footer bytes). When the tree
    /// hits an anti-disassembly tag, the typed read fails; consumers
    /// fall back to opaque-tail mode in that case via
    /// `read_with_size`'s outer error path.
    pub cond_a: OptionalGameCondition<'a>,
    pub cstring_a: CString<'a>,
    pub cstring_b: CString<'a>,
    pub string_pair_list: CArray<StringPair<'a>>,
    /// Bytes 19-26 of the wire layout (2 polymorphic CArrays + 4
    /// helper structs). Stays opaque until the
    /// SequencerStageTrackChangeData family decoder is shipped.
    pub opaque_tail: Vec<u8>,
}

impl<'a> SequencerStageChartDescPartial<'a> {
    /// Read a SequencerStageChartDesc whose total wire size on disk is
    /// known via `total_size`. The 13-field typed prefix is consumed
    /// from `offset`, and the remaining `total_size - prefix_bytes`
    /// trail into `opaque_tail`.
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        total_size: usize,
    ) -> io::Result<Self> {
        let blob_start = *offset;
        let blob_end = blob_start
            .checked_add(total_size)
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "SequencerStageChartDescPartial: total_size overflow",
            ))?;
        if blob_end > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "SequencerStageChartDescPartial: blob extends past data ({} > {})",
                    blob_end, data.len()
                ),
            ));
        }

        let name = CString::read_from(data, offset)?;
        let raw_a = u32::read_from(data, offset)?;
        let prefab_path = CString::read_from(data, offset)?;
        let position = <[f32; 3]>::read_from(data, offset)?;
        let raw_b = u32::read_from(data, offset)?;
        let flag_a = u8::read_from(data, offset)?;
        let flag_b = u8::read_from(data, offset)?;
        let flag_c = u8::read_from(data, offset)?;
        let flag_d = u8::read_from(data, offset)?;
        let flag_e = u8::read_from(data, offset)?;
        let flag_f = u8::read_from(data, offset)?;
        let flag_g = u8::read_from(data, offset)?;
        let flag_h = u8::read_from(data, offset)?;
        let lookup_a = u32::read_from(data, offset)?;
        let cond_a = OptionalGameCondition::read_from(data, offset)?;
        let cstring_a = CString::read_from(data, offset)?;
        let cstring_b = CString::read_from(data, offset)?;
        let string_pair_list = CArray::<StringPair>::read_from(data, offset)?;

        if *offset > blob_end {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "SequencerStageChartDescPartial: typed prefix overran blob ({} > {})",
                    *offset, blob_end
                ),
            ));
        }
        let opaque_tail = data[*offset..blob_end].to_vec();
        *offset = blob_end;

        Ok(Self {
            name, raw_a, prefab_path, position, raw_b,
            flag_a, flag_b, flag_c, flag_d, flag_e, flag_f, flag_g, flag_h,
            lookup_a, cond_a, cstring_a, cstring_b, string_pair_list,
            opaque_tail,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.name.write_to(w)?;
        self.raw_a.write_to(w)?;
        self.prefab_path.write_to(w)?;
        self.position.write_to(w)?;
        self.raw_b.write_to(w)?;
        self.flag_a.write_to(w)?;
        self.flag_b.write_to(w)?;
        self.flag_c.write_to(w)?;
        self.flag_d.write_to(w)?;
        self.flag_e.write_to(w)?;
        self.flag_f.write_to(w)?;
        self.flag_g.write_to(w)?;
        self.flag_h.write_to(w)?;
        self.lookup_a.write_to(w)?;
        self.cond_a.write_to(w)?;
        self.cstring_a.write_to(w)?;
        self.cstring_b.write_to(w)?;
        self.string_pair_list.write_to(w)?;
        w.write_all(&self.opaque_tail)?;
        Ok(())
    }

    pub fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("name".to_string(), self.name.to_json_value());
        m.insert("raw_a".to_string(), self.raw_a.to_json_value());
        m.insert("prefab_path".to_string(), self.prefab_path.to_json_value());
        m.insert("position".to_string(), self.position.to_json_value());
        m.insert("raw_b".to_string(), self.raw_b.to_json_value());
        m.insert("flag_a".to_string(), self.flag_a.to_json_value());
        m.insert("flag_b".to_string(), self.flag_b.to_json_value());
        m.insert("flag_c".to_string(), self.flag_c.to_json_value());
        m.insert("flag_d".to_string(), self.flag_d.to_json_value());
        m.insert("flag_e".to_string(), self.flag_e.to_json_value());
        m.insert("flag_f".to_string(), self.flag_f.to_json_value());
        m.insert("flag_g".to_string(), self.flag_g.to_json_value());
        m.insert("flag_h".to_string(), self.flag_h.to_json_value());
        m.insert("lookup_a".to_string(), self.lookup_a.to_json_value());
        m.insert("cond_a".to_string(), self.cond_a.to_json_value());
        m.insert("cstring_a".to_string(), self.cstring_a.to_json_value());
        m.insert("cstring_b".to_string(), self.cstring_b.to_json_value());
        m.insert("string_pair_list".to_string(), self.string_pair_list.to_json_value());
        m.insert("_opaque_tail_b64".to_string(), Value::String(B64.encode(&self.opaque_tail)));
        Value::Object(m)
    }

    pub fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "SequencerStageChartDescPartial: expected object",
        ))?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "name")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_a")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "prefab_path")?)?;
        <[f32; 3] as WriteJsonValue>::write_from_json(w, json_get_field(obj, "position")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_b")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_a")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_b")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_c")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_d")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_e")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_f")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_g")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_h")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_a")?)?;
        OptionalGameCondition::write_from_json(w, json_get_field(obj, "cond_a")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "cstring_a")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "cstring_b")?)?;
        <CArray<StringPair> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_pair_list")?)?;
        let b64 = json_get_field(obj, "_opaque_tail_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "SequencerStageChartDescPartial: _opaque_tail_b64 must be a string",
            ))?;
        let bytes = B64.decode(b64).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("SequencerStageChartDescPartial: _opaque_tail_b64 invalid base64: {}", e),
        ))?;
        w.extend_from_slice(&bytes);
        Ok(())
    }
}
