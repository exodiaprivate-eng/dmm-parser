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
