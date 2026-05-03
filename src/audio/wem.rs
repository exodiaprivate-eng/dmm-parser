// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! WEM (Wwise Encoded Media) parser. RIFF wrapper + WAVEFORMATEX
//! `fmt ` chunk + Wwise-specific `hash`/`junk` chunks + `data` payload.
//!
//! Stub of the A4 implementation. Module skeleton only — full parser
//! lands in A4. This file provides the public types so dispatch +
//! Python bindings can reference them now.

use std::io;

/// Common WEM `format_tag` values observed in Crimson Desert builds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WemFormatTag {
    /// `0xFFFE` — WAVE_FORMAT_EXTENSIBLE (PCM-style, often 16-bit linear)
    WaveformatExtensible,
    /// `0xFFFF` — Wwise's custom Vorbis variant (most common for game audio)
    WwiseVorbis,
    /// Other tag value preserved for forward compat / unknown codecs
    Other(u16),
}

impl WemFormatTag {
    pub fn from_u16(v: u16) -> Self {
        match v {
            0xFFFE => WemFormatTag::WaveformatExtensible,
            0xFFFF => WemFormatTag::WwiseVorbis,
            other => WemFormatTag::Other(other),
        }
    }

    pub fn raw(&self) -> u16 {
        match *self {
            WemFormatTag::WaveformatExtensible => 0xFFFE,
            WemFormatTag::WwiseVorbis => 0xFFFF,
            WemFormatTag::Other(v) => v,
        }
    }
}

/// Header-only metadata extracted from a WEM file. Audio payload is
/// not decoded — `data_offset` and `data_size` point into the file.
#[derive(Debug, Clone, PartialEq)]
pub struct WemMetadata {
    pub file_size: u64,
    pub format_tag: WemFormatTag,
    pub channels: u16,
    pub sample_rate: u32,
    pub byte_rate: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
    /// Whether the WEM has the Wwise-specific `hash` chunk. Strong
    /// fingerprint that this is a Wwise WEM rather than a raw RIFF-WAVE.
    pub has_wwise_hash_chunk: bool,
    /// Byte offset of the `data` chunk's payload start.
    pub data_offset: u64,
    /// Byte length of the `data` chunk's payload.
    pub data_size: u64,
}

