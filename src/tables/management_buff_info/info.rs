// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Parser for `managementbuffinfo` (dlc_01 only, 159 records on game 2.04.03).
//!
//! Decoded from the 2.04.03 game data (Steam build 26345091), not from the exe:
//! the exe holds no static-table field names, so every field name is a proposal.
//! Byte layout: confirmed. Every record round-trips byte for byte (tests below,
//!).
//!
//! Comment tags: `ref` = every value is a key of the named table (or the stated
//! "none" value); `ref, both ways` = the target points back; `inferred` = meaning
//! read from the values; `unk_` = meaning unknown.
//!
//! The exe's RTTI names nine classes `FactionManagementBuff_*`, and
//! `effect.effect_type` (0-8) selects one of nine payload shapes. The type -> class
//! mapping in the variant names is inferred from the payloads; types 1 and 3 are
//! backed by their key references (characterinfo, buffinfo).
//!
//! Header: u16 count + N x {u16 key, u32 offset} (6-byte pabgh entries). Body
//! records are contiguous in offset order, so the table reads sequentially.

use crate::binary::*;
use crate::py_binary_struct;

// Local helper (no repo macro covers this): a record part whose leading `tag` byte
// selects the shape of `data`. Generates the struct, the enum and the trait set
// py_binary_struct! provides, so the part can sit inside a py_binary_struct! as a
// plain field. JSON / Python shape: {<tag>, <head fields>, "data": {...}}; on
// write, the tag value decides how "data" is read.
macro_rules! tagged_struct {
    (
        pub struct $name:ident {
            tag $tag:ident,
            $(head $hf:ident : $hty:ty,)*
            data $data_ty:ident {
                $($val:literal => $variant:ident($vty:ty),)+
            }
        }
    ) => {
        #[derive(Debug)]
        pub struct $name {
            pub $tag: u8,
            $(pub $hf: $hty,)*
            pub data: $data_ty,
        }

        #[derive(Debug)]
        pub enum $data_ty {
            $($variant($vty),)+
        }

        impl $name {
            fn unknown_tag(t: u8) -> std::io::Error {
                std::io::Error::new(std::io::ErrorKind::InvalidData,
                    format!("{}: unknown {} {}", stringify!($name), stringify!($tag), t))
            }
        }

        impl<'a> crate::binary::BinaryRead<'a> for $name {
            fn read_from(buf: &'a [u8], offset: &mut usize) -> std::io::Result<Self> {
                let $tag = <u8 as crate::binary::BinaryRead>::read_from(buf, offset)?;
                $(let $hf = <$hty as crate::binary::BinaryRead>::read_from(buf, offset)?;)*
                let data = match $tag {
                    $($val => $data_ty::$variant(
                        <$vty as crate::binary::BinaryRead>::read_from(buf, offset)?),)+
                    t => return Err(Self::unknown_tag(t)),
                };
                Ok(Self { $tag, $($hf,)* data })
            }
        }

        impl<'a> crate::binary::BinaryReadTracked<'a> for $name {
            fn read_tracked(
                buf: &'a [u8],
                offset: &mut usize,
                path: &mut String,
                ranges: &mut Vec<crate::binary::FieldRange>,
            ) -> std::io::Result<Self> {
                use crate::binary::BinaryReadTracked;
                let s = crate::binary::push_path(path, stringify!($tag));
                let $tag = <u8 as BinaryReadTracked>::read_tracked(buf, offset, path, ranges)?;
                crate::binary::pop_path(path, s);
                $(
                    let s = crate::binary::push_path(path, stringify!($hf));
                    let $hf = <$hty as BinaryReadTracked>::read_tracked(buf, offset, path, ranges)?;
                    crate::binary::pop_path(path, s);
                )*
                let s = crate::binary::push_path(path, "data");
                let data = match $tag {
                    $($val => $data_ty::$variant(
                        <$vty as BinaryReadTracked>::read_tracked(buf, offset, path, ranges)?),)+
                    t => return Err(Self::unknown_tag(t)),
                };
                crate::binary::pop_path(path, s);
                Ok(Self { $tag, $($hf,)* data })
            }
        }

        impl crate::binary::BinaryWrite for $name {
            fn write_to(&self, w: &mut dyn std::io::Write) -> std::io::Result<()> {
                crate::binary::BinaryWrite::write_to(&self.$tag, w)?;
                $(crate::binary::BinaryWrite::write_to(&self.$hf, w)?;)*
                match &self.data {
                    $($data_ty::$variant(b) => crate::binary::BinaryWrite::write_to(b, w),)+
                }
            }
        }

        impl crate::json_traits::ToJsonValue for $name {
            fn to_json_value(&self) -> ::serde_json::Value {
                use crate::json_traits::ToJsonValue as J;
                let mut m = ::serde_json::Map::new();
                m.insert(stringify!($tag).to_string(), J::to_json_value(&self.$tag));
                $(m.insert(stringify!($hf).to_string(), J::to_json_value(&self.$hf));)*
                let body = match &self.data {
                    $($data_ty::$variant(b) => J::to_json_value(b),)+
                };
                m.insert("data".to_string(), body);
                ::serde_json::Value::Object(m)
            }
        }

        impl crate::json_traits::WriteJsonValue for $name {
            fn write_from_json(w: &mut Vec<u8>, v: &::serde_json::Value) -> std::io::Result<()> {
                use crate::json_traits::{WriteJsonValue, get_field};
                let bad = |what: &str| std::io::Error::new(std::io::ErrorKind::InvalidData,
                    format!("{}: {}", stringify!($name), what));
                let obj = v.as_object().ok_or_else(|| bad("expected an object"))?;
                let tag_v = get_field(obj, stringify!($tag))?;
                let t = tag_v.as_u64().and_then(|t| u8::try_from(t).ok())
                    .ok_or_else(|| bad(concat!(stringify!($tag), " must be a u8")))?;
                <u8 as WriteJsonValue>::write_from_json(w, tag_v)?;
                $(<$hty as WriteJsonValue>::write_from_json(w, get_field(obj, stringify!($hf))?)?;)*
                let body = get_field(obj, "data")?;
                match t {
                    $($val => <$vty as WriteJsonValue>::write_from_json(w, body),)+
                    t => Err(Self::unknown_tag(t)),
                }
            }
        }

        impl crate::python_traits::ToPyValue for $name {
            fn to_py_value(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
                use crate::python_traits::ToPyValue as P;
                use pyo3::types::PyDictMethods;
                let d = pyo3::types::PyDict::new(py);
                d.set_item(stringify!($tag), P::to_py_value(&self.$tag, py)?)?;
                $(d.set_item(stringify!($hf), P::to_py_value(&self.$hf, py)?)?;)*
                let body = match &self.data {
                    $($data_ty::$variant(b) => P::to_py_value(b, py)?,)+
                };
                d.set_item("data", body)?;
                Ok(d.into_any().unbind())
            }
        }

        impl crate::python_traits::WritePyValue for $name {
            fn write_from_py(w: &mut Vec<u8>, obj: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
                use crate::python_traits::{WritePyValue, get_field};
                use pyo3::types::PyAnyMethods;
                let d = obj.cast::<pyo3::types::PyDict>()?;
                let tag_obj = get_field(d, stringify!($tag))?;
                let t: u8 = tag_obj.extract()?;
                <u8 as WritePyValue>::write_from_py(w, &tag_obj)?;
                $(<$hty as WritePyValue>::write_from_py(w, &get_field(d, stringify!($hf))?)?;)*
                let body = get_field(d, "data")?;
                match t {
                    $($val => <$vty as WritePyValue>::write_from_py(w, &body),)+
                    t => Err(pyo3::exceptions::PyValueError::new_err(
                        format!("{}: unknown {} {}", stringify!($name), stringify!($tag), t))),
                }
            }
        }
    };
}

