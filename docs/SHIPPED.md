# Shipped — Session 28 + 1-Minute Loop

Flat catalog of every commit shipped during Session 28 and the
2026-05-10 1-minute breakdown loop. Each entry links to the relevant
docs.

> Cross-reference index for the cron-loop run that ended at iter 19
> (this commit). For active workplan and per-iter notes see
> `docs/_BREAKDOWN_WORKPLAN.md`. For long-term project status see
> `docs/STATUS.md`.

## Session 28 (pre-loop) — v3.1 surface foundation

| Commit | Title | Topic |
|---|---|---|
| `9e29e10` | bulk v3.1 alias surface — 113 tables | mechanical alias generator |
| `2724abe` | step (a) schema-ground v3.1 against NattKh | 463 false-positives → 0 |
| `2312389` | step (b) decoder-gap audit | 584 fields / 41 tables (`docs/V3_1_DECODER_GAPS.md`) |
| `6273c7f` | step (c) pycrimson reflection workflow | first 4-class harvest |
| `103009d` | iter 3 — EmitterCurveData enumerated (info only) | 4 fields |
| `d7f8b88` | iter 4 — SplineDecalComponent enumerated (info only) | 17 fields |
| `d5b9f62` | iter 5 — AttackCommonData blocked (no metaobject) | structural blocker recorded |
| `9d0f585` | iter 6 — AttackHitData blocked (same diagnostic) | confirmation |
| `092c0fe` | docs sync — STATUS / api / 449_TABLE_CATALOG / T0_AUDIT_TRACKING | 4 docs refreshed |
| `dabdd0f` | docs(custom-item-creator) refresh | paloc gap closed |
| `bfc5c2b` | docs(havok) refresh — pycrimson coverage | Layer A vs Layer B |
| `1e7b342` | docs consolidation — 13 → 4 merged + 6 archived | LEGAL/BINARY_FORMATS/ENGINE_INTERNALS/V3_1_README |

## 1-Minute loop (this session, 19 iters)

| Iter | Commit | Phase | Result |
|---|---|---|---|
| 0 | `af0583f` | (workplan setup) | initial 31-task queue |
| 1 | `a5ab120` | REFLECT .meshinfo + .palevel | both BLOCKED (pycrimson upstream bugs) |
| 2 | `29a028b` | REFLECT .pae + .paem | BLOCKED + BLOCKED-PRE (parc-header pattern) |
| 3 | `cb1d4b7` | REFLECT .parg | ✅ SUCCESS — 8,327 classes / 32,091 fields |
| 4 | `2104b28` | REFLECT .pasg | ✅ SUCCESS — +10 classes / +99 fields |
| 5 | `c8ded86` | REFLECT .paa_metabin | ✅ SUCCESS-EMPTY — +1 class, 0 fields |
| 6 | `762e8c9` | REFLECT .paseq | BLOCKED (type-index IndexError) |
| 7 | `f3c1a34` | REFLECT .paseqc | ✅ SUCCESS — +24 classes / +173 fields |
| 8 | `510cafc` | REFLECT .uianiminit | BLOCKED — REFLECT phase complete |
| 9 | `e602d4e` | CATALOG | master class index in `ENGINE_INTERNALS.md` |
| 10 | `fd46a6f` | GAP gimmick_info | 159-field canonical catalog inserted |
| 11 | `93fe885` | GAP batch (14 tables) | all canonical-field catalogs inserted |
| 12 | `d03aecc` | DESC AttackCommonDataDesc | 3rd-source confirmed-blocked |
| 13 | `fce1e64` | DESC batch (4 classes) | all triple-blocked en masse |
| 14 | `76d2f1f` | VARIANT batch (4 files) | all already fully-typed |
| 15 | `7c8ccce` | DOCS Game Surface Coverage Map | `STATUS.md` 14-row table |
| 16 | `d16f1fc` | DOCS variant decoder catalog | `ENGINE_INTERNALS.md` |
| 17 | `bbb6724` | DOCS Layer B Havok binary reference | `ENGINE_INTERNALS.md` |
| 18 | `ee781ac` | DOCS opaque-field audit | `STATUS.md` |
| 19 | (this commit) | DOCS SHIPPED.md root summary | this doc |

