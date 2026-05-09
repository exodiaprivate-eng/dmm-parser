# pycrimson Reflection Verification Workflow

Step (c) of the v3.1 verification loop. Sets up
[LukeFZ/pycrimson](https://github.com/LukeFZ/pycrimson) as a second
canonical-name source alongside NattKh's `pabgb_complete_schema.json`.

NattKh covers `.pabgb` tables (122 of dmm-parser's tables). pycrimson
covers Pearl Abyss's reflection-format files which are
**self-describing** — every parsed value carries its canonical class name
and field names directly.

## Reflection-format files (per pycrimson `file_format_notes.md`)

These extensions go through the reflection serializer. Their parsed
output exposes canonical PA class+field names directly:

- `.prefab`, `.meshinfo`, `.pae`, `.paem`, `.parg`, `.pasg`,
  `.paa_metabin`, `.palevel`, `.paseqc`
- `.paseq`, `.uianiminit` (custom header wrapping a reflection block)

## NOT reflection-format (despite earlier assumption)

- `.paatt` — pycrimson has no parser for this. AttackInfoDataDesc /
  AttackCommonData / AttackHitData fields cannot be recovered through
  pycrimson. They need either:
  - the NattKh Korean-error-grep approach extended to descriptor classes, or
  - direct IDA work on the parse function (blocked: see `T0_AUDIT_TRACKING.md`).

## Setup

```bash
# Clone (PyPI 'pycrimson' is an unrelated package — must clone from GitHub)
git clone --depth 1 https://github.com/LukeFZ/pycrimson \
    "_research_cache/pycrimson"
pip install -e "_research_cache/pycrimson"   # requires Python 3.14
```

## Pipeline

```bash
# 1. Extract reflection-format files from the game's .paz packs.
#    --only-extension picks one type at a time. .prefab / .meshinfo /
#    .palevel are the highest-yield categories.
python _run_main.py extract-pack-files \
    --pack-path  "<Crimson Desert install dir>" \
    --output-path "_research_cache/extracted_reflection" \
    --only-extension prefab

# 2. Parse one file at a time. Output is JSON with `__pycr_type__`
#    markers on every reflection object.
python _run_main.py parse-serialized-file \
    --serialized-path "<extracted_reflection>/path/to/file.prefab" \
    --output-path     "_research_cache/parsed_reflection/file.json"

# 3. Harvest canonical class+field catalog from parsed output.
python scripts/harvest_reflection_schema.py \
    _research_cache/parsed_reflection
# → writes docs/v3_1_reflection_schema.json
```

## Initial demo harvest

502 parsed `.prefab` samples (mostly character armor pieces — homogeneous)
yielded 4 classes / 10 fields:

| Class | Fields |
|---|---|
| `SceneObject` | `_childSceneObjects`, `_components` |
| `SkinnedMeshComponent` | `_boneOffsetTag`, `_modelPropertyIndex`, `_shrinkMaskDistance`, `_shrinkTag`, `_skeletonFileName`, `_skinnedMeshFile`, `_socketFileName` |
| `ResourceReferencePath_SkinnedMesh` | `_path` |
| `ResourceReferencePath_CharacterSkeleton` | (no fields — pointer-only) |

This is intentionally a thin slice. Diverse coverage requires sampling
across formats (`.palevel`, `.pae`, `.paseq`, `.parg`) AND across asset
categories (gimmicks, NPCs, effects, environments, UI). Full-corpus runs
are out of scope for the verification loop — workflow is in place when
needed.

## Why this matters for v3.1

NattKh's schema verifies `.pabgb` table fields. pycrimson verifies
*everything else* the engine touches via reflection — components on
prefabs, mesh metadata, animation graphs, level descriptors. Many of
those are referenced from inside `.pabgb` tables (e.g. CharacterInfo
references prefab paths and skeleton paths whose own schemas live in
the reflection world).

For verifying canonical names of classes we don't already cover, parse a
representative file of the right format and look up the relevant class
in the harvested catalog.

## Limitations

- `.paatt` not supported (see above). AttackInfoDataDesc verification
  remains blocked on this path.
- Full game-corpus parsing is slow (extract step alone produces 220
  `.paatt` and ~46k `.prefab` files; full multi-format extract would be
  hundreds of GB).
- pycrimson requires Python 3.14 specifically.

## Files

- Generator: `scripts/harvest_reflection_schema.py`
- Output: `docs/v3_1_reflection_schema.json` (regenerate by re-running
  the harvester against any directory of pycrimson-parsed JSON)
