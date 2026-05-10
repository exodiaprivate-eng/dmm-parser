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

## 1-Minute loop — full session (64 iters, 2 stop+resume cycles)

> **Refreshed 2026-05-10 iter 65** — was last updated at iter 19,
> missing 45+ commits of subsequent work.

### Phase 1 (iters 1-20): initial workplan execution + first stop

**Major shipped:**

- iters 1-8: REFLECT phase — 4 of 10 reflection formats parsed
  (.prefab, .parg, .pasg, .paseqc, .paa_metabin), 6 blocked on
  pycrimson upstream bugs. Combined harvest catalogued in
  `docs/v3_1_reflection_schema.json`.
- iter 9: CATALOG — master class index in `ENGINE_INTERNALS.md`.
- iters 10-11: GAP — canonical-field catalog inserted at top of every
  *Info table info.rs (15 GAP-tables × ~30 fields each + bulk batch
  for 14 sibling tables).
- iter 12-13: DESC — all 5 engine descriptor classes verified-blocked
  via 3-source cross-check (Win-IDA + Korean fragments + schema/refl).
- iter 14: VARIANT — all 4 variants/*.rs files audited; 0 Raw branches.
- iters 15-19: DOCS — Game Surface Coverage Map, variant decoder
  catalog, Layer B Havok reference, opaque-field audit, SHIPPED.md.
- iter 20: STOP — queue empty + auto-gen exhausted, cron cancelled.

### Phase 2 (iters 21-43): resumed loop, deeper analysis + first decoder closures

**Major shipped:**

- iter 21: late-fire negative-result probe — 9 unannotated extensions
  all fail pycrimson (confirms only annotated reflection formats parse).
- iter 22: **CRITICAL CORRECTION** — discovered 8,131 of "8,362 classes"
  are prefab file paths pycrimson dumps as type markers, not class
  names. Real reflection class count is **231 with 938 fields**.
- iter 23: EXTENSION-MAP — comprehensive 86-extension reference table
  in `BINARY_FORMATS.md` with status per extension.
- iter 24: TABLE-COMPLETION — added canonical-field catalogs to all 68
  fully-decoded *Info tables (every schema-listed table now self-docs).
- iter 25: CROSS-VALIDATE — 13.4% aggregate overlap between NattKh
  schema and pycrimson reflection (mostly disjoint domains by design).
- iter 26: LONG-TAIL-2 — **Houdini integration discovered** (10 classes:
  HoudiniOutput, HoudiniParameter*, HoudiniSubnet*).
- iter 27: DECODER-GAP-PRIORITY — per-table fn pointers identified.
- iter 28: HONEST-DOC — Known Limitations section in STATUS.md.
- iter 29: SCRIPT-DOC — scripts/README.md.
- iter 30: workflow correction — schema fn pointer is generic loader,
  per-record reader is one vtable indirection deeper.
- **iter 31: 🎯 FIRST DECODER GAP CLOSED** — effect_info `_effectDataList`
  + `_meshEffectDataList` via MANUAL_OVERRIDES (mechanical translation
  diverged from canonical _List suffix).
- iter 32: 10 acronym/plural divergences fixed across 9 tables (UI/ID/XXX).
- iter 33: 9 fuzzy-normalized divergences (incl. 2 PA-internal typos).
- iter 34: 6 high-confidence one-of-each pairings (incl. 2 more PA typos).
- iter 35: auto-closure exhaustion audit (557 still open).
- iter 36: ally_group_info structural divergence finding (1-to-7 unrolled).
- iter 37: full structural-divergence scan — 2 1-to-N divergences total.
- iter 38: IDA plugin reference persisted in T0_AUDIT_TRACKING.md
  (after user research request).
- iter 39: WRAPPER-PATTERN-INSIGHT — 8 of 10 small-gap tables fit
  wrapper-vs-unrolled (NattKh's "missing" wraps 2-3 rust fields).
- iter 40: WRAPPER-PATTERN-FULL-SURVEY — 19 wrap / 8 genuine across
  all 27 gap tables; realistic decoder workload ~200-300 not 557.
- iter 41: tribe_info per-table closure plan documented.
- **iter 42: 🎯 CLOSURE WORKFLOW STRUCTURALLY VALIDATED** — pa::TribeInfo
  record reader sub_1410C8A20; type counts cross-validate PERFECTLY
  against NattKh schema (13 u8 + 9 u32 + 4 reader + 1 u64).
- iter 43: action_point_info hidden-wrap discovery — top-level
  rust_count == schema_count misled iter 40 classifier; wrap-pattern
  hides INSIDE typed sub-structs too. STOP for plugin install.

### Phase 3 (iters 44-64): post-plugin install — registry build

**Plugins installed by user:** function-string-associate, AutoRE,
HexRaysPyTools, HRDevHelper (per iter 38 reference).

**Major shipped:**

- iter 44: MOD-AUTHOR-DOC-CATCHUP — `MOD_AUTHOR_GUIDE.md` got new
  Section 0 covering the v3.1 surface that mod authors had never
  been shown (was missed in earlier doc passes).
- iters 45-46: closure-validated interaction_info + mission_info
  (2nd + 3rd tables workflow-ready).
- iter 47: closure-validated knowledge_info; **master typeinfo registry
  STARTED** (4 entries + gimmick_info special case).
- iters 48-61: REGISTRY-EXTEND batch (15 iters, 26 more entries).
  **🎯 Master typeinfo + record-reader registry COMPLETE at iter 61
  with 31 entries.** Every table with v3.1 decoder gaps has its
  Win-IDA typeinfo addr + per-record reader fn pointer mapped.
- iter 62: royal_supply_info 1-to-2 unrolled discovered.
- iters 63-64: extended structural-divergence scan; 7 total 1-to-N
  divergences documented (~22% of gap tables hit this pattern).

### Cumulative metrics (end of iter 64)

| Metric | Value |
|---|---|
| Total commits | 76+ (Session 28 + 64 loop iters) |
| Tests | 562 passing (zero regressions throughout) |
| v3.1 alias closures | 27 fields shipped (iters 31-34) |
| Tables fully workflow-validated | 4 (tribe_info, interaction_info, mission_info, knowledge_info) |
| Tables in master typeinfo registry | 31 (100% of gap tables + special cases) |
| Tables with canonical-field catalog in info.rs | 122 (100% of *Info tables) |
| 1-to-N structural divergences | 7 documented |
| PA-internal typos preserved | 6 (lowercase k in key/knowledge, missing 'e' in Frquency, radgoll vs ragdoll, mid-name underscores) |
| Reflection classes harvested | 231 / 938 fields |
| Extensions mapped | 86 (per BINARY_FORMATS.md) |
| Realistic remaining decoder work | ~150-250 fields (down from headline 557) |

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
| pycrimson reflection harvest (231 classes / 938 fields) | `ENGINE_INTERNALS.md` Master class index (iter 22 corrected the original "8,362" — 8,131 of those were prefab file paths dumped as `__pycr_type__` markers, not real classes) |
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
- **549 schema-listed `_camelCase` fields** across 23 *Info tables are
  not decoded by dmm-parser yet (down from 584/41 at iter-35 baseline,
  and from 398/14 in this doc's earlier draft — that figure was stale).
  Per the 4-class taxonomy in `MOD_AUTHOR_GUIDE.md` § Residual coverage,
  ~10 tables are 1-to-N wraps blocked on the alias-mechanism extension,
  ~2 need real decoder work, ~2 have semantic ambiguity, 1 (faction_node_info,
  14 gaps) is a larger un-audited table. Catalogued in info.rs comments
  and in `V3_1_DECODER_GAPS.md`.

- **iter 70-86 closure work (resumed-loop, 2026-05-10)**: shipped 7
  PA-internal-typo MANUAL_OVERRIDES (`_complteDescription`,
  `_overriedMaxHeight`, `_questGroupkey`, etc.) and 6 name-divergence
  tuple-scoped overrides. Documented per-table closure plans for 11
  single-missing-canonical tables (proven 1-to-N wraps via Win-IDA
  per-record reader cross-checks). Schema verifier: 86 of 109
  schema-covered tables are now 100% canonical-aliased (up from 68).
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
