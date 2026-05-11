# Closing the Remaining 549 v3.1 Gaps — Master Plan

**Status**: Coordination doc.
**Author**: 1-min loop, iter 94 (2026-05-10).
**Audience**: Implementer who wants a single entry point for finishing
the v3.1 surface coverage. Picks priorities, estimates effort, points to
the per-class design memos.

## Where we are (iter 148 update)

```
126 dmm-parser tables
 └─ 109 in NattKh canonical schema
     ├─  90 fully-aliased (100% _camelCase coverage)   [+4 since iter 94]
     └─  19 with-gaps  →  460 missing canonicals total  [-89 since iter 94]
```

**Iters 124-147 progress** (post-iter-124 baseline):
- iters 125-126: field_info 8 closures (+ singleton _regionBitmapPositionInfo).
- iters 127-133+139: faction_node_info 11 closures (now 29/31 = 94%).
- iters 135-137: stage_info 4 closures (now 14/82 = 17%).
- iter 141: persisted MANUAL_OVERRIDES audit as scripts/audit_manual_overrides.py.
- iter 142: persisted singleton-scan as scripts/find_singleton_closures.py.
- iter 147: V3_1_SUB_STRUCT_DECOMPOSE_DESIGN.md (Class 6 design).
- Total iters-124-158 delta: -27 missing (487 → 460).

**Resumed-loop progress** (iters 96-123):
- iters 96-97 (class 3 closed): closed `_executePercent` and
  `_onDiscoverOnlyEnable` via fixture data-range analysis. Both tables
  100% covered. Class 3 of 4-class taxonomy fully resolved.
- iters 99-106 (field_info partial): 9 closures via type-match + data-pattern
  + IDA wire-read disambiguation. Now 11/24.
- iter 108 (tribe_info opening): `_tribeNameForEditor` (singleton CString).
- iter 110 (tribe_info CArray): `_tamedSkillList` (only CArray<u32>).
- iter 112 (mission_info 4-shot): 4 LocalizableString labels via IDA + setter
  order — first big N-to-N closure.
- iter 113: V3_1_CLOSURE_METHODOLOGY.md force-multiplier guide written.
- iter 114 (mission_info 13-shot): biggest single-iter closure. 13 direct_15B
  u8 booleans via type-unique wire-contiguous run + setter order.
- iter 115: tribe_info `_tribeMassLevel` singleton.
- iter 116 (mission_info 3-shot): 2 direct_u16 + 1 direct_u32 via type-match
  + positional adjacency.
- iter 119 (mission_info 2-shot): 2 None-typed CArray closures.
- iter 120 (mission_info FULL): final 3 reader_4B via within-type-group rule
  with non-contiguous-but-order-preserving reads. **mission_info 100% (40/40)**.
- iter 121 (tribe_info 9-shot): 9-consecutive direct_u8 run via the iter-120
  technique. Crossed below 500 missing-canonicals threshold.
- iter 122 (tribe_info 4-shot + bug fix): discovered + fixed critical generator
  bug (`is_placeholder` filter ran BEFORE `MANUAL_OVERRIDES` check, silently
  dropping overrides for placeholder-pattern rust field names). 4 closures.
- iter 123 (tribe_info FULL): 9 direct_u32 via within-type-group rule + final
  singleton. **tribe_info 100% (29/29)**.

4 class-5 tables fully closed this session: global_game_event_group_info,
level_gimmick_scene_object_info, mission_info, tribe_info.

The 21 with-gaps tables fall into 4 active closure classes (per
`MOD_AUTHOR_GUIDE.md` § Residual coverage; class 3 is now CLOSED).
Each class has a dedicated design memo. This doc is the index +
priority recommendation.

## Class summary + memo links

| Class | # tables | # gaps | Closure type | Design memo |
|---|---|---|---|---|
| 1. **1-to-N alias mechanism extension** | ~10 | ~10 | Mechanism + per-table aliases | `V3_1_ALIAS_MECHANISM_EXTENSION_DESIGN.md` |
| 2. **Real decoder work — struct decompose** | 1 (`global_game_event_info`) | 3 | Decompose `execute_data` polymorphic wrapper | `V3_1_GLOBAL_GAME_EVENT_INFO_DECOMPOSE_DESIGN.md` |
| 3. **Semantic ambiguity** | ~~2~~ | ~~2~~ | **CLOSED iters 96-97** via fixture data-range analysis | (resolution recipes inline above) |
| 4. **Larger un-audited table** | 1 (`faction_node_info` residual) | 3 | Sub-struct decomposition (overlaps Class 6) | `V3_1_FACTION_NODE_INFO_AUDIT.md` |
| 5. **The 4 giants** | 4 | 412 (gimmick_info 153 + character_info 146 + gimmick_group_info 45 + stage_info 68) | Per-table audit + multiple sub-classes | TBD per-table |
| 6. **Sub-struct decompose** | 5 (interaction_info, field_info, action_point_info, faction_node_info residual, plus all giants) | 38+ across smalls + most of class 5 | Path A (flatten) or Path B (nested-path mech) | `V3_1_SUB_STRUCT_DECOMPOSE_DESIGN.md` (iter 147) |

