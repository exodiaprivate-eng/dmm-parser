// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.pami` — Static Mesh Instance metadata.
//!
//! Format reality (decoded iter 3): plain UTF-8 XML with root element
//! `<StaticMeshInstance Version="N">`. Every sampled file across the
//! 1.06 install conforms (10/10 in iter-3 audit). **NOT a Havok-layer
//! file** despite earlier classification — the .pami extension stands
//! for **"PA Mesh Instance"**, not "PA aniMation Index" as previously
//! documented. The doc previously had this wrong; corrected iter 3.
//!
//! Structure (typical):
//! ```xml
//! <StaticMeshInstance Version="1">
//!   <StaticMesh Path="object/03_cube.pa..."/>
//!   <Transform .../>
//!   ...
//! </StaticMeshInstance>
//! ```
//!
//! Round-trip: byte-perfect via the `xml_body` field — the parser
//! preserves whitespace, line endings, indentation, and any tail bytes
//! verbatim. Mod authors can either edit the XML text directly
//! (preserves round-trip exactly) or edit the extracted convenience
//! fields and rely on the writer's reconstruction (loses formatting,
//! safe for authoring new instances from scratch).
//!
//! Convenience fields extracted for mod tooling:
//!   - `version`: integer from `<StaticMeshInstance Version="N">`
//!   - `mesh_paths`: list of `Path=` attribute values from `<StaticMesh>`
//!     elements (or whatever element is named within the root).

use std::io::{self};

use serde_json::{Map, Value};

const STATIC_MESH_INSTANCE_OPEN: &str = "<StaticMeshInstance";

/// Parse a `.pami` file body to a JSON dict.
///
/// The dict carries:
///   - `xml_body`: full XML text as UTF-8 (round-trip source of truth)
///   - `version`: integer version attribute, or `null` if not present
///   - `mesh_paths`: list of `Path` attribute strings extracted from
///     `<StaticMesh ...>` elements
///   - `key`: `0` and `string_key`: `""` (v3-intent compatibility)
pub fn parse_pami_to_json(data: &[u8]) -> io::Result<Value> {
    // Strip optional UTF-8 BOM for parsing the version, but preserve the
    // original bytes in xml_body so write-back is byte-identical.
    let text_for_parse = if data.starts_with(&[0xEF, 0xBB, 0xBF]) {
        std::str::from_utf8(&data[3..]).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData,
                format!(".pami body is not valid UTF-8: {}", e))
        })?
    } else {
        std::str::from_utf8(data).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData,
                format!(".pami body is not valid UTF-8: {}", e))
        })?
    };

    if !text_for_parse.trim_start().starts_with(STATIC_MESH_INSTANCE_OPEN) {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!(".pami expected root element {}, got first 32 chars: {:?}",
                STATIC_MESH_INSTANCE_OPEN,
                &text_for_parse[..text_for_parse.len().min(32)])));
    }

    let version = extract_version_attr(text_for_parse);
    let mesh_paths = extract_path_attrs(text_for_parse);

    let xml_body = String::from_utf8(data.to_vec()).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData,
            format!(".pami xml_body utf-8 conversion: {}", e))
    })?;

    let mut map = Map::new();
    map.insert("key".to_string(), Value::from(0u64));
    map.insert("string_key".to_string(), Value::from(""));
    map.insert("xml_body".to_string(), Value::from(xml_body));
    map.insert("version".to_string(),
        match version { Some(v) => Value::from(v), None => Value::Null });
    map.insert("mesh_paths".to_string(),
        Value::Array(mesh_paths.into_iter().map(Value::from).collect()));
    Ok(Value::Object(map))
}

