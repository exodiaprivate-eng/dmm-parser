// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Typed decoding for `.paatt` `BaseData` blobs.
//!
//! `BaseData` inside each `AttackInfo` is a version-sized raw byte blob.
//! This module provides typed wrappers that expose individual fields for V0
//! (base attacks, 79.8% of vanilla records) and V1 (catch attacks, 12.6%)
//! so mod authors can read and write them by name.
//!
//! ## Field-identification method
//!
//! Offsets were confirmed by differential entropy analysis of 10,562 V0 and
//! 1,674 V1 vanilla records (Session 12–13). Fields marked `_unkXXXX` are
//! confirmed real storage positions but their C++ names are not yet resolved.
//!
//! ## Round-trip guarantee
//!
//! `BaseDataV0::parse` + `BaseDataV0::to_bytes` is byte-exact against every
//! vanilla record — every byte of the 264-byte blob is either a named field
//! or a raw `_unk` array, with nothing omitted.

use std::io;

use crate::json_shape::{
    apply_v3_aliases, normalize_input_aliases, FieldAliasTable, JsonShape,
};
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};

/// Per-table v3-shape aliases for `BaseDataV0`.
///
/// Each entry is `(canonical_T0_name, v3_legacy_name)`. The `to_json_value`
/// method emits canonical names; `to_json_value_shaped(JsonShape::V3)`
/// post-processes the map to rename canonical → legacy. The
/// `write_from_json` path normalizes incoming legacy → canonical so
/// either name is accepted on input.
///
/// **Currently empty.** Populate as Tier-0 renames ship — each
/// confirmed `_unkXXXX` → real-C++-name rename adds one entry here.
pub const FIELD_ALIASES_V3: FieldAliasTable = &[];

// ── helpers ──────────────────────────────────────────────────────────────────

fn read_f32(data: &[u8], off: usize) -> f32 {
    f32::from_le_bytes(data[off..off + 4].try_into().unwrap())
}

fn read_u32(data: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(data[off..off + 4].try_into().unwrap())
}

fn read_u16(data: &[u8], off: usize) -> u16 {
    u16::from_le_bytes(data[off..off + 2].try_into().unwrap())
}

fn f32_to_json(f: f32) -> serde_json::Value {
    serde_json::Number::from_f64(f as f64)
        .map(serde_json::Value::Number)
        .unwrap_or(serde_json::Value::Null)
}

fn json_f32(obj: &serde_json::Map<String, serde_json::Value>, key: &str) -> io::Result<f32> {
    Ok(json_get_field(obj, key)?.as_f64().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData,
        format!("BaseDataV0.{}: expected number", key),
    ))? as f32)
}

fn json_u64(obj: &serde_json::Map<String, serde_json::Value>, key: &str) -> io::Result<u64> {
    json_get_field(obj, key)?.as_u64().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData,
        format!("BaseDataV0.{}: expected unsigned integer", key),
    ))
}

fn json_b64(
    obj: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    expected_len: usize,
) -> io::Result<Vec<u8>> {
    use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
    let s = json_get_field(obj, key)?.as_str().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData,
        format!("BaseDataV0.{}: expected string", key),
    ))?;
    let bytes = B64.decode(s).map_err(|e| io::Error::new(
        io::ErrorKind::InvalidData,
        format!("BaseDataV0.{}: base64 decode: {}", key, e),
    ))?;
    if bytes.len() != expected_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("BaseDataV0.{}: expected {} bytes, got {}", key, expected_len, bytes.len()),
        ));
    }
    Ok(bytes)
}

fn json_float3(obj: &serde_json::Map<String, serde_json::Value>, key: &str) -> io::Result<[f32; 3]> {
    let arr = json_get_field(obj, key)?.as_array().ok_or_else(|| io::Error::new(
        io::ErrorKind::InvalidData,
        format!("BaseDataV0.{}: expected array of 3 floats", key),
    ))?;
    if arr.len() != 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("BaseDataV0.{}: expected 3 elements, got {}", key, arr.len()),
        ));
    }
    Ok([
        arr[0].as_f64().unwrap_or(0.0) as f32,
        arr[1].as_f64().unwrap_or(0.0) as f32,
        arr[2].as_f64().unwrap_or(0.0) as f32,
    ])
}

fn float3_to_json(v: [f32; 3]) -> serde_json::Value {
    serde_json::Value::Array(v.iter().map(|&f| f32_to_json(f)).collect())
}

// ── V0 — base AttackInfo (264 bytes) ────────────────────────────────────────

/// Typed BaseData for AttackInfo version 0 (and the shared prefix of V1..V3).
///
/// Named fields are byte-exact at the stated offsets.  `_unk` regions hold
/// real data at confirmed positions whose C++ field names are not yet fully
/// resolved — they are preserved verbatim to maintain byte-perfect roundtrips.
///
/// ## Byte layout (264 bytes total)
///
/// ```text
/// 0x0000  weapon_key                  u32     [4]   weapon / action hash key
/// 0x0004  attack_dir                  u8      [1]   0=forward; 1=catch dir (V1 always)
/// 0x0005  _pad0005                    [u8;3]  [3]
/// 0x0008  attack_pos_offset           [f32;3] [12]  AttackCommonDataDesc.AttackPosOffset
/// 0x0014  _unk_float3_0014            [f32;3] [12]  AttackCommonDataDesc (unnamed float3)
/// 0x0020  attack_degree               f32     [4]   AttackCommonDataDesc.AttackDegree
///                                                   default ≈ 6.2832 (2π = 360°)
/// 0x0024  attack_yaw                  f32     [4]   AttackCommonDataDesc.AttackYaw (0° def)
/// 0x0028  _unk_f32_0028               f32     [4]   AttackCommonDataDesc (unnamed float)
/// 0x002c  physic_impulse_power        f32     [4]   vanilla default 1.0
/// 0x0030  physics_impulse_mass        f32     [4]   vanilla default 1.0
/// 0x0034  attack_hit_check_type       u16     [2]   enum; common value 4 = normal check
/// 0x0036  hit_check_normal_str_idx    u16     [2]   NormalStringIndex; 0xffff=none (98%); values 3-8
/// 0x0038  repeat_degree_weight        f32     [4]   vanilla default -1.0
/// 0x003c  physics_impulse_velocity    f32     [4]   vanilla default 0.0
/// 0x0040  ignore_safe_zone            bool    [1]
/// 0x0041  _pad0041                    [u8;3]  [3]
/// 0x0044  attack_group_index          u8      [1]   common value 1
/// 0x0045  repeat_count                u8      [1]   common value 2
/// 0x0046  _pad0046                    [u8;2]  [2]
/// 0x0048  hit_effect_info_type        u32     [4]   effect hash; 0xf177b780 most common
/// 0x004c  single_hit_pos_offset       [f32;3] [12]  singleHitPositionOffset XYZ; (0,0,0)
///                                                   default (V2/V3 always zero)
/// 0x0058  _ds1_f0                     f32     [4]   AttackDelayDataDesc #1: trigger time (s);
///                                                   0.0=immediate (49%); 0.05/0.10/0.01 typical
/// 0x005c  _ds1_f1                     f32     [4]   AttackDelayDataDesc #1: blend/end factor;
///                                                   0.0=point event (85%); 1.0=full-duration (9%)
/// 0x0060  _ds1_f2                     f32     [4]   AttackDelayDataDesc #1: secondary offset (s);
///                                                   0.0/0.20/0.25/0.10 typical
/// 0x0064  _ds1_f3                     f32     [4]   AttackDelayDataDesc #1: angle modifier (rad);
///                                                   100% zero except 13 records (V2 always 0)
/// 0x0068  _ds1_f4                     f32     [4]   AttackDelayDataDesc #1: integer frame count;
///                                                   0/6/9/11 typical; ds2 is almost always 0
/// 0x006c  _pad_ds1                    [u8;4]  [4]   always zero
/// 0x0070  normal_string_index         u16     [2]   AttackCommonDataDesc.NormalStringIndex;
///                                                   V0: 0x0000 @70%; V2/V3: 0xffff (always)
/// 0x0072  _unk0072                    bool    [1]   bool (B00@85% V0)
/// 0x0073  _unk0073                    u8      [1]   u8 enum (mode=1 @49% V0)
/// 0x0074  _pad0074                    [u8;4]  [4]   always zero
/// 0x0078  _ds2_f0                     f32     [4]   AttackDelayDataDesc #2: trigger time (s);
///                                                   98.4% == ds1_f0
/// 0x007c  _ds2_f1                     f32     [4]   AttackDelayDataDesc #2: blend/end factor;
///                                                   98.8% == ds1_f1
/// 0x0080  _ds2_f2                     f32     [4]   AttackDelayDataDesc #2: secondary offset (s);
///                                                   98.4% == ds1_f2
/// 0x0084  _ds2_f3                     f32     [4]   AttackDelayDataDesc #2: angle modifier (rad);
///                                                   99.9% zero
/// 0x0088  _ds2_f4                     f32     [4]   AttackDelayDataDesc #2: frame/scale factor;
///                                                   99% zero; non-zero ds2 values are 1.1-1.53
/// 0x008c  _pad_ds2                    [u8;4]  [4]   always zero
/// 0x0090  hit_degree                  f32     [4]   AttackHitDataDesc.Degree (degrees);
///                                                   V0 mode=50.0; V2/V3 always 0.0
/// 0x0094  _pad0094                    [u8;8]  [8]   always zero
/// 0x009c  hit_rotation_type           u8      [1]   AttackHitDataDesc.HitRotationType enum;
///                                                   V0 mode=0x00; 0x5e exclusively with _unk00b8=0x00
///                                                   (throw-linked); V2 CONST=0x7a; V3 CONST=0x5a
/// 0x009d  _pad009d                    [u8;3]  [3]   always zero
/// 0x00a0  _unk_f32_00a0               f32     [4]   mostly 0.0
/// 0x00a4  _pad00a4                    [u8;4]  [4]   always zero
/// 0x00a8  equip_slot_name_key         u8      [1]   AttackCommonDataDesc.EquipSlotNameKey;
///                                                   V0 mode=12; V2/V3 always 23
/// 0x00a9  _pad00a9                    [u8;3]  [3]   always zero
/// 0x00ac  _unk00ac                    u8      [1]   u8 ≈0
/// 0x00ad  _unk00ad                    bool    [1]   rarely true
/// 0x00ae  _pad00ae                    [u8;2]  [2]   always zero
/// 0x00b0  exclude_target_type_flag    u32     [4]   AttackInfoDataDesc.excludeTargetTypeFlag;
///                                                   bitmask; 77.5% zero; 25 distinct
/// 0x00b4  single_hit_position_socket  u16     [2]   socket_name_table idx; 0xffff=none
/// 0x00b6  _pad00b6                    [u8;2]  [2]
/// 0x00b8  _unk00b8                    u8      [1]   attack sub-type byte; V0: 0x03@37%, 0x00@27%,
///                                                   0x08@18%, 0x05@12%, 0xff@4%; V2 CONST=0x00;
///                                                   V3 CONST=0xff; V1: 0x08@51%, 0x05@24%
/// 0x00b9  _unk00b9                    u8      [1]   0x07 exclusively when _unk00b8=0x00+_unk009c=0x5e
///                                                   (14% of V0); otherwise 0x00
/// 0x00ba  _unk00ba                    u8      [1]   mode≈0 (98% zero all versions)
/// 0x00bb  _unk00bb                    u8      [1]   mode=5 (V2/V3 always 5; V0: 9 distinct)
/// 0x00bc  _pad00bc                    u8      [1]   always 0
/// 0x00bd  _unk00bd                    bool    [1]   V0 rarely true; V2/V3 always false
/// 0x00be  _pad00be                    u8      [1]   always 0
/// 0x00bf  _unk00bf                    bool    [1]   V0 rarely true; V2/V3 always false
/// 0x00c0  _unk00c0                    bool    [1]   near-const false; V2/V3 always false
/// 0x00c1  _pad00c1                    [u8;3]  [3]   always zero
/// 0x00c4  _pad00c4                    [u8;4]  [4]   always zero
/// 0x00c8  _pad00c8                    u8      [1]   CONST 0x01 (hardcoded)
/// 0x00c9  _unk00c9                    u8      [1]   mode=1 V0/V2; mode=2 V3
/// 0x00ca  _pad00ca                    u8      [1]   CONST 0x01 (hardcoded)
/// 0x00cb  _unk00cb                    u8      [1]   mode=1 V0/V2; mode=2 V3
/// 0x00cc  _unk00cc                    u8      [1]   mode=1 V0/V2; mode=2 V3
/// 0x00cd  _pad00cd                    u8      [1]   CONST 0x01 (hardcoded)
/// 0x00ce  _unk00ce                    u8      [1]   mode=1 V0/V2; mode=2 V3
/// 0x00cf  _unk00cf                    u8      [1]   V0: 0x19@76% (only when b8∈{0x03,0x08,0x05}),
///                                                   0x00 when b8=0x00+b9=0x07, 0x04 when b8=0x00+other;
///                                                   V2: 0x02@64%, 0x05@34%; V3 CONST=0x04
/// 0x00d0  _unk00d0                    bool    [1]   rarely true (all versions)
/// 0x00d1  _unk00d1                    bool    [1]   rarely true V0; false V2/V3
/// 0x00d2  _unk00d2                    bool    [1]   rarely true V0/V2; always false V3
/// 0x00d3  _unk00d3                    bool    [1]   V0 mode=true; V2=false; V3 CONST=true
/// 0x00d4  _unk_f32_00d4               f32     [4]   CONST 0.0 when _unk00b8∈{0x00,0xff};
///                                                   0.0@59% or 198.0@41% when b8∈{0x03,0x08,0x05}
/// 0x00d8  _pad00d8                    [u8;4]  [4]   always zero
/// 0x00dc  _unk_f32_00dc               f32     [4]   ≈0.0 (V2/V3 always 0)
/// 0x00e0  _unk_f32_00e0               f32     [4]   ≈0.0
/// 0x00e4  _unk_f32_00e4               f32     [4]   ≈0.0
/// 0x00e8  _unk00e8                    u8      [1]   CONST 0x01
/// 0x00e9  _unk00e9                    bool    [1]   rarely true
/// 0x00ea  _pad00ea                    [u8;2]  [2]   always zero
/// 0x00ec  _pad00ec                    [u8;4]  [4]   always zero
/// 0x00f0  hit_normal_string_index     u16     [2]   AttackHitDataDesc.NormalStringIndex;
///                                                   V0 often 1021 (0x03fd); V2/V3 often 0
/// 0x00f2  _pad00f2                    [u8;2]  [2]   always zero
/// 0x00f4  _unk00f4                    u8      [1]   AttackHitDataDesc field 6 (unnamed u8);
///                                                   1=no-rotation (pairs with hit_degree=0°);
///                                                   5/2/6/4/3=rotation-type (hit_degree=50°)
/// 0x00f5  _pad00f5                    [u8;3]  [3]   always zero
/// 0x00f8  hit_data_str_idx            u16     [2]   NormalStringIndex; 0=none; 95%+ non-zero
///                                                   cases pair with attack_hit_check_type=4;
///                                                   V2/V3 always 0
/// 0x00fa  _unk00fa                    bool    [1]   bool (V2/V3 always false)
/// 0x00fb  _pad00fb                    u8      [1]   always 0
/// 0x00fc  _unk00fc                    u8      [1]   AttackHitDataDesc field 7 (unnamed bool/u8);
///                                                   0=false (when _unk00f4=1, no-rotation);
///                                                   1=true (when _unk00f4=5/2/6/4/3)
/// 0x00fd  _pad00fd                    [u8;3]  [3]   always zero
/// 0x0100  hit_data_str_idx_b          u16     [2]   secondary NormalStringIndex; 2% non-zero;
///                                                   always co-present with hit_data_str_idx;
///                                                   values in same range (0x0450–0x046d)
/// 0x0102  _pad0102                    [u8;2]  [2]   always zero
/// 0x0104  _unk0104                    u8      [1]   u8, 4 values (98% zero; no clear correlation)
/// 0x0105  _pad0105                    [u8;3]  [3]   always zero
///         ─────────────────────────────────────────
///         TOTAL                               264
/// ```
#[derive(Debug, Clone)]
pub struct BaseDataV0 {
    pub weapon_key: u32,
    pub attack_dir: u8,
    pub _pad0005: [u8; 3],

