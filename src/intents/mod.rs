// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Field-JSON v3.x intent application.
//!
//! Today: parse v3 / v3.1 manifests, dispatch the four custom-item
//! operations (`set`, `clone_record`, `new_record`, `delete_record`)
//! plus storeinfo `array_append` against a parsed table's JSON record
//! list. Iteminfo (sequential — no pabgh sister file) works end-to-end.
//!
//! Tomorrow (task #11): generalize over `dispatch::parse_table_to_json`
//! / `serialize_table_from_json` and rebuild the pabgh sister file for
//! pabgh-bounded tables when records are added or removed.
//!
//! See `docs/CUSTOM_ITEM_CREATOR_V3_1.md` for the design doc and
//! `FIELD_JSON_V3_SPEC.md` (CrimsonGameMods repo) for the manifest
//! format.

pub mod apply;
pub mod path;
pub mod types;

pub use apply::{
    apply_intents_to_iteminfo, apply_resolved_intents, find_record_index,
    ApplyError, ApplyOutcome, ApplyStatus,
};
pub use path::{
    array_append_at_path, get_value_at_path, parse_path, set_value_at_path, PathError,
    PathSegment,
};
pub use types::{
    item_paloc_indices, Intent, IntentDoc, IntentResolveError, ModInfo, Patch,
    ResolvedIntentOp, Target,
};
