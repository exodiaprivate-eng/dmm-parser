# Blob-runtime table RE findings (1.11) — overnight session 2026-06-12

Branch: `parser-blob-RE-1.11`. Goal: make the 7 remaining deep-polymorphic
tables byte-exact roundtrip (they currently soft-degrade to blob-fallback in
the V3 path, so they WORK in production; the failing unit tests document the
incomplete typed parse). IDB available: `1.11/CrimsonDesert_Steam.exe`
(md5 67bcc1cd…) at base 0x100000000 via HTTP JSON-RPC :13337 — **stripped**
(sub_ names only; no `pa::` symbols, no field-name strings found), so the
fast path is **data-driven wirewalk** (decode many records, find the layout
that makes all of them consume to residual 0), not IDA symbol lookup.

Helper added: `dmm_probes/ida.py` (RPC: strings/xrefs/decompile/func/err).
Worked example: `dmm_probes/walk_itemuse.py`.

## ★ item_use_info — BASE FULLY CRACKED (in this branch)

Two **data-verified** base bugs (vs the stale v1.0.4.x doc map in info.rs):
1. `BaseUseData` had a spurious `unk_new_flag: u8` → base fixed-prefix was 19,
   real 1.11 is **18** bytes (4×u8 + 2×u32 + **2**×u8 + u32). REMOVED. ✔ committed
2. `BaseUseDataElem` was missing a **trailing u32** after the LocalizableString.
   ADDED `tail_lookup: u32`. ✔ committed
   (LocalizableString = category u8 + index u64 + CString.)

Proof: `walk_itemuse.py` decodes the base+elements of **all 9676 records to
residual 0** with these two fixes (base_fail=0/9676).

### Per-variant 1.11 extra-byte map (post-base residual) — the v1.0.4.x doc is WRONG
disc → residual size (records) → real layout [current struct → needed]:
- 0  Skill              12B (3656)  → 3×u32   [SkillPayload u32+u32 → +u32]
- 1  ExpandInventorySlot 4B (4)     → u16+u16 [OK]
- 2  RandomBox          8/12/76B (437) variable → existing read_with_size(extra) opaque-ish; re-verify consumes now base is right
- 3  SummonGimmickWithCatch 49/52/54B (149) variable (2 CStrings) → existing struct; re-verify
- 4  ConvertCharacter    8B (34)    → 2×u32   [ConvertCharacterPayload u8 → 2×u32]
- 5  ItemDye             1B (1)     → u8      [ItemDyePayload u32+u8 → u8]
- 7  FeedVehicle         4B (8)     → 1×u32   [reuses SkillPayload(12) → needs own 1×u32 payload]
- 8  DestroyOnly         9B (1155)  → u32+u32+u8 [currently base-only → add payload]
- 9  SealToEquip         0B (26)    → base only [OK]
- 10 TeleportRevivePoint 0B (370)   → base only [TeleportRevivePointPayload 2×u32 → make base-only]
- 11 Projectile          8B (1)     → 2×u32   [OK]
- 13 CustomizeCharacter  1B (3)     → u8      [u32+2×CArray → u8]
- 14 PlaySequencerOnly   14/16/20B (6) variable → existing SequencerStageChartDescPartial(extra); re-verify
- 15 RegisterReserveSlot 140-300B (1273) DEEP nested+strings → current u32+u8 WAY off; opaque-bounded tail recommended
- 16 OpenUI              5B (1561)  → u8+u32  [OpenUIPayload u8+CString → u8+u32]   ← first failure (over-read)
- 17 Inspect             19/20B (12) variable (trailing CString) → currently base-only → type or opaque
- 18 InventoryBuff       0B (873)   → base only [OK]
- 19 SendEventToDockingGimmick 0B (2) → base only [OK]
- 20 UseSealed           0B (95)    → base only [OK]
- 21 UnSealFromEquip     0B (10)    → base only [OK]
- (6 SubLevelUp, 12 ExpandFarmSlot, 22 SpecialMode: 0 records in vanilla — leave)

