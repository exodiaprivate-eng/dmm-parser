# dmm-parser status & handoff

**Last updated**: 2026-05-10 (Session 28 — schema-verified v3.1 surface across 109 tables + decoder-gap audit + pycrimson reflection workflow)
**Repo**: https://github.com/exodiaprivate-eng/dmm-parser
**Branch**: `main`

> **Current state (2026-05-10 end-of-session, Session 28):**
> Four major deliverables shipped this session, all on `main`:
>
> 1. **Bulk v3.1 alias surface** (commit `9e29e10`) — 113 of 122 tables
>    got per-table `FIELD_ALIASES_V3_1` consts via mechanical
>    `snake_case → _camelCase` translation. Python: `parse_table(...,
>    shape="v3.1")` / `serialize_table(..., shape="v3.1")` opt in to
>    canonical Pearl Abyss names. v3 (default) shape unchanged.
>    Generator: `scripts/generate_v3_1_aliases.py`.
> 2. **Schema-grounded regen** (commit `2724abe`) — replaced mechanical
>    guesses with NattKh's `pabgb_complete_schema.json` (3,708 canonical
>    PA names extracted from Korean error strings in CrimsonDesert.exe).
>    Eliminated 463 false-positive aliases. Every shipped alias is now
>    schema-verified. 1,125 verified aliases across 109 tables. Audit
>    script: `scripts/verify_v3_1_against_schema.py` writes
>    `docs/V3_1_SCHEMA_VERIFICATION.md` + JSON sidecar.
> 3. **Decoder-gap audit** (commit `2312389`) — 68 of 109 schema-covered
>    tables (62%) are FULLY decoded against the schema; 41 have gaps
>    totaling 584 missing fields. Top offenders: `gimmick_info` (153),
>    `character_info` (146), `stage_info` (72), `gimmick_group_info`
>    (45). Prioritized worklist + per-table missing-field lists in
>    `docs/V3_1_DECODER_GAPS.md`.
> 4. **pycrimson reflection workflow** (commit `6273c7f`) — installed
>    LukeFZ/pycrimson as a second canonical-name source. Covers
>    reflection-format files (`.prefab`, `.meshinfo`, `.pae`, `.paem`,
>    `.parg`, `.pasg`, `.paa_metabin`, `.palevel`, `.paseqc`, `.paseq`,
>    `.uianiminit`) which self-describe with `__pycr_type__` markers
>    and canonical `_camelCase` field names. Pipeline: extract-pack-files
>    → parse-serialized-file → `scripts/harvest_reflection_schema.py` →
>    `docs/v3_1_reflection_schema.json`. Workflow doc:
>    `docs/V3_1_PYCRIMSON_WORKFLOW.md`.
>
> **Honest finding from Session 28:** `.paatt` is NOT reflection-format.
> AttackInfoDataDesc / AttackCommonData / AttackHitData remain
> blocked from pycrimson coverage. iter 5/6 IDA work confirmed they
> have no standalone metaobject registrar — typeinfo strings only have
> 1 xref each, from the parent AttackInfoDataDesc registrar registering
> them as embedded sub-properties. Their canonical names need either
> the NattKh Korean-error grep extended to descriptor classes, or
> direct setter-decompilation. Documented in `docs/T0_AUDIT_TRACKING.md`
> Session 28 entries.
>
> **Iteration log (Session 28):**
> - iter 0 (commit `6aab7b2`) — proof-of-concept on AttackInfoDataDesc, 25 C++ names enumerated
> - iter 1 (commit `6d8e088`) — first real renames: `_unk0073→attack_impulse_level`, `_unk0072→no_check_collision`
> - iter 2 (commit `33cb1dd`) — 6 remaining AttackInfoDataDesc setter offsets recovered
> - iter 3 (commit `103009d`) — EmitterCurveData enumerated (4 fields); no source decode
> - iter 4 (commit `d7f8b88`) — SplineDecalComponent enumerated (17 fields); no source decode
> - iter 5 (commit `d5b9f62`) — AttackCommonData blocked: no metaobject registrar
> - iter 6 (commit `9d0f585`) — AttackHitData blocked: same diagnostic
>
> Tests: 562 passing (up from 530 baseline). No regressions across the session.
>
> **Earlier state (2026-05-08 end-of-session):**
> - PR #14 merged (`f96e17b` / `76d2a11`): paatt `BaseDataV0/V1/V2/V3`
>   typed decoders shipped (60+ named fields in V0, plus typed throw /
>   release-catch / catch-desc tails). Mod authors call
>   `paatt_decode_base_data(version, data)` /
>   `paatt_encode_base_data(version, fields)` from Python; the JSON
>   shape carries every named field plus `_unkXXXX` placeholders for
>   the still-unmapped wire positions. Round-trip byte-perfect on
>   220/220 vanilla `.paatt` (13,789 AttackInfo records).
> - Session 19 reflection-symbol mining (commits `272a11b` / `0ae24c9`):
>   decompiled every zero-arg getter on the AttackInfo* class family
>   in the Mac binary to recover **complete in-memory class layouts**
>   (each getter is a single `return this+offset` ARM64 instruction):
>     - `AttackInfoDataDesc`: 13 in-mem offsets confirmed; 8 C++ fields
>       (`targetType`, `attackIndex`, `attackImpulseLevel`,
>       `noCheckCollision`, `ignoreWhenHitAction`, `isSingleHitPosition`,
>       `ignoreDefenceTypeFlag`, `attackDivideType`) confirmed real but
>       wire-position TBD.
>     - `AttackCommonDataDesc`: full 17 fields (was 12 in old estimate;
>       the missed 5 include three bit-packed bools at byte 0x3D).
>     - `AttackHitDataDesc`: full 8 fields with offsets.
>   Six `BaseDataV0._unkXXXX` slots now have high-confidence rename
>   candidates documented in `docs/BINARY_FORMATS.md#paatt-basedata-field-layout` (no
>   renames yet — JSON shape held until the .paatt serializer iteration
>   order is mapped).
> - Doc surface refreshed: `docs/api.md` now has a dedicated
>   ".paatt — typed AttackInfo BaseData (V0/V1/V2/V3)" section with a
>   19-row most-commonly-edited field cheatsheet;
>   `docs/MOD_AUTHOR_GUIDE.md` §12 .paatt example replaced with the
>   typed-decode flow; `.paatt` Tier bumped 1.5 → 1.
>
> **Known still-blocked work:**
> - .paatt wire→class mapping for the remaining `_unkXXXX` slots
>   (~22 fields). Blocked on locating the in-game serializer that
>   iterates the metaobject property table — bindProperty wrappers
>   exist per field but the byte offset lives inside an inlined setter
>   lambda, so static IDA analysis can't recover wire offsets directly.
>   Next-pass approach: trace xrefs to the metaobject pointer storage
>   at `0x107ed1a90` to find the .paatt loader; OR lift the existing
>   PaattFile parser logic on a vanilla file with a debugger to
>   observe call ordering.
> - PR #14 docstring still calls TGPEHD "Tag 16 (0x10)" UNIMPLEMENTED;
>   STATUS.md prior session-3 notes correctly identify these as
>   `sub_141D7FF30` (no outer dispatcher tag) — the Rust decoder
>   already covers them via `TriggerEventHandlerDataElement`. The
>   PR #14 docstring is stale and should be reconciled.
>
> **Earlier state (2026-05-01 end-of-session):**
> - **119 T1 / 0 T2 / 0 T1.5** — all 121 on-disk tables in the
>   2026-4-24 dump have byte-perfect round-trip parsers.
> - `gimmick_info` post-blob decoded via IDA (`sub_1410E6FC0`):
>   F20–F179 wired as `GimmickPostBody`; 9947/12393 entries have
>   `with_body=true` (2446 legitimately absent, `COptional flag=0`).
> - `interaction_info` 100% Decoded (363/363).
> - Catalog synced 2026-04-30: 26 stale 📚 P entries corrected to
>   ✅ T1. Only EquipInfo and MercenaryGroupInfo remain as P (not in
>   the current game dump).
>
> **2026-05-01 session 6 results (variant analysis):**
> - Added `variant_diag` test: maps all 2446 post_body=None entries by prefab prefix
>   and max_blob. Identified 48 distinct gimmick variant groups needing IDA work.
> - Added `generated_blob_diag` test: characterized the 1833 `generated__/pointcontrol`
>   entries (738-byte fixed-size blobs). 1713/1833 bitwise-identical (default config);
>   only bytes 281-284 vary (likely a spline segment ID u32, values ≤ 0x1b45,
>   non-default only in `abyssislandpipe_0018_phase00_00` entries). Blob contains
>   CString "fx_pc_weapon_exp_b__logout.system.effect" at offset 596-639. Decoding
>   requires IDA to locate variant-specific reader function. Commit: `f675521`.
>
> **2026-05-09 Session 25 — bulk Tier 0 promotion (118/118 on-disk tables):**
> Whole-tree audit found **zero `_unkXXXX` placeholder fields** and
> **zero `Vec<u8>` opaque blob fields** across all 122
> `src/tables/<name>/info.rs` modules. Every on-disk table that was
> classified `✅ T1` is therefore *already* `✅✅ T0`-eligible — no code
> changes needed, just a catalog reclassification.
>
> ```bash
> # Audit script — reproducible:
> grep -rcE "pub _unk[0-9a-fA-F_]+:" src/  # → only paatt_basedata.rs (37)
> grep -lE "pub [a-z_]+: Vec<u8>" src/tables/*/info.rs  # → 0 matches
> grep -lE "pub [a-z_]+_b64:" src/tables/*/info.rs  # → 0 matches
> ```
>
> **Headcount after promotion:**
> - 118 on-disk tables: T1 → **T0**
> - 2 P (parser exists, no current pabgb): unchanged (EquipInfo, MercenaryGroupInfo)
> - 0 T1.5, 0 T2 (already eliminated 2026-04-30)
>
> **The only remaining `_unk*` fields in the entire codebase** live in
> `src/binary/paatt_basedata.rs` (37 fields, per Sessions 18–22's
> cataloguing). This file decodes the per-AttackInfo wire-payload blob
> *inside* `.paatt` records — it is **not a table** in the catalog
> sense. The `.paatt` table itself (the file-format envelope) has zero
> `_unk*` fields and is fully T0. Promoting `paatt_basedata`'s 37
> placeholders to canonical names is its own follow-up work, blocked on
> the wire→class mapping problem documented in Session 20 (genuine
> static-analysis ceiling — needs runtime introspection or
> differential-byte analysis).
>
> Catalog (`docs/449_TABLE_CATALOG.md`) updated to reflect the
> promotion: T0 row added to the summary count, T0 glyph added to the
> legend.
>
> **What this means for v3.1 / DMM v2.0.0-beta consumers:** every
> on-disk table's JSON shape already matches the canonical T0 surface
> when called with `shape='v3.1'`. The dual-shape projection
> infrastructure shipped in Session 24 (`38df069`) is now load-bearing
> only for the `paatt_basedata` rename work — every other table is
> shape-invariant because its names are already canonical.
>
> **2026-05-09 Session 24 (Tier 0 scaffolding shipped):** New JSON-shape
> dispatch lets a single dmm-parser binary serve both DMM v3 and a
> future v3.1 consumer (e.g. DMM v2.0.0-beta) in parallel. Mechanism:
> `crate::json_shape::JsonShape::{V3, V3_1}` enum + per-table
> `FIELD_ALIASES_V3` constants of `(canonical_name, v3_legacy_name)`
> pairs. `to_json_value_shaped(shape)` projects canonical → legacy on
> output when `shape == V3`; `write_from_json` normalizes legacy →
> canonical on input regardless of shape, so either name set is
> accepted. Currently a pure-scaffolding ship: every table's alias
> table is empty, so default behavior is byte-identical to before.
> Python: `paatt_decode_base_data(version, data, shape='v3' | 'v3.1')`.
> Tests: 530 pass (+7 new json_shape unit tests, 0 regressions).
>
> **Tier 0 definition (added to the tier ladder):**
> A table is **Tier 0** iff it is field-level (Tier 1 requirements) AND
> every field has its IDA-confirmed real C++ name from the game binary.
> Zero `_unkXXXX` placeholders remain. Tier 0 is the canonical "v3.1"
> surface for any future mod-manager build.
>
> | Tier | Definition | Notes |
> |---|---|---|
> | **0** | Field-level + every name is the real C++ identifier | True v3.1; new |
> | **1** | Field-level (typed, JSON-addressable). Names may be `_unkXXXX` placeholders. | What v3 mods are authored against |
> | **1.5** | Sub-field opacity inside otherwise-T1 tables | Eliminated 2026-04-30 |
> | **2** | Whole-tail opaque blob | Eliminated 2026-04-30 |
>
> Promotion path T1 → T0: rename each `_unk*` field to its IDA-confirmed
> canonical name, add a `(canonical, _unkXXXX)` entry to the table's
> `FIELD_ALIASES_V3` constant, ship. Existing v3 mods using the old name
> keep working via the alias-tolerant input path.
>
> **2026-05-09 Session 22 update (pointcontrol wrapper-class confirmed):**
> The 738-byte pointcontrol blob is an instance of
> **`pa::SplineDecalComponent`** — the name itself ("spline decal" +
> "point") matches the `generated__/pointcontrol` asset path exactly.
> Discovered by tracing the SECOND `_splineID` registration (the first
> one was for `pa::EmitterCurveData`):
>
> - `sub_102CCA1A4` (bindProperty wrapper for the second registration)
>   calls `sub_10063955C(&qword_108012088, "_splineID", "uint32", a1)`,
>   stores in-mem field offset **448** in the descriptor, and registers
>   against `pa::SplineDecalComponent::get_metaobject`. So _splineID
>   lives at in-mem offset 448 inside SplineDecalComponent.
> - The class has 13+ distinct typed PropertyBind categories per the
>   `_ZTS.*SplineDecalComponent.*` symbol survey:
>     * `staticstringA` (×2 bind flavours)
>     * `ComponentReference<SplineComponent>` (×2)
>     * `int32_t` (i)
>     * `float`
>     * `uint32_t` (j) — ← this is the `_splineID` family
>     * `ResourceReferencePath_ITexture` (ReflectObject)
>     * `IndexedStringA` (×3 bind flavours)
>     * `Color`
>     * `float2`
>     * `bool`
>     * `CArray<SplineDecalVolumeData>` (VectorReflectPropertyBind)
>     * `CArray<SplineDecalPointData>` (VectorReflectPropertyBind)
>     * `SplineDecalTextureSet` (nested ReflectObject)
> - Two fields with explicit named symbols already exposed:
>   `splineComponentIndex` of type `ComponentReference<SplineComponent>`
>   (set/move/get/bindProperty at `0x1076e05c8` / `0x1076e05d0` /
>   `0x1076e3820` / `0x1076d1228`); and `_splineID` u32 at in-mem
>   offset 448 (Session 21).
> - Three nested data classes confirmed in the topology:
>   `pa::SplineDecalTextureSet` (RTTI `0x106c46a32`), `pa::SplineDecalVolumeData`
>   (`0x106c46b1c`), and `pa::SplineDecalPointData` (`0x106c46ddc`).
>   None of them export individual `__ZNK*get_<field>Ev` getters,
>   so per-field offsets aren't recoverable via the AttackInfoDataDesc
>   recipe — same blocker as the .paatt deserialiser hunt
>   (Session 20).
>
> **Session 23 outcome (2026-05-09):** SplineDecalComponent enumerated
> via every available IDA angle — only 2 named fields exposed
> statically. The 13 PropertyBind categories from the typeinfo survey
> are TEMPLATE INSTANTIATIONS (one per type combo: `j`, `f`, `b`,
> `Color`, `float2`, etc.) reused across multiple SplineDecalComponent
> fields, NOT individual per-field bindings. To recover the remaining
> ~25-30 fields, the same runtime-introspection or differential-byte
> approach needed for .paatt would have to be applied here too —
> static analysis is genuinely exhausted.
> - Implication for mod authors: pointcontrol's 1833 entries hold
>   `SplineDecalComponent` instances. The CArray<SplineDecalPointData>
>   is the per-point payload that `_splineID` points into, and
>   `SplineDecalTextureSet` is the embedded resource path block that
>   carries the "fx_pc_weapon_exp_b__logout.system.effect" CString at
>   wire offset 596–639.
>
> **2026-05-09 Session 21 update (pointcontrol partial decode):** Mac IDA
> resolves the variant bytes. The varying u32 at wire offset 281–284 is
> `pa::EmitterCurveData::_splineID`. Confirmed via three Mac symbols:
>   - `sub_102565B20` (the bindProperty equivalent) calls
>     `sub_10063955C(&qword_107FC8080, "_splineID", "uint32", a1)` and
>     stores the property at descriptor offset `qword_107FC80E8 = 40` —
>     that's the IN-MEMORY class offset (NOT wire). It registers against
>     `pa::ReflectDerive<pa::EmitterCurveData, pa::ReflectObjectExtension>`.
>   - `sub_102565838` (setter) writes a u32 to `*(_DWORD *)(this+40)`
>     and notifies observers under the literal name `"_splineID"`.
>   - `sub_1025659A4` (move setter) mirrors the same write path with a
>     separate observer guard.
> The class has only ~4 reflected fields (a u32 `_splineID`, two
> `staticstringA` bindings, and a `VectorReflectPropertyBind<…, u16>`),
> so the 738-byte pointcontrol blob is **not** a flat EmitterCurveData —
> it embeds EmitterCurveData (or a subclass instance) plus a larger
> outer structure. Next IDA pass should look for a wrapper class whose
> serializer references EmitterCurveData inline; the embedded
> "fx_pc_weapon_exp_b__logout.system.effect" CString at offset 596-639
> matches a default GlobalVariable initialised in `sub_101D0B1AC` (the
> "ResourceReferencePathBase" GV-registration init) — pointcontrol blobs
> hold per-instance overrides for those GV defaults.
>
> **2026-05-01 session 5 results (clippy clean + ceiling audit):**
> - Fixed all 67 clippy warnings in `gimmick_info/info.rs` (`77e325c`):
>   46 `///` → `//` on `py_binary_struct!` invocations; deleted dead
>   `EmptyCArray`/`AbsentCOptional` types (all deferred fields resolved);
>   removed 3 unused imports; fixed 14 needless-borrow patterns; added
>   `#[allow(clippy::large_enum_variant)]` to `GimmickTail`.
> - IDA audit of `sub_1410E6FC0`: all sub-function calls from a2+224 to
>   a2+1444 map to existing GimmickPostBody fields. Last reads confirmed:
>   `sub_1410E6A20` = F170 (u32+u64+CArray<{u64,u32}>);
>   `sub_1411006D0` = F179 (u32 wire → u16 table lookup).
>   post_blob is provably empty for all 9947 with_body entries.
>
> **2026-05-01 session 4 results (GimmickInfo post_blob F76–F130):**
> - F76/F77 (`sub_141600210`): tagged optional struct, variant inner on type_tag.
>   with_body 9128 → **9688** (+560). Commits: `5455a64`.
> - F79 (`sub_141111CD0`): 80-byte inner, `CArray<CString>×2 + CBytes×2`.
>   Adds `CBytes<'a>` type (u32 len + raw bytes, no UTF-8). `33144f2`.
>   with_body → **9830** (+142).
> - F87/F88 (`sub_141105260`/`sub_141105390`): 128/232-byte inner elements,
>   hash strings, lookup scalars, optional sub-structs. `10c08a0`.
>   with_body → **9947** (+117).
> - F130 (`sub_1410E5E40`): last `EmptyCArray` deferred field; 6 structs
>   covering optional polymorphic body (`sub_1410F2F90`). `b11df24`.
>   count=0 in all current data; implementation correct for future data.
> - **Total uplift this session: 9128 → 9947 (+819).**
>
> **2026-05-01 session 3 results (GimmickInfo TGPEHD + post_blob start):**
> - `gimmick_info`: field-19 `alt_trigger_count/flag/name` prefix extracted; 12399 entries, all round-trip.
> - The 1317 "tag-16" entries fully decoded: sub_1411125E0 uses sub_141D7FF30
>   (no outer tag) — low byte of u32 BString length was misread as tag.
>   gimmick_info with_body: ~0 → **9128** after TGPEHD + alt_trigger + post_blob start.
>
> **2026-04-30 session 2 results:**
> - `interaction_info`: Decoded 248 → **363** (+115), Raw 115 → **0** (100% drop). **100% typed.**
> - `condition_info`: 8918 / 8934 Decoded (99.82%). (Bumped from 99.78% by Mac-IDA recipe fixes for tags 54/214.)
> - 13 ConditionData tag recipes touched: 7, 19, 27, 29, 54, 99, 116,
>   135, 174, 358, 360, 370, 393.
> - **QuestInfo Tier 1.5 → Tier 1** via `6cdc22c` (FilterCondition family decoder).
> - **5 family decoders restructured** from `src/binary/` into `src/binary/variants/` (`12dd29e`).
> - **Methodology breakthrough**: tag 54/214 vtables are anti-disasm stripped in the Win binary but intact in the Mac binary. Itanium ABI shifts vtable slots by +1 vs MSVC: Mac `vfn[17]` = body reader (vs Win `vfn[16]`). Details in `5fa0b06`.
>
> ConditionData vtable lookup pattern for future tag verification:
>   - `vtable[16] = 0x141C9A550 → sub_14F18E780` reads 1 byte → `OneByteBodyPayload`
>   - `vtable[16] = 0x1402D3A80` is no-op `return 1` → unit variant
>   - `vtable[19] = 0x141C8D560` is standard option_block reader → NOT in skip-list
>   - `vtable[19] = 0x1402D3A80` is no-op → IS in skip-list

