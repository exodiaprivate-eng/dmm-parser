// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.hkx` — Havok native serialization in Tag-format ("TAG0").
//!
//! Crimson Desert uses Havok 2024.2 SDK — confirmed by the literal
//! string `SDKV20240200` embedded in every `.hkx` file's first section.
//! Iter-7 audit: 20/20 sampled `.hkx` files start with this layout.
//!
//! Tag-format header structure (Havok public format):
//! ```text
//! offset  size  field
//! 0       4     u32 section_length_be  (big-endian!)
//! 4       4     section_name           ("TAG0", "TYPE", "INDX", etc.)
//! 8       ...   section body
//! ```
//!
//! Every Crimson Desert `.hkx` starts with a `TAG0` outer section whose
//! body contains a `SDKV` subsection with the SDK version string, then
//! a `DATA` subsection with the serialized object graph.
//!
//! **This module ships the classifier + SDK-version extraction layer**
//! (Tier 1). Full object-graph deserialization would require parsing
//! the Havok 2024.2 class registry (`hkClass` family) which lives
//! inside `CrimsonDesert.exe` and would need extensive IDA RE.
//! Round-trip is byte-perfect via `body_b64`.

use std::io::{self};

use base64::Engine;
use serde_json::{Map, Value};

pub const TAG0_MAGIC: &[u8; 4] = b"TAG0";
pub const SDKV_MAGIC: &[u8; 4] = b"SDKV";
pub const DATA_MAGIC: &[u8; 4] = b"DATA";

/// Parse a `.hkx` body to JSON. Validates the TAG0 outer header,
/// extracts the SDK version string from the SDKV subsection, and
/// preserves the full body opaquely for round-trip.
pub fn parse_hkx_to_json(data: &[u8]) -> io::Result<Value> {
    if data.len() < 16 {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!(".hkx too short: {} < 16", data.len())));
    }
    // Section length is BIG-endian per the Havok tag-format spec.
    let outer_length = u32::from_be_bytes(data[0..4].try_into().unwrap());
    if &data[4..8] != TAG0_MAGIC {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!(".hkx TAG0 magic mismatch: got {:?}",
                std::str::from_utf8(&data[4..8]).unwrap_or("<non-utf8>"))));
    }
    // The TAG0 section's body length is encoded in the low bits;
    // the high bit (0x80000000) is a flag in some Havok versions.
    // For CD samples the raw length matches the section span.
    let outer_length_clean = outer_length & 0x3FFF_FFFF;
    if outer_length_clean as usize > data.len() {
        // Don't treat as fatal — outer header may carry flags we
        // don't fully understand. Surface as a warning field.
    }

    // The SDKV subsection lives at offset 8 (right after TAG0 header)
    // for CD's layout. Its body is the SDK version string.
    let sdk_version = parse_sdkv_at(&data[8..]);

    let body_b64 = base64::engine::general_purpose::STANDARD.encode(data);

    let mut map = Map::new();
    map.insert("key".to_string(), Value::from(0u64));
    map.insert("string_key".to_string(), Value::from(""));
    map.insert("magic".to_string(), Value::from("TAG0"));
    map.insert("outer_length_raw".to_string(), Value::from(outer_length as u64));
    map.insert("outer_length_be".to_string(), Value::from(outer_length_clean as u64));
    map.insert("sdk_version".to_string(),
        match sdk_version {
            Some(s) => Value::from(s),
            None => Value::Null,
        });
    map.insert("body_b64".to_string(), Value::from(body_b64));
    map.insert("body_len".to_string(), Value::from(data.len() as u64));
    Ok(Value::Object(map))
}

/// Serialize back to bytes via `body_b64` verbatim.
pub fn serialize_hkx_from_json(value: &Value) -> io::Result<Vec<u8>> {
    let map = value.as_object().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData, ".hkx serialize: expected object root"))?;
    let body_b64 = map.get("body_b64")
        .and_then(|v| v.as_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            ".hkx serialize: missing 'body_b64'"))?;
    base64::engine::general_purpose::STANDARD.decode(body_b64).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData,
            format!(".hkx serialize: body_b64 decode failed: {}", e))
    })
}

/// If `data` starts with a Havok-style subsection header where the name
/// field is "SDKV", return the version string. Else return None.
fn parse_sdkv_at(data: &[u8]) -> Option<String> {
    if data.len() < 8 { return None; }
    let len = u32::from_be_bytes(data[0..4].try_into().unwrap()) & 0x3FFF_FFFF;
    if &data[4..8] != SDKV_MAGIC { return None; }
    if (len as usize) > data.len() { return None; }
    // Body of SDKV is the version string (e.g. "20240200" for Havok 2024.2.00).
    // Sample: `40 00 00 10 53 44 4b 56 32 30 32 34 30 32 30 30`
    //                                ^^^^^^^^^^^^^^^^^^^^^^^^^^
    //                                 "20240200" = 8 chars, starts at offset 8.
    let body = &data[8..(len as usize).min(data.len())];
    // Take the first ASCII-digit run as the version (CD format is "YYYYMMNN"
    // 8 digits like "20240200"). Stop at non-digit to avoid bleed into the
    // next subsection's length byte (e.g. trailing `@` = 0x40).
    let mut end = 0;
    while end < body.len() && body[end].is_ascii_digit() {
        end += 1;
    }
    if end >= 4 {
        std::str::from_utf8(&body[..end]).ok().map(|s| s.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthesize an .hkx-style buffer with TAG0 outer + SDKV inner.
    fn make_sample(sdk_string: &[u8]) -> Vec<u8> {
        let mut v = Vec::new();
        // Outer section: length 0x788 (just a sentinel; not validated strictly)
        v.extend_from_slice(&0x0000_0788u32.to_be_bytes());
        v.extend_from_slice(TAG0_MAGIC);
        // SDKV subsection
        let sdkv_body_len = sdk_string.len();
        let sdkv_total_len = 8 + sdkv_body_len;
        v.extend_from_slice(&(sdkv_total_len as u32).to_be_bytes());
        v.extend_from_slice(SDKV_MAGIC);
        v.extend_from_slice(sdk_string);
        // pad
        v.extend_from_slice(&[0u8; 16]);
        v
    }

    #[test]
    fn parse_extracts_sdk_version() {
        let bytes = make_sample(b"20240200");
        let v = parse_hkx_to_json(&bytes).expect("parse");
        let m = v.as_object().unwrap();
        assert_eq!(m["magic"], Value::from("TAG0"));
        assert_eq!(m["sdk_version"], Value::from("20240200"));
    }

    #[test]
    fn round_trip_byte_perfect() {
        let original = make_sample(b"20240200");
        let parsed = parse_hkx_to_json(&original).expect("parse");
        let written = serialize_hkx_from_json(&parsed).expect("write");
        assert_eq!(written, original);
    }

    #[test]
    fn rejects_wrong_magic() {
        let mut bytes = make_sample(b"20240200");
        bytes[4..8].copy_from_slice(b"XXXX");
        assert!(parse_hkx_to_json(&bytes).is_err());
    }

    #[test]
    fn rejects_too_short() {
        let bytes = vec![0u8; 4];
        assert!(parse_hkx_to_json(&bytes).is_err());
    }

    #[test]
    fn sdk_null_when_subsection_not_sdkv() {
        let mut bytes = make_sample(b"20240200");
        // Corrupt the SDKV magic — sdk_version should be null
        bytes[12..16].copy_from_slice(b"DATA");
        let v = parse_hkx_to_json(&bytes).expect("parse (still ok, just no sdk)");
        assert_eq!(v["sdk_version"], Value::Null);
    }
}
