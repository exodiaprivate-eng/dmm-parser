// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Generic Tier 1.5 tokenizer shared by sequencer-family binaries
//! (`.paseq`, `.pastage`, `.paseqc`, `.paschedule`, `.paschedulepath`).
//!
//! All five formats interleave length-prefixed UTF-8 strings (state
//! names, condition expressions, asset paths) with raw binary fields
//! (counts, flags, transition tables, hashes, sentinels). The
//! tokenizer captures both as an ordered stream; round-trip is
//! byte-exact, and the JSON view exposes every string as an
//! addressable mod-editable field.
//!
//! A future Tier 1 pass — once the engine's tagged-field reflection is
//! decoded — will re-classify the `RawBytes` runs into typed numeric
//! fields. Until then this layer ships the bulk of useful field-level
//! mod intents (renames, expression edits, path swaps) while preserving
//! every byte the engine cares about.

use std::io::{self, Write};

use serde_json::{Map, Value};

const MIN_STR_LEN: usize = 1;
const MAX_STR_LEN: usize = 1024;

/// One token of the file. Order across the `Vec<Token>` matters for
/// round-trip; bytes are emitted in the same order they were read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// `u32 length` prefix + that many printable-ASCII bytes. The length
    /// is recomputed from the body on serialize, so authors can edit
    /// `value` to any length without thinking about prefixes.
    LpString(Vec<u8>),
    /// Unclassified byte run (numeric counts, flags, hashes, sentinels).
    /// Round-tripped verbatim.
    RawBytes(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LpTokenFile {
    pub tokens: Vec<Token>,
}

#[inline]
fn is_string_body_byte(b: u8) -> bool {
    // Printable ASCII (incl. space) is our allow-list. Reject NUL,
    // control chars, and high bytes outright. The format does not use
    // UTF-8 multi-byte chars in any sample we've inspected.
    (0x20..=0x7E).contains(&b)
}

/// If an LP-string starts at `offset`, return its content length.
/// Returns None when the bytes don't look like a string.
fn try_read_lp_string(data: &[u8], offset: usize) -> Option<usize> {
    if offset + 4 > data.len() {
        return None;
    }
    let len = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
    if len < MIN_STR_LEN || len > MAX_STR_LEN {
        return None;
    }
    let end = offset.checked_add(4)?.checked_add(len)?;
    if end > data.len() {
        return None;
    }
    let body = &data[offset + 4..end];
    if !body.iter().all(|&b| is_string_body_byte(b)) {
        return None;
    }
    Some(len)
}

impl LpTokenFile {
    pub fn parse(data: &[u8]) -> io::Result<Self> {
        let mut tokens: Vec<Token> = Vec::new();
        let mut p = 0usize;
        let mut raw_start = 0usize;

        while p < data.len() {
            if let Some(len) = try_read_lp_string(data, p) {
                if p > raw_start {
                    tokens.push(Token::RawBytes(data[raw_start..p].to_vec()));
                }
                let body_start = p + 4;
                let body_end = body_start + len;
                tokens.push(Token::LpString(data[body_start..body_end].to_vec()));
                p = body_end;
                raw_start = p;
            } else {
                p += 1;
            }
        }

        if raw_start < data.len() {
            tokens.push(Token::RawBytes(data[raw_start..].to_vec()));
        }

        Ok(LpTokenFile { tokens })
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.write_to(&mut buf)?;
        Ok(buf)
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        for tok in &self.tokens {
            match tok {
                Token::LpString(body) => {
                    let len = body.len() as u32;
                    w.write_all(&len.to_le_bytes())?;
                    w.write_all(body)?;
                }
                Token::RawBytes(bytes) => {
                    w.write_all(bytes)?;
                }
            }
        }
        Ok(())
    }

    pub fn to_json(&self) -> Value {
        let mut arr: Vec<Value> = Vec::with_capacity(self.tokens.len());
        for tok in &self.tokens {
            let mut obj = Map::new();
            match tok {
                Token::LpString(body) => {
                    obj.insert("kind".into(), Value::String("lp_string".into()));
                    match std::str::from_utf8(body) {
                        Ok(s) => obj.insert("value".into(), Value::String(s.to_string())),
                        Err(_) => obj.insert("hex".into(), Value::String(hex_encode(body))),
                    };
                }
                Token::RawBytes(bytes) => {
                    obj.insert("kind".into(), Value::String("raw_bytes".into()));
                    obj.insert("hex".into(), Value::String(hex_encode(bytes)));
                }
            }
            arr.push(Value::Object(obj));
        }
        let mut root = Map::new();
        root.insert("tokens".into(), Value::Array(arr));
        Value::Object(root)
    }

    pub fn from_json(v: &Value) -> io::Result<Self> {
        let arr = v.get("tokens")
            .and_then(|t| t.as_array())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing tokens array"))?;
        let mut tokens = Vec::with_capacity(arr.len());
        for (i, entry) in arr.iter().enumerate() {
            let obj = entry.as_object().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData,
                    format!("token[{}] not an object", i))
            })?;
            let kind = obj.get("kind").and_then(|k| k.as_str()).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData,
                    format!("token[{}] missing kind", i))
            })?;
            match kind {
                "lp_string" => {
                    let body = if let Some(s) = obj.get("value").and_then(|v| v.as_str()) {
                        s.as_bytes().to_vec()
                    } else if let Some(h) = obj.get("hex").and_then(|v| v.as_str()) {
                        hex_decode(h).map_err(|e| io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("token[{}] hex decode: {}", i, e),
                        ))?
                    } else {
                        return Err(io::Error::new(io::ErrorKind::InvalidData,
                            format!("token[{}] lp_string missing value/hex", i)));
                    };
                    tokens.push(Token::LpString(body));
                }
                "raw_bytes" => {
                    let h = obj.get("hex").and_then(|v| v.as_str()).ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidData,
                            format!("token[{}] raw_bytes missing hex", i))
                    })?;
                    let bytes = hex_decode(h).map_err(|e| io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("token[{}] hex decode: {}", i, e),
                    ))?;
                    tokens.push(Token::RawBytes(bytes));
                }
                other => {
                    return Err(io::Error::new(io::ErrorKind::InvalidData,
                        format!("token[{}] unknown kind '{}'", i, other)));
                }
            }
        }
        Ok(LpTokenFile { tokens })
    }
}

