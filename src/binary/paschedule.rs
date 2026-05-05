// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.paschedule` — NPC time-of-day / activity schedule binary.
//! Header is typically `01 00 00 00` (a small minority use `00 00 00 00`).
//! The format is mostly numeric (waypoint hashes, frame counts) with a
//! few embedded asset path strings. Tier 1.5 view via the shared
//! `lp_token_stream` tokenizer.

pub use super::lp_token_stream::{LpTokenFile as PascheduleFile, Token};
