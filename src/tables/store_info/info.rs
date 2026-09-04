//! Tier 1 — fully typed (no _tail_b64).
//!
//! Reader (verified 2026-05-22 against CrimsonDesert.exe 1.0.8):
//! `sub_1410F3B20` — the StoreInfo deserializer. `a1` = byte-stream
//! reader object (vtable+8 = read-N-bytes), `a2` = StoreInfo struct.
//! Each field is read individually into a fixed struct offset; the
//! per-field error string (`unk_144B27XXX`) is that field's name.
//! (The previously-cited `sub_1410FCD20` was stale — it falls inside
//! `sub_1410FCC20`, an unrelated GamePlayVariableInfo lookup, in this
//! build.)
//!
//! Full IDA field map (wire-width / mem-offset → Rust field):
//!   a2+0   u16          key
//!   a2+8   CString      string_key                (sub_14108B300)
//!   a2+16  u8           is_blocked
//!   a2+18  u32 wire     exchange_item_info_for_buy (sub_1410E1B70:
//!          reads 4 wire bytes, remaps via lookup → u16 in RAM)
//!   a2+24  CArray<u32>  exchange_item_info_list_for_sell
//!          (sub_1410E24C0: u32 count + u32 elems, each → u16 in RAM)
//!   a2+40  u64          sell_percents
//!   a2+48  u8           store_type
//!   a2+56  CArray<u64>  price_increase_percent_list (u32 count + u64[])
//!   a2+72  u8           sellable_character_condition_logic
//!          (1.0.8: changed from u32 lookup to plain u8)
//!   a2+76  u32          reset_hour
//!   a2+80  u32          reset_day
//!   a2+84  u32          buyable_stock_count
//!   a2+88  u32          sellable_stock_count
//!   a2+92  u8           sellable_type
//!   a2+96  CArray<StoreStockData>  stock_data_list
//!          (u32 count + 88-mem-byte elems via sub_1410DEEC0)
//!   a2+112 CArray<u8>   sale_item_type_list      (sub_1410E2850)
//!   a2+128 CArray<u8>   not_sale_item_type_list  (sub_1410E2850, same)
//!   a2+144 u32          custom_mesh_obb_max_length
//!   a2+148 u8           fixed_price
//!   a2+149 u8           use_housing_gimmick
//!   a2+150 u8           reduce_price_by_looted_dead_body
//!
//! NOTE on wire vs memory type: fields 4, 5 read a **u32** off the
//! wire then remap it through an ID-resolution table into a **u16**
//! in-memory slot. The Rust struct models the *wire* type (`u32` /
//! `CArray<u32>`), which is correct for read/write roundtrip and for
//! v3 modding — the u16 RAM form is a runtime concern that never hits
//! disk. Field 9 was likewise a u32 lookup in 1.0.7 but changed to
//! a plain u8 in 1.0.8. Byte-exact roundtrip + this IDA cross-check
//! together prove both field boundaries *and* field types at the wire
//! level.
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u16 key                                  (_key, pabgh format 2)
//!   2. CString string_key                       (_stringKey)
//!   3. u8 is_blocked                            (_isBlocked)
//!   4. u32 exchange_item_info_for_buy           (_exchangeItemInfoForBuy,
//!      sub_1410FF5C0 wire u32)
//!   5. CArray<u32> exchange_item_info_list_for_sell
//!      (_exchangeItemInfoListForSell, sub_1410FFF10 wire u32)
//!   6. u64 sell_percents                        (_sellPercents)
//!   7. u8 store_type                            (_storeType)
//!   8. CArray<u64> price_increase_percent_list  (_priceIncreasePercentList,
//!      inline u32 count + N×u64)
//!   9. u8 sellable_character_condition_logic    (_sellableCharacterConditionLogic,
//!      1.0.8: plain u8 read; was u32 lookup in 1.0.7)
//!  10. u32 reset_hour                           (_resetHour)
//!  11. u32 reset_day                            (_resetDay)
//!  12. u32 buyable_stock_count                  (_buyableStockCount)
//!  13. u32 sellable_stock_count                 (_sellableStockCount)
//!  14. u8 sellable_type                         (_sellableType)
//!  15. CArray<StoreStockData> stock_data_list   (_stockDataList,
//!      sub_1410FC8F0; per element 88 mem bytes / variable wire incl.
//!      Optional<StoreStockDataValue> 14-arm polymorphic)
//!  16. CArray<u8> sale_item_type_list           (_saleItemTypeList,
//!      sub_1411002A0)
//!  17. CArray<u8> not_sale_item_type_list       (_notSaleItemTypeList,
//!      sub_1411002A0)
//!  18. u32 custom_mesh_obb_max_length           (_customMeshOBBMaxLength,
//!      mem a2+144)
//!  19. u8 fixed_price                            (_fixedPrice, mem a2+148)
//!  20. u8 use_housing_gimmick                   (_useHousingGimmick,
//!      mem a2+149)
//!  21. u8 reduce_price_by_looted_dead_body      (_reducePriceByLootedDeadBody,
//!      mem a2+150)
//!
//! All 21 wire fields typed. Field names 16-21 were placeholders
//! (`raw_list_a`, `flag_a`, …) before the 1.3.5 IDA audit; renamed to
//! canonical so v3 store mods that target by canonical name resolve.

use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use crate::py_binary_struct;
use serde_json::{Map, Value};
use std::io::{self, Write};

// ── StoreStockDataValue 15-arm polymorphic ─────────────────────────────────
//
// Per `sub_1416098C0` dispatcher (1.0.8). Common header (63 wire bytes) +
// per-disc payload (0-32 wire bytes). Disc 11 (0xB) is the empty variant.
// 1.0.8 adds disc 14 (0xE): u16 + u8 = 3 wire bytes.

