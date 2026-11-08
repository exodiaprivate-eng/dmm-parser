#![allow(clippy::doc_overindented_list_items)]
// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! GlobalGameEventExecuteData polymorphic family wrapper, game 2.04.08 shape.
//!
//! Wire format (dispatcher `sub_141156680` in the 1.x Win build; the 2.04
//! shape below is decoded from the 2.04.08 data, 408/408 records):
//!   - u8 presence (0 = absent, 1 = present)
//!   - if present:
//!     - u8 sub_tag
//!     - body. Every 2.04 body starts with the same head:
//!         * `body_tag`: u8, equal to `sub_tag` in every 2.04.08 record
//!         * `text_1`:   LocalizableString (category 44, index =
//!                       (record key << 32) | 705 where set)
//!         * `text_2`:   LocalizableString (empty except sub_tag 4, code 706)
//!       then the per-sub_tag payload:
//!         - 0 VaryTradeItemPrice: `target_item_group_keys` CArray<u16>,
//!           `price_list` CArray<VaryTradeItemPriceData>, `item_lookup` u32,
//!           `description` LocalizableString (= `text_1` in all 79 records).
//!           Same payload as 1.x, now after the head.
//!         - 1 OpenRoyalSupply: `target_region_keys` CArray<u16> (as 1.x).
//!         - 2 FactionBlock (was "in place", no body, in 1.x): one u16.
//!         - 3 (Graymane_Party_FarmersDay only): CArray<u32> of hashes + u8.
//!         - 4 (Graymane_Party_Hernandian / _Pailunese): CArray<u16>.
//!         - 5 ManagementSpecialEvent: u16 building key (one value per
//!           building, 18) + CArray<u32> characterinfo keys.
//!       Names for 2-5 come from the records that use them; the game's class
//!       names for these constructors are not known.
//!
//! ## Decode strategy: Decoded | Raw fallback
//!
//! `read_from` tries the typed shape per sub_tag and falls back to `Raw(bytes)`
//! on any decode failure, unknown sub_tag or under-consume, so a future build
//! that adds a sub_tag still round-trips byte for byte.
//!
//! Key fields (`*_keys` u16, `item_lookup` u32, ...) are stored as the wire
//! values; the runtime resolves them through its own hash tables.

use crate::binary::*;
use crate::json_traits::get_field as json_get_field;
use crate::py_binary_struct;
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
use std::io::{self, Write};

// ── Per-sub_tag payload structs ────────────────────────────────────────────

py_binary_struct! {
    /// One element of `VaryTradeItemPricePayload.price_list`. Wire: CArray<u16>
    /// region keys + two i64 price bounds (`_targetRegionList`, `_minPrice`,
    /// `_maxPrice`). 2.04.08 values: 500000..800000 or -800000..-500000.
    pub struct VaryTradeItemPriceData {
        pub target_region_keys: CArray<u16>,
        pub min_price: i64,
        pub max_price: i64,
    }
}

py_binary_struct! {
    /// sub_tag 0 body (weather / disaster / trade events, group 16960).
    pub struct VaryTradeItemPricePayload<'a> {
        pub body_tag: u8,
        pub text_1: LocalizableString<'a>,
        pub text_2: LocalizableString<'a>,
        pub target_item_group_keys: CArray<u16>,
        pub price_list: CArray<VaryTradeItemPriceData>,
        pub item_lookup: u32,
        pub description: LocalizableString<'a>,
    }
}

py_binary_struct! {
    /// sub_tag 1 body (RoyalSupply_* records).
    pub struct OpenRoyalSupplyPayload<'a> {
        pub body_tag: u8,
        pub text_1: LocalizableString<'a>,
        pub text_2: LocalizableString<'a>,
        pub target_region_keys: CArray<u16>,
    }
}

py_binary_struct! {
    /// sub_tag 2 body (FactionBlockEvent_* records). `unk_u16_0`: 108
    /// distinct values over 111 records, rising with the record order; not
    /// the factionnode key (those are u32, e.g. Node_Her_PervinFort 1000329).
    pub struct FactionBlockPayload<'a> {
        pub body_tag: u8,
        pub text_1: LocalizableString<'a>,
        pub text_2: LocalizableString<'a>,
        pub unk_u16_0: u16,
    }
}

