// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.wem` — Wwise Encoded Media (Audiokinetic).
//!
//! Standard RIFF/WAVE container with Wwise-specific chunks (`fmt `,
//! `hash`, `data`, optional `akd `, `junk`, `smpl`, `cue `, `LIST`).
//!
//! Round-trip preserves every byte including the trailing pad byte
//! that RIFF inserts when a chunk size is odd.
//!
//! `fmt ` chunk holds the Wwise-Vorbis codec descriptor (sample rate,
//! channels, codec ID); modders can edit those numeric fields by
//! editing the chunk data hex. A future Tier 1.1 pass can split out
//! the fmt sub-fields as named JSON keys; for now they round-trip as
//! one opaque chunk-data blob to keep the parser format-agnostic.

use std::io::{self, Write};

use serde_json::{Map, Value};

pub const RIFF_MAGIC: u32 = 0x4646_4952; // "RIFF" LE
pub const WAVE_MAGIC: u32 = 0x4556_4157; // "WAVE" LE

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiffChunk {
    /// 4-byte ASCII chunk id, e.g. b"fmt ".
    pub id: [u8; 4],
    /// Chunk payload (the size field is recomputed on serialize).
    pub data: Vec<u8>,
    /// Whether the on-disk chunk had a 1-byte zero pad after the data.
    /// RIFF requires chunks to align to even length; if the original
    /// stored a non-zero pad byte (rare but legal) we preserve that here.
    pub pad_byte: Option<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WemFile {
    /// Form type after RIFF header. Always b"WAVE" for WEM in this game.
    pub form: [u8; 4],
    pub chunks: Vec<RiffChunk>,
    /// Bytes (if any) past the end of the last chunk that don't form a
    /// new chunk. Some PAZ-stored WEM files have padding here that we
    /// must preserve byte-exactly.
    pub trailing: Vec<u8>,
}

impl WemFile {
    pub fn parse(data: &[u8]) -> io::Result<Self> {
        if data.len() < 12 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof,
                format!("need 12 bytes for RIFF header, got {}", data.len())));
        }
        let magic = u32::from_le_bytes(data[0..4].try_into().unwrap());
        if magic != RIFF_MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("bad RIFF magic: 0x{:08X}", magic)));
        }
        // Bytes 4..8 are the RIFF chunk size (file length - 8); we
        // recompute it on serialize so we don't store it.
        let mut form = [0u8; 4];
        form.copy_from_slice(&data[8..12]);

        let mut chunks: Vec<RiffChunk> = Vec::new();
        let mut p = 12usize;
        while p + 8 <= data.len() {
            let mut id = [0u8; 4];
            id.copy_from_slice(&data[p..p + 4]);
            let chunk_size = u32::from_le_bytes(data[p + 4..p + 8].try_into().unwrap()) as usize;
            let body_start = p + 8;
            let body_end = body_start.checked_add(chunk_size).ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData, "chunk size overflow"))?;
            if body_end > data.len() {
                // Truncated chunk — preserve as trailing bytes so
                // round-trip can still emit them.
                break;
            }
            let body = data[body_start..body_end].to_vec();
            // Pad byte if size is odd
            let pad_byte = if chunk_size % 2 == 1 && body_end < data.len() {
                Some(data[body_end])
            } else {
                None
            };
            let consumed = 8 + chunk_size + if pad_byte.is_some() { 1 } else { 0 };
            chunks.push(RiffChunk { id, data: body, pad_byte });
            p = p.checked_add(consumed).ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData, "chunk advance overflow"))?;
        }

        let trailing = if p < data.len() { data[p..].to_vec() } else { Vec::new() };

        Ok(WemFile { form, chunks, trailing })
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.write_to(&mut buf)?;
        Ok(buf)
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        // Compute total RIFF payload size: 4 bytes (form) + sum(chunk
        // headers + bodies + pad bytes) + trailing bytes.
        let mut payload_size: u32 = 4;
        for c in &self.chunks {
            payload_size = payload_size.checked_add(8).ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData, "RIFF size overflow"))?;
            payload_size = payload_size.checked_add(c.data.len() as u32).ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData, "RIFF size overflow"))?;
            if c.pad_byte.is_some() {
                payload_size = payload_size.checked_add(1).ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidData, "RIFF size overflow"))?;
            }
        }
        payload_size = payload_size.checked_add(self.trailing.len() as u32).ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "RIFF size overflow"))?;

        w.write_all(&RIFF_MAGIC.to_le_bytes())?;
        w.write_all(&payload_size.to_le_bytes())?;
        w.write_all(&self.form)?;
        for c in &self.chunks {
            w.write_all(&c.id)?;
            let sz = c.data.len() as u32;
            w.write_all(&sz.to_le_bytes())?;
            w.write_all(&c.data)?;
            if let Some(pad) = c.pad_byte {
                w.write_all(&[pad])?;
            }
        }
        w.write_all(&self.trailing)?;
        Ok(())
    }
}

