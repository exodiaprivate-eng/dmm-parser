# faction_node_info — v3.1 Audit Memo

**Status**: Audit (no implementation yet).
**Author**: 1-min loop, iter 93 (2026-05-10).
**Audience**: Future implementer auditing dmm-parser's largest single
residual-gap table (14 missing canonicals out of 549 total).

## Why this table

Per `MOD_AUTHOR_GUIDE.md` § Residual coverage class 4: `faction_node_info`
is the lone "larger un-audited table". Closing it knocks 14 fields off
the 549 total in one pass — roughly 2.5% of all remaining gaps.

It also has historical baggage: 13 of its 33 rust fields are clearly
placeholders (`unknown_a/b/c/d`, `key_str_after`, `lookup_after`,
`flag_after_slots`, `raw_after_de690`, `final_list_u32`, `final_list_u16`,
`final_lookup`, `big_composite_slots`, `de690_data`). The struct name
hints at hand-debugging-by-offset: `de690_data` is the byte offset, not
a semantic name. So this audit is also implicitly a struct-cleanup
opportunity.

## Schema vs rust delta

Schema has 31 canonicals. Rust struct has 33 fields (placeholders +
de690-style). 17 currently aliased; 14 unaliased canonicals remain.

### The 14 missing canonicals

Grouped by schema `type`:

| Canonical | Schema type | Likely wire shape |
|---|---|---|
| `_factionType` | `direct_15B` | typeinfo dispatch byte (15B = "direct dispatch into setter table") |
| `_workerCount` | `direct_15B` | u32 worker count, dispatched same way |
| `_useCustomWayPointforDev` | `direct_15B` | u8 flag, dispatched |
| `_subInnerTypeString` | `None` | CString or u32 hash to string table |
| `_bitMapColorKey` | `None` | u8 or u32 (UI color key) |
| `_memo` *(already aliased)* | `None` | (already in rust) |
| `_observeData` | `array_or_complex` | nested struct, possibly CArray of typed records |
| `_factionEventDataList` | `reader_4B` | CArray of u32 → faction-event-info keys |
| `_factionScheduleInfoList` | `None` | CArray; partial overlap with rust `faction_schedule_list`? |
| `_knockDownCondition` | `reader_4B` | u32 → condition_info key |
| `_religionBlockCostList` | `reader_4B` | CArray<u32> → item costs |
| `_religionMaxBlockDay` | `direct_u32` | u32 day count |
| `_religionEffectRegionInfoList` | `reader_2B` | CArray<u16> → region_info keys |
| `_religionSubLevelInfo` | `reader_4B` | u32 → sub_level_info key |
| `_researchDataList` | `None` | CArray of typed research entries |

### Candidate rust mappings (for tuple-scoped overrides)

The 13 placeholder rust fields likely cover several of the missing
canonicals. Educated guesses (need IDA confirmation):

| Rust field | Possible canonical |
|---|---|
| `unknown_a` (u8) | `_useCustomWayPointforDev` (u8 flag) |
| `key_str_after` (CString) | `_subInnerTypeString` (u32 → string OR raw CString) |
| `unknown_b` (u8) | `_bitMapColorKey` (u8 color key) |
| `lookup_after` (u32) | `_factionType` (u32 type code) |
| `unknown_c` (u8) | `_workerCount`'s dispatch byte? OR `_factionType`'s? |
| `unknown_d` (u8) | …same uncertainty |
| `adjacency_list` (CArray) | `_observeData` OR one of the 4 list canonicals |
| `big_composite_slots` (struct) | wraps several of `_religion*` + `_factionEvent*` + `_research*` |
| `flag_after_slots` (u8) | one of the religion flags |
| `de690_data` (struct) | wraps `_knockDown*` + `_religion*` |
| `raw_after_de690` (u32) | `_religionMaxBlockDay`? |
| `final_list_u32` (CArray<u32>) | `_factionEventDataList`? `_religionBlockCostList`? |
| `final_list_u16` (CArray<u16>) | `_religionEffectRegionInfoList`? `_storeInfo` is already aliased |
| `final_lookup` (u32) | `_religionSubLevelInfo`? |

That's 13 placeholder rust fields and 14 missing canonicals — with one
canonical (`_subInnerTypeString` or one of the `_religion*` list
canonicals) likely living inside the `big_composite_slots` or `de690_data`
sub-struct rather than as a top-level rust field.

## Recommended audit approach

1. **Locate the per-record reader**. Not currently in the master
   typeinfo registry (iter 47 didn't reach `faction_node_info`'s setter
   chain). Step:
   - `mcp__ida-pro-mcp__list_strings_filter("FactionNodeInfo")`
   - find the `pa::FactionNodeInfo` typeinfo string
   - one xref → vtable
   - vtable's read-from-bytes slot → per-record reader address

2. **Decompile the reader** and walk the wire reads in source order.
   Use the iter-66 closure-plan template.

3. **Cross-reference each wire read** against:
   - Mac binary __cstring declaration order (for ground-truth field
     ordering)
   - schema canonicals' `type` field (matches reader sub-call shape)
   - existing rust placeholder field names (substitute one-by-one)

4. **Decompose `big_composite_slots` and `de690_data`** into named
   typed sub-structs matching canonical groups (`FactionReligionData`,
   `FactionResearchData`, etc.). This is the biggest cleanup deliverable.

5. **Add 14 v3.1 aliases** once the rust field names match.

6. **Re-run schema verifier** — expect 17/31 → 31/31 verified, total
   missing-in-dmm 549 → 535.

## Risks

- **`big_composite_slots` and `de690_data` are opaque**: if the wire
  layout has runtime-conditional sub-readers (presence-byte gating
  struct contents), a naive split might break round-trip. Inspect
  the existing `read_from` impls for these types for any conditional
  branches before splitting.
- **`adjacency_list` semantic**: the name suggests graph-edge data but
  the canonical schema lists `_factionEventDataList` as a `reader_4B`
  CArray. If `adjacency_list` is the rust holding for that, naming
  is misleading and a rename + alias is the right move (not a split).
- **Hidden alignment padding**: faction_node_info's reader is 0x5df
  bytes (1.5KB per iter-50 registry). Long readers tend to have
  alignment-padding tricks (e.g. 1-byte read followed by skip-3 to
  align next u32). Watch for these.

## Out of scope

- Rust struct rename of `unknown_X` → semantic names. Memo-only here;
  shipping the rename is its own follow-up PR.
- Resolving the `_observeData` `array_or_complex` semantic — that's the
  hardest field and may need its own typed family wrapper.

## References

- Schema verifier output: `docs/v3_1_schema_verification.json` →
  `per_table.faction_node_info`.
- Per-table rust struct: `src/tables/faction_node_info/info.rs`.
- 4-class taxonomy: `MOD_AUTHOR_GUIDE.md` § Residual v3.1 surface
  coverage.
- Companion design memos (handle classes 1 + 2):
  `V3_1_ALIAS_MECHANISM_EXTENSION_DESIGN.md`,
  `V3_1_GLOBAL_GAME_EVENT_INFO_DECOMPOSE_DESIGN.md`.
