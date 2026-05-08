// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! `.dds` — DirectDraw Surface texture (Microsoft public spec).
//!
//! Structure:
//!   - 4-byte magic `"DDS "` (0x20534444 LE)
//!   - 124-byte `DDS_HEADER` (width / height / mip count / pixel-format /
//!     caps)
//!   - Optional 20-byte `DDS_HEADER_DXT10` extension when
//!     `pixel_format.fourcc == "DX10"`
//!   - Pixel data for mip 0, mip 1, …
//!
//! Round-trip: parse() + to_bytes() preserves every byte. The `data`
//! field is the raw pixel-data tail; modders can drop in a new texture
//! by replacing it. Header fields are individually editable as JSON.
//!
//! Some PA-engine `.dds` files are stored "partial" (header + low-res
//! mips inline; high-res mips streamed). For those, `data` will be
//! shorter than the header advertises; round-trip still works because
//! we round-trip exactly the bytes we received.

use std::io::{self, Write};

use serde_json::{Map, Value};

pub const DDS_MAGIC: u32 = 0x2053_4444; // "DDS "
pub const DX10_FOURCC: u32 = 0x3031_5844; // "DX10"
pub const HEADER_SIZE: usize = 128; // 4-byte magic + 124-byte DDS_HEADER
pub const DX10_HEADER_SIZE: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DdsPixelFormat {
    pub size: u32,        // always 32
    pub flags: u32,       // DDPF_* bitfield
    pub fourcc: u32,      // "DXT1" / "DXT5" / "BC4U" / "DX10" etc.
    pub rgb_bit_count: u32,
    pub r_bit_mask: u32,
    pub g_bit_mask: u32,
    pub b_bit_mask: u32,
    pub a_bit_mask: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DdsHeader {
    pub size: u32,                  // always 124
    pub flags: u32,                 // DDSD_* bitfield
    pub height: u32,
    pub width: u32,
    pub pitch_or_linear_size: u32,
    pub depth: u32,
    pub mip_map_count: u32,
    pub reserved1: [u32; 11],
    pub pixel_format: DdsPixelFormat,
    pub caps: u32,                  // DDSCAPS_*
    pub caps2: u32,                 // DDSCAPS2_* (cubemap, volume)
    pub caps3: u32,
    pub caps4: u32,
    pub reserved2: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DdsHeaderDxt10 {
    pub dxgi_format: u32,
    pub resource_dimension: u32,    // D3D10_RESOURCE_DIMENSION_*
    pub misc_flag: u32,             // bit 2 = TEXTURECUBE
    pub array_size: u32,
    pub misc_flags2: u32,           // alpha mode bits
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DdsFile {
    pub header: DdsHeader,
    pub dx10: Option<DdsHeaderDxt10>,
    /// Raw pixel data tail. For "partial" PA files this may be shorter than
    /// the header advertises — that's expected and round-trips cleanly.
    pub data: Vec<u8>,
}

// ── Parse ───────────────────────────────────────────────────────────────────

#[inline]
fn u32_at(data: &[u8], off: usize) -> u32 {
    u32::from_le_bytes(data[off..off + 4].try_into().unwrap())
}

impl DdsPixelFormat {
    fn parse(data: &[u8], off: usize) -> Self {
        DdsPixelFormat {
            size:           u32_at(data, off),
            flags:          u32_at(data, off + 4),
            fourcc:         u32_at(data, off + 8),
            rgb_bit_count:  u32_at(data, off + 12),
            r_bit_mask:     u32_at(data, off + 16),
            g_bit_mask:     u32_at(data, off + 20),
            b_bit_mask:     u32_at(data, off + 24),
            a_bit_mask:     u32_at(data, off + 28),
        }
    }

    fn write(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.size.to_le_bytes())?;
        w.write_all(&self.flags.to_le_bytes())?;
        w.write_all(&self.fourcc.to_le_bytes())?;
        w.write_all(&self.rgb_bit_count.to_le_bytes())?;
        w.write_all(&self.r_bit_mask.to_le_bytes())?;
        w.write_all(&self.g_bit_mask.to_le_bytes())?;
        w.write_all(&self.b_bit_mask.to_le_bytes())?;
        w.write_all(&self.a_bit_mask.to_le_bytes())?;
        Ok(())
    }
}

impl DdsHeader {
    fn parse(data: &[u8]) -> io::Result<Self> {
        if data.len() < HEADER_SIZE {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof,
                format!("need {} bytes for DDS header, got {}", HEADER_SIZE, data.len())));
        }
        let magic = u32_at(data, 0);
        if magic != DDS_MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("bad DDS magic: 0x{:08X} (expected 0x{:08X})", magic, DDS_MAGIC)));
        }

        // DDS_HEADER starts at byte 4 (after magic). Within DDS_HEADER,
        // dwReserved1[11] sits at offset 32..76 from the file start
        // (after magic + 7 u32 fields = 4 + 28 = 32 bytes consumed).
        let mut reserved1 = [0u32; 11];
        for i in 0..11 {
            reserved1[i] = u32_at(data, 32 + i * 4);
        }

        Ok(DdsHeader {
            size:                  u32_at(data, 4),
            flags:                 u32_at(data, 8),
            height:                u32_at(data, 12),
            width:                 u32_at(data, 16),
            pitch_or_linear_size:  u32_at(data, 20),
            depth:                 u32_at(data, 24),
            mip_map_count:         u32_at(data, 28),
            reserved1,
            pixel_format:          DdsPixelFormat::parse(data, 76),
            caps:                  u32_at(data, 108),
            caps2:                 u32_at(data, 112),
            caps3:                 u32_at(data, 116),
            caps4:                 u32_at(data, 120),
            reserved2:             u32_at(data, 124),
        })
    }

    fn write(&self, w: &mut dyn Write) -> io::Result<()> {
        // DDS magic
        w.write_all(&DDS_MAGIC.to_le_bytes())?;
        // DDS_HEADER fields
        w.write_all(&self.size.to_le_bytes())?;
        w.write_all(&self.flags.to_le_bytes())?;
        w.write_all(&self.height.to_le_bytes())?;
        w.write_all(&self.width.to_le_bytes())?;
        w.write_all(&self.pitch_or_linear_size.to_le_bytes())?;
        w.write_all(&self.depth.to_le_bytes())?;
        w.write_all(&self.mip_map_count.to_le_bytes())?;
        for v in &self.reserved1 {
            w.write_all(&v.to_le_bytes())?;
        }
        self.pixel_format.write(w)?;
        w.write_all(&self.caps.to_le_bytes())?;
        w.write_all(&self.caps2.to_le_bytes())?;
        w.write_all(&self.caps3.to_le_bytes())?;
        w.write_all(&self.caps4.to_le_bytes())?;
        w.write_all(&self.reserved2.to_le_bytes())?;
        Ok(())
    }
}

