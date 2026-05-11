# Havok Layer + 1.06 Table Repair — Autonomous Workplan

> Driver doc for the autonomous loop. Each iteration picks the highest-impact
> open item, ships it, updates this file, and moves on.

**Started:** 2026-05-11
**Triggered by:** 1.06 patch dropped → 5 tables flagged "pabgh parse failed"
in diff, plus longer-term goal of parsing the entire Havok-layer file family
(.hkx, .pac, .pacc, .pam, .pami, .pamlod, .motionblending, .pab, .pabc, .pabv,
.paasmt, .paccd).

## Done so far

### Iter 28 (2026-05-11) — **LOOP WIND-DOWN: final summary**

Per the explicit stop condition in the loop's driver prompt
("Workplan iter 3+ all shipped → final summary, do not re-schedule"),
this iter winds down the autonomous loop. Cron job `af43c492`
deleted; task #29 marked completed.

## Final scorecard (27 productive iters, 2026-05-11)

### 1.06 game patch repair — DONE
- All 5 PABGH PARSE FAILED tables fixed (iter 1-2)
- 3 new pabgh format variants added, opaque fields decoded via
  corpus audit (`parent_id`, `hash_key`, `aux_c`)

### Mod-tool-author deliverable — DONE
- 24 new Tier 1 format parsers shipped, all with byte-perfect
  round-trip + Python bindings
- 4 generic classifiers built (`par_resource`, `xml_resource`,
  `binarystring`, `count_record_table`) that cover most of the
  long-tail formats with one module each
- 1 critical infrastructure fix (partial-compression unblock,
  iter 9 — unblocked extraction for 29/30 .pamlod + 24/30 .pac +
  13/30 .pam files that had been stuck behind the misnamed flag)

### Diff tool — extended for Tier 2
- 3 new pabgh formats recognized
- iteminfo typed diff (string_keys instead of hex)
- sequential pabgb diff support
- Tier 2 file diff covering all 24 new format types with file-level
  + typed field-level deltas (`.pami.mesh_paths`, `.pamlod.lod_count`,
  `.paccd.no_override_byte_count`, PAR `version` FORMAT-CHANGE
  warnings, `.hkx.sdk_version` SDK upgrades, etc.)

### Documentation — complete
- `docs/api.md` "Havok-Layer Formats (Tier 1)" section
- `docs/MOD_AUTHOR_GUIDE.md` §13 single-page summary table
- `docs/ENGINE_INTERNALS.md` Havok section refreshed
- `docs/SHIPPED.md` repair-loop catalog
- `docs/BINARY_FORMATS.md` continuously updated as parsers shipped
- `samples/06_havok_layer_edit/` worked example (mod.py end-to-end
  verified on real game file)

### Health metrics
- Tests: **567 → 635** (+68 new), all passing
- 0 build failures across 27 iters
- 0 test regressions (test failures caught + fixed mid-iter twice,
  never shipped broken)
- 2 wrong-field-name bugs caught by post-extraction corpus audits
  (motionblending iter 11, pamlod iter 14) — single-sample-ship
  anti-pattern documented in the workplan as iter-3 lesson
- Python wheel rebuilt + reinstalled cleanly across iters

## Remaining blockers (deferred — need IDA RE to unblock)
- `.hkx` Havok object-graph decode (needs `hkClass` registry walk)
- `.motionblending` typed value decode (per-tag wire layouts)
- `.paccd` per-slider semantic mapping
- Partial-compression-with-size-differential variant
  (blocks ~17 .pam + ~6 .pac files)
- `.questgaugecount` / `.pathc` / `.paproj` / `.pai` per-record decode

When IDA is reconnected these can be picked up in a new cron loop.
The workplan + per-iter log in this doc are the entry point for
that future work.

### Iter 27 (2026-05-11) — **Worked sample for mod authors: `06_havok_layer_edit`**
With API docs (iter 16), engine internals (iter 26), and SHIPPED.md
catalog (iter 26) all refreshed, the missing piece for mod authors
was a **concrete worked example** of using the new parsers — read,
mutate, write a Havok-layer file end-to-end.

Shipped `samples/06_havok_layer_edit/`:
- `mod.py` — 70-line Python script that loads a `.pami` via
  `parse_pami_bytes`, swaps a `mesh_path` via in-place
  `xml_body.replace(...)`, writes back via `serialize_pami`, and
  re-parses to verify the edit landed
- `README.md` — explains the read/mutate/write flow, lists all 24
  parsers with their mod-relevant fields, documents the round-trip
  discipline (named convenience fields are read-only views; edit
  the body bytes directly)
- Updated `samples/README.md` index to list sample 06

**End-to-end verified against a real game file**:
```
$ python mod.py 03_cube.pami patched.pami "object/03_cube.pam" "object/03_sphere.pam"
in:  03_cube.pami  version=1  mesh_paths=1
  object/03_cube.pam
out: patched.pami  version=1  mesh_paths=1
  object/03_sphere.pam <- new
```

The cube → sphere mesh swap completes cleanly with a re-parse
verification. This pattern (parse → mutate body → serialize → reparse-verify)
applies identically to all 24 new parsers.

No dmm-parser code changes — sample-only iter. **635 lib tests pass**
(unchanged). IDA still disconnected.

This iter completes the **mod-tool-author deliverable** the user
originally requested: parsers shipped + Python-callable + documented
in api.md, MOD_AUTHOR_GUIDE.md, ENGINE_INTERNALS.md, SHIPPED.md +
surfaced in the diff tool + a runnable worked example mod-authors
can copy.

Next iter: more incremental cleanup, deeper IDA-required RE when
IDA is back, OR loop wind-down if user prefers.

### Iter 26 (2026-05-11) — **Documentation refresh: ENGINE_INTERNALS + SHIPPED**
With 25 iters of code + diff-tool work shipped, the long-form
documentation hadn't been refreshed to match. Two key docs updated:

- `docs/ENGINE_INTERNALS.md`: the Havok section's "Status" header
  read "Research notes only. The Havok layer itself ... remains
  Layer 5 — not yet field-level parsed in dmm-parser." That's now
  **WRONG**. Refreshed to reflect the 12 Tier 1 parsers shipped
  this loop, with the still-TBD items (object-graph decode,
  per-tag value layout, etc.) listed below.
- `docs/SHIPPED.md`: added a new "Havok+1.06 Repair Loop" section
  catalogging everything across iters 1-25 — broken-table fixes,
  Tier 1 parsers (24 formats), generic classifiers (3), infra
  fixes (partial-compression unblock), diff tool extensions,
  Python bindings (24 functions), test growth (567 → 635), and
  remaining blockers.

The SHIPPED.md entry is the single-source-of-truth catalog for
mod authors / future Claude sessions / anyone trying to understand
"what was added by this loop". The detailed per-iter log lives
in `_HAVOK_TABLES_WORKPLAN.md`.

