// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Pure-Rust mirror of `python_traits.rs` using `serde_json::Value`.
//!
//! Why this exists: Python consumers of crimson-rs use `parse_iteminfo_from_bytes`
//! and `serialize_iteminfo` which round-trip through dicts. Rust consumers (e.g.
//! DMM, a Tauri app with no embedded Python) need the same dict-style mutation
//! shape without paying for a Python interpreter. This module gives them that
//! by mirroring every `ToPyValue` / `WritePyValue` impl with
//! `ToJsonValue` / `WriteJsonValue` against `serde_json::Value`.
//!
//! The `py_binary_struct!` macro generates `to_json_dict()` and
//! `write_from_json_dict()` methods that parallel `to_py_dict()` and
//! `write_from_py_dict()`. Field names match the Python dict spec verbatim so
//! mod authors writing v3 files don't need to know which language a manager
//! uses.
//!
//! All numeric types serialize as JSON numbers. `u64` and `i64` rely on
//! serde_json::Number's full integer range (it stores them losslessly even
//! though JSON proper has no integer type).

use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use serde_json::{json, Map, Value};
use std::io;

use crate::binary::{CArray, CBytes, COptional, CString, LocalizableString};

// ── Traits ────────────────────────────────────────────────────────────────────

/// Convert a parsed binary value into a JSON value. Mirrors `ToPyValue`.
/// Output shape matches the Python dict shape so v3 mod files written
/// against the spec resolve identically.
pub trait ToJsonValue {
    fn to_json_value(&self) -> Value;
}

/// Read a JSON value and write the binary form of its underlying type.
/// Mirrors `WritePyValue`. Returns `io::Error` on shape mismatch (wrong
/// type, missing field, out-of-range integer) so callers can surface the
/// failure with a path before bailing out of a whole-record encode.
pub trait WriteJsonValue {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()>;
}

// ── Field lookup helper for generated dict writers ────────────────────────────

/// Pull a field out of a JSON object, returning a clear error if it's
/// missing. The generated `write_from_json_dict` calls this once per
/// declared struct field — matches Python's `get_field` behavior.
pub fn get_field<'a>(d: &'a Map<String, Value>, key: &str) -> io::Result<&'a Value> {
    d.get(key).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("missing JSON field '{}'", key),
        )
    })
}

/// Lenient variant of `get_field` used by the Rust/JSON serialization path
/// (`write_from_json_dict`). Returns `Value::Null` (owned) when the key is
/// absent instead of erroring. This lets old V3 mods that were authored
/// before a game patch added a new struct field (e.g. `unk_docking_108` in
/// `DockingChildData`) serialize successfully: the missing field gets the
/// zero-default written by the primitive `WriteJsonValue` impls below.
///
/// The Python path (`write_from_py_dict`) continues to use strict `get_field`
/// so Python callers still get clear errors for missing fields.
pub fn get_field_or_null(d: &Map<String, Value>, key: &str) -> Value {
    d.get(key).cloned().unwrap_or(Value::Null)
}

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn err<T>(msg: String) -> io::Result<T> {
    Err(io::Error::new(io::ErrorKind::InvalidData, msg))
}

// ── Primitives ────────────────────────────────────────────────────────────────

impl ToJsonValue for u8 {
    fn to_json_value(&self) -> Value {
        Value::from(*self)
    }
}
impl WriteJsonValue for u8 {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        // Null: field was absent from the mod dict (e.g. added by a later
        // game patch). Default to 0 so old mods stay forward-compatible.
        if v.is_null() { w.push(0); return Ok(()); }
        let n = v
            .as_u64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                format!("expected u8 number, got {}", type_name(v))))?;
        if n > u8::MAX as u64 {
            // Over-range mod set-value: clamp to field max instead of aborting
            // the whole-table encode. One out-of-range intent must not drop
            // every other intent/mod sharing the overlay (see No-Fall-Damage:
            // a 100-billion value on a u8/u32 field was killing the buff group).
            eprintln!("[V3_CLAMP] value {} exceeds u8 max {} — clamped (over-range mod set-value)", n, u8::MAX);
            w.push(u8::MAX);
            return Ok(());
        }
        w.push(n as u8);
        Ok(())
    }
}

