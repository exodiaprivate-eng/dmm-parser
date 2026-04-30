//! Tier 1.5 — typed prefix + Decoded|Raw fallback tail.
//!
//! Reader: `sub_1410E6FC0` in CrimsonDesert.exe (Win build). Massive
//! 7205-byte function, 100+ wire reads in the body. Fields 1-16 are
//! typed (joined with the prefix when the tail decodes successfully);
//! the 99.93% of vanilla entries that decode cleanly carry the rest as
//! `post_blob`. Field 17 (sub_1411125E0 → sub_141D7FF30 →
//! sub_141D80A90) is the next blocker: sub_141D80A90 is the
//! TriggerGamePlayEventHandlerData polymorphic ReflectObject dispatcher
//! (see STATUS.md "Deferred — ReflectObject reflection layer").
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1. u32 key                       (_key, mem a2+8)
//!   2. CString string_key            (_stringKey, mem a2+16)
//!   3. u8 is_blocked                 (_isBlocked, mem a2+24)
//!   4. CString prefab_path           (_prefabPath, mem a2+32)
//!   5. u32 gimmick_group_info        (_gimmickGroupInfo, sub_141104AE0
//!      → qword_145F11D70 lookup, mem a2+40)
//!   6. u16 breakable_object_info     (_breakableObjectInfo, inline u16 →
//!      qword_145F15960 lookup, wire u16, mem a2+42)
//!      ← TAIL STARTS HERE
//!   7. _gimmickInteractionOverrideDataList (sub_141118470 →
//!      CArray<COptional<144-byte item via sub_1410DF770>>; inner has 15
//!      wire reads incl. LocalizableString, CArray<{CStr hash + u32}>,
//!      sub_141100E90 CArray<32-byte item>, sub_141101AB0/sub_141103C30
//!      lookups, sub_141114FC0/sub_141E2C900 unknown helpers, mem a2+48)
//!   8. u8 _useInteractionUISocket    (mem a2+64)
//!   9. u8 _useSubPartForInteraction  (mem a2+65)
//!  10. _propertyList                 (sub_141101AB0, 16-byte CArray
//!      header at mem a2+72)
//!  11. u32 _gimmickNameHash          (mem a2+88)
//!  12. LocalizableString _gimmickName (mem a2+96)
//!  13. CString _emojiTextureID       (mem a2+128)
//!  14. CString _devMemo              (mem a2+136)
//!  15. sub_141104D20 (16 mem bytes)  (mem a2+144)
//!  16. sub_141102990 (16 mem bytes)  (mem a2+160)
//!  17. sub_1411125E0 (16 mem bytes)  (mem a2+176)
//!  18. _gimmickChartParameterList    (CArray of 16-byte items via
//!      sub_141C7F8B0; per element u32 + u8 + u32 + u8, mem a2+192)
//!  19. … 80+ more wire reads.
//!
//! Steps 1-16 are typed (joined with the prefix when Decoded). Field
//! 17 (sub_1411125E0) blocks further extension — see header note.
//!
//! ## GimmickInteractionOverrideData wire layout (sub_1410DF770)
//!
//! 144 mem bytes per element, 15 wire fields. Decompiled from Win-IDA
//! this session.
//!
//!   1. sub_1411026F0 — u16 lookup                    (mem +0)
//!   2. read_LocalizableString                        (mem +8, 32 b)
//!   3. u32 raw                                       (mem +40)
//!   4. CArray<{u32 hash + u32 raw}> (8-byte stride)  (mem +48, 16 b)
//!      — outer reads u32 count, then each element: sub_1410A9D40
//!      (CString-hash → u32) + u32 raw.
//!   5. sub_141114FC0 — CArray of 48-byte items via   (mem +64, 16 b)
//!      sub_1410DF4C0; per-element wire: u32 + CString-hash +
//!      CString + u32 + Vec3 + 3× u32. (Verified Win-IDA, 7 wire
//!      reads / 48 mem bytes.)
//!   6. sub_141E2C900 — `CArray<ConditionPair>` via   (mem +80, 16 b)
//!      `BareConditionPairCArray`. NO per-element COptional —
//!      bare ConditionPair stride. ← stream-mode GameCondition
//!      blocker starts here.
//!   7. sub_141100E90 — CArray of 32-byte items       (mem +96, 16 b)
//!      (28 wire bytes per element: f32 + 3× 8-byte clusters).
//!   8. sub_141101AB0 — `CArray<u32>`                 (mem +112, 16 b)
//!   9. sub_141103C30 — u32 lookup                    (mem +128)
//!  10. sub_141100370 — u16 lookup                    (mem +132)
//!  11. u8 flag                                       (mem +134)
//!  12. u8 flag                                       (mem +135)
//!  13. u8 flag                                       (mem +136)
//!  14. u8 flag                                       (mem +137)
//!  15. u8 flag                                       (mem +138)
//!
//! Outer wrapper (sub_141118470): `CArray<COptional<...>>` — u32
//! count + per-element u8 presence + (if present) heap-allocated
//! 144-byte GimmickInteractionOverrideData populated by
//! sub_1410DF770.

use crate::binary::*;
use crate::binary::variants::gimmick_interaction_override::GimmickInteractionOverrideCArray;
use crate::binary::variants::trigger_gameplay_event_handler_data::OptionalTriggerGamePlayEventHandlerData;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use crate::py_binary_struct;
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde_json::{Map, Value};
use std::io::{self, Write};

py_binary_struct! {
    /// `sub_141104D20` per-element. 8-byte mem stride; wire = 2× CString
    /// (each consumed via sub_1410A9D40 → u32 hash, packed into a qword).
    pub struct GimmickHashPair<'a> {
        pub hash_a: CString<'a>,
        pub hash_b: CString<'a>,
    }
}

py_binary_struct! {
    /// `sub_1410A9D40` wrapper. 4-byte mem stride; wire = CString.
    pub struct GimmickHashSingle<'a> {
        pub hash: CString<'a>,
    }
}

py_binary_struct! {
    /// Field 18 — `_gimmickChartParameterList` per-element.
    /// Win-IDA `sub_141C7F8B0`: 16-byte mem stride; wire = u32 + u8 + u32 + u8
    /// = 10 wire bytes per element. CArray<GimmickChartParameter>.
    pub struct GimmickChartParameter {
        pub a: u32,
        pub b: u8,
        pub c: u32,
        pub d: u8,
    }
}

