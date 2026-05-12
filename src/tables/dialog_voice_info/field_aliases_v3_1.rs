// Hand-edited 2026-05-12 to add Mac-canonical aliases verified via IDA
// decompile of `sub_10187EFEC` in CrimsonDesert_Steam (Mac binary).
// Mac canonical 16-field list extracted from Korean error strings
// at 0x1073a2126..0x1073a2583.
//
// v3 (snake_case) is the default emit; v3.1 emits the canonical `_camelCase`
// form. Round-trips identically — both names accepted on input.
//
// WIRE-LAYOUT NOTE (2026-05-12): Mac canonical has 16 wire fields; this
// Rust struct has 15. The missing field is `_footStepDisableCollideImpactSound`
// (Mac wire position #9, between `_footStepGroundSoundEvent` and
// `_footStepSoundOffset`). dmm-parser's current struct round-trips on the
// pre-2026-05-12 fixture, so either:
//   (a) 1.06 wire format drops this field, OR
//   (b) the field is present but dmm-parser is mis-consuming the byte
//       somewhere — only observable on 1.06 records with non-default
//       FootStep states.
// Resolution: needs 1.06 fixture roundtrip test or Win-binary parser
// decompile to confirm.

pub const FIELD_ALIASES_V3_1: &[(&str, &str)] = &[
    ("key", "_key"),
    ("string_key", "_stringKey"),
    ("is_blocked", "_isBlocked"),
    ("sound_event", "_soundEvent"),
    ("foot_step_sound_event", "_footStepSoundEvent"),
    ("foot_step_crouch_sound_event", "_footStepCrouchSoundEvent"),
    ("foot_step_land_sound_event", "_footStepLandSoundEvent"),
    ("foot_step_ground_sound_event", "_footStepGroundSoundEvent"),
    ("foot_step_sound_offset", "_footStepSoundOffset"),
    ("foot_step_crouch_sound_offset", "_footStepCrouchSoundOffset"),
    ("foot_step_land_sound_offset", "_footStepLandSoundOffset"),
    ("foot_step_ground_sound_offset", "_footStepGroundSoundOffset"),
    ("gender", "_gender"),
    ("character_age", "_characterAge"),
    ("job_info_list", "_jobInfoList"),
];
