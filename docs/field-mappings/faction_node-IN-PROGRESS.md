# faction_node_info + faction_node_spawn_info — Mac binary field names (in progress)

**Status:** Paused mid-IDA mapping. Pivoted to fix the UEP weapon-equip issue first.

> **Update (2026-05-22, Session 29):** the `faction_node_info` 1.07 *layout*
> drift is now FIXED — the table decodes 0-opaque and byte-roundtrips on the
> live dump (tail field reorder: `religion_max_block_day` before the trailing
> lists). See `docs/STATUS.md` Session 29 / `docs/449_TABLE_CATALOG.md`. This
> doc still tracks the remaining work of mapping the Mac canonical *field
> names* onto the (now correct) struct — a separate, lower-priority axis.

## Session 1.4-part-6 (2026-06-05) — empirical RE findings (1.10 fixture, 1141 records)

**Status: faction_node still 100% blob-fallback on 1.10. NOT fixed. Do NOT commit a
partial struct — it currently blob-roundtrips byte-safe; a struct that types some
records but writes others wrong would corrupt the table → CTD (strictly worse).**

> ⚠️ The current `info.rs` tail (`big_composite_slots` 13-slot, `adjacency_list`,
> `de690_data`, `final_list_*`) does NOT match the canonical 31-field schema below
> (no such fields exist in it). This looks like a **post-Session-29 divergent RE that
> replaced** the Session-29 "religion_max_block_day reorder" tail. `git log -p
> src/tables/faction_node_info/info.rs` to find when it diverged — the Session-29
> version may have been closer. Verify before trusting the in-tree tail.

### What's confirmed
- **Prefix fields 1–18 are correct ONLY for the 782 records with empty
  apply_skill/resource/waypoint/schedule lists.** On the **359 records with populated
  lists the prefix over-consumes** → the `FactionSchedule` (31-wire-field) and likely
  `ApplySkillData`/`ResourceItemData`/`WayPointDeprData` **element structs are wrong**.
  (`dmm_probes/faction_tailfit.py`: 782 clean-prefix, 359 prefix-fail.)
- Fields 1–12 solid: world position aligns exactly on populated nodes (GreymaneCamp
  `(-10498, 609, -4413)`).
- **GreymaneCamp's named sub-records** (`..._Combat_01`, `..._ExpandCamp_Lv2`) live in the
  **TAIL (fields 19–31), not the schedule list** (its schedule count=0).

### Tail layout decoded so far (canonical order = catalog's 31 fields)
Cross-aligning small empty-list nodes (Temporary_Camp, Calphade Artilery I–IV, Senia):
- `+0`  u32 (UNNAMED — error-string name list misses it; always 0 in samples)
- `+4`  u8  `_factionType`            (Temporary=1, Artilery=2, Gimmick-node=2)
- `+5`  cstring `_subInnerTypeString` ("", "Artillery", "Gimmick")
- ~5 unnamed bytes, then u32 `_workerCount` (Artilery=200, Senia=14)
- `_knockDownCondition` (u32), `_bitMapColorKey` (u32?), `_researchDataList` (carray),
  `_factionEventDataList` (carray) — element types NOT yet pinned
- `_useCustomWayPointforDev` (u8)
- `_observeData` — float cluster: observePosition vec3 + startYaw + yawMin + yawMax +
  observeCameraPresetHash (28B element). Temporary_Camp: pos(-10665,633,-3963)
  yaw 1.0/-170/-130 camHash 0xfe89127c. Artilery yaw range -180/180. Likely
  `carray<ObserveData>` ("array_or_complex").
- `_religionMaxBlockDay` (u32), `_religionBlockCostList` (carray),
  `_religionEffectRegionInfoList` (carray u16), `_religionSubLevelInfo` (u32) — at end.

### Fastest path to finish (needs a dedicated session)
1. Best: load the **Mac binary** and disasm the sequential deserializer `sub_101886690`
   (err-label pattern gives exact field order+types). No Mac IDB on this disk currently.
2. Else empirical: (a) fix `FactionSchedule` element so all 359 populated-prefix records
   reconcile, AND (b) finish the tail so all 782 tail-bytes reconcile. Require
   `cargo test faction_node_info::roundtrip` = 1141/1141 BEFORE committing.
   Harnesses: `dmm_probes/faction_full.py`, `faction_tailfit.py`.

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
