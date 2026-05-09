# v3.1 Decoder Gaps — Per-Table Worklist

Tables where dmm-parser's struct definition is missing fields that **do**
exist in the canonical Pearl Abyss schema (NattKh's `pabgb_complete_schema.json`,
extracted from Korean error strings in `CrimsonDesert.exe`).

These are not parser bugs — every shipped field still round-trips correctly.
They're **completeness gaps**: real game-data fields the struct doesn't
currently expose, so mods can't read or write them.

The schema for each missing field includes the byte-offset string address,
reader-function pointer, and stream/type category — enough to write the
field decode without IDA work.

## Summary

- **Tables fully decoded (0 gaps)**: 68 of 109 (62%)
- **Tables with gaps**: 41
- **Total missing fields**: 584

To regenerate this report, run `python scripts/verify_v3_1_against_schema.py`
which writes the canonical JSON to `docs/v3_1_schema_verification.json`.

## Top decoder-gap tables (worklist priority)

| Rank | Table | Missing | Aliases | Schema | % done |
|---|---|---|---|---|---|
| 1 | `gimmick_info` | 153 | 6 | 159 | 4% |
| 2 | `character_info` | 146 | 18 | 164 | 11% |
| 3 | `stage_info` | 72 | 10 | 82 | 12% |
| 4 | `gimmick_group_info` | 45 | 25 | 70 | 36% |
| 5 | `interaction_info` | 29 | 8 | 37 | 22% |
| 6 | `tribe_info` | 26 | 3 | 29 | 10% |
| 7 | `mission_info` | 25 | 15 | 40 | 38% |
| 8 | `field_info` | 22 | 2 | 24 | 8% |
| 9 | `faction_node_info` | 15 | 16 | 31 | 52% |
| 10 | `region_info` | 4 | 19 | 23 | 83% |
| 11 | `global_game_event_info` | 4 | 4 | 8 | 50% |
| 12 | `knowledge_info` | 3 | 27 | 30 | 90% |
| 13 | `vehicle_info` | 3 | 18 | 21 | 86% |
| 14 | `action_point_info` | 2 | 4 | 6 | 67% |
| 15 | `effect_info` | 2 | 6 | 8 | 75% |

## Recommended priority order

**High impact (top 4 tables = 416 missing fields, 71% of total gap):**
- `gimmick_info` (153) — physics/collision/break/buoyancy/battery sub-systems
- `character_info` (146) — AI/equipment/regions/rewards/levels
- `stage_info` (72) — stage geometry/spawn data
- `gimmick_group_info` (45) — group orchestration metadata

**Quick wins (≤5 missing fields each, can polish off in a single session):**
- `region_info` (4), `global_game_event_info` (4), `knowledge_info` (3),
  `vehicle_info` (3), `action_point_info` (2), `effect_info` (2)

**Mid tier (6–30 missing):**
- `mission_info` (25), `tribe_info` (26), `interaction_info` (29),
  `field_info` (22), `faction_node_info` (15)

## Per-field detail

Full per-table missing-field lists live in
`docs/V3_1_SCHEMA_VERIFICATION.md` (auto-generated, regenerated whenever
`scripts/verify_v3_1_against_schema.py` runs). Look up canonical PA names
+ byte-offset string addresses + parser function pointers in
`_research_cache/pabgb_complete_schema.json` — every entry is shaped
`{f: canonical_name, s: string_addr, r: reader_fn, fn: parser_fn,
stream: byte_width, type: type_category}`.

## Notes on the 17 tables not in the schema

Mechanical fallback covers `equip_slot_info`, `faction_waypoint_info`,
`house_info`, `mercenary_group_info` plus 13 zero-field/file-format
tables (`paac`, `paatt`, `pamhc`, `pappt`, etc.). These need separate
verification work — likely via pycrimson (reflection-format
self-describing) or a future Korean-error scrape extension.

## Schema source

NattKh/CrimsonDesertModdingTools — `schemas/pabgb_complete_schema.json`
https://github.com/NattKh/CrimsonDesertModdingTools

Methodology: `parsers/pabgb_schema_dumper.py` in the same repo. Greps
`CrimsonDesert.exe` for the literal Korean error template
`"<TABLE>의 <_FIELD>를 읽어들이는데 실패했다."` ("failed to read
TABLE's _FIELD") and extracts `(class, field)` pairs.
