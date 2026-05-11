# v3.1 Alias Mechanism — 1-to-N Extension Design Memo

**Status**: Design (not yet implemented).
**Author**: 1-min loop, iter 91 (2026-05-10).
**Audience**: Future implementer who wants to close the ~10 wrap-pattern
v3.1 gaps with a single mechanism change rather than per-table refactors.

## Problem

The current `FIELD_ALIASES_V3_1` is a flat slice of `(snake, camel)` pairs:

```rust
// src/tables/<table>/field_aliases_v3_1.rs
pub const FIELD_ALIASES_V3_1: &[(&str, &str)] = &[
    ("key",                "_key"),
    ("string_key",         "_stringKey"),
    ("relation_type_list_0", "_???"),  // ← canonical doesn't exist
    ("relation_type_list_1", "_???"),
    // … relation_type_list_2..6 also unmapped
];
```

11 single-missing-canonical tables (closed via iter 65-77 closure plans)
share the same shape: one PA canonical name (`_relationTypeList`) wraps
2-16 unrolled rust fields (`relation_type_list_0..6`). The wire bytes
already round-trip; only the JSON name surface diverges.

Today's mechanism can't express this. Workarounds tried:
- Tuple-scoped 1-to-1 overrides — collision-safe but only handles
  PA-typo / name-divergence cases (iters 70-82 used this for 6 tables).
- Per-table struct refactor — collapse `relation_type_list_0..6` into
  a single `relation_type_list: [CArray<u32>; 7]` field. Works but
  multiplies churn across 11 tables, breaks every consumer's JSON path.

## Proposed extension

Promote the alias entry to an enum:

```rust
pub enum AliasEntry<'a> {
    /// 1-to-1 (today's behaviour). Snake → canonical camel.
    Single(&'a str, &'a str),

    /// 1-to-N. Canonical camel name expands to (or collects from) a
    /// fixed sequence of snake fields. Order matters — that's the
    /// JSON-array index order on emit, and the input order on consume.
    Wrap(&'a str, &'a [&'a str]),
}

pub const FIELD_ALIASES_V3_1: &[AliasEntry] = &[
    AliasEntry::Single("key",        "_key"),
    AliasEntry::Single("string_key", "_stringKey"),
    AliasEntry::Wrap(
        "_relationTypeList",
        &[
            "relation_type_list_0", "relation_type_list_1",
            "relation_type_list_2", "relation_type_list_3",
            "relation_type_list_4", "relation_type_list_5",
            "relation_type_list_6",
        ],
    ),
];
```

### Emit semantics (rust → JSON, shape='v3.1')

For each `Wrap(canonical, [snake_0..N-1])`:

```jsonc
{
  "_relationTypeList": [
    <snake_0's JSON value>,
    <snake_1's JSON value>,
    …
    <snake_{N-1}'s JSON value>
  ]
}
```

The N source snake fields disappear from the dict (they're consumed
into the array).

### Consume semantics (JSON → rust)

If the input dict has `_relationTypeList` as an array of length N:
distribute its elements back to `snake_0..N-1`. If it has the snake
names directly: pass through unchanged (backward compat).

Mismatched length → error with a clear message naming the table +
canonical + expected/actual length.

### Shape='v3' behaviour

Unchanged. v3 always emits snake names; `Wrap` entries are ignored on
emit (only `Single` entries fire). On consume, both forms accepted as
today.

## Affected files

1. `src/json_shape.rs` — add `AliasEntry` enum + lookup helpers.
2. `scripts/generate_v3_1_aliases.py` — emit the enum form. The
   schema-grounded path can detect wrap candidates by walking
   `<base>_<digit>+` rust field clusters that match a single schema
   canonical (the `_relationTypeList`-style pattern).
3. `src/tables/<table>/field_aliases_v3_1.rs` — regenerated to use
   the new enum form. Backward compatible with the existing 1-to-1
   entries.
4. `src/python.rs` — JSON-shape helpers consuming the registry need
   the new enum dispatch.

## Affected tables (the closure-plan beneficiaries)

All 11 closures from iters 65-77 fit the same structural pattern:

| Table | Canonical | N | Snake-field stem |
|---|---|---|---|
| `ally_group_info` | `_relationTypeList` | 7 | `relation_type_list_*` |
| `character_change_info` | `_characterChangeFilter` | 3 | `name_list` + 2 others |
| `detect_reaction_info` | `_reactionTable` | 5 | `reaction_row_*` |
| `elemental_material_info` | `_flag` | 8 | `flag_*` |
| `equip_type_info` | `_destroyedAiEvent` | 4 | `destroyed_ai_event_*` |
| `faction_node_spawn_info` | `_boundaryBox` | 2 | `boundary_box_min/max` |
| `faction_relation_group_info` | `_relationGroupList` | 4 | `rel_*` |
| `royal_supply_info` | `_royalSupplyRandomMap` | 2 | `royal_supply_random_map_*` |
| `sub_level_info` | `_exp` | 4 | `exp_a/b/c/d` |
| `vehicle_info` | `_vehicleSeatDataList` | 16 | `vehicle_seat_data_*` |
| `vehicle_info` | `_parentLinkAttachDataList` | 2 | `parent_link_attach_data_a/b` |

Each row's wire format is fully decoded today; only the JSON name
surface needs the wrap.

## Validation plan

1. Implement enum + dispatcher with no overrides — confirm
   `cargo test` still passes (562 baseline). Backward-compat check.
2. Add ONE wrap entry (start with `_boundaryBox`, only 2 sub-fields).
   Add roundtrip test: parse → emit shape='v3.1' → parse back → assert
   JSON values stable.
3. Roll out to all 11 tables. Re-run the schema verifier — expect
   `missing-in-dmm` to drop from 549 to ~539 (10-11 closures).

## Out of scope

- Wrap entries that need decoder work (`_eventDesc` / `_uiIconPath`
  / `_targetRegionInfoList` in `global_game_event_info`) are NOT
  this design's concern. They need rust struct decomposition first.
- Semantic-ambiguity cases (`_onDiscoverOnlyEnable`,
  `_executePercent`) need string-xref work, also out of scope.

## References

- Per-table closure plans for the 11 wrap tables: each table's
  `info.rs` top doc-comment, "v3.1 closure analysis (iter NN)" block.
- Current 4-class gap taxonomy: `MOD_AUTHOR_GUIDE.md` §
  "Residual v3.1 surface coverage".
- Trend table showing the closure progression:
  `V3_1_DECODER_GAPS.md` § Summary.
