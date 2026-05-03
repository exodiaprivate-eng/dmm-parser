// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

use std::io::{self, Write};

use serde_json::{Map, Value};

use super::{BinaryRead, BinaryWrite, CString, check_remaining};

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
    let file = LocalizationFile::parse(data)?;
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

/// Inverse of `parse_paloc_to_json`: write a JSON array back to bytes.
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
