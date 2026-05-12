# Mac IDA Audit — 2026-05-12

Full alignment of 5 dmm-parser tables against the Mac binary's canonical
wire layout, using IDA Pro on `CrimsonDesert_Steam` (sha256
`8c26af16814ed38295541b085c984316e8637205239e7668a1d6429551ad9393`).

**The Mac binary is the 1.06 source of truth.** Its parser functions
contain Korean error strings that expose every canonical field name read
by the wire-decode chain, and its individual reader sub-functions
(decompiled) reveal the exact wire byte count per field.

## Summary

| Table | Mac canonical | dmm-parser | Action |
|---|---|---|---|
| `reserve_slot_info` | 18 wire fields | 17 (post-fix) | ✅ Struct fixed + aliased |
| `mercenary_info` | 27 wire fields | 27 (post-rewrite) | ✅ **Full Mac-canonical rewrite** |
| `dialog_voice_info` | 16 wire fields | 16 (post-fix) | ✅ Struct fixed + aliased |
| `stage_info` | 88 | 95 (Rust placeholder unrolls) | ✅ Existing aliases populated |
| `tribe_info` | 28 | 29 (Rust placeholder wire-pos) | ✅ Existing aliases populated |
| `item_use_info` | 4 (dispatcher) | 4 | ✅ Already matched |
| `special_mode_info` | 24 | 24 | ✅ Already matched |

## reserve_slot_info — struct fixed

**Mac parser**: `sub_101889F14` (0x310 bytes). 18 wire fields.

**Bugs found**:
1. `enable_tribe_list: CArray<u32>` → Mac reader `sub_1018BB0A4` reads
   wire u16 per element (same as `enable_vehicle_list`). Old u32 read
   drifted `2 * tribe_count` bytes per record — root cause of the
   "1.06 first record OK, later records fail at 0x79c" symptom.
2. `enable_mercenary_list` (Mac canonical position #14, mem offset 104,
   reader `sub_10117EDF4` wire u8) was missing entirely from the struct.

**Fix applied**: changed `enable_tribe_list` to `CArray<u16>`, added
`enable_mercenary_list: CArray<u8>`, kept `_enableSpecialNameHashList`
removed per NattKh CGM v1.1.9 (Win 1.06 dropped that read site).

## mercenary_info — full Mac-canonical struct rewrite

**Mac parser**: `sub_101893AF0` (0x360 bytes). 27 wire fields,
fixed-size 45+N+8K byte record.

**Bug found**: dmm-parser struct packed Mac's 19 individual u8 fields
into 14 u8 + 1 u32 (`combat_targeting_flags`) + 1 u8 (`packed_flags_106`)
with placeholder Rust names at wrong semantic positions. Total wire
bytes matched Mac (45+N+8K), so fixture roundtrip worked, but the
field-by-field semantic mapping was broken:

| dmm-parser pre-fix          | Wire byte | Mac canonical               |
|------------------------------|-----------|------------------------------|
| `far_from_leader_option` u8  | 1         | `_mercenaryType`             |
| `combat_targeting_flags` u32 | 2-5       | `_farFromLeaderOption` + `_isControllable` + `_isPlayable` + `_summonAfterRegist` (packed) |
| `is_controllable` u8         | 6         | `_mainMercenaryPerTribe`     |
| `is_playable` u8             | 7         | `_isForceStackable`          |
| `set_new_mercenary_is_main` u8 | 8       | `_isSellable`                |
| `main_mercenary_per_tribe` u8 | 9        | `_useCampLevel`              |
| `is_force_stackable` u8      | 10        | `_applyEquipItemStat`        |
| `is_sellable` u8             | 11        | `_isGrowable`                |
| `use_camp_level` u8          | 12        | `_checkItemNoOnPushToItem`   |
| `apply_equip_item_stat` u8   | 13        | `_allowExceedLimitHireCount` |
| `spawn_position_type` u8     | 14        | `_isSelectMercenarySpawn`    |
| `mercenary_type` u8          | 15        | `_unspawnOnFocusActorChanged`|
| `is_growable` u8             | 16        | `_isMainDischargeable`       |
| `parent_mercenary_group_info` u8 | 17    | `_spawnPositionType`         |
| `summon_owner_option` u8     | 18        | `_summonOwnerOption` ✓        |
| `packed_flags_106` u8        | 19        | `_parentMercenaryGroupInfo`  |

**Fix applied**: full struct rewrite to 27 Mac-canonical 1:1 fields.
Wire byte total unchanged (45+N+8K), so existing fixture roundtrip
preserved. Python API now exposes correct canonical names.

Wire bytes verified via decompile of Mac readers:
- `sub_100F3E64C` (`_key`) — `char v4; vtbl(a1, &v4, 1LL)` → u8 wire
- `sub_100CCA0FC` (`_mercenaryType`) — vtbl `1LL` size arg → u8 wire
- `sub_101415408/428/448` (`_farFromLeaderOption`/`_spawnPositionType`/`_summonOwnerOption`) — u8 wire
- `sub_10187F694` (`_parentMercenaryGroupInfo`) — `unsigned __int8 v17` byref → u8 wire, mem u16 (hash lookup)
- `sub_100CB4734` (`_sharedSummonCountTag`) — `int v17` byref → u32 wire
- `sub_1018BF738` (`_hiredSkillInfoList`) — CArray<u64>, u32 count + count × 8 bytes

