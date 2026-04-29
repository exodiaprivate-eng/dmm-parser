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
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use crate::py_binary_struct;
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
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

    /// JSON shape:
    ///   { "kind": "<VariantName>", "discriminator": 0..22,
    ///     "base": <BaseUseData dict>,
    ///     "payload": <typed payload dict>  // null for base-only variants;
    ///                                       // {"_b64": "..."} for the 4 deep
    ///                                       // variants (RandomBox / Customize-
    ///                                       // Character / PlaySequencerOnly /
    ///                                       // UnSealFromEquip)
    ///   }
    /// All typed payload sub-fields are addressable (the macro generates the
    /// dict shape). Deep variants ride as base64; clone-between-entries
    /// stays byte-perfect.
    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("discriminator".to_string(), Value::from(self.discriminator()));
        let (kind, base, payload): (&str, &BaseUseData, Value) = match self {
            Self::Skill { base, payload } => ("Skill", base, Value::Object(payload.to_json_dict())),
            Self::ExpandInventorySlot { base, payload } => ("ExpandInventorySlot", base, Value::Object(payload.to_json_dict())),
            Self::RandomBox { base, payload } => ("RandomBox", base, deep_to_json(&payload.0)),
            Self::SummonGimmickWithCatch { base, payload } => ("SummonGimmickWithCatch", base, Value::Object(payload.to_json_dict())),
            Self::ConvertCharacter { base, payload } => ("ConvertCharacter", base, Value::Object(payload.to_json_dict())),
            Self::ItemDye { base, payload } => ("ItemDye", base, Value::Object(payload.to_json_dict())),
            Self::SubLevelUp { base, payload } => ("SubLevelUp", base, Value::Object(payload.to_json_dict())),
            Self::FeedVehicle { base, payload } => ("FeedVehicle", base, Value::Object(payload.to_json_dict())),
            Self::DestroyOnly { base } => ("DestroyOnly", base, Value::Null),
            Self::SealToEquip { base } => ("SealToEquip", base, Value::Null),
            Self::TeleportRevivePoint { base, payload } => ("TeleportRevivePoint", base, Value::Object(payload.to_json_dict())),
            Self::Projectile { base, payload } => ("Projectile", base, Value::Object(payload.to_json_dict())),
            Self::ExpandFarmSlot { base, payload } => ("ExpandFarmSlot", base, Value::Object(payload.to_json_dict())),
            Self::CustomizeCharacter { base, payload } => ("CustomizeCharacter", base, deep_to_json(&payload.0)),
            Self::PlaySequencerOnly { base, payload } => ("PlaySequencerOnly", base, deep_to_json(&payload.0)),
            Self::RegisterReserveSlot { base, payload } => ("RegisterReserveSlot", base, Value::Object(payload.to_json_dict())),
            Self::OpenUI { base, payload } => ("OpenUI", base, Value::Object(payload.to_json_dict())),
            Self::Inspect { base } => ("Inspect", base, Value::Null),
            Self::InventoryBuff { base } => ("InventoryBuff", base, Value::Null),
            Self::SendEventToDockingGimmick { base } => ("SendEventToDockingGimmick", base, Value::Null),
            Self::UseSealed { base } => ("UseSealed", base, Value::Null),
            Self::UnSealFromEquip { base, payload } => ("UnSealFromEquip", base, deep_to_json(&payload.0)),
            Self::SpecialMode { base, payload } => ("SpecialMode", base, Value::Object(payload.to_json_dict())),
        };
        m.insert("kind".to_string(), Value::String(kind.to_string()));
        m.insert("base".to_string(), Value::Object(base.to_json_dict()));
        m.insert("payload".to_string(), payload);
        m
    }

    /// Write the variant's bytes from a JSON dict (as produced by
    /// `to_json_dict`). The discriminator byte is NOT emitted here —
    /// `ItemUseInfo::write_from_json_dict` writes it before calling us.
    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<u8> {
        let disc = json_get_field(obj, "discriminator")?
            .as_u64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "ItemUseDataVariant.discriminator: expected u8 number"))?;
        if disc > 22 {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("ItemUseDataVariant.discriminator out of range: {}", disc)));
        }
        let disc = disc as u8;
        let base_obj = json_get_field(obj, "base")?
            .as_object()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "ItemUseDataVariant.base: expected object"))?;
        BaseUseData::write_from_json_dict(w, base_obj)?;
        let payload = json_get_field(obj, "payload")?;
        match disc {
            0 => SkillPayload::write_from_json_dict(w, payload_obj(payload, "Skill")?)?,
            1 => ExpandInventorySlotPayload::write_from_json_dict(w, payload_obj(payload, "ExpandInventorySlot")?)?,
            2 => write_deep_from_json(w, payload, "RandomBox")?,
            3 => SummonGimmickWithCatchPayload::write_from_json_dict(w, payload_obj(payload, "SummonGimmickWithCatch")?)?,
            4 => ConvertCharacterPayload::write_from_json_dict(w, payload_obj(payload, "ConvertCharacter")?)?,
            5 => ItemDyePayload::write_from_json_dict(w, payload_obj(payload, "ItemDye")?)?,
            6 => SubLevelUpPayload::write_from_json_dict(w, payload_obj(payload, "SubLevelUp")?)?,
            7 => SkillPayload::write_from_json_dict(w, payload_obj(payload, "FeedVehicle")?)?,
            8 | 9 => {} // base only
            10 => TeleportRevivePointPayload::write_from_json_dict(w, payload_obj(payload, "TeleportRevivePoint")?)?,
            11 => ProjectilePayload::write_from_json_dict(w, payload_obj(payload, "Projectile")?)?,
            12 => ConvertCharacterPayload::write_from_json_dict(w, payload_obj(payload, "ExpandFarmSlot")?)?,
            13 => write_deep_from_json(w, payload, "CustomizeCharacter")?,
            14 => write_deep_from_json(w, payload, "PlaySequencerOnly")?,
            15 => RegisterReserveSlotPayload::write_from_json_dict(w, payload_obj(payload, "RegisterReserveSlot")?)?,
            16 => OpenUIPayload::write_from_json_dict(w, payload_obj(payload, "OpenUI")?)?,
            17 | 18 | 19 | 20 => {} // base only
            21 => write_deep_from_json(w, payload, "UnSealFromEquip")?,
            22 => SpecialModePayload::write_from_json_dict(w, payload_obj(payload, "SpecialMode")?)?,
            _ => unreachable!("disc {} bounds-checked above", disc),
        }
        Ok(disc)
    }
}

