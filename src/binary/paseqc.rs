// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.paseqc` — compiled sequencer chart binary (sister format to .paseq).
//!
//! ## Format
//!
//! `.paseqc` is the **same reflection-based self-describing format**
//! as `.paseq` — same magic, same recursive type-schema-then-values
//! layout. The differences:
//!
//! - No leading 16-byte zero pad. `.paseqc` starts directly with
//!   `ff ff 04 00 ...` (where `.paseq` has 16 bytes of preamble).
//! - Root class name is `"SequencerGamePlayDataFile"` (25 chars) for
//!   chart files vs `"Sequencer"` (9 chars) for top-level sequencer
//!   files. Both share the same engine reflection serializer.
//! - Field count and field list of the root class differ (different
//!   schema), but the wire encoding is identical.
//!
//! ## Tier 1 (typed) coverage
//!
//! Same as `.paseq`: variable-length pre-class-name `header`
//! (preserved verbatim), `class_name` (field-level editable),
//! `opaque_body` (preserved verbatim).
//!
//! ## Tier 1.5 fallback
//!
//! [`PaseqcFile`] (the LP-token-stream view) ships as a discovery
//! tool — walks every embedded CString in the file without parsing
//! the recursive schema.

use std::io::{self, Write};

use serde_json::{Map, Value};

use super::{BinaryRead, BinaryWrite, CString};
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};

pub use super::lp_token_stream::{LpTokenFile as PaseqcFile, Token};

// Note: `PaseqcFile::class_name()` is provided via the `PaseqFile`
// (sister) `impl` block in `paseq.rs` — both are type aliases for
// `LpTokenFile`, so a single `impl` covers both.

// ── Tier 1 typed reader ──────────────────────────────────────────────

/// Maximum offset to scan for the root class-name CString. Same
/// envelope as `.paseq` — class name lands well within the first 64
/// bytes for every vanilla sample.
const PASEQC_HEADER_SCAN_MAX: usize = 64;

/// Typed `.paseqc` file — Tier 1 outer-shell view.
///
/// See module docs for the format relationship with `.paseq`. The
/// parser locates the root class-name CString by scanning the first
/// 64 bytes for a `u32 len + N printable bytes` pattern. Bytes before
/// the match become `header`; the CString itself becomes
/// `class_name`; the rest becomes `opaque_body`.
///
/// Round-trips byte-exact via `parse`/`to_bytes` for every well-formed
/// vanilla sample.
#[derive(Debug)]
pub struct TypedPaseqcFile<'a> {
    pub header: Vec<u8>,
    pub class_name: CString<'a>,
    pub opaque_body: Vec<u8>,
}

impl<'a> TypedPaseqcFile<'a> {
    /// Convenience: parse the outer class block's field directory from
    /// the opaque body. Reuses the `.paseq` schema walker since both
    /// formats share the same reflection layout. Returns the declared
    /// `(field_name, type_name)` pairs for the root
    /// `SequencerGamePlayDataFile` class without recursing into nested
    /// class blocks.
    pub fn outer_fields(&self) -> io::Result<Vec<crate::binary::paseq::PaseqFieldDef>> {
        crate::binary::paseq::parse_outer_fields(&self.opaque_body)
    }

    /// Sister to [`crate::binary::paseq::TypedPaseqFile::value_section_offset`].
    pub fn value_section_offset(&self) -> io::Result<usize> {
        crate::binary::paseq::parse_all_class_blocks_consumed(
            &self.opaque_body, &self.class_name.data,
        )
    }

    /// Sister to [`crate::binary::paseq::TypedPaseqFile::value_section`].
    pub fn value_section(&self) -> io::Result<&[u8]> {
        let off = self.value_section_offset()?;
        Ok(&self.opaque_body[off..])
    }

