// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Typed decoder for the per-element payload of `sub_141102760`, the
//! `CArray<COptional<sub_141600210>>` wire shared by `DropSetInfo._list`
//! and `ItemUseInfo` RandomBox `inner_data` (sub_141D03AA0 wraps the
//! same 128-byte allocation + sub_141600210 reader).
//!
//! ## Wire layout (sub_141600210)
//!
//! Per element when present (after the u8 presence byte from
//! sub_141D03AA0): 63 fixed bytes + variant tail.
//!
//! Fixed prefix (in wire order; mem offsets shown for cross-reference
//! against the IDA decompile):
//!   1.  u64 raw_at_120         (wire 0..7,  mem +120)
//!   2.  u8  dispatch_tag       (wire 8,     mem +112)
//!   3.  u32 lookup_4           (wire 9..12, mem +4,  sub_1410FF430)
//!   4.  u32 lookup_6           (wire 13..16, mem +6,  sub_1410FF430)
//!   5.  u32 lookup_8           (wire 17..20, mem +8,  sub_1410FF430)
//!   6.  u32 raw_12             (wire 21..24, mem +12)
//!   7.  u64 raw_16             (wire 25..32, mem +16)
//!   8.  [REMOVED in 1.18.00] u32 raw_24 / `rates_100` (was wire 33..36, mem +24)
//!   9.  u64 raw_32             (wire 37..44, mem +32)
//!  10.  u64 raw_40             (wire 45..52, mem +40)
//!  11.  u64 raw_48             (wire 53..60, mem +48)
//!  12.  u16 raw_56             (wire 61..62, mem +56)
//!
//! Variant tail (dispatched on `dispatch_tag` from step 2):
//!   tag 0       : u32 (sub_1410FF5C0,  qword_DA00 hash)
//!   tag 1, 2, 3 : u32 (sub_1410FF340,  qword_DA08 hash)
//!   tag 4       : u32 (sub_1411006D0,  qword_DA28 hash)
//!   tag 5       : u32 (sub_1411060F0,  qword_15030 hash)
//!   tag 6       : u32 (sub_141103770)
//!   tag 7, 8    : 32-byte DropTargetItemRef (sub_1410DB4C0)
//!   tag 9       : u32 (sub_141100740,  qword_DA38 hash)
//!   tag A       : u32 + u32 (sub_141100860 + sub_1410FF340)
//!   tag B       : 0 bytes (pure discriminator)
//!   tag C       : u32 (sub_1410FEBE0)
//!   tag D       : u32 + u8 (sub_141102E00 + raw u8)
//!
//! All u32 lookups in the variant tail are u32-wire / u16-mem hashes
//! (4 wire bytes each). Modeling them as `u32` preserves the wire
//! value losslessly for round-trip.

use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use crate::py_binary_struct;
use serde_json::{Map, Value};
use std::io::{self, Write};

py_binary_struct! {
    /// Tag 7 / Tag 8 payload for `DropTargetVariant`. 32 wire bytes
    /// (sub_1410DB4C0 inner). Field names follow the wire order in
    /// sub_1410DB4C0; mem offsets are documented in the master module
    /// docstring.
    pub struct DropTargetItemRef {
        pub flag_a: u8,        // 1 byte at  mem +8
        pub raw_b: u64,        // 8 bytes at mem +16
        pub lookup_c: u32,     // sub_141100860 (u32 wire / u16 mem)
        pub lookup_d: u32,     // sub_1410FF340 (u32 wire / u16 mem)
        pub flag_e: u8,        // 1 byte at  mem +28
        pub lookup_f: u32,     // sub_1411026F0 (u32 wire / u16 mem)
        pub raw_g: u64,        // 8 bytes at mem +32
        pub flag_h: u8,        // 1 byte at  mem +40
        pub flag_i: u8,        // 1 byte at  mem +41
    }
}

