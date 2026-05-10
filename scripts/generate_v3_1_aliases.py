#!/usr/bin/env python3
"""Generate v3.1 canonical-name alias entries for every table.

Walks `src/tables/<name>/info.rs`, finds the main table struct (the one whose
name matches the dir name in PascalCase), extracts `pub <field>: <type>,`
lines.

Schema-grounded mode (default): consults NattKh's `pabgb_complete_schema.json`
(canonical PA field names extracted from Korean error strings in
CrimsonDesert.exe) and ships an alias only if the mechanical snake→camel
translation matches a known canonical name in the schema. Eliminates
false-positive aliases on placeholder field names and unverified guesses.

Mechanical-only fallback: for tables not present in the schema (4 of 113),
falls back to pure mechanical translation with the placeholder filter.

Output: one Rust file per table containing `pub const FIELD_ALIASES_V3_1` plus
a central `src/json_shape_table_registry.rs` indexing them.

v3 names are NEVER renamed. The Rust struct fields stay as-is. Aliases just
let v3.1 consumers (DMM v2.0.0-beta etc.) request the canonical `_camelCase`
form via shape='v3.1'.
"""
import json
import re
import os
import sys
from pathlib import Path

REPO = Path(r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser")
TABLES_DIR = REPO / "src" / "tables"
SCHEMA_PATH = Path(r"C:\Users\corin\Desktop\CD DUMPING TOOLS\_research_cache\pabgb_complete_schema.json")

# Manual overrides where Rust struct field name doesn't mechanically translate
# to the schema's canonical name. Format: {(table_dispatch_name, rust_snake): canonical_camel}
# Each entry must be cross-validated against NattKh schema or Win-IDA evidence.
MANUAL_OVERRIDES = {
    # effect_info: Rust uses singular forms; schema has plural _List suffix.
    # Confirmed via Win-IDA decompile of sub_1410A8670 (real EffectInfo record
    # reader) cross-referenced against NattKh schema entries (iter 31).
    ("effect_info", "effect_data"):      "_effectDataList",
    ("effect_info", "mesh_effect_data"): "_meshEffectDataList",

    # Acronym-casing divergences (iter 32). Mechanical translation produces
    # camelCase like `_isShowUi`; schema preserves uppercase acronyms like
    # `_isShowUI`. These are EXACT camelCase reverses where the Rust snake
    # form is the unambiguous reverse-translation of the canonical name.
    ("knowledge_info",        "is_show_ui"):           "_isShowUI",
    ("knowledge_group_info",  "is_show_ui"):           "_isShowUI",
    ("knowledge_group_info",  "is_show_uialert"):      "_isShowUIAlert",
    ("mini_game_data_info",   "ui_view_id"):           "_uiViewID",
    ("vehicle_info",          "show_count_on_ui"):     "_showCountOnUI",
    ("status_info",           "status_index_xxxxx"):   "_statusIndexXXXXX",
    ("elemental_material_info", "parent_material_key_list_deprecated_xxx"): "_parentMaterialKeyListDeprecatedXXX",
    ("faction_node_info",     "way_point_data_list_deprecated"): "_wayPointDataList_deprecated",
    ("spawning_pool_auto_spawn_info", "collect_filter_dev"): "_collectFilter_Dev",

    # Singular/plural divergence (canonical has _List suffix; rust does not)
    ("global_game_event_info", "execute_data"):        "_executeDataList",

    # Iter 33: aggressive fuzzy match (normalize-equal across underscore +
    # case). Surfaces more acronym-casing divergences plus a few PA-internal
    # typos (lowercase 'k' in 'key' / 'knowledge').
    ("faction_node_spawn_info",        "patrol_ai_spline_data_list"):        "_patrolAISplineDataList",
    ("interaction_info",               "interaction_show_ui_type"):          "_interactionShowUIType",
    ("inventory_info",                 "push_item_alert_ui_text"):           "_pushItemAlertUIText",
    ("knowledge_info",                 "is_show_ui_alert"):                  "_isShowUIAlert",
    ("platform_achievement_info",      "quest_group_key"):                   "_questGroupkey",  # NB: PA typo, lowercase k
    ("region_info",                    "is_ui_map_disable"):                 "_isUIMapDisable",
    ("region_info",                    "region_enter_knowledge_info_list"):  "_regionEnterknowledgeInfoList",  # NB: PA typo, lowercase k
    ("store_info",                     "custom_mesh_obb_max_length"):        "_customMeshOBBMaxLength",
    ("terrain_region_auto_spawn_info", "spawn_at_height_field_landscape"):   "_spawnAtHeightFieldLandScape",

    # Iter 34: one-of-each high-confidence pairings (count-match heuristic +
    # manual review). Includes 2 more PA-internal typos.
    ("frame_event_attr_group_info",    "data_list"):                         "_frameEventAttributeArr",
    ("game_event_handler_info",        "data"):                              "_gameEventHandlerData",
    ("item_use_info",                  "variant"):                           "_itemUseInfoData",
    ("terrain_region_auto_spawn_info", "fish_summon_time_frequency_type"):   "_fishSummonTimeFrquencyType",  # NB: PA typo, Frquency
    ("equip_info",                     "ragdoll_list"):                      "_radgollEquipTableGroupDataList",  # NB: PA typo, radgoll
    ("special_mode_info",              "option_slots"):                      "_optionList",

    # Iter 70: PA-internal typo "complte" (missing 'e' before 'te' in
    # "complete"). Confirmed via Win-IDA decompile of sub_1410BA4C0
    # (multi_change_info per-record reader, iter 57 typeinfo registry):
    # the LocalizableString read at offset 192 corresponds to the
    # `_complteDescription` canonical from the schema.
    ("multi_change_info",              "complete_description"):              "_complteDescription",  # NB: PA typo, missing 'e'

    # Iter 77: rust field `level_gimmick_scene_object_data_list` was an
    # early guess based on the field's payload type. Win-IDA decompile of
    # sub_1410AFE20 (knowledge_info per-record reader, iter 47 typeinfo
    # registry) shows the final CArray<U32U32Pair> read at offset 248 is
    # the only unmapped wire read, and `_linkKnowledgeNodeList` is the
    # only unmapped schema canonical. Each U32U32Pair = (source node key,
    # target node key) — i.e. knowledge-graph link list. Tuple-scoped
    # override is safe because LevelGimmickSceneObjectInfo (which DOES
    # legitimately use `_levelGimmickSceneObjectDataList`) is keyed
    # separately.
    ("knowledge_info",                 "level_gimmick_scene_object_data_list"): "_linkKnowledgeNodeList",

    # Iter 78: region_info
    # `_overriedMaxHeight` is a PA-internal typo (should be "overridden"
    # or at least "overrided"; PA wrote "overried" — missing 'd'). The
    # rust field name `overrided_max_height` has the same wrong-suffix
    # typo independently. Map them directly.
    ("region_info",                    "overrided_max_height"):              "_overriedMaxHeight",  # NB: PA typo, missing 'd'
    # `_isSaveGimmickRegion` is the canonical for the rust field
    # `is_housing_region`. Housing in Crimson Desert uses save-gimmick
    # regions as the persistence backing — so PA's name is the more
    # technical / accurate one. Tuple-scoped override.
    ("region_info",                    "is_housing_region"):                 "_isSaveGimmickRegion",

    # Iter 80: level_gimmick_scene_object_info — the rust field `data_list`
    # is the table's main payload CArray<LevelGimmickSceneObjectData>;
    # canonical `_levelGimmickSceneObjectDataList` matches it directly.
    # (The same canonical name was identified in iter 77 as the legitimate
    # owner of LevelGimmickSceneObjectInfo's main payload — separate from
    # the knowledge_info closure that aliases a different rust field.)
    ("level_gimmick_scene_object_info", "data_list"):                        "_levelGimmickSceneObjectDataList",

    # Iter 81: global_game_event_group_info — `events: CArray<u16>` is the
    # rust holding for the schema's `_globalGameEventInfoList` (canonical
    # type `reader_2B` = CArray<u16> of GlobalGameEventInfo keys). Safe
    # close. The remaining `_executePercent` (direct_u64) maps to one of
    # `unk_b` or `unk_c` (both u64) — needs sample-data analysis to
    # identify which u64 holds percent-range values; deferred.
    ("global_game_event_group_info",   "events"):                            "_globalGameEventInfoList",
}

# Field-name patterns that are clearly placeholders — skip them, no alias.
PLACEHOLDER_PATTERNS = [
    re.compile(r"^_[a-z]$"),                   # _a, _b
    re.compile(r"^_pad[0-9a-fA-F]+$"),         # _pad0072 (typically used in engine descriptors)
    re.compile(r"^_unk[0-9a-fA-F]+$"),         # _unk0073
    re.compile(r"^field_[a-z]$"),              # field_a, field_b
    re.compile(r"^raw_[a-z]$"),                # raw_a, raw_b
    re.compile(r"^lookup_[a-z]$"),             # lookup_a, lookup_b
    re.compile(r"^block_[a-z](_[a-z_]+)?$"),   # block_a_floats, block_b
    re.compile(r"^flag_[a-z]$"),               # flag_a, flag_b
    re.compile(r"^[a-z]_dword_\d+$"),          # block_a_dword_0
    re.compile(r"^header_dword_\d+$"),         # header_dword_0
    re.compile(r"^raw_block_[a-z]_dword_\d+$"),
]

def is_placeholder(name: str) -> bool:
    for pat in PLACEHOLDER_PATTERNS:
        if pat.match(name):
            return True
    return False

def snake_to_underscore_camel(snake: str) -> str:
    """Convert `buff_level_list` → `_buffLevelList` (Pearl Abyss convention).

    Splits on underscores, keeps first segment lowercase, capitalizes the
    rest, joins, prepends an underscore.

    Edge cases:
    - Single-segment names get `_` prefix only (`key` → `_key`)
    - Empty segments collapse (multiple underscores in a row → single underscore)
    - Already-prefixed names get treated as-is for the underscore (just camelize the rest)
    """
    parts = [p for p in snake.split("_") if p]  # drop empty segments
    if not parts:
        return ""
    first = parts[0]
    rest = [p[0].upper() + p[1:] if p else "" for p in parts[1:]]
    return "_" + first + "".join(rest)

def snake_to_pascal(snake: str) -> str:
    return "".join(p[0].upper() + p[1:] for p in snake.split("_") if p)

def extract_main_struct_fields(info_rs_path: Path, dir_name: str,
                                schema_canonical_names: set[str] | None,
                                manual_overrides: dict[tuple[str, str], str] | None = None) -> list[tuple[str, str]]:
    """Return list of (rust_snake_case, _camelCase) field-name tuples for the
    main table struct.

    If `schema_canonical_names` is provided, only ship an alias when the
    mechanical translation matches a name in the set (schema-verified mode).
    If None, fall back to pure mechanical translation with the placeholder
    filter.

    Returns [] if main struct not found or has no fields.
    """
    src = info_rs_path.read_text(encoding="utf-8")
    main_struct_name = snake_to_pascal(dir_name)

    # Match `pub struct <Name>[<lifetime>] { ... }` non-greedily.
    # The struct body is between { and matching }.
    pattern = re.compile(
        r"pub\s+struct\s+" + re.escape(main_struct_name) + r"\s*(?:<[^>]+>)?\s*\{([^{}]*)\}",
        re.DOTALL,
    )
    m = pattern.search(src)
    if not m:
        # Try inside py_binary_struct! { pub struct Name { ... } }
        pattern2 = re.compile(
            r"py_binary_struct!\s*\{\s*(?:[^{}]*?)pub\s+struct\s+"
            + re.escape(main_struct_name)
            + r"\s*(?:<[^>]+>)?\s*\{([^{}]*)\}",
            re.DOTALL,
        )
        m = pattern2.search(src)
    if not m:
        return []

    body = m.group(1)
    field_pattern = re.compile(r"^\s*pub\s+([a-z_][a-z0-9_]*)\s*:", re.MULTILINE)
    fields = field_pattern.findall(body)

    aliases = []
    for snake in fields:
        if is_placeholder(snake):
            continue
        # Manual override takes precedence (for fields where Rust snake_case
        # doesn't mechanically translate to the schema's canonical name).
        override_key = (dir_name, snake)
        if manual_overrides and override_key in manual_overrides:
            aliases.append((snake, manual_overrides[override_key]))
            continue
        camel = snake_to_underscore_camel(snake)
        if schema_canonical_names is not None:
            # Schema-grounded: ship only verified canonical names.
            if camel not in schema_canonical_names:
                continue
        aliases.append((snake, camel))

    return aliases


def load_schema_field_names(schema_entry) -> set[str]:
    """Return canonical `_camelCase` field names from a schema entry."""
    if not isinstance(schema_entry, list):
        return set()
    return {e["f"] for e in schema_entry if isinstance(e, dict) and "f" in e}

def main():
    schema = {}
    schema_loaded = False
    if SCHEMA_PATH.exists():
        try:
            schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
            schema_loaded = True
            print(f"[generate] loaded schema: {len(schema)} canonical class entries")
        except Exception as e:
            print(f"[generate] WARNING: schema load failed: {e} — falling back to mechanical mode")
    else:
        print(f"[generate] WARNING: schema not found at {SCHEMA_PATH} — mechanical mode only")

    tables = sorted([p.name for p in TABLES_DIR.iterdir() if p.is_dir() and (p / "info.rs").exists()])
    print(f"[generate] found {len(tables)} table modules")

    schema_grounded_count = 0
    fallback_count = 0
    rows = []         # (table_name, num_fields_aliased, source)
    central = []      # central registry rows
    for t in tables:
        info_rs = TABLES_DIR / t / "info.rs"
        if schema_loaded:
            schema_key = snake_to_pascal(t)
            schema_entry = schema.get(schema_key)
            schema_names = load_schema_field_names(schema_entry) if schema_entry else None
        else:
            schema_names = None
        if schema_names:
            schema_grounded_count += 1
            source = "schema"
        else:
            fallback_count += 1
            source = "mechanical"
        aliases = extract_main_struct_fields(info_rs, t, schema_names, MANUAL_OVERRIDES)
        if aliases:
            entries = ",\n    ".join(f'("{snake}", "{camel}")' for snake, camel in aliases)
            provenance = (
                "Schema-verified: each canonical name was matched against\n"
                "// NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`\n"
                "// (canonical PA identifiers extracted from Korean error strings\n"
                "// in CrimsonDesert.exe). Only fields whose mechanical snake→camel\n"
                "// translation matched a known canonical name were shipped."
            ) if source == "schema" else (
                "Mechanical translation only: this table is not present in NattKh's\n"
                "// pabgb_complete_schema.json, so canonical names could not be\n"
                "// independently verified. Aliases are derived from the Rust struct\n"
                "// field name via snake → `_camelCase` conversion."
            )
            const = f"""// Auto-generated by scripts/generate_v3_1_aliases.py — do NOT hand-edit.
// To regenerate, re-run the script after any table struct field changes.
//
// {provenance}
//
// v3 (snake_case) is the default emit; v3.1 emits the canonical `_camelCase`
// form. Round-trips identically — both names accepted on input.

pub const FIELD_ALIASES_V3_1: &[(&str, &str)] = &[
    {entries},
];
"""
            (TABLES_DIR / t / "field_aliases_v3_1.rs").write_text(const, encoding="utf-8")
            central.append(f'    ("{t}", crate::tables::{t}::field_aliases_v3_1::FIELD_ALIASES_V3_1),')
            rows.append((t, len(aliases), source))
        else:
            # No aliases: write an empty file so the central registry can still
            # link cleanly, and so re-runs don't leave stale per-field data.
            empty_const = """// Auto-generated by scripts/generate_v3_1_aliases.py — do NOT hand-edit.
// No v3.1 canonical aliases for this table (no schema match or no fields
// match the canonical name set). v3 (snake_case) and v3.1 emit identically.

pub const FIELD_ALIASES_V3_1: &[(&str, &str)] = &[];
"""
            (TABLES_DIR / t / "field_aliases_v3_1.rs").write_text(empty_const, encoding="utf-8")
            central.append(f'    ("{t}", crate::tables::{t}::field_aliases_v3_1::FIELD_ALIASES_V3_1),')
            rows.append((t, 0, source))

    # Write central registry.
    central_rs = REPO / "src" / "json_shape_table_registry.rs"
    central_text = """// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Auto-generated by scripts/generate_v3_1_aliases.py — do NOT hand-edit.
//
// Central index of per-table v3.1 alias tables. Maps table-dispatch name
// (the snake_case identifier used by `dispatch::parse_table_to_json`) to
// the table's `FIELD_ALIASES_V3_1` const.
//
// Lookup function: `crate::json_shape::lookup_table_aliases(name)`.

pub static TABLE_FIELD_ALIASES_V3_1: &[(&str, &[(&str, &str)])] = &[
""" + "\n".join(central) + """
];
"""
    central_rs.write_text(central_text, encoding="utf-8")

    print(f"[generate] wrote {len(central)} per-table alias files")
    print(f"[generate] schema-grounded: {schema_grounded_count}, mechanical fallback: {fallback_count}")
    print(f"[generate] wrote central registry at {central_rs}")
    print()
    print(f"{'TABLE':40} {'FIELDS':>8}  {'SOURCE'}")
    for t, n, src in rows:
        print(f"{t:40} {n:>8}  {src}")

if __name__ == "__main__":
    main()
