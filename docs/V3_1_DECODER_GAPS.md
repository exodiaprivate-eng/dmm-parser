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
| `royal_supply_info` | `_royalSupplyRandomMap` | 2× unrolled fields `royal_supply_random_map_quest`, `royal_supply_random_map_mission` (semantic split: quest vs mission lookup) |

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

### Master typeinfo + record-reader registry (iter 47)

Per-table Win-IDA pointers for future decoder-gap closure work. Each row
gives the typeinfo string address (in CrimsonDesert.exe .rdata) and the
actual per-record reader function (typically a single xref to the typeinfo).

| Table | Typeinfo addr | Record reader | Reader size | Validated |
|---|---|---|---|---|
| `tribe_info` | `0x144af6090` | `sub_1410C8A20` | 0x45d | ✅ iter 42 |
| `interaction_info` | `0x144ac4060` | `sub_1410AC290` | 0x586 | ✅ iter 45 |
| `mission_info` | `0x144add820` | `sub_1410B9BA0` | 0x6ea | ✅ iter 46 |
| `knowledge_info` | `0x144ac9fb0` | `sub_1410AFE20` | 0x57a | ✅ iter 47 |
| `gimmick_info` | (special — no bare typeinfo string; Tier-1.5 typed prefix + opaque blob) | `sub_1410E6FC0` | 7205B | per existing dmm-parser docstring |
| `character_info` | `0x144ab3c20` | `sub_1410A3A40` | 0x2238 (8.7KB — largest) | iter 48 |
| `stage_info` | `0x144af38e0` | `sub_1410C76E0` | 0xd90 (3.5KB) | iter 48 |
| `gimmick_group_info` | `0x144accb90` | `sub_1410B0D60` | 0xadd (2.8KB) | iter 49 |
| `field_info` | `0x144ac6a60` | `sub_1410AD060` | 0x4dd (1.2KB) | iter 49 |
| `faction_node_info` | `0x144ac1af0` | `sub_1410AAE90` | 0x5df (1.5KB) | iter 50 |
| `region_info` | `0x144aeb3b0` | `sub_1410C1E70` | 0x403 (1.0KB) | iter 50 |
| `vehicle_info` | `0x144afb160` | `sub_1410CB230` | 0x530 (1.3KB) | iter 51 |
| `action_point_info` | `0x144ab0380` | `sub_1410A16D0` | 0xf7 (247B — unusually small; possibly wrapper around real reader) | iter 51 |
| `global_game_event_info` | `0x144ace140` | `sub_1410B2150` | 0x132 (306B — small, only 8 fields) | iter 52 |
| `global_stage_sequencer_info` | `0x144ad5310` | `sub_1410B54A0` | 0x213 (531B) | iter 52 |
| `faction_node_spawn_info` | `0x144ac1800` | `sub_1410AB470` | 0x18d (397B) | iter 53 |
| `faction_relation_group_info` | `0x144ac1490` | `sub_1410E7CE0` | 0x318 (792B) | iter 53 |
| `character_change_info` | `0x144ab2bf0` | `sub_1410A2F10` | 0x16e (366B) | iter 54 |
| `detect_reaction_info` | `0x144aba730` | `sub_1410A7170` | 0x1e6 (486B) | iter 54 |
| `equip_info` | `0x144abc440` | `sub_1410A76F0` | 0x130 (304B) | iter 55 |
| `equip_type_info` | `0x144abdfc0` | `sub_1410A9500` | 0x32b (811B) | iter 55 |
| `royal_supply_info` | `0x144aedf00` | `sub_1410C3220` | 0x12b (299B) | iter 56 |
| `sub_level_info` | `0x144af8a10` | `sub_1410C9FF0` | 0x53f (1.3KB) | iter 56 |
| `multi_change_info` | `0x144adf1a0` | `sub_1410BA4C0` | 0x387 (903B) | iter 57 |
| `ally_group_info` | `0x144ab1670` | `sub_1410A21A0` | 0x380 (896B) | iter 57 |
| `elemental_material_info` | `0x144abe4d0` | `sub_1410A8FA0` | 0x55d (1.4KB) | iter 58 |
| `frame_event_attr_group_info` | `0x144ac80c0` | `sub_1410ADF20` | 0x223 (547B) | iter 58 |
| `game_event_handler_info` | `0x144ac7920` | `sub_1410AE5C0` | 0x1cd (461B) | iter 59 |
| `item_use_info` | `0x144ad8db8` | `sub_1410B7380` | 0x176 (374B) | iter 59 |
| `level_gimmick_scene_object_info` | `0x144ad97e0` | `sub_1410B7EB0` | 0x50d (1.3KB) | iter 60 |
| `special_mode_info` | `0x144aee2c0` | `sub_1410C2E30` | 0x3e4 (996B) | iter 60 |
| `terrain_region_auto_spawn_info` | `0x144af40f0` | `sub_1410C7300` | 0x3d7 (983B) | iter 61 |

