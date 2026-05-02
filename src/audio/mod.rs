// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Wwise audio (WEM + BNK) parser + Crimson Desert quirks.
//!
//! Used by SWISS Stacker to validate audio mods before bundling them
//! into v3.1 mod packages. Header-only metadata extraction and structure
//! validation; we do NOT decode audio payloads.
//!
//! ## Scope
//!
//! - **WEM**: parse RIFF wrapper + Wwise-specific chunks; surface
//!   format_tag, channels, sample_rate, payload size for validation.
//! - **BNK**: parse section headers (BKHD/DIDX/DATA/HIRC/STID); surface
//!   bank version + bank ID + embedded WEM index.
//! - **Vpath inference**: path-prefix table for Wwise paths
//!   (`0006/sound/windows/<lang>/...`, `soundcommon/windows/...`,
//!   `0014/sound/...`).
//!
//! See `references/wwise_notes.md`, `references/wem.hexpat`, and
//! `references/bnk.hexpat` for the authoritative format documentation.

pub mod wem;
pub mod bnk;
pub mod vpath;
pub mod validate;

pub use wem::{WemMetadata, WemFormatTag, classify_wem};
pub use bnk::{BnkBank, BnkSection, DidxEntry, parse_bnk};
pub use vpath::{infer_audio_vpath, AudioPathClass};
pub use validate::{validate_audio, has_fatal_audio, AudioValidation, AudioSeverity};
