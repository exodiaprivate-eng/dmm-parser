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
| Realistic remaining decoder work | ~150-250 fields (down from iter-35-baseline 557 → iter-65 549; the iter 70-82 closures absorbed alias-mappable cases, leaving the harder structural-refactor + new-decoder cases) |

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
| **REFLECT** | 8 | 4 SUCCESS (.prefab/.parg/.pasg/.paseqc/.paa_metabin → **231 classes / 938 fields** post iter-22 correction; original raw count was 8,362 entries / 32,363 fields, but 8,131 of the entries were prefab file paths dumped as `__pycr_type__` markers), 6 BLOCKED on pycrimson upstream bugs |
| **CATALOG** | 1 | **231-class** master index in `ENGINE_INTERNALS.md` (iter-22 corrected) |
| **GAP** | 2 | 109 tables in NattKh schema · **86 fully-covered post iters 70-82** + 23 with-gaps / 549 residual fields (iter 35-baseline was 584/41/68; this row's earlier "398/14" figure was stale) |
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
- **460 schema-listed `_camelCase` fields** across 19 *Info tables are
  not decoded by dmm-parser yet (was 584/41 at iter-35 baseline, then
  549/23 at iter-87 → 463/19 at iter-148 → 460/19 at iter-158). Per
  the 6-class taxonomy in
  `MOD_AUTHOR_GUIDE.md` § Residual coverage:
  - ~10 wrap-pattern tables blocked on alias-mechanism extension (Class 1)
  - 1 table needs real decoder work (Class 2: global_game_event_info)
  - Class 3 (semantic ambiguity) — **CLOSED iters 96-97**
  - faction_node_info residual (3 gaps inside sub-structs, Class 4+6)
  - 4 giants holding 411 of 460 = 89% (gimmick_info 153, character_info
    146, gimmick_group_info 45, stage_info 68 — Class 5)
  - ~5 tables have sub-struct fields needing decomposition (Class 6:
    interaction_info 28, field_info 5, action_point_info 2)
  See `docs/V3_1_LOOP_ASSESSMENT.md` for full state + recommended
  resumption priority order.

- **iter 70-148 closure work (resumed-loop, 2026-05-10)**: 86 closures
  shipped across 80+ iters (549 → 460 missing canonicals). 4 class-5
  tables fully closed: global_game_event_group_info, level_gimmick_scene_object_info,
  mission_info, tribe_info. faction_node_info: 0 → 90% (11 closures).
  field_info: 8% → 79% (17 closures). Class 3 (semantic ambiguity)
  fully closed via fixture data-range analysis (iters 96-97).
  Schema verifier: 90 of 109 schema-covered tables are now 100%
  canonical-aliased (up from 68 at iter-35 baseline). Best single-iter
  closures: 13 (mission_info iter 114), 10 (tribe_info iter 123), 9
  (tribe_info iter 121).

  **Methodology + tooling shipped**:
  - `docs/V3_1_CLOSURE_METHODOLOGY.md` (iter 113, refreshed iter 149)
    — 7 closure techniques in priority order + 5 anti-patterns + workflow
  - `scripts/find_singleton_closures.py` (iter 142) — surfaces
    type-singleton ship opportunities across all gap tables
  - `scripts/audit_manual_overrides.py` (iter 141) — validates 121
    MANUAL_OVERRIDES integrity (guards iter-122 silent-drop bug)
  - 5 design memos for the remaining closure classes:
    `V3_1_ALIAS_MECHANISM_EXTENSION_DESIGN.md` (iter 91, class 1),
    `V3_1_GLOBAL_GAME_EVENT_INFO_DECOMPOSE_DESIGN.md` (iter 92, class 2),
    `V3_1_FACTION_NODE_INFO_AUDIT.md` (iter 93, class 4),
    `V3_1_REMAINING_GAPS_MASTER_PLAN.md` (iter 94, coordination),
    `V3_1_SUB_STRUCT_DECOMPOSE_DESIGN.md` (iter 147, class 6)
  - Critical generator bug-fix (iter 122): `is_placeholder` filter
    was running BEFORE `MANUAL_OVERRIDES` check, silently dropping
    overrides for placeholder-pattern rust field names
    (`lookup_a`/`unk_*`/`flag_*`/`raw_*`)
- ~~**Havok binary layer (`.pac`/`.pacc`/`.pam`/`.pami`/`.pamlod`)** has~~
  ~~0% native parsing.~~ **Closed by the Havok+1.06 repair loop**
  (2026-05-11, iter 3-25; see new section below). All 12 Havok-layer
  extensions now Tier 1 classifier + round-trip. Full object-graph
  decode for `.hkx` (the `hkClass` family registry) still needs IDA RE.
- **`paatt_basedata.rs` 35 `_unkXXXX` fields** are the only remaining
  placeholders in the codebase, structurally unrecoverable without new
  evidence (PS5 demo binary or pycrimson .paatt support).

Tests: 562 passing throughout. No source-code regressions.

## Tags

Cron loop scheduled by id `15ac410b` at `*/1 * * * *`; auto-expires
2026-05-17. Cancel with CronDelete.

---

## Havok+1.06 Repair Loop (2026-05-11, iter 1-25)

**Trigger:** Crimson Desert 1.06 patch dropped overnight (Win build
`23174122` published 2026-05-11 00:52 AM). Initial diff showed 5
PABGH PARSE FAILED tables. User asked: fix the broken tables AND
parse the unparsed-Havok-layer extensions for mod tool authors.

**Cron job:** `af43c492` at `* * * * *`; auto-expires 2026-05-18.

### Critical fixes (1.06 regression triage)
- **5 broken pabgh-bounded tables fixed** (iter 1-2). All 5 had
  always-broken pabgh layouts the parser never supported — the
  1.06 diff merely exposed them. Added 3 new format variants to
  `src/binary/pabgh.rs`:
  - `U16CountU8Key` (5-byte entries) — fixes mercenarygroupinfo,
    mercenaryinfo, relationinfo
  - `U32CountU32KeyExtra4` (12-byte) — fixes characterappearanceindexinfo,
    `parent_id` field decoded (-2 = root sentinel, 86% of 8143 entries)
  - `U16CountExtra12` (16-byte) — fixes aieventtableinfo,
    `hash_key` + `aux_c` fields decoded (corrected iter-1 wrong-key bug)
- Round-trip preserved via `extra_bytes`. `parent_id()` + `aux_c()`
  accessors on `PabghEntry`. `pabgh parse fails: 5 → 0`.

### New format parsers (Tier 1, byte-perfect round-trip + Python bindings)

**Havok-layer family (12 extensions):**
| Ext | Module | Key | Iter |
|---|---|---|---|
| `.hkx` | `binary::hkx` | TAG0 magic + SDK version (Havok 2024.2.00) | 7 |
| `.pami` | `binary::pami` | XML `<StaticMeshInstance>` (was wrongly labeled "Animation index"; corrected) | 3 |
| `.pab`/`.paa`/`.pam`/`.pabc`/`.pabv`/`.pac`/`.pat`/`.papr` | `binary::par_resource` | "PAR " magic + per-ext version constants (9 versions catalogued) | 4-20 |
| `.motionblending` | `binary::motionblending` | Named-property records, **full corpus vocab decoded** (15 stable fields × 2 type tags across 1574 files) | 5, 11 |
| `.pamlod` | `binary::pamlod` | Static Mesh LOD (was wrongly labeled "Animation LOD"; corrected) | 6, 14 |
| `.paasmt` | `binary::paasmt` | Animation Set Matching Table | 8, 13 |
| `.paccd` | `binary::paccd` | Character Customization Data (corpus-wide constants + 0xFF "no-override" sentinel) | 8, 12 |

**Non-Havok extensions (+12 more):**
| Ext | Module | Iter |
|---|---|---|
| `.binarystring` | `binary::binarystring` | 17 |
| `.material`/`.technique`/`.mi`/`.spline`/`.spline2d`/`.pma` | `binary::xml_resource` (one module covers 6 formats) | 18, 20 |
| `.imp`/`.impostor` | `binary::impostor` | 19 |
| `.paprojdesc`/`.pashv` | reuses `binarystring` | 20-21 |
| `.paseqh`/`.paschedulectx` | `binary::count_record_table` (generic classifier) | 22 |

### Infrastructure
- **Partial-compression unblock** (iter 9): the `is_partial=true` flag
  was misnamed — actually meant "uncompressed passthrough". One-line
  fix in `src/binary/paz.rs::read_pack_file`. Unlocked extraction for
  29/30 `.pamlod`, 24/30 `.pac`, 13/30 `.pam` files that had been
  stuck behind `OSError: partial compression extraction not yet
  implemented`. (Remaining failures are a separate
  partial-compression-with-size-differential variant — IDA-RE blocked.)
- **3 generic classifiers** that proved high-leverage: `par_resource`
  (8 PAR-family ext), `xml_resource` (6 XML ext), `count_record_table`
  (2 record-table ext)
- **24 Python bindings** added: `parse_<ext>_bytes` + `serialize_<ext>`
  for every new format. Mod authors can `import dmm_parser` and call
  any of them directly.

### Diff tool extensions (`dmm-pabgb-aio/diff_pabgb_versions.py`)
- **3 new pabgh format variants** added to the Python `parse_pabgh`
  shim so the diff tool can read the previously-broken tables
- **Iteminfo typed diff** (iter 8) — reports by `string_key`
  ("Poison_Arrow") instead of raw hex keys; catches same-byte-size
  field-value changes the byte-size diff missed
- **Sequential pabgb diff** (no pabgh sister) — typed-parse via
  `parse_table(name, body, None)`
- **Tier 2 file diff** (iter 24) — 24 Havok-layer + XML + small-binary
  extensions now diffed file-by-file
- **Typed field deltas** (iter 25) — per-format semantic deltas:
  `.pami.mesh_paths` add/remove, `.pamlod.lod_count` changes,
  `.paccd.no_override_byte_count`, PAR `version` "FORMAT CHANGE"
  warnings, `.hkx.sdk_version` SDK upgrade detection

### Documentation
- `docs/_HAVOK_TABLES_WORKPLAN.md` — full per-iter log
- `docs/api.md` — new "Havok-Layer Formats (Tier 1)" section with
  JSON-shape examples for every new parser
- `docs/MOD_AUTHOR_GUIDE.md` §13 — single-page summary table of all
  12 extensions + mod-relevant fields
- `docs/BINARY_FORMATS.md` — every ⚠️ → Tier 1 promotion tracked
- `docs/ENGINE_INTERNALS.md` — Havok status refresh

### Tests + corpus validation
- Tests: **567 → 635** (+68 new tests across 25 iters), all passing
- Real-world corpus validation for every shipped format (20-1641
  samples per format depending on install footprint)
- Caught 2 wrong-field-name bugs from earlier "1-sample" ships
  via post-unblock corpus audits (iter 11 for motionblending,
  iter 14 for pamlod)

### Known blockers (deferred, all need IDA RE)
- Partial-compression-with-size-differential variant (~17 .pam +
  ~6 .pac files still inextractable)
- `.motionblending` per-tag typed value decode
- `.paccd` per-slider semantic mapping
- `.hkx` Havok class registry / object-graph decode
- `.questgaugecount`, `.pathc`, `.paproj`, `.pai` per-record decode