    /// Sister to
    /// [`crate::binary::paseq::TypedPaseqFile::value_section_strings`].
    pub fn value_section_strings(&self) -> io::Result<Vec<(usize, String)>> {
        let value_start = self.value_section_offset()?;
        let values = &self.opaque_body[value_start..];
        let file_offset_base = self.header.len()
            + 4 + self.class_name.data.len()
            + value_start;
        let mut out = Vec::new();
        let mut i = 0;
        while i + 4 <= values.len() {
            let len = u32::from_le_bytes([values[i], values[i + 1], values[i + 2], values[i + 3]]) as usize;
            if !(1..=4096).contains(&len) || i + 4 + len > values.len() {
                i += 1;
                continue;
            }
            let bytes = &values[i + 4..i + 4 + len];
            if bytes.iter().all(|&b| {
                (0x20..=0x7e).contains(&b) || b == b'\n' || b == b'\t'
            }) {
                if let Ok(s) = std::str::from_utf8(bytes) {
                    out.push((file_offset_base + i, s.to_string()));
                    i += 4 + len;
                    continue;
                }
            }
            i += 1;
        }
        Ok(out)
    }

    /// Walk all class blocks (outer + nested) in `opaque_body`. See
    /// [`crate::binary::paseq::TypedPaseqFile::all_class_blocks`] for
    /// the layout — `.paseq` and `.paseqc` share the same format.
    pub fn all_class_blocks(&self) -> io::Result<Vec<crate::binary::paseq::PaseqClassBlock>> {
        // Inline the walker since opaque_body / class_name shape is
        // identical to .paseq's; reuse the same algorithm.
        let mut blocks = Vec::new();
        let outer_fields = crate::binary::paseq::parse_outer_fields(&self.opaque_body)?;
        let mut offset = crate::binary::paseq::parse_outer_fields_consumed(&self.opaque_body)?;
        blocks.push(crate::binary::paseq::PaseqClassBlock {
            class_name: self.class_name.data.to_string(),
            fields: outer_fields,
        });
        while offset + 4 <= self.opaque_body.len() {
            let len = u32::from_le_bytes([
                self.opaque_body[offset], self.opaque_body[offset + 1],
                self.opaque_body[offset + 2], self.opaque_body[offset + 3],
            ]) as usize;
            if !(2..=64).contains(&len) { break; }
            let str_start = offset + 4;
            if str_start + len + 2 > self.opaque_body.len() { break; }
            let name_bytes = &self.opaque_body[str_start..str_start + len];
            if !name_bytes.iter().all(|&b| b.is_ascii_alphanumeric() || b == b'_') { break; }
            let class_name = std::str::from_utf8(name_bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                .to_string();
            let mut try_offset = str_start + len;
            if try_offset + 2 > self.opaque_body.len() { break; }
            let field_count = u16::from_le_bytes([
                self.opaque_body[try_offset], self.opaque_body[try_offset + 1],
            ]) as usize;
            try_offset += 2;
            if field_count > 256 { break; }
            let mut fields = Vec::with_capacity(field_count);
            let mut ok = true;
            for _ in 0..field_count {
                let r = (|| -> io::Result<crate::binary::paseq::PaseqFieldDef> {
                    let fname = crate::binary::paseq::read_cstring(&self.opaque_body, &mut try_offset)?;
                    let tname = crate::binary::paseq::read_cstring(&self.opaque_body, &mut try_offset)?;
                    if try_offset + 8 > self.opaque_body.len() {
                        return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "meta"));
                    }
                    let mut meta = [0u8; 8];
                    meta.copy_from_slice(&self.opaque_body[try_offset..try_offset + 8]);
                    try_offset += 8;
                    Ok(crate::binary::paseq::PaseqFieldDef { field_name: fname, type_name: tname, type_meta: meta })
                })();
                match r {
                    Ok(f) => fields.push(f),
                    Err(_) => { ok = false; break; }
                }
            }
            if !ok { break; }
            blocks.push(crate::binary::paseq::PaseqClassBlock { class_name, fields });
            offset = try_offset;
        }
        Ok(blocks)
    }

    /// Parse an entire `.paseqc` file. Locates the root class-name
    /// CString and splits the file into (`header`, `class_name`,
    /// `opaque_body`).
    pub fn parse(data: &'a [u8]) -> io::Result<Self> {
        // Scan the first 64 bytes for the first `u32 len + identifier
        // bytes` pattern that matches a class-name shape.
        let scan_max = data.len().min(PASEQC_HEADER_SCAN_MAX);
        let mut split_at: Option<usize> = None;
        for i in 0..scan_max.saturating_sub(4) {
            let len = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
            if !(4..=64).contains(&len) {
                continue;
            }
            let str_start = i + 4;
            let str_end = str_start.checked_add(len as usize)
                .filter(|&e| e <= data.len() && e <= scan_max);
            let Some(str_end) = str_end else { continue };
            let bytes = &data[str_start..str_end];
            if bytes.iter().all(|&b| b.is_ascii_alphanumeric() || b == b'_') {
                split_at = Some(i);
                break;
            }
        }
        let split_at = split_at.ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "TypedPaseqcFile: no class-name CString found in first {} bytes",
                scan_max,
            ),
        ))?;
        let header = data[..split_at].to_vec();
        let mut offset = split_at;
        let class_name = CString::read_from(data, &mut offset)?;
        let opaque_body = data
            .get(offset..)
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPaseqcFile: body offset past data length",
            ))?
            .to_vec();
        Ok(Self { header, class_name, opaque_body })
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut out = Vec::new();
        self.write_to(&mut out)?;
        Ok(out)
    }
}

