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


## Done
- [x] SCRIPT-DOC: scripts/README.md — 2026-05-10 13:58 SUCCESS. Audited all 6 .py files in scripts/. Created scripts/README.md documenting each: generate_v3_1_aliases.py (active, schema-grounded), verify_v3_1_against_schema.py (active, audit), harvest_reflection_schema.py (active, pycrimson aggregator), add_json_roundtrip.py (historical), add_cdmtl_headers.py (historical bulk), deploy_cdmtl_license.py (occasional release tooling). Includes naming conventions + how to add a new script.

**Resumed-loop phase complete (8/8 tasks). All loop work exhausted again.**

- [x] HIDDEN-WRAP-DISCOVERY: action_point_info revealed nested-sub-struct wrap pattern — 2026-05-10 14:51 SUCCESS. Top-level rust_count (6) == schema_count (6) misled iter 40 classifier into placing it in GENUINE pattern. Actual structure: `action_point: ActionPoint { field_a: u32, block_a: [u8; 24] }` hides both `_actionYaw` and `_actionPosition` canonicals inside the typed sub-struct. Wrap pattern can hide INSIDE typed sub-structs. Adjusted realistic decoder workload estimate from ~200-300 to ~150-250 fields. Documented in V3_1_DECODER_GAPS.md "Wrap pattern can also hide INSIDE typed sub-structs" subsection.

**Loop stopped at iter 43 (2026-05-10 14:52)** — user invoked stop to install IDA plugins.

**Loop resumed at iter 44 (2026-05-10 18:30)** — user installed 4 IDA plugins (function-string-associate, AutoRE, HexRaysPyTools, HRDevHelper); cron `a51bccde` re-armed.

