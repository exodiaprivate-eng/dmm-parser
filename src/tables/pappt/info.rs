// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Part-prefab table file parser.
//!
//! Field naming follows the spec in
//! `tools/mod-workbench/PAPPT_FORMAT_RESEARCH.md`. Layout:
//!
//! ```text
//! +0x00 u8[8]  opaque header (read-discarded by the loader,
//!              preserved verbatim on round-trip)
//! +0x08 u32    primary_count
//!       primary_entry[primary_count]
//!       u32    secondary_count
//!       secondary_entry[secondary_count]
//! EOF
//! ```
//!
//! Primary entry:
//!
//! ```text
//! pstr key_a
//! pstr key_b
//! pstr key_c     // read and discarded by the loader; preserved here
//! pstr asset_id
//! u8   flag
//! u8   child_count
//! { pstr sub_key, u8 sub_flag } * child_count
//! ```
//!
//! Secondary entry:
//!
//! ```text
//! pstr alias_a
//! pstr alias_b
//! ```
//!
//! `pstr` is a `u8` length prefix followed by `len` bytes of UTF-8
//! payload. The engine treats the bytes as C strings — no NUL is
//! written to the file. Maximum length is 255 bytes.
//!
//! The parser round-trips byte-for-byte against vanilla:
//! `PapptFile::parse(bytes)?.write() == bytes`.

use std::io;

use serde_json::{json, Map, Value};

/// Length-prefixed UTF-8 child variant inside a [`PrimaryEntry`].
///
/// `sub_key` is hashed by the loader through `sub_10055E114` into the
/// global string-intern table; we keep the raw string here so an editor
/// can rewrite it cleanly without depending on the live intern table.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrimaryChild {
    /// Variant key, length-prefixed in the file.
    pub sub_key: String,
    /// Variant flag byte. Semantics unknown — preserved verbatim.
    pub sub_flag: u8,
}

/// One primary part-prefab definition. Holds four short strings, one
/// flag byte, and a length-prefixed list of child variants.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrimaryEntry {
    /// Primary registry key (typically tribe / race name in vanilla).
    pub key_a: String,
    /// Secondary registry key (typically part-slot name in vanilla).
    pub key_b: String,
    /// Legacy / dev-only field. Read and discarded by the runtime
    /// loader, but preserved here so round-trip is byte-clean.
    pub key_c: String,
    /// Cross-cutting asset handle. Hashed into the global string
    /// intern table — same namespace as `_partPrefabKey`.
    pub asset_id: String,
    /// Entry-level flag byte. Stored at runtime offset `+0x14`.
    pub flag: u8,
    /// Variant children. Count is encoded as a `u8` in the file, so up
    /// to 255 children are addressable per primary entry.
    pub children: Vec<PrimaryChild>,
}

/// One secondary alias pair. The runtime registers both directions of
/// the alias so a lookup by either string returns the partner.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SecondaryEntry {
    /// First alias string.
    pub alias_a: String,
    /// Second alias string.
    pub alias_b: String,
}

/// Parsed `.pappt` file. Round-trips byte-for-byte against vanilla.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PapptFile {
    /// Opaque 8-byte header — read but never inspected by the loader.
    /// Preserved verbatim on round-trip so a modded file diffs cleanly
    /// against vanilla.
    pub header: [u8; 8],
    /// Primary entries (per-character / per-tribe part definitions).
    pub primary: Vec<PrimaryEntry>,
    /// Secondary alias pairs.
    pub secondary: Vec<SecondaryEntry>,
}

