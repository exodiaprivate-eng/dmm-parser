# global_game_event_info — Decoder Decompose Design Memo

**Status**: Design (not yet implemented).
**Author**: 1-min loop, iter 92 (2026-05-10).
**Audience**: Future implementer who wants to close the 3 v3.1 gaps in
this table that the alias-mechanism extension (iter 91 memo) cannot
reach because they need real decoder work.

## Problem

`src/tables/global_game_event_info/info.rs` currently has 5 fields:

```rust
pub struct GlobalGameEventInfo<'a> {
    pub key:                       u16,
    pub string_key:                CString<'a>,
    pub is_blocked:                u8,
    pub global_game_event_group_info: u16,
    pub execute_data:              GlobalGameEventExecuteData<'a>,
}
```

`execute_data` is the polymorphic-family wrapper from
`src/binary/variants/global_game_event_execute_data.rs`. Wire-perfect
round-trip works.

NattKh's canonical schema lists **8 fields**, with these 3 marked
missing in dmm-parser:

| Canonical | Schema type | Position in wire |
|---|---|---|
| `_eventDesc` | `direct_8B` (8 raw bytes) | offset 24 |
| `_uiIconPath` | `reader_4B` | inside `execute_data`'s span |
| `_targetRegionInfoList` | `reader_2B` (CArray<u16>) | inside `execute_data`'s span |

Per the iter 66 closure plan in this table's `info.rs` doc-comment, the
Win-IDA per-record reader `sub_1410B2150` does:

```
offset 0    2 bytes      → _key (u16)
offset 8    CString      → _stringKey
offset 16   1 byte       → _isBlocked
offset 18   sub_1410D1B60 → _globalGameEventGroupInfo (u16 lookup)
offset 24   8 bytes      → _eventDesc                        (NOT decoded)
offset 32   sub_1410E35E0 → _uiIconPath OR _executeDataList  (partial)
offset 48   sub_141123F40 → _targetRegionInfoList OR _executeDataList
```

So the current `execute_data` field absorbs reads 5+6+7 as a single
typed wrapper, conflating three separate canonical fields.

## Why the alias-mechanism extension can't fix this

The iter-91 1-to-N alias mechanism only relabels **already-decoded**
rust fields. Here the bytes are decoded, but they're stored under a
single `execute_data` name with no per-canonical accessor. To expose
the 3 canonicals to mods, the rust struct must split.

## Proposed split

```rust
pub struct GlobalGameEventInfo<'a> {
    pub key:                       u16,
    pub string_key:                CString<'a>,
    pub is_blocked:                u8,
    pub global_game_event_group_info: u16,

    /// 8 raw bytes at offset 24. Empirical sweep: head=0, tail=u32-ish hash.
    /// Possibly (head_id, payload_id) or similar 2× u32 pair.
    pub event_desc:                [u8; 8],

    /// 4-byte hash via sub_1410E35E0 (reader_4B family).
    /// Stored as u32 raw on disk; runtime resolves to UI icon path string.
    pub ui_icon_path:              u32,

    /// CArray<u16> via sub_141123F40 (reader_2B family).
    /// List of region_info keys this global event targets.
    pub target_region_info_list:   CArray<u16>,

    /// Polymorphic execute-data tail. Now ONLY contains the post-three-
    /// canonical-fields portion (presence byte + sub_tag + typed body).
    pub execute_data:              GlobalGameEventExecuteData<'a>,
}
```

### Wire-format implications

`GlobalGameEventExecuteData::read_from_bytes` currently consumes 3
sub-reader calls. After the split it must consume only the trailing
polymorphic dispatch (presence byte + sub_tag + typed body per the
existing `sub_141156680` doc-comment). The 3 split-out reads happen
in `GlobalGameEventInfo::read_with_size` BEFORE the
`execute_data.read` call.

### Backward compatibility

This is a breaking change to:

