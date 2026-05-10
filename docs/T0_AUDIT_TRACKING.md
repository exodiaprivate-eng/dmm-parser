# T0 Audit Tracking — IDA-verified C++ name parity per table

**Started:** 2026-05-09 (Session 26)
**Goal:** For every on-disk table, verify each Rust struct field name
matches its real C++ identifier per the Mac-binary IDA symbols.
Where a name was a descriptive translation rather than the canonical
C++ identifier, add a `FIELD_ALIASES_V3` entry so existing v3 mods
keep working AND the canonical T0 name becomes available for v3.1
consumers.

**v3 names are NEVER overwritten.** They become the legacy alias on
input + the v3-shape output projection. The Rust struct field gets
the canonical C++ name; v3 round-trips through the alias mechanism.

## Audit procedure per table

1. Open the table's Rust struct: `src/tables/<name>/info.rs`.
2. List every field name + type.
3. Look up the C++ class in IDA — check for `_ZTSN2pa<class_name>E`
   typeinfo string (e.g. `_ZTSN2pa14GameLevelInfoE`).
4. Enumerate `__ZNK2pa<class>get_<field>Ev` getter symbols.
   - If symbols exist: decompile each (single `return this+offset`),
     confirm field name matches Rust struct field. Mismatches go
     into `FIELD_ALIASES_V3` per Session 24's mechanism.
   - If symbols don't exist (template-aggregated reflection): mark
     the table "T0 unverified — IDA symbols absent, current names
     presumed canonical." This is the same wall hit by .paatt
     deserialiser hunt (Sessions 20, 23).
5. Update this document's row with the result.
6. Commit + push.

## Status legend