/// Tail of GimmickInfo. When the field-7 CArray decode succeeds (and
/// the immediately-following stable scalar block parses cleanly) it
/// joins the typed prefix; the rest of the body (~85 fields) still
/// rides as `post_blob`. On any decode failure the entire post-prefix
/// region is captured as `Raw`.
#[derive(Debug)]
pub enum GimmickTail<'a> {
    Decoded {
        gimmick_interaction_override_list: GimmickInteractionOverrideCArray<'a>,
        use_interaction_ui_socket: u8,
        use_sub_part_for_interaction: u8,
        property_list: CArray<u32>,
        gimmick_name_hash: u32,
        gimmick_name: LocalizableString<'a>,
        emoji_texture_id: CString<'a>,
        dev_memo: CString<'a>,
        hash_pair_list: CArray<GimmickHashPair<'a>>,    // sub_141104D20
        hash_single_list: CArray<GimmickHashSingle<'a>>, // sub_141102990
        /// sub_1411125E0 — `CArray<COptional<TriggerGamePlayEventHandlerData>>`.
        /// Decoded when the typed reader cleanly consumes the bytes; falls
        /// back to leaving these bytes inside `post_blob` if any TGPEHD
        /// variant decode under/over-reads.
        trigger_event_handler_list: Option<CArray<OptionalTriggerGamePlayEventHandlerData<'a>>>,
        /// Field 18 — `_gimmickChartParameterList` (sub_141C7F8B0). 10 wire
        /// bytes per element (u32+u8+u32+u8). Best-effort typed; bytes
        /// remain in post_blob if decode under/over-reads.
        gimmick_chart_parameter_list: Option<CArray<GimmickChartParameter>>,
        /// Field 19 — empirically `CArray<u32>` (count=0 in 99.4% of vanilla
        /// entries; first u32 of post_blob after field 18 is 0x00000000).
        /// Only attempted if field 18 succeeded.
        field_19_u32_list: Option<CArray<u32>>,
        /// Field 20 — empirically `CArray<u32>` (mostly empty, but some
        /// entries have count=1 with item=0).
        field_20_u32_list: Option<CArray<u32>>,
        /// Field 21 — empirically `CArray<u32>` continuation. 98.8% empty.
        field_21_u32_list: Option<CArray<u32>>,
        /// Field 22 — empirically `CArray<u32>` continuation.
        field_22_u32_list: Option<CArray<u32>>,
        /// Field 23 — empirically `CArray<u32>` continuation; some
        /// entries have non-zero (possibly float-bit) values that may
        /// indicate this is actually a different type — defensive parse
        /// keeps it Option.
        field_23_u32_list: Option<CArray<u32>>,
        /// Field 24 — empirically `CArray<u32>` continuation.
        field_24_u32_list: Option<CArray<u32>>,
        /// Field 25 — empirically `CArray<u32>` continuation.
        field_25_u32_list: Option<CArray<u32>>,
        /// Field 26 — single u32 (probably a hash/key, NOT a CArray).
        /// Adding as CArray<u32> dropped typed-entry count 7318→121,
        /// confirming this is a different type.
        field_26_u32: Option<u32>,
        /// Field 27 — empirically `CArray<u32>` (most entries count=0,
        /// 106 entries count=1).
        field_27_u32_list: Option<CArray<u32>>,
        /// Field 28 — single u32 hash/key (7176 entries share value
        /// 0x150B14D0; clearly not a CArray count).
        field_28_u32: Option<u32>,
        /// Field 29 — empirically `CArray<u32>` continuation.
        field_29_u32_list: Option<CArray<u32>>,
        /// Field 30 — empirically `CArray<u32>` continuation.
        field_30_u32_list: Option<CArray<u32>>,
        /// Field 31 — empirically `CArray<u32>` continuation.
        field_31_u32_list: Option<CArray<u32>>,
        /// Field 32 — empirically `CArray<u32>` continuation.
        field_32_u32_list: Option<CArray<u32>>,
        /// Field 33 — single u32 hash (6492 entries share 0x6c000000).
        field_33_u32: Option<u32>,
        /// Field 34 — single u32 hash (6102 entries share 0x00BCDE86 —
        /// likely a default reference shared across gimmicks).
        field_34_u32: Option<u32>,
        /// Field 35 — empirically `CArray<u32>` (6406/6411 have count=0).
        field_35_u32_list: Option<CArray<u32>>,
        /// Field 36 — single u32, flag-packed (6242 entries share
        /// `0x0001FF00`; pattern `0x00FF##00`).
        field_36_u32: Option<u32>,
        /// Field 37 — single u32 hash/value (6228 entries share `0xC39F0000`).
        field_37_u32: Option<u32>,
        /// Field 38 — single u32 (continuation hash).
        field_38_u32: Option<u32>,
        /// Field 39 — empirically `CArray<u32>` (6245 entries have count=0).
        field_39_u32_list: Option<CArray<u32>>,
        /// Field 40 — empirically `CArray<u32>` continuation.
        field_40_u32_list: Option<CArray<u32>>,
        /// Field 41 — single u32 (6242 entries share `0x00008000` = flag bit 1<<15).
        field_41_u32: Option<u32>,
        /// Field 42 — single u32 continuation.
        field_42_u32: Option<u32>,
        /// Field 43 — single u32 (6011 entries share `0xff008000`).
        field_43_u32: Option<u32>,
        /// Field 44 — single u32 (6011 entries share `0xffffffff` = sentinel).
        field_44_u32: Option<u32>,
        /// Field 45 — single u32 (6011 entries share `0xffffffff` = sentinel).
        field_45_u32: Option<u32>,
        /// Field 46 — single u32 (5666 entries share `0x00ffffff` flag pattern).
        field_46_u32: Option<u32>,
        /// Field 47 — single u32 (5746 entries share `0x00bf8000`).
        field_47_u32: Option<u32>,
        /// Field 48 — single u32 (6099 entries share `0x00bf8000` — same as 47).
        field_48_u32: Option<u32>,
        /// Field 49 — empirically `CArray<u32>` (mostly count=0).
        field_49_u32_list: Option<CArray<u32>>,
        /// Field 50 — empirically `CArray<u32>` (mostly count=0).
        field_50_u32_list: Option<CArray<u32>>,
        /// Field 51 — empirically `CArray<u32>` (mostly count=0).
        field_51_u32_list: Option<CArray<u32>>,
        /// Field 52 — empirically `CArray<u32>` (mostly count=0).
        field_52_u32_list: Option<CArray<u32>>,
        /// Field 53 — empirically `CArray<u32>` (mostly count=0).
        field_53_u32_list: Option<CArray<u32>>,
        /// Field 54 — empirically `CArray<u32>` (mostly count=0).
        field_54_u32_list: Option<CArray<u32>>,
        /// Field 55 — empirically `CArray<u32>` (mostly count=0).
        field_55_u32_list: Option<CArray<u32>>,
        /// Field 56 — empirically `CArray<u32>` (mostly count=0).
        field_56_u32_list: Option<CArray<u32>>,
        /// Field 57 — empirically `CArray<u32>` (mostly count=0).
        field_57_u32_list: Option<CArray<u32>>,
        /// Field 58 — empirically `CArray<u32>` (mostly count=0).
        field_58_u32_list: Option<CArray<u32>>,
        /// Field 59 — u32 storing f32 0.5 (`0x3f000000`) for 6018 entries.
        field_59_u32: Option<u32>,
        /// Field 60 — u32 storing f32 0.1 (`0x3dcccccd`).
        field_60_u32: Option<u32>,
        /// Field 61 — u32 storing f32 0.1 (`0x3dcccccd`).
        field_61_u32: Option<u32>,
        /// Field 62 — u32 (`0x00000101` = 257 flag-packed).
        field_62_u32: Option<u32>,
        /// Field 63 — u32 (= 0).
        field_63_u32: Option<u32>,
        /// Field 64 — u32 (= 0).
        field_64_u32: Option<u32>,
        /// Field 65 — u32 storing f32 5.0 (`0x40a00000`).
        field_65_u32: Option<u32>,
        /// Field 66 — u32 storing f32 1.0 (`0x3f800000`).
        field_66_u32: Option<u32>,
        /// Field 67 — u32 (`0x02000000` flag for 6019 entries).
        field_67_u32: Option<u32>,
        /// Field 68 — u32 (often `0x00000004`).
        field_68_u32: Option<u32>,
        /// Field 69 — u32.
        field_69_u32: Option<u32>,
        /// Field 70 — u32.
        field_70_u32: Option<u32>,
        /// Field 71 — u32.
        field_71_u32: Option<u32>,
        /// Field 72 — u32 (`0x00000100` = 256).
        field_72_u32: Option<u32>,
        /// Field 73 — u32.
        field_73_u32: Option<u32>,
        /// Field 74 — u32.
        field_74_u32: Option<u32>,
        /// Field 75 — u32 (mostly 0).
        field_75_u32: Option<u32>,
        /// Field 76 — u32.
        field_76_u32: Option<u32>,
        /// Field 77 — u32.
        field_77_u32: Option<u32>,
        /// Field 78 — u32.
        field_78_u32: Option<u32>,
        /// Field 79 — u32.
        field_79_u32: Option<u32>,
        /// Field 80 — u32.
        field_80_u32: Option<u32>,
        /// Field 81 — u32 (often `0x00003f80`).
        field_81_u32: Option<u32>,
        /// Field 82 — u32.
        field_82_u32: Option<u32>,
        /// Field 83 — u32.
        field_83_u32: Option<u32>,
        /// Field 84 — u32.
        field_84_u32: Option<u32>,
        /// Field 85 — u32 (often `0xffffffff`).
        field_85_u32: Option<u32>,
        /// Field 86 — u32 (often `0xffffffff`).
        field_86_u32: Option<u32>,
        /// Field 87 — u32 (often `0xffffffff`).
        field_87_u32: Option<u32>,
        /// Field 88 — u32.
        field_88_u32: Option<u32>,
        /// Field 89 — u32.
        field_89_u32: Option<u32>,
        /// Field 90 — u32.
        field_90_u32: Option<u32>,
        /// Field 91 — u32 (4498 entries are 0; rest vary widely. Could be
        /// length-prefix of a CString but many entries fail UTF-8 — keep
        /// as u32 for byte-perfect roundtrip).
        field_91_u32: Option<u32>,
        /// Field 92 — u32.
        field_92_u32: Option<u32>,
        /// Field 93 — u32.
        field_93_u32: Option<u32>,
        /// Field 94 — u32.
        field_94_u32: Option<u32>,
        /// Field 95 — u32.
        field_95_u32: Option<u32>,
        /// Field 96 — u32.
        field_96_u32: Option<u32>,
        /// Field 97 — u32.
        field_97_u32: Option<u32>,
        /// Field 98 — u32.
        field_98_u32: Option<u32>,
        /// Field 99 — u32 (often `0x80000002` flag).
        field_99_u32: Option<u32>,
        /// Field 100 — u32 (`0x0000003f` = 63 for 4361 entries).
        field_100_u32: Option<u32>,
        /// Field 101 — u32.
        field_101_u32: Option<u32>,
        /// Field 102 — u32.
        field_102_u32: Option<u32>,
        /// Field 103 — u32.
        field_103_u32: Option<u32>,
        /// Field 104 — u32.
        field_104_u32: Option<u32>,
        /// Field 105 — u32.
        field_105_u32: Option<u32>,
        /// Field 106 — u32.
        field_106_u32: Option<u32>,
        /// Field 107 — u32.
        field_107_u32: Option<u32>,
        post_blob: Vec<u8>,
    },
    Raw(Vec<u8>),
}

