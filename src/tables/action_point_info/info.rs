//! Hand-corrected: typed parser for `ActionPointInfo.pabgb`, game 2.04.08.
//!
//! Replaces the `pabgh_blob_table!` stand-in. That macro reads `key` first,
//! but game 2.04 writes `string_key` FIRST (the same key-order change as
//! npcactivitygroupinfo / contentsphaseinfo), so on 2.04.08 every record
//! failed and came back as a whole-record `_blob_b64` fallback
//! (34,665 of 34,665 records).
//!
//! Decoded from the 2.04.08 game data (Steam build 26480954), not from the exe:
//! the exe holds no static-table field names, so field names are proposals
//! (the Mac catalog names `_actionPoint` / `_actionPointAccessor` members:
//! `_actionPointKey`, `_levelActionPointInfo`, `_platformDockingTagNameHash`,
//! `_platformCharacterKey`, `_levelName`, `_actionPosition`, `_actionYaw`).
//!
//! Byte layout: confirmed. Every record is `string_key` + 230 bytes (key 4,
//! is_blocked 1, then a fixed 225-byte action point), so record size varies
//! only with `string_key`. Every record round-trips byte for byte, and through
//! JSON (tests below); the .pabgh offsets match the body.
//!
//! Comment tags: `ref` = every non-zero value is a key of the named table;
//! `inferred` = meaning read from the values; `unk_` = meaning unknown (width
//! confirmed by the round-trip, split inside a run of equal bytes not).
//! Offsets in field comments count from the first byte after `is_blocked`.
//!
//! Two kinds of record:
//! - 32,860 level-placed points (`LevelActionPoint_*`, `House_*`, ...): every
//!   field below holds its default. Their string_key ends in
//!   `_<n>,<n>,<n>,<n>` and the placement lives in the level data.
//! - 1,805 platform points (`platformactionpoint_Riding_*`: ships, monorail,
//!   big mounts): `action_point_key`, `level_action_point_info`,
//!   `platform_docking_tag_name_hash`, `platform_character_info` and the
//!   `action_position` / `action_rotation` transform are set. Editing these is
//!   how a mod moves or retargets a crew/passenger spot on a vehicle.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct ActionPointInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u32,
        pub is_blocked: u8,
        // +0: equals the record's own key on all 1,805 platform records, 0 on the
        // rest (inferred: `_actionPointKey`)
        pub action_point_key: u32,
        // +4: 0 in every record
        pub unk_bytes_4: [u8; 16],
        // +20: 0xFFFFFFFF, 0xFFFFFFFF in every record (an unset key pair?)
        pub unk_u32x2_20: [u32; 2],
        // +28: 0 in every record
        pub unk_bytes_28: [u8; 24],
        // +52: ref -> levelactionpointinfo (12 of 12 distinct values; names match
        // the string_key, e.g. Action_Sailor 1000060 on `..._Action_Sailor`)
        pub level_action_point_info: u32,
        // +56: 0 in every record
        pub unk_u32_56: u32,
        // +60: hash, 7 distinct values, platform records only (inferred:
        // `_platformDockingTagNameHash`)
        pub platform_docking_tag_name_hash: u32,
        // +64: ref -> characterinfo (the vehicle: Riding_Big_pirateship_1 1001929,
        // Riding_Monorail_2025 2103, ...; 60 distinct values)
        pub platform_character_info: u32,
        // +68: 0xEAC5E173 in every record (inferred: `_levelName` lookup hash)
        pub level_name: u32,
        // +72: 0 in every record
        pub unk_bytes_72: [u8; 16],
        // +88: 0xFFFFFFFF in every record
        pub unk_u32_88: u32,
        // +92: 0 in every record
        pub unk_bytes_92: [u8; 24],
        // +116: position on the platform (x, y, z); 0 on level-placed records
        pub action_position: [f32; 3],
        // +128: rotation quaternion (x, y, z, w): unit length in every record,
        // identity on level-placed records, not identity on 1,234 platform records
        pub action_rotation: [f32; 4],
        // +144: 1.0, 1.0, 1.0 in every record (inferred: scale of the same transform)
        pub action_scale: [f32; 3],
        // +156: 0xFFFFFFFF, 0xFFFFFFFF in every record
        pub unk_u32x2_156: [u32; 2],
        // +164: 0 in every record
        pub unk_bytes_164: [u8; 24],
        // +188: 1.0 x 4 in every record
        pub unk_f32x4_188: [f32; 4],
        // +204: 0 in every record
        pub unk_bytes_204: [u8; 12],
        // +216: 1.0 in every record
        pub unk_f32_216: f32,
        // +220: 0 in every record
        pub unk_bytes_220: [u8; 5],
    }
}