impl PapptFile {
    /// Parse a `.pappt` byte buffer. Returns the parsed structure or an
    /// `io::Error` describing the truncation / overrun on malformed
    /// input.
    pub fn parse(bytes: &[u8]) -> io::Result<PapptFile> {
        let mut offset = 0usize;

        // Header: 8 bytes, opaque.
        if bytes.len() < 8 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!("pappt header needs 8 bytes, file has {}", bytes.len()),
            ));
        }
        let mut header = [0u8; 8];
        header.copy_from_slice(&bytes[..8]);
        offset += 8;

        // Primary count + entries.
        let primary_count = read_u32(bytes, &mut offset)?;
        // Sanity clamp (mirrors CArray in binary/types.rs): each entry is
        // >= 1 byte, so a count exceeding the remaining byte budget is a
        // corrupted stream — Err instead of a huge up-front allocation.
        let remaining = bytes.len().saturating_sub(offset);
        if primary_count as usize > remaining {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!(
                    "pappt primary_count {} exceeds remaining {} at offset {}",
                    primary_count, remaining, offset,
                )));
        }
        let mut primary = Vec::with_capacity((primary_count as usize).min(1 << 20));
        for i in 0..primary_count {
            primary.push(read_primary_entry(bytes, &mut offset).map_err(|e| {
                io::Error::new(
                    e.kind(),
                    format!("primary entry #{}: {}", i, e),
                )
            })?);
        }

        // Secondary count + entries.
        let secondary_count = read_u32(bytes, &mut offset)?;
        // Sanity clamp (mirrors CArray in binary/types.rs): each entry is
        // >= 1 byte, so a count exceeding the remaining byte budget is a
        // corrupted stream — Err instead of a huge up-front allocation.
        let remaining = bytes.len().saturating_sub(offset);
        if secondary_count as usize > remaining {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!(
                    "pappt secondary_count {} exceeds remaining {} at offset {}",
                    secondary_count, remaining, offset,
                )));
        }
        let mut secondary = Vec::with_capacity((secondary_count as usize).min(1 << 20));
        for i in 0..secondary_count {
            secondary.push(read_secondary_entry(bytes, &mut offset).map_err(|e| {
                io::Error::new(
                    e.kind(),
                    format!("secondary entry #{}: {}", i, e),
                )
            })?);
        }

        if offset != bytes.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "trailing bytes after pappt body: consumed {} of {}",
                    offset,
                    bytes.len()
                ),
            ));
        }

        Ok(PapptFile {
            header,
            primary,
            secondary,
        })
    }

    /// Serialize back to bytes. Reciprocal of [`Self::parse`] — the
    /// returned `Vec<u8>` is byte-identical to the input on round-trip.
    pub fn write(&self) -> Vec<u8> {
        // Pre-size the output to avoid mid-write reallocations.
        let mut out = Vec::with_capacity(self.serialized_size());
        out.extend_from_slice(&self.header);

        let primary_count = self.primary.len() as u32;
        out.extend_from_slice(&primary_count.to_le_bytes());
        for entry in &self.primary {
            write_primary_entry(&mut out, entry);
        }

        let secondary_count = self.secondary.len() as u32;
        out.extend_from_slice(&secondary_count.to_le_bytes());
        for entry in &self.secondary {
            write_secondary_entry(&mut out, entry);
        }

        out
    }

    /// Conservative size estimate for the serialized output. Used to
    /// pre-size the writer's buffer; not authoritative.
    fn serialized_size(&self) -> usize {
        let mut sz = 8 + 4 + 4; // header + primary_count + secondary_count
        for entry in &self.primary {
            sz += pstr_size(&entry.key_a)
                + pstr_size(&entry.key_b)
                + pstr_size(&entry.key_c)
                + pstr_size(&entry.asset_id)
                + 1 // flag
                + 1; // child_count
            for child in &entry.children {
                sz += pstr_size(&child.sub_key) + 1;
            }
        }
        for entry in &self.secondary {
            sz += pstr_size(&entry.alias_a) + pstr_size(&entry.alias_b);
        }
        sz
    }
}

// ── JSON layer ───────────────────────────────────────────────────────
//
// pappt is a single-file format (header + two arrays). The dispatch
// contract returns `Vec<serde_json::Value>` with one Value per record;
// for whole-file formats we return a 1-element Vec where the single
// value carries the entire file shape plus `key: 0` / `string_key: ""`
// so v3 intent lookup (`find_record_index`) finds it.
//
// v3 intents address nested fields with paths like:
//   primary[5].key_a
//   primary[2].children[0].sub_key
//   secondary[1].alias_b
//   header[3]
//
// The existing `apply_field_set` path tokenizer (DMM's
// field_json_v3.rs) handles dot/bracket navigation natively.

