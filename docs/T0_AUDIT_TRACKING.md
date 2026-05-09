# T0 Audit Tracking — IDA-verified C++ name parity per table

**Started:** 2026-05-09 (Session 26)
**Goal:** For every on-disk table, verify each Rust struct field name
matches its real C++ identifier per the Mac-binary IDA symbols.
Where a name was a descriptive translation rather than the canonical
C++ identifier, add a `FIELD_ALIASES_V3` entry so existing v3 mods
keep working AND the canonical T0 name becomes available for v3.1
consumers.

**v3 names are NEVER overwritten.** They become the legacy alias on
input + the v3-shape output projection. The Rust struct field gets
the canonical C++ name; v3 round-trips through the alias mechanism.

## Audit procedure per table

1. Open the table's Rust struct: `src/tables/<name>/info.rs`.
2. List every field name + type.
3. Look up the C++ class in IDA — check for `_ZTSN2pa<class_name>E`
   typeinfo string (e.g. `_ZTSN2pa14GameLevelInfoE`).
4. Enumerate `__ZNK2pa<class>get_<field>Ev` getter symbols.
   - If symbols exist: decompile each (single `return this+offset`),
     confirm field name matches Rust struct field. Mismatches go
     into `FIELD_ALIASES_V3` per Session 24's mechanism.
   - If symbols don't exist (template-aggregated reflection): mark
     the table "T0 unverified — IDA symbols absent, current names
     presumed canonical." This is the same wall hit by .paatt
     deserialiser hunt (Sessions 20, 23).
5. Update this document's row with the result.
6. Commit + push.

## Status legend

| Glyph | Meaning |
|---|---|
| ⏳ | Pending — not yet audited |
| 🔍 | In progress — partial audit |
| ✅ | T0 verified — every field name matches C++ exactly, no aliases needed |
| 🟡 | T0 with aliases — one or more renames shipped, v3 alias added |
| ⚠️ | T0 unverified — IDA getter symbols absent for this class |

## ⚠️ Structural blocker discovered (2026-05-09, Session 26 iter 1)

Bulk IDA probe confirmed: **none of the 122 `pa::*Info` table classes
expose individual `__ZNK*get_<field>Ev` getter symbols** in the Mac
binary. Only the engine-level data-descriptor classes used INSIDE
table records do (e.g. `pa::AttackInfoDataDesc`, `pa::SplineDecalComponent`,
`pa::EmitterCurveData` — these were cracked in Sessions 19/22).

`*Info` tables go through `pa::StaticInfoWrapper<Key, Info, Manager, t>`
templated reflection machinery. Property registration happens via
`PascriptComponentProperty` template instantiations that aggregate
across many fields/types — there's no per-field setter/getter symbol
to enumerate.

**Sample probe (all returned zero getter symbols):**
- `pa::ActionPointInfo` → 0 getter symbols
- `pa::ActionRestrictionOrderInfo` → 0 getter symbols
- `pa::AIDialogStringInfo` → 0 getter symbols
- `pa::SkillInfo` → 0 getter symbols
- `pa::ItemInfo` → 0 getter symbols

Same wall as the .paatt deserialiser hunt (Session 20) and
SplineDecalComponent enumeration (Session 23). Same root cause:
Pearl Abyss's metaobject runtime aggregates property registration
into template instantiations rather than per-field symbols.

### What this means for T0

**Strict T0** (every field name == verbatim C++ identifier per IDA):
*structurally impossible* for these 122 classes without runtime
introspection. The verification data simply doesn't exist statically.

**Pragmatic T0** (every field has a stable, descriptive identifier;
no `_unkXXXX` placeholders): **already achieved** in Session 25's
bulk promotion. Names sourced during the long Tier 1 promotion arc
(finished 2026-04-30) from:
- WIN-IDA Hex-Rays decompile of the parse function in CrimsonDesert.exe
- Mac-IDA `__cstring` declaration-order matching against the Mac binary
- Empirical default-value distribution analysis

