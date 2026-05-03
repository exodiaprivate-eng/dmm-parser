// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Raw DDS header parsing. Mirrors the layout documented in
//! `references/dds.hexpat`.

use std::io;

pub const DDS_MAGIC: &[u8; 4] = b"DDS ";
pub const DDS_HEADER_SIZE: usize = 124;
pub const DDS_FILE_HEADER_SIZE: usize = 128; // magic (4) + header (124)
pub const DX10_EXTENSION_SIZE: usize = 20;
pub const DX10_HEADER_TOTAL_SIZE: usize = DDS_FILE_HEADER_SIZE + DX10_EXTENSION_SIZE; // 148

#[derive(Debug, Clone, PartialEq)]
pub struct DdsPixelFormat {
    pub pf_size: u32,
    pub pf_flags: u32,
    pub pf_fourcc: [u8; 4],
    pub pf_rgb_bits: u32,
    pub pf_r_bitmask: u32,
    pub pf_g_bitmask: u32,
    pub pf_b_bitmask: u32,
    pub pf_a_bitmask: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DdsHeader {
    pub size: u32,
    pub flags: u32,
    pub height: u32,
    pub width: u32,
    pub pitch_or_linear_size: u32,
    pub depth: u32,
    pub mip_map_count: u32,

    /// Crimson-specific: Reserved1[0..4] used by the game to store mip
    /// level sizes after PAZ overlay patching. Zero in vanilla on-disk
    /// files. Indexed `[0] = mip0_size` through `[3] = mip3_size`.
    pub crimson_mip_sizes: [u32; 4],

    /// Reserved1[4..11] — truly reserved; always zero.
    pub reserved1_unused: [u32; 7],

    pub pixel_format: DdsPixelFormat,
    pub caps1: u32,
    pub caps2: u32,
    pub caps3: u32,
    pub caps4: u32,

    /// Crimson-specific: at offset 124 (dwReserved2 in standard spec).
    /// Game writes a format-derived "last4" identifier when patching for
    /// overlay. Zero in vanilla. See `references/dds_notes.md` §3.
    pub crimson_last4: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Dx10Header {
    pub dxgi_format: u32,
    pub resource_dimension: u32,
    pub misc_flag: u32,
    pub array_size: u32,
    pub misc_flags2: u32,
}

fn read_u32(data: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(data[off..off + 4].try_into().unwrap())
}

impl DdsHeader {
    /// Parse the 128-byte DDS header (magic + header) from raw bytes.
    /// Returns Err if the magic is wrong or the buffer is too short.
    pub fn parse(data: &[u8]) -> io::Result<Self> {
        if data.len() < DDS_FILE_HEADER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!("DDS header requires >= {} bytes, got {}", DDS_FILE_HEADER_SIZE, data.len()),
            ));
        }
        if &data[0..4] != DDS_MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("DDS magic mismatch: got {:?}", &data[0..4]),
            ));
        }

        let crimson_mip_sizes = [
            read_u32(data, 32),
            read_u32(data, 36),
            read_u32(data, 40),
            read_u32(data, 44),
        ];
        let mut reserved1_unused = [0u32; 7];
        for (i, slot) in reserved1_unused.iter_mut().enumerate() {
            *slot = read_u32(data, 48 + i * 4);
        }

        let pixel_format = DdsPixelFormat {
            pf_size: read_u32(data, 76),
            pf_flags: read_u32(data, 80),
            pf_fourcc: data[84..88].try_into().unwrap(),
            pf_rgb_bits: read_u32(data, 88),
            pf_r_bitmask: read_u32(data, 92),
            pf_g_bitmask: read_u32(data, 96),
            pf_b_bitmask: read_u32(data, 100),
            pf_a_bitmask: read_u32(data, 104),
        };

        Ok(DdsHeader {
            size: read_u32(data, 4),
            flags: read_u32(data, 8),
            height: read_u32(data, 12),
            width: read_u32(data, 16),
            pitch_or_linear_size: read_u32(data, 20),
            depth: read_u32(data, 24),
            mip_map_count: read_u32(data, 28),
            crimson_mip_sizes,
            reserved1_unused,
            pixel_format,
            caps1: read_u32(data, 108),
            caps2: read_u32(data, 112),
            caps3: read_u32(data, 116),
            caps4: read_u32(data, 120),
            crimson_last4: read_u32(data, 124),
        })
    }

    /// Returns true if this DDS uses the DX10 extension (FOURCC = "DX10").
    pub fn is_dx10(&self) -> bool {
        &self.pixel_format.pf_fourcc == b"DX10"
    }

    /// Returns true if this DDS has been patched for Crimson Desert
    /// overlay use (i.e. `crimson_last4` is non-zero).
    pub fn is_overlay_patched(&self) -> bool {
        self.crimson_last4 != 0
    }
}

