# v3.1 Closure Cheatsheet

**One-page index** of all v3.1 surface-coverage docs. Skim this first;
follow links to full memos as needed.

## State at a glance

```
86 / 109 schema-tables fully aliased       (79%)
23 / 109 with residual gaps                (21%)
549      total missing canonicals
```

## Pick a class, follow the link

| Class | Pages | What | Effort | Reward |
|---|---|---|---|---|
| 0. **Big picture** | [Master plan](V3_1_REMAINING_GAPS_MASTER_PLAN.md) | Priority ordering A→E with effort/impact | 5 min read | Pick which class to attack |
| 1. **Wrap aliases** | [Mechanism design](V3_1_ALIAS_MECHANISM_EXTENSION_DESIGN.md) | `AliasEntry::Single \| Wrap` enum | 1 day | +11 closures (10 tables) |
| 2. **Decoder split** | [GGEI decompose](V3_1_GLOBAL_GAME_EVENT_INFO_DECOMPOSE_DESIGN.md) | Split `execute_data` into 3 typed fields | 0.5 day | +3 closures (1 table) |
| 3. **Ambiguity** | inline in master plan §3 | Function-string xref + data-range analysis | 1 hour | +2 closures |
| 4. **Big audit** | [FNI audit](V3_1_FACTION_NODE_INFO_AUDIT.md) | faction_node_info per-record reader walk | 1.5 day | +14 closures (1 table) |
| 5. **Class-5 sweep** | per-table TBD | The 9 unaudited tables holding ~520 of 549 | weeks | +519 closures |

## Tables you can act on TODAY (after iter 91 mech ships)

| Table | Canonical | N | Snake stem |
|---|---|---|---|
| ally_group_info | `_relationTypeList` | 7 | `relation_type_list_*` |
| character_change_info | `_characterChangeFilter` | 3 | `name_list` + 2 |
| detect_reaction_info | `_reactionTable` | 5 | `reaction_row_*` |
| elemental_material_info | `_flag` | 8 | `flag_*` |
| equip_type_info | `_destroyedAiEvent` | 4 | `destroyed_ai_event_*` |
| faction_node_spawn_info | `_boundaryBox` | 2 | `boundary_box_min/max` |
| faction_relation_group_info | `_relationGroupList` | 4 | `rel_*` |
| royal_supply_info | `_royalSupplyRandomMap` | 2 | `royal_supply_random_map_*` |
| sub_level_info | `_exp` | 4 | `exp_a/b/c/d` |
| vehicle_info | `_vehicleSeatDataList` | 16 | `vehicle_seat_data_*` |
| vehicle_info | `_parentLinkAttachDataList` | 2 | `parent_link_attach_data_a/b` |

Wire bytes already round-trip; only the JSON name surface needs the
`Wrap` entry. Per-table closure plans live at the top of each
`src/tables/<name>/info.rs`.

## Pre-flight (cross-cutting)

- Cargo build is broken from out-of-loop in-progress refactor:
  `src/tracked.rs` (untracked) + `src/lib.rs` (modified) + `src/binary/variant.rs`
  (modified). Decide before starting any rust work — see iter 83.
- Replace `cargo build --release --quiet | tail -3` with
  `cargo build --release 2>&1 | grep -E "^error" | head -5` for
  reliable build verification. The `--quiet` pattern silently
  suppressed errors during iters 70-82.

## Doc inventory (all v3.1 surface docs)

| Doc | Purpose |
|---|---|
| `MOD_AUTHOR_GUIDE.md` § 0 | Mod-author-facing overview, PA-typo list, residual coverage |
| `V3_1_README.md` | Spec history + implementation status snapshot |
| `V3_1_DECODER_GAPS.md` | Per-table gap worklist + trend table + master typeinfo registry |
| `V3_1_SCHEMA_VERIFICATION.md` | Auto-generated per-table verification report |
| `V3_1_PYCRIMSON_WORKFLOW.md` | Reflection-format harvest workflow |
| `V3_1_REMAINING_GAPS_MASTER_PLAN.md` | Priority ordering + effort estimates |
| `V3_1_ALIAS_MECHANISM_EXTENSION_DESIGN.md` | Class-1 design |
| `V3_1_GLOBAL_GAME_EVENT_INFO_DECOMPOSE_DESIGN.md` | Class-2 design |
| `V3_1_FACTION_NODE_INFO_AUDIT.md` | Class-4 audit |
| `V3_1_CLOSURE_CHEATSHEET.md` | **You are here** — 1-page index |
