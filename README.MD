# dmm-parser

Pabgb / pabgh / paz / pamt / papgt parser for the DMM mod loader.

Forked from `crimson-rs` by `potter4208467` (Discord) with explicit permission.

## License & Authorized Distribution

Licensed under **CDMTL v1.0** (Crimson Desert Modding Tools License v1.0), effective 2026-05-01. See [LICENSE.txt](LICENSE.txt) for full terms.

**This software is distributed exclusively through the following authorized channels:**

- **Source**: https://github.com/exodiaprivate-eng/dmm-parser
- **Releases**: https://github.com/exodiaprivate-eng/dmm-parser/releases
- **NexusMods**: only via the official RicePaddySoftware author page when published

Any copy of this software, or any derivative work based on it, found outside these channels is **unauthorized** and subject to DMCA takedown. This includes uploads to NexusMods, ModDB, GameBanana, CurseForge, file-sharing services, and third-party GitHub repositories not operated by RicePaddySoftware.

**Trademarks**: "DMM", "Definitive Mod Manager", "SWISS", "SWISS Suite", "Field JSON v3.1", "Multi-Target Field Patching", "RicePaddySoftware", and "CrimsonGameMods" are trademarks of RicePaddySoftware. Use of these names — including confusingly similar variants such as "CDUMM", "DUMM", or "DMM2" — by competing tools is prohibited.

**Found an unauthorized copy?** Open an issue on this repository with the URL and a brief description, or follow the platform's own abuse-reporting flow.

## Status

- **509 / 509** Rust tests pass (`cargo test --release --lib`) — up from 402
- 12 / 12 Python tools smoke tests pass (`python -m dmm_parser.tools._test_smoke`)
- 122 pabgb tables round-trip byte-perfect (+ paloc localization format)
- 67 fully field-decoded (every field individually addressable, including BuffInfo + SkillInfo with all 120 BuffData variants)
- PALOC (localization) format supported with category-based JSON form: `parse_paloc_to_json` / `serialize_paloc_from_json`
- 54 byte-perfect with `Vec<u8>` blob-tail for polymorphic regions still being decoded
- **DDS** textures (Crimson quirks): `classify_dds`, `validate_dds`, `infer_dds_vpath`, `classify_vpath_last4`
- **Wwise audio** (WEM + BNK): `classify_wem`, `parse_bnk`, `validate_audio`, `infer_audio_vpath`
- **Save envelope** (S2): ChaCha20 + LZ4 + HMAC pipeline with caller-supplied keys (no embedded secrets)
- **Field-JSON v3.1** mod-author CLIs: `dmm-mod-validate`, `dmm-mod-pack`, `dmm-mod-inspect`, `dmm-mod-diff`
- **Sequencer / schedule / attack-info Tier 1**: 6 formats round-trip byte-perfect across **18,952 vanilla samples (100%)** — `.pastage`, `.paseq`, `.paseqc`, `.paschedule`, `.paschedulepath`, `.paatt`. Field-level Python access for cutscene scripts, NPC schedules, and per-weapon attack data. Schema enumeration: 272 distinct classes for `.paseq`, 62 for `.paseqc`. End-to-end mod-tooling validated at 195/195 across walk+replace and JSON-path edit patterns.

## Python bindings

Built via `maturin`; the Rust crate exposes a Python module named `dmm_parser`.

```bash
maturin build --release
pip install target/wheels/dmm_parser-*.whl
```

### Quick start

```python
import dmm_parser

items = dmm_parser.parse_iteminfo_from_file("iteminfo_decompressed.pabgb")
print(f"Loaded {len(items)} items")

item = items[0]
print(item["key"])         # 2200
print(item["string_key"])  # "Pyeonjeon_Arrow"
print(item["item_type"])   # 0
print(item["item_tier"])   # 0

# Nested types are also dicts
print(item["item_name"]["category"])
print(item["item_name"]["index"])
print(item["item_name"]["default"])

# Arrays are plain lists
print(item["item_tag_list"])
print(item["price_list"])

# Optional fields are None when absent
print(item["docking_child_data"])

# Modify any field
item["item_tier"] = 5
item["max_stack_count"] = 999
item["item_name"]["default"] = "Custom Name"

# Write modified items back to a file (byte-perfect round-trip)
dmm_parser.write_iteminfo_to_file(items, "output.pabgb")

# Or get raw bytes
data = dmm_parser.serialize_iteminfo(items)

# Parse from bytes
items = dmm_parser.parse_iteminfo_from_bytes(data)
```

### Sequencer / schedule editing (Tier 1 formats)

```python
import dmm_parser

# Find every length-prefixed string in any .paseq / .paseqc / .pastage
data = open("cd_seq_ui_appear_all.paseq", "rb").read()
strings = dmm_parser.walk_lp_strings(data)
for s in strings[:5]:
    print(f"  0x{s['file_offset']:04x}  {s['value']!r}")

# Edit one — length-flexible
modified = dmm_parser.replace_cstring_at(
    data, strings[0]['file_offset'],
    new_value="my_renamed_value",
    expected_value=strings[0]['value'],  # safety check
)

# For structured-header formats (.paschedule, .paatt) use the JSON path
parsed = dmm_parser.parse_paschedule_from_file("npc_schedule.paschedule")
parsed["name"] = "my_renamed_schedule"
dmm_parser.write_paschedule_to_file(parsed, "npc_schedule.paschedule")

# Enumerate the class hierarchy embedded in a .paseqc
blocks = dmm_parser.parse_paseqc_all_class_blocks(data)
print(f"{len(blocks)} class blocks in this sequencer chart")
```

See [docs/MOD_AUTHOR_GUIDE.md §12](docs/MOD_AUTHOR_GUIDE.md#12-sequencer--schedule--attack-info-mods-tier-1-formats) for the full reference.

### API reference

See [docs/api.md](docs/api.md).

## For mod authors

If you're writing mods with the Field-JSON v3.1 manifest format, start at:

- [docs/MOD_AUTHOR_GUIDE.md](docs/MOD_AUTHOR_GUIDE.md) — top-level entry, sections per mod type, common pitfalls
- [docs/FORMATS.md](docs/FORMATS.md) — every binary format dmm-parser understands
- [samples/](samples/) — five worked examples (data, texture, audio, custom item, mixed overhaul)

Mod-author CLIs:

```bash
python -m dmm_parser.tools.validate <field.json> [--assets DIR] [--json]
python -m dmm_parser.tools.pack     <field.json> [--out OUT.zip]
python -m dmm_parser.tools.inspect  <field.json | mod.zip>
python -m dmm_parser.tools.diff     <a> <b>
```

## Rust crate

Add to your workspace as a path or git dependency:

```toml
[dependencies]
dmm-parser = { path = "../dmm-parser" }
```

```rust
use dmm_parser::binary::{BinaryRead, BinaryWrite};
use dmm_parser::tables::buff_info::BuffInfo;
```

## Credits

- **`potter4208467`** — original `crimson-rs` parser; permitted relicensing for this fork (2026-04-27).
- **[LukeFZ](https://github.com/LukeFZ)** — [pycrimson](https://github.com/LukeFZ/pycrimson) (prior art on pabgb structure).
- **exodiaprivate-eng** — current maintenance, polymorphic family decoders, 100+ table additions.
