# Sub-Struct Decomposition for v3.1 Coverage — Design Memo

**Status**: Design (not yet implemented).
**Author**: 1-min loop, iter 147 (2026-05-10).
**Audience**: Future implementer who wants to unblock the ~30+ v3.1
gaps that live INSIDE nested rust sub-structs (class-6 of the closure
taxonomy — added iter 146 to the cheatsheet).

## Problem

`scripts/generate_v3_1_aliases.py` extracts fields from the top-level
`pub struct <TableName>` block in `src/tables/<table>/info.rs` via
regex. The `FIELD_ALIASES_V3_1` const consumed by `src/json_shape.rs`
is a flat `(snake, _camelCase)` slice. The mechanism cannot reach
fields inside nested sub-structs.

This pattern blocks v3.1 coverage in 5+ tables:

| Table | Sub-struct | Gaps inside | First flagged |
|---|---|---|---|
| `interaction_info` | `InteractionTailDecoded` | 28 | iter 124 |
| `field_info` | `FieldInfoComposite` | 3-5 | iter 126 |
| `action_point_info` | `ActionPoint` | 2 | iter 143 |
| `faction_node_info` | `FactionNodeBigCompositeSlots` + `FactionNodeDE690` | 3 | iter 134 |
| `gimmick_info` | `GimmickTail` (Decoded\|Raw enum) | many (~150) | iter 143 |
| `character_info` | (8.7KB reader, multiple sub-structs) | many | iter 143 |
| `gimmick_group_info` | (interleaved sub-readers) | many | iter 118 |
| `stage_info` | (multiple sub-structs) | ~18 | iter 138 |

Conservatively: **30+ closures across class-1 small tables** + many
more in the giants. Per the iter-146 cheatsheet, this would close
roughly Class-6's worth of residual gaps.

## Two paths

### Path A: Decompose sub-structs into top-level fields

For each affected table, flatten the rust struct so sub-struct fields
become top-level fields on the parent. Example for `action_point_info`:

```rust
// BEFORE
pub struct ActionPointInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub action_point: ActionPoint,           // contains _actionPosition, _actionYaw
    pub level_action_point_info: u32,
    pub action_point_b: ActionPoint,
}

pub struct ActionPoint { /* opaque to alias mech */ }

// AFTER
pub struct ActionPointInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub action_position: [f32; 3],           // promoted from ActionPoint
    pub action_yaw: f32,                     // promoted from ActionPoint
    pub level_action_point_info: u32,
    pub action_position_b: [f32; 3],         // promoted from ActionPoint (suffix _b)
    pub action_yaw_b: f32,
}
```

**Pros**:
- Aliases just work via standard tuple-scoped overrides.
- JSON shape becomes flat — easier for mod authors to author.
- No mechanism change needed.

**Cons**:
- Breaks ANY existing consumer that walks the typed struct (mod
  manager Python code, etc.).
- Some sub-structs are large + reused across tables (e.g. ActionPoint
  also in level_action_point_info table) — duplicating fields
  multiplies churn.
- Doesn't work cleanly for Decoded|Raw enum tails (gimmick_info,
  interaction_info) because the polymorphic dispatch needs the enum.

### Path B: Extend alias mechanism to support nested field paths

Change the alias dispatch to support `"sub_struct.field"` snake-case
path strings instead of plain field names:

```rust
pub const FIELD_ALIASES_V3_1: &[(&str, &str)] = &[
    ("action_point.field_a",   "_actionPosition"),
    ("action_point.block_a",   "_actionYaw"),
    // …
];
```

JSON serialization walks the rust struct, recurses into sub-structs
when the snake-path has a `.` separator, and emits the canonical name
at the top level (or as a sub-dict — needs design).

**Pros**:
- Doesn't require struct refactor — touches only the alias dispatch
  + the generator script.
- Works for Decoded|Raw enum tails (the path includes the variant
  match).
- Preserves existing consumer code (typed Rust struct unchanged).

**Cons**:
- JSON shape choice: flatten to top-level OR preserve nesting?
  Flatten breaks if two sub-structs both have a field of the same
  canonical name. Preserve-nesting requires mod authors to write
  `{"_actionPoint": {"_actionPosition": [...]}}` which is more verbose
  than the flat form.
- The generator script needs to walk nested structs to extract
  candidate paths — more complex than top-level regex match.

## Recommendation

**Mixed strategy**:

1. **Path A (decompose)** for tables where the sub-struct is opaque
   (single-use, never referenced elsewhere): `field_info`'s
   `FieldInfoComposite`, `faction_node_info`'s
   `FactionNodeBigCompositeSlots`/`FactionNodeDE690`, `action_point_info`'s
   `ActionPoint`. Small change, no mechanism work.

2. **Path B (nested paths)** for Decoded|Raw enum tails:
   `interaction_info`'s `InteractionTailDecoded`, `gimmick_info`'s
   `GimmickTail`. These have meaningful runtime polymorphism and
   shouldn't lose their typed structure.

## Implementation sketch (Path A for action_point_info)

1. Read iter-93's hidden-wrap discovery on `ActionPoint`:
   `field_a: u32, block_a: [u8; 24]`. The block_a is actually the
   24-byte Vec3+f32 payload that maps to `_actionPosition` + `_actionYaw`.

2. Refactor `ActionPoint` definition to expose typed fields:
   ```rust
   pub struct ActionPoint {
       pub position: [f32; 3],
       pub yaw: f32,
       pub raw_tail: [u8; 8],  // remaining bytes
   }
   ```

3. Update `read_from` / `write_to` accordingly.

4. Promote the two ActionPoint instances in `ActionPointInfo` to
   top-level by struct flattening (or leave as `ActionPoint` and
   choose Path B for this table specifically — needs decision).

5. Add aliases:
   ```rust
   ("action_point.position",   "_actionPosition"),
   ("action_point.yaw",        "_actionYaw"),
   // OR (after flattening)
   ("action_position",         "_actionPosition"),
   ("action_yaw",              "_actionYaw"),
   ```

6. Verify schema verifier closes both gaps.

## Risks

- **Byte round-trip drift**: any struct refactor must preserve byte-
  level round-trip. Add roundtrip tests with `cargo test` BEFORE +
  AFTER the change (currently blocked by iter-83 out-of-loop build
  break).
- **Mod-author JSON compatibility**: existing mods may target the
  current JSON shape. A migration note + backward-compat input
  parsing should accompany any path chosen.
- **Generator complexity**: Path B's nested-path-walk extension is
  non-trivial regex/AST work. Estimate 1-2 days for the generator
  + dispatcher changes.

## Out of scope

- Resolving `gimmick_info` and `character_info`'s polymorphic tails
  fully (those need per-variant decoder design, separate from this
  memo's scope).
- The class-1 1-to-N alias mechanism (covered separately in
  `docs/V3_1_ALIAS_MECHANISM_EXTENSION_DESIGN.md`).

## References

- Class-1 design memo: `V3_1_ALIAS_MECHANISM_EXTENSION_DESIGN.md`.
- Class-2 example (global_game_event_info decompose):
  `V3_1_GLOBAL_GAME_EVENT_INFO_DECOMPOSE_DESIGN.md`.
- Class-4 audit (faction_node_info): `V3_1_FACTION_NODE_INFO_AUDIT.md`.
- Master plan: `V3_1_REMAINING_GAPS_MASTER_PLAN.md`.
- Cheatsheet: `V3_1_CLOSURE_CHEATSHEET.md` § Class 6.
- Methodology: `V3_1_CLOSURE_METHODOLOGY.md`.
