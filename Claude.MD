# dmm-parser — Crimson Desert Archive Toolkit

Rust library with Python bindings (via PyO3/maturin) for reading, writing, and modding Crimson Desert game archives.

# Development
- Build and install: `maturin develop`
- Run tests: `cargo test`
- Lint: `cargo clippy` (must pass with zero warnings)

# Documentation
- `docs/archive-format.md` — Binary format specs (PAPGT, PAMT, PAZ, trie), mod loading overlay approach
- `docs/api.md` — Full Python API reference (enums, pack_mod, PAPGT/PAMT/PAZ, ItemInfo)
- Update docs when adding new features or changing binary format understanding

# Rules
- Ask before adding new dependencies to Cargo.toml
- Binary format must roundtrip (read and write back identically)
- Custom types for binary serialization: CString (u32 len + [u8; len], no null terminator), CBytes (u32 len + raw bytes, no UTF-8 check), COptional<T> (u8 flag + T if nonzero), CArray<T> (u32 count + count × T), Variant (u8 tag + match on tag)

# Binary Exploration
- Use `plcli` + `.hexpat` pattern files for binary format exploration, NOT Python scripts
- Write partial hexpat patterns, test with `plcli run -i <file> -p <pattern.hexpat> -v -d`
- Iterate: write struct, run plcli, check output, refine
- Pattern files go in `references/`

# Reference Files
- `/mnt/e/OpensourceGame/CrimsonDesert/Crimson Browser/iteminfo_decompressed.pabgb` (5993 items)
- `/mnt/e/OpensourceGame/CrimsonDesert/Crimson Browser/Original/0.pamt` — Original PAMT
- `/mnt/e/OpensourceGame/CrimsonDesert/Crimson Browser/Original/0.papgt` — Original PAPGT
- `/mnt/f/Program/Steam/steamapps/common/Crimson Desert` — Game installation folder

# Rust Module Structure
```
src/
├── lib.rs              # Crate root
├── python.rs           # PyO3 bindings (parse_table / serialize_table / pack_mod / etc.)
├── python_traits.rs    # ToPyValue / WritePyValue trait impls for all wire types
├── json_traits.rs      # ToJsonValue / WriteJsonValue trait impls for all wire types
├── binary/
│   ├── mod.rs          # BinaryRead/Write/ReadTracked traits; py_binary_struct! + binary_struct! macros
│   ├── primitives.rs   # u8/u16/u32/u64/i8/i64/f32 BinaryRead/Write/ReadTracked impls
│   ├── types.rs        # CString, CBytes, CArray, COptional, LocalizableString
│   ├── arrays.rs       # [u8;N], [f32;2..4], [u32;2..4] BinaryRead/Write/ReadTracked impls
│   ├── variant.rs      # pabgh_typed_blob_table! macro, entry_ranges, load_pabgh_offsets
│   ├── pamt.rs         # PAMT parse/write
│   ├── papgt.rs        # PAPGT parse/write + add_entry (upsert, front-insert)
│   ├── pabgh.rs        # PABGH index parse (entry offset table for pabgh-bounded tables)
│   ├── paz.rs          # PackGroupBuilder, compression, PAZ creation
│   ├── paloc.rs        # PALOC (localization) parse/write
│   ├── trie.rs         # Trie buffer read + build (radix-compressed)
│   ├── optional_game_condition.rs  # OptionalGameCondition wrapper type
│   └── variants/       # Polymorphic family decoders (Decoded|Raw enum pattern)
│       ├── game_condition.rs               # GameCondition (405-variant ConditionData tree)
│       ├── condition_data.rs               # ConditionData per-tag body readers
│       ├── buff_data.rs                    # BuffData (120-variant)
│       ├── effect_data.rs                  # EffectData family
│       ├── filter_condition.rs             # FilterCondition family (QuestInfo)
│       ├── trigger_gameplay_event_handler_data.rs  # TGPEHD (GimmickInfo field 17)
│       ├── game_event_handler_data.rs      # GameEventHandlerData
│       ├── global_game_event_execute_data.rs
│       ├── sequencer_stage_chart_desc.rs   # SequencerStageChartDesc (26 wire fields)
│       ├── gimmick_interaction_override.rs
│       ├── auto_spawn_entry.rs
│       ├── drop_target.rs
│       ├── mini_game_data.rs
│       └── ...                             # (branch_condition_data, condition_pair, etc.)
├── crypto/
│   ├── mod.rs
│   ├── checksum.rs     # Jenkins hashlittle2
│   └── chacha20.rs     # ChaCha20 encrypt/decrypt
├── item_info/          # iteminfo.pabgb parser (legacy; predates generic table system)
│   ├── mod.rs
│   ├── item.rs
│   ├── keys.rs
│   └── structs.rs
└── tables/             # 122 typed table parsers (one subdirectory per table)
    ├── mod.rs
    ├── blob_runtime.rs # Generic blob-table runtime helpers
    ├── gimmick_info/   # Example: complex table with post-body + family decoders
    ├── condition_info/ # Example: GameCondition Decoded|Raw wrapper
    ├── skill_info/
    └── ...             # (one dir per table, each with info.rs + mod.rs)
```

## RTFM — Indexed Knowledge Base

This project has been indexed with RTFM.

For any **exploratory search** (finding which files/modules/classes are relevant
to a topic), use `rtfm_search` instead of Glob, find, ls, or broad Grep.
Then use `rtfm_expand` to read easily most relevant files/sections.
