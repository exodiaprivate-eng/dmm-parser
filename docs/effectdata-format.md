# EffectData Binary Format — effectinfo.pabgb

Empirically derived from effectinfo.pabgb dumps via Python hex analysis
(`tools/analyze_effectinfo[1-25].py`). Documents the **outer blob structure**
as seen from the wire; the IDA-derived field-level decoder for the inner
EffectDataElement record lives in `src/binary/variants/effect_data.rs` and
uses a different (non-empirical) analysis layer.

Field names are inferred from context; true names are unknown without IDA
symbol access.

> **Research artifact — partially reconciled with IDA analysis.**
> The empirical blob layout (fixed\_prefix, sub\_elements, inner\_map entries at
> 364 bytes each, real MeshEffectData at 50 bytes each) has been cross-checked
> against `effect_data.rs` / `effect_info/info.rs`. Key corrections applied:
> FP=300 (not 299), all blob sizes +1, 364-byte chunks = `inner_map` entries
> (not "MeshEffectData"), real MeshEffectData = 50 bytes per IDA `sub_1410DBD90`.
> Use the IDA-derived code as ground truth for parsing; use this doc for pattern
> observations and offset landmarks within a single element's bytes.

---

## Version History

Three inner sizes changed across patch versions:

| constant         | pre-4-11 | 4-11  | 4-23 / 4-24 |
|------------------|----------|-------|-------------|
| `fixed_prefix`   | 300      | 287   | 300         |
| `sub_element`    | 316      | 303   | 316         |
| `inner_map_elem` | 364      | 351   | 364         |
| baseline blob    | 324      | 311   | 324         |

The 4-11 patch shrank all three constants; the 4-23 patch reverted them.
The outer container layout is unchanged across all versions.