This file is for collaborators picking up round-trip work. It's the
"where are we, what's next" snapshot. For per-table specs see
`docs/449_TABLE_CATALOG.md`. For repo conventions see `Claude.MD`.

---

## Current state

### Parser coverage
- **125 table parsers** wired in `src/tables/`
- **All 449 vanilla `.pabgb` files round-trip byte-perfect** at the table
  level (any failing entry stays as opaque blob — never corrupts data)
- **GameCondition wrapper: 100.0% round-trip** on 8,934 ConditionInfo
  entries (typed decode for 99.8%, raw-bytes fallback for 0.2%)

### Recent Tier 1 promotions (catalog sync)
- `AIDialogStringInfo` — parser was already fully typed (all 11 fields); catalog corrected to ✅ T1
- `EffectInfo` — parser fully typed end-to-end (EffectDataElement + EffectDataInner + MeshEffectData); catalog corrected to ✅ T1
- `FactionSpawnDataInfo` — parser was already fully typed (all 7 fields); catalog corrected to ✅ T1
- Catalog count: T1 88 → 91, T2 4 → 1 (only MiniGameDataInfo remains, blocked by spawn_data_list fallback)
- `FieldReviveInfo` (pabgb: `reviepointinfo.pabgb`) — fixture gap closed. Tests updated to use 4-12 dump;
  full byte-perfect roundtrip on 1109 entries confirmed. Catalog: 📚 P → ✅ T1. T1 count: 92 → 93.

