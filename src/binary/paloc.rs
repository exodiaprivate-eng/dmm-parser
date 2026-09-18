// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

use std::borrow::Cow;
use std::io::{self, Write};

use serde_json::{Map, Value};

use super::{BinaryRead, BinaryWrite, CString, check_remaining};

// ── 2.03.00 container ──────────────────────────────────────────────────────
//
// Game 2.03.00 (2026-09-17) wraps every string table in a 512-byte header
// followed by one raw LZ4 block; the block decompresses to the record format
// below, unchanged. Measured on 0020/item.paloc: 565,947 compressed bytes at
// offset 512 inflate to 1,552,634 and start with the same `07 00 00 00`
// entry the 2.02 file started with. Header layout (little-endian):
//
//   0   "paloc\0"        6 bytes
//   6   zero             3 bytes
//   9   compressed size  u32 (= file length - 512 on every vanilla file)
//   13  decompressed size u32
//   17  zero             to 512
//
// Reads accept both shapes; writes copy the shape of the bytes they replace
// (`serialize_paloc_like`), because the game version, not the content,
// decides which one it will load.

const CONTAINER_MAGIC: &[u8; 6] = b"paloc\0";
pub const CONTAINER_HEADER_LEN: usize = 512;

/// True when `data` is a 2.03.00 container rather than bare records.
pub fn is_container(data: &[u8]) -> bool {
    data.len() >= CONTAINER_HEADER_LEN && data.starts_with(CONTAINER_MAGIC)
}

/// The record bytes: `data` itself for a bare file, the inflated block for a
/// container.
pub fn unwrap_container(data: &[u8]) -> io::Result<Cow<'_, [u8]>> {
    if !is_container(data) {
        return Ok(Cow::Borrowed(data));
    }
    let csize = u32::from_le_bytes(data[9..13].try_into().unwrap()) as usize;
    let dsize = u32::from_le_bytes(data[13..17].try_into().unwrap()) as usize;
    let end = CONTAINER_HEADER_LEN.checked_add(csize).ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData, "paloc container: compressed size overflows",
    ))?;
    if data.len() < end {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "paloc container: header says {} compressed bytes at offset {} but the file is {} bytes",
                csize, CONTAINER_HEADER_LEN, data.len(),
            ),
        ));
    }
    let block = &data[CONTAINER_HEADER_LEN..end];
    let out = lz4_flex::block::decompress(block, dsize).map_err(|e| io::Error::new(
        io::ErrorKind::InvalidData,
        format!("paloc container: LZ4 block does not inflate to {} bytes: {}", dsize, e),
    ))?;
    Ok(Cow::Owned(out))
}

/// Wrap bare record bytes in the 2.03.00 container.
pub fn wrap_container(records: &[u8]) -> Vec<u8> {
    let block = lz4_flex::block::compress(records);
    let mut out = vec![0u8; CONTAINER_HEADER_LEN];
    out[..CONTAINER_MAGIC.len()].copy_from_slice(CONTAINER_MAGIC);
    out[9..13].copy_from_slice(&(block.len() as u32).to_le_bytes());
    out[13..17].copy_from_slice(&(records.len() as u32).to_le_bytes());
    out.extend_from_slice(&block);
    out
}

/// Give `records` the shape of `template`: wrapped when the template is a
/// container, bare otherwise.
pub fn shape_like(records: Vec<u8>, template: &[u8]) -> Vec<u8> {
    if is_container(template) { wrap_container(&records) } else { records }
}

// ── Localization Entry ─────────────────────────────────────────────────────

#[derive(Debug)]
pub struct LocalizationEntry<'a> {
    pub unk_id: u64,
    pub string_key: CString<'a>,
    pub string_value: CString<'a>,
}

impl<'a> BinaryRead<'a> for LocalizationEntry<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        Ok(LocalizationEntry {
            unk_id: u64::read_from(data, offset)?,
            string_key: CString::read_from(data, offset)?,
            string_value: CString::read_from(data, offset)?,
        })
    }
}

impl BinaryWrite for LocalizationEntry<'_> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.unk_id.write_to(w)?;
        self.string_key.write_to(w)?;
        self.string_value.write_to(w)
    }
}

// ── Localization File ──────────────────────────────────────────────────────

#[derive(Debug)]
pub struct LocalizationFile<'a> {
    pub entries: Vec<LocalizationEntry<'a>>,
}

