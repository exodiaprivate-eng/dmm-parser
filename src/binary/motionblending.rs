// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.motionblending` — named-property binary record format.
//!
//! Format decoded iter 5 against 30 real game samples (15 v3 + 15 v4).
//! Two version layouts — v4 adds 8 bytes of u64 reserved padding before
//! the flag block:
//!
//! ```text
//! VERSION 3 (12 bytes before type_name):
//!   offset  size  field
//!   0       2     u16 magic_a    (0xFFFF)
//!   2       2     u16 version    (0x0003)
//!   4       2     u16 reserved   (typically 0)
//!   6       4     u32 flags_a    (e.g. 0x0000000E)
//!   10      2     u16 flags_b    (e.g. 0x0006)
//!   12      4     u32 type_name_len
//!   16      N     type_name_utf8 ("ParameterizedMotionSpace")
//!
//! VERSION 4 (20 bytes before type_name):
//!   offset  size  field
//!   0       2     u16 magic_a    (0xFFFF)
//!   2       2     u16 version    (0x0004)
//!   4       8     u64 reserved   (always 0)
//!   12      4     u32 flags_a    (e.g. 0x000F0000)
//!   16      4     u32 flags_b    (e.g. 0x00040000)
//!   20      4     u32 type_name_len
//!   24      N     type_name_utf8
//! ```
//!
//! Each property record after the type name follows:
//!   - `u16 prefix` (typically 0x0011 — field count or similar marker)
//!   - `u32 name_len` + UTF-8 name bytes (e.g. `_skeletonFileName`)
//!   - `u32 type_len` + UTF-8 type bytes (e.g. `staticstringA`)
//!   - typed value bytes (depends on type)
//!
//! **Vocabulary (iter 11 audit across all 1574 .motionblending files):**
//!
//! Root type: `ParameterizedMotionSpace` (every file)
//!
//! Type tags observed (only 2 in use):
//!   - `staticstringA` — static string array (4722 occurrences corpus-wide)
//!   - `bool` — boolean (4764 occurrences)
//!
//! 15 stable named fields (each appears once per file, in this order):
//!   1. `_skeletonFileName`
//!   2. `_animationFileNames`
//!   3. `_motionPhaseType`
//!   4. `_isLoopMotionBlending`
//!   5. `_numPhases`
//!   6. `_animationScale`
//!   7. `_dimensions`
//!   8. `_thirdDimensionSplitInfo`
//!   9. `_parameterMinMax`
//!   10. `_keepInitialBlendWeights`
//!   11. `_weightSmoothingMinSpeed`
//!   12. `_weightSmoothingMaxSpeed`
//!   13. `_phaseInfo`
//!   14. `_motionExamples`
//!   15. `_delaunayTriangles`
//!
//! This module ships the **classify + round-trip + type/field-name
//! extraction** layer (Tier 1). Typed value decoding for the property
//! records is queued for a future iter once the type-tag vocabulary
//! is mapped (currently only `staticstringA` is observed).
//!
//! Round-trip is byte-perfect via the `body_b64` field.

use std::io::{self, Write};

use base64::Engine;
use serde_json::{Map, Value};

pub const MAGIC_A: u16 = 0xFFFF;

