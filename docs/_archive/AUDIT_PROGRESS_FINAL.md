# Audit Final State (1.3.5 audit phase, iter 1-7)

## Wire-level health
- 423/423 tests pass
- 121/121 byte-perfect round-trips on live 1.05.02 (equip_info absent from PAZ, expected)
- 0 regressions across all renames

## Tables touched (12)
**PASS clean (8)** — verified against IDA, no rename needed:
- inventory_info, drop_set_info, skill_info, item_use_info,
  buff_info, condition_info, vehicle_info, faction_info

**FIXED (4)** — placeholder field names → canonical:
- store_info: 6 fields renamed (sale_item_type_list, fixed_price, etc.)
- reserve_slot_info: 14 fields renamed (time_limit, auto_use_item_info, etc.)
- faction_group_info: 5 fields renamed (faction_group_name, knowledge_info, etc.)
- gimmick_gate_connection_info: 6 fields renamed (material_item_info,
  src_gate_info, etc.)

## Deferred (codegen-pipeline-rerun class)
Hundreds of placeholder field names across:
- character_info (~126), gimmick_info (~100+), stage_info (~62),
  mission_info top-level (~20), elemental_material_info, faction_node_info,
  global_game_event_group_info, etc.

These need either a codegen pipeline rerun with full Mac symbol coverage OR
per-table reader decompiles to confirm wire-position-to-name mapping.
Hand-renaming risks introducing position errors (e.g. mission_info has
swap-vs-IDA-string-order ambiguity that round-trip alone can't detect).

## BagSpace silent-fail
- Diagnostic instrumentation added to DMM-BETA commands.rs V1/V2 patch loop
  (V1V2_APPLY / V1V2_ALREADY log markers) — present in source, awaits
  PANIC9 build approval to ship to testers.

## Next user-facing decision
1. Build PANIC9 with current accumulated DMM-BETA changes (dispatch_isolate,
   per-record splicing, gate removal, V1V2 diagnostics, unmount fix,
   group-replace classifier) — ships BagSpace diagnostics to testers.
2. Schedule codegen-pipeline rerun for the deferred ~800-placeholder
   tables (multi-day work, no wire-format change).
3. Per-table reader decompile push for 4-6 deferred-mapping tables
   (faction_node_info, mission_info, etc.) — could canonicalize 50-80
   more field names with confidence.
