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
/// Stub for A4 — full implementation parses the RIFF chunks.
pub fn classify_wem(_data: &[u8]) -> io::Result<WemMetadata> {
    Err(io::Error::new(
        io::ErrorKind::Other,
        "classify_wem not yet implemented (A4 phase)",
    ))
}