py_binary_struct! {
    /// 1.18.00 tag 16 (0x10) payload — the exe's `DropRegistLivingCharacterData`.
    ///
    /// ★ Field split taken from the MAC 1.18 reader `sub_101FD06E8`, not guessed:
    /// ```text
    ///   sub_101F68128(a1, &v15)      -> int, then a pa::CharacterKey lookup
    ///                                   stored as u16 at mem +8   = _characterInfo
    ///   sub_100E26380(a1, a2 + 10)   -> 1 byte at mem +10          = _toTargetActor
    ///   sub_100E26380(a1, a2 + 11)   -> 1 byte at mem +11          = _isRegister
    /// ```
    /// u32 + u8 + u8 = 6 bytes, matching the width proven from the wire.
    /// `character_info` is the usual "u32 wire → u16 mem" key pattern.
    ///
    /// ⚠ The field-name ORACLE lists these as toTargetActor, isRegister,
    /// characterInfo — i.e. characterInfo LAST. That is wrong for wire order
    /// (address order is not field order); the reader puts it FIRST.
    /// The one vanilla instance, `DropSet_Living_Together` (key 0xF809C), reads
    /// character_info=0, to_target_actor=1, is_register=0.
    pub struct DropRegistLivingCharacterData {
        pub character_info: u32,
        pub to_target_actor: u8,
        pub is_register: u8,
    }
}

/// Tag-dispatched payload following the 63-byte fixed prefix of
/// sub_141600210.
#[derive(Debug)]
pub enum DropTargetVariant {
    Tag0(u32),
    Tag1(u32),
    Tag2(u32),
    Tag3(u32),
    Tag4(u32),
    Tag5(u32),
    Tag6(u32),
    Tag7(DropTargetItemRef),
    Tag8(DropTargetItemRef),
    Tag9(u32),
    TagA { lookup_a: u32, lookup_b: u32 },
    TagB,
    TagC(u32),
    TagD { lookup: u32, flag: u8 },
    /// ── 1.18.00: NEW tag 16 (0x10) — `DropRegistLivingCharacterData`.
    /// Width proven from the wire (6 B), field split read out of the Mac 1.18
    /// reader `sub_101FD06E8`. Only one vanilla record uses it:
    /// `DropSet_Living_Together` (key 0xF809C) — a direct name match.
    Tag16(DropRegistLivingCharacterData),
}

