// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.paasmt` — **PA Animation Set Matching Table**.
//!
//! Identified iter 8 from the only sample in the 1.06 install:
//! `animationsetmatchingtable.paasmt` (10140 bytes). The file maps
//! character model `.pac` files to their corresponding animation
//! set descriptor `.animset.xml` files. Used by the engine to look
//! up "which animation set should this skinned mesh play".
//!
//! Format (verified against the one real game sample, 10140 bytes):
//! ```text
//! offset  size  field
//! 0       4     u32 record_count   (sample = 0x0000003A = 58 records)
//! 4       ...   length-prefixed path records:
//!               { u32 len, [u8; len] utf8_path }, repeating to EOF
//! ```
//!
//! Path records alternate model-and-animset semantics in pairs:
//!   pair[0] = `character/model/.../X.pac`
//!   pair[1] = `character/descriptors/animationset/.../X.animset.xml`
//!
//! Round-trip is byte-perfect via `body_b64`. Convenience fields:
//!   - `record_count`: u32 header value (count of path records following)
//!   - `paths`: ordered list of every path string in the file

use std::io::{self};

use base64::Engine;
use serde_json::{Map, Value};

pub const HEADER_SIZE: usize = 4;

pub fn parse_paasmt_to_json(data: &[u8]) -> io::Result<Value> {
    if data.len() < HEADER_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!(".paasmt too short: {} < {}", data.len(), HEADER_SIZE)));
    }
    let record_count = u32::from_le_bytes(data[0..4].try_into().unwrap());

    // Walk length-prefixed records from offset 4 to EOF.
    let mut paths = Vec::new();
    let mut off = HEADER_SIZE;
    while off + 4 <= data.len() {
        let len = u32::from_le_bytes(data[off..off + 4].try_into().unwrap()) as usize;
        if len == 0 || len > 4096 {
            // implausible length — abort path scan but keep what we have
            break;
        }
        if off + 4 + len > data.len() {
            break;
        }
        let span = &data[off + 4..off + 4 + len];
        match std::str::from_utf8(span) {
            Ok(s) => paths.push(s.to_string()),
            Err(_) => break,
        }
        off += 4 + len;
    }

    let body_b64 = base64::engine::general_purpose::STANDARD.encode(data);

    // Iter 13: group flat paths into structured pairs. Each record =
    // (model_path, animset_xml_path) per the iter-8 observation. The
    // path list always comes in pairs (2 × record_count entries).
    // PA stores paths with a trailing null byte in the bytes (length
    // includes the null) — strip it for the structured view, but the
    // flat `paths` list preserves the raw value for byte-perfect
    // round-trip semantics.
    let mut record_pairs = Vec::new();
    for chunk in paths.chunks(2) {
        if chunk.len() == 2 {
            let model = chunk[0].trim_end_matches('\u{0}').to_string();
            let animset = chunk[1].trim_end_matches('\u{0}').to_string();
            record_pairs.push((model, animset));
        }
    }

    let mut map = Map::new();
    map.insert("key".to_string(), Value::from(0u64));
    map.insert("string_key".to_string(), Value::from(""));
    map.insert("record_count".to_string(), Value::from(record_count as u64));
    map.insert("path_count".to_string(), Value::from(paths.len() as u64));
    map.insert("record_pairs".to_string(),
        Value::Array(record_pairs.into_iter().map(|(m, a)| {
            let mut obj = Map::new();
            obj.insert("model_path".to_string(), Value::from(m));
            obj.insert("animset_xml_path".to_string(), Value::from(a));
            Value::Object(obj)
        }).collect()));
    map.insert("paths".to_string(),
        Value::Array(paths.into_iter().map(Value::from).collect()));
    map.insert("body_b64".to_string(), Value::from(body_b64));
    map.insert("body_len".to_string(), Value::from(data.len() as u64));
    Ok(Value::Object(map))
}

pub fn serialize_paasmt_from_json(value: &Value) -> io::Result<Vec<u8>> {
    let map = value.as_object().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData, ".paasmt serialize: expected object root"))?;
    let body_b64 = map.get("body_b64")
        .and_then(|v| v.as_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            ".paasmt serialize: missing 'body_b64'"))?;
    base64::engine::general_purpose::STANDARD.decode(body_b64).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData,
            format!(".paasmt serialize: body_b64 decode failed: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sample() -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&2u32.to_le_bytes()); // record_count
        let p1 = b"character/model/1_pc/x.pac";
        v.extend_from_slice(&(p1.len() as u32).to_le_bytes());
        v.extend_from_slice(p1);
        let p2 = b"character/descriptors/animationset/x.animset.xml";
        v.extend_from_slice(&(p2.len() as u32).to_le_bytes());
        v.extend_from_slice(p2);
        v
    }

    #[test]
    fn parses_header_and_paths() {
        let bytes = make_sample();
        let v = parse_paasmt_to_json(&bytes).expect("parse");
        let m = v.as_object().unwrap();
        assert_eq!(m["record_count"], Value::from(2u64));
        let paths: Vec<&str> = m["paths"].as_array().unwrap()
            .iter().map(|p| p.as_str().unwrap()).collect();
        assert_eq!(paths.len(), 2);
        assert!(paths[0].ends_with(".pac"));
        assert!(paths[1].ends_with(".animset.xml"));
    }

    #[test]
    fn groups_paths_into_structured_pairs() {
        let bytes = make_sample();
        let v = parse_paasmt_to_json(&bytes).expect("parse");
        let pairs = v["record_pairs"].as_array().unwrap();
        assert_eq!(pairs.len(), 1, "1 pair from 2 paths");
        let p = &pairs[0];
        assert!(p["model_path"].as_str().unwrap().ends_with(".pac"));
        assert!(p["animset_xml_path"].as_str().unwrap().ends_with(".animset.xml"));
    }

    #[test]
    fn round_trip_byte_perfect() {
        let original = make_sample();
        let parsed = parse_paasmt_to_json(&original).expect("parse");
        let written = serialize_paasmt_from_json(&parsed).expect("write");
        assert_eq!(written, original);
    }

    #[test]
    fn rejects_too_short() {
        assert!(parse_paasmt_to_json(&[0u8; 2]).is_err());
    }

    #[test]
    fn tolerates_garbage_tail() {
        // Real-world files might have padding/EOF junk after the last record.
        let mut bytes = make_sample();
        bytes.extend_from_slice(&[0xff; 8]);
        let v = parse_paasmt_to_json(&bytes).expect("parse");
        let paths_len = v["paths"].as_array().unwrap().len();
        assert!(paths_len >= 2);
    }
}