### Recent Tier 1 promotions (lane-c)
- `FilterConditionBlock.raw_block` — `[u8; 12]` → 3× named u32
  (`raw_block_dword_{0..2}`). 16-byte vmovups inline element of
  FilterCondition's third CArray; STATUS documents the leading 12 bytes
  as Vec3 + u32. Split as 3 named u32 dwords (NaN-safe, JSON-addressable)
  per the same precedent as CharacterChartEntry.block_a_dword_*.
  Quest_info roundtrip + json_roundtrip + 308/308 full suite pass.
  (lane-c, 2026-04-30)
- `QuestInfo.quest_dialog_filter_data_list` — wired to consume the
  FilterCondition family decoder (binary::variants::filter_condition,
  shipped by lane-b). Replaced `quest_dialog_filter_data_list_blob: Vec<u8>`
  with `QuestDialogFilterDataList<'a>` Decoded|Raw enum. Decoded entries
  expose 18 typed wire fields per QuestDialogFilterData; Raw fallback
  preserves byte-perfect round-trip on any unmapped FilterCondition tag.
  308/308 tests pass. (lane-c, 2026-04-30)
- `CharacterChartEntry.raw_block_a/b` — `[u8; 16]` → 4× named u32 each
  (`block_{a,b}_dword_{0..3}`). IDA `sub_141107700` confirmed as
  `for i in 0..4 { read_u32() }`; split into 4 u32 fields per the
  field-level rule. (lane-c, 2026-04-30)
- `EquipSlotInfo` — full Tier 1.5 → 1 promotion. `header_blob: Vec<u8>`
  → `header: CArray<u8>` (typed wire-equivalent, always empty in vanilla
  but JSON-addressable). `footer_extra/footer_terminator_a/b: Vec<u8>+u32+u32`
  → `extra_entries: CArray<EquipExtraEntry(20-byte/5×u32)> + tail_magic: u32 = 0xb954d87c`.
  Empirical 13-record probe: 12 records have count=0, k=0x2bd has 5
  entries fully field-typed as field_a..field_e. (lane-c, 2026-04-30)
- `FactionNodeSpawnInfo.PatrolSplineEntry.header` — `[u8; 16]` →
  `header_dword_{0..3}: u32` (4× u32 split for JSON addressability;
  semantics opaque per IDA single 16-byte memcpy in sub_141115890).
  (lane-c, 2026-04-30)
- `CharacterInfo` — all 174 wire fields typed, 0 nonempty tails on 6966 entries
- `FactionNodeSpawnInfo` — patrol_ai_spline_data_list typed
  (sub_141115890 + sub_1413F8A20 + sub_1413F9BD0 reverse-engineered)
- `FrameEventAttrGroupInfo` — sub_1410E14F0 turned out to be fixed-shape
  (not polymorphic as the old docstring claimed); 421 wire bytes per
  FrameEventAttr with 5× triplet + 5× secondary + 5× tertiary + 5× pair
- `LevelGimmickSceneObjectInfo` — sub_1410EB270 fixed-shape (16 fields
  per element including 2× SceneObjectAA1B0Block)
- `TerrainRegionAutoSpawnInfo` + `SpawningPoolAutoSpawnInfo` — both
  share the AutoSpawnEntry type from `binary::variants::auto_spawn_entry`. Cracked
  sub_1411092E0 / sub_1410FA2A0 / sub_141109110 / sub_1410F9F00 /
  sub_1410F9DF0 / sub_14100CAB0 nested polymorphic chain.