impl DropTargetVariant {
    fn read_from(tag: u8, data: &[u8], offset: &mut usize) -> io::Result<Self> {
        Ok(match tag {
            16 => Self::Tag16(DropRegistLivingCharacterData::read_from(data, offset)?),
            0 => Self::Tag0(u32::read_from(data, offset)?),
            1 => Self::Tag1(u32::read_from(data, offset)?),
            2 => Self::Tag2(u32::read_from(data, offset)?),
            3 => Self::Tag3(u32::read_from(data, offset)?),
            4 => Self::Tag4(u32::read_from(data, offset)?),
            5 => Self::Tag5(u32::read_from(data, offset)?),
            6 => Self::Tag6(u32::read_from(data, offset)?),
            7 => Self::Tag7(DropTargetItemRef::read_from(data, offset)?),
            8 => Self::Tag8(DropTargetItemRef::read_from(data, offset)?),
            9 => Self::Tag9(u32::read_from(data, offset)?),
            0xA => {
                let lookup_a = u32::read_from(data, offset)?;
                let lookup_b = u32::read_from(data, offset)?;
                Self::TagA { lookup_a, lookup_b }
            }
            0xB => Self::TagB,
            0xC => Self::TagC(u32::read_from(data, offset)?),
            0xD => {
                let lookup = u32::read_from(data, offset)?;
                let flag = u8::read_from(data, offset)?;
                Self::TagD { lookup, flag }
            }
            other => return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("DropTargetVariant: unknown tag {}", other))),
        })
    }

    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match self {
            Self::Tag0(v) | Self::Tag1(v) | Self::Tag2(v) | Self::Tag3(v)
            | Self::Tag4(v) | Self::Tag5(v) | Self::Tag6(v) | Self::Tag9(v)
            | Self::TagC(v) => v.write_to(w),
            Self::Tag7(p) | Self::Tag8(p) => p.write_to(w),
            Self::TagA { lookup_a, lookup_b } => {
                lookup_a.write_to(w)?;
                lookup_b.write_to(w)
            }
            Self::TagB => Ok(()),
            Self::TagD { lookup, flag } => {
                lookup.write_to(w)?;
                flag.write_to(w)
            }
            Self::Tag16(p) => p.write_to(w),
        }
    }

    fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        match self {
            Self::Tag0(v) => { m.insert("tag".into(), 0.into()); m.insert("lookup".into(), v.to_json_value()); }
            Self::Tag1(v) => { m.insert("tag".into(), 1.into()); m.insert("lookup".into(), v.to_json_value()); }
            Self::Tag2(v) => { m.insert("tag".into(), 2.into()); m.insert("lookup".into(), v.to_json_value()); }
            Self::Tag3(v) => { m.insert("tag".into(), 3.into()); m.insert("lookup".into(), v.to_json_value()); }
            Self::Tag4(v) => { m.insert("tag".into(), 4.into()); m.insert("lookup".into(), v.to_json_value()); }
            Self::Tag5(v) => { m.insert("tag".into(), 5.into()); m.insert("lookup".into(), v.to_json_value()); }
            Self::Tag6(v) => { m.insert("tag".into(), 6.into()); m.insert("lookup".into(), v.to_json_value()); }
            Self::Tag7(p) => { m.insert("tag".into(), 7.into()); m.insert("data".into(), Value::Object(p.to_json_dict())); }
            Self::Tag8(p) => { m.insert("tag".into(), 8.into()); m.insert("data".into(), Value::Object(p.to_json_dict())); }
            Self::Tag9(v) => { m.insert("tag".into(), 9.into()); m.insert("lookup".into(), v.to_json_value()); }
            Self::TagA { lookup_a, lookup_b } => {
                m.insert("tag".into(), 0xA.into());
                m.insert("lookup_a".into(), lookup_a.to_json_value());
                m.insert("lookup_b".into(), lookup_b.to_json_value());
            }
            Self::TagB => { m.insert("tag".into(), 0xB.into()); }
            Self::TagC(v) => { m.insert("tag".into(), 0xC.into()); m.insert("lookup".into(), v.to_json_value()); }
            Self::TagD { lookup, flag } => {
                m.insert("tag".into(), 0xD.into());
                m.insert("lookup".into(), lookup.to_json_value());
                m.insert("flag".into(), flag.to_json_value());
            }
            Self::Tag16(p) => {
                m.insert("tag".into(), 16.into());
                m.insert("data".into(), Value::Object(p.to_json_dict()));
            }
        }
        Value::Object(m)
    }

    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "DropTargetVariant: expected object"))?;
        let tag = json_get_field(obj, "tag")?
            .as_u64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "DropTargetVariant.tag: expected u8"))? as u8;
        match tag {
            0..=6 | 9 | 0xC => {
                <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup")?)
            }
            7 | 8 => {
                <DropTargetItemRef as WriteJsonValue>::write_from_json(w, json_get_field(obj, "data")?)
            }
            0xA => {
                <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_a")?)?;
                <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_b")?)
            }
            0xB => Ok(()),
            0xD => {
                <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup")?)?;
                <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag")?)
            }
            16 => {
                <DropRegistLivingCharacterData as WriteJsonValue>::write_from_json(
                    w, json_get_field(obj, "data")?)
            }
            other => Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("DropTargetVariant: unknown tag {}", other))),
        }
    }
}

/// Per-element payload of `sub_141600210` (the inner reader of
/// sub_141D03AA0). 63 fixed wire bytes + variant tail.
#[derive(Debug)]
pub struct DropTargetData {
    pub raw_at_120: u64,
    pub dispatch_tag: u8,
    pub lookup_4: u32,
    pub lookup_6: u32,
    pub lookup_8: u32,
    pub raw_12: u32,
    pub raw_16: u64,
    // ── 1.18.00: the u32 that used to sit here (`raw_24`) was REMOVED.
    //
    // ⚠ CORRECTION to the older comment on this struct: `raw_24` was NOT
    // `_percent`. The Mac 1.18 reader sub_10184CEAC gives the engine's real
    // `DropInfoData` layout — dropTagNameHash u32 @+12, **percent u64 @+16**,
    // subPercent u64 @+24, minValue u64 @+32, maxValue u64 @+40,
    // enchantLevel @+48 — so `_percent` is `raw_16`, and the deleted field sat
    // between subPercent and minValue.
    //
    // The bytes agree and are decisive: on dropsetinfo key 0x90F561 the 1.18
    // element fits that layout exactly (percent=1000000, subPercent=0,
    // minValue=50, maxValue=50, enchantLevel=0xffff), while 1.17 only fits with
    // ONE extra u32 in that slot — force the 1.18 layout onto 1.17 and minValue
    // reads 214748364800 instead of 50.
    //
    // ★ Neither the byte diff nor the field-name oracle could resolve this
    // alone: the oracle reports DropInfoData UNCHANGED (the removed field emits
    // no error string, and deletions are invisible to it), and the wire is
    // equally consistent with "delete a u32" or "narrow a u64". Only reading
    // the reader settled it.
    //
    // ⚠ MOD IMPACT: the old `rates_100` community alias pointed at `raw_24`,
    // i.e. the LOW HALF of `_subPercent` — not the drop chance. The real drop
    // percent is `raw_16`. A pre-1.18 mod setting `rates_100` now has that
    // value silently ignored (key not consumed) rather than corrupting the
    // record. Anything intending to change drop CHANCE should target `raw_16`.
    //
    // ⚠ Remaining names are one slot off from the engine's (raw_32/raw_40/
    // raw_48 are subPercent/minValue/maxValue). Left as-is: parser field names
    // are the mod contract, so renaming is a deliberate, separate decision.
    pub raw_32: u64,
    pub raw_40: u64,
    pub raw_48: u64,
    pub raw_56: u16,
    pub variant: DropTargetVariant,
}