// ── Hex helpers (no external crate) ─────────────────────────────────────────

pub(crate) fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xF) as usize] as char);
    }
    s
}

pub(crate) fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    if s.len() % 2 != 0 {
        return Err(format!("odd-length hex string ({} chars)", s.len()));
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_nibble(bytes[i])?;
        let lo = hex_nibble(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

fn hex_nibble(c: u8) -> Result<u8, String> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(format!("invalid hex char '{}'", c as char)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lp_string_roundtrip() {
        let bytes = [0x05, 0x00, 0x00, 0x00, b'S', b'T', b'A', b'R', b'T'];
        let f = LpTokenFile::parse(&bytes).unwrap();
        assert_eq!(f.tokens.len(), 1);
        assert_eq!(f.tokens[0], Token::LpString(b"START".to_vec()));
        assert_eq!(f.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn zero_magic_header() {
        let bytes = [0x00, 0x00, 0x00, 0x00, 0x42, 0x00, 0x00, 0x00];
        let f = LpTokenFile::parse(&bytes).unwrap();
        assert_eq!(f.tokens.len(), 1);
        assert_eq!(f.tokens[0], Token::RawBytes(bytes.to_vec()));
        assert_eq!(f.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn paseqc_magic_then_string() {
        // FF FF 04 00 (paseqc magic) then a LP-string
        let mut bytes = vec![0xFF, 0xFF, 0x04, 0x00];
        bytes.extend_from_slice(&[0x05, 0x00, 0x00, 0x00, b'F', b'I', b'R', b'S', b'T']);
        let f = LpTokenFile::parse(&bytes).unwrap();
        assert_eq!(f.tokens.len(), 2);
        assert_eq!(f.tokens[0], Token::RawBytes(vec![0xFF, 0xFF, 0x04, 0x00]));
        assert_eq!(f.tokens[1], Token::LpString(b"FIRST".to_vec()));
        assert_eq!(f.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn json_roundtrip() {
        let bytes = [
            0x00, 0x00, 0x00, 0x00, 0x42, 0x00, 0x00, 0x00,
            0x05, 0x00, 0x00, 0x00, b'F', b'I', b'R', b'S', b'T',
            0xAB, 0xCD,
        ];
        let f = LpTokenFile::parse(&bytes).unwrap();
        let j = f.to_json();
        let f2 = LpTokenFile::from_json(&j).unwrap();
        assert_eq!(f, f2);
        assert_eq!(f2.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn hex_roundtrip() {
        let raw = vec![0x00, 0x01, 0xFF, 0xAB, 0xCD];
        assert_eq!(hex_decode(&hex_encode(&raw)).unwrap(), raw);
    }
}
