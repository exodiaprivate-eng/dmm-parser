// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! PABGH (PA Binary Group Header) — index file describing entry boundaries
//! within a sister .pabgb data file.
//!
//! Six on-disk formats observed in retail Crimson Desert:
//!  - 8-byte entry: u16 count + N × {u32 key, u32 offset}
//!  - 6-byte entry: u16 count + N × {u16 key, u32 offset}
//!  - 8-byte entry, u32 count: u32 count + N × {u32 key, u32 offset}
//!  - 5-byte entry: u16 count + N × {u8 key, u32 offset}
//!    (mercenarygroupinfo, mercenaryinfo, relationinfo)
//!  - 12-byte entry, u32 count: u32 count + N × {u32 key, 4B extra, u32 offset}
//!    (characterappearanceindexinfo)
//!  - 16-byte entry: u16 count + N × {u16 a, u16 b, u32 hash, u32 c, u32 offset}
//!    (aieventtableinfo)
//!
//! For the 3 new variants the middle bytes between key and offset are
//! preserved opaquely in `extra_bytes` so writes round-trip byte-identical.
//! Their semantic decoding is still pending — but the diff tool and the
//! typed-table parsers only need {key, offset} to compute entry sizes,
//! so they work today without further analysis.
//!
//! The format is auto-detected from the file size against the count value.

