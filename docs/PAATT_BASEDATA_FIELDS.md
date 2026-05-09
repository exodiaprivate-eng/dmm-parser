<!-- SPDX-License-Identifier: LicenseRef-CDMTL-1.0
     Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
     Licensed under CDMTL v1.0 - see LICENSE.txt -->

# `.paatt` BaseData — Field Directory

Reverse-engineered from `pa::AttackInfoDataDesc` reflection symbols
in the Mac binary (`CrimsonDesert_Steam`). Every field name and
declared type below is sourced from the C++ name-mangled symbols at
`0x1076df1a0` onwards (setters) and `0x1076e3338` onwards (getters).

## Per-version BaseData sizes (empirical, 220 vanilla `.paatt`,
13,789 AttackInfo records)

| version | BaseData size | infos seen | likely sub-variant |
|---|---|---|---|
| 0 | 264 bytes | 10,562 | `AttackInfo` (base) |
| 1 | 528 bytes | 1,674 | `AttackInfo_AttackCatch` (base + 264-byte `AttackCatchDesc`) |
| 2 | 296 bytes | 851 | `AttackInfo_AttackThrow` (base + 4 fields, 32-byte aligned) |
| 3 | 288 bytes | 702 | `AttackInfo_ReleaseCatch` (base + 24 bytes) |
| 4 | 264 bytes | 0 (unused in vanilla) | reserved (matches base size) |

## `pa::AttackInfoDataDesc` — full field list (25 fields)

Sourced by parsing every `_ZN2pa18AttackInfoDataDesc<N>set_<name>ERK<type>`
symbol. The mangled type code reveals the field type:

| Field | Type | Mangled |
|---|---|---|
| `attackDir` | u8 | `RKh` |
| `weaponKey` | u32 | `RKj` |
| `targetType` | TargetType (enum) | `RKNS_10TargetTypeE` |
| `attackIndex` | u8 | `RKh` |
| `repeatCount` | u8 | `RKh` |
| `attackHitData` | `AttackHitDataDesc*` (nested object pointer) | `PKNS_17AttackHitDataDescE` |
| `attackerDelay` | `ActionChartFrameEvent_AttackDelayDataDesc` (value) | `RKNS_41ActionChartFrameEvent_AttackDelayDataDescE` |
| `ignoreSafeZone` | bool | `RKb` |
| `attackCommonData` | `AttackCommonDataDesc*` (nested object pointer) | `PKNS_20AttackCommonDataDescE` |
| `attackDivideType` | enum | `RKNS_38ActionChartFrameEvent_AttackDivideTypeE` |
| `attackGroupIndex` | u8 | `RKh` |
| `noCheckCollision` | bool | `RKb` |
| `hitEffectInfoType` | u32 | `RKj` |
| `attackHitCheckType` | enum | `RKNS_40ActionChartFrameEvent_AttackHitCheckTypeE` |
| `attackImpulseLevel` | u8 | `RKh` |
| `physicImpulsePower` | f32 | `RKf` |
| `physicsImpulseMass` | f32 | `RKf` |
| `repeatDegreeWeight` | f32 | `RKf` |
| `ignoreWhenHitAction` | bool | `RKb` |
| `isSingleHitPosition` | bool | `RKb` |
| `excludeTargetTypeFlag` | u32 | `RKj` |
| `ignoreDefenceTypeFlag` | u32 | `RKj` |
| `physicsImpulseVelocity` | f32 | `RKf` |
| `singleHitPositionOffset` | float3 (12 bytes) | `RKNS_6float3E` |
| `singleHitPositionSocket` | u16 | `RKt` |

## C++ name-mangling type codes

| Code | Meaning |
|---|---|
| `h` | unsigned char (u8) |
| `j` | unsigned int (u32) |
| `i` | int (i32) |
| `t` | unsigned short (u16) |
| `b` | bool |
| `f` | float |
| `Pf` | float pointer |
| `NS_<N><Name>E` | `pa::Name` (nested type) |
| `RK<T>` | `const T&` (read-only ref) |
| `O<T>` | `T&&` (move ref) |

## Class hierarchy summary (all 4 desc structs)

