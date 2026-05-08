<!-- SPDX-License-Identifier: LicenseRef-CDMTL-1.0
     Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
     Licensed under CDMTL v1.0 - see LICENSE.txt -->

# Tier 1 Examples Index

Examples shipped during the Tier 1.5 → Tier 1 promotion loop
(Sessions 1-31). Run any of them with:

```sh
cargo run --release --example <name>
```

All require a Crimson Desert install at
`C:\Program Files (x86)\Steam\steamapps\common\Crimson Desert`
(edit the `GAME_DIR` constant at the top of each file to point
elsewhere). All extract files via `dmm-parser`'s typed PAZ reader.

## Cross-format validators

| Example | Purpose | Pass rate |
|---|---|---|
| `tier1_full_roundtrip` | Round-trip every `.pastage`/`.paseq`/`.paseqc`/`.paschedule`/`.paschedulepath`/`.paatt` through both direct (`parse → to_bytes`) and JSON (`parse → to_json_value → write_from_json`) paths. **Canonical Tier 1 regression check.** | **18,952 / 18,952 (100%)** |
| `mod_tooling_e2e` | Walk + replace pattern smoke test on `.pastage`/`.paseq`/`.paseqc` (50 samples each): find first LP-string, replace with longer value, re-parse, verify. | **150 / 150 (100%)** |
| `json_path_mod_e2e` | JSON-path edit pattern smoke test on `.paschedule`/`.paatt` (30 samples each): edit named field, re-parse, verify edit stuck. | **45 / 45 (100%)** |
| `generic_string_walker` | Validate `walk_u32_prefixed_strings` across all 6 Tier 1 formats. | informational |

## Per-format round-trip validators

| Example | Format | Validates |
|---|---|---|
| `pastage_roundtrip` | `.pastage` | `TypedPastageFile` + `PastageFileSafe` round-trip across all 3,320 vanilla samples |
| `paseq_roundtrip` | `.paseq` | `TypedPaseqFile` + `PaseqFileSafe` round-trip (4,659 samples) |
| `paseqc_roundtrip` | `.paseqc` | `TypedPaseqcFile` + `PaseqcFileSafe` round-trip (2,932 samples) |
| `paschedule_roundtrip` | `.paschedule` | `TypedPascheduleFile` round-trip (4,084 samples) |
| `paschedulepath_roundtrip` | `.paschedulepath` | `TypedPaschedulePathFile` round-trip (3,737 samples) |
| `paatt_roundtrip` | `.paatt` | `PaattFile::to_bytes` + JSON round-trip (220 files / 13,789 AttackInfos) |

## Sample extractors (pull files from PAZ for hex inspection)

| Example | Output |
|---|---|
| `pastage_extract_one` | 3 specific `.pastage` files extracted to `target/pastage_samples/` with hex companions |
| `paseq_extract_one` | 5 `.paseq` samples to `target/paseq_samples/` |
| `paseqc_extract_one` | 5 `.paseqc` samples to `target/paseqc_samples/` |
| `paschedule_extract_one` | 5 `.paschedule` samples |
| `paschedulepath_extract_one` | 5 `.paschedulepath` samples |
| `paatt_to_json` | Single `.paatt` → JSON for analysis |

## Schema enumeration (`.paseq` / `.paseqc` only)

| Example | Outputs |
|---|---|
| `paseq_field_directory` | Distinct `(field_name, type_name)` pairs across all samples (validated 16 + 19 distinct on 4,659 + 2,932 files) |
| `paseq_full_schema` | All class blocks (outer + nested linear) — 272 distinct `.paseq` classes, 62 distinct `.paseqc` classes |
| `paseq_value_section_stats` | Where the schema/value boundary lives per file (avg 39.5 KB values for `.paseq`, 6.5 KB for `.paseqc`) |
| `paseq_value_strings` | Top embedded value strings across all samples (script expressions, trigger names, asset paths) |

## Edit primitives smoke tests

| Example | Demonstrates |
|---|---|
| `paseq_string_replace_smoke` | End-to-end: parse `cd_seq_ui_empty.paseq`, find `_sequencerName`, replace with longer string, re-parse |

## `.paatt` BaseData analysis

| Example | Outputs |
|---|---|
| `paatt_basedata_entropy` | Per-byte entropy classification for v0/v1/v2/v3 BaseData (10,562 + 1,674 + 851 + 702 = 13,789 records). Identifies always-zero, always-const, bool, low-cardinality enum, and high-entropy positions. Output: `target/paatt_basedata_entropy.txt` (1,396 lines). |

## See also

- `docs/TIER1_PROMOTION_PROGRESS.md` — engineering session log (1-31)
- `docs/MOD_AUTHOR_GUIDE.md` §12 — mod-author reference for Tier 1 formats
- `docs/api.md` — Python API surface for the 36 Tier 1 PyO3 functions
- `docs/PAATT_BASEDATA_FIELDS.md` — `.paatt` field directory deep dive