py_binary_struct! {
    /// sub_tag 3 body (only Graymane_Party_FarmersDay). One record: whether
    /// the trailing u8 belongs to each list element is not confirmed.
    pub struct PartyHashListPayload<'a> {
        pub body_tag: u8,
        pub text_1: LocalizableString<'a>,
        pub text_2: LocalizableString<'a>,
        pub unk_hash_list: CArray<u32>,
        pub unk_u8: u8,
    }
}

py_binary_struct! {
    /// sub_tag 4 body (Graymane_Party_Hernandian / _Pailunese).
    pub struct PartyKeyListPayload<'a> {
        pub body_tag: u8,
        pub text_1: LocalizableString<'a>,
        pub text_2: LocalizableString<'a>,
        pub unk_key_list: CArray<u16>,
    }
}

py_binary_struct! {
    /// sub_tag 5 body (ManagementSpecialEvent_<place>_<building>_<npc>).
    /// `management_building_key`: one value per building (18), one-to-one
    /// with the record's group. `character_info_list`: characterinfo keys
    /// (e.g. 3452 NHW_Unique_Maegu for ..._Vellua_Pub_Maegu, 1002939
    /// NHW_Unique_Wisdom_Witch_Store for ..._Wisdom_Witch).
    pub struct ManagementSpecialEventPayload<'a> {
        pub body_tag: u8,
        pub text_1: LocalizableString<'a>,
        pub text_2: LocalizableString<'a>,
        pub management_building_key: u16,
        pub character_info_list: CArray<u32>,
    }
}

// ── Body enum (per-sub_tag dispatch) ───────────────────────────────────────

macro_rules! gge_bodies {
    ($( $tag:literal => $variant:ident($payload:ident) = $label:literal ),* $(,)?) => {
        #[derive(Debug)]
        pub enum GlobalGameEventExecuteDataBody<'a> {
            $( $variant($payload<'a>), )*
        }

        impl<'a> GlobalGameEventExecuteDataBody<'a> {
            pub fn sub_tag(&self) -> u8 {
                match self { $( Self::$variant(_) => $tag, )* }
            }
            /// Name used in JSON `body_type`.
            pub fn body_type(&self) -> &'static str {
                match self { $( Self::$variant(_) => $label, )* }
            }
            fn read_body(sub_tag: u8, data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
                match sub_tag {
                    $( $tag => $payload::read_from(data, offset).map(Self::$variant), )*
                    other => Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("GlobalGameEventExecuteData: unknown sub_tag {}", other),
                    )),
                }
            }
            fn write_body(&self, w: &mut dyn Write) -> io::Result<()> {
                match self { $( Self::$variant(p) => p.write_to(w), )* }
            }
            fn body_json(&self) -> Map<String, Value> {
                match self { $( Self::$variant(p) => p.to_json_dict(), )* }
            }
            fn write_body_from_json(sub_tag: u8, w: &mut Vec<u8>, body: &Map<String, Value>) -> io::Result<()> {
                match sub_tag {
                    $( $tag => $payload::write_from_json_dict(w, body), )*
                    other => Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("GlobalGameEventExecuteData: unknown sub_tag {}", other),
                    )),
                }
            }
        }
    };
}

gge_bodies! {
    0 => VaryTradeItemPrice(VaryTradeItemPricePayload) = "vary_trade_item_price",
    1 => OpenRoyalSupply(OpenRoyalSupplyPayload) = "open_royal_supply",
    2 => FactionBlock(FactionBlockPayload) = "faction_block",
    3 => PartyHashList(PartyHashListPayload) = "party_hash_list",
    4 => PartyKeyList(PartyKeyListPayload) = "party_key_list",
    5 => ManagementSpecialEvent(ManagementSpecialEventPayload) = "management_special_event",
}

// ── Wrapper enum ───────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum GlobalGameEventExecuteData<'a> {
    /// Presence byte == 0; no sub_tag, no body. On wire: `[0x00]`.
    Absent,
    /// Presence byte == 1; sub_tag-driven typed body.
    /// On wire: `[0x01, sub_tag, ...body]`.
    Present(GlobalGameEventExecuteDataBody<'a>),
    /// Fallback for unrecognized presence/sub_tag, decode failure, or
    /// trailing bytes the typed reader didn't consume. Bytes kept verbatim.
    Raw(Vec<u8>),
}

