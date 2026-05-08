// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.paschedulepath` — companion path data for a `.paschedule`
//! (waypoint coordinates and bookkeeping).
//!
//! ## Format (Session 9 RE)
//!
//! Fixed 12-byte header followed by `record_count` records of
//! sample-specific size:
//!
//! ```text
//! offset  size   field            notes
//! 0x00    [u8;8] outer_id         per-NPC identifier (varies per sample)
//! 0x08    u32    record_count     number of waypoint records to follow
//! 0x0C+   [u8]   opaque_records   record-count records of variable
//!                                 per-format size (25-33 bytes
//!                                 typical). Each record begins with
//!                                 a `u32 hash` matching the parent
//!                                 `.paschedule` plus index/flag
//!                                 fields.
//! ```
//!
//! ## Tier 1 (typed) coverage
//!
//! - `outer_id` — preserved (8 bytes).
//! - `record_count` — typed.
//! - `opaque_records` — preserved verbatim. Per-record decode (the
//!   25/29/33-byte variants share a `u32 hash + ...` prefix but
//!   diverge after that) is future work.
//!
//! ## Tier 1.5 fallback
//!
//! [`PaschedulePathFile`] (LP-token-stream view) ships for sub-record
//! discovery — the body is mostly numeric but a handful of files
//! embed asset path strings.

use std::io::{self, Write};

use serde_json::{Map, Value};

use super::{BinaryRead, BinaryWrite};
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};

pub use super::lp_token_stream::{LpTokenFile as PaschedulePathFile, Token};

// ── Tier 1 typed reader ──────────────────────────────────────────────

/// Length of the fixed `.paschedulepath` pre-records header.
const PASCHEDULEPATH_HEADER_LEN: usize = 12;

/// Typed `.paschedulepath` file — Tier 1 structural view.
#[derive(Debug)]
pub struct TypedPaschedulePathFile {
    pub outer_id: [u8; 8],
    pub record_count: u32,
    pub opaque_records: Vec<u8>,
}

impl TypedPaschedulePathFile {
    pub fn parse(data: &[u8]) -> io::Result<Self> {
        if data.len() < PASCHEDULEPATH_HEADER_LEN {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("TypedPaschedulePathFile: input too short ({} bytes)", data.len()),
            ));
        }
        let mut outer_id = [0u8; 8];
        outer_id.copy_from_slice(&data[..8]);
        let mut offset = 8usize;
        let record_count = u32::read_from(data, &mut offset)?;
        let opaque_records = data[offset..].to_vec();
        Ok(Self { outer_id, record_count, opaque_records })
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut out = Vec::new();
        self.write_to(&mut out)?;
        Ok(out)
    }
}

impl BinaryWrite for TypedPaschedulePathFile {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.outer_id)?;
        self.record_count.write_to(w)?;
        w.write_all(&self.opaque_records)?;
        Ok(())
    }
}

impl ToJsonValue for TypedPaschedulePathFile {
    fn to_json_value(&self) -> Value {
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
        let mut m = Map::new();
        m.insert(
            "outer_id_b64".into(),
            Value::String(B64.encode(self.outer_id)),
        );
        m.insert(
            "record_count".into(),
            Value::Number(self.record_count.into()),
        );
        m.insert(
            "opaque_records_b64".into(),
            Value::String(B64.encode(&self.opaque_records)),
        );
        Value::Object(m)
    }
}

impl WriteJsonValue for TypedPaschedulePathFile {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "TypedPaschedulePathFile: expected object",
        ))?;
        let outer_id_b64 = json_get_field(obj, "outer_id_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPaschedulePathFile.outer_id_b64: expected string",
            ))?;
        let outer_id = B64.decode(outer_id_b64).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("TypedPaschedulePathFile.outer_id_b64: invalid base64: {}", e),
        ))?;
        if outer_id.len() != 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "TypedPaschedulePathFile.outer_id_b64: expected 8 bytes, got {}",
                    outer_id.len(),
                ),
            ));
        }
        w.extend_from_slice(&outer_id);
        let record_count = json_get_field(obj, "record_count")?
            .as_u64()
            .map(|x| x as u32)
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPaschedulePathFile.record_count: expected u32",
            ))?;
        w.extend_from_slice(&record_count.to_le_bytes());
        let recs_b64 = json_get_field(obj, "opaque_records_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPaschedulePathFile.opaque_records_b64: expected string",
            ))?;
        let recs = B64.decode(recs_b64).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("TypedPaschedulePathFile.opaque_records_b64: invalid base64: {}", e),
        ))?;
        w.extend_from_slice(&recs);
        Ok(())
    }
}

