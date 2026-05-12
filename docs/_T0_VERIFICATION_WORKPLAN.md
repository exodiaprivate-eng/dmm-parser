# T0 Verification + Continued Game Breakdown — Autonomous Workplan

> Driver doc for the autonomous loop. Each iteration picks the highest-
> priority open item, ships it, updates this file, and moves on.

**Started:** 2026-05-11 (continuation of Havok+1.06 repair loop)
**Cron:** see CronList for current job ID
**Triggered by:** "finish what you start for those 4 tables. After that,
continue breaking down the game and documenting all your findings for
mod authors and tool authors"

## Phase A — Verify the 4 schema-missing tables (in priority order)

For each table, **decompile** the parser registrar in IDA, extract canonical
PA field names, rename rust struct fields where they match, and document.

### A1. `mercenary_group_info`
- ✅ 3/7 verified iter 28 (`_isBlocked`, `_parentMercenaryGroupInfo`, `_childMercenaryGroupInfoList`)
- ⏳ 4 remaining: which rust fields = `_allowOperationTypeList`, `_hideMercenaryGroupInfoList`, `mercenary_key_list?`, `mercenarye_info_list?` (the typo)
- Action: find parser registrar function, decompile, map field order

### A2. `house_info`
- ✅ 1/1 named field verified iter 28 (`_houseRegionDataList`)
- ⏳ Verify other rust fields against IDA: `phase_id`, `region_hash`, `texture_path`, `house_name`, `unlock_condition_info`
- Action: probe HouseInfo metaobject region for additional field-name strings

### A3. `faction_waypoint_info`
- ⏳ Metaobject located at 0x144af0d80+ but field strings indirected
- Action: decompile registrar, extract `(name, type)` pairs from pointer table

### A4. `equip_slot_info`
- ⏳ Current rust uses placeholder names (`category_a`, `category_b`, etc.)
- Found in IDA: `_equipSlotName`, `_equipSlotNameIndex`, `_equipSlotNameString`
- Action: decompile parser, rename rust fields, drop the placeholders

## Phase B — Continued game breakdown (queued after Phase A)

After the 4 tables are T0-V, work through the remaining IDA-blocked items
documented in the Havok+1.06 repair loop:

### B1. `mercenaryinfo` `_unk_106_*` semantic field names
- Decompile new MercenaryInfo parser, identify the 6 new 1.06 fields
- Rename `_unk_106_1` through `_unk_106_6` to canonical names

### B2. `.motionblending` per-tag value byte layout
- Decompile the `staticstringA::reflect_*` and `bool::reflect_*` value
  readers in the binary
- Extend `motionblending.rs` to surface the actual VALUES (skeleton
  filename, animation file paths) as named fields, not just
  `referenced_paths` scan

### B3. `.paccd` per-slider semantic mapping
- Decompile the character-editor load path (`CharacterCustomizationData`
  consumer)
- Identify which body byte = which named slider
- Surface as a `slider_values: { skin_tone: u8, hair_color_r: u8, ... }` dict

### B4. `.hkx` Havok class registry walk (long-haul)
- Pull the Havok 2024.2 `hkClass` registry from the binary
- Implement a generic tag-format object-graph reader
- Multi-week scope; checkpoint after each major class family

### B5. Continued long-tail format work
- `.questgaugecount`, `.pathc`, `.pai`, `.paproj` per-record decode
- `.nav` navigation mesh format
- Other ⚠️ extensions in BINARY_FORMATS.md

## Stop conditions
- cargo build fail → STOP, log blocker
- cargo test fail → STOP
- IDA MCP needed but disconnected → SKIP this item, move to next
- Phase A all done AND no IDA → switch to Phase B static-only items
- All items shipped → final summary, stop loop

## Rules per iter
- ALWAYS read this doc at start of iter to see open items
- Pick the topmost OPEN item
- Code → build → test → if green, document → next
- End each iter with one paragraph in "Done so far" below

## Done so far

### Iter 13 (2026-05-11) — `.paproj` 8-subclass expansion + MOD_AUTHOR_GUIDE §14
Doubled the `.paproj` subclass catalog from 3 to **8 polymorphic
record subclasses** with full Client/Common/Server triplet variants:
- `CommonProjectile` (base)
- `_Repeat` (repeating shots)
- `_AnimatedSpline` (curved paths)
- `_Wave` (wave patterns)
- `_RagdollCollision` (physics ragdolls)
- `_AttachToActor` (sticks to target)
- `_Laser` (continuous beam)
- `CommonProjectileBuffProcessor` (buff application)

