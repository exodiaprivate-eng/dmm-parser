# dmm-parser status & handoff

**Last updated**: 2026-04-30
**Repo**: https://github.com/exodiaprivate-eng/dmm-parser
**Branch**: `main`

This file is for collaborators picking up round-trip work. It's the
"where are we, what's next" snapshot. For per-table specs see
`docs/449_TABLE_CATALOG.md`. For repo conventions see `Claude.MD`.

---

## Current state

### Parser coverage
- **125 table parsers** wired in `src/tables/`
- **All 449 vanilla `.pabgb` files round-trip byte-perfect** at the table
  level (any failing entry stays as opaque blob — never corrupts data)
- **GameCondition wrapper: 100.0% round-trip** on 8,934 ConditionInfo
  entries (typed decode for 99.8%, raw-bytes fallback for 0.2%)

### Recent Tier 1 promotions (lane-c)
- `CharacterInfo` — all 174 wire fields typed, 0 nonempty tails on 6966 entries
- `FactionNodeSpawnInfo` — patrol_ai_spline_data_list typed
  (sub_141115890 + sub_1413F8A20 + sub_1413F9BD0 reverse-engineered)
- `FrameEventAttrGroupInfo` — sub_1410E14F0 turned out to be fixed-shape
  (not polymorphic as the old docstring claimed); 421 wire bytes per
  FrameEventAttr with 5× triplet + 5× secondary + 5× tertiary + 5× pair
- `LevelGimmickSceneObjectInfo` — sub_1410EB270 fixed-shape (16 fields
  per element including 2× SceneObjectAA1B0Block)
- `TerrainRegionAutoSpawnInfo` + `SpawningPoolAutoSpawnInfo` — both
  share the AutoSpawnEntry type from `binary::auto_spawn_entry`. Cracked
  sub_1411092E0 / sub_1410FA2A0 / sub_141109110 / sub_1410F9F00 /
  sub_1410F9DF0 / sub_14100CAB0 nested polymorphic chain.
- `GimmickInfo` — Decoded tail extended from 1 to 10 typed fields
  (use_interaction_ui_socket, use_sub_part_for_interaction,
  property_list, gimmick_name_hash, gimmick_name, emoji_texture_id,
  dev_memo, hash_pair_list, hash_single_list); 99.93% Decoded

### Remaining Tier 1.5 (blocked by family decoders)
- `DropSetInfo._list` — sub_141102760 / sub_141D03AA0 (ReflectObject reflection)
- `EquipSlotInfo.header_blob` + `.footer` — opaque sub_1410830B0 prefix
- `MiniGameDataInfo.spawn_data_list_blob` — sub_14110E010 nested polymorphic
  with anti-disassembly territory
- `QuestInfo.quest_dialog_filter_data_list_blob` — FilterCondition variant family
- `GimmickInfo.post_blob` — within Decoded; blocked by sub_1411125E0
  (CArray<COptional<sub_141D7FF30>>)

### JSON exposure upgrades (lane-c)
- `SkillInfo.buff_level_list` (CArray<CArray<BuffDataOptional>>) — was
  base64; now fully typed nested JSON via BuffData ToJsonValue +
  BuffDataOptional impls. Each per-level per-buff variant body is
  individually editable.
- `ImmuneBuffData.entries.body` — was base64; now a typed JSON array of
  integers sized by header_tag (u8 / u32 / u64 stride).
- `AdditionalUseResourceStat.f01_entries` — was array of base64 strings;
  now nested JSON arrays of u8 integers (each 22-byte record fully
  byte-addressable through JSON).
- `StageInfo` — vestigial empty `tail_blob` removed (was always-empty
  Vec<u8> + `_tail_blob_b64` JSON field). Reader now strict-asserts
  full consumption.
- Added `json_roundtrip` test for SkillInfo (now part of 305 tests).

