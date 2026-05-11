# Binary Formats Reference

Look-up reference for every Crimson Desert binary format dmm-parser
touches. Use this for "what's the byte layout of X" or "where does field
Y live" questions. For research notes about the engine internals (Havok,
EffectData, etc.) see `ENGINE_INTERNALS.md`.

> **Last consolidated:** 2026-05-10. Merged from `archive-format.md`,
> `FORMATS.md`, `PAATT_BASEDATA_FIELDS.md`, `TEXTURE_VPATH_CHEATSHEET.md`.
> The original four docs are deleted; their content lives in the sections
> below verbatim.

## Contents

- [Extension Reference (all 86 PA file formats)](#extension-reference-all-86-pa-file-formats)
- [Archive Format (PAZ / PAMT / PAPGT / Trie)](#archive-format-paz--pamt--papgt--trie)
- [File Format Reference (FORMATS)](#file-format-reference-formats)
- [PAATT BaseData Field Layout](#paatt-basedata-field-layout)
- [Texture VPath Cheatsheet](#texture-vpath-cheatsheet)

---

## Extension reference (all 86 PA file formats)

Comprehensive index of every PA file extension visible in PAZ archives,
sourced from pycrimson's `file_format_notes.md` plus dmm-parser source
handlers. Use this as the lookup-by-extension table.

Status legend:
- ✅ **dmm-parser fully parses** (typed read + write, byte-perfect roundtrip)
- 🟡 **partial** (typed prefix or classify-only, not full field-level)
- 📚 **pycrimson can parse** (reflection format, harvested into v3.1_reflection_schema.json)
- 🚫 **pycrimson blocked** (extracted to _research_cache/ but parser bug)
- ⚠️ **unparsed** (no handler in either tool)

### Archive / metadata layer

| Ext | Status | Purpose | Handler |
|---|---|---|---|
| `.paz` | ✅ | Pack archive (compressed asset container) | `src/binary/paz.rs` (PackGroupBuilder + LZ4 + ChaCha20) |
| `.pamt` | ✅ | Per-group pack metadata (file→paz index) | `src/binary/pamt.rs` |
| `.papgt` | ✅ | Pack group tree (meta/0.papgt — top-level dispatch) | `src/binary/papgt.rs` |
| `.pabgh` | ✅ | Per-table offset-index file (companion to .pabgb) | `src/binary/pabgh.rs` |
| `.pabgb` | ✅ | Raw table entries (122 *Info tables decode through this) | `src/binary/pabgb.rs` + `src/tables/*` |
| `.pathc` | ⚠️ (structure identified) | **Texture header collection** (2.3 MB). Header: `u64 zero + u32 size_a(148) + u32 size_b(672) + u32 total_records(279221) + u32 + records`. Body is dense binary texture metadata — only 14 embedded `.dds` paths despite huge file size. Per-record decode needs IDA. | none |
| `.binarystring` | 🟢 Tier 1 | **Packed UTF-8 string list** — `u16 count + (u8 len + utf8) × N`. 1/1 game file (`gamestring.binarystring`, 19259b). | `src/binary/binarystring.rs` — strings extracted as a list, byte-perfect round-trip |

### Localization

| Ext | Status | Purpose | Handler |
|---|---|---|---|
| `.paloc` | ✅ | Localization string entries (encrypted UTF-8) | `src/binary/paloc.rs` |

### Game-data file formats

| Ext | Status | Purpose | Handler |
|---|---|---|---|
| `.paatt` | 🟡 | AttackInfo wrapper (per-attack data block) | `src/binary/paatt.rs` + `src/binary/paatt_basedata.rs` (typed BaseDataV0-V3, 22 _unkXXXX remain) |
| `.pamhc` | 🟡 | (file-format table; partial decode) | `src/tables/pamhc/` |
| `.paac` | 🟡 | (file-format table; partial decode) | `src/tables/paac/` |
| `.pappt` | 🟡 | (file-format table; partial decode) | `src/tables/pappt/` |
| `.paacdesc` | 🚫 | unknown | pycrimson blocked |
| `.paprojdesc` | 🟡 Tier 1 (string section) | **Projectile descriptor** — string-prefix section uses `.binarystring` format (`u16 count + (u8 len + utf8) × N`); use `parse_binarystring_bytes`. 32-byte trailing record after the string list (per-projectile data, TBD). | `src/binary/binarystring.rs` covers the prefix section |
| `.paproj` | ⚠️ (structure identified) | **Projectile data** binary. Layout: `u32 record_count + u32 type_hash + u32 something + zero-padding + records`. Per-record structure pending RE. | none |
| `.pas` | 🚫 | unknown | pycrimson blocked |
| `.pashv` | 🟡 Tier 1 (string prefix) | **AI Share Values** — string-prefix section uses `.binarystring` format (`u16 count + (u8 len + null-terminated utf8) × N`). Verified on `allweaponcommonaisharevalue.pashv`: 193/193 strings parsed (e.g. `c_sequencer_movetype`, `c_sequencer_aiactiontype`). Tail section (~85% of file) contains the actual share values keyed by string index — TBD. Use `parse_binarystring_bytes` for the string section. | `src/binary/binarystring.rs` covers prefix |
| `.papr` | 🟢 Tier 1 (classify+round-trip) | **PA particle/projectile resource** — PAR family, version `0x01000135`. | `src/binary/par_resource.rs` |
| `.paschedule` | 🟡 | schedule data (typed prefix + opaque body) | `src/binary/paschedule.rs` |
| `.paschedulectx` | 🟡 Tier 1.5 (count + paths) | **Schedule context** binary. `u32 count + per-record (u32 hash + u8 flag + u32 len + utf8 path)` referencing `sequencer/.../*.paschedule` files. Verified on `schedulecontext.paschedulectx`: count=4088, scanned 4118 paths. Use `parse_count_record_table_bytes`. | `src/binary/count_record_table.rs` |
| `.paschedulepath` | 🟡 | schedule path data | `src/binary/paschedulepath.rs` |
| `.pastage` | 🟡 | stage data | `src/binary/pastage.rs` |
| `.pai` | ⚠️ | AI chart data, large binary (3 MB). u32 count header. Audit pending. | none |
| `.pi` | ⚠️ | NOT FOUND in 1.06 install | none |
| `.pma` | 🟢 Tier 1 | **UTF-8 XML** (no BOM), root `<ARFaceAnimation>` — face/animation reference. Use `parse_xml_bytes`. | `src/binary/xml_resource.rs` |
| `.pmb` | ⚠️ | NOT FOUND in 1.06 install — likely deprecated or generated at runtime | none |

### Reflection-format files (pycrimson territory)

| Ext | Status | Purpose | Handler |
|---|---|---|---|
| `.prefab` | 📚 | Scene-object prefab (component composition) | pycrimson `parse-serialized-file` ✅ |
| `.parg` | 📚 | Animation/render group | pycrimson ✅ |
| `.pasg` | 📚 | State graph | pycrimson ✅ |
| `.paseqc` | 📚 | Sequencer game-play data | pycrimson ✅ |
| `.paa_metabin` | 📚 | Animation metadata (empty wrapper) | pycrimson ✅ (yields empty AnimationMetaData) |
| `.meshinfo` | 🚫 | Mesh metadata | pycrimson `TransferInstructionFlags` enum bug |
| `.palevel` | 🚫 | Level descriptor | pycrimson parc-header buffer underflow |
| `.pae` | 🚫 | PA effect envelope | same parc-header bug |
| `.paem` | 🚫 | PA effect emitter | parc-header bug |
| `.paseq` | 🚫 | Sequence (custom header) | pycrimson type-index IndexError |
| `.uianiminit` | 🚫 | UI animation init (custom header) | same type-index bug |
| `.linkedsceneobject` | 🚫 | Scene object link | pycrimson buffer underflow |
| `.binarygimmick` | 🚫 | Gimmick binary data | pycrimson buffer underflow |
| `.binarygimmickcacheddata` | 🚫 | Gimmick cached data | pycrimson buffer underflow |
| `.binarygimmickframeevent` | 🚫 | Gimmick frame events | pycrimson buffer underflow |
| `.seqmt` | 🚫 | (`! ???????` in pycrimson notes) | pycrimson buffer underflow |
| `.paseqh` | 🟡 Tier 1.5 (count + names) | **Sequence header** binary. `u32 record_count + per-record (u32 name_len + name + binary fields)`. Verified on `sequencerstageheader.paseqh`: count=2949, scanned 6042 names (~2 per record). Use `parse_count_record_table_bytes`. Per-record typed value decode is TBD. | `src/binary/count_record_table.rs` |
| `.questgaugecount` | ⚠️ (structure identified) | **Quest gauge counter**. Layout `u32 count(=382) + variable-size records separated by `0xFFFFFFFF` markers`. 305 separator markers across 25350 u32 values total. Each record carries `u32 hash + u32 sub_value + u32 ref + u32 zero + 0xFFFFFFFF + N×u32 extra`. Per-record decode needs IDA. Only 1 file in install. | `parse_count_record_table_bytes` returns count successfully but extracts no string names (records are binary-only) |

### Havok-layer files (Layer B per ENGINE_INTERNALS.md)

| Ext | Status | Purpose | Handler |
|---|---|---|---|
| `.hkx` | 🟢 Tier 1 (classify+SDK-version) | Havok native **Tag-format** ("TAG0" magic at offset 4). Iter-7 audit: **30/30 game files** contain SDK string `20240200` = **Havok 2024.2.00**. Full object-graph decode requires the in-binary Havok class registry RE (TBD). | `src/binary/hkx.rs` — TAG0 magic + SDKV version extraction + round-trip |
| `.pac` | 🟢 Tier 1 (classify+round-trip) | **PAR family** — main ver `0x01000503` (19/20), older ver `0x01000003` (1/20). Iter 10: added to PAR family classifier. 24/30 extractable; remaining 6 use a partial-compression-with-size-differential format that needs RE (IDA blocker, queued). | `src/binary/par_resource.rs` |
| `.pacc` | ⚠️ | (variant of .pac, deprecated?) | NOT FOUND in 1.06 install — likely removed |
| `.pam` | 🟢 Tier 1 (classify+round-trip) | Single animation file, **PAR family** (magic `"PAR "` + version `0x00001802`). Partial-compression blocker fixed iter 9 (was misnamed flag = "stored uncompressed"). Some .pam still fail on other compression types — queued. | `src/binary/par_resource.rs` |
| `.pami` | 🟢 Tier 1 | **Static Mesh Instance** (XML, root `<StaticMeshInstance>`) — earlier "Animation index" label was wrong; corrected iter 3 (verified 200/200 game files). NOT a Havok-layer file. | `src/binary/pami.rs` (parse + serialize, byte-perfect round-trip, version + mesh_paths extracted) |
| `.pamlod` | 🟢 Tier 1 | **PA Mesh LOD** descriptor (NOT "Animation LOD") — `StaticMeshLODStreamingContext`. **Iter-14 corrected header** (was wrong in iter 6): `u32 lod_count (1-9 observed) + u32 size_hint + f32 lod_distance + u32 geometry_format (always 4) + LOD entries + .dds paths`. 50/50 sampled files parse cleanly post iter-14 correction. Corpus sizes: 802b to 2.4 MB. | `src/binary/pamlod.rs` |
| `.motionblending` | 🟢 Tier 1 | Motion blending — named-property records, root type `ParameterizedMotionSpace`. Two versions (v3 16-byte header, v4 24-byte header), magic `0xFFFF`. 30/30 sampled files decode. **Full corpus vocabulary** (1574 files, iter 11): 15 stable fields per file (`_skeletonFileName`, `_animationFileNames`, ... `_delaunayTriangles`), 2 type tags (`staticstringA`, `bool`). | `src/binary/motionblending.rs` — header + field/type pairs exposed via `field_records`, body round-trip. Typed value decode queued. |
| `.pab` | 🟢 Tier 1 (classify+round-trip) | Skeletal volume, **PAR family** (`"PAR "` + ver `0x01050001`) — verified 30/30 samples | `src/binary/par_resource.rs` |
| `.paa` | 🟢 Tier 1 (classify+round-trip) | Animation set entry, **PAR family** (`"PAR "` + ver `0x01000302`) — verified 20/20 samples | `src/binary/par_resource.rs` |
| `.pabc` | 🟢 Tier 1 (classify+round-trip) | **PAR family** ver `0x01000134`. 20/20 game samples decode. | `src/binary/par_resource.rs` |
| `.pabv` | 🟢 Tier 1 (classify+round-trip) | **PAR family**, two sub-versions: `0x01000136` (14/20) + `0x01000137` (6/20). All samples decode. | `src/binary/par_resource.rs` |
| `.paasmt` | 🟢 Tier 1 | **PA Animation Set Matching Table** — maps `.pac` model paths to `.animset.xml` descriptor paths. `u32 record_count + (u32 path_len + utf8 path) × (2 × record_count)`. 1/1 game file, **100% byte coverage**: 58 records × 2 paths = 116 paths. Iter 13: paths grouped into structured `record_pairs` for mod tooling convenience. | `src/binary/paasmt.rs` |
| `.paccd` | 🟢 Tier 1 | **PA Character Customization Data**. Header (verified across **full 1641-file corpus**): `u32 zero=0 + u32 format_version=14 + u32 flags=2` (both constants — was misnamed as "version_or_count" + "record_count_or_flags" in iter 8). Body is packed slider bytes — **`0xff` is the "no-override" sentinel** (51% of body bytes). Common slider values: 0 (20%), 100 (8%), 50 (6%). | `src/binary/paccd.rs` — header + no-override count + round-trip. Per-slider semantic mapping TBD (needs IDA RE). |

### Texture / mesh assets

| Ext | Status | Purpose | Handler |
|---|---|---|---|
| `.dds` | 🟡 | DDS texture (DXT/BC compressed) | `src/binary/dds.rs` (classify + validate + vpath inference) |
| `.pat` | 🟢 Tier 1 (classify+round-trip) | **PA texture data** — PAR family, version `0x01000404` (verified 3/3 sampled). | `src/binary/par_resource.rs` |
| `.imp` | 🟢 Tier 1 | **Impostor billboard descriptor** — fixed 72 bytes: `magic "IMP " + u32 reserved=256 (constant) + 64-byte body`. 30/30 sampled. | `src/binary/impostor.rs::parse_imp_to_json` |
| `.impostor` | 🟢 Tier 1 | **Impostor spatial parameters** — fixed 48 bytes = 12 × little-endian f32. floats[4..6] always 0 (rotation padding); other floats vary per mesh. | `src/binary/impostor.rs::parse_impostor_to_json` — structured `floats` array, byte-perfect round-trip |
| `.material` | 🟢 Tier 1 | **UTF-8 XML** with BOM, root `<Technique Name="...">`. 10/10 sampled. | `src/binary/xml_resource.rs` (shared with `.technique`, `.mi`, `.spline`, `.spline2d`) |
| `.technique` | 🟢 Tier 1 | **UTF-8 XML** with BOM, root `<Category Name="...">`. 10/10 sampled. | `src/binary/xml_resource.rs` |
| `.mi` | 🟢 Tier 1 | **UTF-8 XML** (no BOM), root `<SkinnedDecalProperty>` — skinned-decal material instance. 10/10 sampled. | `src/binary/xml_resource.rs` |

### Audio

| Ext | Status | Purpose | Handler |
|---|---|---|---|
| `.bnk` | 🟡 | Wwise SoundBank | `src/binary/bnk.rs` (classify + parse_bnk dict) |
| `.wem` | 🟡 | Wwise encoded audio | `src/binary/wem.rs` (raw passthrough) |
| `.pasound` | 🚫 | PA sound metadata | pycrimson buffer underflow |

### Navigation / world

| Ext | Status | Purpose | Handler |
|---|---|---|---|
| `.nav` | ⚠️ | Navigation mesh | none |
| `.road` / `.roadsector` / `.roadidx` | ⚠️ | Road geometry/index | none |
| `.spline` | 🟢 Tier 1 | **UTF-8 XML** (no BOM), root `<SplineDataGroup>` — 3D spline curves. 10/10 sampled. | `src/binary/xml_resource.rs` |
| `.spline2d` | 🟢 Tier 1 | **UTF-8 XML** (no BOM), root `<SplinePresetData>` — 2D spline presets. 10/10 sampled. | `src/binary/xml_resource.rs` |

### Save / template

| Ext | Status | Purpose | Handler |
|---|---|---|---|
| `.save` | ✅ | Save game (encrypted ChaCha20) | DMM `save_engine` module + `src/save/envelope.rs` |

### Misc / unknown

| Ext | Status | Purpose | Handler |
|---|---|---|---|
| `.pbd` / `.pcg` / `.dat` / `.ani` / `.pix` | ⚠️ | unknown PA formats | none |
| `.ies` | 🚫 | color (lighting profile?) | pycrimson buffer underflow |
| `.xml` | ⚠️ | XML (encrypted on disk) | none |
| `.txt` | ✅ | UTF-8 text | n/a |

### Standard third-party formats (asset-only, modded by replacement)

| Ext | Purpose |
|---|---|
| `.png` | PNG image |
| `.ttf` | TrueType font |
| `.mp4` | Video |
| `.cur` | Cursor |
| `.css` | CSS stylesheet |
| `.html` / `.thtml` | HTML / template HTML |

### Summary

**Total extensions cataloged: 86** (per pycrimson file_format_notes.md
+ dmm-parser source).

- **✅ Fully parsed:** 8 (paz, pamt, papgt, pabgh, pabgb, paloc, save, std)
- **🟡 Partial:** 11 (paatt, pamhc, paac, pappt, paschedule, paschedulepath, pastage, dds, bnk, wem)
- **📚 pycrimson catalogued:** 5 (prefab, parg, pasg, paseqc, paa_metabin)
- **🚫 pycrimson blocked:** 14 (meshinfo, palevel, pae, paem, paseq, uianiminit + 8 small)
- **⚠️ Unparsed:** ~50 in long tail (Havok layer, mesh/texture variants, nav, misc)

---

## Archive Format (PAZ / PAMT / PAPGT / Trie)

# Crimson Desert Archive Format

## File Hierarchy

```
game_dir/
├── meta/
│   └── 0.papgt          # Master index — lists all pack groups
├── 0000/
│   ├── 0.pamt           # VFS index for this group (trie + file metadata)
│   ├── 0.paz            # Concatenated compressed/encrypted file data
│   ├── 1.paz
│   └── ...
├── 0001/
│   ├── 0.pamt
│   └── *.paz
└── ...
```

## PAPGT (Pack Group Tree Meta) — `meta/0.papgt`

Master index listing all pack groups in the game.

### Header (12 bytes)
| Offset | Type | Field |
|--------|------|-------|
| 0 | u32 | platform_magic (preserve verbatim — DO NOT overwrite) |
| 4 | u32 | header_crc (Jenkins hashlittle2 of post-header data) |
| 8 | u8 | entry_count |
| 9 | u16 | lang_type (locale/language enum) |
| 11 | u8 | reserved (zero) |

**Header CRC offset gotcha.** The CRC field is at **bytes 4–7**, NOT
bytes 0–3. Bytes 0–3 are the platform magic and must be preserved.
DMM shipped a bug in pre-release.11 (a strip path wrote the recomputed
hash to bytes 0–3, clobbering the magic and leaving the real CRC
stale) which broke parse on next mount with `Checksum mismatch`.

```rust
// CORRECT
let crc = hashlittle(&papgt[12..], INTEGRITY_SEED);
papgt[4..8].copy_from_slice(&crc.to_le_bytes());

// WRONG — clobbers platform_magic, leaves real CRC stale
papgt[0..4].copy_from_slice(&crc.to_le_bytes());
```

PAMT uses the *opposite* layout (its CRC sits at bytes 0–3), so
don't copy-paste between PAMT and PAPGT writers.

### Entry (repeated `entry_count` times)
| Offset | Type | Field |
|--------|------|-------|
| 0 | u8 | is_optional |
| 1 | u16 | language (bitmask, 0x3FFF = ALL) |
| 3 | u8 | always_zero |
| 4 | u32 | group_name_offset (into group_names_buffer) |
| 8 | u32 | pack_meta_checksum (checksum of group's 0.pamt post-header) |

### Group Names Buffer
- i32 length prefix
- Null-terminated C strings, referenced by entries via offset

### Load Order
The game reads entries front-to-back. First match wins for file resolution. This is how mods override game files — see [Mod Loading](#mod-loading-overlay-approach).

---

## PAMT (Pack Meta) — `{group}/0.pamt`

Virtual filesystem index for a single pack group. Maps directory paths and file names to offsets within `.paz` chunk files.

### Header (12 bytes)
| Offset | Type | Field |
|--------|------|-------|
| 0 | u32 | checksum (Jenkins hashlittle2 of post-header data) |
| 4 | u16 | count |
| 6 | u8 | unknown |
| 7 | u8[3] | encrypt_info |
| 10 | u8[2] | padding |

### Post-Header Structure
1. **Chunks array** — `(id: u32, checksum: u32, size: u32)` per `.paz` file
2. **Dir names trie buffer** — trie-encoded directory paths (i32 length prefix)
3. **File names trie buffer** — trie-encoded file names (i32 length prefix)
4. **Directories array** — `(name_checksum: u32, name_offset: i32, file_start_index: u32, file_count: u32)`
5. **Files array** — `(name_offset: i32, chunk_offset: u32, compressed_size: u32, uncompressed_size: u32, chunk_id: u16, flags: u8, unknown0: u8)`

### File Flags Byte
- Bits 0-3: compression type (0=None, 2=LZ4, 3=Zlib, 4=QuickLZ)
- Bits 4-7: crypto type (0=None, 1=ICE, 2=AES, 3=ChaCha20)

---

## PAZ (Pack Archive) — `{group}/{n}.paz`

Headerless concatenated file data. Each file's raw bytes (after compression and optional encryption) are written sequentially. File locations are tracked by the PAMT index via `chunk_id` (which `.paz` file) and `chunk_offset` (byte offset within that file).

### Processing Pipeline
1. Read raw file data
2. Compress (LZ4, Zlib, or None)
3. Encrypt (ChaCha20, AES, ICE, or None)
4. Append to current chunk; split to new `.paz` when `max_chunk_size` exceeded

---

## Trie Buffer Format

Used by PAMT for both directory names and file names. A compact prefix-sharing encoding.

### Entry Format
| Offset | Type | Field |
|--------|------|-------|
| 0 | i32 (LE) | parent_offset (-1 for root entries) |
| 4 | u8 | string_length |
| 5 | u8[string_length] | string_data |

### Encoding Rules
1. Paths are split on `/` into directory segments
2. Non-root segments get `/` prepended (e.g., `"/binary__"`, `"/client"`)
3. Siblings at each trie level are radix-compressed (byte-level prefix sharing)

### Example
For paths `gamedata/binary__` and `gamedata/binarygimmickchart__`:
```
offset=0   parent=-1  data="gamedata"
offset=13  parent=0   data="/binary"           # shared prefix
offset=25  parent=13  data="__"                # completes "binary__"
offset=32  parent=13  data="gimmickchart__"    # completes "binarygimmickchart__"
```

To reconstruct a full string, walk parent pointers to root and concatenate.

---

## Checksum

Jenkins hashlittle2 with constant seed `0xDEBA1DCD`.

Used in:
- PAMT header (covers post-header data)
- PAPGT header (covers post-header data)
- PAZ chunk verification
- Directory name hashing in PAMT

---

## Mod Loading (Overlay Approach)

The game resolves files by scanning PAPGT entries front-to-back. First match wins.

### How It Works
1. **Create a new pack group** (e.g., `0036/`) containing modified files packed into `.paz` + `0.pamt`
2. **Insert the mod entry at the front** of both the PAPGT entries list and the group_names buffer
3. **Replace `meta/0.papgt`** with the updated version

The original game archives are never modified. When the game looks up a file, it finds the mod's version first (because the mod's entry is at index 0), effectively overlaying the original.

This is the same approach used by other Crimson Desert mod loaders.

### Pipeline (automated by `pack_mod()`)
```
mod files on disk
    → compress (LZ4/Zlib) + optional encrypt
    → write .paz chunks
    → build trie buffers for dir/file names
    → create 0.pamt with checksums
    → load original 0.papgt
    → insert mod entry at front (upsert)
    → write updated 0.papgt
```

## PALOC (Pearl Abyss Localization) — `*.paloc`

Pearl Abyss localization format, used for item names, descriptions, UI text, and all in-game strings. Each language has its own paloc file (e.g. `localizationstring_eng.paloc`, `localizationstring_kor.paloc`).

### Format

```
+0x00  entries[]                    ← back-to-back entries until last 4 bytes
+...   entry_count: u32 LE          ← last 4 bytes of file
```

### Entry Layout (per record)

```
+0x00  category: u64 LE             ← only low byte significant; upper 7 bytes always 0
+0x08  key_len: u32 LE              ← length of key string in bytes
+0x0C  key: u8[key_len]             ← UTF-8, no null terminator
+...   value_len: u32 LE            ← length of value string in bytes
+...   value: u8[value_len]         ← UTF-8, no null terminator
```

### Category Codes

The low byte of the `category` u64 indicates the string's type. Observed values:

| Code | Meaning |
|---|---|
| `0x03` | Character names + descriptions |
| `0x07` | Items (currencies, materials) and their descriptions |
| `0x2F` | UI / general game text |
| `0x70` | Item name (matches `(item_key << 32) \| 0x70` formula) |
| `0x71` | Item description (matches `(item_key << 32) \| 0x71` formula) |

Other category codes exist for NPCs, quests, etc. (full enumeration TBD).

### Key String Pattern

For item-related entries, the `key` is the **decimal representation** of `(target_id << 32) | tag_byte`. For example, item key `1` (vanilla "Copper") has:
- name lookup key: `"4294967408"` = `(1 << 32) | 0x70`
- desc lookup key: `"4294967409"` = `(1 << 32) | 0x71`

Custom items at `target_id = 999001` use:
- name lookup key: `"4290772592"` = `(999001 << 32) | 0x70`
- desc lookup key: `"4290772593"` = `(999001 << 32) | 0x71`

### Encryption / Compression

Paloc files stored inside `.paz` archives use the PAZ entry's compression (LZ4) and encryption (ChaCha20) — **NOT** a paloc-internal envelope. After extraction via PAZ tooling, the resulting bytes are plain and use the format documented above.

### Rust API

- `crate::binary::paloc::LocalizationFile::parse(data)` — parse from plain bytes
- `crate::binary::paloc::LocalizationFile::to_bytes()` — serialize to plain bytes
- `crate::binary::paloc::parse_paloc_to_json(data)` — parse to JSON form `[{category, key, value}]`
- `crate::binary::paloc::serialize_paloc_from_json(items)` — inverse

### Dispatch Names

Recognized by `dmm_parser::dispatch::parse_table_to_json` and `serialize_table_from_json`: `"paloc"`, `"paloc.pamt"`, `"localizationstring"`.

### Hex Pattern

See `references/paloc.hexpat` for an ImHex pattern file documenting the format.

### Sample File

Verified against `localizationstring_eng.paloc` from PAZ group 0020: 15.4 MB, 172,152 entries, all parse cleanly with byte-perfect round-trip.

---

## File Format Reference (FORMATS)

# Crimson Desert Binary Formats

> Single-page reference for every binary format the dmm-parser library
> understands. Each section links to the authoritative recon notes,
> hexpat pattern, and Rust/Python entry point for that format.
>
> For format **uses** (mods, packing, validation) see
> `docs/MOD_AUTHOR_GUIDE.md` and `docs/api.md`. The mod-author guide's
> §0.5 is the canonical full-extension inventory (40+ `.pa*` and standard
> formats) — this file covers only the ones dmm-parser parses end-to-end.
>
> **Naming.** Every `pa*` extension is from the **Pearl Abyss** engine
> (`pa::` namespace; debug strings in the binary include
> `PearlAbyssEngine.Debug.PauseFrameIndex` etc.). PABGB/PABGH specifically
> = Pearl Abyss + **Binary Group** + Body/Header (confirmed by the
> `"BinaryGroup"` string at `0x1072db1e9` in the Mac binary).

---

## Contents

- [At a glance](#at-a-glance)
- [1. PAPGT — pack group tree](#1-papgt--pack-group-tree)
- [2. PAMT — pack metadata](#2-pamt--pack-metadata)
- [3. PAZ — pack-zone blocks](#3-paz--pack-zone-blocks)
- [4. Trie buffer](#4-trie-buffer)
- [5. PAOC / PALOC — localization](#5-paoc--paloc--localization)
- [6. PABGB / PABGH — tabular game data](#6-pabgb--pabgh--tabular-game-data)
- [7. DDS — DirectDraw Surface (Crimson flavor)](#7-dds--directdraw-surface-crimson-flavor)
- [8. WEM — Wwise audio clip](#8-wem--wwise-audio-clip)
- [9. BNK — Wwise soundbank](#9-bnk--wwise-soundbank)
- [10. SAVE — save file envelope](#10-save--save-file-envelope)
- [11. Sequencer & attack asset family (PAATT / PASEQ / PASEQC / PASTAGE / PASCHEDULE / PASCHEDULEPATH)](#11-sequencer--attack-asset-family)
- [12. Decode tiers and the Tier 1 promotion goal](#12-decode-tiers-and-the-tier-1-promotion-goal)
- [13. Hexpat conventions](#13-hexpat-conventions)
- [14. Adding a new format](#14-adding-a-new-format)

---

## At a glance

| Format         | Role                              | Tier  | Notes ref                  | Hexpat                        | Rust module                  |
|----------------|-----------------------------------|-------|----------------------------|-------------------------------|------------------------------|
| **PAPGT**      | Pack-group tree (root index)      | 1     | `docs/archive-format.md`   | `references/papgt.hexpat`     | `src/binary/papgt.rs`        |
| **PAMT**       | Per-group pack metadata           | 1     | `docs/archive-format.md`   | `references/pamt.hexpat`      | `src/binary/pamt.rs`         |
| **PAZ**        | Compressed/encrypted file blocks  | 1     | `docs/archive-format.md`   | —                             | `src/binary/paz.rs`          |
| **Trie buffer**| Compact name index (radix tree)   | 1     | `docs/archive-format.md`   | —                             | `src/binary/trie.rs`         |
| **PAOC / PALOC** | Localization string tables     | 1     | `references/paloc_notes.md`| `references/paloc.hexpat`     | `src/binary/paloc.rs`        |
| **PABGB**      | Tabular game data containers      | 1     | `docs/archive-format.md`   | —                             | `src/item_info/`, `src/tables/` |
| **DDS**        | DirectDraw Surface (textures)     | 1     | `references/dds_notes.md`  | `references/dds.hexpat`       | `src/dds/`                   |
| **WEM**        | Wwise audio clip (RIFF-WAVE)      | 1     | `references/wwise_notes.md`| `references/wem.hexpat`       | `src/audio/wem.rs`           |
| **BNK**        | Wwise soundbank                   | 1     | `references/wwise_notes.md`| `references/bnk.hexpat`       | `src/audio/bnk.rs`           |
| **SAVE**       | Save file envelope (encrypted)    | 1 (envelope only — body deferred) | `references/save_notes.md` | `references/save.hexpat`      | `src/save/envelope.rs`       |
| **PAATT**      | Per-weapon attack info            | **1.5 → goal: 1** | (see source) | —                             | `src/binary/paatt.rs`        |
| **PASEQ**      | Sequencer / cutscene script       | **1**             | (see source) | —                             | `src/binary/paseq.rs`        |
| **PASEQC**     | Compiled sequencer chart          | **1.5 → goal: 1** | (see source) | —                             | `src/binary/paseqc.rs`       |
| **PASTAGE**    | Sequencer stage chart             | **1**             | (see source) | —                             | `src/binary/pastage.rs`      |
| **PASCHEDULE** | NPC time-of-day / activity schedule | **1.5 → goal: 1** | (see source) | —                           | `src/binary/paschedule.rs`   |
| **PASCHEDULEPATH** | NPC waypoint / path data      | **1.5 → goal: 1** | (see source) | —                             | `src/binary/paschedulepath.rs` |

All formats are **little-endian**. See [§12](#12-decode-tiers-and-the-tier-1-promotion-goal) for what the tier markers mean and the active Tier 1 promotion goal for the bottom six rows.

---

## 1. PAPGT — pack group tree

The root index. Lists every PAMT (`<group>/0.pamt`) the game knows
about, plus the checksum that proves the PAMT hasn't been tampered
with.

```
+------- 0x00 ------- header (12 bytes total) -------------------------------+
|  bytes 0-3   u32 platform_magic   (preserve, do NOT overwrite)             |
|  bytes 4-7   u32 header_crc       (hashlittle of post-header data)         |
|  byte  8     u8  entry_count                                               |
|  bytes 9-10  u16 lang_type                                                 |
|  byte  11    u8  reserved (zero)                                           |
+------- 0x0C ------- entries[entry_count] -----------------------------------+
|   for each entry (12 bytes):                                               |
|     u8  is_optional                                                        |
|     u16 lang_type                                                          |
|     u8  reserved                                                           |
|     u32 name_offset            (offset into the names block below)         |
|     u32 pamt_checksum          (hashlittle of the PAMT post-header)        |
+------- + N*12 ------ names block --------------------------------------- --+
|     u32 names_block_length                                                 |
|     <ASCII names, NUL-terminated>                                          |
+----------------------------------------------------------------------------+
```

Round-trip via `PackGroupTreeMeta::parse(&bytes) → to_bytes()`.

Front-insert during overlay merge — see `add_papgt_entry` in
`src/binary/papgt.rs` for the upsert behavior.

### 1.1 Header CRC offset gotcha (do not get this wrong)

**The CRC field is at bytes 4–7, NOT 0–3.** Bytes 0–3 are the
platform magic, which must be preserved verbatim across writes.

```rust
// CORRECT
let crc = hashlittle(&papgt[12..], INTEGRITY_SEED);
papgt[4..8].copy_from_slice(&crc.to_le_bytes());

// WRONG — clobbers platform_magic, leaves real CRC stale
papgt[0..4].copy_from_slice(&crc.to_le_bytes());
```

DMM shipped this exact bug in pre-release.11 (a `strip_first_dmm_entry`
path that wrote to bytes 0–3); next mount failed parse with
`Checksum mismatch`. Fixed in pre-release.12. Auto-repair on stale CRC
landed in pre-release.13.

Note: PAMT uses the *opposite* layout — its header CRC is at bytes 0–3,
PazInfo CRC at bytes 16–19. Don't copy-paste between PAMT and PAPGT
writers without checking which format you're touching.

---

## 2. PAMT — pack metadata

Per-group file index. Lists every directory + file the PAZ blocks
contain, with chunk IDs, compressed/uncompressed sizes, encryption
metadata.

Layout:

```
header (size 0x0D):
  u32 checksum           # Jenkins hashlittle2 of post-header bytes
  u16 chunk_count
  u16 unknown0
  u8  encrypt_info_flag
  u8[3] encrypt_info     # XOR'd into ChaCha20 key for entry decryption

then:
  Chunk[chunk_count]     # (id, paz_offset, paz_size_compressed, ...)
  Directory[]            # path + File[] inside each
  File:
    BString name
    u32 chunk_id
    u32 offset_in_chunk
    u32 compressed_size
    u32 uncompressed_size
    u8  compression       # 0=none, 2=lz4, 3=zlib
    u8  crypto_flag
    Trie buffer (radix-compressed name index)
```

ChaCha20 key derivation lives in `src/crypto/chacha20.rs`
(`decrypt_pack_entry`) — uses Jenkins checksum of the filename as the
nonce seed.

---

## 3. PAZ — pack-zone blocks

Just a flat file of concatenated chunks. Each chunk's start/length
comes from the PAMT chunk table; entries within a chunk are
compressed (`compression` field) and optionally encrypted
(`crypto_flag`).

`paz::extract_file(group_dir, file, dir_path, encrypt_info)` does the
full extract pipeline (read chunk → slice entry → decrypt → decompress).

---

## 4. Trie buffer

Radix-compressed prefix trie used inside PAMT for fast name lookups.
Read-only at runtime; rebuilt on PAMT write. Implementation in
`src/binary/trie.rs`.

---

## 5. PAOC / PALOC — localization

Append-only flat file of localized strings, terminated by a u32 count.

```
[entries...]
[u32 trailing_count]    # equals len(entries) — sanity check
```

Each entry:

```
u64 category         # typically 0 for UI text
BString key          # u32 len + bytes + null
BString value        # same
```

`LocalizationFile::parse(bytes) → entries: Vec<{ unk_id, string_key, string_value }>`.

JSON surface (Python): `parse_paloc_from_file`, `parse_paloc_from_bytes`,
`serialize_paloc_to_bytes`.

---

## 6. PABGB / PABGH — tabular game data

A pair of files that together describe a typed table (PABGB = packed
binary "Body", PABGH = headers/schema). Each table type
(`ItemInfo`, `SkillInfo`, `BuffInfo`, `CharacterInfo`, `StageInfo`,
`GimmickInfo`, …) has its own typed parser under `src/tables/<name>/`
or the legacy `src/item_info/`.

Round-trip discipline: serialize must produce byte-identical output to
the input (`test_full_roundtrip` enforces this for ItemInfo, and the
`round_trip_matrix` example covers every Tier-1 table). Tables are
added by writing the typed Rust struct + `BinaryRead` / `BinaryWrite`
impls and registering in `src/dispatch.rs`.

### 6.1 Tier-1 PABGB inventory (post-1.3.3 promotions)

Tables promoted from Tier 1.5 → Tier 1 since 1.3.3, in addition to
the legacy ItemInfo / SkillInfo / BuffInfo / CharacterInfo /
StageInfo / GimmickInfo set:

| Table | Promoted in | Notes |
|---|---|---|
| ConditionInfo | task #114 | Tier 2 → Tier 1; full body byte-clonable; per-variant body JSON for **405 ConditionData variants** (#115, #117) |
| BuffInfo (per-variant) | #116 | Per-variant body JSON for **120 BuffData variants** |
| BranchConditionData (within ConditionData) | #118 | Per-variant body JSON for **14 variants** |
| board_info | #99 | |
| store_info | #120 | Polymorphic `_dropInfoData` body byte-clonable |
| ElementalMaterialInfo | #121 | Polymorphic `_elementalMaterialStateDataList` byte-clonable |
| SpecialModeInfo | #122 | |
| FactionInfo | #119 / #123 | |
| FactionNodeInfo | #124 | |
| CharacterInfo | #125 | |
| GimmickGroupInfo | #126 | |
| GimmickInfo | #127 | Tier-1.5 prefix extended |
| InteractionInfo | #128 | |
| QuestInfo | #106 | 35/35 fields editable; polymorphic body byte-clonable |

15 hand-rolled tables also exposed JSON-editable surfaces in #108/#109
even where they remain Tier 1.5 internally.

### 6.2 1.05.01 ItemInfo schema corrections

Schema work on the user's 1.05.01 iteminfo (5,338,778 bytes,
6,236 items) surfaced four real wire-layout corrections that bit
pre-1.3.4 schemas. All four are now in `src/item_info/`:

1. **`Cooltime` is 24 bytes (3 × i64), not 8.**
   Confirmed via IDA decomp of `sub_101886C44`: three
   `sub_1006B90BC(a1, a2 + N)` calls at memory offsets 0/8/16, each
   reading 8 bytes. Modeled as `Cooltime { a, b, c: i64 }`. JSON layer
   accepts BOTH legacy single-number form (`30`) AND the new object
   form (`{a:30, b:0, c:0}`) so SuperMod-era intents keep working.

2. **`MaxChargedUseableCount` is 12 bytes (3 × u32), not 4.**
   Same wrapper pattern via `sub_101886C94`. Modeled as
   `MaxChargedUseableCount { a, b, c: u32 }`. Same dual-form JSON
   acceptance. **6,236 of 6,236** vanilla items have non-zero `b`/`c`,
   so dropping them on parse breaks round-trip.

3. **`ItemIconData` has 5 fields, not 3** — wire order:
   `icon_path` (u32), `highlight_icon_path` (u32),
   `check_exist_sealed_data` (u8), `gimmick_state_list` (CArray<u32>),
   `check_usable` (u8). Wire field order matches the IDA decomp call
   order in `sub_101884D3C`, NOT the in-memory C++ struct layout.

4. **Restored fields** previously removed in a 1.04-target schema
   revert, all confirmed via IDA decomp of `sub_101885C38`:
   - `extract_additional_drop_set_info: u32`
   - `minimum_extract_enchant_level: u16`
   - `is_housing_only: u8`
   - `usable_alert_type: u8` (renamed from `usable_alert`, moved
     between `item_charge_type` and `sharpness_data`)
   - `discard_attach_terrain: u8`
   - `stage_info: u32`
   - `pattern_description_data_list: CArray<PatternDescriptionData>`
   - `is_has_item_use_data_inventory_buff: u8`
   - `is_preserved_on_extract: u8`

### 6.3 Wrapper-reader pattern

Two ItemInfo fields decoded via wrapper functions that read 3 sub-fields
each (Cooltime / MaxChargedUseableCount above). When auditing other
tables, watch for IDA functions of the shape:

```c
__int64 sub_xxx(__int64 a1, __int64 a2) {
    if ((sub_PRIM(a1, a2) & 1) && (sub_PRIM(a1, a2 + N) & 1))
        return sub_PRIM(a1, a2 + 2*N);
    return 0LL;
}
```

That pattern means **3 chained reads at offsets 0/N/2N**, not "1 field, 1 read".
Likely candidates for the same correction exist across other tables.

### 6.4 Determining wire vs memory width

The `pa::StaticInfoWrapper<Key, Info, Manager, unsigned short>` template
parameter is the **memory** type (the in-game C++ struct holds u16
after hash resolution). The **wire** type is whatever the inner vtable
reader's third arg says:

```c
// 4-byte read on the wire (e.g. ItemKey, EquipTypeKey, MultiChangeKey)
(*(...))(a1, &v4, 4LL);

// 2-byte read on the wire (e.g. CraftToolKey, InventoryKey, CategoryKey)
(*(...))(a1, &v4, 2LL);
```

Don't confuse the template arg (memory layout) with the wire reader's
size constant. They're often different.

---

## 7. DDS — DirectDraw Surface (Crimson flavor)

Standard DDS magic + 124-byte header, but Crimson stashes per-mip
sizes in `dwReserved1[0..4]` and the format ID in `dwReserved2[3]`.

| Format ID | Format        |
|-----------|---------------|
| 12        | DXT1 / BC1    |
| 15        | DXT3/5 / BC7  |
| 4         | BC4 / BC5 / BC6H |

Three-tier vpath classification (PATHC table → path-prefix table →
format-derived last4) lives in `src/dds/vpath.rs`. Validator in
`src/dds/validate.rs` returns the same `{code, severity, message}`
shape as the audio validator.

Python: `classify_dds`, `validate_dds`, `infer_dds_vpath`,
`classify_vpath_last4`.

---

## 8. WEM — Wwise audio clip

RIFF-WAVE wrapper with Wwise-specific chunks (`hash`, `junk`).
Format-tag values:

- `0xFFFE` — `WAVEFORMATEXTENSIBLE` (PCM/uncompressed)
- `0xFFFF` — `WwiseVorbis` (compressed)

Header-only metadata extraction (channels, sample_rate, byte_rate,
block_align, bits_per_sample, has_wwise_hash_chunk, data_offset,
data_size) via `src/audio/wem.rs::classify_wem`. We do **not** decode
the audio payload.

---

## 9. BNK — Wwise soundbank

Sectioned format (no RIFF wrapper). Crimson uses `bank_version=150`.

Sections we recognize:

| Tag    | Meaning                                                  |
|--------|----------------------------------------------------------|
| `BKHD` | Bank header (version, bank_id, mandatory)               |
| `DIDX` | Embedded WEM index (id, offset_in_DATA, size)           |
| `DATA` | Concatenated WEM payloads                                |
| `HIRC` | Hierarchy (events, sounds, actions) — header only       |
| `STID` | String ID table — header only                            |

`src/audio/bnk.rs::parse_bnk` returns the section table + the embedded
WEM index. Validator (`src/audio/validate.rs`) rejects truncated banks
and warns on unknown versions / DIDX/DATA disagreements.

---

## 10. SAVE — save file envelope

Encrypted + compressed save container. 0x80 plaintext header followed
by a ChaCha20-encrypted, LZ4-compressed body.

```
+-- 0x00 ----------------------------------------------------+
| 4   "SAVE"                                                  |
| 2   version  (currently 2)                                  |
| 2   flags    (observed 0x0080)                              |
| 10  reserved_a (preserved on rewrite)                       |
| 4   uncompressed_size  (post-LZ4 plaintext size)           |
| 4   payload_size       (ciphertext size after 0x80)        |
| 16  nonce              (4-byte counter LE + 12-byte nonce)  |
| 32  hmac               (HMAC-SHA256 of compressed plaintext)|
| 54  reserved_b (zeroed)                                     |
+-- 0x80 ----------------------------------------------------+
| chacha20( lz4( body ) )                                     |
+-------------------------------------------------------------+
```

Pipeline (read):

```
ciphertext = file[0x80 : 0x80 + payload_size]
compressed = ChaCha20.decrypt(ciphertext, key, nonce16)
hmac_ok    = HMAC-SHA256(compressed, key) == header.hmac
body       = LZ4.block.decompress(compressed, uncompressed_size)
```

The dmm-parser `SaveEnvelope` module accepts caller-supplied keys and
HMAC closures so the public crate doesn't embed save secrets. See
`references/save_notes.md` §2 for the key-derivation formula and
`src/save/envelope.rs` for the implementation.

---

## 11. Sequencer & attack asset family

Six standalone asset formats live next to (but outside) the PABGB tabular
data: `.paatt`, `.paseq`, `.paseqc`, `.pastage`, `.paschedule`,
`.paschedulepath`. They drive scripted gameplay — attack hitboxes, cutscene
playback, NPC schedules, stage-chart logic. All six round-trip byte-exact
on every vanilla sample. **PASEQ and PASTAGE shipped Tier-1 field-level
decode** (tasks #129, #130). Four remain on Tier 1.5 — see
[§12](#12-decode-tiers-and-the-tier-1-promotion-goal) for the active
promotion goal.

| Format | Loader (Mac binary) | Wire shape | Status |
|---|---|---|---|
| `.paseq` | Reflection-driven via `pa::Sequencer*` dispatchers | Type-name-hash dispatch | **Tier 1** — field-level round-trip, 4,659 vanilla samples (#129). `examples/round_trip_paseq.rs`, `examples/paseq_roundtrip.rs` |
| `.pastage` | Reuses `sub_141D8C6D0` (`SequencerStageChartDesc`) plus a stage-path LP-string prefix | Path prefix + chart body | **Tier 1** — field-level round-trip, 3,320 vanilla samples (#130). `examples/round_trip_pastage.rs`, `examples/pastage_roundtrip.rs` |
| `.paatt` | `pa::sub_100C38E88` (loader) + `pa::sub_100C39A10` (per-info) | u32 info_count + per-info[u8 version + N-byte BaseData (264/528/296/288/264 by version) + 9× count-prefixed frame slots] + 7× LP-prefixed string table + frame-event buffer | **Tier 1.5** — envelope decoded; `base_data` payload still raw. Goal: decode `pa::AttackInfoDataDesc` reflect-property setters → typed BaseData per version; sub-variants `AttackInfo_Attack` / `_AttackThrow` / `_AttackCatch` / `_ReleaseCatch` |
| `.paseqc` | Likely shares paseq dispatcher | Header magic `FF FF 04 00` (or `FF FF 03 00` minority) + sequencer chart body | **Tier 1.5** — `lp_token_stream` tokenizer; magic lands in leading `RawBytes`. Goal: verify dispatcher reuse from `.paseq`, then promote |
| `.paschedule` | TBD (search `pa::Schedule*` / `pa::NPCSchedule` in IDA) | Header `01 00 00 00` (majority) or `00 00 00 00`; mostly numeric (waypoint hashes, frame counts) + a few asset path strings | **Tier 1.5** — `lp_token_stream`. Goal: reflection-driven decode |
| `.paschedulepath` | TBD — companion to paschedule | No fixed magic; per-NPC hash header; almost entirely numeric | **Tier 1.5** — `lp_token_stream`. Goal: reflection-driven decode |

**Why this works** — the engine uses `pa::ReflectObject`-style reflection
to read these files. Every field is a registered reader function in the
loaded binary; we already have the playbook (and IDA MCP access) to walk
the dispatchers and translate them into typed Rust enums. Seven family
decoders have shipped this way (GameCondition, FilterCondition,
TriggerGamePlayEventHandlerData, GameEventHandlerData,
SequencerStageChartDesc, plus the freshly-Tier-1 PASEQ and PASTAGE).

---

## 12. Decode tiers and the Tier 1 promotion goal

Decode coverage is tracked as three tiers, the same vocabulary
`docs/STATUS.md` and `docs/449_TABLE_CATALOG.md` use:

| Tier | Meaning | Mod-author capability |
|---|---|---|
| **1** | Every wire field is named, typed, and individually addressable through JSON. Round-trip is byte-perfect. | Edit any field by path (`item_name.default`, `enchant_data_list[0].value`, etc.) |
| **1.5** | Round-trip byte-exact, but the body is exposed as raw bytes / opaque tokens (e.g. `Vec<u8>`, `LpToken::RawBytes`). | Edit only the parts that *are* typed (e.g. embedded strings); numeric fields stay opaque. |
| **2** | Whole-tail blob — entire payload is one `Vec<u8>`. | Clone or replace only; no field-level edit. (No Tier 2 tables remain in the catalog.) |

### Active goal — finish the sequencer + attack family

Two of the six (`.paseq`, `.pastage`) are now Tier 1. The remaining
four are still Tier 1.5 and are the *only* Tier 1.5 surface left in
dmm-parser. Promoting them is a prerequisite for letting mod authors
edit:

- **Compiled cutscene chart** (`.paseqc`)
- **NPC time-of-day routines** (`.paschedule` + `.paschedulepath`)
- **Per-weapon attack data** — hitboxes, damage, frame events (`.paatt`)

at the field level via the same Field-JSON v3.1 intent vocabulary
already used for PABGB tables.

**Attack order** (smallest scope first, leveraging now-shipped PASEQ
infrastructure):

1. `.paseqc` — expected to share `.paseq`'s dispatcher; verify reuse,
   prepend the `FF FF 04 00` magic, ship.
2. `.paschedule` + `.paschedulepath` — paired NPC schedule decode.
3. `.paatt` — finish per-version BaseData via `pa::AttackInfoDataDesc`
   reflect-property setters.

**Methodology** — the proven family-decoder playbook from
`docs/STATUS.md` §"The reusable playbook":

1. Find the loader / dispatcher in IDA (Mac binary preferred; vtables intact).
2. Extract the tag → reader-function map (template at
   `dmm-pabgb-aio/extract_conditiondata_dispatch.py`).
3. Stand up a recursive enum in `src/binary/variants/<format>.rs`.
4. Build a roundtrip validator in `examples/<format>_roundtrip.rs` with a
   `LAST_ATTEMPTED_TAG` thread-local to pinpoint failing tags.
5. Loop: validator → IDA decompile of the failing tag's reader → fix
   recipe → repeat.
6. Wrap the wrapper enum in a `Decoded | Raw` fallback so anti-disasm tags
   preserve byte-perfect roundtrip even when un-decoded.

**Definition of done per format:**

- 100% byte-perfect roundtrip on all vanilla samples (no regression from
  the current `lp_token_stream`-based baseline).
- ≥99% Decoded share; remaining stays in `Raw(Vec<u8>)` arm.
- `to_json_dict` / `write_from_json_dict` exposes every typed field.
- New entry in `dispatch.rs` (parse + serialize + `supported_tables()`).
- PyO3 binding in `src/python.rs`.
- `docs/api.md` and this file (§11) updated.
- `cargo test --release` clean; new `examples/<format>_roundtrip.rs` validator passes.

---

## 13. Hexpat conventions

Every binary format above has a matching `references/<format>.hexpat`
pattern. To explore in ImHex:

```
plcli run -i <input.bin> -p references/<format>.hexpat -v -d
```

When iterating a new format: write a partial pattern, run plcli on a
known-good sample, refine until the structure is fully covered. Once
the pattern stabilizes, port it into a Rust struct with `BinaryRead` /
`BinaryWrite` impls and add a roundtrip test.

---

## 14. Adding a new format

The shortest path:

1. Drop a sample under `references/samples/` (gitignored fixtures dir).
2. Write `references/<name>.hexpat` and iterate via `plcli` until you
   understand every byte.
3. Add `references/<name>_notes.md` documenting any quirks you found
   during recon (Crimson's "last4" reserved fields, Wwise's `hash`
   chunk, etc.).
4. Add `src/<name>/` with `mod.rs` + a typed reader + a writer that
   produces byte-identical output.
5. Add a `#[cfg(test)] mod tests` block with at least: header parse,
   full parse, full round-trip on a real sample, error-path coverage.
6. Wire into `src/dispatch.rs` for the JSON surface, then add the
   PyO3 binding in `src/python.rs`.
7. Update `docs/api.md` with the Python entry points.
8. Update this file with the new row in the at-a-glance table.

The pattern works: PALOC, DDS, WEM, BNK, and SAVE all landed via this
path.

---

## PAATT BaseData Field Layout

<!-- SPDX-License-Identifier: LicenseRef-CDMTL-1.0
     Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
     Licensed under CDMTL v1.0 - see LICENSE.txt -->

# `.paatt` BaseData — Field Directory

Reverse-engineered from `pa::AttackInfoDataDesc` reflection symbols
in the Mac binary (`CrimsonDesert_Steam`). Every field name and
declared type below is sourced from the C++ name-mangled symbols at
`0x1076df1a0` onwards (setters) and `0x1076e3338` onwards (getters).

## Per-version BaseData sizes (empirical, 220 vanilla `.paatt`,
13,789 AttackInfo records)

| version | BaseData size | infos seen | likely sub-variant |
|---|---|---|---|
| 0 | 264 bytes | 10,562 | `AttackInfo` (base) |
| 1 | 528 bytes | 1,674 | `AttackInfo_AttackCatch` (base + 264-byte `AttackCatchDesc`) |
| 2 | 296 bytes | 851 | `AttackInfo_AttackThrow` (base + 4 fields, 32-byte aligned) |
| 3 | 288 bytes | 702 | `AttackInfo_ReleaseCatch` (base + 24 bytes) |
| 4 | 264 bytes | 0 (unused in vanilla) | reserved (matches base size) |

## `pa::AttackInfoDataDesc` — full field list (25 fields)

Sourced by parsing every `_ZN2pa18AttackInfoDataDesc<N>set_<name>ERK<type>`
symbol. The mangled type code reveals the field type:

| Field | Type | Mangled |
|---|---|---|
| `attackDir` | u8 | `RKh` |
| `weaponKey` | u32 | `RKj` |
| `targetType` | TargetType (enum) | `RKNS_10TargetTypeE` |
| `attackIndex` | u8 | `RKh` |
| `repeatCount` | u8 | `RKh` |
| `attackHitData` | `AttackHitDataDesc*` (nested object pointer) | `PKNS_17AttackHitDataDescE` |
| `attackerDelay` | `ActionChartFrameEvent_AttackDelayDataDesc` (value) | `RKNS_41ActionChartFrameEvent_AttackDelayDataDescE` |
| `ignoreSafeZone` | bool | `RKb` |
| `attackCommonData` | `AttackCommonDataDesc*` (nested object pointer) | `PKNS_20AttackCommonDataDescE` |
| `attackDivideType` | enum | `RKNS_38ActionChartFrameEvent_AttackDivideTypeE` |
| `attackGroupIndex` | u8 | `RKh` |
| `noCheckCollision` | bool | `RKb` |
| `hitEffectInfoType` | u32 | `RKj` |
| `attackHitCheckType` | enum | `RKNS_40ActionChartFrameEvent_AttackHitCheckTypeE` |
| `attackImpulseLevel` | u8 | `RKh` |
| `physicImpulsePower` | f32 | `RKf` |
| `physicsImpulseMass` | f32 | `RKf` |
| `repeatDegreeWeight` | f32 | `RKf` |
| `ignoreWhenHitAction` | bool | `RKb` |
| `isSingleHitPosition` | bool | `RKb` |
| `excludeTargetTypeFlag` | u32 | `RKj` |
| `ignoreDefenceTypeFlag` | u32 | `RKj` |
| `physicsImpulseVelocity` | f32 | `RKf` |
| `singleHitPositionOffset` | float3 (12 bytes) | `RKNS_6float3E` |
| `singleHitPositionSocket` | u16 | `RKt` |

## C++ name-mangling type codes

| Code | Meaning |
|---|---|
| `h` | unsigned char (u8) |
| `j` | unsigned int (u32) |
| `i` | int (i32) |
| `t` | unsigned short (u16) |
| `b` | bool |
| `f` | float |
| `Pf` | float pointer |
| `NS_<N><Name>E` | `pa::Name` (nested type) |
| `RK<T>` | `const T&` (read-only ref) |
| `O<T>` | `T&&` (move ref) |

## Class hierarchy summary (all 4 desc structs)

```
AttackInfoDataDesc                       — root (264 bytes for v0)
├── attackHitData → AttackHitDataDesc    — 7 fields (nested via pointer)
├── attackCommonData → AttackCommonDataDesc — 12 fields (nested via pointer)
├── attackerDelay (value) → ActionChartFrameEvent_AttackDelayDataDesc
└── 22 leaf fields (above)

AttackInfo_Attack            — base wrapper (no extra fields)
AttackInfo_AttackThrow       — base + 4 fields:
   ├── projectileKey (u32, ProjectileKeyFromString custom)
   ├── actionHashCode (u32, ActionNameHashCodeFromString)
   ├── aiEventKey (AiEventKey enum)
   └── frameTime (f32, FrameTimeFromString)
AttackInfo_AttackCatch       — base + AttackCatchDesc (~264 bytes)
AttackInfo_ReleaseCatch      — base + (TBD; ~24 bytes extra)
```

## `AttackCommonDataDesc` fields (17 fields, Mac-IDA confirmed)

Recovered Session 19 by decompiling every `__ZNK2pa20AttackCommonDataDesc...get_<field>Ev` getter
in the Mac binary (each is a single `return this+offset` instruction).
Earlier "12 fields" estimate from setter/getter symbol counting missed the
three bit-packed bools and the equipSlot/attackNameCount group. **In-memory
offsets, NOT wire offsets** (Pearl Abyss serializes via metaobject iteration).

| In-mem offset | Field | C++ type | Wire-position candidate (BaseDataV0) |
|---|---|---|---|
| 0x00 (0) | `attackOffset` | float3 (12 B) | wire 0x08 — `attack_pos_offset` ✅ |
| 0x0C (12) | `attackBoxSize` | float3 (12 B) | wire 0x14 — currently `_unk_float3_0014` |
| 0x18 (24) | `attackAngle` | f32 | wire 0x20 — `attack_degree` ✅ |
| 0x1C (28) | `attackYaw` | f32 | wire 0x24 — `attack_yaw` ✅ |
| 0x20 (32) | `innerAttackLength` | f32 | wire 0x28 — currently `_unk_f32_0028` |
| 0x24 (36) | `impulseLengthScale` | f32 | wire 0x2C — currently `physic_impulse_power` (PR #14 named — verify rename?) |
| 0x28 (40) | `impulseAngleScale` | f32 | wire 0x30 — currently `physics_impulse_mass` (verify rename?) |
| 0x2C (44) | `hitType` | enum (u8) | wire 0x34 — currently `attack_hit_check_type` (verify u16 vs u8) |
| 0x2D (45) | `attackPositionType` | u8 | unmapped |
| 0x2E (46) | `attackPositionBone` | u16 (2 B) | unmapped (string-table key?) |
| 0x30 (48) | `detectEventDistance` | f32 | unmapped |
| 0x34 (52) | `equipSlotNameKey` | enum (u8) | wire 0x00a8 — `equip_slot_name_key` ✅ |
| 0x38 (56) | `equipSlotIndex` | u8 | unmapped |
| 0x3C (60) | `attackNameCount` | u8 | unmapped |
| 0x3D (61) bit0 | `ignoreDecreaseEndurance` | bool | unmapped (rare-true bool) |
| 0x3D (61) bit1 | `checkBackGroundHit` | bool | unmapped (rare-true bool) |
| 0x3D (61) bit2 | `isUseReserveSlot` | bool | unmapped (rare-true bool) |

⚠️ The `physic_impulse_power` / `physics_impulse_mass` rename candidacy
needs double-checking — the wire-offset adjacency strongly suggests
`impulseLengthScale` / `impulseAngleScale`, but the contributor named
them from empirical defaults of 1.0 (which would also match length/angle
scale defaults). Either name is functionally consistent. Hold off on
renaming until the .paatt serializer iteration order is mapped.

## `AttackHitDataDesc` fields (8 fields, Mac-IDA confirmed)

| In-mem offset | Field | C++ type | Wire-position candidate |
|---|---|---|---|
| 0x00 (0) | `attackeeDelay` | nested struct (12 B) | wire 0x58–0x6f — currently `_ds1_*` (5 floats) |
| 0x0C (12) | `hitRotationAngle` | f32 | wire 0x90 — currently `hit_degree` (rename candidate; "Degree" was a guess, the C++ name is `hitRotationAngle`) |
| 0x10 (16) | `pushSpeed` | f32 | wire 0xA0 — currently `_unk_f32_00a0` (strong candidate) |
| 0x14 (20) | `maxPushAngleRange` | f32 | wire 0xD4 / 0xDC / 0xE0 / 0xE4 — currently `_unk_f32_*` (one of these four) |
| 0x18 (24) | `ragdollPresetName` | u16 (2 B) | unmapped (string-table key) |
| 0x1A (26) | `hitRotationType` | enum (u8) | wire 0x9C — currently `hit_rotation_type` ✅ |
| 0x1B (27) | `hitPower` | u8 | wire 0xF4 — currently `_unk00f4` (candidate) |
| 0x1C (28) | `pushWithBoneVelocity` | bool | wire 0xFA / 0xFC — currently `_unk00fa` or `_unk00fc` |

**High-confidence wire-mapping deltas** to apply *if* the serializer order is
later proven:
- `_unk_float3_0014` → `attack_box_size`
- `_unk_f32_0028` → `inner_attack_length`
- `_unk_f32_00a0` → `push_speed`
- `_unk00f4` → `hit_power`
- `hit_degree` → `hit_rotation_angle` (rename of an already-named field)

## IDA reference addresses

| Item | Address |
|---|---|
| `_ZTSN2pa18AttackInfoDataDescE` (RTTI) | `0x1072ed3c0` |
| `AttackInfoDataDesc` setter `_ptr` table | `0x1076df1a0` to `0x1076df3f0` |
| `AttackInfoDataDesc` getter `_ptr` table | `0x1076e3338` to `0x1076e34xx` |
| `AttackCommonDataDesc` (RTTI) | `0x1072ed399` |
| `AttackHitDataDesc` (RTTI) | `0x1072ed3ae` |
| `_ZTVN2pa17MetaObjectBindPODINS_18AttackInfoDataDescEEE` (vtable) | `0x10778a5d0` |
| `_ZTIN2pa17MetaObjectBindPODINS_18AttackInfoDataDescEEE` (typeinfo) | `0x10778a618` |

## Per-byte offset extraction — TODO (Session 12 update)

**Setbacks identified Session 12**:

The `_ptr` globals (`__ZN2pa18AttackInfoDataDesc13set_attackDirERKh_ptr`
etc.) point to runtime-filled function-pointer slots, not to
standalone setter implementations. Reading the qword at
`0x107ED06F8` returns `0` because it's only populated when the
metaobject is constructed at startup.

The metaobject builder for AttackInfoDataDesc is split into 25+
`bindProperty_<fieldname>` functions, one per field. We located
`pa::AttackInfoDataDesc::bindProperty_attackDir` at
`sub_100C41D70` and disassembled it:

- Loads the setter/getter pointers from the `_ptr` globals into a
  static-initialized `SimpleReflectPropertyBindPOD<AttackInfoDataDesc, h, ...>`
  descriptor (vtable at
  `_ZTVN2pa28SimpleReflectPropertyBindPODINS_18AttackInfoDataDescEhRKhS3_...`).
- Stores the type-id `5` (= `u8`) at descriptor offset 0x98.
- Tail-calls `sub_1005F3B64(metaobject, descriptor, group_type,
  ReflectGroupType)` which registers the property.

Crucially, **the byte offset is not stored anywhere in the
descriptor** — it's encoded inside the setter lambda. Since the
lambda is inlined into the metaobject's runtime dispatch and not
exposed as a standalone function, IDA cannot recover the offset
from static analysis alone.

## Recommended decode strategy for next iteration

**Pragmatic approach: differential analysis on vanilla samples.**

We have **10,562 v0 BaseData blobs** (264 bytes each ≈ 2.79 MiB of
data) and **1,674 v1 blobs** (528 bytes each). For each byte
position across the dataset:

1. Compute the value distribution.
2. Bytes with binary distribution (mostly 0/1) → likely `bool`.
3. Bytes with low-cardinality distribution (~10 distinct values) →
   likely `enum` or low-range `u8`.
4. 4-byte groups whose byte 0/1 vary together but byte 2/3 stay 0
   → likely `u16`.
5. 4-byte groups with full 32-bit entropy in IEEE-float range
   (most exponents around 0x3F-0x42) → likely `float`.
6. Boundaries between consecutive zero-runs likely indicate struct
   alignment / nested object boundaries (the
   `attackHitData`/`attackCommonData`/`attackerDelay` nested fields).

Combined with the 25-field directory above (sum of fixed sizes:
5 u8 + 4 u32 + 1 u16 + 5 bool + 4 float + 1 float3 + 3 enums = 5
+ 16 + 2 + 5 + 16 + 12 + 3×4 = 68 bytes of leaf fields), the
remaining 264 - 68 = 196 bytes belong to the 3 nested struct
fields (AttackCommonDataDesc + AttackHitDataDesc + AttackDelayDataDesc).

This gives a constrained search space we can brute-force-validate.

## Round-trip status (current — Session 18, PR #14)

`.paatt` is round-trip byte-perfect via `PaattFile::to_bytes()`.

**All four typed BaseData variants are FIELD-DECODED** in
`src/binary/paatt_basedata.rs`:

| Variant | Size | Coverage |
|---|---|---|
| `BaseDataV0` (base AttackInfo) | 264 B | 60+ named fields + `_unkXXXX` placeholders for unresolved positions |
| `BaseDataV1` (= V0 + AttackCatchDesc) | 528 B | V0 fields + 9 named catch fields (`catch_yaw_hi_rad`, `catch_dist_a`, `catch_elevation_rad_a`, …) inside a `catch_desc` sub-dict |
| `BaseDataV2` (= V0 + ThrowDataDesc) | 296 B | V0 fields + `projectile_key`, `action_hash_code`, `frame_time`, `ai_event_key` |
| `BaseDataV3` (= V0 + ReleaseCatchDataDesc) | 288 B | V0 fields + `release_angle_rad`, `frame_time`, plus `_unk0110` / `_unk0114` (release-catch type hashes) |

Mod authors call `paatt_decode_base_data(version, data)` from Python to
get a named-field dict, edit fields like `weapon_key`,
`physic_impulse_power`, etc., then call
`paatt_encode_base_data(version, fields)` to get bytes back. Every
vanilla `.paatt` (220 files, 13,789 AttackInfo records) round-trips
byte-perfect through this path. See `docs/api.md` →
"**.paatt — typed AttackInfo BaseData**" for the Python entry-point
reference and the most-commonly-edited field cheatsheet.

**Session 18 addendum (PR #14):**
- V2 throw payload: 4 named fields recovered via field analysis across
  851 V2 records (`projectile_key`/`action_hash_code`/`frame_time`/`ai_event_key`).
- V3 release-catch payload: 2 named fields recovered across 702 V3 records
  (`release_angle_rad`/`frame_time`); release-catch-type hashes left as
  `_unk0110` / `_unk0114` pending an IDA reflection-symbol pass.
- AttackCatchDesc (the V1 264-byte tail): 9 catch-geometry fields named
  (yaw range, throw distance, elevation cone) from differential entropy
  on 1,674 V1 records; the trailing 176 bytes remain as `_cd_tail` blob.

### Confirmed V0 field offsets (264 bytes) — Session 17 state

| Offset | Field | Type | Notes |
|--------|-------|------|-------|
| 0x0000 | `weapon_key` | u32 | Weapon/action hash; unique per record |
| 0x0004 | `attack_dir` | u8 | 0=base, 1=catch, 3=release-catch |
| 0x0005 | `_pad0005` | [u8;3] | Alignment |
| 0x0008 | `attack_pos_offset` | [f32;3] | `AttackCommonDataDesc.AttackPosOffset` |
| 0x0014 | `_unk_float3_0014` | [f32;3] | `AttackCommonDataDesc` unnamed float3 |
| 0x0020 | `attack_degree` | f32 | `AttackCommonDataDesc.AttackDegree`; default ≈ 6.2832 (2π rad) |
| 0x0024 | `attack_yaw` | f32 | `AttackCommonDataDesc.AttackYaw`; default 0.0 |
| 0x0028 | `_unk_f32_0028` | f32 | `AttackCommonDataDesc` unnamed float |
| 0x002c | `physic_impulse_power` | f32 | Vanilla default 1.0 |
| 0x0030 | `physics_impulse_mass` | f32 | Vanilla default 1.0 |
| 0x0034 | `attack_hit_check_type` | u16 | Enum; common value 4 |
| 0x0036 | `hit_check_normal_str_idx` | u16 | `NormalStringIndex`; 0xffff=none; 0xffff@98% V0, always for V2/V3 |
| 0x0038 | `repeat_degree_weight` | f32 | Vanilla default -1.0 |
| 0x003c | `physics_impulse_velocity` | f32 | Vanilla default 0.0 |
| 0x0040 | `ignore_safe_zone` | bool | + 3-byte pad |
| 0x0044 | `attack_group_index` | u8 | Common value 1 |
| 0x0045 | `repeat_count` | u8 | Common value 2; + 2-byte pad |
| 0x0048 | `hit_effect_info_type` | u32 | Effect hash; 0xf177b780 most common |
| 0x004c | `single_hit_pos_offset` | [f32;3] | `singleHitPositionOffset` XYZ; default (0,0,0); V2/V3 always zero |
| 0x0058 | `_ds1_f0..f4` | f32×5 | `ActionChartFrameEvent_AttackDelayDataDesc` #1; f0=trigger time (s; 0.0/0.05/0.10), f1=blend/end (0.0/1.0), f2=secondary offset, f3=angle_rad (99.9% zero), f4=frame count (int; 0/6/9/11) |
| 0x006c | `_pad_ds1` | [u8;4] | Always zero |
| 0x0070 | `normal_string_index` | u16 | `AttackCommonDataDesc.NormalStringIndex`; V0: 0x0000 @70%; V2/V3: 0xffff (always) |
| 0x0072 | `_unk0072` | bool | bool (B00@85% V0) |
| 0x0073 | `_unk0073` | u8 | u8 enum (mode=1 @49% V0) |
| 0x0074 | `_pad0074` | [u8;4] | Always zero |
| 0x0078 | `_ds2_f0..f4` | f32×5 | `ActionChartFrameEvent_AttackDelayDataDesc` #2; f0-f3 identical to ds1 (98%+); f4 almost always 0.0 (non-zero: 1.1-1.53 scale) |
| 0x008c | `_pad_ds2` | [u8;4] | Always zero |
| 0x0090 | `_unk_f32_0090` | f32 | `AttackHitDataDesc.Degree` candidate; V0 mode=50.0; V2/V3=0.0 |
| 0x0094 | `_pad0094` | [u8;8] | Always zero |
| 0x009c | `_unk009c` | u8 | Attack-type enum; V0: mode=0; V2: 0x7a; V3: 0x5a |
| 0x009d | `_pad009d` | [u8;3] | Always zero |
| 0x00a0 | `_unk_f32_00a0` | f32 | `AttackHitDataDesc` unnamed float; usually 0.0 |
| 0x00a4 | `_pad00a4` | [u8;4] | Always zero |
| 0x00a8 | `_unk00a8` | u8 | `EquipSlotNameKey` candidate; V0 mode=12; V2/V3=23 |
| 0x00a9 | `_pad00a9` | [u8;3] | Always zero |
| 0x00ac | `_unk00ac` | [u8;4] | byte[0] u8 (≈0); byte[1] bool; bytes[2,3]=0 |
| 0x00b0 | `_unk00b0` | u32 | Bitmask; 77.5% zero; 25 distinct; candidate: `excludeTargetTypeFlag` |
| 0x00b4 | `single_hit_position_socket` | u16 | Socket name-table index; 0xffff=none |
| 0x00b6 | `_pad00b6` | [u8;2] | Alignment |
| 0x00b8 | `_unk00b8` | [u8;16] | Attack-type byte region; byte patterns differ by V0/V2/V3 |
| 0x00c8 | `_unk00c8` | [u8;12] | Dense const/varying byte region (frame timing?) |
| 0x00d4 | `_unk_f32_00d4` | f32 | Integer-valued f32; 5 distinct values; ≈0 |
| 0x00d8 | `_pad00d8` | [u8;4] | Always zero |
| 0x00dc | `_unk_f32_00dc` | f32 | ≈0.0; V2/V3 always 0; V0 sometimes non-zero |
| 0x00e0 | `_unk_f32_00e0` | f32 | ≈0.0 |
| 0x00e4 | `_unk_f32_00e4` | f32 | ≈0.0 |
| 0x00e8 | `_unk00e8` | [u8;4] | byte[0]=CONST 1; byte[1]=bool; bytes[2,3]=0 |
| 0x00ec | `_pad00ec` | [u8;4] | Always zero |
| 0x00f0 | `hit_normal_string_index` | u16 | `AttackHitDataDesc.NormalStringIndex`; V0 often 1021 (0x03fd); V2/V3 often 0 |
| 0x00f2 | `_pad00f2` | [u8;2] | Always zero |
| 0x00f4 | `_unk00f4` | u8 | `AttackHitDataDesc` field 6 (unnamed u8); 1=no-rotation (pairs with hit_degree=0°); 5/2/6/4/3=rotation types |
| 0x00f5 | `_pad00f5` | [u8;3] | Always zero |
| 0x00f8 | `hit_data_str_idx` | u16 | `NormalStringIndex`; 0=none; 95%+ non-zero cases pair with attack_hit_check_type=4; V2/V3 always 0 |
| 0x00fa | `_unk00fa` | bool | bool (99% false); likely `AttackHitDataDesc` field 7 (unnamed bool) |
| 0x00fb | `_pad00fb` | u8 | Always zero |
| 0x00fc | `_unk00fc` | u8 | `AttackHitDataDesc` field 7 candidate; 0=false (when _unk00f4=1); 1=true (rotation types) |
| 0x00fd | `_pad00fd` | [u8;3] | Always zero |
| 0x0100 | `hit_data_str_idx_b` | u16 | Secondary `NormalStringIndex`; 0=none; always co-present with `hit_data_str_idx`; values in same range (0x0450–0x046d) |
| 0x0102 | `_pad0102` | [u8;2] | Always zero |
| 0x0104 | `_unk0104` | u8 | u8; 4 values (98% zero); no clear correlation |
| 0x0105 | `_pad0105` | [u8;3] | Always zero |

V1 = V0 (264 bytes) + `catch_desc` blob (264 bytes).

## What the V2/V3 cross-version analysis confirmed (Session 14)

Running per-version entropy across all 4 versions revealed:

| Offset | V0 mode | V2 (throw) | V3 (rel-catch) | Interpretation |
|--------|---------|------------|-----------------|----------------|
| 0x004c–0x0057 | ≈0 (97%) | always 0 | almost 0 | `singleHitPositionOffset` ← **decoded** |
| 0x0070–0x0071 | 0x0000 (70%) | 0xffff (always) | 0xffff (always) | `NormalStringIndex` (CommonData?) |
| 0x0090 | 50.0 (68%) | 0.0 (always) | 0.0 (always) | `AttackHitDataDesc.Degree` candidate |
| 0x009c | 0 (28%, 19 vals) | 0x7a (92%) | 0x5a (99%) | `AttackNameList` u8 candidate |
| 0x00a8 | 12 (66%, 16 vals) | 23 (always) | 23 (always) | `EquipSlotNameKey` u8 candidate |
| 0x00b8+3 | 5 (43%) | 5 (always) | 5 (89%) | unknown u8, consistent default |

## Remaining decoding work

1. `_unk0058`/`_unk0078` (0x0058–0x008f): two 24-byte delay sub-structs — confirm 5-float layout
   - 0x0058–0x006b: 5 floats (likely `attackerDelay` sub-struct — V2 shows 0.01/1.0/0.0/x/0)
   - 0x006c–0x006f: always-zero pad
   - 0x0078–0x008b: second delay sub-struct (same pattern)
   - 0x008c–0x008f: always-zero pad
2. Sub-fields within blob regions still needing a name:
   - `_unk0070`: confirm as `AttackCommonDataDesc.NormalStringIndex`
   - `_unk0072`: confirm bool/enum split and field names
   - `_unk009c`: confirm as `AttackNameList` or `HitRotationType`
   - `_unk00a8`: confirm as `EquipSlotNameKey`
   - `_unk00b0`: confirm as `excludeTargetTypeFlag` or `ignoreDefenceTypeFlag`
   - `_unk00b8`/`_unk00c8`: 28-byte attack-type region — decode sub-bytes
   - `_unk_f32_00d4`: identify field name (integer-valued float)
   - `_unk_f32_00dc`/`_unk_f32_00e0`/`_unk_f32_00e4`: three adjacent floats — likely one struct member group
   - `_unk00e8[1]`: confirm bool field name
   - `_unk00f0`: confirm as `AttackHitDataDesc.NormalStringIndex`
   - `_unk00f4`: confirm unnamed u8 field
   - `_unk00f8`: decode u16 + bool sub-fields
   - `_unk00fc`: identify u8 enum (8 values)
   - `hit_data_str_idx_b`: secondary NormalStringIndex (resolved)
   - `_unk0104`: identify u8 (4 values)
3. `_unk0036` (0x0036): confirm as `NormalStringIndex` from `AttackHitDataDesc` (0xffff=none)
4. AttackCatchDesc (V1 `catch_desc` blob, 264 bytes): decode field by field

## Appendix: In-memory class layout (Session 19, IDA-confirmed)

Decompiled the `pa::AttackInfoDataDesc::get_<field>` zero-argument
getters in the Mac binary. Each getter is 8 bytes and resolves to a
single ARM64 instruction returning `this + offset`, so the in-memory
class layout falls out for free. **These are class-instance offsets,
NOT wire offsets** — Pearl Abyss serializes field-by-field through
the metaobject's setter table rather than memcpy'ing the class, so
the wire `BaseDataV0` layout (264 B) reorders fields freely. The
in-memory map still confirms (a) every C++ field is real, (b) what
size each leaf is, and (c) which fields cluster together — useful
context when searching for a still-`_unkXXXX` wire field's identity.

| In-mem offset | Field | C++ type | Wire offset (BaseDataV0) | Status |
|---|---|---|---|---|
| 0x94 (148) | `ignoreDefenceTypeFlag` | u32 | **TBD** | unmapped (could be `_unk00b8`-region u32 or one of the late `_unk` slots) |
| 0xA0 (160) | `targetType` | enum (4 B) | **TBD** | unmapped |
| 0xA4 (164) | `excludeTargetTypeFlag` | u32 | 0x00b0 | ✅ already named in `BaseDataV0` |
| 0xA8 (168) | `weaponKey` | u32 | 0x0000 | ✅ already named |
| 0xB0 (176) | `attackImpulseLevel` | u8 | **TBD** | likely candidate for `_unk0073` (u8 enum, mode=1 @49% V0) |
| 0xB1 (177) | `attackIndex` | u8 | **TBD** | unmapped |
| 0xB2 (178) | `attackGroupIndex` | u8 | 0x0044 | ✅ already named |
| 0xB3 (179) | `attackDir` | u8 | 0x0004 | ✅ already named |
| 0xB4 (180) | `repeatCount` | u8 | 0x0045 | ✅ already named |
| 0xB5 (181) | `noCheckCollision` | bool | **TBD** | candidate for `_unk0072` (B00@85% V0) |
| 0xB6 (182) | `ignoreWhenHitAction` | bool | **TBD** | candidate for one of the rare-true bools (`_unk00ad`, `_unk00bd`, `_unk00bf`, `_unk00c0`, `_unk00d0..d3`, `_unk00e9`) |
| 0xB7 (183) | `isSingleHitPosition` | bool | **TBD** | candidate for one of the rare-true bools |
| 0xB8 (184) | `ignoreSafeZone` | bool | 0x0040 | ✅ already named |

**Reflection-symbol provenance:** addresses for the get/set/move/bindProperty
function-pointer slots and the corresponding `bindProperty_<field>`
implementations are listed under the table at `0x1076df1a0` (setters)
and `0x1076e3338` (getters), with bindProperty wrappers at
`0x1076d0560..0x1076d06e8`. The Mac equivalents of the runtime-filled
setters are exposed as `__ZN2pa18AttackInfoDataDesc<N>set_<field>...`.

**Why wire offsets remain unknown:** the contributor noted in
"Per-byte offset extraction — TODO" above that bindProperty wrappers
push the field offset into a *setter lambda* (inlined into the
metaobject runtime); the offset is not stored anywhere in the
descriptor, so static analysis can't recover it. The next step is
to find the `.paatt` reader/writer pair (presumably a templated
function over `pa::ReflectObjectPOD<AttackInfoDataDesc>`) and trace
its iteration order — that gives the wire→class field map directly.

**Newly-confirmed C++ fields not yet present in `BaseDataV0`:**
`targetType`, `attackIndex`, `attackImpulseLevel`, `noCheckCollision`,
`ignoreWhenHitAction`, `isSingleHitPosition`, `ignoreDefenceTypeFlag`,
`attackDivideType` (no getter found, suggests it's enum stored
inside an unnamed slot). Total: 8 C++ fields awaiting wire-position
proof — once mapped, the corresponding `_unkXXXX` placeholders in
`BaseDataV0` get renamed without any JSON-shape break (the rename is
a pure documentation improvement; bytes round-trip identically).

## Session 27 iter 2 — all 6 remaining AttackInfoDataDesc setter offsets recovered

Decompiled the per-field setter functions for the 6 confirmed C++
names. Each is a tiny ~125-byte function of the form:

```c
sub_141XXXXXX(this, *value):
    *(TYPE *)(this + OFFSET) = *value;
    sub_140F330C0(guard, "_<name>", ...);  // metaobject registration
```

Recovered (in-mem class offsets, NOT wire offsets):

| C++ name | Setter | In-mem offset | Type |
|---|---|---|---|
| `_targetType` | `sub_141950200` | 0xA0 (160) | u32 (enum TargetType) |
| `_attackIndex` | `sub_141950990` | 0xB1 (177) | u8 |
| `_ignoreDefenceTypeFlag` | `sub_14194FED0` | 0x94 (148) | u32 |
| `_ignoreWhenHitAction` | `sub_141950F20` | 0xB6 (182) | bool |
| `_isSingleHitPosition` | `sub_141951040` | 0xB7 (183) | bool |
| `_attackDivideType` | `sub_141950760` | 0xAF (175) | u8 (enum) |

**Wire-position mapping is the next step.** The in-mem class layout
differs from the 264-byte `.paatt` wire layout (Session 19
established `weaponKey` at in-mem 0xA8 ↔ wire 0x0000, etc.).
Mapping each of these 6 to a `_unkXXXX` slot in `BaseDataV0` needs
either byte-distribution analysis on a vanilla `.paatt` (match the
type + observed value range to a wire-offset slot) or finding the
`.paatt` deserializer iteration order.

Renames cannot safely ship in `BaseDataV0` until wire offset is
proven for each. Until then, these 6 names are known C++ identifiers
with known in-mem offsets — half the data needed.

The Win build of `CrimsonDesert.exe` (`bin64/CrimsonDesert.exe`) keeps
property names as literal strings in `.rdata` AND emits the
bindProperty registrar function as a single statically-addressable
function per class. Decompiling **`sub_141957EC0`** (Win address)
returned the complete AttackInfoDataDesc property list — exactly 25
names, matching the C++ field count from the doc above.

**The 8 Session-19 candidate names are all CONFIRMED as real C++
identifiers** (no longer "candidates"):

| Wire offset (BaseDataV0) | Was | C++ canonical | Status |
|---|---|---|---|
| 0x0073 | `_unk0073` | `_attackImpulseLevel` (u8) | ✅ Confirmed |
| 0x0072 | `_unk0072` | `_noCheckCollision` (bool) | ✅ Confirmed |
| TBD | (not in struct) | `_targetType` (enum) | ✅ Confirmed C++ name; wire position TBD |
| TBD | (not in struct) | `_attackIndex` (u8) | ✅ Confirmed C++ name; wire position TBD |
| TBD | (not in struct) | `_ignoreDefenceTypeFlag` (u32) | ✅ Confirmed C++ name; wire position TBD |
| TBD | (not in struct) | `_ignoreWhenHitAction` (bool) | ✅ Confirmed C++ name; wire position TBD |
| TBD | (not in struct) | `_isSingleHitPosition` (bool) | ✅ Confirmed C++ name; wire position TBD |
| TBD | (not in struct) | `_attackDivideType` (enum) | ✅ Confirmed C++ name; wire position TBD |

The remaining 17 names matched what BaseDataV0 already had named
(canonical-with-snake_case-translation: `_weaponKey` →
`weapon_key`, `_attackDir` → `attack_dir`, etc.). Snake_case is the
Rust convention; the canonical names are the underscore-prefixed
camelCase per Pearl Abyss.

This unblocks the strict-T0 verification for AttackInfoDataDesc and
proves the Win-binary recipe works at scale for the rest of the codebase.
See `docs/T0_AUDIT_TRACKING.md` for the per-class loop work.

## Appendix: `.paatt` loader anchors (Session 20, IDA-confirmed)

Located the `.paatt` file loader chain in the Mac binary. Useful as
durable IDA anchors for future RE work.

### Format anchors

| Address | Symbol | Role |
|---|---|---|
| `0x100c46104` | `sub_100C46104` | `.paatt` LOADER. Walks `<resource_root>/attackinfo` for `*.paatt` files; per-file calls `sub_100C465A4`. |
| `0x100c465a4` | `sub_100C465A4` | Per-`.paatt` parser. Reads `InfoCount` u32, allocates 88-byte AttackInfo records, then reads 9 trailing string tables in fixed order. |
| `0x100c4712c` | `sub_100C4712C` | Per-AttackInfo record reader. Reads version byte, allocates BaseData blob (264/528/296/288/264 bytes for V0/V1/V2/V3/V4), then reads 9 child sub-structures via `sub_1014123DC`. |
| `0x1014123dc` | `sub_1014123DC` | CArray<16B element> reader. Reads `u8 count`, then `16 × count` bytes as a contiguous buffer; returns 0 on alloc failure. Used by `sub_100C4712C` for each of the 9 child slots, and by `sub_10058F658` consumers throughout the binary as the canonical "16-byte element list" reader. |
| `0x1011a72d0` | `sub_1011A72D0` | Returns the literal `"paatt"` extension string. |
| `0x10732d49e` | (string data) | Literal `"paatt"` (5 bytes). |

### `.paatt` top-level wire layout (IDA-verified, error-message-derived)

Korean error messages inside `sub_100C465A4` reveal the loader's read
order — when a section fails to parse, it emits `AttackInfo 로드 실패(<section>)`.
The order is therefore the WIRE order:

1. **InfoCount** (u32) — number of AttackInfo records.
2. **AttackInfo[InfoCount]** — each record per `sub_100C4712C`:
   - u8 `version` (0/1/2/3/4)
   - BaseData blob: 264 B (V0), 528 B (V1), 296 B (V2), 288 B (V3), 264 B (V4)
   - 9× child 16-byte sub-structures (slot indices 0..8 in the per-record allocation)
3. **StringTable**
4. **EffectNameTable**
5. **EffectInfoKeyTable**
6. **SocketNameTable**
7. **PartNameTable**
8. **SequencerNameTable**
9. **PrefabNameTable**
10. **FrameEventBuffer**

This matches the existing dmm-parser `PaattFile` parser exactly — the
220/220 vanilla round-trip already validated this layout empirically.
The IDA confirmation is a durable correctness anchor for future work.

### Why wire ≠ in-memory class layout (resolved)

`sub_10058F658(stream, size)` (called from `sub_100C4712C` line 50) is
a stream-read primitive — it allocates `size` bytes and reads them
contiguously from the input. The returned pointer is stored at
AttackInfo slot `a1[9]` as the raw serialized blob. The C++ class
`pa::AttackInfoDataDesc` at `a1[9]` having `weaponKey` at in-mem
offset 0xA8 (168) describes the **deserialized in-memory layout** —
that layout differs from the on-disk wire layout because Pearl Abyss
parses each field via the metaobject's setter pipeline, not memcpy.

This means the in-memory offsets recovered in Session 19 do **not**
directly translate to wire offsets, even though both refer to the
same logical fields. Wire→class field mapping still requires either
(a) finding the `pa::AttackInfoDataDesc` `serialize` / `deserialize`
member that walks the metaobject in registration order, or
(b) field-by-field byte-signature analysis on a vanilla record.

Use `examples/paatt_basedata_layout.rs` with per-version output to confirm field boundaries.

---

## Texture VPath Cheatsheet

<!-- SPDX-License-Identifier: LicenseRef-CDMTL-1.0
     Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
     Licensed under CDMTL v1.0 - see LICENSE.txt -->

# Texture vpath + format cheatsheet

One-page reference. Print this. Tape it next to your monitor.

For the full guide, see [TEXTURE_MOD_AUTHORING.md](TEXTURE_MOD_AUTHORING.md).

---

## Format → use case

| Texture is... | Format | Color space | Notes |
|---|---|---|---|
| UI icon | **BC7** or **BC1** | SRGB | BC1 if you need small files |
| Armor diffuse | **BC7** | SRGB | High-quality color + alpha |
| Normal map (`*_n.dds`) | **BC5** | Linear | XY only, BC5 is purpose-built |
| Tattoo (`*tattoo*.dds`) | **BC7** | SRGB | Color + alpha for blending |
| Mask (1 channel) | **BC4** | Linear | Single-channel grayscale |
| Mask (2 channel) | **BC5** | Linear | Two channels packed |
| HDR / lightmap | **BC6H** | Linear | Specialized — rare |

**Always export with mip maps enabled.**

---

## Vpath patterns the game recognizes

| Vpath pattern | last4 (auto) | Used for |
|---|---|---|
| `/ui/icon/<name>.dds` | `0x1580` | Item icons, button icons |
| `/ui/<name>.dds` | `0x1580` | All other UI textures |
| `/character/texture/<name>.dds` | `0x1280` | Generic character / armor |
| `/character/texture/<name>_n.dds` | `0x0480` | Character normal map |
| `/character/texture/*tattoo*.dds` | `0x1380` | Tattoo / decal |
| Other | (format-derived) | Use existing vpath from game |

Forward slashes only. Lowercase preferred. Always ends in `.dds`.

---

## Manifest skeleton

```json
{
  "format": 3,
  "format_minor": 1,
  "modinfo": {
    "title": "Your Mod Name",
    "author": "Your Name",
    "version": "1.0",
    "description": "What it does."
  },
  "targets": [
    {
      "kind": "asset",
      "asset_type": "dds",
      "file": "<relative path under assets/>",
      "vpath": "<game vpath>"
    }
  ]
}
```

Multiple textures? Add more `targets` entries. Mixed with table edits?
See [`MOD_AUTHOR_GUIDE.md`](MOD_AUTHOR_GUIDE.md).

---

## Build pipeline

```sh
# Pre-flight check (catches problems before users see them)
python -m dmm_parser.tools.validate my_mod/mod.field.json --assets my_mod/assets

# Build the shippable zip (auto-fills sha256 + size + vpath)
python -m dmm_parser.tools.pack my_mod/mod.field.json --assets my_mod/assets --out my_mod_v1.0.zip

# Inspect what a packed mod actually does (yours or anyone else's)
python -m dmm_parser.tools.inspect my_mod_v1.0.zip

# Conflict-check between two mods
python -m dmm_parser.tools.diff mod_a.zip mod_b.zip
```

---

## Folder layout (after `dmm-mod-pack`)

```
my_mod_v1.0.zip
├── mod.field.json          ← manifest at root
└── assets/                 ← every file referenced
    ├── ui/icon/sword.dds
    ├── character/texture/macduff.dds
    └── character/texture/macduff_n.dds
```

Drop the zip into DMM's `mods/` folder. Done.

---

## Five most common mistakes

1. **BC7 used for a normal map** → re-export as BC5. Symptom: shading
   looks wrong, lighting breaks at certain angles.
2. **No mip maps** → re-export with mips ON. Symptom: distant
   textures render as solid color.
3. **Wrong color space** (SRGB vs Linear) → match the table above.
   Symptom: washed-out colors or too-dark icon.
4. **Wrong vpath** → check exact path against DMM's vanilla browser.
   Symptom: vanilla texture still loads, your version invisible.
5. **Asset hash mismatch after editing** → run `dmm-mod-pack` again.
   Symptom: validator reports `asset_sha_mismatch`.

---

## DCC tool quick reference

**Photoshop + NVIDIA Texture Tools Exporter**: Format = `BC7 sRGB (DX10)`,
Generate Mipmaps ON.

**GIMP + DDS plugin**: Compression = `BC3 / DXT5`, Generate mipmaps ON.
For modern BC7 use NVIDIA's `nvtt_export` instead.

**Substance Painter**: Output template = DDS. Per-channel format from
the table above. Generate mipmaps ON.

**Command line (recommended)**: NVIDIA Texture Tools (`nvtt_export`):

```sh
# Color
nvtt_export input.png --format bc7 --output output.dds --mips --srgb

# Normal
nvtt_export normal.png --format bc5 --output normal.dds --mips --normal

# UI icon (smaller)
nvtt_export icon.png --format bc1 --output icon.dds --mips --srgb
```

Get nvtt_export: https://developer.nvidia.com/texture-tools-exporter
