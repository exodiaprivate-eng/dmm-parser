//! IDA-derived parser for `DialogVoiceInfo.pabgb`.
//!
//! Field layout extracted from Hex-Rays decompile of the parse function
//! in the current Win exe (CrimsonDesert.exe). Field NAMES paired with
//! Mac binary __cstring declaration order. Round-trip-validated against
//! the vanilla pabgb dump from the live game install.
//!
//! ─── 2026-05-12 Mac IDA verification ────────────────────────────────────
//! Mac binary parser (`sub_10187EFEC` in `CrimsonDesert_Steam`) reads
//! 16 wire fields. Wire-byte math is byte-equivalent to the current
//! 15-field struct — the difference is interpretive:
//!
//!   Mac canonical wire (Korean error chain, in order):
//!     1. _key                                  (sub_100F39E0C, u8)
//!     2. _stringKey                            (CString)
//!     3. _isBlocked                            (u8)
//!     4. _soundEvent                           (CString)
//!     5. _footStepSoundEvent                   (CString)
//!     6. _footStepCrouchSoundEvent             (CString)
//!     7. _footStepLandSoundEvent               (CString)
//!     8. _footStepGroundSoundEvent             (CString)
//!     9. _footStepDisableCollideImpactSound    (u8, mem 64)
//!    10. _footStepSoundOffset                  (u8, mem 65)
//!    11. _footStepCrouchSoundOffset            (u8, mem 66)
//!    12. _footStepLandSoundOffset              (u8, mem 67)
//!    13. _footStepGroundSoundOffset            (u8, mem 68)
//!    14. _gender                               (u8, mem 69)
//!    15. _characterAge                         (u8, mem 70)
//!    16. _jobInfoList                          (CArray<u16>)
//!
//! Mac total wire: 1 + 6*(4+N) + 1 + 7 + (4+2K) = 37 + 6N + 2K bytes
//!
//! Pre-2026-05-12 dmm-parser interpreted the same byte stream as:
//!     1. key: u16              (2 wire bytes) ← Mac canonical is u8 (1 byte)
//!     2-8. (same as Mac)
//!     9. — missing —           ← Mac canonical _footStepDisableCollideImpactSound (1 byte)
//!    10-15. (same as Mac, names match)
//!    16. job_info_list
//!
//! The 1 extra byte read as part of `key: u16` was precisely the 1
//! byte missing for `_footStepDisableCollideImpactSound`. Wire round-
//! trip worked under either interpretation but the semantic field-by-
//! field decoding was off.
//!
//! 2026-05-12 RECONCILED: applied Mac-canonical fix below.
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
        // FIX 2026-05-12: was u16, but Mac reader sub_100F39E0C reads u8
        // (single-byte vtbl call pattern). Pre-fix dmm-parser consumed
        // an extra wire byte here, which coincidentally cancelled out
        // the missing _footStepDisableCollideImpactSound below.
        pub key: u8,
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
