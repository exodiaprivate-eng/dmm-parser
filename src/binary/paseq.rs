// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.paseq` — sequencer (cutscene / scripted action) binary.
//!
//! ## Format overview (Session 5-6 RE)
//!
//! `.paseq` is a **reflection-based self-describing format**. Every
//! file embeds its own class schema before the field values:
//!
//! ```text
//! 0x00-0x1F: 32-byte fixed header (00 00 00 00 00 42 00 ... ff ff
//!                                  04 ... 00 0f 00)
//! 0x20-0x21: u16 zero
//! 0x22-0x23: u16 root_count       (sample-specific, varies)
//! 0x24+:     CString class_name   = "Sequencer"
//! +2 bytes:  u16 field_count      = 15
//! +N bytes:  per-field { CString field_name, CString type_name,
//!                        8 bytes type metadata }
//! +M bytes:  recursively-nested class blocks
//! +K bytes:  field VALUES (decoded via the schema)
//! ```
//!
//! `pa::Sequencer` (RTTI `0x106bc7582`, vtable `0x1079da2b0`) does
//! NOT have a hand-written deserialize virtual — the engine uses a
//! generic reflection-based serializer that walks any
//! metaobject-tagged class. This is the same pattern the engine
//! uses for `.paseqc`, save game state, and other reflective
//! containers.
//!
//! ## Tier 1 (typed) coverage — current
//!
//! - **`header`** — 32 bytes preserved. Magic at `+5` (= 0x42),
//!   sentinel `0xffff` at `+0x10`.
//! - **`zero`** — u16 (always 0 in vanilla samples).
//! - **`root_count`** — u16, sample-specific.
//! - **`class_name`** — `"Sequencer"` for every `.paseq` (read/write
//!   exposed for completeness).
//! - **`opaque_body`** — recursive schema + values, preserved
//!   verbatim. Round-trip byte-perfect.
//!
//! Future iterations: parse the recursive class schema, then the
//! value section, to expose `_sequencerName`, `_version`,
//! `_startTimePiece`, etc. as field-level editable.
//!
//! ## Tier 1.5 fallback
//!
//! [`PaseqFile`] (the LP-token-stream view) still ships as a
//! discovery tool — it walks every embedded CString in the file
//! (field names, type names, value strings) without needing to
//! parse the recursive schema.

use std::io::{self, Write};

use serde_json::{Map, Value};

use super::{BinaryRead, BinaryWrite, CString};
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};

pub use super::lp_token_stream::{LpTokenFile as PaseqFile, Token};

impl PaseqFile {
    /// Convenience: in `.paseq` files the first LP-string is the root
    /// class name (always `"Sequencer"` for vanilla data).
    pub fn class_name(&self) -> Option<&[u8]> {
        for tok in &self.tokens {
            if let Token::LpString(body) = tok {
                return Some(body);
            }
        }
        None
    }
}

// ── Generic LP-string discovery (Session 21) ─────────────────────────

/// Walk `data` looking for `u32 length + N printable bytes` patterns.
/// Returns `(absolute_offset, string)` tuples where `absolute_offset`
/// is `base_offset + position_within_data`.
///
/// Heuristic: a candidate string must be 1..=4096 bytes long and
/// contain only printable ASCII or common whitespace (`\n`, `\t`).
/// On a successful match the walker advances PAST the string so it
/// doesn't double-count overlapping regions.
///
/// Pairs with [`replace_cstring_at`] for mod-author tooling — find
/// strings, edit them by file offset.
pub fn walk_u32_prefixed_strings(
    data: &[u8],
    base_offset: usize,
) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 4 <= data.len() {
        let len = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize;
        if !(1..=4096).contains(&len) || i + 4 + len > data.len() {
            i += 1;
            continue;
        }
        let bytes = &data[i + 4..i + 4 + len];
        if bytes.iter().all(|&b| {
            (0x20..=0x7e).contains(&b) || b == b'\n' || b == b'\t'
        }) {
            if let Ok(s) = std::str::from_utf8(bytes) {
                out.push((base_offset + i, s.to_string()));
                i += 4 + len;
                continue;
            }
        }
        i += 1;
    }
    out
}

