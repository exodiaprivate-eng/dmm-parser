//! Typed structures for the SpecialModeInfo tail.
//!
//! Reader chain: sub_1410F60E0 (top) calls sub_141128AF0 (CArray reader)
//! 24 times. sub_141128AF0 → CArray<sub_1410F5A30_element> (176-byte mem
//! item). Plus trailing sub_1410F5F80 and sub_1410D4540.
//!
//! Per IDA decompile this iteration.

use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use crate::py_binary_struct;
use serde_json::{Map, Value};
use std::io::{self, Write};

py_binary_struct! {
    /// sub_14110A460 element. Wire: CString (sub_1410A9D40 hash-string) +
    /// CString + u8.
    pub struct SpecialModeNamedFlag<'a> {
        pub name_hash: CString<'a>,
        pub label: CString<'a>,
        pub flag: u8,
    }
}

py_binary_struct! {
    /// sub_1410F5F80 (DetectModeAreaData, 64 mem bytes). Wire:
    /// u8 + CString + CString + 12 + 12 + u32 + u32 + u8.
    pub struct DetectModeAreaData<'a> {
        pub flag_a: u8,
        pub name_a: CString<'a>,
        pub name_b: CString<'a>,
        pub block_a: [u8; 12],
        pub block_b: [u8; 12],
        pub raw_a: u32,
        pub raw_b: u32,
        pub flag_b: u8,
    }
}

py_binary_struct! {
    /// sub_14B92C740 (PlayerActionLimitDesc). Wire: 6× u8 +
    /// CArray<u16> + CArray<u16>.
    pub struct PlayerActionLimitDesc {
        pub flag_a: u8,
        pub flag_b: u8,
        pub flag_c: u8,
        pub flag_d: u8,
        pub flag_e: u8,
        pub flag_f: u8,
        pub allow_list: CArray<u16>,
        pub deny_list: CArray<u16>,
    }
}

py_binary_struct! {
    /// sub_1410F5A30 element (176 mem bytes). Wire layout per IDA:
    pub struct SpecialModeInner<'a> {
        pub flag_a: u8,                 // +0 (1 wire byte)
        pub hash_a: u32,                // +2 (4 wire bytes via sub_1410FF430)
        pub hash_b: u32,                // +4 (4 wire bytes via sub_1410FF430)
        pub string_hash_a: CString<'a>, // +8 (sub_1410A9D40, CString shape)
        pub raw_a: u32,                 // +12 (4 wire bytes)
        pub string_hash_b: CString<'a>, // +16 (sub_1410A9D40)
        pub raw_b: u32,                 // +20 (4 wire bytes)
        pub raw_c: u32,                 // +24 (4 wire bytes)
        pub raw_d: u32,                 // +28
        pub raw_e: u32,                 // +32
        pub raw_u64_a: u64,             // +40
        pub raw_u64_b: u64,             // +48
        pub hash_c: u32,                // +56 (4 wire bytes via sub_1410FF5C0)
        pub flag_b: u8,                 // +58
        pub hash_da78_list: CArray<u32>, // +64 (sub_141100B10)
        pub named_flags: CArray<SpecialModeNamedFlag<'a>>, // +80 (sub_14110A460)
        pub hash_113b8: u32,            // +96 (sub_141106210)
        pub label: CString<'a>,         // +104
        pub flag_c: u8,                 // +112
        pub flag_d: u8,                 // +113
        pub flag_e: u8,                 // +114
        pub raw_f: u32,                 // +116
        pub raw_g: u32,                 // +120
        pub raw_h: u32,                 // +124
        pub flag_f: u8,                 // +128
        pub raw_i: u32,                 // +132
        pub raw_j: u32,                 // +136
        pub raw_k: u32,                 // +140
        pub raw_l: u32,                 // +144
        pub raw_m: u32,                 // +148
        pub raw_n: u32,                 // +152
        pub trailing: [u8; 16],         // +156..172 (16 raw bytes)
    }
}
