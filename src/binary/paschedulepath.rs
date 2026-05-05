// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.paschedulepath` — companion path data for a paschedule (waypoint
//! coordinates and bookkeeping). No fixed magic; each file's header
//! starts with a per-NPC hash. Almost entirely numeric, but a handful
//! of files embed asset path strings; the shared `lp_token_stream`
//! tokenizer captures both.

pub use super::lp_token_stream::{LpTokenFile as PaschedulePathFile, Token};