fn deep_to_json(bytes: &[u8]) -> Value {
    let mut m = Map::new();
    m.insert("_b64".to_string(), Value::String(B64.encode(bytes)));
    Value::Object(m)
}

fn payload_obj<'v>(v: &'v Value, kind: &str) -> io::Result<&'v Map<String, Value>> {
    v.as_object().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
        format!("ItemUseDataVariant::{}: payload must be an object", kind)))
}

fn write_deep_from_json(w: &mut Vec<u8>, v: &Value, kind: &str) -> io::Result<()> {
    let obj = payload_obj(v, kind)?;
    let s = json_get_field(obj, "_b64")?
        .as_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
            format!("ItemUseDataVariant::{}: _b64 must be a string", kind)))?;
    let bytes = B64.decode(s).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
        format!("ItemUseDataVariant::{}: _b64 invalid base64: {}", kind, e)))?;
    w.extend_from_slice(&bytes);
    Ok(())
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

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("variant".to_string(), Value::Object(self.variant.to_json_dict()));
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        let variant_obj = json_get_field(obj, "variant")?
            .as_object()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "ItemUseInfo.variant: expected object"))?;
        // Discriminator byte goes between is_blocked and the variant body
        // (matches the wire shape that read_with_size pulls back).
        let mut variant_buf = Vec::new();
        let disc = ItemUseDataVariant::write_from_json_dict(&mut variant_buf, variant_obj)?;
        w.push(disc);
        w.extend_from_slice(&variant_buf);
        Ok(())
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

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(PABGB_PATH) else {
            eprintln!("SKIP: missing pabgb fixture {}", PABGB_PATH);
            return;
        };
        let Ok(pabgh) = std::fs::read(PABGH_PATH) else {
            eprintln!("SKIP: missing pabgh fixture {}", PABGH_PATH);
            return;
        };

        let entries = parse_pabgh(&pabgh);
        for i in 0..entries.len() {
            let off = entries[i].1;
            let next_off = if i + 1 < entries.len() { entries[i + 1].1 } else { data.len() };
            let entry_size = next_off - off;
            let mut cursor = off;
            let item = ItemUseInfo::read_with_size(&data, &mut cursor, entry_size).unwrap();
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            ItemUseInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!(
                    "entry {} key=0x{:x} disc={}: write_from_json_dict: {}",
                    i, entries[i].0, item.variant.discriminator(), e
                ));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x} disc={}: JSON round-trip diverges from typed write",
                i, entries[i].0, item.variant.discriminator()
            );
        }
    }
}
