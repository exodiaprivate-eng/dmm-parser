// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Projectile-attribute `.paatt` file parser.
//!
//! Field names mirror the Python reference at
//! `ResearchFolder/paac/paatt_parser.py`:
//!
//! - `entry_count`: u32 at +0x00 — number of projectile entries.
//! - `hash_marker`: u32 at +0x04 — table identifier hash.
//!
//! The remaining bytes are the per-entry body. Entry boundaries are
//! detected heuristically: each physics-projectile entry contains a
//! `0.01f` default for `projectileRadius` and another `0.01f` exactly
//! 0x20 bytes later (the `endEffectLifeTime` default in the same entry).
//! Pairing these two anchors picks out projectileRadius positions; the
//! UI exposes those as editable entries.
//!
//! This struct round-trips: `read_from(&data, &mut 0)` followed by
//! `write_to(&mut Vec::new())` reproduces the original bytes exactly.
//! The body is held opaquely so format additions in future game patches
//! don't break the parser — only the editable-field overlay needs an
//! update.
//!
//! Per memory `project_paac_paatt_parser_progress.md`: physics entries
//! are 546 bytes each, `PhysicsData.projectileRadius` is at runtime
//! offset `0x440` (default `0.01f`). The serialized layout in `.paatt`
//! mostly matches the in-memory order; the anchor-pair detector below
//! is defensive against entries that don't carry the default.

use std::io;

use serde_json::{json, Map, Value};

use crate::binary::{BinaryRead, BinaryWrite};

/// Distance (in bytes) between the `projectileRadius` default float and
/// the `endEffectLifeTime` default float within a physics-projectile
/// entry. Used by [`PaattFile::physics_radius_offsets`].
pub const RADIUS_PAIR_DISTANCE: usize = 0x20;

/// `0.01f` little-endian — the default value for both `projectileRadius`
/// and `endEffectLifeTime`. The pair-distance match below filters out
/// stray `0.01f` values that aren't part of a physics entry.
pub const DEFAULT_RADIUS_BYTES: [u8; 4] = [0x0A, 0xD7, 0x23, 0x3C];

/// Field offsets (in bytes, relative to a `projectileRadius` anchor) of
/// editable physics fields. Matches the layout used by
/// `ResearchFolder/paac/paatt_patch.py`.
pub const FIELD_OFFSETS: &[(i64, &str)] = &[
    (0x00, "projectileRadius"),
    (0x04, "reflectRate"),
    (0x08, "endEffectLifeTime"),
    (0x0C, "spawnItemKey"),
    (0x10, "shapeSize.x"),
    (0x14, "shapeSize.y"),
    (0x18, "shapeSize.z"),
    (0x1C, "shapeOffset.x"),
    (0x20, "shapeOffset.y"),
    (0x24, "shapeOffset.z"),
];

/// Parsed projectile-attribute file. Fields exactly match the Python
/// `PaattFile` dataclass at `ResearchFolder/paac/paatt_parser.py`.
#[derive(Debug, Clone)]
pub struct PaattFile {
    /// Header u32 at +0x00. Number of entries in the body.
    pub entry_count: u32,
    /// Header u32 at +0x04. Identifier hash (Bob Jenkins per project
    /// memory `project_jenkins_hash_universal.md`); preserved verbatim.
    pub hash_marker: u32,
    /// Bytes from +0x08 to EOF. Held opaquely so the parser round-trips
    /// without needing the full per-entry schema.
    pub body: Vec<u8>,
}

impl PaattFile {
    /// Total file size in bytes when serialized.
    pub fn size(&self) -> usize {
        8 + self.body.len()
    }

