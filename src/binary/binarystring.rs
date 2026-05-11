// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.binarystring` — packed UTF-8 string list.
//!
//! Identified iter 17 from `gamestring.binarystring` (the only file
//! observed in the 1.06 install, 19259 bytes). Format:
//! ```text
//! offset  size  field
//! 0       2     u16 count
//! 2       ...   records: { u8 length, [u8; length] utf8_bytes }
//! ```
//!
//! Length-prefix is **u8** (single byte) — strings are limited to 255
//! chars. Byte-perfect round-trip via raw bytes (no opaque body needed
//! since the format is fully known).

use std::io::{self};

use base64::Engine;
use serde_json::{Map, Value};

pub fn parse_binarystring_to_json(data: &[u8]) -> io::Result<Value> {
    if data.len() < 2 {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!(".binarystring too short: {} < 2", data.len())));
    }
    let count = u16::from_le_bytes(data[0..2].try_into().unwrap()) as usize;

    let mut strings = Vec::with_capacity(count);
    let mut off = 2usize;
    for i in 0..count {
        if off >= data.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!(".binarystring truncated at record {} (offset {})", i, off)));
        }
        let len = data[off] as usize;
        off += 1;
        if off + len > data.len() {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!(".binarystring record {} length {} overflows body at offset {}",
                    i, len, off)));
        }
        let s = std::str::from_utf8(&data[off..off + len]).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData,
                format!(".binarystring record {} not utf-8: {}", i, e))
        })?;
        strings.push(s.to_string());
        off += len;
    }

    let body_b64 = base64::engine::general_purpose::STANDARD.encode(data);

    let mut map = Map::new();
    map.insert("key".to_string(), Value::from(0u64));
    map.insert("string_key".to_string(), Value::from(""));
    map.insert("count".to_string(), Value::from(count as u64));
    map.insert("strings".to_string(),
        Value::Array(strings.into_iter().map(Value::from).collect()));
    map.insert("body_b64".to_string(), Value::from(body_b64));
    map.insert("body_len".to_string(), Value::from(data.len() as u64));
    map.insert("parsed_offset_end".to_string(), Value::from(off as u64));
    Ok(Value::Object(map))
}

pub fn serialize_binarystring_from_json(value: &Value) -> io::Result<Vec<u8>> {
    let map = value.as_object().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData, ".binarystring serialize: expected object root"))?;

    // Prefer round-trip via body_b64 when present (byte-perfect).
    if let Some(Value::String(b)) = map.get("body_b64") {
        if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b) {
            return Ok(bytes);
        }
    }

    // Reconstruction path: rebuild from `strings` array.
    let strings: Vec<&str> = map.get("strings")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    if strings.len() > u16::MAX as usize {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!(".binarystring serialize: too many strings ({} > 65535)", strings.len())));
    }

    let mut out = Vec::new();
    out.extend_from_slice(&(strings.len() as u16).to_le_bytes());
    for (i, s) in strings.iter().enumerate() {
        if s.len() > u8::MAX as usize {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!(".binarystring serialize: string {} too long ({} > 255)", i, s.len())));
        }
        out.push(s.len() as u8);
        out.extend_from_slice(s.as_bytes());
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sample() -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&3u16.to_le_bytes());           // count
        for s in ["RemoteCatchPickup", "alpha", "beta"] {
            v.push(s.len() as u8);
            v.extend_from_slice(s.as_bytes());
        }
        v
    }

    #[test]
    fn parses_count_and_strings() {
        let bytes = make_sample();
        let v = parse_binarystring_to_json(&bytes).expect("parse");
        let m = v.as_object().unwrap();
        assert_eq!(m["count"], Value::from(3u64));
        let strs: Vec<&str> = m["strings"].as_array().unwrap()
            .iter().map(|s| s.as_str().unwrap()).collect();
        assert_eq!(strs, vec!["RemoteCatchPickup", "alpha", "beta"]);
        assert_eq!(m["parsed_offset_end"], Value::from(bytes.len() as u64));
    }

    #[test]
    fn round_trip_byte_perfect() {
        let original = make_sample();
        let parsed = parse_binarystring_to_json(&original).expect("parse");
        let written = serialize_binarystring_from_json(&parsed).expect("write");
        assert_eq!(written, original);
    }

    #[test]
    fn reconstruct_from_strings_when_no_body() {
        let mut m = Map::new();
        m.insert("strings".to_string(),
            Value::Array(vec![Value::from("hello"), Value::from("world")]));
        let v = Value::Object(m);
        let written = serialize_binarystring_from_json(&v).expect("write");
        // count=2 + (1+5) + (1+5) = 14
        assert_eq!(written.len(), 14);
        let reparsed = parse_binarystring_to_json(&written).expect("parse");
        let strs: Vec<&str> = reparsed["strings"].as_array().unwrap()
            .iter().map(|s| s.as_str().unwrap()).collect();
        assert_eq!(strs, vec!["hello", "world"]);
    }

    #[test]
    fn rejects_too_short() {
        assert!(parse_binarystring_to_json(&[0u8; 1]).is_err());
    }

    #[test]
    fn rejects_truncated_string() {
        let mut bytes = vec![1u8, 0]; // count = 1
        bytes.push(50);  // length 50
        bytes.extend_from_slice(b"only_three"); // only 10 bytes — truncated
        assert!(parse_binarystring_to_json(&bytes).is_err());
    }
}
