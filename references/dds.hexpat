// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware
// Licensed under CDMTL v1.0
//
// Hex pattern for DirectDraw Surface (DDS) files with Crimson Desert
// engine specifics. Covers the standard 124-byte header, the optional
// 20-byte DX10 extension, and pixel format identification.
//
// Crimson-specific quirks (see references/dds_notes.md):
// - Reserved1[0..4] (offsets 32-48) are repurposed by the game to store
//   mip-level sizes when the DDS is mounted via PAZ overlay
// - dwReserved2 (bytes 124-128) carries a Crimson-specific "last4"
//   format identifier that differs from the standard caps2 field
// - DX10 textures (BC7) require PATHC template registration
//
// Run: plcli run -i sample.dds -p dds.hexpat -v -d

#pragma endian little

import std.io;
import std.mem;

// ── DDS_PIXELFORMAT (32 bytes, embedded in main header) ───────────────────

bitfield DdsPixelFormatFlags {
    alpha_pixels   : 1;     // 0x00000001 — has alpha channel
    alpha          : 1;     // 0x00000002 — alpha-only
    _reserved_2    : 1;
    _reserved_3    : 1;
    fourcc         : 1;     // 0x00000004 — pf_fourcc is meaningful
    _reserved_5    : 1;
    rgb            : 1;     // 0x00000040 — uncompressed RGB, pf_rgb_bits valid
    yuv            : 9;     // (other rare flag bits)
    luminance      : 1;
    bumpdudv       : 1;
    _padding       : 14;
};

struct DdsPixelFormat {
    u32             pf_size;        // always 32
    DdsPixelFormatFlags pf_flags;
    char            pf_fourcc[4];   // "DXT1", "DXT5", "DX10", "BC4U", "ATI2", etc.
    u32             pf_rgb_bits;    // bits per pixel (uncompressed only)
    u32             pf_r_bitmask;
    u32             pf_g_bitmask;
    u32             pf_b_bitmask;
    u32             pf_a_bitmask;
};

// ── Caps (16 bytes — 4 u32 caps fields) ───────────────────────────────────

bitfield DdsCaps {
    _reserved_low  : 3;
    complex        : 1;     // 0x00000008 — has more than one surface
    _reserved_4_11 : 8;
    texture        : 1;     // 0x00001000 — texture (always set)
    _reserved_13_21: 9;
    mipmap         : 1;     // 0x00400000 — has mipmaps
    _reserved      : 9;
};

// ── Crimson-specific Reserved1 (offsets 32-48) ────────────────────────────
// Game uses these 4 u32 slots to store mip level sizes after overlay
// patching. In a vanilla on-disk DDS, they are all zero.

struct CrimsonReserved1 {
    u32  mip0_size;          // game-overwritten: mip 0 size in bytes (also = LZ4 compressed body size)
    u32  mip1_size;          // mip 1 size
    u32  mip2_size;          // mip 2 size
    u32  mip3_size;          // mip 3 size
};

// ── DDS Header (124 bytes) ────────────────────────────────────────────────

struct DdsHeader {
    char                 magic[4];                  // "DDS "
    u32                  size;                      // 124
    u32                  flags;                     // DDS_FLAGS_* bitmask
    u32                  height;
    u32                  width;
    u32                  pitch_or_linear_size;
    u32                  depth;                     // game requires >= 1
    u32                  mip_map_count;             // game requires >= 1

    // Reserved1[11] in the standard spec (44 bytes). Crimson uses [0..4]
    // for mip sizes and treats [4..11] as truly reserved (always zero).
    CrimsonReserved1     crimson_mip_sizes;         // bytes 32-48
    u32                  reserved1_unused[7];       // bytes 48-76, always 0

    DdsPixelFormat       pixel_format;              // bytes 76-108

    DdsCaps              caps1;                     // bytes 108-112
    u32                  caps2;                     // bytes 112-116 (cubemap/volume flags)
    u32                  caps3;                     // bytes 116-120 (zero)
    u32                  caps4;                     // bytes 120-124 (zero)

    // Crimson-specific override at offset 124 (the spec's `dwReserved2`).
    // Vanilla DDS has 0 here; Crimson PAZ overlays write a format-derived
    // "last4" value (see references/dds_notes.md §3 for the lookup table).
    u32                  crimson_last4;
};

// ── DX10 extension header (20 bytes, present iff pf_fourcc == "DX10") ─────

struct Dx10Header {
    u32  dxgi_format;        // DXGI format enum (e.g. 71/72=BC1, 98/99=BC7)
    u32  resource_dimension;  // 2=Texture1D, 3=Texture2D, 4=Texture3D
    u32  misc_flag;
    u32  array_size;
    u32  misc_flags2;
};

// ── File layout ───────────────────────────────────────────────────────────

DdsHeader header @ 0;

// DX10 extension is conditional; only emit when fourcc reads "DX10"
if (header.pixel_format.pf_fourcc == "DX10") {
    Dx10Header dx10 @ 128;
}

// Body bytes (mip 0, mip 1, ...) follow immediately after the header
// (and DX10 extension if present). Total body size = file size minus
// header offset. Pattern doesn't model individual mips since their size
// depends on format + dimensions (see references/dds_notes.md §6 for
// the formula).

u32 body_offset =
    (header.pixel_format.pf_fourcc == "DX10") ? 0x94 : 0x80;
char body[std::mem::size() - body_offset] @ body_offset;

// ── Sanity probes (visible in plcli -v output) ────────────────────────────

// Format-derived "last4" value the GAME would write (compare against
// header.crimson_last4 to detect already-patched-for-overlay DDS):
//   DXT1 → 12
//   DXT3 / DXT5 → 15
//   ATI1 / BC4*  → 4
//   ATI2 / BC5*  → 4
//   DX10 + DXGI in {71, 72} → 12
//   DX10 + DXGI in {74,75,77,78,98,99} → 15
//   DX10 + DXGI in {80,81,83,84,95,96} → 4
//
// Block bytes per mip (used for size = max(1,(w+3)/4) * max(1,(h+3)/4) * bb):
//   DXT1, BC4*       → 8
//   DXT3, DXT5, BC5*, BC6H?, BC7 → 16
//   (Uncompressed)   → see pf_rgb_bits
