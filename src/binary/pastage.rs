// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.pastage` — sequencer stage chart binary. Tier 1.5 view via the
//! shared `lp_token_stream` tokenizer; round-trips byte-exact on all
//! 3,320 vanilla samples.

pub use super::lp_token_stream::{LpTokenFile as PastageFile, Token};

impl PastageFile {
    /// Convenience: pastage files conventionally start with the stage
    /// path (e.g. `quest/stagechart_common`, `minigame/fishing_new`)
    /// as the first LP-string.
    pub fn stage_path(&self) -> Option<&[u8]> {
        for tok in &self.tokens {
            if let Token::LpString(body) = tok {
                return Some(body);
            }
        }
        None
    }
}
