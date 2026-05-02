// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! BNK (Wwise SoundBank) parser. Section-header walker + BKHD bank ID
//! extraction + DIDX embedded WEM index.
//!
//! Stub of the A5 implementation. Module skeleton only — full parser
//! lands in A5. This file provides the public types so dispatch +
//! Python bindings can reference them now.

use std::io;

/// Single DIDX entry — describes one embedded WEM inside the BNK's
/// DATA section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DidxEntry {
    pub wem_id: u32,
    /// Offset INTO the DATA section's payload (not absolute file offset).
    pub wem_offset: u32,
    pub wem_size: u32,
}

/// One BNK section header + payload range. The payload is referenced
/// by file offset, not copied — keeps parsing cheap on multi-MB BNKs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BnkSection {
    /// 4-character section ID: "BKHD" / "DIDX" / "DATA" / "HIRC" / "STID" / ...
    pub id: [u8; 4],
    /// Absolute file offset where this section's 8-byte header starts.
    pub header_offset: u64,
    /// Section payload size in bytes (NOT including the 8-byte header).
    pub size: u32,
}

impl BnkSection {
    pub fn id_str(&self) -> &str {
        std::str::from_utf8(&self.id).unwrap_or("????")
    }
}

/// Header-only view of a BNK file. DATA payload is NOT loaded — DIDX
/// entries point into it for callers that need the actual WEM bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct BnkBank {
    pub file_size: u64,
    pub bank_version: u32,
    pub bank_id: u32,
    pub sections: Vec<BnkSection>,
    pub embedded_wems: Vec<DidxEntry>,
    /// Absolute file offset of the DATA section's payload start. None
    /// if the BNK has no DATA section (i.e. event/HIRC-only banks).
    pub data_payload_offset: Option<u64>,
    /// True when the BNK contains an HIRC section. Modders typically
    /// don't author HIRC; presence is informational only.
    pub has_hirc: bool,
}