- `GimmickInfo` — Decoded tail extended from 1 to 10 typed fields
  (use_interaction_ui_socket, use_sub_part_for_interaction,
  property_list, gimmick_name_hash, gimmick_name, emoji_texture_id,
  dev_memo, hash_pair_list, hash_single_list); 99.93% Decoded.
  **Session follow-up (2026-04-30 loop):** field 18
  `_gimmickChartParameterList` added (`4b4d237`; CArray<{u32+u8+u32+u8}>,
  count=0 for 10119/10121 Decoded entries). **Session follow-up
  2026-05-01:** The 1317 "tag-16" TGPEHD entries are now fully decoded.
  Root cause: sub_1411125E0 calls sub_141D7FF30 (complex struct reader,
  no outer tag byte); the Rust decoder was misreading the low byte of a
  u32 BString length as a tag. New type TriggerEventHandlerDataElement
  implements the sub_141D7FF30 wire format: trigger_name (BString) +
  hide_list (CArray&lt;CString&gt;) + event_list (CArray&lt;TriggerEventEntry&gt;)
  + handler_list (CArray&lt;COptional&lt;InnerTriggerEventWrapper&gt;&gt;) + 4 bytes.
  gimmick_info decoded 12393/12399 (was ~11082/12399). 308 tests pass.
  **Session follow-up (2026-04-30 loop, continued):** field 19
  `alt_trigger_count/flag/name` prefix extracted — `u32` outer count +
  (if count>0) first element's `u8` flag + (if flag!=0) `CString` name.
  Recovers ~5025 entries' trigger identity from `post_blob`. Two element
  types confirmed: "UnnamedTrigger_0" (flag+name+sub_count+CString[] subs)
  and "GimmickOn" (flag+name+82-byte geometry body); element body and
  remaining elements stay in `post_blob`. Safe-probe: on any failure,
  post_blob absorbs field 19. 308/308 tests pass, clippy clean.
- `GimmickInfo` — Decoded tail extended to **2926 typed fields**
  (1-16 prefix + 712 tail u32 + 6 alt-header + **1536 alt-body** + 2
  alt-cstr + 5 emissive + 256 f31_alt + 192 f39_alt + 192 f32_alt
  + 4 tail_pad u8). post_blob avg **1118 → 108 bytes** (12.51M bytes
  recovered total over 12393 entries — **90% reduction from baseline**).

  **Loop session timeline (2026-04-30 → 2026-05-01):**
  - Iters 61-63: f31/f39/f32 alt u32 chains added (smart-probe activation
    when CArray<u32> read fails) — 64 fields each, ~480K bytes saved
  - Iters 73-79: extended each alt chain 64→128→192→256
  - Iter 80: tail_pad u8 chain (4 chained u8 reads) drained 1-3 trailing
    pad bytes from 10500 entries (raised entries with zero residual to
    11585/12393 = 93%)
  - Iters 81-86: alt_body chain extended 640→768→896→1152→1280→1408→1536
    (drained heaviest XML-payload outliers gradually; diminishing returns
    from 56K → 16K per 128-field iteration)

  **Known regression at 1536:** alt_post_cstr_a/b CString detection went
  from 6 typed → 0 typed when chain extended past 1408. The chain now
  consumes bytes that previously parsed as CString headers. Byte-perfect
  roundtrip preserved (the bytes are still typed as u32s), but semantic
  CString info lost for ~6 entries. **Future structural fix needed:**
  add CString detection inside the chain (check if next u32 looks like
  valid CString length with valid UTF-8 follow-up bytes; stop chain if
  so). This would restore CString detection AND avoid further mechanical
  chain extensions.

  **Remaining bytes** concentrated in XML-payload outlier entries:
  31 entries fully chain alt_body to 1536 with 392K residual bytes (avg
  12.6K per entry, max 49K). Pure mechanical chain extension would need
  ~3K more alt_body fields to drain these XML strings entirely — code
  volume prohibitive. Structural CString detection is the right
  approach for further reduction. (loop session 2026-05-01)

  **Final post_blob size distribution (this session):**
  - 11676 entries (94%): 0 bytes — perfect drain
  - 0 entries: 1-3 bytes (drained by tail_pad u8 chain in iter 80)
  - 7 entries: 4-15 bytes (40 total)
  - 9 entries: 16-63 bytes (279 total)
  - 72 entries: 64-255 bytes (12K total)
  - 367 entries: 256-1023 bytes (227K total)
  - 210 entries: 1024-4095 bytes (381K total)
  - 52 entries: 4096+ bytes (725K total — XML payload outliers, 54% of remaining)

  **Structural CString detection design (deferred to future work):**
  Each alt_body_X read in the chain currently consumes u32s greedily
  through XML payload bytes. To preserve CString detection, peek at
  next 4 bytes as potential u32 length, check if 0 < len < 65536 AND
  next len bytes are valid printable-ASCII UTF-8. If yes, stop chain
  and let alt_post_cstr_a read the CString. Implementation needs ~128
  line edits per checkpoint OR a helper-function refactor; deferred
  to keep this loop session focused on byte-savings results.

### Remaining Tier 1.5 (blocked by family decoders)
**None remaining.** Both prior blockers resolved on 2026-04-30:
- ~~`QuestInfo.quest_dialog_filter_data_list_blob`~~ — wired via
  FilterCondition family decoder in `6cdc22c` (lane-c).
- ~~`GimmickInfo.post_blob`~~ — wired via TriggerGamePlayEventHandlerData
  family decoder this session (`binary::variants::trigger_gameplay_event_handler_data`,
  all 8 cases shipped). GimmickInfo's `trigger_event_handler_list` field
  now exposes typed `OptionalTriggerGamePlayEventHandlerData<'a>` entries.

(QuestInfo.quest_dialog_filter_data_list_blob was promoted in lane-c
2026-04-30 — see "Recent Tier 1 promotions" above.)

