# Sample 02 — Texture swap

A complete worked example for replacing a UI icon with a custom DDS.
This is the canonical "single texture, single target" mod — the
smallest useful texture mod you can ship.

For the full reference see
[`docs/TEXTURE_MOD_AUTHORING.md`](../../docs/TEXTURE_MOD_AUTHORING.md).
For a one-page cheat sheet see
[`docs/TEXTURE_VPATH_CHEATSHEET.md`](../../docs/TEXTURE_VPATH_CHEATSHEET.md).

## Files

```
02_texture_swap/
├── mod.field.json          ← the manifest
├── README.md               ← this file
└── assets/                 ← payload (you create this)
    └── icons/
        └── sword.dds       ← drop your replacement here
```

## What this teaches

- The minimum-viable v3.1 manifest (one target, one DDS, no extras)
- How `kind: "asset"` + `asset_type: "dds"` declares a texture target
- How `file:` (relative to assets folder) and `vpath:` (in-game path)
  are different and both required
- How `dmm-mod-pack` auto-fills `sha256` and `size` so you don't
  hand-manage them
- How the `/ui/icon/...` vpath prefix automatically routes to the right
  Crimson "last4" overlay class (`0x1580`)

## Step-by-step build

### 1. Author your texture

Open your DCC tool of choice. The vanilla in-game icon is small (icons
are typically 96×96 or 128×128). Make sure your replacement matches the
original aspect ratio so it looks right in the inventory grid.

### 2. Export as DDS

For a UI icon, export with these settings:

| Tool | Format | Mips | Color space |
|---|---|---|---|
| Photoshop + NVIDIA Texture Tools | **BC7 sRGB (DX10)** | Generate | sRGB |
| GIMP + DDS plugin | **BC3 / DXT5** | Generate | sRGB |
| Substance Painter | **BC7 SRGB** | Generate | sRGB |
| Command-line (recommended) | see below | — | — |

Command-line via NVIDIA Texture Tools:

```sh
nvtt_export sword.png --format bc7 --output sword.dds --mips --srgb
```

Get nvtt_export from https://developer.nvidia.com/texture-tools-exporter.

### 3. Place the DDS

Drop your exported `sword.dds` at:

```
02_texture_swap/assets/icons/sword.dds
```

The path inside `assets/` matches the manifest's `file:` field.

### 4. Validate

```sh
python -m dmm_parser.tools.validate 02_texture_swap/mod.field.json --assets 02_texture_swap/assets
```

What this checks:
- Manifest JSON is valid v3.1
- The DDS exists at `assets/icons/sword.dds`
- The DDS header is well-formed (correct magic, sane dimensions, mips
  present)
- The DDS format is recognized

A successful validation prints `0 fatal, 0 warning` (or `1 info`
mentioning `requires_pathc` if you used BC7 — that's fine, it's just
informational).

### 5. Pack

```sh
python -m dmm_parser.tools.pack 02_texture_swap/mod.field.json --assets 02_texture_swap/assets --out my_sword_icon_v1.0.zip
```

This produces `my_sword_icon_v1.0.zip` with the manifest at the root
and the asset under `assets/icons/sword.dds`. SHA-256 and file size are
auto-filled in the bundled manifest.

### 6. Test in DMM

1. Drop `my_sword_icon_v1.0.zip` into DMM's `mods/` folder
2. Open DMM — the mod should appear in the list
3. Enable it
4. Click "Mount Mods"
5. Launch Crimson Desert
6. Find an item using that icon (the standard short sword) — your
   custom icon should appear

If the vanilla icon still appears, your `vpath` is wrong. See the
[Troubleshooting](../../docs/TEXTURE_MOD_AUTHORING.md#troubleshooting)
section in the full guide.

### 7. Upload to NexusMods

Upload `my_sword_icon_v1.0.zip` as the main file. See
[NexusMods page template](../../docs/TEXTURE_MOD_AUTHORING.md#releasing-on-nexusmods)
for description boilerplate that pairs cleanly with v3.1 mods.

## Inspecting the manifest

```sh
python -m dmm_parser.tools.inspect 02_texture_swap/mod.field.json
```

prints:
```
[asset] icons/sword.dds -> /ui/icon/sword.dds  (DDS)
```

That's the full surface — one line per target. Useful for verifying
you've wired the manifest right before packing.

## Adapting for your own texture

Open `mod.field.json` and change:

- `modinfo.title` — your mod's display name
- `modinfo.author` — you
- `modinfo.version` — start at `1.0.0`
- `targets[0].file` — the path to your DDS under `assets/`
- `targets[0].vpath` — the EXACT game path you're replacing

For finding the right `vpath`, see
[Where do game textures live?](../../docs/TEXTURE_MOD_AUTHORING.md#where-do-game-textures-live-vpath-cheatsheet)
in the full guide.

## Variants

The full guide covers more advanced cases:

- [Recipe 2 — Replace character armor with diffuse + normal map](../../docs/TEXTURE_MOD_AUTHORING.md#recipe-2-replace-character-armor-diffuse--normal-map)
- [Recipe 3 — Multi-variant retexture (color options)](../../docs/TEXTURE_MOD_AUTHORING.md#recipe-3-multi-variant-retexture-color-options)
- [Recipe 4 — Tattoo / decal mod](../../docs/TEXTURE_MOD_AUTHORING.md#recipe-4-tattoo--decal-mod)
- [Recipe 5 — Texture mod + custom item (mixed)](../../docs/TEXTURE_MOD_AUTHORING.md#recipe-5-texture-mod--custom-item-mixed-mod)
