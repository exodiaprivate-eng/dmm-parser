//! Hand-corrected: IDA-derived polymorphic parser for `ItemUseInfo.pabgb`.
//!
//! ItemUseInfo entries have a polymorphic `item_use_info_data` payload
//! dispatched on a u8 discriminator. There are 23 known concrete
//! `pa::ItemUseData_*` variants (case 0..=22 in `sub_141E43400`); 19 of
//! them appear in vanilla itemuseinfo.pabgb (5, 11, 21, 22 have 0 entries).
//!
//! Schema strategy:
//!   - All variants share a `BaseUseData` prefix (`sub_141E44520`):
//!     22 fixed bytes + a `CArray<{u32 + u8 + LocalizableString}>`.
//!   - 18 of the 23 add 0..12 bytes of fixed/CString-bearing fields after
//!     the base. These are fully decoded into named fields.
//!   - 3 are deeply nested polymorphic readers (Disc 2 RandomBox,
//!     Disc 13 CustomizeCharacter, Disc 14 PlaySequencerOnly) whose
//!     payloads require further recursive variant infrastructure (each
//!     contains its own switch dispatcher with its own case tree). For
//!     these we capture the post-base payload as `Vec<u8>` so the
//!     round-trip is byte-perfect; semantic decoding of those three is
//!     deferred.
//!
//! The pabgh sister file is consulted to know each entry's total size on
//! disk, which is the only way to bound the variant payload reliably for
//! the complex variants without implementing every recursive sub-reader.
//!
//! IDA function map (v1.0.4.x, current Win exe):
//!   0  Skill                    sub_141E44BB0  base + sub_1410FEBE0 + u32
//!   1  ExpandInventorySlot      sub_141E44F00  base + sub_141103F00 + u16
//!   2  RandomBox                sub_141E45240  base + bool + opt(u32-hash) + bool + opt(sub_141D03AA0) + u32-hash  [DEEP]
//!   3  SummonGimmickWithCatch   sub_141E45680  base + u32 + CString + CString + u32 + u32
//!   4  ConvertCharacter         sub_141E45B70  base + u8
//!   5  ItemDye                  sub_141E45C10  base + u32 + u8                   (0 entries in vanilla)
//!   6  SubLevelUp               sub_141E45CD0  base + sub_141103770
//!   7  FeedVehicle              sub_141E44BB0  same as Skill
//!   8  DestroyOnly              sub_141E44520  base only
//!   9  SealToEquip              sub_141E44520  base only
//!   10 TeleportRevivePoint      sub_141E463C0  base + u32 + u32                  (sub_14FDCEC60 thunk)
//!   11 Projectile               sub_141E458F0  base + u32 + u32                   (0 entries in vanilla)
//!   12 ExpandFarmSlot           sub_141E45B70  same as ConvertCharacter
//!   13 CustomizeCharacter       sub_141E466D0  base + sub_1410FF340 + 2*CArray<u16-hash + u8>  [DEEP]
//!   14 PlaySequencerOnly        sub_141E45ED0  base + sub_141D8C6D0 — 4 nested polymorphic CArrays  [DEEPEST]
//!   15 RegisterReserveSlot      sub_141E46C70  base + sub_141102C40 + u8
//!   16 OpenUI                   sub_141E46E20  base + u8 + CString
//!   17 Inspect                  sub_141E46EA0  base only
//!   18 InventoryBuff            sub_141E46EA0  base only
//!   19 SendEventToDockingGimmick sub_141E44520 base only
//!   20 UseSealed                sub_141E44520  base only
//!   21 UnSealFromEquip          sub_141E44520  base only                          (0 entries in vanilla)
//!   22 SpecialMode              sub_141E44FB0  base + u32                         (0 entries in vanilla)
//!
//! All "u32-hash" reads emit a u32 from stream that the game runtime resolves
//! into a u16 via dictionary lookup; for round-trip we keep the u32 file
//! representation.
//!
//! DO NOT REGENERATE. This file is hand-written; bulk_process.py guards
//! it via the "Hand-corrected" header marker on line 1.

use crate::binary::*;
use crate::py_binary_struct;
use std::io::{self, Write};

// ─────────────────────────────────────────────────────────────────────────────
// Shared base (sub_141E44520)
// ─────────────────────────────────────────────────────────────────────────────

py_binary_struct! {
    pub struct BaseUseDataElem<'a> {
        pub key_lookup: u32,
        pub flag: u8,
        pub label: LocalizableString<'a>,
    }
}

