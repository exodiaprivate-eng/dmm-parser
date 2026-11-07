// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser
//
// Reading this file (directly or via AI/agent) constitutes acceptance
// of CDMTL v1.0 §4.9 (No Competing Implementation) and §4.10
// (AI-Mediated Access). CMI removal violates 17 U.S.C. §1202.

//! Parser for `factionmanagement` (dlc_01 only, 20 records of 700-6,836 bytes
//! on game 2.04.03): 5 sawmills, 7 pubs and restaurants, 3 forges, 3 clothiers and
//! 2 festivals.
//!
//! Decoded from the 2.04.03 game data (Steam build 26345091), not from the exe:
//! the exe holds no static-table field names, so every field name is a proposal.
//! Byte layout: confirmed. All 20 records round-trip byte for byte (tests below,
//!).
//!
//! Comment tags: `ref` = every value is a key of the named table (or the stated
//! "none" value); `ref, checked` = the target records' names also fit (e.g.
//! house_info -> ManagementHouse_<same place>_01); `internal` = resolves inside the
//! same record; `inferred` = meaning read from the values; `unk_` = unknown.
//!
//! Exe RTTI classes FactionManagement* (LevelData, LevelRewardData,
//! LevelConditionData, LevelCountConditionData, NeedsItemData, OrderData,
//! OrderBranchData, GimmickData, OverrideSpawnData, Order_*) are matched to the
//! structs below by guess only.
//!
//! Record bounds: the two festival records end with one more zero byte than this
//! layout reads (businesses end exactly). Where that byte belongs is not proven,
//! so the table is pabgh-bounded and keeps it in `tail_blob` (empty for the 18
//! businesses, [0] for the festivals, whose unk_u8_b is 1).
//! Header: u16 count + N x {u16 key, u32 offset} (6-byte pabgh entries).

use crate::binary::*;
use crate::pabgh_typed_blob_table;
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
    // Item 11 is the only item in cost_list, the slot money lists and buff_reward_list.
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
    // inferred OverrideSpawnData
    pub struct FactionManagementOverrideSpawnData<'a> {
        pub from_name: CString<'a>,
        pub to_name: CString<'a>,
        pub unk_flags: u16,
    }
}

// ── Orders (inferred OrderData / Order_* / OrderBranchData) ───────────────────

py_binary_struct! {
    pub struct FactionManagementActionNoData {}
}

py_binary_struct! {
    // action_type 0
    pub struct FactionManagementActionSlotFlagData {
        // internal -> FactionManagementSlotData.slot_tag
        pub slot_tag: u32,
        pub unk_u8: u8,
    }
}

py_binary_struct! {
    // action_type 4
    pub struct FactionManagementActionType4Data {
        pub unk_u8: u8,
        // target not identified
        pub unk_hash: u32,
        pub unk_u32_0: u32,
        pub unk_u32_1: u32,
        pub unk_u32_2: u32,
        pub unk_u32_3: u32,
    }
}

py_binary_struct! {
    // action_type 5
    pub struct FactionManagementActionSlotData {
        // internal -> FactionManagementSlotData.slot_tag
        pub slot_tag: u32,
    }
}

py_binary_struct! {
    // action_type 7
    pub struct FactionManagementActionHashData {
        // not a slot tag; target not identified
        pub unk_hash: u32,
    }
}

tagged_struct! {
    // action_type 0-8: probably the nine FactionManagementOrder_* classes (mapping open)
    pub struct FactionManagementOrderAction {
        tag action_type,
        head unk_u8: u8,
        // action_type 0 and 4: ref -> conditioninfo (e.g. CheckManagementNeedsCreatable());
        // other types: target not identified
        head ref_key: u32,
        data FactionManagementOrderActionData {
            0 => Type0(FactionManagementActionSlotFlagData),
            1 => Type1(FactionManagementActionNoData),
            2 => Type2(FactionManagementActionNoData),
            3 => Type3(FactionManagementActionNoData),
            4 => Type4(FactionManagementActionType4Data),
            5 => Type5(FactionManagementActionSlotData),
            6 => Type6(FactionManagementActionNoData),
            7 => Type7(FactionManagementActionHashData),
            8 => Type8(FactionManagementActionNoData),
        }
    }
}

