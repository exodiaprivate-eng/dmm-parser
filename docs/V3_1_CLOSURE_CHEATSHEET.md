# v3.1 Closure Cheatsheet

**One-page index** of all v3.1 surface-coverage docs. Skim this first;
follow links to full memos as needed.

## State at a glance (iter 146)

```
90 / 109 schema-tables fully aliased       (83%)   [+4 since iter 95]
19 / 109 with residual gaps                (17%)
463      total missing canonicals          [-86 since iter 95]
```

## Pick a class, follow the link

| Class | Pages | What | Effort | Reward |
|---|---|---|---|---|
| 0. **Big picture** | [Master plan](V3_1_REMAINING_GAPS_MASTER_PLAN.md) | Priority ordering A→E with effort/impact | 5 min read | Pick which class to attack |
| 1. **Wrap aliases** | [Mechanism design](V3_1_ALIAS_MECHANISM_EXTENSION_DESIGN.md) | `AliasEntry::Single \| Wrap` enum | 1 day | +11 closures (10 tables) |
| 2. **Decoder split** | [GGEI decompose](V3_1_GLOBAL_GAME_EVENT_INFO_DECOMPOSE_DESIGN.md) | Split `execute_data` into 3 typed fields | 0.5 day | +3 closures (1 table) |
| 3. **Ambiguity** | ~~inline in master plan §3~~ | **CLOSED** iters 96-97 via fixture data-range analysis | (done) | (+2 done) |
| 4. **Big audit** | [FNI audit](V3_1_FACTION_NODE_INFO_AUDIT.md) | faction_node_info residual (3 of 14 left, all in sub-structs) | 0.5 day | +3 closures |
| 5. **Class-5 sweep** | per-table TBD | 4 giants holding 412 of 463 = 89% (gimmick_info 153, character_info 146, gimmick_group_info 45, stage_info 68) | weeks | +412 closures |
| 6. **Sub-struct decompose** | (no design memo yet) | Unblocks iter-143-surfaced singletons in interaction_info, field_info residual, action_point_info | varies | +30+ closures across multiple tables |

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
- **Tools available** for routine maintenance:
  - `python scripts/find_singleton_closures.py` — scan all gap tables for
    type-singleton closure opportunities (iter 142)
  - `python scripts/audit_manual_overrides.py` — verify all 121 MANUAL_OVERRIDES
    target existing rust fields (iter 141, guards iter-122 silent-drop bug)
  - `python scripts/verify_v3_1_against_schema.py` — refresh the canonical
    verification report after any alias change

- **Iter-122 generator bug fix**: prior to iter 122, the `is_placeholder`
  filter in `extract_main_struct_fields()` ran BEFORE the `MANUAL_OVERRIDES`
  check, silently dropping any tuple-keyed override targeting a placeholder-
  pattern rust field name (e.g. `lookup_a` matching `^lookup_[a-z]$`). Fixed
  by reordering. Run `audit_manual_overrides.py` after any future generator
  change to detect regressions.

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
