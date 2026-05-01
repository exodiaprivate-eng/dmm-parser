// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `validate_dds_for_game(bytes)` — surface format quirks that would
//! make a DDS texture mod fail or render incorrectly when applied to
//! Crimson Desert.
//!
//! Used by SWISS Stacker UI (and future mod-author CLI) to flag
//! problems BEFORE a modder ships a broken texture. Ergonomically
//! returns a `Vec<Validation>` mixing fatals (texture won't load) with
//! warnings (texture loads but might look wrong).
//!
//! See `references/dds_notes.md` §7 for the authoritative rules.

use super::classify::{classify, DdsFormat};
use super::header::{DdsHeader, DDS_MAGIC, DX10_HEADER_TOTAL_SIZE};

/// Severity tier for a single validation finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// The DDS will fail to load. Mod author MUST fix before shipping.
    Fatal,
    /// The DDS loads but may render incorrectly or doesn't follow
    /// production conventions. Recommended fix.
    Warning,
    /// Informational note — useful context, not actionable.
    Info,
}

/// Single validation finding with a stable code and human-readable message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Validation {
    /// Stable identifier — UI / scripts can switch on this without
    /// parsing message strings.
    pub code: &'static str,
    pub severity: Severity,
    pub message: String,
}

impl Validation {
    fn fatal(code: &'static str, message: impl Into<String>) -> Self {
        Validation { code, severity: Severity::Fatal, message: message.into() }
    }
    fn warn(code: &'static str, message: impl Into<String>) -> Self {
        Validation { code, severity: Severity::Warning, message: message.into() }
    }
    fn info(code: &'static str, message: impl Into<String>) -> Self {
        Validation { code, severity: Severity::Info, message: message.into() }
    }
}

/// Run all validation rules against a DDS file's raw bytes. Returns an
/// empty vec when the file is fully valid for Crimson Desert use.
///
/// Encountering a Fatal early (e.g. wrong magic) short-circuits — no
/// further checks attempted on a fundamentally broken file.
pub fn validate_dds_for_game(bytes: &[u8]) -> Vec<Validation> {
    let mut out = Vec::new();

    // ── Tier 1: file integrity ────────────────────────────────────────
    if bytes.len() < 128 {
        out.push(Validation::fatal(
            "header_too_short",
            format!("DDS file is {} bytes; need at least 128", bytes.len()),
        ));
        return out;
    }
    if &bytes[0..4] != DDS_MAGIC {
        out.push(Validation::fatal(
            "bad_magic",
            format!("DDS magic missing — expected b\"DDS \", got {:?}", &bytes[0..4]),
        ));
        return out;
    }

    let header = match DdsHeader::parse(bytes) {
        Ok(h) => h,
        Err(e) => {
            out.push(Validation::fatal("header_parse_error", e.to_string()));
            return out;
        }
    };

    // ── Tier 2: DX10 envelope completeness ────────────────────────────
    let is_dx10 = header.is_dx10();
    if is_dx10 && bytes.len() < DX10_HEADER_TOTAL_SIZE {
        out.push(Validation::fatal(
            "dx10_header_too_short",
            format!(
                "DDS is DX10 but file is {} bytes; DX10 extension requires >= {}",
                bytes.len(), DX10_HEADER_TOTAL_SIZE,
            ),
        ));
        return out;
    }

    // ── Tier 3: format identification ─────────────────────────────────
    let classification = match classify(bytes) {
        Ok(c) => c,
        Err(e) => {
            out.push(Validation::fatal("classify_error", e.to_string()));
            return out;
        }
    };

    if classification.format == DdsFormat::Unknown {
        let fourcc_str = String::from_utf8_lossy(&header.pixel_format.pf_fourcc).to_string();
        if is_dx10 {
            let dxgi = classification.dxgi_format.unwrap_or(0);
            out.push(Validation::warn(
                "unknown_dxgi_format",
                format!("DX10 texture uses DXGI format {} which dmm-parser doesn't recognize", dxgi),
            ));
        } else {
            out.push(Validation::warn(
                "unknown_fourcc",
                format!("Unknown pixel format FOURCC {:?}; texture may not render correctly", fourcc_str),
            ));
        }
    }

    // ── Tier 4: header values game requires ───────────────────────────
    if header.depth == 0 {
        out.push(Validation::warn(
            "depth_zero",
            "dwDepth == 0; game's DDS loader requires >= 1 (will be auto-fixed at apply time)",
        ));
    }
    if header.mip_map_count == 0 {
        out.push(Validation::warn(
            "mip_count_zero",
            "mipMapCount == 0; game requires >= 1 (auto-fixed at apply time)",
        ));
    }

    // ── Tier 5: production conventions ────────────────────────────────
    if !is_power_of_two(header.width) || !is_power_of_two(header.height) {
        out.push(Validation::warn(
            "non_power_of_two_dims",
            format!(
                "Non-POW2 dimensions {}x{}; some shaders/mip chains assume POW2",
                header.width, header.height,
            ),
        ));
    }

    let big_enough_to_need_mips = header.width >= 64 && header.height >= 64;
    if big_enough_to_need_mips && classification.mip_count == 1 {
        out.push(Validation::warn(
            "missing_mips",
            format!(
                "{}x{} texture has only 1 mip; production DDS always include mips for textures >= 64px",
                header.width, header.height,
            ),
        ));
    }

    // ── Tier 6: informational ─────────────────────────────────────────
    if classification.requires_pathc {
        out.push(Validation::info(
            "requires_pathc",
            format!(
                "{:?} needs PATHC template registration at mount time (DMM handles this automatically)",
                classification.format,
            ),
        ));
    }
    if header.is_overlay_patched() {
        out.push(Validation::info(
            "overlay_patched",
            format!(
                "DDS appears to already be overlay-patched (last4=0x{:08x}); validating as-is",
                header.crimson_last4,
            ),
        ));
    }

    out
}

