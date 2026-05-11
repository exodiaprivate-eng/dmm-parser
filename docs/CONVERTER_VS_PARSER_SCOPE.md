# dmm-parser vs dmm-converter — scope split

**Date:** 2026-05-11
**Why this exists:** the two crates have overlapping vocabularies ("mod",
"intent", "patch") and engineers occasionally reimplement converter logic
inside dmm-parser when they shouldn't. This document is the contract.

---

## dmm-parser owns

Pure binary-format work on **single game files** of a single known type.

| Responsibility | Module |
|---|---|
| pabgb table parsing (122 tables, 16 with tracked field resolution) | `tables/<name>/info.rs` |
| Tracked-field dispatcher (`offset → (record_key, field_name)`) | `src/tracked.rs` |
| JSON round-trip per table (`to_json_dict` / `write_from_json_dict`) | `tables/<name>/info.rs` |
| Polymorphic family decoders (BuffData / ConditionData / BranchConditionData / GameEventHandler / etc.) | `binary/variants/` |
| Non-pabgb format readers (paac, paatt, pamhc, pappt, paseq, pamt, papgt) | `tables/<name>/` |
| Generic CArray / COptional / CString / CBytes primitives | `binary/` |

If you're touching binary bytes of a single game file, it lives here.

## dmm-parser does NOT own

Anything mod-shaped or filesystem-shaped. **Do not reimplement these in
dmm-parser even if it feels convenient:**

| Responsibility | Lives in |
|---|---|
| Classifying a `.json` file's mod shape (v1 / v2 / v3 / browser-manifest / modinfo-only) | `dmm-converter-core::classify` |
| Walking a mod folder + bundling its assets into a v3.1 mod | `dmm-converter-core::browser_to_v3` |
| Detecting standalone-overlay layouts (`<NNNN>/0.paz` + `0.pamt`) | `dmm-converter-core::browser_to_v3::detect_standalone_overlay` |
| Reading `manifest.json` / `modinfo.json` / `mod.json` / `crimson_sharp_mod_v1` formats | `dmm-converter-core::browser_to_v3::read_manifest` |
| Inferring mod metadata from folder names | `dmm-converter-core::browser_to_v3::infer_modinfo_from_dirname` |
| v2-byte-patch → v3.1 intent conversion (whole pipeline) | `dmm-converter-core::v2_to_v3` |
| Hybrid `.pabgb` diff against vanilla → per-record intents | `dmm-converter-core::hybrid_to_v3` |
| v3.0 → v3.1 schema upgrade | `dmm-converter-core::classify::upgrade_v30_to_v31` |
| Numeric-prefix path stripping for vfs paths (`0012/ui/x.css` → `ui/x.css`) | `dmm-converter-core::browser_to_v3::walk_assets` |
| Detecting trailing-comma JSON / sloppy mod metadata + falling back gracefully | `dmm-converter-core::browser_to_v3::read_manifest` |
| File-extension → asset-kind mapping (dds/audio/prefab/xml/...) | `dmm-converter-core::browser_to_v3::classify_kind` |

If a mod author hands you a folder or a `.json` and you have to *figure
out what it is*, that's converter work. The parser only knows how to
read bytes of a known game format once you've handed it both the bytes
and the format name.

## When dmm-parser CLI gets a mod-shaped request

Examples of requests the dmm-parser CLI should refuse / defer:

- "Convert this folder to v3.1"
- "Detect whether this is a browser-mod or a byte-patch mod"
- "Bundle these DDS files into a v3.1 file_replacement set"
- "Strip the `0009/` prefix from this vpath"
- "Read the manifest's `files_dir` and walk it"

The right reply for each is: **"that lives in dmm-converter-core. Call
the appropriate function there."**

The dmm-parser CLI surface should stay narrow:

- `parse <file>` — print parsed JSON for a known table
- `serialize <file.json>` — round-trip JSON → bytes
- `tracked-parse <table> <pabgb> <pabgh>` — dump tracked record list
- `dispatch-info` — list known tables + their tier

Anything else routes to dmm-converter.

## How they talk

```
┌─────────────────────────┐         ┌────────────────────────────┐
│  dmm-converter (CLI/GUI)│         │       dmm-parser           │
│                         │         │                            │
│  classify.rs            │         │  tracked.rs                │
│  browser_to_v3.rs       │ ──────▶ │  parse_table_tracked()     │
│  v2_to_v3.rs            │         │  is_tracked_table()        │
│  hybrid_to_v3.rs        │         │  parse_table_to_json()     │
│                         │         │                            │
│  ◄─── v3.1 JSON output  │         │  per-table info.rs         │
└─────────────────────────┘         └────────────────────────────┘
```

Converter consumes the parser as a library dep (`dmm-parser = { path =
"..." }`). Parser never depends on converter.

## Test fixtures

| Fixture corpus | Location | Owned by |
|---|---|---|
| Vanilla pabgb dumps | `dmm-pabgb-aio/vanilla_dumps/` | parser tests |
| Modded pabgb files | `dmm-pabgb-aio/modded_dumps/` | parser tests + converter hybrid tests |
| Real mod archives | `Crimson Desert/EXtracted/` | converter end-to-end |
| Workbench-emitted v3 mods | `dmm-workbench/output/` | converter classify + upgrade tests |

Converter tests can use the parser's vanilla fixtures (they share a
crate dep already). Parser tests must NOT reach into converter
fixtures.

## Why this matters

Three observed past mistakes worth not repeating:

1. **Reimplementing `read_manifest` in a parser CLI subcommand.**
   Caused subtle drift when the converter added `mod.json` support and
   the parser CLI didn't. Now both go through one path.

2. **Re-deriving `classify_kind` in the apply pipeline.** DMM-BETA
   had its own DDS/audio detection that fell behind the converter's
   asset-kind list (font, template_html, character_pack, etc.). The
   converter emits `kind` per file; downstream uses it verbatim.

3. **Trying to "extend" dmm-parser to read folders.** Tempting because
   the parser already opens pabgb files. Don't. Folders are
   converter territory. Parser sees bytes-of-known-format and that's it.

---

**Maintainer note:** if you find yourself adding mod-shape detection,
classification, or assembly to a function in `dmm-parser`, stop. Put it
in `dmm-converter-core` and have the parser call into it via the CLI
layer if you really need it at parse time. The converter is already a
path-dep of every CLI that uses the parser, so the wiring is free.
