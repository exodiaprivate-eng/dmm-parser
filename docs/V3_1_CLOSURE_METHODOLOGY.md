# v3.1 Gap-Closure Methodology

**Status**: Living guide.
**Author**: 1-min loop, distilled from iters 70-112.
**Audience**: Future implementer working through the residual 532 v3.1
gaps. Read this BEFORE attacking any class-5 table.

## TL;DR — the 6 techniques in priority order

1. **Type-unique singleton match** — Highest confidence. If the rust struct
   has exactly 1 field of a given type and the schema has exactly 1
   canonical of the matching shape, ship the alias.
2. **Type-unique run + setter-string declaration order** — High confidence.
   N consecutive same-shape reads in the IDA per-record reader paired
   with N same-shape canonicals (sorted by setter-string address) give
   an unambiguous 1:1 mapping. Iter 112's mission_info close (4
   LocalizableString labels in one shot) used this.
3. **Per-entry uniqueness signature** — Medium confidence. A canonical
   that semantically MUST be unique per entry (`_levelName`, `_key`)
   must match the rust field whose fixture data has all-distinct values.
   Iter 102's field_info close used this.
4. **Data-range value pattern** — Medium confidence. Boolean canonicals
   should appear as 0/1 byte values; "default-shared" canonicals show
   one dominant value across entries; "always-zero" canonicals fit
   no-default-set semantics. Iters 96/97/106 used this.
5. **PA-internal typo recognition** — Low-effort. Schema has known typos
   (`_complteDescription`, `_overriedMaxHeight`, `_questGroupkey`,
   `_radgollEquipTableGroupDataList`, `_fishSummonTimeFrquencyType`,
   `_regionEnterknowledgeInfoList`). When the mechanical generator
   skips a translation, check first if the canonical is a typo of an
   obvious rust field name.
6. **Semantic name-divergence mapping** — Lowest confidence. When rust
   placeholder names like `unk_X` or hand-decoded heuristics like
   `byte_at_16` need to map to canonicals, use semantic plausibility
   (`is_housing_region` ↔ `_isSaveGimmickRegion` per the housing-uses-
   save-gimmick-mechanic insight). Iter 78 / 80 / 81 / 99 used this.

## Anti-patterns (don't do these)

- **Don't assume schema 'r' field matches per-record reader signature.**
  The schema's `r` (reader) is the GENERIC dispatcher; per-record
  readers inline different sub-readers with the same wire shape but
  different addresses. Iter 111 found this when sub_1410CCE80 didn't
  match the schema's r=0x14105f040 for `_tamedSkillList`.

- **Don't use schema dict-iteration order as canonical declaration
  order.** The JSON dict order is arbitrary. Sort by `s` (setter-string
  address) for declaration order — that matches the binary layout for
  type-unique runs.

- **Don't treat schema type-tags as wire-size constraints.** Schema's
  `direct_u64` was used for u16 keys (`_key` in many tables) and u32
  values (`_executePercent` in iter 96). The tag is the SETTER DISPATCH
  CLASS, not the wire width. Always cross-check with IDA wire reads.

- **Don't extrapolate setter-string declaration order across type
  groups.** Within a type-unique run, wire order follows setter order.
  Across type groups, wire order can differ (e.g. `_key` is at
  setter-position 9 in field_info but wire-position 1).

- **Don't assume contiguous rust placeholder runs ARE wire-contiguous
  type-unique runs.** Iters 117-118 found gimmick_group_info has
  rust `flag_344..356` (13 consecutive u8 fields) BUT the wire reads
  for those mem positions are interleaved with u32 + sub-reader
  reads. Verify wire-contiguity via IDA decompile BEFORE applying
  iter-114's N-block shipping technique. The rust struct order
  reflects the in-memory layout, which may pad/group differently
  than the wire layout.

## Workflow per table

1. Run `python scripts/verify_v3_1_against_schema.py` and dump the
   missing canonicals for the table.
2. Pull the table's per-record reader address from the iter-47
   typeinfo registry in `docs/V3_1_DECODER_GAPS.md`.
3. Decompile via `mcp__ida-pro-mcp__decompile_function(addr)`.
4. Walk wire reads in source order, annotating each with mem offset
   and rust field name (from the table's `info.rs` struct).
5. Group remaining canonicals by schema type (sort by `s` for
   declaration order within type groups).
6. Apply techniques in TL;DR priority order:
   - Singleton matches first (type-unique 1:1).
   - Type-unique runs next (N consecutive same-shape reads).
   - Then data-pattern + semantic for the long tail.
7. Write the closure plan as the `info.rs` doc-comment header
   (per iters 103 / 109 / 112 templates).
8. Ship the safe closures via tuple-scoped MANUAL_OVERRIDES in
   `scripts/generate_v3_1_aliases.py`.
9. Regenerate aliases + verify schema verifier delta:
   ```bash
   python scripts/generate_v3_1_aliases.py
   python scripts/verify_v3_1_against_schema.py
   ```
10. Commit + push only the targeted files (NEVER `git add -A` —
    that's how iter 67 swept in a 412KB zip; per workplan rule,
    use specific file lists in `git add`).

## Pre-flight gotchas

- **Pabgh format detection**: 3 variants (u16-count + 8-byte entries,
  u16-count + 6-byte entries, u32-count + 8-byte entries). Header
  bytes are ambiguous; auto-detect by which size constraint matches
  the file length. See `src/binary/pabgh.rs`. Iter 99 hit this.
- **Cargo build pre-broken**: `src/tracked.rs` (untracked) +
  `src/lib.rs` modified add `pub mod tracked;` reference an
  unimplemented method on 15 struct types. Closure work that touches
  ONLY scripts/ + alias .rs files + docs/ is safe (build verification
  not needed). See iter 83.

## References

- Per-table closure plans: top doc-comment of `src/tables/<table>/info.rs`.
- Master typeinfo registry: `V3_1_DECODER_GAPS.md` § Master typeinfo +
  record-reader registry (31 entries).
- Master plan: `V3_1_REMAINING_GAPS_MASTER_PLAN.md`.
- Cheatsheet: `V3_1_CLOSURE_CHEATSHEET.md`.