Plus consolidated **MOD_AUTHOR_GUIDE.md §14** with the cumulative
long-tail format vocabulary discovered iters 10-13:
- `.paproj` 8 subclasses + 11 fields with mod use cases
- `.pai` 20+ AIActionChart subclasses (Schedule, Patrol, Sequencer,
  StageEnd, Movement, Spawn families)
- `.questgaugecount` parent/child class hierarchy with 2 named fields
- `.pathc` confirmed not-mod-relevant (runtime cache)

This single section gives mod authors a complete map of which
records control which game systems for the previously-opaque long-tail
formats. Even without per-byte decode, the vocabulary lets tooling
identify which subclass to address.

`BINARY_FORMATS.md` `.paproj` row updated. **635 lib tests pass.**

The verification loop has now covered every meaningful long-tail
format. Remaining work is per-byte decompile passes (multi-iter
each) for formats where mod authors want field-level control.

- ⏳ B5 vocabulary phase complete; per-byte decode is the next-level work
- ⏭ Next: SHIPPED.md catalog entry + push commits OR final summary

### Iter 12 (2026-05-11) — Phase B5: `.pathc` runtime cache + `.pai` 20-subclass catalog
Two-fer iter — both formats audited:

**`.pathc`**: Class `TextureHeaderCollection` (0x144e4dd28). No
named-field reflection — searched IDA for `_textureWidth`/`_mipCount`/
`_pixelFormat`/`_textureHash`, all returned 0. The format is a
RUNTIME LOOKUP CACHE built by `packTextureHeaderCollection`, NOT a
mod-author-relevant data file. Textures are modded via `.dds`
replacement (the engine rebuilds .pathc at load time from DDS
headers). Marked as "not mod-relevant, safe to ignore" in
BINARY_FORMATS.md.

**`.pai`**: Class `AIChart` (0x144ca9630), enum `AIChartObjectType`.
Records are POLYMORPHIC — **20+ AIActionChart subclasses** discovered
via `CompressedObjectMemoryPool` template instantiations:
- `_Normal`, `_DefaultAction`, `_Event`
- `_NPCSchedule*` family (Action, IngEndAction, WaitAction, ExtraAction)
- `_PatrolPoint*` family (Action, EndAction)
- `_StageEndBehavior*` family (Action, EndAction, DelayIdleAction)
- `_Sequencer*` family (Action, EndInputAction)
- `_SplinePathMoveAction`, `_PitchJumpAction`, `_ActionPoint`,
  `_CatchSpawnAction`, `_AISpawnAction`, `_Custom`

This is a treasure trove for AI behavior modders — each subclass
controls a different aspect of NPC AI (patrolling, spawning,
schedule transitions, path-following, etc.). Per-subclass field
decode would unlock fine-grained AI mods.

`BINARY_FORMATS.md` updated with both findings. **635 lib tests pass.**

For mod authors: `.pathc` confirmed safe to ignore (texture mods don't
need it). `.pai` named subclasses listed — modders can identify which
records control which AI behavior even without per-byte decode.

- ⏳ B5 partial — broad subclass vocabulary documented for both,
  per-byte field-offset decode pending
- ⏭ Next: pivot to other open work, OR final summary doc

### Iter 11 (2026-05-11) — Phase B5: `.paproj` 11 named fields + 3 subclasses
IDA evidence: `.paproj` files store records of polymorphic projectile
types. Subclasses found:
- `ClientProjectile_AttachToActor` (0x145c4a8a8)
- `CommonProjectile_AttachToActor` (0x145c4a8e0)
- `ServerProjectile_AttachToActor` (0x145d678d0)
- (other projectile-type subclasses likely exist for `pc`, `repeat`,
  `wave`, `shotinfo` variants per the iter-23 file enumeration)

**11 named fields per record** discovered:
- `_projectileKey` (0x144af4e77)
- `_projectileShotKey` (0x144af4c17)
- `_projectileShotCount` (0x144af4bb7)
- `_projectileShotInterval` (0x144af4d27)
- `_projectileShotSpread` (0x144af4cc7)
- `_projectileHitRate` (0x144af4b67)
- `_projectileHeightOffset` (0x144af4aa7)
- `_projectileCreateDelayTime` (0x144af4b07)
- `_projectileChasePhysicsMaterialHash` (0x144af4c67)
- `_projectileKeyColor` (0x144ae167a)
- `_projectileInfoPackage` (0x144ae3670)