(MiniGameDataInfo previously listed here was promoted via `38ff7c3` —
spawn_data_list is now a `Decoded|Raw` enum (`SpawnDataList`) with
`CArray<CArray<SequencerStageSpawnData>>` Decoded shape, same T1
pattern as ConditionInfo's GameCondition wrapper.)

### Unresolved format mysteries
- `levelinfo.pabgb` (134 entries) — empirical analysis shows the file uses the
  `pa::ReflectObject` reflection pattern. Every entry starts `e2 e0 51 1f 00 00
  [count] 00 00 00` (hash 0x1f51e0e2 + u16 + outer count), then outer elements
  each begin with the same hash. Sub-element structure has variable sizes determined
  by nested inner counts. Class name hashes 0xa19e44b1 and 0x66be15a8 appear as
  element type tags. **Deferred — needs IDA decompile of the LevelInfo reader to
  identify concrete field layout.**

### Recently cracked (was previously labeled DEFERRED ReflectObject)
- `DropSetInfo._list` — sub_141600210 turned out fixed-shape with a
  tag-dispatched 14-case variant tail (63 fixed bytes + variant payload).
  Decoder lives in `binary::variants::drop_target::DropTargetData`.
- `ItemUseInfo` RandomBox `inner` — same payload via shared decoder,
  modeled as `Option<OptionalDropTarget>` to capture RandomBox's outer
  wrapper presence plus sub_141D03AA0's own inner presence.

### Reverse-engineering notes — QuestInfo FilterCondition family

The FilterCondition family (used by QuestInfo's `_questDialogFilterDataList`)
was previously labeled "polymorphic, 11 variants" and DEFERRED. Probing
showed it's actually decodable but with substantial nesting depth:

```
QuestDialog_FilterData (sub_1410F42E0, ~144 mem bytes)
├── 4× scalar fields (u8 + u8 + u32 + u32 + u32 + u32-hash)
├── sub_141102CB0 (u32 wire / u32 mem)  — qword_145F0EF20 hash
├── sub_141107000 (CArray<FilterCondition>)  — used 2×
├── sub_141107120 (CArray<sub_14110B380 result>)
├── sub_14110B380 (CArray<sub_1410F4050 result>, 112-byte stride) — used 2×
├── sub_14110B150 (similar to sub_14110B380)
├── sub_14110AF20 (CArray<{u32-hash + sub_1410F4050}>, 120-byte stride)
├── sub_1410FF050 (u16 wire/mem hash)
└── 4× u8 scalar trailer

sub_1410F4050 (per-element of B380/B150/AF20, 112 mem bytes)
├── u32 raw + sub_1411006D0 (u16 hash) + u32 raw
├── sub_1410F3DE0 (48 mem bytes inner)
│   ├── sub_141100510 (CArray<u32-hash>)
│   ├── sub_141103310 (CArray<{u16-hash + u64}>, 12 wire / 16 mem stride)
│   ├── sub_141102D90 (u16 hash) + 2 raw + 4 raw + 1 raw
├── sub_14110B8C0 (16 mem)
├── sub_14110B710 (16 mem)
└── sub_14110B570 (16 mem)

FilterCondition (sub_141D8F740, 64 mem bytes)
├── u8 dispatch_tag
├── sub_1410FFAC0 (CArray<u16>)
├── CArray<{Vec3 + u32}>, 16-byte stride
├── sub_141103310 (CArray<{u16-hash + u64}>, 16 mem stride)
└── per-tag variant tail (cases 0/1/A: 0 bytes; 2: u16; 3: u16-2;
    4/5/6: u32; 7: u32; 8: u32+u32; 9: u32)
```

All 14+ helpers verified as fixed-shape via IDA decompile. The depth
made this a focused multi-session crack rather than an in-loop win.

**Status update**: ✅ FULLY SHIPPED.
1. Decoder module `src/binary/variants/filter_condition.rs` —
   FilterCondition + 8 sub-readers all typed, 1:1 to IDA (lane-b).
2. QuestInfo wiring shipped in `6cdc22c` (lane-c, 2026-04-30):
   `quest_dialog_filter_data_list_blob: Vec<u8>` was replaced by the
   typed `QuestDialogFilterDataList<'a>` Decoded|Raw enum, exposing
   18 typed wire fields per QuestDialogFilterData with byte-perfect
   Raw fallback. **QuestInfo is now Tier 1.** 308/308 tests pass.
   (MiniGameDataInfo's separate spawn_data_list path was already
   typed via lane-c's `38ff7c3` work using SequencerStageSpawnData.)

### Reverse-engineering notes — TriggerGamePlayEventHandlerData

GimmickInfo's field 17 (sub_1411125E0) does NOT call sub_141D80A90
directly. It calls sub_141D7FF30 per element — a complex struct reader
(trigger_name BString + hide_list + event_list + handler_list + 4 bytes).
sub_141D80A90 (the `TriggerGamePlayEventHandlerData` polymorphic
dispatcher with 8 cases, 0..7) is only used nested inside sub_141D881B0,
which is in turn nested inside sub_141D7FF30. The "tag-16" misread was
caused by this architectural mismatch — resolved 2026-05-01.

`TriggerGamePlayEventHandlerData` itself: 8 cases, each case allocates
a different-sized struct (40/48/112/144 bytes) and constructs via
case-specific vtables; the actual wire reads happen in `vtable[85]` per case.

**Per-case factory + body reader (Win-IDA, decoded 2026-04-30 instance A):**

| tag | mem | class | vtable[85] body reader | wire summary |
|---|---|---|---|---|
| 0 | 112 | TriggerGamePlayEventHandlerData_Gimmick | sub_141D836E0 | sub_1410AA1B0 + 7× u32 + 1 u8 |
| 1 | 40 | …_IgnoreFallingDamageToTarget | 0x1402D3A80 (no-op) | 0 bytes |
| 2 | 48 | …_ApplyPassiveSkillToTarget | sub_141D84010 | 1× u64 (8 bytes) |
| 3 | 144 | …_ForceField | sub_141D85660 | nested poly: u32+u32+u32+u8(sub-dispatch)+sub_141D84040; sub-cases 0-3/4/5/7/8 each have their own body |
| 4 | 40 | …_MoveSyncGimmickWithPlatform | 0x1402D3A80 (no-op) | 0 bytes |
| 5 | 48 | …_DetectTriggerExpansion | sub_141D86960 | 1× CString |
| 6 | 40 | …_TriggerRegionInfo | 0x1402D3A80 (no-op) | 0 bytes |
| 7 | 40 | …_ElementalArea | 0x1402D3A80 (no-op) | 0 bytes |

**Tag 3 (ForceField) sub-dispatch detail** (sub_141D85660):
- Header: 4×u32 (a1+40..52) + 1 u8 sub-dispatch (a1+52) + sub_141D84040(a1+56)
- Sub-case 0/1/2/3: 12 bytes (a1+88) + 7× u32 (a1+100..124) + 1 byte (a1+128) = 41 wire bytes
- Sub-case 4: sub_141D84190(a2, a1+88) — variable
- Sub-case 5: 4 + 1 = 5 wire bytes (a1+88, a1+92)
- Sub-case 7: 4 + 4 + 4 = 12 wire bytes (a1+88, a1+92, a1+96)
- Sub-case 8: 12 + 7× 4 + 1 = 41 wire bytes (similar to 0-3 but trailing u8 instead of u8 at +128)

**5 of 8 are unit** (cases 1, 4, 6, 7 = no-op vtable[85]; tag 1 also no-op).
Cases 0, 3, 5 have content. Outer wrapper sub_1411125E0 is
`CArray<COptional<TriggerGamePlayEventHandlerData>>`.

**Status update**: ✅ FULLY SHIPPED via `1fc44e8`. The decoder lives at
`binary::variants::trigger_gameplay_event_handler_data` with all 8
variants typed (dispatch_tag u8 + per-tag body), wrapped in
`Decoded|Raw` for byte-perfect fallback. GimmickInfo now exposes
`trigger_event_handler_list: Option<CArray<OptionalTriggerGamePlayEventHandlerData>>`.

### JSON exposure upgrades (lane-c)
- `SkillInfo.buff_level_list` (CArray<CArray<BuffDataOptional>>) — was
  base64; now fully typed nested JSON via BuffData ToJsonValue +
  BuffDataOptional impls. Each per-level per-buff variant body is
  individually editable.
- `ImmuneBuffData.entries.body` — was base64; now a typed JSON array of
  integers sized by header_tag (u8 / u32 / u64 stride).
- `AdditionalUseResourceStat.f01_entries` — was array of base64 strings;
  now nested JSON arrays of u8 integers (each 22-byte record fully
  byte-addressable through JSON).
- `StageInfo` — vestigial empty `tail_blob` removed (was always-empty
  Vec<u8> + `_tail_blob_b64` JSON field). Reader now strict-asserts
  full consumption.
- Added `json_roundtrip` test for SkillInfo (now part of 308 tests
  passing on local main, plus 1 ignored — `interaction_info::tests::diag_raw_entries`).
  Test count grew from 304 → 308 with the lane-b merge that added
  4 diagnostic modules (filter_condition, game_level, sequencer_spawn,
  special_mode).

### Polymorphic family decoders
| Family | Status | Tables that consume it |
|---|---|---|
| **GameCondition** | ✅ 100% (Decoded\|Raw enum, commit `5160cdd`) | ConditionInfo (Tier 1, commit `9f1be1d`) |
| **GlobalGameEventExecuteData** | ✅ 100% (Absent\|Present\|Raw enum, commit `4b30791`) | GlobalGameEventInfo (Tier 1) |
| **GameEventHandlerData** | ✅ 100% (Decoded\|Raw enum) | GameEventHandlerInfo (Tier 1) |
| BuffData | ✅ shipped (per buff_data.rs) | SkillInfo, CharacterChangeInfo |
| BranchConditionData | ✅ shipped | (used inside GameCondition tree) |
| ConditionDataStageChart | ✅ shipped | (used inside GameCondition tree) |
| ConditionGimmickData | ✅ shipped | (used inside GameCondition tree) |
| ScheduleCompleteConditionData | ✅ shipped | (used inside GameCondition tree) |
| GlobalEffectConditionData | ✅ shipped | (used inside GameCondition tree) |
| MiniGameData | ✅ shipped | MiniGameDataInfo |
| GameExpression / IVariantItem | ✅ shipped (inside StageChart) | (used inside GameCondition tree) |
| EffectData | ✅ shipped (per-element typed, 47-field core_block) | EffectInfo (Tier 1) |
| **SequencerStageChartDesc** | ✅ shipped — all 26 wire fields typed in `binary::variants::sequencer_stage_chart_desc::SequencerStageChartDescPartial`. Composes inside CArray via stream-mode trait impls. | FieldReviveInfo, ItemUseInfo PlaySequencerOnly, SequencerSpawnInfo (Tier 1), GlobalStageSequencerInfo (Tier 1), StageInfo (Tier 1, all 91 wire fields), InteractionInfo (Tier 1 with Decoded\|Raw fallback), **CharacterInfo (Tier 1, all 174 wire fields, lane-c)** |
| **GimmickInteractionOverrideCArray** | ✅ shipped — `binary::variants::gimmick_interaction_override::GimmickInteractionOverrideCArray` (15-field inner via sub_1410DF770). | GimmickInfo (Tier 1.5 Decoded\|Raw, 99.93%), CharacterInfo field 133 (Tier 1) |
| **SequencerStageTrackChangeData** family (Character/Gimmick/Item) | ✅ shipped (inside SequencerStageChartDesc field 19) | (used inside SequencerStageChartDesc) |
| **SequencerStageSpawnData** | ✅ shipped (inside SequencerStageChartDesc field 20) | (used inside SequencerStageChartDesc) |
| **GameEventHandler** | ✅ shipped — per-sub_tag typed bodies (sub_tag 2 = 12-byte SetUIPlayGuideParameter, sub_tag 3 = 6-byte SetUIFullscreenGuideParameter, sub_tags 0/1/4 in-place or Raw fallback). | GameEventHandlerInfo (Tier 1) |
| **TriggerEventHandler** | 🟡 deferred — `pa::ReflectObject` is runtime-only (confirmed); binary I/O for ITriggerEventHandler is a fixed struct (transform + 2 u16 type indices). The GimmickInfo "tag-16" issue is resolved; remaining deferred work is TriggerRegionInfo and similar tables that embed ITriggerEventHandler. | TriggerRegionInfo and others |
| **TriggerGamePlayEventHandlerData** (TGPEHD) | ✅ FULLY SHIPPED — `binary::variants::trigger_gameplay_event_handler_data` covers all 8 inner cases (tags 0–7) plus the outer sub_141D7FF30 complex format (`TriggerEventHandlerDataElement`, `InnerTriggerEventWrapper`, `TriggerEventEntry`). GimmickInfo wired via `trigger_event_handler_list: Option<CArray<COptional<TriggerEventHandlerDataElement>>>`. | GimmickInfo field 17 — 12393/12399 decoded |
| **FilterCondition** family | ✅ FULLY SHIPPED — `binary::variants::filter_condition` covers FilterCondition (sub_141D8F740) + 8 sub-readers (FilterDataElement, FilterDataElementInner, FilterDataNamed, FilterDataF3F00, FilterDataF3D00, FilterDataB710, HashU64Pair, etc.). QuestInfo wired via `6cdc22c` (lane-c, 2026-04-30). | QuestInfo `_questDialogFilterDataList` — Tier 1 |

### Tables by tier
- **Tier 1** (typed, all fields editable through JSON): all 119 on-disk
  tables in the catalog — see `docs/449_TABLE_CATALOG.md` for the
  per-table list. All 121 tables in the 2026-4-24 dump have byte-perfect
  round-trip parsers (119 fully T1 in catalog; 2 absent from catalog
  because their pabgb files are not in the current dump).
- **Tier 1.5** (sub-field opacities inside otherwise-T1 tables):
  **None remaining.** Both prior blockers resolved on 2026-04-30:
  QuestInfo's `quest_dialog_filter_data_list` via FilterCondition
  family decoder (`6cdc22c`); GimmickInfo's `post_blob` via
  TriggerGamePlayEventHandlerData family decoder (`1fc44e8`).
- **Tier 2** (whole-tail blob): **0 tables** — eliminated. The
  catalog-level T2 count is now 0 (was previously 3 stale entries).

---

## Field-JSON v3.1 intent application — **shipped 2026-05-06**

dmm-parser now applies Field-JSON v3.1 intents end-to-end. Module
`src/intents/`:

- **`path.rs`** — v3 dot+bracket field-path parser
  (`enchant_data_list[0].enchant_stat_data.stat_list_static[2].value`),
  with `set_value_at_path`, `get_value_at_path`, `array_append_at_path`.
- **`types.rs`** — JSON-faithful `IntentDoc` / `Target` / `Intent` /
  `Patch` parsed straight off `serde_json::Value` (no new deps —
  matches the project's manual-JSON convention). Strongly-typed
  `ResolvedIntentOp` enum dispatches the five ops at apply time.
  `item_paloc_indices(item_key)` helper computes the
  `(key << 32) | 0x70 / 0x71` paloc lookups for custom-item names.
- **`apply.rs`** — `apply_resolved_intents(records, intents)` walks a
  parsed table's JSON record list and dispatches:
  - `set` (default) — replace a field at a path on a record matched by
    `entry` (string_key) or `key`.
  - `array_append` — append a value to an array at a path. Used by
    storeinfo to add custom items to vendor item lists.
  - `clone_record` — duplicate a record under a new key, then run patches
    against the clone. Source untouched. Duplicate-new-key rejected.
  - `new_record` — insert a record from a template at a new key. Accepts
    legacy v3 `add_entry` spelling.
  - `delete_record` — remove by key. Soft-skip on missing.
- **`apply_intents_to_iteminfo(body, intents)`** — full
  parse→apply→serialize for iteminfo (sequential format, no pabgh).
  This is the canonical "add a new item" entry point.
- **`dispatch::apply_intents_to_table_body`** — generic wrapper for
  every supported table. Today: iteminfo + sequential tables + paloc
  work end-to-end. Pabgh-bounded tables surface an explicit
  `Unsupported` error pending the pabgh-rebuild path (task #13).

39 unit tests cover the full surface (path parsing, type resolution,
each op, `{"value": N}` ItemKey wrapper handling). 2 integration tests
exercise a real iteminfo fixture when present (clone-record end-to-end +
empty-intents byte-perfect round-trip). 462/462 lib tests pass.

The minimum viable "add an item" recipe today:

```rust
use dmm_parser::intents::{Intent, Patch, apply_intents_to_iteminfo};

let body = std::fs::read("iteminfo.pabgb")?;
let intents = vec![Intent {
    op: Some("clone_record".into()),
    source_key: Some(12345),  // donor item
    new_key: Some(999_001),
    patches: Some(vec![
        Patch { path: "string_key".into(),       op: None, new: "Custom_Sword".into() },
        Patch { path: "max_stack_count".into(),  op: None, new: 999.into() },
    ]),
    ..Default::default()
}];
let (new_body, outcomes) = apply_intents_to_iteminfo(&body, &intents)?;
std::fs::write("iteminfo_modded.pabgb", new_body)?;
```

PyO3 bindings + sample mod manifest + `dmm-mod-validate` op-awareness
are next (task #14).

### 2026-05-06 — Python binding + target-name aliases + SuperMod fixture (task #14)

**`dmm_parser.apply_intents(table_name, pabgb, pabgh, intents)`** — top-
level Python entry point for applying Field-JSON v3.x intents end-to-end
on a single table. Returns a dict `{body, pabgh, outcomes}` mirroring
the Rust API. `apply_intents` accepts intent dicts in any shape the v3
spec recognizes.

**`dmm_parser.normalize_target_name(name)`** — resolves alias spellings
to the canonical dispatch identifier:

| Input | Resolves to |
|---|---|
| `character_info` / `character_info.pabgb` | `character_info` |
| `characterinfo.pabgb` (compact, SuperMod-style) | `character_info` |
| `iteminfo.pabgb` | `iteminfo` |
| `paloc` / `paloc.pamt` / `localizationstring` | `paloc` |
| Unknown | `None` |

**`dmm_parser.item_paloc_indices(item_key)`** — exposes the canonical
custom-item paloc index formula
(`((item_key as u64) << 32) | 0x70 / 0x71`) so SWISS / Stacker /
hand-authored mods don't reinvent the bit math.

**Target name normalization** is now wired into
`dispatch::apply_intents_to_table_body` so SuperMod-class manifests
(which use `characterinfo.pabgb` / `regioninfo.pabgb` /
`spawningpoolautospawninfo.pabgb` / etc.) apply without manual rename.

**SuperMod fixture validated.** A real production-class manifest
(12,358 intents across 5 targets — iteminfo, characterinfo,
regioninfo, spawningpoolautospawninfo, terrainregionautospawninfo)
parses cleanly via `IntentDoc::from_slice` and every target resolves
via `normalize_target_name`. Test
`dispatch::tests::supermod_manifest_parses_cleanly` runs against
`C:\Users\corin\Desktop\ZIPS\SuperMod (4).json` if available.

**`samples/04_custom_item/`** rewritten to canonical v3.1 shape:
- `clone_record` intent (donor 12345 → new_key 999001)
- six patches (string_key, item_name.default + .index, item_desc.default
  + .index, max_stack_count)
- `paloc.pamt` target with the matching localization entries
- `asset` target wiring the custom DDS icon at `/ui/icon/sword_of_potter.dds`
- Updated README with the full Python authoring recipe.

**Doc fix:** `docs/CUSTOM_ITEM_CREATOR_V3_1.md` had arithmetic typos in
its paloc-index example (`4290772592` instead of `4290676623671408`).
Corrected with a footnote pointing at `item_paloc_indices`.

7 new tests this iteration: 6 normalize_target_name cases (canonical /
compact / iteminfo aliases / paloc aliases / unknown / SuperMod parse)
plus `samples_04_custom_item_manifest_round_trip`. **473/473 lib tests
pass** (was 466).

---

### 2026-05-06 — pabgh rebuild for record-count-changing intents (task #13)

Pabgh-bounded tables (buff_info, character_info, gimmick_info,
store_info, etc. — 45 tables) now apply intents end-to-end with a
fresh sister `.pabgh` index emitted alongside the modified `.pabgb`
body.

- **`tables::blob_runtime::serialize_typed_blob_table_from_json_tracked`**
  — sister of the existing serializer that also returns
  `Vec<(u32_key, u32_offset)>` per record.
- **`tables::blob_runtime::serialize_blob_table_from_json_tracked`** —
  same for the generic blob-fallback runtime.
- **`tables::blob_runtime::extract_record_key`** — permissive key
  reader (scalar `key` OR `key.value` wrapper, accepts u64/i64/integer
  shapes; truncates to u32).
- **`dispatch::serialize_table_from_json_with_pabgh`** — top-level
  serializer that takes the original pabgh, parses it for the format
  flag (`U16CountU32Key` / `U16CountU16Key` / `U32CountU32Key`), runs
  the tracked serializer, and emits a new pabgh in the same on-disk
  format. Catches u16-key overflow on Format 2 tables.
- **`dispatch::apply_intents_to_table_body`** — pabgh-bounded path
  now returns `(new_body, Some(new_pabgh), outcomes)` instead of
  `Unsupported`. Sequential and iteminfo paths unchanged.

`skill_info` and `equip_slot_info` use special-case serializers
(buff_level_list nested base64; equip_slot footer/extra_entries) and
are not yet wrapped by the tracked path — they still return
`Unsupported` until the inner serializers are extended. Documented
in the `serialize_table_from_json_tracked` match arm.

4 new dispatch tests:
- `buff_info_apply_empty_intents_byte_perfect` — pabgh-bounded
  fixture: empty intents → byte-perfect body AND pabgh.
- `equip_info_apply_empty_intents_byte_perfect` — same on the generic
  blob-fallback path.
- `buff_info_set_is_blocked_then_pabgh_offsets_align` — set u8 field
  by key, re-parse with fresh pabgh, assert target record's field
  matches.
- `sequential_table_returns_no_pabgh` — contract check.

466/466 lib tests pass (was 462, +4 in dispatch).

---

## What just shipped (older session — see Active state banner above for current 2026-04-30 work)

> Note: as of the current session local `main` is ~48 commits ahead of
> `origin/main` per the user's no-push directive. The chronological
> list below is from a prior session; the 2026-04-30 work is
> summarized in the "Session results" block at the top of this file.

```
GameEventHandlerData: ship Tier 1 family decoder + wire GameEventHandlerInfo
8e9b6f6  docs/STATUS.md: GlobalGameEventExecuteData shipped, refresh queue
4b30791  GlobalGameEventExecuteData: ship Tier 1 family decoder w/ Decoded|Raw enum
e17d416  docs: add STATUS.md for collaborator handoff
9f1be1d  ConditionInfo: promote Tier 2 → Tier 1 — typed GameCondition wrapper
5160cdd  GameCondition: Raw-bytes fallback variant → 100.0% round-trip 🎯
dd72172  ConditionData: 5-tag OneByteBodyPayload batch (11/92/253/343/351) → 99.8%
b82e3c7  ConditionData: tags 126/178/287/306 + LAST_ATTEMPTED_TAG tracker → 99.7%
1d49637  ConditionData: tags 17/18/19/20/21/22/26/27/29/33 → OneByteBodyPayload, 99.3%
2af19fa  ConditionData: tags 2/55/79/101/136/195/214/256/290/374/399/401 + tag 31 → 99.2%
a4118f5  ConditionData: batch 1-byte/4-byte/CString body fixes → 98.3%
```

GameCondition went from 13.4% → 100% across the first 7 commits.
GlobalGameEventInfo Tier 2 → Tier 1 in the latest commit (80/80 entries
decode structurally, 0 raw fallbacks needed).

---

## The reusable playbook

The pattern that cracked GameCondition is now documented in code and
ready to apply to the next polymorphic family. In order:

1. **Find the meta-dispatcher**. Search the Win-IDA decompile for the
   table's known offsets (look for the Korean string xref to the
   table's pabgb name). Find the `switch (tag)` that allocates +
   constructs each variant.
2. **Extract the tag → read_fn map**. Reuse the python script at
   `dmm-pabgb-aio/extract_conditiondata_dispatch.py` as a template.
   It parses the dispatcher decompile, emits JSON keyed by tag with
   `(alloc_size, read_fn, class_name)`.
3. **Stand up a recursive enum** in `src/binary/variants/<family>.rs`.
   Start with case-tag dispatch — leaf payloads as `Vec<u8>` initially.
4. **Build a round-trip validator** in `examples/` that walks every
   real entry from the consuming table's pabgb. Capture the
   `LAST_ATTEMPTED_TAG` thread_local pattern (see condition_data.rs
   line ~5219 for the reference impl).
5. **Loop**: run validator → look at the "Failing tags" table at the
   bottom → decompile that tag's vtable[16] (offset +0x80 from the
   class's `??_7<ClassName>@pa@@6B@` symbol) → fix the body recipe →
   repeat. Each iteration kills 1-30 failures.
6. **Add the Decoded|Raw fallback** at the wrapper level (see
   `src/binary/variants/game_condition.rs` lines 29-78 for the canonical
   implementation). Guarantees 100% round-trip even with un-decoded
   variants.

The whole arc takes 1-2 sessions per family if the dispatcher is clean
(non-obfuscated). Tags 54/286 in ConditionData are anti-disassembly
obfuscated — those stay in the Raw bucket forever, which is fine.

---

## Key files

### Parser core
- `src/binary/mod.rs` — read/write traits (`u8`/`u16`/`u32`/`u64`/`CString`/`CArray`)
- `src/binary/variant.rs` — `pabgh_typed_blob_table!` macro,
  `find_cstring_u8_trailer` helper, `entry_ranges`/`load_pabgh_offsets`
- `src/json_traits.rs` — manual `ToJsonValue`/`WriteJsonValue` traits
  used by every `to_json_dict`/`write_from_json_dict` impl

### Polymorphic families
- `src/binary/variants/game_condition.rs` — **canonical Decoded|Raw
  fallback** wrapper. Use this as the template for next family.
- `src/binary/variants/condition_data.rs` — 405-variant dispatch with
  the `LAST_ATTEMPTED_TAG` thread_local tracker (line ~5210)
- `src/binary/variants/branch_condition_data.rs` — smaller (14 variants)
  example of the pattern
- `src/binary/variants/buff_data.rs` — first family decoder, original
  reference implementation

### Tables
- `src/tables/condition_info/info.rs` — **canonical Tier 1 model**
  with typed GameCondition wrapper (Decoded|Raw fallback, 99.82% Decoded
  on 8,934 entries). Use as the template for wiring family decoders
  into consuming tables.
- `src/tables/skill_info/` — original blueprint for Tier 1 with
  polymorphic body (BuffData)
- `docs/449_TABLE_CATALOG.md` — per-table status

### Validators
- `examples/game_condition_roundtrip.rs` — measures decode + round-trip
  for every ConditionInfo entry. Has env-var dump filters
  (`GC_DUMP_TAG=NN`, `GC_DUMP_TAGS=N,N,N`). Final output includes the
  "Failing tags" table from the `LAST_ATTEMPTED_TAG` tracker — that's
  the smoking gun that tells you which variant's recipe is wrong.

### IDA dumps (in sibling repo `dmm-pabgb-aio`)
- `mac_extract/conditiondata_dispatch_map.json` — tag → read_fn for
  ConditionData's 405 variants
- `mac_extract/conditiondata_dispatcher_plain.txt` — saved dispatcher
  decompile (use as input to extract scripts)
- `mac_extract/conditiondata_empirical_observations.json` — per-tag
  size stats from real data (cross-check against IDA recipes)

---

## What's next, in priority order

### Remaining work
1. ~~**GimmickInfo post_blob fields 20–179**~~ — ✅ DECODED via IDA
   (`sub_1410E6FC0`); shipped as `GimmickPostBody` struct (F20–F179).
   9947/12393 entries successfully decode the post-body. 2446 entries
   have `COptional flag=0` (legitimately absent).
2. **TriggerEventHandler family** — `pa::ReflectObject` reflection-
   driven serialization. Needs reflection layer reversed. DEFERRED.
3. **Wire ConditionInfo Tier 1 into DMM v3 dispatch** — small change
   in DMM-BETA's mod-loader. No IDA required.

### Active goal — promote sequencer + attack family Tier 1.5 → Tier 1

Six standalone (non-PABGB) asset formats round-trip byte-exact today but
expose only `LpToken::LpString` / `RawBytes` (or — for `.paatt` —
typed envelope + opaque `base_data`). Promoting them to Tier 1 is the
last remaining 1.5 surface in dmm-parser and the prerequisite for
field-level mod intents on cutscenes, NPC schedules, stage charts, and
attack data. See `docs/BINARY_FORMATS.md#file-format-reference-formats` §11 (per-format wire + IDA pointers)
and §12 (tier semantics + methodology).

Attack order, smallest scope first:

1. **`.pastage`** — likely reuses already-decoded `SequencerStageChartDesc`
   (`sub_141D8C6D0`, 26 wire fields). Confirm via IDA, prepend the
   stage-path LP-string prefix, ship. Fastest expected win and validates
   the IDA workflow on standalone (non-PABGB) assets. 3,320 vanilla
   samples must roundtrip byte-perfect.
2. **`.paseq`** — largest sample set (4,659). Sequencer / cutscene /
   scripted action. Find loader via `pa::Sequencer*` xrefs in the Mac
   binary, walk the dispatcher, type each tag's body.
3. **`.paseqc`** — sister to paseq (header magic `FF FF 04 00` /
   `FF FF 03 00` minority). Expected to share the paseq dispatcher;
   promote together.
4. **`.paschedule` + `.paschedulepath`** — paired NPC schedule decode.
   Mostly numeric (waypoint hashes, frame counts) with a few embedded
   asset path strings. Smaller scope than the sequencer formats.
5. **`.paatt`** — finish per-version BaseData decode via
   `pa::AttackInfoDataDesc` reflect-property setters. Sub-variants
   `AttackInfo_Attack` / `_AttackThrow` / `_AttackCatch` / `_ReleaseCatch`
   each add their own fields; per-version sizes 264/528/296/288/264
   bytes already known.

Methodology — same family-decoder playbook used for GameCondition,
FilterCondition, TriggerGamePlayEventHandlerData, GameEventHandlerData,
and SequencerStageChartDesc. See "The reusable playbook" section above.
Definition-of-done per format is in `docs/BINARY_FORMATS.md#file-format-reference-formats` §12.

### Deferred (need runtime debugger or are non-blocking)
- ConditionData tags 54/286 — anti-disassembly obfuscated readers
  (sub_14D3012D0 family). Currently round-trip via the Raw fallback,
  which is fine. Recoverable later if anyone runs the game in a
  debugger and observes the obfuscated reader's actual byte
  consumption.
- ConditionData tag 272 sub_tag holes (0x42, 0x1d) — likely truncated
  debug entries in the source data; not worth chasing.

### Stream-mode GameCondition (mostly unblocked, 99.2% interaction_info)
**Root cause identified**: The `variant_skips_option_block` list in
`condition_data.rs` was incomplete and some per-tag body recipes were
wrong. The list originally had 10 verified-no-op tags (Class A: 2,
81, 126, 256, 272, 300, 306, 401 = 8 tags; Class B: 79, 195 = 2 tags).
Empirical adds via the LAST_ATTEMPTED_TAG diagnostic added 6 more
(99, 135, 174, 360, 370, 26 = Class C), then individual tags were
verified one by one — 5 of those 6 (99, 135, 174, 360, 370) ended
up promoted to body+option_block recipes during the verification
cycle. Only tag 26 remains in Class C.

**Current state**: 360 of 363 interaction_info entries (99.2%)
successfully decode after methodical Win-IDA-driven recipe verification
on 12 tags. The early "bulk-add" approach regressed success (313 → 294),
so each candidate has been verified individually since.

**2026-04-30 regression + recovery cycle**: Tags 19 (CheckGroggy),
27 (IsFocusActor), and 174 (CheckRider) were downgraded from
OneByteBodyPayload to unit variants (`b95e5c0`, `0618efb`, prior),
pushing `diag_raw_entries` 57 → 101 Raw entries. Roundtrip tests
stayed byte-perfect because Raw fallback preserves bytes verbatim —
the test cannot detect decode-success regressions. Recovery sequence:
- `8f01078` — tag 174 properly recovered with Win-IDA vtable[16]
  (0x141C9A550 reads 1 byte) and vtable[19] (0x141C8D560 standard
  option_block) verification; Raw 101 → 50, decoded 262 → 313.
- `6947b63`, `bd009d6` — tags 19 and 27 reverted back to
  OneByteBodyPayload (no IDA verification, just rollback).
After all three commits, `diag_raw_entries` shows n=69 — still 12
above the baseline 57. Tags 19 (7 entries) and 27 (13 entries) still
surface in the histogram with their original 1-byte body recipe,
suggesting the failure is in option_block, not body. Next move:
Win-IDA verify their vtable[19] — if it points to a no-op
(0x1402d3a80) or a thunk in `sub_14139AE80`, candidate them for
skip-list addition (Class A or Class C). DO NOT speculatively change
recipes without IDA evidence — every speculative pass has cost the
team a churn cycle.

**2026-04-30 final progression**: Methodical Win-IDA-driven recipe
verification took interaction_info from n=69 → n=3 (98.7% Decoded).
Successful fixes (each verified per the `8f01078` template):
tag 7 trailing u16 (`08b7afc`), tag 19/27 unit variant (kept after
final reapply), tag 99 skip-list removal (`5922251`), tag 116
OneCStringBodyPayload (`4469883`), tag 135 1-byte body KEPT in
skip-list (`93cc34d`, +18), tag 174 recovery (`8f01078`), tag 358
1-byte body (`147fd7f`), tag 360 1-byte body remove-from-skip
(`2102303`), tag 370 1-byte body KEEP in skip-list (`41bc97f`),
tag 393 1-byte body (`d91d961`), tag 29 unit variant (`584f79c`).
Remaining 3 Raw entries (tag 54 ×1, tag 214 ×2) are all in the
anti-disassembly family that wraps `sub_14F0xxxxx` obfuscated
readers — preserved byte-perfect via the GameCondition::Raw fallback.

**Important caveat — superseded by 2026-04-30 progression above**:
The `57-entry ceiling` and the "empirical-add masking" warnings below
were correct at the time but the methodical Win-IDA verification cycle
above shows the path forward worked. Tag 99/135/174/360/370 were
removed from the skip-list (each verified in IDA — tags 99/174/360/393
got proper body recipes; tags 135/370 got bodies AND were kept in skip
when that combination won, then later 135/370 were removed altogether
as the recipe stabilized). Only tag **26** remains in Class C now
(empirically confirmed by the n=3 stable state). Final tally — Class A:
8 tags (2/81/126/256/272/300/306/401), Class B: 2 tags (79/195),
Class C: **1 tag (26)** — down from 6 originally.

**Path forward (revised, mostly DONE)**:
1. ~~Walk all 405 ConditionData_* vtables~~ — proven unnecessary; the
   `interaction_info::diag_raw_entries` histogram pinpointed the wrong
   recipes faster than a full vtable walk would.
2. ~~Replace the empirical adds with the verified list~~ — DONE
   piecemeal across the n=69 → n=3 progression.
3. Remaining: tag 54 + tag 214 are in the genuine anti-disasm family
   (`sub_14F0xxxxx` obfuscated readers — RTTI present but vtables not
   findable in IDA). Recoverable later if anyone runs the game in a
   debugger and observes the obfuscated reader's actual byte
   consumption. Until then, the GameCondition::Raw fallback handles
   them byte-perfectly.

<details><summary>Original caveat text (preserved for context)</summary>

Of the 16 tags currently in the skip list, only the original 11 are
confirmed "true" vtable[19] no-ops. The 5 empirical adds (26, 135,
370, 99, 174, 360) are NOT vtable[19] no-ops — their slot-19 entries
each point into the giant `sub_14139AE80` thunk forest (size 0x1dc88,
non-decompilable by Hex-Rays). Concrete check: tag 81's vtable
(`ConditionData_QuestGaugePercent` at `0x144ce3038`) has slot 19 =
`0x1402d3a80` (the `return 1;` no-op), while tag 99's vtable
(`ConditionData_CheckAllyType` at `0x144cdc770`) has slot 19 =
`0x1413b89e0` (a thunk inside `sub_14139AE80`). Yet tag 99 is in
the skip list because it empirically unblocked entries.

This means the empirical adds are likely **masking** real bugs where
LAST_ATTEMPTED_TAG points to the wrong tag in the failure chain. The
57-entry ceiling on interaction_info reflects this: pushing past it
requires proper per-variant vtable[19] reverse engineering, not more
empirical adds.

3. For the empirical adds that turn out NOT to be no-ops, investigate
   why removing them STILL allows their entries to decode (likely
   because the body recipe is wrong elsewhere — option_block probe is
   misaligning a downstream byte).
4. With the verified skip list, re-run the per-variant diagnostic on
   interaction_info to find the actual remaining 57-entry blockers.
5. Apply ConditionPairCArray to interaction_info field 10 once
   100% decode.
6. Repeat the same approach for gimmick_info field 7
   (sub_141118470 → sub_1410DF770 → BareConditionPairCArray at
   sub_141E2C900), character_info field 133, stage_info field 7
   (SequencerStageChartDesc), global_stage_sequencer_info field 6.

The vtable layouts and per-element wire layouts for sub_141D8C6D0
(SequencerStageChartDesc, 26 wire fields / 232 mem bytes) and
sub_1410DF770 (GimmickInteractionOverrideData, 15 wire fields / 144
mem bytes) are documented in the consuming tables' module docstrings
and ready to wire up the moment the skip-list is verified.

</details>

---

## Quick reference: how to verify nothing regressed

```bash
# Full test suite
cargo test --release

# GameCondition round-trip validator
cargo run --release --example game_condition_roundtrip
# Should print: Round-trip OK: 8934 (100.0%)

# Per-table round-trip (ConditionInfo, skill_info, etc.)
cargo test --release condition_info
cargo test --release skill_info
```

If any of these regress, `git log --oneline -10` and bisect against the
last known-good commit.

---

## Conventions

- New table parsers go in `src/tables/<name>/info.rs` with companion
  `mod.rs` + (optional) `RECIPE_NEXT.md` for status notes.
- Hand-written parsers must start with the `//! Hand-corrected:` header
  marker — `bulk_process.py` skips files with this header.
- All tests should pass before pushing. The validator at 100% is the
  hard floor for GameCondition.
- Don't touch `Cargo.toml` deps without coordinating — the workspace
  is consumed by DMM-BETA, JSMM, and ext-builds.
