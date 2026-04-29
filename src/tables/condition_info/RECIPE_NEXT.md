# ConditionInfo wiring roadmap

The GameCondition expression tree is fully mapped via IDA + Rust infrastructure.
ConditionInfo currently remains blob-tail to preserve the 121/121 round-trip
baseline. To finish wiring it as fully field-decoded, fix the per-variant byte
recipes for the 53 VARIABLE-size ConditionData tags.

## What's done

### Meta-dispatcher mapped (sub_141E65330 — 9 cases)

| Tag | Function | Family | Implemented? |
|---:|---|---|---|
| 0 | sub_141E65740 | BinaryOp_A (recursive) | YES — `binary::variants::game_condition` |
| 1 | sub_141E65D40 | BinaryOp_B (recursive) | YES |
| 2 | sub_141E662D0 | UnaryOp (recursive) | YES |
| 3 | sub_141C87CE0 | ConditionData (405 leaves) | partial — codegen done, ~55 wrong recipes |
| 4 | sub_141D89730 | BranchConditionData (14 leaves) | codegen done — verify against data |
| 5 | sub_141D8B1A0 | ScheduleCompleteConditionData | YES — `binary::variants::schedule_complete_condition_data` |
| 6 | sub_141CB6480 | ConditionGimmickData | YES — `binary::variants::condition_gimmick_data` |
| 7 | sub_141DAC600 | StageChart / StageChart_Event | YES — `binary::variants::condition_data_stage_chart` (incl. GameExpression + IVariantItem) |
| 8 | sub_14114FF40 | GlobalEffectConditionData | YES — `binary::variants::global_effect_condition_data` |

### Sub-families for case 7

- `binary::variants::game_expression` — 7 GameExpression variants (UnaryOperator, BinaryOperator, MemberFunction, Actor, Primitive, ConstObject, Value), recursive
- `binary::variants::ivariant_item` — IVariantItem with 14 case-tag-driven payloads

### Recursive enum

`binary::variants::game_condition::GameConditionNode<'a>` ties everything together with a recursive `read_from`/`write_to` matching the case table above.

### Optional subcond on ConditionData

`ConditionData` struct has fields `option_present: u8` and `option_data: Option<ConditionDataOptionData<'a>>`. Currently `read_from` does NOT consume these bytes (defaults to 0/None) because empirical testing showed the recipe's claim that slot 19 always reads them is wrong for at least some variants. Needs investigation.

## What's blocking full wiring

Per the diagnostic test `binary::variants::diagnose_conditiondata::diagnose_conditiondata_variants`:

- **8934 total ConditionInfo entries**
- **108 unique ConditionData tags observed** (out of 405 in the recipe)
  - **55 are CONSTANT-size** — single byte count across all instances; SAFE TO CORRECT
  - **53 are VARIABLE-size** — multiple byte counts (have CStrings/CArrays inside); need per-variant IDA decompile
  - **297 are unobserved** — exist in binary but not used in vanilla; don't matter for round-trip

Output saved at `dmm-pabgb-aio/mac_extract/conditiondata_empirical_observations.json`. Format per tag:
```json
"206": { "count": 1, "min": 5, "max": 5, "unique_sizes": [5], "kind": "CONSTANT" }
```

The recipe `conditiondata_recipes.json` claims tag 206 has `tail_bytes: 0` — wrong by 5. The obfuscated readers in the `0x14F0xxxxx` range XOR-pack their stream-size constants, and the recipe extractor recorded 0 when it couldn't resolve.

## Steps to finish wiring

1. **Fix the 55 CONSTANT tags first** (highest ROI, no IDA needed):
   - For each CONSTANT tag in the empirical observations file, set the variant payload size in `binary/variants/condition_data.rs` to the observed value.
   - Easiest path: replace existing `tail_fields` with a single `raw_bytes: [u8; N]` field (preserves bytes for round-trip).
   - Unlocks single-leaf entries that use only CONSTANT tags — likely 60-70% coverage.

2. **Decompile the 53 VARIABLE tags** (one IDA decompile per tag):
   - For each VARIABLE tag, decompile its slot-16 read function (per `conditiondata_recipes.json` field `read_fn`).
   - Trace stream reads: u8/u16/u32 + CString + sub-calls.
   - Update the variant's payload struct.

3. **Verify the optional_subcond conditions**: Re-enable in `ConditionData::read_from`, identify breaks, determine if slot 19 is unconditional or has a guard.

4. **Switch `ConditionInfo` to use `GameConditionNode`**: Replace `game_condition: Vec<u8>` with the typed enum. Run round-trip until 8934/8934 pass.

5. **Update v3 docs**: Move ConditionInfo from blob-tail to fully field-decoded in `mod-authors-guide.md` (65 → 66 typed, 56 → 55 blob-tail).

## Why this matters

ConditionInfo is the highest-mod-author-value Tier 2 table — defines every quest condition, dialog trigger, and buff applicability rule. Promoting blob-tail → fully field-decoded means v3 mods can edit individual conditions instead of cloning whole condition blobs.

## See also

- `dmm-pabgb-aio/mac_extract/game_condition_tree_recipe.json` — full meta-dispatcher map
- `dmm-pabgb-aio/mac_extract/conditiondata_empirical_observations.json` — per-tag byte counts from real data
- `dmm-pabgb-aio/mac_extract/conditiondata_recipes.json` — auto-extracted recipe (has bugs in 53 VARIABLE tags)
- Memory: `project_game_condition_tree.md`
