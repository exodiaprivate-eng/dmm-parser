# EffectData Binary Format — effectinfo.pabgb

Empirically derived from effectinfo.pabgb dumps via Python hex analysis
(`tools/analyze_effectinfo[1-9].py`). Documents the **outer blob structure**
as seen from the wire; the IDA-derived field-level decoder for the inner
EffectDataElement record lives in `src/binary/variants/effect_data.rs` and
uses a different (non-empirical) analysis layer.

Field names are guesses derived from context; the IDA class names are
`EffectData` (stride 376 in C++ memory) and `MeshEffectData` (stride 48).

---

## Version History

Three inner sizes changed across patch versions:

| constant         | pre-4-11 | 4-11  | 4-23 / 4-24 |
|------------------|----------|-------|-------------|
| `fixed_prefix`   | 299      | 286   | 299         |
| `sub_element`    | 316      | 303   | 316         |
| `mesh_element`   | 364      | 351   | 364         |
| baseline blob    | 322      | 310   | 323         |

The 4-11 patch shrank all three constants; the 4-23 patch reverted them.
The outer container layout is unchanged across all versions.

**Diff (4-11 → 4-24):** 12 zero bytes inserted at blob offset 172
(= fixed_prefix offset 168) for entries originally 311 bytes; 13 bytes
for entries originally 310 bytes. In both cases the -π/2 constant float
(`db 0f c9 bf`) moved from blob[172] to blob[184]. All common keys between
the two dumps diverge at exactly blob offset 172 (confirmed across 2035/2036
common entries).

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
4       FP      fixed_prefix  (see § Fixed Prefix below)
4+FP    4       named_item_count   u32_le  (0 for ~95% of entries)
8+FP    var     string_pairs       named_item_count × StringPair
X       var     struct_section     u32_le count + count × 144-byte NamedItemStruct
Y       var     sub_elements       K × SUB bytes  (K implicit: (mc_off − Y) ÷ SUB)
mc_off  4       mesh_count         u32_le
mc_off+4  m×MESH  mesh_elements    mesh_count × MeshEffectData
end−8   8       {0,0,0,0,0,0,0,0}  trailing zeros
```

Where per-version constants are:

| symbol   | 4-11 | 4-23/4-24 |
|----------|------|-----------|
| FP       | 286  | 299       |
| SUB      | 303  | 316       |
| MESH     | 351  | 364       |

And:

```
X      = (8+FP) + Σ(4 + len_i)  for i in 0..named_item_count
           ↑ 4 bytes = the length-prefix for each string pair
           ↑ len_i  = byte length of the i-th string (no null terminator)

Y      = X + 4 + named_item_count × 144
           ↑ 4 bytes = the struct_section count prefix
           ↑ struct count always equals named_item_count

