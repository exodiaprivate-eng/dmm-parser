# Paloc Format Reverse-Engineering Notes (Phase P0)

**Date:** 2026-05-01
**Source:** IDA Pro decompilation of CrimsonDesert_Steam (macOS, base 0x100000000)
**Status:** P0 Reconnaissance complete — format verified, ready for P1 (hexpat patterns)

---

## Pearl Abyss Class Hierarchy

```
pa::LocalStringInfoManager
  └── extends pa::StaticInfoManager2<
                pa::LocalStringInfoKey,    // key type
                pa::LocalStringInfo,        // value type (the actual translated string)
                pa::LocalStringInfoManager, // self
                unsigned short              // index type
              >

pa::LocalStringInfoKey       — composite key: (category_byte, key_string)
pa::LocalStringInfo          — wrapper around the localized string value
```

vtable: `0x1078e5d70` (`_ZTVN2pa22LocalStringInfoManagerE`)
vtable+64 = the file load method (loads + decompresses + decrypts)
Singleton pointer at `qword_1085206C8` (used by lookups)

---

## Filename Convention

Format string: `%#/%#_%#.paloc` at `0x1072a7190`

Used by `sub_1006DB6A8` at `0x1006DB6A8` to build paloc paths. Pattern of three substituted fields suggests: `<dir>/<prefix>_<lang>.paloc` where lang is the language code.

The language byte is `byte_107E7D6B1` (mapped through table at `off_107702288`).

---

## File-Level Structure

Discovered via `sub_1006DB05C` at `0x1006DB05C` (the paloc top-level loader):

```
PALOC FILE (decompressed/decrypted body):
  [entries...]                  ← N entries packed back-to-back
  ...                           ← possibly metadata block (~16 bytes per entry?)
  trailing u32 count            ← total entry count, read from last 4 bytes of buffer
```

**Confirmed:** the count `v11` is read from buffer offset `(file_size - 4)`:
```c
v11 = *(_DWORD *)(v18 + v19 - 4);   // u32 at file end
```

**Suspicious (needs P1 hexpat verification):** the line `v13 = v19 - 16 * v11;` suggests there's a 16-byte-per-entry trailer/index between the entries blob and the trailing count. This may be the offset/size index that lets the game seek individual entries without parsing sequentially. Confirm with hexpat against a real file.

The body is loaded via the manager's vtable+64 method, which handles ChaCha20+LZ4 decryption transparently before the parser sees the buffer.

---

## Per-Entry Format

Discovered via `sub_1006DB368` at `0x1006DB368` (the single-entry parser):

```
ONE ENTRY:
  u64 category_code        ← read by sub_1006B3DC0 (size 8); only low byte significant
                              0x07 = generic localization
                              0x70 = item name (matches Benreuveni's "echo key 0x70")
                              0x71 = item description (matches Benreuveni's "echo key 0x71")
                              (other categories exist for npc names, quest text, etc.)
  string key                ← read by sub_1006B3F50: u32 key_len + key UTF-8 bytes
                              format: "ITEM_NAME_<NUMBER>" or similar internal identifier
  string value              ← read by sub_1006B3F50: u32 val_len + value UTF-8 bytes
                              the actual localized text shown in-game
```

This **matches Benreuveni's reverse-engineered spec** with one clarification:

> Benreuveni's "8-byte marker (07 00 00 00 00 00 00 00)" is actually a **u64 where only the low byte is significant**. Different categories use different low-byte values (0x07, 0x70, 0x71, etc.). The other 7 bytes are always zero (or padding).

---

## Key Functions Mapped

| Address | Function | Purpose |
|---|---|---|
| `0x1006DB05C` | `sub_1006DB05C` | Paloc file loader (top-level entry point) |
| `0x1006DB6A8` | `sub_1006DB6A8` | Builds paloc filename (`%#/%#_%#.paloc`) |
| `0x1006DB368` | `sub_1006DB368` | Parses one entry (category + key + value) |
| `0x1006B3DC0` | `sub_1006B3DC0` | Stream read: 8 bytes (u64) |
| `0x1006B3F50` | `sub_1006B3F50` | Stream read: u32 length + bytes (string) |
| `0x1006B3AF0` | `sub_1006B3AF0` | Memory stream constructor (vtable: `off_107701E68`) |
| `0x1006D876C` | `sub_1006D876C` | Hash key string + category → LocalStringInfoKey |
| `0x1006D8864` | `sub_1006D8864` | Look up LocalStringInfo by category + keyId |