    /// `AttackCommonDataDesc.attackOffset` — world-space origin of the attack hitbox
    /// (Mac binary names this `attackOffset`; older WIN-IDA notes used `AttackPosOffset`).
    pub attack_pos_offset: [f32; 3],
    /// `AttackCommonDataDesc` unnamed float3 (purpose TBD from further analysis).
    /// Session 19 IDA candidate: `attackBoxSize` — the next 12-byte field after
    /// `attackOffset` in the C++ class. See `docs/PAATT_BASEDATA_FIELDS.md`.
    pub _unk_float3_0014: [f32; 3],
    /// `AttackCommonDataDesc.attackAngle` — angular width of the attack arc in radians.
    /// Vanilla default ≈ 6.2832 (2π = 360°, full-circle hitbox). Mac C++ name is
    /// `attackAngle`; older WIN-IDA notes used `AttackDegree`.
    pub attack_degree: f32,
    /// `AttackCommonDataDesc.attackYaw` — yaw rotation of the attack arc (radians).
    /// Vanilla default 0.0.
    pub attack_yaw: f32,
    /// `AttackCommonDataDesc` unnamed float (purpose TBD).
    /// Session 19 IDA candidate: `innerAttackLength` (next f32 after `attackYaw`).
    pub _unk_f32_0028: f32,

    /// Scales the physics impulse applied to the target.  Vanilla default: 1.0.
    pub physic_impulse_power: f32,
    /// Simulated mass for physics impulse calculation.  Vanilla default: 1.0.
    pub physics_impulse_mass: f32,

    /// `attackHitCheckType` enum — determines which hitbox collision test to use.
    /// Common value: 4 (standard sphere/box check).
    pub attack_hit_check_type: u16,
    /// Likely `NormalStringIndex` from `AttackHitDataDesc`; 0xffff = no string.
    pub hit_check_normal_str_idx: u16,

    /// Weight applied to repeat-hit degree variation.  Vanilla default: -1.0.
    pub repeat_degree_weight: f32,
    /// Physics impulse velocity applied on hit.  Vanilla default: 0.0.
    pub physics_impulse_velocity: f32,

    /// When true, this attack ignores safe zones (hit suppression areas).
    pub ignore_safe_zone: bool,
    pub _pad0041: [u8; 3],

    /// Attack group index — groups mutually-exclusive attacks.  Common value: 1.
    pub attack_group_index: u8,
    /// Number of hit detections to run per attack frame event.  Common value: 2.
    pub repeat_count: u8,
    pub _pad0046: [u8; 2],

    /// Hit effect info type hash.  Most attacks share value 0xf177b780 (standard
    /// hit effect); specialized attacks use different hashes.
    pub hit_effect_info_type: u32,

    /// `singleHitPositionOffset` — XYZ world-space offset for the single-hit position
    /// anchor.  Used when `single_hit_position_socket` = 0xffff.  Vanilla default: (0,0,0).
    /// V2 (throw) and V3 (release-catch) records always have (0,0,0) here.
    pub single_hit_pos_offset: [f32; 3],

    /// `ActionChartFrameEvent_AttackDelayDataDesc` #1 — five floats.
    /// V2 defaults: 0.01 / 1.0 / 0.0 / ≈0 / 0.0.
    pub _ds1_f0: f32,
    pub _ds1_f1: f32,
    pub _ds1_f2: f32,
    pub _ds1_f3: f32,
    pub _ds1_f4: f32,
    pub _pad_ds1: [u8; 4],
    /// `AttackCommonDataDesc.NormalStringIndex`. V0: 0x0000 @70%; V2/V3: 0xffff (always).
    pub normal_string_index: u16,
    /// bool (B00@85% V0).
    /// Session 19 IDA candidate: `pa::AttackInfoDataDesc::noCheckCollision`
    /// (in-mem class offset 0xB5, single-byte bool sitting in the same
    /// register-cluster as ignoreSafeZone). The wire→class mapping isn't
    /// proven yet — see `docs/PAATT_BASEDATA_FIELDS.md` § Appendix.
    pub _unk0072: bool,
    /// u8 enum (mode=1 @49% V0).
    /// Session 19 IDA candidate: `pa::AttackInfoDataDesc::attackImpulseLevel`
    /// (in-mem class offset 0xB0, sole u8 enum field still unmapped).
    /// Wire→class mapping unproven; see PAATT_BASEDATA_FIELDS.md § Appendix.
    pub _unk0073: u8,
    pub _pad0074: [u8; 4],
    /// `ActionChartFrameEvent_AttackDelayDataDesc` #2 — same structure as ds1.
    /// V2 defaults: 0.01 / 1.0 / 0.0 / 0.0 / 0.0.
    pub _ds2_f0: f32,
    pub _ds2_f1: f32,
    pub _ds2_f2: f32,
    pub _ds2_f3: f32,
    pub _ds2_f4: f32,
    pub _pad_ds2: [u8; 4],
    /// `AttackHitDataDesc.hitRotationAngle` — hit arc width in degrees. V0 mode 50.0;
    /// V2/V3 always 0.0. Mac C++ name is `hitRotationAngle`; field stays as
    /// `hit_degree` for backwards compatibility (rename pending serializer-order
    /// confirmation).
    pub hit_degree: f32,
    pub _pad0094: [u8; 8],
    /// `AttackHitDataDesc.hitRotationType` enum. V0 mode=0; V2=0x7a (122); V3=0x5a (90).
    pub hit_rotation_type: u8,
    pub _pad009d: [u8; 3],
    /// Likely `AttackHitDataDesc` unnamed float. Usually 0.0.
    /// Session 19 IDA candidate: `pushSpeed` — the next f32 after `hitRotationAngle`
    /// in the C++ class layout (in-mem 0x10), tracking knockback velocity.
    pub _unk_f32_00a0: f32,
    pub _pad00a4: [u8; 4],
    /// `AttackCommonDataDesc.EquipSlotNameKey` enum. V0 mode=12; V2/V3 always 23.
    pub equip_slot_name_key: u8,
    pub _pad00a9: [u8; 3],
    /// u8, nearly always 0.
    pub _unk00ac: u8,
    /// bool, rarely true.
    pub _unk00ad: bool,
    pub _pad00ae: [u8; 2],
    /// `AttackInfoDataDesc.excludeTargetTypeFlag` — bitmask; 77.5% zero.
    pub exclude_target_type_flag: u32,

    /// Index into `socket_name_table` for single-hit position anchor.
    /// 0xffff = no socket (use world-space offset from `_unk004c` instead).
    pub single_hit_position_socket: u16,
    pub _pad00b6: [u8; 2],

    /// Version-discriminating: V0 mode=3; V2=0x00; V3=0xff.
    pub _unk00b8: u8,
    pub _unk00b9: u8,
    pub _unk00ba: u8,
    /// Mode=5 across V0/V2/V3.
    pub _unk00bb: u8,
    pub _pad00bc: u8,
    /// V0 rarely true; V2/V3 always false.
    pub _unk00bd: bool,
    pub _pad00be: u8,
    /// V0 rarely true; V2/V3 always false.
    pub _unk00bf: bool,
    /// Near-const false; V2/V3 always false.
    pub _unk00c0: bool,
    pub _pad00c1: [u8; 3],
    pub _pad00c4: [u8; 4],
    /// Mode=1 V0/V2; mode=2 V3.
    pub _unk00c9: u8,
    /// Mode=1 V0/V2; mode=2 V3.
    pub _unk00cb: u8,
    /// Mode=1 V0/V2; mode=2 V3.
    pub _unk00cc: u8,
    /// Mode=1 V0/V2; mode=2 V3.
    pub _unk00ce: u8,
    /// V0 mode=25; V2 mode=2; V3 CONST=4.
    pub _unk00cf: u8,
    pub _unk00d0: bool,
    pub _unk00d1: bool,
    pub _unk00d2: bool,
    pub _unk00d3: bool,
    /// Integer-valued f32 (5 distinct values, ≈0).
    pub _unk_f32_00d4: f32,
    pub _pad00d8: [u8; 4],
    /// ≈0.0 (V2/V3 always 0, V0 sometimes non-zero).
    pub _unk_f32_00dc: f32,
    /// ≈0.0.
    pub _unk_f32_00e0: f32,
    /// ≈0.0.
    pub _unk_f32_00e4: f32,
    /// CONST 0x01.
    pub _unk00e8: u8,
    /// Rarely true.
    pub _unk00e9: bool,
    pub _pad00ea: [u8; 2],
    pub _pad00ec: [u8; 4],
    /// NormalStringIndex? V0 often 1021, V2/V3 often 0.
    pub hit_normal_string_index: u16,
    pub _pad00f2: [u8; 2],
    /// Unnamed u8 (AttackHitDataDesc?).
    pub _unk00f4: u8,
    pub _pad00f5: [u8; 3],
    /// u16 (V2/V3 always 0).
    pub hit_data_str_idx: u16,
    /// bool (V2/V3 always false).
    pub _unk00fa: bool,
    pub _pad00fb: u8,
    /// u8 enum, 8 values.
    pub _unk00fc: u8,
    pub _pad00fd: [u8; 3],
    /// Secondary NormalStringIndex for hit data; 0=none; always co-present with hit_data_str_idx.
    pub hit_data_str_idx_b: u16,
    pub _pad0102: [u8; 2],
    /// u8, 4 values.
    pub _unk0104: u8,
    pub _pad0105: [u8; 3],
}