impl<'a> GimmickTail<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_end: usize) -> io::Result<Self> {
        let tail_start = *offset;
        let mut probe = tail_start;
        let try_decode = (|| -> io::Result<_> {
            let list = GimmickInteractionOverrideCArray::read_from(data, &mut probe)?;
            if probe > entry_end { return Err(io::Error::new(io::ErrorKind::InvalidData, "overrun")); }
            let use_interaction_ui_socket = u8::read_from(data, &mut probe)?;
            let use_sub_part_for_interaction = u8::read_from(data, &mut probe)?;
            let property_list = <CArray<u32>>::read_from(data, &mut probe)?;
            let gimmick_name_hash = u32::read_from(data, &mut probe)?;
            let gimmick_name = LocalizableString::read_from(data, &mut probe)?;
            let emoji_texture_id = CString::read_from(data, &mut probe)?;
            let dev_memo = CString::read_from(data, &mut probe)?;
            let hash_pair_list = <CArray<GimmickHashPair>>::read_from(data, &mut probe)?;
            let hash_single_list = <CArray<GimmickHashSingle>>::read_from(data, &mut probe)?;
            if probe > entry_end { return Err(io::Error::new(io::ErrorKind::InvalidData, "overrun")); }
            Ok((list, use_interaction_ui_socket, use_sub_part_for_interaction,
                property_list, gimmick_name_hash, gimmick_name, emoji_texture_id, dev_memo,
                hash_pair_list, hash_single_list))
        })();
        match try_decode {
            Ok((list, ui, sp, pl, gnh, gn, eti, dm, hpl, hsl)) => {
                // Try to type field 17 (CArray<COptional<TGPEHD>>); fall back
                // to leaving it in post_blob if any sub-decode misaligns.
                let pre_tgpehd = probe;
                let trigger_event_handler_list = match <CArray<OptionalTriggerGamePlayEventHandlerData>>::read_from(data, &mut probe) {
                    Ok(arr) if probe <= entry_end => Some(arr),
                    _ => { probe = pre_tgpehd; None }
                };
                // Field 18: gimmick_chart_parameter_list — only attempted
                // if the TGPEHD list parsed cleanly (probe is aligned).
                let gimmick_chart_parameter_list = if trigger_event_handler_list.is_some() {
                    let pre_chart = probe;
                    match <CArray<GimmickChartParameter>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_chart; None }
                    }
                } else {
                    None
                };
                // Field 19: empirically CArray<u32>, mostly empty.
                let field_19_u32_list = if gimmick_chart_parameter_list.is_some() {
                    let pre_19 = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_19; None }
                    }
                } else {
                    None
                };
                // Field 20: empirically CArray<u32>, mostly empty.
                let field_20_u32_list = if field_19_u32_list.is_some() {
                    let pre_20 = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_20; None }
                    }
                } else {
                    None
                };
                let field_21_u32_list = if field_20_u32_list.is_some() {
                    let pre_21 = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_21; None }
                    }
                } else { None };
                let field_22_u32_list = if field_21_u32_list.is_some() {
                    let pre_22 = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_22; None }
                    }
                } else { None };
                let field_23_u32_list = if field_22_u32_list.is_some() {
                    let pre_23 = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_23; None }
                    }
                } else { None };
                let field_24_u32_list = if field_23_u32_list.is_some() {
                    let pre_24 = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_24; None }
                    }
                } else { None };
                let field_25_u32_list = if field_24_u32_list.is_some() {
                    let pre_25 = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_25; None }
                    }
                } else { None };
                let field_26_u32 = if field_25_u32_list.is_some() && probe + 4 <= entry_end {
                    let pre_26 = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v),
                        _ => { probe = pre_26; None }
                    }
                } else { None };
                let field_27_u32_list = if field_26_u32.is_some() {
                    let pre_27 = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_27; None }
                    }
                } else { None };
                let field_28_u32 = if field_27_u32_list.is_some() && probe + 4 <= entry_end {
                    let pre_28 = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v),
                        _ => { probe = pre_28; None }
                    }
                } else { None };
                let field_29_u32_list = if field_28_u32.is_some() {
                    let pre_ = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_30_u32_list = if field_29_u32_list.is_some() {
                    let pre_ = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_31_u32_list = if field_30_u32_list.is_some() {
                    let pre_ = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_32_u32_list = if field_31_u32_list.is_some() {
                    let pre_ = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_33_u32 = if field_32_u32_list.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_34_u32 = if field_33_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_35_u32_list = if field_34_u32.is_some() {
                    let pre_ = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_36_u32 = if field_35_u32_list.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_37_u32 = if field_36_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_38_u32 = if field_37_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_39_u32_list = if field_38_u32.is_some() {
                    let pre_ = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_40_u32_list = if field_39_u32_list.is_some() {
                    let pre_ = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_41_u32 = if field_40_u32_list.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_42_u32 = if field_41_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_43_u32 = if field_42_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_44_u32 = if field_43_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_45_u32 = if field_44_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_46_u32 = if field_45_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_47_u32 = if field_46_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_48_u32 = if field_47_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_49_u32_list = if field_48_u32.is_some() {
                    let pre_ = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_50_u32_list = if field_49_u32_list.is_some() {
                    let pre_ = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_51_u32_list = if field_50_u32_list.is_some() {
                    let pre_ = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_52_u32_list = if field_51_u32_list.is_some() {
                    let pre_ = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_53_u32_list = if field_52_u32_list.is_some() {
                    let pre_ = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_54_u32_list = if field_53_u32_list.is_some() {
                    let pre_ = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_55_u32_list = if field_54_u32_list.is_some() {
                    let pre_ = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_56_u32_list = if field_55_u32_list.is_some() {
                    let pre_ = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_57_u32_list = if field_56_u32_list.is_some() {
                    let pre_ = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_58_u32_list = if field_57_u32_list.is_some() {
                    let pre_ = probe;
                    match <CArray<u32>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let field_59_u32 = if field_58_u32_list.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_60_u32 = if field_59_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_61_u32 = if field_60_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_62_u32 = if field_61_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_63_u32 = if field_62_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_64_u32 = if field_63_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_65_u32 = if field_64_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_66_u32 = if field_65_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_67_u32 = if field_66_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_68_u32 = if field_67_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_69_u32 = if field_68_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_70_u32 = if field_69_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_71_u32 = if field_70_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_72_u32 = if field_71_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_73_u32 = if field_72_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_74_u32 = if field_73_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_75_u32 = if field_74_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_76_u32 = if field_75_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_77_u32 = if field_76_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_78_u32 = if field_77_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_79_u32 = if field_78_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_80_u32 = if field_79_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_81_u32 = if field_80_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_82_u32 = if field_81_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_83_u32 = if field_82_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_84_u32 = if field_83_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_85_u32 = if field_84_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_86_u32 = if field_85_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_87_u32 = if field_86_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_88_u32 = if field_87_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_89_u32 = if field_88_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_90_u32 = if field_89_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_91_u32 = if field_90_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_92_u32 = if field_91_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_93_u32 = if field_92_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_94_u32 = if field_93_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_95_u32 = if field_94_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_96_u32 = if field_95_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_97_u32 = if field_96_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_98_u32 = if field_97_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_99_u32 = if field_98_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_100_u32 = if field_99_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_101_u32 = if field_100_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_102_u32 = if field_101_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_103_u32 = if field_102_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_104_u32 = if field_103_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_105_u32 = if field_104_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_106_u32 = if field_105_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_107_u32 = if field_106_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let post_blob = data[probe..entry_end].to_vec();
                *offset = entry_end;
                Ok(GimmickTail::Decoded {
                    gimmick_interaction_override_list: list,
                    use_interaction_ui_socket: ui,
                    use_sub_part_for_interaction: sp,
                    property_list: pl,
                    gimmick_name_hash: gnh,
                    gimmick_name: gn,
                    emoji_texture_id: eti,
                    dev_memo: dm,
                    hash_pair_list: hpl,
                    hash_single_list: hsl,
                    trigger_event_handler_list,
                    gimmick_chart_parameter_list,
                    field_19_u32_list,
                    field_20_u32_list,
                    field_21_u32_list,
                    field_22_u32_list,
                    field_23_u32_list,
                    field_24_u32_list,
                    field_25_u32_list,
                    field_26_u32,
                    field_27_u32_list,
                    field_28_u32,
                    field_29_u32_list,
                    field_30_u32_list,
                    field_31_u32_list,
                    field_32_u32_list,
                    field_33_u32,
                    field_34_u32,
                    field_35_u32_list,
                    field_36_u32,
                    field_37_u32,
                    field_38_u32,
                    field_39_u32_list,
                    field_40_u32_list,
                    field_41_u32,
                    field_42_u32,
                    field_43_u32,
                    field_44_u32,
                    field_45_u32,
                    field_46_u32,
                    field_47_u32,
                    field_48_u32,
                    field_49_u32_list,
                    field_50_u32_list,
                    field_51_u32_list,
                    field_52_u32_list,
                    field_53_u32_list,
                    field_54_u32_list,
                    field_55_u32_list,
                    field_56_u32_list,
                    field_57_u32_list,
                    field_58_u32_list,
                    field_59_u32,
                    field_60_u32,
                    field_61_u32,
                    field_62_u32,
                    field_63_u32,
                    field_64_u32,
                    field_65_u32,
                    field_66_u32,
                    field_67_u32,
                    field_68_u32,
                    field_69_u32,
                    field_70_u32,
                    field_71_u32,
                    field_72_u32,
                    field_73_u32,
                    field_74_u32,
                    field_75_u32,
                    field_76_u32,
                    field_77_u32,
                    field_78_u32,
                    field_79_u32,
                    field_80_u32,
                    field_81_u32,
                    field_82_u32,
                    field_83_u32,
                    field_84_u32,
                    field_85_u32,
                    field_86_u32,
                    field_87_u32,
                    field_88_u32,
                    field_89_u32,
                    field_90_u32,
                    field_91_u32,
                    field_92_u32,
                    field_93_u32,
                    field_94_u32,
                    field_95_u32,
                    field_96_u32,
                    field_97_u32,
                    field_98_u32,
                    field_99_u32,
                    field_100_u32,
                    field_101_u32,
                    field_102_u32,
                    field_103_u32,
                    field_104_u32,
                    field_105_u32,
                    field_106_u32,
                    field_107_u32,
                    post_blob,
                })
            }
            Err(_) => {
                let blob = data[tail_start..entry_end].to_vec();
                *offset = entry_end;
                Ok(GimmickTail::Raw(blob))
            }
        }
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match self {
            GimmickTail::Decoded { gimmick_interaction_override_list,
                use_interaction_ui_socket, use_sub_part_for_interaction,
                property_list, gimmick_name_hash, gimmick_name,
                emoji_texture_id, dev_memo,
                hash_pair_list, hash_single_list,
                trigger_event_handler_list, gimmick_chart_parameter_list,
                field_19_u32_list, field_20_u32_list,
                field_21_u32_list, field_22_u32_list,
                field_23_u32_list, field_24_u32_list,
                field_25_u32_list, field_26_u32, field_27_u32_list,
                field_28_u32, field_29_u32_list, field_30_u32_list,
                field_31_u32_list, field_32_u32_list,
                field_33_u32, field_34_u32,
                field_35_u32_list, field_36_u32,
                field_37_u32, field_38_u32,
                field_39_u32_list, field_40_u32_list,
                field_41_u32, field_42_u32, field_43_u32, field_44_u32, field_45_u32, field_46_u32, field_47_u32, field_48_u32, field_49_u32_list, field_50_u32_list,
                field_51_u32_list, field_52_u32_list, field_53_u32_list, field_54_u32_list,
                field_55_u32_list, field_56_u32_list, field_57_u32_list, field_58_u32_list,
                field_59_u32, field_60_u32, field_61_u32, field_62_u32,
                field_63_u32, field_64_u32, field_65_u32, field_66_u32,
                field_67_u32, field_68_u32, field_69_u32, field_70_u32,
                field_71_u32, field_72_u32, field_73_u32, field_74_u32,
                field_75_u32, field_76_u32, field_77_u32, field_78_u32,
                field_79_u32, field_80_u32, field_81_u32, field_82_u32,
                field_83_u32, field_84_u32, field_85_u32, field_86_u32,
                field_87_u32, field_88_u32, field_89_u32, field_90_u32,
                field_91_u32, field_92_u32, field_93_u32, field_94_u32,
                field_95_u32, field_96_u32, field_97_u32, field_98_u32,
                field_99_u32, field_100_u32, field_101_u32, field_102_u32,
                field_103_u32, field_104_u32, field_105_u32, field_106_u32,
                field_107_u32, post_blob } => {
                gimmick_interaction_override_list.write_to(w)?;
                use_interaction_ui_socket.write_to(w)?;
                use_sub_part_for_interaction.write_to(w)?;
                property_list.write_to(w)?;
                gimmick_name_hash.write_to(w)?;
                gimmick_name.write_to(w)?;
                emoji_texture_id.write_to(w)?;
                dev_memo.write_to(w)?;
                hash_pair_list.write_to(w)?;
                hash_single_list.write_to(w)?;
                if let Some(arr) = trigger_event_handler_list {
                    arr.write_to(w)?;
                }
                if let Some(arr) = gimmick_chart_parameter_list {
                    arr.write_to(w)?;
                }
                if let Some(arr) = field_19_u32_list {
                    arr.write_to(w)?;
                }
                if let Some(arr) = field_20_u32_list {
                    arr.write_to(w)?;
                }
                if let Some(arr) = field_21_u32_list {
                    arr.write_to(w)?;
                }
                if let Some(arr) = field_22_u32_list {
                    arr.write_to(w)?;
                }
                if let Some(arr) = field_23_u32_list {
                    arr.write_to(w)?;
                }
                if let Some(arr) = field_24_u32_list {
                    arr.write_to(w)?;
                }
                if let Some(arr) = field_25_u32_list {
                    arr.write_to(w)?;
                }
                if let Some(v) = field_26_u32 {
                    v.write_to(w)?;
                }
                if let Some(arr) = field_27_u32_list {
                    arr.write_to(w)?;
                }
                if let Some(v) = field_28_u32 {
                    v.write_to(w)?;
                }
                if let Some(arr) = field_29_u32_list { arr.write_to(w)?; }
                if let Some(arr) = field_30_u32_list { arr.write_to(w)?; }
                if let Some(arr) = field_31_u32_list { arr.write_to(w)?; }
                if let Some(arr) = field_32_u32_list { arr.write_to(w)?; }
                if let Some(v) = field_33_u32 { v.write_to(w)?; }
                if let Some(v) = field_34_u32 { v.write_to(w)?; }
                if let Some(arr) = field_35_u32_list { arr.write_to(w)?; }
                if let Some(v) = field_36_u32 { v.write_to(w)?; }
                if let Some(v) = field_37_u32 { v.write_to(w)?; }
                if let Some(v) = field_38_u32 { v.write_to(w)?; }
                if let Some(arr) = field_39_u32_list { arr.write_to(w)?; }
                if let Some(arr) = field_40_u32_list { arr.write_to(w)?; }
                if let Some(v) = field_41_u32 { v.write_to(w)?; }
                if let Some(v) = field_42_u32 { v.write_to(w)?; }
                if let Some(v) = field_43_u32 { v.write_to(w)?; }
                if let Some(v) = field_44_u32 { v.write_to(w)?; }
                if let Some(v) = field_45_u32 { v.write_to(w)?; }
                if let Some(v) = field_46_u32 { v.write_to(w)?; }
                if let Some(v) = field_47_u32 { v.write_to(w)?; }
                if let Some(v) = field_48_u32 { v.write_to(w)?; }
                if let Some(arr) = field_49_u32_list { arr.write_to(w)?; }
                if let Some(arr) = field_50_u32_list { arr.write_to(w)?; }
                if let Some(arr) = field_51_u32_list { arr.write_to(w)?; }
                if let Some(arr) = field_52_u32_list { arr.write_to(w)?; }
                if let Some(arr) = field_53_u32_list { arr.write_to(w)?; }
                if let Some(arr) = field_54_u32_list { arr.write_to(w)?; }
                if let Some(arr) = field_55_u32_list { arr.write_to(w)?; }
                if let Some(arr) = field_56_u32_list { arr.write_to(w)?; }
                if let Some(arr) = field_57_u32_list { arr.write_to(w)?; }
                if let Some(arr) = field_58_u32_list { arr.write_to(w)?; }
                if let Some(v) = field_59_u32 { v.write_to(w)?; }
                if let Some(v) = field_60_u32 { v.write_to(w)?; }
                if let Some(v) = field_61_u32 { v.write_to(w)?; }
                if let Some(v) = field_62_u32 { v.write_to(w)?; }
                if let Some(v) = field_63_u32 { v.write_to(w)?; }
                if let Some(v) = field_64_u32 { v.write_to(w)?; }
                if let Some(v) = field_65_u32 { v.write_to(w)?; }
                if let Some(v) = field_66_u32 { v.write_to(w)?; }
                if let Some(v) = field_67_u32 { v.write_to(w)?; }
                if let Some(v) = field_68_u32 { v.write_to(w)?; }
                if let Some(v) = field_69_u32 { v.write_to(w)?; }
                if let Some(v) = field_70_u32 { v.write_to(w)?; }
                if let Some(v) = field_71_u32 { v.write_to(w)?; }
                if let Some(v) = field_72_u32 { v.write_to(w)?; }
                if let Some(v) = field_73_u32 { v.write_to(w)?; }
                if let Some(v) = field_74_u32 { v.write_to(w)?; }
                if let Some(v) = field_75_u32 { v.write_to(w)?; }
                if let Some(v) = field_76_u32 { v.write_to(w)?; }
                if let Some(v) = field_77_u32 { v.write_to(w)?; }
                if let Some(v) = field_78_u32 { v.write_to(w)?; }
                if let Some(v) = field_79_u32 { v.write_to(w)?; }
                if let Some(v) = field_80_u32 { v.write_to(w)?; }
                if let Some(v) = field_81_u32 { v.write_to(w)?; }
                if let Some(v) = field_82_u32 { v.write_to(w)?; }
                if let Some(v) = field_83_u32 { v.write_to(w)?; }
                if let Some(v) = field_84_u32 { v.write_to(w)?; }
                if let Some(v) = field_85_u32 { v.write_to(w)?; }
                if let Some(v) = field_86_u32 { v.write_to(w)?; }
                if let Some(v) = field_87_u32 { v.write_to(w)?; }
                if let Some(v) = field_88_u32 { v.write_to(w)?; }
                if let Some(v) = field_89_u32 { v.write_to(w)?; }
                if let Some(v) = field_90_u32 { v.write_to(w)?; }
                if let Some(v) = field_91_u32 { v.write_to(w)?; }
                if let Some(v) = field_92_u32 { v.write_to(w)?; }
                if let Some(v) = field_93_u32 { v.write_to(w)?; }
                if let Some(v) = field_94_u32 { v.write_to(w)?; }
                if let Some(v) = field_95_u32 { v.write_to(w)?; }
                if let Some(v) = field_96_u32 { v.write_to(w)?; }
                if let Some(v) = field_97_u32 { v.write_to(w)?; }
                if let Some(v) = field_98_u32 { v.write_to(w)?; }
                if let Some(v) = field_99_u32 { v.write_to(w)?; }
                if let Some(v) = field_100_u32 { v.write_to(w)?; }
                if let Some(v) = field_101_u32 { v.write_to(w)?; }
                if let Some(v) = field_102_u32 { v.write_to(w)?; }
                if let Some(v) = field_103_u32 { v.write_to(w)?; }
                if let Some(v) = field_104_u32 { v.write_to(w)?; }
                if let Some(v) = field_105_u32 { v.write_to(w)?; }
                if let Some(v) = field_106_u32 { v.write_to(w)?; }
                if let Some(v) = field_107_u32 { v.write_to(w)?; }
                w.write_all(post_blob)
            }
            GimmickTail::Raw(b) => w.write_all(b),
        }
    }

    pub fn to_json_value(&self) -> Value {
        match self {
            GimmickTail::Decoded { gimmick_interaction_override_list,
                use_interaction_ui_socket, use_sub_part_for_interaction,
                property_list, gimmick_name_hash, gimmick_name,
                emoji_texture_id, dev_memo,
                hash_pair_list, hash_single_list,
                trigger_event_handler_list, gimmick_chart_parameter_list,
                field_19_u32_list, field_20_u32_list,
                field_21_u32_list, field_22_u32_list,
                field_23_u32_list, field_24_u32_list,
                field_25_u32_list, field_26_u32, field_27_u32_list,
                field_28_u32, field_29_u32_list, field_30_u32_list,
                field_31_u32_list, field_32_u32_list,
                field_33_u32, field_34_u32,
                field_35_u32_list, field_36_u32,
                field_37_u32, field_38_u32,
                field_39_u32_list, field_40_u32_list,
                field_41_u32, field_42_u32, field_43_u32, field_44_u32, field_45_u32, field_46_u32, field_47_u32, field_48_u32, field_49_u32_list, field_50_u32_list,
                field_51_u32_list, field_52_u32_list, field_53_u32_list, field_54_u32_list,
                field_55_u32_list, field_56_u32_list, field_57_u32_list, field_58_u32_list,
                field_59_u32, field_60_u32, field_61_u32, field_62_u32,
                field_63_u32, field_64_u32, field_65_u32, field_66_u32,
                field_67_u32, field_68_u32, field_69_u32, field_70_u32,
                field_71_u32, field_72_u32, field_73_u32, field_74_u32,
                field_75_u32, field_76_u32, field_77_u32, field_78_u32,
                field_79_u32, field_80_u32, field_81_u32, field_82_u32,
                field_83_u32, field_84_u32, field_85_u32, field_86_u32,
                field_87_u32, field_88_u32, field_89_u32, field_90_u32,
                field_91_u32, field_92_u32, field_93_u32, field_94_u32,
                field_95_u32, field_96_u32, field_97_u32, field_98_u32,
                field_99_u32, field_100_u32, field_101_u32, field_102_u32,
                field_103_u32, field_104_u32, field_105_u32, field_106_u32,
                field_107_u32, post_blob } => {
                let mut m = Map::new();
                m.insert("kind".to_string(), Value::String("Decoded".to_string()));
                m.insert("gimmick_interaction_override_list".to_string(),
                         gimmick_interaction_override_list.to_json_value());
                m.insert("use_interaction_ui_socket".to_string(), use_interaction_ui_socket.to_json_value());
                m.insert("use_sub_part_for_interaction".to_string(), use_sub_part_for_interaction.to_json_value());
                m.insert("property_list".to_string(), property_list.to_json_value());
                m.insert("gimmick_name_hash".to_string(), gimmick_name_hash.to_json_value());
                m.insert("gimmick_name".to_string(), gimmick_name.to_json_value());
                m.insert("emoji_texture_id".to_string(), emoji_texture_id.to_json_value());
                m.insert("dev_memo".to_string(), dev_memo.to_json_value());
                m.insert("hash_pair_list".to_string(), hash_pair_list.to_json_value());
                m.insert("hash_single_list".to_string(), hash_single_list.to_json_value());
                m.insert("trigger_event_handler_list".to_string(), match trigger_event_handler_list {
                    Some(arr) => arr.to_json_value(),
                    None => Value::Null,
                });
                m.insert("gimmick_chart_parameter_list".to_string(), match gimmick_chart_parameter_list {
                    Some(arr) => arr.to_json_value(),
                    None => Value::Null,
                });
                m.insert("field_19_u32_list".to_string(), match field_19_u32_list {
                    Some(arr) => arr.to_json_value(),
                    None => Value::Null,
                });
                m.insert("field_20_u32_list".to_string(), match field_20_u32_list {
                    Some(arr) => arr.to_json_value(),
                    None => Value::Null,
                });
                m.insert("field_21_u32_list".to_string(), match field_21_u32_list {
                    Some(arr) => arr.to_json_value(),
                    None => Value::Null,
                });
                m.insert("field_22_u32_list".to_string(), match field_22_u32_list {
                    Some(arr) => arr.to_json_value(),
                    None => Value::Null,
                });
                m.insert("field_23_u32_list".to_string(), match field_23_u32_list {
                    Some(arr) => arr.to_json_value(),
                    None => Value::Null,
                });
                m.insert("field_24_u32_list".to_string(), match field_24_u32_list {
                    Some(arr) => arr.to_json_value(),
                    None => Value::Null,
                });
                m.insert("field_25_u32_list".to_string(), match field_25_u32_list {
                    Some(arr) => arr.to_json_value(),
                    None => Value::Null,
                });
                m.insert("field_26_u32".to_string(), match field_26_u32 {
                    Some(v) => v.to_json_value(),
                    None => Value::Null,
                });
                m.insert("field_27_u32_list".to_string(), match field_27_u32_list {
                    Some(arr) => arr.to_json_value(),
                    None => Value::Null,
                });
                m.insert("field_28_u32".to_string(), match field_28_u32 {
                    Some(v) => v.to_json_value(),
                    None => Value::Null,
                });
                m.insert("field_29_u32_list".to_string(), match field_29_u32_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null });
                m.insert("field_30_u32_list".to_string(), match field_30_u32_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null });
                m.insert("field_31_u32_list".to_string(), match field_31_u32_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null });
                m.insert("field_32_u32_list".to_string(), match field_32_u32_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null });
                m.insert("field_33_u32".to_string(), match field_33_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_34_u32".to_string(), match field_34_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_35_u32_list".to_string(), match field_35_u32_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null });
                m.insert("field_36_u32".to_string(), match field_36_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_37_u32".to_string(), match field_37_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_38_u32".to_string(), match field_38_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_39_u32_list".to_string(), match field_39_u32_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null });
                m.insert("field_40_u32_list".to_string(), match field_40_u32_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null });
                m.insert("field_41_u32".to_string(), match field_41_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_42_u32".to_string(), match field_42_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_43_u32".to_string(), match field_43_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_44_u32".to_string(), match field_44_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_45_u32".to_string(), match field_45_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_46_u32".to_string(), match field_46_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_47_u32".to_string(), match field_47_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_48_u32".to_string(), match field_48_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_49_u32_list".to_string(), match field_49_u32_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null });
                m.insert("field_50_u32_list".to_string(), match field_50_u32_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null });
                m.insert("field_51_u32_list".to_string(), match field_51_u32_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null });
                m.insert("field_52_u32_list".to_string(), match field_52_u32_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null });
                m.insert("field_53_u32_list".to_string(), match field_53_u32_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null });
                m.insert("field_54_u32_list".to_string(), match field_54_u32_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null });
                m.insert("field_55_u32_list".to_string(), match field_55_u32_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null });
                m.insert("field_56_u32_list".to_string(), match field_56_u32_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null });
                m.insert("field_57_u32_list".to_string(), match field_57_u32_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null });
                m.insert("field_58_u32_list".to_string(), match field_58_u32_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null });
                m.insert("field_59_u32".to_string(), match field_59_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_60_u32".to_string(), match field_60_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_61_u32".to_string(), match field_61_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_62_u32".to_string(), match field_62_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_63_u32".to_string(), match field_63_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_64_u32".to_string(), match field_64_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_65_u32".to_string(), match field_65_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_66_u32".to_string(), match field_66_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_67_u32".to_string(), match field_67_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_68_u32".to_string(), match field_68_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_69_u32".to_string(), match field_69_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_70_u32".to_string(), match field_70_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_71_u32".to_string(), match field_71_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_72_u32".to_string(), match field_72_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_73_u32".to_string(), match field_73_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_74_u32".to_string(), match field_74_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_75_u32".to_string(), match field_75_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_76_u32".to_string(), match field_76_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_77_u32".to_string(), match field_77_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_78_u32".to_string(), match field_78_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_79_u32".to_string(), match field_79_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_80_u32".to_string(), match field_80_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_81_u32".to_string(), match field_81_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_82_u32".to_string(), match field_82_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_83_u32".to_string(), match field_83_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_84_u32".to_string(), match field_84_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_85_u32".to_string(), match field_85_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_86_u32".to_string(), match field_86_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_87_u32".to_string(), match field_87_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_88_u32".to_string(), match field_88_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_89_u32".to_string(), match field_89_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_90_u32".to_string(), match field_90_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_91_u32".to_string(), match field_91_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_92_u32".to_string(), match field_92_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_93_u32".to_string(), match field_93_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_94_u32".to_string(), match field_94_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_95_u32".to_string(), match field_95_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_96_u32".to_string(), match field_96_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_97_u32".to_string(), match field_97_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_98_u32".to_string(), match field_98_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_99_u32".to_string(), match field_99_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_100_u32".to_string(), match field_100_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_101_u32".to_string(), match field_101_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_102_u32".to_string(), match field_102_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_103_u32".to_string(), match field_103_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_104_u32".to_string(), match field_104_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_105_u32".to_string(), match field_105_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_106_u32".to_string(), match field_106_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_107_u32".to_string(), match field_107_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("_post_blob_b64".to_string(), Value::String(B64.encode(post_blob)));
                Value::Object(m)
            }
            GimmickTail::Raw(b) => {
                let mut m = Map::new();
                m.insert("kind".to_string(), Value::String("Raw".to_string()));
                m.insert("_b64".to_string(), Value::String(B64.encode(b)));
                Value::Object(m)
            }
        }
    }

    pub fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "GimmickTail: expected object",
        ))?;
        let kind = json_get_field(obj, "kind")?.as_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "GimmickTail.kind: expected string"))?;
        match kind {
            "Decoded" => {
                <GimmickInteractionOverrideCArray as WriteJsonValue>::write_from_json(
                    w, json_get_field(obj, "gimmick_interaction_override_list")?,
                )?;
                <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "use_interaction_ui_socket")?)?;
                <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "use_sub_part_for_interaction")?)?;
                <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "property_list")?)?;
                <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "gimmick_name_hash")?)?;
                <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "gimmick_name")?)?;
                <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "emoji_texture_id")?)?;
                <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "dev_memo")?)?;
                <CArray<GimmickHashPair> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "hash_pair_list")?)?;
                <CArray<GimmickHashSingle> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "hash_single_list")?)?;
                let teh = json_get_field(obj, "trigger_event_handler_list")?;
                if !teh.is_null() {
                    <CArray<OptionalTriggerGamePlayEventHandlerData> as WriteJsonValue>::write_from_json(w, teh)?;
                }
                let gcp = json_get_field(obj, "gimmick_chart_parameter_list")?;
                if !gcp.is_null() {
                    <CArray<GimmickChartParameter> as WriteJsonValue>::write_from_json(w, gcp)?;
                }
                let f19 = json_get_field(obj, "field_19_u32_list")?;
                if !f19.is_null() {
                    <CArray<u32> as WriteJsonValue>::write_from_json(w, f19)?;
                }
                let f20 = json_get_field(obj, "field_20_u32_list")?;
                if !f20.is_null() {
                    <CArray<u32> as WriteJsonValue>::write_from_json(w, f20)?;
                }
                let f21 = json_get_field(obj, "field_21_u32_list")?;
                if !f21.is_null() {
                    <CArray<u32> as WriteJsonValue>::write_from_json(w, f21)?;
                }
                let f22 = json_get_field(obj, "field_22_u32_list")?;
                if !f22.is_null() {
                    <CArray<u32> as WriteJsonValue>::write_from_json(w, f22)?;
                }
                let f23 = json_get_field(obj, "field_23_u32_list")?;
                if !f23.is_null() {
                    <CArray<u32> as WriteJsonValue>::write_from_json(w, f23)?;
                }
                let f24 = json_get_field(obj, "field_24_u32_list")?;
                if !f24.is_null() {
                    <CArray<u32> as WriteJsonValue>::write_from_json(w, f24)?;
                }
                let f25 = json_get_field(obj, "field_25_u32_list")?;
                if !f25.is_null() {
                    <CArray<u32> as WriteJsonValue>::write_from_json(w, f25)?;
                }
                let f26 = json_get_field(obj, "field_26_u32")?;
                if !f26.is_null() {
                    <u32 as WriteJsonValue>::write_from_json(w, f26)?;
                }
                let f27 = json_get_field(obj, "field_27_u32_list")?;
                if !f27.is_null() {
                    <CArray<u32> as WriteJsonValue>::write_from_json(w, f27)?;
                }
                let f28 = json_get_field(obj, "field_28_u32")?;
                if !f28.is_null() {
                    <u32 as WriteJsonValue>::write_from_json(w, f28)?;
                }
                for k in &["field_29_u32_list", "field_30_u32_list",
                           "field_31_u32_list", "field_32_u32_list"] {
                    let v = json_get_field(obj, k)?;
                    if !v.is_null() {
                        <CArray<u32> as WriteJsonValue>::write_from_json(w, v)?;
                    }
                }
                let f33 = json_get_field(obj, "field_33_u32")?;
                if !f33.is_null() {
                    <u32 as WriteJsonValue>::write_from_json(w, f33)?;
                }
                let f34 = json_get_field(obj, "field_34_u32")?;
                if !f34.is_null() {
                    <u32 as WriteJsonValue>::write_from_json(w, f34)?;
                }
                let f35 = json_get_field(obj, "field_35_u32_list")?;
                if !f35.is_null() {
                    <CArray<u32> as WriteJsonValue>::write_from_json(w, f35)?;
                }
                let f36 = json_get_field(obj, "field_36_u32")?;
                if !f36.is_null() {
                    <u32 as WriteJsonValue>::write_from_json(w, f36)?;
                }
                let f37 = json_get_field(obj, "field_37_u32")?;
                if !f37.is_null() {
                    <u32 as WriteJsonValue>::write_from_json(w, f37)?;
                }
                let f38 = json_get_field(obj, "field_38_u32")?;
                if !f38.is_null() {
                    <u32 as WriteJsonValue>::write_from_json(w, f38)?;
                }
                for k in &["field_39_u32_list", "field_40_u32_list"] {
                    let v = json_get_field(obj, k)?;
                    if !v.is_null() {
                        <CArray<u32> as WriteJsonValue>::write_from_json(w, v)?;
                    }
                }
                let f41 = json_get_field(obj, "field_41_u32")?;
                if !f41.is_null() {
                    <u32 as WriteJsonValue>::write_from_json(w, f41)?;
                }
                let f42 = json_get_field(obj, "field_42_u32")?;
                if !f42.is_null() {
                    <u32 as WriteJsonValue>::write_from_json(w, f42)?;
                }
                let f43 = json_get_field(obj, "field_43_u32")?;
                if !f43.is_null() {
                    <u32 as WriteJsonValue>::write_from_json(w, f43)?;
                }
                let f44 = json_get_field(obj, "field_44_u32")?;
                if !f44.is_null() {
                    <u32 as WriteJsonValue>::write_from_json(w, f44)?;
                }
                let f45 = json_get_field(obj, "field_45_u32")?;
                if !f45.is_null() {
                    <u32 as WriteJsonValue>::write_from_json(w, f45)?;
                }
                let f46 = json_get_field(obj, "field_46_u32")?;
                if !f46.is_null() {
                    <u32 as WriteJsonValue>::write_from_json(w, f46)?;
                }
                let f47 = json_get_field(obj, "field_47_u32")?;
                if !f47.is_null() {
                    <u32 as WriteJsonValue>::write_from_json(w, f47)?;
                }
                let f48 = json_get_field(obj, "field_48_u32")?;
                if !f48.is_null() {
                    <u32 as WriteJsonValue>::write_from_json(w, f48)?;
                }
                let f49 = json_get_field(obj, "field_49_u32_list")?;
                if !f49.is_null() {
                    <CArray<u32> as WriteJsonValue>::write_from_json(w, f49)?;
                }
                for k in &["field_50_u32_list", "field_51_u32_list",
                           "field_52_u32_list", "field_53_u32_list",
                           "field_54_u32_list", "field_55_u32_list",
                           "field_56_u32_list", "field_57_u32_list",
                           "field_58_u32_list"] {
                    let v = json_get_field(obj, k)?;
                    if !v.is_null() {
                        <CArray<u32> as WriteJsonValue>::write_from_json(w, v)?;
                    }
                }
                for k in &["field_59_u32", "field_60_u32", "field_61_u32",
                           "field_62_u32", "field_63_u32", "field_64_u32",
                           "field_65_u32", "field_66_u32", "field_67_u32",
                           "field_68_u32", "field_69_u32", "field_70_u32",
                           "field_71_u32", "field_72_u32", "field_73_u32",
                           "field_74_u32", "field_75_u32", "field_76_u32",
                           "field_77_u32", "field_78_u32", "field_79_u32",
                           "field_80_u32", "field_81_u32", "field_82_u32",
                           "field_83_u32", "field_84_u32", "field_85_u32",
                           "field_86_u32", "field_87_u32", "field_88_u32",
                           "field_89_u32", "field_90_u32"] {
                    let v = json_get_field(obj, k)?;
                    if !v.is_null() {
                        <u32 as WriteJsonValue>::write_from_json(w, v)?;
                    }
                }
                for k in &["field_91_u32", "field_92_u32", "field_93_u32",
                           "field_94_u32", "field_95_u32", "field_96_u32",
                           "field_97_u32", "field_98_u32", "field_99_u32",
                           "field_100_u32", "field_101_u32", "field_102_u32",
                           "field_103_u32", "field_104_u32", "field_105_u32",
                           "field_106_u32", "field_107_u32"] {
                    let v = json_get_field(obj, k)?;
                    if !v.is_null() {
                        <u32 as WriteJsonValue>::write_from_json(w, v)?;
                    }
                }
                let b64 = json_get_field(obj, "_post_blob_b64")?.as_str()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                        "GimmickTail.Decoded._post_blob_b64: expected string"))?;
                let bytes = B64.decode(b64).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
                    format!("GimmickTail.Decoded._post_blob_b64: invalid base64: {}", e)))?;
                w.extend_from_slice(&bytes);
                Ok(())
            }
            "Raw" => {
                let b64 = json_get_field(obj, "_b64")?.as_str()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                        "GimmickTail.Raw._b64: expected string"))?;
                let bytes = B64.decode(b64).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
                    format!("GimmickTail.Raw._b64: invalid base64: {}", e)))?;
                w.extend_from_slice(&bytes);
                Ok(())
            }
            other => Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("GimmickTail.kind: unknown variant {:?}", other))),
        }
    }
}