impl ToJsonValue for u16 {
    fn to_json_value(&self) -> Value {
        Value::from(*self)
    }
}
impl WriteJsonValue for u16 {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() { w.extend_from_slice(&0u16.to_le_bytes()); return Ok(()); }
        let n = v
            .as_u64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                format!("expected u16 number, got {}", type_name(v))))?;
        if n > u16::MAX as u64 {
            eprintln!("[V3_CLAMP] value {} exceeds u16 max {} — clamped (over-range mod set-value)", n, u16::MAX);
            w.extend_from_slice(&u16::MAX.to_le_bytes());
            return Ok(());
        }
        w.extend_from_slice(&(n as u16).to_le_bytes());
        Ok(())
    }
}

impl ToJsonValue for u32 {
    fn to_json_value(&self) -> Value {
        Value::from(*self)
    }
}
impl WriteJsonValue for u32 {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() { w.extend_from_slice(&0u32.to_le_bytes()); return Ok(()); }
        let n = v
            .as_u64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                format!("expected u32 number, got {}", type_name(v))))?;
        if n > u32::MAX as u64 {
            eprintln!("[V3_CLAMP] value {} exceeds u32 max {} — clamped (over-range mod set-value)", n, u32::MAX);
            w.extend_from_slice(&u32::MAX.to_le_bytes());
            return Ok(());
        }
        w.extend_from_slice(&(n as u32).to_le_bytes());
        Ok(())
    }
}

impl ToJsonValue for u64 {
    fn to_json_value(&self) -> Value {
        Value::from(*self)
    }
}
impl WriteJsonValue for u64 {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() { w.extend_from_slice(&0u64.to_le_bytes()); return Ok(()); }
        // u64 may exceed i64::MAX where JS-side encoders downgrade to
        // string. Accept either form so spec-compliant writers stay
        // compatible with mod files round-tripped through web tooling.
        let n = match v {
            Value::Number(n) => n.as_u64(),
            Value::String(s) => s.parse::<u64>().ok(),
            _ => None,
        };
        let n = n.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            format!("expected u64 number/string, got {}", type_name(v))))?;
        w.extend_from_slice(&n.to_le_bytes());
        Ok(())
    }
}

impl ToJsonValue for i8 {
    fn to_json_value(&self) -> Value {
        Value::from(*self)
    }
}
impl WriteJsonValue for i8 {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() { w.extend_from_slice(&0i8.to_le_bytes()); return Ok(()); }
        let n = v
            .as_i64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                format!("expected i8 number, got {}", type_name(v))))?;
        if !(i8::MIN as i64..=i8::MAX as i64).contains(&n) {
            let c = n.clamp(i8::MIN as i64, i8::MAX as i64) as i8;
            eprintln!("[V3_CLAMP] value {} out of i8 range — clamped to {} (over-range mod set-value)", n, c);
            w.extend_from_slice(&c.to_le_bytes());
            return Ok(());
        }
        w.extend_from_slice(&(n as i8).to_le_bytes());
        Ok(())
    }
}

impl ToJsonValue for i64 {
    fn to_json_value(&self) -> Value {
        Value::from(*self)
    }
}
impl WriteJsonValue for i64 {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() { w.extend_from_slice(&0i64.to_le_bytes()); return Ok(()); }
        let n = match v {
            Value::Number(n) => n.as_i64(),
            Value::String(s) => s.parse::<i64>().ok(),
            _ => None,
        };
        let n = n.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            format!("expected i64 number/string, got {}", type_name(v))))?;
        w.extend_from_slice(&n.to_le_bytes());
        Ok(())
    }
}