// ── Generic CString-at-offset edit primitive (Session 20) ───────────

/// Replace a `u32 length + N bytes` CString at the given file offset
/// with `new_value`. The replacement is length-flexible — the result
/// file is `new_value.len() - old_length` bytes larger or smaller.
///
/// This works on any format that stores values as `u32 length +
/// bytes` — `.paseq`, `.paseqc`, `.pastage`, `.paschedule`, etc.
/// Mod-author tools can locate strings via
/// [`TypedPaseqFile::value_section_strings`] (or sibling accessors)
/// and pass the returned `file_offset` here.
///
/// Caveats:
/// - Caller is responsible for verifying `expected_value` matches the
///   bytes at `file_offset` (set to `None` to skip the check).
/// - If the file format encodes any total-size or downstream-offset
///   fields that reference bytes after `file_offset`, those need to
///   be updated separately. `.paseq`/`.paseqc`/`.pastage`/
///   `.paschedule` formats do NOT have such fields — the parser walks
///   forward without internal back-references.
pub fn replace_cstring_at(
    file_bytes: &[u8],
    file_offset: usize,
    expected_value: Option<&str>,
    new_value: &str,
) -> io::Result<Vec<u8>> {
    if file_offset + 4 > file_bytes.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "replace_cstring_at: offset 0x{:x} + 4 byte u32 length exceeds file size {}",
                file_offset, file_bytes.len(),
            ),
        ));
    }
    let old_len = u32::from_le_bytes([
        file_bytes[file_offset],
        file_bytes[file_offset + 1],
        file_bytes[file_offset + 2],
        file_bytes[file_offset + 3],
    ]) as usize;
    if file_offset + 4 + old_len > file_bytes.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "replace_cstring_at: u32 length {} at offset 0x{:x} exceeds remaining file bytes",
                old_len, file_offset,
            ),
        ));
    }
    let old_bytes = &file_bytes[file_offset + 4..file_offset + 4 + old_len];
    if let Some(expected) = expected_value {
        if old_bytes != expected.as_bytes() {
            let actual = std::str::from_utf8(old_bytes).unwrap_or("<non-utf8>");
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "replace_cstring_at: expected {:?} at offset 0x{:x} but found {:?}",
                    expected, file_offset, actual,
                ),
            ));
        }
    }
    let new_bytes = new_value.as_bytes();
    if new_bytes.len() > u32::MAX as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("replace_cstring_at: new value too long for u32 length: {}", new_bytes.len()),
        ));
    }
    let mut out = Vec::with_capacity(file_bytes.len() + new_bytes.len().saturating_sub(old_len));
    out.extend_from_slice(&file_bytes[..file_offset]);
    out.extend_from_slice(&(new_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(new_bytes);
    out.extend_from_slice(&file_bytes[file_offset + 4 + old_len..]);
    Ok(out)
}

// ── Outer class field directory (Session 16) ─────────────────────────

/// One field declaration from a `.paseq` reflection schema. The wire
/// layout per field is `CString field_name + CString type_name +
/// 8 bytes type metadata`. The metadata block encodes type-id, size,
/// alignment, and flags but its exact bit layout is not yet decoded
/// — we preserve the 8 bytes verbatim so the round-trip stays exact.
#[derive(Debug, Clone)]
pub struct PaseqFieldDef {
    pub field_name: String,
    pub type_name: String,
    pub type_meta: [u8; 8],
}

/// Walk the start of `opaque_body` to extract the outer class block's
/// field declarations. The schema layout immediately after `class_name`
/// is:
///
/// ```text
///   u16 field_count
///   field_count × { CString field_name, CString type_name, u8[8] meta }
/// ```
///
/// This is read-only — the parsed fields are returned without altering
/// the typed reader's round-trip behavior. Mod authors can call this
/// to enumerate the declared fields for inspection / mod-manifest
/// generation. Recursive nested-class blocks following the outer
/// field list are not parsed by this method.
pub fn parse_outer_fields(opaque_body: &[u8]) -> io::Result<Vec<PaseqFieldDef>> {
    if opaque_body.len() < 2 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "parse_outer_fields: opaque_body too short for u16 field_count",
        ));
    }
    let field_count = u16::from_le_bytes([opaque_body[0], opaque_body[1]]) as usize;
    let mut offset = 2usize;
    let mut fields = Vec::with_capacity(field_count);
    for i in 0..field_count {
        // CString = u32 length + N bytes.
        let field_name = read_cstring(opaque_body, &mut offset)
            .map_err(|e| io::Error::new(
                e.kind(),
                format!("parse_outer_fields: field {} name: {}", i, e),
            ))?;
        let type_name = read_cstring(opaque_body, &mut offset)
            .map_err(|e| io::Error::new(
                e.kind(),
                format!("parse_outer_fields: field {} type: {}", i, e),
            ))?;
        if offset + 8 > opaque_body.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!("parse_outer_fields: field {} meta: not enough data", i),
            ));
        }
        let mut type_meta = [0u8; 8];
        type_meta.copy_from_slice(&opaque_body[offset..offset + 8]);
        offset += 8;
        fields.push(PaseqFieldDef { field_name, type_name, type_meta });
    }
    Ok(fields)
}

