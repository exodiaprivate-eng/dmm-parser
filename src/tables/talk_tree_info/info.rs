//! Parser for `TalkTreeInfo.pabgb` (new in 1.12). Game deser sub_101F89958.
//! TalkTreeKey wire=2B (u16, sub_10161FB84). Condition/Interaction/DropSet keys
//! wire=4B. button_text=LocalizableString (sub_100D5D6D8). result_data_list elem
//! = TalkTreeResultData (sub_101F89760): dropSetInfo(DropSetKey u32)+condition(ConditionKey u32).
use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct TalkTreeResultData {
        pub drop_set_info: u32,
        pub condition: u32,
    }
}

py_binary_struct! {
    pub struct TalkTreeInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub root_talk_tree_info: u16,
        pub parent_talk_tree_info: u16,
        pub child_talk_tree_info_list: CArray<u16>,
        pub condition_info: u32,
        pub target_condition_info: u32,
        pub button_text: LocalizableString<'a>,
        pub interaction_info: u32,
        pub result_data_list: CArray<TalkTreeResultData>,
    }
}