impl BaseDataV0 {
    pub const SIZE: usize = 264;

    pub fn parse(data: &[u8]) -> io::Result<Self> {
        if data.len() != Self::SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("BaseDataV0: expected {} bytes, got {}", Self::SIZE, data.len()),
            ));
        }
        Ok(Self {
            weapon_key:   read_u32(data, 0x0000),
            attack_dir:   data[0x0004],
            _pad0005:     data[0x0005..0x0008].try_into().unwrap(),

            attack_pos_offset:   [read_f32(data, 0x0008), read_f32(data, 0x000c), read_f32(data, 0x0010)],
            _unk_float3_0014:    [read_f32(data, 0x0014), read_f32(data, 0x0018), read_f32(data, 0x001c)],
            attack_degree:       read_f32(data, 0x0020),
            attack_yaw:          read_f32(data, 0x0024),
            _unk_f32_0028:       read_f32(data, 0x0028),

            physic_impulse_power:  read_f32(data, 0x002c),
            physics_impulse_mass:  read_f32(data, 0x0030),

            attack_hit_check_type: read_u16(data, 0x0034),
            hit_check_normal_str_idx:              read_u16(data, 0x0036),
            repeat_degree_weight:  read_f32(data, 0x0038),
            physics_impulse_velocity: read_f32(data, 0x003c),

            ignore_safe_zone: data[0x0040] != 0,
            _pad0041: data[0x0041..0x0044].try_into().unwrap(),

            attack_group_index: data[0x0044],
            repeat_count:       data[0x0045],
            _pad0046: data[0x0046..0x0048].try_into().unwrap(),

            hit_effect_info_type: read_u32(data, 0x0048),

            single_hit_pos_offset: [read_f32(data, 0x004c), read_f32(data, 0x0050), read_f32(data, 0x0054)],

            _ds1_f0: read_f32(data, 0x0058),
            _ds1_f1: read_f32(data, 0x005c),
            _ds1_f2: read_f32(data, 0x0060),
            _ds1_f3: read_f32(data, 0x0064),
            _ds1_f4: read_f32(data, 0x0068),
            _pad_ds1: data[0x006c..0x0070].try_into().unwrap(),
            normal_string_index: read_u16(data, 0x0070),
            _unk0072: data[0x0072] != 0,
            _unk0073: data[0x0073],
            _pad0074: data[0x0074..0x0078].try_into().unwrap(),
            _ds2_f0: read_f32(data, 0x0078),
            _ds2_f1: read_f32(data, 0x007c),
            _ds2_f2: read_f32(data, 0x0080),
            _ds2_f3: read_f32(data, 0x0084),
            _ds2_f4: read_f32(data, 0x0088),
            _pad_ds2: data[0x008c..0x0090].try_into().unwrap(),
            hit_degree: read_f32(data, 0x0090),
            _pad0094: data[0x0094..0x009c].try_into().unwrap(),
            hit_rotation_type: data[0x009c],
            _pad009d: data[0x009d..0x00a0].try_into().unwrap(),
            _unk_f32_00a0: read_f32(data, 0x00a0),
            _pad00a4: data[0x00a4..0x00a8].try_into().unwrap(),
            equip_slot_name_key: data[0x00a8],
            _pad00a9: data[0x00a9..0x00ac].try_into().unwrap(),
            _unk00ac: data[0x00ac],
            _unk00ad: data[0x00ad] != 0,
            _pad00ae: data[0x00ae..0x00b0].try_into().unwrap(),
            exclude_target_type_flag: read_u32(data, 0x00b0),

            single_hit_position_socket: read_u16(data, 0x00b4),
            _pad00b6: data[0x00b6..0x00b8].try_into().unwrap(),
            _unk00b8: data[0x00b8],
            _unk00b9: data[0x00b9],
            _unk00ba: data[0x00ba],
            _unk00bb: data[0x00bb],
            _pad00bc: data[0x00bc],
            _unk00bd: data[0x00bd] != 0,
            _pad00be: data[0x00be],
            _unk00bf: data[0x00bf] != 0,
            _unk00c0: data[0x00c0] != 0,
            _pad00c1: data[0x00c1..0x00c4].try_into().unwrap(),
            _pad00c4: data[0x00c4..0x00c8].try_into().unwrap(),
            _unk00c9: data[0x00c9],
            _unk00cb: data[0x00cb],
            _unk00cc: data[0x00cc],
            _unk00ce: data[0x00ce],
            _unk00cf: data[0x00cf],
            _unk00d0: data[0x00d0] != 0,
            _unk00d1: data[0x00d1] != 0,
            _unk00d2: data[0x00d2] != 0,
            _unk00d3: data[0x00d3] != 0,
            _unk_f32_00d4: read_f32(data, 0x00d4),
            _pad00d8:      data[0x00d8..0x00dc].try_into().unwrap(),
            _unk_f32_00dc: read_f32(data, 0x00dc),
            _unk_f32_00e0: read_f32(data, 0x00e0),
            _unk_f32_00e4: read_f32(data, 0x00e4),
            _unk00e8: data[0x00e8],
            _unk00e9: data[0x00e9] != 0,
            _pad00ea: data[0x00ea..0x00ec].try_into().unwrap(),
            _pad00ec:      data[0x00ec..0x00f0].try_into().unwrap(),
            hit_normal_string_index:      read_u16(data, 0x00f0),
            _pad00f2:      data[0x00f2..0x00f4].try_into().unwrap(),
            _unk00f4:      data[0x00f4],
            _pad00f5:      data[0x00f5..0x00f8].try_into().unwrap(),
            hit_data_str_idx: read_u16(data, 0x00f8),
            _unk00fa: data[0x00fa] != 0,
            _pad00fb: data[0x00fb],
            _unk00fc:      data[0x00fc],
            _pad00fd:      data[0x00fd..0x0100].try_into().unwrap(),
            hit_data_str_idx_b:      read_u16(data, 0x0100),
            _pad0102:      data[0x0102..0x0104].try_into().unwrap(),
            _unk0104:      data[0x0104],
            _pad0105:      data[0x0105..0x0108].try_into().unwrap(),
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut out = [0u8; Self::SIZE];
        out[0x0000..0x0004].copy_from_slice(&self.weapon_key.to_le_bytes());
        out[0x0004] = self.attack_dir;
        out[0x0005..0x0008].copy_from_slice(&self._pad0005);

        for (i, &f) in self.attack_pos_offset.iter().enumerate() {
            out[0x0008 + i * 4..0x000c + i * 4].copy_from_slice(&f.to_le_bytes());
        }
        for (i, &f) in self._unk_float3_0014.iter().enumerate() {
            out[0x0014 + i * 4..0x0018 + i * 4].copy_from_slice(&f.to_le_bytes());
        }
        out[0x0020..0x0024].copy_from_slice(&self.attack_degree.to_le_bytes());
        out[0x0024..0x0028].copy_from_slice(&self.attack_yaw.to_le_bytes());
        out[0x0028..0x002c].copy_from_slice(&self._unk_f32_0028.to_le_bytes());

        out[0x002c..0x0030].copy_from_slice(&self.physic_impulse_power.to_le_bytes());
        out[0x0030..0x0034].copy_from_slice(&self.physics_impulse_mass.to_le_bytes());

        out[0x0034..0x0036].copy_from_slice(&self.attack_hit_check_type.to_le_bytes());
        out[0x0036..0x0038].copy_from_slice(&self.hit_check_normal_str_idx.to_le_bytes());
        out[0x0038..0x003c].copy_from_slice(&self.repeat_degree_weight.to_le_bytes());
        out[0x003c..0x0040].copy_from_slice(&self.physics_impulse_velocity.to_le_bytes());

        out[0x0040] = self.ignore_safe_zone as u8;
        out[0x0041..0x0044].copy_from_slice(&self._pad0041);

        out[0x0044] = self.attack_group_index;
        out[0x0045] = self.repeat_count;
        out[0x0046..0x0048].copy_from_slice(&self._pad0046);

        out[0x0048..0x004c].copy_from_slice(&self.hit_effect_info_type.to_le_bytes());
        for (i, &f) in self.single_hit_pos_offset.iter().enumerate() {
            out[0x004c + i * 4..0x0050 + i * 4].copy_from_slice(&f.to_le_bytes());
        }

        out[0x0058..0x005c].copy_from_slice(&self._ds1_f0.to_le_bytes());
        out[0x005c..0x0060].copy_from_slice(&self._ds1_f1.to_le_bytes());
        out[0x0060..0x0064].copy_from_slice(&self._ds1_f2.to_le_bytes());
        out[0x0064..0x0068].copy_from_slice(&self._ds1_f3.to_le_bytes());
        out[0x0068..0x006c].copy_from_slice(&self._ds1_f4.to_le_bytes());
        out[0x006c..0x0070].copy_from_slice(&self._pad_ds1);
        out[0x0070..0x0072].copy_from_slice(&self.normal_string_index.to_le_bytes());
        out[0x0072] = self._unk0072 as u8;
        out[0x0073] = self._unk0073;
        out[0x0074..0x0078].copy_from_slice(&self._pad0074);
        out[0x0078..0x007c].copy_from_slice(&self._ds2_f0.to_le_bytes());
        out[0x007c..0x0080].copy_from_slice(&self._ds2_f1.to_le_bytes());
        out[0x0080..0x0084].copy_from_slice(&self._ds2_f2.to_le_bytes());
        out[0x0084..0x0088].copy_from_slice(&self._ds2_f3.to_le_bytes());
        out[0x0088..0x008c].copy_from_slice(&self._ds2_f4.to_le_bytes());
        out[0x008c..0x0090].copy_from_slice(&self._pad_ds2);
        out[0x0090..0x0094].copy_from_slice(&self.hit_degree.to_le_bytes());
        out[0x0094..0x009c].copy_from_slice(&self._pad0094);
        out[0x009c] = self.hit_rotation_type;
        out[0x009d..0x00a0].copy_from_slice(&self._pad009d);
        out[0x00a0..0x00a4].copy_from_slice(&self._unk_f32_00a0.to_le_bytes());
        out[0x00a4..0x00a8].copy_from_slice(&self._pad00a4);
        out[0x00a8] = self.equip_slot_name_key;
        out[0x00a9..0x00ac].copy_from_slice(&self._pad00a9);
        out[0x00ac] = self._unk00ac;
        out[0x00ad] = self._unk00ad as u8;
        out[0x00ae..0x00b0].copy_from_slice(&self._pad00ae);
        out[0x00b0..0x00b4].copy_from_slice(&self.exclude_target_type_flag.to_le_bytes());

        out[0x00b4..0x00b6].copy_from_slice(&self.single_hit_position_socket.to_le_bytes());
        out[0x00b6..0x00b8].copy_from_slice(&self._pad00b6);
        out[0x00b8] = self._unk00b8;
        out[0x00b9] = self._unk00b9;
        out[0x00ba] = self._unk00ba;
        out[0x00bb] = self._unk00bb;
        out[0x00bc] = self._pad00bc;
        out[0x00bd] = self._unk00bd as u8;
        out[0x00be] = self._pad00be;
        out[0x00bf] = self._unk00bf as u8;
        out[0x00c0] = self._unk00c0 as u8;
        out[0x00c1..0x00c4].copy_from_slice(&self._pad00c1);
        out[0x00c4..0x00c8].copy_from_slice(&self._pad00c4);
        out[0x00c8] = 0x01;
        out[0x00c9] = self._unk00c9;
        out[0x00ca] = 0x01;
        out[0x00cb] = self._unk00cb;
        out[0x00cc] = self._unk00cc;
        out[0x00cd] = 0x01;
        out[0x00ce] = self._unk00ce;
        out[0x00cf] = self._unk00cf;
        out[0x00d0] = self._unk00d0 as u8;
        out[0x00d1] = self._unk00d1 as u8;
        out[0x00d2] = self._unk00d2 as u8;
        out[0x00d3] = self._unk00d3 as u8;
        out[0x00d4..0x00d8].copy_from_slice(&self._unk_f32_00d4.to_le_bytes());
        out[0x00d8..0x00dc].copy_from_slice(&self._pad00d8);
        out[0x00dc..0x00e0].copy_from_slice(&self._unk_f32_00dc.to_le_bytes());
        out[0x00e0..0x00e4].copy_from_slice(&self._unk_f32_00e0.to_le_bytes());
        out[0x00e4..0x00e8].copy_from_slice(&self._unk_f32_00e4.to_le_bytes());
        out[0x00e8] = self._unk00e8;
        out[0x00e9] = self._unk00e9 as u8;
        out[0x00ea..0x00ec].copy_from_slice(&self._pad00ea);
        out[0x00ec..0x00f0].copy_from_slice(&self._pad00ec);
        out[0x00f0..0x00f2].copy_from_slice(&self.hit_normal_string_index.to_le_bytes());
        out[0x00f2..0x00f4].copy_from_slice(&self._pad00f2);
        out[0x00f4] = self._unk00f4;
        out[0x00f5..0x00f8].copy_from_slice(&self._pad00f5);
        out[0x00f8..0x00fa].copy_from_slice(&self.hit_data_str_idx.to_le_bytes());
        out[0x00fa] = self._unk00fa as u8;
        out[0x00fb] = self._pad00fb;
        out[0x00fc] = self._unk00fc;
        out[0x00fd..0x0100].copy_from_slice(&self._pad00fd);
        out[0x0100..0x0102].copy_from_slice(&self.hit_data_str_idx_b.to_le_bytes());
        out[0x0102..0x0104].copy_from_slice(&self._pad0102);
        out[0x0104] = self._unk0104;
        out[0x0105..0x0108].copy_from_slice(&self._pad0105);
        out
    }
}