pub fn read_cstring(data: &[u8], offset: &mut usize) -> io::Result<String> {
    if *offset + 4 > data.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("CString u32 length at {} but only {} bytes available",
                offset, data.len()),
        ));
    }
    let len = u32::from_le_bytes([
        data[*offset], data[*offset + 1],
        data[*offset + 2], data[*offset + 3],
    ]) as usize;
    *offset += 4;
    if *offset + len > data.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("CString len={} at {} exceeds data length {}",
                len, *offset, data.len()),
        ));
    }
    let s = std::str::from_utf8(&data[*offset..*offset + len])
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        .to_string();
    *offset += len;
    Ok(s)
}

impl<'a> TypedPaseqFile<'a> {
    /// Convenience: parse the outer class block's field directory from
    /// the opaque body. See [`parse_outer_fields`] for layout details.
    /// Returns the declared `(field_name, type_name)` pairs for the
    /// root `Sequencer` (or `SequencerGamePlayDataFile` for `.paseqc`)
    /// class, without recursing into nested class blocks.
    pub fn outer_fields(&self) -> io::Result<Vec<PaseqFieldDef>> {
        parse_outer_fields(&self.opaque_body)
    }

    /// Return the byte offset within `opaque_body` where the value
    /// section starts (i.e. the first byte after the last class
    /// block). Use [`Self::value_section`] to get the slice directly.
    pub fn value_section_offset(&self) -> io::Result<usize> {
        parse_all_class_blocks_consumed(&self.opaque_body, &self.class_name.data)
    }

    /// Bytes after the schema — the actual field values declared by
    /// the class blocks. Decoding these per-type is future work, but
    /// the slice is exposed for tools that want to do their own
    /// analysis or surgical edits. Round-trips byte-perfect via
    /// `opaque_body` whether or not the value section is decoded.
    pub fn value_section(&self) -> io::Result<&[u8]> {
        let off = self.value_section_offset()?;
        Ok(&self.opaque_body[off..])
    }