### Polymorphic family decoders
| Family | Status | Tables that consume it |
|---|---|---|
| **GameCondition** | ✅ 100% (Decoded\|Raw enum, commit `5160cdd`) | ConditionInfo (Tier 1, commit `9f1be1d`) |
| **GlobalGameEventExecuteData** | ✅ 100% (Absent\|Present\|Raw enum, commit `4b30791`) | GlobalGameEventInfo (Tier 1) |
| **GameEventHandlerData** | ✅ 100% (Decoded\|Raw enum) | GameEventHandlerInfo (Tier 1) |
| BuffData | ✅ shipped (per buff_data.rs) | SkillInfo, CharacterChangeInfo |
| BranchConditionData | ✅ shipped | (used inside GameCondition tree) |
| ConditionDataStageChart | ✅ shipped | (used inside GameCondition tree) |
| ConditionGimmickData | ✅ shipped | (used inside GameCondition tree) |
| ScheduleCompleteConditionData | ✅ shipped | (used inside GameCondition tree) |
| GlobalEffectConditionData | ✅ shipped | (used inside GameCondition tree) |
| MiniGameData | ✅ shipped | MiniGameDataInfo |
| GameExpression / IVariantItem | ✅ shipped (inside StageChart) | (used inside GameCondition tree) |
| EffectData | ✅ shipped (per-element typed, 47-field core_block) | EffectInfo (Tier 1) |
| **SequencerStageChartDesc** | ✅ shipped — all 26 wire fields typed in `binary::sequencer_stage_chart_desc::SequencerStageChartDescPartial`. Composes inside CArray via stream-mode trait impls. | FieldReviveInfo, ItemUseInfo PlaySequencerOnly, SequencerSpawnInfo (Tier 1), GlobalStageSequencerInfo (Tier 1), StageInfo (Tier 1, all 91 wire fields), InteractionInfo (Tier 1 with Decoded\|Raw fallback), **CharacterInfo (Tier 1, all 174 wire fields, lane-c)** |
| **GimmickInteractionOverrideCArray** | ✅ shipped — `binary::gimmick_interaction_override::GimmickInteractionOverrideCArray` (15-field inner via sub_1410DF770). | GimmickInfo (Tier 1.5 Decoded\|Raw, 99.93%), CharacterInfo field 133 (Tier 1) |
| **SequencerStageTrackChangeData** family (Character/Gimmick/Item) | ✅ shipped (inside SequencerStageChartDesc field 19) | (used inside SequencerStageChartDesc) |
| **SequencerStageSpawnData** | ✅ shipped (inside SequencerStageChartDesc field 20) | (used inside SequencerStageChartDesc) |
| **GameEventHandler** | ❌ **next target** | GameEventHandlerInfo |
| **TriggerEventHandler** | 🟡 deferred (uses `pa::ReflectObject` reflection-driven serialization, different pattern from bespoke dispatchers — needs reflection layer reversed first) | TriggerRegionInfo and others |

### Tables by tier
- **Tier 1** (typed, all fields editable through JSON): the bulk of the
  125 tables — see `docs/449_TABLE_CATALOG.md` for the per-table list.
  ConditionInfo just joined this tier (commit `9f1be1d`).
- **Tier 1.5** (typed-internal, blob payload via base64 / clone-only):
  tables whose polymorphic body waits on its family decoder.
- **Tier 2** (whole-tail blob): no longer the default — only used for
  tables where we haven't yet hand-corrected the wire format.

---

## What just shipped (this session, all in `origin/main`)

```
GameEventHandlerData: ship Tier 1 family decoder + wire GameEventHandlerInfo
8e9b6f6  docs/STATUS.md: GlobalGameEventExecuteData shipped, refresh queue
4b30791  GlobalGameEventExecuteData: ship Tier 1 family decoder w/ Decoded|Raw enum
e17d416  docs: add STATUS.md for collaborator handoff
9f1be1d  ConditionInfo: promote Tier 2 → Tier 1 — typed GameCondition wrapper
5160cdd  GameCondition: Raw-bytes fallback variant → 100.0% round-trip 🎯
dd72172  ConditionData: 5-tag OneByteBodyPayload batch (11/92/253/343/351) → 99.8%
b82e3c7  ConditionData: tags 126/178/287/306 + LAST_ATTEMPTED_TAG tracker → 99.7%
1d49637  ConditionData: tags 17/18/19/20/21/22/26/27/29/33 → OneByteBodyPayload, 99.3%
2af19fa  ConditionData: tags 2/55/79/101/136/195/214/256/290/374/399/401 + tag 31 → 99.2%
a4118f5  ConditionData: batch 1-byte/4-byte/CString body fixes → 98.3%
```

GameCondition went from 13.4% → 100% across the first 7 commits.
GlobalGameEventInfo Tier 2 → Tier 1 in the latest commit (80/80 entries
decode structurally, 0 raw fallbacks needed).

---

## The reusable playbook

The pattern that cracked GameCondition is now documented in code and
ready to apply to the next polymorphic family. In order:

1. **Find the meta-dispatcher**. Search the Win-IDA decompile for the
   table's known offsets (look for the Korean string xref to the
   table's pabgb name). Find the `switch (tag)` that allocates +
   constructs each variant.
2. **Extract the tag → read_fn map**. Reuse the python script at
   `dmm-pabgb-aio/extract_conditiondata_dispatch.py` as a template.
   It parses the dispatcher decompile, emits JSON keyed by tag with
   `(alloc_size, read_fn, class_name)`.
3. **Stand up a recursive enum** in `src/binary/variants/<family>.rs`.
   Start with case-tag dispatch — leaf payloads as `Vec<u8>` initially.