// ── Decoded | Raw safety wrapper ─────────────────────────────────────

#[derive(Debug)]
pub enum PaschedulePathFileSafe {
    Decoded(TypedPaschedulePathFile),
    Raw {
        bytes: Vec<u8>,
        decode_error: String,
    },
}

impl PaschedulePathFileSafe {
    pub fn parse(data: &[u8]) -> io::Result<Self> {
        match TypedPaschedulePathFile::parse(data) {
            Ok(typed) => Ok(Self::Decoded(typed)),
            Err(e) => Ok(Self::Raw {
                bytes: data.to_vec(),
                decode_error: e.to_string(),
            }),
        }
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        match self {
            Self::Decoded(t) => t.to_bytes(),
            Self::Raw { bytes, .. } => Ok(bytes.clone()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_paschedulepath(outer_id: [u8; 8], record_count: u32, records: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&outer_id);
        out.extend_from_slice(&record_count.to_le_bytes());
        out.extend_from_slice(records);
        out
    }

    #[test]
    fn typed_paschedulepath_round_trip_synthetic() {
        let outer_id = [0x2f, 0x89, 0xc5, 0xc8, 0xcf, 0xb5, 0x00, 0x00];
        let records = b"\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a";
        let original = make_paschedulepath(outer_id, 1, records);
        let parsed = TypedPaschedulePathFile::parse(&original).expect("parse ok");
        assert_eq!(parsed.outer_id, outer_id);
        assert_eq!(parsed.record_count, 1);
        assert_eq!(parsed.opaque_records, records);

        let written = parsed.to_bytes().expect("write ok");
        assert_eq!(written, original);
    }

    #[test]
    fn typed_paschedulepath_rejects_too_short() {
        // Only 8 bytes — not enough for outer_id (8) + u32 record_count (4).
        let too_short = vec![0u8; 8];
        let err = TypedPaschedulePathFile::parse(&too_short)
            .expect_err("must fail on truncated input");
        assert!(format!("{}", err).contains("input too short"),
            "error should mention input too short: {}", err);
    }

    #[test]
    fn typed_paschedulepath_zero_records() {
        // Header only — no records.
        let original = make_paschedulepath([1, 2, 3, 4, 5, 6, 7, 8], 0, b"");
        let parsed = TypedPaschedulePathFile::parse(&original).expect("parse ok");
        assert_eq!(parsed.record_count, 0);
        assert!(parsed.opaque_records.is_empty());
        assert_eq!(parsed.to_bytes().unwrap(), original);
    }

    #[test]
    fn paschedulepath_safe_decoded_arm_synthetic() {
        let original = make_paschedulepath([0; 8], 0, b"");
        let safe = PaschedulePathFileSafe::parse(&original).expect("safe ok");
        match &safe {
            PaschedulePathFileSafe::Decoded(t) => {
                assert_eq!(t.record_count, 0);
            }
            PaschedulePathFileSafe::Raw { .. } => panic!("expected Decoded arm"),
        }
        assert_eq!(safe.to_bytes().unwrap(), original);
    }

    #[test]
    fn paschedulepath_safe_raw_arm_on_truncated() {
        let truncated = vec![0u8, 1, 2];  // 3 bytes — well under header size
        let safe = PaschedulePathFileSafe::parse(&truncated)
            .expect("safe wrapper never fails parse");
        match &safe {
            PaschedulePathFileSafe::Raw { bytes, decode_error } => {
                assert_eq!(bytes, &truncated);
                assert!(!decode_error.is_empty());
            }
            PaschedulePathFileSafe::Decoded(_) => panic!("expected Raw arm"),
        }
        assert_eq!(safe.to_bytes().unwrap(), truncated);
    }
}