impl<'a> BinaryWrite for TypedPaseqcFile<'a> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.header)?;
        self.class_name.write_to(w)?;
        w.write_all(&self.opaque_body)?;
        Ok(())
    }
}

impl<'a> ToJsonValue for TypedPaseqcFile<'a> {
    fn to_json_value(&self) -> Value {
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
        let mut m = Map::new();
        m.insert("header_b64".into(), Value::String(B64.encode(&self.header)));
        m.insert("class_name".into(), Value::String(self.class_name.data.to_string()));
        m.insert(
            "opaque_body_b64".into(),
            Value::String(B64.encode(&self.opaque_body)),
        );
        Value::Object(m)
    }
}

impl<'a> WriteJsonValue for TypedPaseqcFile<'a> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "TypedPaseqcFile: expected object",
        ))?;
        let header_b64 = json_get_field(obj, "header_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPaseqcFile.header_b64: expected string",
            ))?;
        let header = B64.decode(header_b64).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("TypedPaseqcFile.header_b64: invalid base64: {}", e),
        ))?;
        w.extend_from_slice(&header);
        let class_name = json_get_field(obj, "class_name")?
            .as_str()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPaseqcFile.class_name: expected string",
            ))?;
        let cn_bytes = class_name.as_bytes();
        if cn_bytes.len() > u32::MAX as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPaseqcFile.class_name: too long for u32 length prefix",
            ));
        }
        w.extend_from_slice(&(cn_bytes.len() as u32).to_le_bytes());
        w.extend_from_slice(cn_bytes);
        let body_b64 = json_get_field(obj, "opaque_body_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPaseqcFile.opaque_body_b64: expected base64 string",
            ))?;
        let body = B64.decode(body_b64).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("TypedPaseqcFile.opaque_body_b64: invalid base64: {}", e),
        ))?;
        w.extend_from_slice(&body);
        Ok(())
    }
}

// ── Decoded | Raw safety wrapper ─────────────────────────────────────

#[derive(Debug)]
pub enum PaseqcFileSafe<'a> {
    Decoded(TypedPaseqcFile<'a>),
    Raw {
        bytes: Vec<u8>,
        decode_error: String,
    },
}

impl<'a> PaseqcFileSafe<'a> {
    pub fn parse(data: &'a [u8]) -> io::Result<Self> {
        match TypedPaseqcFile::parse(data) {
            Ok(typed) => Ok(Self::Decoded(typed)),
            Err(e) => Ok(Self::Raw {
                bytes: data.to_vec(),
                decode_error: e.to_string(),
            }),
        }
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        match self {
            Self::Decoded(t) => t.to_bytes(),
            Self::Raw { bytes, .. } => Ok(bytes.clone()),
        }
    }
}
