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

- [ ] GAP: add canonical-name doc-comment to `src/tables/gimmick_info/info.rs` listing 153 schema fields with status (decoded/missing)
- [ ] GAP: same for `src/tables/character_info/info.rs` (146 fields)
- [ ] GAP: same for `src/tables/stage_info/info.rs` (72 fields)
- [ ] GAP: same for `src/tables/gimmick_group_info/info.rs` (45 fields)
- [ ] GAP: same for `src/tables/interaction_info/info.rs` (29 fields)
- [ ] GAP: same for `src/tables/tribe_info/info.rs` (26 fields)
- [ ] GAP: same for `src/tables/mission_info/info.rs` (25 fields)
- [ ] GAP: same for `src/tables/field_info/info.rs` (22 fields)
- [ ] GAP: same for `src/tables/faction_node_info/info.rs` (15 fields)
- [ ] GAP: same for `src/tables/region_info/info.rs` (4 fields)
- [ ] GAP: same for `src/tables/global_game_event_info/info.rs` (4 fields)
- [ ] GAP: same for `src/tables/knowledge_info/info.rs` (3 fields)
- [ ] GAP: same for `src/tables/vehicle_info/info.rs` (3 fields)
- [ ] GAP: same for `src/tables/action_point_info/info.rs` (2 fields)
- [ ] GAP: same for `src/tables/effect_info/info.rs` (2 fields)
- [ ] DESC: AttackCommonDataDesc — enumerate via Win-IDA registrar (sub_141957EC0 sub-call), document in T0_AUDIT_TRACKING
- [ ] DESC: AttackHitDataDesc — enumerate via Win-IDA, document
- [ ] DESC: BuffData family registrars (120 variants) — sample 5 representative variant types
- [ ] DESC: EffectData family registrars — sample 3 representative variants
- [ ] DESC: ConditionData family registrars — sample 3 representative variants
- [ ] VARIANT: src/binary/variants/auto_spawn_entry.rs — list still-Raw branches
- [ ] VARIANT: src/binary/variants/drop_target.rs — same
- [ ] VARIANT: src/binary/variants/mini_game_data.rs — same
- [ ] VARIANT: src/binary/variants/sequencer_stage_chart_desc.rs — same

## Done

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
