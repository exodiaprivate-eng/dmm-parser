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
/// Recognized patterns (case-insensitive; leading slash optional):
///   `0006/sound/windows/<lang>/<id>.bnk` → LocalizedVoiceBank
///   `0006/sound/windows/<lang>/<id>.wem` → LocalizedVoiceClip
///   `soundcommon/windows/<id>.bnk`       → CommonSoundBank
///   `soundcommon/windows/<id>.wem`       → CommonSoundClip
///   any other `.wem` / `.bnk` path        → OtherAudio
///   non-audio extension                   → None
pub fn infer_audio_vpath(vpath: &str) -> Option<AudioPathClass> {
    let lower = vpath.to_ascii_lowercase();
    let p = lower.strip_prefix('/').unwrap_or(&lower);

    let is_bnk = p.ends_with(".bnk");
    let is_wem = p.ends_with(".wem");
    if !is_bnk && !is_wem {
        return None;
    }

    if p.starts_with("0006/sound/windows/") {
        Some(if is_bnk { AudioPathClass::LocalizedVoiceBank } else { AudioPathClass::LocalizedVoiceClip })
    } else if p.starts_with("soundcommon/windows/") {
        Some(if is_bnk { AudioPathClass::CommonSoundBank } else { AudioPathClass::CommonSoundClip })
    } else {
        Some(AudioPathClass::OtherAudio)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn localized_voice_bank() {
        assert_eq!(
            infer_audio_vpath("0006/sound/windows/english(us)/3684722581.bnk"),
            Some(AudioPathClass::LocalizedVoiceBank),
        );
        assert_eq!(
            infer_audio_vpath("/0006/sound/windows/korean/694511365.bnk"),
            Some(AudioPathClass::LocalizedVoiceBank),
        );
    }

    #[test]
    fn localized_voice_clip() {
        assert_eq!(
            infer_audio_vpath("0006/sound/windows/english(us)/1045272379.wem"),
            Some(AudioPathClass::LocalizedVoiceClip),
        );
    }

    #[test]
    fn common_sound_bank() {
        assert_eq!(
            infer_audio_vpath("soundcommon/windows/2498340951.bnk"),
            Some(AudioPathClass::CommonSoundBank),
        );
    }

    #[test]
    fn common_sound_clip() {
        assert_eq!(
            infer_audio_vpath("soundcommon/windows/113958244.wem"),
            Some(AudioPathClass::CommonSoundClip),
        );
    }

    #[test]
    fn other_audio_paths() {
        // Legacy DMM convention or non-canonical Wwise paths
        assert_eq!(
            infer_audio_vpath("0014/sound/banks/macduff_voices.bnk"),
            Some(AudioPathClass::OtherAudio),
        );
        assert_eq!(
            infer_audio_vpath("0014/sound/character/macduff/voice_attack01.wem"),
            Some(AudioPathClass::OtherAudio),
        );
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(
            infer_audio_vpath("0006/Sound/Windows/English(US)/3684722581.BNK"),
            Some(AudioPathClass::LocalizedVoiceBank),
        );
    }

    #[test]
    fn non_audio_returns_none() {
        assert_eq!(infer_audio_vpath("0009/character/texture/foo.dds"), None);
        assert_eq!(infer_audio_vpath("paloc.pamt"), None);
        assert_eq!(infer_audio_vpath(""), None);
    }
}
