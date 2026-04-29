//! EffectData per-element decoder.
//!
//! Per Win-IDA `sub_1410DBAF0` (one EffectData element on the wire):
//!
//! ```text
//! 1. byte_a:        u8                                               (1)
//! 2. lookup_b:      u32 hash → u16 (read_u32_lookup_EF18)            (4)
//! 3. core:          254-byte fixed block via sub_1410D4110           (254)
//! 4. lookups_c:     6 × u32 hash → u16 (read_u32_lookup_DA30)        (24)
//! 5. fields_d:      4 × u32                                          (16)
//! 6. byte_e:        u8                                               (1)
//! 7. cstring_list:  CArray<CString> (sub_14106BAC0)                  (4 + Σ)
//! 8. fixed144_list: CArray<[u8;144]> (sub_141117080)                 (4 + 144*N)
//! 9. nested_u32_lists: CArray<{u32 key, CArray<u32> values}>
//!    (sub_141116ED0 → sub_141101AB0)                                 (variable)
//! 10. inner_map:    CArray<{u32 key, EffectDataInner}>
//!    (sub_141116CA0 → sub_1410DB840)                                 (variable, recursive)
//! ```
//!
//! `EffectDataInner` (sub_1410DB840) is similar shape and contains
//! nested CArrays; it's the recursive part of the family. Walked to
//! determine wire size; stored as opaque bytes for now (typing it
//! end-to-end is mechanical follow-up work).

use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
use std::io::{self, Write};

const CORE_FIXED_SIZE: usize = 254;
const FIXED144_ELEMENT_SIZE: usize = 144;

/// One EffectData element on the wire. Most fields are individually
/// typed; the recursive `inner_map_blob` is captured byte-perfect for
/// round-trip until its full struct shape ships.
#[derive(Debug)]
pub struct EffectDataElement<'a> {
    pub byte_a: u8,
    pub lookup_b: u32,
    /// 254-byte fixed block read by sub_1410D4110:
    /// 7×Vec3 + 7×u32 + Vec4(16B) + 2×u32 + u8 + u8 + u16 + u32  (sub_1410D3DC0, 144B)
    /// + u32 + 2×Vec3 + qword(8B) + u32 + 4×Vec3 + u32 + u32 + 14×u8
    /// Captured as raw bytes; field-level typing is straightforward
    /// follow-up work since shape is fully known.
    pub core_block: [u8; CORE_FIXED_SIZE],
    pub lookups_c: [u32; 6],
    pub fields_d: [u32; 4],
    pub byte_e: u8,
    pub cstring_list: Vec<CString<'a>>,
    pub fixed144_list: Vec<[u8; FIXED144_ELEMENT_SIZE]>,
    /// `CArray<{u32 key, CArray<u32> values}>` opaque — typed shape
    /// known but not yet expanded into Rust. Walked to find boundary.
    pub nested_u32_lists_blob: Vec<u8>,
    /// `CArray<{u32 key, EffectDataInner value}>` opaque — recursive
    /// nested struct, typing deferred.
    pub inner_map_blob: Vec<u8>,
}

/// Walk a CArray<u32> wire layout and return its total byte size.
fn walk_carray_u32(data: &[u8], offset: usize) -> io::Result<usize> {
    if offset + 4 > data.len() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "carray u32 count"));
    }
    let n = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
    let total = 4 + n * 4;
    if offset + total > data.len() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "carray u32 body"));
    }
    Ok(total)
}

/// Walk a CArray<CString>.
fn walk_carray_cstring(data: &[u8], offset: usize) -> io::Result<usize> {
    if offset + 4 > data.len() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "carray cstring count"));
    }
    let n = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
    let mut cur = offset + 4;
    for _ in 0..n {
        if cur + 4 > data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "cstring length prefix"));
        }
        let len = u32::from_le_bytes(data[cur..cur+4].try_into().unwrap()) as usize;
        cur += 4 + len;
        if cur > data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "cstring body"));
        }
    }
    Ok(cur - offset)
}

/// Walk a CArray<{fixed N bytes}>.
fn walk_carray_fixed(data: &[u8], offset: usize, item_size: usize) -> io::Result<usize> {
    if offset + 4 > data.len() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "carray fixed count"));
    }
    let n = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
    let total = 4 + n * item_size;
    if offset + total > data.len() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "carray fixed body"));
    }
    Ok(total)
}

/// Walk `nested_u32_lists`: CArray<CArray<u32>>. Each outer element is
/// just a nested CArray<u32> — no per-element key. Per sub_141116ED0
/// reading sub_141101AB0 once per outer element.
fn walk_nested_u32_lists(data: &[u8], offset: usize) -> io::Result<usize> {
    if offset + 4 > data.len() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "nested u32 lists count"));
    }
    let n = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
    let mut cur = offset + 4;
    for _ in 0..n {
        cur += walk_carray_u32(data, cur)?;
    }
    Ok(cur - offset)
}