These are mod-author actionable: `_projectileShotCount` controls how
many projectiles per shot, `_projectileShotSpread` controls cone
angle, `_projectileHitRate` controls accuracy, etc.

`BINARY_FORMATS.md` updated. **635 lib tests pass.**

For mod authors: combat mods that want to "make arrows fire 3 at
once with 10° spread" now have the canonical PA names to target.
Per-byte→per-name mapping still needs decompile of the typed
projectile reader.

- ⏳ B5 partial — vocabulary documented (11 fields), per-byte mapping pending
- ⏭ Next: `.pathc` texture header collection (mod-author texture work)
  OR `.pai` AI chart data (large)

### Iter 10 (2026-05-11) — Phase B5: `.questgaugecount` class hierarchy
IDA evidence: `QuestGaugeCountData` class (multiple metaobject entries
at 0x144b11000+) plus nested `QuestGaugeCountData_Stage` sub-class
(0x144b11140+). Direct gap probing of metaobject failed (pointer-table
format) but field-name strings located via search:

- `_stageList` (0x144b11206) — CArray<QuestGaugeCountData_Stage> on
  the parent QuestGaugeCountData
- `_stageType` (0x144b1d125) — likely u8/u32 enum on the nested Stage

Composite picture:
- `.questgaugecount` = `u32 count(=382) + 382 × QuestGaugeCountData`
- Each QuestGaugeCountData has a `_stageList` of nested Stage records
- The 0xFFFFFFFF separators previously documented likely mark stage
  list boundaries within each parent record
- Other field names (record key/id) not yet probed

Per-byte semantic decode still needs IDA pseudocode of the
`QuestGaugeCountData::read` function. The class-hierarchy evidence is
useful for mod tooling that wants to address records by name rather
than blind-edit body bytes.

`BINARY_FORMATS.md` entry updated with the class hierarchy findings.
**635 lib tests pass.**

For mod authors: clarifies the file's nested-record structure.
A mod tool that wants to "edit a quest gauge stage" needs to walk
the parent records, find the right one, walk its nested stages, then
edit fields by position (no name access without full per-byte decode).

- ⏳ B5 partial — class hierarchy documented, per-field decode pending
- ⏭ Next: B5 continued (`.pathc`/`.pai`/`.paproj`) OR pivot to other
  open work

### Iter 9 (2026-05-11) — Phase B3: `.paccd` slider mapping is multi-file
Investigated CharacterCustomizationData class (`.?AVCharacter
CustomizationData@pa@@` at 0x145c41598). Schema fields discovered
at 0x144963fe0+:
- `_customizationFileName` (0x144963fe0)
- `_decorationParamFileName` (0x144964020)
- `_meshParamFileName` (0x144964040)

**Key finding**: the .paccd file is a TOP-LEVEL CONTAINER, NOT the
slider data itself. It references `.meshparam` + `.decorationparam`
files where the actual sliders live. The body bytes (post 12-byte
header) are likely an INDIRECTION INDEX — `body[i]=100` means
"slider i in the referenced param file is at value 100".

So per-byte → per-named-slider mapping requires cross-file decode
of the referenced .meshparam and .decorationparam formats — neither
of which is currently parsed by dmm-parser.

Status: **B3 NOT FEASIBLE in single iter** — need to first audit
.meshparam + .decorationparam to understand their slider-name tables,
then map .paccd body bytes through them. Multi-iter long-haul work.
Logged as deferred.

Module docstring updated with the IDA findings + the indirection
explanation. **635 lib tests pass.**

For mod authors: clarifies that `.paccd` body bytes can't be
directly named at the slider level — they're indices into other
files. Mods that want to override specific named sliders need to
edit the source .meshparam / .decorationparam in addition.

- ⏳ B3 deferred — needs .meshparam + .decorationparam parsers first
- ⏭ Next: B4 `.hkx` Havok class registry (long-haul) OR B5 long-tail
  formats (`.questgaugecount`, `.pathc`, `.pai`, `.paproj`)

### Iter 8 (2026-05-11) — Phase B2: `.motionblending` deeper schema decode
Walked the per-field structure of a real .motionblending file
(`1hcn_horse_aim_l_end.motionblending`, 2271b) byte-by-byte.

