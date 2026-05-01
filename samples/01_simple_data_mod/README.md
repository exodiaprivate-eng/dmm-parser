# Sample 01 — Simple data mod

A mod that touches one ItemInfo row to make a stack of healing potions
fit 99 in a slot instead of the vanilla limit. No binary assets — just
a pure table edit.

## Files

- `mod.field.json` — the manifest

## Try it

```sh
python -m dmm_parser.tools.inspect 01_simple_data_mod/mod.field.json
python -m dmm_parser.tools.validate 01_simple_data_mod/mod.field.json
```

## What this teaches

- `format_minor: 1` is required for the new `targets:` wrapper
- A `kind: "table"` target is just `{ table, ops[] }`
- Each op has `key`, `field`, `value` — the parser figures out the type