#[derive(Debug)]
pub struct GimmickInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub prefab_path: CString<'a>,
    pub gimmick_group_info: u32,
    pub breakable_object_info: u16,
    pub tail: GimmickTail<'a>,
}

impl<'a> GimmickInfo<'a> {
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
    ) -> io::Result<Self> {
        let entry_start = *offset;
        let entry_end = entry_start + entry_size;

        let key = u32::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;
        let prefab_path = CString::read_from(data, offset)?;
        let gimmick_group_info = u32::read_from(data, offset)?;
        let breakable_object_info = u16::read_from(data, offset)?;
        let tail = GimmickTail::read_with_size(data, offset, entry_end)?;

        Ok(Self {
            key, string_key, is_blocked, prefab_path,
            gimmick_group_info, breakable_object_info, tail,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.prefab_path.write_to(w)?;
        self.gimmick_group_info.write_to(w)?;
        self.breakable_object_info.write_to(w)?;
        self.tail.write_to(w)
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("prefab_path".to_string(), self.prefab_path.to_json_value());
        m.insert("gimmick_group_info".to_string(), self.gimmick_group_info.to_json_value());
        m.insert("breakable_object_info".to_string(), self.breakable_object_info.to_json_value());
        m.insert("tail".to_string(), self.tail.to_json_value());
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "prefab_path")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "gimmick_group_info")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "breakable_object_info")?)?;
        GimmickTail::write_from_json(w, json_get_field(obj, "tail")?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gimmickinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gimmickinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        let mut decoded = 0usize;
        let mut raw = 0usize;
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            let item = GimmickInfo::read_with_size(&data, &mut c, e - s)
                .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er));
            assert_eq!(c, *e);
            match &item.tail {
                GimmickTail::Decoded { .. } => decoded += 1,
                GimmickTail::Raw(_) => raw += 1,
            }
            items.push(item);
        }
        eprintln!("gimmickinfo: decoded={} raw={} (total={})", decoded, raw, ranges.len());
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "gimmickinfo roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
        let Ok(data) = std::fs::read(PABGB) else {
            eprintln!("SKIP: missing fixture {}", PABGB);
            return;
        };
        let Some(entries) = load_pabgh_offsets(PABGH) else {
            eprintln!("SKIP: missing pabgh fixture {}", PABGH);
            return;
        };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = GimmickInfo::read_with_size(&data, &mut cursor, end - start).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            GimmickInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }
}