/// Classify a WEM file's metadata from raw bytes. Header-only inspection;
/// does NOT decode audio.
///
/// Walks the RIFF chunk list, locates `fmt `, `data`, and (if present)
/// the Wwise-specific `hash` chunk. Returns `WemMetadata` populated
/// from the `fmt ` payload + `data` chunk position/size.
pub fn classify_wem(data: &[u8]) -> io::Result<WemMetadata> {
    if data.len() < 12 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("WEM file is {} bytes; need at least 12 for RIFF wrapper", data.len()),
        ));
    }
    if &data[0..4] != b"RIFF" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Not a RIFF file: magic = {:?}", &data[0..4]),
        ));
    }
    if &data[8..12] != b"WAVE" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Not a WAVE RIFF: form type = {:?}", &data[8..12]),
        ));
    }

    let mut fmt_payload: Option<&[u8]> = None;
    let mut data_offset: Option<u64> = None;
    let mut data_size: Option<u64> = None;
    let mut has_wwise_hash_chunk = false;

    let mut off: usize = 12;
    while off + 8 <= data.len() {
        let chunk_id: [u8; 4] = data[off..off + 4].try_into().unwrap();
        let chunk_size = u32::from_le_bytes(data[off + 4..off + 8].try_into().unwrap()) as usize;
        let payload_off = off + 8;
        let payload_end = payload_off.checked_add(chunk_size).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("WEM chunk size overflow at offset 0x{:x}", off),
            )
        })?;
        if payload_end > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "WEM chunk {:?} at offset 0x{:x} extends beyond file end",
                    chunk_id, off,
                ),
            ));
        }

        match &chunk_id {
            b"fmt " => {
                fmt_payload = Some(&data[payload_off..payload_end]);
            }
            b"data" => {
                data_offset = Some(payload_off as u64);
                data_size = Some(chunk_size as u64);
            }
            b"hash" => {
                has_wwise_hash_chunk = true;
            }
            _ => {
                // Unknown chunk — skip (junk, future Wwise extensions, etc.)
            }
        }

        // RIFF chunks pad to even byte alignment.
        let advance = if chunk_size % 2 == 1 { chunk_size + 1 } else { chunk_size };
        off = payload_off + advance;
    }

    let fmt = fmt_payload.ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "WEM missing 'fmt ' chunk")
    })?;
    if fmt.len() < 16 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("WEM 'fmt ' chunk too small: {} bytes (need >= 16)", fmt.len()),
        ));
    }

    let format_tag = u16::from_le_bytes(fmt[0..2].try_into().unwrap());
    let channels = u16::from_le_bytes(fmt[2..4].try_into().unwrap());
    let sample_rate = u32::from_le_bytes(fmt[4..8].try_into().unwrap());
    let byte_rate = u32::from_le_bytes(fmt[8..12].try_into().unwrap());
    let block_align = u16::from_le_bytes(fmt[12..14].try_into().unwrap());
    let bits_per_sample = u16::from_le_bytes(fmt[14..16].try_into().unwrap());

    let data_offset = data_offset.ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "WEM missing 'data' chunk")
    })?;
    let data_size = data_size.unwrap_or(0);

    Ok(WemMetadata {
        file_size: data.len() as u64,
        format_tag: WemFormatTag::from_u16(format_tag),
        channels,
        sample_rate,
        byte_rate,
        block_align,
        bits_per_sample,
        has_wwise_hash_chunk,
        data_offset,
        data_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal valid WEM with the given fmt payload + data payload.
    fn make_wem(format_tag: u16, channels: u16, sr: u32, with_hash: bool, data_payload: &[u8]) -> Vec<u8> {
        // fmt chunk: 16 bytes minimum (WAVEFORMATEX)
        let mut fmt = Vec::new();
        fmt.extend_from_slice(&format_tag.to_le_bytes());
        fmt.extend_from_slice(&channels.to_le_bytes());
        fmt.extend_from_slice(&sr.to_le_bytes());
        fmt.extend_from_slice(&0u32.to_le_bytes());  // byte_rate
        fmt.extend_from_slice(&0u16.to_le_bytes());  // block_align
        fmt.extend_from_slice(&0u16.to_le_bytes());  // bits_per_sample

        let hash_payload = vec![0xAA; 16];

        let mut chunks = Vec::new();
        // fmt chunk
        chunks.extend_from_slice(b"fmt ");
        chunks.extend_from_slice(&(fmt.len() as u32).to_le_bytes());
        chunks.extend_from_slice(&fmt);
        // hash chunk (optional)
        if with_hash {
            chunks.extend_from_slice(b"hash");
            chunks.extend_from_slice(&(hash_payload.len() as u32).to_le_bytes());
            chunks.extend_from_slice(&hash_payload);
        }
        // data chunk
        chunks.extend_from_slice(b"data");
        chunks.extend_from_slice(&(data_payload.len() as u32).to_le_bytes());
        chunks.extend_from_slice(data_payload);

        let mut buf = Vec::new();
        buf.extend_from_slice(b"RIFF");
        buf.extend_from_slice(&((4 + chunks.len()) as u32).to_le_bytes()); // size after this field
        buf.extend_from_slice(b"WAVE");
        buf.extend_from_slice(&chunks);
        buf
    }

    #[test]
    fn classify_wwise_vorbis_wem() {
        let buf = make_wem(0xFFFF, 2, 48000, true, &[0xCC; 1000]);
        let m = classify_wem(&buf).unwrap();
        assert_eq!(m.format_tag, WemFormatTag::WwiseVorbis);
        assert_eq!(m.channels, 2);
        assert_eq!(m.sample_rate, 48000);
        assert!(m.has_wwise_hash_chunk);
        assert_eq!(m.data_size, 1000);
    }

    #[test]
    fn classify_pcm_extensible_wem_no_hash() {
        let buf = make_wem(0xFFFE, 1, 44100, false, &[0xDD; 256]);
        let m = classify_wem(&buf).unwrap();
        assert_eq!(m.format_tag, WemFormatTag::WaveformatExtensible);
        assert_eq!(m.channels, 1);
        assert_eq!(m.sample_rate, 44100);
        assert!(!m.has_wwise_hash_chunk);
    }

    #[test]
    fn rejects_non_riff() {
        let buf = vec![b'X'; 64];
        let err = classify_wem(&buf).unwrap_err();
        assert!(err.to_string().contains("Not a RIFF"));
    }

    #[test]
    fn rejects_non_wave_riff() {
        let mut buf = make_wem(0xFFFF, 2, 48000, true, &[0; 16]);
        buf[8..12].copy_from_slice(b"AVI ");
        let err = classify_wem(&buf).unwrap_err();
        assert!(err.to_string().contains("Not a WAVE RIFF"));
    }

    #[test]
    fn rejects_truncated() {
        let buf = vec![b'R', b'I', b'F'];
        let err = classify_wem(&buf).unwrap_err();
        assert!(err.to_string().contains("at least 12"));
    }

    #[test]
    fn rejects_missing_fmt() {
        // Build a RIFF/WAVE with only a data chunk, no fmt
        let mut buf = Vec::new();
        buf.extend_from_slice(b"RIFF");
        buf.extend_from_slice(&20u32.to_le_bytes());
        buf.extend_from_slice(b"WAVE");
        buf.extend_from_slice(b"data");
        buf.extend_from_slice(&8u32.to_le_bytes());
        buf.extend_from_slice(&[0u8; 8]);
        let err = classify_wem(&buf).unwrap_err();
        assert!(err.to_string().contains("missing 'fmt '"));
    }

    #[test]
    fn classify_real_vanilla_wem_sample() {
        // Real Wwise Vorbis WEM from DMM backups. Skip if not present.
        let path = "C:/Users/corin/Desktop/CD JSON Mod Manager/Definitive Mod Manager/src-tauri/target/debug/backups/1045272379.wem";
        let Ok(bytes) = std::fs::read(path) else {
            eprintln!("SKIP: real WEM sample not found");
            return;
        };
        let m = classify_wem(&bytes).unwrap();
        assert_eq!(m.format_tag, WemFormatTag::WwiseVorbis);
        assert_eq!(m.channels, 2);
        assert_eq!(m.sample_rate, 48000);
        assert!(m.has_wwise_hash_chunk);
        assert_eq!(m.data_size, 433579);
    }
}
