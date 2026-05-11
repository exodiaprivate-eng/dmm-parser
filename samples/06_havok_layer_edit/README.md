# Sample 06 — Havok-Layer File Edit

A worked example showing how a mod author uses the new Tier 1
Havok-layer parsers shipped by the Havok+1.06 repair loop
(iter 3-25, 2026-05-11).

Goal: swap the mesh reference inside a `.pami` (Static Mesh Instance)
file so a placed object in the world points at a different mesh.
Demonstrates the full read → mutate → write flow that all 24 new
Tier 2 parsers support.

## What this teaches

- All 24 new Tier 1 parsers expose `parse_<ext>_bytes(data) -> dict`
  and `serialize_<ext>(dict) -> bytes` via `import dmm_parser`
- Byte-perfect round-trip via the body-opaque field (`xml_body` for
  XML formats, `body_b64` for binaries)
- Named convenience fields (e.g. `mesh_paths`, `version`,
  `record_pairs`) let you decide what to change without re-parsing
  the body yourself
- For the XML family (`.pami`, `.material`, `.technique`, `.mi`,
  `.spline`, `.spline2d`, `.pma`), the simplest edit pattern is
  to text-replace inside `xml_body`

## Files

- `mod.py` — the actual edit script (Python)
- `expected_in/03_cube.pami` (placeholder — not shipped; users supply
  their own extracted .pami file)
- `expected_out/03_cube.pami` (placeholder — written by mod.py)

## How to run

```python
# 1. Extract a .pami from your game install via the diff tool's
#    extraction or directly via dmm_parser.extract_file(...)
#    (out of scope for this sample — see samples/04_custom_item/)
#
# 2. Run the edit script:
python mod.py path/to/03_cube.pami path/to/new_03_cube.pami old_mesh new_mesh
#
# 3. Diff the result:
python -m dmm_parser.diff path/to/03_cube.pami path/to/new_03_cube.pami
```

## Reference

| Format | Parser | Mod-relevant fields |
|---|---|---|
| `.pami` | `parse_pami_bytes` | `xml_body`, `version`, `mesh_paths` |
| `.pab`/`.paa`/`.pam`/`.pabc`/`.pabv`/`.pac`/`.pat`/`.papr` | `parse_par_bytes` | `version_hex`, `ext_classification`, `body_b64` |
| `.motionblending` | `parse_motionblending_bytes` | `field_records[].name/type_tag`, `body_b64` |
| `.pamlod` | `parse_pamlod_bytes` | `lod_count`, `lod_distance`, `texture_paths` |
| `.paasmt` | `parse_paasmt_bytes` | `record_pairs[].model_path / animset_xml_path` |
| `.paccd` | `parse_paccd_bytes` | `format_version`, `no_override_byte_count` |
| `.hkx` | `parse_hkx_bytes` | `sdk_version`, `body_b64` |
| `.material`/`.technique`/`.mi`/`.spline`/`.spline2d`/`.pma` | `parse_xml_bytes` | `xml_body`, `root_element`, `has_bom` |
| `.binarystring` | `parse_binarystring_bytes` | `count`, `strings[]` |
| `.imp` | `parse_imp_bytes` | `reserved`, `body_b64` |
| `.impostor` | `parse_impostor_bytes` | `floats[12]` (3 vec3 + extras) |

See `docs/api.md` "Havok-Layer Formats (Tier 1)" for full JSON
shapes and `docs/MOD_AUTHOR_GUIDE.md` §13 for cross-references.

## Round-trip discipline

All 24 parsers preserve the body verbatim. Named convenience fields
(e.g. `mesh_paths`) are **read-only views derived from the body** —
mutating them in the dict will NOT affect the serialized output;
edit the `xml_body` / `body_b64` directly.

For XML formats the simplest pattern is in-place string substitution
in `xml_body`. For binary formats decode `body_b64` to a `bytearray`,
mutate bytes at known offsets, re-encode and write the dict back.
