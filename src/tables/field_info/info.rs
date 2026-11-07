//! Hand-corrected: typed parser for `FieldInfo.pabgb`, game 2.04.08.
//!
//! Replaces the `pabgh_blob_table!` stand-in. That macro reads `key` first,
//! but game 2.04 writes `string_key` FIRST (the same key-order change as
//! npcactivitygroupinfo / contentsphaseinfo / actionpointinfo), so on 2.04.08
//! every record failed and came back as a whole-record `_blob_b64` fallback
//! (8 of 8 base records, 6 of 6 dlc_01 records).
//!
//! Decoded from the 2.04.08 game data (Steam build 26480954), not from the exe.
//! Field names are proposals: where the values fit, the canonical `_camelCase`
//! names of the 1.x schema are reused (`_levelName`, `_boundaryPositionMin`,
//! `_boundaryPositionMax`, `_returnPosition`, `_startSectorIndex`,
//! `_endSectorIndex`, `_regionBitmapPositionInfo`, ...).
//!
//! Byte layout: confirmed. Every record is `string_key` + 112 bytes (key 4,
//! is_blocked 1, then a fixed 107-byte field body), so record size varies only
//! with `string_key`. Every record round-trips byte for byte, and through JSON
//! (tests below); the .pabgh offsets match the body.
//!
//! Comment tags: `C` = confirmed by the values; `inferred` = meaning read from
//! the values; `unk_` = meaning unknown (width confirmed by the round-trip).
//! Offsets in field comments count from the first byte after `is_blocked`.
//!
//! Records (base): MainField 1, LobbyField 3, RematchField 100,
//! RoguelikeField 200, MainField_UIMap_Texture 10, MainField_UIMap 99999,
//! MainField_Multi 2, ImageField 1000; dlc_01 adds BossRushField 300.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct FieldInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u32,
        pub is_blocked: u8,
        // +0: 0x04F6D06D in every record (inferred: `_levelName` lookup hash)
        pub level_name: u32,
        // +4: (x, z) world bounds of the field (inferred: `_boundaryPositionMin`;
        // MainField -15359, -9215)
        pub boundary_position_min: [f32; 2],
        // +12: (x, z) (inferred: `_boundaryPositionMax`; MainField 1023, 5119)
        pub boundary_position_max: [f32; 2],
        // +20: (x, y, z); 0 except MainField_UIMap_Texture (inferred:
        // `_returnPosition`)
        pub return_position: [f32; 3],
        // +32: sector (x, z) of boundary_position_min, 64-unit sectors
        // (inferred: within one sector of min / 64 in every record)
        pub start_sector_index_x: i32,
        pub start_sector_index_z: i32,
        // +40: sector (x, z) of boundary_position_max (inferred: max / 64)
        pub end_sector_index_x: i32,
        pub end_sector_index_z: i32,
        // +48: 1000001 / 1000002 / 1000003 on MainField, RematchField and
        // RoguelikeField, 0 elsewhere (inferred: refs into bitmappositioninfo,
        // `_regionBitmapPositionInfo`, `_crimeRegionBitmapPositionInfo`,
        // `_natureRegionBitmapPositionInfo`; order not confirmed)
        pub region_bitmap_position_info: u32,
        pub crime_region_bitmap_position_info: u32,
        pub nature_region_bitmap_position_info: u32,
        // +60: 31-byte block (1.x `sub_14EB7E370` composite: 5 x u32, u8, u16,
        // [u8; 8]); all 0 except the two values noted
        pub unk_u32_60: u32,
        // +64: 24 on LobbyField, 0 elsewhere
        pub unk_u32_64: u32,
        pub unk_u32_68: u32,
        pub unk_u32_72: u32,
        pub unk_u32_76: u32,
        pub unk_u8_80: u8,
        // +81: 12 in every record except LobbyField (0)
        pub unk_u16_81: u16,
        pub unk_bytes_83: [u8; 8],
        // +91: 3 in every record except BossRushField (2)
        pub unk_u8_91: u8,
        // +92..+106: 0/1 flags, vary per field
        pub unk_u8_92: u8,
        pub unk_u8_93: u8,
        pub unk_u8_94: u8,
        pub unk_u8_95: u8,
        pub unk_u8_96: u8,
        pub unk_u8_97: u8,
        pub unk_u8_98: u8,
        pub unk_u8_99: u8,
        pub unk_u8_100: u8,
        pub unk_u8_101: u8,
        pub unk_u8_102: u8,
        pub unk_u8_103: u8,
        pub unk_u8_104: u8,
        pub unk_u8_105: u8,
        pub unk_u8_106: u8,
    }
}