/// Parse a `.motionblending` body to JSON, extracting the recognized
/// header fields + scanning the body for length-prefixed UTF-8 strings
/// (type name, field names). Body bytes round-trip via `body_b64`.
pub fn parse_motionblending_to_json(data: &[u8]) -> io::Result<Value> {
    if data.len() < 16 {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!(".motionblending too short: {} < 16", data.len())));
    }
    let magic_a = u16::from_le_bytes(data[0..2].try_into().unwrap());
    if magic_a != MAGIC_A {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!(".motionblending magic_a mismatch: got 0x{:04x}, want 0x{:04x}",
                magic_a, MAGIC_A)));
    }
    let version = u16::from_le_bytes(data[2..4].try_into().unwrap());

    // Version-dependent header layout (v3 = 16-byte header, v4 = 24-byte
    // header). Both have magic+version at offset 0..4, type_name_len at
    // header_size - 4, and type_name immediately after.
    let (header_size, reserved, flags_a, flags_b) = match version {
        3 => {
            if data.len() < 16 {
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                    "v3 header needs >= 16 bytes"));
            }
            let reserved = u16::from_le_bytes(data[4..6].try_into().unwrap()) as u64;
            let flags_a = u32::from_le_bytes(data[6..10].try_into().unwrap());
            let flags_b = u16::from_le_bytes(data[10..12].try_into().unwrap()) as u32;
            (16usize, reserved, flags_a, flags_b)
        }
        4 => {
            if data.len() < 24 {
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                    "v4 header needs >= 24 bytes"));
            }
            let reserved = u64::from_le_bytes(data[4..12].try_into().unwrap());
            let flags_a = u32::from_le_bytes(data[12..16].try_into().unwrap());
            let flags_b = u32::from_le_bytes(data[16..20].try_into().unwrap());
            (24usize, reserved, flags_a, flags_b)
        }
        v => {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!(".motionblending unsupported version {} (known: 3, 4)", v)));
        }
    };

    let type_name_len = u32::from_le_bytes(
        data[header_size - 4..header_size].try_into().unwrap()) as usize;
    if header_size + type_name_len > data.len() {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!(".motionblending type_name_len={} overflows body ({}) at v{}",
                type_name_len, data.len(), version)));
    }
    let type_name = std::str::from_utf8(
        &data[header_size..header_size + type_name_len]).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData,
            format!(".motionblending type_name not utf-8: {}", e))
    })?.to_string();

    let strings = scan_length_prefixed_strings(&data[header_size + type_name_len..]);
    // Iter 11: pair every `_`-prefixed field name with its following type
    // tag to give mod authors a structured (name, type) view. The body
    // walks records as: field_name (starts with `_`) → type_tag (one of
    // staticstringA/bool/staticfloat/etc.) → value_bytes. Value bytes
    // are skipped here — typed-value decode needs per-tag handlers TBD.
    let field_records = extract_field_records(&strings);
    // Iter 28: classify the remaining scanned strings into asset paths
    // (`*.pab`, `*.paa*`, etc.) vs other strings (parameter names, etc.).
    // The schema section at the top of the file declares field names +
    // type tags; the values section at the bottom contains the actual
    // referenced asset paths. These splits give mod authors direct
    // access to the file references without manual string-fishing.
    let (referenced_paths, value_strings) = split_value_strings(&strings, &field_records);

    let body_b64 = base64::engine::general_purpose::STANDARD.encode(data);

    let mut map = Map::new();
    map.insert("key".to_string(), Value::from(0u64));
    map.insert("string_key".to_string(), Value::from(""));
    map.insert("magic_a".to_string(), Value::from(format!("0x{:04x}", magic_a)));
    map.insert("version".to_string(), Value::from(version as u64));
    map.insert("reserved".to_string(), Value::from(reserved));
    map.insert("flags_a".to_string(), Value::from(flags_a as u64));
    map.insert("flags_b".to_string(), Value::from(flags_b as u64));
    map.insert("type_name".to_string(), Value::from(type_name));
    map.insert("scanned_strings".to_string(),
        Value::Array(strings.into_iter().map(Value::from).collect()));
    // Structured (name, type) pairs for mod-author convenience.
    map.insert("field_records".to_string(),
        Value::Array(field_records.into_iter().map(|(n, t)| {
            let mut o = Map::new();
            o.insert("name".to_string(), Value::from(n));
            o.insert("type_tag".to_string(), Value::from(t));
            Value::Object(o)
        }).collect()));
    // Iter 28: surface the referenced asset paths (the actual animation
    // + skeleton file references that the engine loads at runtime) and
    // other value strings (parameter names like `LeftHandAimYaw`).
    map.insert("referenced_paths".to_string(),
        Value::Array(referenced_paths.into_iter().map(Value::from).collect()));
    map.insert("value_strings".to_string(),
        Value::Array(value_strings.into_iter().map(Value::from).collect()));
    map.insert("body_b64".to_string(), Value::from(body_b64));
    map.insert("body_len".to_string(), Value::from(data.len() as u64));
    Ok(Value::Object(map))
}

/// Serialize a `.motionblending` JSON dict back to bytes. Uses
/// `body_b64` verbatim — this is the safe path. (Header field
/// reconstruction is not implemented because the property-record
/// section's typed values aren't fully decoded yet; modifying header
/// fields without rewriting the record section could break parsing.)
pub fn serialize_motionblending_from_json(value: &Value) -> io::Result<Vec<u8>> {
    let map = value.as_object().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData, ".motionblending serialize: expected object root"))?;
    let body_b64 = map.get("body_b64")
        .and_then(|v| v.as_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            ".motionblending serialize: missing 'body_b64'"))?;
    base64::engine::general_purpose::STANDARD.decode(body_b64).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData,
            format!(".motionblending serialize: body_b64 decode failed: {}", e))
    })
}