impl<'a> LocalizationFile<'a> {
    pub fn parse(data: &'a [u8]) -> io::Result<Self> {
        check_remaining(data, 0, 4)?;
        let count_offset = data.len() - 4;
        let entry_count = u32::from_le_bytes(
            data[count_offset..].try_into().unwrap(),
        ) as usize;

        let mut offset = 0;
        let mut entries = Vec::with_capacity(entry_count);
        for _ in 0..entry_count {
            entries.push(LocalizationEntry::read_from(data, &mut offset)?);
        }

        if offset != count_offset {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "entry data ends at 0x{:X} but expected 0x{:X} (before trailing count)",
                    offset, count_offset,
                ),
            ));
        }

        Ok(LocalizationFile { entries })
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        for entry in &self.entries {
            entry.write_to(&mut buf)?;
        }
        (self.entries.len() as u32).write_to(&mut buf)?;
        Ok(buf)
    }
}

// ── JSON Surface ──────────────────────────────────────────────────────────
//
// JSON form: `[{"category": <u8>, "key": <string>, "value": <string>}, ...]`
//
// `unk_id` is a u64 where the upper 7 bytes are always zero; only the low
// byte (`category`) is meaningful. The JSON form exposes just `category` for
// cleanliness and reconstructs the full u64 with zero padding on serialize.

/// Parse paloc bytes into a JSON array. One entry per record.
///
/// Returns `Vec<{category: u8, key: String, value: String}>`.
pub fn parse_paloc_to_json(data: &[u8]) -> io::Result<Vec<Value>> {
    let raw = unwrap_container(data)?;
    let file = LocalizationFile::parse(&raw)?;
    let mut out = Vec::with_capacity(file.entries.len());
    for entry in &file.entries {
        // Validate that upper 7 bytes are zero — else the file uses category
        // codes we don't understand yet and we want to fail loudly.
        if entry.unk_id >> 8 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "paloc entry has non-zero upper bytes in category u64 (0x{:016x}); \
                     update the parser to handle this variant",
                    entry.unk_id,
                ),
            ));
        }
        let mut obj = Map::new();
        obj.insert("category".to_string(), Value::Number((entry.unk_id as u8).into()));
        obj.insert("key".to_string(), Value::String(entry.string_key.data.to_string()));
        obj.insert("value".to_string(), Value::String(entry.string_value.data.to_string()));
        out.push(Value::Object(obj));
    }
    Ok(out)
}

/// Inverse of `parse_paloc_to_json`: write a JSON array back to BARE record
/// bytes (no 2.03.00 container). A caller replacing a game file uses
/// `serialize_paloc_like` so the file keeps the shape the game version reads.
///
/// Each value must be an object with shape `{category: u8, key: string, value: string}`.
pub fn serialize_paloc_from_json(items: &[Value]) -> io::Result<Vec<u8>> {
    let mut buf = Vec::with_capacity(items.len() * 64);
    for (i, v) in items.iter().enumerate() {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("entry[{}]: expected object, got {:?}", i, v),
        ))?;

        let category = obj.get("category")
            .and_then(|c| c.as_u64())
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                format!("entry[{}]: missing or non-integer 'category'", i),
            ))?;
        if category > u8::MAX as u64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("entry[{}]: category {} exceeds u8 range", i, category),
            ));
        }
        let key = obj.get("key")
            .and_then(|k| k.as_str())
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                format!("entry[{}]: missing or non-string 'key'", i),
            ))?;
        let value = obj.get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                format!("entry[{}]: missing or non-string 'value'", i),
            ))?;

        // Reconstruct u64 with category in low byte, upper 7 bytes zero
        category.write_to(&mut buf)?;
        // Write key (u32 len + bytes, no null terminator — matches CString)
        (key.len() as u32).write_to(&mut buf)?;
        buf.extend_from_slice(key.as_bytes());
        (value.len() as u32).write_to(&mut buf)?;
        buf.extend_from_slice(value.as_bytes());
    }
    // Trailing entry count
    (items.len() as u32).write_to(&mut buf)?;
    Ok(buf)
}

/// `serialize_paloc_from_json`, then the container when `template` (the bytes
/// being replaced) is one. The game version decides the shape, not the content.
pub fn serialize_paloc_like(items: &[Value], template: &[u8]) -> io::Result<Vec<u8>> {
    Ok(shape_like(serialize_paloc_from_json(items)?, template))
}

#[cfg(test)]
mod container_tests {
    use super::*;