/// Prefix for the lossless non-finite f32 encoding. See `ToJsonValue for f32`.
pub const F32_BITS_PREFIX: &str = "f32bits:";

impl ToJsonValue for f32 {
    fn to_json_value(&self) -> Value {
        // ⚠ JSON has no NaN/Infinity: `Value::from(f64::NAN)` is `Value::Null`,
        // and the writer below turns Null into 0.0 — so a non-finite float used
        // to be SILENTLY DESTROYED by any read→write JSON round-trip. That is
        // real data loss for a V3 mod that merely touches such a record.
        // (Found on 1.18 interaction_info, whose pivot vec3s contain 0xFFFFFFFF,
        // but the bug was general and version-independent.)
        //
        // Finite values stay plain JSON numbers — the common case and the mod
        // contract are unchanged. Only values that were previously lost get the
        // "f32bits:0x…" string form, so nothing that worked before can break.
        if self.is_finite() {
            Value::from(*self as f64)
        } else {
            Value::String(format!("{}{:#010x}", F32_BITS_PREFIX, self.to_bits()))
        }
    }
}
impl WriteJsonValue for f32 {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() { w.extend_from_slice(&0f32.to_le_bytes()); return Ok(()); }
        // Accept the lossless non-finite form emitted above.
        if let Some(hex) = v.as_str().and_then(|t| t.strip_prefix(F32_BITS_PREFIX)) {
            let digits = hex.strip_prefix("0x").unwrap_or(hex);
            let bits = u32::from_str_radix(digits, 16).map_err(|_| io::Error::new(
                io::ErrorKind::InvalidData,
                format!("bad {}{} value", F32_BITS_PREFIX, hex)))?;
            w.extend_from_slice(&f32::from_bits(bits).to_le_bytes());
            return Ok(());
        }
        let f = v
            .as_f64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                format!("expected f32 number, got {}", type_name(v))))?;
        w.extend_from_slice(&(f as f32).to_le_bytes());
        Ok(())
    }
}

impl ToJsonValue for f64 {
    fn to_json_value(&self) -> Value {
        Value::from(*self)
    }
}
impl WriteJsonValue for f64 {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() { w.extend_from_slice(&0f64.to_le_bytes()); return Ok(()); }
        let f = v
            .as_f64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                format!("expected f64 number, got {}", type_name(v))))?;
        w.extend_from_slice(&f.to_le_bytes());
        Ok(())
    }
}

// ── Fixed-size arrays ─────────────────────────────────────────────────────────
// [u8; N] base64 impl lives in `binary/arrays.rs` (predates this module).

impl ToJsonValue for [f32; 2] {
    fn to_json_value(&self) -> Value {
        Value::Array(self.iter().map(|x| x.to_json_value()).collect())
    }
}
impl WriteJsonValue for [f32; 2] {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let arr = v.as_array().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            format!("expected array of 2 f32, got {}", type_name(v))))?;
        if arr.len() != 2 {
            return err(format!("expected 2 elements for [f32; 2], got {}", arr.len()));
        }
        for elem in arr {
            f32::write_from_json(w, elem)?;
        }
        Ok(())
    }
}

impl ToJsonValue for [f32; 3] {
    fn to_json_value(&self) -> Value {
        Value::Array(self.iter().map(|x| x.to_json_value()).collect())
    }
}
impl WriteJsonValue for [f32; 3] {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let arr = v.as_array().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            format!("expected array of 3 f32, got {}", type_name(v))))?;
        if arr.len() != 3 {
            return err(format!("expected 3 elements for [f32; 3], got {}", arr.len()));
        }
        for elem in arr {
            f32::write_from_json(w, elem)?;
        }
        Ok(())
    }
}

impl ToJsonValue for [f32; 4] {
    fn to_json_value(&self) -> Value {
        Value::Array(self.iter().map(|x| x.to_json_value()).collect())
    }
}
impl WriteJsonValue for [f32; 4] {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let arr = v.as_array().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            format!("expected array of 4 f32, got {}", type_name(v))))?;
        if arr.len() != 4 {
            return err(format!("expected 4 elements for [f32; 4], got {}", arr.len()));
        }
        for elem in arr {
            f32::write_from_json(w, elem)?;
        }
        Ok(())
    }
}

