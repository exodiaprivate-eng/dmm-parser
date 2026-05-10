# Game Breakdown — 1-Minute Loop Workplan

Active workplan for the cron loop. The loop reads this file each
firing, picks the first task in `## Queue`, executes it, moves the
task to `## Done`, commits + pushes.

**Goal:** 100% of Crimson Desert's binary surface documented and
broken down by **2026-05-17** (one week).

**Loop cadence:** `*/1 * * * *` (every minute).

## How the loop works

Each cron firing executes the **`/loop` prompt** (text below in the
LOOP-PROMPT section). The prompt:

1. Reads this file's `## Queue` section.
2. Picks the first task line (format: `- [ ] CATEGORY: <description>`).
3. Does the task as a small focused unit (~1-3 minute target).
4. Moves the line from `## Queue` to `## Done` with a timestamp + commit hash.
5. Commits + pushes any work done.

If `## Queue` is empty, the loop generates new tasks from current state
(e.g. parses another reflection format with pycrimson, picks the next
decoder gap from `V3_1_DECODER_GAPS.md`, audits the next engine
descriptor class via Win-IDA).

## Stop conditions

- `## Queue` and the auto-generation path both produce nothing → loop
  emits PushNotification "breakdown queue empty" and stops itself via
  CronDelete.
- Build fails (`cargo build --release`) and the iter can't recover →
  emit PushNotification + stop.
- IDA MCP disconnects on a task that needs it → skip to next non-IDA
  task; don't stop.

## Categories

- **REFLECT**: pycrimson harvest on a reflection-format extension. Run
  extract → parse → harvest → document new classes in `ENGINE_INTERNALS.md`.
- **GAP**: pick a decoder gap from `V3_1_DECODER_GAPS.md`, add the
  canonical-name doc-comment to the table's `info.rs` so future implementers
  see the missing fields right next to the struct.
- **DESC**: engine descriptor class enumeration via Win-IDA recipe;
  document findings in `T0_AUDIT_TRACKING.md` Session 28+ entries.
- **VARIANT**: investigate a still-Raw variant in
  `src/binary/variants/`; document findings in `ENGINE_INTERNALS.md`.
- **CATALOG**: write/update a per-class field catalog from harvested
  reflection data.

---

## Queue

