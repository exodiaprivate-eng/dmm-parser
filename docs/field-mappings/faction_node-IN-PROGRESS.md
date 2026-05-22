# faction_node_info + faction_node_spawn_info — Mac binary field names (in progress)

**Status:** Paused mid-IDA mapping. Pivoted to fix the UEP weapon-equip issue first.

> **Update (2026-05-22, Session 29):** the `faction_node_info` 1.07 *layout*
> drift is now FIXED — the table decodes 0-opaque and byte-roundtrips on the
> live dump (tail field reorder: `religion_max_block_day` before the trailing
> lists). See `docs/STATUS.md` Session 29 / `docs/449_TABLE_CATALOG.md`. This
> doc still tracks the remaining work of mapping the Mac canonical *field
> names* onto the (now correct) struct — a separate, lower-priority axis.

## Pickup points

- Field-name lists already extracted from Mac binary error strings (see below).
- Deserializer functions identified — still need to disassemble and map struct offsets:
  - `FactionNodeInfo` deserializer: `sub_101886690` @ `0x101886740` (size `0x5d8`)
  - `FactionNodeSpawnInfo` deserializer: `sub_1018A9B6C` @ `0x1018a9c1c` (size `0x1c0`)

To resume: disassemble each deserializer, find every `ADD X1, X20, #N ; MOV X0,X19 ; BL reader ; TBZ W0,#0,err_label` pattern, then match the order of error-label string addresses to the order of reads. Use the same script as `examples/uep_runtime_package_bridge.rs` field mapping pattern.

## FactionNodeInfo — 32 top-level fields (Mac order, from error-string addresses)

| # | Mac binary field |
|---|---|
| 1 | `_key` |
| 2 | `_stringKey` |
| 3 | `_isBlocked` |
| 4 | `_knowledgeInfo` |
| 5 | `_skillTreeInfo` |
| 6 | `_connectResearchNodeInfo` |
| 7 | `_storeInfo` |
| 8 | `_royalSupplyInfo` |
| 9 | `_memo` |
| 10 | `_childFactionInfoList` |
| 11 | `_nodeLineMainFactionInfoList` |
| 12 | `_worldPosition` |
| 13 | `_nodeRadius` |
| 14 | `_applySkillDataList` |
| 15 | `_resourceItemList` |
| 16 | `_revivalStageInfoList` |
| 17 | `_wayPointDataList_deprecated` |
| 18 | `_factionScheduleInfoList` |
| 19 | `_factionType` |
| 20 | `_subInnerTypeString` |
| 21 | `_workerCount` |
| 22 | `_knockDownCondition` |
| 23 | `_bitMapColorKey` |
| 24 | `_researchDataList` |
| 25 | `_factionEventDataList` |
| 26 | `_useCustomWayPointforDev` |
| 27 | `_observeData` (struct `FactionNodeInfo_ObserveData`) |
| 28 | `_fieldInfo` |
| 29 | `_religionMaxBlockDay` |
| 30 | `_religionBlockCostList` |
| 31 | `_religionEffectRegionInfoList` |
| 32 | `_religionSubLevelInfo` |

### Nested: `FactionNodeInfo_ObserveData`

- `_observePosition`
- `_startYaw`
- `_yawMin`
- `_yawMax`
- `_observeCameraPresetHash`

## FactionNodeSpawnInfo — 6 top-level fields (Mac order)

| # | Mac binary field |
|---|---|
| 1 | `_key` |
| 2 | `_stringKey` |
| 3 | `_isBlocked` |
| 4 | `_factionNodeInfo` |
| 5 | `_boundaryBox` |
| 6 | `_patrolAISplineDataList` |

## TODO

1. Disassemble both deserializers, capture per-field `(struct_offset, reader_func, err_label_addr)` triples.
2. Match err_label_addr (`aFactionnodeinfoX` ADRL targets) to the canonical Mac name above.
3. For each Mac field, identify the dmm-parser field at that struct/wire position.
4. Update `field_aliases_v3_1.rs` for both tables with canonical `(snake, _camelCase)` pairs.
5. Generate the final mapping markdown.

Field-name extraction is done — only the offset-mapping/disasm pass remains.
