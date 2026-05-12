// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.paccd` — **PA Character Customization Data**.
//!
//! Identified iter 8 via sample `defaultcustomization.paccd`. The file
//! stores character customization parameters (skin tone, hair, body
//! sliders, etc.) as packed bytes where common values are 0, 50 (0x32),
//! and 100 (0x64) — slider midpoints/maxes.
//!
//! Header (decoded iter 12 against the **full 1641-file corpus**):
//! ```text
//! offset  size  field
//! 0       4     u32 zero_marker  (always 0)
//! 4       4     u32 format_version  (always 14 — entire corpus)
//! 8       4     u32 flags  (always 2 — entire corpus)
//! 12      ...   packed customization-slider bytes
//! ```
//!
//! Body-byte distribution (1641 files, bytes from offset 12):
//!   - `0xFF` = 51% — **"no-override" sentinel** (slider not customized)
//!   - `0x00` = 20% — explicit zero
//!   - `0x64` (= 100) = 8% — slider max
//!   - `0x32` (= 50) = 6% — slider midpoint
//!   - `0x7d`, `0x63`, `0x01`..`0x04` — other slider values + bitfield bits
//!
//! **B3 (iter 9 of T0 verification): slider semantic mapping**
//! IDA evidence: the CharacterCustomizationData class (`.?AVCharacter
//! CustomizationData@pa@@` at 0x145c41598) has 3 schema fields
//! discovered at 0x144963fe0+:
//!   - `_customizationFileName` (0x144963fe0)
//!   - `_decorationParamFileName` (0x144964020)
//!   - `_meshParamFileName` (0x144964040)
//!
//! The .paccd file is a **TOP-LEVEL CONTAINER** that references
//! `.meshparam` and `.decorationparam` files where actual slider
//! data lives. The body bytes (post 12-byte header) are likely an
//! INDIRECTION INDEX into those referenced files — not direct slider
//! names. So `.paccd[byte i] = 100` means "slider i in the referenced
//! mesh/decoration param file is set to value 100".
//!
//! Per-byte → per-named-slider mapping needs cross-file decode of
//! the referenced .meshparam / .decorationparam formats (which are
//! not yet parsed by dmm-parser). This is a multi-file dependency
//! chain. Status: **per-slider mapping NOT FEASIBLE without decoding
//! the referenced parameter files first**. Logged as long-haul work.
//!
//! Size range across corpus: 298 to 3370 bytes (avg 434).
//!
//! Round-trip is byte-perfect via `body_b64`. Per-slider semantic
//! decode (which byte = which slider) requires IDA RE of the
//! character editor load path; queued.

use std::io::{self};

use base64::Engine;
use serde_json::{Map, Value};

pub const HEADER_SIZE: usize = 12;

pub fn parse_paccd_to_json(data: &[u8]) -> io::Result<Value> {
    if data.len() < HEADER_SIZE {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!(".paccd too short: {} < {}", data.len(), HEADER_SIZE)));
    }
    let zero_marker = u32::from_le_bytes(data[0..4].try_into().unwrap());
    if zero_marker != 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!(".paccd expected zero u32 at offset 0, got 0x{:08x}", zero_marker)));
    }
    let format_version = u32::from_le_bytes(data[4..8].try_into().unwrap());
    let flags = u32::from_le_bytes(data[8..12].try_into().unwrap());

    // Count the no-override bytes in the body for the report.
    let body_start = 12;
    let no_override_count = data[body_start..].iter().filter(|&&b| b == 0xFF).count();

    let body_b64 = base64::engine::general_purpose::STANDARD.encode(data);

    let mut map = Map::new();
    map.insert("key".to_string(), Value::from(0u64));
    map.insert("string_key".to_string(), Value::from(""));
    map.insert("zero_marker".to_string(), Value::from(zero_marker as u64));
    map.insert("format_version".to_string(), Value::from(format_version as u64));
    map.insert("flags".to_string(), Value::from(flags as u64));
    map.insert("no_override_byte_count".to_string(),
        Value::from(no_override_count as u64));
    map.insert("body_b64".to_string(), Value::from(body_b64));
    map.insert("body_len".to_string(), Value::from(data.len() as u64));
    Ok(Value::Object(map))
}

pub fn serialize_paccd_from_json(value: &Value) -> io::Result<Vec<u8>> {
    let map = value.as_object().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData, ".paccd serialize: expected object root"))?;
    let body_b64 = map.get("body_b64")
        .and_then(|v| v.as_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            ".paccd serialize: missing 'body_b64'"))?;
    base64::engine::general_purpose::STANDARD.decode(body_b64).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData,
            format!(".paccd serialize: body_b64 decode failed: {}", e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_sample() -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&0u32.to_le_bytes());           // zero_marker
        v.extend_from_slice(&14u32.to_le_bytes());          // format_version
        v.extend_from_slice(&2u32.to_le_bytes());           // flags
        // customization params (typical distribution: 0xff sentinels, sliders)
        v.extend_from_slice(&[0xff, 0xff, 100, 100, 0xff, 50, 50, 50, 0, 100]);
        v
    }

    #[test]
    fn parse_extracts_header() {
        let bytes = make_sample();
        let v = parse_paccd_to_json(&bytes).expect("parse");
        let m = v.as_object().unwrap();
        assert_eq!(m["zero_marker"], Value::from(0u64));
        assert_eq!(m["format_version"], Value::from(14u64));
        assert_eq!(m["flags"], Value::from(2u64));
    }

    #[test]
    fn counts_no_override_sentinels() {
        let bytes = make_sample();
        let v = parse_paccd_to_json(&bytes).expect("parse");
        // Body has 3 × 0xff (the sentinel)
        assert_eq!(v["no_override_byte_count"], Value::from(3u64));
    }

    #[test]
    fn round_trip_byte_perfect() {
        let original = make_sample();
        let parsed = parse_paccd_to_json(&original).expect("parse");
        let written = serialize_paccd_from_json(&parsed).expect("write");
        assert_eq!(written, original);
    }

    #[test]
    fn rejects_nonzero_marker() {
        let mut bytes = make_sample();
        bytes[0] = 0xFF;
        assert!(parse_paccd_to_json(&bytes).is_err());
    }

    #[test]
    fn rejects_too_short() {
        assert!(parse_paccd_to_json(&[0u8; 8]).is_err());
    }
}
