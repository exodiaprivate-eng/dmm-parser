# Sample 02 — Texture swap

Replaces a UI icon with a custom DDS. Demonstrates the DDS asset target,
including the Crimson "last4" overlay class inferred from the vpath.

## Files

- `mod.field.json` — manifest with one DDS asset target
- `assets/icons/sword.dds` — drop your replacement texture here

## Building

1. Author your DDS in your DCC (export with vanilla format, e.g. BC7
   SRGB for a UI icon).
2. Place it at `assets/icons/sword.dds`.
3. Validate + pack:

```sh
python -m dmm_parser.tools.validate 02_texture_swap/mod.field.json --assets 02_texture_swap/assets
python -m dmm_parser.tools.pack    02_texture_swap/mod.field.json --assets 02_texture_swap/assets
```

`pack` will fill in `sha256` and `size` automatically. The vpath is
already set in the manifest so the loader gets the right last4 class.

## What this teaches

- `kind: "asset"` + `asset_type: "dds"` + `file:` + `vpath:`
- vpath under `/ui/icon/...` → last4 `0x1580`
- `dmm-mod-pack` auto-fills sha256/size at pack time