/// Serialize a `.pami` JSON dict back to bytes. Uses `xml_body` verbatim
/// if present (round-trip byte-perfect). If absent, reconstructs a
/// minimal valid document from `version` + `mesh_paths` (intended only
/// for newly-authored instances; existing files should round-trip via
/// xml_body to preserve formatting).
pub fn serialize_pami_from_json(value: &Value) -> io::Result<Vec<u8>> {
    let map = value.as_object().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData, ".pami serialize: expected object root"))?;

    if let Some(Value::String(body)) = map.get("xml_body") {
        return Ok(body.as_bytes().to_vec());
    }

    // Reconstruction path: build a minimal valid document.
    let version = map.get("version")
        .and_then(|v| v.as_i64())
        .unwrap_or(1);
    let mesh_paths: Vec<&str> = map.get("mesh_paths")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let mut out = String::new();
    out.push_str(&format!("<StaticMeshInstance Version=\"{}\">\r\n", version));
    for p in &mesh_paths {
        out.push_str(&format!("\t<StaticMesh Path=\"{}\"/>\r\n", p));
    }
    out.push_str("</StaticMeshInstance>\r\n");
    Ok(out.into_bytes())
}

/// Extract `Version="N"` from a `<StaticMeshInstance ...>` opening tag.
fn extract_version_attr(text: &str) -> Option<i64> {
    let after_root = text.split_once(STATIC_MESH_INSTANCE_OPEN)?.1;
    let close_idx = after_root.find('>')?;
    let attrs = &after_root[..close_idx];
    let v_idx = attrs.find("Version=\"")?;
    let after_v = &attrs[v_idx + "Version=\"".len()..];
    let v_end = after_v.find('"')?;
    after_v[..v_end].parse().ok()
}

/// Extract every `Path="..."` attribute value from the document.
/// Naive — splits on the literal substring, sufficient for the
/// well-formed XML these files contain.
fn extract_path_attrs(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(idx) = rest.find("Path=\"") {
        rest = &rest[idx + "Path=\"".len()..];
        if let Some(end) = rest.find('"') {
            out.push(rest[..end].to_string());
            rest = &rest[end + 1..];
        } else {
            break;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_XML: &[u8] = b"<StaticMeshInstance Version=\"1\">\r\n\t<StaticMesh Path=\"object/03_cube.pa\"/>\r\n\t<StaticMesh Path=\"object/03_plane.pa\"/>\r\n</StaticMeshInstance>\r\n";

    #[test]
    fn parse_extracts_version_and_paths() {
        let v = parse_pami_to_json(SAMPLE_XML).expect("parse");
        let m = v.as_object().unwrap();
        assert_eq!(m["version"], Value::from(1i64));
        let paths: Vec<&str> = m["mesh_paths"].as_array().unwrap()
            .iter().map(|v| v.as_str().unwrap()).collect();
        assert_eq!(paths, vec!["object/03_cube.pa", "object/03_plane.pa"]);
        assert!(m["xml_body"].as_str().unwrap().contains("StaticMeshInstance"));
    }

    #[test]
    fn round_trip_byte_perfect() {
        let parsed = parse_pami_to_json(SAMPLE_XML).expect("parse");
        let written = serialize_pami_from_json(&parsed).expect("write");
        assert_eq!(written, SAMPLE_XML, ".pami round-trip mismatch");
    }

    #[test]
    fn rejects_non_xml_input() {
        let bad = b"\x00\x01\x02\x03random binary";
        let result = parse_pami_to_json(bad);
        assert!(result.is_err());
    }

    #[test]
    fn reconstruct_from_fields_when_no_xml_body() {
        let mut m = Map::new();
        m.insert("version".to_string(), Value::from(2i64));
        m.insert("mesh_paths".to_string(),
            Value::Array(vec![Value::from("test/path1.pa"), Value::from("test/path2.pa")]));
        let v = Value::Object(m);
        let written = serialize_pami_from_json(&v).expect("write");
        let s = std::str::from_utf8(&written).unwrap();
        assert!(s.contains("Version=\"2\""));
        assert!(s.contains("Path=\"test/path1.pa\""));
        assert!(s.contains("Path=\"test/path2.pa\""));
    }
}