impl DdsHeaderDxt10 {
    fn parse(data: &[u8], off: usize) -> io::Result<Self> {
        if data.len() < off + DX10_HEADER_SIZE {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof,
                format!("need {} bytes for DX10 header at offset {}, got {}",
                    DX10_HEADER_SIZE, off, data.len())));
        }
        Ok(DdsHeaderDxt10 {
            dxgi_format:        u32_at(data, off),
            resource_dimension: u32_at(data, off + 4),
            misc_flag:          u32_at(data, off + 8),
            array_size:         u32_at(data, off + 12),
            misc_flags2:        u32_at(data, off + 16),
        })
    }

    fn write(&self, w: &mut dyn Write) -> io::Result<()> {
        w.write_all(&self.dxgi_format.to_le_bytes())?;
        w.write_all(&self.resource_dimension.to_le_bytes())?;
        w.write_all(&self.misc_flag.to_le_bytes())?;
        w.write_all(&self.array_size.to_le_bytes())?;
        w.write_all(&self.misc_flags2.to_le_bytes())?;
        Ok(())
    }
}

impl DdsFile {
    pub fn parse(data: &[u8]) -> io::Result<Self> {
        let header = DdsHeader::parse(data)?;
        let mut data_start = HEADER_SIZE;
        let dx10 = if header.pixel_format.fourcc == DX10_FOURCC {
            let h = DdsHeaderDxt10::parse(data, HEADER_SIZE)?;
            data_start += DX10_HEADER_SIZE;
            Some(h)
        } else {
            None
        };
        let pixel_data = data.get(data_start..)
            .map(|s| s.to_vec())
            .unwrap_or_default();
        Ok(DdsFile { header, dx10, data: pixel_data })
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::with_capacity(HEADER_SIZE + DX10_HEADER_SIZE + self.data.len());
        self.write_to(&mut buf)?;
        Ok(buf)
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.header.write(w)?;
        if let Some(dx10) = &self.dx10 {
            dx10.write(w)?;
        }
        w.write_all(&self.data)?;
        Ok(())
    }
}

