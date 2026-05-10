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

## Decoder-fn cluster analysis (2026-05-10 iter 27)

For each gap table, every missing field routes through a SINGLE parser
function. This means **one IDA decompile per table unlocks all of that
table's gaps at once** — no per-field reverse-engineering needed.

| Table | Missing | Parser fn | Type distribution (top 3) |
|---|---|---|---|
| `gimmick_info` | 153 | `0x141046310` | direct_13B:58, ?:46, direct_u32:23 |
| `character_info` | 146 | `0x141037900` | direct_u8:57, reader_4B:31, ?:24 |
| `stage_info` | 72 | `0x141059e90` | reader_4B:24, direct_u8:22, direct_u32:12 |
| `gimmick_group_info` | 45 | `0x141042970` | direct_u8:36, ?:6, direct_u32:1 |
| `interaction_info` | 29 | `0x141043c40` | direct_u8:17, reader_4B:5, ?:4 |
| `tribe_info` | 26 | `0x14105aef0` | direct_u8:12, direct_u32:9, reader_4B:3 |
| `mission_info` | 25 | `0x14104ca30` | direct_15B:13, reader_8B:4, reader_4B:3 |
| `field_info` | 22 | `0x1410403f0` | direct_u8:12, ?:3, reader_4B:3 |
| `faction_node_info` | 15 | `0x14103e930` | ?:5, reader_4B:4, direct_15B:3 |
| `region_info` | 4 | `0x141053790` | direct_u8:2, direct_u32:1, reader_4B:1 |
| `global_game_event_info` | 4 | `0x141044b80` | reader_8B:1, reader_4B:1, ?:1 |
| `knowledge_info` | 3 | `0x14104a4c0` | direct_15B:2, reader_4B:1 |
| `vehicle_info` | 3 | `0x14105d470` | direct_u32:2, direct_15B:1 |
| `action_point_info` | 2 | `0x141035530` | direct_u32:1, direct_12B:1 |
| `effect_info` | 2 | `0x14103c140` | ?:1, direct_12B:1 |

### What this means for implementation

**Cross-table function reuse is zero** — each .pabgb table has its own
dedicated parser function in CrimsonDesert.exe. So gap-closing work is
**per-table**, not per-field.

**Per-table workflow:**
1. Decompile the parser function (Win-IDA, e.g. `decompile_function 0x141046310` for gimmick_info)
2. Walk reads in source order — each `*(TYPE *)(this + OFFSET) = read_X(...)` corresponds to one field
3. Cross-reference field order against the canonical-name list (already in `info.rs` doc-comment from iter 10/11)
4. Add typed Rust struct fields preserving the wire order
5. Cargo build + test (must keep 562 passing baseline)

**Quick-win target order (smallest first, easier first decompiles):**
- effect_info (2) and action_point_info (2) — single-iter quick wins
- vehicle_info (3), knowledge_info (3) — also quick
- global_game_event_info (4), region_info (4) — small gaps
- faction_node_info (15), field_info (22) — mid-tier
- larger ones (gimmick_info 153, character_info 146) — multi-iter; may benefit from splitting the decompile across multiple iters

**Type distribution intuition:**
- `direct_u8` / `direct_u32` / `direct_u64` = scalar reads (easiest)
- `direct_NB` (e.g. `direct_15B`) = packed bool sequences (each bit = one field)
- `reader_4B` / `reader_8B` = lookup-via-table reader functions (need to follow the `r` pointer in schema for the reader fn)
- `array_or_complex` = nested struct or list (heaviest)
- `?` = type unknown to the schema dumper (needs IDA to determine)

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