impl PapptFile {
    /// Convert this parsed file into the canonical JSON shape used by
    /// `parse_table_to_json("pappt", ...)`. Round-trips byte-perfect
    /// when paired with [`Self::from_json_value`].
    ///
    /// `key` and `string_key` fields are added so the v3 intent
    /// dispatcher can look this record up by key=0.
    pub fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("key".into(), json!(0));
        m.insert("string_key".into(), json!(""));
        m.insert(
            "header".into(),
            Value::Array(self.header.iter().map(|b| json!(*b)).collect()),
        );

        let primary: Vec<Value> = self
            .primary
            .iter()
            .map(|p| {
                let mut po = Map::new();
                po.insert("key_a".into(), json!(p.key_a));
                po.insert("key_b".into(), json!(p.key_b));
                po.insert("key_c".into(), json!(p.key_c));
                po.insert("asset_id".into(), json!(p.asset_id));
                po.insert("flag".into(), json!(p.flag));
                let children: Vec<Value> = p
                    .children
                    .iter()
                    .map(|c| {
                        let mut co = Map::new();
                        co.insert("sub_key".into(), json!(c.sub_key));
                        co.insert("sub_flag".into(), json!(c.sub_flag));
                        Value::Object(co)
                    })
                    .collect();
                po.insert("children".into(), Value::Array(children));
                Value::Object(po)
            })
            .collect();
        m.insert("primary".into(), Value::Array(primary));

        let secondary: Vec<Value> = self
            .secondary
            .iter()
            .map(|s| {
                let mut so = Map::new();
                so.insert("alias_a".into(), json!(s.alias_a));
                so.insert("alias_b".into(), json!(s.alias_b));
                Value::Object(so)
            })
            .collect();
        m.insert("secondary".into(), Value::Array(secondary));

        Value::Object(m)
    }

    /// Inverse of [`Self::to_json_value`]. Tolerant of missing fields
    /// (uses defaults) so intent-mutated dicts that only carry edited
    /// fields still parse cleanly — but the caller is normally going
    /// to pass a complete dict produced by `to_json_value` then mutated
    /// in place.
    pub fn from_json_value(v: &Value) -> io::Result<PapptFile> {
        let obj = v.as_object().ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "pappt record: expected object")
        })?;

        let header = match obj.get("header") {
            Some(Value::Array(arr)) => {
                if arr.len() != 8 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("pappt header: expected 8 bytes, got {}", arr.len()),
                    ));
                }
                let mut h = [0u8; 8];
                for (i, byte) in arr.iter().enumerate() {
                    h[i] = byte.as_u64().ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("pappt header[{}]: expected u8 number", i),
                        )
                    })? as u8;
                }
                h
            }
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "pappt header: expected array of 8 bytes",
                ));
            }
            None => [0u8; 8],
        };

        let primary = match obj.get("primary") {
            Some(Value::Array(arr)) => arr
                .iter()
                .enumerate()
                .map(|(i, v)| primary_from_json(v).map_err(|e| {
                    io::Error::new(e.kind(), format!("primary[{}]: {}", i, e))
                }))
                .collect::<io::Result<Vec<_>>>()?,
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "pappt primary: expected array",
                ));
            }
            None => Vec::new(),
        };

        let secondary = match obj.get("secondary") {
            Some(Value::Array(arr)) => arr
                .iter()
                .enumerate()
                .map(|(i, v)| secondary_from_json(v).map_err(|e| {
                    io::Error::new(e.kind(), format!("secondary[{}]: {}", i, e))
                }))
                .collect::<io::Result<Vec<_>>>()?,
            Some(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "pappt secondary: expected array",
                ));
            }
            None => Vec::new(),
        };

        Ok(PapptFile { header, primary, secondary })
    }
}

