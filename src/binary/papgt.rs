// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

use std::io::{self, Write};

use super::{BinaryRead, BinaryWrite, check_remaining};
#[allow(unused_imports)]
use super::trie::read_cstring;
use crate::binary_struct;
use crate::crypto::checksum;

// i32 is used for group_names_buffer length prefix

// ── Language flags ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LanguageType(pub u16);

#[allow(dead_code)]
impl LanguageType {
    pub const KOR: u16 = 1 << 0;
    pub const ENG: u16 = 1 << 1;
    pub const JPN: u16 = 1 << 2;
    pub const RUS: u16 = 1 << 3;
    pub const TUR: u16 = 1 << 4;
    pub const SPA_ES: u16 = 1 << 5;
    pub const SPA_MX: u16 = 1 << 6;
    pub const FRE: u16 = 1 << 7;
    pub const GER: u16 = 1 << 8;
    pub const ITA: u16 = 1 << 9;
    pub const POL: u16 = 1 << 10;
    pub const POR_BR: u16 = 1 << 11;
    pub const ZHO_TW: u16 = 1 << 12;
    pub const ZHO_CN: u16 = 1 << 13;
    pub const ALL: u16 = 0x3FFF;
}

impl<'a> BinaryRead<'a> for LanguageType {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        Ok(LanguageType(u16::read_from(data, offset)?))
    }
}

impl BinaryWrite for LanguageType {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.0.write_to(w)
    }
}

// ── Structs ────────────────────────────────────────────────────────────────

binary_struct! {
    pub struct PackGroupTreeMetaHeader {
        pub unknown0: u32,
        pub checksum: u32,
        pub entry_count: u8,
        pub unknown1: u8,
        pub unknown2: u16,
    }
}

binary_struct! {
    pub struct PackGroupTreeMetaEntry {
        pub is_optional: u8,
        pub language: LanguageType,
        pub always_zero: u8,
        pub group_name_offset: u32,
        pub pack_meta_checksum: u32,
    }
}

// ── Resolved entry (with group name string) ────────────────────────────────

#[derive(Debug)]
pub struct ResolvedEntry {
    pub group_name: String,
    pub entry: PackGroupTreeMetaEntry,
}

// ── Top-level PAPGT ────────────────────────────────────────────────────────

/// 2.04.00 on: the PAPGT carries no checksum in its header and ends in a 64-byte Ed25519
/// signature over everything before it. The game checks the signature with a key compiled
/// into the exe and refuses to start when it fails, so a changed signed PAPGT cannot be
/// written: `to_bytes` returns the original bytes or an error.
#[derive(Debug, Clone)]
pub struct SignedPapgt {
    /// The first two header bytes (`81 16` on 2.04.00), meaning unknown, kept as read.
    pub prefix: u16,
    pub signature: [u8; 64],
    /// The signed bytes as read (everything but the signature).
    pub signed_body: Vec<u8>,
}

#[derive(Debug)]
pub struct PackGroupTreeMeta {
    pub header: PackGroupTreeMetaHeader,
    pub entries: Vec<ResolvedEntry>,
    /// Raw group names buffer (for roundtrip writing).
    pub group_names_buffer: Vec<u8>,
    /// `Some` for the signed 2.04.00 layout.
    pub signed: Option<SignedPapgt>,
}

impl Default for PackGroupTreeMeta {
    fn default() -> Self {
        Self::new()
    }
}

impl PackGroupTreeMeta {
    /// Create a new empty PAPGT.
    #[allow(dead_code)]
    pub fn new() -> Self {
        PackGroupTreeMeta {
            header: PackGroupTreeMetaHeader {
                unknown0: 0,
                checksum: 0,
                entry_count: 0,
                unknown1: 0,
                unknown2: 0,
            },
            entries: Vec::new(),
            group_names_buffer: Vec::new(),
            signed: None,
        }
    }

    /// True for the signed 2.04.00 layout: a 4-byte header (u16, entry count, u8), the
    /// entries, the names and exactly 64 bytes of signature after them.
    fn is_signed_layout(data: &[u8]) -> bool {
        if data.len() < 4 + 4 + 64 {
            return false;
        }
        let count = data[2] as usize;
        let names_at = 4 + 12 * count;
        if data.len() < names_at + 4 {
            return false;
        }
        let names_len = i32::from_le_bytes(data[names_at..names_at + 4].try_into().unwrap());
        names_len >= 0 && names_at + 4 + names_len as usize + 64 == data.len()
    }

    /// Parse a PAPGT from raw bytes (the entire file contents).
    pub fn parse(data: &[u8]) -> io::Result<Self> {
        let mut offset = 0;
        let (header, signed) = if Self::is_signed_layout(data) {
            offset = 4;
            let body_len = data.len() - 64;
            let signed = SignedPapgt {
                prefix: u16::from_le_bytes([data[0], data[1]]),
                signature: data[body_len..].try_into().unwrap(),
                signed_body: data[..body_len].to_vec(),
            };
            let header = PackGroupTreeMetaHeader {
                unknown0: 0,
                checksum: 0,
                entry_count: data[2],
                unknown1: data[3],
                unknown2: 0,
            };
            (header, Some(signed))
        } else {
            let header = PackGroupTreeMetaHeader::read_from(data, &mut offset)?;

            // Everything after the header is checksummed
            let post_header_data = &data[offset..];
            checksum::validate_checksum(post_header_data, header.checksum)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            (header, None)
        };

        // Read entries
        let mut raw_entries = Vec::with_capacity(header.entry_count as usize);
        for _ in 0..header.entry_count {
            raw_entries.push(PackGroupTreeMetaEntry::read_from(data, &mut offset)?);
        }

        // Read group names buffer: i32 length prefix + data
        check_remaining(data, offset, 4)?;
        let buf_len = i32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;
        check_remaining(data, offset, buf_len)?;
        let group_names_buffer = data[offset..offset + buf_len].to_vec();

        // Resolve group names
        let mut entries = Vec::with_capacity(raw_entries.len());
        for entry in raw_entries {
            let group_name = read_cstring(&group_names_buffer, entry.group_name_offset as usize)?;
            entries.push(ResolvedEntry { group_name, entry });
        }

        Ok(PackGroupTreeMeta {
            header,
            entries,
            group_names_buffer,
            signed,
        })
    }

