# Sample 05 — Mixed overhaul

A "small overhaul" mod that touches several systems at once: tweaks
multiple ItemInfo rows, swaps two textures, replaces a voice clip, and
ships paloc updates for the strings it touches. Demonstrates how
`targets:` composes and how `dmm-mod-diff` flags conflicts when two
overhauls touch the same row.

## Files

- `mod.field.json` — manifest with five targets covering all four kinds
- `assets/` — drop the referenced DDS / WEM files

## What this teaches

- `targets: [...]` order doesn't affect apply order — SWISS sorts by
  kind during apply
- A single mod can mix table + asset + paloc freely
- For overhauls, run `dmm-mod-diff` against any other large mod
  before shipping — overhauls collide more often than focused mods

## Building

```sh
python -m dmm_parser.tools.validate 05_mixed_overhaul/mod.field.json --assets 05_mixed_overhaul/assets
python -m dmm_parser.tools.diff     05_mixed_overhaul/mod.field.json some-other-overhaul.zip
python -m dmm_parser.tools.pack    05_mixed_overhaul/mod.field.json --assets 05_mixed_overhaul/assets
```