/// Dispatch entry point: parse a pappt file body and return a 1-element
/// Vec<Value> matching the contract of `dispatch::parse_table_to_json`.
pub fn parse_pappt_to_json(bytes: &[u8]) -> io::Result<Vec<Value>> {
    let parsed = PapptFile::parse(bytes)?;
    Ok(vec![parsed.to_json_value()])
}

/// Inverse of `parse_pappt_to_json`. Reads the first item, stripping
/// `key` / `string_key` index fields if present.
pub fn serialize_pappt_from_json(items: &[Value]) -> io::Result<Vec<u8>> {
    if items.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "pappt: expected at least 1 item, got 0",
        ));
    }
    let parsed = PapptFile::from_json_value(&items[0])?;
    Ok(parsed.write())
}

fn primary_from_json(v: &Value) -> io::Result<PrimaryEntry> {
    let obj = v.as_object().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "expected object")
    })?;
    let s = |k: &str| -> io::Result<String> {
        Ok(obj.get(k)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_default())
    };
    let u8f = |k: &str| -> io::Result<u8> {
        Ok(obj.get(k)
            .and_then(|v| v.as_u64())
            .map(|n| n as u8)
            .unwrap_or(0))
    };
    let children = match obj.get("children") {
        Some(Value::Array(arr)) => arr
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let co = c.as_object().ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("children[{}]: expected object", i),
                    )
                })?;
                Ok(PrimaryChild {
                    sub_key: co.get("sub_key").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    sub_flag: co.get("sub_flag").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
                })
            })
            .collect::<io::Result<Vec<_>>>()?,
        Some(_) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "children: expected array",
            ));
        }
        None => Vec::new(),
    };
    Ok(PrimaryEntry {
        key_a: s("key_a")?,
        key_b: s("key_b")?,
        key_c: s("key_c")?,
        asset_id: s("asset_id")?,
        flag: u8f("flag")?,
        children,
    })
}

fn secondary_from_json(v: &Value) -> io::Result<SecondaryEntry> {
    let obj = v.as_object().ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "expected object")
    })?;
    Ok(SecondaryEntry {
        alias_a: obj.get("alias_a").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        alias_b: obj.get("alias_b").and_then(|v| v.as_str()).unwrap_or("").to_string(),
    })
}

// ── pstr helpers ─────────────────────────────────────────────────────

/// Read a `u8`-prefixed length-counted byte string and decode as UTF-8.
/// On success returns `(decoded_string, bytes_consumed)` so callers can
/// advance their cursor without re-scanning.
///
/// The engine treats the bytes as C strings — no NUL terminator is
/// written to the file. UTF-8 decode is lossy on bad sequences (the
/// loader's `strlen` would terminate at a NUL but won't validate
/// encoding), but real shipped files appear to be ASCII / UTF-8.
pub fn pstr_read(bytes: &[u8], offset: usize) -> io::Result<(String, usize)> {
    if offset >= bytes.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!(
                "pstr length byte out of bounds at offset {} (file size {})",
                offset,
                bytes.len()
            ),
        ));
    }
    let len = bytes[offset] as usize;
    let data_start = offset + 1;
    let data_end = data_start + len;
    if data_end > bytes.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!(
                "pstr body of length {} would overrun: needs {} bytes, have {}",
                len,
                data_end,
                bytes.len()
            ),
        ));
    }
    // Lossy UTF-8 decode mirrors the engine's relaxed treatment of
    // these bytes. A round-trip writer doesn't depend on the decode
    // being perfect — it re-encodes the `String` as UTF-8 and rewrites
    // the same byte sequence for ASCII payloads, which is what every
    // observed file uses.
    let s = String::from_utf8_lossy(&bytes[data_start..data_end]).into_owned();
    Ok((s, 1 + len))
}

/// Write a `u8`-prefixed length-counted byte string. Panics if the
/// UTF-8 byte length exceeds 255 — callers that accept user input
/// should validate before calling.
pub fn pstr_write(out: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    assert!(
        bytes.len() <= 255,
        "pstr exceeds u8 length cap: {} bytes",
        bytes.len()
    );
    out.push(bytes.len() as u8);
    out.extend_from_slice(bytes);
}

