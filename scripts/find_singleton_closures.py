#!/usr/bin/env python3
"""Scan all gap tables for type-singleton closure opportunities.

For each table with v3.1 gaps remaining, find missing canonicals where
a type group has exactly 1 entry — these are candidates for the
"type-unique singleton" closure technique (highest-confidence pattern
per `docs/V3_1_CLOSURE_METHODOLOGY.md`).

Use this BEFORE attempting per-canonical work on any gap table to find
the easy wins first.

Output
------
For each gap table (sorted by gap count ascending), prints:
- table name + missing-canonical count
- type-singleton candidates: (schema-type, canonical-name) tuples

Tables already at 0 gaps are skipped.

Background
----------
Iter 139 used this scan to find faction_node_info `_researchDataList`
as a type-singleton (only None-typed list missing) with the only
top-level CArray-of-struct unaliased rust field — shipped in 1 iter.

Iter 96+97 used the same pattern manually to close `_executePercent`
and `_onDiscoverOnlyEnable` (only direct_u64 / direct_u8 missing of
their kind in their tables).

Iter 112 (4 LocalizableString labels) used a related N-to-N pattern
where a TYPE GROUP had exactly N missing canonicals matching N rust
unaliased fields of the same shape.
"""
import json
import re
import sys
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SCHEMA_PATH = Path(
    r"C:\Users\corin\Desktop\CD DUMPING TOOLS\_research_cache\pabgb_complete_schema.json"
)
VERIF_PATH = REPO / "docs" / "v3_1_schema_verification.json"
TABLES_DIR = REPO / "src" / "tables"

# Schema-type to rust-type mapping for the rust-side singleton check.
# Conservative — only types where we can confidently grep the rust struct.
SCHEMA_TO_RUST_TYPES = {
    "direct_u8":    [r": u8\b"],
    "direct_u16":   [r": u16\b"],
    "direct_u32":   [r": u32\b"],
    "direct_u64":   [r": u64\b"],
    "direct_12B":   [r": \[f32; 3\]"],     # Vec3
    "direct_15B":   [r": u8\b"],            # 15B = setter dispatch class for u8
    "reader_1B":    [r": u8\b"],            # u8 hash via lookup
    "reader_2B":    [r": u16\b"],           # u16 hash via lookup
    "reader_4B":    [r": u32\b", r"CArray<u32>"],  # u32 hash OR CArray<u32>
    "reader_8B":    [r"LocalizableString"],  # 32 wire bytes / fat pointer
    # 'None' and 'array_or_complex' too varied to grep reliably
}


def get_unaliased_rust_field_count(table: str, rust_type_patterns: list[str]) -> int | None:
    """Count rust struct fields matching any of the type patterns whose
    snake_case name is NOT in the table's existing FIELD_ALIASES_V3_1.

    Returns None if struct or alias file can't be located (graceful skip).

    KNOWN LIMITATION (iter 157): the regex-based struct body extraction
    sometimes returns 0 even for tables with clearly unaliased fields.
    The "[BLOCKED rust 0]" annotation should therefore be treated as
    "needs manual verification" rather than authoritative — re-grep
    `src/tables/<table>/info.rs` to confirm. The tool's [SHIPPABLE]
    annotation IS reliable when it appears (rust-side singleton confirmed).
    """
    info_path = TABLES_DIR / table / "info.rs"
    alias_path = TABLES_DIR / table / "field_aliases_v3_1.rs"
    if not info_path.exists():
        return None

    info_src = info_path.read_text(encoding="utf-8")
    aliased_fields: set[str] = set()
    if alias_path.exists():
        for m in re.finditer(r'\("(\w+)"\s*,', alias_path.read_text(encoding="utf-8")):
            aliased_fields.add(m.group(1))

    # Locate the main struct body (top-level pub struct <PascalName>)
    pascal = "".join(p[0].upper() + p[1:] for p in table.split("_") if p)
    struct_match = re.search(
        r"pub\s+struct\s+" + re.escape(pascal) + r"\b[^{]*\{([^{}]*)\}",
        info_src, re.DOTALL,
    )
    if not struct_match:
        return None
    body = struct_match.group(1)

    # Count `pub <name>: <type>,` lines where type matches AND name not aliased
    count = 0
    for m in re.finditer(r"^\s*pub\s+(\w+)\s*:\s*([^,\n]+)", body, re.MULTILINE):
        snake, rust_type = m.group(1), m.group(2).strip()
        if snake in aliased_fields:
            continue
        for pat in rust_type_patterns:
            if re.search(pat, rust_type):
                count += 1
                break
    return count


def main() -> int:
    if not SCHEMA_PATH.exists():
        sys.stderr.write(f"ERROR: schema not found at {SCHEMA_PATH}\n")
        return 2
    if not VERIF_PATH.exists():
        sys.stderr.write(
            f"ERROR: verifier output not found at {VERIF_PATH}\n"
            "Run: python scripts/verify_v3_1_against_schema.py first.\n"
        )
        return 2

    schema = json.load(SCHEMA_PATH.open(encoding="utf-8"))
    verif = json.load(VERIF_PATH.open(encoding="utf-8"))

    gap_tables = [
        (name, t)
        for name, t in verif["per_table"].items()
        if t.get("missing_in_dmm_count", 0) > 0
    ]

    print(f"{len(gap_tables)} tables with gaps. Scanning for type-singletons...\n")

    tables_with_singletons = 0
    total_singletons = 0

    for name, t in sorted(gap_tables, key=lambda x: x[1]["missing_in_dmm_count"]):
        schema_name = t.get("schema_key")
        if not schema_name:
            continue
        fields = schema.get(schema_name, [])
        if not fields:
            continue
        missing = set(t["missing_in_dmm"])
        type_counts: Counter = Counter()
        for x in fields:
            if x.get("f") in missing:
                type_counts[str(x.get("type"))] += 1
        singletons = [
            (
                typ,
                next(
                    x.get("f") for x in fields
                    if x.get("f") in missing and str(x.get("type")) == typ
                ),
            )
            for typ, c in type_counts.items()
            if c == 1
        ]
        if singletons:
            tables_with_singletons += 1
            total_singletons += len(singletons)
            print(f"  {name} ({t['missing_in_dmm_count']} gaps total):")
            for typ, canonical in singletons:
                # Annotate with rust-side type-singleton check (iter 155 lesson)
                rust_patterns = SCHEMA_TO_RUST_TYPES.get(typ)
                if rust_patterns:
                    rust_count = get_unaliased_rust_field_count(name, rust_patterns)
                    if rust_count is None:
                        marker = "[?]"
                    elif rust_count == 1:
                        marker = "[SHIPPABLE]"  # Both sides singleton
                    elif rust_count == 0:
                        marker = "[BLOCKED rust 0]"  # Likely class-2 sub-struct
                    else:
                        marker = f"[N-to-1 rust={rust_count}]"
                else:
                    marker = "[type ungreppable]"
                print(f"    [{typ}] {canonical}  {marker}")
            print()

    print(
        f"Summary: {tables_with_singletons} of {len(gap_tables)} gap tables have "
        f"type-singleton candidates ({total_singletons} singletons total)."
    )
    print(
        "\nNext step: for each singleton, check if there's exactly 1 rust\n"
        "unaliased field of the matching shape. If yes, ship a tuple-scoped\n"
        "MANUAL_OVERRIDES entry per the docs/V3_1_CLOSURE_METHODOLOGY.md\n"
        "technique 1 (Type-unique singleton match)."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