/// Split the post-schema string list into:
/// - `referenced_paths`: file references (extensions: `.pab`, `.pa<X>`,
///   `.paaF`, `.paaG`, etc. — the engine's asset references)
/// - `value_strings`: everything else not already in `field_records`
///   (parameter names, identifiers, etc.)
///
/// The schema section (field names + type tags) is excluded from both —
/// those live in `field_records`.
fn split_value_strings(
    all_strings: &[String],
    field_records: &[(String, String)],
) -> (Vec<String>, Vec<String>) {
    // Strings already exposed via field_records (both name + type tag)
    let mut schema_set: std::collections::HashSet<&str> = std::collections::HashSet::new();
    for (n, t) in field_records {
        schema_set.insert(n.as_str());
        schema_set.insert(t.as_str());
    }
    // Also exclude the root type_name and the known type tags
    schema_set.insert("ParameterizedMotionSpace");
    let mut paths = Vec::new();
    let mut others = Vec::new();
    for s in all_strings {
        if schema_set.contains(s.as_str()) { continue; }
        // Asset-path heuristic: contains a slash AND has a recognizable
        // PA extension somewhere in its tail.
        let looks_like_path = s.contains('/')
            && (s.contains(".pa") || s.contains(".pab") || s.contains(".paa")
                || s.contains(".pam") || s.contains(".pac") || s.contains(".pat"));
        if looks_like_path {
            paths.push(s.clone());
        } else {
            others.push(s.clone());
        }
    }
    (paths, others)
}

/// Pair every `_`-prefixed name string with the following type-tag string.
/// Returns a list of (field_name, type_tag) tuples in document order.
///
/// Observed type tags (across all .motionblending samples):
///   `staticstringA`, `staticstring`, `staticfloat`, `staticint`,
///   `staticbool`, `bool`, `floatA`, `intA`.
///
/// Field names always start with underscore (`_skeletonFileName`,
/// `_animationFileNames`, `_motionPhaseType`, `_isLoopMotionBlending`,
/// `_isSyncMotionBlending`, ...).
fn extract_field_records(strings: &[String]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i + 1 < strings.len() {
        let a = &strings[i];
        let b = &strings[i + 1];
        if a.starts_with('_') && !b.starts_with('_') {
            out.push((a.clone(), b.clone()));
            i += 2;
        } else {
            i += 1;
        }
    }
    out
}