impl<'a> ActionPointInfo<'a> {
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
                    "ActionPointInfo: read {} bytes, pabgh entry is {} bytes",
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

    // A missing fixture is skipped (the dlc_01 header holds 0 records).
    const FIXTURES: &[&str] = &["actionpointinfo.pabgb"];

    /// Records in pabgh order, each bounded by the next offset.
    fn read_all<'d>(data: &'d [u8], pabgh: &[u8]) -> Vec<ActionPointInfo<'d>> {
        let count = u16::from_le_bytes([pabgh[0], pabgh[1]]) as usize;
        assert_eq!(2 + count * 8, pabgh.len(), "pabgh = u16 count + count x (u32 key, u32 offset)");
        let mut offs: Vec<usize> = (0..count)
            .map(|i| {
                let p = 2 + i * 8 + 4;
                u32::from_le_bytes(pabgh[p..p + 4].try_into().unwrap()) as usize
            })
            .collect();
        offs.sort_unstable();
        offs.push(data.len());
        let mut items = Vec::with_capacity(count);
        for w in offs.windows(2) {
            let mut offset = w[0];
            items.push(ActionPointInfo::read_with_size(data, &mut offset, w[1] - w[0]).unwrap());
        }
        items
    }

    fn load(name: &str) -> Option<(Vec<u8>, Vec<u8>)> {
        let data = std::fs::read(crate::testenv::resolve(name)).ok()?;
        let pabgh = std::fs::read(crate::testenv::resolve(&name.replace(".pabgb", ".pabgh"))).ok()?;
        Some((data, pabgh))
    }

    #[test]
    fn roundtrip() {
        for name in FIXTURES {
            let Some((data, pabgh)) = load(name) else {
                eprintln!("SKIP: fixture {name} not found");
                continue;
            };
            let items = read_all(&data, &pabgh);
            let mut out = Vec::with_capacity(data.len());
            for item in &items {
                item.write_to(&mut out).unwrap();
            }
            assert_eq!(out.len(), data.len(), "{name}: length");
            assert!(out == data, "{name}: bytes differ after round-trip");
            eprintln!("{name}: {} records round-trip", items.len());
        }
    }

    #[test]
    fn json_roundtrip() {
        for name in FIXTURES {
            let Some((data, pabgh)) = load(name) else {
                eprintln!("SKIP: fixture {name} not found");
                continue;
            };
            let items = read_all(&data, &pabgh);
            let mut out = Vec::with_capacity(data.len());
            for item in &items {
                let dict = item.to_json_dict();
                ActionPointInfo::write_from_json_dict(&mut out, &dict).unwrap();
            }
            assert!(out == data, "{name}: bytes differ after JSON round-trip");
        }
    }

    #[test]
    fn pabgh_matches_body() {
        for name in FIXTURES {
            let Some((data, pabgh)) = load(name) else {
                eprintln!("SKIP: fixture {name} not found");
                continue;
            };
            let count = u16::from_le_bytes([pabgh[0], pabgh[1]]) as usize;
            let mut offset = 0usize;
            for i in 0..count {
                let p = 2 + i * 8;
                let key = u32::from_le_bytes(pabgh[p..p + 4].try_into().unwrap());
                let off = u32::from_le_bytes(pabgh[p + 4..p + 8].try_into().unwrap()) as usize;
                assert_eq!(off, offset, "{name}: entry {i} offset");
                let rec = ActionPointInfo::read_from(&data, &mut offset).unwrap();
                assert_eq!(rec.key, key, "{name}: entry {i} key");
            }
            assert_eq!(offset, data.len(), "{name}: records cover the body");
        }
    }

    #[test]
    fn platform_records() {
        for name in FIXTURES {
            let Some((data, pabgh)) = load(name) else {
                eprintln!("SKIP: fixture {name} not found");
                continue;
            };
            let items = read_all(&data, &pabgh);
            for it in &items {
                // action_point_key is either 0 or the record's own key
                assert!(it.action_point_key == 0 || it.action_point_key == it.key);
                // rotation is a unit quaternion
                let q = it.action_rotation;
                let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
                assert!((n - 1.0).abs() < 1e-3, "key {}: |q| = {n}", it.key);
            }
        }
    }
}
