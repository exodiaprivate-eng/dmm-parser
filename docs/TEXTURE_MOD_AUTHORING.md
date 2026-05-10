<!-- SPDX-License-Identifier: LicenseRef-CDMTL-1.0
     Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
     Licensed under CDMTL v1.0 - see LICENSE.txt -->

# Texture Mod Authoring Guide — Crimson Desert (Field-JSON v3.1)

A complete guide for authoring DDS texture mods that ship as Field-JSON
v3.1 packages, validate clean, and install on every player's machine
without manual fixup.

**Prerequisites:** A copy of Crimson Desert, a DCC tool (Photoshop with
the NVIDIA Texture Tools plugin, GIMP with DDS plugin, Substance
Painter, or any tool that can export DDS / BC7 / BC5), and DMM 1.3.4+.
No coding experience needed.

---

## Contents

- [Five-minute quickstart](#five-minute-quickstart)
- [Anatomy of a texture mod](#anatomy-of-a-texture-mod)
- [Picking the right DDS format](#picking-the-right-dds-format)
- [Where do game textures live? (vpath cheatsheet)](#where-do-game-textures-live-vpath-cheatsheet)
- [DCC tool exports](#dcc-tool-exports)
- [Validation and packing](#validation-and-packing)
- [Releasing on NexusMods](#releasing-on-nexusmods)
- [Recipes](#recipes)
- [Troubleshooting](#troubleshooting)

---

## Five-minute quickstart

The simplest texture mod in the world: replace one icon, one file, one
manifest.

```
my_first_mod/
├── mod.field.json
└── assets/
    └── ui/
        └── icon/
            └── my_icon.dds
```

**`mod.field.json`:**

```json
{
  "format": 3,
  "format_minor": 1,
  "modinfo": {
    "title": "My First Texture Mod",
    "author": "Your Name",
    "version": "1.0",
    "description": "A custom UI icon."
  },
  "targets": [
    {
      "kind": "asset",
      "asset_type": "dds",
      "file": "ui/icon/my_icon.dds",
      "vpath": "/ui/icon/my_icon.dds"
    }
  ]
}
```

**Validate and pack:**

```sh
python -m dmm_parser.tools.validate my_first_mod/mod.field.json --assets my_first_mod/assets
python -m dmm_parser.tools.pack     my_first_mod/mod.field.json --assets my_first_mod/assets
```

`pack` produces `my_first_mod.zip` ready to upload to NexusMods or drop
into DMM's mod folder. `validate` flags problems before users ever see
them.

That's it. Everything below is helpful detail for when you want more
than one icon or your texture isn't loading right.

---

## Anatomy of a texture mod

Five rules that aren't obvious from a single example. Internalize these
once and the rest of the doc makes more sense.

### 1. The folder layout

Source folder (what you author) and the packed `.zip` (what `dmm-mod-pack`
produces) have **the same shape**:

```
my_mod/                       ← the .zip is just this folder, zipped
├── mod.field.json            ← manifest at the root, REQUIRED
└── assets/                   ← every file referenced by the manifest
    ├── ui/
    │   └── icon/
    │       └── sword.dds
    └── character/
        └── texture/
            ├── macduff_diffuse.dds
            └── macduff_diffuse_n.dds
```

So the same `dmm-mod-pack` command both packs the zip *and* validates
the same source layout. If your source builds cleanly, your zip works.

### 2. The `file` field is relative to `assets/`

Inside `mod.field.json`, every target's `file` field is a path
**relative to the `assets/` folder**, not relative to the manifest or
the project root. Forward slashes only.

```jsonc
"file": "ui/icon/sword.dds"          // ✅ correct — assets/ui/icon/sword.dds
"file": "/assets/ui/icon/sword.dds"  // ❌ wrong — leading slash + redundant assets/
"file": "C:/Users/.../sword.dds"     // ❌ wrong — must be relative
```

### 3. The `assets/` folder layout mirrors the `vpath`

The convention (and the thing that lets `dmm-mod-pack` auto-fill `vpath`
when you omit it): the path under `assets/` is the same as the in-game
`vpath`, with the leading `/` dropped.

| `vpath` in manifest | Path on disk |
|---|---|
| `/ui/icon/sword.dds` | `assets/ui/icon/sword.dds` |
| `/character/texture/macduff.dds` | `assets/character/texture/macduff.dds` |
| `/character/texture/macduff_n.dds` | `assets/character/texture/macduff_n.dds` |

You can deviate (the `file` and `vpath` fields are independent), but
mirroring is the convention everyone reads. Stick with it.

### 4. Manifest filename

Name it **`mod.field.json`**. That's the canonical name DMM and
`dmm-mod-*` look for first. Tools accept any `*.field.json` name but
you'll save yourself confusion if you stick with the canonical form.

The `.field.json` double-extension is intentional — it signals "this is
a Field-JSON v3.1 manifest" so DMM and other tools can recognize the
format from the filename alone.

### 5. Required vs optional manifest fields

```jsonc
{
  "format": 3,                  // REQUIRED — must be the integer 3
  "format_minor": 1,            // REQUIRED — pinned at 1 for v3.1, do NOT bump

  "modinfo": {                  // REQUIRED block
    "title":       "<string>",  // REQUIRED — appears in DMM's mod list
    "author":      "<string>",  // REQUIRED — credit yourself
    "version":     "<semver>",  // REQUIRED — `1.0`, `1.0.1`, etc.
    "description": "<string>",  // optional but strongly recommended
    "category":    "<string>"   // optional — `texture`, `custom_item`, etc.
  },

  "targets": [                  // REQUIRED — at least one entry
    { /* see recipes below */ }
  ]
}
```

Anything else you want to track (changelog, contact, screenshots-url) is
fine to add at the top level — DMM ignores unknown keys.

### Where to put the packed zip

After `dmm-mod-pack` produces `my_mod_v1.0.zip`, drop it into
**`<DMM-install-folder>/mods/`**. DMM watches that folder and
auto-imports new zips. From there:

1. Open DMM
2. Find your mod in the **Mods** list
3. Click the toggle to enable it
4. Click **Mount Mods**

If you're publishing on NexusMods, that zip is what users download —
they drop it into their own `mods/` folder.

---

## Picking the right DDS format

The single most common reason a texture mod doesn't work: wrong DDS
format. Each kind of texture has a specific format Crimson expects.
Use this table.

| Texture kind | Format | Why |
|---|---|---|
| **UI icon** (item icon, map icon, button) | **BC7 SRGB** or **BC1 SRGB** | UI textures are sampled in SRGB color space. BC7 = best quality, BC1 = smallest file. Both work. |
| **Character armor diffuse / albedo** (`/character/texture/<name>.dds`) | **BC7 SRGB** | High-quality color data with alpha for armor. BC1 also acceptable for budget mods. |
| **Character normal map** (`/character/texture/<name>_n.dds`) | **BC5** (linear, 2-channel) | Normal maps store XY only — BC5 is purpose-built for this. **Do NOT use BC7 for normals — colors will be wrong.** |
| **Character tattoo / decal** (`/character/texture/*tattoo*`) | **BC7 SRGB** | Same as armor — color + alpha. The `tattoo` in the path triggers a different blend mode in-game. |
| **Mask / metallic / roughness** | **BC4** (single-channel) or **BC5** (two-channel) | Linear, no alpha. BC4 for one channel, BC5 for two. |
| **Lightmap / HDR data** | **BC6H** | Float HDR. Specialized — most modders won't touch this. |

**Rule of thumb:** if you don't know, use **BC7** for color textures
and **BC5** for normal maps. Crimson accepts both standard `DXT*`
fourcc and modern `DX10` headers — modern DCC tools default to DX10/BC7
which is fine.

### What about mip maps?

**Always export with mip maps enabled.** The game expects them. Without
mips, distant textures render as solid color. Most DCC tools auto-
generate mips during DDS export — leave the option enabled.

### Common mistake: SRGB vs Linear

- **Color textures** (diffuse, UI, tattoo) → **SRGB**
- **Data textures** (normal, mask, metallic) → **Linear**

If your normal map looks washed-out or your icon looks dark, you
probably picked the wrong color space. Re-export with the right one.

---

## Where do game textures live? (vpath cheatsheet)

The `vpath` field tells DMM where in the game's archive your texture
goes. Get this wrong and your mod won't load.

### Vpath rules

- Forward slashes only (`/`), even on Windows
- Lowercase preferred (Crimson is case-insensitive but the convention is lowercase)
- Leading `/` optional but recommended for clarity
- Must end in `.dds`

### Common vpath patterns

| Vpath | Use case | last4 (auto-inferred) |
|---|---|---|
| `/ui/icon/<name>.dds` | Item icons, button icons, map icons | `0x1580` |
| `/ui/<anything>.dds` | Other UI textures (frames, panels) | `0x1580` |
| `/character/texture/<name>.dds` | Generic character/armor diffuse | `0x1280` |
| `/character/texture/<name>_n.dds` | Character normal map (suffix matters!) | `0x0480` |
| `/character/texture/<anything>tattoo<anything>.dds` | Tattoo/decal | `0x1380` |
| Other paths | Use existing path from the game | (depends on file) |

### How to find the exact vpath for an in-game texture

Three options, easiest first:

**Option A — copy from another mod.** Most NexusMods texture mods list
the vpaths they replace in their description. If your mod is replacing
the same files, copy those.

**Option B — extract from DMM's vanilla index.** DMM ships with PATHC
indices that list every texture file the game knows about. From the
DMM install:

```sh
python -c "import dmm_parser; pamt = dmm_parser.parse_pamt_from_file('<game>/0008/0.pamt'); [print(d.path + '/' + f.name) for d in pamt['directories'] for f in d['files'] if f['name'].endswith('.dds')]"
```

This lists every DDS the game has under group 0008 (UI textures).
Substitute other groups (`0009`, `0012`, etc.) for character/world.

**Option C — DMM's built-in browser.** Open DMM → Settings → Game
Maintenance → "Browse vanilla files" (1.3.4b+). Filter by extension
`.dds`. Click any row to copy its vpath.

### What the "last4" thing is

Crimson stores a 4-byte format identifier at byte offset 124 of every
DDS in its archive (Microsoft's `dwReserved2` field, repurposed). DMM
auto-fills this based on your vpath:

- If your vpath matches one of the patterns above, DMM uses the
  inferred value
- If your vpath isn't recognized, DMM falls back to the value derived
  from the DDS format itself (BC7 → `15`, BC1 → `12`, BC5/BC4 → `4`)

You don't need to manually set last4 in your manifest. The validator
will tell you if there's a mismatch.

---

## DCC tool exports

Specific export settings for the common tools.

### Photoshop with NVIDIA Texture Tools Exporter

1. Image → Mode → 8 Bits/Channel (or 16 for HDR)
2. File → Export → NVIDIA DDS Exporter (or save as `.dds` directly)
3. Format dropdown:
   - Color textures → **BC7 sRGB (DX10)**
   - Normal maps → **BC5 Snorm** (or `BC5 Unorm` if your normals are 0-1)
   - UI icons (small) → **BC1 sRGB** for smaller file size
4. Mipmaps: **Generate Mipmaps** ON, all levels
5. Alpha: keep alpha channel if your texture has transparency
6. Click Save

### GIMP with the DDS Plugin

GIMP's DDS plugin is older and doesn't handle DX10/BC7 cleanly. For
modern textures, install [NVIDIA Texture Tools](https://developer.nvidia.com/texture-tools-exporter)
and use Photoshop, OR use the command-line `nvtt_export`:

```sh
nvtt_export input.png --format bc7 --output output.dds --mips
```

For older `DXT5` workflow in GIMP:
1. File → Export As → `name.dds`
2. Compression: **BC3 / DXT5**
3. Mipmaps: **Generate mipmaps**
4. Format: **DDS** with no DX10 header

### Substance Painter / Designer

1. Export Textures dialog
2. Output template: pick "DDS" or create a custom one
3. Per channel:
   - Diffuse / albedo → **BC7 SRGB**
   - Normal → **BC5**
   - Metal / Rough / AO → **BC4** or pack two into **BC5**
4. Generate mipmaps: ON

### Command-line: NVIDIA Texture Tools (nvtt_export)

The most reliable cross-platform way to produce game-ready DDS:

```sh
# Color (SRGB)
nvtt_export input.png --format bc7 --output output.dds --mips --srgb

# Normal map (linear)
nvtt_export normal.png --format bc5 --output normal.dds --mips --normal

# UI icon (smaller)
nvtt_export icon.png --format bc1 --output icon.dds --mips --srgb
```

Get nvtt_export from:
https://developer.nvidia.com/texture-tools-exporter

### Verifying your DDS in DMM

Before packing, run:

```sh
python -c "import dmm_parser; print(dmm_parser.classify_dds(open('your.dds', 'rb').read()))"
```

This prints the format, dimensions, mip count, and last4 value. If
`requires_pathc: True` shows up, your texture is BC7/DX10 — that's
fine, DMM handles PATHC registration automatically.

---

## Validation and packing

The two CLI tools you'll use for every mod:

### `dmm-mod-validate`

Pre-flight check. Catches problems before upload.

```sh
python -m dmm_parser.tools.validate my_mod/mod.field.json --assets my_mod/assets
```

What it checks:
- Manifest JSON is valid
- Every asset file referenced exists at the given relative path
- Every DDS passes the validator (correct magic, header size, sane
  dimensions, mips present, format recognized)
- Every audio asset (WEM/BNK) passes its respective validator
- SHA-256 hashes in the manifest match the actual files
- vpaths look sane

Output is a list of findings. `severity: fatal` blocks packing.
`severity: warning` is fixable but not blocking. `severity: info` is
informational.

### `dmm-mod-pack`

Builds the shippable zip.

```sh
python -m dmm_parser.tools.pack my_mod/mod.field.json --assets my_mod/assets --out my_mod_v1.0.zip
```

What it does:
- Auto-fills `sha256` and `size` for every asset
- Auto-infers `vpath` for assets where you didn't set one (when the
  asset folder is laid out under a 4-digit PAZ group prefix)
- Bundles the manifest + assets into a zip with the layout DMM expects
- Refuses to pack if SHA-256 doesn't match (catches "you edited a file
  but forgot to update the manifest" errors)

`--no-fill` preserves your manifest exactly as written. Use this if
you're hand-managing hashes for reproducibility.

### `dmm-mod-inspect`

For testing what a packed mod actually does:

```sh
python -m dmm_parser.tools.inspect my_mod_v1.0.zip
```

Prints the mod's identity, every target it touches, and a summary of
what each target does. Run this on third-party mods before installing
them.

### `dmm-mod-diff`

Conflict check between two mods:

```sh
python -m dmm_parser.tools.diff mod_a.zip mod_b.zip
```

Reports:
- Asset vpath collisions (both mods replacing the same file)
- Table-row collisions (both editing the same item)
- Paloc-key collisions (both setting the same localized string)

Useful when you have two mods that both modify armor — confirms which
files conflict.

---

## Releasing on NexusMods

The standard NexusMods page format that pairs cleanly with v3.1 mods:

### Page structure

```
Name: <Mod Title>
Version: <semver>
Category: Models and Textures (or appropriate)
Tags: textures, [armor / ui / weapon / etc.]

Description:
  - One-line summary
  - Screenshots (vanilla vs. modded comparison helps a LOT)
  - What's changed (list of files / textures replaced)
  - Compatibility (other mods this stacks with / conflicts with)
  - Installation (link to DMM)
  - Credits (asset sources, references, etc.)
```

### Files tab

Upload **the zip produced by `dmm-mod-pack`** as the main file. Don't
upload the loose `.dds` files or the manifest separately — DMM expects
the packed format. Users drop the zip into DMM's mod folder, DMM
auto-imports it, they enable it, they mount.

### A reusable description template

```markdown
## What this mod does

Replaces the diffuse texture for [item / armor / icon] with [your
custom version].

## Screenshots

[before/after pairs]

## Files replaced

- `/character/texture/macduff_diffuse.dds`
- `/character/texture/macduff_diffuse_n.dds`

## Installation

1. Install DMM: https://www.nexusmods.com/crimsondesert/mods/<dmm-id>
2. Drop this mod's `.zip` into DMM's `mods/` folder
3. Open DMM, enable the mod, click Mount Mods

## Compatibility

- Compatible with all v3.1 mods that don't replace the same vpaths
- Use `dmm-mod-diff` (built into DMM 1.3.4+) to check for conflicts
  before mounting

## Credits

- Original mesh: [game]
- Textures: [your tool / source]
```

### Versioning convention

Match the manifest's `modinfo.version`. Use [semver](https://semver.org/)
- `1.0.0` for first release
- `1.0.1` for bug fixes (texture seam, alpha channel issues)
- `1.1.0` for new variants (color options)
- `2.0.0` for breaking changes (renaming files, restructuring vpaths)

Bump `version` in `mod.field.json` for every NexusMods upload so users
can see they're getting the latest.

---

## Recipes

Common scenarios with concrete manifests.

### Recipe 1: Replace a single UI icon

```json
{
  "format": 3,
  "format_minor": 1,
  "modinfo": { "title": "Custom Sword Icon", "author": "you", "version": "1.0" },
  "targets": [
    {
      "kind": "asset",
      "asset_type": "dds",
      "file": "ui/icon/sword.dds",
      "vpath": "/ui/icon/sword.dds"
    }
  ]
}
```

### Recipe 2: Replace character armor (diffuse + normal map)

```json
{
  "format": 3,
  "format_minor": 1,
  "modinfo": { "title": "Macduff Armor Retexture", "author": "you", "version": "1.0" },
  "targets": [
    {
      "kind": "asset",
      "asset_type": "dds",
      "file": "character/texture/macduff_diffuse.dds",
      "vpath": "/character/texture/macduff_diffuse.dds"
    },
    {
      "kind": "asset",
      "asset_type": "dds",
      "file": "character/texture/macduff_diffuse_n.dds",
      "vpath": "/character/texture/macduff_diffuse_n.dds"
    }
  ]
}
```

The `_n.dds` suffix automatically routes to the normal-map last4
class (`0x0480`). Don't forget to export the normal map as **BC5**,
not BC7 — wrong format on a normal map produces broken-looking shading.

### Recipe 3: Multi-variant retexture (color options)

Ship one zip with multiple variants — let users pick which to enable.
Currently DMM treats this as multiple separate mods; you ship multiple
manifests:

```
macduff_retexture/
├── red/
│   ├── mod.field.json    # vpath stays the same; assets differ
│   └── assets/
├── blue/
│   ├── mod.field.json
│   └── assets/
└── README.md
```

Each variant gets its own zip via `dmm-mod-pack`. Users install one or
the other (not both — vpath collision).

### Recipe 4: Tattoo / decal mod

```json
{
  "targets": [
    {
      "kind": "asset",
      "asset_type": "dds",
      "file": "character/texture/dragon_tattoo_alpha.dds",
      "vpath": "/character/texture/dragon_tattoo_alpha.dds"
    }
  ]
}
```

The word `tattoo` anywhere in the vpath triggers tattoo blending
(`0x1380`). Export as **BC7 SRGB** with alpha for proper transparency.

### Recipe 5: Texture mod + custom item (mixed mod)

If your texture is for a custom item you're also adding (e.g. a unique
sword with a unique icon), ship them together:

```json
{
  "targets": [
    {
      "target": "iteminfo.pabgb",
      "intents": [
        {
          "op": "clone_record",
          "source_key": 12345,
          "new_key": 999001,
          "patches": [
            { "path": "string_key", "new": "Custom_Sword" },
            { "path": "item_name.default", "new": "Sword of Glory" }
          ]
        }
      ]
    },
    {
      "target": "paloc.pamt",
      "intents": [
        { "op": "set_localization", "key": 4290676623671408, "lang": "en", "value": "Sword of Glory" }
      ]
    },
    {
      "kind": "asset",
      "asset_type": "dds",
      "file": "ui/icon/sword_of_glory.dds",
      "vpath": "/ui/icon/sword_of_glory.dds"
    }
  ]
}
```

See `docs/CUSTOM_ITEM_CREATOR_V3_1.md` for the full custom-item recipe.

### Recipe 6: Auto-vpath — skip the `vpath` field for whole-folder ports

If your `assets/` layout already mirrors the in-game vpaths exactly,
omit the `vpath` field entirely. `dmm-mod-pack` infers it from `file`.

Source folder:

```
icon_overhaul/
├── mod.field.json
└── assets/
    └── ui/
        └── icon/
            ├── sword.dds
            ├── shield.dds
            ├── potion.dds
            └── ... (50 more icons)
```

Manifest (no `vpath` fields needed):

```json
{
  "format": 3,
  "format_minor": 1,
  "modinfo": { "title": "Icon Overhaul", "author": "you", "version": "1.0" },
  "targets": [
    { "kind": "asset", "asset_type": "dds", "file": "ui/icon/sword.dds" },
    { "kind": "asset", "asset_type": "dds", "file": "ui/icon/shield.dds" },
    { "kind": "asset", "asset_type": "dds", "file": "ui/icon/potion.dds" }
  ]
}
```

`dmm-mod-pack` reads each `file`, prepends `/`, and writes
`vpath: /ui/icon/sword.dds` etc. into the packed manifest. The
shipped zip has fully-resolved vpaths so consumers don't need to
re-run inference.

Use this when you have lots of textures and the layout is uniform.
For a single-file mod the explicit-vpath form (Recipe 1) is clearer.

---

## Troubleshooting

### "Texture loads in-game but looks wrong (purple, magenta, broken)"

Almost always a **format mismatch**. Likely causes:

- Used BC7 for a normal map → re-export as BC5
- Used BC1 with alpha → re-export as BC3 (DXT5) or BC7 SRGB
- Wrong color space (linear data exported as SRGB or vice versa)
- Mip maps missing → re-export with mips ON

### "validate reports `dds_unknown_fourcc` warning"

DMM doesn't recognize the FOURCC in your DDS. The texture might still
load (DMM falls back to format-derived last4), but for safety re-export
with one of the standard FOURCCs: `DXT1`, `DXT5`, `BC4U`, `BC5U`, or
`DX10`. Modern tools default to DX10 which is fine.

### "validate reports `dds_non_power_of_two` warning"

Crimson is more permissive than older engines but POW2 is still safer.
Resize your source to the nearest power of two (256, 512, 1024, 2048,
4096) and re-export.

### "validate reports `requires_pathc` info"

This is **informational, not an error**. BC7 / DX10 textures need to
be registered in the game's PATHC index. DMM does this automatically at
mount time. You don't need to do anything.

### "validate reports `asset_sha_mismatch`"

You edited a DDS but the `sha256` in your manifest is stale. Either:

- Run `dmm-mod-pack` again (it re-computes hashes), OR
- Run with `--fill` to update without packing:
  `python -m dmm_parser.tools.pack <manifest> --assets <dir> --fill --no-pack`

### "Texture loads in-game but it's the vanilla version, not mine"

Your `vpath` doesn't match what the game actually loads. Verify the
exact path:

1. Use DMM's vanilla browser (Settings → Game Maintenance → Browse
   vanilla files)
2. Look for the file you intend to replace
3. Copy its vpath (right-click → Copy)
4. Paste into your manifest's `vpath`

### "Mount fails with `dispatch_panic_caught` on my mod's asset"

A specific texture in your mod is malformed in a way DMM's pre-flight
didn't catch. Run:

```sh
python -c "import dmm_parser; print(dmm_parser.validate_dds(open('your.dds', 'rb').read()))"
```

Look for `severity: fatal` entries. The most common: header truncation
(file got corrupted during transfer or export) or a non-DX10 file
claiming BC7 fourcc (mix-up between fourcc='BC7 ' which doesn't exist
and the proper DX10 + DXGI 98).

### "Two of my mods conflict but I want them both active"

You can't — DMM rejects asset-vpath collisions at mount time. Either:

- Combine the two mods into one with both textures
- Choose which to enable per session
- Author a different vpath in one of the mods (only works if the
  duplicate file is a NEW file, not a vanilla replacement)

---

## See also

- [`MOD_AUTHOR_GUIDE.md`](MOD_AUTHOR_GUIDE.md) — top-level overview of
  every mod type (data, texture, audio, custom item, mixed)
- [`BINARY_FORMATS.md#file-format-reference-formats`](BINARY_FORMATS.md#file-format-reference-formats) §7 — DDS format specification with
  Crimson-specific quirks
- [`api.md`](api.md) "DDS Textures" — Python API reference
- [`references/dds_notes.md`](../references/dds_notes.md) — deep-dive
  reverse-engineering notes on Crimson's DDS handling
- [`samples/02_texture_swap/`](../samples/02_texture_swap/) — runnable
  texture-mod example
- [`samples/04_custom_item/`](../samples/04_custom_item/) — texture +
  paloc + iteminfo mixed example
- [`samples/05_mixed_overhaul/`](../samples/05_mixed_overhaul/) —
  large-scale multi-asset example

---

*Questions or improvements? Open an issue at
https://github.com/exodiaprivate-eng/dmm-parser/issues.*