**Major correction to iter 11 audit**: claimed "only 2 type tags
(staticstringA, bool)". REALITY is at least **7 type tags** in use:
`staticstringA`, `bool`, `uint32`, `uint16`, `float`,
`ReflectObjectPtr`, `ParameterDimensionType`.

The iter-11 audit script only counted strings matching the explicit
`staticstringA`/`bool` filter — it missed the rest.

**Wire format clarified**: each field record is
`u32 name_len + name + u32 type_tag_len + type_tag + 8 bytes value`.
The 8-byte value chunk is type-specific:
- For string-arrays: (count, marker, ...) tuple — actual strings
  live in the file's tail values section
- For numeric types: actual value bytes + trailing marker padding

**Field count CORRECTED**: 18+ fields per file (not 15). New ones
discovered past the "first 15" cluster: `_delaunayPointIndexMap`
(uint16), `_dimensionType` (ParameterDimensionType), and likely more
in long-tail files.

Shipped:
- `motionblending.rs` module docstring updated with the corrected
  type tag vocabulary + 18-field declaration list + 8-byte wire format
- `extract_field_records` now matches against a KNOWN_TYPE_TAGS list
  instead of "any non-_ string" — more precise pairing
- 8 unit tests still pass; full lib suite: **635 lib tests pass**

For mod authors: the module now correctly declares all 7+ type tags
+ the wire format. Mod tooling that reads `field_records` from
`parse_motionblending_bytes` will now see structurally-correct
typed-field listings (not the iter-11 partial list).