impl ToJsonValue for [u32; 2] {
    fn to_json_value(&self) -> Value {
        Value::Array(self.iter().map(|x| Value::from(*x)).collect())
    }
}
impl WriteJsonValue for [u32; 2] {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let arr = v.as_array().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            format!("expected array of 2 u32, got {}", type_name(v))))?;
        if arr.len() != 2 {
            return err(format!("expected 2 elements for [u32; 2], got {}", arr.len()));
        }
        for elem in arr {
            u32::write_from_json(w, elem)?;
        }
        Ok(())
    }
}

impl ToJsonValue for [u32; 3] {
    fn to_json_value(&self) -> Value {
        Value::Array(self.iter().map(|x| Value::from(*x)).collect())
    }
}
impl WriteJsonValue for [u32; 3] {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let arr = v.as_array().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            format!("expected array of 3 u32, got {}", type_name(v))))?;
        if arr.len() != 3 {
            return err(format!("expected 3 elements for [u32; 3], got {}", arr.len()));
        }
        for elem in arr { u32::write_from_json(w, elem)?; }
        Ok(())
    }
}

impl ToJsonValue for [u32; 4] {
    fn to_json_value(&self) -> Value {
        Value::Array(self.iter().map(|x| Value::from(*x)).collect())
    }
}
impl WriteJsonValue for [u32; 4] {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let arr = v.as_array().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            format!("expected array of 4 u32, got {}", type_name(v))))?;
        if arr.len() != 4 {
            return err(format!("expected 4 elements for [u32; 4], got {}", arr.len()));
        }
        for elem in arr {
            u32::write_from_json(w, elem)?;
        }
        Ok(())
    }
}

// ── CString ───────────────────────────────────────────────────────────────────

impl ToJsonValue for CString<'_> {
    fn to_json_value(&self) -> Value {
        Value::String(self.data.to_string())
    }
}
impl WriteJsonValue for CString<'_> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let s = v.as_str().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            format!("expected string for CString, got {}", type_name(v))))?;
        let bytes = s.as_bytes();
        if bytes.len() > u32::MAX as usize {
            return err(format!("string too long ({} bytes)", bytes.len()));
        }
        w.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        w.extend_from_slice(bytes);
        Ok(())
    }
}

// ── CBytes ────────────────────────────────────────────────────────────────────

impl ToJsonValue for CBytes<'_> {
    fn to_json_value(&self) -> Value {
        Value::String(B64.encode(self.data))
    }
}
impl WriteJsonValue for CBytes<'_> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let s = v.as_str().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            "expected base64 string for CBytes"))?;
        let bytes = B64.decode(s).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
            format!("CBytes base64 decode: {}", e)))?;
        if bytes.len() > u32::MAX as usize {
            return err(format!("CBytes too long ({} bytes)", bytes.len()));
        }
        w.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        w.extend_from_slice(&bytes);
        Ok(())
    }
}

// ── LocalizableString ─────────────────────────────────────────────────────────
//
// Layout: u8 category, u64 index, u32 default_len, [u8; default_len].
// The Python bridge uses a dict with `category`, `index`, `default` keys —
// same shape here.

impl ToJsonValue for LocalizableString<'_> {
    fn to_json_value(&self) -> Value {
        json!({
            "category": self.category,
            "index": self.index,
            "default": self.default.data,
        })
    }
}
impl WriteJsonValue for LocalizableString<'_> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            format!("expected object for LocalizableString, got {}", type_name(v))))?;
        let category = get_field(obj, "category")?
            .as_u64().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "LocalizableString.category: expected u8 number"))?;
        if category > u8::MAX as u64 {
            return err(format!("LocalizableString.category {} out of u8 range", category));
        }
        let index = get_field(obj, "index")?
            .as_u64().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "LocalizableString.index: expected u64 number"))?;
        let default = get_field(obj, "default")?
            .as_str().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "LocalizableString.default: expected string"))?;
        w.push(category as u8);
        w.extend_from_slice(&index.to_le_bytes());
        w.extend_from_slice(&(default.len() as u32).to_le_bytes());
        w.extend_from_slice(default.as_bytes());
        Ok(())
    }
}