Also fixed the existing SHIPPED.md "Havok layer 0% parsing" bullet
to point at the new section.

No code changes — docs only. **635 lib tests pass** (unchanged).
IDA still disconnected — partial-compression RE remains blocked.

Next iter: more docs/cleanup, or wait for IDA to enable deeper RE.

### Iter 25 (2026-05-11) — **Diff tool typed field deltas (Tier 2)**
Built on iter 24's file-level Tier 2 diff with per-format **typed
field-level deltas** for the highest-mod-value formats. When a changed
Tier 2 file is detected, the diff tool now invokes the typed parser
on both sides and surfaces semantic deltas instead of just size deltas.

Wired typed delta extraction for:

| Ext / Family | Deltas surfaced |
|---|---|
| `.pami` | `version` change + added/removed `mesh_paths` |
| `.material`/`.technique`/`.mi`/`.spline`/`.spline2d`/`.pma` (XML) | `root_element` change (rare; bytes-differ-with-same-root collapses to size-only) |
| `.pab`/`.paa`/`.pam`/`.pabc`/`.pabv`/`.pac`/`.pat`/`.papr` (PAR) | `version` change with **"FORMAT CHANGE — parser may need work"** warning + `body_len` delta |
| `.pamlod` | `lod_count` + `lod_distance` + added/removed `texture_paths` |
| `.paccd` | `format_version` + `no_override_byte_count` (slider customization changed) |
| `.motionblending` | `version` + `type_name` + added/removed `field_records` names |
| `.hkx` | `sdk_version` change → flags Havok SDK upgrade |
| `.paasmt` | `record_count` delta |
| `.binarystring` | `count` delta |

Shipped:
- `_typed_tier2_deltas(cr, ext, base_bytes, new_bytes)` dispatcher
  in `diff_pabgb_versions.py`
- Best-effort design — parser failures collapse to size-only delta,
  so adding new format types to TIER2_FORMATS doesn't require a
  matching delta handler to land at the same time
- Synthetic verification: `.pami` mesh-path delta + `.pab` version
  delta both print correctly

Examples of new mod-author-actionable output:
```
.pami: ! object/foo.pami size: 1992 -> 1995 bytes (+3)
                          mesh_paths +1: ['object/new.pa']
.pab:  ! character.pab    version: 0x01050001 -> 0x01050002 (FORMAT CHANGE)
.pamlod: ! tree.pamlod    lod_count: 4 -> 5
.paccd: ! npc_001.paccd   no_override_byte_count: 312 -> 280 (slider changed)
```

No dmm-parser code changes — diff tool only. **635 lib tests pass**
(unchanged).

Next iter: rebuild the wheel + re-extract a baseline to populate
Tier 2 dump dirs, then actually run the enhanced diff against a
patch to validate real-world output. Alternatively shift focus to
parser deepening once IDA is back.

### Iter 24 (2026-05-11) — **Diff tool Tier 2 coverage**
Pivoted from per-format RE (poor ROI without IDA) to fixing the
visibility gap: the diff tool was only diffing PABGB + Tier 1
sequencer/schedule/attack-info formats. The 24 new format types
shipped iters 3-22 (Havok-layer + XML family + small binaries)
were silently ignored during patch diff — mod authors couldn't see
which `.pami`/`.material`/`.motionblending` files changed between
game versions.

Shipped in `dmm-pabgb-aio/diff_pabgb_versions.py`:
- `TIER2_FORMATS` tuple covering 24 extensions:
  `.pami`, `.pab`, `.paa`, `.pam`, `.pabc`, `.pabv`, `.pac`, `.pat`,
  `.papr`, `.motionblending`, `.pamlod`, `.paasmt`, `.paccd`, `.hkx`,
  `.material`, `.technique`, `.mi`, `.spline`, `.spline2d`, `.pma`,
  `.binarystring`, `.imp`, `.impostor`, `.paprojdesc`
- `EXTRACT_EXTENSIONS` now includes Tier 2 so re-extraction picks
  them up automatically
- `collect_tier2_files` + `diff_tier2_file` + `diff_tier2` —
  parallel to Tier 1 infrastructure
- New report section "Tier 2 file diff" — per-extension grouping
  with `+N -M !K` counts and the first 3 changed files per ext
  shown with size deltas
- "Tier 2 summary" footer with identical/added/removed/changed
  totals
- JSON output extended with `tier2_summary` + `tier2_files`
- Top-of-file docstring updated: THREE families → FOUR families

The Tier 2 diff is file-level (size delta only) — field-level
deltas via typed parsers (e.g. `.pami.mesh_paths` additions,
`.pamlod.lod_count` changes, `.paccd.no_override_byte_count`)
remain opt-in for future iters where the per-format work has
clear mod-author value.

Self-diff smoke test: passes cleanly. Existing 1.05 baselines
don't have Tier 2 files extracted (they predate this iter's
TIER2_FORMATS) — next extraction populates them automatically.

No new dmm-parser tests this iter (diff tool only). **635 lib
tests still pass** (unchanged).

This closes the mod-author visibility gap for the 24 new format
types. When the next game patch lands, the diff report will
surface changes across `.material`/`.technique`/`.motionblending`/
etc. by file count + per-ext breakdown.

Next iter pivot: enrich a few high-mod-value Tier 2 formats with
field-level deltas (e.g. `.pami.mesh_paths` to show which static
mesh references changed between patches).

### Iter 23 (2026-05-11) — **`.questgaugecount` + `.pathc` structure identification**
Both files investigated. Single-sample-per-format situation in both
cases — limits what we can ship without overfitting.

- **`.questgaugecount`** (101 KB, 1 file): u32 count(=382) + variable-
  size records separated by 0xFFFFFFFF markers. 305 separators across
  25350 u32 values. Each record: `u32 hash + u32 sub_value + u32 ref
  + u32 zero + 0xFFFFFFFF + N×u32 extra`. Per-record fields don't
  follow a regular pattern across the file — likely a type-dispatched
  record format. Without IDA RE the per-record body decode would be
  guesswork.
- **`.pathc`** (2.3 MB, texture header collection): `u64 zero + u32
  size_a(148) + u32 size_b(672) + u32 total_records(279221) + records`.
  Body is dense binary texture metadata; only 14 `.dds` path strings
  embedded despite the huge file size. Without IDA RE the per-record
  texture-header structure can't be cleanly decoded.

Both moved from ⚠️ to ⚠️ (structure identified) in BINARY_FORMATS.md
with their identified field layouts documented. Use
`parse_count_record_table_bytes` to surface counts; treat bodies as
opaque for round-trip-only mod tooling.

No code changes — docs only. **635 lib tests pass** (unchanged).

**Pattern observation**: post-iter-21 the remaining unparsed formats
are all in one of two buckets:
- (a) single-file-in-install novelties where deep RE has poor
  ROI without IDA (.questgaugecount, .pathc, .pai)