    fn records() -> Vec<u8> {
        LocalizationFile { entries: vec![
            LocalizationEntry { unk_id: 7, string_key: CString { length: 6, data: "262897", raw: b"262897" },
                                string_value: CString { length: 6, data: "Copper", raw: b"Copper" } },
            LocalizationEntry { unk_id: 3, string_key: CString { length: 1, data: "x", raw: b"x" },
                                string_value: CString { length: 1, data: "y", raw: b"y" } },
        ]}.to_bytes().unwrap()
    }

    #[test]
    fn bare_bytes_pass_through_untouched() {
        let raw = records();
        assert!(!is_container(&raw));
        assert!(matches!(unwrap_container(&raw).unwrap(), Cow::Borrowed(_)));
        assert_eq!(shape_like(raw.clone(), &raw), raw);
    }

    #[test]
    fn wrap_then_unwrap_returns_the_records_and_the_header_is_as_measured() {
        let raw = records();
        let wrapped = wrap_container(&raw);
        assert!(is_container(&wrapped));
        assert_eq!(&wrapped[..6], b"paloc\0");
        assert_eq!(&wrapped[6..9], &[0, 0, 0]);
        let csize = u32::from_le_bytes(wrapped[9..13].try_into().unwrap()) as usize;
        let dsize = u32::from_le_bytes(wrapped[13..17].try_into().unwrap()) as usize;
        assert_eq!(csize, wrapped.len() - CONTAINER_HEADER_LEN);
        assert_eq!(dsize, raw.len());
        assert!(wrapped[17..CONTAINER_HEADER_LEN].iter().all(|b| *b == 0));
        assert_eq!(unwrap_container(&wrapped).unwrap().as_ref(), raw.as_slice());
        // the JSON surface sees through the container both ways
        let items = parse_paloc_to_json(&wrapped).unwrap();
        assert_eq!(items.len(), 2);
        let back = serialize_paloc_like(&items, &wrapped).unwrap();
        assert!(is_container(&back));
        assert_eq!(unwrap_container(&back).unwrap().as_ref(), raw.as_slice());
        assert_eq!(serialize_paloc_like(&items, &raw).unwrap(), raw);
    }

    #[test]
    fn a_short_container_is_refused_with_the_sizes_named() {
        let mut wrapped = wrap_container(&records());
        wrapped.truncate(CONTAINER_HEADER_LEN + 3);
        let err = unwrap_container(&wrapped).unwrap_err().to_string();
        assert!(err.contains("compressed bytes"), "{err}");
    }
}

#[cfg(test)]
mod json_tests {
    use super::*;
    use serde_json::json;

    fn build(entries: Vec<LocalizationEntry>) -> Vec<u8> {
        LocalizationFile { entries }.to_bytes().unwrap()
    }

    #[test]
    fn roundtrip_synthetic() {
        // Build a synthetic paloc with two entries, parse to JSON, serialize back.
        let bytes = build(vec![
            LocalizationEntry {
                unk_id: 0x70,
                string_key: CString { length: 10, data: "4294967408", raw: b"4294967408" },
                string_value: CString { length: 6, data: "Copper", raw: b"Copper" },
            },
            LocalizationEntry {
                unk_id: 0x07,
                string_key: CString { length: 6, data: "262897", raw: b"262897" },
                string_value: CString { length: 26, data: "Unavailable during combat.", raw: b"Unavailable during combat." },
            },
        ]);

        let json_array = parse_paloc_to_json(&bytes).unwrap();
        assert_eq!(json_array.len(), 2);
        assert_eq!(json_array[0]["category"], 0x70);
        assert_eq!(json_array[0]["key"], "4294967408");
        assert_eq!(json_array[0]["value"], "Copper");

        let written = serialize_paloc_from_json(&json_array).unwrap();
        assert_eq!(written, bytes, "JSON round-trip should be byte-perfect");
    }

    #[test]
    fn empty_file_roundtrip() {
        // Zero entries — just the trailing u32 count of 0.
        let bytes = build(vec![]);
        let json_array = parse_paloc_to_json(&bytes).unwrap();
        assert_eq!(json_array.len(), 0);
        assert_eq!(serialize_paloc_from_json(&json_array).unwrap(), bytes);
    }

    #[test]
    fn empty_strings_allowed() {
        // Both key and value can be empty strings (length 0).
        let bytes = build(vec![
            LocalizationEntry {
                unk_id: 0x07,
                string_key: CString { length: 0, data: "", raw: b"" },
                string_value: CString { length: 0, data: "", raw: b"" },
            },
        ]);
        let json_array = parse_paloc_to_json(&bytes).unwrap();
        assert_eq!(json_array.len(), 1);
        assert_eq!(json_array[0]["key"], "");
        assert_eq!(json_array[0]["value"], "");
        assert_eq!(serialize_paloc_from_json(&json_array).unwrap(), bytes);
    }