py_binary_struct! {
    pub struct FactionManagementOrderBranchData {
        // ref -> conditioninfo (e.g. CheckManagementNeedsGimmickTag(Grill)); 0 = always
        pub condition_info: u32,
        // internal -> FactionManagementOrderData.order_hash in the same slot (220 of 220)
        pub next_order_hash: u32,
        pub unk_u8: u8,
    }
}

py_binary_struct! {
    pub struct FactionManagementOrderData {
        // internal: target of branch_list[].next_order_hash
        pub order_hash: u32,
        pub unk_hash_list: CArray<u32>,
        // ref, checked -> npcactivityinfo (Activity_Management_Customer_* / Worker_*)
        pub npc_activity_info: u32,
        // ref, checked -> npcactivitygroupinfo (Customer_Wait, Worker_Commute, Festival_Customer)
        pub npc_activity_group_info: u32,
        pub unk_u32_w: u32,
        // ref -> stringinfo; 0xEAC5E173 = none
        pub string_info: u32,
        pub unk_u32_a: u32,
        // ref, checked -> itemuseinfo (Food_*_item_use_N); 0 = none
        pub item_use_info: u32,
        // inferred: 0, 1500, 3000 or 14400 (unit unknown)
        pub duration: u32,
        pub unk_u8_0: u8,
        pub unk_u8_1: u8,
        pub unk_u8_2: u8,
        pub action_list: CArray<FactionManagementOrderAction>,
        pub branch_list: CArray<FactionManagementOrderBranchData>,
    }
}

// ── Slots: one customer slot, then workstation (gimmick) slots ────────────────

py_binary_struct! {
    // inferred GimmickData. Order flow first, identity block (tags .. names) last.
    pub struct FactionManagementSlotData<'a> {
        pub order_list: CArray<FactionManagementOrderData>,
        // workstations: item 11 x 10
        pub unk_item_list_a: CArray<ItemCountData>,
        // workstations: item 11 x 1000
        pub unk_item_list_b: CArray<ItemCountData>,
        pub unk_u32_z: u32,
        // [0x5D7E2EBE customers | 0x3752E1A1 workstations, slot_tag]
        pub tag_list: CArray<u32>,
        // workstation gimmick names (management_soup ...); none on customer slots
        pub gimmick_name_list: CArray<CString<'a>>,
        // equals tag_list[1]; actions, levels and needs refer to it
        pub slot_tag: u32,
        pub unk_u32_0: u32,
        // values 1000056-1000080; target table not identified
        pub unk_u32_1: u32,
        pub unk_u32_2: u32,
        pub unk_u32_3: u32,
        pub unk_u32_4: u32,
        // ref -> characterinfo (customers: 8-10 per region; workers: one key)
        pub character_info_list: CArray<u32>,
        // 1 on customer slots, 0 on workstations
        pub unk_u32_a: u32,
        // customers 2 (festival 10), workstations 2-4
        pub unk_u32_b: u32,
        pub unk_hash_c: u32,
        pub unk_hash_d: u32,
        pub slot_name: LocalizableString<'a>,
        pub unk_u8_0: u8,
        pub unk_u8_1: u8,
        pub unk_u8_2: u8,
    }
}

// ── Needs, levels, scoring ────────────────────────────────────────────────────

py_binary_struct! {
    // inferred NeedsItemData (pub 65, sawmill 49, clothier 36, forge 30 per record)
    pub struct FactionManagementNeedsData {
        // signed: 0, -2, -3 or -4
        pub unk_a: i32,
        // 1-3
        pub unk_u32_b: u32,
        // 2-5
        pub unk_u32_c: u32,
        // internal -> tag_name_list[].tag_hash (3-6 category tags)
        pub tag_hash_list: CArray<u32>,
        // ref -> gimmickinfo (gimmick_management_needs_box_01)
        pub gimmick_info: u32,
        // ref -> iteminfo (the product)
        pub item_info: u32,
        pub unk_u32_x: u32,
        pub unk_u8_0: u8,
        pub unk_u8_1: u8,
        pub unk_u8_2: u8,
    }
}