- (b) huge binaries where the engineering cost-benefit ratio is poor
  (.nav 6-33MB navmeshes, .road* geometry, .pbd/.pcg/.dat/.ani/.pix)

Next iter pivot: the diff tool currently doesn't surface changes in
the 22 newly-shipped format types — when a patch lands the user
only sees pabgb table + Tier 1 file-format deltas. Adding diff-tool
coverage for the new format families would be a high-mod-author-
value enhancement.

### Iter 22 (2026-05-11) — **Generic `count_record_table` classifier**
Shipped a generic classifier for "u32 count + records" formats. The
records can have any per-format binary structure — the classifier just
walks the body scanning for embedded u32-length-prefixed UTF-8 strings
(record names) and returns count + names.

Verified against real game files:
- **`.paseqh`** (sequencerstageheader.paseqh, 893 KB):
  count=2949 records, scanned 6042 names (~2 per record).
  Got proper names like `cd_seq_quest_boar_rider_1550_schedule`.
- **`.paschedulectx`** (schedulecontext.paschedulectx, 968 KB):
  count=4088, scanned 4118 paths referencing `sequencer/.../*.paschedule`.
  Near-1:1 match.
- **`.paproj`** (commonprojectile.paproj, 107 KB):
  count=62 but only 2 names scanned — projectile records are
  binary-only, no embedded name strings. Generic classifier doesn't
  help this format; needs custom per-format parser.

Shipped:
- `src/binary/count_record_table.rs` — `parse` + `serialize` + 4 unit
  tests + Python binding `parse_count_record_table_bytes`
- Promoted `.paseqh` and `.paschedulectx` from ⚠️ → 🟡 Tier 1.5 in
  BINARY_FORMATS.md (count + name extraction available; per-record
  typed decode still TBD)
- Implausible-count rejection (cap 1M) so format mismatches fail loud

Tests: **635 lib tests pass** (was 631, +4).

This is another high-leverage iter — one module + one Python binding
covers 2 production formats with stable count+name decode, plus
provides a generic tool any future "count+records" PA format can
reuse. The pattern of leveraging generic classifiers (now 4 of them:
`par_resource`, `xml_resource`, `binarystring`, `count_record_table`)
has been the dominant code-shipping pattern from iter 17 onwards.

Remaining unparsed: `.paproj` (custom binary records — no name
strings), `.questgaugecount` (101 KB), `.pathc` (2.3 MB texture
header collection), `.pai` (3 MB AI chart), plus huge binaries
`.nav`/`.road*`/`.pbd`/`.pcg`/`.dat`/`.ani`/`.pix`. Next iter:
audit `.questgaugecount` (smallest of the unparsed remainder).

### Iter 21 (2026-05-11) — **`.pashv` Tier 1 + structure ID for 3 binaries**
Investigated 4 remaining ⚠️ binary formats. Findings:

- **`.pashv`** → reuses `binarystring` parser. 193/193 strings parsed
  cleanly on `allweaponcommonaisharevalue.pashv` (strings like
  `c_sequencer_movetype`, `c_sequencer_aiactiontype`). Same pattern
  as `.paprojdesc`: string-prefix section + binary value tail (~85%
  of file size is tail). String section is Tier 1; tail decode TBD.
- **`.paseqh`**: `u32 record_count + per-record (u32 name_len + name
  + binary fields)`. Each record has extra binary fields beyond the
  name. record_count=2949 in the main sequencer header.
- **`.paschedulectx`**: `u32 count + per-record (u32 hash + u8 flag
  + u32 len + utf8 path)`. ~4088 records referencing `sequencer/...`
  paths.
- **`.paproj`**: `u32 record_count + u32 type_hash + u32 something
  + records`. Record types vary by .paproj file (attachtoactor, pc,
  repeat, wave, etc.).

Updated BINARY_FORMATS.md for all 4 — `.pashv` → 🟡 Tier 1 (string
section); others → ⚠️ (structure identified). The structure notes
are mod-author-useful even without full parsers — let mod tools
locate the relevant bytes.

Tests: **631 lib tests pass** (no code changes — pure docs +
investigation iter).

Pattern observation: 3 of the 4 audited formats follow the same
"u32-count + per-record-with-name-and-binary-fields" shape but
each has its own per-record byte layout. A generic
`count_record_table` classifier (count + name extraction without
typed-value decode) could ship multiple Tier 1.5 entries in one
module — queued as a future iter.

Remaining unparsed: `.questgaugecount`, `.pathc`, `.pai` (3 MB AI
chart), `.nav` (huge navmesh), `.road*`, `.pbd`/`.pcg`/`.dat`/
`.ani`/`.pix`. Next iter: build the generic count+record-name
classifier OR audit `.questgaugecount` (101 KB, manageable size).

### Iter 20 (2026-05-11) — **Long-tail batch audit: 3 more Tier 1**
Audited 12 long-tail extensions in one pass. **Three reuse existing
parsers** with no new modules needed:

| Ext | Reuses | Notes |
|---|---|---|
| `.papr` | `par_resource` | NEW PAR-family ver `0x01000135` (particle/projectile resource). Added to ParExt enum. |
| `.pma` | `xml_resource` | UTF-8 XML, root `<ARFaceAnimation>` (face/animation reference). No code change needed — `parse_xml_bytes` handles it. |
| `.paprojdesc` | `binarystring` | u16 count + LP strings work via `parse_binarystring_bytes`. File has 32-byte trailing record after the string list (per-projectile data, TBD). |

Promoted from ⚠️ → 🟢. Plus negative findings:
- `.pmb` NOT FOUND in install (deprecated)
- `.pi` NOT FOUND in install
- `.pai` is 3 MB AI chart data (deferred — too big for sample-decode)
- `.pashv`, `.paproj`, `.paseqh`, `.questgaugecount`, `.paschedulectx`,
  `.pathc` are all binary with no obvious magic/structure — need
  IDA RE or pattern-mining iters

Tests: **631 lib tests pass** (was 630, +1 for the new PAR variant
classification).

The reusability finding is the iter's headline — the 3 generic
classifiers (`par_resource`, `xml_resource`, `binarystring`) cover
a surprising number of formats. The remaining ⚠️ formats are
genuinely novel binary structures that need per-format RE.

Next iter: deeper investigation on one of the remaining binary
formats (.pashv looks promising — clear length-prefix structure
starting at offset 2). Or wait for IDA to investigate the others.

### Iter 19 (2026-05-11) — **`.imp` + `.impostor` shipped**
Two more Tier 1 formats — vegetation/prop billboard descriptors:

- **`.imp`**: fixed 72-byte binary. Magic `"IMP "` + u32 reserved=256
  (constant across 30 samples) + 64-byte body. Body field semantics
  queued (likely impostor billboard parameters).
