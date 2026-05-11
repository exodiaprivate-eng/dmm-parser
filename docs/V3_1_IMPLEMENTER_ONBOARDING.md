# v3.1 Implementer Onboarding — Day 1

**Status**: Action guide.
**Audience**: Whoever picks up the v3.1 closure work after the loop's
iter-151 stop. Action-focused; not a summary (see
`V3_1_LOOP_ASSESSMENT.md` for that).

## Day 1 morning — orientation (30 min)

```bash
cd "C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser"
```

1. **Refresh + read the schema verifier output** — know the current state:
   ```bash
   python scripts/verify_v3_1_against_schema.py
   ```
   Then skim:
   - `docs/V3_1_CLOSURE_CHEATSHEET.md` (1 page) — class-priority table.
   - `docs/V3_1_LOOP_ASSESSMENT.md` (1 page) — what was done + what's left.
   - `docs/V3_1_REMAINING_GAPS_MASTER_PLAN.md` (5 pages) — full plan + memo links.

2. **Run the loop's tools** to confirm nothing's drifted:
   ```bash
   python scripts/audit_manual_overrides.py        # should print '[OK]'
   python scripts/find_singleton_closures.py       # surfaces opportunities
   ```

3. **Verify cargo build state**:
   ```bash
   cargo build --release 2>&1 | grep -E "^error" | head -5
   ```
   If it shows errors involving `read_tracked_with_size`: that's the
   **iter-83 build break**. Resolve before any rust work — see
   "Day 1 afternoon" below.

## Day 1 afternoon — fix iter-83 build break (1-2 hours)

**Problem**: `src/tracked.rs` is untracked but `src/lib.rs` (modified)
references it via `pub mod tracked;`. The `tracked_p!` macro inside
calls `<TableStruct>::read_tracked_with_size(...)` which 15 table structs
don't define. Build fails.

**Diagnostic**:
```bash
git status --short src/  # should show ?? src/tracked.rs + M src/lib.rs + M src/binary/variant.rs
```

**3 resolution options** (pick one):

| Option | Risk | Time |
|---|---|---|
| (a) `git restore src/lib.rs` and `rm src/tracked.rs` — full revert of the in-progress work | low; loses the in-progress refactor | 5 min |
| (b) Comment out `pub mod tracked;` in `src/lib.rs` line 17 | low; preserves the refactor file for later | 2 min |
| (c) Add stub `read_tracked_with_size` impls to the 15 affected structs | high; you'd be guessing at the in-progress design | 1+ hour |

**Recommendation**: option (a) or (b). The in-progress refactor is from
outside the loop's commits and should be authored by whoever started it.

After resolving, confirm:
```bash
cargo build --release 2>&1 | tail -3   # should print no errors
cargo test --release 2>&1 | tail -3    # should show 562 passing tests
```

## Day 2 — Class 1: alias mechanism extension (1 day, +11 closures)

Read `docs/V3_1_ALIAS_MECHANISM_EXTENSION_DESIGN.md` cold. The memo lays
out the `AliasEntry::Single | Wrap` enum design end-to-end. Implementation
sketch:

1. Add the `AliasEntry` enum to `src/json_shape.rs`.
2. Update `lookup_table_aliases()` to dispatch on the enum variant.
3. Update `scripts/generate_v3_1_aliases.py` to emit the new enum form
   (the wrap-detection logic walks `<base>_<digit>+` rust-field clusters
   matching a single schema canonical).
4. Roll out wrap entries for the 11 tables enumerated in the memo.
5. Run `python scripts/audit_manual_overrides.py` + verifier to confirm
   no regressions.

Expected delta: 463 → ~452 missing.

## Day 3-4 — Class 6: sub-struct decompose (1-2 days, +30+ closures)

Read `docs/V3_1_SUB_STRUCT_DECOMPOSE_DESIGN.md` cold. Per the memo's
recommended mixed strategy:

- **Path A (decompose to top-level fields)** for opaque single-use
  sub-structs: `field_info::FieldInfoComposite` (5 closures),
  `faction_node_info::big_composite_slots`/`de690_data` (3 closures
  residual), `action_point_info::ActionPoint` (2 closures).