## Phase totals

| Phase | Iters | Outcome |
|---|---|---|
| **REFLECT** | 8 | 4 SUCCESS (.prefab/.parg/.pasg/.paseqc/.paa_metabin → 8,362 classes / 32,363 fields), 6 BLOCKED on pycrimson upstream bugs |
| **CATALOG** | 1 | 8,362-class master index in `ENGINE_INTERNALS.md` |
| **GAP** | 2 | 15 tables · 573 PA fields catalogued at top of each `info.rs` (175 ✅ decoded + 398 ⏳ pending) |
| **DESC** | 2 | 5 engine descriptor classes triple-blocked en masse (Win-IDA + Korean fragments + schema/reflection all confirm same structural blocker) |
| **VARIANT** | 1 | 4 variant files audited; all already fully typed (no Raw branches) |
| **DOCS** | 5 | Coverage map + variant catalog + Layer B + opaque audit + SHIPPED.md (this) |

## Where to look for what

| Question | Doc |
|---|---|
| Per-format coverage % | `STATUS.md` Game Surface Coverage Map |
| Per-table v3.1 verification | `V3_1_SCHEMA_VERIFICATION.md` (auto-generated) |
| Per-table decoder gaps | `V3_1_DECODER_GAPS.md` |
| pycrimson workflow | `V3_1_PYCRIMSON_WORKFLOW.md` |
| 8,362-class catalogue | `ENGINE_INTERNALS.md` Master class index |
| Variant decoder inventory | `ENGINE_INTERNALS.md` Variant decoder catalog |
| Havok layer reference | `ENGINE_INTERNALS.md` Layer B section |
| File-format byte layouts | `BINARY_FORMATS.md` |
| Per-table canonical-field list | `src/tables/<name>/info.rs` (top doc-comment) |
| Audit tracking | `T0_AUDIT_TRACKING.md` |
| Active loop workplan | `_BREAKDOWN_WORKPLAN.md` |

## What this run did NOT solve

Honest accounting of what's still blocked or future work:

- **6 of 10 PA reflection formats** can't be parsed by pycrimson
  (`.meshinfo`, `.palevel`, `.pae`, `.paem`, `.paseq`, `.uianiminit`).
  Files extracted to `_research_cache/` for use after pycrimson is fixed
  upstream.
- **5 engine descriptor classes** (AttackCommonData, AttackHitData,
  BuffData, EffectData, ConditionData) cannot have their canonical PA
  field names verified from any current source. Triple-blocked.
  Decoders work; only naming is unverified.
- **398 schema-listed `_camelCase` fields** across 14 *Info tables are
  not decoded by dmm-parser yet. Catalogued in info.rs comments and in
  `V3_1_DECODER_GAPS.md`. Implementation requires per-field decoder
  writing using NattKh's schema (byte offsets + reader function pointers
  available).
- **Havok binary layer (`.pac`/`.pacc`/`.pam`/`.pami`/`.pamlod`)** has
  0% native parsing. Layer B in `ENGINE_INTERNALS.md`. Standard Havok
  2024.2 SDK, would need DCC-plugin-equivalent reader.
- **`paatt_basedata.rs` 35 `_unkXXXX` fields** are the only remaining
  placeholders in the codebase, structurally unrecoverable without new
  evidence (PS5 demo binary or pycrimson .paatt support).

Tests: 562 passing throughout. No source-code regressions.

## Tags

Cron loop scheduled by id `15ac410b` at `*/1 * * * *`; auto-expires
2026-05-17. Cancel with CronDelete.
