// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Generic UTF-8 XML resource classifier (iter 18).
//!
//! Crimson Desert ships several plain UTF-8 XML metadata files that
//! the engine reads at load time. As of iter 18, 5 extensions verified:
//!
//! | Ext | Root element | BOM | Sample purpose |
//! |---|---|---|---|
//! | `.material`   | `<Technique Name="...">`    | yes | Render material |
//! | `.technique`  | `<Category Name="...">`     | yes | Render technique category |
//! | `.mi`         | `<SkinnedDecal...>` etc.    | no  | Skinned-decal material instance |
//! | `.spline`     | `<SplineDataGroup>`         | no  | 3D spline curve |
//! | `.spline2d`   | `<SplinePresetData...>`     | no  | 2D spline preset |
//!
//! The 6th `.pami` format also fits this pattern (`<StaticMeshInstance>`)
//! but already has its own dedicated module `src/binary/pami.rs` with
//! mesh-path extraction.
//!
//! This module ships **byte-perfect round-trip + root-element extraction**
//! (Tier 1). The full XML body is preserved verbatim in `xml_body`; the
//! convenience field `root_element` lets mod tooling dispatch on root
//! without re-parsing the XML.

use std::io::{self};

use serde_json::{Map, Value};

/// Parse any PA-engine UTF-8 XML resource. Strips optional BOM for the
/// parse pass but preserves it (and the rest of the body) in `xml_body`
/// for byte-perfect round-trip.
pub fn parse_xml_to_json(data: &[u8]) -> io::Result<Value> {
    let has_bom = data.starts_with(&[0xEF, 0xBB, 0xBF]);
    let body_for_parse = if has_bom { &data[3..] } else { data };

    let text = std::str::from_utf8(body_for_parse).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData,
            format!("xml body not utf-8: {}", e))
    })?;

    let trimmed = text.trim_start();
    if !trimmed.starts_with('<') {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!("xml body doesn't start with '<': got {:?}",
                &trimmed[..trimmed.len().min(32)])));
    }

    let root_element = extract_root_element(trimmed).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData,
            "xml body has no parseable root element name")
    })?;

    let xml_body = String::from_utf8(data.to_vec()).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData,
            format!("xml_body utf-8 conversion: {}", e))
    })?;

    let mut map = Map::new();
    map.insert("key".to_string(), Value::from(0u64));
    map.insert("string_key".to_string(), Value::from(""));
    map.insert("has_bom".to_string(), Value::from(has_bom));
    map.insert("root_element".to_string(), Value::from(root_element));
    map.insert("xml_body".to_string(), Value::from(xml_body));
    map.insert("body_len".to_string(), Value::from(data.len() as u64));
    Ok(Value::Object(map))
}

/// Serialize back — uses `xml_body` verbatim for byte-perfect output.
pub fn serialize_xml_from_json(value: &Value) -> io::Result<Vec<u8>> {
    let map = value.as_object().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData, "xml serialize: expected object root"))?;
    let body = map.get("xml_body")
        .and_then(|v| v.as_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            "xml serialize: missing 'xml_body' string"))?;
    Ok(body.as_bytes().to_vec())
}

/// Extract the root element name from a (BOM-stripped) XML string.
/// Cheap regex-free parse — walks the first tag and captures the
/// name up to whitespace, `/`, or `>`.
fn extract_root_element(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut i = 0;
    // Skip whitespace + XML declaration (`<?xml ...?>`) + comments.
    loop {
        // skip whitespace
        while i < bytes.len() && bytes[i].is_ascii_whitespace() { i += 1; }
        if i >= bytes.len() { return None; }
        if bytes[i] != b'<' { return None; }
        // <? prolog → skip to ?>
        if i + 1 < bytes.len() && bytes[i + 1] == b'?' {
            while i + 1 < bytes.len() && !(bytes[i] == b'?' && bytes[i + 1] == b'>') { i += 1; }
            i = i.saturating_add(2);
            continue;
        }
        // <!-- comment --> → skip to -->
        if i + 3 < bytes.len() && &bytes[i + 1..i + 4] == b"!--" {
            while i + 2 < bytes.len()
                && !(bytes[i] == b'-' && bytes[i + 1] == b'-' && bytes[i + 2] == b'>') {
                i += 1;
            }
            i = i.saturating_add(3);
            continue;
        }
        // <!DOCTYPE ...> etc.
        if i + 1 < bytes.len() && bytes[i + 1] == b'!' {
            while i < bytes.len() && bytes[i] != b'>' { i += 1; }
            i = i.saturating_add(1);
            continue;
        }
        // Actual root element tag
        break;
    }
    // Now bytes[i] = '<'. Read name until whitespace, '/', or '>'.
    i += 1;
    let start = i;
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_whitespace() || b == b'/' || b == b'>' { break; }
        i += 1;
    }
    if i > start {
        std::str::from_utf8(&bytes[start..i]).ok().map(|s| s.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_no_bom() {
        let body = b"<SplineDataGroup>\n  <Item id=\"0\"/>\n</SplineDataGroup>";
        let v = parse_xml_to_json(body).expect("parse");
        let m = v.as_object().unwrap();
        assert_eq!(m["has_bom"], Value::from(false));
        assert_eq!(m["root_element"], Value::from("SplineDataGroup"));
    }

    #[test]
    fn parses_with_bom() {
        let mut body = vec![0xEF, 0xBB, 0xBF];
        body.extend_from_slice(b"<Technique Name=\"Foo\">\n</Technique>");
        let v = parse_xml_to_json(&body).expect("parse");
        let m = v.as_object().unwrap();
        assert_eq!(m["has_bom"], Value::from(true));
        assert_eq!(m["root_element"], Value::from("Technique"));
    }

    #[test]
    fn parses_with_prolog_and_comment() {
        let body = b"<?xml version=\"1.0\"?>\n<!-- header -->\n<Category Name=\"x\"/>";
        let v = parse_xml_to_json(body).expect("parse");
        assert_eq!(v["root_element"], Value::from("Category"));
    }

    #[test]
    fn round_trip_byte_perfect() {
        let original = b"<Technique Name=\"Foo\">\r\n  <Pass/>\r\n</Technique>\r\n".to_vec();
        let v = parse_xml_to_json(&original).expect("parse");
        let written = serialize_xml_from_json(&v).expect("write");
        assert_eq!(written, original);
    }

    #[test]
    fn rejects_non_xml() {
        let bad = b"\x00\x01\x02not xml";
        assert!(parse_xml_to_json(bad).is_err());
    }

    #[test]
    fn rejects_invalid_utf8() {
        let bad = &[0xFF, 0xFE, 0xFD][..];
        assert!(parse_xml_to_json(bad).is_err());
    }

    #[test]
    fn handles_self_closing_root() {
        let body = b"<SkinnedDecal/>";
        let v = parse_xml_to_json(body).expect("parse");
        assert_eq!(v["root_element"], Value::from("SkinnedDecal"));
    }
}