/// Convenience: returns true if the DDS has at least one Fatal validation.
pub fn has_fatal(validations: &[Validation]) -> bool {
    validations.iter().any(|v| v.severity == Severity::Fatal)
}

fn is_power_of_two(n: u32) -> bool {
    n > 0 && (n & (n - 1)) == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dds::header::DDS_FILE_HEADER_SIZE;

    fn make_dds(fourcc: &[u8; 4], width: u32, height: u32, mips: u32) -> Vec<u8> {
        let mut buf = vec![0u8; DDS_FILE_HEADER_SIZE];
        buf[0..4].copy_from_slice(b"DDS ");
        buf[4..8].copy_from_slice(&124u32.to_le_bytes());
        buf[12..16].copy_from_slice(&height.to_le_bytes());
        buf[16..20].copy_from_slice(&width.to_le_bytes());
        buf[24..28].copy_from_slice(&1u32.to_le_bytes());
        buf[28..32].copy_from_slice(&mips.to_le_bytes());
        buf[76..80].copy_from_slice(&32u32.to_le_bytes());
        buf[80..84].copy_from_slice(&4u32.to_le_bytes());
        buf[84..88].copy_from_slice(fourcc);
        buf
    }

    fn codes(v: &[Validation]) -> Vec<&str> {
        v.iter().map(|x| x.code).collect()
    }

    #[test]
    fn valid_dxt5_passes_clean() {
        let buf = make_dds(b"DXT5", 256, 256, 8);
        let v = validate_dds_for_game(&buf);
        assert!(v.is_empty(), "expected no findings, got {:?}", v);
        assert!(!has_fatal(&v));
    }

    #[test]
    fn truncated_file_is_fatal() {
        let buf = vec![0u8; 64];
        let v = validate_dds_for_game(&buf);
        assert!(has_fatal(&v));
        assert_eq!(codes(&v), vec!["header_too_short"]);
    }

    #[test]
    fn bad_magic_is_fatal() {
        let mut buf = make_dds(b"DXT5", 256, 256, 8);
        buf[0..4].copy_from_slice(b"XXXX");
        let v = validate_dds_for_game(&buf);
        assert!(has_fatal(&v));
        assert_eq!(codes(&v), vec!["bad_magic"]);
    }

    #[test]
    fn dx10_truncated_is_fatal() {
        let buf = make_dds(b"DX10", 256, 256, 8); // only 128 bytes — DX10 needs 148
        let v = validate_dds_for_game(&buf);
        assert!(has_fatal(&v));
        assert!(codes(&v).iter().any(|c| *c == "dx10_header_too_short"));
    }

    #[test]
    fn unknown_fourcc_warns() {
        let buf = make_dds(b"WTF!", 64, 64, 4);
        let v = validate_dds_for_game(&buf);
        assert!(!has_fatal(&v));
        assert!(codes(&v).iter().any(|c| *c == "unknown_fourcc"));
    }

    #[test]
    fn dx10_unknown_dxgi_warns() {
        let mut buf = make_dds(b"DX10", 256, 256, 8);
        buf.resize(148, 0);
        buf[128..132].copy_from_slice(&999u32.to_le_bytes()); // not a known DXGI value
        let v = validate_dds_for_game(&buf);
        assert!(!has_fatal(&v));
        assert!(codes(&v).iter().any(|c| *c == "unknown_dxgi_format"));
    }

    #[test]
    fn depth_zero_warns() {
        let mut buf = make_dds(b"DXT5", 256, 256, 8);
        buf[24..28].copy_from_slice(&0u32.to_le_bytes());
        let v = validate_dds_for_game(&buf);
        assert!(!has_fatal(&v));
        assert!(codes(&v).iter().any(|c| *c == "depth_zero"));
    }

    #[test]
    fn mip_zero_warns() {
        let buf = make_dds(b"DXT5", 256, 256, 0);
        let v = validate_dds_for_game(&buf);
        assert!(!has_fatal(&v));
        assert!(codes(&v).iter().any(|c| *c == "mip_count_zero"));
    }

    #[test]
    fn non_pow2_warns() {
        let buf = make_dds(b"DXT5", 100, 100, 4);
        let v = validate_dds_for_game(&buf);
        assert!(!has_fatal(&v));
        assert!(codes(&v).iter().any(|c| *c == "non_power_of_two_dims"));
    }

    #[test]
    fn large_no_mips_warns() {
        let buf = make_dds(b"DXT5", 512, 512, 1); // 512px should have mips
        let v = validate_dds_for_game(&buf);
        assert!(!has_fatal(&v));
        assert!(codes(&v).iter().any(|c| *c == "missing_mips"));
    }

    #[test]
    fn small_no_mips_ok() {
        let buf = make_dds(b"DXT5", 32, 32, 1); // 32px doesn't need mips
        let v = validate_dds_for_game(&buf);
        assert!(!codes(&v).iter().any(|c| *c == "missing_mips"));
    }

    #[test]
    fn bc7_surfaces_pathc_info() {
        let mut buf = make_dds(b"DX10", 256, 256, 8);
        buf.resize(148, 0);
        buf[128..132].copy_from_slice(&98u32.to_le_bytes()); // BC7
        let v = validate_dds_for_game(&buf);
        assert!(!has_fatal(&v));
        assert!(codes(&v).iter().any(|c| *c == "requires_pathc"));
    }

    #[test]
    fn overlay_patched_surfaces_info() {
        let mut buf = make_dds(b"DXT5", 256, 256, 8);
        buf[124..128].copy_from_slice(&0x1280u32.to_le_bytes()); // patched
        let v = validate_dds_for_game(&buf);
        assert!(!has_fatal(&v));
        assert!(codes(&v).iter().any(|c| *c == "overlay_patched"));
    }

    #[test]
    fn pow2_helper() {
        assert!(is_power_of_two(1));
        assert!(is_power_of_two(2));
        assert!(is_power_of_two(64));
        assert!(is_power_of_two(1024));
        assert!(!is_power_of_two(0));
        assert!(!is_power_of_two(3));
        assert!(!is_power_of_two(100));
    }
}