impl<'a> BinaryRead<'a> for DropTargetData {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let raw_at_120 = u64::read_from(data, offset)?;
        let dispatch_tag = u8::read_from(data, offset)?;
        let lookup_4 = u32::read_from(data, offset)?;
        let lookup_6 = u32::read_from(data, offset)?;
        let lookup_8 = u32::read_from(data, offset)?;
        let raw_12 = u32::read_from(data, offset)?;
        let raw_16 = u64::read_from(data, offset)?;
        let raw_32 = u64::read_from(data, offset)?;
        let raw_40 = u64::read_from(data, offset)?;
        let raw_48 = u64::read_from(data, offset)?;
        let raw_56 = u16::read_from(data, offset)?;
        let variant = DropTargetVariant::read_from(dispatch_tag, data, offset)?;
        Ok(Self {
            raw_at_120, dispatch_tag, lookup_4, lookup_6, lookup_8,
            raw_12, raw_16, raw_32, raw_40, raw_48, raw_56, variant,
        })
    }
}

impl BinaryWrite for DropTargetData {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.raw_at_120.write_to(w)?;
        self.dispatch_tag.write_to(w)?;
        self.lookup_4.write_to(w)?;
        self.lookup_6.write_to(w)?;
        self.lookup_8.write_to(w)?;
        self.raw_12.write_to(w)?;
        self.raw_16.write_to(w)?;
        self.raw_32.write_to(w)?;
        self.raw_40.write_to(w)?;
        self.raw_48.write_to(w)?;
        self.raw_56.write_to(w)?;
        self.variant.write_to(w)
    }
}

impl ToJsonValue for DropTargetData {
    fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("raw_at_120".into(), self.raw_at_120.to_json_value());
        m.insert("dispatch_tag".into(), self.dispatch_tag.to_json_value());
        m.insert("lookup_4".into(), self.lookup_4.to_json_value());
        m.insert("lookup_6".into(), self.lookup_6.to_json_value());
        m.insert("lookup_8".into(), self.lookup_8.to_json_value());
        m.insert("raw_12".into(), self.raw_12.to_json_value());
        m.insert("raw_16".into(), self.raw_16.to_json_value());
        m.insert("raw_32".into(), self.raw_32.to_json_value());
        m.insert("raw_40".into(), self.raw_40.to_json_value());
        m.insert("raw_48".into(), self.raw_48.to_json_value());
        m.insert("raw_56".into(), self.raw_56.to_json_value());
        m.insert("variant".into(), self.variant.to_json_value());
        Value::Object(m)
    }
}

