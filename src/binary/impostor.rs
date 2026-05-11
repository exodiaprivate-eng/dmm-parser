// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.imp` and `.impostor` — low-poly billboard fallback descriptors.
//!
//! Both formats describe impostor (camera-facing billboard) variants
//! used as far-distance LODs for vegetation, props, etc. Audited
//! iter 19 against the 1.06 install:
//!
//! - **`.imp`** (30/30 samples = 72 bytes each):
//!   ```text
//!   offset  size  field
//!   0       4     magic "IMP "
//!   4       4     u32 reserved   (always 256 across corpus)
//!   8       64    opaque body    (impostor parameters TBD)
//!   ```
//!
//! - **`.impostor`** (48 bytes = 12 little-endian f32 values):
//!   ```text
//!   floats[0..4]   — likely bounding/scale extents (varies per mesh)
//!   floats[4..6]   — always 0 (rotation pad?)
//!   floats[6..12]  — additional spatial parameters
//!   ```
//!
//! Semantic decoding of the .imp body and the .impostor float fields
//! is queued; the formats are fully classified + round-trip via
//! `body_b64` / structured float array. Mod authors can edit the
//! floats directly via the structured view.

use std::io::{self};

use base64::Engine;
use serde_json::{Map, Value};

pub const IMP_MAGIC: &[u8; 4] = b"IMP ";
pub const IMP_FIXED_SIZE: usize = 72;
pub const IMPOSTOR_FIXED_SIZE: usize = 48;
pub const IMPOSTOR_FLOAT_COUNT: usize = 12;

// ── .imp ────────────────────────────────────────────────────────────────────

pub fn parse_imp_to_json(data: &[u8]) -> io::Result<Value> {
    if data.len() != IMP_FIXED_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!(".imp expected {} bytes, got {}", IMP_FIXED_SIZE, data.len())));
    }
    if &data[0..4] != IMP_MAGIC {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!(".imp magic mismatch: got {:?}",
                std::str::from_utf8(&data[0..4]).unwrap_or("<non-utf8>"))));
    }
    let reserved = u32::from_le_bytes(data[4..8].try_into().unwrap());
    let body_b64 = base64::engine::general_purpose::STANDARD.encode(data);

    let mut map = Map::new();
    map.insert("key".to_string(), Value::from(0u64));
    map.insert("string_key".to_string(), Value::from(""));
    map.insert("magic".to_string(), Value::from("IMP "));
    map.insert("reserved".to_string(), Value::from(reserved as u64));
    map.insert("body_b64".to_string(), Value::from(body_b64));
    map.insert("body_len".to_string(), Value::from(data.len() as u64));
    Ok(Value::Object(map))
}

pub fn serialize_imp_from_json(value: &Value) -> io::Result<Vec<u8>> {
    let map = value.as_object().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData, ".imp serialize: expected object root"))?;
    let body_b64 = map.get("body_b64")
        .and_then(|v| v.as_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            ".imp serialize: missing 'body_b64'"))?;
    base64::engine::general_purpose::STANDARD.decode(body_b64).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData,
            format!(".imp serialize: body_b64 decode failed: {}", e))
    })
}

// ── .impostor ───────────────────────────────────────────────────────────────

pub fn parse_impostor_to_json(data: &[u8]) -> io::Result<Value> {
    if data.len() != IMPOSTOR_FIXED_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!(".impostor expected {} bytes, got {}", IMPOSTOR_FIXED_SIZE, data.len())));
    }
    let mut floats = Vec::with_capacity(IMPOSTOR_FLOAT_COUNT);
    for i in 0..IMPOSTOR_FLOAT_COUNT {
        let f = f32::from_le_bytes(data[i * 4..(i + 1) * 4].try_into().unwrap());
        floats.push(f);
    }

    let mut map = Map::new();
    map.insert("key".to_string(), Value::from(0u64));
    map.insert("string_key".to_string(), Value::from(""));
    map.insert("floats".to_string(),
        Value::Array(floats.iter().map(|&f| {
            serde_json::Number::from_f64(f as f64)
                .map(Value::Number).unwrap_or(Value::Null)
        }).collect()));
    map.insert("body_len".to_string(), Value::from(data.len() as u64));
    Ok(Value::Object(map))
}

pub fn serialize_impostor_from_json(value: &Value) -> io::Result<Vec<u8>> {
    let map = value.as_object().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData, ".impostor serialize: expected object root"))?;
    let floats = map.get("floats")
        .and_then(|v| v.as_array())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            ".impostor serialize: missing 'floats' array"))?;
    if floats.len() != IMPOSTOR_FLOAT_COUNT {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!(".impostor serialize: expected {} floats, got {}",
                IMPOSTOR_FLOAT_COUNT, floats.len())));
    }
    let mut out = Vec::with_capacity(IMPOSTOR_FIXED_SIZE);
    for (i, f) in floats.iter().enumerate() {
        let v = f.as_f64().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            format!(".impostor serialize: floats[{}] not numeric", i)))?;
        out.extend_from_slice(&(v as f32).to_le_bytes());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_imp() -> Vec<u8> {
        let mut v = Vec::with_capacity(IMP_FIXED_SIZE);
        v.extend_from_slice(IMP_MAGIC);
        v.extend_from_slice(&256u32.to_le_bytes());
        v.resize(IMP_FIXED_SIZE, 0xAB);
        v
    }

    #[test]
    fn imp_parse_and_round_trip() {
        let bytes = make_imp();
        let v = parse_imp_to_json(&bytes).expect("parse");
        let m = v.as_object().unwrap();
        assert_eq!(m["magic"], Value::from("IMP "));
        assert_eq!(m["reserved"], Value::from(256u64));
        let written = serialize_imp_from_json(&v).expect("write");
        assert_eq!(written, bytes);
    }

    #[test]
    fn imp_rejects_wrong_size() {
        let bytes = vec![0u8; 71];
        assert!(parse_imp_to_json(&bytes).is_err());
    }

    #[test]
    fn imp_rejects_wrong_magic() {
        let mut bytes = make_imp();
        bytes[0] = b'X';
        assert!(parse_imp_to_json(&bytes).is_err());
    }

    fn make_impostor() -> Vec<u8> {
        let mut v = Vec::new();
        for f in [2.508f32, -0.026, 1.548, 0.128, 0.0, 0.0, -1.359, -0.164, -1.13, 1.308, 3.26, 1.386] {
            v.extend_from_slice(&f.to_le_bytes());
        }
        v
    }

    #[test]
    fn impostor_parse_and_round_trip() {
        let bytes = make_impostor();
        let v = parse_impostor_to_json(&bytes).expect("parse");
        let floats: Vec<f32> = v["floats"].as_array().unwrap()
            .iter().map(|x| x.as_f64().unwrap() as f32).collect();
        assert_eq!(floats.len(), 12);
        assert!((floats[0] - 2.508).abs() < 0.001);
        assert_eq!(floats[4], 0.0);
        assert_eq!(floats[5], 0.0);
        let written = serialize_impostor_from_json(&v).expect("write");
        assert_eq!(written, bytes);
    }

    #[test]
    fn impostor_rejects_wrong_size() {
        let bytes = vec![0u8; 47];
        assert!(parse_impostor_to_json(&bytes).is_err());
    }
}