```
AttackInfoDataDesc                       — root (264 bytes for v0)
├── attackHitData → AttackHitDataDesc    — 7 fields (nested via pointer)
├── attackCommonData → AttackCommonDataDesc — 12 fields (nested via pointer)
├── attackerDelay (value) → ActionChartFrameEvent_AttackDelayDataDesc
└── 22 leaf fields (above)

AttackInfo_Attack            — base wrapper (no extra fields)
AttackInfo_AttackThrow       — base + 4 fields:
   ├── projectileKey (u32, ProjectileKeyFromString custom)
   ├── actionHashCode (u32, ActionNameHashCodeFromString)
   ├── aiEventKey (AiEventKey enum)
   └── frameTime (f32, FrameTimeFromString)
AttackInfo_AttackCatch       — base + AttackCatchDesc (~264 bytes)
AttackInfo_ReleaseCatch      — base + (TBD; ~24 bytes extra)
```

## `AttackCommonDataDesc` fields (17 fields, Mac-IDA confirmed)

Recovered Session 19 by decompiling every `__ZNK2pa20AttackCommonDataDesc...get_<field>Ev` getter
in the Mac binary (each is a single `return this+offset` instruction).
Earlier "12 fields" estimate from setter/getter symbol counting missed the
three bit-packed bools and the equipSlot/attackNameCount group. **In-memory
offsets, NOT wire offsets** (Pearl Abyss serializes via metaobject iteration).