4. **Build a round-trip validator** in `examples/` that walks every
   real entry from the consuming table's pabgb. Capture the
   `LAST_ATTEMPTED_TAG` thread_local pattern (see condition_data.rs
   line ~5219 for the reference impl).
5. **Loop**: run validator → look at the "Failing tags" table at the
   bottom → decompile that tag's vtable[16] (offset +0x80 from the
   class's `??_7<ClassName>@pa@@6B@` symbol) → fix the body recipe →
   repeat. Each iteration kills 1-30 failures.
6. **Add the Decoded|Raw fallback** at the wrapper level (see
   `src/binary/variants/game_condition.rs` lines 29-78 for the canonical
   implementation). Guarantees 100% round-trip even with un-decoded
   variants.

The whole arc takes 1-2 sessions per family if the dispatcher is clean
(non-obfuscated). Tags 54/286 in ConditionData are anti-disassembly
obfuscated — those stay in the Raw bucket forever, which is fine.

---

## Key files

### Parser core
- `src/binary/mod.rs` — read/write traits (`u8`/`u16`/`u32`/`u64`/`CString`/`CArray`)
- `src/binary/variant.rs` — `pabgh_typed_blob_table!` macro,
  `find_cstring_u8_trailer` helper, `entry_ranges`/`load_pabgh_offsets`
- `src/json_traits.rs` — manual `ToJsonValue`/`WriteJsonValue` traits
  used by every `to_json_dict`/`write_from_json_dict` impl

### Polymorphic families
- `src/binary/variants/game_condition.rs` — **canonical Decoded|Raw
  fallback** wrapper. Use this as the template for next family.
- `src/binary/variants/condition_data.rs` — 405-variant dispatch with
  the `LAST_ATTEMPTED_TAG` thread_local tracker (line ~5210)
- `src/binary/variants/branch_condition_data.rs` — smaller (14 variants)
  example of the pattern
- `src/binary/variants/buff_data.rs` — first family decoder, original
  reference implementation

### Tables
- `src/tables/condition_info/info.rs` — **just-shipped Tier 1** with
  typed GameCondition wrapper. Use as the model for wiring future
  family decoders into their consuming tables.
- `src/tables/skill_info/` — original blueprint for Tier 1 with
  polymorphic body (BuffData)
- `docs/449_TABLE_CATALOG.md` — per-table status

### Validators
- `examples/game_condition_roundtrip.rs` — measures decode + round-trip
  for every ConditionInfo entry. Has env-var dump filters
  (`GC_DUMP_TAG=NN`, `GC_DUMP_TAGS=N,N,N`). Final output includes the
  "Failing tags" table from the `LAST_ATTEMPTED_TAG` tracker — that's
  the smoking gun that tells you which variant's recipe is wrong.

### IDA dumps (in sibling repo `dmm-pabgb-aio`)
- `mac_extract/conditiondata_dispatch_map.json` — tag → read_fn for
  ConditionData's 405 variants
- `mac_extract/conditiondata_dispatcher_plain.txt` — saved dispatcher
  decompile (use as input to extract scripts)
- `mac_extract/conditiondata_empirical_observations.json` — per-tag
  size stats from real data (cross-check against IDA recipes)

---

## What's next, in priority order

### Big wins (each enables a polymorphic family)
1. **EffectData family**. EffectInfo consumer. Most likely the next
   bespoke-dispatcher target.
2. **Per-sub_tag typed payloads inside GlobalGameEventExecuteData &
   GameEventHandlerData** (follow-up). The `Decoded { sub_tag, body:
   Vec<u8> }` shape is shipped for both; full per-sub_tag typed body
   structs are mechanical work that unlocks field-level JSON editing
   inside the body.
