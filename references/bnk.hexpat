// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware
// Licensed under CDMTL v1.0
//
// Hex pattern for Wwise BNK (SoundBank) files. Used by Crimson Desert
// for voice banks (per-language) and common sound effects. Format is a
// sequence of 8-byte-headered sections back-to-back from offset 0
// (no RIFF wrapper, unlike WEM).
//
// dmm-parser uses this for header-only metadata extraction — bank ID,
// version, embedded WEM index — NOT audio decoding. Mod authors use the
// Wwise authoring tool to produce valid BNKs; we just validate them.
//
// Run: plcli run -i sample.bnk -p bnk.hexpat -v -d
//
// Verified against real Crimson Desert BNK:
//   2498340951.bnk (135 MB, bank_version=150, 3 embedded WEMs)

#pragma endian little

import std.io;
import std.mem;

// ── Section envelope ──────────────────────────────────────────────────────
// Common to all BNK sections. Section ID is a 4-character ASCII tag,
// payload size is the body byte count NOT including this 8-byte header.

// ── BKHD (Bank Header) — required, always first ───────────────────────────

struct BkhdPayload {
    u32  bank_version;        // 150 = Crimson-era Wwise build
    u32  bank_id;              // unique numeric ID (matches the file's name)
    u32  language_id;          // Wwise language hash (0 = SFX/no-language)
    u32  feedback_in_bank;     // boolean-ish, 0 typically
    u32  project_id;           // Wwise project hash
    // Padding to fill section_size (varies by version; ~36-40 bytes total)
    u8   padding[28];
};

// ── DIDX (Data Index) — embedded WEM index ────────────────────────────────

struct DidxEntry {
    u32  wem_id;               // Wwise-hashed WEM identifier
    u32  wem_offset;           // offset INTO the DATA section's payload
    u32  wem_size;             // bytes
};

// ── Generic section + payload dispatch ────────────────────────────────────

struct BnkSection {
    char  id[4];               // "BKHD", "DIDX", "DATA", "HIRC", "STID", ...
    u32   size;                // payload size in bytes (NOT including this header)

    // Payload — pattern dispatches on id for the structured ones.
    if (id == "BKHD") {
        BkhdPayload header;
    } else if (id == "DIDX") {
        // DIDX payload is N × 12-byte DidxEntry records
        DidxEntry entries[size / 12];
    } else {
        // DATA / HIRC / STID / unknown — leave as opaque bytes for now
        // (HIRC can be unpacked further but is verbose; v1 of the
        // dmm-parser BNK module only inspects section structure, not
        // HIRC contents)
        u8 raw[size];
    }
};

// ── File layout ───────────────────────────────────────────────────────────

BnkSection sections[while(!std::mem::eof())] @ 0;

// ── Section reference (verified order in 2498340951.bnk) ──────────────────
//
// BKHD (52 bytes)        bank_version=150, bank_id=2498340951
// DIDX (36 bytes)        3 entries × 12 bytes = 3 embedded WEMs
// DATA (135,321,456 b)   raw WEM payload referenced by DIDX entries
// HIRC (varies)          event/sound/action graph (optional)
// STID (varies)          referenced bank IDs (optional)

// ── Validation hooks (matched in dmm-parser's validate.rs at A7) ──────────
//
// Fatal:
// - File starts with "BKHD"                                 (bad_bnk_magic)
// - BKHD payload >= 8 bytes (must hold version + bank_id)   (bnk_header_truncated)
// - All section sizes within file bounds                    (bnk_section_overflow)
//
// Warning:
// - bank_version != 150 (Crimson-era)                       (bnk_unknown_version)
// - DIDX without DATA section                               (bnk_didx_without_data)
// - DIDX entry's offset+size > DATA payload size            (bnk_didx_offset_oob)
//
// Info:
// - HIRC section present (informational)                    (bnk_has_hirc)
// - STID section lists N referenced banks                   (bnk_referenced_banks)

// ── DIDX → DATA cross-reference (informational, not encoded in pattern) ──
//
// Pattern doesn't navigate the DIDX→DATA pointers because the DATA
// section's payload is opaque RIFF-WAVE blobs that need their own
// recursive descent into wem.hexpat. dmm-parser's BnkBank parser
// handles the cross-reference programmatically.