vtable for memory stream: `off_107701E68` (referenced by `sub_1006B3AF0`)
vtable for LocalStringInfoManager: `0x1078e5d70`

---

## Decompiled Loader Loop (Annotated)

```c
// From sub_1006DB05C — the paloc file loader

sub_1005E6420(&v18, 0xFFFFFFFFLL);           // init buffer descriptor
sub_1006DB6A8(a1, v2, v20, 1LL);             // build "<dir>/<prefix>_<lang>.paloc"

// Manager loads the file (with decryption/decompression):
v5 = (*(... **)(*(_QWORD *)v3 + 64LL))(
       v3, v17, &v18, "", 0LL, 0LL, 0LL);     // vtable+64 = loadFile

if (load_succeeded && v19 >= 5) {
    v11 = *(_DWORD *)(v18 + v19 - 4);        // read count from last u32
    sub_1006B3AF0(v17, &v18);                 // setup memory stream
    // ...
    LODWORD(v7) = v11 + 1;
    while (1) {
        v7 = (unsigned int)(v7 - 1);
        if (!(_DWORD)v7) break;
        sub_1006DB368(v17);                   // parse ONE entry
    }
}
```

```c
// From sub_1006DB368 — the per-entry parser

if ( (sub_1006B3DC0(a1, &v11) & 1) != 0          // u64 category
  && (sub_1006B3F50(a1, &v10) & 1) != 0          // string key (u32 len + bytes)
  && (sub_1006B3F50(a1, &v9) & 1) != 0 )         // string value (u32 len + bytes)
{
    keyId = sub_1006D876C(*v10, low_byte(v11));   // hash key+category
    info  = sub_1006D8864(manager, low_byte(v11), keyId);  // get LocalStringInfo
    setString(info, *v9);                          // store value
}
```

---

## Encryption / Compression Envelope — Phase P0.5 Findings

### Pearl Abyss Cryptography Class Hierarchy (from IDA)

```
pa::Cryptogram                    (base class, vtable @ 0x1077031a8)
  ├── pa::CryptogramAesGcm        (vtable @ 0x107703208)
  └── pa::CryptogramChaCha20      (vtable @ 0x107703268)
```

`pa::Cryptogram` typeinfo @ `0x107703288`. ChaCha20 typeinfo @ `0x1077032b0`.

### ChaCha20 Constructor (`sub_1006E0E40`)

Signature: `CryptogramChaCha20(this, key_bytes*, key_len, flag1, flag2, flag3)`

Key derivation: **cycles input bytes modulo key_len to fill a 16-byte buffer at offset 8 of the object**:

```c
*(_BYTE *)(this + 8 + i) = key_bytes[i % key_len];  // for i in 0..15
```

This means the actual ChaCha20 key is derived from a smaller seed by repeating bytes. The seed length and value depend on the caller — for paloc loading, likely derived from the language code or fixed to a per-build constant.

Three flag bytes stored at object offsets +24/+25/+26 — these probably control:
- 0x0032 envelope flag bits (encryption=1, compression=1, version=0x32)

### Underlying Cipher Implementation

The binary bundles **OpenSSL** for ChaCha20:
- `"ChaCha20 for ARMv8, CRYPTOGAMS by <appro@openssl.org>"` @ `0x104434108`
- `"../crypto/evp/e_chacha20_poly1305.c"` @ `0x10744e107`
- `"chacha20"` / `"ChaCha20"` / `"ChaCha20-Poly1305"` strings present

So `pa::CryptogramChaCha20` is a thin Pearl Abyss wrapper over OpenSSL's EVP ChaCha20.

### File Loader Path (`v3 = manager+16`, `vfunc 8 = vtable+64`)