3. **TriggerEventHandler family** (task #95). DEFERRED — uses
   `pa::ReflectObject` reflection-driven serialization (sub_14055F190
   constructor reveals the inheritance chain through `ReflectDerive
   <ITriggerEventHandler, ReflectObjectExtension>`). Different pattern
   from the bespoke dispatchers; needs the reflection layer reversed
   first. Worth tackling because cracking it unlocks ALL reflection-
   serialized tables at once.

### Smaller wins
5. **Wire JSON tree exposure for GameCondition's Decoded variant** —
   the typed wrapper is in place but JSON still ships as base64.
   Implementing per-variant `ToJsonValue`/`WriteJsonValue` for
   `GameConditionNode` (9 cases) + `ConditionData` (405 variants) lets
   users edit the recursive tree directly through JSON. Mechanical work
   — generate from the existing variant struct definitions.
6. **Wire ConditionInfo Tier 1 into DMM v3 dispatch** — needs a check
   in DMM-BETA's mod-loader to route conditioninfo edits through the
   new typed parser. Small CLAUDE.md change in the consuming repo.
7. **Promote remaining Tier 1.5 tables to Tier 1** — list in
   `docs/449_TABLE_CATALOG.md`. Each is mechanical when its family
   decoder is ready.

### Deferred (need runtime debugger or are non-blocking)
- ConditionData tags 54/286 — anti-disassembly obfuscated readers
  (sub_14D3012D0 family). Currently round-trip via the Raw fallback,
  which is fine. Recoverable later if anyone runs the game in a
  debugger and observes the obfuscated reader's actual byte
  consumption.
- ConditionData tag 272 sub_tag holes (0x42, 0x1d) — likely truncated
  debug entries in the source data; not worth chasing.

### Stream-mode GameCondition (partially unblocked, 84% interaction_info)
**Root cause identified**: The `variant_skips_option_block` list in
`condition_data.rs` was incomplete. Variants whose vtable[19] is a
no-op (return 1 with no read) need to be in the skip list. The list
originally had 11 variants (81, 272, 300, 256, 401, 2, 79, 195, 306,
126); empirical adds via LAST_ATTEMPTED_TAG diagnostic loop bumped it
to 16 (added 26, 135, 370, 99, 174, 360).

**Current state**: 306 of 363 interaction_info entries (84.3%)
successfully decode. Bulk-adding remaining candidates regressed the
success rate (297 → 206), so each candidate must be tested individually.

**Important caveat (verified this session via Win-IDA)**: Of the 16
tags currently in the skip list, only the original 11 are confirmed
"true" vtable[19] no-ops. The 5 empirical adds (26, 135, 370, 99,
174, 360) are NOT vtable[19] no-ops — their slot-19 entries each
point into the giant `sub_14139AE80` thunk forest (size 0x1dc88,
non-decompilable by Hex-Rays). Concrete check: tag 81's vtable
(`ConditionData_QuestGaugePercent` at `0x144ce3038`) has slot 19 =
`0x1402d3a80` (the `return 1;` no-op), while tag 99's vtable
(`ConditionData_CheckAllyType` at `0x144cdc770`) has slot 19 =
`0x1413b89e0` (a thunk inside `sub_14139AE80`). Yet tag 99 is in
the skip list because it empirically unblocked entries.

This means the empirical adds are likely **masking** real bugs where
LAST_ATTEMPTED_TAG points to the wrong tag in the failure chain. The
57-entry ceiling on interaction_info reflects this: pushing past it
requires proper per-variant vtable[19] reverse engineering, not more
empirical adds.

**Path forward (revised)**:
1. Walk all 405 ConditionData_* vtables, read slot 19 of each, and
   build a definitive list of `slot_19 == 0x1402d3a80` (true no-ops).
2. Replace the empirical adds (26, 135, 370, 99, 174, 360) with the
   verified list — likely a strict subset.
3. For the empirical adds that turn out NOT to be no-ops, investigate
   why removing them STILL allows their entries to decode (likely
   because the body recipe is wrong elsewhere — option_block probe is
   misaligning a downstream byte).
4. With the verified skip list, re-run the per-variant diagnostic on
   interaction_info to find the actual remaining 57-entry blockers.
5. Apply ConditionPairCArray to interaction_info field 10 once
   100% decode.
6. Repeat the same approach for gimmick_info field 7
   (sub_141118470 → sub_1410DF770 → BareConditionPairCArray at
   sub_141E2C900), character_info field 133, stage_info field 7
   (SequencerStageChartDesc), global_stage_sequencer_info field 6.

The vtable layouts and per-element wire layouts for sub_141D8C6D0
(SequencerStageChartDesc, 26 wire fields / 232 mem bytes) and
sub_1410DF770 (GimmickInteractionOverrideData, 15 wire fields / 144
mem bytes) are documented in the consuming tables' module docstrings
and ready to wire up the moment the skip-list is verified.

---

## Quick reference: how to verify nothing regressed

```bash
# Full test suite
cargo test --release

# GameCondition round-trip validator
cargo run --release --example game_condition_roundtrip
# Should print: Round-trip OK: 8934 (100.0%)

# Per-table round-trip (ConditionInfo, skill_info, etc.)
cargo test --release condition_info
cargo test --release skill_info
```

If any of these regress, `git log --oneline -10` and bisect against the
last known-good commit.

---

## Conventions

- New table parsers go in `src/tables/<name>/info.rs` with companion
  `mod.rs` + (optional) `RECIPE_NEXT.md` for status notes.
- Hand-written parsers must start with the `//! Hand-corrected:` header
  marker — `bulk_process.py` skips files with this header.
- All tests should pass before pushing. The validator at 100% is the
  hard floor for GameCondition.
- Don't touch `Cargo.toml` deps without coordinating — the workspace
  is consumed by DMM-BETA, JSMM, and ext-builds.
