// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `validate_audio(bytes)` — surface format quirks that would make a
//! Wwise audio asset (WEM or BNK) fail or behave incorrectly when
//! mounted in Crimson Desert.
//!
//! Auto-detects WEM vs BNK from the file's magic bytes ("RIFF" → WEM,
//! "BKHD" → BNK) and dispatches to the appropriate rule set.
//!
//! See `references/wwise_notes.md` for the authoritative validation rules.

use super::bnk::parse_bnk;
use super::wem::{classify_wem, WemFormatTag};

/// Same severity tiers as the DDS validator — mod-author UI can render
/// audio + texture findings in a unified list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioSeverity {
    Fatal,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioValidation {
    pub code: &'static str,
    pub severity: AudioSeverity,
    pub message: String,
}

impl AudioValidation {
    fn fatal(code: &'static str, message: impl Into<String>) -> Self {
        Self { code, severity: AudioSeverity::Fatal, message: message.into() }
    }
    fn warn(code: &'static str, message: impl Into<String>) -> Self {
        Self { code, severity: AudioSeverity::Warning, message: message.into() }
    }
    fn info(code: &'static str, message: impl Into<String>) -> Self {
        Self { code, severity: AudioSeverity::Info, message: message.into() }
    }
}

/// Run all validation rules against a Wwise audio file (WEM or BNK).
/// Returns an empty vec for clean files.
pub fn validate_audio(bytes: &[u8]) -> Vec<AudioValidation> {
    let mut out = Vec::new();
    if bytes.len() < 4 {
        out.push(AudioValidation::fatal(
            "audio_too_short",
            format!("Audio file is {} bytes; need at least 4 for magic", bytes.len()),
        ));
        return out;
    }

    match &bytes[0..4] {
        b"RIFF" => validate_wem(bytes, &mut out),
        b"BKHD" => validate_bnk(bytes, &mut out),
        magic => {
            out.push(AudioValidation::fatal(
                "audio_unknown_magic",
                format!(
                    "Unknown audio magic {:?}; expected \"RIFF\" (WEM) or \"BKHD\" (BNK)",
                    magic,
                ),
            ));
        }
    }

    out
}

/// True if the validations include any Fatal findings.
pub fn has_fatal_audio(validations: &[AudioValidation]) -> bool {
    validations.iter().any(|v| v.severity == AudioSeverity::Fatal)
}

// ── WEM rules ────────────────────────────────────────────────────────────

fn validate_wem(bytes: &[u8], out: &mut Vec<AudioValidation>) {
    let m = match classify_wem(bytes) {
        Ok(m) => m,
        Err(e) => {
            out.push(AudioValidation::fatal("wem_parse_error", e.to_string()));
            return;
        }
    };

    if !m.has_wwise_hash_chunk {
        out.push(AudioValidation::warn(
            "wem_missing_hash_chunk",
            "WEM has no `hash` chunk — Wwise convention is to include one. \
             May be a non-Wwise WAVE file or built with an older toolchain.",
        ));
    }

    match m.format_tag {
        WemFormatTag::WaveformatExtensible | WemFormatTag::WwiseVorbis => {}
        WemFormatTag::Other(tag) => {
            out.push(AudioValidation::warn(
                "wem_unknown_format_tag",
                format!(
                    "WEM format_tag = 0x{:04X}; Crimson typically uses 0xFFFE \
                     (WaveformatExtensible) or 0xFFFF (Wwise Vorbis)",
                    tag,
                ),
            ));
        }
    }

    if m.channels == 0 || m.channels > 8 {
        out.push(AudioValidation::warn(
            "wem_unusual_channel_count",
            format!("WEM channels = {}; sane range is 1..=8", m.channels),
        ));
    }
    if m.sample_rate < 8_000 || m.sample_rate > 96_000 {
        out.push(AudioValidation::warn(
            "wem_unusual_sample_rate",
            format!(
                "WEM sample_rate = {} Hz; sane range is 8000..=96000",
                m.sample_rate,
            ),
        ));
    }
    if m.data_size == 0 {
        out.push(AudioValidation::warn(
            "wem_empty_data",
            "WEM `data` chunk is empty (0 bytes of audio payload)",
        ));
    }
}

// ── BNK rules ────────────────────────────────────────────────────────────

const CRIMSON_BANK_VERSION: u32 = 150;