py_binary_struct! {
    pub struct BaseUseData<'a> {
        pub flag_a: u8,
        pub flag_b: u8,
        pub flag_c: u8,
        pub flag_d: u8,
        pub group_lookup_a: u32,
        pub group_lookup_b: u32,
        pub flag_e: u8,
        pub flag_f: u8,
        pub group_id: u32,
        pub elements: CArray<BaseUseDataElem<'a>>,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Per-variant payload structs (after the base prefix)
// ─────────────────────────────────────────────────────────────────────────────

py_binary_struct! {
    pub struct SkillPayload {
        pub skill_lookup: u32,
        pub level: u32,
    }
}

py_binary_struct! {
    pub struct ExpandInventorySlotPayload {
        pub slot_count_lookup: u16,
        pub extra: u16,
    }
}

py_binary_struct! {
    pub struct SummonGimmickWithCatchPayload<'a> {
        pub gimmick_lookup: u32,
        pub catch_name: CString<'a>,
        pub spawn_tag: CString<'a>,
        pub flags_a: u32,
        pub flags_b: u32,
    }
}

py_binary_struct! {
    pub struct ConvertCharacterPayload {
        pub flag: u8,
    }
}

py_binary_struct! {
    pub struct ItemDyePayload {
        pub dye_lookup: u32,
        pub flag: u8,
    }
}

py_binary_struct! {
    pub struct SubLevelUpPayload {
        pub level_lookup: u32,
    }
}

py_binary_struct! {
    pub struct TeleportRevivePointPayload {
        pub point_id_a: u32,
        pub point_id_b: u32,
    }
}

py_binary_struct! {
    pub struct ProjectilePayload {
        pub projectile_a: u32,
        pub projectile_b: u32,
    }
}

py_binary_struct! {
    pub struct RegisterReserveSlotPayload {
        pub slot_lookup: u32,
        pub flag: u8,
    }
}

py_binary_struct! {
    pub struct OpenUIPayload<'a> {
        pub flag: u8,
        pub ui_name: CString<'a>,
    }
}

py_binary_struct! {
    pub struct UnSealFromEquipPayload {
        pub flag: u8,
        pub raw_extra: [u8; 8],
    }
}