**🎯 Registry COMPLETE (2026-05-10 iter 61):** all 27 gap tables + 4 fully-validated tables + 1 special case (gimmick_info) = **31 entries**. Every table with v3.1 decoder gaps has its typeinfo + record reader address mapped. Future closure sessions skip typeinfo discovery entirely — straight to decompile.

**Workflow:** for any future per-table decoder closure session:

1. Look up the table here for the record reader address
2. `mcp__ida-pro-mcp__decompile_function(record_reader_addr)`
3. Walk wire-reads in source order
4. Cross-reference against the table's canonical-field catalog (already
   in `src/tables/<table>/info.rs` top doc-comment from iter 10/11/24)
5. Pair each `*(TYPE *)(this+OFFSET) = read_X(...)` with the canonical
   in schema declaration order
6. Rename `unk_XX` rust fields → canonical_snake
7. Add v3.1 alias entries via MANUAL_OVERRIDES if mechanical translation
   doesn't match (iter 31-34 patterns)
8. cargo build + test (must keep 562 passing)

**Not-yet-registered** (remaining gap tables — future iters can extend):

character_info, stage_info, gimmick_group_info, field_info,
faction_node_info, region_info, global_game_event_info,
global_stage_sequencer_info, action_point_info, vehicle_info,
faction_node_spawn_info, faction_relation_group_info,
character_change_info, detect_reaction_info, equip_info, equip_type_info,
royal_supply_info, sub_level_info, multi_change_info, ally_group_info,
elemental_material_info, frame_event_attr_group_info,
game_event_handler_info, item_use_info, level_gimmick_scene_object_info,
special_mode_info, terrain_region_auto_spawn_info.

### mission_info closure validation (iter 46)

Located `pa::MissionInfo` typeinfo at `0x144add820` (single xref to
record reader) → `sub_1410B9BA0` (size 0x6ea = 1.7KB, larger than
interaction_info — 40 fields total).

Schema breakdown (40 fields total):

| Type | Count |
|---|---|
| `direct_15B` (15-byte packed bool/enum) | 14 |
| `reader_4B` | 10 |
| `?` (unknown) | 7 |
| `reader_8B` | 4 |
| `direct_u16` | 3 |
| `direct_u32` | 1 |
| `reader_1B` | 1 |

Workflow-ready. Per-field semantic naming deferred to dedicated
decoder-writing session.

### interaction_info closure validation (iter 45)

Located `pa::InteractionInfo` typeinfo at `0x144ac4060`, single xref to
`sub_1410AC290` (size 0x586 = 1.4KB) — the actual record reader.

Schema (NattKh) type counts: **37 total fields**

| Type | Count |
|---|---|
| `direct_u8` | 22 |
| `?` (unknown) | 6 |
| `reader_4B` | 5 |
| `direct_u32` | 1 |
| `array_or_complex` | 1 |
| `reader_8B` | 1 |
| `reader_2B` | 1 |

Wire reader observation: dense nested chain of small reads (mostly
1-byte single calls + several sub_xxx polymorphic decoders for the
reader_NB / array_or_complex fields). Pattern matches the type tally;
per-field semantic naming requires careful per-position analysis with
HexRaysPyTools struct rebuild + game-data inspection.

Closure path documented; status: workflow-ready. Per-field naming is
the multi-hour focused work, deferred to a dedicated decoder-writing
session with the IDA plugins active.

### tribe_info closure validation (iter 42)

Decompiled `pa::TribeInfo` record reader at `sub_1410C8A20` (Win
typeinfo at .rdata `0x144af6090`, single xref). Wire-read sequence
extracted in source order:

```
offset  size       wire-call
   0    4 bytes    *(_a1+8)(...) → _key (u32)
   8    CString    sub_141076050 → _stringKey
  16    1 byte     direct → _isBlocked
  18    sub-call   sub_1410CD790 → already-aliased lookup_a
  20    sub-call   sub_1410CBB90 → already-aliased lookup_b
  22    1 byte     direct (1× u8)
  24    4 bytes    direct (1× u32)
  28-36 9× 1 byte  direct (9× u8)
  40    4 bytes    direct (1× u32)
  44    4 bytes    direct (1× u32)
  48    4 bytes    direct (1× u32)
  56    CString    sub_141076050 → CString
  64    4 bytes    direct (1× u32)
  68    4 bytes    direct (1× u32)
  72    4 bytes    direct (1× u32)
  76    4 bytes    direct (1× u32)
  80    1 byte     direct (1× u8)
  81    1 byte     direct (1× u8)
  84    4 bytes    direct (1× u32)
  88    8 bytes    direct (1× u64)
  96    sub-call   sub_1410CCE80 → ref_list (CArray)
```

**Type-count cross-validation against NattKh schema:**