| In-mem offset | Field | C++ type | Wire-position candidate (BaseDataV0) |
|---|---|---|---|
| 0x00 (0) | `attackOffset` | float3 (12 B) | wire 0x08 — `attack_pos_offset` ✅ |
| 0x0C (12) | `attackBoxSize` | float3 (12 B) | wire 0x14 — currently `_unk_float3_0014` |
| 0x18 (24) | `attackAngle` | f32 | wire 0x20 — `attack_degree` ✅ |
| 0x1C (28) | `attackYaw` | f32 | wire 0x24 — `attack_yaw` ✅ |
| 0x20 (32) | `innerAttackLength` | f32 | wire 0x28 — currently `_unk_f32_0028` |
| 0x24 (36) | `impulseLengthScale` | f32 | wire 0x2C — currently `physic_impulse_power` (PR #14 named — verify rename?) |
| 0x28 (40) | `impulseAngleScale` | f32 | wire 0x30 — currently `physics_impulse_mass` (verify rename?) |
| 0x2C (44) | `hitType` | enum (u8) | wire 0x34 — currently `attack_hit_check_type` (verify u16 vs u8) |
| 0x2D (45) | `attackPositionType` | u8 | unmapped |
| 0x2E (46) | `attackPositionBone` | u16 (2 B) | unmapped (string-table key?) |
| 0x30 (48) | `detectEventDistance` | f32 | unmapped |
| 0x34 (52) | `equipSlotNameKey` | enum (u8) | wire 0x00a8 — `equip_slot_name_key` ✅ |
| 0x38 (56) | `equipSlotIndex` | u8 | unmapped |
| 0x3C (60) | `attackNameCount` | u8 | unmapped |
| 0x3D (61) bit0 | `ignoreDecreaseEndurance` | bool | unmapped (rare-true bool) |
| 0x3D (61) bit1 | `checkBackGroundHit` | bool | unmapped (rare-true bool) |
| 0x3D (61) bit2 | `isUseReserveSlot` | bool | unmapped (rare-true bool) |

⚠️ The `physic_impulse_power` / `physics_impulse_mass` rename candidacy
needs double-checking — the wire-offset adjacency strongly suggests
`impulseLengthScale` / `impulseAngleScale`, but the contributor named
them from empirical defaults of 1.0 (which would also match length/angle
scale defaults). Either name is functionally consistent. Hold off on
renaming until the .paatt serializer iteration order is mapped.

## `AttackHitDataDesc` fields (8 fields, Mac-IDA confirmed)

| In-mem offset | Field | C++ type | Wire-position candidate |
|---|---|---|---|
| 0x00 (0) | `attackeeDelay` | nested struct (12 B) | wire 0x58–0x6f — currently `_ds1_*` (5 floats) |
| 0x0C (12) | `hitRotationAngle` | f32 | wire 0x90 — currently `hit_degree` (rename candidate; "Degree" was a guess, the C++ name is `hitRotationAngle`) |
| 0x10 (16) | `pushSpeed` | f32 | wire 0xA0 — currently `_unk_f32_00a0` (strong candidate) |
| 0x14 (20) | `maxPushAngleRange` | f32 | wire 0xD4 / 0xDC / 0xE0 / 0xE4 — currently `_unk_f32_*` (one of these four) |
| 0x18 (24) | `ragdollPresetName` | u16 (2 B) | unmapped (string-table key) |
| 0x1A (26) | `hitRotationType` | enum (u8) | wire 0x9C — currently `hit_rotation_type` ✅ |
| 0x1B (27) | `hitPower` | u8 | wire 0xF4 — currently `_unk00f4` (candidate) |
| 0x1C (28) | `pushWithBoneVelocity` | bool | wire 0xFA / 0xFC — currently `_unk00fa` or `_unk00fc` |

**High-confidence wire-mapping deltas** to apply *if* the serializer order is
later proven:
- `_unk_float3_0014` → `attack_box_size`
- `_unk_f32_0028` → `inner_attack_length`
- `_unk_f32_00a0` → `push_speed`
- `_unk00f4` → `hit_power`
- `hit_degree` → `hit_rotation_angle` (rename of an already-named field)

## IDA reference addresses

| Item | Address |
|---|---|
| `_ZTSN2pa18AttackInfoDataDescE` (RTTI) | `0x1072ed3c0` |
| `AttackInfoDataDesc` setter `_ptr` table | `0x1076df1a0` to `0x1076df3f0` |
| `AttackInfoDataDesc` getter `_ptr` table | `0x1076e3338` to `0x1076e34xx` |
| `AttackCommonDataDesc` (RTTI) | `0x1072ed399` |
| `AttackHitDataDesc` (RTTI) | `0x1072ed3ae` |
| `_ZTVN2pa17MetaObjectBindPODINS_18AttackInfoDataDescEEE` (vtable) | `0x10778a5d0` |
| `_ZTIN2pa17MetaObjectBindPODINS_18AttackInfoDataDescEEE` (typeinfo) | `0x10778a618` |

## Per-byte offset extraction — TODO (Session 12 update)

**Setbacks identified Session 12**:

The `_ptr` globals (`__ZN2pa18AttackInfoDataDesc13set_attackDirERKh_ptr`
etc.) point to runtime-filled function-pointer slots, not to
standalone setter implementations. Reading the qword at
`0x107ED06F8` returns `0` because it's only populated when the
metaobject is constructed at startup.

The metaobject builder for AttackInfoDataDesc is split into 25+
`bindProperty_<fieldname>` functions, one per field. We located
`pa::AttackInfoDataDesc::bindProperty_attackDir` at
`sub_100C41D70` and disassembled it:

- Loads the setter/getter pointers from the `_ptr` globals into a
  static-initialized `SimpleReflectPropertyBindPOD<AttackInfoDataDesc, h, ...>`
  descriptor (vtable at
  `_ZTVN2pa28SimpleReflectPropertyBindPODINS_18AttackInfoDataDescEhRKhS3_...`).
- Stores the type-id `5` (= `u8`) at descriptor offset 0x98.
- Tail-calls `sub_1005F3B64(metaobject, descriptor, group_type,
  ReflectGroupType)` which registers the property.

Crucially, **the byte offset is not stored anywhere in the
descriptor** — it's encoded inside the setter lambda. Since the
lambda is inlined into the metaobject's runtime dispatch and not
exposed as a standalone function, IDA cannot recover the offset
from static analysis alone.

## Recommended decode strategy for next iteration

**Pragmatic approach: differential analysis on vanilla samples.**

We have **10,562 v0 BaseData blobs** (264 bytes each ≈ 2.79 MiB of
data) and **1,674 v1 blobs** (528 bytes each). For each byte
position across the dataset:

1. Compute the value distribution.
2. Bytes with binary distribution (mostly 0/1) → likely `bool`.
3. Bytes with low-cardinality distribution (~10 distinct values) →
   likely `enum` or low-range `u8`.
4. 4-byte groups whose byte 0/1 vary together but byte 2/3 stay 0
   → likely `u16`.
5. 4-byte groups with full 32-bit entropy in IEEE-float range
   (most exponents around 0x3F-0x42) → likely `float`.
6. Boundaries between consecutive zero-runs likely indicate struct
   alignment / nested object boundaries (the
   `attackHitData`/`attackCommonData`/`attackerDelay` nested fields).

Combined with the 25-field directory above (sum of fixed sizes:
5 u8 + 4 u32 + 1 u16 + 5 bool + 4 float + 1 float3 + 3 enums = 5
+ 16 + 2 + 5 + 16 + 12 + 3×4 = 68 bytes of leaf fields), the
remaining 264 - 68 = 196 bytes belong to the 3 nested struct
fields (AttackCommonDataDesc + AttackHitDataDesc + AttackDelayDataDesc).

This gives a constrained search space we can brute-force-validate.

## Round-trip status (current — Session 18, PR #14)

`.paatt` is round-trip byte-perfect via `PaattFile::to_bytes()`.

**All four typed BaseData variants are FIELD-DECODED** in
`src/binary/paatt_basedata.rs`:

| Variant | Size | Coverage |
|---|---|---|
| `BaseDataV0` (base AttackInfo) | 264 B | 60+ named fields + `_unkXXXX` placeholders for unresolved positions |
| `BaseDataV1` (= V0 + AttackCatchDesc) | 528 B | V0 fields + 9 named catch fields (`catch_yaw_hi_rad`, `catch_dist_a`, `catch_elevation_rad_a`, …) inside a `catch_desc` sub-dict |
| `BaseDataV2` (= V0 + ThrowDataDesc) | 296 B | V0 fields + `projectile_key`, `action_hash_code`, `frame_time`, `ai_event_key` |
| `BaseDataV3` (= V0 + ReleaseCatchDataDesc) | 288 B | V0 fields + `release_angle_rad`, `frame_time`, plus `_unk0110` / `_unk0114` (release-catch type hashes) |

Mod authors call `paatt_decode_base_data(version, data)` from Python to
get a named-field dict, edit fields like `weapon_key`,
`physic_impulse_power`, etc., then call
`paatt_encode_base_data(version, fields)` to get bytes back. Every
vanilla `.paatt` (220 files, 13,789 AttackInfo records) round-trips
byte-perfect through this path. See `docs/api.md` →
"**.paatt — typed AttackInfo BaseData**" for the Python entry-point
reference and the most-commonly-edited field cheatsheet.

**Session 18 addendum (PR #14):**
- V2 throw payload: 4 named fields recovered via field analysis across
  851 V2 records (`projectile_key`/`action_hash_code`/`frame_time`/`ai_event_key`).
- V3 release-catch payload: 2 named fields recovered across 702 V3 records
  (`release_angle_rad`/`frame_time`); release-catch-type hashes left as
  `_unk0110` / `_unk0114` pending an IDA reflection-symbol pass.
- AttackCatchDesc (the V1 264-byte tail): 9 catch-geometry fields named
  (yaw range, throw distance, elevation cone) from differential entropy
  on 1,674 V1 records; the trailing 176 bytes remain as `_cd_tail` blob.

### Confirmed V0 field offsets (264 bytes) — Session 17 state

| Offset | Field | Type | Notes |
|--------|-------|------|-------|
| 0x0000 | `weapon_key` | u32 | Weapon/action hash; unique per record |
| 0x0004 | `attack_dir` | u8 | 0=base, 1=catch, 3=release-catch |
| 0x0005 | `_pad0005` | [u8;3] | Alignment |
| 0x0008 | `attack_pos_offset` | [f32;3] | `AttackCommonDataDesc.AttackPosOffset` |
| 0x0014 | `_unk_float3_0014` | [f32;3] | `AttackCommonDataDesc` unnamed float3 |
| 0x0020 | `attack_degree` | f32 | `AttackCommonDataDesc.AttackDegree`; default ≈ 6.2832 (2π rad) |
| 0x0024 | `attack_yaw` | f32 | `AttackCommonDataDesc.AttackYaw`; default 0.0 |
| 0x0028 | `_unk_f32_0028` | f32 | `AttackCommonDataDesc` unnamed float |
| 0x002c | `physic_impulse_power` | f32 | Vanilla default 1.0 |
| 0x0030 | `physics_impulse_mass` | f32 | Vanilla default 1.0 |
| 0x0034 | `attack_hit_check_type` | u16 | Enum; common value 4 |
| 0x0036 | `hit_check_normal_str_idx` | u16 | `NormalStringIndex`; 0xffff=none; 0xffff@98% V0, always for V2/V3 |
| 0x0038 | `repeat_degree_weight` | f32 | Vanilla default -1.0 |
| 0x003c | `physics_impulse_velocity` | f32 | Vanilla default 0.0 |
| 0x0040 | `ignore_safe_zone` | bool | + 3-byte pad |
| 0x0044 | `attack_group_index` | u8 | Common value 1 |
| 0x0045 | `repeat_count` | u8 | Common value 2; + 2-byte pad |
| 0x0048 | `hit_effect_info_type` | u32 | Effect hash; 0xf177b780 most common |
| 0x004c | `single_hit_pos_offset` | [f32;3] | `singleHitPositionOffset` XYZ; default (0,0,0); V2/V3 always zero |
| 0x0058 | `_ds1_f0..f4` | f32×5 | `ActionChartFrameEvent_AttackDelayDataDesc` #1; f0=trigger time (s; 0.0/0.05/0.10), f1=blend/end (0.0/1.0), f2=secondary offset, f3=angle_rad (99.9% zero), f4=frame count (int; 0/6/9/11) |
| 0x006c | `_pad_ds1` | [u8;4] | Always zero |
| 0x0070 | `normal_string_index` | u16 | `AttackCommonDataDesc.NormalStringIndex`; V0: 0x0000 @70%; V2/V3: 0xffff (always) |
| 0x0072 | `_unk0072` | bool | bool (B00@85% V0) |
| 0x0073 | `_unk0073` | u8 | u8 enum (mode=1 @49% V0) |
| 0x0074 | `_pad0074` | [u8;4] | Always zero |
| 0x0078 | `_ds2_f0..f4` | f32×5 | `ActionChartFrameEvent_AttackDelayDataDesc` #2; f0-f3 identical to ds1 (98%+); f4 almost always 0.0 (non-zero: 1.1-1.53 scale) |
| 0x008c | `_pad_ds2` | [u8;4] | Always zero |
| 0x0090 | `_unk_f32_0090` | f32 | `AttackHitDataDesc.Degree` candidate; V0 mode=50.0; V2/V3=0.0 |
| 0x0094 | `_pad0094` | [u8;8] | Always zero |
| 0x009c | `_unk009c` | u8 | Attack-type enum; V0: mode=0; V2: 0x7a; V3: 0x5a |
| 0x009d | `_pad009d` | [u8;3] | Always zero |
| 0x00a0 | `_unk_f32_00a0` | f32 | `AttackHitDataDesc` unnamed float; usually 0.0 |
| 0x00a4 | `_pad00a4` | [u8;4] | Always zero |
| 0x00a8 | `_unk00a8` | u8 | `EquipSlotNameKey` candidate; V0 mode=12; V2/V3=23 |
| 0x00a9 | `_pad00a9` | [u8;3] | Always zero |
| 0x00ac | `_unk00ac` | [u8;4] | byte[0] u8 (≈0); byte[1] bool; bytes[2,3]=0 |
| 0x00b0 | `_unk00b0` | u32 | Bitmask; 77.5% zero; 25 distinct; candidate: `excludeTargetTypeFlag` |
| 0x00b4 | `single_hit_position_socket` | u16 | Socket name-table index; 0xffff=none |
| 0x00b6 | `_pad00b6` | [u8;2] | Alignment |
| 0x00b8 | `_unk00b8` | [u8;16] | Attack-type byte region; byte patterns differ by V0/V2/V3 |
| 0x00c8 | `_unk00c8` | [u8;12] | Dense const/varying byte region (frame timing?) |
| 0x00d4 | `_unk_f32_00d4` | f32 | Integer-valued f32; 5 distinct values; ≈0 |
| 0x00d8 | `_pad00d8` | [u8;4] | Always zero |
| 0x00dc | `_unk_f32_00dc` | f32 | ≈0.0; V2/V3 always 0; V0 sometimes non-zero |
| 0x00e0 | `_unk_f32_00e0` | f32 | ≈0.0 |
| 0x00e4 | `_unk_f32_00e4` | f32 | ≈0.0 |
| 0x00e8 | `_unk00e8` | [u8;4] | byte[0]=CONST 1; byte[1]=bool; bytes[2,3]=0 |
| 0x00ec | `_pad00ec` | [u8;4] | Always zero |
| 0x00f0 | `hit_normal_string_index` | u16 | `AttackHitDataDesc.NormalStringIndex`; V0 often 1021 (0x03fd); V2/V3 often 0 |
| 0x00f2 | `_pad00f2` | [u8;2] | Always zero |
| 0x00f4 | `_unk00f4` | u8 | `AttackHitDataDesc` field 6 (unnamed u8); 1=no-rotation (pairs with hit_degree=0°); 5/2/6/4/3=rotation types |
| 0x00f5 | `_pad00f5` | [u8;3] | Always zero |
| 0x00f8 | `hit_data_str_idx` | u16 | `NormalStringIndex`; 0=none; 95%+ non-zero cases pair with attack_hit_check_type=4; V2/V3 always 0 |
| 0x00fa | `_unk00fa` | bool | bool (99% false); likely `AttackHitDataDesc` field 7 (unnamed bool) |
| 0x00fb | `_pad00fb` | u8 | Always zero |
| 0x00fc | `_unk00fc` | u8 | `AttackHitDataDesc` field 7 candidate; 0=false (when _unk00f4=1); 1=true (rotation types) |
| 0x00fd | `_pad00fd` | [u8;3] | Always zero |
| 0x0100 | `hit_data_str_idx_b` | u16 | Secondary `NormalStringIndex`; 0=none; always co-present with `hit_data_str_idx`; values in same range (0x0450–0x046d) |
| 0x0102 | `_pad0102` | [u8;2] | Always zero |
| 0x0104 | `_unk0104` | u8 | u8; 4 values (98% zero); no clear correlation |
| 0x0105 | `_pad0105` | [u8;3] | Always zero |

V1 = V0 (264 bytes) + `catch_desc` blob (264 bytes).

## What the V2/V3 cross-version analysis confirmed (Session 14)

Running per-version entropy across all 4 versions revealed:

| Offset | V0 mode | V2 (throw) | V3 (rel-catch) | Interpretation |
|--------|---------|------------|-----------------|----------------|
| 0x004c–0x0057 | ≈0 (97%) | always 0 | almost 0 | `singleHitPositionOffset` ← **decoded** |
| 0x0070–0x0071 | 0x0000 (70%) | 0xffff (always) | 0xffff (always) | `NormalStringIndex` (CommonData?) |
| 0x0090 | 50.0 (68%) | 0.0 (always) | 0.0 (always) | `AttackHitDataDesc.Degree` candidate |
| 0x009c | 0 (28%, 19 vals) | 0x7a (92%) | 0x5a (99%) | `AttackNameList` u8 candidate |
| 0x00a8 | 12 (66%, 16 vals) | 23 (always) | 23 (always) | `EquipSlotNameKey` u8 candidate |
| 0x00b8+3 | 5 (43%) | 5 (always) | 5 (89%) | unknown u8, consistent default |

## Remaining decoding work

1. `_unk0058`/`_unk0078` (0x0058–0x008f): two 24-byte delay sub-structs — confirm 5-float layout
   - 0x0058–0x006b: 5 floats (likely `attackerDelay` sub-struct — V2 shows 0.01/1.0/0.0/x/0)
   - 0x006c–0x006f: always-zero pad
   - 0x0078–0x008b: second delay sub-struct (same pattern)
   - 0x008c–0x008f: always-zero pad
2. Sub-fields within blob regions still needing a name:
   - `_unk0070`: confirm as `AttackCommonDataDesc.NormalStringIndex`
   - `_unk0072`: confirm bool/enum split and field names
   - `_unk009c`: confirm as `AttackNameList` or `HitRotationType`
   - `_unk00a8`: confirm as `EquipSlotNameKey`
   - `_unk00b0`: confirm as `excludeTargetTypeFlag` or `ignoreDefenceTypeFlag`
   - `_unk00b8`/`_unk00c8`: 28-byte attack-type region — decode sub-bytes
   - `_unk_f32_00d4`: identify field name (integer-valued float)
   - `_unk_f32_00dc`/`_unk_f32_00e0`/`_unk_f32_00e4`: three adjacent floats — likely one struct member group
   - `_unk00e8[1]`: confirm bool field name
   - `_unk00f0`: confirm as `AttackHitDataDesc.NormalStringIndex`
   - `_unk00f4`: confirm unnamed u8 field
   - `_unk00f8`: decode u16 + bool sub-fields
   - `_unk00fc`: identify u8 enum (8 values)
   - `hit_data_str_idx_b`: secondary NormalStringIndex (resolved)
   - `_unk0104`: identify u8 (4 values)
3. `_unk0036` (0x0036): confirm as `NormalStringIndex` from `AttackHitDataDesc` (0xffff=none)
4. AttackCatchDesc (V1 `catch_desc` blob, 264 bytes): decode field by field

## Appendix: In-memory class layout (Session 19, IDA-confirmed)

Decompiled the `pa::AttackInfoDataDesc::get_<field>` zero-argument
getters in the Mac binary. Each getter is 8 bytes and resolves to a
single ARM64 instruction returning `this + offset`, so the in-memory
class layout falls out for free. **These are class-instance offsets,
NOT wire offsets** — Pearl Abyss serializes field-by-field through
the metaobject's setter table rather than memcpy'ing the class, so
the wire `BaseDataV0` layout (264 B) reorders fields freely. The
in-memory map still confirms (a) every C++ field is real, (b) what
size each leaf is, and (c) which fields cluster together — useful
context when searching for a still-`_unkXXXX` wire field's identity.

| In-mem offset | Field | C++ type | Wire offset (BaseDataV0) | Status |
|---|---|---|---|---|
| 0x94 (148) | `ignoreDefenceTypeFlag` | u32 | **TBD** | unmapped (could be `_unk00b8`-region u32 or one of the late `_unk` slots) |
| 0xA0 (160) | `targetType` | enum (4 B) | **TBD** | unmapped |
| 0xA4 (164) | `excludeTargetTypeFlag` | u32 | 0x00b0 | ✅ already named in `BaseDataV0` |
| 0xA8 (168) | `weaponKey` | u32 | 0x0000 | ✅ already named |
| 0xB0 (176) | `attackImpulseLevel` | u8 | **TBD** | likely candidate for `_unk0073` (u8 enum, mode=1 @49% V0) |
| 0xB1 (177) | `attackIndex` | u8 | **TBD** | unmapped |
| 0xB2 (178) | `attackGroupIndex` | u8 | 0x0044 | ✅ already named |
| 0xB3 (179) | `attackDir` | u8 | 0x0004 | ✅ already named |
| 0xB4 (180) | `repeatCount` | u8 | 0x0045 | ✅ already named |
| 0xB5 (181) | `noCheckCollision` | bool | **TBD** | candidate for `_unk0072` (B00@85% V0) |
| 0xB6 (182) | `ignoreWhenHitAction` | bool | **TBD** | candidate for one of the rare-true bools (`_unk00ad`, `_unk00bd`, `_unk00bf`, `_unk00c0`, `_unk00d0..d3`, `_unk00e9`) |
| 0xB7 (183) | `isSingleHitPosition` | bool | **TBD** | candidate for one of the rare-true bools |
| 0xB8 (184) | `ignoreSafeZone` | bool | 0x0040 | ✅ already named |

**Reflection-symbol provenance:** addresses for the get/set/move/bindProperty
function-pointer slots and the corresponding `bindProperty_<field>`
implementations are listed under the table at `0x1076df1a0` (setters)
and `0x1076e3338` (getters), with bindProperty wrappers at
`0x1076d0560..0x1076d06e8`. The Mac equivalents of the runtime-filled
setters are exposed as `__ZN2pa18AttackInfoDataDesc<N>set_<field>...`.

**Why wire offsets remain unknown:** the contributor noted in
"Per-byte offset extraction — TODO" above that bindProperty wrappers
push the field offset into a *setter lambda* (inlined into the
metaobject runtime); the offset is not stored anywhere in the
descriptor, so static analysis can't recover it. The next step is
to find the `.paatt` reader/writer pair (presumably a templated
function over `pa::ReflectObjectPOD<AttackInfoDataDesc>`) and trace
its iteration order — that gives the wire→class field map directly.

**Newly-confirmed C++ fields not yet present in `BaseDataV0`:**
`targetType`, `attackIndex`, `attackImpulseLevel`, `noCheckCollision`,
`ignoreWhenHitAction`, `isSingleHitPosition`, `ignoreDefenceTypeFlag`,
`attackDivideType` (no getter found, suggests it's enum stored
inside an unnamed slot). Total: 8 C++ fields awaiting wire-position
proof — once mapped, the corresponding `_unkXXXX` placeholders in
`BaseDataV0` get renamed without any JSON-shape break (the rename is
a pure documentation improvement; bytes round-trip identically).

## Appendix: `.paatt` loader anchors (Session 20, IDA-confirmed)

Located the `.paatt` file loader chain in the Mac binary. Useful as
durable IDA anchors for future RE work.

### Format anchors

| Address | Symbol | Role |
|---|---|---|
| `0x100c46104` | `sub_100C46104` | `.paatt` LOADER. Walks `<resource_root>/attackinfo` for `*.paatt` files; per-file calls `sub_100C465A4`. |
| `0x100c465a4` | `sub_100C465A4` | Per-`.paatt` parser. Reads `InfoCount` u32, allocates 88-byte AttackInfo records, then reads 9 trailing string tables in fixed order. |
| `0x100c4712c` | `sub_100C4712C` | Per-AttackInfo record reader. Reads version byte, allocates BaseData blob (264/528/296/288/264 bytes for V0/V1/V2/V3/V4), then reads 9 child sub-structures. |
| `0x1011a72d0` | `sub_1011A72D0` | Returns the literal `"paatt"` extension string. |
| `0x10732d49e` | (string data) | Literal `"paatt"` (5 bytes). |

### `.paatt` top-level wire layout (IDA-verified, error-message-derived)

Korean error messages inside `sub_100C465A4` reveal the loader's read
order — when a section fails to parse, it emits `AttackInfo 로드 실패(<section>)`.
The order is therefore the WIRE order:

1. **InfoCount** (u32) — number of AttackInfo records.
2. **AttackInfo[InfoCount]** — each record per `sub_100C4712C`:
   - u8 `version` (0/1/2/3/4)
   - BaseData blob: 264 B (V0), 528 B (V1), 296 B (V2), 288 B (V3), 264 B (V4)
   - 9× child 16-byte sub-structures (slot indices 0..8 in the per-record allocation)
3. **StringTable**
4. **EffectNameTable**
5. **EffectInfoKeyTable**
6. **SocketNameTable**
7. **PartNameTable**
8. **SequencerNameTable**
9. **PrefabNameTable**
10. **FrameEventBuffer**

This matches the existing dmm-parser `PaattFile` parser exactly — the
220/220 vanilla round-trip already validated this layout empirically.
The IDA confirmation is a durable correctness anchor for future work.

### Why wire ≠ in-memory class layout (resolved)

`sub_10058F658(stream, size)` (called from `sub_100C4712C` line 50) is
a stream-read primitive — it allocates `size` bytes and reads them
contiguously from the input. The returned pointer is stored at
AttackInfo slot `a1[9]` as the raw serialized blob. The C++ class
`pa::AttackInfoDataDesc` at `a1[9]` having `weaponKey` at in-mem
offset 0xA8 (168) describes the **deserialized in-memory layout** —
that layout differs from the on-disk wire layout because Pearl Abyss
parses each field via the metaobject's setter pipeline, not memcpy.

This means the in-memory offsets recovered in Session 19 do **not**
directly translate to wire offsets, even though both refer to the
same logical fields. Wire→class field mapping still requires either
(a) finding the `pa::AttackInfoDataDesc` `serialize` / `deserialize`
member that walks the metaobject in registration order, or
(b) field-by-field byte-signature analysis on a vanilla record.

Use `examples/paatt_basedata_layout.rs` with per-version output to confirm field boundaries.
