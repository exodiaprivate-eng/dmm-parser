// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! BNK (Wwise SoundBank) parser. Section-header walker + BKHD bank ID
//! extraction + DIDX embedded WEM index.
//!
//! Stub of the A5 implementation. Module skeleton only — full parser
//! lands in A5. This file provides the public types so dispatch +
//! Python bindings can reference them now.

use std::io;

/// Single DIDX entry — describes one embedded WEM inside the BNK's
/// DATA section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DidxEntry {
    pub wem_id: u32,
    /// Offset INTO the DATA section's payload (not absolute file offset).
    pub wem_offset: u32,
    pub wem_size: u32,
}

/// One BNK section header + payload range. The payload is referenced
/// by file offset, not copied — keeps parsing cheap on multi-MB BNKs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BnkSection {
    /// 4-character section ID: "BKHD" / "DIDX" / "DATA" / "HIRC" / "STID" / ...
    pub id: [u8; 4],
    /// Absolute file offset where this section's 8-byte header starts.
    pub header_offset: u64,
    /// Section payload size in bytes (NOT including the 8-byte header).
    pub size: u32,
}

impl BnkSection {
    pub fn id_str(&self) -> &str {
        std::str::from_utf8(&self.id).unwrap_or("????")
    }
}

/// Header-only view of a BNK file. DATA payload is NOT loaded — DIDX
/// entries point into it for callers that need the actual WEM bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct BnkBank {
    pub file_size: u64,
    pub bank_version: u32,
    pub bank_id: u32,
    pub sections: Vec<BnkSection>,
    pub embedded_wems: Vec<DidxEntry>,
    /// Absolute file offset of the DATA section's payload start. None
    /// if the BNK has no DATA section (i.e. event/HIRC-only banks).
    pub data_payload_offset: Option<u64>,
    /// True when the BNK contains an HIRC section. Modders typically
    /// don't author HIRC; presence is informational only.
    pub has_hirc: bool,
}

/// Parse a BNK file's section structure + bank header from raw bytes.
/// Stub for A5 — full implementation walks the section list.
pub fn parse_bnk(_data: &[u8]) -> io::Result<BnkBank> {
    Err(io::Error::new(
        io::ErrorKind::Other,
        "parse_bnk not yet implemented (A5 phase)",
    ))
}
