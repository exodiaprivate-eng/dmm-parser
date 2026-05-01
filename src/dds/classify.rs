// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! DDS format classification — maps FOURCC + DXGI to a clean enum and
//! computes per-mip pixel-data size + Crimson "last4" identifier.
//!
//! Stub of the full D3 implementation; D3 will fill in mip-size
//! computation and the full validation matrix.

use std::io;

use super::header::{DdsHeader, Dx10Header};

/// Logical pixel format detected from FOURCC + DXGI extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DdsFormat {
    Dxt1,
    Dxt3,
    Dxt5,
    Bc4Unorm,
    Bc4Snorm,
    Bc5Unorm,
    Bc5Snorm,
    Bc6hUf16,
    Bc6hSf16,
    Bc7Unorm,
    UncompressedRgb,
    Unknown,
}

impl DdsFormat {
    /// Crimson-specific "last4" identifier (dwReserved2 at byte 124) the
    /// game expects for an overlay-patched DDS in this format.
    /// Returns None for formats without a known mapping.
    pub fn crimson_last4(self) -> Option<u32> {
        match self {
            DdsFormat::Dxt1 => Some(12),
            DdsFormat::Dxt3 | DdsFormat::Dxt5 | DdsFormat::Bc7Unorm => Some(15),
            DdsFormat::Bc4Unorm | DdsFormat::Bc4Snorm
            | DdsFormat::Bc5Unorm | DdsFormat::Bc5Snorm
            | DdsFormat::Bc6hUf16 | DdsFormat::Bc6hSf16 => Some(4),
            DdsFormat::UncompressedRgb | DdsFormat::Unknown => None,
        }
    }

    /// Bytes per 4x4 block (block-compressed formats only).
    /// Returns None for uncompressed formats.
    pub fn block_bytes(self) -> Option<u32> {
        match self {
            DdsFormat::Dxt1 | DdsFormat::Bc4Unorm | DdsFormat::Bc4Snorm => Some(8),
            DdsFormat::Dxt3 | DdsFormat::Dxt5
            | DdsFormat::Bc5Unorm | DdsFormat::Bc5Snorm
            | DdsFormat::Bc6hUf16 | DdsFormat::Bc6hSf16
            | DdsFormat::Bc7Unorm => Some(16),
            DdsFormat::UncompressedRgb | DdsFormat::Unknown => None,
        }
    }

    /// True if this format requires DX10 PATHC template registration
    /// when mounted as a Crimson overlay.
    pub fn requires_pathc(self) -> bool {
        matches!(
            self,
            DdsFormat::Bc7Unorm
            | DdsFormat::Bc6hUf16
            | DdsFormat::Bc6hSf16
        )
    }
}

/// Classification result combining format + dimensions + Crimson hints.
#[derive(Debug, Clone, PartialEq)]
pub struct DdsClassification {
    pub format: DdsFormat,
    pub width: u32,
    pub height: u32,
    pub mip_count: u32,
    pub depth: u32,
    /// True if FOURCC == "DX10" (DXGI extension present).
    pub is_dx10: bool,
    /// DXGI format value when `is_dx10`, else None.
    pub dxgi_format: Option<u32>,
    /// Crimson last4 the game would write for this format. None if
    /// unmapped.
    pub crimson_last4: Option<u32>,
    /// True if Crimson PATHC template registration is needed at mount time.
    pub requires_pathc: bool,
}

/// Classify a DDS file's format, dimensions, and Crimson-specific hints
/// from the raw bytes. Returns Err if the header is malformed.
pub fn classify(data: &[u8]) -> io::Result<DdsClassification> {
    let header = DdsHeader::parse(data)?;
    let format = format_from_header(&header, data)?;
    let dxgi_format = if header.is_dx10() {
        Some(Dx10Header::parse(data)?.dxgi_format)
    } else {
        None
    };

    Ok(DdsClassification {
        format,
        width: header.width,
        height: header.height,
        mip_count: header.mip_map_count.max(1),
        depth: header.depth.max(1),
        is_dx10: header.is_dx10(),
        dxgi_format,
        crimson_last4: format.crimson_last4(),
        requires_pathc: format.requires_pathc(),
    })
}

fn format_from_header(header: &DdsHeader, data: &[u8]) -> io::Result<DdsFormat> {
    let fourcc = &header.pixel_format.pf_fourcc;
    let format = match fourcc {
        b"DXT1" => DdsFormat::Dxt1,
        b"DXT3" => DdsFormat::Dxt3,
        b"DXT5" => DdsFormat::Dxt5,
        b"ATI1" | b"BC4U" => DdsFormat::Bc4Unorm,
        b"BC4S" => DdsFormat::Bc4Snorm,
        b"ATI2" | b"BC5U" => DdsFormat::Bc5Unorm,
        b"BC5S" => DdsFormat::Bc5Snorm,
        b"DX10" => format_from_dxgi(data)?,
        _ => {
            // Uncompressed RGB if pf_flags has the RGB bit (0x40).
            if (header.pixel_format.pf_flags & 0x40) != 0 {
                DdsFormat::UncompressedRgb
            } else {
                DdsFormat::Unknown
            }
        }
    };
    Ok(format)
}