- **Path B (nested-path alias mech)** for Decoded|Raw enum tails:
  `interaction_info::InteractionTailDecoded` (28 closures),
  `gimmick_info::GimmickTail` (eventually).

Start with `action_point_info` (smallest case). The iter-43 hidden-wrap
analysis already documented `ActionPoint { field_a: u32, block_a: [u8; 24] }`
— that 24-byte block decomposes into Vec3 + f32 = `_actionPosition` +
`_actionYaw`.

Expected delta: 463 → ~395-425 missing depending on Class-6 scope.

## Day 5 — Class 2: global_game_event_info decompose (0.5 day, +3 closures)

Read `docs/V3_1_GLOBAL_GAME_EVENT_INFO_DECOMPOSE_DESIGN.md`. Decompose the
`execute_data` polymorphic wrapper into 3 typed fields:
`event_desc: [u8; 8]`, `ui_icon_path: u32`, `target_region_info_list: CArray<u16>`.

Expected delta: -3 missing.

## Beyond Day 5 — the 4 giants (weeks)

Each of `gimmick_info` (153), `character_info` (146),
`gimmick_group_info` (45), `stage_info` (68) needs its own audit cycle
applying the methodology. For each table:

1. Decompile its per-record reader (address in
   `docs/V3_1_DECODER_GAPS.md` § Master typeinfo + record-reader registry).
2. Walk wire reads + match against schema canonicals using the 7
   techniques in `docs/V3_1_CLOSURE_METHODOLOGY.md`.
3. Apply `find_singleton_closures.py` after each refresh to surface
   newly-tractable opportunities.
4. Update `docs/V3_1_REMAINING_GAPS_MASTER_PLAN.md` with each
   per-table audit progress.

Recommended order: stage_info → gimmick_group_info → character_info →
gimmick_info (easiest to hardest by reader size).

## Tooling cheat-card

```bash
# Refresh schema verifier:
python scripts/verify_v3_1_against_schema.py

# Find ship opportunities:
python scripts/find_singleton_closures.py

# Validate overrides after any change:
python scripts/audit_manual_overrides.py

# Regenerate per-table alias files:
python scripts/generate_v3_1_aliases.py
```

## When to update what

After landing closures, update these (in order):
1. `scripts/generate_v3_1_aliases.py` (add MANUAL_OVERRIDES entries)
2. `python scripts/generate_v3_1_aliases.py` (regenerate)
3. `python scripts/verify_v3_1_against_schema.py` (refresh)
4. `python scripts/audit_manual_overrides.py` (validate)
5. `docs/V3_1_DECODER_GAPS.md` § Summary (numerical updates)
6. `docs/V3_1_CLOSURE_CHEATSHEET.md` (state-at-glance)
7. `docs/V3_1_REMAINING_GAPS_MASTER_PLAN.md` (if class-state changes)
8. `docs/MOD_AUTHOR_GUIDE.md` (residual-coverage section)

## References

All in `docs/`:
- `V3_1_LOOP_ASSESSMENT.md` — what the loop did + why it stopped
- `V3_1_CLOSURE_METHODOLOGY.md` — the 7 closure techniques + workflow
- `V3_1_REMAINING_GAPS_MASTER_PLAN.md` — coordination doc
- `V3_1_CLOSURE_CHEATSHEET.md` — 1-page index
- `V3_1_ALIAS_MECHANISM_EXTENSION_DESIGN.md` — class 1
- `V3_1_GLOBAL_GAME_EVENT_INFO_DECOMPOSE_DESIGN.md` — class 2
- `V3_1_FACTION_NODE_INFO_AUDIT.md` — class 4
- `V3_1_SUB_STRUCT_DECOMPOSE_DESIGN.md` — class 6
- `V3_1_DECODER_GAPS.md` — gap inventory + typeinfo registry
- `V3_1_README.md` — implementation status
- `V3_1_PYCRIMSON_WORKFLOW.md` — pycrimson reflection harvest
- `V3_1_SCHEMA_VERIFICATION.md` — auto-generated per-table report
- `MOD_AUTHOR_GUIDE.md` § 0 — mod-author-facing overview