// ── JSON bridge ─────────────────────────────────────────────────────────────

fn fourcc_str(v: u32) -> String {
    let bytes = v.to_le_bytes();
    let mut s = String::new();
    for b in &bytes {
        if (0x20..=0x7E).contains(b) { s.push(*b as char); } else { s.clear(); break; }
    }
    s
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xF) as usize] as char);
    }
    s
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    if s.len() % 2 != 0 {
        return Err(format!("odd-length hex ({} chars)", s.len()));
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let hi = hex_nibble(bytes[i])?;
        let lo = hex_nibble(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

fn hex_nibble(c: u8) -> Result<u8, String> {
    match c {
        b'0'..=b'9' => Ok(c - b'0'),
        b'a'..=b'f' => Ok(c - b'a' + 10),
        b'A'..=b'F' => Ok(c - b'A' + 10),
        _ => Err(format!("invalid hex char '{}'", c as char)),
    }
}

impl DdsFile {
    pub fn to_json(&self) -> Value {
        let mut header = Map::new();
        header.insert("size".into(), Value::from(self.header.size));
        header.insert("flags".into(), Value::from(self.header.flags));
        header.insert("height".into(), Value::from(self.header.height));
        header.insert("width".into(), Value::from(self.header.width));
        header.insert("pitch_or_linear_size".into(), Value::from(self.header.pitch_or_linear_size));
        header.insert("depth".into(), Value::from(self.header.depth));
        header.insert("mip_map_count".into(), Value::from(self.header.mip_map_count));
        header.insert("reserved1".into(),
            Value::Array(self.header.reserved1.iter().map(|v| Value::from(*v)).collect()));

        let mut pf = Map::new();
        pf.insert("size".into(), Value::from(self.header.pixel_format.size));
        pf.insert("flags".into(), Value::from(self.header.pixel_format.flags));
        pf.insert("fourcc".into(), Value::from(self.header.pixel_format.fourcc));
        let fcs = fourcc_str(self.header.pixel_format.fourcc);
        if !fcs.is_empty() {
            pf.insert("fourcc_str".into(), Value::String(fcs));
        }
        pf.insert("rgb_bit_count".into(), Value::from(self.header.pixel_format.rgb_bit_count));
        pf.insert("r_bit_mask".into(), Value::from(self.header.pixel_format.r_bit_mask));
        pf.insert("g_bit_mask".into(), Value::from(self.header.pixel_format.g_bit_mask));
        pf.insert("b_bit_mask".into(), Value::from(self.header.pixel_format.b_bit_mask));
        pf.insert("a_bit_mask".into(), Value::from(self.header.pixel_format.a_bit_mask));
        header.insert("pixel_format".into(), Value::Object(pf));

        header.insert("caps".into(),  Value::from(self.header.caps));
        header.insert("caps2".into(), Value::from(self.header.caps2));
        header.insert("caps3".into(), Value::from(self.header.caps3));
        header.insert("caps4".into(), Value::from(self.header.caps4));
        header.insert("reserved2".into(), Value::from(self.header.reserved2));

        let mut root = Map::new();
        root.insert("header".into(), Value::Object(header));
        if let Some(dx10) = &self.dx10 {
            let mut d = Map::new();
            d.insert("dxgi_format".into(),         Value::from(dx10.dxgi_format));
            d.insert("resource_dimension".into(),  Value::from(dx10.resource_dimension));
            d.insert("misc_flag".into(),           Value::from(dx10.misc_flag));
            d.insert("array_size".into(),          Value::from(dx10.array_size));
            d.insert("misc_flags2".into(),         Value::from(dx10.misc_flags2));
            root.insert("dx10".into(), Value::Object(d));
        }
        root.insert("data_hex".into(), Value::String(hex_encode(&self.data)));
        Value::Object(root)
    }

    pub fn from_json(v: &Value) -> io::Result<Self> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "expected object at root"))?;

        fn u32_field(m: &Map<String, Value>, k: &str) -> io::Result<u32> {
            m.get(k).and_then(|v| v.as_u64()).map(|v| v as u32)
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                    format!("missing or non-numeric field '{}'", k)))
        }

        let header_obj = obj.get("header").and_then(|v| v.as_object())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing header object"))?;
        let pf_obj = header_obj.get("pixel_format").and_then(|v| v.as_object())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing pixel_format object"))?;

        let mut reserved1 = [0u32; 11];
        let r1 = header_obj.get("reserved1").and_then(|v| v.as_array())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing reserved1 array"))?;
        if r1.len() != 11 {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("reserved1 must have 11 entries, got {}", r1.len())));
        }
        for (i, v) in r1.iter().enumerate() {
            reserved1[i] = v.as_u64().map(|x| x as u32).ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData, format!("reserved1[{}] not numeric", i)))?;
        }

        let pixel_format = DdsPixelFormat {
            size:          u32_field(pf_obj, "size")?,
            flags:         u32_field(pf_obj, "flags")?,
            fourcc:        u32_field(pf_obj, "fourcc")?,
            rgb_bit_count: u32_field(pf_obj, "rgb_bit_count")?,
            r_bit_mask:    u32_field(pf_obj, "r_bit_mask")?,
            g_bit_mask:    u32_field(pf_obj, "g_bit_mask")?,
            b_bit_mask:    u32_field(pf_obj, "b_bit_mask")?,
            a_bit_mask:    u32_field(pf_obj, "a_bit_mask")?,
        };

        let header = DdsHeader {
            size:                  u32_field(header_obj, "size")?,
            flags:                 u32_field(header_obj, "flags")?,
            height:                u32_field(header_obj, "height")?,
            width:                 u32_field(header_obj, "width")?,
            pitch_or_linear_size:  u32_field(header_obj, "pitch_or_linear_size")?,
            depth:                 u32_field(header_obj, "depth")?,
            mip_map_count:         u32_field(header_obj, "mip_map_count")?,
            reserved1,
            pixel_format,
            caps:                  u32_field(header_obj, "caps")?,
            caps2:                 u32_field(header_obj, "caps2")?,
            caps3:                 u32_field(header_obj, "caps3")?,
            caps4:                 u32_field(header_obj, "caps4")?,
            reserved2:             u32_field(header_obj, "reserved2")?,
        };

        let dx10 = if let Some(d) = obj.get("dx10").and_then(|v| v.as_object()) {
            Some(DdsHeaderDxt10 {
                dxgi_format:         u32_field(d, "dxgi_format")?,
                resource_dimension:  u32_field(d, "resource_dimension")?,
                misc_flag:           u32_field(d, "misc_flag")?,
                array_size:          u32_field(d, "array_size")?,
                misc_flags2:         u32_field(d, "misc_flags2")?,
            })
        } else {
            None
        };

        let data_hex = obj.get("data_hex").and_then(|v| v.as_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing data_hex string"))?;
        let data = hex_decode(data_hex).map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData, format!("data_hex decode: {}", e)))?;

        Ok(DdsFile { header, dx10, data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_minimal_dds() -> Vec<u8> {
        // 4-byte magic + 124-byte DDS_HEADER + 0 bytes data
        let mut buf = Vec::with_capacity(HEADER_SIZE);
        buf.extend_from_slice(&DDS_MAGIC.to_le_bytes());           // magic
        buf.extend_from_slice(&124u32.to_le_bytes());              // size
        buf.extend_from_slice(&0x0000100Fu32.to_le_bytes());       // flags
        buf.extend_from_slice(&64u32.to_le_bytes());               // height
        buf.extend_from_slice(&64u32.to_le_bytes());               // width
        buf.extend_from_slice(&2048u32.to_le_bytes());             // pitch
        buf.extend_from_slice(&0u32.to_le_bytes());                // depth
        buf.extend_from_slice(&1u32.to_le_bytes());                // mips
        for _ in 0..11 { buf.extend_from_slice(&0u32.to_le_bytes()); } // reserved1
        // PixelFormat (32 bytes)
        buf.extend_from_slice(&32u32.to_le_bytes());               // size
        buf.extend_from_slice(&0x00000004u32.to_le_bytes());       // flags = DDPF_FOURCC
        buf.extend_from_slice(&0x31545844u32.to_le_bytes());       // fourcc "DXT1"
        for _ in 0..5 { buf.extend_from_slice(&0u32.to_le_bytes()); }
        buf.extend_from_slice(&0x1000u32.to_le_bytes());           // caps
        for _ in 0..4 { buf.extend_from_slice(&0u32.to_le_bytes()); }
        buf
    }

    #[test]
    fn parse_and_write_minimal() {
        let bytes = build_minimal_dds();
        assert_eq!(bytes.len(), HEADER_SIZE);
        let f = DdsFile::parse(&bytes).unwrap();
        assert_eq!(f.header.width, 64);
        assert_eq!(f.header.height, 64);
        assert_eq!(f.header.mip_map_count, 1);
        assert_eq!(f.header.pixel_format.fourcc, 0x31545844);
        assert_eq!(f.dx10, None);
        assert!(f.data.is_empty());
        assert_eq!(f.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn parse_with_data_tail() {
        let mut bytes = build_minimal_dds();
        bytes.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);
        let f = DdsFile::parse(&bytes).unwrap();
        assert_eq!(f.data, vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE]);
        assert_eq!(f.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn parse_with_dx10_header() {
        let mut bytes = build_minimal_dds();
        // Patch fourcc to "DX10"
        bytes[84..88].copy_from_slice(&DX10_FOURCC.to_le_bytes());
        // Append 20-byte DX10 header
        bytes.extend_from_slice(&98u32.to_le_bytes());            // dxgi_format BC7_UNORM
        bytes.extend_from_slice(&3u32.to_le_bytes());             // resource_dim TEXTURE2D
        bytes.extend_from_slice(&0u32.to_le_bytes());             // misc_flag
        bytes.extend_from_slice(&1u32.to_le_bytes());             // array_size
        bytes.extend_from_slice(&0u32.to_le_bytes());             // misc_flags2
        // Plus tail data
        bytes.extend_from_slice(&[1, 2, 3]);
        let f = DdsFile::parse(&bytes).unwrap();
        let dx10 = f.dx10.as_ref().unwrap();
        assert_eq!(dx10.dxgi_format, 98);
        assert_eq!(dx10.array_size, 1);
        assert_eq!(f.data, vec![1, 2, 3]);
        assert_eq!(f.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn json_roundtrip_no_dx10() {
        let mut bytes = build_minimal_dds();
        bytes.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]);
        let f = DdsFile::parse(&bytes).unwrap();
        let j = f.to_json();
        let f2 = DdsFile::from_json(&j).unwrap();
        assert_eq!(f, f2);
        assert_eq!(f2.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn json_roundtrip_with_dx10() {
        let mut bytes = build_minimal_dds();
        bytes[84..88].copy_from_slice(&DX10_FOURCC.to_le_bytes());
        bytes.extend_from_slice(&98u32.to_le_bytes());
        bytes.extend_from_slice(&3u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&1u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&[1, 2, 3]);
        let f = DdsFile::parse(&bytes).unwrap();
        let j = f.to_json();
        let f2 = DdsFile::from_json(&j).unwrap();
        assert_eq!(f, f2);
        assert_eq!(f2.to_bytes().unwrap(), bytes);
    }

    #[test]
    fn rejects_bad_magic() {
        let mut bytes = build_minimal_dds();
        bytes[0] = 0;
        assert!(DdsFile::parse(&bytes).is_err());
    }
}
