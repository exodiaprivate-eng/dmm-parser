# Crimson Desert Binary Formats

> Single-page reference for every binary format the dmm-parser library
> understands. Each section links to the authoritative recon notes,
> hexpat pattern, and Rust/Python entry point for that format.
>
> For format **uses** (mods, packing, validation) see
> `docs/MOD_AUTHOR_GUIDE.md` and `docs/api.md`.

---

## At a glance

| Format         | Role                              | Notes ref                  | Hexpat                        | Rust module                  |
|----------------|-----------------------------------|----------------------------|-------------------------------|------------------------------|
| **PAPGT**      | Pack-group tree (root index)      | `docs/archive-format.md`   | `references/papgt.hexpat`     | `src/binary/papgt.rs`        |
| **PAMT**       | Per-group pack metadata           | `docs/archive-format.md`   | `references/pamt.hexpat`      | `src/binary/pamt.rs`         |
| **PAZ**        | Compressed/encrypted file blocks  | `docs/archive-format.md`   | —                             | `src/binary/paz.rs`          |
| **Trie buffer**| Compact name index (radix tree)   | `docs/archive-format.md`   | —                             | `src/binary/trie.rs`         |
| **PAOC / PALOC** | Localization string tables     | `references/paloc_notes.md`| `references/paloc.hexpat`     | `src/binary/paloc.rs`        |
| **PABGB**      | Tabular game data containers      | `docs/archive-format.md`   | —                             | `src/item_info/`, `src/tables/` |
| **DDS**        | DirectDraw Surface (textures)     | `references/dds_notes.md`  | `references/dds.hexpat`       | `src/dds/`                   |
| **WEM**        | Wwise audio clip (RIFF-WAVE)      | `references/wwise_notes.md`| `references/wem.hexpat`       | `src/audio/wem.rs`           |
| **BNK**        | Wwise soundbank                   | `references/wwise_notes.md`| `references/bnk.hexpat`       | `src/audio/bnk.rs`           |
| **SAVE**       | Save file envelope (encrypted)    | `references/save_notes.md` | `references/save.hexpat`      | `src/save/envelope.rs`       |

All formats are **little-endian**.

---

## 1. PAPGT — pack group tree

The root index. Lists every PAMT (`<group>/0.pamt`) the game knows
about, plus the checksum that proves the PAMT hasn't been tampered
with.

```
+------- 0x00 ------- header (4-byte CRC + 2-byte count + 2-byte u0 + 1-byte ei + 3-byte enc) ---+
| ...                                                                                            |
+------- 0x10 ------- entries[count]                                                              |
|   for each entry:                                                                              |
|     u32 pack_meta_checksum  (Jenkins hashlittle2 of the PAMT post-header)                      |
|     u32 language            (locale enum)                                                      |
|     u8  is_optional                                                                            |
|     BString group_name      (u32 len + bytes + null)                                           |
+-------------------------------------------------------------------------------------------------+
```

Round-trip via `PackGroupTreeMeta::parse(&bytes) → to_bytes()`.

Front-insert during overlay merge — see `add_papgt_entry` in
`src/binary/papgt.rs` for the upsert behavior.

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
binary, PABGH = headers/schema). Each table type
(`ItemInfo`, `SkillInfo`, `BuffInfo`, `CharacterInfo`, `StageInfo`,
`GimmickInfo`, …) has its own typed parser under `src/tables/<name>/`
or the legacy `src/item_info/`.

Round-trip discipline: serialize must produce byte-identical output to
the input (`test_full_roundtrip` enforces this for ItemInfo). Tables
are added by writing the typed Rust struct + `BinaryRead` / `BinaryWrite`
impls and registering in `src/dispatch.rs`.

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

## 11. Hexpat conventions

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

## 12. Adding a new format

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