py_binary_struct! {
    pub struct StoreStockDataValueDisc7 {
        pub flag_a: u8,
        pub raw_a: u64,
        pub lookup_a: u32,    // sub_141100860 wire u32
        pub lookup_b: u32,    // sub_1410FF340 wire u32
        pub flag_b: u8,
        pub lookup_c: u32,    // sub_1411026F0 wire u32
        pub raw_b: u64,
        pub flag_c: u8,
        pub flag_d: u8,
    }
}

#[derive(Debug)]
pub enum StoreStockDataValuePayload {
    Disc0(u32),                          // sub_1410FF5C0
    Disc1(u32),                          // sub_1410FF340
    Disc2(u32),
    Disc3(u32),
    Disc4(u32),                          // sub_1411006D0
    Disc5(u32),                          // sub_1411060F0
    Disc6(u32),                          // sub_141103770
    Disc7(StoreStockDataValueDisc7),
    Disc8(StoreStockDataValueDisc7),
    Disc9(u32),                          // sub_141100740
    DiscA { lookup_a: u32, lookup_b: u32 },
    DiscB,
    DiscC(u32),                          // sub_1410FEBE0
    DiscD { lookup: u32, flag: u8 },     // sub_141102E00 + u8
    DiscE { lookup: u16, flag: u8 },     // sub_14111D4C0 (u16 wire) + u8  [1.0.8]
}

impl StoreStockDataValuePayload {
    pub fn read_from(disc: u8, data: &[u8], offset: &mut usize) -> io::Result<Self> {
        Ok(match disc {
            0 => Self::Disc0(u32::read_from(data, offset)?),
            1 => Self::Disc1(u32::read_from(data, offset)?),
            2 => Self::Disc2(u32::read_from(data, offset)?),
            3 => Self::Disc3(u32::read_from(data, offset)?),
            4 => Self::Disc4(u32::read_from(data, offset)?),
            5 => Self::Disc5(u32::read_from(data, offset)?),
            6 => Self::Disc6(u32::read_from(data, offset)?),
            7 => Self::Disc7(StoreStockDataValueDisc7::read_from(data, offset)?),
            8 => Self::Disc8(StoreStockDataValueDisc7::read_from(data, offset)?),
            9 => Self::Disc9(u32::read_from(data, offset)?),
            10 => {
                let lookup_a = u32::read_from(data, offset)?;
                let lookup_b = u32::read_from(data, offset)?;
                Self::DiscA { lookup_a, lookup_b }
            }
            11 => Self::DiscB,
            12 => Self::DiscC(u32::read_from(data, offset)?),
            13 => {
                let lookup = u32::read_from(data, offset)?;
                let flag = u8::read_from(data, offset)?;
                Self::DiscD { lookup, flag }
            }
            14 => {
                let lookup = u16::read_from(data, offset)?;
                let flag = u8::read_from(data, offset)?;
                Self::DiscE { lookup, flag }
            }
            other => return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("StoreStockDataValuePayload: unknown disc {}", other),
            )),
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match self {
            Self::Disc0(v) | Self::Disc1(v) | Self::Disc2(v) | Self::Disc3(v)
            | Self::Disc4(v) | Self::Disc5(v) | Self::Disc6(v) | Self::Disc9(v)
            | Self::DiscC(v) => v.write_to(w),
            Self::Disc7(p) | Self::Disc8(p) => p.write_to(w),
            Self::DiscA { lookup_a, lookup_b } => {
                lookup_a.write_to(w)?;
                lookup_b.write_to(w)
            }
            Self::DiscB => Ok(()),
            Self::DiscD { lookup, flag } => {
                lookup.write_to(w)?;
                flag.write_to(w)
            }
            Self::DiscE { lookup, flag } => {
                lookup.write_to(w)?;
                flag.write_to(w)
            }
        }
    }

