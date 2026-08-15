# Patch-day toolkit

Built during the 1.16.00 update (2026-08-01). Reusable every game patch.

Set both fixture dirs at the top of the scripts (or edit the constants):

- `OLD` = last known-good dump, e.g. `C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-7-16`
- `NEW` = the new dump, e.g. `C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-8-1`

## Order of operations

**0. `bytediff.py` — run this FIRST, before anything else.**
Two-fixture byte diff over EVERY table in one command. Gives you the work queue
ranked by how much each table changed, and names the drift shape per table.

```
python bytediff.py                 # all tables
python bytediff.py buffinfo        # one table
python bytediff.py --json out.json
```

| verdict | meaning | next step |
|---|---|---|
| `FIELD ADDED/REMOVED: … a N-byte field` | constant size delta — N *is* the field width | find it with `korean_fields.py` |
| `ENUM RENUMBER: N diffs of ±d, lowest X -> Y` | a variant index shifted; X is where it happened | fix the enum, **skip IDA** |
| `mixed size deltas [...]` | a variable-length field (CArray/CString) drifted | `ruststruct.py` shift profile |
| `STRUCTURAL: many distinct size deltas` | several things moved | `triage.py` then `ruststruct.py` |
| `[IDX]` flag | keys did not line up (remapped/reordered) — treat as approximate | check whether keys were rehashed |

This automates the buff_info procedure in "Hard-won gotchas" below; it reproduces
that exact result (`1523 diffs of -1, lowest 37 -> 36`) automatically.
⚠ Both signals are reported independently — buff_info has a size change AND an
enum renumber, and only the enum half is cheap to act on.

**1. `triage.py` — run this after bytediff.**
Walks every table's records with its own Rust struct, on BOTH fixtures, and
reports where each stops.

```
for t in <tables>; do echo "$t|<fixture>|<RootStruct>"; done | python triage.py
```

Read it like this:

| pattern | meaning |
|---|---|
| `1.15 = 200/200`, `1.16 = 0/200` | **clean 1.16 drift** — the stop-field names it. Patch-diffable. |
| both `0/200` | usually the walker can't model that table (polymorphic variant / hand-written reader), **not** proof the Rust parser is broken. Verify with `cargo test`. |
| same ratio on both | pre-existing, untouched by this patch. |

**2. `rec.py` — record slicing.** `.pabgh` has TWO header formats (u16 count at
offset 2, u32 count at offset 4) and the entry list is **not** offset-ordered.
This auto-detects and sorts. Getting this wrong silently yields wrong record
boundaries (it reported key `0x10000` where the real key at offset 0 was `0x1`).

**3. `ruststruct.py` — the per-field offset tracer.** Parses a dmm-parser Rust
struct and walks a record with it, recording every field's byte span. Diffing
old vs new spans NAMES the drifting field, instead of leaving you with a CArray
count blow-up hundreds of bytes downstream. Validate it first: it must walk
100% of the OLD fixture before you trust anything it says about the NEW one.

**4. `korean_fields.py` — the field-name oracle.** Dumps every
`"<Table>의 _<field>를 읽어들이는데 실패했다"` string from the Mac binary →
each table's exact field list *and order* for the new build. 4,421 fields /
516 tables on 1.16. This is the single highest-value tool here; most 1.16
fixes fell out of diffing this list against the Rust struct.

**5. `schema116.py` — field list + reader + memory offsets.** Xrefs a table's
first field string to its reader, decompiles it, and pairs the field names with
the reader-call sequence (`sub_XXX(a1, a2 + N)`). Needs IDA open on the Mac
binary. `classify.py` bulk-classifies unknown readers (`FIX n` / `FWD` / `LOOP`).

**6. `ida.py` — IDA-MCP helper.** Plain JSON-RPC to `localhost:13337/mcp`.
Use this rather than raw curl: the decompiler output contains Korean text and a
bare `curl | python` pipe dies on cp1252.

## Hard-won gotchas

- **Data beats RE for enum renumbers.** For `buff_info`, diffing same-size
  records and collecting every single-byte change gave 1523 diffs, *all* −1,
  lowest `37 → 36` — that pinned both the removed variant index and the shift
  direction with no disassembly. Do this before opening IDA.
- **difflib is not a field-boundary oracle.** It slides inserts inside zero runs,
  so its offsets are not field boundaries. Align at a *known anchor* instead
  (e.g. walk backwards from the record end through fixed-size tail fields).
- **Tail-merged strings lie.** Naive `[\x20-\x7e]+BuffData\x00` mining returns
  ~104 "class names" that are mostly shared suffixes (`amageBuffData` from
  `DamageBuffData`). Only 2 standalone NUL-delimited `*BuffData` strings exist
  in the whole binary.
- **A field that looks new may be an existing one's sentinel.** The
  `ff ff ff ff ff ff ff ff` chased in `iteminfo` was `respawn_time_seconds = -1`.
- **New fields in modded tables must be null-safe on write.** Mods written
  before the patch have no key for them; see `store_info`'s
  `low_price_threshold_count_116` for the idiom.
