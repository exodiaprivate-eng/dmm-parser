# DDS Format Quirks — Crimson Desert (Phase D0)

**Date:** 2026-05-01
**Source:** Reference reading of DMM's `commands.rs` (NOT modifying DMM — scope is dmm-parser + SWISS)
**Status:** D0 Recon complete — ready for D1 (hexpat) and D2 (Rust skeleton)

This document captures DDS format quirks that the game enforces beyond the standard Microsoft DDS spec. The dmm-parser DDS classifier (D2-D8) needs to honor these for assets bundled via v3.1 mods.

---

## 1. DDS Header Layout (Standard + Crimson-Specific Fields)

```
+0x00  magic: u8[4]                  "DDS " (with trailing space)
+0x04  size: u32                     124 (header size)
+0x08  flags: u32
+0x0C  height: u32
+0x10  width: u32
+0x14  pitch_or_linear_size: u32
+0x18  depth: u32                    Game requires >= 1 (else DDS loader fails)
+0x1C  mip_map_count: u32            Game uses max(1, this)
+0x20  reserved1: u32[11]            ← ALL OF THESE OVERWRITTEN BY GAME-SPECIFIC LOGIC
       └── reserved1[0..4] (offsets 32-48) — game stores MIP LEVEL SIZES here
+0x4C  pixel_format (DDS_PIXELFORMAT, 32 bytes):
       +0x4C  pf_size: u32           32
       +0x50  pf_flags: u32          0x40 = uncompressed RGB
       +0x54  pf_fourcc: u8[4]       "DXT1", "DXT5", "ATI1", "ATI2", "DX10", "BC4U/S", "BC5U/S"...
       +0x58  pf_rgb_bits: u32       Bits per pixel for uncompressed
       +0x5C  pf_r_bitmask: u32
       +0x60  pf_g_bitmask: u32
       +0x64  pf_b_bitmask: u32
       +0x68  pf_a_bitmask: u32
+0x6C  caps[4]: u32
+0x7C  reserved2 ("last4"): u32      ← GAME WRITES FORMAT-SPECIFIC ID here (bytes 124-128)
                                       Crimson-Desert override (NOT standard DDS)
+0x80  body or DX10 extension header
       (if pf_fourcc == "DX10", next 20 bytes = DXGI extension):
       +0x80  dxgi_format: u32
       +0x84  resource_dimension: u32
       +0x88  misc_flag: u32
       +0x8C  array_size: u32
       +0x90  misc_flags2: u32
+0x94  body bytes (mip 0 + mip 1 + ...) for DX10 textures
+0x80  body bytes (mip 0 + ...) for non-DX10 textures
```

---

## 2. Crimson-Specific Reserved1 Patching

Per DMM's `patch_dds_header_for_overlay` (commands.rs:18957):

> Reserved1[0..3] (offsets 32-48) — game stores MIP LEVEL SIZES in these 4 u32 slots.
> `comp_size == decomp_size == full file size (game uses Reserved1[0] for actual LZ4 size)`

For overlay use, the game requires:
- `reserved1[0]` = compressed body size
- `reserved1[1..4]` = mip 0/1/2 sizes
- For non-DX10 (DXT1/DXT5): split-compression header, reserved1 patched, payload = header + LZ4 body + zero-padding
- For DX10/BC7: PATHC template registration required (see §4)

---

## 3. The "last4" Field (dwReserved2, bytes 124-128)

**Crimson-specific override** — not part of standard DDS spec.