- **`.impostor`**: fixed 48-byte binary = 12 little-endian f32.
  floats[4..6] are always 0 (rotation padding); other floats are
  spatial extents that vary per mesh. Round-trip preserves bytes
  exactly. Mod authors can edit floats directly in the structured
  view.

Shipped `src/binary/impostor.rs` containing both `parse_imp_to_json`/
`serialize_imp_from_json` and `parse_impostor_to_json`/
`serialize_impostor_from_json`. 5 unit tests. Python bindings added:
`parse_imp_bytes`, `parse_impostor_bytes`, etc.

Tests: **630 lib tests pass** (was 625, +5).

`.imp`/`.impostor` were a logical pair to ship together since their
extensions are confusingly similar and the engine uses them
side-by-side for the same vegetation/prop LOD step. Removed both
from the ⚠️ table in BINARY_FORMATS.md.

Remaining unparsed: long-tail PA formats `.paprojdesc`, `.paproj`,
`.pashv`, `.papr`, `.pai`, `.pi`, `.pma`, `.pmb`, `.paseqh`,
`.questgaugecount`, `.paschedulectx`, `.pathc`, plus huge binaries
`.nav`, `.road*`, `.pbd`, `.pcg`, `.dat`, `.ani`, `.pix`. Next iter:
sample the small ones in a batch and ship classifiers.

### Iter 18 (2026-05-11) — **XML-family classifier: 5 formats in one ship**
Shipped a unified UTF-8 XML resource classifier covering 5 extensions
in one module (`src/binary/xml_resource.rs`). Each format is a plain
UTF-8 XML file with a stable root element:

| Ext | Root element | BOM | Audited corpus |
|---|---|---|---|
| `.material` | `<Technique>` | yes | 10/10 |
| `.technique` | `<Category>` | yes | 10/10 |
| `.mi` | `<SkinnedDecalProperty>` | no | 10/10 |
| `.spline` | `<SplineDataGroup>` | no | 10/10 |
| `.spline2d` | `<SplinePresetData>` | no | 10/10 |

Total: **50/50 real-game samples parse cleanly** with byte-perfect
round-trip via `xml_body`. BOM presence is captured per-file in
`has_bom`. Root-element name extracted via a from-scratch tag
scanner that handles XML prolog (`<?xml ...?>`), comments, and
self-closing tags.

Shipped:
- `parse_xml_to_json` / `serialize_xml_from_json` in `xml_resource.rs`
- 7 unit tests (BOM/no-BOM, prolog+comment, byte-perfect, error cases)
- Python binding `parse_xml_bytes` / `serialize_xml`
- BINARY_FORMATS.md: 5 extensions ⚠️ → 🟢 Tier 1

Tests: **625 lib tests pass** (was 618, +7).

This is a high-leverage iter — one module + one Python binding gets
all 5 formats production-ready. With `.pami`'s pre-existing dedicated
module, **6 of the formerly-⚠️ XML formats are now Tier 1**.

Remaining unparsed: `.imp` (72b binary, `IMP ` magic), `.impostor`
(48b floats-only), `.nav` (huge navmesh — defer), plus the long-tail
`.paprojdesc`/`.paproj`/`.pashv`/`.papr` and friends.

Next iter: `.imp` should be a quick win (small fixed-size binary
with magic — likely impostor billboard descriptor).

### Iter 17 (2026-05-11) — **Pivot to non-Havok extensions: `.pat` + `.binarystring`**
Pivoted past the Havok-layer milestone (closed iter 16) to attack the
broader `⚠️ unparsed` table in BINARY_FORMATS.md. Audit pass revealed:

- **`.paa` had a stale duplicate row** in the texture section labeling
  it "unknown PA texture variant" — REMOVED. The format is uniformly
  PAR-family animation set entries (verified iter 4).
- **`.pat`**: NEW PAR-family member. Version `0x01000404`. Texture
  data, verified 3/3 samples (tree/bush textures). Added to
  `par_resource.rs` ParExt enum.
- **`.binarystring`**: simple packed UTF-8 string list,
  `u16 count + (u8 len + utf8) × N`. Shipped `src/binary/binarystring.rs`
  with byte-perfect round-trip + reconstruction-from-strings path.
- **XML family discovery**: `.material`, `.technique`, `.mi`, `.spline`,
  `.spline2d` are all UTF-8 XML (different root elements per format).
  Queued for next iter — same pattern as `.pami`.
- **`.imp`**: 72-byte fixed binary with literal `"IMP "` magic. Queued.
- **`.impostor`**: 48-byte fixed binary, no magic (just floats).
- **`.nav`**: huge navigation meshes (6-33 MB). Defer indefinitely.

Tests: **618 lib tests pass** (was 612, +6: 5 binarystring + 1 pat).
Python bindings added for `parse_binarystring_bytes` /
`serialize_binarystring`.

Next iter: ship the XML-family classifier covering `.material`,
`.technique`, `.mi`, `.spline`, `.spline2d` in one shot (same pattern
as `.pami`). Then `.imp` (small, magic-validated).

### Iter 16 (2026-05-11) — **Mod-author documentation refresh**
With Python bindings shipped iter 15, the parsers are now usable from
`import dmm_parser` but **undocumented**. Mod authors had no way to
discover the new functions or their JSON shapes.

Shipped:
- New `docs/api.md` section "Havok-Layer Formats (Tier 1)" — 7
  subsections (one per module) showing JSON example output, version
  constants, and round-trip discipline (~150 lines added)
- New `docs/MOD_AUTHOR_GUIDE.md` §13 — single-page summary table of
  all 12 extensions + their mod-relevant fields, plus a "what's not
  done yet" callout listing the 4 remaining IDA-RE blockers

No code changes this iter — pure documentation. Tests unchanged at
**612 lib tests pass**.

This closes the documentation gap for the iter-3-to-15 work. Mod
authors picking up dmm_parser at HEAD can now:
1. Discover the format via the MOD_AUTHOR_GUIDE §13 table
2. Look up the JSON shape in api.md
3. Call the Python binding directly

Next iter pivot: with breadth + docs done, the natural next track is
either deeper field decode (still IDA-RE-blocked for most) or starting
on the OTHER unparsed extensions in BINARY_FORMATS.md outside the
Havok layer (texture formats `.paa`/`.pat`/`.mi`, audio variants,
non-Tier-1 misc).

### Iter 15 (2026-05-11) — **Python bindings for all new Havok-layer parsers**
Until this iter, the 7 new modules shipped iters 3-12 (pami, par_resource,
motionblending, pamlod, paasmt, paccd, hkx) had **no Python bindings** —
they were rust-only. Mod authors using `import dmm_parser` could not
parse any of them.

Pivoted away from this iter's planned PAR-family corpus audit when the
full-install enumeration took 5+ minutes per query (.paz extraction is
slow). Shipped bindings instead — higher actual value.