/// Walk one EffectDataInner record (sub_1410DB840).
fn walk_effect_data_inner(data: &[u8], offset: usize) -> io::Result<usize> {
    let mut cur = offset;
    // u32 prefix
    cur += 4;
    // sub_1410D4110 block: 254 bytes fixed
    cur += CORE_FIXED_SIZE;
    // 6 × read_u32_lookup_DA30 (4 bytes each)
    cur += 24;
    // sub_141102990: CArray<u32-lookup-as-u16>. Per sub_1410A9D40, each
    // element is a CString-style record (u32 len + len bytes that get
    // hashed at runtime), NOT a fixed-4-byte u32 — the v11-byte seek
    // forward in the reader is the giveaway.
    cur += walk_carray_cstring(data, cur)?;
    // sub_141102A60: CArray<f32> (4 bytes per element)
    cur += walk_carray_u32(data, cur)?;
    // 12 + 12 + 12 + 12 + 4 = 52 bytes of vectors
    cur += 52;
    // sub_14106BAC0: CArray<CString>
    cur += walk_carray_cstring(data, cur)?;
    // sub_141117080: CArray<[u8; 144]>
    cur += walk_carray_fixed(data, cur, FIXED144_ELEMENT_SIZE)?;
    // 2-byte trailing field
    cur += 2;
    if cur > data.len() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "effect data inner overran"));
    }
    Ok(cur - offset)
}

/// Walk inner_map: CArray<{u32 key, EffectDataInner value}>.
fn walk_inner_map(data: &[u8], offset: usize) -> io::Result<usize> {
    if offset + 4 > data.len() {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "inner_map count"));
    }
    let n = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
    let mut cur = offset + 4;
    for _ in 0..n {
        // u32 key
        if cur + 4 > data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "inner_map key"));
        }
        cur += 4;
        // EffectDataInner value
        cur += walk_effect_data_inner(data, cur)?;
    }
    Ok(cur - offset)
}

