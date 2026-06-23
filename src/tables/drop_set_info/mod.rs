// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

mod info;
pub use info::DropSetInfo;
pub mod field_aliases_v3_1;

/// Community / third-party-exporter field-name aliases — `(canonical_snake,
/// accepted_alias)`. An intent that names a field by its alias is rewritten to
/// the canonical field before apply (see `intents::normalize_intent_community`).
/// Distinct from the auto-generated v3.1 camelCase table, which never carries a
/// foreign tool's vocabulary.
///
/// `list` ← `drops`: CrimsonGameMods "DropSets" / Stacker exports name the
/// drop-entry array `drops`; this parser exposes it as `list`. Without this,
/// every `drops` intent silently dropped (the resolver wrote a junk key that
/// serialize ignores → loot never changed). The element sub-fields
/// (`item_key`/`rates`/`rates_100`/`min_amt`/`max_amt`) are already accepted on
/// the write side by `DropTargetData::write_from_json`, so aliasing the array
/// field name is the only missing piece.
pub const FIELD_ALIASES_COMMUNITY: &[(&str, &str)] = &[("list", "drops")];
