# Sample Mods

Five worked examples covering each mod type the Field-JSON v3.1 format
supports. Each directory contains a `README.md` plus a runnable
`.field.json` so you can copy, tweak, and pack with
`python -m dmm_parser.tools.pack`.

| Sample                       | Demonstrates                              |
|------------------------------|-------------------------------------------|
| `01_simple_data_mod/`        | Pure table edit (no assets)               |
| `02_texture_swap/`           | DDS asset replacement                     |
| `03_audio_replacement/`      | WEM voice clip replacement                |
| `04_custom_item/`            | Adding a new ItemInfo row + icon + strings|
| `05_mixed_overhaul/`         | Multiple targets in one manifest          |

## Running a sample

The samples don't ship binary asset payloads — they reference asset
files by path so you can drop your own DDS/WEM in. Use them as
manifest templates:

```sh
# Inspect the manifest
python -m dmm_parser.tools.inspect samples/01_simple_data_mod/mod.field.json

# Validate (only the manifest-level checks pass without real assets)
python -m dmm_parser.tools.validate samples/02_texture_swap/mod.field.json
```

Each sample's `README.md` documents what to drop into the `assets/`
subfolder before packing.