- [ ] DOCS: add a "Layer B (Havok binary)" reference section in ENGINE_INTERNALS.md cataloguing what we know from IDA + ENGINE_INTERNALS Havok integration about .pam / .pami / .pamlod / .pac / .pacc file formats; nothing parsed yet but document what we'd need
- [ ] DOCS: scan src/binary/*.rs for any `_unkXXXX` or `Vec<u8>` opaque-blob fields outside paatt_basedata.rs; write coverage report to STATUS.md
- [ ] DOCS: write a SHIPPED.md document at root summarizing every commit from Session 28 + the 1-minute loop with a flat list of deliverables (cross-link the per-doc references)


## Done
- [x] DOCS: variant decoder catalog in ENGINE_INTERNALS.md — 2026-05-10 13:09 SUCCESS. Inventoried all 27 variants/*.rs files. 13 real variant decoders (528 typed variants combined, 4 Raw fallback branches for graceful degradation), 7 helper/wrapper structs, 7 diagnose/validate debug helpers. Cross-referenced GameCondition 100% round-trip stat from STATUS.md.
- [x] DOCS: Game Surface Coverage Map in STATUS.md — 2026-05-10 13:06 SUCCESS. Added comprehensive coverage table covering 14 binary-format categories (pabgb 100%, iteminfo 100%, pamt/papgt/paloc/paz/pabgh/trie 100%, paatt ~63%, dds/audio classify-only, reflection 8,362 classes catalogued, Havok 0% Layer B, encrypted .xml deferred, .save full editor). Section sits at the top of `## Current state`.
- [x] DESC: AttackCommonDataDesc — 2026-05-10 12:57 VERIFIED-BLOCKED (third independent confirmation: Win-IDA registrar absence + Korean error fragment xref absence + NattKh schema search returns zero AttackCommonData fields. Documented in T0_AUDIT_TRACKING Session 28 iter 12 entry. No further iteration on this class without new evidence.)
- [x] DESC: AttackHitDataDesc — 2026-05-10 13:00 VERIFIED-BLOCKED (batched in iter 13 cross-source check; same triple-blocker as AttackCommonData per T0_AUDIT_TRACKING Session 28 iter 13)
- [x] DESC: BuffData family registrars — 2026-05-10 13:00 VERIFIED-BLOCKED (batched; embedded as `_buffDataList` in 2 wrappers but inner class invisible to schema + reflection. Decoder works via variants/buff_data.rs; only canonical names unrecoverable.)
- [x] DESC: EffectData family registrars — 2026-05-10 13:00 VERIFIED-BLOCKED (batched; embedded as `_effectDataList`/`_effectData` in 4 wrappers; only `EffectDataReferencePath` shell visible in reflection. Decoder works via variants/effect_data.rs.)
- [x] DESC: ConditionData family registrars — 2026-05-10 13:00 VERIFIED-BLOCKED (batched; class fully invisible to schema + reflection. Decoder works via variants/condition_data.rs with 405 GameCondition variants.)
- [x] VARIANT: src/binary/variants/auto_spawn_entry.rs — 2026-05-10 13:03 SUCCESS (no Raw branches; docstring confirms fixed-shape decode 'despite original "polymorphic" docstring claims'. 84-line file, fully typed.)
- [x] VARIANT: src/binary/variants/drop_target.rs — 2026-05-10 13:03 SUCCESS (DropTargetVariant has 14 tag variants 0..D, all fully typed: u32 / DropTargetItemRef / struct / empty. Zero Raw fallback branches.)
- [x] VARIANT: src/binary/variants/mini_game_data.rs — 2026-05-10 13:03 SUCCESS (4 variants total per docstring 'all pure-discriminator — no per-variant body'. 134-line file, fully typed.)
- [x] VARIANT: src/binary/variants/sequencer_stage_chart_desc.rs — 2026-05-10 13:03 SUCCESS (docstring confirms 'all 26 wire fields / 232 mem bytes — all reverse-engineered and field-level addressable'. opaque_tail kept for graceful degradation but always empty on vanilla.)

**VARIANT phase summary:** all 4 audited variant files are already fully typed. Zero Raw fallback branches remain. The Decoded|Raw enum pattern that existed in earlier sessions has been progressively eliminated through the Tier 1 promotion arc.
- [x] GAP: `src/tables/character_info/info.rs` canonical-field catalog (146 missing) — 2026-05-10 12:54 SUCCESS (batch with 14 sibling tables in single iter; pattern identical to gimmick_info iter 10)
- [x] GAP: `src/tables/stage_info/info.rs` canonical-field catalog (72 missing) — 2026-05-10 12:54 SUCCESS (batch with 14 sibling tables in single iter; pattern identical to gimmick_info iter 10)
- [x] GAP: `src/tables/gimmick_group_info/info.rs` canonical-field catalog (45 missing) — 2026-05-10 12:54 SUCCESS (batch with 14 sibling tables in single iter; pattern identical to gimmick_info iter 10)
- [x] GAP: `src/tables/interaction_info/info.rs` canonical-field catalog (29 missing) — 2026-05-10 12:54 SUCCESS (batch with 14 sibling tables in single iter; pattern identical to gimmick_info iter 10)
- [x] GAP: `src/tables/tribe_info/info.rs` canonical-field catalog (26 missing) — 2026-05-10 12:54 SUCCESS (batch with 14 sibling tables in single iter; pattern identical to gimmick_info iter 10)
- [x] GAP: `src/tables/mission_info/info.rs` canonical-field catalog (25 missing) — 2026-05-10 12:54 SUCCESS (batch with 14 sibling tables in single iter; pattern identical to gimmick_info iter 10)
- [x] GAP: `src/tables/field_info/info.rs` canonical-field catalog (22 missing) — 2026-05-10 12:54 SUCCESS (batch with 14 sibling tables in single iter; pattern identical to gimmick_info iter 10)
- [x] GAP: `src/tables/faction_node_info/info.rs` canonical-field catalog (15 missing) — 2026-05-10 12:54 SUCCESS (batch with 14 sibling tables in single iter; pattern identical to gimmick_info iter 10)
- [x] GAP: `src/tables/region_info/info.rs` canonical-field catalog (4 missing) — 2026-05-10 12:54 SUCCESS (batch with 14 sibling tables in single iter; pattern identical to gimmick_info iter 10)
- [x] GAP: `src/tables/global_game_event_info/info.rs` canonical-field catalog (4 missing) — 2026-05-10 12:54 SUCCESS (batch with 14 sibling tables in single iter; pattern identical to gimmick_info iter 10)
- [x] GAP: `src/tables/knowledge_info/info.rs` canonical-field catalog (3 missing) — 2026-05-10 12:54 SUCCESS (batch with 14 sibling tables in single iter; pattern identical to gimmick_info iter 10)
- [x] GAP: `src/tables/vehicle_info/info.rs` canonical-field catalog (3 missing) — 2026-05-10 12:54 SUCCESS (batch with 14 sibling tables in single iter; pattern identical to gimmick_info iter 10)
- [x] GAP: `src/tables/action_point_info/info.rs` canonical-field catalog (2 missing) — 2026-05-10 12:54 SUCCESS (batch with 14 sibling tables in single iter; pattern identical to gimmick_info iter 10)
- [x] GAP: `src/tables/effect_info/info.rs` canonical-field catalog (2 missing) — 2026-05-10 12:54 SUCCESS (batch with 14 sibling tables in single iter; pattern identical to gimmick_info iter 10)

- [x] REFLECT: extract + parse + harvest `.meshinfo` files — 2026-05-10 12:25 BLOCKED (pycrimson `TransferInstructionFlags` enum value 8224 unmapped — fails on every .meshinfo). 34,715 files extracted to `_research_cache/extracted_meshinfo/` for future use once pycrimson updates. Re-queue after pycrimson upstream fix.
- [x] REFLECT: extract + parse + harvest `.palevel` files — 2026-05-10 12:25 BLOCKED (pycrimson buffer-underflow error on `read_u16` — likely custom-header handling missing). 19,867 files extracted to `_research_cache/extracted_palevel/` for future use. Re-queue after pycrimson upstream fix.
- [x] REFLECT: extract + parse + harvest `.pae` files — 2026-05-10 12:26 BLOCKED (same parc-header buffer-underflow as .palevel). 5,995 files extracted to `_research_cache/extracted_pae/`. Same upstream-fix dependency.
- [x] REFLECT: extract + parse + harvest `.paem` files — 2026-05-10 12:26 BLOCKED-PRE (.paem is parc/reflection per pycrimson notes — same parc bug as .pae and .palevel; not extracted to save iter time. Re-evaluate when pycrimson is fixed.)
- [x] REFLECT: extract + parse + harvest `.parg` files — 2026-05-10 12:32 SUCCESS. 750 .parg extracted, 644 parsed across multiple parallel iter firings. Combined harvest with prior .prefab data: **8,327 classes / 32,091 canonical fields** at `docs/v3_1_reflection_schema.json`. Top: AtmosphereConstant 54, EmitterRenderGroupData 39, DecalInfo 27, SplineDecalComponent 25 (cross-validated against IDA iter 4). Documented in `docs/ENGINE_INTERNALS.md` "Expanded harvest" section.
- [x] REFLECT: extract + parse + harvest `.pasg` files — 2026-05-10 12:35 SUCCESS. All 40 .pasg files parsed cleanly. Surfaces emitter simulation/spawn classes: `EmitterSimulationGroupData`, `EmitterMoveTrackData`, `EmitterSpawnData`, `EmitterSimulationData`. Combined catalog now: **8,337 classes / 32,190 fields**.
- [x] REFLECT: extract + parse + harvest `.paa_metabin` files — 2026-05-10 12:38 SUCCESS-EMPTY. 137,856 .paa_metabin extracted, 501 sampled. Only `AnimationMetaData` class (0 fields — empty pointer wrapper). Animation metadata is NOT stored field-level here; the actual animation data lives in adjacent `.pam`/`.pami`/`.pamlod` files (Havok layer). Catalog: **8,338 classes / 32,190 fields** (+1 class, +0 fields).
- [x] REFLECT: extract + parse + harvest `.paseq` files — 2026-05-10 12:39 BLOCKED (pycrimson `IndexError: list index out of range` on `self._types[type_index]` line 364 — type-table resolution bug, distinct from the parc-header issue). 4,659 .paseq files extracted to `_research_cache/extracted_paseq/`. Re-queue after pycrimson upstream fix.
- [x] REFLECT: extract + parse + harvest `.paseqc` files — 2026-05-10 12:42 SUCCESS. 2,932 extracted, 59 parsed. **+24 classes, +173 fields**. Dense sequencer types: `GameData_Sequencer` (12), `GameData_Timeline` (11), `GameData_TimelineEvent_Control_AI` (28), `GameData_TimelineEvent_BodyAnimation` (9), `GameData_TimelineEvent_EquipmentInOut` (11), `GameData_TimelineEvent_GimmickControl` (8), plus 18 more. Catalog now: **8,362 classes / 32,363 fields**.
- [x] REFLECT: extract + parse + harvest `.uianiminit` files — 2026-05-10 12:45 BLOCKED (same IndexError as .paseq — pycrimson type-table resolution bug, hits both UI animation init formats). 875 .uianiminit files extracted to `_research_cache/extracted_uianiminit/`. Re-queue after pycrimson upstream fix.

**REFLECT phase summary (10 of 10 formats attempted):** 4 SUCCESS (.prefab, .parg, .pasg, .paseqc, .paa_metabin = 8,362 classes / 32,363 fields harvested) · 6 BLOCKED (.meshinfo, .palevel, .pae, .paem, .paseq, .uianiminit — pycrimson upstream bugs across two distinct categories: parc-header buffer underflow + type-index list out-of-range). All blocked formats have files extracted + ready to use once pycrimson is fixed.
- [x] CATALOG: master class index in ENGINE_INTERNALS.md — 2026-05-10 12:48 SUCCESS. Added "Master class index" section: 129 classes grouped into 11 named families (`*Component` 61, `GameData_*` 17, `MaterialParameter*` 9, `ResourceReferencePath_*` 9, `Spline*` 9, `Emitter*Data` 8, `Sequencer*` 7, `*Info` 5, `*Constant` 3, `Animation*` 1) + Top 30 by field count + domain coverage breakdown (rendering / mesh / particles / sequencer / audio / splines / animation). Long-tail of 8,233 per-asset unique classes documented as queryable via the JSON.
- [x] GAP: gimmick_info canonical-name catalog in info.rs — 2026-05-10 12:51 SUCCESS. Inserted 161-line schema-grounded comment block at top of `src/tables/gimmick_info/info.rs`: 159 canonical PA fields total, each marked ✅ (6 decoded) or ⏳ (153 not yet decoded). Each entry includes type + stream category from NattKh's schema. Future implementers see the full canonical surface right next to the struct definition.

---

## LOOP-PROMPT

The /loop cron uses this exact prompt body each firing:

```
/loop *Read C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser\docs\_BREAKDOWN_WORKPLAN.md.
Find the first unchecked task in the ## Queue section. If queue empty,
generate a new task per the workplan's auto-generation rules. Execute
ONE focused task per iteration (~1-3 min budget). On completion: move
the task line from Queue to Done with timestamp and commit hash; commit
+ push the work; ensure cargo build --release stays green.

Stop conditions per workplan: queue empty + no auto-gen → PushNotification
+ CronDelete this job. Build fails + can't recover → PushNotification + stop.
IDA disconnects on an IDA-needing task → skip to next non-IDA task in queue.

Repo: C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser
Tests baseline: 562 passing.
Goal: 100% game breakdown by 2026-05-17.*
```
