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
}

// ── HIRC Sound media-size patch (voice-mod auto-rebuild) ─────────────────────
//
// A .bnk-less voice mod ships re-recorded `.wem` but not the soundbank that
// drives them. The engine reads exactly `uInMemoryMediaSize` bytes for each
// clip, and the VANILLA invariant is `uInMemoryMediaSize == .wem file size`
// (verified exact across 0006 dialogue AND 0004 media/combat-grunt clips —
// see dmm_probes/wwise_bnk_proof.py). When a mod replaces a `.wem` with a
// different-sized one and the bank keeps the stale size, the engine reads the
// wrong byte count → the clip drops (silent). DMM can synthesize a correct
// bank by re-stamping each Sound's `uInMemoryMediaSize` to the modded `.wem`'s
// actual size — a same-length edit, so the bank round-trips and stays valid.
//
// HIRC layout: `u32 count`, then per object `{u8 type, u32 sectionSize, body}`.
// Sound object (type 2) body: `ulID(4) pluginID(4) streamType(1) sourceID(4)
// uInMemoryMediaSize(4) …`. (Verified on BKHD version 150.)

/// HIRC object type for a Sound (CAkSound).
const HIRC_OBJ_SOUND: u8 = 2;
/// Byte offset of `sourceID` within a Sound object body.
const SOUND_SOURCE_ID_OFF: usize = 9;
/// Byte offset of `uInMemoryMediaSize` within a Sound object body.
const SOUND_MEDIA_SIZE_OFF: usize = 13;
/// Minimum Sound body length to contain `uInMemoryMediaSize` (ends at +17).
const SOUND_MIN_BODY: usize = SOUND_MEDIA_SIZE_OFF + 4;

impl BnkFile {
    fn hirc_data(&self) -> Option<&[u8]> {
        self.chunks.iter().find(|c| &c.id == b"HIRC").map(|c| c.data.as_slice())
    }
    fn hirc_data_mut(&mut self) -> Option<&mut Vec<u8>> {
        self.chunks.iter_mut().find(|c| &c.id == b"HIRC").map(|c| &mut c.data)
    }

    /// Walk HIRC Sound objects, returning `(sourceID, offset-within-HIRC-data of
    /// the u32 uInMemoryMediaSize)`. Skips non-Sound / unknown object types by
    /// their declared section size, so it is robust to a heterogeneous HIRC.
    fn sound_media_size_fields(data: &[u8]) -> Vec<(u32, usize)> {
        let mut out = Vec::new();
        if data.len() < 4 {
            return out;
        }
        let count = u32::from_le_bytes(data[0..4].try_into().unwrap());
        let mut p = 4usize;
        for _ in 0..count {
            if p + 5 > data.len() {
                break;
            }
            let otype = data[p];
            let osize = u32::from_le_bytes(data[p + 1..p + 5].try_into().unwrap()) as usize;
            let body = p + 5;
            let end = match body.checked_add(osize) {
                Some(e) if e <= data.len() => e,
                _ => break,
            };
            if otype == HIRC_OBJ_SOUND && osize >= SOUND_MIN_BODY {
                let source_id = u32::from_le_bytes(
                    data[body + SOUND_SOURCE_ID_OFF..body + SOUND_SOURCE_ID_OFF + 4]
                        .try_into()
                        .unwrap(),
                );
                out.push((source_id, body + SOUND_MEDIA_SIZE_OFF));
            }
            p = end;
        }
        out
    }

    /// Each Sound's `(sourceID, current uInMemoryMediaSize)`. Empty if no HIRC.
    pub fn sound_media_sizes(&self) -> Vec<(u32, u32)> {
        let data = match self.hirc_data() {
            Some(d) => d,
            None => return Vec::new(),
        };
        Self::sound_media_size_fields(data)
            .into_iter()
            .map(|(sid, off)| {
                (sid, u32::from_le_bytes(data[off..off + 4].try_into().unwrap()))
            })
            .collect()
    }