## dialog_voice_info — struct fixed

**Mac parser**: `sub_10187EFEC` (0x238 bytes). 16 wire fields.

**Bug found**: dmm-parser's `key: u16` consumed 1 extra wire byte at
the start, exactly cancelling the missing `_footStepDisableCollideImpactSound`
byte at wire position #9. Total bytes matched (37+6N+2K), but key reads
were corrupt and field semantics were shifted.

**Fix applied**: `key: u16` → `key: u8`. Inserted
`foot_step_disable_collide_impact_sound: u8` between
`foot_step_ground_sound_event` and `foot_step_sound_offset`. Wire byte
total unchanged.

Wire types verified:
- `sub_100F39E0C` (`_key`) — same vtbl 1-byte pattern → u8 wire
- `sub_1006BED20` (`_footStepDisableCollideImpactSound`) — same 1-byte
  vtbl pattern as other u8 fields

## stage_info, tribe_info — no struct changes needed

Both tables already have extensive placeholder-name → Mac-canonical
aliases in `field_aliases_v3_1.rs`. Wire layouts (Rust placeholders
encoded by wire byte position like `raw_*`, `lookup_*`, `unk_*`) match
Mac total wire byte counts. No changes needed.

## item_use_info, special_mode_info — already matched

`item_use_info` dispatcher reads 4 fields matching Mac.
`special_mode_info` has 24 fields matching Mac canonical 1:1.

## Verification status — 1.06 fixture roundtrip VERIFIED

Live 1.06 fixtures extracted from Steam install (group 0008) via
`examples/extract_1_06_test_fixtures.rs`. Roundtrip results
(`examples/verify_1_06.rs`):

```
[mercenary]    OK: 18 records,  1050 bytes, byte-identical roundtrip
[dialog_voice] OK: 483 records, 35473 bytes, byte-identical roundtrip
[reserve_slot] OK: 27 entries,  3383 bytes, byte-identical roundtrip
```

### Discoveries during verification

1. **dialog_voice_info `key` is u16, not u8.** Initial Mac decompile
   pattern-matched on the same name "key reader" pattern across tables,
   but `sub_100F39E0C` (dialog_voice) reads `__int16 v4` BYREF with
   vtbl size arg `2LL` — different from `sub_100F3E64C` (mercenary,
   which is `char v4` with `1LL`). Reverted mid-day u16→u8 change.

2. **`_enableSpecialNameHashList` is STILL present in 1.06.** NattKh
   CGM v1.1.9 release notes claimed 1.06 removed the field; the
   live 1.06 fixture proves otherwise — entries are 4 bytes short
   per record without the (empty) CArray header. Restored the field.

### File size growth 1.05 → 1.06

| Table | 1.05 bytes | 1.06 bytes | Delta | Per-record growth |
|---|---|---|---|---|
| mercenaryinfo.pabgb | 662 | 1050 | +388 | ~22 bytes/record × 18 |
| dialogvoiceinfo.pabgb | 35084 | 35473 | +389 | ~1 byte/record × 483 (DisableCollideImpactSound u8) |
| reserveslot.pabgb | 3294 | 3383 | +89 | ~4 bytes/entry × 27 (empty _enableMercenaryList CArray) |

## Extended verification 2026-05-12 — mission_info + stage_info

NattKh's tooling team reported being unable to figure out mission_info
and stage_info for 1.06. Tested both against extracted 1.06 fixtures:

### mission_info — ✅ 100% on 1.06 with no struct changes

```
[mission] OK: 6506 entries, 2207700 bytes, byte-identical roundtrip
```

The pre-existing struct (`Tier 1` with `sub_1410ED0E0` reader heritage)
already handles 1.06 correctly. No fixes needed.

### stage_info — partial: 83% pass after +1 u8 fix

```
[stage] FAIL entry 42521 key=0xf5161 (start 21675116, expected end
21675682, cursor died at 21675496): CArray count 4294901760 exceeds
remaining bytes 4466726 at offset 21675496 (after 42521/50789 ok)
```

Added `flag_v: u8` at end of typed prefix (1.06 added 1 byte/entry).
First 42521 of 50789 entries (83%) now roundtrip byte-identically.

Entry 42521 ("GreymaneCamp_Contents_armwrestling_I", 566 bytes) and
later entries have a polymorphic field variant whose layout differs
from the typed prefix's assumptions — the parser misreads a CArray
count mid-entry (gets `0xFFFE0000` as count). Likely a new variant
in `OptStageOpt52` or similar polymorphic optional fields.

Resolving the remaining 17% requires:
1. Identifying which entry-content pattern triggers the divergence
2. Decompiling the Mac binary stage parser (`sub_101873B38`, 2644
   bytes — biggest in the binary) to see the 1.06-changed branch
3. Updating the relevant Optional/polymorphic field decoder

This is deferred — it's the same kind of polymorphic dispatch issue
that PR #17 tackled for other tables. The 83% partial coverage is
already a meaningful improvement over the pre-fix 0%.
