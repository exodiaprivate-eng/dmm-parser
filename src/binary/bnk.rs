// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.bnk` — Wwise SoundBank.
//!
//! Flat ordered list of chunks. Unlike WEM, there is no outer RIFF
//! wrapper — the file is just `BKHD` directly, then more chunks. Each
//! chunk is `{id: 4 bytes ASCII, size: u32 LE, data: size bytes}`.
//! Common chunk types: `BKHD` (header), `DIDX` (data index), `DATA`
//! (raw .wem payloads), `HIRC` (event hierarchy), `STID` (string
//! table), `STMG` (state manager), `ENVS` / `PLAT` / `INIT`.
//!
//! Round-trip preserves every byte. Field-level value: HIRC chunk
//! holds Wwise event objects (Sound, RandomContainer, Event, etc.) —
//! a future Tier 1.1 pass can split those out as named JSON keys.

use std::collections::HashMap;
use std::io::{self, Write};

use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BnkChunk {
    pub id: [u8; 4],
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BnkFile {
    pub chunks: Vec<BnkChunk>,
    /// Trailing bytes after the last well-formed chunk (rare). Preserved
    /// verbatim for round-trip.
    pub trailing: Vec<u8>,
}

impl BnkFile {
    pub fn parse(data: &[u8]) -> io::Result<Self> {
        let mut chunks: Vec<BnkChunk> = Vec::new();
        let mut p = 0usize;
        while p + 8 <= data.len() {
            let mut id = [0u8; 4];
            id.copy_from_slice(&data[p..p + 4]);
            let sz = u32::from_le_bytes(data[p + 4..p + 8].try_into().unwrap()) as usize;
            let body_end = p.checked_add(8).and_then(|x| x.checked_add(sz))
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "chunk overflow"))?;
            if body_end > data.len() {
                break;
            }
            chunks.push(BnkChunk { id, data: data[p + 8..body_end].to_vec() });
            p = body_end;
        }
        // First chunk must be BKHD for a real BNK; we still parse if missing
        // so callers can decide policy, but emit a clear sentinel.
        if chunks.first().map(|c| &c.id != b"BKHD").unwrap_or(true) {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                "BNK missing BKHD as first chunk"));
        }
        let trailing = if p < data.len() { data[p..].to_vec() } else { Vec::new() };
        Ok(BnkFile { chunks, trailing })
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.write_to(&mut buf)?;
        Ok(buf)
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        for c in &self.chunks {
            w.write_all(&c.id)?;
            let sz = c.data.len() as u32;
            w.write_all(&sz.to_le_bytes())?;
            w.write_all(&c.data)?;
        }
        w.write_all(&self.trailing)?;
        Ok(())
    }

    /// Return `(source_id, in_memory_media_size)` pairs from HIRC Sound objects.
    ///
    /// Wwise HIRC objects are stored as `type:u8, size:u32, id:u32, payload...`;
    /// Sound objects (`type == 2`) start with `AkBankSourceData`:
    /// `plugin_id:u32, stream_type:u8, source_id:u32, uInMemoryMediaSize:u32`.
    pub fn sound_media_sizes(&self) -> Vec<(u32, u32)> {
        self.sound_media_size_offsets()
            .into_iter()
            .map(|(source_id, size, _)| (source_id, size))
            .collect()
    }

    /// Patch HIRC Sound `uInMemoryMediaSize` fields for matching source IDs.
    ///
    /// Returns the number of fields changed. The patch is same-length and only
    /// mutates bytes inside existing HIRC payloads, so `to_bytes()` preserves the
    /// BNK section layout.
    pub fn patch_media_sizes(&mut self, sizes: &HashMap<u32, u32>) -> usize {
        if sizes.is_empty() {
            return 0;
        }

        let mut changed = 0usize;
        for chunk in &mut self.chunks {
            if &chunk.id != b"HIRC" {
                continue;
            }
            for (source_id, current_size, media_size_offset) in hirc_sound_media_size_offsets(&chunk.data) {
                let Some(&new_size) = sizes.get(&source_id) else {
                    continue;
                };
                if new_size == current_size {
                    continue;
                }
                chunk.data[media_size_offset..media_size_offset + 4]
                    .copy_from_slice(&new_size.to_le_bytes());
                changed += 1;
            }
        }
        changed
    }

    fn sound_media_size_offsets(&self) -> Vec<(u32, u32, usize)> {
        let mut out = Vec::new();
        for chunk in &self.chunks {
            if &chunk.id == b"HIRC" {
                out.extend(hirc_sound_media_size_offsets(&chunk.data));
            }
        }
        out
    }
}

