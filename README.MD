# dmm-parser

Pabgb / pabgh / paz / pamt / papgt parser for the DMM mod loader.

Forked from `crimson-rs` by `potter4208467` (Discord) with explicit permission.
Distributed under a source-available license restricted to DMM-affiliated use — see [LICENSE](LICENSE).

## Status

- 162 / 162 tests pass (`cargo test --release`)
- 121 pabgb tables round-trip byte-perfect
- 67 fully field-decoded (every field individually addressable, including BuffInfo + SkillInfo with all 120 BuffData variants)
- 54 byte-perfect with `Vec<u8>` blob-tail for polymorphic regions still being decoded

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

### API reference

See [docs/api.md](docs/api.md).

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
