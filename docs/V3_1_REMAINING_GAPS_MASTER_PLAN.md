# Closing the Remaining 549 v3.1 Gaps — Master Plan

**Status**: Coordination doc.
**Author**: 1-min loop, iter 94 (2026-05-10).
**Audience**: Implementer who wants a single entry point for finishing
the v3.1 surface coverage. Picks priorities, estimates effort, points to
the per-class design memos.

## Where we are (iter 107 update)

```
126 dmm-parser tables
 └─ 109 in NattKh canonical schema
     ├─  88 fully-aliased (100% _camelCase coverage)   [+2 since iter 94]
     └─  21 with-gaps  →  538 missing canonicals total  [-11 since iter 94]
```

**Iters 96-106 progress** (resumed-loop):
- iter 96: closed `_executePercent` in global_game_event_group_info via fixture
  data-range analysis (resolved iter-81 deferred ambiguity 1 of 2). Table now
  100% covered.
- iter 97: closed `_onDiscoverOnlyEnable` in level_gimmick_scene_object_info
  via fixture data-range analysis (resolved iter-80 deferred ambiguity 2 of 2,
  also closing class-3 entirely). Table now 100% covered.
- iters 99-106: 9 partial closures in field_info via type-match + data-pattern
  + IDA wire-read disambiguation. Now 11/24 verified (was 2/24).

The 21 with-gaps tables fall into 4 active closure classes (per
`MOD_AUTHOR_GUIDE.md` § Residual coverage; class 3 is now CLOSED).
Each class has a dedicated design memo. This doc is the index +
priority recommendation.

## Class summary + memo links

| Class | # tables | # gaps | Closure type | Design memo |
|---|---|---|---|---|
| 1. **1-to-N alias mechanism extension** | ~10 | ~10 | Mechanism + per-table aliases | `V3_1_ALIAS_MECHANISM_EXTENSION_DESIGN.md` |
| 2. **Real decoder work — struct decompose** | 1 (`global_game_event_info`) | 3 | Decompose `execute_data` polymorphic wrapper | `V3_1_GLOBAL_GAME_EVENT_INFO_DECOMPOSE_DESIGN.md` |
| 3. **Semantic ambiguity** | 2 | 2 | String-xref work via function-string-associate | (no dedicated memo — see below) |
| 4. **Larger un-audited table** | 1 (`faction_node_info`) | 14 | Per-record-reader decompile + struct cleanup | `V3_1_FACTION_NODE_INFO_AUDIT.md` |
| 5. **Other partially-aliased tables** | ~9 | ~520 | Per-table audit (likely mix of all above) | TBD per-table |

**Total reachable via classes 1-4**: ~29 closures (~5% of 549).
**Class 5 dominates the residual** — those 9 unaudited tables hold
~95% of remaining gaps. They likely contain the same mix of patterns
documented in classes 1-4, just spread across more tables.

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
| (today, iter 107) | — | (88/109 closed) | 81% |
| A (alias mech)    | 1 day  | +11  | 91% |
| B (ambiguity)     | DONE iters 96-97 — class 3 closed | (+2 done) | (was 91%, now 81% baseline already includes this) |
| C (decoder split) | 0.5 day | +3  | 92% |
| D (faction audit) | 1.5 day | +14 | 99% (no — see note) |
| E (class-5 sweep) | weeks (started iter 99) | +517 → 506 in-progress | 100% |

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