    /// Walk the value section and return every `u32 length + N printable
    /// bytes` pattern as a `(byte_offset, string)` tuple. The byte
    /// offset is RELATIVE TO THE START OF THE FILE so callers can do
    /// surgical bin-edits (e.g. replacing a `_sequencerName` value).
    /// Captures `staticstringA` field values, embedded asset path
    /// references, and similar variable-length string data.
    ///
    /// Heuristic: a candidate string must be 1..=4096 bytes long and
    /// contain only printable ASCII + a few common separators
    /// (`/ \ . _ - : ` plus space). This avoids false positives from
    /// random byte values that happen to look like u32 lengths.
    pub fn value_section_strings(&self) -> io::Result<Vec<(usize, String)>> {
        let value_start = self.value_section_offset()?;
        let values = &self.opaque_body[value_start..];
        // file_offset_base = header.len() + (4 + class_name.data.len()) + value_start
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

    /// Walk all class blocks (outer + linearly-following nested
    /// classes) in `opaque_body`. Each nested block has the same
    /// shape as the outer field list, just prefixed by a
    /// `CString class_name + u16 field_count`. The walker stops when
    /// the next 4 bytes don't look like a reasonable u32 CString
    /// length (i.e., we've hit the value section).
    pub fn all_class_blocks(&self) -> io::Result<Vec<PaseqClassBlock>> {
        let mut blocks = Vec::new();
        // Outer block: opaque_body starts with `u16 field_count`,
        // class_name comes from the typed reader.
        let outer_fields = parse_outer_fields(&self.opaque_body)?;
        // Compute how many bytes the outer fields consumed; the
        // walker resumes from there to find linearly-following nested
        // class blocks.
        let mut offset = parse_outer_fields_consumed(&self.opaque_body)?;
        blocks.push(PaseqClassBlock {
            class_name: self.class_name.data.to_string(),
            fields: outer_fields,
        });
        // Walk nested blocks linearly.
        while offset + 4 <= self.opaque_body.len() {
            let len = u32::from_le_bytes([
                self.opaque_body[offset], self.opaque_body[offset + 1],
                self.opaque_body[offset + 2], self.opaque_body[offset + 3],
            ]) as usize;
            // A class name is short, alphanumeric, and not zero.
            if !(2..=64).contains(&len) { break; }
            let str_start = offset + 4;
            if str_start + len + 2 > self.opaque_body.len() { break; }
            let name_bytes = &self.opaque_body[str_start..str_start + len];
            if !name_bytes.iter().all(|&b| b.is_ascii_alphanumeric() || b == b'_') {
                break;
            }
            let class_name = std::str::from_utf8(name_bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                .to_string();
            // Try to read the rest of the block.
            let mut try_offset = str_start + len;
            if try_offset + 2 > self.opaque_body.len() { break; }
            let field_count = u16::from_le_bytes([
                self.opaque_body[try_offset], self.opaque_body[try_offset + 1],
            ]) as usize;
            try_offset += 2;
            if field_count > 256 { break; }  // sanity check
            let mut fields = Vec::with_capacity(field_count);
            let mut ok = true;
            for _ in 0..field_count {
                match read_cstring(&self.opaque_body, &mut try_offset)
                    .and_then(|fname| {
                        let tname = read_cstring(&self.opaque_body, &mut try_offset)?;
                        if try_offset + 8 > self.opaque_body.len() {
                            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "meta"));
                        }
                        let mut meta = [0u8; 8];
                        meta.copy_from_slice(&self.opaque_body[try_offset..try_offset + 8]);
                        try_offset += 8;
                        Ok(PaseqFieldDef {
                            field_name: fname,
                            type_name: tname,
                            type_meta: meta,
                        })
                    })
                {
                    Ok(f) => fields.push(f),
                    Err(_) => { ok = false; break; }
                }
            }
            if !ok { break; }
            // Successfully read this nested class block.
            blocks.push(PaseqClassBlock {
                class_name,
                fields,
            });
            offset = try_offset;
        }
        Ok(blocks)
    }
}

/// Convenience output for [`TypedPaseqFile::all_class_blocks`].
#[derive(Debug, Clone)]
pub struct PaseqClassBlock {
    pub class_name: String,
    pub fields: Vec<PaseqFieldDef>,
}