The names ARE canonical for mod-author purposes — stable, semantic,
round-trip byte-perfect. They are NOT byte-for-byte equal to the
internal C++ identifiers Pearl Abyss used — that information was
never exposed in the binary's symbol table.

### Where IDA-verified renames CAN still happen

Data-descriptor classes embedded INSIDE table records DO expose
per-field getters and ARE in scope for verbatim renames. Tracked
separately per-target in PAATT_BASEDATA_FIELDS.md and STATUS.md
session entries. Targets include `AttackInfoDataDesc`,
`AttackCommonDataDesc`, `AttackHitDataDesc`, `SplineDecalComponent`
(partial), `EmitterCurveData`, and a few dozen other nested blobs.

The rename work for these polishes the v3.1 embedded-blob surface
but doesn't change any TABLE's tier classification.

### Decision

**Loop stopped after this iteration.** Burning cycles trying to verify
what can't be verified statically is waste. The 118 on-disk tables
stay at "T0 (pragmatic)" per Session 25's promotion. Embedded-descriptor
rename work continues per-target as IDA evidence exists.

If a runtime-introspection method later becomes available (in-game
debugger trace, fuzzer-driven setter mapping, leaked PDB symbols),
strict-T0 verification can resume — this tracking doc stays in place
as the worklist for that future effort.

## Per-table tracking