impl Dx10Header {
    /// Parse the 20-byte DX10 extension at offset 128. Caller must verify
    /// `header.is_dx10()` first.
    pub fn parse(data: &[u8]) -> io::Result<Self> {
        if data.len() < DX10_HEADER_TOTAL_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                format!("DX10 header requires >= {} bytes, got {}", DX10_HEADER_TOTAL_SIZE, data.len()),
            ));
        }
        Ok(Dx10Header {
            dxgi_format: read_u32(data, 128),
            resource_dimension: read_u32(data, 132),
            misc_flag: read_u32(data, 136),
            array_size: read_u32(data, 140),
            misc_flags2: read_u32(data, 144),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_minimal_dds(fourcc: &[u8; 4]) -> Vec<u8> {
        let mut buf = vec![0u8; DDS_FILE_HEADER_SIZE];
        buf[0..4].copy_from_slice(b"DDS ");
        buf[4..8].copy_from_slice(&124u32.to_le_bytes());
        buf[12..16].copy_from_slice(&32u32.to_le_bytes());  // height
        buf[16..20].copy_from_slice(&32u32.to_le_bytes());  // width
        buf[24..28].copy_from_slice(&1u32.to_le_bytes());   // depth
        buf[28..32].copy_from_slice(&1u32.to_le_bytes());   // mips
        buf[76..80].copy_from_slice(&32u32.to_le_bytes());  // pf_size
        buf[80..84].copy_from_slice(&4u32.to_le_bytes());   // pf_flags
        buf[84..88].copy_from_slice(fourcc);
        buf
    }

    #[test]
    fn parse_dxt5_header() {
        let buf = make_minimal_dds(b"DXT5");
        let h = DdsHeader::parse(&buf).unwrap();
        assert_eq!(h.size, 124);
        assert_eq!(h.width, 32);
        assert_eq!(h.height, 32);
        assert_eq!(&h.pixel_format.pf_fourcc, b"DXT5");
        assert!(!h.is_dx10());
        assert!(!h.is_overlay_patched());
    }

    #[test]
    fn parse_dx10_header() {
        let mut buf = make_minimal_dds(b"DX10");
        buf.resize(DX10_HEADER_TOTAL_SIZE, 0);
        buf[128..132].copy_from_slice(&98u32.to_le_bytes()); // BC7
        let h = DdsHeader::parse(&buf).unwrap();
        assert!(h.is_dx10());
        let dx10 = Dx10Header::parse(&buf).unwrap();
        assert_eq!(dx10.dxgi_format, 98);
    }

    #[test]
    fn detects_overlay_patched() {
        let mut buf = make_minimal_dds(b"DXT1");
        buf[124..128].copy_from_slice(&12u32.to_le_bytes()); // last4 for DXT1
        buf[32..36].copy_from_slice(&512u32.to_le_bytes());  // mip0 size
        let h = DdsHeader::parse(&buf).unwrap();
        assert!(h.is_overlay_patched());
        assert_eq!(h.crimson_last4, 12);
        assert_eq!(h.crimson_mip_sizes[0], 512);
    }

    #[test]
    fn rejects_bad_magic() {
        let mut buf = make_minimal_dds(b"DXT5");
        buf[0..4].copy_from_slice(b"XXXX");
        assert!(DdsHeader::parse(&buf).is_err());
    }

    #[test]
    fn rejects_truncated() {
        let buf = vec![0u8; 64]; // way too short
        assert!(DdsHeader::parse(&buf).is_err());
    }
}
