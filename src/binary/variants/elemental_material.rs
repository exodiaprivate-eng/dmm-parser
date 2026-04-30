//! Typed structures for the ElementalMaterialInfo tail.
//!
//! Reader chain: sub_1410DC8F0 (top) calls sub_1411166F0, sub_141102B30,
//! sub_1411168A0, sub_1410DC480, sub_1410DC310, sub_1410DC7F0, etc.
//! See `tables/elemental_material_info/info.rs` for the per-byte wire
//! survey.

use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use crate::py_binary_struct;
use serde_json::{Map, Value};
use std::io::{self, Write};

py_binary_struct! {
    /// sub_1410DC310 element. Wire: 4× u32 hash (sub_1410FEBE0) +
    /// 5× u32 raw = 36 wire bytes. Memory layout matches.
    pub struct MaterialInner36 {
        pub hash_a: u32,
        pub raw_a: u32,
        pub hash_b: u32,
        pub raw_b: u32,
        pub hash_c: u32,
        pub raw_c: u32,
        pub hash_d: u32,
        pub raw_d: u32,
        pub raw_e: u32,
    }
}

py_binary_struct! {
    /// sub_1410DC480 152-byte composite element.
    /// Wire: 4 + 8 + 8 + 4 + CString + 1 + 5×CString + 2×MaterialInner36
    ///       + 4 + 4 + 4× u8.
    /// The 5 sub_1410A9D40 fields wire-read as CString (u32 length +
    /// bytes) but the runtime hashes them to u32; we preserve as
    /// CString for byte-perfect round-trip.
    pub struct MaterialInnerItem<'a> {
        pub hash_da10: u32,
        pub raw_u64_a: u64,
        pub raw_u64_b: u64,
        pub hash_da30: u32,
        pub label: CString<'a>,
        pub flag: u8,
        pub string_hash_a: CString<'a>,
        pub string_hash_b: CString<'a>,
        pub string_hash_c: CString<'a>,
        pub string_hash_d: CString<'a>,
        pub string_hash_e: CString<'a>,
        pub block_a: MaterialInner36,
        pub block_b: MaterialInner36,
        pub raw_u32_a: u32,
        pub raw_u32_b: u32,
        pub flag_a: u8,
        pub flag_b: u8,
        pub flag_c: u8,
        pub flag_d: u8,
    }
}

py_binary_struct! {
    /// sub_1411166F0 element. Per IDA: u32 raw + sub_1411168A0 result
    /// (CArray of MaterialInnerItem).
    pub struct MaterialOuterEntry<'a> {
        pub raw: u32,
        pub items: CArray<MaterialInnerItem<'a>>,
    }
}

py_binary_struct! {
    /// sub_141102B30 element. Wire: u32 hash + u64 raw = 12 wire bytes.
    pub struct MaterialHashU64 {
        pub hash: u32,
        pub raw: u64,
    }
}

py_binary_struct! {
    /// CArray-of-pairs at mem +112 in elemental_material reader.
    /// Wire per element: u32 + u32 = 8 wire bytes.
    pub struct MaterialU32Pair {
        pub a: u32,
        pub b: u32,
    }
}

py_binary_struct! {
    /// sub_1410DC7F0 element. Wire: u32 + u32 + u32 + u8 = 13 wire bytes.
    pub struct MaterialStateData {
        pub raw_a: u32,
        pub raw_b: u32,
        pub raw_c: u32,
        pub flag: u8,
    }
}