Shipped:
- `bind_json_format!` macro in `src/python.rs` for JSON-returning parsers
- 7 module-pyshim aliases to bridge `parse_*_to_json` / `serialize_*_from_json`
- 14 PyO3 functions: `parse_{pami,par,motionblending,pamlod,paasmt,paccd,hkx}_bytes`
  + matching `serialize_*` writers
- All 14 registered in the Python module init

End-to-end verification: rebuilt wheel via `maturin build --release`,
reinstalled with pip, parsed 4 real game files via the new bindings
(`.pami`, `.motionblending`, `.paasmt`, `.paccd`) — all return JSON dicts
with named keys. The .pami binding exposes `mesh_paths` + `xml_body`,
.motionblending exposes `field_records` + `body_b64`, .paasmt exposes
`record_pairs` + `paths`, .paccd exposes `format_version` +
`no_override_byte_count`.

Mod authors can now do:
```python
import dmm_parser as cr
data = open('foo.motionblending', 'rb').read()
parsed = cr.parse_motionblending_bytes(data)
for rec in parsed['field_records']:
    print(rec['name'], rec['type_tag'])
```

Tests: **612 lib tests pass** (no regressions). The PAR-family
corpus audit remains queued for a future iter (next time the .paz
extraction is faster or I sample fewer files).

### Iter 14 (2026-05-11) — **`.pamlod` header bug catch + correction**
With 50 post-iter-9 extractable .pamlod samples (up from 1 in iter 6),
audited the header and caught a critical iter-6 mistake.

**Iter-6 bug:** The parser hardcoded a `version != 1` rejection check
based on 1 sample. Full-corpus distribution of u32@0 shows it's NOT
a version — it's the actual **lod_count** with values 1, 4, 5, 6, 7,
8, 9 (17 + 12 + 11 + 3 + 3 + 3 + 1 = 50). The pre-fix parser would
have rejected 47/50 game files outright.

**Iter-6 second mistake:** What was labeled `lod_count` at offset 12
is actually CONSTANT (always 4 across 50 samples) — renamed to
`geometry_format` to reflect the constant-flag nature.

Shipped corrected header:
```text
u32 lod_count      (1-9 observed)
u32 size_hint
f32 lod_distance
u32 geometry_format  (always 4)
```

Renamed JSON fields, replaced the bad `version != 1` test with
`accepts_observed_lod_count_range` that checks all 7 corpus values.
Added `rejects_zero_lod_count` test. Module docstring + BINARY_FORMATS.md
entry updated with corpus stats.

Tests: **612 lib tests pass** (was 611, +1 net — removed one stale
test, added two new ones).

This is the **second iter (after iter 11)** where the post-unblock
corpus audit revealed a critical wrong-field-name bug from an
earlier "1 sample only" ship. Lesson reinforced: every Tier 1 ship
needs at least N=20+ corpus validation before claiming the field
names are right.

Next iter: similar audit on the still-PAR-family-only `.pab`, `.paa`,
`.pabc`, `.pabv` (we have 30+ samples each post-unblock — same risk
of wrong-version-claim).

### Iter 13 (2026-05-11) — **`.paasmt` structured pairs**
Refined the flat `paths` list into a structured `record_pairs` array,
each entry exposing `{model_path, animset_xml_path}`. Trailing null
bytes (PA-style null-terminated paths inside the length-prefixed
record) are stripped in the structured view; the raw `paths` field
still carries the unmodified strings for round-trip semantics.

This makes the file directly usable by mod tooling: an entry like
```json
{
  "model_path": "character/model/1_pc/.../fishingrod_0001.pac",
  "animset_xml_path": "character/descriptors/animationset/.../fishingrod_0001.animset.xml"
}
```
tells you exactly which animation set to swap when changing a model.

1 new unit test. **611 lib tests pass** (was 610, +1). Python wheel
rebuilt.

IDA MCP checked — still disconnected. Partial-compression RE remains
blocked.

Next iter: pick another deeper-decode target. Candidates:
- `.pamlod` LOD entries (have 30 samples post-iter-9 unblock)
- `.hkx` Havok class registry exploration via static strings dump
- `.motionblending` per-tag value layout (need to guess at staticstringA
  + bool wire formats and verify against samples)

### Iter 12 (2026-05-11) — **`.paccd` corpus-wide constants + sentinel**
Audited the full 1641-file `.paccd` corpus. Key findings:

- The two u32 header fields previously labeled "version_or_count" and
  "record_count_or_flags" are **CONSTANTS** across the entire corpus:
  - `format_version = 14` (1641/1641)
  - `flags = 2` (1641/1641)
  - Renamed in the parser to reflect actual semantics.
- Body byte distribution dominates with **`0xFF` (51%)** — confirmed as
  the **"no-override" sentinel** (slider not customized). Other common
  values: `0x00` (20%, explicit zero), `0x64` (8%, slider max 100),
  `0x32` (6%, slider midpoint 50). Bitfield-like small values 0x01-0x04
  in the long tail.
- Size range across corpus: 298..3370 bytes (avg 434).

Shipped:
- Renamed `version_or_count` → `format_version`, `record_count_or_flags`
  → `flags` in `src/binary/paccd.rs`
- New JSON field `no_override_byte_count` exposing the count of 0xFF
  sentinels in the body (visibility into how customized each character
  is — 0 means fully customized, body_len-12 means default everything)
- 1 new unit test asserting the sentinel count
- Module docstring now carries the full corpus distribution table
- `BINARY_FORMATS.md` entry updated with header constants + sentinel info

Tests: **610 lib tests pass** (was 609, +1).

Per-slider semantic mapping (which body byte = which slider in the
character editor) still requires IDA RE — queued.

Next iter: `.paasmt` (refine path-pair structure) OR wait for IDA to
return for partial-compression-with-differential RE.

### Iter 11 (2026-05-11) — **`.motionblending` full corpus vocabulary**
Past the original 12-extension Tier-1 milestone — pivoting to deeper
field decode. Picked `.motionblending` (highest mod-author value:
animation blend trees, used by every character).

Audited the **entire 1.06 corpus** (1574 .motionblending files):
- Root type: `ParameterizedMotionSpace` (every file)
- Type tags: ONLY 2 in use — `staticstringA` (4722) + `bool` (4764)
- 15 stable named fields per file, deterministic order:
  `_skeletonFileName`, `_animationFileNames`, `_motionPhaseType`,
  `_isLoopMotionBlending`, `_numPhases`, `_animationScale`,
  `_dimensions`, `_thirdDimensionSplitInfo`, `_parameterMinMax`,
  `_keepInitialBlendWeights`, `_weightSmoothingMinSpeed`,
  `_weightSmoothingMaxSpeed`, `_phaseInfo`, `_motionExamples`,
  `_delaunayTriangles`

Shipped:
- `extract_field_records(strings)` — pairs `_`-prefixed names with
  their following type tag into structured `(name, type_tag)` records
