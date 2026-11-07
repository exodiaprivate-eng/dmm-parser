//! Hand-corrected: typed parser for `LevelGimmickSceneObjectInfo.pabgb`,
//! game 2.04.08.
//!
//! The 1.18 typed-blob layout (see `src/tables/LEVELGIMMICK_112_RE.md`) reads
//! `key` first, but game 2.04 writes `string_key` FIRST (the same key-order
//! change as npcactivitygroupinfo / contentsphaseinfo / actionpointinfo), so on
//! 2.04.08 every record failed and came back as a whole-record `_blob_b64`
//! fallback (249 of 249 base records, 79 of 79 dlc_01 records).
//!
//! 2.04 also reshaped the record: the per-record placement list
//! (`_levelGimmickSceneObjectDataList`) is no longer in this table, the
//! map-icon / distance scalars come first, and `level_name` moved behind
//! `guide_effect_socket_name`. Decoded from the 2.04.08 game data (Steam build
//! 26480954), not from the exe; names reuse the 1.18 canonical names where the
//! values fit.
//!
//! Byte layout: confirmed. A fixed 65-byte head (key 4, is_blocked 1, 60 bytes
//! of scalars), `ore_vein_index`, two CStrings and a 13-byte-element list.
//! Every record round-trips byte for byte, and through JSON (tests below);
//! the .pabgh offsets match the body.
//!
//! Comment tags: `C` = confirmed by the values / record names; `inferred` =
//! meaning read from the values; `unk_` = meaning unknown (width confirmed by
//! the round-trip). Offsets in field comments count from the first byte after
//! `is_blocked`.

use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    /// 13-byte element of `unk_entry_list` (same wire shape as
    /// `GlobalGameEventListEntry`: u32, u8, u32, u32). Present only on the 18
    /// dlc_01 `Management_Structure_*` records (2 entries each).
    pub struct LevelGimmickSceneObjectEntry {
        // a key in the 1,000,000+ range (1000064, 1001228, 1015521, ...)
        pub unk_key: u32,
        pub unk_u8: u8,
        // hash-like value
        pub unk_hash: u32,
        pub unk_u32: u32,
    }
}

py_binary_struct! {
    pub struct LevelGimmickSceneObjectInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u32,
        pub is_blocked: u8,
        // +0..+9: u8 flags (0/1), except +2
        pub unk_u8_0: u8,
        pub unk_u8_1: u8,
        // +2: enum 0..9 (5 x145, 9 x108, 6 x27, 7 x18, ...)
        pub unk_u8_2: u8,
        pub unk_u8_3: u8,
        pub unk_u8_4: u8,
        pub unk_u8_5: u8,
        pub unk_u8_6: u8,
        pub unk_u8_7: u8,
        pub unk_u8_8: u8,
        pub unk_u8_9: u8,
        // +10: ContentsPhaseKey, u16 (C: set only on 9 *MovePoint / TeleportPoint
        // records, e.g. HousingMovePoint 16960, ManagementMovePoint_Pub 16968)
        pub contents_phase_info_for_move_point: u16,
        // +12..+32: six u32 keys in the 5,000-6,000 range (inferred:
        // uimaptextureinfo keys = the 1.18 map-icon fields `_mapIconTextureInfo`,
        // `_fogMapIconTextureInfo`, `_overAbyss*`, `_completedDiscover*`;
        // which is which not confirmed). +16 is the common one (216 records).
        pub map_texture_info_12: u32,
        pub map_texture_info_16: u32,
        pub map_texture_info_20: u32,
        pub map_texture_info_24: u32,
        pub map_texture_info_28: u32,
        pub map_texture_info_32: u32,
        // +36: small enum (0, 1, 2, 3, 9, 16) (inferred: `_discoverType`)
        pub discover_type: u32,
        // +40: f32 (0, 2, 5, 6, 8, 25) (inferred: a distance)
        pub unk_f32_40: f32,
        // +44: hash on 5 records (SulfurStone_01..04, Mercury), 0 elsewhere
        // (inferred: `_discoverGimmickStateHash`)
        pub discover_gimmick_state_hash: u32,
        // +48, +52, +56: f32 (0, 10, 30 / 0, 50 / 0, 50) (inferred: distances)
        pub unk_f32_48: f32,
        pub unk_f32_52: f32,
        pub unk_f32_56: f32,
        // +60: -1 except the Vein_Minerals_*_Levelindex_NN records, where it is
        // NN (C: `_oreVeinIndex`)
        pub ore_vein_index: i32,
        // +64: "GuideFXSocket_01" on 18 records, empty elsewhere
        // (C: `_guideEffectSocketName`)
        pub guide_effect_socket_name: CString<'a>,
        // 1 on SubInner_Dock_0001 / _0002 only (inferred: `_isSubInnerGimmick`)
        pub is_sub_inner_gimmick: u8,
        // level / area name ("Faction", "AbyssIsland", "rootlevel/SectorRootLevel",
        // "Management_DLC01", ...); empty on 80 records (inferred: `_levelName`)
        pub level_name: CString<'a>,
        // 0 in every record
        pub unk_u32_tail: u32,
        pub unk_entry_list: CArray<LevelGimmickSceneObjectEntry>,
        // 1 + 1013697 on SaveSearch_ShinyStone only, 0 elsewhere
        pub unk_u8_tail: u8,
        pub unk_key_tail: u32,
    }
}