fn hirc_sound_media_size_offsets(data: &[u8]) -> Vec<(u32, u32, usize)> {
    const HIRC_SOUND: u8 = 2;
    const SOUND_SOURCE_ID_OFFSET: usize = 5;
    const SOUND_MEDIA_SIZE_OFFSET: usize = 9;
    const SOUND_SOURCE_DATA_MIN: usize = 13;

    let mut out = Vec::new();
    if data.len() < 4 {
        return out;
    }

    let count = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
    let mut p = 4usize;
    for _ in 0..count {
        if p + 5 > data.len() {
            break;
        }

        let object_type = data[p];
        let object_size = u32::from_le_bytes(data[p + 1..p + 5].try_into().unwrap()) as usize;
        let object_start = p + 5;
        let Some(object_end) = object_start.checked_add(object_size) else {
            break;
        };
        if object_end > data.len() {
            break;
        }

        // object_size includes the 4-byte Wwise object ID before the payload.
        let payload_start = object_start + 4;
        if object_type == HIRC_SOUND && payload_start + SOUND_SOURCE_DATA_MIN <= object_end {
            let source_id_offset = payload_start + SOUND_SOURCE_ID_OFFSET;
            let media_size_offset = payload_start + SOUND_MEDIA_SIZE_OFFSET;
            let source_id = u32::from_le_bytes(
                data[source_id_offset..source_id_offset + 4].try_into().unwrap(),
            );
            let media_size = u32::from_le_bytes(
                data[media_size_offset..media_size_offset + 4].try_into().unwrap(),
            );
            out.push((source_id, media_size, media_size_offset));
        }

        p = object_end;
    }

    out
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
    if s.len() % 2 != 0 { return Err(format!("odd-length hex ({} chars)", s.len())); }
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

impl BnkFile {
    pub fn to_json(&self) -> Value {
        let mut arr: Vec<Value> = Vec::with_capacity(self.chunks.len());
        for c in &self.chunks {
            let mut obj = Map::new();
            let s = id_str(&c.id);
            if !s.is_empty() { obj.insert("id".into(), Value::String(s)); }
            else { obj.insert("id_hex".into(), Value::String(hex_encode(&c.id))); }
            obj.insert("size".into(), Value::from(c.data.len() as u64));
            obj.insert("data_hex".into(), Value::String(hex_encode(&c.data)));
            arr.push(Value::Object(obj));
        }
        let mut root = Map::new();
        root.insert("chunks".into(), Value::Array(arr));
        if !self.trailing.is_empty() {
            root.insert("trailing_hex".into(), Value::String(hex_encode(&self.trailing)));
        }
        Value::Object(root)
    }

    pub fn from_json(v: &Value) -> io::Result<Self> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "expected object"))?;
        let arr = obj.get("chunks").and_then(|v| v.as_array())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing chunks array"))?;
        let mut chunks = Vec::with_capacity(arr.len());
        for (i, e) in arr.iter().enumerate() {
            let cobj = e.as_object().ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData, format!("chunk[{}] not object", i)))?;
            let id = if let Some(s) = cobj.get("id").and_then(|v| v.as_str()) {
                let b = s.as_bytes();
                if b.len() != 4 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData,
                        format!("chunk[{}].id must be 4 chars", i)));
                }
                let mut out = [0u8; 4]; out.copy_from_slice(b); out
            } else if let Some(h) = cobj.get("id_hex").and_then(|v| v.as_str()) {
                let v = hex_decode(h).map_err(|e| io::Error::new(
                    io::ErrorKind::InvalidData, format!("chunk[{}].id_hex: {}", i, e)))?;
                if v.len() != 4 {
                    return Err(io::Error::new(io::ErrorKind::InvalidData,
                        format!("chunk[{}].id_hex must decode to 4 bytes", i)));
                }
                let mut out = [0u8; 4]; out.copy_from_slice(&v); out
            } else {
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                    format!("chunk[{}] missing id/id_hex", i)));
            };
            let data_hex = cobj.get("data_hex").and_then(|v| v.as_str())
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                    format!("chunk[{}] missing data_hex", i)))?;
            let data = hex_decode(data_hex).map_err(|e| io::Error::new(
                io::ErrorKind::InvalidData, format!("chunk[{}].data_hex: {}", i, e)))?;
            chunks.push(BnkChunk { id, data });
        }
        let trailing = if let Some(s) = obj.get("trailing_hex").and_then(|v| v.as_str()) {
            hex_decode(s).map_err(|e| io::Error::new(
                io::ErrorKind::InvalidData, format!("trailing_hex: {}", e)))?
        } else {
            Vec::new()
        };
        Ok(BnkFile { chunks, trailing })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_minimal_bnk() -> Vec<u8> {
        let mut buf = Vec::new();
        // BKHD chunk: 16-byte body
        buf.extend_from_slice(b"BKHD");
        buf.extend_from_slice(&16u32.to_le_bytes());
        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        // DIDX chunk: 0 bytes
        buf.extend_from_slice(b"DIDX");
        buf.extend_from_slice(&0u32.to_le_bytes());
        // HIRC chunk: 4 bytes
        buf.extend_from_slice(b"HIRC");
        buf.extend_from_slice(&4u32.to_le_bytes());
        buf.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]);
        buf
    }

    fn build_hirc_sound_bnk(source_id: u32, media_size: u32) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"BKHD");
        buf.extend_from_slice(&16u32.to_le_bytes());
        buf.extend_from_slice(&[0u8; 16]);

        let mut hirc = Vec::new();
        hirc.extend_from_slice(&1u32.to_le_bytes()); // one HIRC object
        hirc.push(2); // Sound
        hirc.extend_from_slice(&17u32.to_le_bytes()); // object ID + 13-byte AkBankSourceData
        hirc.extend_from_slice(&1234u32.to_le_bytes()); // object ID
        hirc.extend_from_slice(&0x0004_0001u32.to_le_bytes()); // plugin ID
        hirc.push(0); // stream type
        hirc.extend_from_slice(&source_id.to_le_bytes());
        hirc.extend_from_slice(&media_size.to_le_bytes());

        buf.extend_from_slice(b"HIRC");
        buf.extend_from_slice(&(hirc.len() as u32).to_le_bytes());
        buf.extend_from_slice(&hirc);
        buf
    }

    #[test]
    fn parse_minimal() {
        let bytes = build_minimal_bnk();
        let f = BnkFile::parse(&bytes).unwrap();
        assert_eq!(f.chunks.len(), 3);
        assert_eq!(&f.chunks[0].id, b"BKHD");
        assert_eq!(f.chunks[0].data.len(), 16);
        assert_eq!(&f.chunks[1].id, b"DIDX");
        assert!(f.chunks[1].data.is_empty());
        assert_eq!(&f.chunks[2].id, b"HIRC");
        assert_eq!(f.chunks[2].data, vec![0xAA, 0xBB, 0xCC, 0xDD]);
        assert_eq!(f.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn rejects_no_bkhd() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"DIDX");
        buf.extend_from_slice(&0u32.to_le_bytes());
        assert!(BnkFile::parse(&buf).is_err());
    }

    #[test]
    fn json_roundtrip() {
        let bytes = build_minimal_bnk();
        let f = BnkFile::parse(&bytes).unwrap();
        let j = f.to_json();
        let f2 = BnkFile::from_json(&j).unwrap();
        assert_eq!(f, f2);
        assert_eq!(f2.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn sound_media_sizes_patch_roundtrip() {
        let bytes = build_hirc_sound_bnk(1045272379, 100);
        let mut f = BnkFile::parse(&bytes).unwrap();
        assert_eq!(f.sound_media_sizes(), vec![(1045272379, 100)]);

        let changed = f.patch_media_sizes(&HashMap::from([(1045272379, 433_000)]));
        assert_eq!(changed, 1);
        assert_eq!(f.sound_media_sizes(), vec![(1045272379, 433_000)]);

        let patched = f.to_bytes().unwrap();
        assert_eq!(patched.len(), bytes.len());
        assert_ne!(patched, bytes);

        let reparsed = BnkFile::parse(&patched).unwrap();
        assert_eq!(reparsed.sound_media_sizes(), vec![(1045272379, 433_000)]);
    }
}
