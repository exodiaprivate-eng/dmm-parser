# EffectInfo Binary Format — effectinfo.pabgb

Derived from Win-IDA decompilation of `sub_1410DBFC0` (entry parser) and
`sub_1410DBAF0` (per-element EffectData reader). The Rust decoder lives in
`src/binary/variants/effect_data.rs` and `src/tables/effect_info/info.rs`.

IDA class names: `EffectData` (C++ memory stride 376), `MeshEffectData`
(C++ memory stride 48).

---

## pabgb/pabgh Container

`effectinfo.pabgb` uses the standard indexed blob format:

- **`.pabgh`**: u16 entry_count, then entry_count × (u32 key, u32 offset)
- **`.pabgb`**: concatenated raw entries; each entry spans `[offset .. next_offset)`

---

## Entry Layout (sub_1410DBFC0)

```
u32          key
CString      string_key          (u32 len + len bytes; no null on wire)
u8           is_blocked
u32          effect_count
effect_count × EffectDataElement (variable size each)
u32          mesh_count
mesh_count × MeshEffectData      (50 bytes each)
u8           has_equip_type
u8           has_preset
u8           target_color_lerp_type
```

The `find_mesh_split` helper in `effect_info/info.rs` locates the mesh-array
boundary by reverse-probing: it finds the largest `n` such that a u32 equal
to `n` appears exactly 4 + n×50 bytes from the end of the combined blob.

---

## MeshEffectData (50 bytes, sub_1410DBD90)

```
u8    field_a
u32   field_b
u32   field_c
u32   field_d
u32   field_e
u32   field_f
u32   field_g
u32   field_h
u32   field_i
u8    field_flag
u32   lookup_a      (u32 hash → u16 in C++ memory)
u32   lookup_b
u32   lookup_c
u32   lookup_d
```

Wire: 1 + 8×4 + 1 + 4×4 = **50 bytes** total.

---

## EffectDataElement (sub_1410DBAF0)

Variable-length. Fixed header is 300 bytes, followed by four CArrays.

```
u8                 byte_a
u32                lookup_b        (u32 hash → u16 via read_u32_lookup_EF18)
EffectDataCoreBlock core_block     (254 bytes, sub_1410D4110)
u32[6]             lookups_c       (6 × u32 hash → u16 via read_u32_lookup_DA30)
u32[4]             fields_d
u8                 byte_e
CArray<CString>    cstring_list    (sub_14106BAC0)
CArray<EffectDataD3Block> fixed144_list  (sub_141117080; each 144 bytes)
CArray<CArray<u32>> nested_u32_lists    (sub_141116ED0 → sub_141101AB0)
CArray<{u32, EffectDataInner}> inner_map (sub_141116CA0 → sub_1410DB840)
```

Fixed total: 1 + 4 + 254 + 24 + 16 + 1 = **300 bytes**, plus four CArray headers
(4 bytes each) = 316 bytes minimum.

---

## EffectDataD3Block (144 bytes, sub_1410D3DC0)

Used both as the leading 144 bytes of `EffectDataCoreBlock` and as the
element type of `fixed144_list`.

```
f32[3]   vec_a       (12)
f32[3]   vec_b       (12)
f32[3]   vec_c       (12)
f32[3]   vec_d       (12)
f32[3]   vec_e       (12)
f32[3]   vec_f       (12)
f32[3]   vec_g       (12)
u32      field_84
u32      field_88
u32      field_92
u32      field_96
u32      field_100
u32      field_104
u32      field_108
u32[4]   vec4_a      (16)
u32      field_128
u32      field_132
u8       byte_136
u8       byte_137
u16      word_138
u32      field_140
```

7×12 + 7×4 + 16 + 2×4 + 2 + 2 + 4 = 84 + 28 + 16 + 8 + 2 + 2 + 4 = **144 bytes** ✓

---

## EffectDataCoreBlock (254 bytes, sub_1410D4110)

The 144-byte D3 block followed by 110 additional bytes.

```
EffectDataD3Block   d3          (144)
u32                 field_144
f32[3]              vec_h       (12)
f32[3]              vec_i       (12)
u64                 qword_172
u32                 field_180
f32[3]              vec_j       (12)
f32[3]              vec_k       (12)
f32[3]              vec_l       (12)
f32[3]              vec_m       (12)
u32                 field_232
u32                 field_236
u8 × 14             byte_240 .. byte_253   (14 individual reads per IDA)
```

144 + 4 + 12+12 + 8 + 4 + 12+12+12+12 + 4+4 + 14 = **254 bytes** ✓

---

## EffectDataInner (sub_1410DB840)

Recursive value type inside `inner_map`. Variable-length; fixed portion is
336 bytes.

```
u32                 field_0
EffectDataCoreBlock core_block     (254 bytes)
u32[6]              lookups        (via read_u32_lookup_DA30)
CArray<CString>     list_a         (sub_141102990 → sub_1410A9D40;
                                    each element is u32 len + len bytes)
CArray<u32>         list_b         (sub_141102A60)
f32[3]              vec_a
f32[3]              vec_b
f32[3]              vec_c
f32[3]              vec_d
u32                 field_after_vecs
CArray<CString>     cstring_list   (sub_14106BAC0)
CArray<EffectDataD3Block> fixed144_list (sub_141117080)
u16                 trailing_word
```

Fixed: 4 + 254 + 24 + 48 + 4 + 2 = **336 bytes**, plus four CArray headers = 352 bytes minimum.

---

## CString Wire Format

Used by `cstring_list`, `list_a`, etc.:

```
u32   len     (byte length, no null terminator in stream)
u8[len] name
```

---

## Coverage

The typed decoder (`EffectDataElement` + `EffectDataInner`) parses all entries
in the 2039-entry 2026-4-11 effectinfo.pabgb. Round-trip is verified by the
`roundtrip` and `json_roundtrip` tests in `src/tables/effect_info/info.rs`.