py_binary_struct! {
    // inferred LevelRewardData
    pub struct FactionManagementRewardData {
        // ref, checked -> dropsetinfo (e.g. Item(Mining_Drill,1))
        pub drop_set_info: u32,
        pub unk_hash: u32,
    }
}

py_binary_struct! {
    pub struct FactionManagementSlotCountData {
        // internal -> FactionManagementSlotData.slot_tag (97 of 97)
        pub slot_tag: u32,
        pub count: u32,
    }
}

py_binary_struct! {
    // inferred LevelConditionData
    pub struct FactionManagementConditionTextData<'a> {
        // ref -> conditioninfo (e.g. GetManagementTier(<node>)>=3)
        pub condition_info: u32,
        pub text: LocalizableString<'a>,
    }
}

py_binary_struct! {
    // inferred LevelCountConditionData
    pub struct FactionManagementCountConditionData<'a> {
        // 0, 1 or 2
        pub condition_type: u8,
        // e.g. 1, 10, 50; 3, 30, 50
        pub count: u32,
        pub text: LocalizableString<'a>,
        // always 99
        pub unk_u8_99: u8,
        // always 0
        pub unk_u64_0: u64,
        // ref -> conditioninfo (e.g. FactionManagementScore(High), IsCancel(False))
        pub condition_info: u32,
    }
}

py_binary_struct! {
    // inferred LevelData: 6 per business (level 0-5), 1 per festival
    pub struct FactionManagementLevelData<'a> {
        // ref -> iteminfo
        pub unlock_item_info_list: CArray<u32>,
        pub reward_group_list: CArray<CArray<FactionManagementRewardData>>,
        // 0-5
        pub level: u32,
        // 1 on levels 0-3, 2 on levels 4-5
        pub unk_u32_z: u32,
        pub slot_count_list: CArray<FactionManagementSlotCountData>,
        // rise per level: 1.08 / 0.5 up to 1.5 / 2.0
        pub unk_rate_a: f32,
        pub unk_rate_b: f32,
        // inferred: score to reach the level; 0, 3, 14, 40, 301, 901 in every business
        pub threshold: u32,
        // 1-2
        pub unk_u32_b1: u32,
        // 1-3
        pub unk_u32_b2: u32,
        pub condition_text_list: CArray<FactionManagementConditionTextData<'a>>,
        pub count_condition_list: CArray<FactionManagementCountConditionData<'a>>,
    }
}

py_binary_struct! {
    // high, midhigh, mid, midlow, low
    pub struct FactionManagementScoreGradeData<'a> {
        // per million; the three sum to 1,000,000
        pub rate_a: u64,
        pub rate_b: u64,
        pub rate_c: u64,
        // internal: record key << 32 | hash (high word = record key, 90 of 90)
        pub text_id: u64,
        pub unk_u16: u16,
        // staticinfo-management-score-<grade>
        pub text_key: CString<'a>,
        // 1, 0, -1, -2, -FLT_MAX
        pub min_score: f32,
        pub unk_u8: u8,
    }
}

py_binary_struct! {
    // (0, 0) (1, 8) (2, 40) (3, 100) (4, 220)
    pub struct FactionManagementStepData {
        pub index: u32,
        pub value: u64,
    }
}

py_binary_struct! {
    pub struct FactionManagementGimmickTagData {
        // ref -> gimmickinfo
        pub gimmick_info: u32,
        // internal -> tag_name_list[].tag_hash
        pub tag_hash_list: CArray<u32>,
    }
}

py_binary_struct! {
    pub struct FactionManagementTagNameData {
        pub tag_hash: u32,
        // localstringinfo hash
        pub name_hash: u32,
    }
}

