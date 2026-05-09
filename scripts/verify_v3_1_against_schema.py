#!/usr/bin/env python3
"""Cross-reference dmm-parser's per-table FIELD_ALIASES_V3_1 against
NattKh's pabgb_complete_schema.json.

For each table:
  1. Map dmm-parser dispatch-name (snake_case) to schema key (PascalCase).
  2. Read the per-table FIELD_ALIASES_V3_1 const → list of (snake, camel) pairs.
  3. Read the schema entry → list of canonical `_camelCase` field names.
  4. Cross-check:
     - MATCH:    our (snake, camel) where camel == schema name → verified
     - MISMATCH: our camel does NOT match any schema name → mechanical rule
                 produced wrong identifier
     - MISSING:  schema has names dmm-parser doesn't expose → decoder gap
     - EXTRA:    dmm-parser has aliases for fields not in schema (likely
                 sub-struct fields, padding, or schema gap)

Output: a markdown report at docs/V3_1_SCHEMA_VERIFICATION.md plus a JSON
summary at docs/v3_1_schema_verification.json (machine-readable).
"""
import json
import re
from pathlib import Path

REPO = Path(r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser")
TABLES_DIR = REPO / "src" / "tables"
SCHEMA_PATH = Path(r"C:\Users\corin\Desktop\CD DUMPING TOOLS\_research_cache\pabgb_complete_schema.json")
REPORT_MD   = REPO / "docs" / "V3_1_SCHEMA_VERIFICATION.md"
REPORT_JSON = REPO / "docs" / "v3_1_schema_verification.json"


def snake_to_pascal(snake: str) -> str:
    return "".join(p[0].upper() + p[1:] for p in snake.split("_") if p)


def load_aliases(field_aliases_path: Path) -> list[tuple[str, str]]:
    """Parse FIELD_ALIASES_V3_1 entries from an alias file."""
    if not field_aliases_path.exists():
        return []
    src = field_aliases_path.read_text(encoding="utf-8")
    pairs = re.findall(r'\("([^"]+)",\s*"([^"]+)"\)', src)
    return pairs


def load_schema_field_names(schema_entry) -> list[str]:
    """Schema entry is a list of dicts each with key 'f' = canonical name."""
    if not isinstance(schema_entry, list):
        return []
    return [e["f"] for e in schema_entry if isinstance(e, dict) and "f" in e]


def main():
    schema = json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))
    print(f"[verify] schema loaded: {len(schema)} classes")

    tables = sorted(p.name for p in TABLES_DIR.iterdir()
                    if p.is_dir() and (p / "field_aliases_v3_1.rs").exists())
    print(f"[verify] dmm-parser tables with aliases: {len(tables)}")

    summary = {
        "total_tables": len(tables),
        "tables_in_schema": 0,
        "tables_missing_from_schema": [],
        "per_table": {},
        "totals": {
            "verified": 0,
            "mismatch": 0,
            "missing_in_dmm": 0,
            "extra_in_dmm": 0,
        },
    }

    md = ["# v3.1 Alias Verification Against NattKh Schema", ""]
    md.append("Cross-reference of dmm-parser's mechanically-generated v3.1 aliases")
    md.append("against the canonical Pearl Abyss field names in NattKh's schema")
    md.append("(extracted from Korean error strings in CrimsonDesert.exe).")
    md.append("")
    md.append("Schema source: https://github.com/NattKh/CrimsonDesertModdingTools")
    md.append("")

    for tbl in tables:
        schema_key = snake_to_pascal(tbl)
        aliases = load_aliases(TABLES_DIR / tbl / "field_aliases_v3_1.rs")
        schema_entry = schema.get(schema_key)
        schema_names = set(load_schema_field_names(schema_entry)) if schema_entry else set()

        if schema_entry is None:
            summary["tables_missing_from_schema"].append(tbl)
            summary["per_table"][tbl] = {
                "schema_key": schema_key,
                "in_schema": False,
                "alias_count": len(aliases),
            }
            continue

        summary["tables_in_schema"] += 1

        verified, mismatch, extra = [], [], []
        our_camels = set()
        for snake, camel in aliases:
            our_camels.add(camel)
            if camel in schema_names:
                verified.append((snake, camel))
            else:
                mismatch.append((snake, camel))

        missing_in_dmm = sorted(schema_names - our_camels)

        summary["totals"]["verified"] += len(verified)
        summary["totals"]["mismatch"] += len(mismatch)
        summary["totals"]["missing_in_dmm"] += len(missing_in_dmm)
        summary["totals"]["extra_in_dmm"] += len(extra)

        summary["per_table"][tbl] = {
            "schema_key": schema_key,
            "in_schema": True,
            "alias_count": len(aliases),
            "schema_field_count": len(schema_names),
            "verified": len(verified),
            "mismatch": [{"rust_snake": s, "our_camel": c} for s, c in mismatch],
            "missing_in_dmm_count": len(missing_in_dmm),
            "missing_in_dmm": missing_in_dmm,
        }

    md.append("## Summary")
    md.append("")
    md.append(f"- Tables with aliases in dmm-parser: **{summary['total_tables']}**")
    md.append(f"- Tables present in NattKh schema:    **{summary['tables_in_schema']}**")
    md.append(f"- Tables missing from schema:         **{len(summary['tables_missing_from_schema'])}**")
    md.append(f"- Total field aliases verified:       **{summary['totals']['verified']}**")
    md.append(f"- Total mechanical-rule mismatches:   **{summary['totals']['mismatch']}**")
    md.append(f"- Total schema fields not decoded:    **{summary['totals']['missing_in_dmm']}**")
    md.append("")

    if summary["tables_missing_from_schema"]:
        md.append("## Tables not in NattKh schema")
        md.append("")
        md.append("These tables exist in dmm-parser but the schema doesn't have a")
        md.append("matching PascalCase entry. Possible reasons: schema gap, table")
        md.append("name divergence, or dmm-parser table doesn't go through the")
        md.append("Korean-error-string parser path.")
        md.append("")
        for t in summary["tables_missing_from_schema"]:
            md.append(f"- `{t}` (expected schema key: `{snake_to_pascal(t)}`)")
        md.append("")

    md.append("## Per-table details")
    md.append("")
    md.append("| Table | Aliases | Verified | Mismatches | Missing in dmm-parser |")
    md.append("|---|---|---|---|---|")
    for tbl, info in summary["per_table"].items():
        if info.get("in_schema"):
            mc = len(info["mismatch"])
            mw = info["missing_in_dmm_count"]
            md.append(f"| `{tbl}` | {info['alias_count']} | {info['verified']} | {mc} | {mw} |")
    md.append("")

    # Detail block for any table with mismatches or missing fields
    detail_tables = [t for t, i in summary["per_table"].items()
                     if i.get("in_schema") and (i["mismatch"] or i["missing_in_dmm_count"])]
    if detail_tables:
        md.append("## Mismatch / missing-field detail")
        md.append("")
        for tbl in detail_tables:
            info = summary["per_table"][tbl]
            md.append(f"### `{tbl}` (schema key: `{info['schema_key']}`)")
            if info["mismatch"]:
                md.append("")
                md.append("**Mismatches** — dmm-parser's mechanical translation produced a name")
                md.append("not present in the schema. Likely a wrong canonical name.")
                md.append("")
                md.append("| Rust snake | Our v3.1 (mechanical) | Status |")
                md.append("|---|---|---|")
                for m in info["mismatch"]:
                    md.append(f"| `{m['rust_snake']}` | `{m['our_camel']}` | NOT IN SCHEMA |")
                md.append("")
            if info["missing_in_dmm_count"]:
                md.append("")
                md.append(f"**Schema fields not in dmm-parser** ({info['missing_in_dmm_count']}):")
                md.append("")
                for f in info["missing_in_dmm"]:
                    md.append(f"- `{f}`")
                md.append("")

    REPORT_MD.write_text("\n".join(md), encoding="utf-8")
    REPORT_JSON.write_text(json.dumps(summary, indent=2), encoding="utf-8")
    print(f"[verify] markdown report: {REPORT_MD}")
    print(f"[verify] json report:     {REPORT_JSON}")
    print()
    print("=== Top-level totals ===")
    print(f"  tables in schema:     {summary['tables_in_schema']} / {summary['total_tables']}")
    print(f"  verified aliases:     {summary['totals']['verified']}")
    print(f"  mismatch aliases:     {summary['totals']['mismatch']}")
    print(f"  missing-in-dmm:       {summary['totals']['missing_in_dmm']}")


if __name__ == "__main__":
    main()
