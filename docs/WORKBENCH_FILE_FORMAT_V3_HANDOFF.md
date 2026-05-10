# Workbench → DMM Field-JSON v3 Handoff: File-Format Tables

**Audience:** NattKh / mod-workbench maintainer
**Author:** RicePaddySoftware (DMM)
**Status:** dmm-parser Phase 1+2 + DMM Phase 3 complete (pre-release.7).
Workbench-side v3-intent export is the missing piece — that's the ask
in this doc.
**Date:** 2026-05-09

---

## What just landed in DMM (pre-release.7)

DMM now applies field-JSON-v3 intents end-to-end against the four file-format tables Workbench currently exports as PAZ-overlay folders:

- `paac` (action chart — `commonactioninfo.paac`, `*_upper.paac`)
- `paatt` (projectile attribute — `actionchart/projectileinfo*.paatt`)
- `pamhc` (`miscellaneous/modelpropertyheadercollection.pamhc`)
- `pappt` (`character/bin__/partprefabtable.pappt`)

The pipeline:

1. Mod manifest declares a target whose key is the file's name
   (`partprefabtable.pappt`) or vfs path
   (`character/bin__/partprefabtable.pappt`).
2. DMM detects the file-format extension, finds the live file in any
   vanilla group via `find_file_in_game`, extracts via `extract_from_paz`.
3. Bytes flow through `dmm_parser::dispatch::parse_table_to_json` →
   intent apply → `serialize_table_from_json`.
4. Modified bytes pack into a fresh overlay group:
   - `dmmv3_pappt` / `dmmv3_pamhc` (singletons)
   - `dmmv3_paac_<stem>` / `dmmv3_paatt_<stem>` (per-file)
5. Group registers in PAPGT, `.dmm_owned` marker dropped, mount log
   carries `[V3_FILE]` lines for the apply.
6. Unmount cleans up automatically (`is_dmm_owned_group` already
   recognises `dmmv3_*` prefixes).

Library code paths (all public):

- `dmm_parser::dispatch::parse_table_to_json("pappt", bytes, None)` →
  `Vec<Value>`
- `dmm_parser::dispatch::serialize_table_from_json("pappt", &items)` →
  `Vec<u8>`
- `dmm_parser::dispatch::is_file_format_table("pappt")` → `bool`
- DMM-internal: `apply_v3_to_file_format_body`,
  `install_v3_file_format_overlay`

---

## JSON shape per format

Each file-format table parses to a 1-element `Vec<Value>` where the
single value carries the entire file shape plus synthetic
`key: 0` / `string_key: ""` so the v3 intent dispatcher's
`find_record_index` can resolve it.

### pappt

```json
{
  "key": 0,
  "string_key": "",
  "header": [222, 173, 190, 239, 0, 1, 2, 3],
  "primary": [
    {
      "key_a": "Kliff",
      "key_b": "hair",
      "key_c": "src/kliff_hair.pmod",
      "asset_id": "kliff_hair_default",
      "flag": 1,
      "children": [
        { "sub_key": "kliff_hair_long", "sub_flag": 2 }
      ]
    }
  ],
  "secondary": [
    { "alias_a": "old_kliff_hair", "alias_b": "kliff_hair_default" }
  ]
}
```

Field paths v3 intents address:
- `primary[N].key_a` / `key_b` / `key_c` / `asset_id` / `flag`
- `primary[N].children[M].sub_key` / `sub_flag`
- `secondary[N].alias_a` / `alias_b`
- `header[N]` (one byte of the 8-byte opaque header)

### pamhc

```json
{
  "key": 0,
  "string_key": "",
  "header": [202, 254, 186, 190, 1, 2, 3, 4],
  "section_a": [16909060, 84281096, 151587081],
  "section_b": [176, 177, 178],
  "section_c": [],
  "section_d": [],
  "section_e": [224, 225, 226]
}
```

Field paths:
- `header[N]`
- `section_a[N]` (u32 entries)
- `section_b[N]` / `section_c[N]` / `section_d[N]` / `section_e[N]`
  (opaque bytes — element schemas not decoded)

### paatt

```json
{
  "key": 0,
  "string_key": "",
  "entry_count": 209,
  "hash_marker": 1160449792,
  "body": [...]
}
```

Field paths:
- `entry_count` / `hash_marker`
- `body[N]` (raw byte at offset N — physics fields like
  `projectileRadius` are anchor-detected; Workbench computes the
  byte offset and emits a `body[OFFSET]` intent)

