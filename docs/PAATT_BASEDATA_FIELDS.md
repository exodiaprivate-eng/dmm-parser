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

## `AttackCommonDataDesc` fields (12 fields)

From `_ZN2pa20AttackCommonDataDesc...` symbols. Field types:

| Field | Type |
|---|---|
| `AttackPosOffset` | float3 (custom toString) |
| (unnamed float3) | float3 |
| `AttackDegree` | f32 (DegreeToString) |
| `AttackYaw` | f32 (DegreeToString) |
| (unnamed float) | f32 |
| `AttackHitType` | enum |
| (3× bool bitfield) | bool×3 (bit-packed) |
| `AttackPositionType` | enum |
| `NormalStringIndex` | u16 |
| `AttackNameList` | u8 |
| `EquipSlotNameKey` | enum |
| (unnamed i32) | i32 |

## `AttackHitDataDesc` fields (7 fields)

| Field | Type |
|---|---|
| `attackerDelay` | `ActionChartFrameEvent_AttackDelayDataDesc` |
| `HitRotationType` | enum |
| `Degree` | f32 (DegreeToString) |
| (unnamed float) | f32 |
| `NormalStringIndex` | u16 |
| (unnamed u8) | u8 |
| (unnamed bool) | bool |

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

## Round-trip status (current)

`.paatt` is round-trip byte-perfect via `PaattFile::to_bytes()`
(added Session 10) — BaseData is preserved verbatim as `Vec<u8>`.
Mod authors can read every other field and the BaseData bytes; they
just cannot YET set individual BaseData fields by name (would have
to byte-edit the BaseData blob).

## Once per-byte offsets are known

Convert `AttackInfo.base_data: Vec<u8>` into:

```rust
pub enum AttackInfoBaseData {
    V0(BaseDataV0),  // 264 bytes
    V1(BaseDataV1),  // 528 bytes (V0 + AttackCatchDesc)
    V2(BaseDataV2),  // 296 bytes (V0 + AttackThrow extra fields)
    V3(BaseDataV3),  // 288 bytes (V0 + ReleaseCatch extra)
    V4(BaseDataV4),  // 264 bytes (matches V0)
}

pub struct BaseDataV0 {
    pub attack_dir: u8,
    pub weapon_key: u32,
    pub target_type: u32,  // TargetType
    pub attack_index: u8,
    pub repeat_count: u8,
    pub attack_hit_data: AttackHitDataDesc,
    pub attacker_delay: AttackDelayDataDesc,
    pub ignore_safe_zone: bool,
    pub attack_common_data: AttackCommonDataDesc,
    pub attack_divide_type: u32,
    // ... etc, 25 fields total
    pub _padding_or_unused: [u8; N],  // any leftover bytes
}
```

Each typed struct will round-trip byte-perfect against the existing
13,789 vanilla AttackInfo records.