NEXT STEP for item_use_info: apply the per-variant struct sizes above (typed for
the fixed ones, opaque entry-bounded tail for 15/17 and any variable one that
doesn't re-verify). All external deps are on top-level `ItemUseInfo` only
(dispatch p!/d!/dt! + tracked) — the variant payload structs are internal, so
edits are contained to info.rs. Roundtrip needs only size-correct symmetric
read/write (py_binary_struct is symmetric), so exact field semantics aren't
required for the test to pass.

## Other 6 tables — first-failure entry points (same class: a field-width drift
## mid-record misaligns a downstream CArray count). Each needs a per-table wirewalk
## (clone walk_itemuse.py: decode the record header + known fields, find where the
## byte boundary drifts vs 1.11 data).

- knowledge_info        (info.rs:314)  e0 k=0xf43e8: bogus CArray count 909521969 @off 2176 (deep record). 3 sub-tests fail incl typed_lists_populated.
- mini_game_data_info   (info.rs:400)  entry 1 k=0x4240 (1785B): bogus CArray count @off 152. Record 0 OK → drift is in a field present in rec1 not rec0.
- mission_info          (info.rs:301)  e0 k=0xf4282: "not enough data" (under-read → a field is wider in 1.11 than modeled).
- quest_info            (info.rs:622)  e0 k=0xf44de: bogus CArray count @off 1046147 (very deep — FilterCondition family region likely).
- sequencer_spawn_info  (info.rs:245)  entry 0 k=0x3e9 (1117B): "not enough data".
- skill_info            (info.rs:700)  e245 k=0x1250b & e1295: bogus count 16777216=0x01000000 (= 3 zero bytes + 0x01 → off-by-1..3; a specific skill variant has an extra small field). Most records parse; only certain variants drift.

skill_info is the closest to done (244 records parse before the first failure;
it's a specific variant shape). quest_info/knowledge_info/mission_info are the
deepest (large polymorphic families — FilterCondition / GameCondition).

## ★ STRATEGIC CONCLUSION (after fixing item_use_info)
item_use_info was tractable because it has a clean **typed-base + trailing-
variant** boundary → typed base + opaque entry-bounded tail = byte-exact, done.
The other 6 are NOT like that: each embeds a deep SHARED polymorphic family
**mid-record** (followed by more typed fields), so an opaque tail at record-end
does NOT apply. The real blocker is the shared family decoders drifting in 1.11:
  - `SequencerStageChartDescPartial`  → sequencer_spawn_info (+ item_use disc 14)
  - `SequencerStageSpawnData` → `OptionalGameCondition`/`GameCondition` (≈405 variants)
       → mini_game_data_info (+ stage/field_revive/global_stage_sequencer)
  - `FilterCondition` family → quest_info
  - `GameCondition`/`ConditionData` → knowledge_info / mission_info / skill_info
These families use a Decoded|Raw fallback; the roundtrip test fails because the
Decoded path mis-sizes a variant in 1.11 (the drift is almost certainly small —
a few variants gained/lost a field, exactly like the simple-table 1.11 fixes:
+u8 / u32→CArray). **Highest leverage: fix the family decoder, not per-table.**
A family fix likely clears multiple tables at once. Method: build a family-level
wirewalk (decode each variant arm; find the disc/offset where the byte boundary
drifts vs 1.11 data), then adjust that variant's body reader. This is focused
multi-session RE, NOT a safe single unattended pass — attempting blind edits to
405-variant families risks shipping subtly-wrong decoders, so it was left for a
dedicated session with this map in hand.

## RESULT THIS SESSION
- item_use_info: FIXED, byte-exact (commit ab1c3b5).
- skill_info: FIXED, byte-exact (commit 800c658). SummonBuffData (BuffData
  tag 10) under-read by exactly 7 trailing bytes in 1.11 — the outer reader
  (sub_1419DD000) gained a 7-byte block after u32_outer_384. Added
  u8_outer_388 + u16_outer_389 + u32_outer_392 to SummonBuffDataPayload.
  Method: BUFF_DIAG instrumentation localized tag 10; data-driven wirewalk
  (dmm_probes/walk_skill{,2,3}.py) proved the true buff end is exactly where
  SkillInfo.skill_group_key (=own key) begins, +7 vs the old model.
- sequencer_spawn_info: FIXED, byte+JSON exact (commit de84423). **Shared
  GameCondition-family fix**: ConditionData tag 131 (ConditionData_IsInRegion-
  Type) was modeled as a pure-discriminator but was NEVER exercised by any
  validated table until 1.11's sequencerspawninfo. It actually carries a
  `CArray<u16>` body (same shape as sibling tag 128 IsInRegion). Added
  ConditionData_IsInRegionTypePayload and wired all 7 dispatch sites
  (enum/disc/name/to_json/write_to/read_from/write_from_json). NOTE: the
  write_from_json arm is easy to forget — leaving `131 => {}` there passes
  byte-roundtrip but fails JSON-roundtrip (and regresses condition_info json).
- Full suite 626→632 pass, 9 fail (was 15). Method that worked on the family
  drift: the built-in ALLCASE=1 / ALLDISC=1 env tracers in game_condition.rs
  + condition_data.rs print every GameCondition case_tag / ConditionData disc
  as it's read — run on a failing table's roundtrip, the LAST disc printed
  before the panic is the culprit variant.
- Tooling: dmm_probes/ida.py, walk_itemuse.py, walk_skill{,2,3}.py.

## REMAINING 4 (knowledge / mini_game / mission / quest) — table-specific drifts
Confirmed via ALLCASE/ALLDISC: **none of these 4 fail inside the shared
GameCondition tree** — each fails in the table's OWN typed fields (a CArray
count misreads early and propagates). So each needs its own per-table
wirewalk (no shared-family shortcut).

### mini_game_data_info — TWO distinct drifts, partially cracked
1. `MiniGameParticipantData` is +2 bytes in 1.11: an added u16 between
   `value_dword` and `spawn_keys` (CArray<u32>). VALIDATED on the big records
   (e1 k=0x4240, e2 k=0x4246 pass with the +2). This fix is real — re-apply it.
2. BUT the small "stub" mini-game records (e3 Fishing/e4 Rodeo/e9-e17, sizes
   68-86, all with script/phase/ui = sentinel 0xeac5e173 "unresolved" hashes)
   have a SECOND, VARIABLE header drift: player_data_list count reads garbage
   (0x10000 / 0x1000000), and the misalignment amount differs per record
   (−2 for some, −3 for others) → tied to string length / per-record
   conditional. A single fixed-layout model can't satisfy both the big and
   stub records. NEEDS: decode several stub records' headers byte-by-byte to
   find the conditional/format field that shortens them (candidate: a
   presence-gated field after one of the sentinel hashes, or a record-format
   byte). The +2 participant fix was REVERTED to keep the tree clean until the
   stub-record layout is understood (committing it alone leaves the test red).
   Probe: a full-header Python mirror across all 24 entries is in this
   session's transcript (reports which entries align).

### mission_info  — TWO+ drifts, partially cracked (probe: walk_mission.py)
MissionInfo has NO embedded GameCondition — all nested structs are simple
(MissionBranchData 19B, MissionExecuteStage 18B, MissionResultData 35+4n,
TriggerVolumeData, MissionUIDesc, 4×LocalizableString, MissionResultData2,
MissionStageData). Fully wirewalk-able in Python (walk_mission.py mirrors it).
1. **MissionUIDesc is +4 bytes in 1.11** (a trailing u32 after `trailing:u16`).
   VALIDATED: with the +4, e0-e38 pass (the label[0] LocalizableString — cat u8
   + idx u64 + cstr len 19 — only aligns at ui_desc_end+4). Re-apply this.
2. BUT a SECOND drift hits ~4677/6817 records (clustered ranges e39, e695-711,
   e763-810, …) — all fail at `ui_desc.list_a` count = garbage, i.e. a field
   BEFORE ui_desc is mis-sized for a whole CATEGORY of missions (likely
   MissionResultData or result_data_list elements gained a field in 1.11;
   e0-e38 had empty/small result_data so didn't surface it). NEEDS: wirewalk a
   failing record (e.g. e39 k=0xf4292) through result_data_list element-by-
   element to find the per-element +N. The ui_desc +4 was REVERTED to keep the
   tree clean (committing it alone leaves the test red since 4677 still fail).

### quest_info  (info.rs:622) e0 k=0xf44de bogus count @1046147 (very deep)
Own fields + FilterCondition family. Deepest offset → a count misreads early
and the cursor runs far past the record.

### knowledge_info  (info.rs:314) e0 k=0xf43e8 bogus count 909521969 @2176
Own fields (fails before reaching any GameCondition tree — ALLDISC silent).

## Method recap (NattKh, adapted for the stripped 1.11 IDB)
1. `cargo test --lib tables::X::info::tests::roundtrip` → first-failure record/offset.
2. Clone `walk_itemuse.py` for table X: decode header + fields, dump residual-by-
   group, find the byte where the boundary drifts vs the data.
3. If IDA needed: `ida.py strings "<Field>"` (Korean `…를 읽어들이는데 실패했다`
   error strings) → `ida.py xrefs <addr>` → `ida.py decompile <reader>` → field
   widths (vtable 3rd arg = byte count: F4C=2/u16, F2C=1, EEC=1).
4. Fix struct; re-run; iterate to residual 0 / byte-exact.