/// Walk every class block in `opaque_body` (outer + linearly-following
/// nested classes) and return the byte offset where the walker stopped.
/// Bytes from this offset to the end of `opaque_body` constitute the
/// "value section" — actual field values for the schema declared
/// before. Sister to [`TypedPaseqFile::all_class_blocks`].
pub fn parse_all_class_blocks_consumed(opaque_body: &[u8], _root_class_name: &str) -> io::Result<usize> {
    if opaque_body.len() < 2 {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof,
            "opaque_body too short"));
    }
    let mut offset = parse_outer_fields_consumed(opaque_body)?;
    while offset + 4 <= opaque_body.len() {
        let len = u32::from_le_bytes([
            opaque_body[offset], opaque_body[offset + 1],
            opaque_body[offset + 2], opaque_body[offset + 3],
        ]) as usize;
        if !(2..=64).contains(&len) { break; }
        let str_start = offset + 4;
        if str_start + len + 2 > opaque_body.len() { break; }
        let name_bytes = &opaque_body[str_start..str_start + len];
        if !name_bytes.iter().all(|&b| b.is_ascii_alphanumeric() || b == b'_') {
            break;
        }
        let mut try_offset = str_start + len;
        if try_offset + 2 > opaque_body.len() { break; }
        let field_count = u16::from_le_bytes([
            opaque_body[try_offset], opaque_body[try_offset + 1],
        ]) as usize;
        try_offset += 2;
        if field_count > 256 { break; }
        let mut ok = true;
        for _ in 0..field_count {
            let r: io::Result<()> = (|| {
                let _ = read_cstring(opaque_body, &mut try_offset)?;
                let _ = read_cstring(opaque_body, &mut try_offset)?;
                if try_offset + 8 > opaque_body.len() {
                    return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "meta"));
                }
                try_offset += 8;
                Ok(())
            })();
            if r.is_err() { ok = false; break; }
        }
        if !ok { break; }
        offset = try_offset;
    }
    Ok(offset)
}

/// Same walk as [`parse_outer_fields`] but returns the byte offset where
/// the walker stopped (i.e., the first byte after the outer field list).
pub fn parse_outer_fields_consumed(opaque_body: &[u8]) -> io::Result<usize> {
    if opaque_body.len() < 2 {
        return Err(io::Error::new(io::ErrorKind::UnexpectedEof,
            "opaque_body too short for u16 field_count"));
    }
    let field_count = u16::from_le_bytes([opaque_body[0], opaque_body[1]]) as usize;
    let mut offset = 2usize;
    for _ in 0..field_count {
        let _ = read_cstring(opaque_body, &mut offset)?;
        let _ = read_cstring(opaque_body, &mut offset)?;
        if offset + 8 > opaque_body.len() {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "meta"));
        }
        offset += 8;
    }
    Ok(offset)
}

// ── Tier 1 typed reader ──────────────────────────────────────────────

/// Maximum offset to scan when looking for the root class-name CString.
/// In practice the class name lives within the first 64 bytes of every
/// vanilla `.paseq` file regardless of which header variant the engine
/// chose (different `magic` byte at `+5` produces different header
/// lengths). Scanning past this would risk false positives if a
/// CString-like pattern happened to exist in the value section.
const PASEQ_HEADER_SCAN_MAX: usize = 64;

/// Typed `.paseq` file — Tier 1 outer-shell view.
///
/// Exposes the variable-length pre-class-name header (preserved
/// verbatim) and the root class name as field-level addressable. The
/// recursive schema and the field-value section are preserved as
/// opaque bytes for now.
///
/// `.paseq` has at least two distinct header layouts identified by
/// the magic byte at `+5`: `0x42` produces a 0x24-byte header before
/// the class-name CString; `0x2C` produces a 0x1C-byte header. To stay
/// robust across both, the parser scans for the `"Sequencer"` CString
/// (the canonical root class name for every vanilla `.paseq`) within
/// the first 64 bytes and splits there.
///
/// Round-trips byte-exact via `parse`/`to_bytes` for every vanilla
/// sample.
#[derive(Debug)]
pub struct TypedPaseqFile<'a> {
    pub header: Vec<u8>,
    pub class_name: CString<'a>,
    pub opaque_body: Vec<u8>,
}