    /// Add or update an entry in this PAPGT (upsert), placed at the front.
    ///
    /// Mod entries are inserted at position 0 so they take priority over
    /// original game entries (matches other mod loaders' behavior).
    /// If an entry with the same `group_name` already exists, it is updated
    /// and moved to the front.
    /// Call `to_bytes()` afterwards to get the serialized form with recalculated checksum.
    pub fn add_entry(
        &mut self,
        group_name: &str,
        pack_meta_checksum: u32,
        is_optional: u8,
        language: u16,
    ) {
        // Remove existing entry with the same group name (if any)
        self.entries.retain(|e| e.group_name != group_name);

        // Prepend the group name to the front of the names buffer
        let name_bytes = group_name.as_bytes();
        let insert_len = name_bytes.len() + 1; // +1 for null terminator
        let mut new_buffer = Vec::with_capacity(insert_len + self.group_names_buffer.len());
        new_buffer.extend_from_slice(name_bytes);
        new_buffer.push(0); // null terminator
        new_buffer.extend_from_slice(&self.group_names_buffer);
        self.group_names_buffer = new_buffer;

        // Shift all existing entries' group_name_offset forward
        for entry in &mut self.entries {
            entry.entry.group_name_offset += insert_len as u32;
        }

        self.entries.insert(0, ResolvedEntry {
            group_name: group_name.to_string(),
            entry: PackGroupTreeMetaEntry {
                is_optional,
                language: LanguageType(language),
                always_zero: 0,
                group_name_offset: 0, // at the front of the buffer
                pack_meta_checksum,
            },
        });

        self.header.entry_count = self.entries.len() as u8;
    }

    /// Serialize back to bytes (roundtrip).
    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        // First, serialize the post-header data
        let mut post_header = Vec::new();

        for resolved in &self.entries {
            resolved.entry.write_to(&mut post_header)?;
        }

        // Write group names buffer with i32 length prefix
        (self.group_names_buffer.len() as i32).write_to(&mut post_header)?;
        post_header.write_all(&self.group_names_buffer)?;

        if let Some(signed) = &self.signed {
            let mut body = Vec::with_capacity(4 + post_header.len());
            body.write_all(&signed.prefix.to_le_bytes())?;
            body.push(self.entries.len() as u8);
            body.push(self.header.unknown1);
            body.write_all(&post_header)?;
            if body != signed.signed_body {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "this PAPGT is signed (game 2.04.00 or later): a changed copy fails the game's \
                     signature check and the game will not start, so it is not written",
                ));
            }
            body.write_all(&signed.signature)?;
            return Ok(body);
        }

        // Now build the full output with recalculated checksum
        let computed_checksum = checksum::calculate_checksum(&post_header);

        let mut out = Vec::new();
        // Write header with updated checksum
        let header = PackGroupTreeMetaHeader {
            checksum: computed_checksum,
            ..PackGroupTreeMetaHeader {
                unknown0: self.header.unknown0,
                checksum: computed_checksum,
                entry_count: self.header.entry_count,
                unknown1: self.header.unknown1,
                unknown2: self.header.unknown2,
            }
        };
        header.write_to(&mut out)?;
        out.write_all(&post_header)?;

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A signed 2.04.00-shaped PAPGT: 4-byte header, two entries, names, 64-byte signature.
    fn signed_papgt() -> Vec<u8> {
        let mut b = vec![0x81, 0x16, 2, 2];
        for (name_off, ck) in [(0u32, 0x1111_1111u32), (5, 0x2222_2222)] {
            b.extend_from_slice(&[0, 0xFF, 0x7F, 0]);
            b.extend_from_slice(&name_off.to_le_bytes());
            b.extend_from_slice(&ck.to_le_bytes());
        }
        let names = b"0000\x000001\x00";
        b.extend_from_slice(&(names.len() as i32).to_le_bytes());
        b.extend_from_slice(names);
        b.extend_from_slice(&[0xAB; 64]);
        b
    }

    #[test]
    fn a_signed_papgt_parses_and_writes_back_unchanged() {
        let data = signed_papgt();
        let papgt = PackGroupTreeMeta::parse(&data).unwrap();
        assert!(papgt.signed.is_some());
        let names: Vec<&str> = papgt.entries.iter().map(|e| e.group_name.as_str()).collect();
        assert_eq!(names, ["0000", "0001"]);
        assert_eq!(papgt.entries[1].entry.pack_meta_checksum, 0x2222_2222);
        assert_eq!(papgt.to_bytes().unwrap(), data);
    }

    #[test]
    fn a_changed_signed_papgt_is_never_written() {
        let mut papgt = PackGroupTreeMeta::parse(&signed_papgt()).unwrap();
        papgt.entries[0].entry.pack_meta_checksum = 0x3333_3333;
        assert!(papgt.to_bytes().is_err(), "a changed checksum must not be written");

        let mut papgt = PackGroupTreeMeta::parse(&signed_papgt()).unwrap();
        papgt.add_entry("0036", 0x4444_4444, 0, LanguageType::ALL);
        assert!(papgt.to_bytes().is_err(), "an added group must not be written");
    }
}