**Total reachable via classes 1+2+4+6 (small tables)**: ~52 closures.
**Class 5 (the 4 giants) dominates the residual**: 411 of 460 = 89%
of remaining gaps. Those tables likely benefit from a mix of class 1,
2, and 6 techniques as their internal structure varies.

## Recommended priority order

### Priority A — alias mechanism extension (class 1)

**Effort**: ~1 day for an experienced rust dev who reads the design
memo cold. Mostly mechanical: enum + dispatcher + regen scripts +
roll out 11 wrap entries.

**Impact**: 10-11 closures across the 11 wrap-pattern tables. Permanently
removes one entire gap class so future tables with the same pattern get
auto-closed by the generator.

**Why first**: lowest-risk shipping path. No IDA work. Backward-compatible
with all existing alias entries. Roundtrip-test-validatable per memo's
incremental plan. Unblocks future audits of the class-5 tables (most of
which probably have wrap-pattern gaps too).

### Priority B — class-3 ambiguity resolution

**Effort**: ~30 min per ambiguity. Needs the function-string-associate
IDA plugin output to identify which u8/u64 the canonical name's error
string references.

**Impact**: 2 closures. But also unblocks the 2 affected tables to fully-
covered status, which is a UX win for mod authors targeting them.

**Why second**: very small, very fast. Worth doing while context is
fresh from priority A.

The 2 ambiguities (no dedicated memo because the work is small enough
to inline here):

#### `_onDiscoverOnlyEnable` (level_gimmick_scene_object_info)

- Maps to one of `unk_new_u8_a` (offset 77) or `unk_new_u8_b` (offset 78).
- Per iter 80: schema metadata's `s` field for the canonical
  (`0x144943480`) points to a setter-dispatch table, not a string.
  Direct string xref insufficient.
- Resolution: use function-string-associate to tag setter functions
  with the error-string fragments they reference; then walk back from
  `_onDiscoverOnlyEnable`'s setter (entry in dispatch table at index N)
  to find which struct offset it writes to. Match offset to rust field.

#### `_executePercent` (global_game_event_group_info)

- Maps to one of `unk_b: u64` or `unk_c: u64` per iter 81/82 analysis.
- Both at adjacent offsets inside `sub_1410B03A0` (40-byte sub-struct
  at offset 24 of the per-record reader).
- Easier path than ambiguity 1: range-analyze the actual data. Open
  `globalgameeventgroup.pabgb` fixture, sample all entries, look at
  which u64 has values in 0-100 (or 0-10000 for percent×100) range.
  The other is presumably a hash or large counter.

### Priority C — global_game_event_info decompose (class 2)

**Effort**: ~half-day. Per the dedicated memo: re-IDA confirm wire
layout, write 3 typed readers, trim execute_data wrapper, fixtures,
aliases, verifier rerun.

**Impact**: 3 closures. Also clean structural improvement to the table
(execute_data was always a confusing wrapper).

**Why third**: small, well-scoped, has a dedicated memo. The order
A → C is fine; C doesn't depend on A.

### Priority D — faction_node_info audit (class 4)

**Effort**: ~1 day for the per-record reader walk + composite-slot
decomposition. Plus another ~half-day for the rust struct cleanup
(rename `unknown_a/b/c/d` → semantic names).

**Impact**: 14 closures + meaningful struct-quality improvement.

**Why fourth**: biggest single-table win, but also the most IDA-heavy.
Wait until A-C are done so context is fully warm and the alias
mechanism is available for the wrap-pattern fields the audit will
likely surface.

### Priority E — class 5 sweep

**Effort**: per-table; mostly proportional to gap count.

**Class 5 breakdown** (per iter 98 enumeration; 8 tables / 517 gaps):

| Table | Gaps | Notes |
|---|---|---|
| `gimmick_info` | 153 | Tier-1.5 typed-prefix + opaque blob; biggest closure target but blob-decode is hard |
| `character_info` | 146 | Largest table by struct size (8.7KB per-record reader) |
| `stage_info` | 72 | |
| `gimmick_group_info` | 45 | |
| `interaction_info` | 28 | iter 45 validated structurally; field-level audit still pending |
| `tribe_info` | 26 | iter 42 validated structurally; field-level audit still pending |
| `mission_info` | 25 | iter 46 validated structurally; field-level audit still pending |
| `field_info` | 22 | Smallest gap; only 7 vanilla records — easiest fixture-driven analysis |

