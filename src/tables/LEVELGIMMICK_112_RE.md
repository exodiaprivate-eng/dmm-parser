# levelgimmicksceneobjectinfo (level_gimmick_scene_object_info) — 1.12 RE

## ✅ DONE 2026-06-24 — 165/165 typed, byte-exact roundtrip on live 1.12.02.
Fix vs first draft: `sub_100D392D8` distance fields are u32 (not u64); `_contentsPhaseInfoForMovePoint` is u16 wire. All 500 Shop placements now typed (named fields: level_name=area key, gimmick_alias_name=scene ref, world_transform.vec_a=position). custom_npc/level_place.py byte-surgery still works; can retarget to named fields.


**Why:** THE additive-spawn lever for town NPCs (record 1000011 "Shop" = ~500 placements).
Currently the parser returns all 165 records as opaque `_body_b64`. Goal: full typed parser,
165/165 typed, byte-exact roundtrip — so placements are edited as named fields, not byte
surgery. RE'd from Mac binary CrimsonDesert_Steam 1.12.02 (ida-pro-mcp).

## Record reader = `sub_101F76270` (LevelGimmickSceneObjectInfo, the 165 records)
Field order (from Korean error strings, sequential), with mem offset + reader:
1. `_key` (guard sub_101608E10)            — u32
2. `_stringKey` @8  sub_100D39448          — CString
3. `_isBlocked` @16 sub_100D391B8          — u8
4. `_levelName` @24 sub_100D39448          — CString
5. **`_levelGimmickSceneObjectDataList`** @32 sub_101FB3D10 — **CArray<LevelGimmickSceneObjectData>** (the placements)
6. `_mapIconTextureInfo` @48 (UIMapTextureKey lookup) — u32 wire→u16 mem
7. `_discoverNearFog` @50 sub_100D391B8    — u8
8. `_fogMapIconTextureInfo` @52 sub_101F766DC — (u16? 2-byte mem)
9. `_fogDistance` @56 sub_100D392D8        — u64
10. `_overAbyssIconTextureInfo` @60 sub_101F766DC
11. `_overAbyssFogMapIconTextureInfo` @62 sub_101F766DC
12. `_overAbyssFogDistance` @64 sub_100D392D8 — u64
13. `_discoverDistance` @68 sub_100D392D8   — u64
14. `_showIconConditionType` @72 sub_1020DD2C4
15. `_useTeleport` @73 sub_100D391B8        — u8
16. `_useGuideEffect` @74 u8
17. `_isSubInnerGimmick` @75 u8
18. `_checkGameLevelLoadState` @76 u8
19. `_useGimmickKnowledgeForUI` @77 u8
20. `_checkBlockCondition` @78 u8
21. `_isRestoreStockTargetItem` @79 u8
22. `_completedDiscoverMapIconTextureInfo` @80 sub_101F766DC
23. `_overAbyssCompletedDiscoverMapIconTextureInfo` @82 sub_101F766DC
24. `_guideEffectSocketName` @84 sub_100D395EC — CString(binarystring)
25. `_oreVeinIndex` @88 sub_100D39278       — u32
26. `_contentsPhaseInfoForMovePoint` @92 sub_101F767D4
27. `_discoverType` @96 sub_100D39278       — u32
28. `_ignoreSameGimmickDiscoverDistance` @100 sub_100D392D8 — u64
29. `_discoverGimmickStateHash` @104 sub_100D39278 — u32
30. `_isEmptyInfo` @108 sub_100D391B8       — u8

## Element reader = `sub_101F75C18` (LevelGimmickSceneObjectData — one placement)
Wire reads in order (mem offset + reader → WIRE type):
1. `_levelGimmicKSceneObjectInfo` @0  (LevelGimmickSceneObjectInfoKey) — **u32**
2. `_gimmickInfo` @4  (GimmickInfoKey)          — u32 wire→u16 mem
3. `_itemInfo` @6  (ItemKey)                    — u32 wire→u16 mem
4. `_parentSpawningPoolAutoSpawnInfo` @8 (SpawningPoolAutoSpawnInfoKey) — u32
5. `_levelName` @16 sub_100D39448               — **CString** (= the placement/area key, e.g. `Shop_Hernand_0001_Phase00_05_sub_1_0`)
6. `_relatedGameLevelInfo` @24 (GameLevelKey)   — u32
7. `_levelNameControlledByGameLevelInfo` @28 sub_1013633A4 (StringInfoKey) — u32 wire→u16 mem
8. `_sceneObjectUuid` @32 sub_101A726DC         — **16 bytes (UUID)**
9. `_rootGimmickSceneObjectUuid` @48 sub_101A726DC — **16 bytes (UUID)**
10. `_spawnReason` @64 sub_100D39278            — u32
11. `_gimmickAliasName` @72 sub_100D39448       — **CString** (= the scene/shop ref, e.g. `Shop_Butcher_Hernand`)
12. `_worldTransform` @80 sub_100D39CD4         — **40 bytes (transform — position is in here)**
13. `_teleportOffsetTransform` @120 sub_100D39CD4 — 40 bytes
14. `_guideEffectOffsetPosition` @160 sub_100D39DA0 — 12 bytes ([f32;3])
15. `_fogRevealBitmapColorR` @172 sub_100D391F8  — (type TBD)
16. `_linkedCompleteGimmickList` @176 sub_101FB3AA8 — CArray<?>