mc_off = Y + K × SUB   (solve from blob_size: see mc_off detection below)
```

**mc_off detection** — two-step: (1) iterate candidate `m` from largest to
smallest; for each, compute `mc_off = blob_size − 8 − 4 − m×MESH`; check
that `u32_le(blob, mc_off) == m`. (2) verify `(mc_off − Y) % SUB == 0`.
The divisibility check is required to avoid false positives: when mesh data
happens to be zero at a candidate mc_off, step (1) alone gives wrong m.

---

## Size Examples

### 2026-4-11 pabgb (2039 entries, FP=286, SUB=303, MESH=351)

General formula: `blob_size = 310 + named_items_extra + K×303 + m×351`

where `named_items_extra = Σ(4+len_i) + named_item_count×144`.

| blob_size | named_item_count | K (×303) | m (×351) | notes |
|-----------|-----------------|----------|----------|-------|
| 310       | 0               | 0        | 0        | baseline (1935 entries) |
| 462       | 1 ("leaf", 4)   | 0        | 0        | |
| 463       | 1 ("dist1", 5)  | 0        | 0        | |
| 464       | 1 ("smoke1", 6) | 0        | 0        | |
| 614       | 2               | 0        | 0        | |
| 615       | 2               | 0        | 0        | |
| 613       | 0               | 1        | 0        | |
| 916       | 0               | 2        | 0        | |
| 661       | 0               | 0        | 1        | |
| 1012      | 0               | 0        | 2        | |
| 1363      | 0               | 0        | 3        | |
| 1714      | 0               | 0        | 4        | |

### 2026-4-24 pabgb (2057 entries, FP=299, SUB=316, MESH=364)

General formula: `blob_size = 323 + named_items_extra + K×316 + m×364`

| blob_size | named_item_count | K (×316) | m (×364) | notes |
|-----------|-----------------|----------|----------|-------|
| 323       | 0               | 0        | 0        | baseline (1952 entries) |
| 475       | 1 ("leaf", 4)   | 0        | 0        | |
| 476       | 1 ("dist1", 5)  | 0        | 0        | |
| 477       | 1 ("smoke1", 6) | 0        | 0        | |
| 627       | 2               | 0        | 0        | |
| 628       | 2               | 0        | 0        | |
| 639       | 0               | 1        | 0        | |
| 955       | 0               | 2        | 0        | |
| 687       | 0               | 0        | 1        | |
| 1051      | 0               | 0        | 2        | 26 entries |
| 1415      | 0               | 0        | 3        | |
| 1779      | 0               | 0        | 4        | |
| 2143      | 0               | 0        | 5        | |
| 2507      | 0               | 0        | 6        | |
| 2871      | 0               | 0        | 7        | 19 entries |
| 6147      | 0               | 0        | 16       | max observed |

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

Total size confirmed: 144 bytes (derived from 4-24 475-blob: mc_off 463 −
struct_start 319 = 144).

---

## Sub-Element (303 bytes in 4-11, 316 bytes in 4-24)

Present when `(mc_off − Y) > 0`. Count K is implicit (no count field stored).
Known landmarks in the 4-24 639-blob (K=1), sub-element starting at blob[311]:

| offset (within sub-element) | observation |
|-----------------------------|-------------|
| 0..7                        | zeros |
| 8                           | 0x01 |
| 9..11                       | 0x57 0x04 0x06 |
| 12                          | 0x24 |
| ~92..                       | float 1.0 values — same pattern as main fixed_prefix |
| ~140..                      | `0a 05` flag signature |

The sub-element appears to be a fixed-size copy of an EffectData sub-record
(same float cluster and flag patterns as the main prefix).

Internal layout **not yet fully mapped**.

---

## MeshEffectData (351 bytes in 4-11, 364 bytes in 4-24)

Internal layout **not yet mapped**. The IDA C++ stride is 48 bytes (memory).

Location: immediately after `mesh_count` u32 at `mc_off + 4`.

In 4-24, confirmed up to m=16 (6147-byte blob).

---

## Fixed Prefix (blob[4 .. 4+FP])

The bulk of scalar EffectData fields. Landmarks identified from constant-byte
scan across 200 baseline blobs in the 4-24 dump (FP=299, blob[4..303]):

| blob offset (4-24) | prefix offset | observation |
|--------------------|---------------|-------------|
| 4..96              | 0..92         | mostly zeros; 240 of 299 prefix bytes are constant |
| 96..100            | 92..96        | `00 00 80 3f` = float 1.0 |
| 108..112           | 104..108      | float 1.0 |
| 112..116           | 108..112      | float 1.0 (variable across entries) |
| 124..128           | 120..124      | float 1.0 |
| 128..132           | 124..128      | float 1.0 |
| 136..140           | 132..136      | float 1.0 |
| 140..144           | 136..140      | float 1.0 |
| 144..146           | 140..142      | `0a 05` — likely type/enum field (constant) |
| 148..172           | 144..168      | variable scalar region |
| 172..184           | 168..180      | **12 zero bytes new in 4-24** (absent in 4-11) |
| 184..188           | 180..184      | `db 0f c9 bf` = float ≈ −π/2 (constant across entries) |
| 188..204           | 184..200      | variable |
| 204..240           | 200..236      | variable scalar data |
| 240                | 236           | `0x01` (constant) |
| 256                | 252           | `0x05` (constant) |
| 259                | 255           | `0x01` (constant) |
| 270..278           | 266..274      | 8 bytes: `73 e1 c5 ea 73 e1 c5 ea` (repeated — hash/timestamp?) |
| 282..286           | 278..282      | `73 e1 c5 ea` (constant) |
| 286..303           | 282..299      | trailing zeros |

The float cluster at prefix[92..140] (7 × 1.0f) mirrors the same cluster
visible in sub-elements at sub[92..140], suggesting a shared sub-struct.
The `0a 05` sequence at prefix[140] appears again in sub-elements at the same
relative offset — likely a type discriminator byte pair.

---

## Coverage

| dump      | entries | parsed | failures | failure sizes |
|-----------|---------|--------|----------|---------------|
| 2026-4-11 | 2039    | ~2032  | 7        | 360×2, 805×1, 1355×2, 1722×1, 2073×1 |
| 2026-4-24 | 2057    | 2050   | 7        | 373×2, 831×1, 1407×2, 1787×1, 2151×1 |

Each failure size is exactly 13 more than the 4-11 counterpart, confirming
the 13-byte per-entry growth. All failures fail the mc_off divisibility check:
no K exists such that `(mc_off − Y) % SUB == 0`. These are the variable-length
bone-name sub-element variant.

---

## Irregular Blobs (bone-name sub-elements)

The 7 irregular entries contain a **bone name list** (CString array) inside
what appears to be a variable-length sub-element. Evidence from the 4-24
831-blob (which has the bone list visible at blob[616..]):

```
07 00 00 00             ← count = 7 bone names
0b 00 00 00             ← len = 11
42 69 70 30 31 20 53 70 69 6e 65   "Bip01 Spine"
0c 00 00 00             ← len = 12
42 69 70 30 31 20 53 70 69 6e 65 31  "Bip01 Spine1"
... (7 names total: Spine, Spine1, Spine2, R Clavicle, R UpperArm, R Elbow, R Hand)
06 00 00 00             ← count = 6 (float array following bone names)
89 88 08 3e ...         ← 6 × f32 ≈ 0.133f (bone weights)
```

Hypothesis: the fixed-size sub-element (SUB bytes) and this variable-size
bone-name sub-element are different variants of the same `_effectDataList`
sub-item type. Full structure TBD.

---

## Next Steps

1. **Map fixed_prefix field-by-field** — use the constant/variable scan from
   script 8 as a skeleton. Target offsets: the variable bytes at prefix[100..],
   prefix[144..168], prefix[200..236]. Compare 10+ entries to find which
   fields vary per-entry vs which are constants.

2. **Map NamedItemStruct (144 bytes)** — dump single-item 4-24 blobs (475,
   476, 477) and compare byte-by-byte with the 323 baseline.

3. **Map the sub-element (316 bytes in 4-24)** — compare 639-blob and
   955-blob sub-element regions. Note the shared float pattern with the prefix.

4. **Map MeshEffectData (364 bytes in 4-24)** — compare 687 (m=1) and
   1051 (m=2) blobs in the mesh region.

5. **Resolve irregular blobs** — figure out whether the bone-name
   sub-element is preceded by a size prefix, and what the 373-blob
   (smaller irregular) contains.