    pub fn variant_name(&self) -> &'static str {
        match self {
            Self::Disc0(_) => "Disc0",
            Self::Disc1(_) => "Disc1",
            Self::Disc2(_) => "Disc2",
            Self::Disc3(_) => "Disc3",
            Self::Disc4(_) => "Disc4",
            Self::Disc5(_) => "Disc5",
            Self::Disc6(_) => "Disc6",
            Self::Disc7(_) => "Disc7",
            Self::Disc8(_) => "Disc8",
            Self::Disc9(_) => "Disc9",
            Self::DiscA { .. } => "DiscA",
            Self::DiscB => "DiscB",
            Self::DiscC(_) => "DiscC",
            Self::DiscD { .. } => "DiscD",
            Self::DiscE { .. } => "DiscE",
        }
    }

    pub fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("type".to_string(), Value::String(self.variant_name().to_string()));
        match self {
            Self::Disc0(v) | Self::Disc1(v) | Self::Disc2(v) | Self::Disc3(v)
            | Self::Disc4(v) | Self::Disc5(v) | Self::Disc6(v) | Self::Disc9(v)
            | Self::DiscC(v) => { m.insert("body".to_string(), v.to_json_value()); }
            Self::Disc7(p) | Self::Disc8(p) => { m.insert("body".to_string(), Value::Object(p.to_json_dict())); }
            Self::DiscA { lookup_a, lookup_b } => {
                let mut body = Map::new();
                body.insert("lookup_a".to_string(), lookup_a.to_json_value());
                body.insert("lookup_b".to_string(), lookup_b.to_json_value());
                m.insert("body".to_string(), Value::Object(body));
            }
            Self::DiscB => { m.insert("body".to_string(), Value::Null); }
            Self::DiscD { lookup, flag } => {
                let mut body = Map::new();
                body.insert("lookup".to_string(), lookup.to_json_value());
                body.insert("flag".to_string(), flag.to_json_value());
                m.insert("body".to_string(), Value::Object(body));
            }
            Self::DiscE { lookup, flag } => {
                let mut body = Map::new();
                body.insert("lookup".to_string(), lookup.to_json_value());
                body.insert("flag".to_string(), flag.to_json_value());
                m.insert("body".to_string(), Value::Object(body));
            }
        }
        Value::Object(m)
    }

    pub fn write_from_json(disc: u8, w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "StoreStockDataValuePayload: expected object",
        ))?;
        let body = obj.get("body").ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "StoreStockDataValuePayload: missing body",
        ))?;
        match disc {
            0..=6 | 9 | 12 => <u32 as WriteJsonValue>::write_from_json(w, body)?,
            7 | 8 => {
                let body_obj = body.as_object().ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidData, "Disc7/8: expected object body"))?;
                StoreStockDataValueDisc7::write_from_json_dict(w, body_obj)?;
            }
            10 => {
                let body_obj = body.as_object().ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidData, "DiscA: expected object body"))?;
                <u32 as WriteJsonValue>::write_from_json(w, json_get_field(body_obj, "lookup_a")?)?;
                <u32 as WriteJsonValue>::write_from_json(w, json_get_field(body_obj, "lookup_b")?)?;
            }
            11 => { /* empty */ }
            13 => {
                let body_obj = body.as_object().ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidData, "DiscD: expected object body"))?;
                <u32 as WriteJsonValue>::write_from_json(w, json_get_field(body_obj, "lookup")?)?;
                <u8 as WriteJsonValue>::write_from_json(w, json_get_field(body_obj, "flag")?)?;
            }
            14 => {
                let body_obj = body.as_object().ok_or_else(|| io::Error::new(
                    io::ErrorKind::InvalidData, "DiscE: expected object body"))?;
                <u16 as WriteJsonValue>::write_from_json(w, json_get_field(body_obj, "lookup")?)?;
                <u8 as WriteJsonValue>::write_from_json(w, json_get_field(body_obj, "flag")?)?;
            }
            other => return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("StoreStockDataValuePayload: unknown disc {}", other),
            )),
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct StoreStockDataValue {
    pub raw_q: u64,
    pub disc: u8,
    pub lookup_a: u32,    // sub_1410FF430
    pub lookup_b: u32,    // sub_1410FF430
    pub lookup_c: u32,    // sub_1410FF430
    pub raw_a: u32,
    pub raw_b: u64,
    // ── 1.18.00: the u32 that used to sit here (`raw_c`) was REMOVED.
    // This struct is the engine's `DropInfoData` — the same shape as
    // DropTargetData in binary/variants/drop_target.rs. The Mac 1.18 reader
    // sub_10184CEAC lays it out as: dropTagNameHash u32 @+12, percent u64 @+16,
    // subPercent u64 @+24, minValue u64 @+32, maxValue u64 @+40,
    // enchantLevel @+48.
    // 1.18 bytes fit that exactly (percent=1000000, subPercent=0, min=50,
    // max=50); 1.17 only fits with ONE extra u32 between subPercent and
    // minValue — without it minValue reads 214748364800 instead of 50.
    // That extra field emits no error string, so the field-name oracle reports
    // DropInfoData unchanged: deletions AND string-less fields are both
    // invisible to it. Only the bytes + the reader can find this.
    //
    // ⚠ The remaining names are one slot off from the engine's: raw_b IS
    // `_percent`, and raw_d/raw_e/raw_f are subPercent/minValue/maxValue.
    // Left alone because parser field names are the mod contract.
    pub raw_d: u64,
    pub raw_e: u64,
    pub raw_f: u64,
    pub raw_g: u16,
    pub payload: StoreStockDataValuePayload,
}

impl StoreStockDataValue {
    pub fn read_from(data: &[u8], offset: &mut usize) -> io::Result<Self> {
        let raw_q = u64::read_from(data, offset)?;
        let disc = u8::read_from(data, offset)?;
        let lookup_a = u32::read_from(data, offset)?;
        let lookup_b = u32::read_from(data, offset)?;
        let lookup_c = u32::read_from(data, offset)?;
        let raw_a = u32::read_from(data, offset)?;
        let raw_b = u64::read_from(data, offset)?;
        let raw_d = u64::read_from(data, offset)?;
        let raw_e = u64::read_from(data, offset)?;
        let raw_f = u64::read_from(data, offset)?;
        let raw_g = u16::read_from(data, offset)?;
        let payload = StoreStockDataValuePayload::read_from(disc, data, offset)?;
        Ok(Self {
            raw_q, disc, lookup_a, lookup_b, lookup_c,
            raw_a, raw_b, raw_d, raw_e, raw_f, raw_g, payload,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.raw_q.write_to(w)?;
        self.disc.write_to(w)?;
        self.lookup_a.write_to(w)?;
        self.lookup_b.write_to(w)?;
        self.lookup_c.write_to(w)?;
        self.raw_a.write_to(w)?;
        self.raw_b.write_to(w)?;
        self.raw_d.write_to(w)?;
        self.raw_e.write_to(w)?;
        self.raw_f.write_to(w)?;
        self.raw_g.write_to(w)?;
        self.payload.write_to(w)
    }

    pub fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("raw_q".to_string(), self.raw_q.to_json_value());
        m.insert("disc".to_string(), self.disc.to_json_value());
        m.insert("lookup_a".to_string(), self.lookup_a.to_json_value());
        m.insert("lookup_b".to_string(), self.lookup_b.to_json_value());
        m.insert("lookup_c".to_string(), self.lookup_c.to_json_value());
        m.insert("raw_a".to_string(), self.raw_a.to_json_value());
        m.insert("raw_b".to_string(), self.raw_b.to_json_value());
        m.insert("raw_d".to_string(), self.raw_d.to_json_value());
        m.insert("raw_e".to_string(), self.raw_e.to_json_value());
        m.insert("raw_f".to_string(), self.raw_f.to_json_value());
        m.insert("raw_g".to_string(), self.raw_g.to_json_value());
        m.insert("payload".to_string(), self.payload.to_json_value());
        Value::Object(m)
    }

    pub fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "StoreStockDataValue: expected object"))?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_q")?)?;
        let disc = json_get_field(obj, "disc")?
            .as_u64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "disc: expected u64"))?
            as u8;
        disc.write_to(w)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_a")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_b")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_c")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_a")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_b")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_d")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_e")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_f")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_g")?)?;
        StoreStockDataValuePayload::write_from_json(disc, w, json_get_field(obj, "payload")?)?;
        Ok(())
    }
}

