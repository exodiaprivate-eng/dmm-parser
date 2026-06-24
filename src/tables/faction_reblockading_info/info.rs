//! Parser for `FactionReblockadingInfo.pabgb` (new in 1.12). Game deser
//! sub_101F9D2B4; quest-data element sub_101F9D074; node list sub_1017AD748.
use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    // ReoccupationQuestData (sub_101F9D074): questInfo(QuestKey u32) +
    // playerCondition(ConditionKey u32) + rate(f64) + closeTime(u32) = 20B.
    pub struct ReoccupationQuestData {
        pub quest_info: u32,
        pub player_condition: u32,
        pub rate: f64,
        pub close_time: u32,
    }
}

py_binary_struct! {
    pub struct FactionReblockadingInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub reoccupation_quest_data_list: CArray<ReoccupationQuestData>,
        pub faction_node_list: CArray<u32>,
        pub delay_time: u32,
        pub protect_combat_power: f32,
    }
}