// ── JSON ─────────────────────────────────────────────────────────────────────

impl ToJsonValue for BaseDataV0 {
    fn to_json_value(&self) -> serde_json::Value {
        let mut m = serde_json::Map::new();
        m.insert("weapon_key".into(),              self.weapon_key.into());
        m.insert("attack_dir".into(),              self.attack_dir.into());
        m.insert("attack_pos_offset".into(),       float3_to_json(self.attack_pos_offset));
        m.insert("_unk_float3_0014".into(),        float3_to_json(self._unk_float3_0014));
        m.insert("attack_degree".into(),           f32_to_json(self.attack_degree));
        m.insert("attack_yaw".into(),              f32_to_json(self.attack_yaw));
        m.insert("_unk_f32_0028".into(),           f32_to_json(self._unk_f32_0028));
        m.insert("physic_impulse_power".into(),    f32_to_json(self.physic_impulse_power));
        m.insert("physics_impulse_mass".into(),    f32_to_json(self.physics_impulse_mass));
        m.insert("attack_hit_check_type".into(),   (self.attack_hit_check_type as u64).into());
        m.insert("hit_check_normal_str_idx".into(),                (self.hit_check_normal_str_idx as u64).into());
        m.insert("repeat_degree_weight".into(),    f32_to_json(self.repeat_degree_weight));
        m.insert("physics_impulse_velocity".into(),f32_to_json(self.physics_impulse_velocity));
        m.insert("ignore_safe_zone".into(),        self.ignore_safe_zone.into());
        m.insert("attack_group_index".into(),      self.attack_group_index.into());
        m.insert("repeat_count".into(),            self.repeat_count.into());
        m.insert("hit_effect_info_type".into(),    self.hit_effect_info_type.into());
        m.insert("single_hit_pos_offset".into(),   float3_to_json(self.single_hit_pos_offset));
        m.insert("_ds1_f0".into(),                 f32_to_json(self._ds1_f0));
        m.insert("_ds1_f1".into(),                 f32_to_json(self._ds1_f1));
        m.insert("_ds1_f2".into(),                 f32_to_json(self._ds1_f2));
        m.insert("_ds1_f3".into(),                 f32_to_json(self._ds1_f3));
        m.insert("_ds1_f4".into(),                 f32_to_json(self._ds1_f4));
        m.insert("normal_string_index".into(),                (self.normal_string_index as u64).into());
        m.insert("_unk0072".into(),                self._unk0072.into());
        m.insert("_unk0073".into(),                self._unk0073.into());
        m.insert("_ds2_f0".into(),                 f32_to_json(self._ds2_f0));
        m.insert("_ds2_f1".into(),                 f32_to_json(self._ds2_f1));
        m.insert("_ds2_f2".into(),                 f32_to_json(self._ds2_f2));
        m.insert("_ds2_f3".into(),                 f32_to_json(self._ds2_f3));
        m.insert("_ds2_f4".into(),                 f32_to_json(self._ds2_f4));
        m.insert("hit_degree".into(),           f32_to_json(self.hit_degree));
        m.insert("hit_rotation_type".into(),                self.hit_rotation_type.into());
        m.insert("_unk_f32_00a0".into(),           f32_to_json(self._unk_f32_00a0));
        m.insert("equip_slot_name_key".into(),                self.equip_slot_name_key.into());
        m.insert("_unk00ac".into(),                self._unk00ac.into());
        m.insert("_unk00ad".into(),                self._unk00ad.into());
        m.insert("exclude_target_type_flag".into(),                self.exclude_target_type_flag.into());
        m.insert("single_hit_position_socket".into(), (self.single_hit_position_socket as u64).into());
        m.insert("_unk00b8".into(),                self._unk00b8.into());
        m.insert("_unk00b9".into(),                self._unk00b9.into());
        m.insert("_unk00ba".into(),                self._unk00ba.into());
        m.insert("_unk00bb".into(),                self._unk00bb.into());
        m.insert("_unk00bd".into(),                self._unk00bd.into());
        m.insert("_unk00bf".into(),                self._unk00bf.into());
        m.insert("_unk00c0".into(),                self._unk00c0.into());
        m.insert("_unk00c9".into(),                self._unk00c9.into());
        m.insert("_unk00cb".into(),                self._unk00cb.into());
        m.insert("_unk00cc".into(),                self._unk00cc.into());
        m.insert("_unk00ce".into(),                self._unk00ce.into());
        m.insert("_unk00cf".into(),                self._unk00cf.into());
        m.insert("_unk00d0".into(),                self._unk00d0.into());
        m.insert("_unk00d1".into(),                self._unk00d1.into());
        m.insert("_unk00d2".into(),                self._unk00d2.into());
        m.insert("_unk00d3".into(),                (self._unk00d3 as u64).into());
        m.insert("_unk_f32_00d4".into(),           f32_to_json(self._unk_f32_00d4));
        m.insert("_unk_f32_00dc".into(),           f32_to_json(self._unk_f32_00dc));
        m.insert("_unk_f32_00e0".into(),           f32_to_json(self._unk_f32_00e0));
        m.insert("_unk_f32_00e4".into(),           f32_to_json(self._unk_f32_00e4));
        m.insert("_unk00e8".into(),                self._unk00e8.into());
        m.insert("_unk00e9".into(),                self._unk00e9.into());
        m.insert("hit_normal_string_index".into(),                (self.hit_normal_string_index as u64).into());
        m.insert("_unk00f4".into(),                self._unk00f4.into());
        m.insert("hit_data_str_idx".into(),                (self.hit_data_str_idx as u64).into());
        m.insert("_unk00fa".into(),                self._unk00fa.into());
        m.insert("_unk00fc".into(),                self._unk00fc.into());
        m.insert("hit_data_str_idx_b".into(),                (self.hit_data_str_idx_b as u64).into());
        m.insert("_unk0104".into(),                self._unk0104.into());
        serde_json::Value::Object(m)
    }
}

impl BaseDataV0 {
    /// Emit JSON with the requested shape:
    /// * `JsonShape::V3` — applies any v3-legacy renames from `FIELD_ALIASES_V3`.
    ///   Until the alias table starts getting entries, this is byte-identical
    ///   to `to_json_value()`.
    /// * `JsonShape::V3_1` — emits canonical names verbatim (same as
    ///   `to_json_value()` today).
    pub fn to_json_value_shaped(&self, shape: JsonShape) -> serde_json::Value {
        let mut v = self.to_json_value();
        if shape == JsonShape::V3 {
            if let Some(map) = v.as_object_mut() {
                apply_v3_aliases(map, FIELD_ALIASES_V3);
            }
        }
        v
    }
}