fn validate_bnk(bytes: &[u8], out: &mut Vec<AudioValidation>) {
    let bnk = match parse_bnk(bytes) {
        Ok(b) => b,
        Err(e) => {
            out.push(AudioValidation::fatal("bnk_parse_error", e.to_string()));
            return;
        }
    };

    if bnk.bank_version != CRIMSON_BANK_VERSION {
        out.push(AudioValidation::warn(
            "bnk_unknown_version",
            format!(
                "BNK bank_version = {}; Crimson Desert uses {} (newer Wwise builds may be incompatible)",
                bnk.bank_version, CRIMSON_BANK_VERSION,
            ),
        ));
    }

    let has_didx = bnk.sections.iter().any(|s| &s.id == b"DIDX");
    let has_data = bnk.data_payload_offset.is_some();
    if has_didx && !has_data {
        out.push(AudioValidation::warn(
            "bnk_didx_without_data",
            "BNK has DIDX (embedded WEM index) but no DATA section — embedded WEMs unreachable",
        ));
    }

    // DIDX bounds check: every entry's offset+size must fit inside DATA.
    if let Some(data_off) = bnk.data_payload_offset {
        // Find the DATA section's payload size.
        let data_size = bnk
            .sections
            .iter()
            .find(|s| &s.id == b"DATA")
            .map(|s| s.size as u64)
            .unwrap_or(0);
        for (i, e) in bnk.embedded_wems.iter().enumerate() {
            let end = e.wem_offset as u64 + e.wem_size as u64;
            if end > data_size {
                out.push(AudioValidation::warn(
                    "bnk_didx_offset_oob",
                    format!(
                        "DIDX entry[{}] (wem_id={}, offset={}, size={}) extends past DATA payload (size={})",
                        i, e.wem_id, e.wem_offset, e.wem_size, data_size,
                    ),
                ));
            }
        }
        let _ = data_off; // silence unused if no entries
    }

    if bnk.has_hirc {
        out.push(AudioValidation::info(
            "bnk_has_hirc",
            "BNK contains HIRC section (event/sound graph). v1 of dmm-parser \
             validates structure only — modders typically don't author HIRC by hand.",
        ));
    }

    if !bnk.embedded_wems.is_empty() {
        out.push(AudioValidation::info(
            "bnk_embedded_wems",
            format!("BNK contains {} embedded WEM(s)", bnk.embedded_wems.len()),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::wem::WemFormatTag;

    fn make_wem_bytes(tag: u16, channels: u16, sr: u32, with_hash: bool, data_size: usize) -> Vec<u8> {
        let mut fmt = Vec::new();
        fmt.extend_from_slice(&tag.to_le_bytes());
        fmt.extend_from_slice(&channels.to_le_bytes());
        fmt.extend_from_slice(&sr.to_le_bytes());
        fmt.extend_from_slice(&0u32.to_le_bytes());
        fmt.extend_from_slice(&0u16.to_le_bytes());
        fmt.extend_from_slice(&0u16.to_le_bytes());

        let mut chunks = Vec::new();
        chunks.extend_from_slice(b"fmt ");
        chunks.extend_from_slice(&(fmt.len() as u32).to_le_bytes());
        chunks.extend_from_slice(&fmt);
        if with_hash {
            chunks.extend_from_slice(b"hash");
            chunks.extend_from_slice(&16u32.to_le_bytes());
            chunks.extend_from_slice(&[0xAA; 16]);
        }
        chunks.extend_from_slice(b"data");
        chunks.extend_from_slice(&(data_size as u32).to_le_bytes());
        chunks.extend(std::iter::repeat(0xCCu8).take(data_size));

        let mut buf = Vec::new();
        buf.extend_from_slice(b"RIFF");
        buf.extend_from_slice(&((4 + chunks.len()) as u32).to_le_bytes());
        buf.extend_from_slice(b"WAVE");
        buf.extend_from_slice(&chunks);
        buf
    }

    fn make_bnk_bytes(version: u32, bank_id: u32, didx: &[(u32, u32, u32)], data_size: usize, with_hirc: bool) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut bkhd = Vec::new();
        bkhd.extend_from_slice(&version.to_le_bytes());
        bkhd.extend_from_slice(&bank_id.to_le_bytes());
        bkhd.resize(52, 0);
        buf.extend_from_slice(b"BKHD");
        buf.extend_from_slice(&(bkhd.len() as u32).to_le_bytes());
        buf.extend_from_slice(&bkhd);

        if !didx.is_empty() {
            buf.extend_from_slice(b"DIDX");
            buf.extend_from_slice(&((didx.len() * 12) as u32).to_le_bytes());
            for &(id, off, sz) in didx {
                buf.extend_from_slice(&id.to_le_bytes());
                buf.extend_from_slice(&off.to_le_bytes());
                buf.extend_from_slice(&sz.to_le_bytes());
            }
        }

        if data_size > 0 {
            buf.extend_from_slice(b"DATA");
            buf.extend_from_slice(&(data_size as u32).to_le_bytes());
            buf.extend(std::iter::repeat(0u8).take(data_size));
        }

        if with_hirc {
            buf.extend_from_slice(b"HIRC");
            buf.extend_from_slice(&0u32.to_le_bytes());
        }

        buf
    }

    fn codes(v: &[AudioValidation]) -> Vec<&str> {
        v.iter().map(|x| x.code).collect()
    }

    #[test]
    fn valid_wem_passes_clean() {
        let buf = make_wem_bytes(0xFFFF, 2, 48000, true, 1024);
        let v = validate_audio(&buf);
        assert!(v.is_empty(), "expected no findings, got {:?}", v);
    }

    #[test]
    fn unknown_magic_fatal() {
        let buf = b"XXXXfoo".to_vec();
        let v = validate_audio(&buf);
        assert!(has_fatal_audio(&v));
        assert_eq!(codes(&v), vec!["audio_unknown_magic"]);
    }

    #[test]
    fn truncated_fatal() {
        let buf = vec![b'R', b'I'];
        let v = validate_audio(&buf);
        assert!(has_fatal_audio(&v));
        assert_eq!(codes(&v), vec!["audio_too_short"]);
    }

    #[test]
    fn wem_parse_error_fatal() {
        // RIFF magic but truncated form type
        let mut buf = vec![b'R', b'I', b'F', b'F'];
        buf.extend_from_slice(&0u32.to_le_bytes());
        // Missing WAVE — only RIFF+size present
        let v = validate_audio(&buf);
        assert!(has_fatal_audio(&v));
        assert!(codes(&v).iter().any(|c| *c == "wem_parse_error"));
    }

    #[test]
    fn wem_no_hash_chunk_warns() {
        let buf = make_wem_bytes(0xFFFE, 2, 44100, false, 256);
        let v = validate_audio(&buf);
        assert!(!has_fatal_audio(&v));
        assert!(codes(&v).iter().any(|c| *c == "wem_missing_hash_chunk"));
    }

    #[test]
    fn wem_unknown_format_tag_warns() {
        let buf = make_wem_bytes(0x1234, 2, 48000, true, 256);
        let v = validate_audio(&buf);
        assert!(codes(&v).iter().any(|c| *c == "wem_unknown_format_tag"));
    }

    #[test]
    fn wem_unusual_channels_warns() {
        let buf = make_wem_bytes(0xFFFE, 16, 48000, true, 256);
        let v = validate_audio(&buf);
        assert!(codes(&v).iter().any(|c| *c == "wem_unusual_channel_count"));
    }

    #[test]
    fn wem_unusual_sample_rate_warns() {
        let buf = make_wem_bytes(0xFFFE, 2, 200000, true, 256);
        let v = validate_audio(&buf);
        assert!(codes(&v).iter().any(|c| *c == "wem_unusual_sample_rate"));
    }

    #[test]
    fn valid_bnk_passes_with_info_only() {
        let buf = make_bnk_bytes(150, 1234, &[], 0, false);
        let v = validate_audio(&buf);
        assert!(!has_fatal_audio(&v));
        // No DIDX, no HIRC → no warnings or info
        assert!(v.is_empty(), "expected no findings, got {:?}", v);
    }

    #[test]
    fn bnk_with_data_didx_passes_with_info() {
        let buf = make_bnk_bytes(150, 9999, &[(100, 0, 256), (200, 256, 512)], 1024, true);
        let v = validate_audio(&buf);
        assert!(!has_fatal_audio(&v));
        assert!(codes(&v).iter().any(|c| *c == "bnk_has_hirc"));
        assert!(codes(&v).iter().any(|c| *c == "bnk_embedded_wems"));
    }

    #[test]
    fn bnk_unknown_version_warns() {
        let buf = make_bnk_bytes(999, 1, &[], 0, false);
        let v = validate_audio(&buf);
        assert!(codes(&v).iter().any(|c| *c == "bnk_unknown_version"));
    }

    #[test]
    fn bnk_didx_offset_oob_warns() {
        // DIDX entry references offset+size beyond DATA section
        let buf = make_bnk_bytes(150, 1, &[(100, 0, 9999)], 256, false);
        let v = validate_audio(&buf);
        assert!(codes(&v).iter().any(|c| *c == "bnk_didx_offset_oob"));
    }

    #[test]
    fn bnk_parse_error_fatal() {
        // BKHD magic but body too short (only BKHD + 4-byte size, no payload)
        let buf = vec![b'B', b'K', b'H', b'D', 0xFF, 0xFF, 0xFF, 0x7F];
        let v = validate_audio(&buf);
        assert!(has_fatal_audio(&v));
        assert!(codes(&v).iter().any(|c| *c == "bnk_parse_error"));
    }

    // Sanity check that Severity round-trips through has_fatal_audio
    #[test]
    fn has_fatal_helper() {
        let v = vec![AudioValidation::fatal("x", "msg")];
        assert!(has_fatal_audio(&v));
        let v = vec![AudioValidation::warn("x", "msg")];
        assert!(!has_fatal_audio(&v));
    }

    // Also verify WemFormatTag is used (this is here to silence "imported but unused")
    #[test]
    fn format_tag_round_trip() {
        let _ = WemFormatTag::from_u16(0xFFFF);
    }
}