    /// Parse from a byte slice. Returns the parsed file plus the number
    /// of bytes consumed (always equal to `data.len()` on success).
    pub fn parse(data: &[u8]) -> io::Result<Self> {
        let mut offset = 0usize;
        let pf = Self::read_from(data, &mut offset)?;
        if offset != data.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "trailing bytes after paatt body: consumed {} of {}",
                    offset,
                    data.len()
                ),
            ));
        }
        Ok(pf)
    }

    /// Serialize back to bytes. Reciprocal of [`Self::parse`].
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.size());
        // BinaryWrite::write_to is infallible for a Vec<u8> writer, but
        // the trait signature returns Result, so we unwrap.
        self.write_to(&mut out).expect("write to Vec");
        out
    }

    /// Find every `projectileRadius` anchor offset (relative to the body
    /// start, i.e. the file offset minus the 8-byte header). An anchor
    /// is a `0.01f` value that has another `0.01f` exactly
    /// [`RADIUS_PAIR_DISTANCE`] bytes later — that pair is strong
    /// evidence of a physics-projectile entry.
    pub fn physics_radius_offsets(&self) -> Vec<usize> {
        let occurrences = find_all(&self.body, &DEFAULT_RADIUS_BYTES);
        let set: std::collections::HashSet<usize> = occurrences.iter().copied().collect();
        occurrences
            .into_iter()
            .filter(|o| set.contains(&(o + RADIUS_PAIR_DISTANCE)))
            .collect()
    }

    /// Read the float field at `body_offset + rel_offset`. Returns
    /// `None` when the read would go out of bounds.
    pub fn read_field_f32(&self, body_offset: usize, rel_offset: i64) -> Option<f32> {
        let pos = body_offset.checked_add_signed(rel_offset as isize)?;
        if pos.checked_add(4)? > self.body.len() {
            return None;
        }
        let bytes: [u8; 4] = self.body[pos..pos + 4].try_into().ok()?;
        Some(f32::from_le_bytes(bytes))
    }

    /// Write the float field at `body_offset + rel_offset`. Returns
    /// `false` when the write would go out of bounds.
    pub fn write_field_f32(
        &mut self,
        body_offset: usize,
        rel_offset: i64,
        value: f32,
    ) -> bool {
        let Some(pos) = body_offset.checked_add_signed(rel_offset as isize) else {
            return false;
        };
        let Some(end) = pos.checked_add(4) else {
            return false;
        };
        if end > self.body.len() {
            return false;
        }
        self.body[pos..end].copy_from_slice(&value.to_le_bytes());
        true
    }

    /// Read the u32 field at `body_offset + rel_offset`. Used for
    /// `spawnItemKey` and other 4-byte key references.
    pub fn read_field_u32(&self, body_offset: usize, rel_offset: i64) -> Option<u32> {
        let pos = body_offset.checked_add_signed(rel_offset as isize)?;
        if pos.checked_add(4)? > self.body.len() {
            return None;
        }
        let bytes: [u8; 4] = self.body[pos..pos + 4].try_into().ok()?;
        Some(u32::from_le_bytes(bytes))
    }

    /// Write the u32 field at `body_offset + rel_offset`.
    pub fn write_field_u32(
        &mut self,
        body_offset: usize,
        rel_offset: i64,
        value: u32,
    ) -> bool {
        let Some(pos) = body_offset.checked_add_signed(rel_offset as isize) else {
            return false;
        };
        let Some(end) = pos.checked_add(4) else {
            return false;
        };
        if end > self.body.len() {
            return false;
        }
        self.body[pos..end].copy_from_slice(&value.to_le_bytes());
        true
    }
}

impl<'a> BinaryRead<'a> for PaattFile {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        if *offset + 8 > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "paatt header needs 8 bytes, have {} at offset {}",
                    data.len() - *offset,
                    *offset
                ),
            ));
        }
        let entry_count = u32::from_le_bytes(data[*offset..*offset + 4].try_into().unwrap());
        let hash_marker =
            u32::from_le_bytes(data[*offset + 4..*offset + 8].try_into().unwrap());
        let body = data[*offset + 8..].to_vec();
        *offset = data.len();
        Ok(PaattFile {
            entry_count,
            hash_marker,
            body,
        })
    }
}

impl BinaryWrite for PaattFile {
    fn write_to(&self, w: &mut dyn io::Write) -> io::Result<()> {
        w.write_all(&self.entry_count.to_le_bytes())?;
        w.write_all(&self.hash_marker.to_le_bytes())?;
        w.write_all(&self.body)?;
        Ok(())
    }
}

// ── JSON layer ───────────────────────────────────────────────────────
//
// paatt is a single-file format (header + opaque body). Same dispatch
// contract as pappt/pamhc: returns 1-element Vec<Value> with synthetic
// `key: 0` / `string_key: ""` for v3 intent dispatch lookup.
//
// JSON field paths address scalars + body bytes:
//   entry_count                   (u32)
//   hash_marker                   (u32)
//   body[N]                       (one byte at body offset N)
//
// Per-entry physics fields (projectileRadius etc.) aren't surfaced as
// named fields in the JSON because their offsets are anchor-detected
// and vary per file. v3 mod authors who want to edit physics fields
// would address them by raw body offset (`body[1234]`) — same way the
// Workbench editor finds them today.

