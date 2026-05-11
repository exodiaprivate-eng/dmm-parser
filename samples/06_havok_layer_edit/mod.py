"""Sample 06 — Havok-Layer File Edit

Reads a .pami (Static Mesh Instance) file, swaps a mesh path, writes
back. Demonstrates the read → mutate → write flow that all 24 new
Tier 1 Havok-layer + non-Havok parsers support.

Usage:
    python mod.py <in.pami> <out.pami> <old_mesh_path> <new_mesh_path>

Example:
    python mod.py 03_cube.pami patched.pami \
        "object/03_cube.pa..." "object/03_sphere.pa..."

Exits 0 on success, 1 on missing args or no-op edit.
"""

from __future__ import annotations

import sys
from pathlib import Path


def main() -> int:
    if len(sys.argv) != 5:
        print(__doc__)
        return 1
    in_path = Path(sys.argv[1])
    out_path = Path(sys.argv[2])
    old_mesh = sys.argv[3]
    new_mesh = sys.argv[4]

    try:
        import dmm_parser as cr
    except ImportError:
        print("ERROR: dmm_parser module not installed.")
        print("Build + install the wheel from C:/.../dmm-parser:")
        print("  maturin build --release && pip install --force-reinstall target/wheels/*.whl")
        return 1

    data = in_path.read_bytes()
    parsed = cr.parse_pami_bytes(data)

    print(f"in:  {in_path.name}  version={parsed['version']}  "
          f"mesh_paths={len(parsed['mesh_paths'])}")
    for p in parsed["mesh_paths"]:
        print(f"  {p}")

    if old_mesh not in parsed["mesh_paths"]:
        print(f"ERROR: old_mesh {old_mesh!r} not found in file's mesh_paths")
        return 1

    # Mutate via xml_body — the convenience `mesh_paths` field is a
    # read-only derived view, so editing it won't affect serialize().
    new_xml = parsed["xml_body"].replace(old_mesh, new_mesh)
    if new_xml == parsed["xml_body"]:
        print("ERROR: replace was a no-op (old_mesh not in xml_body)")
        return 1
    parsed["xml_body"] = new_xml

    out_bytes = cr.serialize_pami(parsed)
    out_path.write_bytes(out_bytes)

    # Re-parse to verify the edit landed
    verify = cr.parse_pami_bytes(out_bytes)
    print(f"out: {out_path.name}  version={verify['version']}  "
          f"mesh_paths={len(verify['mesh_paths'])}")
    for p in verify["mesh_paths"]:
        marker = " <- new" if p == new_mesh else ""
        print(f"  {p}{marker}")

    if new_mesh not in verify["mesh_paths"]:
        print("WARNING: new_mesh not present after re-parse — edit may have failed")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