py_binary_struct! {
    pub struct FactionManagementBuffRewardData {
        // ref, checked -> managementbuffinfo (ManagementBuff_<type>_<place>)
        pub management_buff_info: u16,
        pub unk_u8: u8,
        // item 11 x 15,000
        pub item: ItemCountData,
    }
}

pabgh_typed_blob_table! {
    pub struct FactionManagementInfo<'a> {
        // Management_Node_<place>_<nn>
        pub string_key: CString<'a>,
        pub key: u16,
        pub is_blocked: u8,
        // businesses 1, festivals 0
        pub unk_u8_a: u8,
        // businesses 0, festivals 1
        pub unk_u8_b: u8,
        // businesses 1, festivals 0
        pub unk_u8_c: u8,
        // ref, checked -> houseinfo (ManagementHouse_<same place>_01); festivals 0
        pub house_info: u16,
        // ref, checked -> gimmickinfo (management_node_<place>_0001)
        pub gimmick_info: u32,
        // one value per business type (a localstringinfo hash); festivals 0
        pub business_type_hash: u32,
        pub unk_u32_0: u32,
        // businesses: unk_time_b = 1.5 x unk_time_a; festivals 7200 / 7200
        pub unk_time_a: u32,
        pub unk_time_b: u32,
        // inferred: purchase price (item 11 x 20,000)
        pub cost_list: CArray<ItemCountData>,
        pub unk_u32_1: u32,
        // ref, checked -> gameplaytrigger (GamePlayTrigger_Node_<place>; festivals CampParty_<camp>)
        pub game_play_trigger_info_list: CArray<u32>,
        // ref, checked -> gameadviceinfo (Advice_Management_*)
        pub game_advice_info_list: CArray<u32>,
        pub override_spawn_list: CArray<FactionManagementOverrideSpawnData<'a>>,
        // always 1
        pub customer_slot_list: CArray<FactionManagementSlotData<'a>>,
        // 2 (festivals 1)
        pub gimmick_slot_list: CArray<FactionManagementSlotData<'a>>,
        pub needs_list: CArray<FactionManagementNeedsData>,
        pub level_list: CArray<FactionManagementLevelData<'a>>,
        pub score_grade_list: CArray<FactionManagementScoreGradeData<'a>>,
        pub step_list: CArray<FactionManagementStepData>,
        pub gimmick_tag_list: CArray<FactionManagementGimmickTagData>,
        pub tag_name_list: CArray<FactionManagementTagNameData>,
        pub unk_text_a: LocalizableString<'a>,
        pub unk_text_b: LocalizableString<'a>,
        pub unk_u32_r0: u32,
        pub unk_u32_r1: u32,
        pub buff_reward_list: CArray<FactionManagementBuffRewardData>,
        // Management_Node_<place>_0001_Phase01_00 or GameLevel_...
        pub level_name_list: CArray<CString<'a>>,
        // Management_<place>; empty on festivals
        pub place_name: CString<'a>,
    }
    tail: tail_blob;
}

#[cfg(test)]
mod tests {
    use super::*;

    // (body, header); the table ships in dlc_01 only, a missing fixture is skipped.
    const FIXTURES: &[(&str, &str)] = &[
        ("factionmanagement.pabgb", "factionmanagement.pabgh"),
        ("factionmanagement.dlc_01.pabgb", "factionmanagement.dlc_01.pabgh"),
    ];

    // (key, start, size) per record in body order, from the 6-byte pabgh entries.
    fn record_bounds(head: &[u8], body_len: usize) -> Vec<(u16, usize, usize)> {
        let n = u16::from_le_bytes([head[0], head[1]]) as usize;
        assert_eq!(head.len(), 2 + 6 * n, "unexpected pabgh size");
        let mut ents: Vec<(u16, usize)> = (0..n)
            .map(|i| {
                let p = 2 + 6 * i;
                (u16::from_le_bytes([head[p], head[p + 1]]),
                 u32::from_le_bytes([head[p + 2], head[p + 3], head[p + 4], head[p + 5]]) as usize)
            })
            .collect();
        ents.sort_by_key(|e| e.1);
        (0..n)
            .map(|i| {
                let end = if i + 1 < n { ents[i + 1].1 } else { body_len };
                (ents[i].0, ents[i].1, end - ents[i].1)
            })
            .collect()
    }