impl WriteJsonValue for DropTargetData {
    /// Accepts two JSON shapes:
    ///
    /// 1. **Wire-faithful** — every field present with its raw name
    ///    (`raw_at_120`, `dispatch_tag`, `lookup_4`, ..., `variant`).
    ///    Used by round-trip from `to_json_value`.
    ///
    /// 2. **Semantic** — Stacker / CrimsonGameMods DropSets style. Field
    ///    names map per the C++ reflection metadata captured from
    ///    `pa::DropInfoData` registration in CrimsonDesert_Steam (Mac
    ///    binary, sub_1007515B0):
    ///
    ///    | semantic       | C++ field           | wire field     | type | default |
    ///    |----------------|---------------------|----------------|------|---------|
    ///    | `item_key`     | `_keyRaw`           | `raw_at_120`   | u64  | 0       |
    ///    | (no alias)     | `_dropResultType`   | `dispatch_tag` | u8   | 0 (item)|
    ///    | (no alias)     | `_ownerConditionInfo` | `lookup_4`   | u32  | 0xFFFFFFFF |
    ///    | (no alias)     | `_playerConditionInfo` | `lookup_6`  | u32  | 0xFFFFFFFF |
    ///    | (no alias)     | `_gimmickCachedTargetConditionInfo` | `lookup_8` | u32 | 0xFFFFFFFF |
    ///    | (no alias)     | `_dropTagNameHash`  | `raw_12`       | u32  | 0       |
    ///    | `rates`        | `_percent` (main)   | `raw_16`       | u64  | 0       |
    ///    | `rates_100`    | REMOVED in 1.18. ⚠ It mapped to `raw_24`, which was the low half of `_subPercent`, NOT the drop chance — that is `raw_16` (`_percent`, u64). |
    ///    | (no alias)     | `_subPercent`       | `raw_32`       | u64  | 0       |
    ///    | `min_amt`      | `_minValue`         | `raw_40`       | u64  | 0       |
    ///    | `max_amt`      | `_maxValue`         | `raw_48`       | u64  | 0       |
    ///    | (no alias)     | `_enchantLevel`     | `raw_56`       | u16  | 0       |
    ///
    ///    For the variant tail at `dispatch_tag = 0` (item drop), the
    ///    semantic shape writes `item_key as u32` as the 4-byte ItemKey
    ///    hash. Other tags fall back to the wire-faithful `variant`
    ///    field; semantic shape with non-zero `dispatch_tag` requires
    ///    the user to supply `variant` explicitly.
    ///
    /// Detection: any object containing both `dispatch_tag` and `variant`
    /// is wire-faithful; otherwise the semantic path fills in defaults.
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "DropTargetData: expected object"))?;

        // Pick wire field; fall back to semantic alias; then default.
        let item_key = pick_u64(obj, &["raw_at_120", "item_key"], 0);
        let dispatch_tag = pick_u8(obj, &["dispatch_tag"], 0);
        let lookup_4 = pick_u32(obj, &["lookup_4"], 0xFFFFFFFF);
        let lookup_6 = pick_u32(obj, &["lookup_6"], 0xFFFFFFFF);
        let lookup_8 = pick_u32(obj, &["lookup_8"], 0xFFFFFFFF);
        let raw_12 = pick_u32(obj, &["raw_12"], 0);
        let raw_16 = pick_u64(obj, &["raw_16", "rates"], 0);
        let raw_32 = pick_u64(obj, &["raw_32"], 0);
        let raw_40 = pick_u64(obj, &["raw_40", "min_amt"], 0);
        let raw_48 = pick_u64(obj, &["raw_48", "max_amt"], 0);
        let raw_56 = pick_u16(obj, &["raw_56"], 0);

        item_key.write_to(w)?;
        dispatch_tag.write_to(w)?;
        lookup_4.write_to(w)?;
        lookup_6.write_to(w)?;
        lookup_8.write_to(w)?;
        raw_12.write_to(w)?;
        raw_16.write_to(w)?;
        raw_32.write_to(w)?;
        raw_40.write_to(w)?;
        raw_48.write_to(w)?;
        raw_56.write_to(w)?;

        // Variant tail. Wire-faithful path: explicit `variant` object.
        // Semantic path: synthesize from dispatch_tag + item_key.
        if let Some(variant_v) = obj.get("variant") {
            DropTargetVariant::write_from_json(w, variant_v)
        } else {
            write_default_variant_tail(w, dispatch_tag, item_key)
        }
    }
}

