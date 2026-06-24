//! Parser for `ZoneInfo.pabgb` (new in 1.12). Layout from game deser
//! sub_101F8A558: key(u16) + stringKey(CString) + isBlocked(u8) + prefabPath(CString).
use crate::binary::*;
use crate::py_binary_struct;

py_binary_struct! {
    pub struct ZoneInfo<'a> {
        pub key: u16,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub prefab_path: CString<'a>,
    }
}