impl<'a> GlobalGameEventExecuteData<'a> {
    /// `data` must end where the wrapper ends (the record end). Any decode
    /// that doesn't consume exactly to the end produces a Raw fallback.
    pub fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let start = *offset;
        if start >= data.len() {
            return Ok(Self::Raw(Vec::new()));
        }
        let typed = match data[start] {
            0 if data.len() - start == 1 => Some(Self::Absent),
            1 if data.len() - start >= 2 => {
                let mut probe = start + 2;
                match GlobalGameEventExecuteDataBody::read_body(data[start + 1], data, &mut probe) {
                    Ok(body) if probe == data.len() => Some(Self::Present(body)),
                    _ => None,
                }
            }
            _ => None,
        };
        *offset = data.len();
        Ok(typed.unwrap_or_else(|| Self::Raw(data[start..].to_vec())))
    }

    /// True for the verbatim fallback.
    pub fn is_raw(&self) -> bool {
        matches!(self, Self::Raw(_))
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match self {
            Self::Absent => 0u8.write_to(w),
            Self::Present(body) => {
                1u8.write_to(w)?;
                body.sub_tag().write_to(w)?;
                body.write_body(w)
            }
            Self::Raw(bytes) => w.write_all(bytes),
        }
    }

    /// JSON shape:
    /// - `kind`: "absent" | "present" | "raw"
    /// - when "present": `sub_tag` (u8, decides the wire shape), `body_type`
    ///   (informational name), `body` (typed object: `body_tag`, `text_1`,
    ///   `text_2`, then the payload fields)
    /// - when "raw": `raw_b64` (base64 string)
    pub fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        match self {
            Self::Absent => {
                m.insert("kind".into(), Value::String("absent".into()));
            }
            Self::Present(body) => {
                m.insert("kind".into(), Value::String("present".into()));
                m.insert("sub_tag".into(), Value::Number(body.sub_tag().into()));
                m.insert("body_type".into(), Value::String(body.body_type().into()));
                m.insert("body".into(), Value::Object(body.body_json()));
            }
            Self::Raw(bytes) => {
                m.insert("kind".into(), Value::String("raw".into()));
                m.insert("raw_b64".into(), Value::String(B64.encode(bytes)));
            }
        }
        Value::Object(m)
    }

    pub fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let bad = |msg: String| io::Error::new(io::ErrorKind::InvalidData, msg);
        let obj = v
            .as_object()
            .ok_or_else(|| bad("GlobalGameEventExecuteData: expected object".into()))?;
        let kind = json_get_field(obj, "kind")?
            .as_str()
            .ok_or_else(|| bad("GlobalGameEventExecuteData.kind: expected string".into()))?;
        match kind {
            "absent" => {
                w.push(0);
                Ok(())
            }
            "present" => {
                let sub_tag = json_get_field(obj, "sub_tag")?
                    .as_u64()
                    .filter(|t| *t <= u8::MAX as u64)
                    .ok_or_else(|| bad("GlobalGameEventExecuteData.sub_tag: expected u8".into()))?
                    as u8;
                let body = json_get_field(obj, "body")?.as_object().ok_or_else(|| {
                    bad(format!("GlobalGameEventExecuteData.body: expected object for sub_tag {}", sub_tag))
                })?;
                w.push(1);
                w.push(sub_tag);
                GlobalGameEventExecuteDataBody::write_body_from_json(sub_tag, w, body)
            }
            "raw" => {
                let b64 = json_get_field(obj, "raw_b64")?
                    .as_str()
                    .ok_or_else(|| bad("GlobalGameEventExecuteData.raw_b64: expected base64 string".into()))?;
                let bytes = B64
                    .decode(b64)
                    .map_err(|e| bad(format!("GlobalGameEventExecuteData.raw_b64: invalid base64: {}", e)))?;
                w.extend_from_slice(&bytes);
                Ok(())
            }
            other => Err(bad(format!("GlobalGameEventExecuteData.kind: unknown value {:?}", other))),
        }
    }
}