fn format_from_dxgi(data: &[u8]) -> io::Result<DdsFormat> {
    let dx10 = Dx10Header::parse(data)?;
    Ok(match dx10.dxgi_format {
        71 | 72 => DdsFormat::Dxt1,        // BC1_UNORM / _SRGB
        74 | 75 => DdsFormat::Dxt3,        // BC2_UNORM / _SRGB
        77 | 78 => DdsFormat::Dxt5,        // BC3_UNORM / _SRGB
        80 => DdsFormat::Bc4Unorm,
        81 => DdsFormat::Bc4Snorm,
        83 => DdsFormat::Bc5Unorm,
        84 => DdsFormat::Bc5Snorm,
        95 => DdsFormat::Bc6hUf16,
        96 => DdsFormat::Bc6hSf16,
        98 | 99 => DdsFormat::Bc7Unorm,    // BC7_UNORM / _SRGB
        _ => DdsFormat::Unknown,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dds::header::DDS_FILE_HEADER_SIZE;

    fn make_dds(fourcc: &[u8; 4], width: u32, height: u32) -> Vec<u8> {
        let mut buf = vec![0u8; DDS_FILE_HEADER_SIZE];
        buf[0..4].copy_from_slice(b"DDS ");
        buf[4..8].copy_from_slice(&124u32.to_le_bytes());
        buf[12..16].copy_from_slice(&height.to_le_bytes());
        buf[16..20].copy_from_slice(&width.to_le_bytes());
        buf[24..28].copy_from_slice(&1u32.to_le_bytes());
        buf[28..32].copy_from_slice(&4u32.to_le_bytes());
        buf[76..80].copy_from_slice(&32u32.to_le_bytes());
        buf[80..84].copy_from_slice(&4u32.to_le_bytes());
        buf[84..88].copy_from_slice(fourcc);
        buf
    }

    #[test]
    fn classify_dxt5() {
        let buf = make_dds(b"DXT5", 256, 256);
        let c = classify(&buf).unwrap();
        assert_eq!(c.format, DdsFormat::Dxt5);
        assert_eq!(c.width, 256);
        assert_eq!(c.height, 256);
        assert_eq!(c.crimson_last4, Some(15));
        assert!(!c.requires_pathc);
        assert!(!c.is_dx10);
    }

    #[test]
    fn classify_dxt1() {
        let buf = make_dds(b"DXT1", 64, 64);
        let c = classify(&buf).unwrap();
        assert_eq!(c.format, DdsFormat::Dxt1);
        assert_eq!(c.crimson_last4, Some(12));
        assert_eq!(c.format.block_bytes(), Some(8));
    }

    #[test]
    fn classify_dx10_bc7() {
        let mut buf = make_dds(b"DX10", 512, 512);
        buf.resize(148, 0);
        buf[128..132].copy_from_slice(&98u32.to_le_bytes()); // BC7_UNORM
        let c = classify(&buf).unwrap();
        assert_eq!(c.format, DdsFormat::Bc7Unorm);
        assert_eq!(c.crimson_last4, Some(15));
        assert!(c.requires_pathc, "BC7 should require PATHC template registration");
        assert!(c.is_dx10);
        assert_eq!(c.dxgi_format, Some(98));
    }

    #[test]
    fn classify_unknown_fourcc() {
        let buf = make_dds(b"WTF!", 64, 64);
        let c = classify(&buf).unwrap();
        assert_eq!(c.format, DdsFormat::Unknown);
        assert_eq!(c.crimson_last4, None);
    }

    #[test]
    fn block_bytes_table() {
        assert_eq!(DdsFormat::Dxt1.block_bytes(), Some(8));
        assert_eq!(DdsFormat::Dxt5.block_bytes(), Some(16));
        assert_eq!(DdsFormat::Bc4Unorm.block_bytes(), Some(8));
        assert_eq!(DdsFormat::Bc7Unorm.block_bytes(), Some(16));
        assert_eq!(DdsFormat::UncompressedRgb.block_bytes(), None);
    }

    #[test]
    fn classify_real_vanilla_dxt5_sample() {
        // Real DDS sample from DMM backups. Skip if not present (loop env
        // may not have the same paths). When present, verifies our classifier
        // produces sensible output on a real production DDS.
        let path = std::path::Path::new(
            "C:/Users/corin/Desktop/CD JSON Mod Manager/Definitive Mod Manager/src-tauri/target/debug/backups/cd_icon_map_enemy_die_1.dds",
        );
        let Ok(bytes) = std::fs::read(path) else {
            eprintln!("SKIP: real DDS sample not found at {:?}", path);
            return;
        };
        let c = classify(&bytes).unwrap();
        assert_eq!(c.format, DdsFormat::Dxt5);
        assert_eq!(c.width, 32);
        assert_eq!(c.height, 32);
        assert!(!c.is_dx10);
        assert_eq!(c.crimson_last4, Some(15));
        assert!(!c.requires_pathc);
    }
}
