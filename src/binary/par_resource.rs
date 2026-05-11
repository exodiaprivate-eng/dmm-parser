// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! PAR family — `.pab`, `.paa`, `.pam` share a common 8-byte header
//! `"PAR " + u32 version`. Each extension uses a stable version
//! constant verified across 30+ real game samples (iter 4):
//!
//! | Ext   | Version u32  | Bytes (LE)    | Purpose                            |
//! |-------|--------------|---------------|------------------------------------|
//! | `.pab` | `0x01050001` | `01 05 00 01` | skeletal volume (per KR strings)   |
//! | `.paa` | `0x01000302` | `02 03 00 01` | animation set entry                |
//! | `.pam` | `0x00001802` | `02 18 00 00` | single animation file (Havok)      |
//! | `.pabc`| `0x01000134` | `34 01 00 01` | (iter 7) PA character body — uniform v1.0.0.52 across all 20 sampled |
//! | `.pabv`| `0x01000136` | `36 01 00 01` | (iter 7) PA body variant — v1.0.0.54 (14/20) |
//! | `.pabv`| `0x01000137` | `37 01 00 01` | (iter 7) PA body variant — v1.0.0.55 (6/20)  |
//! | `.pac` | `0x01000503` | `03 05 00 01` | (iter 10) PA character archive — main 19/20  |
//! | `.pac` | `0x01000003` | `03 00 00 01` | (iter 10) PA character archive — older 1/20  |
//! | `.pat` | `0x01000404` | `04 04 00 01` | (iter 17) PA texture data — 3/3 sampled       |
//! | `.papr`| `0x01000135` | `35 01 00 01` | (iter 20) PA particle/projectile resource     |
//!
//! This module ships the **classifier + round-trip** layer (Tier 1).
//! Per-extension typed field decoding is queued for later iters once
//! sample volumes + IDA pseudocode are available. The opaque body is
//! preserved verbatim in `body_b64` so writes are byte-identical.
//!
//! NOTE: `.pam` files are commonly stored with "partial compression"
//! in the paz packages; dmm-parser's current `extract_file` does not
//! support partial decompression, so `.pam` round-trip testing is
//! limited to whichever subset extracts cleanly. The format detection
//! itself works on any byte buffer.

use std::io::{self, Write};

use base64::Engine;
use serde_json::{Map, Value};

pub const PAR_MAGIC: &[u8; 4] = b"PAR ";

pub const VERSION_PAB: u32 = 0x01050001;
pub const VERSION_PAA: u32 = 0x01000302;
pub const VERSION_PAM: u32 = 0x00001802;
pub const VERSION_PABC: u32 = 0x01000134;
pub const VERSION_PABV_136: u32 = 0x01000136;
pub const VERSION_PABV_137: u32 = 0x01000137;
pub const VERSION_PAC_503: u32 = 0x01000503;
pub const VERSION_PAC_003: u32 = 0x01000003;
pub const VERSION_PAT: u32 = 0x01000404;
pub const VERSION_PAPR: u32 = 0x01000135;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParExt {
    Pab,
    Paa,
    Pam,
    Pabc,
    Pabv,
    Pac,
    Pat,
    Papr,
    Unknown,
}

impl ParExt {
    pub fn from_version(v: u32) -> Self {
        match v {
            VERSION_PAB => ParExt::Pab,
            VERSION_PAA => ParExt::Paa,
            VERSION_PAM => ParExt::Pam,
            VERSION_PABC => ParExt::Pabc,
            VERSION_PABV_136 | VERSION_PABV_137 => ParExt::Pabv,
            VERSION_PAC_503 | VERSION_PAC_003 => ParExt::Pac,
            VERSION_PAT => ParExt::Pat,
            VERSION_PAPR => ParExt::Papr,
            _ => ParExt::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ParExt::Pab => "pab",
            ParExt::Paa => "paa",
            ParExt::Pam => "pam",
            ParExt::Pabc => "pabc",
            ParExt::Pabv => "pabv",
            ParExt::Pac => "pac",
            ParExt::Pat => "pat",
            ParExt::Papr => "papr",
            ParExt::Unknown => "unknown",
        }
    }
}

/// Parse a PAR-family file body to JSON. Validates magic, captures
/// version + ext classification, base64-encodes the body for opaque
/// round-trip.
pub fn parse_par_to_json(data: &[u8]) -> io::Result<Value> {
    if data.len() < 8 {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!("PAR body too short ({} bytes < 8)", data.len())));
    }
    if &data[0..4] != PAR_MAGIC {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!("PAR magic mismatch: got {:02x?}, expected {:02x?}",
                &data[0..4], PAR_MAGIC)));
    }
    let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
    let ext = ParExt::from_version(version);
    let body_b64 = base64::engine::general_purpose::STANDARD.encode(&data[8..]);

    let mut map = Map::new();
    map.insert("key".to_string(), Value::from(0u64));
    map.insert("string_key".to_string(), Value::from(""));
    map.insert("magic".to_string(), Value::from("PAR "));
    map.insert("version".to_string(), Value::from(version as u64));
    map.insert("version_hex".to_string(), Value::from(format!("0x{:08x}", version)));
    map.insert("ext_classification".to_string(), Value::from(ext.as_str()));
    map.insert("body_b64".to_string(), Value::from(body_b64));
    map.insert("body_len".to_string(), Value::from((data.len() - 8) as u64));
    Ok(Value::Object(map))
}