Mapping by FOURCC (DMM's `patch_dds_header_for_overlay`):

| FOURCC | last4 value | Decimal |
|---|---|---|
| `DXT1` | 12 | 12 |
| `DXT2`/`DXT3`/`DXT4`/`DXT5` | 15 | 15 |
| `ATI1`/`ATI2`/`BC4U`/`BC4S`/`BC5U`/`BC5S` | 4 | 4 |
| `DX10` | (look up DXGI format — see below) |  |
| (other) | None — fallback to PATHC class or path-prefix |  |

DX10 / DXGI mapping:

| DXGI format | Common name | last4 |
|---|---|---|
| 71, 72 | BC1 (DXT1-equiv) | 12 |
| 74, 75 | BC2 (DXT3-equiv) | 15 |
| 77, 78 | BC3 (DXT5-equiv) | 15 |
| 80, 81 | BC4 | 4 |
| 83, 84 | BC5 | 4 |
| 95, 96 | BC6H | 4 |
| 98, 99 | BC7 | 15 |

**Three-tier resolution** (DMM's actual production path):
1. Look up the vpath in vanilla PATHC → if hit, use that exact stored value (most accurate per-file)
2. Path-prefix classifier (see §5)
3. Format-derived value from FOURCC/DXGI lookup above

dmm-parser's classifier should expose all three layers so SWISS Stacker can validate textures in the same way DMM applies them.

---

## 4. PATHC Integration (DX10/BC7 Only)

DX10/BC7 textures cannot be naively dropped into the PAZ overlay — they need:

1. **Backup vanilla `meta/0.pathc`** (one-time per mount)
2. **Register the DDS in PATHC**:
   - Build a "template record" of `pathc.header.dds_record_size` bytes from the DDS header
   - Deduplicate (existing identical templates reused)
   - Append to `pathc.dds_records`, get index
3. **Update PATHC entry**:
   - Hash the vpath (Jenkins hashlittle2, seed `0xC5EDE`)
   - Bind hash → `(template_index, mip_sizes)`
4. **Patch DDS header**:
   - Reserved1[0..4] = mip sizes from `get_dds_metadata`
   - Bytes 124..128 (last4) = three-tier resolution result

For the dmm-parser classifier (D5), we expose `requires_pathc: bool` on the metadata struct — `true` for DX10/BC7, `false` for standard DXT1/DXT5.

---

## 5. Path-Prefix Classifier

DMM's `classify_overlay_last4` (commands.rs:18937) — overrides the format-derived last4 for specific paths:

| vpath prefix / pattern | last4 (hex) | Notes |
|---|---|---|
| `/ui/*` | `0x00001580` | UI textures |
| `/character/texture/*_n.dds` | `0x00000480` | Normal maps (suffix-based) |
| `/character/texture/*tattoo*` | `0x00001380` | Tattoos / decals |
| `/character/texture/*` (default) | `0x00001280` | Generic character texture |
| (other) | None | Use format-derived value |

For dmm-parser's `infer_vpath` helper (D4), this same path table is needed to validate that an asset's vpath is recognized.

---

## 6. Mip Level Size Computation

Per `get_dds_metadata` (commands.rs:18777):

For block-compressed formats (BC1/2/3/4/5/6/7):
```
size_per_mip = max(1, (width + 3) / 4) * max(1, (height + 3) / 4) * block_bytes
```

Block bytes by FOURCC:
| FOURCC | block_bytes |
|---|---|
| `DXT1` | 8 |
| `DXT3` / `DXT5` | 16 |
| `ATI1` / `BC4*` | 8 |
| `ATI2` / `BC5*` | 16 |
| (DX10 with DXGI 71-72) | 8 |
| (DX10 with DXGI 74-99) | 16 |

For uncompressed RGB:
```
size_per_mip = ((width * bpp + 7) / 8) * height
```
(`bpp` from `pf_rgb_bits` if `pf_flags & 0x40`, else from DXGI format table)

Mip 1 width/height = max(1, prev/2). Mip 2 same. dmm-parser only needs the first 4 mip sizes for the Reserved1[0..4] patch.

---

## 7. Validation Rules (for D6 — `validate_dds_for_game`)

dmm-parser's DDS validator should reject or warn on:

- **Wrong magic**: not `b"DDS "` at offset 0 → fatal
- **Header too short**: less than 128 bytes → fatal
- **DX10 header too short**: pf_fourcc=DX10 but file < 148 bytes → fatal
- **dwDepth == 0**: game requires >= 1 → warn (auto-fixable on apply)
- **mip_map_count == 0**: game requires >= 1 → warn (auto-fixable)
- **Unknown FOURCC**: not in mapping table → warn (last4 will fall back to None)
- **DX10 with unknown DXGI format**: not in mapping table → warn
- **POW2 dimensions**: many engines require — Crimson is more permissive but still warn on non-POW2
- **Missing mips**: production DDS always has mips → warn if mip_count == 1 for non-tiny textures

---

## 8. Key DMM Functions Mapped (Reference Only — NOT Modifying DMM)

| Address | Purpose |
|---|---|
| `commands.rs:18777` | `get_dds_metadata(data)` — extract (size0, size1, size2, size3) for Reserved1 patching |
| `commands.rs:18864` | `add_dds_to_pathc(pathc, dds_path, vpath)` — full PATHC registration from file |
| `commands.rs:18897` | `add_dds_data_to_pathc(pathc, dds_data, vpath)` — same from in-memory bytes |
| `commands.rs:18921` | `lookup_overlay_class_in_pathc(pathc, vpath)` — three-tier last4 lookup, tier 1 |
| `commands.rs:18937` | `classify_overlay_last4(vpath)` — three-tier, tier 2 (path-prefix) |
| `commands.rs:18957` | `patch_dds_header_for_overlay(dds_data, idx)` — Reserved1 + last4 patch |
| `commands.rs:11534-11700` | DDS routing in mount pipeline (queue dispatch) |

---

## 9. dmm-parser Targets (D2-D8 Implementation Plan)

| Phase | Deliverable | dmm-parser code |
|---|---|---|
| D2 | Module skeleton | `src/dds/{mod,header,classify}.rs` |
| D3 | `classify(bytes) -> DdsClassification` | Parse header, return `{format, dimensions, mips, last4_format_derived, requires_pathc}` |
| D4 | `infer_vpath(path)` | Path-prefix table from §5 |
| D5 | `DdsAssetMetadata` | `{vpath_hint, format, dimensions, mip_count, sha256, requires_pathc}` for v3.1 packaging |
| D6 | `validate_dds_for_game(bytes)` | Validation rules from §7 |
| D7 | Python bindings | `dmm_parser.classify_dds`, `validate_dds`, `infer_dds_vpath` |
| D8 | Tests + docs | Vanilla samples, `docs/api.md` update |

---

## 10. Open Questions

1. **What block_bytes for DX10 BC6H/BC7?** DMM treats them as last4=4 and last4=15 respectively. Block sizes need testing — likely 16 for both.
2. **Are non-POW2 textures actually accepted in production?** Need to test or accept as warning rather than error.
3. **How does Crimson handle volume textures (depth > 1)?** Game's loader requires >= 1; whether it actually renders is unclear.
4. **PATHC dds_record_size variation** — the size of a "template record" might vary across game versions. Capture once per mount; don't hardcode.
5. **Does the game accept BC7 in non-DX10 fourcc?** Modern tools sometimes write BC7 via "BC7 " fourcc instead of DX10. Behavior unknown.

---

*End of D0 notes. Ready for D1 (hexpat pattern) and D2 (Rust skeleton).*
