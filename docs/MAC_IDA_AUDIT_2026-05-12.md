# Mac IDA Audit — 2026-05-12

Audit of 5 dmm-parser tables against the Mac binary's canonical wire
layout, using IDA Pro on `CrimsonDesert_Steam` (sha256
`8c26af16814ed38295541b085c984316e8637205239e7668a1d6429551ad9393`).
The Mac binary's parser functions contain Korean error strings that
expose every canonical field name read by the wire-decode chain — the
ground truth for canonical schema reconciliation.

## Summary

| Table | Mac canonical | dmm-parser struct | Status |
|---|---|---|---|
| `reserve_slot_info` | 18 | 17 (post-fix) | ✅ **Fixed** (struct + aliases) |
| `mercenary_info` | 27 | 24 | ⚠️ **Aliases updated**, struct wire-layout uncertain |
| `dialog_voice_info` | 16 | 15 | ⚠️ **Aliases updated**, +1 field unverified |
| `stage_info` | 88 | 95 | ✅ Aliases already populated, struct round-trips |
| `tribe_info` | 28 | 29 (wire-position placeholders) | ✅ Aliases already populated, struct round-trips |
| `item_use_info` | 4 (dispatcher) | 4 | ✅ Already matched |
| `special_mode_info` | 24 | 24 | ✅ Already matched |

PR #17 (NattKh, merged 2026-05-11) already fixed `character_info`,
`vehicle_info`, and `faction_node_info` for game 1.0.5.

## reserve_slot_info — confirmed broken, fixed

**Mac parser**: `sub_101889F14` (0x310 bytes). 18 wire fields.

**dmm-parser bug**: pre-fix had `enable_tribe_list: CArray<u32>`, but
Mac reader `sub_1018BB0A4` shows wire `u16` per element (same shape as
`enable_vehicle_list`). Each record with `tribe_count > 0` drifted
`2 * tribe_count` bytes — the root cause of the "1.06 first record OK,
later records fail at offset 0x79c" symptom.

**Additional bug**: dmm-parser was missing `_enableMercenaryList`
entirely. Mac canonical position #14 at mem offset 104, reader
`sub_10117EDF4`, wire u8 per element.

**Fix applied 2026-05-12** (commit pending):
1. `enable_tribe_list: CArray<u32>` → `CArray<u16>`
2. Added `enable_mercenary_list: CArray<u8>` between
   `enable_vehicle_list` and `target_item_group_list`
3. Updated `field_aliases_v3_1.rs` with `_enableMercenaryList`
4. Kept `_enableSpecialNameHashList` removed per NattKh CGM v1.1.9
   release notes (1.06 dropped the read site even though Mac retains
   the string at 0x144b1d342 / mem offset 120)

Result: dmm-parser now has 17 wire fields matching the Win 1.06 layout
(Mac canonical 18 minus the 1.06-removed `_enableSpecialNameHashList`).

## mercenary_info — wire-layout uncertain

**Mac parser**: `sub_101893AF0` (0x360 bytes). 27 wire fields,
fixed-size 56-byte record + `_hiredSkillInfoList: CArray`.

**Wire order discrepancy**: dmm-parser Rust struct has fields at
positions that don't 1:1 match Mac wire positions. Specifically:
- Rust `combat_targeting_flags: u32` (1 wider field) appears to pack
  Mac positions 8-11 (`_farFromLeaderOption` + `_isControllable` +
  `_isPlayable` + `_summonAfterRegist`, all u8).
- Rust `packed_flags_106: u8` represents some subset of the 1.06-added
  byte cluster (`_checkItemNoOnPushToItem` /
  `_allowExceedLimitHireCount` / `_isSelectMercenarySpawn` /
  `_unspawnOnFocusActorChanged` / `_isMainDischargeable`).
- Rust `set_new_mercenary_is_main` alias was `_setNewMercenaryIsMain`
  (non-canonical) — Mac canonical at that wire position is
  `_summonAfterRegist`.

**Action taken 2026-05-12**: corrected and extended
`field_aliases_v3_1.rs` to map verified-canonical fields:
- `set_new_mercenary_is_main` → `_summonAfterRegist` (was
  `_setNewMercenaryIsMain`)
- Added: `is_playable`, `mercenary_type`, `is_growable`,
  `parent_mercenary_group_info`, `summon_owner_option`,
  `shared_summon_count_tag`, `hired_skill_info_list`

**Not changed**: struct field types/positions. The current dmm-parser
round-trips on the test fixture; changing wire layout without a 1.06
fixture roundtrip test risks regression.

## dialog_voice_info — 1 field unverified

**Mac parser**: `sub_10187EFEC` (0x238 bytes). 16 wire fields.

Mac canonical has `_footStepDisableCollideImpactSound` at wire position
#9 (between `_footStepGroundSoundEvent` and `_footStepSoundOffset`).
dmm-parser struct does not have this field.

Either 1.06 wire dropped the field, or dmm-parser is currently
mis-consuming the byte somewhere. Resolution requires 1.06 fixture
roundtrip test.

**Action taken 2026-05-12**: extended `field_aliases_v3_1.rs` with the
two trivially-mappable existing fields (`gender`, `character_age`) and
documented the missing-field hypothesis inline.

## stage_info, tribe_info — already aliased

Both tables have extensive placeholder-name → Mac-canonical name
aliases in their `field_aliases_v3_1.rs`. Their Rust structs use
unique-name placeholder wire-position encoding (`raw_*`, `lookup_*`,
`unk_*`) and round-trip on existing fixtures. No struct changes
needed; the aliases supply Swiss-interop canonical names.

## What's still needed for 100% verification

For each of `mercenary_info`, `dialog_voice_info`, `stage_info`,
`tribe_info`: run `cargo test --lib <table>::tests::roundtrip` against
a 1.06 `.pabgb` fixture. If round-trip passes, the wire layout is
correct and the audit is done. If it fails, the failing offset
identifies which field needs depth correction.

The IDA Mac binary supplies canonical names; Win 1.06 parser
decompiles would confirm field counts/widths. Both will be needed for
the next pass.