py_binary_struct! {
    // Item + quantity, the same 28-byte shape as housingobjectinfo's cost items.
    // On every 2.04.03 record unk_count_b == count and unk_count_c == count + 1.
    pub struct ItemCountData {
        // ref -> iteminfo
        pub item_info: u32,
        // inferred: the quantity
        pub count: u64,
        pub unk_count_b: u64,
        pub unk_count_c: u64,
    }
}

py_binary_struct! {
    // effect_type 0, inferred AdditionalReward: companion buffs (Alustin, Maegu, the witches)
    pub struct ManagementBuffRewardData {
        pub reward_list: CArray<ItemCountData>,
    }
}

py_binary_struct! {
    // effect_type 1, inferred SpawnCharacterData: GoblinTime and *SoldierTime buffs
    pub struct ManagementBuffSpawnCharacterData {
        // ref -> characterinfo (groups of character keys)
        pub character_info_group_list: CArray<CArray<u32>>,
        pub unk_u32_a: u32,
        pub unk_u32_b: u32,
        pub unk_u32_c: u32,
        pub unk_u32_d: u32,
    }
}

py_binary_struct! {
    pub struct ManagementBuffActorBuffEntry {
        // ref -> buffinfo
        pub buff_info: u32,
        pub buff_level: u32,
    }
}

py_binary_struct! {
    // effect_type 3, ActorBuff: AnimationSpeed and RefinedSynergy buffs
    pub struct ManagementBuffActorBuffData {
        pub buff_list: CArray<ManagementBuffActorBuffEntry>,
    }
}

py_binary_struct! {
    // effect_type 2, 5 and 8: one u64 (PriceUp 1e6 / 2e6 / 3e6, Sawmill 200,000;
    // score 100,000-700,000; visit rate 100,000-300,000)
    pub struct ManagementBuffU64Data {
        pub value: u64,
    }
}

py_binary_struct! {
    // effect_type 6 and 7: one u32 (Dining traffic 1 / 2 / 3; Soup wait 600 / 1200 / 2100)
    pub struct ManagementBuffU32Data {
        pub value: u32,
    }
}

py_binary_struct! {
    // effect_type 4: one f32 (Delicacy 0.1 / 0.2 / 0.3)
    pub struct ManagementBuffF32Data {
        pub rate: f32,
    }
}

