# EffectData Binary Format — effectinfo.pabgb

Empirically derived from 2057 effectinfo entries via Python hex analysis
(`tools/analyze_effectinfo[1-7].py`). Updated 2026-04-29 against the 2026-4-11
pabgb (2039 entries). The game updated between the two dumps and all three inner
blob sizes shrank: fixed_prefix 299→287, sub_element 316→303, mesh_element
364→351. The outer container layout is unchanged. The Rust decoder in
`src/binary/variants/effect_data.rs` uses the current (2026-4-11) sizes.

Field names are guesses derived from context; the IDA class names are
`EffectData` (stride 376 in C++ memory) and `MeshEffectData` (stride 48).

---

## Outer pabgb/pabgh Container

`effectinfo.pabgb` uses the standard indexed blob format:

- **`.pabgh`**: u16 entry_count, then entry_count × (u32 key, u32 offset)
- **`.pabgb`**: concatenated entries; each entry spans `[offset .. next_offset)`

Each raw entry starts with:

```
u32_le   key
u32_le   string_len
u8[string_len]  string_key
u8       null          (CString null terminator)
u8       is_blocked
[blob]
u8       has_equip_type    ┐
u8       has_preset        ┤ TAIL_SIZE = 3 bytes, stripped before blob parsing
u8       target_color_lerp_type ┘
```

The "blob" extracted after stripping the outer wrapper and the 3-byte tail
is what the layout below describes.

---

## Blob Layout

```
Offset  Size    Field
──────────────────────────────────────────────────────────────
0       4       constant {0x01,0x00,0x00,0x00}  (LE u32=1; always present)
4       287     fixed_prefix  (see § Fixed Prefix below)
291     4       named_item_count   u32_le  (0 for ~95% of entries)
295     var     string_pairs       named_item_count × StringPair
X       var     struct_section     u32_le count + count × 144-byte NamedItemStruct
Y       var     sub_elements       K × 303 bytes  (K implicit: (mc_off − Y) ÷ 303)
mc_off  4       mesh_count         u32_le
mc_off+4  m×351  mesh_elements    mesh_count × MeshEffectData (351 bytes each)
end−8   8       {0,0,0,0,0,0,0,0}  trailing zeros
```

Where:

```
X       = 295 + Σ(4 + len_i)  for i in 0..named_item_count
            ↑ 4 bytes = the length-prefix for each string pair
            ↑ len_i  = byte length of the i-th string (no null terminator)

Y       = X + 4 + named_item_count × 144
            ↑ 4 bytes = the struct_section count prefix
            ↑ always equal to named_item_count

mc_off  = Y + K × 303   (K can be 0; solve from blob_size: mc_off = blob_size − 8 − 4 − mesh_count × 351)
```

---

## Size Examples (2026-4-11 pabgb, current)

| blob_size | named_item_count | items | K (×303) | mesh_count (m×351) | notes |
|-----------|-----------------|-------|----------|--------------------|-------|
| 311       | 0               | —     | 0        | 0                  | baseline (1935 entries) |
| 463       | 1               | "leaf"(4)     | 0 | 0              | |
| 464       | 1               | "dist1"(5)    | 0 | 0              | |
| 465       | 1               | "smoke1"(6)   | 0 | 0              | |
| 615       | 2               | "par1"(4)+"par2"(4) | 0 | 0         | |
| 616       | 2               | "lens1"(5)+"par1"(4) | 0 | 0        | |
| 773       | 3               | "par1"+"vector1"+"vector2" | 0 | 0  | |
| 614       | 0               | —     | 1        | 0                  | |
| 917       | 0               | —     | 2        | 0                  | |
| 662       | 0               | —     | 0        | 1                  | |
| 1013      | 0               | —     | 0        | 2                  | |
| 1364      | 0               | —     | 0        | 3                  | |
| 1715      | 0               | —     | 0        | 4                  | |

General formula for standard blobs: `blob_size = 311 + named_items_extra + K×303 + m×351`

where `named_items_extra` = `Σ(4 + len_i)` for each named item string + `named_item_count × 144` for structs.

---

## StringPair

```
u32_le  len          (byte length of name, no null terminator in stream)
u8[len] name         (ASCII, e.g. "leaf", "core", "sub", "par1", "vector1")
```

All string pairs for an entry are stored consecutively (no structs between them).

---

## NamedItemStruct (144 bytes)

Immediately follows the struct_section count u32. One per named item, same
order as the string pairs. Internal layout **not yet fully mapped** — known
landmarks:

| offset (within struct) | observation |
|------------------------|-------------|
| ~132..135              | bytes `0a 05` appear (type or flag field) |
| ~137..140              | `05 01 00 00` (u32=261) — appears identically in struct1 and struct2 of 626-blob |
| various                | several `00 00 80 3f` = 1.0f IEEE 754, and `cd cc 4c 3d` ≈ 0.05f |

Total size confirmed: 144 bytes (derived from 475-blob: mc_off 463 − struct_start 319 = 144).

---

## Sub-Element (303 bytes each)

Present when the blob_size exceeds what named items alone account for.
Count K is implicit (no count field stored). Known landmarks in the
614-blob (K=1) sub-element starting at blob offset 299:

| offset (within sub-element) | observation |
|-----------------------------|-------------|
| 0..7                        | zeros (blob[311..318] = all 0) |
| 8                           | 0x01 (blob[319] = 0x01) |
| 9..11                       | 0x57 0x04 0x06 (blob[320..322]) |
| 12                          | 0x24 (blob[323]) |
| ~92..                       | float 1.0 values appear (same pattern as main EffectData prefix) |
| ~140..                      | `0a 05` flag signature |

The 303-byte size is a fixed-size EffectData sub-element (a simplified copy
of the main structure), based on the repeated float and flag patterns.

Internal layout **not yet fully mapped**.

---

## MeshEffectData (351 bytes each)

Each element is 351 bytes. Internal layout **not yet mapped**.
The IDA C++ stride is 48 bytes (memory); the stream size changed from 364
(older pabgb) to 351 (2026-4-11 pabgb).

Location: immediately after `mesh_count` u32 at `mc_off + 4`.

Confirmed present in: 662 (m=1), 1013 (m=2), 1364 (m=3), 1715 (m=4), 2066 (m=5).

---

## Fixed Prefix (287 bytes, blob[4..291])

Contains the bulk of scalar EffectData fields. Not yet decomposed field-by-field.
Known landmarks from hex dumps across multiple entries:

| blob offset | observation |
|-------------|-------------|
| 4..99       | mostly zeros for most entries |
| 96..111     | float cluster: `00 00 80 3f` (1.0f) appears at ~4-byte-aligned positions |
| 108, 112, 124, 128, 136, 140 | float 1.0 confirmed (from sub-element float scan in analyze_effectinfo6) |
| ~144 (0x90) | `0a 05 00` sequence — likely a type/enum field |
| ~172..220   | more scalar data including floats |
| ~224..302   | trailing fixed fields |

The 373-blob (50 bytes extra, str_count=0, K=0 implied) has non-zero content
at the END that doesn't conform to 8 trailing zeros — suggesting it may have
a different internal structure or the sub-element is of a variable type. **TBD.**

---

## Coverage

The confirmed layout (with 2026-4-11 constants) parses **2032 / 2039 entries
(99.7%)** correctly. 7 entries across 5 blob sizes don't fit: sz=361 (2),
sz=806 (1), sz=1356 (2), sz=1723 (1), sz=2074 (1).

All failures are "cannot determine mesh count": no value of `m` in 0..20
satisfies `mc_off = sz − 8 − 4 − m×351` with `mc_off ≥ Y` and
`(mc_off − Y) % 303 == 0`. These are likely the variable-length bone-name
sub-element variant (same class as the 831-blob from the original analysis).
They fall back to Raw bytes in `EffectDataBlob::Raw`, preserving round-trip.

---

## Irregular Blobs (7 entries across 5 sizes: 361, 806, 1356, 1723, 2074)

These entries don't fit `311 + named_items_extra + K×303 + m×351`.
Evidence from the original-pabgb 831-blob: it contains a **bone name list**
(CString array) inside what appears to be a sub-element with variable-length
content:

```
(from 831-blob of older pabgb, starting at blob ~614)
07 00 00 00             ← count = 7 bone names
0b 00 00 00             ← len = 11
42 69 70 30 31 20 53 70 69 6e 65   "Bip01 Spine"
0c 00 00 00             ← len = 12
42 69 70 30 31 20 53 70 69 6e 65 31  "Bip01 Spine1"
... (7 names total: Spine, Spine1, Spine2, R Clavicle, R UpperArm, R Elbow, R Hand)
(followed by 7 floats: 89 88 08 3e ≈ 0.133f, one per bone)
```

This bone name + weight structure is likely a `CArray<NamedBoneWeight>` living
inside a variable-size sub-element type. The 303-byte fixed sub-element and this
variable-size bone-name sub-element are likely different variants of the same
`_effectDataList` sub-item type. **Full structure TBD.**

---

## Next Steps for Decoder

1. **Map the 299-byte fixed prefix** — compare 5-10 baseline blobs at the
   byte level; extract scalar fields, floats, and enum bytes. Target: a
   `FixedEffectDataPrefix` struct with ~15-20 named fields.

2. **Map the 144-byte NamedItemStruct** — dump single-item blobs (475, 476,
   477) and identify float clusters, enum bytes, and zero-padding.

3. **Map the 316-byte sub-element** — compare multiple ×316 blobs (639, 955)
   field by field. Note overlap with the main prefix float pattern.

4. **Map the 364-byte MeshEffectData** — compare 687 (m=1) and 1051 (m=2)
   blobs in the mesh region.

5. **Resolve irregular blobs** — understand whether the bone-name sub-element
   has its own count/size prefix, and whether the 373-blob is genuinely
   malformed or uses a different variant tag.

6. **Write Rust structs** in `src/binary/variants/effect_data.rs` using
   the confirmed layout above. Start with the outer frame (constant, prefix
   blob, named_item_count, mc_off detection, mesh_count, trailing) and
   keep the prefix/item/sub-element/mesh-element bodies as `Vec<u8>` initially.
   Then fill in field-by-field once each inner layout is confirmed.