// ── JSON ────────────────────────────────────────────────────────────────────

fn id_str(id: &[u8; 4]) -> String {
    let mut s = String::with_capacity(4);
    for b in id {
        if (0x20..=0x7E).contains(b) { s.push(*b as char); } else { return String::new(); }
    }
    s
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xF) as usize] as char);
    }
    s
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    if s.len() % 2 != 0 {
        return Err(format!("odd-length hex ({} chars)", s.len()));
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

impl WemFile {
    pub fn to_json(&self) -> Value {
        let mut chunks_arr: Vec<Value> = Vec::with_capacity(self.chunks.len());
        for c in &self.chunks {
            let mut obj = Map::new();
            let id_s = id_str(&c.id);
            if !id_s.is_empty() {
                obj.insert("id".into(), Value::String(id_s));
            } else {
                obj.insert("id_hex".into(), Value::String(hex_encode(&c.id)));
            }
            obj.insert("size".into(), Value::from(c.data.len() as u64));
            obj.insert("data_hex".into(), Value::String(hex_encode(&c.data)));
            if let Some(pad) = c.pad_byte {
                obj.insert("pad_byte".into(), Value::from(pad as u64));
            }
            chunks_arr.push(Value::Object(obj));
        }
        let mut root = Map::new();
        let form_s = id_str(&self.form);
        if !form_s.is_empty() {
            root.insert("form".into(), Value::String(form_s));
        } else {
            root.insert("form_hex".into(), Value::String(hex_encode(&self.form)));
        }
        root.insert("chunks".into(), Value::Array(chunks_arr));
        if !self.trailing.is_empty() {
            root.insert("trailing_hex".into(), Value::String(hex_encode(&self.trailing)));
        }
        Value::Object(root)
    }

    pub fn from_json(v: &Value) -> io::Result<Self> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "expected object at root"))?;
        let form = parse_id_or_hex(obj, "form", "form_hex")?;
        let chunks_arr = obj.get("chunks").and_then(|v| v.as_array())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing chunks array"))?;
        let mut chunks = Vec::with_capacity(chunks_arr.len());
        for (i, e) in chunks_arr.iter().enumerate() {
            let cobj = e.as_object().ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData, format!("chunk[{}] not object", i)))?;
            let id = parse_id_or_hex(cobj, "id", "id_hex")
                .map_err(|e| io::Error::new(e.kind(), format!("chunk[{}].id: {}", i, e)))?;
            let data_hex = cobj.get("data_hex").and_then(|v| v.as_str())
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                    format!("chunk[{}] missing data_hex", i)))?;
            let data = hex_decode(data_hex).map_err(|e| io::Error::new(
                io::ErrorKind::InvalidData, format!("chunk[{}].data_hex: {}", i, e)))?;
            let pad_byte = cobj.get("pad_byte")
                .and_then(|v| v.as_u64())
                .map(|v| v as u8);
            chunks.push(RiffChunk { id, data, pad_byte });
        }
        let trailing = if let Some(s) = obj.get("trailing_hex").and_then(|v| v.as_str()) {
            hex_decode(s).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
                format!("trailing_hex: {}", e)))?
        } else {
            Vec::new()
        };
        Ok(WemFile { form, chunks, trailing })
    }
}