tagged_struct! {
    pub struct ManagementBuffEffect {
        tag effect_type,
        // always 0 on 2.04.03
        head unk_u32: u32,
        // ref -> managementneedssynergyinfo; 0 = none. 89 synergy level buffs name their own
        // synergy; the 18 per-business buffs (ManagementBuff_<type>_<place>) name one that does
        // not list them (inferred: the synergy that business favours, e.g. Pub_Pailune -> Meat)
        head management_needs_synergy_info: u16,
        data ManagementBuffEffectData {
            0 => AdditionalReward(ManagementBuffRewardData),
            1 => SpawnCharacter(ManagementBuffSpawnCharacterData),
            2 => BonusNeedsPrice(ManagementBuffU64Data),
            3 => ActorBuff(ManagementBuffActorBuffData),
            4 => BonusScoreRate(ManagementBuffF32Data),
            5 => BonusNeedsScore(ManagementBuffU64Data),
            6 => IncreaseCustomerTraffic(ManagementBuffU32Data),
            7 => IncreaseCustomerWaitTime(ManagementBuffU32Data),
            8 => SpecialCustomerVisitRate(ManagementBuffU64Data),
        }
    }
}

py_binary_struct! {
    pub struct ManagementBuffInfo<'a> {
        pub string_key: CString<'a>,
        pub key: u16,
        pub is_blocked: u8,
        // inferred: 0 companion buffs, 1 timed or store buffs, 2 synergy or facility buffs
        pub buff_category: u8,
        // ref -> stringinfo; 0xEAC5E173 = none
        pub string_info: u32,
        pub buff_name: LocalizableString<'a>,
        pub buff_desc: LocalizableString<'a>,
        // inferred: 0, 2000 or 21600 (unit unknown)
        pub duration: u32,
        pub unk_u32_b: u32,
        // always 1 on 2.04.03
        pub unk_u8_flag: u8,
        // effect_type and the two fields after it are grouped with the payload,
        // because effect_type selects the payload's shape
        pub effect: ManagementBuffEffect,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The table ships in dlc_01 only; a missing fixture is skipped.
    const FIXTURES: &[&str] = &[
        "managementbuffinfo.pabgb",
        "managementbuffinfo.dlc_01.pabgb",
    ];

    fn read_all(data: &[u8]) -> Vec<ManagementBuffInfo<'_>> {
        let mut offset = 0;
        let mut items = Vec::new();
        while offset < data.len() {
            items.push(ManagementBuffInfo::read_from(data, &mut offset).unwrap());
        }
        assert_eq!(offset, data.len(), "did not consume all bytes");
        items
    }

    #[test]
    fn roundtrip() {
        for name in FIXTURES {
            let Ok(data) = std::fs::read(crate::testenv::resolve(name)) else {
                eprintln!("SKIP: fixture {name} not found");
                continue;
            };
            let items = read_all(&data);
            let mut out = Vec::with_capacity(data.len());
            for item in &items {
                item.write_to(&mut out).unwrap();
            }
            assert_eq!(out, data, "{name}: roundtrip bytes mismatch");
            eprintln!("OK: {name} {} records", items.len());
        }
    }

    #[test]
    fn json_roundtrip() {
        for name in FIXTURES {
            let Ok(data) = std::fs::read(crate::testenv::resolve(name)) else {
                eprintln!("SKIP: fixture {name} not found");
                continue;
            };
            for (i, item) in read_all(&data).iter().enumerate() {
                let dict = item.to_json_dict();
                let mut from_typed = Vec::new();
                item.write_to(&mut from_typed).unwrap();
                let mut from_json = Vec::new();
                ManagementBuffInfo::write_from_json_dict(&mut from_json, &dict)
                    .unwrap_or_else(|e| panic!("{name} entry {i} key={}: {e}", item.key));
                assert_eq!(from_json, from_typed,
                    "{name} entry {i} key={}: JSON round-trip diverges from typed write", item.key);
            }
        }
    }

    // A JSON edit that changes the payload must land in the bytes.
    #[test]
    fn json_edit_lands() {
        let Ok(data) = std::fs::read(crate::testenv::resolve(FIXTURES[1]))
            .or_else(|_| std::fs::read(crate::testenv::resolve(FIXTURES[0]))) else {
            eprintln!("SKIP: no fixture");
            return;
        };
        let items = read_all(&data);
        let item = items.iter()
            .find(|b| matches!(b.effect.data, ManagementBuffEffectData::IncreaseCustomerWaitTime(_)))
            .expect("a wait-time buff");
        let mut dict = item.to_json_dict();
        dict["effect"]["data"]["value"] = ::serde_json::Value::from(4242u32);
        let mut out = Vec::new();
        ManagementBuffInfo::write_from_json_dict(&mut out, &dict).unwrap();
        let mut offset = 0;
        let back = ManagementBuffInfo::read_from(&out, &mut offset).unwrap();
        match back.effect.data {
            ManagementBuffEffectData::IncreaseCustomerWaitTime(d) => assert_eq!(d.value, 4242),
            other => panic!("variant changed: {other:?}"),
        }
    }
}