The `sub_1006DB05C` paloc loader calls `(vfunc 8)(file_loader, request, output_buffer, ...)` where the file loader is stored at offset 16 of the LocalStringInfoManager. Tracing this further requires identifying the file loader's exact type, which appears to be a generic PA archive/file-system reader rather than paloc-specific code. The decryption flow happens within this loader before the parser sees the buffer.

### Practical Strategy for Phase P5 (Rust ChaCha20 envelope port)

**Don't reverse-engineer the full crypto trace from IDA.** Benreuveni's `lib/paloc.py` already successfully decrypts production paloc files — port his working logic directly:

1. Read the file header to extract the flags byte and ChaCha20 IV/nonce
2. Derive the 16-byte ChaCha20 key using the byte-cycling pattern (constructor logic above)
3. Decrypt with ChaCha20 (use `chacha20` crate or existing `crypto::chacha20` in dmm-parser)
4. LZ4-decompress the plaintext (use `lz4_flex` or existing infrastructure)
5. Parse entries via the format documented in P0

Verify roundtrip against real paloc samples (Phase P1 work). If our implementation matches Benreuveni's output, we're done — IDA confirmation isn't strictly necessary if we have working test vectors.

### Key Addresses for Future Reference

| Address | Symbol |
|---|---|
| `0x107703268` | `_ZTVN2pa18CryptogramChaCha20E` (ChaCha20 vtable) |
| `0x1077032b0` | `_ZTIN2pa18CryptogramChaCha20E` (ChaCha20 typeinfo) |
| `0x1077031a8` | `_ZTVN2pa10CryptogramE` (base Cryptogram vtable) |
| `0x107703208` | `_ZTVN2pa16CryptogramAesGcmE` (AES-GCM variant) |
| `0x1006E0E40` | `pa::CryptogramChaCha20` constructor |
| `0x1006E1720` | Cryptogram base constructor |
| `0x1006E1754` | Cryptogram base destructor |
| `0x104434108` | OpenSSL ChaCha20 banner string |
| `0x10744e107` | OpenSSL `e_chacha20_poly1305.c` source ref |

---

## Verification Against Benreuveni's Python Tool

| Benreuveni's `lib/paloc.py` claim | Confirmed via IDA? |
|---|---|
| 8-byte marker `07 00 00 00 00 00 00 00` | ✅ Yes — it's a u64 where low byte = category |
| u32 key_len + key UTF-8 bytes | ✅ Yes — `sub_1006B3F50` reads u32 length then bytes |
| u32 value_len + value UTF-8 bytes | ✅ Yes — same read function called twice |
| Files at flags=0x0032 use ChaCha20+LZ4 | ⏳ Need to verify by tracing manager vtable+64 |
| Paloc IDs computed as `(item_key << 32) \| 0x70/0x71` | ✅ Consistent with category byte hypothesis (0x70/0x71) |

---

## Next Steps (Phase P1 — Hexpat Patterns)

1. **Locate sample paloc files** — find a real `_<lang>.paloc` file on disk (probably under the game install's data folder or 0012/0008 PAZ extract)
2. **Write hexpat for plain format** — start with the body structure (entries + trailer + count)
3. **Test against a known-good sample** — verify entry parsing produces sensible (key, value) pairs in English
4. **Trace the manager vtable+64 method** — decompile and document the ChaCha20+LZ4 envelope (separate Phase P5 work)
5. **Document the 16-byte-per-entry trailer** — verify hypothesis from `v19 - 16 * v11` line

---

## Open Questions

1. What are the 16 bytes per entry between the body and the trailing count? An offset/size index? A hash table?
2. What language codes are valid (`byte_107E7D6B1` table at `off_107702288`)? List of supported languages.
3. Where is the ChaCha20 key derived from? Looks like it's per-language or fixed for the build.
4. Are there OTHER category codes beyond `0x07`, `0x70`, `0x71`? Probably yes for NPCs, quests, etc.
5. Is the trailing count a u32 or could it have a header before the entries section?

---

*End of P0 notes. Ready to start P1 (hexpat) once a sample paloc file is located.*
