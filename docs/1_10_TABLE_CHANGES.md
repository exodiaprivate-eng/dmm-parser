# Crimson Desert 1.10 — Table Format Changes (vs 1.09)

Game updated **1.09 → 1.10** (2026-06-04). `meta/0.paver` version field 9 → 10.
The **container format is unchanged** (PAMT/PATHC/PAPGT/paver/pabgh all parse as before;
PATHC `rec_size` still 148). Only individual **table record layouts** changed.

## Validation harness
- 1.10 fixtures: `C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-6-4` (all 126 `.pabgb`+`.pabgh`).
- `src/testenv.rs` FALLBACK_DIRS has this dir at the top → `cargo test --lib` validates byte-exact.
- Next update: re-extract to a new dated dir, add to top of FALLBACK_DIRS, `cargo test`, fix red tables.

## Changed tables

### Fixed (cargo-validated byte-exact roundtrip)
| Table | File | 1.10 change |
|---|---|---|
| `game_version_data_info` | gameversiondatainfo | `RestoreItemData` 16→17B: `u8` inserted between `raw_data:u64` and `target_key:u32` |
| `game_advice_info` | gameadviceinfo | +1 `u32` in fixed tail (a `*_string_info` name-hash; 0xEAC5E173 sentinel when empty) |
| `quest_gauge_info` | questgaugeinfo | +5B after `percent`: `u32` + `u8` |
| `vehicle_info` | vehicleinfo | +1 `u32` in fixed tail |
| `terrain_region_auto_spawn_info` | terrainregionautospawninfo | `PoolSplineEntry` +4B: `block_extra:u32` (float 1.0) after `block:[u32;4]` — in `src/binary/variants/auto_spawn_entry.rs` |
| `spawning_pool_auto_spawn_info` | spawningpoolautospawninfo | same `PoolSplineEntry` fix (shared struct) |
| `item_info` (iteminfo) | iteminfo | a `u32` (`money_icon_path`) **removed** from the item record (after `item_icon` block / before `map_icon`); 6325/6325 records align, `use_map_icon_alert`∈{0,1}, `item_type` clean enum |
| `mercenary_info` | mercenaryinfo | tail RESTRUCTURED: `shared_summon_count_tag`(u32) + `hired_skill_info_list`(CArray) + `set_new_mercenary_is_main`(u8) removed; post-block tail now fixed `u8 + u32 + u32`. All 18 records byte-exact. `HiredSkillData` struct removed (now unused). |

### Follow-up (not yet byte-exact; SAFE — dispatch hard-errors → DMM leaves table as vanilla, no corruption)
| Table | Symptom | Lead |
|---|---|---|
| `character_info` | ~4B drift remains at offset 285 | agent added `skeleton_variation_name:u32` after `lookup_25` (helped); one more field/position between +213 and +285. `pabgh_typed_blob_table!` macro. |
| `skill_tree_info` | `SkillTreeStatNode` ~16B larger before `ui_position` | trace rec 0 stat_nodes (abs off 87); identify new UI float/id fields |
| `reserve_slot_info` | rec 0 +4B, rec 16 (populated lists) `special` count garbage | merc list (5×u8) parses; `enable_special_name_hash_list` element/boundary shifted in 1.10; +4 not a simple insert (full sweep found none). Trace rec 16 special-list region. |

### Unchanged but blob-by-design (roundtrip-safe, partial typed coverage — NOT 1.10 regressions)
`effect_info`, `knowledge_info`, `mission_info`, `stage_info`, `item_use_info`, `npc_info`,
`faction_node_info`, `game_play_trigger_info`, `global_stage_sequencer_info`, `game_level_info`,
`special_mode_info`, `mini_game_data_info`, `faction_spawn_data_info`, `sequencer_spawn_info`,
`buff_info`, `quest_info`, `skill_info` — dispatch carries unparsed records as `_blob_b64`, so they
round-trip byte-exact even where typed coverage is incomplete.

## Tools added (C:\Users\justi\Desktop\dmm_probes)
- `recdump.py <base> <idx> <count>` — dump a record with exact pabgh boundaries + hex.
- `wirewalk.py` — define a field spec; `walk()` reconciles all records to their pabgh length;
  `trace()` walks one record field-by-field to find the divergence offset.
