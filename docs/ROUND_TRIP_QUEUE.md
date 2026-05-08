# Round-Trip Queue — STOP CONDITION REACHED

The loop's primary goal — "all parsable tables field-level parsed" — is met.

## Decisive insight (from `docs/449_TABLE_CATALOG.md`)

The 449 `pa::*Info` C++ classes in the binary symbol dump break down:

| Status | Count | What it means                                      |
|---|---:|---|
| ✅ T1 on-disk fully field-decoded | 118 | typed `.pabgb` parsers shipped     |
| 📚 parser exists, no .pabgb in dump | 2  | covered, just no current sample    |
| 🧠 in-memory only, never serialized  | 327 | runtime C++ structs; no .pabgb   |

**The "327 untyped tables" I'd been quoting were never on disk.** They're
runtime helper structs the game uses while loading; the engine never
writes them to a `.pabgb`, so there's nothing to parse. All on-disk
pabgb tables (118 + 2 stale = 120) are field-level typed.

## Final round-trip totals

**1,276,963 vanilla files at 100% round-trip across ~60 file extensions.**

### Tier 1 — typed (full named-field JSON)

| Family / Ext     |   Files |
|------------------|--------:|
| pabgb (118 tables) |     122 |
| pabgh / paloc / paatt | 356 |
| dds              | 280,826 |
| wem              | 238,242 |
| bnk              |   3,157 |
| **Tier 1 total** | **522,703** |

### Tier 1.5 — `lp_token_stream` (round-trip + addressable strings)

| Family             |   Files |
|--------------------|--------:|
| Sequencer (5 ext)  |  18,732 |
| Proprietary (17)   | 149,065 |
| Mesh / animation (8) | 524,849 |
| Long-tail (25)     |   4,126 |
| HKX                |  57,268 |
| MP4                |     695 |
| **Tier 1.5 total** | **754,735** |

## Live-game audit (2026-05-05)

`examples/round_trip_matrix.rs` against live 1.05.02 install:

```
PASS clean round-trip:  121
PARSE failed:           0
SERIALIZE failed:       0
Byte mismatch:          0
Missing from PAZ:       1   (equip_info — removed in this game version)
```

Specifically confirmed parsed correctly:

- **inventory_info** (Tier 1) — `_inventoryMoveDataList` polymorphic
  via `OptionalGameCondition` + per-element `InventoryMoveData`. Round-
  trip byte-perfect on 8 entries.
- **drop_set_info** (Tier 1) — `_dropInfoData` polymorphic via
  `OptionalDropTarget` (`crate::binary::variants::drop_target`).
  Round-trip byte-perfect on 1004 entries.

Both were previously flagged as "deferred polymorphic blockers" in the
task list (#100, #102) but are in fact fully Tier-1 typed and round-
trip clean. Task list was stale — this doc is now the source of truth.

## What's still open (for future, not loop-blocking)

These are nice-to-have polish items, not part of the "all parsable
tables" goal. The loop deliberately stops short of starting them:

### Polymorphic family Tier 1.5 → Tier 1 polish (open tasks)
- #92 / #93 / #94 / #107 / #113 — GameCondition family variant
  byte-recipe extraction (already partially done; remaining variants
  ship as `Decoded | Raw` enum fallback per task #106's pattern).
  Note: 0.2% Raw fallback still round-trips byte-perfect; this only
  affects field-level decoding of those specific variants.
- #95 — TriggerEventHandler family decoder (file exists at
  `binary/variants/trigger_gameplay_event_handler_data.rs`, 667 lines,
  all 8 dispatch cases mapped per Win-IDA — round-trip clean).

### Tier 1.5 → Tier 1 promotions (multi-week per format)
- prefab (34k files, ReflectObject-based with variable headers + LZ4)
- pami (33k files, material parameter bag)
- pae (5.7k files, effect emitter graph)
- paac / pabc (1k files, action chart / behavior chart)
- pac / pam (60k files, mesh family — biggest single project)

### Texture-streaming polish
- Decode partial-DDS mip streaming index (217k files have some mips
  inline + some streamed; full mip inventory currently opaque)
- Build a mip-merge tool (combine inline + streamed into a single
  standalone DDS for export)

## How to resume

The loop is paused, not abandoned. To restart on a specific direction:

- "Resume polymorphic family Tier 1 polish" — tackles open tasks
  #92/93/94/95/107/113 in the IDA + recipe-extraction workflow.
- "Promote prefab to Tier 1" — opens the multi-week ReflectObject
  decoder project.
- "Add new pabgb table" — only if a new Crimson Desert build ships
  a new on-disk table type beyond the current 118.

## Loop workflow (preserved for resumption)

1. Pick next entry from priority section.
2. Survey samples (`examples/survey_<ext>.rs`).
3. Build typed parser in `src/binary/<ext>.rs`.
4. Bulk round-trip via `examples/round_trip_<ext>.rs`. 100% required.
5. Update this doc.
