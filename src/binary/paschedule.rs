// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.paschedule` — NPC time-of-day / activity schedule binary.
//!
//! ## Format (Session 8 RE)
//!
//! Fixed 21-byte header followed by the schedule's name CString and
//! an opaque body of waypoint data:
//!
//! ```text
//! offset  size  field            notes
//! ----------------------------------------------------------
//! 0x00    u32   version          0x00000001 in vanilla data
//! 0x04    u32   hash             sample-specific identifier
//! 0x08    u8    flag             0x05 in vanilla data
//! 0x09    u32   hash_repeated    same as `hash` at +4
//! 0x0D    [u8;8] reserved        always zero in vanilla data
//! 0x15    CString name           e.g. "cd_seq_spawn_doc_animal_bear_idle_01_schedule"
//! 0x??+   [u8]   opaque_body     waypoint data + nested name re-use
//! ```
//!
//! ## Tier 1 (typed) coverage
//!
//! - `version`, `hash`, `flag`, `hash_repeated`, `reserved` —
//!   structurally addressable.
//! - `name` — field-level editable.
//! - `opaque_body` — preserved verbatim (waypoint hashes, nested
//!   schedule references, frame counts).
//!
//! ## Tier 1.5 fallback
//!
//! [`PascheduleFile`] (LP-token-stream view) still ships for sub-name
//! discovery (the body re-uses the schedule name for nested
//! waypoints).

use std::io::{self, Write};

use serde_json::{Map, Value};

use super::{BinaryRead, BinaryWrite, CString};
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};

pub use super::lp_token_stream::{LpTokenFile as PascheduleFile, Token};

// ── Tier 1 typed reader ──────────────────────────────────────────────

/// Length of the fixed `.paschedule` pre-name header.
const PASCHEDULE_HEADER_LEN: usize = 21;

/// Typed `.paschedule` file — Tier 1 structural view.
///
/// Round-trips byte-exact via `parse`/`to_bytes` on every well-formed
/// vanilla sample.
#[derive(Debug)]
pub struct TypedPascheduleFile<'a> {
    pub version: u32,
    pub hash: u32,
    pub flag: u8,
    pub hash_repeated: u32,
    pub reserved: [u8; 8],
    pub name: CString<'a>,
    pub opaque_body: Vec<u8>,
}

impl<'a> TypedPascheduleFile<'a> {
    pub fn parse(data: &'a [u8]) -> io::Result<Self> {
        if data.len() < PASCHEDULE_HEADER_LEN + 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("TypedPascheduleFile: input too short ({} bytes)", data.len()),
            ));
        }
        let mut offset = 0usize;
        let version = u32::read_from(data, &mut offset)?;
        let hash = u32::read_from(data, &mut offset)?;
        let flag = u8::read_from(data, &mut offset)?;
        let hash_repeated = u32::read_from(data, &mut offset)?;
        let mut reserved = [0u8; 8];
        reserved.copy_from_slice(&data[offset..offset + 8]);
        offset += 8;
        let name = CString::read_from(data, &mut offset)?;
        let opaque_body = data
            .get(offset..)
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPascheduleFile: body offset past data length",
            ))?
            .to_vec();
        Ok(Self {
            version,
            hash,
            flag,
            hash_repeated,
            reserved,
            name,
            opaque_body,
        })
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut out = Vec::new();
        self.write_to(&mut out)?;
        Ok(out)
    }
}

impl<'a> BinaryWrite for TypedPascheduleFile<'a> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.version.write_to(w)?;
        self.hash.write_to(w)?;
        self.flag.write_to(w)?;
        self.hash_repeated.write_to(w)?;
        w.write_all(&self.reserved)?;
        self.name.write_to(w)?;
        w.write_all(&self.opaque_body)?;
        Ok(())
    }
}

impl<'a> ToJsonValue for TypedPascheduleFile<'a> {
    fn to_json_value(&self) -> Value {
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
        let mut m = Map::new();
        m.insert("version".into(), Value::Number(self.version.into()));
        m.insert("hash".into(), Value::Number(self.hash.into()));
        m.insert("flag".into(), Value::Number(self.flag.into()));
        m.insert(
            "hash_repeated".into(),
            Value::Number(self.hash_repeated.into()),
        );
        m.insert(
            "reserved_b64".into(),
            Value::String(B64.encode(self.reserved)),
        );
        m.insert(
            "name".into(),
            Value::String(self.name.data.to_string()),
        );
        m.insert(
            "opaque_body_b64".into(),
            Value::String(B64.encode(&self.opaque_body)),
        );
        Value::Object(m)
    }
}

