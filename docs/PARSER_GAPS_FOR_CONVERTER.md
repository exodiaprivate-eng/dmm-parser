# dmm-parser — gaps that affect dmm-converter coverage

**Date:** 2026-05-11
**Audience:** dmm-parser maintainers
**Why this exists:** the dmm-converter is downstream of dmm-parser. Every
gap below blocks specific *converter* features automatically — closing
the parser gap unlocks the converter feature without any converter code
change. This doc lists each gap, its current status, the game data
needed (IDA decompile, vanilla dumps, etc.), and the converter feature
it unblocks.

> **Companion doc:** `CONVERTER_VS_PARSER_SCOPE.md` — the architectural
> contract that says what each crate owns.

---

## 2026-06-05 — DamageBuffData 1-byte fix (unblocks skill_info field intents)

**Symptom:** the Infinite Stamina mod's `use_resource_stat_list[0].d = 0`
intents were silently skipped for 56 skills at mount
(`[V3_OVERLAY] skill.pabgb → 330 applied, 56 unresolved`). Those skills
fell to blob-fallback because skill_info typed-parse failed on them.

**Root cause:** `DamageBuffDataPayload` (BuffData disc 0) was 116 bytes;
the real wire payload is **117**. A prior edit removed a trailing byte
(`f0c_new`) despite its own note saying the data was *"only consistent
with a 117-byte payload."* Every skill entry whose buff is DamageBuffData
(178 entries on 1.10) drifted by 1 byte after the buff and failed typed
parse → no field-level edit possible. Re-verified empirically: for all 31
single-buff disc-0 skill entries, `skill_group_key` (== the skill key)
lands at `buff_end + exactly 1`. Fix: restored the trailing `f0f: u8` in
`src/binary/variants/buff_data.rs`.

**Result:** skill_info typed-parse failures 212 → 7; all 56 stamina skills
now typed and editable. Also cleared **buff_info** (shares the same
DamageBuffData decoder). Validated byte-exact against the 1.10 vanilla
fixture (`tables::skill_info::info::tests::verify_stamina_keys_typed`).

**Remaining skill_info gap (7 entries, NOT stamina):** keys 41355, 41357,
41358, 65009, 65010, 75019, 91251 use **SummonBuffData (disc 10)**, whose
variant under-reads by a consistent **7 bytes** in *skill* context (the
`*_outer_*` trailing fields differ from buff_info's). SummonBuffData is
shared across tables, so this needs an IDA-anchored fix rather than a
blind append; these 7 safely blob-fallback (opaque passthrough, byte-exact
roundtrip) in the meantime. Tracked as a separate follow-up.

---

## D1. Tracked variants of hand-rolled Tier 1.5 tables — ✅ COMPLETE

**Status:** all 16 tables shipped 2026-05-11 (commit `fef7d85`).

**What was missing.** The 16 most-touched hand-rolled Tier 1.5 tables
had a `read_with_size` inherent method but no tracked equivalent. The
`pabgh_typed_blob_table!` macro can't auto-emit `BinaryReadTracked` for
Tier 1.5 because the trait signature has no `entry_size` parameter —
the tail boundary comes from outside (pabgh).

**What shipped.**

1. `tracked_p_typed!` macro in `src/tracked.rs` that calls each
   table's inherent `read_tracked_with_size(data, offset, entry_size,
   path, ranges)` directly, sidestepping the trait.
2. Two public helpers in `src/binary/mod.rs`:
   - `track_read_field<T: BinaryRead<'a>>(...)` — primitives + simple types
   - `track_read_with<T, F>(...)` — for fields with custom read logic
     (polymorphic variants, length-prefixed lists, recursive trees)

   Each helper pushes the field name onto `path`, captures the byte
   span before/after the read, registers one `FieldRange`, and pops.

3. Per-table `read_tracked_with_size` impls added for all 16 tables:
   character_info, character_change_info, inventory_info, buff_info,
   condition_info, drop_set_info, effect_info, gimmick_info,
   interaction_info, store_info, faction_node_spawn_info, quest_info,
   item_use_info, ai_dialog_string_info, frame_event_attr_group_info,
   stage_info.

**Validation.** `examples/smoke_test_tracked_16.rs` runs
`parse_table_tracked` against the vanilla dump for each table.

| Table | Records | Tracked ranges | Avg/record |
|---|---:|---:|---:|
| stage_info | 50,783 | 4,824,385 | 95 |
| gimmick_info | 12,648 | 88,536 | 7 |
| item_use_info | 6,811 | 34,055 | 5 |
| effect_info | 2,069 | 16,552 | 8 |
| faction_node_spawn_info | 1,158 | 8,106 | 7 |
| ai_dialog_string_info | 1,101 | 12,111 | 11 |
| quest_info | 934 | 32,690 | 35 |
| interaction_info | 370 | 3,700 | 10 |
| store_info | 292 | 6,132 | 21 |
| frame_event_attr_group_info | 2 | 8 | 4 |
| character_info, character_change_info, inventory_info, buff_info, condition_info, drop_set_info | varies | varies | varies |