impl WriteJsonValue for BaseDataV0 {
    fn write_from_json(w: &mut Vec<u8>, v: &serde_json::Value) -> io::Result<()> {
        // Accept BOTH canonical and v3-legacy names on input. Normalize
        // to canonical first so the per-field reads below find them under
        // their authoritative names regardless of which shape the caller
        // authored their JSON in.
        let mut v_owned = v.clone();
        normalize_input_aliases(&mut v_owned, FIELD_ALIASES_V3);
        let v = &v_owned;

        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "BaseDataV0: expected object",
        ))?;

        let bd = BaseDataV0 {
            weapon_key:   json_u64(obj, "weapon_key")? as u32,
            attack_dir:   json_u64(obj, "attack_dir")? as u8,
            _pad0005: [0; 3],

            attack_pos_offset:   json_float3(obj, "attack_pos_offset")?,
            _unk_float3_0014:    json_float3(obj, "_unk_float3_0014")?,
            attack_degree:       json_f32(obj, "attack_degree")?,
            attack_yaw:          json_f32(obj, "attack_yaw")?,
            _unk_f32_0028:       json_f32(obj, "_unk_f32_0028")?,

            physic_impulse_power:     json_f32(obj, "physic_impulse_power")?,
            physics_impulse_mass:     json_f32(obj, "physics_impulse_mass")?,
            attack_hit_check_type:    json_u64(obj, "attack_hit_check_type")? as u16,
            hit_check_normal_str_idx:                 json_u64(obj, "hit_check_normal_str_idx")? as u16,
            repeat_degree_weight:     json_f32(obj, "repeat_degree_weight")?,
            physics_impulse_velocity: json_f32(obj, "physics_impulse_velocity")?,

            ignore_safe_zone: json_get_field(obj, "ignore_safe_zone")?.as_bool().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "BaseDataV0.ignore_safe_zone: expected bool")
            })?,
            _pad0041: [0; 3],

            attack_group_index: json_u64(obj, "attack_group_index")? as u8,
            repeat_count:       json_u64(obj, "repeat_count")? as u8,
            _pad0046: [0; 2],

            hit_effect_info_type: json_u64(obj, "hit_effect_info_type")? as u32,

            single_hit_pos_offset: json_float3(obj, "single_hit_pos_offset")?,

            _ds1_f0: json_f32(obj, "_ds1_f0")?,
            _ds1_f1: json_f32(obj, "_ds1_f1")?,
            _ds1_f2: json_f32(obj, "_ds1_f2")?,
            _ds1_f3: json_f32(obj, "_ds1_f3")?,
            _ds1_f4: json_f32(obj, "_ds1_f4")?,
            _pad_ds1: [0; 4],
            normal_string_index: json_u64(obj, "normal_string_index")? as u16,
            _unk0072: json_get_field(obj, "_unk0072")?.as_bool().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "BaseDataV0._unk0072: expected bool"))?,
            _unk0073: json_u64(obj, "_unk0073")? as u8,
            _pad0074: [0; 4],
            _ds2_f0: json_f32(obj, "_ds2_f0")?,
            _ds2_f1: json_f32(obj, "_ds2_f1")?,
            _ds2_f2: json_f32(obj, "_ds2_f2")?,
            _ds2_f3: json_f32(obj, "_ds2_f3")?,
            _ds2_f4: json_f32(obj, "_ds2_f4")?,
            _pad_ds2: [0; 4],
            hit_degree: json_f32(obj, "hit_degree")?,
            _pad0094: [0; 8],
            hit_rotation_type: json_u64(obj, "hit_rotation_type")? as u8,
            _pad009d: [0; 3],
            _unk_f32_00a0: json_f32(obj, "_unk_f32_00a0")?,
            _pad00a4: [0; 4],
            equip_slot_name_key: json_u64(obj, "equip_slot_name_key")? as u8,
            _pad00a9: [0; 3],
            _unk00ac: json_u64(obj, "_unk00ac")? as u8,
            _unk00ad: json_get_field(obj, "_unk00ad")?.as_bool().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "BaseDataV0._unk00ad: expected bool"))?,
            _pad00ae: [0; 2],
            exclude_target_type_flag: json_u64(obj, "exclude_target_type_flag")? as u32,

            single_hit_position_socket: json_u64(obj, "single_hit_position_socket")? as u16,
            _pad00b6: [0; 2],
            _unk00b8: json_u64(obj, "_unk00b8")? as u8,
            _unk00b9: json_u64(obj, "_unk00b9")? as u8,
            _unk00ba: json_u64(obj, "_unk00ba")? as u8,
            _unk00bb: json_u64(obj, "_unk00bb")? as u8,
            _pad00bc: 0,
            _unk00bd: json_get_field(obj, "_unk00bd")?.as_bool().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "BaseDataV0._unk00bd: expected bool"))?,
            _pad00be: 0,
            _unk00bf: json_get_field(obj, "_unk00bf")?.as_bool().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "BaseDataV0._unk00bf: expected bool"))?,
            _unk00c0: json_get_field(obj, "_unk00c0")?.as_bool().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "BaseDataV0._unk00c0: expected bool"))?,
            _pad00c1: [0; 3],
            _pad00c4: [0; 4],
            _unk00c9: json_u64(obj, "_unk00c9")? as u8,
            _unk00cb: json_u64(obj, "_unk00cb")? as u8,
            _unk00cc: json_u64(obj, "_unk00cc")? as u8,
            _unk00ce: json_u64(obj, "_unk00ce")? as u8,
            _unk00cf: json_u64(obj, "_unk00cf")? as u8,
            _unk00d0: json_get_field(obj, "_unk00d0")?.as_bool().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "BaseDataV0._unk00d0: expected bool"))?,
            _unk00d1: json_get_field(obj, "_unk00d1")?.as_bool().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "BaseDataV0._unk00d1: expected bool"))?,
            _unk00d2: json_get_field(obj, "_unk00d2")?.as_bool().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "BaseDataV0._unk00d2: expected bool"))?,
            _unk00d3: json_u64(obj, "_unk00d3")? != 0,
            _unk_f32_00d4: json_f32(obj, "_unk_f32_00d4")?,
            _pad00d8:      [0; 4],
            _unk_f32_00dc: json_f32(obj, "_unk_f32_00dc")?,
            _unk_f32_00e0: json_f32(obj, "_unk_f32_00e0")?,
            _unk_f32_00e4: json_f32(obj, "_unk_f32_00e4")?,
            _unk00e8: json_u64(obj, "_unk00e8")? as u8,
            _unk00e9: json_get_field(obj, "_unk00e9")?.as_bool().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "BaseDataV0._unk00e9: expected bool"))?,
            _pad00ea: [0; 2],
            _pad00ec:      [0; 4],
            hit_normal_string_index:      json_u64(obj, "hit_normal_string_index")? as u16,
            _pad00f2:      [0; 2],
            _unk00f4:      json_u64(obj, "_unk00f4")? as u8,
            _pad00f5:      [0; 3],
            hit_data_str_idx: json_u64(obj, "hit_data_str_idx")? as u16,
            _unk00fa: json_get_field(obj, "_unk00fa")?.as_bool().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "BaseDataV0._unk00fa: expected bool"))?,
            _pad00fb: 0,
            _unk00fc:      json_u64(obj, "_unk00fc")? as u8,
            _pad00fd:      [0; 3],
            hit_data_str_idx_b:      json_u64(obj, "hit_data_str_idx_b")? as u16,
            _pad0102:      [0; 2],
            _unk0104:      json_u64(obj, "_unk0104")? as u8,
            _pad0105:      [0; 3],
        };
        w.extend_from_slice(&bd.to_bytes());
        Ok(())
    }
}

// ── V1 — AttackCatch (528 bytes = V0 + 264-byte AttackCatchDesc) ─────────────

/// Decoded layout of the 264-byte `AttackCatchDesc` extra block in V1.
///
/// ## Field layout (offsets relative to catch_desc start)
///
/// ```text
/// 0x0000  _cd_unk0000          u32     [4]   near-const hash (5 values, mode=0xb4af8d6f@97%)
/// 0x0004  _cd_unk0004          u32     [4]   4 values; 3 hashes + 0x00000000 (~50/22/22/5%)
/// 0x0008  _cd_unk0008          u32     [4]   near-const hash (21 values, mode=0x89d0606e@92%)
/// 0x000c  catch_normal_str_idx u16     [2]   NormalStringIndex; 0xffff=none (98%); 0x0011 = 2%
/// 0x000e  _pad000e             [u8;2]  [2]   always zero
/// 0x0010  _cd_unk0010          u32     [4]   near-const hash (4 values, mode=0x88d2badd@81%)
/// 0x0014  _pad0014             [u8;16] [16]  CONST: ff×4, 12 00 00 00, 8c c9 28 ba, ff×4
/// 0x0024  _cd_unk0024          f32     [4]   mode=-1.0 (98%), 4 distinct values
/// 0x0028  _cd_unk0028          f32     [4]   mode=-0.0 / 0x80000000 (99.9%)
/// 0x002c  catch_yaw_hi_rad     f32     [4]   positive yaw limit; mode=0.5 rad (28.7°)  @93%
/// 0x0030  catch_yaw_lo_rad     f32     [4]   negative yaw limit; mode=-0.5 rad (-28.7°) @80%
/// 0x0034  _pad0034             [u8;12] [12]  CONST: FLT_MAX × 3
/// 0x0040  catch_dist_a         f32     [4]   catch distance param; always == catch_dist_b; mode=0.0@76%
/// 0x0044  catch_dist_b         f32     [4]   catch distance param; always == catch_dist_a; mode=0.0@76%
/// 0x0048  _cd_unk0048          f32     [4]   mode=0.0 (99.5%), 4 values
/// 0x004c  _cd_unk004c          f32     [4]   mode=0.0 (99.5%), 4 values
/// 0x0050  catch_elevation_rad_a f32    [4]   pitch half-angle; always == catch_elevation_rad_b; mode=0.4363 rad (25°)
/// 0x0054  catch_elevation_rad_b f32    [4]   pitch half-angle; always == catch_elevation_rad_a
/// 0x0058  _cd_tail             [u8;176][176] mostly CONST/ZERO sentinel region
/// ```
#[derive(Debug, Clone)]
pub struct AttackCatchDesc {
    pub _cd_unk0000: u32,
    pub _cd_unk0004: u32,
    pub _cd_unk0008: u32,
    pub catch_normal_str_idx: u16,
    pub _cd_unk0010: u32,
    pub _cd_unk0024: f32,
    pub _cd_unk0028: f32,
    pub catch_yaw_hi_rad: f32,
    pub catch_yaw_lo_rad: f32,
    pub catch_dist_a: f32,
    pub catch_dist_b: f32,
    pub _cd_unk0048: f32,
    pub _cd_unk004c: f32,
    pub catch_elevation_rad_a: f32,
    pub catch_elevation_rad_b: f32,
    pub _cd_tail: [u8; 176],
}

impl AttackCatchDesc {
    pub const SIZE: usize = 264;

    pub fn parse(d: &[u8]) -> Self {
        assert_eq!(d.len(), Self::SIZE);
        Self {
            _cd_unk0000: read_u32(d, 0x0000),
            _cd_unk0004: read_u32(d, 0x0004),
            _cd_unk0008: read_u32(d, 0x0008),
            catch_normal_str_idx: read_u16(d, 0x000c),
            _cd_unk0010: read_u32(d, 0x0010),
            _cd_unk0024: read_f32(d, 0x0024),
            _cd_unk0028: read_f32(d, 0x0028),
            catch_yaw_hi_rad: read_f32(d, 0x002c),
            catch_yaw_lo_rad: read_f32(d, 0x0030),
            catch_dist_a: read_f32(d, 0x0040),
            catch_dist_b: read_f32(d, 0x0044),
            _cd_unk0048: read_f32(d, 0x0048),
            _cd_unk004c: read_f32(d, 0x004c),
            catch_elevation_rad_a: read_f32(d, 0x0050),
            catch_elevation_rad_b: read_f32(d, 0x0054),
            _cd_tail: d[0x0058..0x0108].try_into().unwrap(),
        }
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut out = [0u8; Self::SIZE];
        out[0x0000..0x0004].copy_from_slice(&self._cd_unk0000.to_le_bytes());
        out[0x0004..0x0008].copy_from_slice(&self._cd_unk0004.to_le_bytes());
        out[0x0008..0x000c].copy_from_slice(&self._cd_unk0008.to_le_bytes());
        out[0x000c..0x000e].copy_from_slice(&self.catch_normal_str_idx.to_le_bytes());
        // 0x000e–0x000f: _pad000e always zero
        out[0x0010..0x0014].copy_from_slice(&self._cd_unk0010.to_le_bytes());
        // 0x0014–0x0023: CONST bytes [ff×4, 12 00 00 00, 8c c9 28 ba, ff×4]
        out[0x0014..0x0018].copy_from_slice(&[0xff, 0xff, 0xff, 0xff]);
        out[0x0018..0x001c].copy_from_slice(&[0x12, 0x00, 0x00, 0x00]);
        out[0x001c..0x0020].copy_from_slice(&[0x8c, 0xc9, 0x28, 0xba]);
        out[0x0020..0x0024].copy_from_slice(&[0xff, 0xff, 0xff, 0xff]);
        out[0x0024..0x0028].copy_from_slice(&self._cd_unk0024.to_le_bytes());
        out[0x0028..0x002c].copy_from_slice(&self._cd_unk0028.to_le_bytes());
        out[0x002c..0x0030].copy_from_slice(&self.catch_yaw_hi_rad.to_le_bytes());
        out[0x0030..0x0034].copy_from_slice(&self.catch_yaw_lo_rad.to_le_bytes());
        // 0x0034–0x003f: CONST FLT_MAX × 3
        let flt_max = 0x7f7fffffu32.to_le_bytes();
        out[0x0034..0x0038].copy_from_slice(&flt_max);
        out[0x0038..0x003c].copy_from_slice(&flt_max);
        out[0x003c..0x0040].copy_from_slice(&flt_max);
        out[0x0040..0x0044].copy_from_slice(&self.catch_dist_a.to_le_bytes());
        out[0x0044..0x0048].copy_from_slice(&self.catch_dist_b.to_le_bytes());
        out[0x0048..0x004c].copy_from_slice(&self._cd_unk0048.to_le_bytes());
        out[0x004c..0x0050].copy_from_slice(&self._cd_unk004c.to_le_bytes());
        out[0x0050..0x0054].copy_from_slice(&self.catch_elevation_rad_a.to_le_bytes());
        out[0x0054..0x0058].copy_from_slice(&self.catch_elevation_rad_b.to_le_bytes());
        out[0x0058..0x0108].copy_from_slice(&self._cd_tail);
        out
    }
}

impl ToJsonValue for AttackCatchDesc {
    fn to_json_value(&self) -> serde_json::Value {
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
        let mut m = serde_json::Map::new();
        m.insert("_cd_unk0000".into(), (self._cd_unk0000 as u64).into());
        m.insert("_cd_unk0004".into(), (self._cd_unk0004 as u64).into());
        m.insert("_cd_unk0008".into(), (self._cd_unk0008 as u64).into());
        m.insert("catch_normal_str_idx".into(), (self.catch_normal_str_idx as u64).into());
        m.insert("_cd_unk0010".into(), (self._cd_unk0010 as u64).into());
        m.insert("_cd_unk0024".into(), f32_to_json(self._cd_unk0024));
        m.insert("_cd_unk0028".into(), f32_to_json(self._cd_unk0028));
        m.insert("catch_yaw_hi_rad".into(), f32_to_json(self.catch_yaw_hi_rad));
        m.insert("catch_yaw_lo_rad".into(), f32_to_json(self.catch_yaw_lo_rad));
        m.insert("catch_dist_a".into(), f32_to_json(self.catch_dist_a));
        m.insert("catch_dist_b".into(), f32_to_json(self.catch_dist_b));
        m.insert("_cd_unk0048".into(), f32_to_json(self._cd_unk0048));
        m.insert("_cd_unk004c".into(), f32_to_json(self._cd_unk004c));
        m.insert("catch_elevation_rad_a".into(), f32_to_json(self.catch_elevation_rad_a));
        m.insert("catch_elevation_rad_b".into(), f32_to_json(self.catch_elevation_rad_b));
        m.insert("_cd_tail_b64".into(), B64.encode(self._cd_tail).into());
        serde_json::Value::Object(m)
    }
}

