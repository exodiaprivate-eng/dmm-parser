#!/usr/bin/env python3
"""Harvest canonical Pearl Abyss class+field names from pycrimson-parsed
reflection output.

pycrimson dumps reflection-format files (.prefab, .meshinfo, .pae, .paem,
.parg, .pasg, .paa_metabin, .paseq, .uianiminit, .paseqc, .palevel) as
nested JSON with `"__pycr_type__": "ClassName"` markers and the actual
canonical field names as object keys (every key starting with `_` is a
canonical PA identifier).

This script walks a directory of parsed JSON files, collects every
(class_name, field_set) pair, merges across all files, and writes a
per-class field catalog to docs/v3_1_reflection_schema.json.

Usage:
  python scripts/harvest_reflection_schema.py <parsed-json-dir>

Where <parsed-json-dir> is a directory containing pycrimson output
(produced by `pycrimson parse-serialized-file --output-path <path>` for
each reflection file).

Output: docs/v3_1_reflection_schema.json with shape:
  {
    "ClassName": {
      "fields": ["_field1", "_field2", ...],
      "files_observed_in": 42,
      "first_observed_in": "path/to/sample.prefab.json"
    },
    ...
  }

This complements NattKh's pabgb_complete_schema.json (which covers
.pabgb tables only) by adding coverage for reflection-format classes
(SceneObject, SkinnedMeshComponent, ResourceReferencePath_*, etc.).
"""
import json
import sys
from pathlib import Path

REPO = Path(r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser")
OUT_PATH = REPO / "docs" / "v3_1_reflection_schema.json"


def collect(node, classes: dict, src_path: str):
    if isinstance(node, dict):
        cls = node.get("__pycr_type__")
        if cls:
            entry = classes.setdefault(cls, {
                "fields": set(),
                "files_observed_in": set(),
                "first_observed_in": src_path,
            })
            for k in node.keys():
                if k.startswith("_") and not k.startswith("__pycr"):
                    entry["fields"].add(k)
            entry["files_observed_in"].add(src_path)
        for v in node.values():
            collect(v, classes, src_path)
    elif isinstance(node, list):
        for v in node:
            collect(v, classes, src_path)


def main():
    if len(sys.argv) < 2:
        print(f"usage: {sys.argv[0]} <parsed-json-dir>")
        sys.exit(1)
    parsed_dir = Path(sys.argv[1])
    if not parsed_dir.is_dir():
        print(f"not a directory: {parsed_dir}")
        sys.exit(1)

    classes: dict = {}
    files = list(parsed_dir.rglob("*.json"))
    print(f"[harvest] scanning {len(files)} parsed JSON files")
    for i, p in enumerate(files):
        try:
            data = json.loads(p.read_text(encoding="utf-8"))
        except Exception as e:
            print(f"  skip {p.name}: {e}")
            continue
        collect(data, classes, str(p.relative_to(parsed_dir)))
        if (i + 1) % 500 == 0:
            print(f"  ... {i+1}/{len(files)} files, {len(classes)} classes")

    out = {
        cls: {
            "fields": sorted(d["fields"]),
            "field_count": len(d["fields"]),
            "files_observed_in": len(d["files_observed_in"]),
            "first_observed_in": d["first_observed_in"],
        }
        for cls, d in sorted(classes.items())
    }
    OUT_PATH.write_text(json.dumps(out, indent=2), encoding="utf-8")
    print(f"[harvest] wrote {len(out)} classes to {OUT_PATH}")
    total_fields = sum(c["field_count"] for c in out.values())
    print(f"[harvest] total canonical fields harvested: {total_fields}")


if __name__ == "__main__":
    main()