/// `u8 presence + (if presence: StoreStockDataValue)` — sub_141D03AA0.
#[derive(Debug)]
pub struct OptionalStoreStockDataValue {
    pub inner: Option<StoreStockDataValue>,
}

impl OptionalStoreStockDataValue {
    pub fn read_from(data: &[u8], offset: &mut usize) -> io::Result<Self> {
        let presence = u8::read_from(data, offset)?;
        let inner = if presence != 0 {
            Some(StoreStockDataValue::read_from(data, offset)?)
        } else {
            None
        };
        Ok(Self { inner })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match &self.inner {
            Some(v) => { 1u8.write_to(w)?; v.write_to(w) }
            None => 0u8.write_to(w),
        }
    }

    pub fn to_json_value(&self) -> Value {
        match &self.inner {
            Some(v) => v.to_json_value(),
            None => Value::Null,
        }
    }

    pub fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() {
            w.push(0);
            Ok(())
        } else {
            w.push(1);
            StoreStockDataValue::write_from_json(w, v)
        }
    }
}

// sub_1410D9E90 inner — 8 mem bytes / 4 wire fields = 13 wire bytes.
py_binary_struct! {
    pub struct StoreStockSubData {
        pub lookup_a: u32,    // sub_1410FF430 wire u32
        pub flag: u8,
        pub lookup_b: u32,    // sub_1410FF050 wire u32
        pub lookup_c: u32,    // sub_1410FF050 wire u32
    }
}

/// `u8 presence + (if presence: StoreStockSubData)`.
#[derive(Debug)]
pub struct OptionalStoreStockSubData {
    pub inner: Option<StoreStockSubData>,
}

impl OptionalStoreStockSubData {
    pub fn read_from(data: &[u8], offset: &mut usize) -> io::Result<Self> {
        let presence = u8::read_from(data, offset)?;
        let inner = if presence != 0 {
            Some(StoreStockSubData::read_from(data, offset)?)
        } else {
            None
        };
        Ok(Self { inner })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match &self.inner {
            Some(v) => { 1u8.write_to(w)?; v.write_to(w) }
            None => 0u8.write_to(w),
        }
    }

    pub fn to_json_value(&self) -> Value {
        match &self.inner {
            Some(v) => Value::Object(v.to_json_dict()),
            None => Value::Null,
        }
    }

    pub fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() {
            w.push(0);
            Ok(())
        } else {
            w.push(1);
            let obj = v.as_object().ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData, "OptionalStoreStockSubData: expected object"))?;
            StoreStockSubData::write_from_json_dict(w, obj)
        }
    }
}

// CArray inner element at StoreStockData a2+72: u32 lookup + u64 raw.
py_binary_struct! {
    pub struct StoreStockEffectEntry {
        pub lookup: u32,    // sub_1410FF430 wire u32
        pub raw: u64,
    }
}

/// `sub_1410F36D0` — StoreStockData (1.0.8: added lookup_c after lookup_b).
///
/// ⚠ DO NOT RENAME these placeholder fields. V3 store mods target them by field-path
/// (e.g. `stock_data_list[N].value.raw_q`, `stock_data_list[N].raw_c`) — GildyBoye's Shop
/// Editor and hand-authored mods alike. Renaming silently breaks every such mod on apply
/// ("missing field"). Canonical RE names live in memory dmm_store_buyprice_itemid_1130_RE
/// (lookup_a=_storeInfo, raw_a=_minPricePercent, raw_c=_maxRefillCount, value=_dropInfoData…).
#[derive(Debug)]
pub struct StoreStockData {
    pub lookup_a: u16,                                    // sub_1410FA410 wire u16
    pub raw_a: u64,
    pub raw_b: u64,
    pub raw_c: u32,
    // 1.16.00: `_lowPriceThresholdCount` (u32), inserted between
    // _maxRefillCount (raw_c) and _stockIndex (raw_d) in the binary's 17-field
    // StockData list. +4 per stock entry, which is exactly the 4n+1 shape of
    // the store record deltas (the +1 being _enterCityWagonStore above).
    // Inserted, NOT renamed: mods address stock_data_list[N].raw_c by name.
    pub low_price_threshold_count_116: u32,
    pub raw_d: u32,
    pub raw_e: u32,
    // 1.13.00: new u32 _orderIndex (Mac reader sub_101FBF4E4: _maxRefillCount,
    // _stockIndex, _importantSaveIndex, _orderIndex — 4 consecutive u32 where
    // 1.12.2 had 3). Default 0xFFFFFFFF (-1). +4 per StoreStockData.
    pub order_index_113: u32,
    pub flag_a: u8,
    pub flag_b: u8,
    pub flag_c: u8,
    // 1.11: new _isRestoreItem u8 between _isStockBuyable (flag_c) and
    // _dropInfoData (value). IDA: StoreStockData reader sub_10190AD14 reads it at
    // a2+47 via the EEC u8 reader. Missing it shifted the value's disc byte →
    // effect_list count blew up at offset 185.
    pub is_restore_item: u8,
    pub value: OptionalStoreStockDataValue,
    pub lookup_b: u32,                                    // sub_1410F61C0 wire u32
    pub lookup_c: u32,                                    // sub_1410F61C0 wire u32  [1.0.8]
    pub sub_data: OptionalStoreStockSubData,
    // 2.01.00: `_maxRefillCondition`, 8 wire bytes between `_conditionOption` (sub_data)
    // and `_orderCountDataList` (effect_list) in the binary's 18-field StockData. Every
    // stock entry of every store grew by exactly 8. Kept opaque (u64) like the other
    // condition-ish quantities here; vanilla writes 0.
    pub max_refill_condition_201: u64,
    pub effect_list: CArray<StoreStockEffectEntry>,
}