impl WriteJsonValue for AttackCatchDesc {
    fn write_from_json(w: &mut Vec<u8>, v: &serde_json::Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "AttackCatchDesc: expected object",
        ))?;
        let cd = AttackCatchDesc {
            _cd_unk0000: json_u64(obj, "_cd_unk0000")? as u32,
            _cd_unk0004: json_u64(obj, "_cd_unk0004")? as u32,
            _cd_unk0008: json_u64(obj, "_cd_unk0008")? as u32,
            catch_normal_str_idx: json_u64(obj, "catch_normal_str_idx")? as u16,
            _cd_unk0010: json_u64(obj, "_cd_unk0010")? as u32,
            _cd_unk0024: json_f32(obj, "_cd_unk0024")?,
            _cd_unk0028: json_f32(obj, "_cd_unk0028")?,
            catch_yaw_hi_rad: json_f32(obj, "catch_yaw_hi_rad")?,
            catch_yaw_lo_rad: json_f32(obj, "catch_yaw_lo_rad")?,
            catch_dist_a: json_f32(obj, "catch_dist_a")?,
            catch_dist_b: json_f32(obj, "catch_dist_b")?,
            _cd_unk0048: json_f32(obj, "_cd_unk0048")?,
            _cd_unk004c: json_f32(obj, "_cd_unk004c")?,
            catch_elevation_rad_a: json_f32(obj, "catch_elevation_rad_a")?,
            catch_elevation_rad_b: json_f32(obj, "catch_elevation_rad_b")?,
            _cd_tail: json_b64(obj, "_cd_tail_b64", 176)?.try_into().unwrap(),
        };
        w.extend_from_slice(&cd.to_bytes());
        Ok(())
    }
}

/// V1 BaseData: the V0 shared region plus 264 bytes of `AttackCatchDesc`.
#[derive(Debug, Clone)]
pub struct BaseDataV1 {
    pub base: BaseDataV0,
    pub catch_desc: AttackCatchDesc,
}

impl BaseDataV1 {
    pub const SIZE: usize = 528;

    pub fn parse(data: &[u8]) -> io::Result<Self> {
        if data.len() != Self::SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("BaseDataV1: expected {} bytes, got {}", Self::SIZE, data.len()),
            ));
        }
        Ok(Self {
            base: BaseDataV0::parse(&data[..264])?,
            catch_desc: AttackCatchDesc::parse(&data[264..528]),
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut out = [0u8; Self::SIZE];
        out[..264].copy_from_slice(&self.base.to_bytes());
        out[264..].copy_from_slice(&self.catch_desc.to_bytes());
        out
    }
}

impl ToJsonValue for BaseDataV1 {
    fn to_json_value(&self) -> serde_json::Value {
        let mut m = self.base.to_json_value().as_object().cloned().unwrap_or_default();
        m.insert("catch_desc".into(), self.catch_desc.to_json_value());
        serde_json::Value::Object(m)
    }
}

impl WriteJsonValue for BaseDataV1 {
    fn write_from_json(w: &mut Vec<u8>, v: &serde_json::Value) -> io::Result<()> {
        BaseDataV0::write_from_json(w, v)?;
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "BaseDataV1: expected object",
        ))?;
        let cd_val = obj.get("catch_desc").ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "BaseDataV1: missing catch_desc",
        ))?;
        AttackCatchDesc::write_from_json(w, cd_val)
    }
}

// ── V2 — AttackThrow (296 bytes = V0 + 32-byte ThrowDesc) ────────────────────

/// V2 BaseData: shared V0 region plus 32 bytes of `AttackThrowDataDesc`.
///
/// ## Extra byte layout (0x0108–0x0127, 32 bytes)
///
/// ```text
/// 0x0108  projectile_key              u32     [4]   `projectileKey`; packed u32 (two u16 parts);
///                                                   30 distinct values; (0,0)=24% (1,1)=23%
/// 0x010c  action_hash_code            u32     [4]   `actionHashCode` — high-cardinality u32 key
/// 0x0110  frame_time                  f32     [4]   `frameTime`; V2 default ≈ 0.2
/// 0x0114  ai_event_key                u32     [4]   `aiEventKey` enum; 2 values
/// 0x0118  _pad0118                    [u8;4]  [4]   always zero
/// 0x011c  _unk011c_kind               u8      [1]   small enum: 0@87%, 1@12%, 2@1%
/// 0x011d  _unk011c_flag1              bool    [1]   bool; 99% false
/// 0x011e  _unk011c_b2                 u8      [1]   99% CONST 1; 1% zero
/// 0x011f  _unk011c_flag3              bool    [1]   bool; 77% true
/// 0x0120  _unk0120_flag               bool    [1]   bool; 89% false
/// 0x0121  _pad0121                    [u8;3]  [3]   CONST [0x00, 0x00, 0x01]
/// 0x0124  _unk0124                    u32     [4]   4 distinct bit patterns: 0x00000000@77%,
///                                                   0xda8e5094@16%, 0xb75d2454@7%, 0xc5c2812c@1%;
///                                                   stored as u32 (patterns are not valid game floats)
/// ```
#[derive(Debug, Clone)]
pub struct BaseDataV2 {
    pub base: BaseDataV0,
    /// `projectileKey` — packed u32 (two independent u16 parts); 30 distinct combos.
    pub projectile_key: u32,
    /// `actionHashCode` — u32 hash key for the throw action name.
    pub action_hash_code: u32,
    /// `frameTime` — throw frame timing; V2 default ≈ 0.2.
    pub frame_time: f32,
    /// `aiEventKey` — enum (2 values in vanilla).
    pub ai_event_key: u32,
    pub _pad0118: [u8; 4],
    pub _unk011c_kind: u8,
    pub _unk011c_flag1: bool,
    pub _unk011c_b2: u8,
    pub _unk011c_flag3: bool,
    pub _unk0120_flag: bool,
    pub _unk0124: u32,
}

impl BaseDataV2 {
    pub const SIZE: usize = 296;

    pub fn parse(data: &[u8]) -> io::Result<Self> {
        if data.len() != Self::SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("BaseDataV2: expected {} bytes, got {}", Self::SIZE, data.len()),
            ));
        }
        Ok(Self {
            base:             BaseDataV0::parse(&data[..264])?,
            projectile_key:   read_u32(data, 0x0108),
            action_hash_code: read_u32(data, 0x010c),
            frame_time:       read_f32(data, 0x0110),
            ai_event_key:     read_u32(data, 0x0114),
            _pad0118:         data[0x0118..0x011c].try_into().unwrap(),
            _unk011c_kind:    data[0x011c],
            _unk011c_flag1:   data[0x011d] != 0,
            _unk011c_b2:      data[0x011e],
            _unk011c_flag3:   data[0x011f] != 0,
            _unk0120_flag:    data[0x0120] != 0,
            _unk0124:         read_u32(data, 0x0124),
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut out = [0u8; Self::SIZE];
        out[..264].copy_from_slice(&self.base.to_bytes());
        out[0x0108..0x010c].copy_from_slice(&self.projectile_key.to_le_bytes());
        out[0x010c..0x0110].copy_from_slice(&self.action_hash_code.to_le_bytes());
        out[0x0110..0x0114].copy_from_slice(&self.frame_time.to_le_bytes());
        out[0x0114..0x0118].copy_from_slice(&self.ai_event_key.to_le_bytes());
        out[0x0118..0x011c].copy_from_slice(&self._pad0118);
        out[0x011c] = self._unk011c_kind;
        out[0x011d] = self._unk011c_flag1 as u8;
        out[0x011e] = self._unk011c_b2;
        out[0x011f] = self._unk011c_flag3 as u8;
        out[0x0120] = self._unk0120_flag as u8;
        out[0x0121..0x0124].copy_from_slice(&[0x00, 0x00, 0x01]);
        out[0x0124..0x0128].copy_from_slice(&self._unk0124.to_le_bytes());
        out
    }
}

impl ToJsonValue for BaseDataV2 {
    fn to_json_value(&self) -> serde_json::Value {
        let mut m = self.base.to_json_value().as_object().cloned().unwrap_or_default();
        m.insert("projectile_key".into(),   (self.projectile_key as u64).into());
        m.insert("action_hash_code".into(), self.action_hash_code.into());
        m.insert("frame_time".into(),       f32_to_json(self.frame_time));
        m.insert("ai_event_key".into(),     self.ai_event_key.into());
        m.insert("_unk011c_kind".into(),   (self._unk011c_kind as u64).into());
        m.insert("_unk011c_flag1".into(),  self._unk011c_flag1.into());
        m.insert("_unk011c_b2".into(),     (self._unk011c_b2 as u64).into());
        m.insert("_unk011c_flag3".into(),  self._unk011c_flag3.into());
        m.insert("_unk0120_flag".into(),   self._unk0120_flag.into());
        m.insert("_unk0124".into(),         (self._unk0124 as u64).into());
        serde_json::Value::Object(m)
    }
}

impl WriteJsonValue for BaseDataV2 {
    fn write_from_json(w: &mut Vec<u8>, v: &serde_json::Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "BaseDataV2: expected object",
        ))?;
        let bd = BaseDataV2 {
            base:             {
                let mut tmp = Vec::new();
                BaseDataV0::write_from_json(&mut tmp, v)?;
                BaseDataV0::parse(&tmp).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?
            },
            projectile_key:   json_u64(obj, "projectile_key")? as u32,
            action_hash_code: json_u64(obj, "action_hash_code")? as u32,
            frame_time:       json_f32(obj, "frame_time")?,
            ai_event_key:     json_u64(obj, "ai_event_key")? as u32,
            _pad0118:         [0; 4],
            _unk011c_kind:    json_u64(obj, "_unk011c_kind")? as u8,
            _unk011c_flag1:   json_get_field(obj, "_unk011c_flag1")?.as_bool().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "BaseDataV2._unk011c_flag1: expected bool"))?,
            _unk011c_b2:      json_u64(obj, "_unk011c_b2")? as u8,
            _unk011c_flag3:   json_get_field(obj, "_unk011c_flag3")?.as_bool().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "BaseDataV2._unk011c_flag3: expected bool"))?,
            _unk0120_flag:    json_get_field(obj, "_unk0120_flag")?.as_bool().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "BaseDataV2._unk0120_flag: expected bool"))?,
            _unk0124:         json_u64(obj, "_unk0124")? as u32,
        };
        w.extend_from_slice(&bd.to_bytes());
        Ok(())
    }
}

// ── V3 — ReleaseCatch (288 bytes = V0 + 24-byte ReleaseCatchDesc) ─────────────

/// V3 BaseData: shared V0 region plus 24 bytes of `AttackReleaseCatchDataDesc`.
///
/// ## Extra byte layout (0x0108–0x011f, 24 bytes)
///
/// ```text
/// 0x0108  release_angle_rad           f32     [4]   release angle in radians; 0.0@98%;
///                                                   non-zero values are π, π/2, -π/2 (exact)
/// 0x010c  frame_time                  f32     [4]   `frameTime`; 0.2@71%; mirrors V2 frame_time
/// 0x0110  _unk0110                    u32     [4]   release-catch type hash; 6 distinct values;
///                                                   mode=0x1fc0e737 @86%
/// 0x0114  _unk0114                    u32     [4]   key/hash; 355 distinct values; 40% zero
/// 0x0118  _unk0118                    [u8;4]  [4]   byte[0]=small enum (mode=3); byte[1,2]=bools
/// 0x011c  _pad011c                    [u8;4]  [4]   always zero
/// ```
#[derive(Debug, Clone)]
pub struct BaseDataV3 {
    pub base: BaseDataV0,
    /// Release angle in radians; 0.0=forward, π=backward, ±π/2=sides.
    pub release_angle_rad: f32,
    /// `frameTime` — release-catch frame timing; default ≈ 0.2.
    pub frame_time: f32,
    pub _unk0110: u32,
    pub _unk0114: u32,
    pub _unk0118: [u8; 4],
    pub _pad011c: [u8; 4],
}

