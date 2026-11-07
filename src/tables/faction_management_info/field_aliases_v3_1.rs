// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

// v3.1 aliases. This table is not in NattKh's pabgb_complete_schema.json and its
// field names are proposals decoded from game data, so only the three head
// fields every table shares are aliased. Regenerate with
// scripts/generate_v3_1_aliases.py once canonical names are known.

pub const FIELD_ALIASES_V3_1: &[(&str, &str)] = &[
    ("key", "_key"),
    ("string_key", "_stringKey"),
    ("is_blocked", "_isBlocked"),
];
