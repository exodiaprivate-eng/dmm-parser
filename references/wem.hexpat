// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware
// Licensed under CDMTL v1.0
//
// Hex pattern for Wwise WEM (Encoded Media) audio files. Format is a
// standard RIFF-WAVE wrapper plus Wwise-specific chunks (`hash`, `junk`).
//
// dmm-parser uses this for header-only metadata extraction — codec
// identification, channel count, sample rate, payload size — NOT audio
// decoding. Mod authors use external tools (Wwise authoring, vgmstream)
// to produce valid WEMs; we just validate them.
//
// Run: plcli run -i sample.wem -p wem.hexpat -v -d
//
// Verified against real Crimson Desert WEMs:
//   1045272379.wem (433 KB, format_tag=0xFFFF Wwise Vorbis)
//   113958244.wem  (62 MB, format_tag=0xFFFE PCM extensible)

#pragma endian little

import std.io;
import std.mem;

// ── WAVEFORMATEX-style fmt chunk payload ──────────────────────────────────

struct WaveFormatEx {
    u16  format_tag;          // 0xFFFE = WaveformatExtensible (PCM-style)
                               // 0xFFFF = Wwise Vorbis (most common)
                               // (other Wwise-specific values exist)
    u16  channels;             // 1=mono, 2=stereo, ...
    u32  sample_rate;          // 44100, 48000 typical
    u32  byte_rate;            // 0 for compressed formats
    u16  block_align;          // 0 for compressed
    u16  bits_per_sample;      // 0 for compressed; 16 for PCM
    u16  cb_size;              // size of extension data (0 for plain WAVEFORMATEX)
    // Followed by `cb_size` bytes of WAVEFORMATEXTENSIBLE / Wwise-specific
    // extension data when cb_size > 0.
};

// ── RIFF chunk envelope ───────────────────────────────────────────────────

struct RiffChunk {
    char  id[4];               // "fmt ", "hash", "data", "junk", ...
    u32   size;                // payload size in bytes (NOT including this 8-byte header)
    u8    payload[size];

    // RIFF chunks pad to even byte alignment. plcli pattern doesn't model
    // the padding byte explicitly — the next chunk just starts at the
    // even-aligned offset.
};

// ── File layout ───────────────────────────────────────────────────────────

struct WemFile {
    char  riff_magic[4];       // "RIFF"
    u32   riff_size;           // file_size - 8
    char  wave_id[4];          // "WAVE"
    RiffChunk chunks[while(!std::mem::eof())];
};

WemFile wem @ 0;

// ── Crimson-specific Wwise chunks (informational) ─────────────────────────
//
// `fmt `  required, always first per Wwise convention
//          → contains WaveFormatEx (codec metadata, no audio data)
//
// `hash`  REQUIRED for Wwise WEMs (16 bytes)
//          → content hash used by Wwise build pipeline; presence is a
//            strong fingerprint that this is a Wwise-encoded WEM
//            rather than a raw RIFF-WAVE
//
// `junk`  optional, 12 bytes
//          → Wwise version-specific padding/filler
//
// `data`  required, last
//          → encoded audio bytes (Vorbis or PCM)
//
// Other chunks may appear in older Wwise builds; loaders should skip
// unknown chunk IDs gracefully.

// ── Sanity probes ─────────────────────────────────────────────────────────
//
// Validation hooks (matched in dmm-parser's validate.rs at A7):
// - riff_magic must be "RIFF"
// - wave_id must be "WAVE"
// - At least one chunk with id="fmt " (WaveFormatEx parses cleanly)
// - At least one chunk with id="data"
// - format_tag in {0xFFFE, 0xFFFF} for Crimson Wwise builds
// - sample_rate in 8000..=96000
// - channels in 1..=8
//
// Format tag reference:
//   0x0001 = WAVE_FORMAT_PCM             (raw PCM)
//   0xFFFE = WAVE_FORMAT_EXTENSIBLE      (extended PCM with channel mask)
//   0xFFFF = Wwise-internal Vorbis        (Wwise's custom Vorbis variant)