## ✅ ALL TYPES DECODED (2026-06-24) — ready to implement
Type-reader sizes (from `(a1,a2,N)` byte-count + lookup patterns):
- `sub_101A726DC` = **[u8;16]** (UUID, raw 16 bytes)
- `sub_100D39CD4` = **40-byte transform** — wire read order: `[f32;3]` + `u32`×4 + `[f32;3]`
  (model as `[u8;40]` or `WorldTransform{ a:[f32;3], raw:[u32;4], b:[f32;3] }`, 40 wire bytes)
- `sub_100D39DA0` = **[f32;3]** (Vec3, 12B) · `sub_100D39DA0` also used for guideEffectOffset
- `sub_100D391F8` = **u8** · `sub_1020DD2C4` = u8 · `sub_100D391B8` = u8 · `sub_100D39278` = u32
  · `sub_100D392D8` = u64 · `sub_100D39448` = CString · `sub_100D395EC` = CString(binarystring)
- lookups (u32 wire → u16/u32 mem): `sub_101F766DC`=UIMapTextureKey · `sub_101F767D4`=ContentsPhaseKey
  · `sub_1013633A4`=StringInfoKey · element key readers (GimmickInfo/Item/SpawningPool/GameLevel/
  LevelGimmickSceneObjectInfo) all u32 wire.
- `sub_101FB3AA8` = **CArray<LinkedCompleteGimmick>** where element (`sub_101FB3AA8` body) =
  `{ sceneObjectUuid:[u8;16], completeGimmickIndex:u32 }` (wire 20B/elem, mem 24B).
- `sub_101FB3D10` = **CArray<LevelGimmickSceneObjectData>** (the placements; mem stride 192B).

### Final ELEMENT wire (LevelGimmickSceneObjectData) — implement exactly:
`u32 _levelGimmicKSceneObjectInfo` · `u32 _gimmickInfo` · `u32 _itemInfo` ·
`u32 _parentSpawningPoolAutoSpawnInfo` · `CString _levelName` · `u32 _relatedGameLevelInfo` ·
`u32 _levelNameControlledByGameLevelInfo` · `[u8;16] _sceneObjectUuid` ·
`[u8;16] _rootGimmickSceneObjectUuid` · `u32 _spawnReason` · `CString _gimmickAliasName` ·
`[u8;40] _worldTransform` · `[u8;40] _teleportOffsetTransform` · `[f32;3] _guideEffectOffsetPosition` ·
`u8 _fogRevealBitmapColorR` · `CArray<LinkedCompleteGimmick> _linkedCompleteGimmickList`

### Final RECORD wire (LevelGimmickSceneObjectInfo): per the field list above —
key/stringKey/isBlocked/levelName(CString) · **CArray<element> _levelGimmickSceneObjectDataList** ·
then the u32/u64/u8/lookup scalars (mapIcon … _isEmptyInfo). Use `pabgh_typed_blob_table!`
with `tail: tail_blob` as a safety net, but target 165/165 fully typed (tail empty).

## TODO (B-1 continuation)
- [x] reader funcs found · [x] record fields · [x] element fields · [x] all type sizes
- [ ] implement `dmm-parser-src/src/tables/level_gimmick_scene_object_info/{info.rs,mod.rs}`,
      register in dispatch.rs + tables/mod.rs, rebuild, prove 165/165 typed + byte-exact.
- [ ] retarget `custom_npc/level_place.py` to named fields.
- Decompile + type: sub_101A726DC (UUID 16B), sub_100D39CD4 (transform 40B — confirm
  pos[3]+rot+scale layout), sub_100D39DA0 (Vec3), sub_100D391F8, sub_101FB3AA8 (list elem),
  sub_101FB3D10 (data-list = CArray of the element), sub_101F766DC, sub_101F767D4,
  sub_1020DD2C4, sub_100D395EC, the UIMapTextureKey lookup.
- Also RE the OTHER record shapes — 165 recs aren't all "Shop"; rec 0 "Faction" / bells.
  Actually all 165 share the SAME LevelGimmickSceneObjectInfo schema (above); only the
  _levelGimmickSceneObjectDataList contents differ. Confirm via roundtrip.
- Implement in dmm-parser-src/src/tables/level_gimmick_scene_object_info/, rebuild, prove
  165/165 typed + byte-exact on live 1.12.02. Then retarget `custom_npc/level_place.py` to
  named fields.