// ── CArray ────────────────────────────────────────────────────────────────────

impl<T: ToJsonValue> ToJsonValue for CArray<T> {
    fn to_json_value(&self) -> Value {
        Value::Array(self.items.iter().map(|x| x.to_json_value()).collect())
    }
}
impl<T: WriteJsonValue> WriteJsonValue for CArray<T> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        // Absent field (null) → empty array. Keeps old V3 mods forward-compatible
        // when a game patch adds a new CArray field they don't carry (e.g. 1.13.00
        // PrefabData gained animation_path_list): the macro's get_field_or_null
        // passes Null, and we serialize count 0 instead of aborting the overlay.
        if v.is_null() {
            w.extend_from_slice(&0u32.to_le_bytes());
            return Ok(());
        }
        let arr = v.as_array().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            format!("expected array for CArray, got {}", type_name(v))))?;
        if arr.len() > u32::MAX as usize {
            return err(format!("CArray too long ({} elements)", arr.len()));
        }
        w.extend_from_slice(&(arr.len() as u32).to_le_bytes());
        for (i, elem) in arr.iter().enumerate() {
            T::write_from_json(w, elem).map_err(|e| io::Error::new(
                e.kind(), format!("[{}]: {}", i, e),
            ))?;
        }
        Ok(())
    }
}

// ── COptional ─────────────────────────────────────────────────────────────────
//
// Wire format: u8 flag (0 = absent, 1 = present), then T's bytes if flag=1.
// JSON encoding: `null` ↔ absent, anything else ↔ present.

impl<T: ToJsonValue> ToJsonValue for COptional<T> {
    fn to_json_value(&self) -> Value {
        match &self.value {
            Some(v) => v.to_json_value(),
            None => Value::Null,
        }
    }
}
impl<T: WriteJsonValue> WriteJsonValue for COptional<T> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() {
            w.push(0);
        } else {
            w.push(1);
            T::write_from_json(w, v)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod clamp_tests {
    use super::*;
    use serde_json::json;

    // Over-range mod set-values must CLAMP to the field max (with a warning),
    // not error — one bad intent must never abort a whole-table encode and
    // drop every other intent/mod sharing the overlay. (No-Fall-Damage set a
    // u32 field to 100_000_000_000; that previously killed the buff group.)
    #[test]
    fn over_range_integers_clamp_not_error() {
        let mut w = Vec::new();
        u32::write_from_json(&mut w, &json!(100_000_000_000u64)).expect("u32 clamps");
        assert_eq!(w, u32::MAX.to_le_bytes());

        let mut w = Vec::new();
        u16::write_from_json(&mut w, &json!(70_000u64)).expect("u16 clamps");
        assert_eq!(w, u16::MAX.to_le_bytes());

        let mut w = Vec::new();
        u8::write_from_json(&mut w, &json!(999u64)).expect("u8 clamps");
        assert_eq!(w, [u8::MAX]);

        let mut w = Vec::new();
        i8::write_from_json(&mut w, &json!(-999i64)).expect("i8 clamps");
        assert_eq!(w, (i8::MIN).to_le_bytes());
    }

    // In-range values are unaffected — roundtrip stays byte-exact.
    #[test]
    fn in_range_integers_unchanged() {
        let mut w = Vec::new();
        u32::write_from_json(&mut w, &json!(1_000_000u64)).unwrap();
        assert_eq!(w, 1_000_000u32.to_le_bytes());
    }
}
