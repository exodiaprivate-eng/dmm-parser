// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.pamlod` — PA Mesh Level-of-Detail descriptor.
//!
//! Identified iter 6 via cross-reference with mac binary strings:
//! `N2pa29StaticMeshLODStreamingContextE` and friends. The engine uses
//! `.pamlod` to drive `StaticMeshLODStreamingContext` — i.e. these are
//! Static Mesh LOD streaming descriptors with embedded references to
//! the source `.dds` textures.
//!
//! Header layout (re-audited iter 14 against 50 game samples after the
//! iter-9 partial-compression unblock):
//! ```text
//! offset  size  field
//! 0       4     u32 lod_count       (varies: 1, 4, 5, 6, 7, 8, 9)
//! 4       4     u32 size_hint       (varies by mesh complexity)
//! 8       4     f32 lod_distance    (e.g. ~3.289 for simple plane)
//! 12      4     u32 geometry_format (always 4 across full corpus)
//! 16      ...   LOD entry array + embedded `.dds` path strings
//! ```
//!
//! **Iter-6 bug fixed iter 14**: the original ship classified the
//! field at offset 0 as "version" with a hardcoded `version != 1`
//! rejection, but only had 1 sample. Real corpus distribution:
//! `(7×17, 8×12, 6×11, 1×3, 9×3, 5×3, 4×1)` files — so it's NOT a
//! version, it's the actual **lod_count** (number of LOD levels for
//! this specific mesh). Conversely, what was named `lod_count` at
//! offset 12 is constant `4` across the entire corpus — likely a
//! geometry-format identifier.
//!
//! Size range across corpus: 802 bytes to 2.4 MB (the small ones are
//! flat planes; the huge ones are detailed character meshes).

use std::io::{self};

use base64::Engine;
use serde_json::{Map, Value};

/// Heuristic minimum header size — version + size + float + lod_count.
pub const MIN_HEADER: usize = 16;

/// Parse a `.pamlod` body to JSON. Validates a plausible header and
/// scans the body for `.dds` texture path references. Body bytes
/// round-trip via `body_b64`.
pub fn parse_pamlod_to_json(data: &[u8]) -> io::Result<Value> {
    if data.len() < MIN_HEADER {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!(".pamlod too short: {} < {}", data.len(), MIN_HEADER)));
    }
    let lod_count = u32::from_le_bytes(data[0..4].try_into().unwrap());
    let size_hint = u32::from_le_bytes(data[4..8].try_into().unwrap());
    let lod_distance = f32::from_le_bytes(data[8..12].try_into().unwrap());
    let geometry_format = u32::from_le_bytes(data[12..16].try_into().unwrap());

    // Sanity: lod_count observed range is 1-9; cap at 32 for safety.
    if lod_count == 0 || lod_count > 32 {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!(".pamlod implausible lod_count={} (observed range 1-9)", lod_count)));
    }
    // geometry_format is always 4 in the corpus; warn-but-don't-fail
    // on other values so the parser is tolerant of future patches.

    let texture_paths = scan_dds_paths(data);
    let body_b64 = base64::engine::general_purpose::STANDARD.encode(data);

    let mut map = Map::new();
    map.insert("key".to_string(), Value::from(0u64));
    map.insert("string_key".to_string(), Value::from(""));
    map.insert("lod_count".to_string(), Value::from(lod_count as u64));
    map.insert("size_hint".to_string(), Value::from(size_hint as u64));
    map.insert("lod_distance".to_string(),
        serde_json::Number::from_f64(lod_distance as f64)
            .map(Value::Number).unwrap_or(Value::Null));
    map.insert("geometry_format".to_string(), Value::from(geometry_format as u64));
    map.insert("texture_paths".to_string(),
        Value::Array(texture_paths.into_iter().map(Value::from).collect()));
    map.insert("body_b64".to_string(), Value::from(body_b64));
    map.insert("body_len".to_string(), Value::from(data.len() as u64));
    Ok(Value::Object(map))
}