    #[test]
    fn long_value_64k() {
        // Values can be very long. Localization for some items is paragraph-length.
        let long_value = "A".repeat(64 * 1024);
        let bytes = build(vec![
            LocalizationEntry {
                unk_id: 0x71,
                string_key: CString { length: 6, data: "999001", raw: b"999001" },
                string_value: CString { length: long_value.len() as u32, data: &long_value, raw: long_value.as_bytes() },
            },
        ]);
        let json_array = parse_paloc_to_json(&bytes).unwrap();
        assert_eq!(json_array[0]["value"].as_str().unwrap().len(), 64 * 1024);
        assert_eq!(serialize_paloc_from_json(&json_array).unwrap(), bytes);
    }

    #[test]
    fn unicode_korean_roundtrip() {
        // Korean characters — 3 bytes each in UTF-8. The CString length is byte
        // count, not character count.
        let kor = "안녕하세요"; // "Hello" in Korean
        let bytes = build(vec![
            LocalizationEntry {
                unk_id: 0x07,
                string_key: CString { length: 6, data: "262897", raw: b"262897" },
                string_value: CString { length: kor.len() as u32, data: kor, raw: kor.as_bytes() },
            },
        ]);
        let json_array = parse_paloc_to_json(&bytes).unwrap();
        assert_eq!(json_array[0]["value"], kor);
        assert_eq!(serialize_paloc_from_json(&json_array).unwrap(), bytes);
    }

    #[test]
    fn unicode_emoji_and_mixed_scripts() {
        // Emoji + Latin + CJK in one value (modern game UI text).
        let mixed = "Sword ⚔️ 名刀 «Excalibur»";
        let bytes = build(vec![
            LocalizationEntry {
                unk_id: 0x70,
                string_key: CString { length: 10, data: "4294967408", raw: b"4294967408" },
                string_value: CString { length: mixed.len() as u32, data: mixed, raw: mixed.as_bytes() },
            },
        ]);
        let json_array = parse_paloc_to_json(&bytes).unwrap();
        assert_eq!(json_array[0]["value"], mixed);
        assert_eq!(serialize_paloc_from_json(&json_array).unwrap(), bytes);
    }

    #[test]
    fn max_category_byte_0xff() {
        // Even if no production paloc uses category 0xFF, the parser should not
        // arbitrarily reject high u8 values (only > u8::MAX is invalid).
        let bytes = build(vec![
            LocalizationEntry {
                unk_id: 0xFF,
                string_key: CString { length: 1, data: "x", raw: b"x" },
                string_value: CString { length: 1, data: "y", raw: b"y" },
            },
        ]);
        let json_array = parse_paloc_to_json(&bytes).unwrap();
        assert_eq!(json_array[0]["category"], 0xFF);
        assert_eq!(serialize_paloc_from_json(&json_array).unwrap(), bytes);
    }

    #[test]
    fn rejects_non_zero_upper_bytes_in_category_u64() {
        // If a paloc file has a category u64 with non-zero upper bytes, we
        // currently fail loudly so unknown variants surface immediately.
        let mut bytes = vec![];
        // Synthesize: category u64 = 0x00FF000000000007 (non-zero upper bytes)
        bytes.extend_from_slice(&0x00FF000000000007u64.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.push(b'k');
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.push(b'v');
        bytes.extend_from_slice(&1u32.to_le_bytes()); // trailing count

        let result = parse_paloc_to_json(&bytes);
        assert!(result.is_err(), "should reject non-zero upper bytes in category u64");
        assert!(
            result.unwrap_err().to_string().contains("non-zero upper bytes"),
            "error message should mention upper bytes"
        );
    }

    #[test]
    fn rejects_oversized_category_in_json_input() {
        // serialize_paloc_from_json must reject category values > u8::MAX.
        let bad = vec![json!({"category": 256, "key": "k", "value": "v"})];
        let result = serialize_paloc_from_json(&bad);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds u8 range"));
    }

    #[test]
    fn rejects_missing_fields_in_json_input() {
        let missing_key = vec![json!({"category": 7, "value": "v"})];
        assert!(serialize_paloc_from_json(&missing_key).is_err());

        let missing_value = vec![json!({"category": 7, "key": "k"})];
        assert!(serialize_paloc_from_json(&missing_value).is_err());

        let missing_category = vec![json!({"key": "k", "value": "v"})];
        assert!(serialize_paloc_from_json(&missing_category).is_err());
    }
}