/// Write a default variant tail for semantic-shape JSON that omitted
/// `variant`. Each tag has a deterministic shape (see top-level module
/// docstring) — we fill the simplest valid bytes that get the variant
/// to round-trip:
///
/// - tag 0 (item drop)            : 4 wire bytes — `item_key as u32`
///   This is the 99% case for Stacker DropSets exports.
/// - tags 1, 2, 3 (character)     : 4 wire bytes — `item_key as u32`
/// - tag 4 (knowledge)            : 4 wire bytes — `item_key as u32`
/// - tag 5, 6, 9, 0xC             : 4 wire bytes — `item_key as u32`
/// - tag 7, 8 (DropFriendlyData)  : 32 wire bytes of zeros — needs
///   explicit `variant` object for non-default friendly NPC drops
/// - tag 0xA                      : 8 bytes (two u32 zero)
/// - tag 0xB                      : 0 bytes
/// - tag 0xD                      : 5 bytes (u32 + u8 zero)
/// - tag 0xE / others             : error — caller must supply `variant`
fn write_default_variant_tail(w: &mut Vec<u8>, dispatch_tag: u8, item_key: u64) -> io::Result<()> {
    let key_u32 = item_key as u32;
    match dispatch_tag {
        0 | 1 | 2 | 3 | 4 | 5 | 6 | 9 | 0xC => key_u32.write_to(w),
        7 | 8 => {
            // DropFriendlyData: 32 zero bytes default. Stacker DropSets
            // doesn't use these tags; explicit `variant` required for
            // friendly-NPC drops.
            for _ in 0..32 { 0u8.write_to(w)?; }
            Ok(())
        }
        0xA => {
            0u32.write_to(w)?;
            0u32.write_to(w)?;
            Ok(())
        }
        0xB => Ok(()),
        0xD => {
            0u32.write_to(w)?;
            0u8.write_to(w)?;
            Ok(())
        }
        other => Err(io::Error::new(io::ErrorKind::InvalidData,
            format!("DropTargetData semantic shape: dispatch_tag {} requires explicit `variant` object", other))),
    }
}

/// Helper: read u64 from the first present alias name; default if none.
fn pick_u64(obj: &Map<String, Value>, names: &[&str], default: u64) -> u64 {
    for n in names {
        if let Some(v) = obj.get(*n) {
            if let Some(x) = v.as_u64() { return x; }
            if let Some(x) = v.as_i64() { return x as u64; }
        }
    }
    default
}

fn pick_u32(obj: &Map<String, Value>, names: &[&str], default: u32) -> u32 {
    for n in names {
        if let Some(v) = obj.get(*n) {
            if let Some(x) = v.as_u64() { return x as u32; }
            if let Some(x) = v.as_i64() { return x as u32; }
        }
    }
    default
}

fn pick_u16(obj: &Map<String, Value>, names: &[&str], default: u16) -> u16 {
    for n in names {
        if let Some(v) = obj.get(*n) {
            if let Some(x) = v.as_u64() { return x as u16; }
            if let Some(x) = v.as_i64() { return x as u16; }
        }
    }
    default
}

fn pick_u8(obj: &Map<String, Value>, names: &[&str], default: u8) -> u8 {
    for n in names {
        if let Some(v) = obj.get(*n) {
            if let Some(x) = v.as_u64() { return x as u8; }
            if let Some(x) = v.as_i64() { return x as u8; }
        }
    }
    default
}

/// `sub_141D03AA0` per-element: u8 presence + (if present:
/// `DropTargetData` populated by sub_141600210).
#[derive(Debug)]
pub struct OptionalDropTarget {
    pub inner: Option<DropTargetData>,
}

impl<'a> BinaryRead<'a> for OptionalDropTarget {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let presence = u8::read_from(data, offset)?;
        let inner = if presence != 0 {
            Some(DropTargetData::read_from(data, offset)?)
        } else {
            None
        };
        Ok(Self { inner })
    }
}

impl BinaryWrite for OptionalDropTarget {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match &self.inner {
            Some(d) => { 1u8.write_to(w)?; d.write_to(w) }
            None => 0u8.write_to(w),
        }
    }
}

impl ToJsonValue for OptionalDropTarget {
    fn to_json_value(&self) -> Value {
        match &self.inner {
            Some(d) => d.to_json_value(),
            None => Value::Null,
        }
    }
}

impl WriteJsonValue for OptionalDropTarget {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() {
            0u8.write_to(w)
        } else {
            1u8.write_to(w)?;
            DropTargetData::write_from_json(w, v)
        }
    }
}

impl<'a> BinaryReadTracked<'a> for OptionalDropTarget {
    fn read_tracked(data: &'a [u8], offset: &mut usize, _path: &mut String, _ranges: &mut Vec<FieldRange>) -> io::Result<Self> {
        Self::read_from(data, offset)
    }
}

impl crate::python_traits::ToPyValue for OptionalDropTarget {
    fn to_py_value(&self, _py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "OptionalDropTarget: use JSON path",
        ))
    }
}

impl crate::python_traits::WritePyValue for OptionalDropTarget {
    fn write_from_py(_w: &mut Vec<u8>, _obj: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "OptionalDropTarget: use JSON path",
        ))
    }
}