impl BaseDataV3 {
    pub const SIZE: usize = 288;

    pub fn parse(data: &[u8]) -> io::Result<Self> {
        if data.len() != Self::SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("BaseDataV3: expected {} bytes, got {}", Self::SIZE, data.len()),
            ));
        }
        Ok(Self {
            base:            BaseDataV0::parse(&data[..264])?,
            release_angle_rad: read_f32(data, 0x0108),
            frame_time:        read_f32(data, 0x010c),
            _unk0110:        read_u32(data, 0x0110),
            _unk0114:        read_u32(data, 0x0114),
            _unk0118:        data[0x0118..0x011c].try_into().unwrap(),
            _pad011c:        data[0x011c..0x0120].try_into().unwrap(),
        })
    }

    pub fn to_bytes(&self) -> [u8; Self::SIZE] {
        let mut out = [0u8; Self::SIZE];
        out[..264].copy_from_slice(&self.base.to_bytes());
        out[0x0108..0x010c].copy_from_slice(&self.release_angle_rad.to_le_bytes());
        out[0x010c..0x0110].copy_from_slice(&self.frame_time.to_le_bytes());
        out[0x0110..0x0114].copy_from_slice(&self._unk0110.to_le_bytes());
        out[0x0114..0x0118].copy_from_slice(&self._unk0114.to_le_bytes());
        out[0x0118..0x011c].copy_from_slice(&self._unk0118);
        out[0x011c..0x0120].copy_from_slice(&self._pad011c);
        out
    }
}

impl ToJsonValue for BaseDataV3 {
    fn to_json_value(&self) -> serde_json::Value {
        use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
        let mut m = self.base.to_json_value().as_object().cloned().unwrap_or_default();
        m.insert("release_angle_rad".into(), f32_to_json(self.release_angle_rad));
        m.insert("frame_time".into(),        f32_to_json(self.frame_time));
        m.insert("_unk0110".into(),      self._unk0110.into());
        m.insert("_unk0114".into(),      self._unk0114.into());
        m.insert("_unk0118_b64".into(),  B64.encode(self._unk0118).into());
        serde_json::Value::Object(m)
    }
}

impl WriteJsonValue for BaseDataV3 {
    fn write_from_json(w: &mut Vec<u8>, v: &serde_json::Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "BaseDataV3: expected object",
        ))?;
        let bd = BaseDataV3 {
            base: {
                let mut tmp = Vec::new();
                BaseDataV0::write_from_json(&mut tmp, v)?;
                BaseDataV0::parse(&tmp).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?
            },
            release_angle_rad: json_f32(obj, "release_angle_rad")?,
            frame_time:        json_f32(obj, "frame_time")?,
            _unk0110:      json_u64(obj, "_unk0110")? as u32,
            _unk0114:      json_u64(obj, "_unk0114")? as u32,
            _unk0118:      json_b64(obj, "_unk0118_b64", 4)?.try_into().unwrap(),
            _pad011c:      [0; 4],
        };
        w.extend_from_slice(&bd.to_bytes());
        Ok(())
    }
}

// ── AttackInfoBaseData enum ───────────────────────────────────────────────────

/// Typed wrapper around the raw `base_data` blob in an `AttackInfo`.
///
/// V0 and V1 are fully field-decoded.  V2 (Throw) and V3 (ReleaseCatch) expose
/// their extra bytes as named fields.  V4 retains raw bytes.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum AttackInfoBaseData {
    V0(BaseDataV0),
    V1(BaseDataV1),
    V2(BaseDataV2),
    V3(BaseDataV3),
    /// V4 (same base size as V0) — raw bytes retained.
    Raw { version: u8, data: Vec<u8> },
}

impl AttackInfoBaseData {
    /// Decode the `base_data` blob from an `AttackInfo`.
    pub fn decode(version: u8, data: &[u8]) -> io::Result<Self> {
        match version {
            0 => Ok(Self::V0(BaseDataV0::parse(data)?)),
            1 => Ok(Self::V1(BaseDataV1::parse(data)?)),
            2 => Ok(Self::V2(BaseDataV2::parse(data)?)),
            3 => Ok(Self::V3(BaseDataV3::parse(data)?)),
            v => Ok(Self::Raw { version: v, data: data.to_vec() }),
        }
    }

