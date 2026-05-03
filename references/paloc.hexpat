// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware
// Licensed under CDMTL v1.0
//
// Hex pattern for Pearl Abyss .paloc files (Crimson Desert localization tables).
// Verified format: u64 category + (u32 len + bytes) key + (u32 len + bytes) value
// Trailer: trailing u32 entry_count at end of file.
//
// Format applies to PLAIN (uncompressed/unencrypted) paloc files. Production
// paloc files use a ChaCha20+LZ4 envelope (flags=0x0032) — decrypt + decompress
// before applying this pattern.
//
// Sample file: localizationstring_eng.paloc (15.4 MB, ~172k entries)

#pragma endian little

import std.io;
import std.mem;

// ── BString — u32 length-prefixed UTF-8 (no null terminator) ──────────────

struct BString {
    u32 length;
    char data[length];
};

// ── Localization Entry ────────────────────────────────────────────────────
// Per-entry layout. The u64 category code's LOW BYTE is the meaningful part:
//   0x07 — generic localization
//   0x2F — game text (most common in localizationstring_eng.paloc)
//   0x70 — item name (matches Benreuveni's "echo key 0x70")
//   0x71 — item description (matches Benreuveni's "echo key 0x71")
//   ... other category codes for NPCs, quests, UI text, etc.
// Upper 7 bytes are always zero (padding/reserved).

struct LocalizationEntry {
    u64 category;          // low byte = category code; upper 7 bytes always 0
    BString key;           // internal identifier, e.g. "262897", "ITEM_NAME_1234567"
    BString value;         // localized text shown in-game
};

// ── File Header / Body ────────────────────────────────────────────────────
// The file is just back-to-back entries followed by a trailing u32 count.
// No header. No per-entry trailer/index (the v19 - 16*v11 line in IDA was
// memory-allocation arithmetic, not file structure).
//
// To use: read entries until offset == file_size - 4, then read the trailing
// u32 count and verify it matches the number of entries parsed.

LocalizationEntry entries[while($ < std::mem::size() - 4)] @ 0;

// Trailing entry count (last 4 bytes)
u32 entry_count @ std::mem::size() - 4;

// ── Sanity checks ─────────────────────────────────────────────────────────
//
// Run with: plcli run -i <sample>.paloc -p paloc.hexpat -v -d
// Expected output:
//   - First entry: category=0x2F, key="262897", value="Unavailable during combat."
//   - First few hundred entries should all have category in {0x07, 0x2F, 0x70, 0x71}
//   - Final entry_count should match the count of parsed entries
//
// If the count doesn't match, the file may be:
//   1. ChaCha20+LZ4-encrypted (flags=0x0032) — decrypt first
//   2. Truncated or corrupt
//   3. A different .paloc variant (untested)

// ── Format validation summary ─────────────────────────────────────────────
//
// File: localizationstring_eng.paloc
// Size: 15,464,086 bytes
// Trailing count: 0x0002A078 = 172,152 entries
// First entry: category=0x2F (47), key="262897" (6 bytes),
//              value="Unavailable during combat." (26 bytes)
// Total entry bytes used: 15,464,082 (size - 4 for trailing count)