    /// Re-stamp each Sound's `uInMemoryMediaSize` to the modded `.wem` size,
    /// keyed by `sourceID`. Sounds whose `sourceID` is absent from `wem_sizes`
    /// are left untouched. Same-length edit (round-trip-safe). Returns the
    /// number of Sound objects updated.
    pub fn patch_media_sizes(
        &mut self,
        wem_sizes: &std::collections::HashMap<u32, u32>,
    ) -> usize {
        let fields = match self.hirc_data() {
            Some(d) => Self::sound_media_size_fields(d),
            None => return 0,
        };
        let data = match self.hirc_data_mut() {
            Some(d) => d,
            None => return 0,
        };
        let mut n = 0;
        for (sid, off) in fields {
            if let Some(&new_sz) = wem_sizes.get(&sid) {
                data[off..off + 4].copy_from_slice(&new_sz.to_le_bytes());
                n += 1;
            }
        }
        n
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

    // ── HIRC media-size patch ────────────────────────────────────────────────

    /// A 21-byte CAkSound body: ulID, pluginID(=Vorbis), streamType, sourceID,
    /// uInMemoryMediaSize, + 4 trailing param bytes.
    fn sound_body(ul_id: u32, source_id: u32, media_size: u32) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&ul_id.to_le_bytes()); // +0  ulID
        b.extend_from_slice(&0x0004_0001u32.to_le_bytes()); // +4  pluginID (Vorbis)
        b.push(0); // +8  streamType
        b.extend_from_slice(&source_id.to_le_bytes()); // +9  sourceID
        b.extend_from_slice(&media_size.to_le_bytes()); // +13 uInMemoryMediaSize
        b.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]); // +17 trailing params
        b
    }

    fn hirc_object(otype: u8, body: &[u8]) -> Vec<u8> {
        let mut o = Vec::new();
        o.push(otype);
        o.extend_from_slice(&(body.len() as u32).to_le_bytes());
        o.extend_from_slice(body);
        o
    }

    /// BKHD + a HIRC with two Sounds (src 500 @1000, src 600 @2000) and a
    /// non-Sound Event object that must be left untouched.
    fn build_bnk_with_hirc() -> Vec<u8> {
        let mut hirc = Vec::new();
        hirc.extend_from_slice(&3u32.to_le_bytes()); // object count
        hirc.extend(hirc_object(HIRC_OBJ_SOUND, &sound_body(100, 500, 1000)));
        hirc.extend(hirc_object(HIRC_OBJ_SOUND, &sound_body(101, 600, 2000)));
        hirc.extend(hirc_object(4 /*Event*/, &[0x11; 10]));

        let mut buf = Vec::new();
        buf.extend_from_slice(b"BKHD");
        buf.extend_from_slice(&16u32.to_le_bytes());
        buf.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        buf.extend_from_slice(b"HIRC");
        buf.extend_from_slice(&(hirc.len() as u32).to_le_bytes());
        buf.extend_from_slice(&hirc);
        buf
    }

    #[test]
    fn reads_sound_media_sizes() {
        let f = BnkFile::parse(&build_bnk_with_hirc()).unwrap();
        assert_eq!(f.sound_media_sizes(), vec![(500, 1000), (600, 2000)]);
    }

    #[test]
    fn patch_media_sizes_restamps_and_roundtrips() {
        let orig = build_bnk_with_hirc();
        let mut f = BnkFile::parse(&orig).unwrap();
        let mut sizes = std::collections::HashMap::new();
        sizes.insert(500u32, 1234u32);
        sizes.insert(600u32, 5678u32);
        // a sourceID not in the bank — must be ignored, not error
        sizes.insert(999u32, 42u32);

        let n = f.patch_media_sizes(&sizes);
        assert_eq!(n, 2, "both Sounds re-stamped");
        assert_eq!(f.sound_media_sizes(), vec![(500, 1234), (600, 5678)]);

        // same-length edit → byte length unchanged, still valid
        let out = f.to_bytes().unwrap();
        assert_eq!(out.len(), orig.len(), "media-size patch is length-preserving");
        // the Event object (last 10 body bytes) is untouched
        assert!(out.windows(10).any(|w| w == [0x11u8; 10]));
        // re-parse is stable
        assert_eq!(BnkFile::parse(&out).unwrap(), f);
    }

    #[test]
    fn patch_only_present_source_ids() {
        let mut f = BnkFile::parse(&build_bnk_with_hirc()).unwrap();
        let mut sizes = std::collections::HashMap::new();
        sizes.insert(600u32, 5678u32); // only the second Sound
        let n = f.patch_media_sizes(&sizes);
        assert_eq!(n, 1);
        assert_eq!(f.sound_media_sizes(), vec![(500, 1000), (600, 5678)]);
    }

    #[test]
    fn patch_no_hirc_is_noop() {
        // BKHD-only bank: no HIRC → nothing to patch, no panic.
        let mut buf = Vec::new();
        buf.extend_from_slice(b"BKHD");
        buf.extend_from_slice(&4u32.to_le_bytes());
        buf.extend_from_slice(&[0, 0, 0, 0]);
        let mut f = BnkFile::parse(&buf).unwrap();
        let mut sizes = std::collections::HashMap::new();
        sizes.insert(1u32, 2u32);
        assert_eq!(f.patch_media_sizes(&sizes), 0);
        assert!(f.sound_media_sizes().is_empty());
    }

    /// Validate the HIRC walker against REAL heterogeneous banks (every object
    /// type: Attenuation, RanSeqCntr, BlendCntr, ActorMixer, Event, …) using
    /// local fixtures from `dmm_probes/dump_bnk_fixture.py`. Confirms the
    /// vanilla invariant `uInMemoryMediaSize == .wem size` on real data and that
    /// the real bank round-trips byte-for-byte. Skips if fixtures are absent
    /// (CI / other machines) — the synthetic tests above always run.
    #[test]
    fn real_bank_media_size_invariant_and_roundtrip() {
        use std::path::Path;
        let base = concat!(env!("CARGO_MANIFEST_DIR"), "/dmm_probes/fixtures");
        if !Path::new(base).exists() {
            eprintln!("skipping real-bank validation: no fixtures at {base}");
            return;
        }
        let mut total_checked = 0usize;
        for bid in ["3684722581", "694511365"] {
            let bin = format!("{base}/bnk_{bid}.bin");
            let js = format!("{base}/bnk_{bid}_wem.json");
            if !Path::new(&bin).exists() || !Path::new(&js).exists() {
                continue;
            }
            let bytes = std::fs::read(&bin).unwrap();
            let sizes: Map<String, Value> =
                serde_json::from_slice(&std::fs::read(&js).unwrap()).unwrap();
            let f = BnkFile::parse(&bytes).unwrap();

            // byte-exact round-trip on a real bank
            assert_eq!(f.to_bytes().unwrap(), bytes, "real bank {bid} must round-trip");

            // every locatable Sound's field must equal the vanilla .wem size
            let mut matched = 0;
            for (sid, field) in f.sound_media_sizes() {
                if let Some(v) = sizes.get(&sid.to_string()).and_then(|v| v.as_u64()) {
                    assert_eq!(
                        field as u64, v,
                        "bank {bid} sourceID {sid}: uInMemoryMediaSize {field} != vanilla .wem {v}"
                    );
                    matched += 1;
                }
            }
            assert!(matched > 50, "bank {bid}: only {matched} clips validated");
            total_checked += matched;
        }
        assert!(total_checked > 100, "expected >100 real clips validated, got {total_checked}");
        eprintln!("real-bank validation: {total_checked} clips match vanilla invariant");
    }
}