**Recommended starting table**: `field_info` (22 gaps × 7 entries = small
analysis space; rust struct uses placeholder names like `byte_at_16`,
`lookup_u32_a/b`, `unk_u32_b` — similar pattern to faction_node_info,
likely closeable with the same data-range methodology validated in iters
96-97).

**Pre-flight gotcha for `field_info`**: pabgh format isn't standard
format-1 (u32 count + u32-key + u32-offset entries) or format-2
(u16-count + u16-key + u32-offset entries). The actual bytes
`07 00 01 00 00 00 00 00 00 00 64 00 00 00 79 00 …` (58 bytes total
for 7 entries claimed) need inspection of the rust pabgh parser to
identify the format variant before any field-level analysis can run.

**Impact**: ~520 closures (95% of remaining 549).

**Why last**: largest workload but also where the alias-mechanism
extension (priority A) will pay the most dividends. After A ships,
many class-5 tables likely close mostly via wrap aliases.

## Effort/impact summary

| Priority | Effort | Closures | Cumulative coverage |
|---|---|---|---|
| (today, iter 124) | — | (90/109 closed) | 83% |
| A (alias mech)    | 1 day | +11 (the 9 single-missing wrap tables + 2 vehicle_info wraps) | 93% |
| C (decoder split) | 0.5 day | +3  | 95% |
| D (faction audit) | 1.5 day | +14 | 99% (no — see note) |
| E (class-5 sweep, ongoing) | weeks | -62 done, ~456 remaining | 100% |

Class B (ambiguity) closed in iters 96-97. Class 3 from the 4-class
taxonomy is fully resolved.

**Remaining gap distribution by table** (iter 124 snapshot):

| Table | Gaps | Notes |
|---|---|---|
| `gimmick_info` | 153 | Tier-1.5 typed-prefix + opaque blob |
| `character_info` | 146 | Largest table by per-record reader (8.7KB) |
| `stage_info` | 72 | 3.5KB reader; 60+ placeholder rust fields |
| `gimmick_group_info` | 45 | Wire reads heavily interleaved per iter 118 |
| `interaction_info` | 28 | All 28 inside InteractionTailDecoded sub-struct (class-2) |
| `faction_node_info` | 14 | Per iter 93 audit memo |
| `field_info` | 13 | Iter 103 closure plan; needs 13 more u8 mappings |
| `global_game_event_info` | 3 | Per iter 92 design memo |
| `vehicle_info` | 2 | 1-to-N wraps (class 1) |
| `action_point_info` | 2 | Hidden wrap (class 1) |
| 8× single-missing tables | 8 | All 1-to-N wraps (class 1) |

**Iters 96-106 actual closures: -11 from iter-94's 549 baseline → 538.**

**Note**: cumulative coverage is by table-count, not by gap-count. The
88 → 99 jump after D is misleading — D closes 14 gaps in 1 table, but
8 tables remain in class 5. True 100% coverage requires E.

By gap-count: closing A+C+D drops 538 → 510. E owns the rest (~510 gaps,
of which 13 already closed in field_info iters 99-106 and ~13 remain
there for follow-up iters).

## Cross-cutting follow-ups

These aren't per-class but matter for the project as a whole:

1. **Build-check pattern fix** (per iter 83 follow-up). The
   `cargo build --quiet | tail -3` pattern silently misses errors
   when `--quiet` suppresses output that goes to stderr. Replace with
   `cargo build --release 2>&1 | grep -E "^error" | head -5` to
   surface real failures.

2. **`src/tracked.rs` disposition** (per iter 83 follow-up). The
   untracked file is currently breaking cargo build but is unused
   (no external `tracked::` callers). Decide: revert from worktree,
   comment out `pub mod tracked;` in lib.rs, OR add stub
   `read_tracked_with_size` impls to the 15 affected structs.

3. **Iter-66 closure-plan validation**. The 11 wrap-pattern closure
   plans documented in iters 65-77 each cite a Win-IDA per-record
   reader. Re-confirm those readers' wire layouts haven't drifted
   since iter 47's typeinfo registry was built. (Low risk — game
   binary unchanged — but worth a sanity pass before priority A
   ships).

## References

- Trend table: `V3_1_DECODER_GAPS.md` § Summary.
- 4-class taxonomy: `MOD_AUTHOR_GUIDE.md` § Residual v3.1 surface
  coverage.
- Class 1 memo: `V3_1_ALIAS_MECHANISM_EXTENSION_DESIGN.md`.
- Class 2 memo: `V3_1_GLOBAL_GAME_EVENT_INFO_DECOMPOSE_DESIGN.md`.
- Class 4 memo: `V3_1_FACTION_NODE_INFO_AUDIT.md`.
- Per-table closure plans: top doc-comment of each
  `src/tables/<table>/info.rs`.
- Schema verifier output: `docs/v3_1_schema_verification.json`
  (regenerate with `python scripts/verify_v3_1_against_schema.py`).