**100% pass, 0 fail, 0 skip.** ~70k records, ~5M field ranges.

**Granularity.** Top-level fields only (no recursion into nested struct
types). That's enough for v2-byte-patch resolution: any patch whose
offset falls within a field's span resolves to a `(record_key,
field_name)` pair. Patches landing in `_tail_blob` (the opaque tail of
Tier 1.5 tables) automatically fall back to file_replacement in the
converter.

**Converter feature unblocked.** Full v2-byte-patch → v3.1 intent
conversion against every commonly-modded table. Previously bottlenecked
to iteminfo only.

---

## D2. Polymorphic family decoders — partially complete

Some v2 byte-patches land in polymorphic-variant body bytes. Mods
targeting `BuffData` / `ConditionData` / `EffectData` variants need the
per-variant byte-recipe so the converter can produce per-field intents
*inside* the variant body (not just at the parent record's level).

**Current status:**

| Family | Status | Variants | Tracked by parser task |
|---|---|---:|---|
| BuffData | ✅ shipped | 120 | #116 |
| ConditionData | ✅ shipped | 405 | #115 |
| BranchConditionData | ✅ shipped | 14 | #118 |
| GameEventHandler | ✅ shipped | — | #97 |
| GlobalGameEventExecuteData | ✅ shipped | — | #96 |
| TriggerEventHandler | ⏳ queued | — | #95 |
| ContentsLogicFunctionBase | 🔄 in-progress | — | #66 |
| EffectData | 🔄 in-progress | — | #66 |
| GameCondition family | ⏳ variant-byte-recipes pending | — | #107 |

**Game data needed.** IDA decompile of each variant's per-byte read
recipe. Mac symbols + Win IDB anchored against Korean error strings are
the tools.

**Converter feature unblocked.** Field-level intents *inside* variant
bodies. Without this, a byte-patch that lands inside a
`SummonBuffData`'s variant payload resolves to "variant_body[3]" (an
opaque-range fallback) rather than "duration_ms". The v3 intent still
works but loses semantic meaning across game version bumps.

---

## D3. Tier 2 → Tier 1 promotion — ongoing

Tier 2 tables ship as `[typed prefix][opaque tail]`. v3 intents can
edit the prefix but the tail stays opaque. Each promotion increases
field-level coverage on mods that touch that table.

**Active work per the dmm-parser task list:**
- ContentsLogicFunctionBase (task #66)
- EffectData (task #66)
- 449-table grind (tasks #61–#66)

**Game data needed.** IDA decompile of each table's full read
function. Vanilla dumps for round-trip validation already exist (task
#59 complete).

**Converter feature unblocked.** Increases the number of patches that
resolve to a structured field path rather than `_tail_blob` fallback.
No converter code change required — the converter auto-detects new
Tier 1 tables via `is_tracked_table()`.

---

## Files (parser) that the converter consumes

```
dmm-parser/
├── src/
│   ├── lib.rs                                ← pub mod tracked;
│   ├── tracked.rs                            ← parse_table_tracked, is_tracked_table
│   ├── dispatch.rs                           ← parse_table_to_json (for hybrid path)
│   ├── binary/
│   │   ├── mod.rs                            ← FieldRange, BinaryReadTracked,
│   │   │                                       track_read_field, track_read_with
│   │   ├── variant.rs                        ← pabgh_typed_blob_table! macro
│   │   └── variants/                         ← polymorphic family decoders
│   └── tables/<name>/info.rs                 ← per-table read/write + tracked
└── examples/
    └── smoke_test_tracked_16.rs              ← validation
```

The converter's only entry points into the parser are:

- `dmm_parser::tracked::parse_table_tracked` — for v2-byte-patch resolution
- `dmm_parser::tracked::is_tracked_table` — for converter dispatch decisions
- `dmm_parser::dispatch::parse_table_to_json` — for hybrid mod diffing
- Per-table `<Type>::write_from_json_dict` — for hybrid re-serialize on apply

Anything in the parser CLI that mimics these for "mod-shaped" input
should be considered a layering violation — see
`CONVERTER_VS_PARSER_SCOPE.md`.

---

## Recommended next milestones (parser-side)

Ordered by converter-coverage impact:

1. **TriggerEventHandler family decoder** (#95). Closes the
   `GameEventHandler` family. Mods touching trigger volumes
   (NPC_Instant, Skip-Loading) get field-level intents.
2. **GameCondition variant-byte-recipe fix** (#107). Currently
   `Decoded|Raw` fallback gives byte-perfect round-trip, but mods that
   tweak condition thresholds resolve to `_raw_blob` instead of
   `threshold`. Cosmetic, but raises field-level coverage 2-3%.
3. **EffectData / ContentsLogicFunctionBase promotion** (#66). Two of
   the last big polymorphic blockers. After these, the typed prefix
   covers the visual-effect mod surface.
4. **Continue the 449-table grind** (#61–#66). Each promotion is small
   ROI individually but compounds.

The converter is **not** the bottleneck on any of these — it auto-picks
up new tables via `is_tracked_table()` and new variants via the
per-variant JSON pipeline. Parser work translates directly into
converter coverage.
