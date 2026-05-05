// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.paseqc` — compiled sequencer chart binary (sister format to .paseq).
//! Header magic `FF FF 04 00` (or `FF FF 03 00` on a small minority of
//! older files). Tier 1.5 view via the shared `lp_token_stream`
//! tokenizer; the magic header lands in the leading `RawBytes` token so
//! round-trip is byte-exact.

pub use super::lp_token_stream::{LpTokenFile as PaseqcFile, Token};
