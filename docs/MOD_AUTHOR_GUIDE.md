# Crimson Desert Mod-Author Guide

> **Audience.** People writing mods for Crimson Desert using the
> Field-JSON v3 / v3.1 manifest format. Goal: ship a mod that SWISS
> Stacker can load + verify, and that other mods can compose with.
>
> This is the user-facing entry point. For the binary formats themselves
> see `docs/FORMATS.md`. For the Python API surface see `docs/api.md`.

---

## Contents

- [0. The big picture](#0-the-big-picture)
- [1. Picking a mod type](#1-picking-a-mod-type)
- [2. Data mods (table targets)](#2-data-mods-table-targets)
- [3. Texture mods (DDS asset targets)](#3-texture-mods-dds-asset-targets)
- [4. Audio mods (WEM / BNK asset targets)](#4-audio-mods-wem--bnk-asset-targets)
- [5. Paloc mods (localization)](#5-paloc-mods-localization)
- [6. Custom items](#6-custom-items)
- [7. Mixed mods](#7-mixed-mods)
- [8. The author workflow](#8-the-author-workflow)
- [9. Distribution](#9-distribution)
- [10. Common pitfalls](#10-common-pitfalls)
- [11. Where to look next](#11-where-to-look-next)

**Companion docs:**

- [`docs/FORMATS.md`](FORMATS.md) — every binary format reference in one place
- [`docs/api.md`](api.md) — Python API surface (classify_dds / parse_bnk / paloc / save / etc.)
- [`docs/CUSTOM_ITEM_CREATOR_V3_1.md`](CUSTOM_ITEM_CREATOR_V3_1.md) — end-to-end custom-item walkthrough
- [`samples/`](../samples/) — five runnable example mods (data, texture, audio, custom item, mixed)

---

## 0. The big picture

A Crimson Desert mod is a single **manifest** (`*.field.json`) that
describes a set of **targets** — the things the mod changes. Each
target is one of:

| Kind   | Touches                                                            |
|--------|--------------------------------------------------------------------|
| `table` | Game data (items, skills, NPCs, …) parsed from `pabgb` archives  |
| `asset` | Binary asset files: `.dds` textures, `.wem`/`.bnk` audio         |
| `paloc` | Localization strings (`*.paloc`)                                 |

A v3.1 manifest is just JSON with a list of targets:

```jsonc
{
  "format": 3,
  "format_minor": 1,
  "name": "Macduff Voice Pack",
  "author": "potter",
  "version": "1.0.0",

  "targets": [
    { "kind": "asset", "asset_type": "wem",
      "file": "voice/macduff_attack01.wem",
      "vpath": "0006/sound/windows/english(us)/3684722581.wem" },

    { "kind": "table", "table": "ItemInfo",
      "ops": [ { "key": "id_4242", "field": "max_count", "value": 99 } ] }
  ]
}
```

When SWISS Stacker loads this manifest it:

1. **Validates** every target (`dmm-mod-validate`)
2. **Diffs** it against every other enabled mod (`dmm-mod-diff`)
3. Compiles the deltas into the runtime archive overlay

---

## 1. Picking a mod type

| You want to…                       | Mod type      | Section |
|------------------------------------|---------------|---------|
| Change item / skill / NPC numbers  | Data mod      | §2      |
| Replace an in-game texture         | Texture mod   | §3      |
| Replace voice / SFX                | Audio mod     | §4      |
| Translate / change UI strings      | Paloc mod     | §5      |
| Add a brand-new item               | Custom item   | §6      |
| Combine multiple of the above      | Mixed mod     | §7      |

Most useful mods are **mixed** — a custom-item mod usually ships a DDS
icon, a few paloc strings, and a row of `ItemInfo` data.

---

## 2. Data mods (table targets)

A table target is a list of operations against a parsed game table. The
authoritative table list is whatever `dmm_parser.is_supported_table`
returns — currently includes `ItemInfo`, `SkillInfo`, `BuffInfo`,
`CharacterInfo`, `StageInfo`, `GimmickInfo`, plus the localization
catalogs.

```jsonc
{
  "kind": "table",
  "table": "ItemInfo",
  "ops": [
    { "key": "id_4242", "field": "max_count",   "value": 99 },
    { "key": "id_4242", "field": "buy_price",   "value": 0  },
    { "key": "id_4242", "field": "description", "value": "A trusty blade." }
  ]
}
```

**What each op needs:**
- `key` — the stable row identifier the parser exposes. For
  ItemInfo this is the `id_<int>` form returned by
  `dmm_parser.parse_iteminfo_from_file`.
- `field` — the field name, exactly as it appears in the parsed JSON.
- `value` — the new value. Strings get written as PAMT BStrings, ints
  go straight through, optional fields can be set to `null` to clear.

**Pitfalls:**
- Whitespace inside string values is preserved verbatim — including
  trailing newlines. Strip them.
- Numeric overflow is not caught at parse time. Read the field's
  underlying type from the schema before writing huge values.

---

## 3. Texture mods (DDS asset targets)

> **For mod authors:** the dedicated guide
> [`TEXTURE_MOD_AUTHORING.md`](TEXTURE_MOD_AUTHORING.md) covers everything
> — five-minute quickstart, DDS format chart, vpath cheatsheet, DCC tool
> exports (Photoshop / GIMP / Substance Painter / `nvtt_export`),
> validation, packing, NexusMods page templates, recipes, and
> troubleshooting. The one-page
> [`TEXTURE_VPATH_CHEATSHEET.md`](TEXTURE_VPATH_CHEATSHEET.md) is the
> printable reference card. The summary below is the minimum to know
> textures exist as a v3.1 target type.

A DDS asset target replaces a texture file in the game's archive
overlay. The mounted vpath must match the in-game path exactly,
**including the Crimson "last4" overlay class** (see `docs/api.md`
→ DDS Textures).

```jsonc
{
  "kind": "asset",
  "asset_type": "dds",
  "file": "textures/sword_diffuse.dds",
  "vpath": "/character/texture/macduff/sword_diffuse.dds",
  "sha256": "...auto-filled by dmm-mod-pack..."
}
```

**The 60-second version of the rules:**
- Color textures (UI / armor / tattoo) → **BC7 SRGB** with mips
- Normal maps (`*_n.dds`) → **BC5 Linear** with mips
- vpath path-prefix decides the last4 (`/ui/*` → `0x1580`,
  `/character/texture/*_n.dds` → `0x0480`, etc.)
- `dmm-mod-pack` auto-fills sha256 + size + last4
- `dmm-mod-validate` catches malformed DDS before users see it

**Build pipeline:**
1. Author your texture in your DCC tool.
2. Export to DDS with the right format per
   [TEXTURE_VPATH_CHEATSHEET.md](TEXTURE_VPATH_CHEATSHEET.md).
3. Run `python -m dmm_parser.tools.validate <manifest>` to catch
   format / vpath issues.
4. Run `python -m dmm_parser.tools.pack <manifest>` to produce the
   distributable zip.

---

## 4. Audio mods (WEM / BNK asset targets)

Audio replacement uses Wwise's WEM (per-clip) and BNK (soundbank)
formats. The Crimson conventions are documented in
`references/wwise_notes.md`.

```jsonc
{
  "kind": "asset",
  "asset_type": "wem",
  "file": "voice/macduff_attack01.wem",
  "vpath": "0006/sound/windows/english(us)/3684722581.wem"
}
```

**Rules:**
- WEM file size must match what the original soundbank's DIDX entry
  expects, OR you must ship a replacement BNK with the new DIDX
  offsets. SWISS warns if the DIDX of the parent BNK and the new
  WEM size disagree.
- Voice clips for English go under
  `0006/sound/windows/english(us)/<id>.wem`. Other languages use
  the language tag in place of `english(us)` (e.g. `korean`,
  `japanese`).
- Common SFX use `soundcommon/windows/<id>.wem` / `.bnk`.
- `dmm_parser.infer_audio_vpath(path)` returns the Crimson class
  (`LocalizedVoiceClip`, `CommonSoundBank`, etc.) which SWISS uses
  to pick the right archive group.

**Building WEMs.** Use Wwise authoring (free for non-commercial) to
encode and export. The `validate_audio` checker covers format-tag,
sample-rate sanity, channel counts, and basic envelope structure;
real perceptual quality is on you.

---

## 5. Paloc mods (localization)

```jsonc
{
  "kind": "paloc",
  "language": "english(us)",
  "entries": [
    { "category": 0, "key": "STR_ITEM_4242_NAME", "value": "Trusty Sword" },
    { "category": 0, "key": "STR_ITEM_4242_DESC", "value": "Reliable, sharp." }
  ]
}
```

The `category` field matches the upstream paloc `u64` category. For
typical UI text it's `0`. Use `dmm_parser.parse_paloc_from_file` on a
vanilla file to discover the category convention for your strings.

---

## 6. Custom items

A "custom item" is a mod that **adds** a new row to `ItemInfo` rather
than editing an existing one. The right pattern:

1. Pick a free `id_<int>` (anything ≥ 100000 to avoid colliding with
   future game patches).
2. One **table** target inserting the row.
3. One **asset** target with the icon DDS.
4. One **paloc** target with the name + description strings.

See `docs/CUSTOM_ITEM_CREATOR_V3_1.md` for the full worked example
including the equipment / sockets fields.

---

## 7. Mixed mods

Just put multiple targets in `targets: [...]` — order doesn't matter,
SWISS sorts them by kind during apply. Hash + size on each asset
target lets the validator catch corruption before the mod hits the
runtime overlay.

```jsonc
{
  "format": 3, "format_minor": 1,
  "name": "Sword of Potter",
  "author": "potter",
  "version": "1.0.0",
  "targets": [
    { "kind": "table", "table": "ItemInfo",
      "ops": [{"key": "id_100001", "field": "name", "value": "Sword of Potter"}] },
    { "kind": "asset", "asset_type": "dds",
      "file": "icons/sword.dds", "vpath": "/ui/icon/sword.dds" },
    { "kind": "paloc",
      "entries": [{"category": 0, "key": "STR_ITEM_100001_DESC",
                   "value": "Forged in the loop."}] }
  ]
}
```

---

## 8. The author workflow

```
my_mod/
├── my_mod.field.json
├── icons/
│   └── sword.dds
└── voice/
    └── macduff_attack01.wem
```

```sh
# 1. Validate while iterating
python -m dmm_parser.tools.validate my_mod/my_mod.field.json --assets my_mod/

# 2. Inspect what your manifest will do
python -m dmm_parser.tools.inspect my_mod/my_mod.field.json

# 3. Pack for distribution (auto-fills sha256 + size + vpath)
python -m dmm_parser.tools.pack my_mod/my_mod.field.json --out my_mod-1.0.0.zip

# 4. Diff against another mod to check for conflicts
python -m dmm_parser.tools.diff my_mod-1.0.0.zip other_mod.zip
```

Exit codes: 0 = no fatals/errors; 1 = at least one finding requires
attention. SWISS Stacker uses these exit codes during its enable/disable
flow.

---

## 9. Distribution

- **NexusMods**: ship the `.zip` produced by `dmm-mod-pack`. SWISS
  Stacker reads `.zip` directly. Tag with the `crimson-desert-modding`
  category and link to the `dmm-parser` README so users know where the
  manifest format comes from.
- **CDMTL v1.0** (this repo's license): the parser tooling itself is
  source-available with a no-competing-implementation clause; mod
  manifests you author with it are yours to license however you like.

---

## 10. Common pitfalls

- **Forgot to update SHA-256.** Symptom: validator complains
  `asset_sha_mismatch` after you tweak a DDS. Fix: re-pack — `dmm-mod-pack`
  recomputes the hash if you didn't pre-set one.
- **Path case mismatch.** Crimson is case-insensitive at lookup but
  the SWISS Stacker overlay map is case-sensitive. Always use lowercase
  vpaths.
- **Missing `dwReserved2[3]` in DDS.** Symptom: texture loads but
  appears as a blank/pink fallback in-game. Fix: re-export keeping the
  Crimson reserved fields, or fix in Hex with `classify_dds` as guide.
- **WEM sample-rate outside 8k–96k.** Triggers the
  `wem_unusual_sample_rate` warning. Crimson will technically play it,
  but the resampler's pitch envelope was tuned around 44.1k/48k.
- **Custom item ID under 100k.** Will eventually collide with a game
  patch. Pick high.

---

## 11. Where to look next

- `docs/FORMATS.md` — binary format reference (PAPGT, PAMT, PAZ, paloc,
  DDS, WEM, BNK, save).
- `docs/api.md` — full Python API.
- `docs/CUSTOM_ITEM_CREATOR_V3_1.md` — end-to-end custom-item example.
- `references/*.hexpat` — ImHex pattern files for binary exploration.
- `samples/` — runnable example mods (each its own README).