- [x] REGISTRY-EXTEND: vehicle_info + action_point_info added — 2026-05-10 18:55 SUCCESS. pa::VehicleInfo at 0x144afb160 → sub_1410CB230 (0x530 = 1.3KB). pa::ActionPointInfo at 0x144ab0380 → sub_1410A16D0 (0xf7 = 247B — suspiciously small; possibly a dispatch wrapper, real reader may be called inside). Registry now covers 12 of 27 gap tables.
- [x] REGISTRY-EXTEND: faction_node_info + region_info added — 2026-05-10 18:52 SUCCESS. pa::FactionNodeInfo at 0x144ac1af0 → sub_1410AAE90 (0x5df = 1.5KB). pa::RegionInfo at 0x144aeb3b0 → sub_1410C1E70 (0x403 = 1.0KB). Registry now covers 10 of 27 gap tables.
- [x] REGISTRY-EXTEND: gimmick_group_info + field_info added — 2026-05-10 18:49 SUCCESS. pa::GimmickGroupInfo at 0x144accb90 → sub_1410B0D60 (0xadd = 2.8KB). pa::FieldInfo at 0x144ac6a60 → sub_1410AD060 (0x4dd = 1.2KB). Registry now covers 8 of 27 gap tables.
- [x] REGISTRY-EXTEND: character_info + stage_info added to master registry — 2026-05-10 18:46 SUCCESS. Located: pa::CharacterInfo typeinfo at 0x144ab3c20 → reader sub_1410A3A40 (0x2238 = 8.7KB, the largest reader by far — matches its 164-field count). pa::StageInfo typeinfo at 0x144af38e0 → reader sub_1410C76E0 (0xd90 = 3.5KB). Registry now has 6 of 27 worst gap tables mapped.
- [x] CLOSURE-VALIDATION + REGISTRY: knowledge_info validated + master typeinfo registry started — 2026-05-10 18:43 SUCCESS. Located pa::KnowledgeInfo typeinfo at 0x144ac9fb0 → reader sub_1410AFE20 (size 0x57a). Plus discovered gimmick_info has no bare typeinfo string (Tier-1.5 special — uses sub_1410E6FC0 per existing dmm-parser docstring). Built "Master typeinfo + record-reader registry" table in V3_1_DECODER_GAPS.md cataloguing 4 validated tables + gimmick_info special case + listing the 27 not-yet-registered tables for future iters. Provides skip-the-typeinfo-find shortcut for all future closure sessions.
- [x] CLOSURE-VALIDATION: mission_info workflow validated — 2026-05-10 18:39 SUCCESS. Located pa::MissionInfo typeinfo at 0x144add820 → record reader sub_1410B9BA0 (size 0x6ea = 1.7KB, 40 fields). Schema breakdown: 14 direct_15B + 10 reader_4B + 7 unknown + 4 reader_8B + 3 direct_u16 + 1 each direct_u32/reader_1B. Workflow-ready. **3rd table workflow-validated.**
- [x] CLOSURE-VALIDATION: interaction_info workflow validated — 2026-05-10 18:36 SUCCESS. Located pa::InteractionInfo typeinfo at 0x144ac4060 (single xref) → real record reader sub_1410AC290 (size 0x586 = 1.4KB). Schema breakdown: 37 fields total (22 direct_u8 + 5 reader_4B + 6 unknown + 1 each of direct_u32/array_or_complex/reader_8B/reader_2B). Wire reader dense pattern matches schema type tally. Per-field semantic naming deferred to focused decoder-writing session with HexRaysPyTools struct rebuild + game-data inspection. Documented in V3_1_DECODER_GAPS.md.
- [x] MOD-AUTHOR-DOC-CATCHUP: refresh MOD_AUTHOR_GUIDE.md with v3.1 surface — 2026-05-10 18:32 SUCCESS. Added "Section 0: Authoring against canonical Pearl Abyss field names (v3.1 surface)" at top of MOD_AUTHOR_GUIDE.md catching up on Session 28 + 1-min-loop work that was missed in earlier doc passes. Covers: why v3.1 exists, coverage table (109 schema-verified + 13 fallback + 27 closures + iteminfo gap + reflection harvest), shape parameter usage, when to use v3.1 vs v3, where to find per-table canonical-field catalogs, PA-internal typo preservation note. Major mod-author-facing surface refresh.
- [x] CLOSURE-VALIDATION: tribe_info workflow validated via IDA + type-count cross-check — 2026-05-10 14:48 SUCCESS. Decompiled pa::TribeInfo record reader sub_1410C8A20 (Win typeinfo at 0x144af6090, single xref). Wire-read sequence extracted in source order. Type counts cross-validate PERFECTLY against NattKh schema: 13 direct_u8 / 9 direct_u32 / 4 reader_4B / 1 direct_u64 — exact match. Closure workflow validated structurally; per-field semantic naming (which unk_XX becomes which canonical) requires deliberate per-position review with IDA plugins installed. Documented in V3_1_DECODER_GAPS.md as the gap-table workflow-readiness smoke test.
- [x] PER-TABLE-PLAN: tribe_info closure plan — 2026-05-10 14:45 SUCCESS. tribe_info struct revealed as having 26 `unk_XX` placeholder fields named by byte offset (unk_22, unk_24, unk_28, ...) matching the 26 missing canonicals 1:1. The CLOSURE PATH is documented as the template for genuine-pattern tables: find pa::TribeInfo typeinfo → record reader → walk reads → pair offset literals with canonical declaration order → rename unk_XX → canonical_snake → ship aliases. Real implementation requires hours of IDA + careful test runs; this iter just lays the closure plan in V3_1_DECODER_GAPS.md.
- [x] WRAPPER-PATTERN-FULL-SURVEY: full quantitative split across all 27 gap tables — 2026-05-10 14:42 SUCCESS. Programmatic classification: 19 tables fit WRAP pattern (rust > schema, ~452 missing), 8 tables fit GENUINE pattern (rust ≤ schema, ~105 truly missing). Worst genuine-pattern: gimmick_info 153 (Tier-1.5 typed-prefix + opaque blob), interaction_info 28, tribe_info 26, mission_info 25. Worst wrap-pattern: character_info (27 rust-excess fields), vehicle_info (17), stage_info (13). **Real decoder-writing workload likely ~200-300 fields max, not 557** — the rest is Rust refactor work (collapse unrolled fields into typed sub-structs). Documented "Full quantitative split" subsection in V3_1_DECODER_GAPS.md.
- [x] WRAPPER-PATTERN-INSIGHT: 8 of 10 small-gap tables fit wrapper-vs-unrolled pattern — 2026-05-10 14:39 SUCCESS. Spot-check survey reveals NattKh schema "single missing canonical" frequently corresponds to a WRAPPER name for the 2-3 extra rust fields in those tables (e.g. `_characterChangeFilter` wraps `name_list` + `hash_lookup_list` + `trailing_id`). Implication: many of the 557 remaining "missing fields" are actually wrapper-vs-unrolled mismatches, not new-decoder needs. Real decoder-writing workload is much smaller than the headline number suggests. Documented "Wrapper-vs-unrolled" section in V3_1_DECODER_GAPS.md.
- [x] DOCS-PLUGINS: IDA plugin reference for future decoder-gap closure — 2026-05-10 14:36 SUCCESS. Added "Recommended IDA plugins" section to T0_AUDIT_TRACKING.md cataloging 6 plugins per the user's research request: ClassInformer (RTTI/vtable), IDA-VTableExplorer (vtable browser), FunctionStringAssociate (string xref tagging), AutoRE (auto-rename from debug strings), HexRaysPyTools (struct rebuild + class hierarchy), HRDevHelper (ctree inspector). Includes per-plugin source URL, IDA-version compatibility caveats (esp. HexRaysPyTools breaking on IDA 9.0), and 4-step recommended workflow with these installed. Force-multiplier for closing the 557 remaining decoder gaps.
- [x] STRUCTURAL-DIVERGENCE-SCAN: full 1-to-N audit across all gap tables — 2026-05-10 14:33 SUCCESS. Programmatically scanned every gap table for unrolled-array patterns (`<base>_<num>`) matching a single canonical. Result: only 2 such divergences exist across the entire codebase: `ally_group_info.relation_type_list_*` (7 fields → `_relationTypeList`) and `elemental_material_info.flag_*` (8 fields → `_flag`). Both documented. Confirms structural divergence is rare; most remaining 557 gaps are genuine missing fields.
- [x] DECODE-CLOSURE-COMPLEX: ally_group_info structural divergence finding — 2026-05-10 14:30 INSIGHT. The 1 missing canonical `_relationTypeList` corresponds to SEVEN unrolled rust struct fields (`relation_type_list_0` through `relation_type_list_6`). 1-to-N mapping not expressible by current v3.1 alias mechanism. Documented in V3_1_DECODER_GAPS.md "Known structural divergences" section. Future work: either Rust struct refactor or alias mechanism extension.
- [x] DECODE-CLOSURE-EXHAUSTION-AUDIT: auto-closure analysis done — 2026-05-10 14:27 SUCCESS. Iter 35 audited remaining 557 missing fields. Heuristics (Jaccard ≥0.75) found 4 candidates, all too uncertain to ship without IDA. Per-table breakdown: 272 missing canonicals correspond to rust placeholder fields (lookup_NN, field_X, raw_Y) needing IDA wire-position trace; 208 missing are real-named unaliased rust fields (likely actual additions or divergent forms requiring per-field IDA verification); 77 are genuinely missing from rust struct entirely. Documented in V3_1_DECODER_GAPS.md "Auto-closure analysis" section. **Auto-closure phase complete — 27 fields shipped, further closure requires IDA decompile per table.**
- [x] DECODE-CLOSURE-BATCH-3: 6 one-of-each high-confidence pairings — 2026-05-10 14:23 SUCCESS. Used count-match heuristic + manual review to find 6 more divergences. Includes 2 more PA-internal typos: `_fishSummonTimeFrquencyType` (Frquency, missing 'e') and `_radgollEquipTableGroupDataList` (radgoll vs ragdoll). Tables: frame_event_attr_group_info, game_event_handler_info, item_use_info, terrain_region_auto_spawn_info, equip_info, special_mode_info. Aggregate: verified 1146→1152 (+6), missing 563→557 (-6). Skipped 4 false-positive pairings from the heuristic (action_point_info, global_stage_sequencer_info, knowledge_info — semantically wrong, would need IDA verification).
- [x] DECODE-CLOSURE-BATCH-2: 9 fuzzy-normalized divergences fixed across 8 tables — 2026-05-10 14:18 SUCCESS. Aggressive fuzzy matcher (normalize-equal across underscore + case, no space relaxation) found 9 more divergences. Includes 2 PA-INTERNAL TYPOS captured as-is in canonical (`_questGroupkey` lowercase k, `_regionEnterknowledgeInfoList` lowercase k — these are quirks in PA's own naming). Tables: faction_node_spawn_info, interaction_info, inventory_info, knowledge_info, platform_achievement_info, region_info (×2), store_info, terrain_region_auto_spawn_info. Aggregate: verified 1137→1146 (+9), missing 572→563 (-9).
- [x] DECODE-CLOSURE-BATCH: 10 acronym/plural divergences fixed across 9 tables — 2026-05-10 14:12 SUCCESS. Auto-discovery script found 10 candidates where Rust snake matched canonical via reverse camel-to-snake but mechanical forward translation diverged (mostly UI/ID/XXX/Dev acronym casing). All added to MANUAL_OVERRIDES + regenerated. Tables impacted: knowledge_info, knowledge_group_info, mini_game_data_info, vehicle_info, status_info, elemental_material_info, faction_node_info, spawning_pool_auto_spawn_info, global_game_event_info. Aggregate: verified 1127→1137 (+10), missing 582→572 (-10).
- [x] DECODE-CLOSURE: effect_info FIRST DECODER GAP CLOSED — 2026-05-10 14:08 SUCCESS. Win-IDA decompile of sub_1410A8670 (real EffectInfo record reader, found via pa::EffectInfo typeinfo at 0x144abce30 → 1 xref) revealed both "missing" fields ALREADY exist in the Rust struct: `effect_data` (mismechanically translates to `_effectData` but schema has `_effectDataList`) and `mesh_effect_data` (similar). Added MANUAL_OVERRIDES dict to scripts/generate_v3_1_aliases.py for cases where Rust snake doesn't mechanically translate to schema canonical. Regenerated: effect_info now 8 verified / 0 missing / 0 mismatch (was 6/2/0). Aggregate: verified 1125 → 1127, missing 584 → 582. **First actual decoder gap closure via the loop.** Sets template for the other 14 gap tables — many likely have similar "field exists but mechanical name diverges from canonical" cases.
- [x] DECODE-ATTEMPT: effect_info gap closure attempt — 2026-05-10 14:02 INSIGHT-NOT-CLOSURE. Attempted to decode the 2 effect_info missing fields via the schema's parser fn pointer (0x14103c140). Decompile revealed sub_14103BE80 — the GENERIC table-loader, not the per-table record parser. Iter 27's "one decompile per table" claim is corrected: schema fn = generic loader, per-table record parser is one vtable indirection deeper. Updated V3_1_DECODER_GAPS.md "Per-table workflow" with corrected steps (find pa::<TableName> typeinfo → vtable → read-from-bytes virtual method → that's the real parser). Decoder gap closure remains feasible but is hours-per-table IDA work, not 1-min-loop-amenable. No code changes shipped this iter.

- [x] HONEST-DOC: Known Limitations section in STATUS.md — 2026-05-10 13:54 SUCCESS. Comprehensive honest accounting of what "100% game breakdown" means in practice. Three sections: What we have (structural surfaces fully catalogued), What we DON'T have (out-of-control: PS5 demo binary, pycrimson upstream fixes, embedded data class names), What we DON'T have (in-scope: 398 missing decoder fields per priority worklist + Havok Layer B + iteminfo v3.1 surface + ~50 long-tail extensions). Frames the loop's contribution as "making every remaining gap visible and actionable" — future sessions can pick from priority list directly.
- [x] DECODER-GAP-PRIORITY: cluster analysis + priority worklist — 2026-05-10 13:50 SUCCESS. Discovered each table's missing fields all route through a SINGLE parser fn (zero cross-table reuse), so per-table workflow: ONE IDA decompile reveals ALL gaps for that table. Added "Decoder-fn cluster analysis" section to V3_1_DECODER_GAPS.md with per-table fn pointer + type distribution + per-table workflow + quick-win target order (smallest tables first: effect_info 2, action_point_info 2, vehicle_info 3, knowledge_info 3, global_game_event_info 4, region_info 4).
- [x] LONG-TAIL-2: domain-suffix/prefix tally + subsystem coverage — 2026-05-10 13:46 SUCCESS. Tallied 231 reflection classes by 25 suffixes + 22 prefixes. Top categories: *Component (61), *Data (32), Game* (20), Custom* (14), Spline* (12), Material* (10), Pa* (10), Resource* (9), Emitter* (9), Sequencer* (8), Camera* (7), Audio* (6). **Notable discoveries**: 10 Houdini* classes confirm SideFX Houdini integration for procedural content; MassPlacement system for instanced foliage/props; NPCSchedule for NPC patrol routing; PositionConstraintMotor + AttachingClothToMesh for physics. Documented in ENGINE_INTERNALS.md "Engine subsystem coverage" section.
- [x] CROSS-VALIDATE: NattKh schema ↔ pycrimson reflection cross-source check — 2026-05-10 13:42 SUCCESS. Aggregate: 81/605 reflection fields (13.4%) appear in NattKh schema. Domains mostly disjoint by design (reflection = wrappers/components, NattKh = pabgb tables); overlap concentrated where they share primitives. Highest-confidence double-confirmed classes: RagdollConstraintData 83%, SplinePoint3D 50%, GameData_TimelineEvent_EquipmentInOut 46%, ScenePostProcessing 40%, AtmosphereConstant 33%. Documented in ENGINE_INTERNALS.md "Cross-source validation" section + identified pycrimson-only classes (Emitter*Data, SequencerGamePlayData_*, Camera*, etc.) where pycrimson is the sole canonical-name source.
- [x] TABLE-COMPLETION: 68 fully-decoded tables canonical-field catalogs — 2026-05-10 13:38 SUCCESS. Inserted catalog blocks at top of all 68 fully-decoded *Info table info.rs files. Pattern: 100% ✅ (all fields decoded by dmm-parser, schema-confirmed). Combined with the 15 GAP table catalogs from iter 10/11, EVERY schema-listed *Info table now has a self-documenting canonical-field reference at the top of its info.rs (83 of 122 tables; the remaining 39 are tables not in NattKh schema).
- [x] EXTENSION-MAP: complete extension reference in BINARY_FORMATS.md — 2026-05-10 13:34 SUCCESS. Added comprehensive 86-extension table at top of `BINARY_FORMATS.md` ("Extension reference (all 86 PA file formats)"). Categories: archive/metadata layer, localization, game-data files, reflection (pycrimson territory), Havok layer, texture/mesh assets, audio, navigation, save, misc, third-party. Status legend (✅ ✓ 📚 🚫 ⚠️) tied to handler. Summary: 8 ✅ fully parsed, 11 🟡 partial, 5 📚 pycrimson-catalogued, 14 🚫 pycrimson-blocked, ~50 ⚠️ long-tail unparsed.
- [x] LONG-TAIL: Pattern-analyze 8,233 uncategorized "classes" — 2026-05-10 13:30 CRITICAL CORRECTION FOUND. Discovered the 8,233 entries are ~8,131 prefab FILE PATHS (e.g. `/object/cd_gimmick/...prefab`) that pycrimson dumps as `__pycr_type__` for component-array instances, NOT class names. **Real PA reflection class count is 231 (with 938 canonical fields)**, not 8,362 / 32,363. Corrected the headline numbers in ENGINE_INTERNALS.md Master class index + STATUS.md Game Surface Coverage Map. Cron `15ac410b` was previously cancelled; re-armed as `ed2ab1e1` to continue work per user direction.
- [x] DOCS: SHIPPED.md cross-reference index — 2026-05-10 13:18 SUCCESS. Wrote `docs/SHIPPED.md` (placed in docs/ rather than root per project CLAUDE.md "NEVER save working files, text/mds, or tests to the root folder"). Catalogues all 31 commits from Session 28 + 19 loop iters with cross-links to STATUS / V3_1_README / ENGINE_INTERNALS / BINARY_FORMATS / per-table info.rs. Phase totals + "what this run did NOT solve" honest accounting included.

**DOCS phase complete (5/5).** **Loop phases all complete: REFLECT 10/10 · CATALOG 1/1 · GAP 15/15 · DESC 5/5 · VARIANT 4/4 · DOCS 5/5.**

**🛑 Loop stopped at iter 20 (2026-05-10 13:21).** Queue empty + auto-generation
exhausted (every loop-amenable task done). Cron `15ac410b` cancelled via
CronDelete. PushNotification sent.

**iter 21 — late-fire confirms stop (2026-05-10 13:25).** A queued cron firing
landed after CronDelete and used the iter as a final negative-result probe:
tested pycrimson on 9 unannotated extensions (`.pasound`, `.seqmt`,
`.linkedsceneobject`, `.binarygimmick`, `.binarygimmickcacheddata`,
`.binarygimmickframeevent`, `.paacdesc`, `.pas`, `.ies`). All 9 fail with
the same parc-header buffer-underflow as `.palevel`/`.pae`. Confirms
pycrimson only handles the explicitly (reflection)-annotated formats;
no hidden wins available. Files NOT extracted to disk for the failures.
Loop confirmed stopped, no further re-arming.

What remains is NOT 1-min-loop-amenable:
- 398 missing fields needing actual decoder Rust code (per *Info table; ~hours each)
- Havok Layer B parser implementation (multi-day project)
- 22 _unkXXXX in paatt_basedata.rs (triple-blocked from name verification per Session 28 iter 13)
- 6 reflection formats blocked on pycrimson upstream fixes (out of our control)
- Extending v3.1 alias generator to walk src/item_info/ (substantive change)
- Per-format research for non-reflection PA extensions (.pbd, .pcg, .material, etc.)

To resume: re-arm cron with new tasks via `_BREAKDOWN_WORKPLAN.md` queue
edit + CronCreate, or pick up substantive work directly without loop wrapping.

- [x] DOCS: opaque-field audit in STATUS.md — 2026-05-10 13:15 SUCCESS. Whole-tree scan for `_unkXXXX` + `Vec<u8>` opaques. Result: 35 _unkXXXX in paatt_basedata.rs only (zero elsewhere); 30 Vec<u8> fields classified into 9 decoder-gaps (paseq/paseqc/pastage/paschedule/paschedulepath/paatt-bodies), 10 raw-by-design (audio/texture/string-pools), 11 file-format tables (paac/paatt/pamhc not yet field-decoded). Documented as audit subsection in STATUS.md "Current state".
- [x] DOCS: Layer B Havok binary reference in ENGINE_INTERNALS.md — 2026-05-10 13:12 SUCCESS. Added comprehensive Layer B section: extension family table (.hkx/.pac/.pacc/.pam/.pami/.pamlod/.skel/.mesh), Havok packfile detection signatures, all known hka/hkx/hknp/hcl class names from prior IDA scan, Layer A → Layer B bridge map (which PA-side reflection fields resolve to which Havok files), what a Layer B implementation would need, and why this isn't blocking current mod work.
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
