//! IDA-derived parser for `DialogVoiceInfo.pabgb`.
//!
//! Field layout extracted from Hex-Rays decompile of the parse function
//! in the current Win exe (CrimsonDesert.exe). Field NAMES paired with
//! Mac binary __cstring declaration order. Round-trip-validated against
//! the vanilla pabgb dump from the live game install.
//!
//! ─── 2026-05-12 Mac IDA verification (1.06 fixture-confirmed) ──────────
//! Mac binary parser `sub_10187EFEC` in `CrimsonDesert_Steam` reads
//! 16 wire fields. Verified against extracted live 1.06 fixture:
//! 483 records, 35473 bytes, byte-identical roundtrip.
//!
//! Wire reads in order:
//!   1.  _key                                (sub_100F39E0C, u16)
//!   2.  _stringKey                          (CString)
//!   3.  _isBlocked                          (u8)
//!   4.  _soundEvent                         (CString)
//!   5.  _footStepSoundEvent                 (CString)
//!   6.  _footStepCrouchSoundEvent           (CString)
//!   7.  _footStepLandSoundEvent             (CString)
//!   8.  _footStepGroundSoundEvent           (CString)
//!   9.  _footStepDisableCollideImpactSound  (u8) ← 1.06 ADDITION
//!  10.  _footStepSoundOffset                (u8)
//!  11.  _footStepCrouchSoundOffset          (u8)
//!  12.  _footStepLandSoundOffset            (u8)
//!  13.  _footStepGroundSoundOffset          (u8)
//!  14.  _gender                             (u8)
//!  15.  _characterAge                       (u8)
//!  16.  _jobInfoList                        (CArray<u16>)
//!
//! Total wire: 2 + 6*(4+N) + 1 + 7 + (4+2K) = 38 + 6N + 2K bytes
//!
//! Pre-2026-05-12 dmm-parser was missing `_footStepDisableCollideImpactSound`
//! at wire position #9 — this commit adds it. Key remains u16 (Mac
//! reader sub_100F39E0C confirmed via `__int16 v4` BYREF, vtbl `2LL`).
//!
//! DO NOT EDIT BY HAND - regenerate via tools/ida_extract.py.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::DialogVoiceInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  13
// Decoded by dmm-parser:   13
// Missing in this struct:  0
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ✅ _footStepGroundSoundOffset (direct_u8, stream=1)
// ✅ _footStepLandSoundOffset (direct_u8, stream=1)
// ✅ _jobInfoList (reader_2B, stream=2)
// ✅ _soundEvent
// ✅ _isBlocked (direct_u8, stream=1)
// ✅ _footStepCrouchSoundEvent
// ✅ _footStepSoundEvent
// ✅ _footStepGroundSoundEvent
// ✅ _footStepLandSoundEvent
// ✅ _footStepCrouchSoundOffset (direct_u8, stream=1)
// ✅ _footStepSoundOffset (direct_u8, stream=1)
// ✅ _stringKey
// ✅ _key (reader_2B, stream=2)

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct DialogVoiceInfo<'a> {
        // 2026-05-12: confirmed u16 wire via Mac reader sub_100F39E0C
        // (`__int16 v4` BYREF, vtbl call with `2LL` size arg). An
        // earlier mid-day commit briefly changed this to u8 — that was
        // a misread of the reader (mercenary uses sub_100F3E64C which
        // IS u8 with `1LL` arg; dialog_voice uses a different reader
        // with `2LL`). 1.06 fixture roundtrip confirms u16.
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub sound_event: CString<'a>,
        pub foot_step_sound_event: CString<'a>,
        pub foot_step_crouch_sound_event: CString<'a>,
        pub foot_step_land_sound_event: CString<'a>,
        pub foot_step_ground_sound_event: CString<'a>,
        // NEW 2026-05-12: Mac canonical wire position #9, reader
        // sub_1006BED20 (u8). dmm-parser was always missing this
        // field; pre-2026-05-12 the missing byte was hidden by the
        // u16 key reading 1 extra byte (sum still matched per record).
        pub foot_step_disable_collide_impact_sound: u8,
        pub foot_step_sound_offset: u8,
        pub foot_step_crouch_sound_offset: u8,
        pub foot_step_land_sound_offset: u8,
        pub foot_step_ground_sound_offset: u8,
        pub gender: u8,
        pub character_age: u8,
        pub job_info_list: CArray<u16>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/dialogvoiceinfo.pabgb";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(DialogVoiceInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out, data, "dialogvoiceinfo roundtrip bytes mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing fixture {}", PABGB_PATH);
            return;
        };
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(DialogVoiceInfo::read_from(&data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");

        for (i, item) in items.iter().enumerate() {
            let _ = &item;
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            DialogVoiceInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, item.key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write",
                i, item.key
            );
        }
    }
}