impl PaattFile {
    /// Convert this parsed file into the canonical JSON shape used by
    /// `parse_table_to_json("paatt", ...)`. Round-trips byte-perfect
    /// when paired with [`Self::from_json_value`].
    pub fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("key".into(), json!(0));
        m.insert("string_key".into(), json!(""));
        m.insert("entry_count".into(), json!(self.entry_count));
        m.insert("hash_marker".into(), json!(self.hash_marker));
        m.insert(
            "body".into(),
            Value::Array(self.body.iter().map(|b| json!(*b)).collect()),
        );
        Value::Object(m)
    }

    /// Inverse of [`Self::to_json_value`]. Tolerant of missing fields
    /// (defaults `entry_count`/`hash_marker` to 0 and `body` to empty).
    pub fn from_json_value(v: &Value) -> io::Result<PaattFile> {
        let obj = v.as_object().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "paatt record: expected object")
        })?;

        let entry_count = obj.get("entry_count")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32)
            .unwrap_or(0);
        let hash_marker = obj.get("hash_marker")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32)
            .unwrap_or(0);
        let body = match obj.get("body") {
            Some(Value::Array(arr)) => arr
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    v.as_u64()
                        .map(|n| n as u8)
                        .ok_or_else(|| {
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                format!("paatt body[{}]: expected u8 number", i),
                            )
                        })
                })
                .collect::<io::Result<Vec<_>>>()?,
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "paatt body: expected byte array",
                ));
            }
            None => Vec::new(),
        };

        Ok(PaattFile { entry_count, hash_marker, body })
    }
}

/// Dispatch entry point: parse a paatt file body into a 1-element
/// Vec<Value>. Matches the contract of `dispatch::parse_table_to_json`.
pub fn parse_paatt_to_json(bytes: &[u8]) -> io::Result<Vec<Value>> {
    let parsed = PaattFile::parse(bytes)?;
    Ok(vec![parsed.to_json_value()])
}

/// Inverse of `parse_paatt_to_json`. Reads the first item.
pub fn serialize_paatt_from_json(items: &[Value]) -> io::Result<Vec<u8>> {
    if items.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "paatt: expected at least 1 item, got 0",
        ));
    }
    let parsed = PaattFile::from_json_value(&items[0])?;
    Ok(parsed.to_bytes())
}