impl StoreStockData {
    pub fn read_from(data: &[u8], offset: &mut usize) -> io::Result<Self> {
        let lookup_a = u16::read_from(data, offset)?;
        let raw_a = u64::read_from(data, offset)?;
        let raw_b = u64::read_from(data, offset)?;
        let raw_c = u32::read_from(data, offset)?;
        let low_price_threshold_count_116 = u32::read_from(data, offset)?;
        let raw_d = u32::read_from(data, offset)?;
        let raw_e = u32::read_from(data, offset)?;
        let order_index_113 = u32::read_from(data, offset)?;
        let flag_a = u8::read_from(data, offset)?;
        let flag_b = u8::read_from(data, offset)?;
        let flag_c = u8::read_from(data, offset)?;
        let is_restore_item = u8::read_from(data, offset)?;
        let value = OptionalStoreStockDataValue::read_from(data, offset)?;
        let lookup_b = u32::read_from(data, offset)?;
        let lookup_c = u32::read_from(data, offset)?;
        let sub_data = OptionalStoreStockSubData::read_from(data, offset)?;
        let max_refill_condition_201 = u64::read_from(data, offset)?;
        let effect_list = CArray::<StoreStockEffectEntry>::read_from(data, offset)?;
        Ok(Self {
            lookup_a, raw_a, raw_b, raw_c, low_price_threshold_count_116, raw_d, raw_e, order_index_113,
            flag_a, flag_b, flag_c, is_restore_item, value, lookup_b, lookup_c, sub_data, max_refill_condition_201, effect_list,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.lookup_a.write_to(w)?;
        self.raw_a.write_to(w)?;
        self.raw_b.write_to(w)?;
        self.raw_c.write_to(w)?;
        self.low_price_threshold_count_116.write_to(w)?;
        self.raw_d.write_to(w)?;
        self.raw_e.write_to(w)?;
        self.order_index_113.write_to(w)?;
        self.flag_a.write_to(w)?;
        self.flag_b.write_to(w)?;
        self.flag_c.write_to(w)?;
        self.is_restore_item.write_to(w)?;
        self.value.write_to(w)?;
        self.lookup_b.write_to(w)?;
        self.lookup_c.write_to(w)?;
        self.sub_data.write_to(w)?;
        self.max_refill_condition_201.write_to(w)?;
        self.effect_list.write_to(w)
    }

    pub fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("lookup_a".to_string(), self.lookup_a.to_json_value());
        m.insert("raw_a".to_string(), self.raw_a.to_json_value());
        m.insert("raw_b".to_string(), self.raw_b.to_json_value());
        m.insert("raw_c".to_string(), self.raw_c.to_json_value());
        m.insert("low_price_threshold_count_116".to_string(), self.low_price_threshold_count_116.to_json_value());
        m.insert("raw_d".to_string(), self.raw_d.to_json_value());
        m.insert("raw_e".to_string(), self.raw_e.to_json_value());
        m.insert("order_index_113".to_string(), self.order_index_113.to_json_value());
        m.insert("flag_a".to_string(), self.flag_a.to_json_value());
        m.insert("flag_b".to_string(), self.flag_b.to_json_value());
        m.insert("flag_c".to_string(), self.flag_c.to_json_value());
        m.insert("is_restore_item".to_string(), self.is_restore_item.to_json_value());
        m.insert("value".to_string(), self.value.to_json_value());
        m.insert("lookup_b".to_string(), self.lookup_b.to_json_value());
        m.insert("lookup_c".to_string(), self.lookup_c.to_json_value());
        m.insert("sub_data".to_string(), self.sub_data.to_json_value());
        m.insert("max_refill_condition_201".to_string(), self.max_refill_condition_201.to_json_value());
        m.insert("effect_list".to_string(), self.effect_list.to_json_value());
        Value::Object(m)
    }

    pub fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "StoreStockData: expected object"))?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_a")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_a")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_b")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_c")?)?;
        // null-safe like order_index_113 below: mods written before 1.16 have no
        // low_price_threshold_count_116 key, and must keep applying.
        <u32 as WriteJsonValue>::write_from_json(w,
            obj.get("low_price_threshold_count_116").unwrap_or(&serde_json::Value::Null))?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_d")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "raw_e")?)?;
        // null-safe: old mods omit order_index_113 → default 0.
        <u32 as WriteJsonValue>::write_from_json(w, obj.get("order_index_113").unwrap_or(&Value::Null))?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_a")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_b")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "flag_c")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_restore_item")?)?;
        OptionalStoreStockDataValue::write_from_json(w, json_get_field(obj, "value")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_b")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "lookup_c")?)?;
        OptionalStoreStockSubData::write_from_json(w, json_get_field(obj, "sub_data")?)?;
        // Null-tolerant: stock entries captured before 2.01.00 have no key for this.
        <u64 as WriteJsonValue>::write_from_json(w, obj.get("max_refill_condition_201").unwrap_or(&Value::Null))?;
        <CArray<StoreStockEffectEntry> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "effect_list")?)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct StoreInfo<'a> {
    pub key: u16,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub exchange_item_info_for_buy: u32,
    pub exchange_item_info_list_for_sell: CArray<u32>,
    pub sell_percents: u64,
    pub store_type: u8,
    pub price_increase_percent_list: CArray<u64>,
    pub sellable_character_condition_logic: u8,
    // 1.11: new u8 read right after _sellableCharacterConditionLogic, before
    // _resetHour (StoreInfo reader sub_10190B0A0 @ byte 73). Shifts the whole
    // stock_data_list region — without it the value disc misreads.
    pub pre_reset_extra_111: u8,
    // ── 2.01.00: `_hasStockCondition`, one u8 between `_hasRestoreItem` (the field
    // above, named before the oracle existed) and `_enterCityWagonStore`. Record 0 grew
    // by exactly 1 byte at offset 33.
    pub has_stock_condition: u8,
    // 1.16.00: `_enterCityWagonStore` (u8). The binary's 23-field StoreInfo
    // list has THREE flags here — _hasSellableCharacter, _hasRestoreItem,
    // _enterCityWagonStore — where we modelled only two, leaving every record
    // 1 byte short (the store deltas cluster on 4n+1) and desyncing the stock
    // list, which surfaced as the bogus "unknown disc 182".
    pub enter_city_wagon_store_116: u8,
    pub reset_hour: u32,
    pub reset_day: u32,
    pub buyable_stock_count: u32,
    pub sellable_stock_count: u32,
    pub sellable_type: u8,
    pub stock_data_list: Vec<StoreStockData>,
    pub sale_item_type_list: CArray<u8>,
    pub not_sale_item_type_list: CArray<u8>,
    pub custom_mesh_obb_max_length: u32,
    pub fixed_price: u8,
    pub use_housing_gimmick: u8,
    pub reduce_price_by_looted_dead_body: u8,
}