/// Byte size of the on-disk encoding of `s` as a pstr (length byte +
/// payload). Used by the size estimator.
fn pstr_size(s: &str) -> usize {
    1 + s.as_bytes().len()
}

// ── Internal helpers ────────────────────────────────────────────────

fn read_u32(bytes: &[u8], offset: &mut usize) -> io::Result<u32> {
    if *offset + 4 > bytes.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!(
                "u32 read at offset {} needs 4 bytes, have {}",
                *offset,
                bytes.len() - *offset
            ),
        ));
    }
    let v = u32::from_le_bytes(bytes[*offset..*offset + 4].try_into().unwrap());
    *offset += 4;
    Ok(v)
}

fn read_u8(bytes: &[u8], offset: &mut usize) -> io::Result<u8> {
    if *offset >= bytes.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("u8 read at offset {} past EOF", *offset),
        ));
    }
    let v = bytes[*offset];
    *offset += 1;
    Ok(v)
}

fn read_pstr(bytes: &[u8], offset: &mut usize) -> io::Result<String> {
    let (s, consumed) = pstr_read(bytes, *offset)?;
    *offset += consumed;
    Ok(s)
}

fn read_primary_entry(bytes: &[u8], offset: &mut usize) -> io::Result<PrimaryEntry> {
    let key_a = read_pstr(bytes, offset)?;
    let key_b = read_pstr(bytes, offset)?;
    let key_c = read_pstr(bytes, offset)?;
    let asset_id = read_pstr(bytes, offset)?;
    let flag = read_u8(bytes, offset)?;
    let child_count = read_u8(bytes, offset)?;

    // child_count is u8 (<= 255); cap the pre-allocation defensively to keep
    // the Vec::with_capacity bounded, matching the clamp pattern elsewhere.
    let mut children = Vec::with_capacity((child_count as usize).min(1 << 20));
    for i in 0..child_count {
        let sub_key = read_pstr(bytes, offset).map_err(|e| {
            io::Error::new(e.kind(), format!("child #{} sub_key: {}", i, e))
        })?;
        let sub_flag = read_u8(bytes, offset).map_err(|e| {
            io::Error::new(e.kind(), format!("child #{} sub_flag: {}", i, e))
        })?;
        children.push(PrimaryChild { sub_key, sub_flag });
    }

    Ok(PrimaryEntry {
        key_a,
        key_b,
        key_c,
        asset_id,
        flag,
        children,
    })
}

fn read_secondary_entry(bytes: &[u8], offset: &mut usize) -> io::Result<SecondaryEntry> {
    let alias_a = read_pstr(bytes, offset)?;
    let alias_b = read_pstr(bytes, offset)?;
    Ok(SecondaryEntry { alias_a, alias_b })
}

fn write_primary_entry(out: &mut Vec<u8>, entry: &PrimaryEntry) {
    pstr_write(out, &entry.key_a);
    pstr_write(out, &entry.key_b);
    pstr_write(out, &entry.key_c);
    pstr_write(out, &entry.asset_id);
    out.push(entry.flag);
    // child_count is a u8 in the wire format — assert before truncation
    // so a faulty editor bug surfaces as a panic rather than a silent
    // partial write.
    let child_count = entry.children.len();
    assert!(
        child_count <= 255,
        "primary entry has {} children; wire format caps at 255",
        child_count
    );
    out.push(child_count as u8);
    for child in &entry.children {
        pstr_write(out, &child.sub_key);
        out.push(child.sub_flag);
    }
}