    fn load(body: &str, head: &str) -> Option<(Vec<u8>, Vec<u8>)> {
        let b = std::fs::read(crate::testenv::resolve(body)).ok()?;
        let h = std::fs::read(crate::testenv::resolve(head)).ok()?;
        Some((b, h))
    }

    #[test]
    fn roundtrip() {
        for (body, head) in FIXTURES {
            let Some((data, h)) = load(body, head) else {
                eprintln!("SKIP: fixture {body} / {head} not found");
                continue;
            };
            let mut out = Vec::with_capacity(data.len());
            let (mut n, mut tails) = (0, 0);
            for (key, start, size) in record_bounds(&h, data.len()) {
                let mut offset = start;
                let rec = FactionManagementInfo::read_with_size(&data, &mut offset, size)
                    .unwrap_or_else(|e| panic!("{body} key={key}: {e}"));
                assert_eq!(rec.key, key, "{body}: header key != record key");
                // Observed on 2.04.03: one trailing zero byte exactly on the festivals.
                assert_eq!(rec.tail_blob.is_empty(), rec.unk_u8_b == 0, "{body} key={key}: tail rule");
                assert!(rec.tail_blob.iter().all(|&b| b == 0) && rec.tail_blob.len() <= 1,
                    "{body} key={key}: unexpected tail {:?}", rec.tail_blob);
                tails += usize::from(!rec.tail_blob.is_empty());
                rec.write_to(&mut out).unwrap();
                n += 1;
            }
            assert_eq!(out, data, "{body}: roundtrip bytes mismatch");
            eprintln!("OK: {body} {n} records, {tails} with a 1-byte tail");
        }
    }

    #[test]
    fn json_roundtrip() {
        for (body, head) in FIXTURES {
            let Some((data, h)) = load(body, head) else {
                eprintln!("SKIP: fixture {body} / {head} not found");
                continue;
            };
            for (key, start, size) in record_bounds(&h, data.len()) {
                let mut offset = start;
                let rec = FactionManagementInfo::read_with_size(&data, &mut offset, size).unwrap();
                let dict = rec.to_json_dict();
                let mut from_json = Vec::new();
                FactionManagementInfo::write_from_json_dict(&mut from_json, &dict)
                    .unwrap_or_else(|e| panic!("{body} key={key}: write_from_json_dict: {e}"));
                assert_eq!(from_json, &data[start..start + size],
                    "{body} key={key}: JSON round-trip diverges from the original bytes");
            }
        }
    }

    // needs_list[].unk_a is signed on the wire: JSON must show -2, never 4294967294,
    // and a negative value written from JSON must land as the same bytes.
    #[test]
    fn needs_unk_a_is_signed() {
        for (body, head) in FIXTURES {
            let Some((data, h)) = load(body, head) else {
                eprintln!("SKIP: fixture {body} / {head} not found");
                continue;
            };
            let mut negatives = 0;
            for (key, start, size) in record_bounds(&h, data.len()) {
                let mut offset = start;
                let rec = FactionManagementInfo::read_with_size(&data, &mut offset, size).unwrap();
                let dict = rec.to_json_dict();
                for need in dict["needs_list"].as_array().unwrap() {
                    let a = &need["unk_a"];
                    assert!(a.as_i64().is_some_and(|v| (i32::MIN as i64..=0).contains(&v)),
                        "{body} key={key}: unk_a {a} is not a small signed value");
                    negatives += usize::from(a.as_i64().unwrap() < 0);
                }
            }
            if negatives > 0 {
                let mut w = Vec::new();
                <i32 as crate::json_traits::WriteJsonValue>::write_from_json(&mut w, &serde_json::json!(-3)).unwrap();
                assert_eq!(w, (-3i32).to_le_bytes());
            }
            eprintln!("OK: {body} {negatives} negative needs unk_a");
        }
    }
}