`fixed_prefix` (FP) = 300 bytes: `byte_a`(1) + `lookup_b`(4) + `EffectDataCoreBlock`(254) +
`lookups_c`(24) + `fields_d`(16) + `byte_e`(1). The last field `byte_e` is always 0 in
vanilla, which caused early empirical analysis (and Snow's doc) to count FP as 299 by
absorbing `byte_e` into the "15 trailing zeros" at the end of Region 5.

**Diff (4-11 → 4-24):** 12 zero bytes inserted at blob offset 172
(= fixed_prefix offset 168) for entries originally 312 bytes; 13 bytes
for entries originally 311 bytes. In both cases the -π/2 constant float
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
0       4       CArray<EffectDataElement> count — always 1 in vanilla
4       FP      fixed_prefix  (= EffectDataElement fixed fields; see § Fixed Prefix)
4+FP    4       named_item_count   u32_le  (0 for ~95% of entries)
8+FP    var     string_pairs       named_item_count × StringPair
X       var     struct_section     u32_le count + count × 144-byte NamedItemStruct
Y       var     sub_elements       K × SUB bytes  (K implicit: (mc_off − Y) ÷ SUB)
mc_off  4       inner_map_count    u32_le  (IDA: CArray<{u32 key, EffectDataInner}> count)
mc_off+4  n×364   inner_map_elems  n × 364-byte inner_map entry (key + EffectDataInner)
end−8   8       {0,0,0,0,0,0,0,0}  trailing zeros (= inner_map_count=0 + mesh_count=0
                                    when no mesh; see IDA note below)
```

> **IDA-reality note:** Per `info.rs`, the actual blob wire format is
> `[effect_count(4)][EffectDataElement(var)][mesh_count(4)][m×50 MeshEffectData]`.
> The empirical "8 trailing zeros" = the inner_map CArray count (last 4 bytes of
> EffectDataElement, = 0 when n=0) plus the outer mesh_count (= 0 when m=0). For
> entries with m>0 real mesh elements the trailing section is
> `[inner_map_count=0][mesh_count=m][m×50 bytes]` — not all zeros.
>
> **Naming note:** Early empirical analysis (and Snow's external doc) called the 364-byte
> chunks "MeshEffectData." Per IDA they are `inner_map` entries: `u32 key +
> EffectDataInner`. The 364-byte wire size holds when all of EffectDataInner's embedded
> CArrays are empty. Real `MeshEffectData` per IDA `sub_1410DBD90` is 50 bytes:
> `u8 + 8×u32 + u8 + 4×u32` (lookups).

Where per-version constants are:

| symbol   | 4-11 | 4-23/4-24 |
|----------|------|-----------|
| FP       | 287  | 300       |
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

**mc_off detection** — two-step: (1) iterate candidate `n` (inner_map count) from
largest to smallest; for each, compute `mc_off = blob_size − 8 − n×364`; check
that `u32_le(blob, mc_off) == n`. (2) verify `(mc_off − Y) % SUB == 0`.
The divisibility check is required to avoid false positives: when inner_map data
happens to be zero at a candidate mc_off, step (1) alone gives wrong n.
For entries with real mesh (m>0) the blob is longer by `m×50` — `find_mesh_split`
in `info.rs` locates the mesh boundary first, then mc_off detection runs on the
effect-data sub-slice.

---

## Size Examples

### 2026-4-11 pabgb (2039 entries, FP=287, SUB=303, MESH=351)

General formula: `blob_size = 311 + named_items_extra + K×303 + inner_map×351 + mesh×50`

where `named_items_extra = Σ(4+len_i) + named_item_count×144`.

| blob_size | named_item_count | K (×303) | inner_map (×351) | mesh (×50) | notes |
|-----------|-----------------|----------|-----------------|------------|-------|
| 311       | 0               | 0        | 0               | 0          | baseline (1935 entries) |
| 463       | 1 ("leaf", 4)   | 0        | 0               | 0          | |
| 464       | 1 ("dist1", 5)  | 0        | 0               | 0          | |
| 465       | 1 ("smoke1", 6) | 0        | 0               | 0          | |
| 615       | 2               | 0        | 0               | 0          | |
| 616       | 2               | 0        | 0               | 0          | |
| 614       | 0               | 1        | 0               | 0          | |
| 917       | 0               | 2        | 0               | 0          | |
| 662       | 0               | 0        | 1               | 0          | |
| 1013      | 0               | 0        | 2               | 0          | |
| 1364      | 0               | 0        | 3               | 0          | |
| 1715      | 0               | 0        | 4               | 0          | |

### 2026-4-24 pabgb (2057 entries, FP=300, SUB=316, MESH=364)

General formula: `blob_size = 324 + named_items_extra + K×316 + inner_map×364 + mesh×50`

| blob_size | named_item_count | K (×316) | inner_map (×364) | mesh (×50) | notes |
|-----------|-----------------|----------|-----------------|------------|-------|
| 324       | 0               | 0        | 0               | 0          | baseline (1952 entries) |
| 476       | 1 ("leaf", 4)   | 0        | 0               | 0          | |
| 477       | 1 ("dist1", 5)  | 0        | 0               | 0          | |
| 478       | 1 ("smoke1", 6) | 0        | 0               | 0          | |
| 628       | 2               | 0        | 0               | 0          | |
| 629       | 2               | 0        | 0               | 0          | |
| 374       | 0               | 0        | 0               | 1          | one real MeshEffectData (50 bytes) |
| 640       | 0               | 1        | 0               | 0          | |
| 956       | 0               | 2        | 0               | 0          | |
| 688       | 0               | 0        | 1               | 0          | |
| 1052      | 0               | 0        | 2               | 0          | 26 entries |
| 1416      | 0               | 0        | 3               | 0          | |
| 1780      | 0               | 0        | 4               | 0          | |
| 2144      | 0               | 0        | 5               | 0          | |
| 2508      | 0               | 0        | 6               | 0          | |
| 2872      | 0               | 0        | 7               | 0          | 19 entries |
| 6148      | 0               | 0        | 16              | 0          | max observed |
| 832       | 0               | 0        | 1†              | 0          | irregular — bone-name inner_map body (+144 bytes) |
| 2536      | 0               | 7        | 0               | 0          | confirmed: 312 + 7×316 + 12 = 2536 (`Weapon_Fire_ing`) |
| 1788      | 0               | —        | —               | —          | irregular — split-reference (K=5, see Type C below) |
| 2152      | 0               | —        | —               | —          | irregular — split-reference (K=6, see Type C below) |

---

## StringPair

```
u32_le  len          (byte length of name, no null terminator in stream)
u8[len] name         (ASCII, e.g. "leaf", "core", "sub", "par1", "vector1")
```

All string pairs for an entry are stored consecutively (no structs between them).

---

## NamedItemStruct (144 bytes)

Immediately follows the struct_section count u32 (which equals named_item_count).
One struct per named item, same order as the string pairs. The struct_section
header is a single `u32=N` count (4 bytes only). The 12 zero bytes that appear
to follow the count in most entries are the first struct's colour field
(struct[0..12] = f32[3] default (0,0,0)) — not extra header padding. Blob size
examples confirm the 4-byte interpretation: a 1-named-item 475-byte blob gives
4+299+4+(4+4)+( **4** +144)+0+4+0+8 = 475, whereas a 16-byte header gives 487.

Internal layout from systematic byte scan across all 27 named item structs
(27 entries across 4-24 475–933 blobs):

| offset (within struct) | size | observation |
|------------------------|------|-------------|
| 0..12                  | 12   | f32[3]: **named-item colour** — independent of prefix color1/color2; default (0,0,0); e.g. (0.784, 0.392, 0.078) for one `leaf` component |
| 12..24                 | 12   | f32[3]: **named-item secondary colour** — default (0,0,0); not necessarily equal to prefix color2 |
| 24..36                 | 12   | f32[3]: mirrors prefix[40:52] — all three = 0.0f or all three = 0.05f (`cd cc 4c 3d`); only set when prefix[40:52] is set |
| 36..72                 | 36   | zeros |
| 72..76                 | 4    | f32: 0.0 or 0.3 |
| 76..80                 | 4    | f32: 0.0, 0.3, or 1.0 |
| 80..84                 | 4    | u32: 0, 2, or 30 — nonzero **only** for `par1` components (2 of 27 structs); possibly particle emitter parameter |
| 84..88                 | 4    | zeros |
| 88..92                 | 4    | f32 = 1.0 (constant) |
| 92..96                 | 4    | f32 = 1.0 (constant) |
| 96..100                | 4    | f32 = **−1.0** (constant sentinel) |
| 100..104               | 4    | f32 = 0.0 (constant) |
| 104..108               | 4    | f32 = 1.0 (constant) |
| 108..112               | 4    | f32 = 1.0 (constant) |
| 112..116               | 4    | f32 = 0.0 (constant) |
| 116..120               | 4    | f32 = 1.0 (constant) |
| 120..124               | 4    | f32 = 1.0 (constant) |
| 124..126               | 2    | `0a 05` (constant type marker — same as prefix[140:142]) |
| 126..127               | 1    | u8 bitmask flags (same role as prefix[142]) — values: 0x00,0x01,0x02,0x04,0x09,0x10,0x20 |
| 127..128               | 1    | u8 bool: 0 or 1 (same role as prefix[143]) |
| 128..144               | 16   | zeros |

Total size confirmed: 144 bytes across all 27 entries.

**Float cluster alignment:** struct[88:126] mirrors prefix[104:142] with a +16
offset (struct[88+X] ≅ prefix[104+X]). The struct omits prefix[92:104] (the
first 12 bytes of the prefix's inner sub-struct). No TRS or hash/ID region.

**EffectDataD3Block semantic labels** (Rust field name ↔ semantic meaning from cross-analysis):

| Rust field   | struct offset | semantic                                                  |
|-------------|--------------|-----------------------------------------------------------|
| `vec_a`     | 0..12        | named-item colour (RGB f32[3])                            |
| `vec_b`     | 12..24       | named-item secondary colour (RGB f32[3])                  |
| `vec_c`     | 24..36       | 0.0 or 0.05f triplet (mirrors prefix[40:52])              |
| `byte_136`  | 136..137     | bitmask flags (same role as prefix[142])                  |
| `byte_137`  | 137..138     | named-item bool: 0 or 1 (same role as prefix[143])        |
| `word_138`  | 138..140     | `0x0a 0x05` constant type marker (same as prefix[140:142])|

`vec_d`–`vec_g` and `field_84`–`field_140` are IDA-derived anonymous names;
semantics not yet confirmed beyond the mirror relationship with prefix[92:144].

---

## Sub-Element (303 bytes in 4-11, 316 bytes in 4-24)

Present when `(mc_off − Y) > 0`. Count K is implicit (no count field stored).
Mapped from the 4-24 640-blob (K=1), sub-element at blob[312..628].

**Header (bytes 0..92 — all zeros except the header packet):**

| offset (within sub-element) | size | observation |
|-----------------------------|------|-------------|
| 0..8                        | 8    | zeros |
| 8                           | 1    | `0x01` (constant — version or type byte) |
| 9..13                       | 4    | **sub-element type ID** — 4 bytes identifying the sub-element class; shared across all blobs of the same class (e.g. `57 04 06 24` for 5 different 639-blobs, `79 1c a5 9a` for two 955-blobs and one sub within a 2535-blob). Not per-blob-unique. |
| 13..92                      | 79   | zeros |

**Body (bytes 9..316):**

The inner EffectData record begins at sub[9]. Its fields mirror
fixed_prefix[0..] with a +9 byte offset: sub[9+X] corresponds to prefix[X].
Byte-match sub[9+X] vs prefix[X] = 272/299 identical across available entries
(variable fields like TRS and IDs naturally differ).

| sub offset    | prefix equiv  | description |
|---------------|---------------|-------------|
| 9+92 = 101    | prefix[92]    | inner sub-struct float cluster begins |
| 9+140 = 149   | prefix[140]   | `0a 05` type marker |
| 9+200 = 209   | prefix[200]   | position XYZ (real values, e.g. (−0.020, 0, −0.237)) |
| 9+212 = 221   | prefix[212]   | scale XYZ (e.g. (0.5, 1, 1)) |
| 9+224 = 233   | prefix[224]   | rotation XYZ radians |

Sub-element TRS differs from the parent blob's TRS — each sub-element has
its own transform relative to the parent effect origin.

---

## InnerMapElement (351 bytes in 4-11, 364 bytes in 4-24)

> **Naming:** Snow's doc and early empirical analysis called these "MeshEffectData." Per IDA
> they are `inner_map` entries: `u32 key + EffectDataInner`. The 364-byte wire size applies
> when all of EffectDataInner's embedded CArrays are empty. This section retains the
> landmark offsets from empirical analysis; see `effect_data.rs` for the field-typed
> IDA-derived decoder.

Location: immediately after `inner_map_count` u32 at `mc_off + 4`.
In 4-24, confirmed up to n=16 (6148-byte blob).

**Activity flag:** mesh[0] = u8, either 0x01 (active) or 0x00 (null slot).
Only mesh[0] carries a full slot directory; trailing null slots carry only a
truncated self-reference and their own hash.

**Slot directory (mesh[0] only, variable size 28..68 bytes depending on M):**

mesh[0] encodes a linked "slot directory" covering all M active mesh slots.
The directory is `max(M,2)×8 + 12` bytes (always ≥ 28 bytes):

```
[0:4]         = 1 (active flag — slot 0)
[4:8]         = hash_A (slot 0's own hash/ID)

For k = 1 .. max(M-1, 1):         ← (M-1) other slots; for M=1 this is one self-ref
  [8k:8k+4]   = 1 (active flag for slot k)
  [8k+4:8k+8] = hash_k (slot k's hash; for M=1, k=1 → hash_A self-ref)

[8+8M:8+8M+4] = M  (total slot count)
[8+8M+4:8+8M+8]  = hash_A (repeated)
[8+8M+8:8+8M+12] = hash_A (repeated again)
[8+8M+12 : 80]   = zeros (for standard entries; some complex entries override)
```

Concrete examples (from 70-sample scan — all M values confirmed):

| M | directory bytes | M field at | hash_A terminator at |
|---|-----------------|------------|----------------------|
| 1 | 28  | [16:20] | [20:28] |
| 2 | 28  | [16:20] | [20:28] |
| 3 | 36  | [24:28] | [28:36] |
| 4 | 44  | [32:36] | [36:44] |
| 5 | 52  | [40:44] | [44:52] |
| 6 | 60  | [48:52] | [52:60] |
| 7 | 68  | [56:60] | [60:68] |

For M=1: the second slot pair (k=1) is a self-reference, so [8:12]=1 and [12:16]=hash_A.
For M≥2: each slot pair (k=1..M-1) references one of the other active mesh slots.

**Null/trailing mesh slots** contain only a compact back-reference (no directory):
zeros at [0:4] (inactive), hash at [4:8], zeros elsewhere in [0:80].

**Remaining mesh[0..108] fields:**

| mesh offset | size | description |
|-------------|------|-------------|
| 0..8+8M+12  | var  | slot directory (see above) |
| 8+8M+12..80 | var  | zeros for standard entries; one observed entry has f32=4.0 and RGB at [44:56] |
| 80..104     | 24   | zeros (confirmed across all 70 active mesh[0] samples) |
| 104..108    | 4    | f32: 0.0 or ~1.4 (1 of 70 active samples) |

**Shared inner sub-struct (mesh[108..364]):**

The first 208 bytes (mesh[108..316]) mirror fixed_prefix[92..300] with a +16
byte shift (mesh[108+X] ≅ prefix[92+X]). Confirmed landmarks:

| mesh offset | prefix equiv | landmark |
|-------------|--------------|----------|
| 108         | prefix[92]   | float cluster start |
| 156         | prefix[140]  | `0a 05` type marker |
| 196         | prefix[180]  | −π/2 constant |
| 216..252    | prefix[200..236] | TRS (position, scale, rotation) |
| 252         | prefix[236]  | constant `0x01` |
| 260..284    | prefix[244..268] | flags/IDs region |
| 280..296    | prefix[264..280] | `0xe173eac5` hash region |

The remaining mesh[316..364] (48 bytes) extends beyond the fixed_prefix and
is not yet mapped.

---

## Fixed Prefix (blob[4 .. 4+FP])

Full field-level map from systematic byte and 4-byte-window scans across all
1952 baseline blobs in the 4-24 dump (FP=300, prefix offset = blob offset − 4).

### Region 1 — Colour parameters (prefix[0..92])

| prefix offset | size | type    | description |
|---------------|------|---------|-------------|
| 0..4          | 4    | zero    | always zero |
| 4..16         | 12   | f32[3]  | **grayscale tint** (all three always equal): default (0,0,0); 5 entries set value to 0.3/0.5/0.6/0.85. Mutually exclusive with color1/color2 — entries use one or the other. |
| 16..28        | 12   | f32[3]  | **color1** (start color, RGB normalised 0..1): default (0,0,0); 47 entries non-zero |
| 28..40        | 12   | f32[3]  | **color2** (end color, RGB normalised 0..1): default (0,0,0); 22 entries non-zero. When both color1 and color2 are non-zero they are usually equal (constant color). |
| 40..52        | 12   | f32[3]  | 3 floats, each 0.0 or 0.05f (`cd cc 4c 3d`); only 2 of 1952 entries use this |
| 52..88        | 36   | zero    | always zero |
| 88..92        | 4    | f32     | ~99% zero; 15 of 2057 entries non-zero with clean values {0.3, 0.5, 1.0, 1.5} — a float parameter (possibly opacity or blend multiplier) |

### Region 2 — Inner sub-struct (prefix[92..145])

This block mirrors the same inner structure found in NamedItemStruct[88..141]
and in sub-elements at sub[92..145]. Offset −4 shift in NamedItemStruct.

| prefix offset | size | type    | description |
|---------------|------|---------|-------------|
| 92..96        | 4    | f32     | ~99.7% = 1.0; 5 unique values, range 0.02..1.0 |
| 96..104       | 8    | f32[2]  | 0.0, 0.0 (constant) |
| 104..108      | 4    | f32     | ~98% = 1.0; 13 unique values, range 0.3..3.0 |
| 108..112      | 4    | f32     | ~98% = 1.0; 16 unique values, range 0.0..4.0 |
| 112..116      | 4    | f32     | default **−1.0** (sentinel); 6 unique values, can be positive |
| 116..120      | 4    | f32     | 0.0 (constant) |
| 120..128      | 8    | f32[2]  | 1.0, 1.0 (constant) |
| 128..132      | 4    | f32     | 0.0 (constant) |
| 132..140      | 8    | f32[2]  | 1.0, 1.0 (constant) |
| 140..142      | 2    | u8[2]   | `0x0a 0x05` — **constant type marker** |
| 142..143      | 1    | u8      | bitmask flags: 96% zero; nonzero values are powers of 2 {2,4,6,8,16,32,48} |
| 143..144      | 1    | u8      | bool: 0 (1950 entries) or 1 (2 entries) |
| 144..145      | 1    | u8      | enum 0..5: 73% zero, then 1(23%), 2(2%), 3(1%), 4(0.5%), 5(0.2%) |

### Region 3 — Zero padding (prefix[145..200])

All 55 bytes are constant zero in every entry. This includes the 12 zero
bytes inserted at prefix[168:180] in the 4-24 patch (absent in 4-11 where
the `−π/2` constant began at prefix[168]).

Exception embedded within the zero run:

| prefix offset | size | type | description |
|---------------|------|------|-------------|
| 180..184      | 4    | f32  | `db 0f c9 bf` = **−π/2 ≈ −1.5708** (constant) |

### Region 4 — Transform (prefix[200..236])

TRS (translation, scale, rotation) transform for this effect element,
confirmed from non-trivial entries (e.g., fire-effect entries with realistic
position, rotated turret effects with π/2 angles, etc.).

| prefix offset | size | type   | description |
|---------------|------|--------|-------------|
| 200..212      | 12   | f32[3] | **position** XYZ offset — default (0,0,0); range ~±40 |
| 212..224      | 12   | f32[3] | **scale** XYZ — default (1,1,1); **never zero**; range −1..10 |
| 224..236      | 12   | f32[3] | **rotation** XYZ in radians — default (0,0,0); range ±π |

Sample non-trivial entries:
- `pos=(0,0,0.75)  scale=(1,1,1)  rot=(0, π/2, 0)` — vertical offset, 90° yaw
- `pos=(−0.36,0,0) scale=(1.5,0.7,1.5)` — lateral shift, non-uniform scale
- `pos=(0,0,0.035) scale=(2.5,2.5,2.5)` — vertical offset, uniform upscale
- `scale=(0.05,0.05,0.01)` — tiny uniform scale

### Region 5 — Flags and IDs (prefix[236..284])

| prefix offset | size | type | description |
|---------------|------|------|-------------|
| 236..240      | 4    | u32  | constant `0x00000001` (single `0x01` byte) |
| 240..244      | 4    | —    | mostly zero; 2 of 1952 entries non-zero |
| 244..248      | 4    | u8[4]| `{0x00, 0x01, 0x00, 0x00}` for most entries (byte 245 = 1) |
| 248..252      | 4    | u8[4]| `{0x01, 0x00, 0x00, X}` where X ∈ {0,1,2,3,5} — byte 251 is an enum |
| 252..256      | 4    | u32  | constant `0x01000005` (bytes: `05 00 00 01`) |
| 256..260      | 4    | u32  | low 16 bits always 0x0000; high 16 bits = unique per-entry ID (1521 unique) |
| 260..264      | 4    | u32  | fully variable — 1741 unique values; both halves non-zero |
| 264..280      | 16   | —    | mostly constant `c5 ea 73 e1` repeated × 4 (LE u32 = `0xe173eac5`); 941/1952 entries are exactly this; others have same pattern in bytes [2:4] of each u32 with variable low bytes |
| 280..284      | 4    | u32  | constant `0x0000eac5` |
| 284..299      | 15   | zero | trailing zeros |
| 299..300      | 1    | u8   | `byte_e` — always 0 in vanilla; IDA reads as a named field (`EffectDataElement.byte_e`), making FP=300 not 299 |

**Hash region note:** The 28-byte block prefix[256..284] as 14 × u16 shows
a structured alternating pattern: u16s at positions [3,5,7,9,11] (from start
of block) are always `0xe173`; the remaining positions hold variable or
constant `0xeac5` values. The two fully-variable fields are effectively:
- `u16` at prefix[258:260] — unique per-entry identifier A (1521 unique)
- `u16` at prefix[260:262] — unique per-entry identifier B (in combination
  with the following constant `0xe173` half)

---

## Coverage

| dump      | entries | parsed | failures | failure sizes |
|-----------|---------|--------|----------|---------------|
| 2026-4-11 | 2039    | ~2035  | ~5       | 361×2, 806×1, 1356×2, 1723×1, 2074×1 (sizes +1 from original; not re-classified) |
| 2026-4-24 | 2057    | 2054   | 3        | 832×1 (TypeA), 1788×1 (TypeC), 2152×1 (TypeC) |

4-24 failure sizes are exactly 13 more than their 4-11 counterparts. Types B, D, and E
are standard after the FP=300 correction:
- Type B (374-byte) = standard mesh=1 (one 50-byte MeshEffectData)
- Type D (2536-byte) = standard K=7 sub-elements
- Type E (1416-byte) = standard inner_map=3

4-11 failure classification not re-verified after FP correction; some may also resolve.

---

## Irregular Blobs

The 3 irregular entries in 4-24 (after reconciliation; Types B, D, and E are now
standard). Byte offsets below use FP=300 boundaries:

### Type A — Bone-name inner_map body (832-byte blob)

One entry (`pafx_mc_rotationbash_lightning_gain_001a_switch_01`) has inner_map=1
but the entry body is variable-length because it embeds a bone name list and bone
weight array. The outer layout is identical to a standard inner_map blob
(blob[304:312]=8 zeros, blob[312:316]=inner_map_count=1, body, 8 trailing zeros),
but the body is 508 bytes instead of the standard 364.

The bone name list begins at **mesh offset 298**:

```
                                        ... (298 bytes standard mesh data)
07 00 00 00             ← bone_count = 7
0b 00 00 00  42 69 70 30 31 20 53 70 69 6e 65          "Bip01 Spine" (11)
0c 00 00 00  42 69 70 30 31 20 53 70 69 6e 65 31       "Bip01 Spine1" (12)
0c 00 00 00  42 69 70 30 31 20 53 70 69 6e 65 32       "Bip01 Spine2" (12)
10 00 00 00  42 69 70 30 31 20 52 20 43 6c 61 76 69 63 6c 65  "Bip01 R Clavicle" (16)
10 00 00 00  42 69 70 30 31 20 52 20 55 70 70 65 72 41 72 6d  "Bip01 R UpperArm" (16)
0d 00 00 00  42 69 70 30 31 20 52 20 45 6c 62 6f 77          "Bip01 R Elbow" (13)
0c 00 00 00  42 69 70 30 31 20 52 20 48 61 6e 64             "Bip01 R Hand" (12)
06 00 00 00             ← weight_count = 6
89 88 08 3e  ×6         ← 6 × f32 ≈ 0.1333 (bone weights per bone attachment?)
[58 trailing zeros]
```

Total body size: 508 bytes (298 standard + 4 count + 120 names + 4 count + 24 weights + 58 zeros).

### Type C — Split-reference mesh (1787, 2151-byte blobs)

Two entries use a "split header" format where K mesh headers are stored
separately from K−1 mesh bodies:

```
blob[304:312]         8 zeros (standard — named_item_count=0, struct_count=0)
blob[312:316]         K  (u32 reference count — NOT inner_map count)
blob[316:316+K×8]     K reference entries, each = (u32=1, u32=hash)
blob[316+K×8:end-8]   M = K−1  bodies, each 356 bytes
blob[end-8:end]       8 trailing zeros
```

Size formula: `316 + K×8 + (K−1)×356 + 8 = 364×K − 32`

| blob size | K | M=K-1 | entry name |
|-----------|---|-------|------------|
| 1788      | 5 | 4     | `pafx_Swim_Foot_Warmachine` |
| 2152      | 6 | 5     | `fx_smokeshell_out` |

The last two reference entries always share the same hash (a back-reference or
deduplication marker). Body layout (each 356 bytes):

| body offset | size | description |
|-------------|------|-------------|
| 0..4        | 4    | M (total body count) for body[0]; 0 for body[i>0] |
| 4..8        | 4    | hash — equals ref[i].hash for the corresponding reference entry |
| 8..12       | 4    | same hash repeated |
| 12..100     | 88   | zeros |
| 100..356    | 256  | **inner sub-struct** — mirrors prefix[92..348] with body[100+X] ≅ prefix[92+X]; confirmed by body[100:104]=f32=1.0 (≅ prefix[92]=1.0) and body[148:150]=`0a 05` (≅ prefix[140:142]) |

The body's inner sub-struct is identical in structure to standard InnerMapElement[108..364],
but positioned 8 bytes earlier in the body's own coordinate space.

### Type D — Extended sub-element blob (2536-byte blob)

> **Reconciled:** this is a standard blob with K=7 sub-elements; not irregular.

One entry (`Weapon_Fire_ing`) has K=7 sub-elements, fitting the standard
sub-element formula `312 + K×316 + 12`:

```
312 + 7×316 + 12 = 2536  ✓
```

Each sub-element starts at blob[312 + i×316] with the standard header
`00 00 00 00 00 00 00 00 01` (8 zeros + 0x01). The trailing 12 zeros are
also standard. This blob fits the general formula — it was previously
miscounted because K=7 is larger than the K≤2 samples used to calibrate.

### Type E — Reconciled: standard inner_map=3 (1416-byte blobs)

> **Reconciled:** these entries are no longer classified as irregular. With FP=300
> the correct size for inner_map=3 is 324 + 3×364 = 1416, not 1407. The earlier
> "keyed-entry" analysis was derived from a wrong byte boundary (byte_e missed).
> Both entries fit the general formula with inner_map_count=3.

Two entries (`cdfx_mc_onguard_shield_fxpreset_01` and
`cdfx_mc_onguard_shield_fxpreset_01_applyAnimationSpeed`) — blob_size 1416 =
baseline(324) + 3×364(inner_map). Standard layout; no special handling required.

---

## Next Steps

1. **prefix[88:92]** — carries clean floats {0.3, 0.5, 1.0, 1.5} in ~0.7% of
   entries; semantics not yet confirmed (possibly opacity or blend multiplier).
   prefix[40:52] is confirmed to mirror NamedItemStruct struct[24:36] (0.05f triplet).

2. **Identify prefix[256:264] IDs** — two per-entry u16 identifiers at
   prefix[258:262]; likely reference external tables (texture IDs, material hashes?).

3. **inner_map slot directory hashes** — the slot_k hashes in InnerMapElement[0] are
   external asset IDs: confirmed NOT blob keys within effectinfo.pabgb (0 hits out of
   233 cross-referenced). Likely mesh/model asset IDs pointing into a different archive.

4. **NamedItemStruct struct[80:84]** — u32 ∈ {0, 2, 30}; unknown role.

5. **Type C body remainder (body[152:356])** — inner sub-struct bytes after the
   `0a 05` marker at body[148] contain TRS and ID fields mirroring prefix[140:300];
   not yet scanned for Type C specifically.

6. **InnerMapElement inner sub-struct tail (mesh[316..364], 48 bytes)** — the inner
   sub-struct mirrors prefix[92..300] (208 bytes) at mesh[108..316]; the remaining
   48 bytes likely contain the flags/IDs region analogous to prefix[284..300].
