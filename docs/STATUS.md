# dmm-parser status & handoff

**Last updated**: 2026-04-29
**Repo**: https://github.com/DatGuySnowfox/dmm-parser (fork of exodiaprivate-eng/dmm-parser)
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

### Polymorphic family decoders
| Family | Status | Tables that consume it |
|---|---|---|
| **GameCondition** | ✅ 100% (Decoded\|Raw enum, commit `5160cdd`) | ConditionInfo (Tier 1, commit `9f1be1d`) |
| **GlobalGameEventExecuteData** | ✅ 100% (Absent\|Present\|Raw enum, commit `4b30791`) | GlobalGameEventInfo (Tier 1) |
| BuffData | ✅ shipped (per buff_data.rs) | SkillInfo, CharacterChangeInfo |
| BranchConditionData | ✅ shipped | (used inside GameCondition tree) |
| ConditionDataStageChart | ✅ shipped | (used inside GameCondition tree) |
| ConditionGimmickData | ✅ shipped | (used inside GameCondition tree) |
| ScheduleCompleteConditionData | ✅ shipped | (used inside GameCondition tree) |
| GlobalEffectConditionData | ✅ shipped | (used inside GameCondition tree) |
| MiniGameData | ✅ shipped | MiniGameDataInfo |
| GameExpression / IVariantItem | ✅ shipped (inside StageChart) | (used inside GameCondition tree) |
| **GameEventHandler** | ❌ **next target** | GameEventHandlerInfo |
| EffectData | ❌ pending | EffectInfo |
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

## What just shipped (DatGuySnowfox fork, merged upstream + additions)

```
85a3a49  normalize line endings: add .gitattributes, convert CRLF → LF
950eed4  merge upstream/main: ConditionData improvements + GlobalGameEventExecuteData
c0fe23d  docs: remove duplicate horizontal rule in api.md
97f4066  fix gitignore, add BuffInfo/SkillInfo Python bindings, update docs
```

### Fork additions on top of upstream
- **Python bindings for BuffInfo and SkillInfo** — `parse_buffinfo_from_file`,
  `parse_skillinfo_from_file`, `serialize_*`, `write_*_to_file` all registered
  in `src/python.rs`. Both tables fully round-trip; `buff_level_list` /
  `_buff_data_list_b64` remain base64 until BuffData gets a JSON shim.
- **PALOC (localization) bindings** — `parse_paloc_bytes` / `serialize_paloc`
  already existed; now documented in `docs/api.md`.
- **`docs/api.md` expanded** — PALOC, SkillInfo, BuffInfo sections added with
  full field tables and nested struct docs (GraphData, ResourceStat, ResourceItem).
- **`.gitignore` fixed** — previous file had the shell heredoc command baked in
  as literal text; replaced with a proper ignore list covering `target/`, `.venv/`,
  `pabgb/`, `*.paz`/`*.pamt`/`*.papgt`/`*.pabgh`, `out/`, `tools/`, `node_modules/`.
- **`.gitattributes` added** — enforces LF line endings repo-wide; renormalized
  all 278 CRLF source files. Fixes spurious merge conflicts with upstream.
- **Movement mod fixed** — `tools/generate_movement_mod.py` now catches horse
  movement skills (`Skill_HorseStamina`, `Skill_Mount_Ing`, `SKill_HorseFastStart`,
  `Skill_HorseDrift`, `SKill_HorseLateralMove`, etc.) that were missed because
  `"horse"` and `"mount"` were absent from `MOVEMENT_KEYWORDS`. Horse attacks
  (`HorseKick`, `HorsePawStamp`, `HorseRushAttack`, `Mount_Attack_*`) remain
  excluded. All 5 mod variants regenerated (83 intents each, up from ~70).

### From upstream (merged in `950eed4`)
```
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

GameCondition went from 13.4% → 100% across the first 7 upstream commits.
GlobalGameEventInfo Tier 2 → Tier 1 in the latest upstream commit (80/80 entries
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
1. **GameEventHandler family** (task #97). GameEventHandlerInfo
   consumer. Likely uses bespoke-dispatcher pattern like GameCondition;
   reuse the playbook.
2. **EffectData family**. EffectInfo consumer.
3. **Per-sub_tag typed payloads inside GlobalGameEventExecuteData**
   (task #96 follow-up). The `Present { sub_tag, body: Vec<u8> }` shape
   is shipped; full body fields per sub_tag (sub_141155000 / sub_141155300
   recipes) are mechanical follow-up work.
4. **TriggerEventHandler family** (task #95). DEFERRED — uses
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
