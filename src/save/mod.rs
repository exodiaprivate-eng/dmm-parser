// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Crimson Desert save file format (Phase S).
//!
//! ## Scope
//!
//! - **Envelope** (S2): the 0x80-byte header + ChaCha20/LZ4/HMAC pipeline.
//!   Keys and HMAC are caller-supplied so dmm-parser stays a pure format
//!   library (no embedded keys in the public crate).
//! - **Body** (S3+): typed inventory / equipment / quest / dye sections,
//!   landing in submodules as the recon for each section completes.
//!
//! See `references/save_notes.md` and `references/save.hexpat` for the
//! authoritative format documentation.

pub mod envelope;

pub use envelope::{
    SaveHeader, SaveEnvelope, EnvelopeError, EnvelopeWarning,
    HEADER_SIZE, MAGIC,
    parse_header, decrypt_envelope, encrypt_envelope,
};
