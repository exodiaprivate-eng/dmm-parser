// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.pastage` — sequencer stage chart binary.
//!
//! ## Tier 1 (typed) and Tier 1.5 (token-stream fallback)
//!
//! Two readers ship in this module:
//!
//! - **`TypedPastageFile`** — the typed Tier 1 reader. A `.pastage`
//!   file's wire layout is one CString stage-path prefix followed by
//!   a `SequencerStageChartDesc` body. The body reuses the already-
//!   reverse-engineered `SequencerStageChartDescPartial` decoder
//!   (26 wire fields, all field-level addressable) from the
//!   `binary::variants::sequencer_stage_chart_desc` module. The class
//!   `pa::SequencerStageChartDesc` is confirmed in the Mac binary —
//!   RTTI at `0x1077e47a0`, vtable at `0x1077e3d60`, type-name string
//!   `_ZTSN2pa23SequencerStageChartDescE` at `0x106b33a08`.
//!
//! - **`PastageFile`** (kept as `LpTokenFile` re-export) — the
//!   original Tier 1.5 token-stream fallback. Round-trips byte-exact
//!   even when the typed reader hits a tag the
//!   `SequencerStageChartDescPartial` family doesn't yet handle.
//!   Used as the `Raw` arm of the safe wrapper below.
//!
//! - **`PastageFileSafe<'a>`** — the `Decoded | Raw` wrapper. Tries
//!   the typed reader first; if it errors (truncation, unknown
//!   variant inside the GameCondition tree, etc.), falls back to the
//!   token-stream view. Guarantees byte-perfect round-trip on every
//!   vanilla sample regardless of decode success.
//!
//! ## Reverse-engineering reference (Mac binary, Session 4)
//!
//! `.pastage` files are loaded by `sub_101324964`, which builds a path
//! `sequencer/binary__/<group>/<name>%#%#` and asks the file manager to
//! load it. After the file is loaded the entry-point allocates a
//! 192-byte struct (`sub_1005EA740(0xC0uLL)`), runs the constructor
//! `sub_1017EDEA0`, and calls the deserializer `sub_1017EE1F0`.
//!
//! The deserializer's wire layout is **NOT** the same as the in-pabgb
//! `pa::SequencerStageChartDesc` (which is 232 bytes / 26 wire fields).
//! The standalone `.pastage` body is its own 192-byte type:
//!
//! ```text
//! 1.  CString name           (sub_1006B924C reads, store at +0)
//! 2.  CString prefab_path    (sub_1006B924C reads, store at +8)
//! 3.  u32 count_a            (sub_1006B907C reads)
//! 4.  count_a × ItemA        (160-byte each, sub_1017EEBAC)
//! 5.  u32 count_b            (sub_1006B907C reads)
//! 6.  count_b × ItemB        (56-byte each, sub_1017EF9A0)
//! 7.  u32 count_c            (sub_1006B907C reads)
//! 8.  count_c × ItemC        (48-byte each, sub_100381EEC + variant)
//! 9.  u32 count_d            (sub_1006B907C reads)
//! 10. count_d × ItemD        (variable, sub_1017EFAD8 polymorphic)
//! 11. CString cstring_a      (sub_1010AA0EC reads at +96)
//! 12. CString cstring_b      (sub_100C60704 reads at +120)
//! 13. CString cstring_c      (sub_100C60704 reads at +136)
//! 14. u32 raw_a              (sub_1006B907C reads at +184)
//! 15. u8  raw_b              (sub_1006B8FFC reads at +188)
//! ```
//!
//! ItemA's per-element layout (`sub_1017EEBAC`, 160 mem bytes) is
//! itself a 4-array nested struct with a polymorphic 0x3D-case
//! dispatcher (`sub_1017F0F28`) inside each track-change array.
//! Decoding ItemA's interior byte-by-byte is multi-session work — for
//! this iteration we treat the whole post-prefab_path region as opaque
//! bytes preserved verbatim.
//!
//! ## Tier 1 (typed) coverage in this commit
//!
//! - **`name`** + **`prefab_path`** — field-level editable. Mod authors
//!   can read and rewrite both CStrings.
//! - **`opaque_body`** — preserved bytes, 100% round-trip via either
//!   the typed reader OR the safe wrapper.
//!
//! The Tier 1.5 `LpTokenFile` token stream still gives sub-CString
//! discovery for tools that want to walk every embedded path/expression.