impl<'a> WriteJsonValue for TypedPascheduleFile<'a> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "TypedPascheduleFile: expected object",
        ))?;
        let read_u32 = |key: &str| -> io::Result<u32> {
            json_get_field(obj, key)?
                .as_u64()
                .map(|x| x as u32)
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("TypedPascheduleFile.{}: expected u32", key),
                ))
        };
        let read_u8 = |key: &str| -> io::Result<u8> {
            json_get_field(obj, key)?
                .as_u64()
                .map(|x| x as u8)
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("TypedPascheduleFile.{}: expected u8", key),
                ))
        };
        w.extend_from_slice(&read_u32("version")?.to_le_bytes());
        w.extend_from_slice(&read_u32("hash")?.to_le_bytes());
        w.push(read_u8("flag")?);
        w.extend_from_slice(&read_u32("hash_repeated")?.to_le_bytes());
        let reserved_b64 = json_get_field(obj, "reserved_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPascheduleFile.reserved_b64: expected string",
            ))?;
        let reserved = B64.decode(reserved_b64).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("TypedPascheduleFile.reserved_b64: invalid base64: {}", e),
        ))?;
        if reserved.len() != 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "TypedPascheduleFile.reserved_b64: expected 8 bytes, got {}",
                    reserved.len(),
                ),
            ));
        }
        w.extend_from_slice(&reserved);
        let name = json_get_field(obj, "name")?
            .as_str()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPascheduleFile.name: expected string",
            ))?;
        let nb = name.as_bytes();
        if nb.len() > u32::MAX as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPascheduleFile.name: too long for u32 length prefix",
            ));
        }
        w.extend_from_slice(&(nb.len() as u32).to_le_bytes());
        w.extend_from_slice(nb);
        let body_b64 = json_get_field(obj, "opaque_body_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPascheduleFile.opaque_body_b64: expected base64 string",
            ))?;
        let body = B64.decode(body_b64).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("TypedPascheduleFile.opaque_body_b64: invalid base64: {}", e),
        ))?;
        w.extend_from_slice(&body);
        Ok(())
    }
}

// ── Decoded | Raw safety wrapper ─────────────────────────────────────

#[derive(Debug)]
pub enum PascheduleFileSafe<'a> {
    Decoded(TypedPascheduleFile<'a>),
    Raw {
        bytes: Vec<u8>,
        decode_error: String,
    },
}

impl<'a> PascheduleFileSafe<'a> {
    pub fn parse(data: &'a [u8]) -> io::Result<Self> {
        match TypedPascheduleFile::parse(data) {
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

    /// Build a minimal valid `.paschedule` byte string.
    fn make_paschedule(
        version: u32,
        hash: u32,
        flag: u8,
        hash_repeated: u32,
        reserved: [u8; 8],
        name: &str,
        body: &[u8],
    ) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&version.to_le_bytes());
        out.extend_from_slice(&hash.to_le_bytes());
        out.push(flag);
        out.extend_from_slice(&hash_repeated.to_le_bytes());
        out.extend_from_slice(&reserved);
        out.extend_from_slice(&(name.len() as u32).to_le_bytes());
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(body);
        out
    }

    #[test]
    fn typed_paschedule_round_trip_synthetic() {
        let original = make_paschedule(
            1, 0x277a8ee3, 5, 0x277a8ee3, [0u8; 8],
            "cd_seq_spawn_doc_animal_bear_idle_01_schedule",
            b"\x01\x02\x03",
        );
        let parsed = TypedPascheduleFile::parse(&original).expect("parse ok");
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.hash, 0x277a8ee3);
        assert_eq!(parsed.flag, 5);
        assert_eq!(parsed.hash_repeated, 0x277a8ee3);
        assert_eq!(parsed.reserved, [0u8; 8]);
        assert_eq!(parsed.name.data, "cd_seq_spawn_doc_animal_bear_idle_01_schedule");
        assert_eq!(parsed.opaque_body, vec![0x01, 0x02, 0x03]);

        let written = parsed.to_bytes().expect("write ok");
        assert_eq!(written, original);
    }

    #[test]
    fn typed_paschedule_rejects_too_short() {
        // Just 4 bytes — far smaller than the 21-byte fixed header
        // plus a CString length prefix.
        let too_short = vec![0u8; 4];
        let err = TypedPascheduleFile::parse(&too_short)
            .expect_err("must fail on truncated input");
        assert!(format!("{}", err).contains("input too short"),
            "error should mention input too short: {}", err);
    }

    #[test]
    fn typed_paschedule_empty_body() {
        // Valid header + name with no opaque_body trailing bytes.
        let original = make_paschedule(1, 0xdeadbeef, 5, 0xdeadbeef, [0; 8], "x", b"");
        let parsed = TypedPascheduleFile::parse(&original).expect("parse ok");
        assert!(parsed.opaque_body.is_empty());
        assert_eq!(parsed.to_bytes().unwrap(), original);
    }

    #[test]
    fn paschedule_safe_decoded_arm_synthetic() {
        let original = make_paschedule(1, 1, 5, 1, [0; 8], "name", b"body");
        let safe = PascheduleFileSafe::parse(&original).expect("safe ok");
        match &safe {
            PascheduleFileSafe::Decoded(t) => {
                assert_eq!(t.version, 1);
                assert_eq!(t.name.data, "name");
            }
            PascheduleFileSafe::Raw { .. } => panic!("expected Decoded arm"),
        }
        assert_eq!(safe.to_bytes().unwrap(), original);
    }

    #[test]
    fn paschedule_safe_raw_arm_on_truncated() {
        // Truncated input — typed reader fails; safe wrapper should
        // fall back to Raw arm and round-trip the original bytes.
        let truncated = vec![1u8, 0, 0, 0]; // just version u32, nothing else
        let safe = PascheduleFileSafe::parse(&truncated).expect("safe wrapper never fails parse");
        match &safe {
            PascheduleFileSafe::Raw { bytes, decode_error } => {
                assert_eq!(bytes, &truncated);
                assert!(!decode_error.is_empty());
            }
            PascheduleFileSafe::Decoded(_) => panic!("expected Raw arm on truncated input"),
        }
        assert_eq!(safe.to_bytes().unwrap(), truncated);
    }
}
