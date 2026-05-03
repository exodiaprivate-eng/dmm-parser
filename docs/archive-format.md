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
| 0 | u32 | unknown0 |
| 4 | u32 | checksum (Jenkins hashlittle2 of post-header data) |
| 8 | u8 | entry_count |
| 9 | u8 | unknown1 |
| 10 | u16 | unknown2 |

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