fn write_secondary_entry(out: &mut Vec<u8>, entry: &SecondaryEntry) {
    pstr_write(out, &entry.alias_a);
    pstr_write(out, &entry.alias_b);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal synthetic file: two primary entries each with
    /// one child, two secondary aliases. Exercises the basic
    /// parse/write round-trip.
    #[test]
    fn synthetic_two_primary_two_secondary_roundtrips() {
        let original = PapptFile {
            header: [0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01, 0x02, 0x03],
            primary: vec![
                PrimaryEntry {
                    key_a: "Kliff".into(),
                    key_b: "hair".into(),
                    key_c: "src/kliff_hair.pmod".into(),
                    asset_id: "kliff_hair_default".into(),
                    flag: 0x01,
                    children: vec![PrimaryChild {
                        sub_key: "kliff_hair_long".into(),
                        sub_flag: 0x02,
                    }],
                },
                PrimaryEntry {
                    key_a: "Damiane".into(),
                    key_b: "face".into(),
                    key_c: "".into(),
                    asset_id: "damiane_face_default".into(),
                    flag: 0x00,
                    children: vec![PrimaryChild {
                        sub_key: "damiane_face_battle".into(),
                        sub_flag: 0x10,
                    }],
                },
            ],
            secondary: vec![
                SecondaryEntry {
                    alias_a: "old_kliff_hair".into(),
                    alias_b: "kliff_hair_default".into(),
                },
                SecondaryEntry {
                    alias_a: "legacy_face".into(),
                    alias_b: "damiane_face_default".into(),
                },
            ],
        };

        let bytes = original.write();
        let parsed = PapptFile::parse(&bytes).expect("parse synthetic");
        assert_eq!(parsed, original);

        // Byte-identical re-emit.
        let reemitted = parsed.write();
        assert_eq!(
            reemitted, bytes,
            "round-trip diverged at len {} vs {}",
            reemitted.len(),
            bytes.len()
        );
    }

    /// A single primary entry with the maximum addressable child count
    /// (`u8::MAX = 255`) must round-trip cleanly. Catches off-by-one
    /// errors in the child-count writer / reader.
    #[test]
    fn primary_with_255_children_roundtrips() {
        let mut children = Vec::with_capacity(255);
        for i in 0..255u8 {
            children.push(PrimaryChild {
                sub_key: format!("variant_{:03}", i),
                sub_flag: i,
            });
        }
        let original = PapptFile {
            header: [0u8; 8],
            primary: vec![PrimaryEntry {
                key_a: "Common".into(),
                key_b: "torso".into(),
                key_c: "".into(),
                asset_id: "common_torso".into(),
                flag: 0xFF,
                children,
            }],
            secondary: Vec::new(),
        };

        let bytes = original.write();
        let parsed = PapptFile::parse(&bytes).expect("parse 255-child entry");
        assert_eq!(parsed.primary.len(), 1);
        assert_eq!(parsed.primary[0].children.len(), 255);
        assert_eq!(parsed, original);
        assert_eq!(parsed.write(), bytes);
    }

    /// Empty file (header + zero primary + zero secondary) round-trips.
    #[test]
    fn empty_file_roundtrips() {
        let original = PapptFile {
            header: [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88],
            primary: Vec::new(),
            secondary: Vec::new(),
        };
        let bytes = original.write();
        // 8 (header) + 4 (primary_count=0) + 4 (secondary_count=0) = 16
        assert_eq!(bytes.len(), 16);
        let parsed = PapptFile::parse(&bytes).expect("parse empty");
        assert_eq!(parsed, original);
        assert_eq!(parsed.write(), bytes);
    }

    /// Truncated body (declared primary count larger than available
    /// data) should produce a clean error rather than panicking. With the
    /// 1.4.7.1 count-clamp (count exceeding the remaining byte budget is
    /// rejected up front), the declared `primary_count=1` against zero
    /// remaining bytes now surfaces as `InvalidData` instead of reaching
    /// the entry reader's `UnexpectedEof`. Both are clean, catchable errors
    /// — the contract is "Err, never panic/abort".
    #[test]
    fn truncated_body_errors_clean() {
        // Header + primary_count=1 + (no body)
        let mut bytes = vec![0u8; 8];
        bytes.extend_from_slice(&1u32.to_le_bytes());
        let err = PapptFile::parse(&bytes).expect_err("expected truncation error");
        assert!(
            matches!(err.kind(), io::ErrorKind::UnexpectedEof | io::ErrorKind::InvalidData),
            "expected a clean Err, got {:?}",
            err.kind(),
        );
    }

    /// JSON round-trip — parse synthetic file to JSON, serialize from
    /// JSON back to bytes, byte-equal. Covers the dispatch path used by
    /// `parse_table_to_json("pappt", ...)` /
    /// `serialize_table_from_json("pappt", ...)`.
    #[test]
    fn json_roundtrip() {
        let original = PapptFile {
            header: [0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01, 0x02, 0x03],
            primary: vec![
                PrimaryEntry {
                    key_a: "Kliff".into(),
                    key_b: "hair".into(),
                    key_c: "src/kliff_hair.pmod".into(),
                    asset_id: "kliff_hair_default".into(),
                    flag: 0x01,
                    children: vec![
                        PrimaryChild { sub_key: "kliff_hair_long".into(), sub_flag: 0x02 },
                        PrimaryChild { sub_key: "".into(), sub_flag: 0x00 },
                    ],
                },
            ],
            secondary: vec![
                SecondaryEntry {
                    alias_a: "old_kliff_hair".into(),
                    alias_b: "kliff_hair_default".into(),
                },
            ],
        };
        let bytes = original.write();
        let json_items = parse_pappt_to_json(&bytes).expect("parse to json");
        assert_eq!(json_items.len(), 1, "pappt parses to a 1-element vec");
        let reemitted = serialize_pappt_from_json(&json_items).expect("serialize from json");
        assert_eq!(reemitted, bytes, "JSON round-trip diverged");

        // The single record carries the synthetic indexing fields so v3
        // intent dispatch can find it by key=0.
        let rec = json_items[0].as_object().unwrap();
        assert_eq!(rec.get("key").and_then(|v| v.as_u64()), Some(0));
        assert_eq!(rec.get("string_key").and_then(|v| v.as_str()), Some(""));
    }

    /// Mutating a single nested field via JSON path navigation produces
    /// a different byte output that still parses cleanly. Sanity check
    /// for the v3 apply pipeline when it edits nested arrays.
    #[test]
    fn json_field_edit_through_path() {
        let original = PapptFile {
            header: [0u8; 8],
            primary: vec![PrimaryEntry {
                key_a: "Kliff".into(),
                key_b: "hair".into(),
                key_c: "".into(),
                asset_id: "kliff_hair_default".into(),
                flag: 0,
                children: vec![PrimaryChild { sub_key: "old_name".into(), sub_flag: 0 }],
            }],
            secondary: Vec::new(),
        };
        let bytes_before = original.write();

        let mut json_items = parse_pappt_to_json(&bytes_before).expect("parse");
        // Hand-mutate the JSON to simulate a v3 intent set on
        // `primary[0].children[0].sub_key`.
        json_items[0]
            .as_object_mut().unwrap()
            .get_mut("primary").unwrap()
            .as_array_mut().unwrap()[0]
            .as_object_mut().unwrap()
            .get_mut("children").unwrap()
            .as_array_mut().unwrap()[0]
            .as_object_mut().unwrap()
            .insert("sub_key".into(), json!("new_name"));

        let bytes_after = serialize_pappt_from_json(&json_items).expect("serialize");
        assert_ne!(bytes_after, bytes_before, "edit must change output bytes");
        // Re-parse to verify the new bytes are still well-formed and
        // the edit landed.
        let reparsed = PapptFile::parse(&bytes_after).expect("re-parse mutated");
        assert_eq!(reparsed.primary[0].children[0].sub_key, "new_name");
    }

    /// pstr_read / pstr_write are public helpers; smoke-test them
    /// directly against the in-spec encoding.
    #[test]
    fn pstr_helpers_roundtrip() {
        let inputs = ["", "a", "hello", "kliff_hair_long_v2"];
        for s in inputs {
            let mut buf = Vec::new();
            pstr_write(&mut buf, s);
            let (decoded, consumed) = pstr_read(&buf, 0).expect("read pstr");
            assert_eq!(decoded, s);
            assert_eq!(consumed, buf.len());
        }
    }
}
