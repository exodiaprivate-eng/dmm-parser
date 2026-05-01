// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! DDS (DirectDraw Surface) parser + Crimson Desert quirks.
//!
//! Used by SWISS Stacker to validate texture mods before bundling them
//! into v3.1 mod packages, and by future tooling to inspect / classify
//! DDS files extracted from PAZ archives.
//!
//! ## Scope
//!
//! - **Header parsing only** — pixel data is treated as opaque body bytes.
//!   This module classifies and validates DDS files; it does NOT
//!   decompress or render them.
//! - **Crimson-specific fields exposed** — `crimson_mip_sizes`
//!   (Reserved1[0..4]) and `crimson_last4` (dwReserved2 at offset 124)
//!   are surfaced as named fields rather than buried in raw "reserved"
//!   regions.
//! - **Format detection** — FOURCC and DXGI dispatch tables for
//!   identifying DXT1/3/5, BC4/5/6/7, and uncompressed RGB.
//!
//! See `references/dds_notes.md` and `references/dds.hexpat` for the
//! authoritative format documentation.

pub mod header;
pub mod classify;
pub mod vpath;

pub use header::{DdsHeader, DdsPixelFormat, Dx10Header};
pub use classify::{DdsFormat, DdsClassification, classify};
pub use vpath::{classify_vpath_last4, infer_vpath_from_disk_path};