1. **JSON output for shape='v3'**: existing keys `key`, `string_key`,
   `is_blocked`, `global_game_event_group_info`, `execute_data` all
   stay. Three NEW keys appear: `event_desc`, `ui_icon_path`,
   `target_region_info_list`. Mods that wrote those old keys still
   parse fine; mods that *omitted* the new keys on input will fail
   the new "all required fields present" check unless we make the
   3 new fields default-able.
2. **Wire round-trip**: identical bytes go in and out — what changes
   is which struct field claims which slice. Tests at the byte level
   continue to pass without modification.
3. **`execute_data` JSON sub-shape**: if the 3 split-out reads were
   previously serialized inside `execute_data`'s typed sub-fields,
   those sub-fields disappear from the `execute_data` JSON dict.
   Mods that touched them by name-inside-execute_data break.

## Implementation steps

1. **Confirm wire layout against IDA**: re-decompile `sub_1410B2150`
   with the new IDA plugins (HRDevHelper to inspect ctree, function-
   string-associate to confirm sub-reader semantics). The current
   doc-comment is iter-66-era; double-check that
   - `sub_1410E35E0` truly reads exactly 4 wire bytes
   - `sub_141123F40` truly reads `u32 count + N×u16`
   - no hidden alignment between offset 32 and 48

2. **Write the 3 typed readers**:
   - `event_desc: [u8; 8]` — trivial, primitives.rs already has `[u8; N]`
   - `ui_icon_path: u32` — primitive read; if sub_1410E35E0 wraps a
     hash-lookup post-process, document but store the wire u32
   - `target_region_info_list: CArray<u16>` — already exists in
     binary/types.rs

3. **Update `GlobalGameEventExecuteData::read_from_bytes`** to skip
   the 3 reads it previously absorbed. Adjust mem-stride if needed.

4. **Update fixtures**: re-record any pabgb roundtrip test snapshots
   so JSON dicts include the 3 new keys.

5. **Add v3.1 aliases**:
   ```rust
   ("event_desc",              "_eventDesc"),
   ("ui_icon_path",            "_uiIconPath"),
   ("target_region_info_list", "_targetRegionInfoList"),
   ```

6. **Re-run schema verifier** — expect:
   - `global_game_event_info`: 5/8 → 8/8 verified
   - Total missing-in-dmm: 549 → 546

## Risks

- **Wrong wire layout**: if iter-66's doc-comment is partially wrong
  about which sub-reader consumes which canonical, the split mis-
  attributes bytes. Mitigation: roundtrip test must pass at byte level
  before commit.
- **execute_data sub-tag dispatch coupling**: if any of the 3 split-
  out reads was acting as the polymorphic-presence byte, the dispatch
  logic in `GlobalGameEventExecuteData` breaks. Mitigation: read
  `sub_141156680` (the polymorphic dispatcher) carefully — its first
  byte must come AFTER the 3 split-out reads, not at offset 32.
- **JSON breaking change for existing mods**: list `execute_data`'s
  prior sub-field-by-sub-field accessors in the migration note.

## Out of scope

- The `_eventDesc` 8 bytes' semantic decomposition (head/tail meaning).
  Stored as raw bytes; let mod authors interpret.
- Hash-lookup-table inversion for `_uiIconPath` (i.e. resolving the u32
  to an actual filesystem path string). The wire u32 is what matters
  for round-trip.

## References

- iter-66 closure plan: `src/tables/global_game_event_info/info.rs`
  top doc-comment, "v3.1 closure plan (iter 66)" block.
- Polymorphic dispatcher analysis:
  `src/binary/variants/global_game_event_execute_data.rs` header.
- 4-class gap taxonomy: `MOD_AUTHOR_GUIDE.md` § Residual coverage,
  class 2 ("Real decoder work needed").
- Companion alias-mechanism design: `V3_1_ALIAS_MECHANISM_EXTENSION_DESIGN.md`.