/// Find every offset where `needle` appears in `haystack`. Equivalent to
/// the Python `data.find(needle, pos)` loop in `paatt_patch.py`.
fn find_all(haystack: &[u8], needle: &[u8]) -> Vec<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos + needle.len() <= haystack.len() {
        if &haystack[pos..pos + needle.len()] == needle {
            out.push(pos);
            pos += 1;
        } else {
            pos += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sample fixtures — guarded with a SKIP message when missing so the
    /// test suite still passes on machines without the research folder.
    const FIXTURES: &[&str] = &[
        r"C:\Users\Coding\CrimsonDesertModding\ResearchFolder\paac\game_projectileinfo_pc.paatt",
        r"C:\Users\Coding\CrimsonDesertModding\ResearchFolder\paac\sample_projectileinfo.paatt",
        r"C:\Users\Coding\CrimsonDesertModding\ResearchFolder\paac\sample_projectileinfo_pc.paatt",
        r"C:\Users\Coding\CrimsonDesertModding\ResearchFolder\paac\sample_projectileinfo_pc_x50.paatt",
        r"C:\Users\Coding\CrimsonDesertModding\ResearchFolder\paac\game_projectileinfo_pc_x50.paatt",
    ];

    #[test]
    fn roundtrip_each_sample() {
        let mut tested = 0;
        for path in FIXTURES {
            let Ok(data) = std::fs::read(path) else {
                eprintln!("SKIP: missing fixture {}", path);
                continue;
            };
            tested += 1;
            let parsed = PaattFile::parse(&data).unwrap_or_else(|e| {
                panic!("parse failed for {}: {}", path, e);
            });
            let serialized = parsed.to_bytes();
            assert_eq!(
                serialized.len(),
                data.len(),
                "size mismatch for {}: got {} expected {}",
                path,
                serialized.len(),
                data.len()
            );
            assert_eq!(
                serialized,
                data,
                "roundtrip bytes diverge for {} (first diff: {:?})",
                path,
                first_diff(&serialized, &data)
            );
        }
        if tested == 0 {
            eprintln!("WARNING: no paatt fixtures available; roundtrip test skipped");
        }
    }

    #[test]
    fn header_fields_match_python_reference() {
        // Per the Python paatt_parser.py output:
        //   game_projectileinfo_pc.paatt: entry_count=209, hash_marker=0x4526AD00
        let path = FIXTURES[0];
        let Ok(data) = std::fs::read(path) else {
            eprintln!("SKIP: missing fixture {}", path);
            return;
        };
        let parsed = PaattFile::parse(&data).unwrap();
        assert_eq!(parsed.entry_count, 209);
        assert_eq!(parsed.hash_marker, 0x4526AD00);
        assert_eq!(parsed.body.len(), data.len() - 8);
    }

    #[test]
    fn physics_radius_anchors_match_python() {
        // The Python paatt_patch.py finds N pairs in each fixture; we
        // don't pin the count (it varies by fixture and game version)
        // but we do require: every returned offset has a paired anchor
        // exactly RADIUS_PAIR_DISTANCE bytes later.
        let path = FIXTURES[0];
        let Ok(data) = std::fs::read(path) else {
            eprintln!("SKIP: missing fixture {}", path);
            return;
        };
        let parsed = PaattFile::parse(&data).unwrap();
        let anchors = parsed.physics_radius_offsets();
        assert!(
            !anchors.is_empty(),
            "expected at least one physics anchor in {}",
            path
        );
        for &a in &anchors {
            let lo = parsed.read_field_f32(a, 0).unwrap();
            let hi = parsed.read_field_f32(a, RADIUS_PAIR_DISTANCE as i64).unwrap();
            assert!(
                (lo - 0.01).abs() < 1e-9,
                "anchor {} not 0.01f: {}",
                a,
                lo
            );
            assert!(
                (hi - 0.01).abs() < 1e-9,
                "paired anchor at +0x{:X} not 0.01f: {}",
                a + RADIUS_PAIR_DISTANCE,
                hi
            );
        }
    }

    /// JSON round-trip on a synthetic file (no fixture needed).
    /// Covers `parse_paatt_to_json` / `serialize_paatt_from_json`.
    #[test]
    fn json_roundtrip_synthetic() {
        let original = PaattFile {
            entry_count: 42,
            hash_marker: 0xDEADBEEF,
            body: vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A],
        };
        let bytes = original.to_bytes();
        let json_items = parse_paatt_to_json(&bytes).expect("parse to json");
        assert_eq!(json_items.len(), 1);
        let reemitted = serialize_paatt_from_json(&json_items).expect("serialize");
        assert_eq!(reemitted, bytes);

        let rec = json_items[0].as_object().unwrap();
        assert_eq!(rec.get("key").and_then(|v| v.as_u64()), Some(0));
        assert_eq!(rec.get("entry_count").and_then(|v| v.as_u64()), Some(42));
        assert_eq!(rec.get("hash_marker").and_then(|v| v.as_u64()), Some(0xDEADBEEF));
    }

    /// Edit a body byte via JSON path, verify bytes change. Mirrors how
    /// a v3 intent like `field: "body[3]", new: 0xFF` would land.
    #[test]
    fn json_body_byte_edit() {
        let original = PaattFile {
            entry_count: 1,
            hash_marker: 0,
            body: vec![0x10, 0x20, 0x30, 0x40, 0x50],
        };
        let bytes_before = original.to_bytes();
        let mut items = parse_paatt_to_json(&bytes_before).expect("parse");
        items[0]
            .as_object_mut().unwrap()
            .get_mut("body").unwrap()
            .as_array_mut().unwrap()[3] = json!(0xFFu8);
        let bytes_after = serialize_paatt_from_json(&items).expect("serialize");
        assert_ne!(bytes_after, bytes_before);
        let reparsed = PaattFile::parse(&bytes_after).expect("re-parse");
        assert_eq!(reparsed.body[3], 0xFF);
        assert_eq!(reparsed.body[0], 0x10);
        assert_eq!(reparsed.body[4], 0x50);
    }

    #[test]
    fn write_field_roundtrip() {
        let path = FIXTURES[0];
        let Ok(data) = std::fs::read(path) else {
            eprintln!("SKIP: missing fixture {}", path);
            return;
        };
        let mut parsed = PaattFile::parse(&data).unwrap();
        let anchors = parsed.physics_radius_offsets();
        if let Some(&first) = anchors.first() {
            assert!(parsed.write_field_f32(first, 0, 0.5));
            assert_eq!(parsed.read_field_f32(first, 0).unwrap(), 0.5);
            // Round-trip: serialize then re-parse.
            let bytes = parsed.to_bytes();
            let reparsed = PaattFile::parse(&bytes).unwrap();
            assert_eq!(
                reparsed.read_field_f32(first, 0).unwrap(),
                0.5,
                "field edit lost across roundtrip"
            );
        }
    }

    fn first_diff(a: &[u8], b: &[u8]) -> Option<(usize, u8, u8)> {
        a.iter()
            .zip(b.iter())
            .enumerate()
            .find_map(|(i, (x, y))| if x == y { None } else { Some((i, *x, *y)) })
    }
}
