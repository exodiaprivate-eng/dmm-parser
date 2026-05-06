<!-- SPDX-License-Identifier: LicenseRef-CDMTL-1.0
     Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
     Licensed under CDMTL v1.0 - see LICENSE.txt -->

# Texture vpath + format cheatsheet

One-page reference. Print this. Tape it next to your monitor.

For the full guide, see [TEXTURE_MOD_AUTHORING.md](TEXTURE_MOD_AUTHORING.md).

---

## Format → use case

| Texture is... | Format | Color space | Notes |
|---|---|---|---|
| UI icon | **BC7** or **BC1** | SRGB | BC1 if you need small files |
| Armor diffuse | **BC7** | SRGB | High-quality color + alpha |
| Normal map (`*_n.dds`) | **BC5** | Linear | XY only, BC5 is purpose-built |
| Tattoo (`*tattoo*.dds`) | **BC7** | SRGB | Color + alpha for blending |
| Mask (1 channel) | **BC4** | Linear | Single-channel grayscale |
| Mask (2 channel) | **BC5** | Linear | Two channels packed |
| HDR / lightmap | **BC6H** | Linear | Specialized — rare |

**Always export with mip maps enabled.**

---

## Vpath patterns the game recognizes

| Vpath pattern | last4 (auto) | Used for |
|---|---|---|
| `/ui/icon/<name>.dds` | `0x1580` | Item icons, button icons |
| `/ui/<name>.dds` | `0x1580` | All other UI textures |
| `/character/texture/<name>.dds` | `0x1280` | Generic character / armor |
| `/character/texture/<name>_n.dds` | `0x0480` | Character normal map |
| `/character/texture/*tattoo*.dds` | `0x1380` | Tattoo / decal |
| Other | (format-derived) | Use existing vpath from game |

Forward slashes only. Lowercase preferred. Always ends in `.dds`.

---

## Manifest skeleton

```json
{
  "format": 3,
  "format_minor": 1,
  "modinfo": {
    "title": "Your Mod Name",
    "author": "Your Name",
    "version": "1.0",
    "description": "What it does."
  },
  "targets": [
    {
      "kind": "asset",
      "asset_type": "dds",
      "file": "<relative path under assets/>",
      "vpath": "<game vpath>"
    }
  ]
}
```

Multiple textures? Add more `targets` entries. Mixed with table edits?
See [`MOD_AUTHOR_GUIDE.md`](MOD_AUTHOR_GUIDE.md).

---

## Build pipeline

```sh
# Pre-flight check (catches problems before users see them)
python -m dmm_parser.tools.validate my_mod/mod.field.json --assets my_mod/assets

# Build the shippable zip (auto-fills sha256 + size + vpath)
python -m dmm_parser.tools.pack my_mod/mod.field.json --assets my_mod/assets --out my_mod_v1.0.zip

# Inspect what a packed mod actually does (yours or anyone else's)
python -m dmm_parser.tools.inspect my_mod_v1.0.zip

# Conflict-check between two mods
python -m dmm_parser.tools.diff mod_a.zip mod_b.zip
```

---

## Folder layout (after `dmm-mod-pack`)

```
my_mod_v1.0.zip
├── mod.field.json          ← manifest at root
└── assets/                 ← every file referenced
    ├── ui/icon/sword.dds
    ├── character/texture/macduff.dds
    └── character/texture/macduff_n.dds
```

Drop the zip into DMM's `mods/` folder. Done.

---

## Five most common mistakes

1. **BC7 used for a normal map** → re-export as BC5. Symptom: shading
   looks wrong, lighting breaks at certain angles.
2. **No mip maps** → re-export with mips ON. Symptom: distant
   textures render as solid color.
3. **Wrong color space** (SRGB vs Linear) → match the table above.
   Symptom: washed-out colors or too-dark icon.
4. **Wrong vpath** → check exact path against DMM's vanilla browser.
   Symptom: vanilla texture still loads, your version invisible.
5. **Asset hash mismatch after editing** → run `dmm-mod-pack` again.
   Symptom: validator reports `asset_sha_mismatch`.

---

## DCC tool quick reference

**Photoshop + NVIDIA Texture Tools Exporter**: Format = `BC7 sRGB (DX10)`,
Generate Mipmaps ON.

**GIMP + DDS plugin**: Compression = `BC3 / DXT5`, Generate mipmaps ON.
For modern BC7 use NVIDIA's `nvtt_export` instead.

**Substance Painter**: Output template = DDS. Per-channel format from
the table above. Generate mipmaps ON.

**Command line (recommended)**: NVIDIA Texture Tools (`nvtt_export`):

```sh
# Color
nvtt_export input.png --format bc7 --output output.dds --mips --srgb

# Normal
nvtt_export normal.png --format bc5 --output normal.dds --mips --normal

# UI icon (smaller)
nvtt_export icon.png --format bc1 --output icon.dds --mips --srgb
```

Get nvtt_export: https://developer.nvidia.com/texture-tools-exporter
