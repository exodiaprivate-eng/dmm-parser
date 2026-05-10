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

## Known structural divergences (2026-05-10 iter 36)

Cases where the schema's single canonical name maps to **multiple
rust struct fields**, so a 1-to-1 alias entry can't express the mapping:

### Numbered-suffix unrolling (1 canonical → N+ numbered fields)

| Table | Schema canonical | Rust struct |
|---|---|---|
| `ally_group_info` | `_relationTypeList` | 7× unrolled fields `relation_type_list_0` .. `relation_type_list_6` |
| `elemental_material_info` | `_flag` | 8× unrolled fields `flag_0` .. `flag_7` |

### Wrapper-vs-unrolled (1 canonical → N named sub-fields)

Spot-check survey of 10 small-gap tables (iter 39) found 8 fit this
pattern: NattKh schema names a single top-level "_thingFilter" or
"_xxxData" canonical, while dmm-parser's rust struct unrolls its
internal sub-struct fields. The "1 missing canonical" is actually the
WRAPPER name for the 2-3 extra rust fields.

| Table | Schema canonical (1 missing) | Likely rust unrolled sub-fields |
|---|---|---|
| `character_change_info` | `_characterChangeFilter` | `name_list`, `hash_lookup_list`, `trailing_id` (3 extra rust fields) |
| `detect_reaction_info` | `_reactionTable` | 4 extra rust fields |
| `royal_supply_info` | `_royalSupplyRandomMap` | 1 extra rust field |
| `sub_level_info` | `_exp` | 3 extra rust fields |
| `faction_node_spawn_info` | (1 missing) | 1 extra rust field |
| `faction_relation_group_info` | (1 missing) | 3 extra rust fields |
| `multi_change_info` | (1 missing) | 1 extra rust field |
| `equip_type_info` | (1 missing) | 3 extra rust fields |

**Implication for the 557 number:** many of those gaps are wrapper-vs-
unrolled pattern, not truly missing fields needing new Rust code. The
real decoder-writing workload is significantly smaller. Closing these
needs either:
- v3.1 alias mechanism extension to express 1-to-N nested-field aliases, OR
- Rust struct refactor to use a single typed sub-struct matching the canonical wrapper.

### Full quantitative split (iter 40 — all 27 gap tables)

Programmatic wrap-vs-genuine classification by comparing schema-field
count to rust-struct field count per gap table:

| Pattern | Tables | Total missing fields |
|---|---|---|
| **WRAP** (rust > schema, missing partly explainable as wrapper-vs-unrolled) | 19 | ~452 |
| **GENUINE** (rust ≤ schema, missing genuinely needs new Rust struct fields) | 8 | ~105 |

**Genuine-gap tables (need actual decoder writing):**

| Table | Missing | Rust struct shape |
|---|---|---|
| `gimmick_info` | 153 | Tier-1.5 typed-prefix + opaque blob (rust=7, schema=159) |
| `interaction_info` | 28 | Compact rust struct (rust=10, schema=37) |
| `tribe_info` | 26 | Even split (rust=29, schema=29) but several name divergences |
| `mission_info` | 25 | Even split (rust=40, schema=40) — divergent names |
| `global_game_event_info` | 3 | rust=5, schema=8 |
| `knowledge_info` | 1 | rust=30, schema=30 (1 unique addition) |
| `global_stage_sequencer_info` | 2 | rust=14, schema=14 |
| `action_point_info` | 2 | rust=6, schema=6 |

**Wrap-pattern tables (need alias mechanism extension or Rust refactor):**

19 tables totaling ~452 missing canonicals where rust struct has more
fields than the schema headcount. The schema "missing" here is mostly
a single wrapper name covering 2-30 unrolled rust fields. character_info
alone has 27 rust-excess fields, stage_info has 13, gimmick_group_info
has 2, vehicle_info has 17 (probably the densest case — 17 rust fields
that could collapse into a small handful of canonical wrappers).

**Honest conclusion:** the real decoder-writing workload is likely
~200-300 fields max (the genuine-pattern tables + the residual genuine
gaps within wrap-pattern tables), not 557. Rust struct refactoring
(collapsing unrolled fields into typed sub-structs) closes the
remainder mechanically.



