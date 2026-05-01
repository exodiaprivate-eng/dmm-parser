# Sample 04 — Custom item

The full custom-item recipe: a new ItemInfo row, a custom icon, and
localized name + description strings.

## Files

- `mod.field.json` — manifest with three targets: table + asset + paloc
- `assets/icons/sword_of_potter.dds` — drop your icon here

## What this teaches

- The three-target pattern that every custom item needs: data row +
  icon DDS + paloc strings
- ID 100001 is in the safe range (≥100000) so it won't collide with
  future game patches
- Each paloc entry needs the right `category` (0 for typical UI text)

## Building

```sh
python -m dmm_parser.tools.validate 04_custom_item/mod.field.json --assets 04_custom_item/assets
python -m dmm_parser.tools.pack    04_custom_item/mod.field.json --assets 04_custom_item/assets
```

For a deep-dive into the `ItemInfo` row fields (equipment slot,
sockets, max enchant, etc.) see `docs/CUSTOM_ITEM_CREATOR_V3_1.md`.