impl<'a> TypedPaseqFile<'a> {
    /// Parse an entire `.paseq` file. Locates the `"Sequencer"` CString
    /// and splits the file into (`header`, `class_name`, `opaque_body`).
    pub fn parse(data: &'a [u8]) -> io::Result<Self> {
        // Find a u32-prefixed CString within the header scan window
        // whose payload is valid printable ASCII (the root class
        // name). We accept ANY such match — historically every vanilla
        // sample names "Sequencer" but keeping the parse generic
        // tolerates the engine introducing a renamed root class.
        let scan_max = data.len().min(PASEQ_HEADER_SCAN_MAX);
        let mut split_at: Option<usize> = None;
        for i in 0x10..scan_max.saturating_sub(4) {
            // Read candidate u32 length at offset i.
            let len = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
            // Reasonable class-name length: at least 4 chars, fits.
            if !(4..=64).contains(&len) {
                continue;
            }
            let str_start = i + 4;
            let str_end = str_start.checked_add(len as usize)
                .filter(|&e| e <= data.len() && e <= scan_max);
            let Some(str_end) = str_end else { continue };
            let bytes = &data[str_start..str_end];
            // Class names are pure printable ASCII (no spaces, no
            // control chars). Reject candidates that don't match.
            if bytes.iter().all(|&b| b.is_ascii_alphanumeric() || b == b'_') {
                split_at = Some(i);
                break;
            }
        }
        let split_at = split_at.ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "TypedPaseqFile: no class-name CString found in first {} bytes",
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
                "TypedPaseqFile: body offset past data length",
            ))?
            .to_vec();
        Ok(Self { header, class_name, opaque_body })
    }

    /// Serialize back to the on-disk byte layout. Always round-trips
    /// byte-exact against `parse`.
    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut out = Vec::new();
        self.write_to(&mut out)?;
        Ok(out)
    }
}

impl<'a> BinaryWrite for TypedPaseqFile<'a> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.header)?;
        self.class_name.write_to(w)?;
        w.write_all(&self.opaque_body)?;
        Ok(())
    }
}

impl<'a> ToJsonValue for TypedPaseqFile<'a> {
    fn to_json_value(&self) -> Value {
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
        let mut m = Map::new();
        m.insert(
            "header_b64".into(),
            Value::String(B64.encode(&self.header)),
        );
        m.insert(
            "class_name".into(),
            Value::String(self.class_name.data.to_string()),
        );
        m.insert(
            "opaque_body_b64".into(),
            Value::String(B64.encode(&self.opaque_body)),
        );
        Value::Object(m)
    }
}

impl<'a> WriteJsonValue for TypedPaseqFile<'a> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "TypedPaseqFile: expected object",
        ))?;
        let header_b64 = json_get_field(obj, "header_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPaseqFile.header_b64: expected string",
            ))?;
        let header = B64.decode(header_b64).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("TypedPaseqFile.header_b64: invalid base64: {}", e),
        ))?;
        w.extend_from_slice(&header);
        let class_name = json_get_field(obj, "class_name")?
            .as_str()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPaseqFile.class_name: expected string",
            ))?;
        let cn_bytes = class_name.as_bytes();
        if cn_bytes.len() > u32::MAX as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPaseqFile.class_name: too long for u32 length prefix",
            ));
        }
        w.extend_from_slice(&(cn_bytes.len() as u32).to_le_bytes());
        w.extend_from_slice(cn_bytes);
        let body_b64 = json_get_field(obj, "opaque_body_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPaseqFile.opaque_body_b64: expected base64 string",
            ))?;
        let body = B64.decode(body_b64).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("TypedPaseqFile.opaque_body_b64: invalid base64: {}", e),
        ))?;
        w.extend_from_slice(&body);
        Ok(())
    }
}

// ── Decoded | Raw safety wrapper ─────────────────────────────────────

/// Safe wrapper that always round-trips, even when the typed reader
/// fails (e.g. truncated file).
#[derive(Debug)]
pub enum PaseqFileSafe<'a> {
    Decoded(TypedPaseqFile<'a>),
    Raw {
        bytes: Vec<u8>,
        decode_error: String,
    },
}

impl<'a> PaseqFileSafe<'a> {
    pub fn parse(data: &'a [u8]) -> io::Result<Self> {
        match TypedPaseqFile::parse(data) {
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

// ── Unit tests (Session 33) ──────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a `u32 length + bytes` CString in a Vec<u8>.
    fn lp(s: &str) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&(s.len() as u32).to_le_bytes());
        out.extend_from_slice(s.as_bytes());
        out
    }