| Glyph | Meaning |
|---|---|
| ⏳ | Pending — not yet audited |
| 🔍 | In progress — partial audit |
| ✅ | T0 verified — every field name matches C++ exactly, no aliases needed |
| 🟡 | T0 with aliases — one or more renames shipped, v3 alias added (or schema-verified via NattKh's `pabgb_complete_schema.json`, see Session 28) |
| ⚠️ | T0 unverified — IDA getter symbols absent for this class **AND** not present in NattKh schema. Mechanical-fallback v3.1 aliases ship but canonical names are not independently verified. |

## 2026-05-10 Session 28 — bulk per-table flip ⏳ → 🟡 / ⚠️

Per-table tracking rows below were flipped from ⏳ pending to either:
- **🟡** (109 tables) — present in NattKh's `pabgb_complete_schema.json`,
  every shipped v3.1 alias name independently verified against the
  canonical Pearl Abyss identifier list extracted from Korean error
  strings in CrimsonDesert.exe.
- **⚠️** (13 tables) — not in the NattKh schema. 4 named *Info tables
  (`equip_slot_info`, `faction_waypoint_info`, `house_info`,
  `mercenary_group_info`) plus 9 zero-field tables fall back to
  mechanical translation only.

The strict-T0 wall in the Session 26-27 sections still applies:
**these statuses are schema-verified, not IDA-getter-symbol-verified.**
The two are different kinds of evidence with similar strength — both
trace back to the same underlying canonical names, just via different
paths (Korean error strings vs metaobject reflection symbols).

See `docs/V3_1_SCHEMA_VERIFICATION.md` for per-table verification detail
and `docs/V3_1_DECODER_GAPS.md` for the 584 schema fields dmm-parser's
struct definitions don't currently expose.

## ⚠️ Structural blocker discovered (2026-05-09, Session 26 iter 1)

Bulk IDA probe confirmed: **none of the 122 `pa::*Info` table classes
expose individual `__ZNK*get_<field>Ev` getter symbols** in the Mac
binary. Only the engine-level data-descriptor classes used INSIDE
table records do (e.g. `pa::AttackInfoDataDesc`, `pa::SplineDecalComponent`,
`pa::EmitterCurveData` — these were cracked in Sessions 19/22).

`*Info` tables go through `pa::StaticInfoWrapper<Key, Info, Manager, t>`
templated reflection machinery. Property registration happens via
`PascriptComponentProperty` template instantiations that aggregate
across many fields/types — there's no per-field setter/getter symbol
to enumerate.

**Sample probe (all returned zero getter symbols):**
- `pa::ActionPointInfo` → 0 getter symbols
- `pa::ActionRestrictionOrderInfo` → 0 getter symbols
- `pa::AIDialogStringInfo` → 0 getter symbols
- `pa::SkillInfo` → 0 getter symbols
- `pa::ItemInfo` → 0 getter symbols

Same wall as the .paatt deserialiser hunt (Session 20) and
SplineDecalComponent enumeration (Session 23). Same root cause:
Pearl Abyss's metaobject runtime aggregates property registration
into template instantiations rather than per-field symbols.

### What this means for T0

**Strict T0** (every field name == verbatim C++ identifier per IDA):
*structurally impossible* for these 122 classes without runtime
introspection. The verification data simply doesn't exist statically.

**Pragmatic T0** (every field has a stable, descriptive identifier;
no `_unkXXXX` placeholders): **already achieved** in Session 25's
bulk promotion. Names sourced during the long Tier 1 promotion arc
(finished 2026-04-30) from:
- WIN-IDA Hex-Rays decompile of the parse function in CrimsonDesert.exe
- Mac-IDA `__cstring` declaration-order matching against the Mac binary
- Empirical default-value distribution analysis

The names ARE canonical for mod-author purposes — stable, semantic,
round-trip byte-perfect. They are NOT byte-for-byte equal to the
internal C++ identifiers Pearl Abyss used — that information was
never exposed in the binary's symbol table.

### Where IDA-verified renames CAN still happen

Data-descriptor classes embedded INSIDE table records DO expose
per-field getters and ARE in scope for verbatim renames. Tracked
separately per-target in BINARY_FORMATS.md#paatt-basedata-field-layout and STATUS.md
session entries. Targets include `AttackInfoDataDesc`,
`AttackCommonDataDesc`, `AttackHitDataDesc`, `SplineDecalComponent`
(partial), `EmitterCurveData`, and a few dozen other nested blobs.

The rename work for these polishes the v3.1 embedded-blob surface
but doesn't change any TABLE's tier classification.

### ~~Decision~~ — RESCINDED 2026-05-09 Session 27

The Mac-binary ceiling does NOT apply to the Win build of
`CrimsonDesert.exe` (~410 MB). The Win build keeps every property
name as a plain literal string in `.rdata`, and emits a separately
addressable setter function for each field that writes
`*(TYPE *)(this+offset) = value` followed by a metaobject registration
call passing the literal property-name string. That's exactly the
data we need.

**Proof-of-concept (Session 27 iter 0):** Decompiled
`sub_141957EC0` (Win address) — the AttackInfoDataDesc bindProperty
registrar. It referenced exactly 25 property names — matching the
field count documented in BINARY_FORMATS.md#paatt-basedata-field-layout, and CONFIRMING
all 8 of the "C++ fields not yet present in BaseDataV0" candidates
from Session 19:

`_attackImpulseLevel`, `_attackIndex`, `_attackDivideType`,
`_noCheckCollision`, `_ignoreWhenHitAction`, `_isSingleHitPosition`,
`_ignoreDefenceTypeFlag`, `_targetType` — all real C++ names.

The remaining 17 names matched what BaseDataV0 already had named.

### Recommended IDA plugins (force-multiplier for decoder-gap closure)

For closing the 557 remaining v3.1 decoder gaps (per `V3_1_DECODER_GAPS.md`),
the following plugins make the per-table workflow significantly faster
by automating typeinfo-recovery + vtable-resolution + struct rebuilding:

| Plugin | Source | What it does | IDA-version note |
|---|---|---|---|
| **ClassInformer** | [kweatherman/IDA_ClassInformer_PlugIn](https://github.com/kweatherman/IDA_ClassInformer_PlugIn) (8.x) · [herosi/classinformer](https://github.com/herosi/classinformer) (9.x) | Recovers RTTI vtables + class names — directly unblocks the `pa::<TableName>` typeinfo → vtable lookup step | MSVC-target only (perfect for CrimsonDesert.exe). Drop `*64.dll` into `IDA\plugins\` |
| **IDA-VTableExplorer** | [K4ryuu/IDA-VTableExplorer](https://github.com/K4ryuu/IDA-VTableExplorer) | Browse + inspect vtables (binary file is `vtable64-windows-x64.dll`) | IDA 9+. Hotkey Ctrl+Shift+V |
| **FunctionStringAssociate** | C++: [kweatherman/...](https://github.com/kweatherman/IDA_FunctionStringAssociate_PlugIn) · IDA-9 build: [cafeed28/...IDA90](https://github.com/cafeed28/IDA_FunctionStringAssociate_PlugIn_IDA90) · Python: [oxiKKK/ida-function-string-associate](https://github.com/oxiKKK/ida-function-string-associate) | Tags every function with its referenced strings — speeds up finding the `pa::<TableName>` typeinfo xref | Python rewrite is most portable across IDA 9.x |
| **AutoRE** | [a1ext/auto_re](https://github.com/a1ext/auto_re) | Auto-renames functions from debug/log strings — the per-table parsers gain meaningful names | Older — IDA 7.x + Py3 confirmed; IDA 9.x may need minor `ida_struct`/`ida_typeinf` patching |
| **HexRaysPyTools** | Original: [igogo-x86/HexRaysPyTools](https://github.com/igogo-x86/HexRaysPyTools) · Active fork: [oopsmishap/HexRaysPyTools](https://github.com/oopsmishap/HexRaysPyTools) | Build structs from void access patterns; recover C++ class hierarchies from vtables | **IDA 9.0 deprecated `ida_struct`/`ida_enum` — known breakage on 9.x.** If on 9.x, search community PRs first or pin IDA 8.4 for this plugin |
| **HRDevHelper** | [patois/HRDevHelper](https://github.com/patois/HRDevHelper) | See Hex-Rays ctree when decompiler does something weird | Pure IDAPython. IDA 8.x/9.x ✅. Drop `hrdevhelper.py` + `hrdh/` into `plugins/` |

**Workflow with these installed:**

1. **AutoRE** + **FunctionStringAssociate** — pre-process the IDB once
   to enrich every function with its string references. Decoder-reader
   functions become discoverable by reference name (e.g. searching
   functions tagged with `_effectDataList`).
2. **ClassInformer** — auto-recover `pa::<TableName>` vtables. Each
   table's record reader is at a known vtable slot.
3. **HexRaysPyTools** — auto-build the typed struct from the decompiled
   reader's void-pointer access patterns. Output is directly usable as
   a Rust struct field list.
4. **HRDevHelper** — consult when the decompiler produces confusing
   pseudocode (rare but unblocks edge cases).

Without these, the per-table workflow is still feasible (raw mcp__ida-pro-mcp
calls work) but takes ~hours per table instead of ~minutes. Install
during a future decoder-gap closure session.

### Per-class extraction recipe (Win-binary)

1. Pick a unique-to-class property name (e.g., `_inventoryGroup` for
   ItemInfo).
2. `list_strings_filter` → returns address of the literal string.
3. `get_xrefs_to(string_address)` → returns 2-3 small setter functions
   + 1 large registrar function (typically several KB).
4. `decompile_function(registrar_address)` → returns the full
   property registration sequence; every `_xxx` literal in the body
   is a real C++ field name belonging to this class.
5. Per setter (small, ~125 bytes): decompile to extract
   `*(TYPE *)(this + OFFSET) = value` → recovers in-mem offset + type.
6. Cross-reference the recovered name set against the dmm-parser
   Rust struct field names; mismatches go into `FIELD_ALIASES_V3`
   per Session 24's mechanism. v3 names are preserved as legacy
   aliases.

### Loop is restarting

The /loop will now work through the per-table audit using the
Win-binary recipe. Per iteration: 1-3 classes audited, tracking doc
updated, alias tables shipped, commit + push. Stop only when all 118
on-disk tables are ✅ verified.

### ⚠️ Session 27 iter 1 — second structural finding

The Win-binary recipe **only works for engine descriptor classes**
(types like `pa::AttackInfoDataDesc`, `pa::EmitterCurveData`,
`pa::SplineDecalComponent`) that use the metaobject reflection
system. The 118 on-disk `pa::*Info` table classes do **NOT** go
through metaobject reflection at all — they're read by dedicated
parse functions and never register their fields with the runtime
property system.

Empirical evidence:
- `_skillGroupKey`: only ` _skillGroupKey` (with leading space) in
  the binary — that's a sprintf-style debug-log format string, not
  a bindProperty literal. No bare `_skillGroupKey` exists.
- `_buffLevelList`: same pattern — only the leading-space variant.
- `_priceItemKey` (ItemInfo field): doesn't exist as any string at all.
- Compare with `_attackImpulseLevel` (AttackInfoDataDesc engine
  descriptor): bare literal exists at `0x144c3e428`, has 3 valid
  xrefs, registrar enumerates the full class. Engine-class pattern.

**Conclusion:** The Win-binary recipe expands what's verifiable
beyond the Mac binary, but it expands it for **engine descriptors**,
not for `*Info` tables. The 118 on-disk tables remain pragmatic-T0
(stable descriptive names, no `_unkXXXX` placeholders) per Session
25's bulk promotion. They cannot be strict-T0-verified via static
analysis on either binary, because Pearl Abyss's table parsers
don't expose the C++ field names at all.

### Iteration 1 outcome

Pivoted to the actual reachable T0 work: shipped the two
**confirmed-by-Win-binary** AttackInfoDataDesc renames in
`src/binary/paatt_basedata.rs`:

- `_unk0073` → `attack_impulse_level` (canonical: `_attackImpulseLevel`)
- `_unk0072` → `no_check_collision` (canonical: `_noCheckCollision`)

Both shipped via the alias mechanism (Session 24 `JsonShape`):
- shape='v3' default → emits `_unk0073` / `_unk0072` (v3 mods unaffected)
- shape='v3.1' → emits `attack_impulse_level` / `no_check_collision`
  (v3.1 consumers see canonical names)
- write path accepts BOTH on input regardless of shape

551 tests pass. The remaining 6 Session-27-confirmed names from
AttackInfoDataDesc (`_targetType`, `_attackIndex`,
`_ignoreDefenceTypeFlag`, `_ignoreWhenHitAction`,
`_isSingleHitPosition`, `_attackDivideType`) need their wire
positions resolved via setter decompilation before the rename can
ship — separate per-field work.

### Loop stop decision

Continuing the loop on `*Info` tables would just produce ⚠️ markers
for every entry (recipe doesn't apply to that class family). Better
spend: ship the engine-descriptor renames where Win-binary evidence
exists, plus the gap-fill setter-decompile work to surface wire
positions for the 6 remaining AttackInfoDataDesc fields. Both are
real progress; the table catalog stays at pragmatic-T0 per Session 25.

### ✅ Session 28 — bulk v3.1 alias surface for *Info tables (commit 9e29e10)

Pivot accepted. Rather than wait on per-class IDA work for 113
*Info tables, the existing snake_case Rust struct fields are
mechanically translated to `_camelCase` and shipped behind a v3.1
alias table per-table.

**Why mechanical conversion is sound for *Info tables:** the
existing snake_case names were derived from IDA decompile of
Korean error strings inside parse functions during the Tier 1
promotion pass. Those identifiers ARE the canonical C++ names
with the underscore-camelCase convention flipped (`_camelCase` →
`snake_case`). External evidence: the FemaleAnimations.json mod
in `Crimson Desert/bin64/` references names like `_skeletonName`,
`_gender`, `_defaultActionActionIndex` which map exactly to
dmm-parser's `skeleton_name`, `gender`, `default_action_action_index`.

The v3.1 surface is opt-in via `shape="v3.1"` on
`parse_table` / `serialize_table`. v3 mods see zero diff.

Coverage: 113 of 122 tables. The 9 remaining have zero extracted
fields (the script's regex couldn't find a main struct or all fields
are placeholders) — they round-trip identically regardless of shape.

### Session 28 iter 13 — Embedded data classes triple-blocked en masse

Cross-checked the four remaining DESC targets (AttackHitData, BuffData,
EffectData, ConditionData) against both NattKh's pabgb schema AND the
pycrimson reflection harvest (8,362 classes). Same pattern applies to all.

| Class | NattKh class | NattKh embedded refs | Reflection class |
|---|---|---|---|
| `AttackHitData` | ❌ | 0 | ❌ |
| `BuffData` | ❌ | 2 (`BuffInfo._buffDataList`, `BuffLevelData._buffDataList`) | ❌ |
| `EffectData` | ❌ | 4 (EffectPresetElement, EffectInfo, EffectInfoData, GameGlobalEffectInfo) | ❌ (only `EffectDataReferencePath` shell) |
| `ConditionData` | ❌ | 0 | ❌ |

Pattern: Pearl Abyss's runtime data classes (the ones used **inside**
table records like `BuffInfo._buffDataList[i]`) are systematically not
exposed to reflection. Only the *wrappers that contain them* appear in
either source — `_buffDataList` is recognized as a list-of-something
but the something has no canonical-field manifest.

This means Tier 0 verification for these classes is structurally
impossible from any source we have access to. dmm-parser already
decodes their byte layout via the variants in `src/binary/variants/`
(120 BuffData variants, EffectData family, ConditionData with 405
GameCondition variants, etc.); the DECODE works, only the canonical
NAMES are unrecoverable.

All four classes are equivalently triple-blocked:
- Win-IDA: no per-field metaobject registrar (sub-property only)
- Korean error fragments: present but with zero static xrefs
- Schema/reflection: not present as standalone classes

Verified-blocked. The decoder gaps catalogued in
`docs/V3_1_DECODER_GAPS.md` are the source of truth for what fields
still need wire-position work; canonical naming for those fields is
NOT achievable until either pycrimson supports `.paatt` reflection or
a PS5 demo binary with debug symbols becomes available.

### Session 28 iter 12 — AttackCommonData second verification (cross-ref NattKh schema)

Re-verified iter 5 finding via second-source cross-check. Searched all 434
pabgb-table classes in NattKh's `pabgb_complete_schema.json` for any
canonical PA field name that would have come from AttackCommonData.

Result: zero AttackCommonData fields anywhere in the schema. The closest
matches surface in *other* classes:
- `_targetType` → GimmickInfo_ConstraintData (not AttackCommonData)
- `_damageType` → SkillInfo (not AttackCommonData)
- `_damagePercent` → MaterialRelationMatchUpData (not AttackCommonData)

This is a **third independent confirmation** (after Win-IDA registrar
absence + Korean error fragment xref absence) that AttackCommonData's
canonical field names are not recoverable from any source available to us.
The class exists at runtime, dmm-parser's `paatt_basedata.rs` decodes its
data structurally, but the canonical `_camelCase` identifiers are baked
into hand-written deserializers Pearl Abyss didn't expose to reflection.

Verified-blocked. No further iteration on this class without new evidence.

### Session 28 iter 6 — AttackHitData also blocked (same pattern as AttackCommonData)

Win-binary verification ATTEMPTED. Same diagnostic as iter 5:

- `AttackHitData` typeinfo at 0x144c3ec18 — exactly 1 xref, from
  `sub_141957EC0` (parent AttackInfoDataDesc registrar)
- `_attackHitData` at 0x144c3ec08 — 3 xrefs (2 small setters + parent),
  no standalone bindProperty registrar
- No separate registrar function for AttackHitDataDesc fields

Conclusion: Same structural blocker. AttackHitData is a hand-parsed
sub-struct embedded in AttackInfoDataDesc, not metaobject-registered.

paatt_basedata.rs has partial decode coverage; v3.1 surface for those
fields would need the generator-extension noted in iter 5.

Pivoting to the BuffData / EffectData / ConditionData family decoders
in `src/binary/variants/` for the next iter — those have clear source
decodes (Decoded|Raw enum pattern with typed sub-structs) so the
verify-and-ship potential is highest.

### Session 28 iter 5 — AttackCommonData hits structural blocker (no metaobject registrar on Win)

Win-binary verification ATTEMPTED for `pa::AttackCommonData` (17 fields
per Session 22 Mac findings). Result: **same structural blocker as
the *Info tables**.

Evidence:
- `AttackCommonData` typeinfo string at 0x144c3ebf0 has exactly 1 xref:
  `sub_141957EC0` (the AttackInfoDataDesc parent registrar) — registering
  AttackCommonData as a sub-property, not as its own metaobject.
- `_attackCommonData` string at 0x144c3ec68 has 3 xrefs: 2 small setters
  + the same parent registrar. No standalone bindProperty registrar.
- Field-name strings like `_damageType` and `_damagePercent` exist in
  `.rdata` but **only** as error-message fragments with a leading space
  (`" _damageType"`, `" _damagePercent"`) — NOT as bare property
  literals. This is the *Info-table pattern, not the EmitterCurveData/
  SplineDecalComponent pattern.

Conclusion: AttackCommonData fields are parsed by hand-written
deserializers, not registered via metaobject. The Win-binary recipe
does not apply.

paatt_basedata.rs already has Korean-error-derived snake_case names for
several AttackCommonData fields (`attack_pos_offset`, `attack_degree`,
`attack_yaw`, `normal_string_index`, `equip_slot_name_key` per existing
doc comments) — same mechanical-translation situation as the *Info
tables. Could ship a v3.1 surface for paatt_basedata.rs in a future
pass by extending `scripts/generate_v3_1_aliases.py` to walk
`src/binary/paatt*.rs`, but that's separate from strict-T0 IDA work.

### Session 28 iter 4 — SplineDecalComponent enumerated (info only, no decode in dmm-parser)

Win-binary recipe applied to SplineDecalComponent registrar at
`sub_1432329F0` (0x1e90 bytes). 17 properties recovered (Session 22's
"13 PropertyBind types" undercount — Win view shows 17):

`_splineComponentIndex`, `_groupName`, `_splineID`, `_textureFilename`,
`_textureFileName2`, `_textureFilename3`, `_textureFilename4`,
`_detailHeightTexture`, `_detailNormalTexture`, `_placementId`,
`_exceptCapture`, `_volumeDatas`, `_pointDatas`, `_textureSet0`,
`_textureSet1`, `_textureSet2`, `_textureSet3`.

Partial in-mem offset map (registrar still has more `MEMORY[...] = N`
writes beyond what the regex caught): 136, 144, 152, 160, 344, 420,
424, 448, 456.

**No renames shipped:** SplineDecalComponent is not decoded in
`src/` — grep for `SplineDecal|spline_decal|splineDecal|_splineComponentIndex|
_textureFilename|_pointDatas|_volumeDatas` returns 0 source hits.
Recorded for future use.

### Session 28 iter 3 — EmitterCurveData enumerated (info only, no decode in dmm-parser)

Win-binary recipe applied to `pa::EmitterCurveData` registrar at
`sub_142C228D0` (0x5f4 bytes) + sibling `sub_15414B890` (registers
the 4th field). All 4 properties recovered:

| Property | Type | In-mem offset |
|---|---|---|
| `_splineID` | uint32 | +40 |
| `_splineTextureIndex` | uint32 | +44 |
| `_splineData` | uint16 vector (capacity 4096) | +0 (buffer base) |
| `_presetName` | staticstringA | +48 |

**No renames shipped:** EmitterCurveData is not currently decoded
anywhere in `src/` (grep returns 0 source hits — only this doc and
STATUS.md). Recorded here for future reference if a downstream
table embeds this descriptor.

Next engine descriptor in queue: SplineDecalComponent (~13 fields
per Session 22), then AttackCommonDataDesc (17 fields), AttackHitDataDesc
(8 fields). Same pattern applies — verify on Win, document, ship
renames only if dmm-parser decodes the type.

## Per-table tracking

| # | Module | Status | Notes |
|---|---|---|---|
| 1 | action_point_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 2 | action_restriction_order_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 3 | ai_dialog_string_info | ⚠️ | not in NattKh schema; mechanical-fallback v3.1 aliases (Session 28) |
| 4 | aiaction_attribute_info | ⚠️ | not in NattKh schema; mechanical-fallback v3.1 aliases (Session 28) |
| 5 | aidialog_type_info | ⚠️ | not in NattKh schema; mechanical-fallback v3.1 aliases (Session 28) |
| 6 | aievent_table_info | ⚠️ | not in NattKh schema; mechanical-fallback v3.1 aliases (Session 28) |
| 7 | aimemory_info | ⚠️ | not in NattKh schema; mechanical-fallback v3.1 aliases (Session 28) |
| 8 | aimove_speed_info | ⚠️ | not in NattKh schema; mechanical-fallback v3.1 aliases (Session 28) |
| 9 | ally_group_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 10 | auto_spawn_filter_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 11 | bitmap_position_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 12 | board_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 13 | breakable_object_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 14 | buff_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 15 | category_group_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 16 | category_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 17 | character_appearance_index_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 18 | character_change_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 19 | character_group_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 20 | character_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 21 | condition_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 22 | craft_tool_group_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 23 | craft_tool_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 24 | detect_detail_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 25 | detect_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 26 | detect_reaction_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 27 | dialog_voice_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 28 | drop_set_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 29 | dye_color_group_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 30 | effect_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 31 | elemental_material_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 32 | equip_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 33 | equip_slot_info | ⚠️ | not in NattKh schema; mechanical-fallback v3.1 aliases (Session 28) |
| 34 | equip_type_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 35 | faction_group_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 36 | faction_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 37 | faction_node_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 38 | faction_node_spawn_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 39 | faction_relation_group_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 40 | faction_spawn_data_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 41 | faction_waypoint_info | ⚠️ | not in NattKh schema; mechanical-fallback v3.1 aliases (Session 28) |
| 42 | fail_message_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 43 | field_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 44 | field_level_name_table_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 45 | field_revive_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 46 | formation_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 47 | frame_event_attr_group_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 48 | game_advice_group_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 49 | game_advice_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 50 | game_event_handler_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 51 | game_global_effect_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 52 | game_level_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 53 | game_play_trigger_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 54 | game_play_variable_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 55 | gimmick_event_table_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 56 | gimmick_gate_connection_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 57 | gimmick_gate_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 58 | gimmick_group_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 59 | gimmick_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 60 | global_game_event_group_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 61 | global_game_event_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 62 | global_stage_sequencer_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 63 | house_info | ⚠️ | not in NattKh schema; mechanical-fallback v3.1 aliases (Session 28) |
| 64 | interaction_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 65 | inventory_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 66 | item_group_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 67 | item_use_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 68 | job_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 69 | key_map_setting_list_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 70 | knowledge_group_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 71 | knowledge_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 72 | level_action_point_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 73 | level_gimmick_scene_object_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 74 | local_string_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 75 | material_blood_decal_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 76 | material_match_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 77 | material_relation_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 78 | mercenary_group_info | ⚠️ | not in NattKh schema; mechanical-fallback v3.1 aliases (Session 28) |
| 79 | mercenary_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 80 | mini_game_data_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 81 | mission_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 82 | multi_change_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 83 | npc_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 84 | part_prefab_dye_slot_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 85 | part_prefab_dye_texture_pallete_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 86 | pattern_description_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 87 | platform_achievement_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 88 | platform_entitlement_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 89 | quest_gauge_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 90 | quest_group_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 91 | quest_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 92 | quick_time_event_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 93 | region_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 94 | relation_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 95 | reserve_slot_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 96 | royal_supply_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 97 | sequencer_spawn_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 98 | skill_group_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 99 | skill_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 100 | skill_tree_group_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 101 | skill_tree_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 102 | socket_group_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 103 | socket_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 104 | spawning_pool_auto_spawn_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 105 | special_mode_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 106 | stage_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 107 | status_group_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 108 | status_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 109 | store_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 110 | string_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 111 | sub_level_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 112 | terrain_region_auto_spawn_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 113 | terrain_region_navi_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 114 | tribe_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 115 | trigger_region_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 116 | ui_social_action_info | ⚠️ | not in NattKh schema; mechanical-fallback v3.1 aliases (Session 28) |
| 117 | uifilter_group_info | ⚠️ | not in NattKh schema; mechanical-fallback v3.1 aliases (Session 28) |
| 118 | uimap_texture_info | ⚠️ | not in NattKh schema; mechanical-fallback v3.1 aliases (Session 28) |
| 119 | valid_schedule_action_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 120 | vehicle_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 121 | vibrate_pattern_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
| 122 | wanted_info | 🟡 | schema-verified via NattKh (Session 28); v3.1 aliases shipped |