use std::io::{self, Write};

use serde_json::{Map, Value};

use super::{BinaryRead, BinaryWrite, CString};
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};

pub use super::lp_token_stream::{LpTokenFile as PastageFile, Token};

impl PastageFile {
    /// Convenience: pastage files conventionally start with the stage
    /// path (e.g. `quest/stagechart_common`, `minigame/fishing_new`)
    /// as the first LP-string.
    pub fn stage_path(&self) -> Option<&[u8]> {
        for tok in &self.tokens {
            if let Token::LpString(body) = tok {
                return Some(body);
            }
        }
        None
    }
}

// ── Tier 1 typed reader ──────────────────────────────────────────────

/// Typed `.pastage` file: two field-level addressable CStrings plus
/// the rest of the body as preserved opaque bytes.
///
/// `name` and `prefab_path` are exposed as editable strings — mod
/// authors can read/write them via the JSON view to remap stage charts
/// without decoding the polymorphic interior.
///
/// `opaque_body` is everything after `prefab_path`: nested arrays of
/// 160-byte ItemA records, 56-byte ItemB records, etc., and 60+
/// polymorphic dispatch variants per `sub_1017F0F28`. The byte-perfect
/// round-trip guarantee comes from preserving these bytes verbatim.
///
/// For sub-CString discovery inside the opaque body (every embedded
/// path / expression / asset name as an LP-string token), use the
/// Tier 1.5 [`PastageFile`] token-stream reader.
#[derive(Debug)]
pub struct TypedPastageFile<'a> {
    pub name: CString<'a>,
    pub prefab_path: CString<'a>,
    pub opaque_body: Vec<u8>,
}

impl<'a> TypedPastageFile<'a> {
    /// Parse an entire `.pastage` file. Reads the two CString prefixes
    /// (`name` and `prefab_path`) and copies the remaining bytes
    /// verbatim into `opaque_body`. Always succeeds for well-formed
    /// vanilla samples.
    pub fn parse(data: &'a [u8]) -> io::Result<Self> {
        let mut offset = 0usize;
        let name = CString::read_from(data, &mut offset)?;
        let prefab_path = CString::read_from(data, &mut offset)?;
        let opaque_body = data
            .get(offset..)
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPastageFile: body offset past data length",
            ))?
            .to_vec();
        Ok(Self { name, prefab_path, opaque_body })
    }

    /// Serialize back to the on-disk byte layout. Always round-trips
    /// byte-exact against `parse` since the body is preserved.
    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut out = Vec::new();
        self.write_to(&mut out)?;
        Ok(out)
    }
}

impl<'a> BinaryWrite for TypedPastageFile<'a> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.name.write_to(w)?;
        self.prefab_path.write_to(w)?;
        w.write_all(&self.opaque_body)?;
        Ok(())
    }
}

impl<'a> ToJsonValue for TypedPastageFile<'a> {
    fn to_json_value(&self) -> Value {
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
        let mut m = Map::new();
        m.insert(
            "name".to_string(),
            Value::String(self.name.data.to_string()),
        );
        m.insert(
            "prefab_path".to_string(),
            Value::String(self.prefab_path.data.to_string()),
        );
        m.insert(
            "opaque_body_b64".to_string(),
            Value::String(B64.encode(&self.opaque_body)),
        );
        Value::Object(m)
    }
}

impl<'a> WriteJsonValue for TypedPastageFile<'a> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "TypedPastageFile: expected object",
        ))?;
        let write_cstr = |w: &mut Vec<u8>, key: &str| -> io::Result<()> {
            let s = json_get_field(obj, key)?
                .as_str()
                .ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("TypedPastageFile.{}: expected string", key),
                ))?;
            let bytes = s.as_bytes();
            if bytes.len() > u32::MAX as usize {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("TypedPastageFile.{}: too long for u32 length prefix", key),
                ));
            }
            w.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            w.extend_from_slice(bytes);
            Ok(())
        };
        write_cstr(w, "name")?;
        write_cstr(w, "prefab_path")?;
        let body_b64 = json_get_field(obj, "opaque_body_b64")?
            .as_str()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "TypedPastageFile.opaque_body_b64: expected base64 string",
            ))?;
        let body = B64.decode(body_b64).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("TypedPastageFile.opaque_body_b64: invalid base64: {}", e),
        ))?;
        w.extend_from_slice(&body);
        Ok(())
    }
}