- New JSON field `field_records` on parser output (alongside the
  existing flat `scanned_strings`)
- 1 new unit test
- Module docstring updated with full vocabulary inventory
- `BINARY_FORMATS.md` entry updated with corpus statistics

Tests: **609 lib tests pass** (was 608, +1).

Per-tag typed-value decode is queued — recovering the byte layout of
`staticstringA` value bodies and `bool` payloads needs either IDA RE
of the property-record reader or sample-pair labeling experiments.

Next iter: same playbook on `.paccd` (audit the 1641-file corpus for
patterns) or `.paasmt` structure refinement, or wait for IDA to RE
the partial-compression-with-differential format.

### Iter 10 (2026-05-11) — **`.pac` added to PAR family + RE blocker logged**
Two parallel tracks this iter:

**1. Investigated remaining `.pam`/`.pac` failures** after the iter-9
unblock. Discovery: there are TWO distinct partial-compression formats:
- `is_partial=true && compressed_size == uncompressed_size`: raw
  passthrough. Fixed iter 9.
- `is_partial=true && compressed_size < uncompressed_size`: GENUINELY
  partial-compressed format with internal compression. **None of LZ4
  block, LZ4 frame, raw deflate, or zlib decompress the body**. Format
  needs IDA RE to recover. IDA MCP currently disconnected → logged as
  blocker per workplan stop conditions, moved on.

**2. Shipped `.pac` PAR-family classification.** Audit of 20 extractable
samples: all PAR magic. Two versions: `0x01000503` (19/20, main),
`0x01000003` (1/20, older). Added to `par_resource.rs` ParExt enum +
classification table. 1 new unit test. **608 lib tests pass** (was 607).

**Status check** of original 12 Havok-layer extensions:
- 🟢 Tier 1 (full): `.hkx`, `.pami`, `.pab`, `.paa`, `.pabc`, `.pabv`,
  `.motionblending`, `.paasmt`, `.paccd`, `.pamlod`, `.pac`, `.pam` (12/12)
- ⚠️ Remaining gap: ~6 `.pac` + ~17 `.pam` files with genuinely-partial
  compression (waiting on IDA RE)
- REMOVED: `.pacc` (not in install)

**Original workplan goal achieved**: every listed Havok-layer extension
is now AT LEAST Tier 1 classified + round-trip via `body_b64`. Per-format
typed decode for the property records (.motionblending), Havok class
graph (.hkx), and customization sliders (.paccd) are now optional
deepening work, not blocking gaps.

Next iter pivot candidates (workplan complete; lower-priority items):
- Wait for IDA to return → tackle partial-compression-with-differential RE
- Deeper typed decode of `.motionblending` property records
- `.hkx` Havok class registry decode (long-haul)
- Other unparsed extensions in BINARY_FORMATS.md outside Havok layer

### Iter 9 (2026-05-11) — **Partial-compression unblock**
Investigated the long-standing "partial compression extraction not yet
implemented" blocker that was preventing extraction of `.pam`,
`.pamlod`, and most `.pac` files. **Key finding**: the flag was
misnamed in dmm-parser. `compression-nibble == 1` doesn't mean
"partially compressed" (some compressed + some raw chunks); it means
**"stored uncompressed"** (`compressed_size == uncompressed_size`,
raw bytes are the file).

Verified by reading raw bytes of `03_sphere.pam` directly from the
.paz at the chunk offset: bytes start with `50 41 52 20 02 18 00 00`
(literal PAR magic + .pam version). No compression header, no chunk
list — just raw passthrough.

**Fix**: in `src/binary/paz.rs::read_pack_file`, when `is_partial=true`
return the decrypted bytes directly (with a sanity check that
compressed_size == uncompressed_size). One-liner replacing the
`Err(Unsupported)` return.

**Unblock impact** (sampled across 30 files each):
- `.pamlod`: **1/30 → 30/30** (fully unblocked)
- `.pac`: **0/30 → 24/30**
- `.pam`: **3/30 → 13/30**

Remaining `.pam`/`.pac` failures are a DIFFERENT cause (not is_partial)
— investigation queued for next iter. Possibilities: another
compression method not yet wired (QuickLz?), or split-chunk files.

Tests: **607 lib tests pass** (no regression — same as iter 8 baseline).
Python wheel rebuilt + reinstalled via `maturin build --release` + pip
reinstall.

Next iter: investigate the remaining `.pam`/`.pac` extraction errors
(likely QuickLz or another compression). Then deeper field decode on
`.pamlod` (now have 30 samples, can verify the header math).

### Iter 8 (2026-05-11) — **`.paasmt` + `.paccd` shipped**
Two more Tier 1 formats shipped. Initial parser had wrong header
assumption (treated 8-byte prefix as 2 u32 header fields when the
real layout is 1 u32 record_count + length-prefixed records starting
at offset 4). Caught by real-world audit returning 0 paths — fixed.

- **`.paasmt`**: PA Animation Set Matching Table. `u32 record_count
  + (u32 path_len + utf8 path) × (2 × record_count)` records. Pairs
  of (model.pac, animset.xml) per matching entry. Verified: 58 records
  × 2 paths = 116 paths, **100% byte coverage** on the one real file.
- **`.paccd`**: PA Character Customization Data. Header validated:
  `u32 zero_marker (== 0) + u32 version_or_count + u32 record_count`.
  Body packed slider bytes (commonly 0, 50, 100). Real-world check:
  **1641/1641 game files** have zero_marker=0 ✓. Slider semantic
  decode TBD.

Tests: **607 lib tests pass** (was 599, +8 = 4 paasmt + 4 paccd, after
fixing the iter-8 wrong-header bug). BINARY_FORMATS.md updated.

**Status of original 12 Havok-layer extensions:** 9 SHIPPED, 3 BLOCKED
(`.pac`, `.pam`, `.pamlod` — partial-compression in paz unsupported).
`.pacc` NOT FOUND (deprecated).

Next iter pivot: tackle paz partial-decompression — that unblocks 3
formats at once and is the highest-value remaining infrastructure work.
Alternative: deeper field decode of already-shipped Tier 1 formats
(e.g. .motionblending property-record vocabulary, .paccd slider
semantics, .hkx object-graph via Havok class registry).

### Iter 7 (2026-05-11) — **THREE Tier 1 shipped in one iter**
Batch audit of 7 remaining extensions surfaced:

1. **`.pabc`** added to PAR family (version `0x01000134`, 20/20 samples)
2. **`.pabv`** added to PAR family (versions `0x01000136`+`0x01000137`,
   20/20 samples across the two sub-versions)
3. **`.hkx`** classifier shipped (`src/binary/hkx.rs`). Confirmed Havok
   Tag-format ("TAG0" at offset 4), **all 30/30 sampled files contain
   SDK version string `20240200` = Havok 2024.2.00 statically linked**.
   Full object-graph decode requires the in-binary Havok class registry,
   queued as long-term TBD.