/// Serialize a PAR-family JSON dict back to bytes. Rebuilds the
/// 8-byte header from `version`, decodes `body_b64`, concatenates.
pub fn serialize_par_from_json(value: &Value) -> io::Result<Vec<u8>> {
    let map = value.as_object().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData, "PAR serialize: expected object root"))?;
    let version = map.get("version")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            "PAR serialize: missing or invalid 'version' u64"))?;
    let body_b64 = map.get("body_b64")
        .and_then(|v| v.as_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            "PAR serialize: missing 'body_b64' string"))?;
    let body = base64::engine::general_purpose::STANDARD.decode(body_b64).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData,
            format!("PAR serialize: body_b64 decode failed: {}", e))
    })?;

    let mut out = Vec::with_capacity(8 + body.len());
    out.write_all(PAR_MAGIC)?;
    out.write_all(&(version as u32).to_le_bytes())?;
    out.write_all(&body)?;
    Ok(out)
}

/// Validate that a PAR file is the expected extension. Returns the
/// parsed JSON on success; returns Err if the version doesn't match.
/// Use for `.pab`/`.paa`/`.pam` dispatch to catch swapped-content bugs.
pub fn parse_par_expect(data: &[u8], expected: ParExt) -> io::Result<Value> {
    let v = parse_par_to_json(data)?;
    let actual_str = v.as_object().unwrap()["ext_classification"].as_str().unwrap();
    if actual_str != expected.as_str() {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!("PAR ext mismatch: file looks like .{}, caller asked for .{}",
                actual_str, expected.as_str())));
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_par(version: u32, body: &[u8]) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(PAR_MAGIC);
        v.extend_from_slice(&version.to_le_bytes());
        v.extend_from_slice(body);
        v
    }

    #[test]
    fn parse_pab_real_sample_header() {
        // First 32 bytes of identityskeleton.pab (real install sample, iter 4)
        let body_after_header: [u8; 24] = [
            0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x15, 0xf1,
            0x6f, 0x52, 0x10, 0x42, 0x5f, 0x4d, 0x6f, 0x76,
        ];
        let bytes = make_par(VERSION_PAB, &body_after_header);
        let v = parse_par_to_json(&bytes).expect("parse .pab");
        let m = v.as_object().unwrap();
        assert_eq!(m["magic"], Value::from("PAR "));
        assert_eq!(m["version"], Value::from(VERSION_PAB as u64));
        assert_eq!(m["ext_classification"], Value::from("pab"));
        assert_eq!(m["body_len"], Value::from(24u64));
    }

    #[test]
    fn parse_paa_classification() {
        let bytes = make_par(VERSION_PAA, &[0; 16]);
        let v = parse_par_to_json(&bytes).expect("parse");
        assert_eq!(v["ext_classification"], Value::from("paa"));
    }

    #[test]
    fn parse_pam_classification() {
        let bytes = make_par(VERSION_PAM, &[0; 16]);
        let v = parse_par_to_json(&bytes).expect("parse");
        assert_eq!(v["ext_classification"], Value::from("pam"));
    }

    #[test]
    fn round_trip_byte_perfect() {
        let body = b"arbitrary opaque body bytes \xff\x00\x01\x02".to_vec();
        let original = make_par(VERSION_PAB, &body);
        let parsed = parse_par_to_json(&original).expect("parse");
        let written = serialize_par_from_json(&parsed).expect("write");
        assert_eq!(written, original, "PAR round-trip mismatch");
    }

    #[test]
    fn rejects_bad_magic() {
        let bytes = b"XXX \x01\x05\x00\x01extra".to_vec();
        let result = parse_par_to_json(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_too_short() {
        let bytes = b"PAR".to_vec();
        let result = parse_par_to_json(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn ext_mismatch_detected() {
        // .pab bytes parsed as .paa should error
        let bytes = make_par(VERSION_PAB, &[0; 8]);
        let result = parse_par_expect(&bytes, ParExt::Paa);
        assert!(result.is_err());
    }

    #[test]
    fn unknown_version_classified_unknown() {
        let bytes = make_par(0xDEADBEEF, &[0; 8]);
        let v = parse_par_to_json(&bytes).expect("parse (unknown version ok)");
        assert_eq!(v["ext_classification"], Value::from("unknown"));
    }

    #[test]
    fn parse_pabc_classification() {
        let bytes = make_par(VERSION_PABC, &[0; 16]);
        let v = parse_par_to_json(&bytes).expect("parse");
        assert_eq!(v["ext_classification"], Value::from("pabc"));
    }

    #[test]
    fn parse_pabv_both_versions_classified() {
        for v in [VERSION_PABV_136, VERSION_PABV_137] {
            let bytes = make_par(v, &[0; 16]);
            let p = parse_par_to_json(&bytes).expect("parse");
            assert_eq!(p["ext_classification"], Value::from("pabv"),
                "version 0x{:08x} not classified as pabv", v);
        }
    }

    #[test]
    fn parse_pac_both_versions_classified() {
        for v in [VERSION_PAC_503, VERSION_PAC_003] {
            let bytes = make_par(v, &[0; 16]);
            let p = parse_par_to_json(&bytes).expect("parse");
            assert_eq!(p["ext_classification"], Value::from("pac"),
                "version 0x{:08x} not classified as pac", v);
        }
    }

    #[test]
    fn parse_pat_classification() {
        let bytes = make_par(VERSION_PAT, &[0; 16]);
        let p = parse_par_to_json(&bytes).expect("parse");
        assert_eq!(p["ext_classification"], Value::from("pat"));
    }

    #[test]
    fn parse_papr_classification() {
        let bytes = make_par(VERSION_PAPR, &[0; 16]);
        let p = parse_par_to_json(&bytes).expect("parse");
        assert_eq!(p["ext_classification"], Value::from("papr"));
    }
}
