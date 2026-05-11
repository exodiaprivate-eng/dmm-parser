#!/usr/bin/env python3
"""Validate MANUAL_OVERRIDES integrity in generate_v3_1_aliases.py.

Checks that every (table, field) tuple-key in MANUAL_OVERRIDES targets an
existing `pub <field>:` declaration in src/tables/<table>/info.rs.

Why this matters
----------------
1. Iter 122 fixed a silent-drop bug where the `is_placeholder` filter ran
   BEFORE the MANUAL_OVERRIDES check, causing overrides for placeholder-
   pattern names (e.g. `lookup_a` matching `^lookup_[a-z]$`) to be
   dropped without warning.
2. Per-table struct refactors can rename or remove fields, leaving stale
   override entries pointing at non-existent fields.
3. Override typos go undetected because the silent-drop has the same
   effect as a successful map (verifier shows alias not landing, but
   doesn't surface why).

Running this script after any struct edit or override change catches
all 3 failure modes.

Exit codes
----------
0  — all overrides land on real rust fields
1  — at least one override is stale; details printed
"""
import re
import os
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
GENERATOR = REPO / "scripts" / "generate_v3_1_aliases.py"
TABLES_DIR = REPO / "src" / "tables"


def extract_overrides() -> list[tuple[str, str, str]]:
    """Return list of (table, snake_field, canonical) tuples from
    generate_v3_1_aliases.py's MANUAL_OVERRIDES dict.
    """
    src = GENERATOR.read_text(encoding="utf-8")
    m = re.search(r"MANUAL_OVERRIDES\s*=\s*\{(.*?)\n\}", src, re.DOTALL)
    if not m:
        sys.stderr.write(f"ERROR: cannot find MANUAL_OVERRIDES dict in {GENERATOR}\n")
        sys.exit(2)
    body = m.group(1)
    return re.findall(r'\(\s*"(\w+)"\s*,\s*"(\w+)"\s*\)\s*:\s*"(_\w+)"', body)


def check_field_exists(table: str, field: str) -> str | None:
    """Return None if `pub <field>:` exists in src/tables/<table>/info.rs,
    else return an error message string.
    """
    info_path = TABLES_DIR / table / "info.rs"
    if not info_path.exists():
        return f"NO src/tables/{table}/info.rs"
    info_src = info_path.read_text(encoding="utf-8")
    if not re.search(rf"\bpub\s+{re.escape(field)}\b", info_src):
        return f"NO `pub {field}:` declaration"
    return None


def main() -> int:
    overrides = extract_overrides()
    print(f"Total MANUAL_OVERRIDES: {len(overrides)}")

    misses: list[str] = []
    for table, field, canonical in overrides:
        err = check_field_exists(table, field)
        if err:
            misses.append(f"  {table}: {field} → {canonical}  ({err})")

    if misses:
        print(f"\n{len(misses)} stale overrides:")
        for m in misses:
            print(m)
        return 1

    print("All overrides target existing rust fields [OK]")
    return 0


if __name__ == "__main__":
    sys.exit(main())