| # | Module | Status | Notes |
|---|---|---|---|
| 1 | action_point_info | ⏳ | |
| 2 | action_restriction_order_info | ⏳ | |
| 3 | ai_dialog_string_info | ⏳ | |
| 4 | aiaction_attribute_info | ⏳ | |
| 5 | aidialog_type_info | ⏳ | |
| 6 | aievent_table_info | ⏳ | |
| 7 | aimemory_info | ⏳ | |
| 8 | aimove_speed_info | ⏳ | |
| 9 | ally_group_info | ⏳ | |
| 10 | auto_spawn_filter_info | ⏳ | |
| 11 | bitmap_position_info | ⏳ | |
| 12 | board_info | ⏳ | |
| 13 | breakable_object_info | ⏳ | |
| 14 | buff_info | ⏳ | |
| 15 | category_group_info | ⏳ | |
| 16 | category_info | ⏳ | |
| 17 | character_appearance_index_info | ⏳ | |
| 18 | character_change_info | ⏳ | |
| 19 | character_group_info | ⏳ | |
| 20 | character_info | ⏳ | |
| 21 | condition_info | ⏳ | |
| 22 | craft_tool_group_info | ⏳ | |
| 23 | craft_tool_info | ⏳ | |
| 24 | detect_detail_info | ⏳ | |
| 25 | detect_info | ⏳ | |
| 26 | detect_reaction_info | ⏳ | |
| 27 | dialog_voice_info | ⏳ | |
| 28 | drop_set_info | ⏳ | |
| 29 | dye_color_group_info | ⏳ | |
| 30 | effect_info | ⏳ | |
| 31 | elemental_material_info | ⏳ | |
| 32 | equip_info | ⏳ | |
| 33 | equip_slot_info | ⏳ | |
| 34 | equip_type_info | ⏳ | |
| 35 | faction_group_info | ⏳ | |
| 36 | faction_info | ⏳ | |
| 37 | faction_node_info | ⏳ | |
| 38 | faction_node_spawn_info | ⏳ | |
| 39 | faction_relation_group_info | ⏳ | |
| 40 | faction_spawn_data_info | ⏳ | |
| 41 | faction_waypoint_info | ⏳ | |
| 42 | fail_message_info | ⏳ | |
| 43 | field_info | ⏳ | |
| 44 | field_level_name_table_info | ⏳ | |
| 45 | field_revive_info | ⏳ | |
| 46 | formation_info | ⏳ | |
| 47 | frame_event_attr_group_info | ⏳ | |
| 48 | game_advice_group_info | ⏳ | |
| 49 | game_advice_info | ⏳ | |
| 50 | game_event_handler_info | ⏳ | |
| 51 | game_global_effect_info | ⏳ | |
| 52 | game_level_info | ⏳ | |
| 53 | game_play_trigger_info | ⏳ | |
| 54 | game_play_variable_info | ⏳ | |
| 55 | gimmick_event_table_info | ⏳ | |
| 56 | gimmick_gate_connection_info | ⏳ | |
| 57 | gimmick_gate_info | ⏳ | |
| 58 | gimmick_group_info | ⏳ | |
| 59 | gimmick_info | ⏳ | |
| 60 | global_game_event_group_info | ⏳ | |
| 61 | global_game_event_info | ⏳ | |
| 62 | global_stage_sequencer_info | ⏳ | |
| 63 | house_info | ⏳ | |
| 64 | interaction_info | ⏳ | |
| 65 | inventory_info | ⏳ | |
| 66 | item_group_info | ⏳ | |
| 67 | item_use_info | ⏳ | |
| 68 | job_info | ⏳ | |
| 69 | key_map_setting_list_info | ⏳ | |
| 70 | knowledge_group_info | ⏳ | |
| 71 | knowledge_info | ⏳ | |
| 72 | level_action_point_info | ⏳ | |
| 73 | level_gimmick_scene_object_info | ⏳ | |
| 74 | local_string_info | ⏳ | |
| 75 | material_blood_decal_info | ⏳ | |
| 76 | material_match_info | ⏳ | |
| 77 | material_relation_info | ⏳ | |
| 78 | mercenary_group_info | ⏳ | |
| 79 | mercenary_info | ⏳ | |
| 80 | mini_game_data_info | ⏳ | |
| 81 | mission_info | ⏳ | |
| 82 | multi_change_info | ⏳ | |
| 83 | npc_info | ⏳ | |
| 84 | part_prefab_dye_slot_info | ⏳ | |
| 85 | part_prefab_dye_texture_pallete_info | ⏳ | |
| 86 | pattern_description_info | ⏳ | |
| 87 | platform_achievement_info | ⏳ | |
| 88 | platform_entitlement_info | ⏳ | |
| 89 | quest_gauge_info | ⏳ | |
| 90 | quest_group_info | ⏳ | |
| 91 | quest_info | ⏳ | |
| 92 | quick_time_event_info | ⏳ | |
| 93 | region_info | ⏳ | |
| 94 | relation_info | ⏳ | |
| 95 | reserve_slot_info | ⏳ | |
| 96 | royal_supply_info | ⏳ | |
| 97 | sequencer_spawn_info | ⏳ | |
| 98 | skill_group_info | ⏳ | |
| 99 | skill_info | ⏳ | |
| 100 | skill_tree_group_info | ⏳ | |
| 101 | skill_tree_info | ⏳ | |
| 102 | socket_group_info | ⏳ | |
| 103 | socket_info | ⏳ | |
| 104 | spawning_pool_auto_spawn_info | ⏳ | |
| 105 | special_mode_info | ⏳ | |
| 106 | stage_info | ⏳ | |
| 107 | status_group_info | ⏳ | |
| 108 | status_info | ⏳ | |
| 109 | store_info | ⏳ | |
| 110 | string_info | ⏳ | |
| 111 | sub_level_info | ⏳ | |
| 112 | terrain_region_auto_spawn_info | ⏳ | |
| 113 | terrain_region_navi_info | ⏳ | |
| 114 | tribe_info | ⏳ | |
| 115 | trigger_region_info | ⏳ | |
| 116 | ui_social_action_info | ⏳ | |
| 117 | uifilter_group_info | ⏳ | |
| 118 | uimap_texture_info | ⏳ | |
| 119 | valid_schedule_action_info | ⏳ | |
| 120 | vehicle_info | ⏳ | |
| 121 | vibrate_pattern_info | ⏳ | |
| 122 | wanted_info | ⏳ | |
