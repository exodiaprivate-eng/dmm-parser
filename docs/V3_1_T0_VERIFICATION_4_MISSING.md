# T0 Verification — 4 Tables Missing from NattKh's Schema

The 449 Table Catalog reports 109 of 118 T0 tables as
"schema-verified" against NattKh's `pabgb_complete_schema.json`.
The remaining 9 are split into:

- **5 tables** with zero extractable rust fields (regex couldn't find
  a main struct or all fields are placeholders) — round-trip identical
  under shape="v3" and shape="v3.1" alike, no verification needed
- **4 tables NOT in NattKh's schema**:
  - `equip_slot_info`
  - `faction_waypoint_info`
  - `house_info`
  - `mercenary_group_info`

Per Session 28 these 4 fall back to mechanical translation. This doc
captures the **IDA-direct verification** done against the live 1.06
binary (`CrimsonDesert.exe` SHA256 `50f9d4e7...`) to graduate them
from T0 (mechanical) → T0-V (verified).

## Methodology

For each table:
1. Searched IDA strings for the class name + `_camelCase` field name
   patterns
2. Read raw strings at metaobject addresses (the engine's runtime
   reflection table — pointer arrays of `(class_name_str,
   field_name_str)` pairs)
3. Cross-referenced against the rust struct field names in
   `src/tables/<name>/info.rs`

## Findings

### `mercenary_group_info`

Verified canonical names (from IDA strings):
- `_isBlocked` → rust `is_blocked` ✓
- `_parentMercenaryGroupInfo` → rust `parent_mercenary_group_info` ✓
- `_childMercenaryGroupInfoList` → rust `child_mercenary_group_info_list` ✓

Extra canonical names found in binary (not currently mapped to rust
fields — TBD which rust field they correspond to):
- `_allowOperationTypeList`
- `_hideMercenaryGroupInfoList`

Rust fields whose canonical names are NOT yet verified:
- `mercenary_key_list` → likely `_mercenaryKeyList`
- `mercenarye_info_list` → typo in current code; likely `_mercenaryInfoList`

Status: **T0-V partial** (3 of ~7 fields verified)

### `house_info`

Verified canonical names:
- `_houseRegionDataList` → rust `house_region_data_list` ✓
- `_houseInfo` (address 0x144b09646; container/parent reference)

Status: **T0-V partial** (1 high-confidence match — only one field
in the rust struct that wasn't already a primitive like `key`/`is_blocked`)

### `faction_waypoint_info`

Class metaobject found at 0x144af0d80+ (multiple `FactionWayPointInfo`
class-name entries). Field-name strings in the metaobject gaps could
not be read via `data_read_string` (likely indirected through pointer
table). Verification requires decompiling the registrar function.

Status: **TBD** — needs IDA decompile pass on the parser registrar

### `equip_slot_info`

Related strings found:
- `_equipSlotName`
- `_equipSlotNameIndex`
- `_equipSlotNameString`
- `EquipSlotName` (class)
- `EquipSlotNo` (related class)

Current rust struct uses placeholder names (`category_a`, `category_b`,
`etl_hashes`, `complex_blob`, `tail_magic`). The canonical names above
strongly suggest the fields concern slot-name addressing rather than
"categories" — but mapping each placeholder to its canonical equivalent
needs the parser's decompiled C++ to confirm field order.

Status: **needs IDA decompile + rename pass**

## Summary

| Table | Verified | Notes |
|---|---|---|
| `mercenary_group_info` | 3/7 fields | Rest need decompile |
| `house_info` | 1/1 named field | Effectively complete |
| `faction_waypoint_info` | 0 | Metaobject found but fields not yet readable |
| `equip_slot_info` | 0 | Rust uses placeholders; needs full rename pass |

## Bottom line

The 4 schema-missing tables are **functionally fine**:
- All 4 round-trip byte-perfectly on the live 1.06 install
- All 4 have rust structs that read every byte
- Only `equip_slot_info` has obviously-wrong-feeling field names
  (placeholders that look guessed)

The verification gap in NattKh's schema doesn't break anything for
mod authors — it just means the v3.1 _camelCase aliases (when present)
are mechanically translated rather than canonically verified. The
IDA evidence above moves 2 of the 4 closer to T0-V; full verification
for all 4 needs a decompile pass on the parse registrars.

This document is the entry point for that future work.
