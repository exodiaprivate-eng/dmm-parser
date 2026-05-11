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
import sys
from collections import Counter
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SCHEMA_PATH = Path(
    r"C:\Users\corin\Desktop\CD DUMPING TOOLS\_research_cache\pabgb_complete_schema.json"
)
VERIF_PATH = REPO / "docs" / "v3_1_schema_verification.json"


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
                print(f"    [{typ}] {canonical}")
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
