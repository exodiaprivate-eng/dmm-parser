// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Wwise audio vpath helpers. Recognizes Crimson Desert path conventions
//! for voice banks (per-language) and common sound effects.
//!
//! Stub for A6. Module skeleton only — full path table lands in A6.

/// Logical asset class inferred from a Wwise file's vpath.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioPathClass {
    /// `0006/sound/windows/<lang>/<id>.bnk` — localized voice bank
    LocalizedVoiceBank,
    /// `0006/sound/windows/<lang>/<id>.wem` — localized voice clip
    LocalizedVoiceClip,
    /// `soundcommon/windows/<id>.bnk` — common sound effect bank
    CommonSoundBank,
    /// `soundcommon/windows/<id>.wem` — common sound effect clip
    CommonSoundClip,
    /// Other recognized but uncategorized audio path
    OtherAudio,
}

/// Infer the audio asset class from a Crimson Desert vpath. Returns
/// `None` if the path doesn't match any known audio convention.
///
/// Stub for A6 — full path-prefix table lands in A6.
pub fn infer_audio_vpath(_vpath: &str) -> Option<AudioPathClass> {
    None
}
