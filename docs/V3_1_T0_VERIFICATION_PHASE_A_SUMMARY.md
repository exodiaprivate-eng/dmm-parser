# Phase A Summary — T0 Verification of 4 Schema-Missing Tables

The 449 Table Catalog's "T0-V (verified against NattKh)" count was 109
of 118 T0 tables. The remaining 9 split into:
- 5 with zero extractable rust fields (no struct/all placeholders) —
  round-trip identical, no verification needed
- 4 NOT in NattKh's schema — this Phase A covers them

**Loop iters 1-4 (2026-05-11)** — IDA cross-referenced each table's
rust struct against the in-binary metaobject (when present) at the
live Win 1.06 binary (`C:\Program Files (x86)\Steam\steamapps\common\
Crimson Desert\bin64\CrimsonDesert.exe`).

## Results

| Table | Status | Top-level | Nested | Notes |
|---|---|---|---|---|
| `faction_waypoint_info` | **T0-V FULL** | 4/4 ✓ | 3/3 ✓ | Cleanest hit — all 7 fields verified |
| `house_info` | **T0-V top-level** | 6/6 ✓ | 0/3 | Renamed `HouseRegionPhase` → `HouseRegionData` |
| `mercenary_group_info` | T0-V partial | 3/4 named-CArrays ✓ | n/a | `_allowOperationTypeList` maps to one of `mercenary_key_list`/`mercenarye_info_list` (disambiguation needs decompile) |
| `equip_slot_info` | **T0-S structural** | 0 (no metaobject) | 0 | Parser is hand-rolled; no reflection table exists in binary |

## Key findings per table

### `faction_waypoint_info` (cleanest)
Metaobject at 0x144af0d80+. All 7 fields probed via `data_read_string`
on gap addresses between class-name strings. Every rust field name
exactly matches its canonical PA name with snake_case ↔ camelCase
mechanical translation (`way_point_data` ↔ `_wayPointData`, etc.).

Naming quirk: rust struct named `FactionWaypointInfo` (one word);
canonical is `FactionWayPointInfo` (two words / camelCase). Same data.

### `house_info` (full top-level + rename)
Metaobject at 0x144afbcd0+. All 6 top-level fields verified canonical
via the same gap-probing technique. Plus the nested `HouseRegionPhase`
struct was renamed to `HouseRegionData` (the actual canonical class
name found at 0x144afbf20+). The 3 nested-struct fields (`phase_id`,
`region_hash`, `texture_path`) couldn't be probed because the
HouseRegionData metaobject uses a pointer-table layout — needs
decompile of the nested reader.

### `mercenary_group_info` (3/4 partial)
Metaobject at 0x144b0a300+. 3 canonical names verified:
- `is_blocked` ↔ `_isBlocked`
- `parent_mercenary_group_info` ↔ `_parentMercenaryGroupInfo`
- `child_mercenary_group_info_list` ↔ `_childMercenaryGroupInfoList`

A 4th canonical (`_allowOperationTypeList` at 0x144b0a315) maps to
one of the two CArray rust fields (`mercenary_key_list` or
`mercenarye_info_list` — note the typo on the latter). Disambiguation
needs decompile of the parser to see field order.

### `equip_slot_info` (T0-S only)
**No metaobject in the binary.** The parser is hand-rolled
(`sub_141048F10` + `sub_141048B40`) without reflection registration.
None of `_etlHashes`, `_categoryA`, `_slotIndex`, `_equipTypeInfoList`,
`_equipInfoData`, `_equipTypeInfoKey` exist as strings in the binary.

The current rust field names (`etl_hashes`, `category_a`, `category_b`,
`name_hash`, `slot_index`, `complex_blob`, `tail_magic`, etc.) are the
dmm-parser team's semantic interpretations from decompile analysis.
SEMANTICS are sound (proven by round-trip + documented `etl_hashes`
mod use cases). NAMES are not canonically verifiable without a different
evidence source (PS5 demo binary, leaked SDK headers).

## Catalog impact

| Status | Before | After Phase A |
|---|---|---|
| T0-V (schema-verified) | 109 | **111** (faction_waypoint_info + house_info) |
| T0-V partial | — | 1 (mercenary_group_info) |
| T0-S structural-only | — | 1 (equip_slot_info) |
| T0 mechanical-only | 9 of 118 | 5 of 118 (4 graduated) |

## What this means for mod authors

All 4 tables already parsed + round-trip byte-perfect on the live
1.06 install before Phase A. **No functional changes** to mod-author
workflows — every field accessible by name today is still accessible.

What CHANGED: the rust field names are now documented as
IDA-verified canonical (for faction_waypoint_info + house_info) or
explicitly flagged as structural-only (for equip_slot_info), letting
mod authors trust they're using the right names for `etl_hashes`
unlock mods etc.

The verification tables in each module's docstring (`src/tables/<name>/info.rs`)
are the authoritative reference per-table.

## Next: Phase B continued game breakdown

With Phase A done, the loop continues into deeper-decode work on
formats with TBD per-field semantics:
- `mercenaryinfo` `_unk_106_*` semantic field names (the 1.06 6-byte
  addition)
- `.motionblending` per-tag value byte layout
- `.paccd` per-slider semantic mapping
- `.hkx` Havok class registry (long-haul)
- Long-tail formats `.questgaugecount`, `.pathc`, `.pai`, `.paproj`

See `_T0_VERIFICATION_WORKPLAN.md` "Phase B" for the queued list.