fn parse_id_or_hex(obj: &Map<String, Value>, str_key: &str, hex_key: &str) -> io::Result<[u8; 4]> {
    if let Some(s) = obj.get(str_key).and_then(|v| v.as_str()) {
        let b = s.as_bytes();
        if b.len() != 4 {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("{} must be exactly 4 ASCII chars (got {})", str_key, b.len())));
        }
        let mut out = [0u8; 4];
        out.copy_from_slice(b);
        Ok(out)
    } else if let Some(s) = obj.get(hex_key).and_then(|v| v.as_str()) {
        let v = hex_decode(s).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        if v.len() != 4 {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("{} must decode to 4 bytes (got {})", hex_key, v.len())));
        }
        let mut out = [0u8; 4];
        out.copy_from_slice(&v);
        Ok(out)
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidData,
            format!("missing both '{}' and '{}'", str_key, hex_key)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_minimal_wem() -> Vec<u8> {
        let mut buf = Vec::new();
        // RIFF magic + size + WAVE
        buf.extend_from_slice(b"RIFF");
        // size placeholder, will be 4 (form) + 8 (fmt header) + 16 (fmt body)
        //                          + 8 (data header) + 4 (data body) = 40
        buf.extend_from_slice(&40u32.to_le_bytes());
        buf.extend_from_slice(b"WAVE");
        // fmt chunk
        buf.extend_from_slice(b"fmt ");
        buf.extend_from_slice(&16u32.to_le_bytes());
        buf.extend_from_slice(&[1u8, 0, 1, 0, 0x44, 0xAC, 0, 0, 0x88, 0x58, 1, 0, 2, 0, 16, 0]);
        // data chunk (4 bytes)
        buf.extend_from_slice(b"data");
        buf.extend_from_slice(&4u32.to_le_bytes());
        buf.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);
        buf
    }

    #[test]
    fn parse_minimal() {
        let bytes = build_minimal_wem();
        let f = WemFile::parse(&bytes).unwrap();
        assert_eq!(&f.form, b"WAVE");
        assert_eq!(f.chunks.len(), 2);
        assert_eq!(&f.chunks[0].id, b"fmt ");
        assert_eq!(f.chunks[0].data.len(), 16);
        assert_eq!(&f.chunks[1].id, b"data");
        assert_eq!(f.chunks[1].data, vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert!(f.trailing.is_empty());
        assert_eq!(f.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn odd_chunk_pad() {
        // fmt chunk of size 3 (odd) requires a pad byte
        let mut buf = Vec::new();
        buf.extend_from_slice(b"RIFF");
        buf.extend_from_slice(&16u32.to_le_bytes());  // 4 + 8 + 3 + 1 (pad) = 16
        buf.extend_from_slice(b"WAVE");
        buf.extend_from_slice(b"fmt ");
        buf.extend_from_slice(&3u32.to_le_bytes());
        buf.extend_from_slice(&[0xAA, 0xBB, 0xCC]);
        buf.extend_from_slice(&[0x00]); // pad byte
        let f = WemFile::parse(&buf).unwrap();
        assert_eq!(f.chunks[0].pad_byte, Some(0));
        assert_eq!(f.to_bytes().unwrap(), buf);
    }

    #[test]
    fn json_roundtrip() {
        let bytes = build_minimal_wem();
        let f = WemFile::parse(&bytes).unwrap();
        let j = f.to_json();
        let f2 = WemFile::from_json(&j).unwrap();
        assert_eq!(f, f2);
        assert_eq!(f2.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn rejects_bad_magic() {
        let mut bytes = build_minimal_wem();
        bytes[0] = 0;
        assert!(WemFile::parse(&bytes).is_err());
    }
}