impl<'a> EffectDataElement<'a> {
    pub fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let byte_a = u8::read_from(data, offset)?;
        let lookup_b = u32::read_from(data, offset)?;
        if *offset + CORE_FIXED_SIZE > data.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "effect data core block"));
        }
        let mut core_block = [0u8; CORE_FIXED_SIZE];
        core_block.copy_from_slice(&data[*offset..*offset + CORE_FIXED_SIZE]);
        *offset += CORE_FIXED_SIZE;
        let mut lookups_c = [0u32; 6];
        for x in &mut lookups_c { *x = u32::read_from(data, offset)?; }
        let mut fields_d = [0u32; 4];
        for x in &mut fields_d { *x = u32::read_from(data, offset)?; }
        let byte_e = u8::read_from(data, offset)?;

        let cstring_count = u32::read_from(data, offset)? as usize;
        let mut cstring_list = Vec::with_capacity(cstring_count);
        for _ in 0..cstring_count {
            cstring_list.push(CString::read_from(data, offset)?);
        }

        let fixed144_count = u32::read_from(data, offset)? as usize;
        let mut fixed144_list = Vec::with_capacity(fixed144_count);
        for _ in 0..fixed144_count {
            if *offset + FIXED144_ELEMENT_SIZE > data.len() {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "fixed144 element"));
            }
            let mut buf = [0u8; FIXED144_ELEMENT_SIZE];
            buf.copy_from_slice(&data[*offset..*offset + FIXED144_ELEMENT_SIZE]);
            *offset += FIXED144_ELEMENT_SIZE;
            fixed144_list.push(buf);
        }

        let nested_size = walk_nested_u32_lists(data, *offset)?;
        let nested_u32_lists_blob = data[*offset..*offset + nested_size].to_vec();
        *offset += nested_size;

        let map_size = walk_inner_map(data, *offset)?;
        let inner_map_blob = data[*offset..*offset + map_size].to_vec();
        *offset += map_size;

        Ok(Self {
            byte_a, lookup_b, core_block, lookups_c, fields_d, byte_e,
            cstring_list, fixed144_list, nested_u32_lists_blob, inner_map_blob,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.byte_a.write_to(w)?;
        self.lookup_b.write_to(w)?;
        w.write_all(&self.core_block)?;
        for x in &self.lookups_c { x.write_to(w)?; }
        for x in &self.fields_d { x.write_to(w)?; }
        self.byte_e.write_to(w)?;
        (self.cstring_list.len() as u32).write_to(w)?;
        for s in &self.cstring_list { s.write_to(w)?; }
        (self.fixed144_list.len() as u32).write_to(w)?;
        for buf in &self.fixed144_list { w.write_all(buf)?; }
        w.write_all(&self.nested_u32_lists_blob)?;
        w.write_all(&self.inner_map_blob)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("byte_a".to_string(), self.byte_a.to_json_value());
        m.insert("lookup_b".to_string(), self.lookup_b.to_json_value());
        m.insert("_core_block_b64".to_string(), Value::String(B64.encode(&self.core_block)));
        m.insert(
            "lookups_c".to_string(),
            Value::Array(self.lookups_c.iter().map(|v| v.to_json_value()).collect()),
        );
        m.insert(
            "fields_d".to_string(),
            Value::Array(self.fields_d.iter().map(|v| v.to_json_value()).collect()),
        );
        m.insert("byte_e".to_string(), self.byte_e.to_json_value());
        m.insert(
            "cstring_list".to_string(),
            Value::Array(self.cstring_list.iter().map(|s| s.to_json_value()).collect()),
        );
        m.insert(
            "fixed144_list".to_string(),
            Value::Array(self.fixed144_list.iter()
                .map(|buf| Value::String(B64.encode(buf)))
                .collect()),
        );
        m.insert(
            "_nested_u32_lists_blob_b64".to_string(),
            Value::String(B64.encode(&self.nested_u32_lists_blob)),
        );
        m.insert(
            "_inner_map_blob_b64".to_string(),
            Value::String(B64.encode(&self.inner_map_blob)),
        );
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "byte_a")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_b")?)?;
        let core_b64 = json_get_field(obj, "_core_block_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "EffectDataElement: _core_block_b64 must be base64 string"))?;
        let core_bytes = B64.decode(core_b64).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("EffectDataElement: _core_block_b64 invalid: {}", e)))?;
        if core_bytes.len() != CORE_FIXED_SIZE {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("EffectDataElement: _core_block_b64 must be {} bytes, got {}",
                    CORE_FIXED_SIZE, core_bytes.len())));
        }
        w.extend_from_slice(&core_bytes);
        let lookups_c = json_get_field(obj, "lookups_c")?.as_array()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "EffectDataElement: lookups_c must be array"))?;
        if lookups_c.len() != 6 {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("EffectDataElement: lookups_c must have 6 items, got {}", lookups_c.len())));
        }
        for v in lookups_c { <u32 as WriteJsonValue>::write_from_json(w, v)?; }
        let fields_d = json_get_field(obj, "fields_d")?.as_array()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "EffectDataElement: fields_d must be array"))?;
        if fields_d.len() != 4 {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("EffectDataElement: fields_d must have 4 items, got {}", fields_d.len())));
        }
        for v in fields_d { <u32 as WriteJsonValue>::write_from_json(w, v)?; }
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "byte_e")?)?;
        let cstrs = json_get_field(obj, "cstring_list")?.as_array()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "EffectDataElement: cstring_list must be array"))?;
        (cstrs.len() as u32).write_to(w)?;
        for v in cstrs { <CString as WriteJsonValue>::write_from_json(w, v)?; }
        let f144s = json_get_field(obj, "fixed144_list")?.as_array()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "EffectDataElement: fixed144_list must be array"))?;
        (f144s.len() as u32).write_to(w)?;
        for v in f144s {
            let s = v.as_str().ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "EffectDataElement: fixed144_list elements must be base64 strings"))?;
            let bytes = B64.decode(s).map_err(|e| io::Error::new(
                io::ErrorKind::InvalidData,
                format!("EffectDataElement: fixed144 base64: {}", e)))?;
            if bytes.len() != FIXED144_ELEMENT_SIZE {
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                    format!("EffectDataElement: fixed144 must be {} bytes, got {}",
                        FIXED144_ELEMENT_SIZE, bytes.len())));
            }
            w.extend_from_slice(&bytes);
        }
        let nested_b64 = json_get_field(obj, "_nested_u32_lists_blob_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "EffectDataElement: _nested_u32_lists_blob_b64 must be base64 string"))?;
        let nested_bytes = B64.decode(nested_b64).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData, format!("nested blob: {}", e)))?;
        w.extend_from_slice(&nested_bytes);
        let inner_b64 = json_get_field(obj, "_inner_map_blob_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "EffectDataElement: _inner_map_blob_b64 must be base64 string"))?;
        let inner_bytes = B64.decode(inner_b64).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData, format!("inner_map blob: {}", e)))?;
        w.extend_from_slice(&inner_bytes);
        Ok(())
    }
}