These need either (a) a Rust struct refactor to use a single `CArray`
or fixed-array field matching the canonical, or (b) extending the v3.1
alias mechanism to support 1-to-N mappings. Neither is in-scope for the
loop; documented for future work.

## Auto-closure analysis (2026-05-10 iters 31-35)

Iters 31-34 captured every name-divergence closure reachable via
script-based heuristics: 27 fields closed across 23 unique tables.
Aggregate: 1125 → 1152 verified, 584 → 557 missing.

Iter 35 surveyed the remaining 557 missing fields against unaliased
rust struct fields per-table. Result:

| Category | Count | Implication |
|---|---|---|
| **Rust placeholders** (`lookup_NN`, `field_X`, `raw_Y`, `_unkN`) | 272 | Need IDA wire-position trace to map placeholder ↔ canonical |
| **Real-named unaliased rust** | 208 | Likely actual decoder additions OR divergent canonical forms requiring per-field IDA verification |
| **Genuine missing** (no rust field) | 77 | Need new typed Rust struct fields per IDA decompile |

Per-table breakdown of unfinished work:

| Table | Missing | Placeholders | Real-unaliased |
|---|---|---|---|
| gimmick_info | 153 | 0 | 1 |
| character_info | 146 | 139 | 34 |
| stage_info | 72 | 58 | 27 |
| gimmick_group_info | 45 | 39 | 8 |
| interaction_info | 28 | 0 | 1 |
| tribe_info | 26 | 2 | 24 |
| mission_info | 25 | 16 | 9 |
| field_info | 22 | 7 | 18 |
| faction_node_info | 14 | 3 | 12 |
| (8 small tables) | ≤3 each | mostly 0 | varies |

**Heuristics reached their limit.** Further gap closures (the remaining
557 fields) all require IDA decompile of the per-table record reader
(found via `pa::<TableName>` typeinfo → vtable → read-from-bytes
virtual method per iter 30 corrected workflow). Each table is hours
of focused work; not 1-min-loop-amenable.

**6 PA-internal typos** were preserved as canonical aliases during the
auto-closure pass (kept verbatim per NattKh schema):
`_questGroupkey`, `_regionEnterknowledgeInfoList` (lowercase k),
`_fishSummonTimeFrquencyType` (missing 'e'),
`_radgollEquipTableGroupDataList` (radgoll vs ragdoll),
`_collectFilter_Dev` (mid-name underscore),
`_wayPointDataList_deprecated` (mid-name underscore).

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

**Important correction (iter 30):** The `fn` pointer in NattKh's schema
is the **generic table-loader** function, not the per-table record
parser. Decompile of `0x14103c140` (the supposed effect_info parser)
revealed `sub_14103BE80`, the shared loader that dispatches via vtable
to per-table record readers. So the per-table grouping above is correct
(each table routes through its own loader entry), but the actual
field-by-field record parser is one indirection deeper.

To find the **real** per-record parser for a table:
1. Locate the table's record class typeinfo in IDA (e.g. `pa::EffectInfo`)
2. Find its vtable
3. The "read from bytes" virtual method on the vtable is the real parser
4. THAT function has the per-field reads we want to cross-reference

This is significantly more involved than "just decompile the schema's
fn pointer". Per-table decoder gap closure remains feasible but is
**hours-per-table** of focused IDA work, not a 1-min loop iter.

**Cross-table function reuse is zero at the loader-entry level** —
each .pabgb table has its own dedicated loader entry pointer (the
schema fn). Per-table decoder structure also varies, so the actual
record-reading code is also per-table.

**Per-table workflow (corrected):**
1. Find `pa::<TableName>` typeinfo via IDA strings
2. Identify the read-from-bytes virtual method on the typeinfo's vtable
3. Decompile that method
4. Walk reads in source order — each `*(TYPE *)(this + OFFSET) = read_X(...)` corresponds to one field
5. Cross-reference field order against the canonical-name list (already in `info.rs` doc-comment from iter 10/11/24)
6. Add typed Rust struct fields preserving the wire order
7. Cargo build + test (must keep 562 passing baseline)

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