impl<'a> FieldInfo<'a> {
    /// pabgh-bounded read (dispatcher `p!`): the typed read must end exactly on
    /// the record boundary, otherwise the dispatcher keeps the record as a
    /// `_blob_b64` fallback.
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
    ) -> std::io::Result<Self> {
        let start = *offset;
        let rec = Self::read_from(data, offset)?;
        if *offset - start != entry_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "FieldInfo: read {} bytes, pabgh entry is {} bytes",
                    *offset - start,
                    entry_size
                ),
            ));
        }
        Ok(rec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A missing fixture is skipped.
    const FIXTURES: &[&str] = &["fieldinfo.pabgb", "fieldinfo.dlc_01.pabgb"];

    fn load(name: &str) -> Option<(Vec<u8>, Vec<u8>)> {
        let data = std::fs::read(crate::testenv::resolve(name)).ok()?;
        let pabgh = std::fs::read(crate::testenv::resolve(&name.replace(".pabgb", ".pabgh"))).ok()?;
        Some((data, pabgh))
    }

    /// pabgh = u16 count + count x (u32 key, u32 offset); records in body order.
    fn offsets(pabgh: &[u8], body_len: usize) -> Vec<(u32, usize, usize)> {
        let count = u16::from_le_bytes([pabgh[0], pabgh[1]]) as usize;
        assert_eq!(2 + count * 8, pabgh.len(), "pabgh = u16 count + count x (u32 key, u32 offset)");
        let mut e: Vec<(u32, usize)> = (0..count)
            .map(|i| {
                let p = 2 + i * 8;
                (
                    u32::from_le_bytes(pabgh[p..p + 4].try_into().unwrap()),
                    u32::from_le_bytes(pabgh[p + 4..p + 8].try_into().unwrap()) as usize,
                )
            })
            .collect();
        e.sort_by_key(|x| x.1);
        let mut out = Vec::new();
        for i in 0..e.len() {
            let end = if i + 1 < e.len() { e[i + 1].1 } else { body_len };
            out.push((e[i].0, e[i].1, end - e[i].1));
        }
        out
    }

    #[test]
    fn roundtrip() {
        for name in FIXTURES {
            let Some((data, pabgh)) = load(name) else {
                eprintln!("SKIP: fixture {name} not found");
                continue;
            };
            let mut out = Vec::with_capacity(data.len());
            let mut json_out = Vec::with_capacity(data.len());
            for (key, off, size) in offsets(&pabgh, data.len()) {
                let mut o = off;
                let rec = FieldInfo::read_with_size(&data, &mut o, size).unwrap();
                assert_eq!(rec.key, key, "{name}: pabgh key");
                rec.write_to(&mut out).unwrap();
                FieldInfo::write_from_json_dict(&mut json_out, &rec.to_json_dict()).unwrap();
            }
            assert!(out == data, "{name}: bytes differ after round-trip");
            assert!(json_out == data, "{name}: bytes differ after JSON round-trip");
        }
    }

    #[test]
    fn sectors_match_bounds() {
        for name in FIXTURES {
            let Some((data, pabgh)) = load(name) else {
                eprintln!("SKIP: fixture {name} not found");
                continue;
            };
            for (_, off, size) in offsets(&pabgh, data.len()) {
                let mut o = off;
                let r = FieldInfo::read_with_size(&data, &mut o, size).unwrap();
                // within one 64-unit sector of the bound (exact multiples of 64
                // land one sector lower on the min side)
                let near = |sec: i32, v: f32| (sec - (v / 64.0).floor() as i32).abs() <= 1;
                assert!(near(r.start_sector_index_x, r.boundary_position_min[0]), "{name} {}", r.key);
                assert!(near(r.start_sector_index_z, r.boundary_position_min[1]), "{name} {}", r.key);
                assert!(near(r.end_sector_index_x, r.boundary_position_max[0]), "{name} {}", r.key);
                assert!(near(r.end_sector_index_z, r.boundary_position_max[1]), "{name} {}", r.key);
            }
        }
    }
}
