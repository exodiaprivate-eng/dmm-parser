# Crimson Desert Save File Format

> Recon notes for Phase S of the Mod-Author Framework. Format derived
> from Benreuveni's public `CrimsonSaveEditor` (`save_crypto.py`,
> `save_parser.py`) plus targeted hex inspection of v2 saves.
>
> **Key handling note.** The save key is derived from a fixed base key
> XOR'd against a version-specific material string. Both the base key
> and the version prefixes are publicly disclosed in the upstream
> Python tool, so the **format** is documented here in full, but the
> Rust port (`src/save/envelope.rs`) accepts keys as caller-supplied
> parameters rather than baking byte literals into the public repo.
> Callers (SWISS, CrimsonSaveEditor) hold the keys out-of-tree.

---

## 1. File envelope

```
+-----------------------------------------------+
| Header  (0x80 bytes, plaintext)               |
+-----------------------------------------------+
| ChaCha20-encrypted, LZ4-compressed payload    |
+-----------------------------------------------+
```

### 1.1 Header layout

| Offset | Size | Field            | Notes                                      |
|--------|------|------------------|--------------------------------------------|
| 0x00   | 4    | `magic`          | ASCII `SAVE`                               |
| 0x04   | 2    | `version`        | u16 LE — current saves use **2**           |
| 0x06   | 2    | `flags`          | u16 LE — observed `0x0080` on writes       |
| 0x08   | 10   | `_reserved_a`    | Preserved from original on rewrite         |
| 0x12   | 4    | `uncompressed_size` | u32 LE — size of the LZ4-decompressed body |
| 0x16   | 4    | `payload_size`   | u32 LE — size of the ciphertext after 0x80 |
| 0x1A   | 16   | `nonce`          | First 4 bytes = ChaCha20 init counter (LE), last 12 = nonce |
| 0x2A   | 32   | `hmac`           | HMAC-SHA256 of the **compressed plaintext** (pre-encryption) |
| 0x4A   | 54   | `_reserved_b`    | Padding to 0x80; not meaningful for parsing |
| 0x80   | …    | `payload`        | `chacha20( lz4( body ) )` — `payload_size` bytes |

### 1.2 Read pipeline

```
ciphertext = file[0x80 : 0x80 + payload_size]
compressed = ChaCha20.decrypt(ciphertext, key, nonce16)
hmac_ok    = HMAC-SHA256(compressed, key) == header.hmac
body       = LZ4.block.decompress(compressed, uncompressed_size)
```

HMAC mismatch is treated as a soft warning by the upstream tool — the
decompressed body is still returned. The Rust port should surface this
as an `EnvelopeWarning::HmacMismatch` finding rather than an error so
SWISS UI can decide whether to refuse the file.

### 1.3 Write pipeline

```
compressed = LZ4.block.compress(body, mode="high_compression", level=9, store_size=False)
nonce16    = random 16 bytes (4 bytes init counter + 12 bytes nonce)
hmac       = HMAC-SHA256(compressed, key)
ciphertext = ChaCha20.encrypt(compressed, key, nonce16)
header     = preserve original 0..0x12, then write magic/version/flags/sizes/nonce/hmac
file       = header || ciphertext
```

The first 0x12 bytes of the original header (magic, version, flags,
`_reserved_a`) are preserved on rewrite. Magic/version/flags are also
re-asserted (`SAVE`, version=2, flags=0x0080).

---

## 2. ChaCha20 specifics

- Standard 20-round ChaCha20 (10 double-rounds in `_chacha20_block`).
- Key is **31 bytes** of derived material padded with one trailing
  `0x00` to reach the 32-byte ChaCha20 key size.
- Nonce16 is split: first u32 LE = block counter, remaining 12 bytes =
  ChaCha20 nonce. (When using OpenSSL/`cryptography`'s ChaCha20
  primitive, the full 16-byte nonce16 is passed directly — that
  primitive treats the leading 4 bytes as the counter.)

### 2.1 Key derivation (informational)

```python
KEY = (BASE_KEY[:31] XOR (VERSION_PREFIX + "PRIVATE_HMAC_SECRET_CHECK")[:31]) || 0x00
```

Version prefixes (from upstream):
- v1: `^Qgbrm/.#@`zsr]\@rvfal#"`
- v2: `^Pearl--#Abyss__@!!`

The base key + prefixes are public. The Rust port takes `key: &[u8; 32]`
as a parameter and does NOT embed these literals — derivation lives in
the calling tool (SWISS / CrimsonSaveEditor).

---

## 3. Body structure (S3 will expand this)

The decompressed body is **not yet typed end-to-end**. Known landmarks
from the upstream `save_parser.py` and `item_scanner.py`:

- Inventory: a flat array of `SaveItem` records anchored by the
  `iteminfo`-derived item-key index. Each record carries item key,
  count, enchant level, sharpness, optional socket entries.
- Equipment: per-slot data referencing inventory rows.
- Sockets: 5-fill range, packed as parallel arrays.
- Quest progress, knowledge, dye palette: separate sub-blobs scattered
  through the decompressed buffer; addresses derived by the upstream
  scanner via signature-walk rather than fixed offsets.

Phase S4–S6 will replace the signature-walk model with a typed,
offset-driven parser as we map sections.

---

## 4. Out-of-scope for the parser

The dmm-parser save module is **read/write of the envelope and of
typed sections we explicitly support**. Out of scope:

- Save game progression/balance changes (gameplay design, not format).
- Cosmetic encoding of any DRM/anti-cheat fields beyond what's
  necessary to round-trip.
- Anti-tamper telemetry. We do not exfiltrate save contents.

---

## 5. Sample paths

- Real v2 saves: `%APPDATA%\Pearl Abyss\CrimsonDesert\Saves\` on
  Windows (per SWISS save editor's default lookup).
- Use a freshly-saved character file for round-trip testing in
  Phase S4+; older saves may carry stale layouts.

---

## 6. Implementation targets

| Phase | Module                       | Surface                              |
|-------|------------------------------|--------------------------------------|
| S2    | `src/save/envelope.rs`       | `decrypt_save(bytes, &key)` + inverse |
| S4    | `src/save/inventory.rs`      | `SaveItem { item_key, count, enchant_level, sharpness, sockets }` + r/w |
| S5    | `src/save/equipment.rs`      | Equipment slot + socket parsers       |
| S6    | `src/save/quest.rs`/`knowledge.rs`/`dye.rs` | Per-section r/w + tests |
| S7    | `src/dispatch.rs`            | `"save"` arm → `parse_save_to_json` / `serialize_save_from_json` |
| S8    | v3.1 spec extension          | `swap_item`, `add_item`, `set_quest_state`, `unlock_knowledge` ops |
| S9    | `src/python.rs`              | `parse_save_from_file`, `decrypt_save`, `serialize_save`, etc. |

---

## 7. Open questions

1. Does `flags` (0x06) carry any non-`0x0080` values in older saves?
2. Are bytes `0x4A..0x80` ever non-zero? Upstream rewrites zero the
   trailing reserved region; need to check field samples.
3. What is the in-body magic / section table? S3 recon will record this.
4. Does HMAC mismatch ever happen on legit saves (e.g. partial save
   during a crash) or is it strictly tamper detection?