    /// Encode back to raw bytes, ready to replace `AttackInfo.base_data`.
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::V0(v) => v.to_bytes().to_vec(),
            Self::V1(v) => v.to_bytes().to_vec(),
            Self::V2(v) => v.to_bytes().to_vec(),
            Self::V3(v) => v.to_bytes().to_vec(),
            Self::Raw { data, .. } => data.clone(),
        }
    }

    /// Version number matching the outer `AttackInfo.version` field.
    pub fn version(&self) -> u8 {
        match self {
            Self::V0(_) => 0,
            Self::V1(_) => 1,
            Self::V2(_) => 2,
            Self::V3(_) => 3,
            Self::Raw { version, .. } => *version,
        }
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn zero_v0() -> BaseDataV0 {
        BaseDataV0 {
            weapon_key: 0,
            attack_dir: 0,
            _pad0005: [0; 3],
            attack_pos_offset: [0.0; 3],
            _unk_float3_0014: [0.0; 3],
            attack_degree: 0.0,
            attack_yaw: 0.0,
            _unk_f32_0028: 0.0,
            physic_impulse_power: 0.0,
            physics_impulse_mass: 0.0,
            attack_hit_check_type: 0,
            hit_check_normal_str_idx: 0,
            repeat_degree_weight: 0.0,
            physics_impulse_velocity: 0.0,
            ignore_safe_zone: false,
            _pad0041: [0; 3],
            attack_group_index: 0,
            repeat_count: 0,
            _pad0046: [0; 2],
            hit_effect_info_type: 0,
            single_hit_pos_offset: [0.0; 3],
            _ds1_f0: 0.0,
            _ds1_f1: 0.0,
            _ds1_f2: 0.0,
            _ds1_f3: 0.0,
            _ds1_f4: 0.0,
            _pad_ds1: [0; 4],
            normal_string_index: 0,
            _unk0072: false,
            _unk0073: 0,
            _pad0074: [0; 4],
            _ds2_f0: 0.0,
            _ds2_f1: 0.0,
            _ds2_f2: 0.0,
            _ds2_f3: 0.0,
            _ds2_f4: 0.0,
            _pad_ds2: [0; 4],
            hit_degree: 0.0,
            _pad0094: [0; 8],
            hit_rotation_type: 0,
            _pad009d: [0; 3],
            _unk_f32_00a0: 0.0,
            _pad00a4: [0; 4],
            equip_slot_name_key: 0,
            _pad00a9: [0; 3],
            _unk00ac: 0,
            _unk00ad: false,
            _pad00ae: [0; 2],
            exclude_target_type_flag: 0,
            single_hit_position_socket: 0,
            _pad00b6: [0; 2],
            _unk00b8: 0,
            _unk00b9: 0,
            _unk00ba: 0,
            _unk00bb: 0,
            _pad00bc: 0,
            _unk00bd: false,
            _pad00be: 0,
            _unk00bf: false,
            _unk00c0: false,
            _pad00c1: [0; 3],
            _pad00c4: [0; 4],
            _unk00c9: 0,
            _unk00cb: 0,
            _unk00cc: 0,
            _unk00ce: 0,
            _unk00cf: 0,
            _unk00d0: false,
            _unk00d1: false,
            _unk00d2: false,
            _unk00d3: false,
            _unk_f32_00d4: 0.0,
            _pad00d8: [0; 4],
            _unk_f32_00dc: 0.0,
            _unk_f32_00e0: 0.0,
            _unk_f32_00e4: 0.0,
            _unk00e8: 0,
            _unk00e9: false,
            _pad00ea: [0; 2],
            _pad00ec: [0; 4],
            hit_normal_string_index: 0,
            _pad00f2: [0; 2],
            _unk00f4: 0,
            _pad00f5: [0; 3],
            hit_data_str_idx: 0,
            _unk00fa: false,
            _pad00fb: 0,
            _unk00fc: 0,
            _pad00fd: [0; 3],
            hit_data_str_idx_b: 0,
            _pad0102: [0; 2],
            _unk0104: 0,
            _pad0105: [0; 3],
        }
    }

    #[test]
    fn v0_size_is_264() {
        assert_eq!(BaseDataV0::SIZE, 264);
        assert_eq!(zero_v0().to_bytes().len(), 264);
    }

    #[test]
    fn v1_size_is_528() {
        assert_eq!(BaseDataV1::SIZE, 528);
    }

    #[test]
    fn v0_roundtrip_zeros() {
        let bd = zero_v0();
        let bytes = bd.to_bytes();
        let bd2 = BaseDataV0::parse(&bytes).unwrap();
        assert_eq!(bd2.to_bytes(), bytes);
    }

    #[test]
    fn v0_roundtrip_realistic_defaults() {
        let mut bd = zero_v0();
        bd.weapon_key = 0xdeadbeef;
        bd.attack_dir = 1;
        bd.attack_degree = std::f32::consts::TAU;
        bd.physic_impulse_power = 1.0;
        bd.physics_impulse_mass = 1.0;
        bd.repeat_degree_weight = -1.0;
        bd.attack_hit_check_type = 4;
        bd.hit_check_normal_str_idx = 0xffff;
        bd.attack_group_index = 1;
        bd.repeat_count = 2;
        bd.hit_effect_info_type = 0xf177b780;
        bd.single_hit_pos_offset = [3.5, 0.0, 0.0];
        bd.ignore_safe_zone = true;
        bd.single_hit_position_socket = 0xffff;

        let bytes = bd.to_bytes();
        let bd2 = BaseDataV0::parse(&bytes).unwrap();
        assert_eq!(bd2.weapon_key, 0xdeadbeef);
        assert_eq!(bd2.attack_dir, 1);
        assert!((bd2.attack_degree - std::f32::consts::TAU).abs() < 1e-6);
        assert_eq!(bd2.physic_impulse_power, 1.0);
        assert_eq!(bd2.physics_impulse_mass, 1.0);
        assert_eq!(bd2.repeat_degree_weight, -1.0);
        assert_eq!(bd2.attack_hit_check_type, 4);
        assert_eq!(bd2.hit_check_normal_str_idx, 0xffff);
        assert_eq!(bd2.attack_group_index, 1);
        assert_eq!(bd2.repeat_count, 2);
        assert_eq!(bd2.hit_effect_info_type, 0xf177b780);
        assert_eq!(bd2.single_hit_pos_offset, [3.5, 0.0, 0.0]);
        assert!(bd2.ignore_safe_zone);
        assert_eq!(bd2.single_hit_position_socket, 0xffff);
        assert_eq!(bd2.to_bytes(), bytes);
    }

    #[test]
    fn v0_field_offsets() {
        let mut data = vec![0u8; 264];
        // weapon_key at 0x0000
        data[0x00..0x04].copy_from_slice(&0xdeadbeefu32.to_le_bytes());
        // attack_dir at 0x0004
        data[0x04] = 1;
        // attack_degree at 0x0020
        data[0x20..0x24].copy_from_slice(&std::f32::consts::TAU.to_le_bytes());
        // physic_impulse_power at 0x002c
        data[0x2c..0x30].copy_from_slice(&1.5f32.to_le_bytes());
        // physics_impulse_mass at 0x0030
        data[0x30..0x34].copy_from_slice(&2.0f32.to_le_bytes());
        // attack_hit_check_type at 0x0034
        data[0x34..0x36].copy_from_slice(&4u16.to_le_bytes());
        // hit_check_normal_str_idx at 0x0036
        data[0x36..0x38].copy_from_slice(&0xffffu16.to_le_bytes());
        // repeat_degree_weight at 0x0038
        data[0x38..0x3c].copy_from_slice(&(-1.0f32).to_le_bytes());
        // physics_impulse_velocity at 0x003c
        data[0x3c..0x40].copy_from_slice(&3.0f32.to_le_bytes());
        // ignore_safe_zone at 0x0040
        data[0x40] = 1;
        // attack_group_index at 0x0044
        data[0x44] = 7;
        // repeat_count at 0x0045
        data[0x45] = 3;
        // hit_effect_info_type at 0x0048
        data[0x48..0x4c].copy_from_slice(&0xf177b780u32.to_le_bytes());
        // single_hit_pos_offset at 0x004c
        data[0x4c..0x50].copy_from_slice(&5.0f32.to_le_bytes());
        data[0x50..0x54].copy_from_slice(&10.0f32.to_le_bytes());
        data[0x54..0x58].copy_from_slice(&0.0f32.to_le_bytes());
        // delay desc #1 at 0x0058
        data[0x58..0x5c].copy_from_slice(&0.01f32.to_le_bytes());
        data[0x5c..0x60].copy_from_slice(&1.0f32.to_le_bytes());
        data[0x60..0x64].copy_from_slice(&2.5f32.to_le_bytes());
        // delay desc #2 at 0x0078
        data[0x78..0x7c].copy_from_slice(&0.01f32.to_le_bytes());
        data[0x7c..0x80].copy_from_slice(&1.0f32.to_le_bytes());
        data[0x80..0x84].copy_from_slice(&3.0f32.to_le_bytes());
        // normal_string_index at 0x0070
        data[0x70..0x72].copy_from_slice(&0xffffu16.to_le_bytes());
        // hit_degree at 0x0090
        data[0x90..0x94].copy_from_slice(&50.0f32.to_le_bytes());
        // hit_rotation_type at 0x009c
        data[0x9c] = 0x7a;
        // equip_slot_name_key at 0x00a8
        data[0xa8] = 23;
        // exclude_target_type_flag at 0x00b0
        data[0xb0..0xb4].copy_from_slice(&0xdeadu32.to_le_bytes());
        // single_hit_position_socket at 0x00b4
        data[0xb4..0xb6].copy_from_slice(&42u16.to_le_bytes());
        // _unk_f32_00d4 at 0x00d4
        data[0xd4..0xd8].copy_from_slice(&3.0f32.to_le_bytes());
        // _unk_f32_00dc at 0x00dc
        data[0xdc..0xe0].copy_from_slice(&7.0f32.to_le_bytes());
        // hit_normal_string_index at 0x00f0
        data[0xf0..0xf2].copy_from_slice(&1021u16.to_le_bytes());
        // _unk00f4 at 0x00f4
        data[0xf4] = 5;
        // _unk00fc at 0x00fc
        data[0xfc] = 2;
        // _unk0104 at 0x0104
        data[0x104] = 1;
        // CONST bytes (hardcoded in to_bytes)
        data[0x00c8] = 0x01;
        data[0x00ca] = 0x01;
        data[0x00cd] = 0x01;

        let bd = BaseDataV0::parse(&data).unwrap();
        assert_eq!(bd.weapon_key, 0xdeadbeef,                "weapon_key @ 0x0000");
        assert_eq!(bd.attack_dir, 1,                         "attack_dir @ 0x0004");
        assert!((bd.attack_degree - std::f32::consts::TAU).abs() < 1e-6, "attack_degree @ 0x0020");
        assert_eq!(bd.physic_impulse_power, 1.5,             "physic_impulse_power @ 0x002c");
        assert_eq!(bd.physics_impulse_mass, 2.0,             "physics_impulse_mass @ 0x0030");
        assert_eq!(bd.attack_hit_check_type, 4,              "attack_hit_check_type @ 0x0034");
        assert_eq!(bd.hit_check_normal_str_idx, 0xffff,                      "hit_check_normal_str_idx @ 0x0036");
        assert_eq!(bd.repeat_degree_weight, -1.0,            "repeat_degree_weight @ 0x0038");
        assert_eq!(bd.physics_impulse_velocity, 3.0,         "physics_impulse_velocity @ 0x003c");
        assert!(bd.ignore_safe_zone,                         "ignore_safe_zone @ 0x0040");
        assert_eq!(bd.attack_group_index, 7,                 "attack_group_index @ 0x0044");
        assert_eq!(bd.repeat_count, 3,                       "repeat_count @ 0x0045");
        assert_eq!(bd.hit_effect_info_type, 0xf177b780,      "hit_effect_info_type @ 0x0048");
        assert_eq!(bd.single_hit_pos_offset, [5.0, 10.0, 0.0], "single_hit_pos_offset @ 0x004c");
        assert!((bd._ds1_f0 - 0.01).abs() < 1e-6,            "_ds1_f0 @ 0x0058");
        assert_eq!(bd._ds1_f1, 1.0,                           "_ds1_f1 @ 0x005c");
        assert_eq!(bd._ds1_f2, 2.5,                           "_ds1_f2 @ 0x0060");
        assert!((bd._ds2_f0 - 0.01).abs() < 1e-6,            "_ds2_f0 @ 0x0078");
        assert_eq!(bd._ds2_f1, 1.0,                           "_ds2_f1 @ 0x007c");
        assert_eq!(bd._ds2_f2, 3.0,                           "_ds2_f2 @ 0x0080");
        assert_eq!(bd.normal_string_index, 0xffff,                       "normal_string_index @ 0x0070");
        assert_eq!(bd.hit_degree, 50.0,                    "hit_degree @ 0x0090");
        assert_eq!(bd.hit_rotation_type, 0x7a,                         "hit_rotation_type @ 0x009c");
        assert_eq!(bd.equip_slot_name_key, 23,                           "equip_slot_name_key @ 0x00a8");
        assert_eq!(bd.exclude_target_type_flag, 0xdead,                       "exclude_target_type_flag @ 0x00b0");
        assert_eq!(bd.single_hit_position_socket, 42,         "single_hit_position_socket @ 0x00b4");
        assert_eq!(bd._unk_f32_00d4, 3.0,                    "_unk_f32_00d4 @ 0x00d4");
        assert_eq!(bd._unk_f32_00dc, 7.0,                    "_unk_f32_00dc @ 0x00dc");
        assert_eq!(bd.hit_normal_string_index, 1021,                        "hit_normal_string_index @ 0x00f0");
        assert_eq!(bd._unk00f4, 5,                           "_unk00f4 @ 0x00f4");
        assert_eq!(bd._unk00fc, 2,                           "_unk00fc @ 0x00fc");
        assert_eq!(bd._unk0104, 1,                           "_unk0104 @ 0x0104");

        // roundtrip
        assert_eq!(bd.to_bytes().as_slice(), data.as_slice());
    }

    #[test]
    fn v0_wrong_size_errors() {
        let err = BaseDataV0::parse(&vec![0u8; 100]).unwrap_err();
        assert!(format!("{}", err).contains("264"));
    }

    #[test]
    fn v0_json_roundtrip() {
        let mut bd = zero_v0();
        bd.weapon_key = 0x12345678;
        bd.attack_degree = std::f32::consts::TAU;
        bd.physic_impulse_power = 1.5;
        bd.ignore_safe_zone = true;
        bd.single_hit_position_socket = 42;

        let bytes_before = bd.to_bytes();
        let json = bd.to_json_value();
        let mut w = Vec::new();
        BaseDataV0::write_from_json(&mut w, &json).unwrap();
        assert_eq!(w, bytes_before.as_slice(), "JSON round-trip must be byte-perfect");
    }

    #[test]
    fn attack_info_base_data_decode_encode_v0() {
        let mut data = vec![0u8; 264];
        data[0..4].copy_from_slice(&0xabcd1234u32.to_le_bytes());
        data[0x2c..0x30].copy_from_slice(&1.0f32.to_le_bytes());
        data[0x00c8] = 0x01; // CONST pad
        data[0x00ca] = 0x01; // CONST pad
        data[0x00cd] = 0x01; // CONST pad

        let decoded = AttackInfoBaseData::decode(0, &data).unwrap();
        let encoded = decoded.encode();
        assert_eq!(encoded, data, "decode+encode must round-trip");

        if let AttackInfoBaseData::V0(ref v0) = decoded {
            assert_eq!(v0.weapon_key, 0xabcd1234);
            assert_eq!(v0.physic_impulse_power, 1.0);
        } else {
            panic!("expected V0");
        }
    }

    #[test]
    fn v2_size_is_296() {
        assert_eq!(BaseDataV2::SIZE, 296);
    }

    #[test]
    fn v3_size_is_288() {
        assert_eq!(BaseDataV3::SIZE, 288);
    }

    #[test]
    fn v2_roundtrip() {
        let mut data = vec![0u8; 296];
        data[0x00..0x04].copy_from_slice(&0xdeadbeefu32.to_le_bytes());
        data[0x2c..0x30].copy_from_slice(&1.0f32.to_le_bytes());
        data[0x00c8] = 0x01; data[0x00ca] = 0x01; data[0x00cd] = 0x01;
        data[0x0108..0x010c].copy_from_slice(&0x00030007u32.to_le_bytes());
        data[0x010c..0x0110].copy_from_slice(&0xabcd1234u32.to_le_bytes());
        data[0x0110..0x0114].copy_from_slice(&0.2f32.to_le_bytes());
        data[0x0114..0x0118].copy_from_slice(&1u32.to_le_bytes());
        data[0x0123] = 0x01; // _pad0121 CONST byte[2]=1

        let bd = BaseDataV2::parse(&data).unwrap();
        assert_eq!(bd.base.weapon_key, 0xdeadbeef);
        assert_eq!(bd.projectile_key, 0x00030007);
        assert_eq!(bd.action_hash_code, 0xabcd1234);
        assert!((bd.frame_time - 0.2).abs() < 1e-6);
        assert_eq!(bd.ai_event_key, 1);
        assert_eq!(bd.to_bytes().as_slice(), data.as_slice());

        let decoded = AttackInfoBaseData::decode(2, &data).unwrap();
        assert_eq!(decoded.encode(), data);
        assert_eq!(decoded.version(), 2);
    }

    #[test]
    fn v3_roundtrip() {
        let mut data = vec![0u8; 288];
        data[0x00..0x04].copy_from_slice(&0x11223344u32.to_le_bytes());
        data[0x00c8] = 0x01; data[0x00ca] = 0x01; data[0x00cd] = 0x01;
        data[0x0108..0x010c].copy_from_slice(&0.5f32.to_le_bytes());
        data[0x010c..0x0110].copy_from_slice(&0.2f32.to_le_bytes());
        data[0x0110..0x0114].copy_from_slice(&0x1fc0e737u32.to_le_bytes());
        data[0x0114..0x0118].copy_from_slice(&0xdeadbeefu32.to_le_bytes());
        data[0x0118] = 3;

        let bd = BaseDataV3::parse(&data).unwrap();
        assert_eq!(bd.base.weapon_key, 0x11223344);
        assert!((bd.release_angle_rad - 0.5).abs() < 1e-6);
        assert!((bd.frame_time - 0.2).abs() < 1e-6);
        assert_eq!(bd._unk0110, 0x1fc0e737);
        assert_eq!(bd._unk0114, 0xdeadbeef);
        assert_eq!(bd._unk0118[0], 3);
        assert_eq!(bd.to_bytes().as_slice(), data.as_slice());

        let decoded = AttackInfoBaseData::decode(3, &data).unwrap();
        assert_eq!(decoded.encode(), data);
        assert_eq!(decoded.version(), 3);
    }

    #[test]
    fn v2_json_roundtrip() {
        let mut data = vec![0u8; 296];
        data[0x010c..0x0110].copy_from_slice(&0xdeadbeefu32.to_le_bytes());
        data[0x0110..0x0114].copy_from_slice(&0.2f32.to_le_bytes());
        data[0x00c8] = 0x01; data[0x00ca] = 0x01; data[0x00cd] = 0x01;
        data[0x0123] = 0x01; // _pad0121 CONST byte[2]=1
        let bd = BaseDataV2::parse(&data).unwrap();
        let json = bd.to_json_value();
        let mut w = Vec::new();
        BaseDataV2::write_from_json(&mut w, &json).unwrap();
        assert_eq!(w, data, "V2 JSON round-trip must be byte-perfect");
    }

    #[test]
    fn v3_json_roundtrip() {
        let mut data = vec![0u8; 288];
        data[0x010c..0x0110].copy_from_slice(&0.2f32.to_le_bytes());
        data[0x0110..0x0114].copy_from_slice(&0x1fc0e737u32.to_le_bytes());
        data[0x00c8] = 0x01; data[0x00ca] = 0x01; data[0x00cd] = 0x01;
        let bd = BaseDataV3::parse(&data).unwrap();
        let json = bd.to_json_value();
        let mut w = Vec::new();
        BaseDataV3::write_from_json(&mut w, &json).unwrap();
        assert_eq!(w, data, "V3 JSON round-trip must be byte-perfect");
    }
}