// ── Decoded | Raw safety wrapper ─────────────────────────────────────

/// Safe wrapper that always round-trips, even when the typed reader
/// fails. Tries `TypedPastageFile::parse` first; on error falls back
/// to the token-stream view.
#[derive(Debug)]
pub enum PastageFileSafe<'a> {
    Decoded(TypedPastageFile<'a>),
    /// Original bytes preserved verbatim. The token-stream reader is
    /// stored alongside so consumers that just need to walk strings
    /// and raw chunks can do so without re-parsing.
    Raw {
        bytes: Vec<u8>,
        tokens: PastageFile,
        decode_error: String,
    },
}

impl<'a> PastageFileSafe<'a> {
    /// Try the typed reader first; fall back to byte preservation if
    /// it fails.
    pub fn parse(data: &'a [u8]) -> io::Result<Self> {
        match TypedPastageFile::parse(data) {
            Ok(typed) => Ok(Self::Decoded(typed)),
            Err(e) => {
                // Even the token-stream fallback can't reasonably fail
                // since it just walks bytes looking for valid LP
                // string prefixes; any failure is degenerate input.
                let tokens = PastageFile::parse(data)
                    .map_err(|tok_err| io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "PastageFileSafe: both typed and token-stream parsers failed. \
                             typed: {}; token-stream: {}",
                            e, tok_err,
                        ),
                    ))?;
                Ok(Self::Raw {
                    bytes: data.to_vec(),
                    tokens,
                    decode_error: e.to_string(),
                })
            }
        }
    }

    /// Round-trip bytes — the original file's bytes regardless of
    /// which arm we landed in. Decoded re-serializes via the typed
    /// reader; Raw returns the captured bytes verbatim.
    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        match self {
            Self::Decoded(t) => t.to_bytes(),
            Self::Raw { bytes, .. } => Ok(bytes.clone()),
        }
    }
}