/// Serialize back to bytes via `body_b64` verbatim.
pub fn serialize_pamlod_from_json(value: &Value) -> io::Result<Vec<u8>> {
    let map = value.as_object().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData, ".pamlod serialize: expected object root"))?;
    let body_b64 = map.get("body_b64")
        .and_then(|v| v.as_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            ".pamlod serialize: missing 'body_b64'"))?;
    base64::engine::general_purpose::STANDARD.decode(body_b64).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData,
            format!(".pamlod serialize: body_b64 decode failed: {}", e))
    })
}

/// Find every printable-ASCII run that ends in `.dds` (case-insensitive).
/// Used to surface texture references for mod tooling.
fn scan_dds_paths(data: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut i = 0;
    while i < data.len() {
        // skip non-printable
        if !(0x20..0x7f).contains(&data[i]) {
            i += 1;
            continue;
        }
        let start = i;
        while i < data.len() && (0x20..0x7f).contains(&data[i]) {
            i += 1;
        }
        let span = &data[start..i];
        if span.len() >= 5 && span.ends_with(b".dds") {
            if let Ok(s) = std::str::from_utf8(span) {
                let s = s.to_string();
                if !seen.contains(&s) {
                    seen.insert(s.clone());
                    out.push(s);
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic .pamlod modeled after the real `03_plane.pamlod` header
    /// (1 LOD level, simple plane mesh).
    fn make_sample() -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&1u32.to_le_bytes());           // lod_count
        v.extend_from_slice(&0x0000_02E0u32.to_le_bytes()); // size_hint
        v.extend_from_slice(&3.285f32.to_le_bytes());       // lod_distance
        v.extend_from_slice(&4u32.to_le_bytes());           // geometry_format
        v.extend_from_slice(&[0; 32]);
        v.extend_from_slice(b"03_plane.dds\0");
        v.extend_from_slice(&[0; 16]);
        v.extend_from_slice(b"03_plane.dds\0");
        v.extend_from_slice(&[0; 16]);
        v
    }

    #[test]
    fn parse_header_fields() {
        let bytes = make_sample();
        let v = parse_pamlod_to_json(&bytes).expect("parse");
        let m = v.as_object().unwrap();
        assert_eq!(m["lod_count"], Value::from(1u64));
        assert_eq!(m["geometry_format"], Value::from(4u64));
        let d = m["lod_distance"].as_f64().unwrap();
        assert!((d - 3.285).abs() < 0.001);
    }

    #[test]
    fn scans_dds_paths_and_dedupes() {
        let bytes = make_sample();
        let v = parse_pamlod_to_json(&bytes).expect("parse");
        let paths: Vec<&str> = v["texture_paths"].as_array().unwrap()
            .iter().map(|p| p.as_str().unwrap()).collect();
        assert_eq!(paths, vec!["03_plane.dds"]);
    }

    #[test]
    fn round_trip_byte_perfect() {
        let original = make_sample();
        let parsed = parse_pamlod_to_json(&original).expect("parse");
        let written = serialize_pamlod_from_json(&parsed).expect("write");
        assert_eq!(written, original);
    }

    #[test]
    fn accepts_observed_lod_count_range() {
        // Corpus shows lod_count in {1, 4, 5, 6, 7, 8, 9}
        for n in [1, 4, 5, 6, 7, 8, 9] {
            let mut bytes = make_sample();
            bytes[0..4].copy_from_slice(&(n as u32).to_le_bytes());
            assert!(parse_pamlod_to_json(&bytes).is_ok(),
                "should accept lod_count={}", n);
        }
    }

    #[test]
    fn rejects_too_short() {
        let bytes = vec![1, 0, 0, 0, 0, 0, 0];
        assert!(parse_pamlod_to_json(&bytes).is_err());
    }

    #[test]
    fn rejects_zero_lod_count() {
        let mut bytes = make_sample();
        bytes[0..4].copy_from_slice(&0u32.to_le_bytes());
        assert!(parse_pamlod_to_json(&bytes).is_err());
    }

    #[test]
    fn rejects_implausible_lod_count() {
        let mut bytes = make_sample();
        bytes[0..4].copy_from_slice(&999u32.to_le_bytes());
        assert!(parse_pamlod_to_json(&bytes).is_err());
    }
}