    #[test]
    fn walk_finds_back_to_back_strings() {
        let mut data = Vec::new();
        data.extend_from_slice(&lp("HELLO"));
        data.extend_from_slice(&lp("WORLD"));
        let strings = walk_u32_prefixed_strings(&data, 0);
        assert_eq!(strings.len(), 2);
        assert_eq!(strings[0].1, "HELLO");
        assert_eq!(strings[1].1, "WORLD");
        // First string at offset 0, second after `4 + 5` bytes.
        assert_eq!(strings[0].0, 0);
        assert_eq!(strings[1].0, 9);
    }

    #[test]
    fn walk_skips_non_printable_bytes() {
        // u32 = 5, then 5 non-printable bytes — should be skipped.
        let mut data = vec![5u8, 0, 0, 0, 0xff, 0xfe, 0x01, 0x02, 0x03];
        // Append a real string after.
        data.extend_from_slice(&lp("REAL"));
        let strings = walk_u32_prefixed_strings(&data, 0);
        let values: Vec<&str> = strings.iter().map(|(_, s)| s.as_str()).collect();
        assert!(values.contains(&"REAL"), "expected 'REAL' in {:?}", values);
    }

    #[test]
    fn walk_respects_length_bounds() {
        // u32 = 4096 (max valid), but only 4 bytes follow — should skip.
        let data = vec![0x00, 0x10, 0x00, 0x00, b'A', b'B', b'C', b'D'];
        let strings = walk_u32_prefixed_strings(&data, 0);
        // Should not match — declared len exceeds available bytes.
        assert!(strings.is_empty() || strings.iter().all(|(_, s)| s != "ABCD"));
    }

    #[test]
    fn walk_base_offset_added_to_each_match() {
        let data = lp("FOO");
        let strings = walk_u32_prefixed_strings(&data, 1000);
        assert_eq!(strings.len(), 1);
        assert_eq!(strings[0].0, 1000);
        assert_eq!(strings[0].1, "FOO");
    }

    #[test]
    fn replace_cstring_length_flexible_grow() {
        let data = lp("OLD");
        let modified = replace_cstring_at(&data, 0, Some("OLD"), "NEW_LONGER")
            .expect("replace ok");
        // Length prefix updated to 10
        assert_eq!(u32::from_le_bytes([
            modified[0], modified[1], modified[2], modified[3]
        ]), 10);
        // String content
        assert_eq!(&modified[4..14], b"NEW_LONGER");
        // File grew by 7 bytes (10 - 3)
        assert_eq!(modified.len(), data.len() + 7);
    }

    #[test]
    fn replace_cstring_length_flexible_shrink() {
        let mut data = lp("LONG_VALUE");
        data.extend_from_slice(b"\xff\xff\xff");  // tail bytes
        let modified = replace_cstring_at(&data, 0, Some("LONG_VALUE"), "X")
            .expect("replace ok");
        assert_eq!(u32::from_le_bytes([
            modified[0], modified[1], modified[2], modified[3]
        ]), 1);
        assert_eq!(&modified[4..5], b"X");
        // Tail bytes preserved
        assert_eq!(&modified[5..8], &[0xff, 0xff, 0xff]);
        // File shrank by 9 bytes
        assert_eq!(modified.len(), data.len() - 9);
    }

    #[test]
    fn replace_cstring_validates_expected() {
        let data = lp("ACTUAL");
        let err = replace_cstring_at(&data, 0, Some("WRONG"), "NEW")
            .expect_err("must fail on expected mismatch");
        assert!(format!("{}", err).contains("expected"),
            "error should mention expected: {}", err);
    }

    #[test]
    fn replace_cstring_skips_check_when_expected_is_none() {
        let data = lp("WHATEVER");
        let modified = replace_cstring_at(&data, 0, None, "NEW")
            .expect("replace ok with None expected_value");
        assert_eq!(&modified[4..7], b"NEW");
    }