/// Walk `data` from start, attempting to read each u32 as a string length
/// prefix. If the resulting span is printable ASCII >= 4 chars, treat it
/// as a string. Advance past the span and continue. Skip 1 byte on
/// non-matches. Returns the unique strings found in iteration order.
fn scan_length_prefixed_strings(data: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut i = 0;
    while i + 4 < data.len() {
        let len = u32::from_le_bytes(data[i..i + 4].try_into().unwrap()) as usize;
        // Sanity caps: len must be plausible
        if (4..=512).contains(&len) && i + 4 + len <= data.len() {
            let span = &data[i + 4..i + 4 + len];
            if span.iter().all(|&b| (0x20..0x7f).contains(&b)) {
                if let Ok(s) = std::str::from_utf8(span) {
                    let s = s.to_string();
                    if !seen.contains(&s) {
                        seen.insert(s.clone());
                        out.push(s);
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

    /// Synthetic .motionblending: header + type_name + a couple of
    /// length-prefixed strings that the scanner should pick up.
    fn make_sample() -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&MAGIC_A.to_le_bytes());        // magic_a
        v.extend_from_slice(&4u16.to_le_bytes());           // version
        v.extend_from_slice(&0u64.to_le_bytes());           // reserved
        v.extend_from_slice(&0x000F_0000u32.to_le_bytes()); // flags_a
        v.extend_from_slice(&0x0004_0000u32.to_le_bytes()); // flags_b
        let type_name = b"ParameterizedMotionSpace";
        v.extend_from_slice(&(type_name.len() as u32).to_le_bytes());
        v.extend_from_slice(type_name);
        // synthetic property records
        v.extend_from_slice(&0x0011u16.to_le_bytes());        // prefix
        let n1 = b"_skeletonFileName";
        v.extend_from_slice(&(n1.len() as u32).to_le_bytes());
        v.extend_from_slice(n1);
        let t1 = b"staticstringA";
        v.extend_from_slice(&(t1.len() as u32).to_le_bytes());
        v.extend_from_slice(t1);
        // synthetic value padding
        v.extend_from_slice(&[0u8; 16]);
        v
    }

    #[test]
    fn parse_extracts_header_fields() {
        let bytes = make_sample();
        let v = parse_motionblending_to_json(&bytes).expect("parse");
        let m = v.as_object().unwrap();
        assert_eq!(m["magic_a"], Value::from("0xffff"));
        assert_eq!(m["version"], Value::from(4u64));
        assert_eq!(m["type_name"], Value::from("ParameterizedMotionSpace"));
        let strs: Vec<&str> = m["scanned_strings"].as_array().unwrap()
            .iter().map(|v| v.as_str().unwrap()).collect();
        assert!(strs.contains(&"_skeletonFileName"),
            "field name not scanned: got {:?}", strs);
        assert!(strs.contains(&"staticstringA"),
            "type tag not scanned: got {:?}", strs);
    }

    #[test]
    fn round_trip_byte_perfect() {
        let original = make_sample();
        let parsed = parse_motionblending_to_json(&original).expect("parse");
        let written = serialize_motionblending_from_json(&parsed).expect("write");
        assert_eq!(written, original, ".motionblending round-trip mismatch");
    }

    #[test]
    fn rejects_short_body() {
        let bytes = vec![0xff, 0xff, 0x04, 0x00];
        assert!(parse_motionblending_to_json(&bytes).is_err());
    }

    #[test]
    fn rejects_wrong_magic() {
        let mut bytes = make_sample();
        bytes[0] = 0xAA;
        bytes[1] = 0xBB;
        assert!(parse_motionblending_to_json(&bytes).is_err());
    }

    /// v3 layout has a tighter 16-byte header (vs v4's 24-byte).
    /// Synthesized from `bird_bigbird_move.motionblending` head:
    /// `ff ff 03 00 00 00 0e 00 00 00 06 00 18 00 00 00 ParameterizedMotionSpace`
    fn make_v3_sample() -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&MAGIC_A.to_le_bytes());        // 0xFFFF
        v.extend_from_slice(&3u16.to_le_bytes());           // version 3
        v.extend_from_slice(&0u16.to_le_bytes());           // reserved (u16 in v3)
        v.extend_from_slice(&0x0000_000Eu32.to_le_bytes()); // flags_a
        v.extend_from_slice(&0x0006u16.to_le_bytes());      // flags_b (u16 in v3)
        let type_name = b"ParameterizedMotionSpace";
        v.extend_from_slice(&(type_name.len() as u32).to_le_bytes());
        v.extend_from_slice(type_name);
        // synthetic field
        let n1 = b"_animationFileNames";
        v.extend_from_slice(&(n1.len() as u32).to_le_bytes());
        v.extend_from_slice(n1);
        v
    }

    #[test]
    fn parse_v3_layout() {
        let bytes = make_v3_sample();
        let v = parse_motionblending_to_json(&bytes).expect("parse v3");
        let m = v.as_object().unwrap();
        assert_eq!(m["version"], Value::from(3u64));
        assert_eq!(m["type_name"], Value::from("ParameterizedMotionSpace"));
        let strs: Vec<&str> = m["scanned_strings"].as_array().unwrap()
            .iter().map(|v| v.as_str().unwrap()).collect();
        assert!(strs.contains(&"_animationFileNames"));
    }

    #[test]
    fn v3_round_trip() {
        let original = make_v3_sample();
        let parsed = parse_motionblending_to_json(&original).expect("parse v3");
        let written = serialize_motionblending_from_json(&parsed).expect("write");
        assert_eq!(written, original);
    }

    #[test]
    fn pairs_field_name_with_type_tag() {
        let bytes = make_sample();
        let v = parse_motionblending_to_json(&bytes).expect("parse");
        let recs = v["field_records"].as_array().unwrap();
        assert_eq!(recs.len(), 1, "should pair one (_skeletonFileName, staticstringA)");
        assert_eq!(recs[0]["name"], Value::from("_skeletonFileName"));
        assert_eq!(recs[0]["type_tag"], Value::from("staticstringA"));
    }

    #[test]
    fn rejects_unsupported_version() {
        let mut bytes = make_sample();
        bytes[2] = 5;
        bytes[3] = 0;
        let result = parse_motionblending_to_json(&bytes);
        assert!(result.is_err());
        let msg = format!("{}", result.unwrap_err());
        assert!(msg.contains("unsupported version"), "got: {}", msg);
    }
}