Other findings:
- `.pacc` NOT FOUND in install (likely deprecated; was in old workplan).
- `.pac` partial-compression-blocked (0/20 extractable; same family as `.pam`).
- `.paasmt` 1/1 extractable, length-prefixed strings — easy next iter.
- `.paccd` 20/20 extractable, customization data — next iter.

Tests: **599 lib tests pass** (was 592, +7: 2 par_resource + 5 hkx).
BINARY_FORMATS.md updated for all of `.hkx`, `.pabc`, `.pabv`, `.pacc`,
`.pac`. Workplan priority list refreshed.

Remaining unparsed extensions: `.paasmt`, `.paccd`, `.pac`, `.pam`,
`.pamlod` (last 3 blocked by partial-compression). Next iter ship
`.paasmt` (the easy one) + `.paccd`.

### Iter 6 (2026-05-11)
`.pamlod` shipped as **Tier 1** (with partial-compression caveat).
Re-identified: NOT "Animation LOD" — it's **PA Static Mesh LOD**
streaming descriptor. Confirmed via mac binary string xrefs to
`StaticMeshLODStreamingContext` family (8 distinct symbols). Header
layout: `u32 version=1 + u32 size_hint + f32 lod_distance + u32
lod_count + LOD entries + embedded .dds paths`. Real sample decodes:
`03_plane.pamlod`: version=1, lod_distance=3.289, lod_count=4,
texture_paths=['03_plane.dds']. Shipped `src/binary/pamlod.rs` —
header decode + dds-path scanner + byte-perfect round-trip + 6 unit
tests. **592 lib tests pass** (was 586, +6).

**Partial-compression blocker persists** — 29/30 `.pamlod` files
inextractable. Same blocker as `.pam`. Logged as the highest-value
infrastructure unblock — once paz partial-decompression lands, the
full `.pam`/`.pamlod` corpus + likely many `.hkx`/`.pac` files
become testable. Filed as next-iter pivot candidate.

Next iter: audit `.pac`, `.pacc`, `.pabc`, `.pabv`, `.paasmt`, `.paccd`
extractability in one pass — if all hit partial-compression, pivot to
attacking paz partial-decompression itself. If extractable, ship
classifiers in order.

### Iter 5 (2026-05-11)
`.motionblending` shipped as **Tier 1**. Format: named-property binary
record with root type `ParameterizedMotionSpace`. Two version variants
that differ only in header size (v3 = 16 bytes, v4 = 24 bytes — v4
adds 8 bytes of u64 reserved padding). Magic `0xFFFF`. Field names
scanned: `_skeletonFileName`, `_animationFileNames`, `_motionPhaseType`,
`_isLoopMotionBlending`. Type-tag vocabulary observed: `staticstringA`.
Shipped `src/binary/motionblending.rs` with version-dispatched header
decode + length-prefixed-string scanner + byte-perfect round-trip via
`body_b64`. **7 unit tests** (including v3 + v4 layouts + roundtrip).
**586 lib tests pass** (was 579, +7). Real-world audit: **30/30 game
files decode** (15 v3 + 15 v4). BINARY_FORMATS.md updated to 🟢 Tier 1.
Next iter: `.pamlod` (head `01 00 00 00 e0 02 ...` — distinct binary,
also non-PAR).

### Iter 4 (2026-05-11)
**PAR family discovered** — `.pab`, `.paa`, `.pam` all share the same
8-byte header: ASCII magic `"PAR "` + u32 version. Each extension has
its own stable version constant (verified across 30+ real game samples
each):

| Ext   | Version u32 | Bytes (LE)    | Tier   |
|-------|-------------|---------------|--------|
| `.pab`| `0x01050001`| `01 05 00 01` | 🟢 1   |
| `.paa`| `0x01000302`| `02 03 00 01` | 🟢 1   |
| `.pam`| `0x00001802`| `02 18 00 00` | 🟡 1   |

Shipped `src/binary/par_resource.rs` with shared parser + per-extension
version validation. Body is base64 opaque for byte-perfect round-trip;
typed field decode queued for future iters once IDA pseudocode is
available for the per-ext bodies. **8 unit tests** including real-sample
header bytes from `identityskeleton.pab`. `579 lib tests pass` (was
571, +8). BINARY_FORMATS.md: `.pab`/`.paa`/`.pam` all promoted out of
⚠️ ; `.pab`/`.paa` to 🟢 Tier 1, `.pam` to 🟡 (partial-compression
extraction blocks full-corpus testing; format detection itself works).

**Blocker surfaced:** `.pam` files commonly use "partial compression"
inside .paz packages which `dmm_parser.extract_file` returns OSError on
("partial compression extraction not yet implemented"). This is a
pre-existing limitation, not a regression. Filed as future work — once
paz partial-decompression lands, the full `.pam` corpus can be audited.

Next iter: `.motionblending` (head `ff ff 04 00...` — NOT PAR family,
distinct binary format. Iter-3 lesson again: audit before assuming).

### Iter 3 (2026-05-11)
`.pami` shipped as **Tier 1**. Major reclassification: the format is NOT
Havok-related at all — it's plain UTF-8 XML with root `<StaticMeshInstance>`,
storing static mesh placement metadata. The earlier "Animation index"
label in BINARY_FORMATS.md and the workplan was wrong; corrected. Audited
200 sampled .pami files across the 1.06 install: **200/200 conform** to
the XML structure. Implemented `src/binary/pami.rs` with byte-perfect
round-trip + extracted convenience fields (`version`, `mesh_paths`).
4 unit tests + 200-file real-world audit. **571 lib tests pass** (up from
567). BINARY_FORMATS.md updated. Removed `.pami` from the Havok-layer
table — moved to a corrected XML/metadata category. Next iter: pick the
next file extension from the priority list — likely `.pab` (skeletal
volume per Korean error strings) OR re-audit other "Havok-layer"
extensions for misclassifications.

### Iter 2 (2026-05-11)
Decoded all opaque fields in the 3 new pabgh formats via sampling against
the real 1.05 game files. `U32CountU32KeyExtra4` middle is `i32 parent_id`
(-2 = root sentinel, 86% of entries); `U16CountExtra12` real layout is
`u16=FFFF + u16=0 + u32 hash_key + u32 c + u32 offset` — corrected the
key field in the rust parser (was exposing the FFFF sentinel, now exposes
the hash_key). Added `parent_id()` + `aux_c()` accessors on `PabghEntry`,
updated tests + Python diff tool. All 567 lib tests still pass. Next iter:
start on Iter 3 (Havok-layer file format parsing — begin with `.pami` as
the simplest target).

### Iter 1 (2026-05-11)
- **5 broken pabgh tables FIXED** in dmm-parser. They were always broken; the
  diff tool exposed them via the 1.06 diagnostic.