/// Parse a BNK file's section structure + bank header from raw bytes.
///
/// Walks the section list (no RIFF wrapper — sections start at offset 0),
/// extracts BKHD's bank_version + bank_id, parses DIDX into DidxEntry
/// records, locates DATA payload start, flags HIRC presence. Does NOT
/// decompose HIRC contents — modders ship whole BNKs and we just
/// validate structure.
pub fn parse_bnk(data: &[u8]) -> io::Result<BnkBank> {
    if data.len() < 8 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("BNK file is {} bytes; need at least 8 for first section header", data.len()),
        ));
    }
    if &data[0..4] != b"BKHD" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Not a BNK: first section id = {:?}, expected BKHD", &data[0..4]),
        ));
    }

    let mut sections: Vec<BnkSection> = Vec::new();
    let mut bank_version: Option<u32> = None;
    let mut bank_id: Option<u32> = None;
    let mut embedded_wems: Vec<DidxEntry> = Vec::new();
    let mut data_payload_offset: Option<u64> = None;
    let mut has_hirc = false;

    let mut off: usize = 0;
    while off + 8 <= data.len() {
        let id: [u8; 4] = data[off..off + 4].try_into().unwrap();
        let size = u32::from_le_bytes(data[off + 4..off + 8].try_into().unwrap()) as usize;
        let payload_off = off + 8;
        let payload_end = payload_off.checked_add(size).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("BNK section size overflow at offset 0x{:x}", off),
            )
        })?;
        if payload_end > data.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!(
                    "BNK section {:?} at offset 0x{:x} extends beyond file end (size={})",
                    id, off, size,
                ),
            ));
        }

        sections.push(BnkSection {
            id,
            header_offset: off as u64,
            size: size as u32,
        });

        match &id {
            b"BKHD" => {
                if size < 8 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("BKHD payload too small ({} bytes; need >= 8)", size),
                    ));
                }
                bank_version = Some(u32::from_le_bytes(
                    data[payload_off..payload_off + 4].try_into().unwrap(),
                ));
                bank_id = Some(u32::from_le_bytes(
                    data[payload_off + 4..payload_off + 8].try_into().unwrap(),
                ));
            }
            b"DIDX" => {
                if size % 12 != 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("DIDX size {} not divisible by 12 (entry size)", size),
                    ));
                }
                let count = size / 12;
                embedded_wems.reserve(count);
                for i in 0..count {
                    let e = payload_off + i * 12;
                    embedded_wems.push(DidxEntry {
                        wem_id: u32::from_le_bytes(data[e..e + 4].try_into().unwrap()),
                        wem_offset: u32::from_le_bytes(data[e + 4..e + 8].try_into().unwrap()),
                        wem_size: u32::from_le_bytes(data[e + 8..e + 12].try_into().unwrap()),
                    });
                }
            }
            b"DATA" => {
                data_payload_offset = Some(payload_off as u64);
            }
            b"HIRC" => {
                has_hirc = true;
            }
            _ => {
                // STID and other unknown sections — leave as opaque.
            }
        }

        off = payload_end;
    }

    let bank_version = bank_version.ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "BNK missing BKHD section")
    })?;
    let bank_id = bank_id.unwrap_or(0);

    Ok(BnkBank {
        file_size: data.len() as u64,
        bank_version,
        bank_id,
        sections,
        embedded_wems,
        data_payload_offset,
        has_hirc,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a minimal valid BNK with optional DIDX/DATA/HIRC sections.
    fn make_bnk(
        bank_version: u32,
        bank_id: u32,
        didx_entries: &[DidxEntry],
        data_payload: &[u8],
        with_hirc: bool,
    ) -> Vec<u8> {
        let mut buf = Vec::new();

        // BKHD: version + bank_id + 36 bytes of padding (matches Crimson's 52-byte BKHD)
        let mut bkhd = Vec::new();
        bkhd.extend_from_slice(&bank_version.to_le_bytes());
        bkhd.extend_from_slice(&bank_id.to_le_bytes());
        bkhd.resize(52, 0); // pad to 52 bytes

        buf.extend_from_slice(b"BKHD");
        buf.extend_from_slice(&(bkhd.len() as u32).to_le_bytes());
        buf.extend_from_slice(&bkhd);

        // DIDX
        if !didx_entries.is_empty() {
            buf.extend_from_slice(b"DIDX");
            let didx_size = didx_entries.len() * 12;
            buf.extend_from_slice(&(didx_size as u32).to_le_bytes());
            for e in didx_entries {
                buf.extend_from_slice(&e.wem_id.to_le_bytes());
                buf.extend_from_slice(&e.wem_offset.to_le_bytes());
                buf.extend_from_slice(&e.wem_size.to_le_bytes());
            }
        }

        // DATA
        if !data_payload.is_empty() {
            buf.extend_from_slice(b"DATA");
            buf.extend_from_slice(&(data_payload.len() as u32).to_le_bytes());
            buf.extend_from_slice(data_payload);
        }

        // HIRC (empty payload, just marker)
        if with_hirc {
            buf.extend_from_slice(b"HIRC");
            buf.extend_from_slice(&0u32.to_le_bytes());
        }

        buf
    }

    #[test]
    fn parse_minimal_bnk() {
        let buf = make_bnk(150, 12345, &[], &[], false);
        let bnk = parse_bnk(&buf).unwrap();
        assert_eq!(bnk.bank_version, 150);
        assert_eq!(bnk.bank_id, 12345);
        assert_eq!(bnk.sections.len(), 1);
        assert_eq!(&bnk.sections[0].id, b"BKHD");
        assert!(bnk.embedded_wems.is_empty());
        assert!(bnk.data_payload_offset.is_none());
        assert!(!bnk.has_hirc);
    }

    #[test]
    fn parse_bnk_with_didx_data_hirc() {
        let entries = [
            DidxEntry { wem_id: 100, wem_offset: 0, wem_size: 256 },
            DidxEntry { wem_id: 200, wem_offset: 256, wem_size: 512 },
            DidxEntry { wem_id: 300, wem_offset: 768, wem_size: 128 },
        ];
        let data_payload = vec![0x55u8; 1024];
        let buf = make_bnk(150, 99999, &entries, &data_payload, true);
        let bnk = parse_bnk(&buf).unwrap();
        assert_eq!(bnk.bank_version, 150);
        assert_eq!(bnk.bank_id, 99999);
        assert_eq!(bnk.embedded_wems.len(), 3);
        assert_eq!(bnk.embedded_wems[0].wem_id, 100);
        assert_eq!(bnk.embedded_wems[1].wem_size, 512);
        assert_eq!(bnk.embedded_wems[2].wem_offset, 768);
        assert!(bnk.data_payload_offset.is_some());
        assert!(bnk.has_hirc);
        // Sections in order: BKHD, DIDX, DATA, HIRC
        assert_eq!(bnk.sections.len(), 4);
        assert_eq!(&bnk.sections[0].id, b"BKHD");
        assert_eq!(&bnk.sections[1].id, b"DIDX");
        assert_eq!(&bnk.sections[2].id, b"DATA");
        assert_eq!(&bnk.sections[3].id, b"HIRC");
    }

    #[test]
    fn rejects_bad_first_section() {
        let mut buf = make_bnk(150, 1, &[], &[], false);
        buf[0..4].copy_from_slice(b"XXXX");
        let err = parse_bnk(&buf).unwrap_err();
        assert!(err.to_string().contains("expected BKHD"));
    }

    #[test]
    fn rejects_truncated() {
        let buf = vec![b'B', b'K', b'H', b'D', 0, 0];
        let err = parse_bnk(&buf).unwrap_err();
        assert!(err.to_string().contains("at least 8"));
    }

    #[test]
    fn rejects_didx_size_not_multiple_of_12() {
        let mut buf = make_bnk(150, 1, &[], &[], false);
        // Append a malformed DIDX section
        buf.extend_from_slice(b"DIDX");
        buf.extend_from_slice(&13u32.to_le_bytes()); // not divisible by 12
        buf.extend_from_slice(&[0u8; 13]);
        let err = parse_bnk(&buf).unwrap_err();
        assert!(err.to_string().contains("not divisible by 12"));
    }

    #[test]
    fn rejects_section_extending_past_eof() {
        let mut buf = make_bnk(150, 1, &[], &[], false);
        // Append a section claiming size larger than remaining bytes
        buf.extend_from_slice(b"DATA");
        buf.extend_from_slice(&999999u32.to_le_bytes());
        let err = parse_bnk(&buf).unwrap_err();
        assert!(err.to_string().contains("extends beyond file end"));
    }

    #[test]
    fn parse_real_vanilla_bnk_sample() {
        // Real BNK sample from DMM backups — bank_version=150, 3 embedded WEMs.
        let path = "C:/Users/corin/Desktop/CD JSON Mod Manager/Definitive Mod Manager/src-tauri/target/debug/backups/2498340951.bnk";
        let Ok(bytes) = std::fs::read(path) else {
            eprintln!("SKIP: real BNK sample not found");
            return;
        };
        let bnk = parse_bnk(&bytes).unwrap();
        assert_eq!(bnk.bank_version, 150);
        assert_eq!(bnk.bank_id, 2498340951);
        assert_eq!(bnk.embedded_wems.len(), 3);
        assert!(bnk.data_payload_offset.is_some());
    }
}