py_binary_struct! {
    pub struct SpecialModePayload {
        pub mode_lookup: u32,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Deep variants — payload kept as opaque bytes (round-trip safe).
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct DeepVariantPayload(pub Vec<u8>);

// ─────────────────────────────────────────────────────────────────────────────
// Top-level variant enum
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum ItemUseDataVariant<'a> {
    Skill { base: BaseUseData<'a>, payload: SkillPayload },
    ExpandInventorySlot { base: BaseUseData<'a>, payload: ExpandInventorySlotPayload },
    RandomBox { base: BaseUseData<'a>, payload: DeepVariantPayload },
    SummonGimmickWithCatch { base: BaseUseData<'a>, payload: SummonGimmickWithCatchPayload<'a> },
    ConvertCharacter { base: BaseUseData<'a>, payload: ConvertCharacterPayload },
    ItemDye { base: BaseUseData<'a>, payload: ItemDyePayload },
    SubLevelUp { base: BaseUseData<'a>, payload: SubLevelUpPayload },
    FeedVehicle { base: BaseUseData<'a>, payload: SkillPayload },
    DestroyOnly { base: BaseUseData<'a> },
    SealToEquip { base: BaseUseData<'a> },
    TeleportRevivePoint { base: BaseUseData<'a>, payload: TeleportRevivePointPayload },
    Projectile { base: BaseUseData<'a>, payload: ProjectilePayload },
    ExpandFarmSlot { base: BaseUseData<'a>, payload: ConvertCharacterPayload },
    CustomizeCharacter { base: BaseUseData<'a>, payload: DeepVariantPayload },
    PlaySequencerOnly { base: BaseUseData<'a>, payload: DeepVariantPayload },
    RegisterReserveSlot { base: BaseUseData<'a>, payload: RegisterReserveSlotPayload },
    OpenUI { base: BaseUseData<'a>, payload: OpenUIPayload<'a> },
    Inspect { base: BaseUseData<'a> },
    InventoryBuff { base: BaseUseData<'a> },
    SendEventToDockingGimmick { base: BaseUseData<'a> },
    UseSealed { base: BaseUseData<'a> },
    UnSealFromEquip { base: BaseUseData<'a>, payload: DeepVariantPayload },
    SpecialMode { base: BaseUseData<'a>, payload: SpecialModePayload },
}

impl<'a> ItemUseDataVariant<'a> {
    pub fn discriminator(&self) -> u8 {
        match self {
            Self::Skill { .. } => 0,
            Self::ExpandInventorySlot { .. } => 1,
            Self::RandomBox { .. } => 2,
            Self::SummonGimmickWithCatch { .. } => 3,
            Self::ConvertCharacter { .. } => 4,
            Self::ItemDye { .. } => 5,
            Self::SubLevelUp { .. } => 6,
            Self::FeedVehicle { .. } => 7,
            Self::DestroyOnly { .. } => 8,
            Self::SealToEquip { .. } => 9,
            Self::TeleportRevivePoint { .. } => 10,
            Self::Projectile { .. } => 11,
            Self::ExpandFarmSlot { .. } => 12,
            Self::CustomizeCharacter { .. } => 13,
            Self::PlaySequencerOnly { .. } => 14,
            Self::RegisterReserveSlot { .. } => 15,
            Self::OpenUI { .. } => 16,
            Self::Inspect { .. } => 17,
            Self::InventoryBuff { .. } => 18,
            Self::SendEventToDockingGimmick { .. } => 19,
            Self::UseSealed { .. } => 20,
            Self::UnSealFromEquip { .. } => 21,
            Self::SpecialMode { .. } => 22,
        }
    }

    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        disc: u8,
        payload_size: usize,
    ) -> io::Result<Self> {
        let payload_end = *offset + payload_size;
        let base = BaseUseData::read_from(data, offset)?;
        let extra_size = payload_end.saturating_sub(*offset);

        let result = match disc {
            0 => Self::Skill { base, payload: SkillPayload::read_from(data, offset)? },
            1 => Self::ExpandInventorySlot {
                base,
                payload: ExpandInventorySlotPayload::read_from(data, offset)?,
            },
            2 => Self::RandomBox {
                base,
                payload: read_deep_payload(data, offset, extra_size)?,
            },
            3 => Self::SummonGimmickWithCatch {
                base,
                payload: SummonGimmickWithCatchPayload::read_from(data, offset)?,
            },
            4 => Self::ConvertCharacter {
                base,
                payload: ConvertCharacterPayload::read_from(data, offset)?,
            },
            5 => Self::ItemDye { base, payload: ItemDyePayload::read_from(data, offset)? },
            6 => Self::SubLevelUp { base, payload: SubLevelUpPayload::read_from(data, offset)? },
            7 => Self::FeedVehicle { base, payload: SkillPayload::read_from(data, offset)? },
            8 => Self::DestroyOnly { base },
            9 => Self::SealToEquip { base },
            10 => Self::TeleportRevivePoint {
                base,
                payload: TeleportRevivePointPayload::read_from(data, offset)?,
            },
            11 => Self::Projectile { base, payload: ProjectilePayload::read_from(data, offset)? },
            12 => Self::ExpandFarmSlot {
                base,
                payload: ConvertCharacterPayload::read_from(data, offset)?,
            },
            13 => Self::CustomizeCharacter {
                base,
                payload: read_deep_payload(data, offset, extra_size)?,
            },
            14 => Self::PlaySequencerOnly {
                base,
                payload: read_deep_payload(data, offset, extra_size)?,
            },
            15 => Self::RegisterReserveSlot {
                base,
                payload: RegisterReserveSlotPayload::read_from(data, offset)?,
            },
            16 => Self::OpenUI { base, payload: OpenUIPayload::read_from(data, offset)? },
            17 => Self::Inspect { base },
            18 => Self::InventoryBuff { base },
            19 => Self::SendEventToDockingGimmick { base },
            20 => Self::UseSealed { base },
            21 => Self::UnSealFromEquip {
                base,
                payload: read_deep_payload(data, offset, extra_size)?,
            },
            22 => Self::SpecialMode {
                base,
                payload: SpecialModePayload::read_from(data, offset)?,
            },
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unknown ItemUseData discriminator: {}", disc),
                ));
            }
        };
        Ok(result)
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match self {
            Self::Skill { base, payload } => { base.write_to(w)?; payload.write_to(w) }
            Self::ExpandInventorySlot { base, payload } => { base.write_to(w)?; payload.write_to(w) }
            Self::RandomBox { base, payload } => { base.write_to(w)?; w.write_all(&payload.0) }
            Self::SummonGimmickWithCatch { base, payload } => { base.write_to(w)?; payload.write_to(w) }
            Self::ConvertCharacter { base, payload } => { base.write_to(w)?; payload.write_to(w) }
            Self::ItemDye { base, payload } => { base.write_to(w)?; payload.write_to(w) }
            Self::SubLevelUp { base, payload } => { base.write_to(w)?; payload.write_to(w) }
            Self::FeedVehicle { base, payload } => { base.write_to(w)?; payload.write_to(w) }
            Self::DestroyOnly { base } => base.write_to(w),
            Self::SealToEquip { base } => base.write_to(w),
            Self::TeleportRevivePoint { base, payload } => { base.write_to(w)?; payload.write_to(w) }
            Self::Projectile { base, payload } => { base.write_to(w)?; payload.write_to(w) }
            Self::ExpandFarmSlot { base, payload } => { base.write_to(w)?; payload.write_to(w) }
            Self::CustomizeCharacter { base, payload } => { base.write_to(w)?; w.write_all(&payload.0) }
            Self::PlaySequencerOnly { base, payload } => { base.write_to(w)?; w.write_all(&payload.0) }
            Self::RegisterReserveSlot { base, payload } => { base.write_to(w)?; payload.write_to(w) }
            Self::OpenUI { base, payload } => { base.write_to(w)?; payload.write_to(w) }
            Self::Inspect { base } => base.write_to(w),
            Self::InventoryBuff { base } => base.write_to(w),
            Self::SendEventToDockingGimmick { base } => base.write_to(w),
            Self::UseSealed { base } => base.write_to(w),
            Self::UnSealFromEquip { base, payload } => { base.write_to(w)?; w.write_all(&payload.0) }
            Self::SpecialMode { base, payload } => { base.write_to(w)?; payload.write_to(w) }
        }
    }
}

fn read_deep_payload<'a>(
    data: &'a [u8],
    offset: &mut usize,
    size: usize,
) -> io::Result<DeepVariantPayload> {
    if *offset + size > data.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("deep payload size {} runs past end of data", size),
        ));
    }
    let bytes = data[*offset..*offset + size].to_vec();
    *offset += size;
    Ok(DeepVariantPayload(bytes))
}

// ─────────────────────────────────────────────────────────────────────────────
// ItemUseInfo top-level
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct ItemUseInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub variant: ItemUseDataVariant<'a>,
}

impl<'a> ItemUseInfo<'a> {
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
    ) -> io::Result<Self> {
        let start = *offset;
        let key = u32::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;
        let disc = u8::read_from(data, offset)?;
        let entry_end = start + entry_size;
        let payload_size = entry_end - *offset;
        let variant = ItemUseDataVariant::read_with_size(data, offset, disc, payload_size)?;
        Ok(ItemUseInfo { key, string_key, is_blocked, variant })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.variant.discriminator().write_to(w)?;
        self.variant.write_to(w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PABGB_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\itemuseinfo.pabgb";
    const PABGH_PATH: &str = r"C:\\Users\\corin\\Desktop\\CD DUMPING TOOLS\\dmm-pabgb-aio\\vanilla_dumps\\itemuseinfo.pabgh";

    fn parse_pabgh(pabgh: &[u8]) -> Vec<(u32, usize)> {
        let count = u32::from_le_bytes(pabgh[..4].try_into().unwrap()) as usize;
        let mut entries = Vec::with_capacity(count);
        for i in 0..count {
            let pos = 4 + i * 8;
            let key = u32::from_le_bytes(pabgh[pos..pos + 4].try_into().unwrap());
            let off = u32::from_le_bytes(pabgh[pos + 4..pos + 8].try_into().unwrap()) as usize;
            entries.push((key, off));
        }
        entries.sort_by_key(|e| e.1);
        entries
    }

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing pabgb fixture {}", PABGB_PATH);
            return;
        };
        let Ok(pabgh) = std::fs::read(PABGH_PATH) else {
            eprintln!("SKIP: missing pabgh fixture {}", PABGH_PATH);
            return;
        };

        let entries = parse_pabgh(&pabgh);
        let mut items = Vec::with_capacity(entries.len());
        for i in 0..entries.len() {
            let off = entries[i].1;
            let next_off = if i + 1 < entries.len() {
                entries[i + 1].1
            } else {
                data.len()
            };
            let entry_size = next_off - off;
            let mut cursor = off;
            let item = ItemUseInfo::read_with_size(&data, &mut cursor, entry_size)
                .unwrap_or_else(|e| {
                    panic!(
                        "parse failed at entry {} (key=0x{:x}, offset 0x{:x}, size {}): {}",
                        i, entries[i].0, off, entry_size, e
                    )
                });
            assert_eq!(
                cursor,
                off + entry_size,
                "entry {} (disc {}) under/over-consumed: read {} bytes, expected {}",
                i, item.variant.discriminator(), cursor - off, entry_size
            );
            items.push(item);
        }

        let mut out = Vec::with_capacity(data.len());
        for item in &items {
            item.write_to(&mut out).unwrap();
        }
        assert_eq!(out.len(), data.len(), "itemuseinfo roundtrip size mismatch");
        assert_eq!(out, data, "itemuseinfo roundtrip bytes mismatch");
    }
}
