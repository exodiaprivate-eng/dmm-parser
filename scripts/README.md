# scripts/

Utility scripts for dmm-parser maintenance. Each Python file is
self-contained — run with `python scripts/<name>.py` from the repo root.

## Active scripts

### `generate_v3_1_aliases.py` (237 lines)

**Purpose:** Bulk-generate per-table `FIELD_ALIASES_V3_1` Rust consts
that map snake_case Rust struct fields to canonical Pearl Abyss
`_camelCase` identifiers.

**Schema-grounded mode** (default): consults
`_research_cache/pabgb_complete_schema.json` (NattKh's canonical PA
names extracted from Korean error strings in CrimsonDesert.exe). Only
ships an alias when the mechanical snake→camel translation matches a
known canonical name. Eliminates false-positive aliases.

**Mechanical fallback:** for tables not in the schema (4 of 122),
uses pure mechanical translation with a placeholder filter.

**Outputs:**
- `src/tables/<name>/field_aliases_v3_1.rs` (one per table)
- `src/json_shape_table_registry.rs` (central index)

**When to re-run:** after any change to `src/tables/<name>/info.rs`
struct definitions, or when NattKh's schema is updated.

```bash
python scripts/generate_v3_1_aliases.py
```

### `verify_v3_1_against_schema.py` (197 lines)

**Purpose:** Audit script that cross-references every shipped v3.1
alias against NattKh's schema. Reports verified / mismatch / missing
counts per-table.

**Outputs:**
- `docs/V3_1_SCHEMA_VERIFICATION.md` (markdown report)
- `docs/v3_1_schema_verification.json` (machine-readable)

**When to re-run:** after `generate_v3_1_aliases.py` regeneration, or
when checking decoder-gap progress.

```bash
python scripts/verify_v3_1_against_schema.py
```

### `find_singleton_closures.py` (~95 lines)

**Purpose:** Scan all gap tables for type-singleton closure
opportunities — missing canonicals where a type group has exactly 1
entry, candidates for the highest-confidence "type-unique singleton
match" closure technique (per `docs/V3_1_CLOSURE_METHODOLOGY.md`
technique 1).

Run BEFORE attempting per-canonical work on any gap table to find
the easy wins first. Surfaces opportunities like the iter 96/97
ambiguity-resolution closes and iter 139's faction_node_info
`_researchDataList` close.

```bash
python scripts/find_singleton_closures.py
```

Output sorts by gap-count ascending so the smallest tables (easiest
wins) appear first.

### `audit_manual_overrides.py` (~85 lines)

**Purpose:** Validate that every entry in `MANUAL_OVERRIDES` (in
`generate_v3_1_aliases.py`) targets an existing `pub <field>:`
declaration in `src/tables/<table>/info.rs`.

**Why this matters:** iter 122 fixed a silent-drop bug where the
`is_placeholder` filter ran BEFORE the `MANUAL_OVERRIDES` check,
silently dropping overrides for placeholder-pattern names like
`lookup_a` (matching `^lookup_[a-z]$`). This script catches:
- Overrides that target renamed/removed rust fields
- Override field-name typos (silent-drop has same effect as success)
- Overrides re-introduced for placeholder-pattern names if the
  pre-iter-122 generator code is ever re-applied

Exit code 0 if all overrides are valid, 1 if any are stale (with
details).

```bash
python scripts/audit_manual_overrides.py
```

### `harvest_reflection_schema.py` (102 lines)

**Purpose:** Walks a directory of pycrimson-parsed JSON output and
builds a class→fields catalog. Used to harvest canonical Pearl Abyss
class+field names from reflection-format files (`.prefab`, `.parg`,
`.pasg`, `.paseqc`, `.paa_metabin`).

**Output:**
- `docs/v3_1_reflection_schema.json` (~2.5 MB)

**Pipeline (see V3_1_PYCRIMSON_WORKFLOW.md for full setup):**
1. `pycrimson extract-pack-files` to pull files from PAZ archives
2. `pycrimson parse-serialized-file` to dump each as JSON
3. This script aggregates the dumps into the catalog

```bash
python scripts/harvest_reflection_schema.py <parsed-json-dir>
```

### `add_json_roundtrip.py` (266 lines)

**Purpose:** Inserts a `json_roundtrip` test into each table's
`info.rs`. Historical bulk-insertion script used during the Tier 1
promotion arc; kept for reference + occasional one-off table additions.

**Status:** Mostly historical. New tables added by hand or via the
table-creation pattern in existing tables. Not regularly invoked.

### `add_cdmtl_headers.py` (185 lines)

**Purpose:** Adds CDMTL v1.0 copyright/CMI headers to all source files
across the suite (dmm-parser + DMM + CrimsonGameMods). One-time bulk
operation; has been run.

**Status:** Run once at CDMTL adoption (Session 23 era). Re-run only
when new source files are added without headers, or when the license
text changes.

### `deploy_cdmtl_license.py` (111 lines)

**Purpose:** Strips DRAFT markers and reviewer notes from
`docs/LEGAL.md` (formerly `LICENSE_DRAFT_v1.md`) and deploys it as
the canonical `LICENSE.txt` at repo root.

**Status:** Run when shipping a new license version.

## Data files

### `json_targets.txt`

37-line pipe-delimited list of `dispatch_name|PascalCaseClass` for
table-targeting bulk operations. Used by historical bulk-edit scripts.

## Naming conventions

- All scripts use absolute paths via `pathlib.Path` (the repo path is
  hardcoded at the top of each script — see `REPO = Path(...)`).
- All schema/research data lives in
  `C:\Users\corin\Desktop\CD DUMPING TOOLS\_research_cache\` (NOT
  committed; see `_research_cache/` in `.gitignore` if added).
- All generated docs land in `docs/` (committed).
- All generated Rust source lands in `src/tables/<table>/` (committed).

## Adding a new script

1. Place in `scripts/` with a leading `"""` docstring describing
   purpose + when to run.
2. Add an entry to this README under "Active scripts" or
   "Historical scripts" as appropriate.
3. If the script generates committed output, document the output path
   + when to regenerate.