use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PabghFormat {
    /// u16 count, u32 key, u32 offset (8 bytes per entry).
    U16CountU32Key,
    /// u16 count, u16 key, u32 offset (6 bytes per entry).
    U16CountU16Key,
    /// u32 count, u32 key, u32 offset (8 bytes per entry).
    U32CountU32Key,
    /// u16 count, u8 key, u32 offset (5 bytes per entry).
    /// First seen in: mercenarygroupinfo, mercenaryinfo, relationinfo.
    U16CountU8Key,
    /// u32 count, u32 key + i32 parent_id + u32 offset (12 bytes per entry).
    /// First seen in: characterappearanceindexinfo. parent_id = -2
    /// (`0xFFFFFFFE`) is the root/no-parent sentinel (~86% of entries);
    /// small positives reference a parent group. Field decoded iter 2.
    U32CountU32KeyExtra4,
    /// u16 count, 12B header + u32 offset (16 bytes per entry).
    /// First seen in: aieventtableinfo. Header layout (decoded iter 2):
    /// `u16 const_FFFF + u16 reserved + u32 hash_key + u32 c`. The real
    /// lookup key is `hash_key` at bytes [4:8] (always unique across N
    /// entries). The Pabgh.key field exposes this hash.
    U16CountExtra12,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PabghEntry {
    pub key: u32,
    pub offset: u32,
    /// Auxiliary header bytes that flank the key for variants that carry
    /// them. Layout depends on the parent `Pabgh.format`:
    ///   - `U32CountU32KeyExtra4`: 4 bytes — `i32 parent_id` (-2 = root)
    ///   - `U16CountExtra12`: 12 bytes — `u16 const_FFFF + u16 reserved +
    ///     u32 hash_key + u32 c`. (Note: `key` field already mirrors
    ///     `hash_key`; the full 12 bytes are kept here for round-trip.)
    ///   - all other variants: empty
    pub extra_bytes: Vec<u8>,
}

impl PabghEntry {
    /// For `U32CountU32KeyExtra4` entries (characterappearanceindexinfo),
    /// returns the parent_id field (signed; -2 = root). For other formats,
    /// returns None.
    pub fn parent_id(&self) -> Option<i32> {
        if self.extra_bytes.len() == 4 {
            Some(i32::from_le_bytes(self.extra_bytes[..4].try_into().unwrap()))
        } else {
            None
        }
    }

    /// For `U16CountExtra12` entries (aieventtableinfo), returns the
    /// trailing `u32 c` field (typically 0xFFFFFFFF). For other formats,
    /// returns None.
    pub fn aux_c(&self) -> Option<u32> {
        if self.extra_bytes.len() == 12 {
            Some(u32::from_le_bytes(self.extra_bytes[8..12].try_into().unwrap()))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Pabgh {
    pub format: PabghFormat,
    pub entries: Vec<PabghEntry>,
}

impl Pabgh {
    /// Auto-detect format and parse the index. Returns Err if the layout
    /// doesn't match any of the three known formats.
    pub fn parse(data: &[u8]) -> io::Result<Self> {
        if data.len() < 4 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "pabgh too short"));
        }
        let c16 = u16::from_le_bytes(data[0..2].try_into().unwrap()) as usize;
        let c32 = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;

        // Format 1: u16 count + N × 8-byte entries (u32 key + u32 offset)
        if 2 + c16 * 8 == data.len() {
            return Ok(Self::parse_inner(data, 2, c16, PabghFormat::U16CountU32Key));
        }
        // Format 2: u16 count + N × 6-byte entries (u16 key + u32 offset)
        if 2 + c16 * 6 == data.len() {
            return Ok(Self::parse_inner(data, 2, c16, PabghFormat::U16CountU16Key));
        }
        // Format 3: u32 count + N × 8-byte entries (u32 key + u32 offset)
        if 4 + c32 * 8 == data.len() {
            return Ok(Self::parse_inner(data, 4, c32, PabghFormat::U32CountU32Key));
        }
        // Format 4: u16 count + N × 5-byte entries (u8 key + u32 offset).
        // Used by tiny-keyed tables (mercenarygroupinfo, mercenaryinfo,
        // relationinfo) where total key count fits in a byte.
        if 2 + c16 * 5 == data.len() {
            return Ok(Self::parse_inner(data, 2, c16, PabghFormat::U16CountU8Key));
        }
        // Format 5: u32 count + N × 12-byte entries (u32 key + 4B extra + u32 offset).
        // Used by characterappearanceindexinfo. Middle 4 bytes are typically
        // 0xFFFFFFFE sentinel; preserved opaquely for round-trip.
        if 4 + c32 * 12 == data.len() {
            return Ok(Self::parse_inner(data, 4, c32, PabghFormat::U32CountU32KeyExtra4));
        }
        // Format 6: u16 count + N × 16-byte entries (12B opaque + u32 offset).
        // Used by aieventtableinfo. Middle 12 bytes likely (u16 a, u16 b,
        // u32 hash, u32 c) but exact field decode TBD; preserved opaquely.
        if 2 + c16 * 16 == data.len() {
            return Ok(Self::parse_inner(data, 2, c16, PabghFormat::U16CountExtra12));
        }
        Err(io::Error::new(io::ErrorKind::InvalidData,
            format!("pabgh size {} doesn't match any known layout (c16={}, c32={})",
                data.len(), c16, c32)))
    }

    fn parse_inner(data: &[u8], idx_start: usize, count: usize, format: PabghFormat) -> Self {
        let mut entries = Vec::with_capacity(count);
        let entry_size = match format {
            PabghFormat::U16CountU32Key | PabghFormat::U32CountU32Key => 8,
            PabghFormat::U16CountU16Key => 6,
            PabghFormat::U16CountU8Key => 5,
            PabghFormat::U32CountU32KeyExtra4 => 12,
            PabghFormat::U16CountExtra12 => 16,
        };
        for i in 0..count {
            let pos = idx_start + i * entry_size;
            let (key, offset, extra) = match format {
                PabghFormat::U16CountU16Key => {
                    let k = u16::from_le_bytes(data[pos..pos + 2].try_into().unwrap()) as u32;
                    let o = u32::from_le_bytes(data[pos + 2..pos + 6].try_into().unwrap());
                    (k, o, Vec::new())
                }
                PabghFormat::U16CountU8Key => {
                    let k = data[pos] as u32;
                    let o = u32::from_le_bytes(data[pos + 1..pos + 5].try_into().unwrap());
                    (k, o, Vec::new())
                }
                PabghFormat::U32CountU32KeyExtra4 => {
                    // Layout decoded iter 2: u32 key + i32 parent_id + u32 offset.
                    // parent_id sentinel is -2 (0xFFFFFFFE) = root/no-parent for
                    // ~86% of characterappearanceindexinfo entries; small positives
                    // reference a parent group/cluster. Preserved in extra_bytes
                    // for round-trip; semantic field is `parent_id` accessor.
                    let k = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
                    let extra = data[pos + 4..pos + 8].to_vec();
                    let o = u32::from_le_bytes(data[pos + 8..pos + 12].try_into().unwrap());
                    (k, o, extra)
                }
                PabghFormat::U16CountExtra12 => {
                    // Layout decoded iter 2 (aieventtableinfo n=940):
                    //   u16 const  always 0xFFFF (sentinel/marker)
                    //   u16 reserv always 0
                    //   u32 hash   ALL UNIQUE — this is the actual lookup key
                    //   u32 c      0xFFFFFFFF for 99%, small positives otherwise
                    //   u32 offset (the offset field)
                    // The previous iter 1 implementation synthesized `key` from
                    // bytes[0:4] which was always 0xFFFF — WRONG. The real key
                    // is the u32 hash at bytes[4:8].
                    let extra = data[pos..pos + 12].to_vec();
                    let k = u32::from_le_bytes(data[pos + 4..pos + 8].try_into().unwrap());
                    let o = u32::from_le_bytes(data[pos + 12..pos + 16].try_into().unwrap());
                    (k, o, extra)
                }
                PabghFormat::U16CountU32Key | PabghFormat::U32CountU32Key => {
                    let k = u32::from_le_bytes(data[pos..pos + 4].try_into().unwrap());
                    let o = u32::from_le_bytes(data[pos + 4..pos + 8].try_into().unwrap());
                    (k, o, Vec::new())
                }
            };
            entries.push(PabghEntry { key, offset, extra_bytes: extra });
        }
        Self { format, entries }
    }

    /// Serialize back to the original byte layout. Round-trips byte-identical
    /// against the source file.
    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match self.format {
            PabghFormat::U16CountU32Key => {
                w.write_all(&(self.entries.len() as u16).to_le_bytes())?;
                for e in &self.entries {
                    w.write_all(&e.key.to_le_bytes())?;
                    w.write_all(&e.offset.to_le_bytes())?;
                }
            }
            PabghFormat::U16CountU16Key => {
                w.write_all(&(self.entries.len() as u16).to_le_bytes())?;
                for e in &self.entries {
                    w.write_all(&(e.key as u16).to_le_bytes())?;
                    w.write_all(&e.offset.to_le_bytes())?;
                }
            }
            PabghFormat::U32CountU32Key => {
                w.write_all(&(self.entries.len() as u32).to_le_bytes())?;
                for e in &self.entries {
                    w.write_all(&e.key.to_le_bytes())?;
                    w.write_all(&e.offset.to_le_bytes())?;
                }
            }
            PabghFormat::U16CountU8Key => {
                w.write_all(&(self.entries.len() as u16).to_le_bytes())?;
                for e in &self.entries {
                    if e.key > 0xFF {
                        return Err(io::Error::new(io::ErrorKind::InvalidData,
                            format!("U16CountU8Key: key {} doesn't fit in u8", e.key)));
                    }
                    w.write_all(&[e.key as u8])?;
                    w.write_all(&e.offset.to_le_bytes())?;
                }
            }
            PabghFormat::U32CountU32KeyExtra4 => {
                w.write_all(&(self.entries.len() as u32).to_le_bytes())?;
                for e in &self.entries {
                    w.write_all(&e.key.to_le_bytes())?;
                    if e.extra_bytes.len() != 4 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData,
                            format!("U32CountU32KeyExtra4: extra_bytes must be 4, got {}",
                                e.extra_bytes.len())));
                    }
                    w.write_all(&e.extra_bytes)?;
                    w.write_all(&e.offset.to_le_bytes())?;
                }
            }
            PabghFormat::U16CountExtra12 => {
                w.write_all(&(self.entries.len() as u16).to_le_bytes())?;
                for e in &self.entries {
                    if e.extra_bytes.len() != 12 {
                        return Err(io::Error::new(io::ErrorKind::InvalidData,
                            format!("U16CountExtra12: extra_bytes must be 12, got {}",
                                e.extra_bytes.len())));
                    }
                    // Extra contains the full pre-offset 12 bytes including
                    // what we exposed as `key`. Emit extra verbatim, then offset.
                    w.write_all(&e.extra_bytes)?;
                    w.write_all(&e.offset.to_le_bytes())?;
                }
            }
        }
        Ok(())
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut out = Vec::new();
        self.write_to(&mut out)?;
        Ok(out)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_u16_count_u8_key_5byte() {
        // mercenarygroupinfo.pabgh-style (count=10, 5-byte entries).
        // First 6 entries match the real 1.05/1.06 file leading bytes.
        let mut full = Vec::new();
        full.extend_from_slice(&10u16.to_le_bytes());
        let leading: [(u8, u32); 6] = [
            (0x40, 0), (0x41, 0x22), (0x42, 0x3d),
            (0x43, 0x56), (0x44, 0x6d), (0x48, 0x8a),
        ];
        for (k, o) in leading {
            full.push(k);
            full.extend_from_slice(&o.to_le_bytes());
        }
        // Pad to 10 entries
        for i in 6..10 {
            full.push(0x50 + i as u8);
            full.extend_from_slice(&((i as u32) * 100).to_le_bytes());
        }
        assert_eq!(full.len(), 52);
        let pabgh = Pabgh::parse(&full).expect("parse 5-byte format");
        assert_eq!(pabgh.format, PabghFormat::U16CountU8Key);
        assert_eq!(pabgh.entries.len(), 10);
        assert_eq!(pabgh.entries[0].key, 0x40);
        assert_eq!(pabgh.entries[0].offset, 0);
        assert_eq!(pabgh.entries[1].key, 0x41);
        assert_eq!(pabgh.entries[1].offset, 0x22);
        // Round-trip
        let written = pabgh.to_bytes().expect("write 5-byte format");
        assert_eq!(written, full, "5-byte pabgh round-trip mismatch");
    }

    #[test]
    fn parse_u32_count_u32_key_extra4_12byte() {
        // characterappearanceindexinfo first 28 bytes (count=8143, 12-byte entries)
        // Build a small valid file: count=2, two 12-byte entries
        let mut data = Vec::new();
        data.extend_from_slice(&2u32.to_le_bytes());            // count
        // entry 0: key=1, extra=0xFFFFFFFE, offset=0
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&[0xfe, 0xff, 0xff, 0xff]);
        data.extend_from_slice(&0u32.to_le_bytes());
        // entry 1: key=0xF4797, extra=0xFFFFFFFE, offset=0x1d
        data.extend_from_slice(&0xF4797u32.to_le_bytes());
        data.extend_from_slice(&[0xfe, 0xff, 0xff, 0xff]);
        data.extend_from_slice(&0x1du32.to_le_bytes());
        assert_eq!(data.len(), 4 + 2 * 12);

        let pabgh = Pabgh::parse(&data).expect("parse 12-byte format");
        assert_eq!(pabgh.format, PabghFormat::U32CountU32KeyExtra4);
        assert_eq!(pabgh.entries.len(), 2);
        assert_eq!(pabgh.entries[0].key, 1);
        assert_eq!(pabgh.entries[0].offset, 0);
        assert_eq!(pabgh.entries[0].extra_bytes, vec![0xfe, 0xff, 0xff, 0xff]);
        assert_eq!(pabgh.entries[0].parent_id(), Some(-2), "parent_id=-2 (root sentinel)");
        assert_eq!(pabgh.entries[1].key, 0xF4797);
        assert_eq!(pabgh.entries[1].offset, 0x1d);
        assert_eq!(pabgh.entries[1].parent_id(), Some(-2));
        let written = pabgh.to_bytes().expect("write 12-byte format");
        assert_eq!(written, data, "12-byte pabgh round-trip mismatch");
    }

    #[test]
    fn parse_u16_count_extra12_16byte() {
        // aieventtableinfo-style: count=2, 16-byte entries.
        // Layout per iter 2 decode: u16=FFFF, u16=0, u32 hash_key, u32 c, u32 offset.
        let mut data = Vec::new();
        data.extend_from_slice(&2u16.to_le_bytes());            // count
        // entry 0: hash_key=0x9C49B150, c=0xFFFFFFFF, offset=0
        let e0_extra: [u8; 12] = [0xff, 0xff, 0x00, 0x00, 0x50, 0xb1, 0x49, 0x9c, 0xff, 0xff, 0xff, 0xff];
        data.extend_from_slice(&e0_extra);
        data.extend_from_slice(&0u32.to_le_bytes());
        // entry 1: hash_key=0xE69AB342, c=0xFFFFFFFF, offset=0x30
        let e1_extra: [u8; 12] = [0xff, 0xff, 0x00, 0x00, 0x42, 0xb3, 0x9a, 0xe6, 0xff, 0xff, 0xff, 0xff];
        data.extend_from_slice(&e1_extra);
        data.extend_from_slice(&0x30u32.to_le_bytes());
        assert_eq!(data.len(), 2 + 2 * 16);

        let pabgh = Pabgh::parse(&data).expect("parse 16-byte format");
        assert_eq!(pabgh.format, PabghFormat::U16CountExtra12);
        assert_eq!(pabgh.entries.len(), 2);
        // key field exposes hash_key at bytes[4:8]
        assert_eq!(pabgh.entries[0].key, 0x9C49B150);
        assert_eq!(pabgh.entries[0].offset, 0);
        assert_eq!(pabgh.entries[0].extra_bytes, e0_extra.to_vec());
        assert_eq!(pabgh.entries[0].aux_c(), Some(0xFFFFFFFF));
        assert_eq!(pabgh.entries[1].key, 0xE69AB342);
        assert_eq!(pabgh.entries[1].offset, 0x30);
        assert_eq!(pabgh.entries[1].aux_c(), Some(0xFFFFFFFF));
        let written = pabgh.to_bytes().expect("write 16-byte format");
        assert_eq!(written, data, "16-byte pabgh round-trip mismatch");
    }
}
