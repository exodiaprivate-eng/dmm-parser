// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Generic `u32 count + records` table classifier (iter 22).
//!
//! Several PA binary formats follow the same outer shape:
//!   `u32 record_count + (per-record bytes with embedded name + values)`
//!
//! The per-record byte layout differs per format (and each format's
//! per-record decode needs its own IDA RE), but the COMMON structure
//! is: a header u32 count, followed by length-prefixed name strings
//! scattered throughout the body alongside binary fields.
//!
//! This module ships a **generic partial classifier** that:
//! 1. Reads the u32 count header
//! 2. Walks the body looking for plausible u32-length-prefixed UTF-8
//!    strings (the record names) — same heuristic as
//!    `motionblending.scan_length_prefixed_strings`
//! 3. Returns the count + the scanned name list + body_b64
//!
//! Verified iter 22 to work on:
//!   - `.paseqh` (Sequencer stage header)
//!   - `.paschedulectx` (Schedule context)
//!   - `.paproj` (Projectile data)
//!
//! Tier 1.5: classification + name extraction + byte-perfect
//! round-trip. Typed value decode is per-format work, TBD.

use std::io::{self};

use base64::Engine;
use serde_json::{Map, Value};

/// Parse a `u32 count + records` body. The record content is not decoded
/// per-format — only the outer count + a flat scan of embedded names.
pub fn parse_count_record_table_to_json(data: &[u8]) -> io::Result<Value> {
    if data.len() < 4 {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!("count_record_table too short: {} < 4", data.len())));
    }
    let count = u32::from_le_bytes(data[0..4].try_into().unwrap());

    // Cap to a sensible max so absurd "count" values from non-matching
    // formats fail loudly rather than crashing.
    if count > 1_000_000 {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!("count_record_table implausible count {} > 1M", count)));
    }

    let scanned_names = scan_length_prefixed_strings(&data[4..]);
    let body_b64 = base64::engine::general_purpose::STANDARD.encode(data);

    let mut map = Map::new();
    map.insert("key".to_string(), Value::from(0u64));
    map.insert("string_key".to_string(), Value::from(""));
    map.insert("record_count".to_string(), Value::from(count as u64));
    map.insert("scanned_name_count".to_string(),
        Value::from(scanned_names.len() as u64));
    map.insert("scanned_names".to_string(),
        Value::Array(scanned_names.into_iter().map(Value::from).collect()));
    map.insert("body_b64".to_string(), Value::from(body_b64));
    map.insert("body_len".to_string(), Value::from(data.len() as u64));
    Ok(Value::Object(map))
}

pub fn serialize_count_record_table_from_json(value: &Value) -> io::Result<Vec<u8>> {
    let map = value.as_object().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData, "count_record_table serialize: expected object root"))?;
    let body_b64 = map.get("body_b64")
        .and_then(|v| v.as_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            "count_record_table serialize: missing 'body_b64'"))?;
    base64::engine::general_purpose::STANDARD.decode(body_b64).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData,
            format!("count_record_table serialize: body_b64 decode failed: {}", e))
    })
}

/// Walk `data` looking for u32-length-prefixed printable-ASCII strings
/// of length >= 4. Same algorithm as the motionblending scanner.
fn scan_length_prefixed_strings(data: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut i = 0;
    while i + 4 < data.len() {
        let len = u32::from_le_bytes(data[i..i + 4].try_into().unwrap()) as usize;
        if (4..=512).contains(&len) && i + 4 + len <= data.len() {
            let span = &data[i + 4..i + 4 + len];
            if span.iter().all(|&b| (0x20..0x7f).contains(&b) || b == 0) {
                if let Ok(s) = std::str::from_utf8(span) {
                    let trimmed = s.trim_end_matches('\u{0}').to_string();
                    if trimmed.len() >= 3 && !seen.contains(&trimmed) {
                        seen.insert(trimmed.clone());
                        out.push(trimmed);
                    }
                    i += 4 + len;
                    continue;
                }
            }
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sample() -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&3u32.to_le_bytes());  // count
        // synthetic records with length-prefixed names interleaved with junk
        for name in ["sequencer_one", "sequencer_two", "schedule_three"] {
            // junk before
            v.extend_from_slice(&[0xAB, 0xCD, 0xEF, 0xFE]);
            // length-prefixed name
            v.extend_from_slice(&(name.len() as u32).to_le_bytes());
            v.extend_from_slice(name.as_bytes());
            // junk after
            v.extend_from_slice(&[0; 16]);
        }
        v
    }

    #[test]
    fn parses_count_and_scans_names() {
        let bytes = make_sample();
        let v = parse_count_record_table_to_json(&bytes).expect("parse");
        let m = v.as_object().unwrap();
        assert_eq!(m["record_count"], Value::from(3u64));
        let names: Vec<&str> = m["scanned_names"].as_array().unwrap()
            .iter().map(|n| n.as_str().unwrap()).collect();
        assert!(names.contains(&"sequencer_one"), "got: {:?}", names);
        assert!(names.contains(&"schedule_three"));
    }

    #[test]
    fn round_trip_byte_perfect() {
        let original = make_sample();
        let v = parse_count_record_table_to_json(&original).expect("parse");
        let written = serialize_count_record_table_from_json(&v).expect("write");
        assert_eq!(written, original);
    }

    #[test]
    fn rejects_too_short() {
        assert!(parse_count_record_table_to_json(&[0u8; 3]).is_err());
    }

    #[test]
    fn rejects_implausible_count() {
        let mut bytes = vec![0u8; 16];
        // count = u32::MAX
        bytes[0..4].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(parse_count_record_table_to_json(&bytes).is_err());
    }
}