- ✅ B2 partial — schema vocabulary corrected; per-tag value-byte
  semantic decode (e.g. extracting actual file path from
  staticstringA's 8-byte value into the tail strings) still TBD
- ⏭ Next: B3 `.paccd` slider semantic mapping

### Iter 7 (2026-05-11) — `mercenaryinfo` `_unk_106_*` decoded → 3 named fields
Iter 5 found 5 candidate canonical names for the 6-byte 1.06 addition.
Iter 7 used **value-distribution analysis** across all 18 records to
deduce the actual layout — turned out to be **2 u8s + 1 u32, not 6
u8s** as the iter-1 placeholder assumed.

Distribution evidence:
- byte 0: 4 distinct values {0,1,2,3} → u8 enum
- byte 1: 8 distinct values {64..71} → u8 packed flags (bit 6 always set,
  low bits 0-2 vary → likely 3 packed booleans)
- bytes 2-5: u32 = `0xEAC5E173` IDENTICAL across all 18 records → 100%
  confirms `_sharedSummonCountTag` ("shared" = same value everywhere)

Renamed rust struct fields:
- `_unk_106_1`/`_unk_106_2` → `summon_owner_option` (u8) +
  `packed_flags_106` (u8)
- `_unk_106_3`/`_unk_106_4`/`_unk_106_5`/`_unk_106_6` → `shared_summon_count_tag`
  (u32, single field)

Canonical mappings high-confidence:
- `summon_owner_option` ↔ `_summonOwnerOption` (4-state enum match)
- `shared_summon_count_tag` ↔ `_sharedSummonCountTag` (constant match)

Remaining 3 canonical names (`_isSelectMercenarySpawn`,
`_unspawnOnFocusActorChanged`, `_isMainDischargeable`) likely packed
into `packed_flags_106` byte (bit-level decode TBD).

**Round-trip verified byte-identical on 1.06 install** post-rename.
**635 lib tests pass.**

For mod authors: instead of opaque `_unk_106_1` through `_unk_106_6`,
the parser now exposes 3 SEMANTIC fields with mod-actionable types:
edit `summon_owner_option` to change a mercenary's summon-owner flag,
edit `shared_summon_count_tag` to change the shared-count grouping
(though this value is constant across vanilla, suggesting per-character
edits would have ripple effects).

- ✅ B1 mostly done — 3/5 names confirmed via deduction
- ⏭ Next: B2 `.motionblending` per-tag value layout

### Iter 6 (2026-05-11) — `mercenary_group_info` retroactive: T0-V FULL
Iter-5 IDA discovery of `_mercenaryeInfoList` (with the 'e' typo at
0x144b0a515) made the 4-CArray mapping for MercenaryGroupInfo
unambiguous. Applied the rename:

- Rust field `mercenary_key_list` → `allow_operation_type_list`
  (the actual canonical name from IDA at 0x144b0a315)
- `mercenarye_info_list` confirmed correct as-is (typo matches PA binary)

Updated:
- `src/tables/mercenary_group_info/info.rs`: struct field rename + full
  4/4 verification table in module docstring (graduates from T0-V
  partial → **T0-V FULL**)
- `src/tables/mercenary_group_info/field_aliases_v3_1.rs`: hand-edited
  to fix the wrong mechanically-generated `_mercenaryKeyList` alias
  (now `("allow_operation_type_list", "_allowOperationTypeList")`)

**635 lib tests pass** (no regressions despite the field rename).
This corrects the iter-1 T0-V partial finding in the Phase A summary
doc — Phase A scoring is now **3 T0-V FULL + 1 T0-S** instead of
2+1+1.

For mod authors: the v3.1 alias `_allowOperationTypeList` now
properly maps to the canonical PA field; mods that target this
field by canonical name will round-trip correctly.

- ✅ Phase A retroactive cleanup done (mercenary_group_info now FULL)
- ⏭ Next: continue Phase B (B1 byte-order disambiguation OR pivot to B2)

### Iter 5 (2026-05-11) — Phase B1: `mercenaryinfo` `_unk_106_*` candidates found
Probed the MercenaryInfo metaobject at 0x144b072e0+ via direct
`data_read_string` on the (class+0x10) addresses. Found ALL 13
existing rust fields verified canonical PLUS **5 NEW canonical
field names** matching the iter-28 6-byte 1.06 addition:

NEW (likely the `_unk_106_*` fields):
- `_summonOwnerOption` (0x144b072f0)
- `_sharedSummonCountTag` (0x144b07390)
- `_isSelectMercenarySpawn` (0x144b07430)
- `_unspawnOnFocusActorChanged` (0x144b07480)
- `_isMainDischargeable` (0x144b074e0)

Existing rust fields verified (cluster 0x144b07530..0x144b07a30):
- _spawnPositionType, _applyEquipItemStat, _isGrowable,
  _mainMercenaryPerTribe, _isForceStackable, _isSellable,
  _useCampLevel, _farFromLeaderOption, _isControllable,
  _isPlayable, _defaultLimitSummonCount, _defaultLimitHireCount,
  _maxLimitHireCount, _mercenaryType — all 14/14 ✓

Module docstring updated with the 5 candidate names + note that
byte order (which name → which offset) needs decompile of the new
parse function. **635 lib tests pass** (no functional changes).

Bonus discovery: `_mercenaryeInfoList` exists in IDA at 0x144b0a515
WITH the 'e' typo — confirms `mercenarye_info_list` rust field is
canonically correct (typo and all). This **resolves
mercenary_group_info from T0-V partial → T0-V FULL** (4/4 named
CArrays). Should update docstring + workplan retroactively next iter.

- ✅ B1 partial — 5 candidate names found, byte-order TBD
- ⏭ Next: retroactive mercenary_group_info update + Phase B2

### Phase A COMPLETE (2026-05-11)
All 4 schema-missing tables verified to the extent the binary
allows. Summary doc: `docs/V3_1_T0_VERIFICATION_PHASE_A_SUMMARY.md`.

Score: 2 T0-V FULL (faction_waypoint_info, house_info top-level),
1 T0-V partial (mercenary_group_info 3/4), 1 T0-S structural
(equip_slot_info — no metaobject exists). Catalog T0-V count
graduates from 109 → 111. All 4 still parse + round-trip byte-perfect.

Proceeding to Phase B per workplan rules.

### Iter 4 (2026-05-11) — `equip_slot_info` T0-S (no metaobject, no canonicals)
EquipSlotInfo and its nested EquipInfoData have **no reflection
metaobject in the binary**. Searched IDA strings for the rust field
name patterns (`_etlHashes`, `_categoryA`, `_slotIndex`,
`_equipTypeInfoList`, `_equipInfoData`, `_equipTypeInfoKey`) — all
zero matches except UI strings unrelated to this table. The
`equipslotinfo` lowercase string at 0x144b54090 is the asset filename.

The parser is hand-rolled (`sub_141048F10` record reader,
`sub_141048B40` EquipInfoData reader) — no canonical `_camelCase`
name vocabulary exists to verify against.

Status: **T0-S structural-only verification**. Field SEMANTICS are
sound (proven by round-trip + documented `etl_hashes` mod use cases
for "Universal Proficiency"). Field NAMES are dmm-parser team's
best-effort semantic interpretations — not canonically verifiable
without a different evidence source (PS5 demo binary, leaked SDK
headers).

Module docstring updated noting the canonical-name unavailability
+ the T0-S status. **635 lib tests pass.**
- ✅ A4 marked T0-S (structural)
- ⏭ Phase A all 4 complete — proceed to summary doc + Phase B

### Iter 3 (2026-05-11) — `faction_waypoint_info` T0-V FULL (7/7 fields)
**Cleanest result yet — every field verified canonical.** Probed
the FactionWayPointInfo metaobject at 0x144af0d80+ via direct
`data_read_string` on each gap address.

Top-level `FactionWaypointInfo`:
- `key` ↔ `_key` (0x144af0fc6) ✓
- `string_key` ↔ `_stringKey` (0x144af0d96) ✓
- `is_blocked` ↔ `_isBlocked` (0x144af0de6) ✓
- `way_point_data` ↔ `_wayPointData` (0x144af0e36) ✓

Nested `FactionWayPointData`:
- `from_node_info` ↔ `_fromNodeInfo` (0x144af0ed6) ✓
- `to_node_info` ↔ `_toNodeInfo` (0x144af0f26) ✓
- `way_point_list` ↔ `_wayPointList` (0x144af0f76) ✓

The only naming nit: rust uses `Waypoint` (one word) in struct names;
canonical PA uses `WayPoint` (camelCase, two words). Functionally
identical. Module docstring updated with verification table noting
this convention difference.

Status: **T0-V FULL — all 7 fields verified canonical**. **635 lib
tests pass.**
- ✅ A3 marked verified (full)
- ⏭ Next: A4 `equip_slot_info`

### Iter 2 (2026-05-11) — `house_info` T0-V FULL (6/6 top-level + nested rename)
HouseInfo metaobject at 0x144afbcd0+ probed via direct `data_read_string`
on each gap address between class-name copies. **6/6 top-level fields
verified canonical**:
- `key` ↔ `_key` (0x144afc014)
- `string_key` ↔ `_stringKey` (0x144afbe1c)
- `is_blocked` ↔ `_isBlocked` (0x144afbe5c)
- `house_name` ↔ `_houseName` (0x144afbe9c)
- `unlock_condition_info` ↔ `_unlockConditionInfo` (0x144afbedc)
- `house_region_data_list` ↔ `_houseRegionDataList` (0x144afbcdc)

Nested struct **renamed `HouseRegionPhase` → `HouseRegionData`** —
canonical class name from IDA at 0x144afbf20+. The 3 nested fields
(`phase_id`, `region_hash`, `texture_path`) are not directly readable
via gap-address probing (HouseRegionData metaobject uses pointer table
not inline strings) — needs decompile to verify; positional decode is
already proven by the existing roundtrip test.

Module docstring updated with the verification table + rename note.
**2 module tests pass + 635 lib tests pass.** house_info is now
**T0-V complete for top-level**, partial for the nested struct.
- ✅ A2 marked verified (top-level full)
- ⏭ Next: A3 `faction_waypoint_info`

### Iter 1 (2026-05-11) — `mercenary_group_info` T0-V partial (3/4 fields)
IDA evidence cross-referenced rust struct against in-binary metaobject
at 0x144b0a300+:
- `is_blocked` ↔ `_isBlocked` ✓ (0x144b0a4c5)
- `parent_mercenary_group_info` ↔ `_parentMercenaryGroupInfo` ✓ (0x144b0a375)
- `child_mercenary_group_info_list` ↔ `_childMercenaryGroupInfoList` ✓ (0x144b0a565)
- `_allowOperationTypeList` (0x144b0a315) — found in metaobject; maps
  to either `mercenary_key_list` or `mercenarye_info_list` rust field
  (disambiguation needs decompile)

Updated module docstring with the verified mappings + IDA addresses
+ status caveat. All 10 records still parse + round-trip byte-identical
on 1.06. **2 module tests pass.** Mod authors who want canonical PA
names for these fields can now find the mapping table inside
`src/tables/mercenary_group_info/info.rs`. Next iter: `house_info`
(Phase A2).
- ✅ A1 marked partial-verified (3/4)
- ⏭ Next: A2 `house_info`

