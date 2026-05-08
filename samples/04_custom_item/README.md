# Sample 04 — Custom item

The full custom-item recipe: clone an existing item under a fresh key,
retitle it, register localized strings, and ship a custom icon. This
mod demonstrates the **canonical v3.1 custom-item shape** — three
targets (table + paloc + asset) wired together.

## Files

- `mod.field.json` — manifest with three targets:
  - `iteminfo.pabgb` with a `clone_record` intent (source_key 12345 → new_key 999001)
  - `paloc.pamt` with localization for the new item's name + description
  - `asset` (DDS) for the in-game icon
- `assets/icons/sword_of_potter.dds` — drop your 96×96 BC7 icon here before packing

## Why `clone_record` instead of building from scratch

Iteminfo records have ~70 fields with cross-table references (gimmicks,
buffs, drop sets, prefab data). Cloning an existing item gives you a
known-good starting point — the donor's enchant data, prefab visuals,
and equipment slot are all valid by construction. Patches replace only
the fields that need to change for your custom version.

## The paloc index formula

For each new item key, the game looks up its localized name and
description via two paloc entries with computed keys:

| Field | Paloc index |
|---|---|
| `item_name.index` | `(new_key << 32) \| 0x70` |
| `item_desc.index` | `(new_key << 32) \| 0x71` |

For `new_key = 999001`:

```python
import dmm_parser
name_idx, desc_idx = dmm_parser.item_paloc_indices(999001)
# (4290772592, 4290772593)
```

Both indices appear twice in the manifest — once on the iteminfo clone
(the lookup pointer) and once on the paloc target (the actual string).

## Authoring

```python
import dmm_parser, json

# 1) Read vanilla iteminfo + sister pabgh
pabgb = open("iteminfo.pabgb", "rb").read()

# 2) Load this manifest's iteminfo intents
doc = json.load(open("samples/04_custom_item/mod.field.json"))
target = next(t for t in doc["targets"] if t.get("target") == "iteminfo.pabgb")

# 3) Apply
result = dmm_parser.apply_intents("iteminfo.pabgb", pabgb, None, target["intents"])
open("iteminfo_modded.pabgb", "wb").write(result["body"])
print("outcomes:", result["outcomes"])

# 4) Repeat for paloc.pamt with the corresponding target's intents.
# 5) Pack the asset folder and the modified pabgb files into a PAZ overlay
#    via dmm_parser.pack_mod (see docs/api.md).
```

## Building from the CLI

```sh
python -m dmm_parser.tools.validate samples/04_custom_item/mod.field.json --assets samples/04_custom_item/assets
python -m dmm_parser.tools.pack     samples/04_custom_item/mod.field.json --assets samples/04_custom_item/assets
```

## Adapting for your own item

1. Pick a donor item key from a vanilla iteminfo dump
   (`dmm_parser.parse_iteminfo_from_file` returns the full list).
2. Pick a `new_key` ≥ 999000 to stay clear of vanilla and future patches.
3. Compute paloc indices via `dmm_parser.item_paloc_indices(new_key)`.
4. Edit the patches list to override whatever fields you want
   (`max_stack_count`, `enchant_data_list[0].equip_buffs`, etc. —
   see `FIELD_JSON_V3_SPEC.md` for path syntax).
5. Drop your DDS icon at `assets/icons/<name>.dds` and update `vpath`.
6. Update both paloc strings to match.

For a deeper field-level reference see `docs/CUSTOM_ITEM_CREATOR_V3_1.md`.
