// Hand-edited 2026-05-12 — full Mac-canonical alignment.
// Verified via IDA decompile of `sub_10187EFEC` in CrimsonDesert_Steam
// (Mac binary == 1.06 source of truth). Mac canonical 16-field list
// extracted from Korean error strings at 0x1073a2126..0x1073a2583.
//
// Struct rewrite in info.rs same day: `key` changed u16 → u8 to match
// Mac reader sub_100F39E0C, and `foot_step_disable_collide_impact_sound`
// inserted at wire position #9. Wire byte total unchanged.
//
// v3 (snake_case) is the default emit; v3.1 emits the canonical `_camelCase`
// form. Round-trips identically — both names accepted on input.

pub const FIELD_ALIASES_V3_1: &[(&str, &str)] = &[
    ("key", "_key"),
    ("string_key", "_stringKey"),
    ("is_blocked", "_isBlocked"),
    ("sound_event", "_soundEvent"),
    ("foot_step_sound_event", "_footStepSoundEvent"),
    ("foot_step_crouch_sound_event", "_footStepCrouchSoundEvent"),
    ("foot_step_land_sound_event", "_footStepLandSoundEvent"),
    ("foot_step_ground_sound_event", "_footStepGroundSoundEvent"),
    ("foot_step_disable_collide_impact_sound", "_footStepDisableCollideImpactSound"),
    ("foot_step_sound_offset", "_footStepSoundOffset"),
    ("foot_step_crouch_sound_offset", "_footStepCrouchSoundOffset"),
    ("foot_step_land_sound_offset", "_footStepLandSoundOffset"),
    ("foot_step_ground_sound_offset", "_footStepGroundSoundOffset"),
    ("gender", "_gender"),
    ("character_age", "_characterAge"),
    ("job_info_list", "_jobInfoList"),
];