| Type | Wire reads | Schema canonicals (excluding aliased) | Match |
|---|---|---|---|
| `direct_u8` | 13 | 13 (_tribeMassLevel, _wantedCrimeType, _interactionUIDistanceLv, _ignoreWaterFall, _isBird, _isHumanoid, _hasChild, _isDeathByDrowning, _detourOnRoad, _detectModeShowEnemy, _escapePlatform, _ignoreOverlapPush, …) | ✅ |
| `direct_u32` | 9 | 9 (_bumpTypeHash, _footMaterialKey, _characterPauseType, _detourMaxDegree, _velocityDampSpeed, _activityWaterDepth, _weaponMaterialKey, _armorMaterialKey, _baseMaterialKey) | ✅ |
| `reader_4B` (sub-call CString-like) | 4 | 4 (_key, _footStepTypeEffectName, _tamedSkillList, _ignoredReactionInSafeZoneFlag) | ✅ |
| `direct_u64` | 1 | 1 (`?` type, likely `_parentTribeInfo` or `_tribeNameForEditor`) | ✅ |

**Conclusion:** the closure workflow works structurally. Counts match
perfectly. Per-field semantic naming (which `unk_XX` becomes which
canonical) requires deliberate per-position review against game data —
e.g. bool field at offset 22 could be any of the 13 bool canonicals.
This is hours of focused work that's safer with IDA plugins (per the
T0_AUDIT_TRACKING plugin reference) installed first.

Cross-table-typed-count validation now standardised as the smoke test
for "is this gap table workflow-ready" assessment.

### Per-table closure plan: `tribe_info` (iter 41)

26 missing canonicals; rust struct has 26 `unk_XX` placeholder fields
named by byte offset:

```rust
pub struct TribeInfo<'a> {
    pub key, string_key, is_blocked, lookup_a, lookup_b,         // already aliased
    pub unk_22, unk_24, unk_28, unk_29, unk_30, unk_31,           // placeholders by wire offset
    pub unk_32, unk_33, unk_34, unk_35, unk_36, unk_40,
    pub unk_44, unk_48, unk_56, unk_64, unk_68, unk_72,
    pub unk_76, unk_80, unk_81, unk_84, unk_88,
    pub ref_list,
}
```

26 missing canonicals (in NattKh schema declaration order — likely
matches wire order):

```
_activityWaterDepth, _armorMaterialKey, _baseMaterialKey, _bumpTypeHash,
_characterPauseType, _detectModeShowEnemy, _detourMaxDegree, _detourOnRoad,
_escapePlatform, _footMaterialKey, _footStepTypeEffectName, _hasChild,
_ignoreOverlapPush, _ignoreWaterFall, _ignoredReactionInSafeZoneFlag,
_interactionUIDistanceLv, _isBird, _isDeathByDrowning, _isHumanoid,
_parentTribeInfo, _tamedSkillList, _tribeMassLevel, _tribeNameForEditor,
_velocityDampSpeed, _wantedCrimeType, _weaponMaterialKey
```

**Closure path (multi-iter, hours of IDA work):**
1. Find `pa::TribeInfo` typeinfo in IDA strings
2. Get xref → real record reader (per iter 30 corrected workflow)
3. Decompile reader; walk reads in order
4. Pair each `*(TYPE *)(this+OFFSET) = read_X(...)` with the next
   canonical in schema order. The offsets in the unk_XX names
   (22, 24, 28, ...) should match the OFFSET literals in the decompile.
5. Rename `unk_XX` → canonical_snake (e.g. `unk_22` → `activity_water_depth`)
6. Add v3.1 alias entries
7. cargo build + test (must keep 562 passing)

This is the template for the 7 other GENUINE-pattern tables. The
wrapper-pattern tables follow a different template (Rust struct
refactor to collapse unrolled fields into typed sub-structs).

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

### Wrap pattern can also hide INSIDE typed sub-structs (iter 43)

`action_point_info` was originally placed in GENUINE pattern because
top-level rust_count (6) == schema_count (6). But the rust struct uses
nested typed sub-structs:

```rust
pub struct ActionPointInfo<'a> {
    pub key, string_key, is_blocked,
    pub action_point: ActionPoint,        // ← typed sub-struct
    pub level_action_point_info: u32,
    pub action_point_b: ActionPoint,      // ← same sub-struct again
}

pub struct ActionPoint {
    pub field_a: u32,                     // = canonical _actionYaw (direct_u32)
    pub block_a: [u8; 24],                // first 12 bytes = canonical _actionPosition (direct_12B)
                                          // last 12 bytes = extra wire data not in schema
}
```

Both missing canonicals (`_actionPosition`, `_actionYaw`) are fields
INSIDE the `ActionPoint` sub-struct. The top-level count match was
misleading.

**Implication:** iter 40's WRAP-vs-GENUINE quantitative split may
**undercount** wrap-pattern tables. Other tables classified as GENUINE
(global_stage_sequencer_info 14=14, knowledge_info 30=30,
global_game_event_info 8 vs 5, etc.) may have similar nested-sub-struct
hidden wraps. A more accurate classifier would walk into nested types,
not just count top-level fields.

For the headline: realistic decoder-writing workload is likely closer
to **~150-250 fields** (further reduced from the iter 40 estimate of
~200-300 once nested wraps are accounted for).



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