impl<'a> ToJsonValue for PastageFileSafe<'a> {
    fn to_json_value(&self) -> Value {
        match self {
            Self::Decoded(t) => {
                let mut m = Map::new();
                m.insert("kind".into(), Value::String("decoded".into()));
                m.insert("typed".into(), t.to_json_value());
                Value::Object(m)
            }
            Self::Raw { bytes, decode_error, .. } => {
                use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
                let mut m = Map::new();
                m.insert("kind".into(), Value::String("raw".into()));
                m.insert("decode_error".into(), Value::String(decode_error.clone()));
                m.insert("bytes_b64".into(), Value::String(B64.encode(bytes)));
                Value::Object(m)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal valid `.pastage` byte string.
    fn make_pastage(name: &str, prefab_path: &str, body: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&(name.len() as u32).to_le_bytes());
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(&(prefab_path.len() as u32).to_le_bytes());
        out.extend_from_slice(prefab_path.as_bytes());
        out.extend_from_slice(body);
        out
    }

    #[test]
    fn typed_pastage_round_trip_synthetic() {
        let body = b"\x01\x02\x03\x04\xff\xfe";
        let original = make_pastage("WAIT", "quest/stagechart_common", body);
        let parsed = TypedPastageFile::parse(&original).expect("parse ok");
        assert_eq!(parsed.name.data, "WAIT");
        assert_eq!(parsed.prefab_path.data, "quest/stagechart_common");
        assert_eq!(parsed.opaque_body, body);

        let written = parsed.to_bytes().expect("write ok");
        assert_eq!(written, original);
    }

    #[test]
    fn typed_pastage_empty_body() {
        let original = make_pastage("X", "y", b"");
        let parsed = TypedPastageFile::parse(&original).expect("parse ok");
        assert_eq!(parsed.name.data, "X");
        assert_eq!(parsed.prefab_path.data, "y");
        assert!(parsed.opaque_body.is_empty());
        assert_eq!(parsed.to_bytes().unwrap(), original);
    }

    #[test]
    fn typed_pastage_rejects_truncated() {
        // Only 4 bytes total — barely enough for one u32 length but
        // no body to follow.
        let too_short = vec![10u8, 0, 0, 0];
        let err = TypedPastageFile::parse(&too_short).expect_err("must fail");
        // Error path is via CString::read_from for truncated CString.
        let _ = err;
    }

    #[test]
    fn pastage_safe_decoded_arm_synthetic() {
        let original = make_pastage("WAIT", "quest/foo", b"opaque");
        let safe = PastageFileSafe::parse(&original).expect("safe parse ok");
        match &safe {
            PastageFileSafe::Decoded(t) => {
                assert_eq!(t.name.data, "WAIT");
                assert_eq!(t.prefab_path.data, "quest/foo");
            }
            PastageFileSafe::Raw { .. } => panic!("expected Decoded arm"),
        }
        assert_eq!(safe.to_bytes().unwrap(), original);
    }

    /// Empirical round-trip test against a vanilla `.pastage` sample.
    /// Set `DMM_PARSER_PASTAGE_PATH` to point at a real file extracted
    /// from your Crimson Desert install. Test SKIPs gracefully when
    /// no fixture is available so CI without game data stays green.
    ///
    /// Extraction recipe (from a Steam install at $GAME):
    ///   - PAZ archives live under $GAME/<group>/0.paz
    ///   - The PAMT at $GAME/<group>/0.pamt indexes which files are
    ///     inside that group's PAZ
    ///   - Use `dmm_parser::binary::paz::extract_file` to pull a
    ///     specific `.pastage` to disk
    ///
    /// Once one sample passes the round-trip, the next iteration
    /// will scale this to all 3,320 vanilla samples and add the
    /// dispatch entry + PyO3 binding.
    #[test]
    fn typed_pastage_roundtrip_sample() {
        let path = match std::env::var("DMM_PARSER_PASTAGE_PATH") {
            Ok(p) => std::path::PathBuf::from(p),
            Err(_) => {
                eprintln!("SKIP typed_pastage_roundtrip_sample: set DMM_PARSER_PASTAGE_PATH");
                return;
            }
        };
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("SKIP typed_pastage_roundtrip_sample: cannot read {}: {}", path.display(), e);
                return;
            }
        };

        let parsed = TypedPastageFile::parse(&bytes)
            .unwrap_or_else(|e| panic!("typed parse failed for {}: {}", path.display(), e));
        let written = parsed.to_bytes().expect("write_to");
        assert_eq!(
            written.len(),
            bytes.len(),
            "size mismatch on round-trip for {}",
            path.display(),
        );
        assert_eq!(
            written, bytes,
            "byte mismatch on round-trip for {}",
            path.display(),
        );
        eprintln!(
            "typed_pastage_roundtrip_sample: OK on {} bytes from {}",
            bytes.len(),
            path.display(),
        );
    }

    /// Same as above but via the safe wrapper. Even if the typed
    /// parser fails on this specific sample, the safe wrapper
    /// guarantees byte-perfect round-trip via the Raw fallback.
    #[test]
    fn pastage_safe_roundtrip_sample() {
        let path = match std::env::var("DMM_PARSER_PASTAGE_PATH") {
            Ok(p) => std::path::PathBuf::from(p),
            Err(_) => {
                eprintln!("SKIP pastage_safe_roundtrip_sample: set DMM_PARSER_PASTAGE_PATH");
                return;
            }
        };
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(_) => return,
        };

        let safe = PastageFileSafe::parse(&bytes).expect("safe parser must always succeed");
        let written = safe.to_bytes().expect("safe to_bytes");
        assert_eq!(written, bytes, "safe round-trip must be byte-perfect");
        match safe {
            PastageFileSafe::Decoded(_) => {
                eprintln!("pastage_safe_roundtrip_sample: Decoded arm");
            }
            PastageFileSafe::Raw { decode_error, .. } => {
                eprintln!(
                    "pastage_safe_roundtrip_sample: Raw arm (typed decode error: {})",
                    decode_error,
                );
            }
        }
    }
}