- 3 new pabgh format variants added to `src/binary/pabgh.rs`:
  - `U16CountU8Key` (5-byte entries) → mercenarygroupinfo, mercenaryinfo, relationinfo
  - `U32CountU32KeyExtra4` (12-byte entries) → characterappearanceindexinfo
  - `U16CountExtra12` (16-byte entries) → aieventtableinfo
- Round-trip preserved via `PabghEntry.extra_bytes` (opaque middle bytes).
- 3 unit tests, all pass. Full lib test suite: 564→567 passing.
- Python diff-tool side `parse_pabgh` updated to match.
- Diff re-run confirms: **5 pabgh fails → 0**. 1.06 is now fully diff-able.
- Havok class inventory pass: **1931 unique `hk*` class names extracted**
  from static `mac_extract/crimson_mac_strings.txt`, saved at
  `/tmp/hk_class_inventory.txt`.
- File extension purpose confirmation from Korean error strings:
  - `.pac` = SkinnedMesh resource container (character meshes)
  - `.pab` = skeletal volume file
  - `.hkx` = Havok native animation/ragdoll/mesh
  - `.pami` = animation info file
  - `.motionblending` / `.paa` = animation set entry extensions
  - `.pacc` / `.pabc` / `.paccd` / `.pabv` / `.paasmt` = TBD wrappers

## Iter 2 — Triage the unknown opaque fields in 3 new pabgh formats — **DONE**

**Decoded across all entries in the real game files:**

- ✅ `U16CountU8Key` (5-byte) — no opaque bytes; format fully known (iter 1)
- ✅ `U32CountU32KeyExtra4` (12-byte) — middle 4 bytes = `i32 parent_id`.
  Across 8142 entries: **97 unique values**, dominated by `-2` (0xFFFFFFFE)
  for 7026 entries (86%) = root/no-parent sentinel; small positives (1, 2,
  3, ...) reference a parent group. Exposed via `PabghEntry::parent_id()`.
- ✅ `U16CountExtra12` (16-byte) — full layout:
  `u16 const_0xFFFF + u16 reserved + u32 hash_key + u32 c + u32 offset`.
  Across 940 entries: const=FFFF always, reserved=0 always, hash_key all
  unique (this is the real lookup key), c=0xFFFFFFFF for 99%. The iter-1
  parser had the wrong field as `key` (was exposing the 0xFFFF sentinel);
  corrected to expose `hash_key` at bytes [4:8]. Exposed via
  `PabghEntry.key` + `PabghEntry::aux_c()`.

### Iter 2 shipped:
- `pabgh.rs`: corrected `U16CountExtra12` key offset (bytes[0:4] → bytes[4:8])
- `PabghEntry::parent_id() -> Option<i32>` accessor for 12-byte format
- `PabghEntry::aux_c() -> Option<u32>` accessor for 16-byte format
- Docstrings + format enum comments updated with verified semantics
- Unit tests updated to assert parent_id + aux_c values
- Python diff-tool `parse_pabgh` updated to match (key offset 4 for 16-byte)
- All 567 lib tests pass.

## Iter 3+ — Havok-layer file format parsing

**Long-haul goal:** move all 12 unparsed extensions from `⚠️` to Tier 1 in
`docs/BINARY_FORMATS.md`. Tier 0 may not be achievable for `.hkx` itself
(Havok native — would need full 2024.2 SDK class registry RE).

### File format priority order
| Order | Ext | Purpose | Status |
|---|---|---|---|
| 1 | `.pami` | **Static Mesh Instance (XML)** — NOT animation index | ✅ Tier 1 (iter 3) |
| 2 | `.pab` | Skeletal volume — **PAR family** ver `0x01050001` | ✅ Tier 1 (iter 4) |
| 3 | `.paa` | Animation set entry — **PAR family** ver `0x01000302` | ✅ Tier 1 (iter 4) |
| 4 | `.pam` | Single animation file — **PAR family** ver `0x00001802` | 🟡 Tier 1 (iter 4) — partial-compression extract blocker |
| 5 | `.motionblending` | Distinct binary, head `ff ff 04 00` — NOT PAR family | TODO — next iter |
| 6 | `.pamlod` | Distinct binary, head `01 00 00 00 e0 02 ...` | TODO |
| 7 | `.pac` | SkinnedMesh character archive | TODO — Havok wrapper |
| 8 | `.pacc` | `.pac` variant | TODO |
| 9 | `.pabc` | Unknown | TODO — audit |
| 10 | `.pabv` | Unknown | TODO — audit |
| 11 | `.paasmt` | Unknown | TODO — audit |
| 12 | `.paccd` | Unknown | TODO — audit |
| 13 | `.hkx` | Havok native (magic `57 E0 E0 57`) | TODO — Havok tag-format reader |

**Iter-3 lesson:** classifications in the original workplan were inferred
from filename guessing, not from inspecting actual files. Each iter should
START with "dump the first 256 bytes from a real sample" before assuming
the format. The .pami misclassification cost ~0 (the parser is correct
either way) but later items might have similar surprises (e.g., `.pam`
might also be XML, not Havok).

### Per-format work template
1. Find a sample file in the live install (`Crimson Desert/packages/...`)
2. Dump first 256 bytes — identify magic / header structure
3. Cross-reference Korean error strings in `crimson_mac_strings.txt` —
   field names PA exposes for that format
4. Grep for the format's parser function in mac binary
   (`crimson_mac_functions.txt`) — read pseudocode if IDA MCP is up,
   else infer from string xrefs
5. Implement a `parse_<ext>_to_json` + `serialize_<ext>_from_json`
6. Add round-trip test using one or more real sample files
7. Wire into `dmm_parser` python binding + dispatch
8. Update `docs/BINARY_FORMATS.md`: ⚠️ → Tier 1
9. Update mod-author guide if format is mod-relevant

## Iter N — Cleanup + documentation

- [ ] Update `docs/BINARY_FORMATS.md` with 3 new pabgh variants table.
- [ ] Update `docs/ENGINE_INTERNALS.md` Havok section with the 1931-class
      inventory + scope statement on what dmm-parser can/will parse.
- [ ] Update `docs/MOD_AUTHOR_GUIDE.md` (if exists) with new pabgh formats
      so mod tooling can reuse the parser.
- [ ] Update `dmm-pabgb-aio/diff_pabgb_versions.py` docstring with format
      list.

## Stop conditions for the loop
- Test failure → STOP (don't ship broken code)
- Build failure → STOP
- IDA MCP needed for a step + IDA is disconnected → SKIP that item, move
  to next, document blocker
- All Havok wrappers shipped → STOP, final summary
- Recurring cron 7-day auto-expire → final summary

## Open blockers
- **IDA MCP disconnected** as of last iter — can't do live decompilation.
  Fallback to static `mac_extract/` dumps where possible. RE work that
  truly needs decompile output will queue until IDA is back.