impl<'a> StoreInfo<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        let consumed = *offset - start;
        if consumed != entry_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("StoreInfo: consumed {} bytes, expected {}", consumed, entry_size),
            ));
        }
        Ok(item)
    }

    pub fn read_tracked_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        let start = *offset;
        let key = track_read_field::<u16>(data, offset, path, ranges, "key", "u16")?;
        let string_key = track_read_field::<CString<'a>>(data, offset, path, ranges, "string_key", "CString")?;
        let is_blocked = track_read_field::<u8>(data, offset, path, ranges, "is_blocked", "u8")?;
        let exchange_item_info_for_buy = track_read_field::<u32>(data, offset, path, ranges, "exchange_item_info_for_buy", "u32")?;
        let exchange_item_info_list_for_sell = track_read_field::<CArray<u32>>(data, offset, path, ranges, "exchange_item_info_list_for_sell", "CArray<u32>")?;
        let sell_percents = track_read_field::<u64>(data, offset, path, ranges, "sell_percents", "u64")?;
        let store_type = track_read_field::<u8>(data, offset, path, ranges, "store_type", "u8")?;
        let price_increase_percent_list = track_read_field::<CArray<u64>>(data, offset, path, ranges, "price_increase_percent_list", "CArray<u64>")?;
        let sellable_character_condition_logic = track_read_field::<u8>(data, offset, path, ranges, "sellable_character_condition_logic", "u8")?;
        let pre_reset_extra_111 = track_read_field::<u8>(data, offset, path, ranges, "pre_reset_extra_111", "u8")?;
        let has_stock_condition = track_read_field::<u8>(data, offset, path, ranges, "has_stock_condition", "u8")?;
        let enter_city_wagon_store_116 = track_read_field::<u8>(data, offset, path, ranges, "enter_city_wagon_store_116", "u8")?;
        let reset_hour = track_read_field::<u32>(data, offset, path, ranges, "reset_hour", "u32")?;
        let reset_day = track_read_field::<u32>(data, offset, path, ranges, "reset_day", "u32")?;
        let buyable_stock_count = track_read_field::<u32>(data, offset, path, ranges, "buyable_stock_count", "u32")?;
        let sellable_stock_count = track_read_field::<u32>(data, offset, path, ranges, "sellable_stock_count", "u32")?;
        let sellable_type = track_read_field::<u8>(data, offset, path, ranges, "sellable_type", "u8")?;
        let stock_data_list = track_read_with(offset, path, ranges, "stock_data_list", "Vec<StoreStockData>", |o| {
            let count = u32::read_from(data, o)?;
            // Sanity clamp (mirrors CArray in binary/types.rs): each StoreStockData
            // is >= 1 byte, so a count exceeding the remaining byte budget is a
            // corrupted/misaligned stream. Returning Err here turns a garbage count
            // into a clean blob-fallback instead of a >500GB Vec::with_capacity abort.
            let remaining = data.len().saturating_sub(*o);
            if count as usize > remaining {
                return Err(io::Error::new(io::ErrorKind::InvalidData,
                    format!(
                        "storeinfo stock_data_list count {} exceeds remaining {} at offset {}",
                        count, remaining, *o,
                    )));
            }
            let mut v = Vec::with_capacity((count as usize).min(1 << 20));
            for _ in 0..count {
                v.push(StoreStockData::read_from(data, o)?);
            }
            Ok(v)
        })?;
        let sale_item_type_list = track_read_field::<CArray<u8>>(data, offset, path, ranges, "sale_item_type_list", "CArray<u8>")?;
        let not_sale_item_type_list = track_read_field::<CArray<u8>>(data, offset, path, ranges, "not_sale_item_type_list", "CArray<u8>")?;
        let custom_mesh_obb_max_length = track_read_field::<u32>(data, offset, path, ranges, "custom_mesh_obb_max_length", "u32")?;
        let fixed_price = track_read_field::<u8>(data, offset, path, ranges, "fixed_price", "u8")?;
        let use_housing_gimmick = track_read_field::<u8>(data, offset, path, ranges, "use_housing_gimmick", "u8")?;
        let reduce_price_by_looted_dead_body = track_read_field::<u8>(data, offset, path, ranges, "reduce_price_by_looted_dead_body", "u8")?;
        let consumed = *offset - start;
        if consumed != entry_size {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("StoreInfo: consumed {} bytes, expected {}", consumed, entry_size)));
        }
        Ok(Self {
            key, string_key, is_blocked,
            exchange_item_info_for_buy, exchange_item_info_list_for_sell,
            sell_percents, store_type, price_increase_percent_list,
            sellable_character_condition_logic, pre_reset_extra_111,
            has_stock_condition, enter_city_wagon_store_116, reset_hour, reset_day,
            buyable_stock_count, sellable_stock_count, sellable_type,
            stock_data_list, sale_item_type_list, not_sale_item_type_list,
            custom_mesh_obb_max_length,
            fixed_price, use_housing_gimmick, reduce_price_by_looted_dead_body,
        })
    }

    pub fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let key = u16::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;
        let exchange_item_info_for_buy = u32::read_from(data, offset)?;
        let exchange_item_info_list_for_sell = CArray::<u32>::read_from(data, offset)?;
        let sell_percents = u64::read_from(data, offset)?;
        let store_type = u8::read_from(data, offset)?;
        let price_increase_percent_list = CArray::<u64>::read_from(data, offset)?;
        let sellable_character_condition_logic = u8::read_from(data, offset)?;
        let pre_reset_extra_111 = u8::read_from(data, offset)?;
        let has_stock_condition = u8::read_from(data, offset)?;
        let enter_city_wagon_store_116 = u8::read_from(data, offset)?;
        let reset_hour = u32::read_from(data, offset)?;
        let reset_day = u32::read_from(data, offset)?;
        let buyable_stock_count = u32::read_from(data, offset)?;
        let sellable_stock_count = u32::read_from(data, offset)?;
        let sellable_type = u8::read_from(data, offset)?;
        let count = u32::read_from(data, offset)?;
        // Sanity clamp (mirrors CArray in binary/types.rs): each StoreStockData
        // is >= 1 byte, so a count exceeding the remaining byte budget is a
        // corrupted/misaligned stream. Returning Err here turns a garbage count
        // into a clean blob-fallback instead of a >500GB Vec::with_capacity abort.
        let remaining = data.len().saturating_sub(*offset);
        if count as usize > remaining {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!(
                    "storeinfo stock_data_list count {} exceeds remaining {} at offset {}",
                    count, remaining, *offset,
                )));
        }
        let mut stock_data_list = Vec::with_capacity((count as usize).min(1 << 20));
        for _ in 0..count {
            stock_data_list.push(StoreStockData::read_from(data, offset)?);
        }
        let sale_item_type_list = CArray::<u8>::read_from(data, offset)?;
        let not_sale_item_type_list = CArray::<u8>::read_from(data, offset)?;
        let custom_mesh_obb_max_length = u32::read_from(data, offset)?;
        let fixed_price = u8::read_from(data, offset)?;
        let use_housing_gimmick = u8::read_from(data, offset)?;
        let reduce_price_by_looted_dead_body = u8::read_from(data, offset)?;
        Ok(Self {
            key, string_key, is_blocked,
            exchange_item_info_for_buy, exchange_item_info_list_for_sell,
            sell_percents, store_type, price_increase_percent_list,
            sellable_character_condition_logic, pre_reset_extra_111,
            has_stock_condition, enter_city_wagon_store_116, reset_hour, reset_day,
            buyable_stock_count, sellable_stock_count, sellable_type,
            stock_data_list, sale_item_type_list, not_sale_item_type_list,
            custom_mesh_obb_max_length,
            fixed_price, use_housing_gimmick, reduce_price_by_looted_dead_body,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.exchange_item_info_for_buy.write_to(w)?;
        self.exchange_item_info_list_for_sell.write_to(w)?;
        self.sell_percents.write_to(w)?;
        self.store_type.write_to(w)?;
        self.price_increase_percent_list.write_to(w)?;
        self.sellable_character_condition_logic.write_to(w)?;  // u8 in 1.0.8 (was u32 in 1.0.7)
        self.pre_reset_extra_111.write_to(w)?;
        self.has_stock_condition.write_to(w)?;
        self.enter_city_wagon_store_116.write_to(w)?;
        self.reset_hour.write_to(w)?;
        self.reset_day.write_to(w)?;
        self.buyable_stock_count.write_to(w)?;
        self.sellable_stock_count.write_to(w)?;
        self.sellable_type.write_to(w)?;
        (self.stock_data_list.len() as u32).write_to(w)?;
        for sd in &self.stock_data_list { sd.write_to(w)?; }
        self.sale_item_type_list.write_to(w)?;
        self.not_sale_item_type_list.write_to(w)?;
        self.custom_mesh_obb_max_length.write_to(w)?;
        self.fixed_price.write_to(w)?;
        self.use_housing_gimmick.write_to(w)?;
        self.reduce_price_by_looted_dead_body.write_to(w)
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("exchange_item_info_for_buy".to_string(), self.exchange_item_info_for_buy.to_json_value());
        m.insert("exchange_item_info_list_for_sell".to_string(), self.exchange_item_info_list_for_sell.to_json_value());
        m.insert("sell_percents".to_string(), self.sell_percents.to_json_value());
        m.insert("store_type".to_string(), self.store_type.to_json_value());
        m.insert("price_increase_percent_list".to_string(), self.price_increase_percent_list.to_json_value());
        m.insert("sellable_character_condition_logic".to_string(), self.sellable_character_condition_logic.to_json_value());
        m.insert("pre_reset_extra_111".to_string(), self.pre_reset_extra_111.to_json_value());
        m.insert("has_stock_condition".to_string(), self.has_stock_condition.to_json_value());
        m.insert("enter_city_wagon_store_116".to_string(), self.enter_city_wagon_store_116.to_json_value());
        m.insert("reset_hour".to_string(), self.reset_hour.to_json_value());
        m.insert("reset_day".to_string(), self.reset_day.to_json_value());
        m.insert("buyable_stock_count".to_string(), self.buyable_stock_count.to_json_value());
        m.insert("sellable_stock_count".to_string(), self.sellable_stock_count.to_json_value());
        m.insert("sellable_type".to_string(), self.sellable_type.to_json_value());
        let stock_list: Vec<Value> = self.stock_data_list.iter().map(|s| s.to_json_value()).collect();
        m.insert("stock_data_list".to_string(), Value::Array(stock_list));
        m.insert("sale_item_type_list".to_string(), self.sale_item_type_list.to_json_value());
        m.insert("not_sale_item_type_list".to_string(), self.not_sale_item_type_list.to_json_value());
        m.insert("custom_mesh_obb_max_length".to_string(), self.custom_mesh_obb_max_length.to_json_value());
        m.insert("fixed_price".to_string(), self.fixed_price.to_json_value());
        m.insert("use_housing_gimmick".to_string(), self.use_housing_gimmick.to_json_value());
        m.insert("reduce_price_by_looted_dead_body".to_string(), self.reduce_price_by_looted_dead_body.to_json_value());
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "exchange_item_info_for_buy")?)?;
        <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "exchange_item_info_list_for_sell")?)?;
        <u64 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "sell_percents")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "store_type")?)?;
        <CArray<u64> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "price_increase_percent_list")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "sellable_character_condition_logic")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "pre_reset_extra_111")?)?;
        // Null-tolerant: a V3 store mod authored before 2.01.00 has no key for this.
        <u8 as WriteJsonValue>::write_from_json(w, obj.get("has_stock_condition").unwrap_or(&Value::Null))?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "enter_city_wagon_store_116")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "reset_hour")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "reset_day")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "buyable_stock_count")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "sellable_stock_count")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "sellable_type")?)?;
        let arr = json_get_field(obj, "stock_data_list")?
            .as_array()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "stock_data_list: expected array"))?;
        (arr.len() as u32).write_to(w)?;
        for v in arr { StoreStockData::write_from_json(w, v)?; }
        <CArray<u8> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "sale_item_type_list")?)?;
        <CArray<u8> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "not_sale_item_type_list")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "custom_mesh_obb_max_length")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "fixed_price")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "use_housing_gimmick")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "reduce_price_by_looted_dead_body")?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    fn pabgb_path() -> std::path::PathBuf { crate::testenv::resolve("storeinfo.pabgb") }