### paac

```json
{
  "key": 0,
  "string_key": "",
  "format": "action_chart_v1",
  "size": 12345,
  "header_node_count": 703,
  "header_speed": 1.3333,
  "state_count": 35,
  "transition_count": 371,
  "condition_record_count": 50,
  "raw": [...]
}
```

`format` / `size` / `header_*` / `*_count` are read-only derived
views. The writable field is `raw[N]` — Workbench computes byte
offsets from its parsed view and emits `raw[OFFSET]` intents.

---

## Sample v3 intent file

`examples/v3_file_format_samples/sample_pappt.field.json`:

```json
{
  "format": 3,
  "modinfo": {
    "title": "Sample pappt mod (test fixture)",
    "version": "1.0.0",
    "author": "DMM Phase 3 test",
    "description": "Edits primary[0].flag in partprefabtable.pappt",
    "note": ""
  },
  "targets": [
    {
      "file": "partprefabtable.pappt",
      "intents": [
        {
          "entry": "",
          "key": 0,
          "field": "primary[0].flag",
          "op": "set",
          "new": 1
        }
      ]
    }
  ]
}
```

`entry` and `key` are the canonical synthetic-record identifiers that
the JSON layer emits — file-format records don't have real string keys
or numeric keys, so v3 intents target the implicit single record by
key=0.

The `file` field accepts either:
- File name only: `"partprefabtable.pappt"` — DMM walks every vanilla
  group PAMT to locate it.
- Full vfs path: `"character/bin__/partprefabtable.pappt"` —
  unambiguous, faster lookup.

For paac/paatt where multiple files share the table (`fist_upper.paac`
vs `pistol_upper.paac` vs `sword_upper.paac`), the file name is what
disambiguates and what determines the overlay group stem
(`dmmv3_paac_fist_upper`).

---

## The ask: Workbench v3-intent export

Mod-workbench currently exports file-format mods via dedicated PAZ-overlay deploy paths (`paac_editor::deploy_paac_overlay`, etc.). Each
deploy writes directly to a numbered PAZ overlay group on disk
(`0066/`, `0067/`, ...) — bypassing any user-visible mod folder.

**Request:** add a parallel "Export as Field-JSON v3" button to each
of the four editors that emits the JSON shape above. Concretely, in
`mod-workbench/src/mod_io.rs::export_dmm_v3` (or a sibling function):

1. After the user finishes editing, diff the modified PaacFile /
   PaattFile / PamhcFile / PapptFile against the original.
2. For each scalar leaf that differs, emit one `{ entry: "", key: 0,
   field: <path>, op: "set", new: <value> }` intent.
3. Write the standard v3 envelope (`format: 3`, `modinfo`, `targets`)
   with one target whose `file` is the file's basename.

Field-path generation can use the same `flatten_leaves` helper that
already powers pabgb v3 export — the JSON shapes round-trip the same
way, so dot/bracket notation works unchanged.

For paac specifically, the simplest first-cut export is "diff `raw`
byte-by-byte and emit `field: "raw[N]"` for every changed offset" —
that mirrors how `patch_float` / `patch_transition` already mutate
the parser. Per-state-machine semantic exports can come later once
the JSON shape design is settled on both sides.

---

## Verification

DMM-side verification per phase (all green as of pre-release.7):

| Format | dmm-parser tests | DMM build |
|--------|------------------|-----------|
| pappt  | 7/7 pass         | clean     |
| pamhc  | 8/8 pass         | clean     |
| paatt  | 6/6 pass         | clean     |
| paac   | 9/9 pass         | clean     |

Mount-time end-to-end verification still requires:
- A live game install with one of these files in vanilla state
- A v3 mod targeting that file dropped into DMM's mods directory
- `mount_log.txt` after mount contains `[V3_FILE] <target> → group dmmv3_<table>` line

Sample mod at `examples/v3_file_format_samples/sample_pappt.field.json`
is the easiest one to test — the edit is benign (flag byte change with
no observable in-game effect) and exercises the full dispatch path.

---

## Contact

DMM bugs, JSON shape questions, or coordination on the export format:
exodiaprivate@gmail.com / `exodiaprivate-eng/DMM-BETA` issues.

dmm-parser bugs or schema questions:
`exodiaprivate-eng/dmm-parser` issues.

Both repos are under CDMTL v1.0; mod-workbench is part of the
Authorized Software Suite under §1(g) so straight ports of these
shapes back into Workbench are explicitly allowed.