    #[test]
    fn replace_cstring_rejects_out_of_bounds_offset() {
        let data = lp("ABC");
        let err = replace_cstring_at(&data, data.len() + 100, None, "X")
            .expect_err("offset past end of file must fail");
        assert!(format!("{}", err).contains("exceeds"),
            "error should mention bounds: {}", err);
    }

    #[test]
    fn replace_cstring_rejects_corrupt_length() {
        // u32 length says 100 but only 4 bytes follow.
        let data = vec![100u8, 0, 0, 0, b'A', b'B', b'C', b'D'];
        let err = replace_cstring_at(&data, 0, None, "X")
            .expect_err("corrupt length must fail");
        assert!(format!("{}", err).contains("exceeds"),
            "error should mention exceeds: {}", err);
    }

    #[test]
    fn parse_outer_fields_empty_when_zero_count() {
        // u16 field_count = 0 — should return empty vec without consuming
        // anything past the count.
        let data = [0u8, 0];
        let fields = parse_outer_fields(&data).expect("zero-count parse ok");
        assert!(fields.is_empty());
        let consumed = parse_outer_fields_consumed(&data).expect("consumed ok");
        assert_eq!(consumed, 2);
    }

    #[test]
    fn parse_outer_fields_rejects_short_input() {
        // Only 1 byte — not enough for u16 field_count.
        let data = [0u8];
        let err = parse_outer_fields(&data).expect_err("short input must fail");
        assert!(format!("{}", err).contains("too short"),
            "error should mention too short: {}", err);
    }

    #[test]
    fn parse_outer_fields_rejects_truncated_field() {
        // u16 field_count = 1, then a CString-length prefix that
        // exceeds available bytes.
        let mut data = vec![1u8, 0];
        // Field name CString claims 100 bytes but only 4 follow.
        data.extend_from_slice(&100u32.to_le_bytes());
        data.extend_from_slice(b"_x_y");
        let err = parse_outer_fields(&data)
            .expect_err("truncated field must fail");
        assert!(format!("{}", err).contains("field 0"),
            "error should reference field index: {}", err);
    }

    #[test]
    fn parse_outer_fields_handles_many_fields() {
        // 15 fields (matching the canonical Sequencer schema size).
        let mut data = Vec::new();
        data.extend_from_slice(&15u16.to_le_bytes());
        for i in 0..15 {
            let name = format!("_field_{:02}", i);
            data.extend_from_slice(&lp(&name));
            data.extend_from_slice(&lp("int32"));
            data.extend_from_slice(&[0u8; 8]);
        }
        let fields = parse_outer_fields(&data).expect("many fields ok");
        assert_eq!(fields.len(), 15);
        assert_eq!(fields[0].field_name, "_field_00");
        assert_eq!(fields[14].field_name, "_field_14");
        for f in &fields {
            assert_eq!(f.type_name, "int32");
            assert_eq!(f.type_meta, [0u8; 8]);
        }
    }

    #[test]
    fn parse_outer_fields_consumed_matches_walker_position() {
        // Build a fake outer field list: u16 field_count + N fields.
        let mut data = Vec::new();
        data.extend_from_slice(&2u16.to_le_bytes());  // field_count = 2
        // Field 0: name, type, 8-byte meta
        data.extend_from_slice(&lp("_field_a"));
        data.extend_from_slice(&lp("int32"));
        data.extend_from_slice(&[0u8; 8]);
        // Field 1
        data.extend_from_slice(&lp("_field_b"));
        data.extend_from_slice(&lp("bool"));
        data.extend_from_slice(&[0u8; 8]);

        let consumed = parse_outer_fields_consumed(&data).expect("parse ok");
        assert_eq!(consumed, data.len());

        let fields = parse_outer_fields(&data).expect("parse ok");
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].field_name, "_field_a");
        assert_eq!(fields[0].type_name, "int32");
        assert_eq!(fields[1].field_name, "_field_b");
        assert_eq!(fields[1].type_name, "bool");
    }
}