#[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                StoreInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "storeinfo roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(pabgb_path()) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(&pabgb_path().with_extension("pabgh").to_string_lossy()) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = StoreInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            StoreInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }

    /// Regression for the DMM 1.4.7.1 mid-mount `0xc0000409` OOM abort
    /// (`docs/diag-1.4.7.1-storeinfo-mount-oom-abort.md`). A misaligned /
    /// corrupted storeinfo body whose `stock_data_list` count reads as a
    /// garbage `u32` (here `0xB401D000` ≈ 3.0e9) must produce a clean,
    /// catchable `Err` — NOT an unbounded `Vec::with_capacity` that fails
    /// to allocate ~579 GB and `abort()`s the process. The clamp at
    /// `read_from` / `read_tracked_with_size` returns `Err` when the count
    /// exceeds the remaining byte budget.
    #[test]
    fn garbage_stock_count_is_err_not_abort() {
        let mut body: Vec<u8> = Vec::new();
        body.extend_from_slice(&0u16.to_le_bytes()); // key
        body.extend_from_slice(&0u32.to_le_bytes()); // string_key CString len=0
        body.push(0); // is_blocked
        body.extend_from_slice(&0u32.to_le_bytes()); // exchange_item_info_for_buy
        body.extend_from_slice(&0u32.to_le_bytes()); // exchange_item_info_list_for_sell count=0
        body.extend_from_slice(&0u64.to_le_bytes()); // sell_percents
        body.push(0); // store_type
        body.extend_from_slice(&0u32.to_le_bytes()); // price_increase_percent_list count=0
        body.push(0); // sellable_character_condition_logic
        body.push(0); // pre_reset_extra_111
        body.push(0); // has_stock_condition (2.01.00)
        body.push(0); // enter_city_wagon_store_116
        body.extend_from_slice(&0u32.to_le_bytes()); // reset_hour
        body.extend_from_slice(&0u32.to_le_bytes()); // reset_day
        body.extend_from_slice(&0u32.to_le_bytes()); // buyable_stock_count
        body.extend_from_slice(&0u32.to_le_bytes()); // sellable_stock_count
        body.push(0); // sellable_type
        body.extend_from_slice(&0xB401D000u32.to_le_bytes()); // stock_data_list count = GARBAGE

        // Untracked reader.
        let mut off = 0usize;
        let res = StoreInfo::read_from(&body, &mut off);
        assert!(res.is_err(), "garbage stock count must return Err, not abort");

        // Tracked reader (the other unbounded site) must also be Err.
        let mut off2 = 0usize;
        let mut path = String::new();
        let mut ranges: Vec<crate::binary::FieldRange> = Vec::new();
        let res2 = StoreInfo::read_tracked_with_size(
            &body,
            &mut off2,
            body.len(),
            &mut path,
            &mut ranges,
        );
        assert!(res2.is_err(), "tracked reader: garbage stock count must return Err, not abort");
    }
}