impl<'a> LevelGimmickSceneObjectInfo<'a> {
    /// pabgh-bounded read (dispatcher `p!`): the typed read must end exactly on
    /// the record boundary, otherwise the dispatcher keeps the record as a
    /// `_blob_b64` fallback.
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
    ) -> std::io::Result<Self> {
        let start = *offset;
        let end = start.checked_add(entry_size).filter(|e| *e <= data.len()).ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "LevelGimmickSceneObjectInfo: entry past end of data")
        })?;
        // Bound the read to this entry so a bad length cannot run into the next one.
        let rec = Self::read_from(&data[..end], offset)?;
        let local = *offset - start;
        if local != entry_size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "LevelGimmickSceneObjectInfo: read {} bytes, pabgh entry is {} bytes",
                    local, entry_size
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
    const FIXTURES: &[&str] = &[
        "levelgimmicksceneobjectinfo.pabgb",
        "levelgimmicksceneobjectinfo.dlc_01.pabgb",
    ];

    fn load(name: &str) -> Option<(Vec<u8>, Vec<u8>)> {
        let data = std::fs::read(crate::testenv::resolve(name)).ok()?;
        let pabgh = std::fs::read(crate::testenv::resolve(&name.replace(".pabgb", ".pabgh"))).ok()?;
        Some((data, pabgh))
    }

    /// pabgh = u32 count + count x (u32 key, u32 offset); records in body order.
    fn offsets(pabgh: &[u8], body_len: usize) -> Vec<(u32, usize, usize)> {
        let count = u32::from_le_bytes(pabgh[0..4].try_into().unwrap()) as usize;
        assert_eq!(4 + count * 8, pabgh.len(), "pabgh = u32 count + count x (u32 key, u32 offset)");
        let mut e: Vec<(u32, usize)> = (0..count)
            .map(|i| {
                let p = 4 + i * 8;
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
                let rec = LevelGimmickSceneObjectInfo::read_with_size(&data, &mut o, size).unwrap();
                assert_eq!(rec.key, key, "{name}: pabgh key");
                rec.write_to(&mut out).unwrap();
                LevelGimmickSceneObjectInfo::write_from_json_dict(&mut json_out, &rec.to_json_dict()).unwrap();
            }
            assert!(out == data, "{name}: bytes differ after round-trip");
            assert!(json_out == data, "{name}: bytes differ after JSON round-trip");
        }
    }

    #[test]
    fn named_fields_match_record_names() {
        for name in FIXTURES {
            let Some((data, pabgh)) = load(name) else {
                eprintln!("SKIP: fixture {name} not found");
                continue;
            };
            for (_, off, size) in offsets(&pabgh, data.len()) {
                let mut o = off;
                let r = LevelGimmickSceneObjectInfo::read_with_size(&data, &mut o, size).unwrap();
                let sk = std::str::from_utf8(r.string_key.data.as_bytes()).unwrap().to_string();
                if let Some(nn) = sk.rsplit("_Levelindex_").next().filter(|_| sk.contains("_Levelindex_")) {
                    assert_eq!(r.ore_vein_index, nn.parse::<i32>().unwrap(), "{sk}");
                } else {
                    assert_eq!(r.ore_vein_index, -1, "{sk}");
                }
                if r.contents_phase_info_for_move_point != 0 {
                    assert!(sk.contains("MovePoint") || sk.contains("TeleportPoint"), "{sk}");
                }
            }
        }
    }
}
