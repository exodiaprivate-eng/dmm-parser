# EffectData Binary Format — effectinfo.pabgb

Empirically derived from effectinfo.pabgb dumps via Python hex analysis
(`tools/analyze_effectinfo[1-13].py`). Documents the **outer blob structure**
as seen from the wire; the IDA-derived field-level decoder for the inner
EffectDataElement record lives in `src/binary/variants/effect_data.rs` and
uses a different (non-empirical) analysis layer.

Field names are inferred from context; true names are unknown without IDA
symbol access.

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
order as the string pairs. Internal layout partially mapped from 475-blob
(3 entries with `named_item_count=1`):

| offset (within struct) | size | observation |
|------------------------|------|-------------|
| 0..12                  | 12   | f32[3]: 3 unique values each — 0 or 1.5 (2 entries vs 1) |
| 12..24                 | 12   | f32[3]: small values — 0 or ~{0.784, 0.392, 0.078} (colour-like) |
| 24..36                 | 12   | zeros |
| 36..48                 | 12   | f32[3]: 0.0 or all three = 0.05f (`cd cc 4c 3d`) |
| 48..88                 | 40   | zeros |
| 88..136                | 48   | float cluster — mirrors fixed_prefix[92:140] (same field layout) |
| 136..138               | 2    | `0a 05` (constant — same type marker as prefix[140:142]) |
| 138..144               | 6    | trailing — `0x09 0x01` or `0x02 0x01` + zeros (3-entry variation) |

Total size confirmed: 144 bytes (4-24 475-blob: mc_off 463 − struct_start 319 = 144).
The float cluster at struct[88..136] aligns with prefix[92..140] with a −4 byte
shift, confirming NamedItemStruct and the main prefix share the same inner
sub-struct layout starting 4 bytes earlier.

---

## Sub-Element (303 bytes in 4-11, 316 bytes in 4-24)

Present when `(mc_off − Y) > 0`. Count K is implicit (no count field stored).
Mapped from the 4-24 639-blob (K=1), sub-element at blob[311..627].

**Header (bytes 0..92 — all zeros except the header packet):**

| offset (within sub-element) | size | observation |
|-----------------------------|------|-------------|
| 0..8                        | 8    | zeros |
| 8                           | 1    | `0x01` (constant — version or type byte) |
| 9..12                       | 3    | variable — `57 04 06` (5 of 6 entries) or `c6 46 5e` (1 entry) |
| 12                          | 1    | variable — `0x24` (= 36) or `0x26` (= 38) |
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

## MeshEffectData (351 bytes in 4-11, 364 bytes in 4-24)

Location: immediately after `mesh_count` u32 at `mc_off + 4`.
In 4-24, confirmed up to m=16 (6147-byte blob).

**Activity flag:** mesh[0] = u8, either 0x01 (active) or 0x00 (null slot).
In m=2 entries, all second slots are null (mesh2[0]=0, mesh2[4:8]=0).
Only active slots carry meaningful body data.

**Mesh-specific prefix (mesh[0..108]):**

| mesh offset | size | description |
|-------------|------|-------------|
| 0..4        | 4    | activity flag: u32=1 (active) or 0 (null) |
| 4..8        | 4    | mesh asset hash/ID; 0x00000000 for null slots |
| 8..12       | 4    | mirrors activity flag: u32=1 or 0 |
| 12..20      | 8    | mesh IDs — same value as mesh[4:8] for active; stale for null |
| 20..32      | 12   | f32[3]: 0.0 or 4.0 per component (dimension/bounds?) |
| 32..44      | 12   | zeros (3 entries sampled) |
| 44..56      | 12   | f32[3]: RGB colour (0 or normalised colour values) |
| 56..104     | 48   | mostly zeros |
| 104..108    | 4    | f32: 0.0 or ~1.4 |

**Shared inner sub-struct (mesh[108..364]):**

Mirrors fixed_prefix[92..299] with a +16 byte shift (mesh[108+X] ≅ prefix[92+X]).
All the same landmarks appear: float cluster at mesh[108], `0a 05` at mesh[156],
−π/2 at mesh[196], TRS at mesh[216:252], constant 0x01 at mesh[252],
flags/IDs region at mesh[260..284], `0xe173eac5` sentinel at mesh[284..300].

---

## Fixed Prefix (blob[4 .. 4+FP])

Full field-level map from systematic byte and 4-byte-window scans across all
1952 baseline blobs in the 4-24 dump (FP=299, prefix offset = blob offset − 4).

### Region 1 — Colour parameters (prefix[0..92])

| prefix offset | size | type    | description |
|---------------|------|---------|-------------|
| 0..4          | 4    | zero    | always zero |
| 4..16         | 12   | f32[3]  | **grayscale tint** (all three always equal): default (0,0,0); 5 entries set value to 0.3/0.5/0.6/0.85. Mutually exclusive with color1/color2 — entries use one or the other. |
| 16..28        | 12   | f32[3]  | **color1** (start color, RGB normalised 0..1): default (0,0,0); 47 entries non-zero |
| 28..40        | 12   | f32[3]  | **color2** (end color, RGB normalised 0..1): default (0,0,0); 22 entries non-zero. When both color1 and color2 are non-zero they are usually equal (constant color). |
| 40..52        | 12   | f32[3]  | 3 floats, each 0.0 or 0.05f (`cd cc 4c 3d`); only 2 of 1952 entries use this |
| 52..88        | 36   | zero    | always zero |
| 88..92        | 4    | f32     | ~99% zero; rarely non-zero (small positive float) |

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

1. **Map prefix[16..92] precisely** — color1 and color2 RGB triplets are
   identified; the 3 floats at prefix[40..52] (0 or 0.05) and the float at
   prefix[88..92] remain unnamed. Scan co-variation with other fields.

2. **Map sub-element[9:13]** — the 4-byte header packet (e.g. `57 04 06 24`)
   has only 2 distinct values across 6 entries; the 955-blob (K=2) would
   provide a second sub-element to compare.

3. **Identify prefix[256..264] IDs** — two per-entry u16 identifiers at
   prefix[258..262]; likely reference external tables (texture IDs, material
   hashes?).

4. **Resolve irregular blobs** — figure out whether the bone-name
   sub-element is preceded by a size prefix, and what the 373-blob
   (smaller irregular) contains.

5. **MeshEffectData mesh[56..108]** — 48 bytes mostly zero; expand beyond
   3-entry sample to characterize the f32 at mesh[104].

6. **NamedItemStruct struct[0..36]** — only 3 instances available; the
   colour-like values at struct[12..24] need cross-referencing with prefix
   color1/color2 fields.
