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
        /// Field 108 — u32.
        field_108_u32: Option<u32>,
        /// Field 109 — u32 (`0x00bf8000`).
        field_109_u32: Option<u32>,
        /// Field 110 — u32.
        field_110_u32: Option<u32>,
        /// Field 111 — u32 (`0x00bf8000`).
        field_111_u32: Option<u32>,
        /// Field 112 — u32.
        field_112_u32: Option<u32>,
        /// Field 113 — u32.
        field_113_u32: Option<u32>,
        /// Field 114 — u32.
        field_114_u32: Option<u32>,
        /// Field 115 — u32.
        field_115_u32: Option<u32>,
        /// Field 116 — u32.
        field_116_u32: Option<u32>,
        /// Field 117 — u32.
        field_117_u32: Option<u32>,
        /// Field 118 — u32.
        field_118_u32: Option<u32>,
        /// Field 119 — u32.
        field_119_u32: Option<u32>,
        /// Field 120 — u32.
        field_120_u32: Option<u32>,
        /// Field 121 — u32.
        field_121_u32: Option<u32>,
        /// Field 122 — u32.
        field_122_u32: Option<u32>,
        /// Field 123 — u32.
        field_123_u32: Option<u32>,
        /// Fields 124-139 — generic u32 sequence (mostly 0 + various values).
        field_124_u32: Option<u32>,
        field_125_u32: Option<u32>,
        field_126_u32: Option<u32>,
        field_127_u32: Option<u32>,
        field_128_u32: Option<u32>,
        field_129_u32: Option<u32>,
        field_130_u32: Option<u32>,
        field_131_u32: Option<u32>,
        field_132_u32: Option<u32>,
        field_133_u32: Option<u32>,
        field_134_u32: Option<u32>,
        field_135_u32: Option<u32>,
        field_136_u32: Option<u32>,
        field_137_u32: Option<u32>,
        field_138_u32: Option<u32>,
        field_139_u32: Option<u32>,
        /// Fields 140-155 — generic u32 sequence.
        field_140_u32: Option<u32>,
        field_141_u32: Option<u32>,
        field_142_u32: Option<u32>,
        field_143_u32: Option<u32>,
        field_144_u32: Option<u32>,
        field_145_u32: Option<u32>,
        field_146_u32: Option<u32>,
        field_147_u32: Option<u32>,
        field_148_u32: Option<u32>,
        field_149_u32: Option<u32>,
        field_150_u32: Option<u32>,
        field_151_u32: Option<u32>,
        field_152_u32: Option<u32>,
        field_153_u32: Option<u32>,
        field_154_u32: Option<u32>,
        field_155_u32: Option<u32>,
        /// Fields 156-171 — generic u32 sequence (string region).
        field_156_u32: Option<u32>,
        field_157_u32: Option<u32>,
        field_158_u32: Option<u32>,
        field_159_u32: Option<u32>,
        field_160_u32: Option<u32>,
        field_161_u32: Option<u32>,
        field_162_u32: Option<u32>,
        field_163_u32: Option<u32>,
        field_164_u32: Option<u32>,
        field_165_u32: Option<u32>,
        field_166_u32: Option<u32>,
        field_167_u32: Option<u32>,
        field_168_u32: Option<u32>,
        field_169_u32: Option<u32>,
        field_170_u32: Option<u32>,
        field_171_u32: Option<u32>,
        /// Fields 172-181 — generic u32 sequence (terminal trailing fields).
        field_172_u32: Option<u32>,
        field_173_u32: Option<u32>,
        field_174_u32: Option<u32>,
        field_175_u32: Option<u32>,
        field_176_u32: Option<u32>,
        field_177_u32: Option<u32>,
        field_178_u32: Option<u32>,
        field_179_u32: Option<u32>,
        field_180_u32: Option<u32>,
        field_181_u32: Option<u32>,
        /// Fields 182-197 — generic u32 sequence.
        field_182_u32: Option<u32>,
        field_183_u32: Option<u32>,
        field_184_u32: Option<u32>,
        field_185_u32: Option<u32>,
        field_186_u32: Option<u32>,
        field_187_u32: Option<u32>,
        field_188_u32: Option<u32>,
        field_189_u32: Option<u32>,
        field_190_u32: Option<u32>,
        field_191_u32: Option<u32>,
        field_192_u32: Option<u32>,
        field_193_u32: Option<u32>,
        field_194_u32: Option<u32>,
        field_195_u32: Option<u32>,
        field_196_u32: Option<u32>,
        field_197_u32: Option<u32>,
        /// Fields 198-213 — long-tail u32 sequence.
        field_198_u32: Option<u32>,
        field_199_u32: Option<u32>,
        field_200_u32: Option<u32>,
        field_201_u32: Option<u32>,
        field_202_u32: Option<u32>,
        field_203_u32: Option<u32>,
        field_204_u32: Option<u32>,
        field_205_u32: Option<u32>,
        field_206_u32: Option<u32>,
        field_207_u32: Option<u32>,
        field_208_u32: Option<u32>,
        field_209_u32: Option<u32>,
        field_210_u32: Option<u32>,
        field_211_u32: Option<u32>,
        field_212_u32: Option<u32>,
        field_213_u32: Option<u32>,
        /// Fields 214-229 — long-tail u32 sequence.
        field_214_u32: Option<u32>,
        field_215_u32: Option<u32>,
        field_216_u32: Option<u32>,
        field_217_u32: Option<u32>,
        field_218_u32: Option<u32>,
        field_219_u32: Option<u32>,
        field_220_u32: Option<u32>,
        field_221_u32: Option<u32>,
        field_222_u32: Option<u32>,
        field_223_u32: Option<u32>,
        field_224_u32: Option<u32>,
        field_225_u32: Option<u32>,
        field_226_u32: Option<u32>,
        field_227_u32: Option<u32>,
        field_228_u32: Option<u32>,
        field_229_u32: Option<u32>,
        /// Fields 230-245 — long-tail u32 sequence.
        field_230_u32: Option<u32>,
        field_231_u32: Option<u32>,
        field_232_u32: Option<u32>,
        field_233_u32: Option<u32>,
        field_234_u32: Option<u32>,
        field_235_u32: Option<u32>,
        field_236_u32: Option<u32>,
        field_237_u32: Option<u32>,
        field_238_u32: Option<u32>,
        field_239_u32: Option<u32>,
        field_240_u32: Option<u32>,
        field_241_u32: Option<u32>,
        field_242_u32: Option<u32>,
        field_243_u32: Option<u32>,
        field_244_u32: Option<u32>,
        field_245_u32: Option<u32>,
        /// Fields 246-277 — long-tail u32 sequence.
        field_246_u32: Option<u32>,
        field_247_u32: Option<u32>,
        field_248_u32: Option<u32>,
        field_249_u32: Option<u32>,
        field_250_u32: Option<u32>,
        field_251_u32: Option<u32>,
        field_252_u32: Option<u32>,
        field_253_u32: Option<u32>,
        field_254_u32: Option<u32>,
        field_255_u32: Option<u32>,
        field_256_u32: Option<u32>,
        field_257_u32: Option<u32>,
        field_258_u32: Option<u32>,
        field_259_u32: Option<u32>,
        field_260_u32: Option<u32>,
        field_261_u32: Option<u32>,
        field_262_u32: Option<u32>,
        field_263_u32: Option<u32>,
        field_264_u32: Option<u32>,
        field_265_u32: Option<u32>,
        field_266_u32: Option<u32>,
        field_267_u32: Option<u32>,
        field_268_u32: Option<u32>,
        field_269_u32: Option<u32>,
        field_270_u32: Option<u32>,
        field_271_u32: Option<u32>,
        field_272_u32: Option<u32>,
        field_273_u32: Option<u32>,
        field_274_u32: Option<u32>,
        field_275_u32: Option<u32>,
        field_276_u32: Option<u32>,
        field_277_u32: Option<u32>,
        /// Fields 278-309 — long-tail u32 sequence.
        field_278_u32: Option<u32>,
        field_279_u32: Option<u32>,
        field_280_u32: Option<u32>,
        field_281_u32: Option<u32>,
        field_282_u32: Option<u32>,
        field_283_u32: Option<u32>,
        field_284_u32: Option<u32>,
        field_285_u32: Option<u32>,
        field_286_u32: Option<u32>,
        field_287_u32: Option<u32>,
        field_288_u32: Option<u32>,
        field_289_u32: Option<u32>,
        field_290_u32: Option<u32>,
        field_291_u32: Option<u32>,
        field_292_u32: Option<u32>,
        field_293_u32: Option<u32>,
        field_294_u32: Option<u32>,
        field_295_u32: Option<u32>,
        field_296_u32: Option<u32>,
        field_297_u32: Option<u32>,
        field_298_u32: Option<u32>,
        field_299_u32: Option<u32>,
        field_300_u32: Option<u32>,
        field_301_u32: Option<u32>,
        field_302_u32: Option<u32>,
        field_303_u32: Option<u32>,
        field_304_u32: Option<u32>,
        field_305_u32: Option<u32>,
        field_306_u32: Option<u32>,
        field_307_u32: Option<u32>,
        field_308_u32: Option<u32>,
        field_309_u32: Option<u32>,
        /// Fields 310-341 — long-tail u32 sequence (XML-string region).
        field_310_u32: Option<u32>,
        field_311_u32: Option<u32>,
        field_312_u32: Option<u32>,
        field_313_u32: Option<u32>,
        field_314_u32: Option<u32>,
        field_315_u32: Option<u32>,
        field_316_u32: Option<u32>,
        field_317_u32: Option<u32>,
        field_318_u32: Option<u32>,
        field_319_u32: Option<u32>,
        field_320_u32: Option<u32>,
        field_321_u32: Option<u32>,
        field_322_u32: Option<u32>,
        field_323_u32: Option<u32>,
        field_324_u32: Option<u32>,
        field_325_u32: Option<u32>,
        field_326_u32: Option<u32>,
        field_327_u32: Option<u32>,
        field_328_u32: Option<u32>,
        field_329_u32: Option<u32>,
        field_330_u32: Option<u32>,
        field_331_u32: Option<u32>,
        field_332_u32: Option<u32>,
        field_333_u32: Option<u32>,
        field_334_u32: Option<u32>,
        field_335_u32: Option<u32>,
        field_336_u32: Option<u32>,
        field_337_u32: Option<u32>,
        field_338_u32: Option<u32>,
        field_339_u32: Option<u32>,
        field_340_u32: Option<u32>,
        field_341_u32: Option<u32>,
        /// Field 342 — u32 trigger count (for long-tail entries this starts a
        /// CArray<TriggerEntry> with `u8 flag + CString name + body`).
        field_342_u32_count: Option<u32>,
        /// Field 343 — u8 flag (typically 0x01) for first trigger entry.
        field_343_u8_flag: Option<u8>,
        /// Field 344 — u32 (length-prefix or value).
        field_344_u32: Option<u32>,
        /// Fields 345-376 — long-tail u32 sequence (trigger entry body).
        field_345_u32: Option<u32>,
        field_346_u32: Option<u32>,
        field_347_u32: Option<u32>,
        field_348_u32: Option<u32>,
        field_349_u32: Option<u32>,
        field_350_u32: Option<u32>,
        field_351_u32: Option<u32>,
        field_352_u32: Option<u32>,
        field_353_u32: Option<u32>,
        field_354_u32: Option<u32>,
        field_355_u32: Option<u32>,
        field_356_u32: Option<u32>,
        field_357_u32: Option<u32>,
        field_358_u32: Option<u32>,
        field_359_u32: Option<u32>,
        field_360_u32: Option<u32>,
        field_361_u32: Option<u32>,
        field_362_u32: Option<u32>,
        field_363_u32: Option<u32>,
        field_364_u32: Option<u32>,
        field_365_u32: Option<u32>,
        field_366_u32: Option<u32>,
        field_367_u32: Option<u32>,
        field_368_u32: Option<u32>,
        field_369_u32: Option<u32>,
        field_370_u32: Option<u32>,
        field_371_u32: Option<u32>,
        field_372_u32: Option<u32>,
        field_373_u32: Option<u32>,
        field_374_u32: Option<u32>,
        field_375_u32: Option<u32>,
        field_376_u32: Option<u32>,
        /// Fields 377-408 — long-tail u32 sequence.
        field_377_u32: Option<u32>,
        field_378_u32: Option<u32>,
        field_379_u32: Option<u32>,
        field_380_u32: Option<u32>,
        field_381_u32: Option<u32>,
        field_382_u32: Option<u32>,
        field_383_u32: Option<u32>,
        field_384_u32: Option<u32>,
        field_385_u32: Option<u32>,
        field_386_u32: Option<u32>,
        field_387_u32: Option<u32>,
        field_388_u32: Option<u32>,
        field_389_u32: Option<u32>,
        field_390_u32: Option<u32>,
        field_391_u32: Option<u32>,
        field_392_u32: Option<u32>,
        field_393_u32: Option<u32>,
        field_394_u32: Option<u32>,
        field_395_u32: Option<u32>,
        field_396_u32: Option<u32>,
        field_397_u32: Option<u32>,
        field_398_u32: Option<u32>,
        field_399_u32: Option<u32>,
        field_400_u32: Option<u32>,
        field_401_u32: Option<u32>,
        field_402_u32: Option<u32>,
        field_403_u32: Option<u32>,
        field_404_u32: Option<u32>,
        field_405_u32: Option<u32>,
        field_406_u32: Option<u32>,
        field_407_u32: Option<u32>,
        field_408_u32: Option<u32>,
        /// Fields 409-440 — long-tail u32 sequence.
        field_409_u32: Option<u32>,
        field_410_u32: Option<u32>,
        field_411_u32: Option<u32>,
        field_412_u32: Option<u32>,
        field_413_u32: Option<u32>,
        field_414_u32: Option<u32>,
        field_415_u32: Option<u32>,
        field_416_u32: Option<u32>,
        field_417_u32: Option<u32>,
        field_418_u32: Option<u32>,
        field_419_u32: Option<u32>,
        field_420_u32: Option<u32>,
        field_421_u32: Option<u32>,
        field_422_u32: Option<u32>,
        field_423_u32: Option<u32>,
        field_424_u32: Option<u32>,
        field_425_u32: Option<u32>,
        field_426_u32: Option<u32>,
        field_427_u32: Option<u32>,
        field_428_u32: Option<u32>,
        field_429_u32: Option<u32>,
        field_430_u32: Option<u32>,
        field_431_u32: Option<u32>,
        field_432_u32: Option<u32>,
        field_433_u32: Option<u32>,
        field_434_u32: Option<u32>,
        field_435_u32: Option<u32>,
        field_436_u32: Option<u32>,
        field_437_u32: Option<u32>,
        field_438_u32: Option<u32>,
        field_439_u32: Option<u32>,
        field_440_u32: Option<u32>,
        /// Fields 441-472 — long-tail u32 sequence.
        field_441_u32: Option<u32>,
        field_442_u32: Option<u32>,
        field_443_u32: Option<u32>,
        field_444_u32: Option<u32>,
        field_445_u32: Option<u32>,
        field_446_u32: Option<u32>,
        field_447_u32: Option<u32>,
        field_448_u32: Option<u32>,
        field_449_u32: Option<u32>,
        field_450_u32: Option<u32>,
        field_451_u32: Option<u32>,
        field_452_u32: Option<u32>,
        field_453_u32: Option<u32>,
        field_454_u32: Option<u32>,
        field_455_u32: Option<u32>,
        field_456_u32: Option<u32>,
        field_457_u32: Option<u32>,
        field_458_u32: Option<u32>,
        field_459_u32: Option<u32>,
        field_460_u32: Option<u32>,
        field_461_u32: Option<u32>,
        field_462_u32: Option<u32>,
        field_463_u32: Option<u32>,
        field_464_u32: Option<u32>,
        field_465_u32: Option<u32>,
        field_466_u32: Option<u32>,
        field_467_u32: Option<u32>,
        field_468_u32: Option<u32>,
        field_469_u32: Option<u32>,
        field_470_u32: Option<u32>,
        field_471_u32: Option<u32>,
        field_472_u32: Option<u32>,
        /// Fields 473-504 — long-tail u32 sequence.
        field_473_u32: Option<u32>,
        field_474_u32: Option<u32>,
        field_475_u32: Option<u32>,
        field_476_u32: Option<u32>,
        field_477_u32: Option<u32>,
        field_478_u32: Option<u32>,
        field_479_u32: Option<u32>,
        field_480_u32: Option<u32>,
        field_481_u32: Option<u32>,
        field_482_u32: Option<u32>,
        field_483_u32: Option<u32>,
        field_484_u32: Option<u32>,
        field_485_u32: Option<u32>,
        field_486_u32: Option<u32>,
        field_487_u32: Option<u32>,
        field_488_u32: Option<u32>,
        field_489_u32: Option<u32>,
        field_490_u32: Option<u32>,
        field_491_u32: Option<u32>,
        field_492_u32: Option<u32>,
        field_493_u32: Option<u32>,
        field_494_u32: Option<u32>,
        field_495_u32: Option<u32>,
        field_496_u32: Option<u32>,
        field_497_u32: Option<u32>,
        field_498_u32: Option<u32>,
        field_499_u32: Option<u32>,
        field_500_u32: Option<u32>,
        field_501_u32: Option<u32>,
        field_502_u32: Option<u32>,
        field_503_u32: Option<u32>,
        field_504_u32: Option<u32>,
        /// Fields 505-536 — long-tail u32 sequence.
        field_505_u32: Option<u32>,
        field_506_u32: Option<u32>,
        field_507_u32: Option<u32>,
        field_508_u32: Option<u32>,
        field_509_u32: Option<u32>,
        field_510_u32: Option<u32>,
        field_511_u32: Option<u32>,
        field_512_u32: Option<u32>,
        field_513_u32: Option<u32>,
        field_514_u32: Option<u32>,
        field_515_u32: Option<u32>,
        field_516_u32: Option<u32>,
        field_517_u32: Option<u32>,
        field_518_u32: Option<u32>,
        field_519_u32: Option<u32>,
        field_520_u32: Option<u32>,
        field_521_u32: Option<u32>,
        field_522_u32: Option<u32>,
        field_523_u32: Option<u32>,
        field_524_u32: Option<u32>,
        field_525_u32: Option<u32>,
        field_526_u32: Option<u32>,
        field_527_u32: Option<u32>,
        field_528_u32: Option<u32>,
        field_529_u32: Option<u32>,
        field_530_u32: Option<u32>,
        field_531_u32: Option<u32>,
        field_532_u32: Option<u32>,
        field_533_u32: Option<u32>,
        field_534_u32: Option<u32>,
        field_535_u32: Option<u32>,
        field_536_u32: Option<u32>,
        /// Fields 537-600 — long-tail u32 sequence (64-field batch).
        field_537_u32: Option<u32>, field_538_u32: Option<u32>, field_539_u32: Option<u32>, field_540_u32: Option<u32>,
        field_541_u32: Option<u32>, field_542_u32: Option<u32>, field_543_u32: Option<u32>, field_544_u32: Option<u32>,
        field_545_u32: Option<u32>, field_546_u32: Option<u32>, field_547_u32: Option<u32>, field_548_u32: Option<u32>,
        field_549_u32: Option<u32>, field_550_u32: Option<u32>, field_551_u32: Option<u32>, field_552_u32: Option<u32>,
        field_553_u32: Option<u32>, field_554_u32: Option<u32>, field_555_u32: Option<u32>, field_556_u32: Option<u32>,
        field_557_u32: Option<u32>, field_558_u32: Option<u32>, field_559_u32: Option<u32>, field_560_u32: Option<u32>,
        field_561_u32: Option<u32>, field_562_u32: Option<u32>, field_563_u32: Option<u32>, field_564_u32: Option<u32>,
        field_565_u32: Option<u32>, field_566_u32: Option<u32>, field_567_u32: Option<u32>, field_568_u32: Option<u32>,
        field_569_u32: Option<u32>, field_570_u32: Option<u32>, field_571_u32: Option<u32>, field_572_u32: Option<u32>,
        field_573_u32: Option<u32>, field_574_u32: Option<u32>, field_575_u32: Option<u32>, field_576_u32: Option<u32>,
        field_577_u32: Option<u32>, field_578_u32: Option<u32>, field_579_u32: Option<u32>, field_580_u32: Option<u32>,
        field_581_u32: Option<u32>, field_582_u32: Option<u32>, field_583_u32: Option<u32>, field_584_u32: Option<u32>,
        field_585_u32: Option<u32>, field_586_u32: Option<u32>, field_587_u32: Option<u32>, field_588_u32: Option<u32>,
        field_589_u32: Option<u32>, field_590_u32: Option<u32>, field_591_u32: Option<u32>, field_592_u32: Option<u32>,
        field_593_u32: Option<u32>, field_594_u32: Option<u32>, field_595_u32: Option<u32>, field_596_u32: Option<u32>,
        field_597_u32: Option<u32>, field_598_u32: Option<u32>, field_599_u32: Option<u32>, field_600_u32: Option<u32>,
        /// Fields 601-664 — long-tail u32 sequence (64-field batch).
        field_601_u32: Option<u32>, field_602_u32: Option<u32>, field_603_u32: Option<u32>, field_604_u32: Option<u32>,
        field_605_u32: Option<u32>, field_606_u32: Option<u32>, field_607_u32: Option<u32>, field_608_u32: Option<u32>,
        field_609_u32: Option<u32>, field_610_u32: Option<u32>, field_611_u32: Option<u32>, field_612_u32: Option<u32>,
        field_613_u32: Option<u32>, field_614_u32: Option<u32>, field_615_u32: Option<u32>, field_616_u32: Option<u32>,
        field_617_u32: Option<u32>, field_618_u32: Option<u32>, field_619_u32: Option<u32>, field_620_u32: Option<u32>,
        field_621_u32: Option<u32>, field_622_u32: Option<u32>, field_623_u32: Option<u32>, field_624_u32: Option<u32>,
        field_625_u32: Option<u32>, field_626_u32: Option<u32>, field_627_u32: Option<u32>, field_628_u32: Option<u32>,
        field_629_u32: Option<u32>, field_630_u32: Option<u32>, field_631_u32: Option<u32>, field_632_u32: Option<u32>,
        field_633_u32: Option<u32>, field_634_u32: Option<u32>, field_635_u32: Option<u32>, field_636_u32: Option<u32>,
        field_637_u32: Option<u32>, field_638_u32: Option<u32>, field_639_u32: Option<u32>, field_640_u32: Option<u32>,
        field_641_u32: Option<u32>, field_642_u32: Option<u32>, field_643_u32: Option<u32>, field_644_u32: Option<u32>,
        field_645_u32: Option<u32>, field_646_u32: Option<u32>, field_647_u32: Option<u32>, field_648_u32: Option<u32>,
        field_649_u32: Option<u32>, field_650_u32: Option<u32>, field_651_u32: Option<u32>, field_652_u32: Option<u32>,
        field_653_u32: Option<u32>, field_654_u32: Option<u32>, field_655_u32: Option<u32>, field_656_u32: Option<u32>,
        field_657_u32: Option<u32>, field_658_u32: Option<u32>, field_659_u32: Option<u32>, field_660_u32: Option<u32>,
        field_661_u32: Option<u32>, field_662_u32: Option<u32>, field_663_u32: Option<u32>, field_664_u32: Option<u32>,
        /// Alternate-format trigger header (4975 entries fail field 17 but
        /// have a structurally different tail starting with u32 count).
        /// These are likely scene/region gimmicks with named TriggerEntry
        /// CArrays (e.g. "UnnamedTrigger_0", "Platform").
        alt_trigger_count: Option<u32>,
        /// Alternate-format flag byte (typically 0x01).
        alt_trigger_flag: Option<u8>,
        /// Alternate-format trigger name (e.g. "UnnamedTrigger_0").
        alt_trigger_name: Option<CString<'a>>,
        /// Alt-format inner count (e.g. 1 sub-trigger).
        alt_inner_count: Option<u32>,
        /// Alt-format inner name (e.g. "hide_bush_crouch").
        alt_inner_name: Option<CString<'a>>,
        /// Alt-format inner flag/value u32.
        alt_inner_flag: Option<u32>,
        /// Alt-format body u32 sequence (64 fields after inner header).
        alt_body_001: Option<u32>, alt_body_002: Option<u32>, alt_body_003: Option<u32>, alt_body_004: Option<u32>,
        alt_body_005: Option<u32>, alt_body_006: Option<u32>, alt_body_007: Option<u32>, alt_body_008: Option<u32>,
        alt_body_009: Option<u32>, alt_body_010: Option<u32>, alt_body_011: Option<u32>, alt_body_012: Option<u32>,
        alt_body_013: Option<u32>, alt_body_014: Option<u32>, alt_body_015: Option<u32>, alt_body_016: Option<u32>,
        alt_body_017: Option<u32>, alt_body_018: Option<u32>, alt_body_019: Option<u32>, alt_body_020: Option<u32>,
        alt_body_021: Option<u32>, alt_body_022: Option<u32>, alt_body_023: Option<u32>, alt_body_024: Option<u32>,
        alt_body_025: Option<u32>, alt_body_026: Option<u32>, alt_body_027: Option<u32>, alt_body_028: Option<u32>,
        alt_body_029: Option<u32>, alt_body_030: Option<u32>, alt_body_031: Option<u32>, alt_body_032: Option<u32>,
        alt_body_033: Option<u32>, alt_body_034: Option<u32>, alt_body_035: Option<u32>, alt_body_036: Option<u32>,
        alt_body_037: Option<u32>, alt_body_038: Option<u32>, alt_body_039: Option<u32>, alt_body_040: Option<u32>,
        alt_body_041: Option<u32>, alt_body_042: Option<u32>, alt_body_043: Option<u32>, alt_body_044: Option<u32>,
        alt_body_045: Option<u32>, alt_body_046: Option<u32>, alt_body_047: Option<u32>, alt_body_048: Option<u32>,
        alt_body_049: Option<u32>, alt_body_050: Option<u32>, alt_body_051: Option<u32>, alt_body_052: Option<u32>,
        alt_body_053: Option<u32>, alt_body_054: Option<u32>, alt_body_055: Option<u32>, alt_body_056: Option<u32>,
        alt_body_057: Option<u32>, alt_body_058: Option<u32>, alt_body_059: Option<u32>, alt_body_060: Option<u32>,
        alt_body_061: Option<u32>, alt_body_062: Option<u32>, alt_body_063: Option<u32>, alt_body_064: Option<u32>,
        alt_body_065: Option<u32>, alt_body_066: Option<u32>, alt_body_067: Option<u32>, alt_body_068: Option<u32>,
        alt_body_069: Option<u32>, alt_body_070: Option<u32>, alt_body_071: Option<u32>, alt_body_072: Option<u32>,
        alt_body_073: Option<u32>, alt_body_074: Option<u32>, alt_body_075: Option<u32>, alt_body_076: Option<u32>,
        alt_body_077: Option<u32>, alt_body_078: Option<u32>, alt_body_079: Option<u32>, alt_body_080: Option<u32>,
        alt_body_081: Option<u32>, alt_body_082: Option<u32>, alt_body_083: Option<u32>, alt_body_084: Option<u32>,
        alt_body_085: Option<u32>, alt_body_086: Option<u32>, alt_body_087: Option<u32>, alt_body_088: Option<u32>,
        alt_body_089: Option<u32>, alt_body_090: Option<u32>, alt_body_091: Option<u32>, alt_body_092: Option<u32>,
        alt_body_093: Option<u32>, alt_body_094: Option<u32>, alt_body_095: Option<u32>, alt_body_096: Option<u32>,
        alt_body_097: Option<u32>, alt_body_098: Option<u32>, alt_body_099: Option<u32>, alt_body_100: Option<u32>,
        alt_body_101: Option<u32>, alt_body_102: Option<u32>, alt_body_103: Option<u32>, alt_body_104: Option<u32>,
        alt_body_105: Option<u32>, alt_body_106: Option<u32>, alt_body_107: Option<u32>, alt_body_108: Option<u32>,
        alt_body_109: Option<u32>, alt_body_110: Option<u32>, alt_body_111: Option<u32>, alt_body_112: Option<u32>,
        alt_body_113: Option<u32>, alt_body_114: Option<u32>, alt_body_115: Option<u32>, alt_body_116: Option<u32>,
        alt_body_117: Option<u32>, alt_body_118: Option<u32>, alt_body_119: Option<u32>, alt_body_120: Option<u32>,
        alt_body_121: Option<u32>, alt_body_122: Option<u32>, alt_body_123: Option<u32>, alt_body_124: Option<u32>,
        alt_body_125: Option<u32>, alt_body_126: Option<u32>, alt_body_127: Option<u32>, alt_body_128: Option<u32>,
        alt_body_129: Option<u32>, alt_body_130: Option<u32>, alt_body_131: Option<u32>, alt_body_132: Option<u32>,
        alt_body_133: Option<u32>, alt_body_134: Option<u32>, alt_body_135: Option<u32>, alt_body_136: Option<u32>,
        alt_body_137: Option<u32>, alt_body_138: Option<u32>, alt_body_139: Option<u32>, alt_body_140: Option<u32>,
        alt_body_141: Option<u32>, alt_body_142: Option<u32>, alt_body_143: Option<u32>, alt_body_144: Option<u32>,
        alt_body_145: Option<u32>, alt_body_146: Option<u32>, alt_body_147: Option<u32>, alt_body_148: Option<u32>,
        alt_body_149: Option<u32>, alt_body_150: Option<u32>, alt_body_151: Option<u32>, alt_body_152: Option<u32>,
        alt_body_153: Option<u32>, alt_body_154: Option<u32>, alt_body_155: Option<u32>, alt_body_156: Option<u32>,
        alt_body_157: Option<u32>, alt_body_158: Option<u32>, alt_body_159: Option<u32>, alt_body_160: Option<u32>,
        alt_body_161: Option<u32>, alt_body_162: Option<u32>, alt_body_163: Option<u32>, alt_body_164: Option<u32>,
        alt_body_165: Option<u32>, alt_body_166: Option<u32>, alt_body_167: Option<u32>, alt_body_168: Option<u32>,
        alt_body_169: Option<u32>, alt_body_170: Option<u32>, alt_body_171: Option<u32>, alt_body_172: Option<u32>,
        alt_body_173: Option<u32>, alt_body_174: Option<u32>, alt_body_175: Option<u32>, alt_body_176: Option<u32>,
        alt_body_177: Option<u32>, alt_body_178: Option<u32>, alt_body_179: Option<u32>, alt_body_180: Option<u32>,
        alt_body_181: Option<u32>, alt_body_182: Option<u32>, alt_body_183: Option<u32>, alt_body_184: Option<u32>,
        alt_body_185: Option<u32>, alt_body_186: Option<u32>, alt_body_187: Option<u32>, alt_body_188: Option<u32>,
        alt_body_189: Option<u32>, alt_body_190: Option<u32>, alt_body_191: Option<u32>, alt_body_192: Option<u32>,
        alt_body_193: Option<u32>, alt_body_194: Option<u32>, alt_body_195: Option<u32>, alt_body_196: Option<u32>,
        alt_body_197: Option<u32>, alt_body_198: Option<u32>, alt_body_199: Option<u32>, alt_body_200: Option<u32>,
        alt_body_201: Option<u32>, alt_body_202: Option<u32>, alt_body_203: Option<u32>, alt_body_204: Option<u32>,
        alt_body_205: Option<u32>, alt_body_206: Option<u32>, alt_body_207: Option<u32>, alt_body_208: Option<u32>,
        alt_body_209: Option<u32>, alt_body_210: Option<u32>, alt_body_211: Option<u32>, alt_body_212: Option<u32>,
        alt_body_213: Option<u32>, alt_body_214: Option<u32>, alt_body_215: Option<u32>, alt_body_216: Option<u32>,
        alt_body_217: Option<u32>, alt_body_218: Option<u32>, alt_body_219: Option<u32>, alt_body_220: Option<u32>,
        alt_body_221: Option<u32>, alt_body_222: Option<u32>, alt_body_223: Option<u32>, alt_body_224: Option<u32>,
        alt_body_225: Option<u32>, alt_body_226: Option<u32>, alt_body_227: Option<u32>, alt_body_228: Option<u32>,
        alt_body_229: Option<u32>, alt_body_230: Option<u32>, alt_body_231: Option<u32>, alt_body_232: Option<u32>,
        alt_body_233: Option<u32>, alt_body_234: Option<u32>, alt_body_235: Option<u32>, alt_body_236: Option<u32>,
        alt_body_237: Option<u32>, alt_body_238: Option<u32>, alt_body_239: Option<u32>, alt_body_240: Option<u32>,
        alt_body_241: Option<u32>, alt_body_242: Option<u32>, alt_body_243: Option<u32>, alt_body_244: Option<u32>,
        alt_body_245: Option<u32>, alt_body_246: Option<u32>, alt_body_247: Option<u32>, alt_body_248: Option<u32>,
        alt_body_249: Option<u32>, alt_body_250: Option<u32>, alt_body_251: Option<u32>, alt_body_252: Option<u32>,
        alt_body_253: Option<u32>, alt_body_254: Option<u32>, alt_body_255: Option<u32>, alt_body_256: Option<u32>,
        alt_body_257: Option<u32>, alt_body_258: Option<u32>, alt_body_259: Option<u32>, alt_body_260: Option<u32>,
        alt_body_261: Option<u32>, alt_body_262: Option<u32>, alt_body_263: Option<u32>, alt_body_264: Option<u32>,
        alt_body_265: Option<u32>, alt_body_266: Option<u32>, alt_body_267: Option<u32>, alt_body_268: Option<u32>,
        alt_body_269: Option<u32>, alt_body_270: Option<u32>, alt_body_271: Option<u32>, alt_body_272: Option<u32>,
        alt_body_273: Option<u32>, alt_body_274: Option<u32>, alt_body_275: Option<u32>, alt_body_276: Option<u32>,
        alt_body_277: Option<u32>, alt_body_278: Option<u32>, alt_body_279: Option<u32>, alt_body_280: Option<u32>,
        alt_body_281: Option<u32>, alt_body_282: Option<u32>, alt_body_283: Option<u32>, alt_body_284: Option<u32>,
        alt_body_285: Option<u32>, alt_body_286: Option<u32>, alt_body_287: Option<u32>, alt_body_288: Option<u32>,
        alt_body_289: Option<u32>, alt_body_290: Option<u32>, alt_body_291: Option<u32>, alt_body_292: Option<u32>,
        alt_body_293: Option<u32>, alt_body_294: Option<u32>, alt_body_295: Option<u32>, alt_body_296: Option<u32>,
        alt_body_297: Option<u32>, alt_body_298: Option<u32>, alt_body_299: Option<u32>, alt_body_300: Option<u32>,
        alt_body_301: Option<u32>, alt_body_302: Option<u32>, alt_body_303: Option<u32>, alt_body_304: Option<u32>,
        alt_body_305: Option<u32>, alt_body_306: Option<u32>, alt_body_307: Option<u32>, alt_body_308: Option<u32>,
        alt_body_309: Option<u32>, alt_body_310: Option<u32>, alt_body_311: Option<u32>, alt_body_312: Option<u32>,
        alt_body_313: Option<u32>, alt_body_314: Option<u32>, alt_body_315: Option<u32>, alt_body_316: Option<u32>,
        alt_body_317: Option<u32>, alt_body_318: Option<u32>, alt_body_319: Option<u32>, alt_body_320: Option<u32>,
        /// Fields 665-728 — long-tail u32 sequence (64-field batch).
        field_665_u32: Option<u32>, field_666_u32: Option<u32>, field_667_u32: Option<u32>, field_668_u32: Option<u32>,
        field_669_u32: Option<u32>, field_670_u32: Option<u32>, field_671_u32: Option<u32>, field_672_u32: Option<u32>,
        field_673_u32: Option<u32>, field_674_u32: Option<u32>, field_675_u32: Option<u32>, field_676_u32: Option<u32>,
        field_677_u32: Option<u32>, field_678_u32: Option<u32>, field_679_u32: Option<u32>, field_680_u32: Option<u32>,
        field_681_u32: Option<u32>, field_682_u32: Option<u32>, field_683_u32: Option<u32>, field_684_u32: Option<u32>,
        field_685_u32: Option<u32>, field_686_u32: Option<u32>, field_687_u32: Option<u32>, field_688_u32: Option<u32>,
        field_689_u32: Option<u32>, field_690_u32: Option<u32>, field_691_u32: Option<u32>, field_692_u32: Option<u32>,
        field_693_u32: Option<u32>, field_694_u32: Option<u32>, field_695_u32: Option<u32>, field_696_u32: Option<u32>,
        field_697_u32: Option<u32>, field_698_u32: Option<u32>, field_699_u32: Option<u32>, field_700_u32: Option<u32>,
        field_701_u32: Option<u32>, field_702_u32: Option<u32>, field_703_u32: Option<u32>, field_704_u32: Option<u32>,
        field_705_u32: Option<u32>, field_706_u32: Option<u32>, field_707_u32: Option<u32>, field_708_u32: Option<u32>,
        field_709_u32: Option<u32>, field_710_u32: Option<u32>, field_711_u32: Option<u32>, field_712_u32: Option<u32>,
        field_713_u32: Option<u32>, field_714_u32: Option<u32>, field_715_u32: Option<u32>, field_716_u32: Option<u32>,
        field_717_u32: Option<u32>, field_718_u32: Option<u32>, field_719_u32: Option<u32>, field_720_u32: Option<u32>,
        field_721_u32: Option<u32>, field_722_u32: Option<u32>, field_723_u32: Option<u32>, field_724_u32: Option<u32>,
        field_725_u32: Option<u32>, field_726_u32: Option<u32>, field_727_u32: Option<u32>, field_728_u32: Option<u32>,
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
                let field_108_u32 = if field_107_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_109_u32 = if field_108_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_110_u32 = if field_109_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_111_u32 = if field_110_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_112_u32 = if field_111_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_113_u32 = if field_112_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_114_u32 = if field_113_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_115_u32 = if field_114_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_116_u32 = if field_115_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_117_u32 = if field_116_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_118_u32 = if field_117_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_119_u32 = if field_118_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_120_u32 = if field_119_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_121_u32 = if field_120_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_122_u32 = if field_121_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_123_u32 = if field_122_u32.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                macro_rules! read_u32_chained {
                    ($prev:ident) => {{
                        if $prev.is_some() && probe + 4 <= entry_end {
                            let pre_ = probe;
                            match u32::read_from(data, &mut probe) {
                                Ok(v) => Some(v), _ => { probe = pre_; None }
                            }
                        } else { None }
                    }};
                }
                let field_124_u32 = read_u32_chained!(field_123_u32);
                let field_125_u32 = read_u32_chained!(field_124_u32);
                let field_126_u32 = read_u32_chained!(field_125_u32);
                let field_127_u32 = read_u32_chained!(field_126_u32);
                let field_128_u32 = read_u32_chained!(field_127_u32);
                let field_129_u32 = read_u32_chained!(field_128_u32);
                let field_130_u32 = read_u32_chained!(field_129_u32);
                let field_131_u32 = read_u32_chained!(field_130_u32);
                let field_132_u32 = read_u32_chained!(field_131_u32);
                let field_133_u32 = read_u32_chained!(field_132_u32);
                let field_134_u32 = read_u32_chained!(field_133_u32);
                let field_135_u32 = read_u32_chained!(field_134_u32);
                let field_136_u32 = read_u32_chained!(field_135_u32);
                let field_137_u32 = read_u32_chained!(field_136_u32);
                let field_138_u32 = read_u32_chained!(field_137_u32);
                let field_139_u32 = read_u32_chained!(field_138_u32);
                let field_140_u32 = read_u32_chained!(field_139_u32);
                let field_141_u32 = read_u32_chained!(field_140_u32);
                let field_142_u32 = read_u32_chained!(field_141_u32);
                let field_143_u32 = read_u32_chained!(field_142_u32);
                let field_144_u32 = read_u32_chained!(field_143_u32);
                let field_145_u32 = read_u32_chained!(field_144_u32);
                let field_146_u32 = read_u32_chained!(field_145_u32);
                let field_147_u32 = read_u32_chained!(field_146_u32);
                let field_148_u32 = read_u32_chained!(field_147_u32);
                let field_149_u32 = read_u32_chained!(field_148_u32);
                let field_150_u32 = read_u32_chained!(field_149_u32);
                let field_151_u32 = read_u32_chained!(field_150_u32);
                let field_152_u32 = read_u32_chained!(field_151_u32);
                let field_153_u32 = read_u32_chained!(field_152_u32);
                let field_154_u32 = read_u32_chained!(field_153_u32);
                let field_155_u32 = read_u32_chained!(field_154_u32);
                let field_156_u32 = read_u32_chained!(field_155_u32);
                let field_157_u32 = read_u32_chained!(field_156_u32);
                let field_158_u32 = read_u32_chained!(field_157_u32);
                let field_159_u32 = read_u32_chained!(field_158_u32);
                let field_160_u32 = read_u32_chained!(field_159_u32);
                let field_161_u32 = read_u32_chained!(field_160_u32);
                let field_162_u32 = read_u32_chained!(field_161_u32);
                let field_163_u32 = read_u32_chained!(field_162_u32);
                let field_164_u32 = read_u32_chained!(field_163_u32);
                let field_165_u32 = read_u32_chained!(field_164_u32);
                let field_166_u32 = read_u32_chained!(field_165_u32);
                let field_167_u32 = read_u32_chained!(field_166_u32);
                let field_168_u32 = read_u32_chained!(field_167_u32);
                let field_169_u32 = read_u32_chained!(field_168_u32);
                let field_170_u32 = read_u32_chained!(field_169_u32);
                let field_171_u32 = read_u32_chained!(field_170_u32);
                let field_172_u32 = read_u32_chained!(field_171_u32);
                let field_173_u32 = read_u32_chained!(field_172_u32);
                let field_174_u32 = read_u32_chained!(field_173_u32);
                let field_175_u32 = read_u32_chained!(field_174_u32);
                let field_176_u32 = read_u32_chained!(field_175_u32);
                let field_177_u32 = read_u32_chained!(field_176_u32);
                let field_178_u32 = read_u32_chained!(field_177_u32);
                let field_179_u32 = read_u32_chained!(field_178_u32);
                let field_180_u32 = read_u32_chained!(field_179_u32);
                let field_181_u32 = read_u32_chained!(field_180_u32);
                let field_182_u32 = read_u32_chained!(field_181_u32);
                let field_183_u32 = read_u32_chained!(field_182_u32);
                let field_184_u32 = read_u32_chained!(field_183_u32);
                let field_185_u32 = read_u32_chained!(field_184_u32);
                let field_186_u32 = read_u32_chained!(field_185_u32);
                let field_187_u32 = read_u32_chained!(field_186_u32);
                let field_188_u32 = read_u32_chained!(field_187_u32);
                let field_189_u32 = read_u32_chained!(field_188_u32);
                let field_190_u32 = read_u32_chained!(field_189_u32);
                let field_191_u32 = read_u32_chained!(field_190_u32);
                let field_192_u32 = read_u32_chained!(field_191_u32);
                let field_193_u32 = read_u32_chained!(field_192_u32);
                let field_194_u32 = read_u32_chained!(field_193_u32);
                let field_195_u32 = read_u32_chained!(field_194_u32);
                let field_196_u32 = read_u32_chained!(field_195_u32);
                let field_197_u32 = read_u32_chained!(field_196_u32);
                let field_198_u32 = read_u32_chained!(field_197_u32);
                let field_199_u32 = read_u32_chained!(field_198_u32);
                let field_200_u32 = read_u32_chained!(field_199_u32);
                let field_201_u32 = read_u32_chained!(field_200_u32);
                let field_202_u32 = read_u32_chained!(field_201_u32);
                let field_203_u32 = read_u32_chained!(field_202_u32);
                let field_204_u32 = read_u32_chained!(field_203_u32);
                let field_205_u32 = read_u32_chained!(field_204_u32);
                let field_206_u32 = read_u32_chained!(field_205_u32);
                let field_207_u32 = read_u32_chained!(field_206_u32);
                let field_208_u32 = read_u32_chained!(field_207_u32);
                let field_209_u32 = read_u32_chained!(field_208_u32);
                let field_210_u32 = read_u32_chained!(field_209_u32);
                let field_211_u32 = read_u32_chained!(field_210_u32);
                let field_212_u32 = read_u32_chained!(field_211_u32);
                let field_213_u32 = read_u32_chained!(field_212_u32);
                let field_214_u32 = read_u32_chained!(field_213_u32);
                let field_215_u32 = read_u32_chained!(field_214_u32);
                let field_216_u32 = read_u32_chained!(field_215_u32);
                let field_217_u32 = read_u32_chained!(field_216_u32);
                let field_218_u32 = read_u32_chained!(field_217_u32);
                let field_219_u32 = read_u32_chained!(field_218_u32);
                let field_220_u32 = read_u32_chained!(field_219_u32);
                let field_221_u32 = read_u32_chained!(field_220_u32);
                let field_222_u32 = read_u32_chained!(field_221_u32);
                let field_223_u32 = read_u32_chained!(field_222_u32);
                let field_224_u32 = read_u32_chained!(field_223_u32);
                let field_225_u32 = read_u32_chained!(field_224_u32);
                let field_226_u32 = read_u32_chained!(field_225_u32);
                let field_227_u32 = read_u32_chained!(field_226_u32);
                let field_228_u32 = read_u32_chained!(field_227_u32);
                let field_229_u32 = read_u32_chained!(field_228_u32);
                let field_230_u32 = read_u32_chained!(field_229_u32);
                let field_231_u32 = read_u32_chained!(field_230_u32);
                let field_232_u32 = read_u32_chained!(field_231_u32);
                let field_233_u32 = read_u32_chained!(field_232_u32);
                let field_234_u32 = read_u32_chained!(field_233_u32);
                let field_235_u32 = read_u32_chained!(field_234_u32);
                let field_236_u32 = read_u32_chained!(field_235_u32);
                let field_237_u32 = read_u32_chained!(field_236_u32);
                let field_238_u32 = read_u32_chained!(field_237_u32);
                let field_239_u32 = read_u32_chained!(field_238_u32);
                let field_240_u32 = read_u32_chained!(field_239_u32);
                let field_241_u32 = read_u32_chained!(field_240_u32);
                let field_242_u32 = read_u32_chained!(field_241_u32);
                let field_243_u32 = read_u32_chained!(field_242_u32);
                let field_244_u32 = read_u32_chained!(field_243_u32);
                let field_245_u32 = read_u32_chained!(field_244_u32);
                let field_246_u32 = read_u32_chained!(field_245_u32);
                let field_247_u32 = read_u32_chained!(field_246_u32);
                let field_248_u32 = read_u32_chained!(field_247_u32);
                let field_249_u32 = read_u32_chained!(field_248_u32);
                let field_250_u32 = read_u32_chained!(field_249_u32);
                let field_251_u32 = read_u32_chained!(field_250_u32);
                let field_252_u32 = read_u32_chained!(field_251_u32);
                let field_253_u32 = read_u32_chained!(field_252_u32);
                let field_254_u32 = read_u32_chained!(field_253_u32);
                let field_255_u32 = read_u32_chained!(field_254_u32);
                let field_256_u32 = read_u32_chained!(field_255_u32);
                let field_257_u32 = read_u32_chained!(field_256_u32);
                let field_258_u32 = read_u32_chained!(field_257_u32);
                let field_259_u32 = read_u32_chained!(field_258_u32);
                let field_260_u32 = read_u32_chained!(field_259_u32);
                let field_261_u32 = read_u32_chained!(field_260_u32);
                let field_262_u32 = read_u32_chained!(field_261_u32);
                let field_263_u32 = read_u32_chained!(field_262_u32);
                let field_264_u32 = read_u32_chained!(field_263_u32);
                let field_265_u32 = read_u32_chained!(field_264_u32);
                let field_266_u32 = read_u32_chained!(field_265_u32);
                let field_267_u32 = read_u32_chained!(field_266_u32);
                let field_268_u32 = read_u32_chained!(field_267_u32);
                let field_269_u32 = read_u32_chained!(field_268_u32);
                let field_270_u32 = read_u32_chained!(field_269_u32);
                let field_271_u32 = read_u32_chained!(field_270_u32);
                let field_272_u32 = read_u32_chained!(field_271_u32);
                let field_273_u32 = read_u32_chained!(field_272_u32);
                let field_274_u32 = read_u32_chained!(field_273_u32);
                let field_275_u32 = read_u32_chained!(field_274_u32);
                let field_276_u32 = read_u32_chained!(field_275_u32);
                let field_277_u32 = read_u32_chained!(field_276_u32);
                let field_278_u32 = read_u32_chained!(field_277_u32);
                let field_279_u32 = read_u32_chained!(field_278_u32);
                let field_280_u32 = read_u32_chained!(field_279_u32);
                let field_281_u32 = read_u32_chained!(field_280_u32);
                let field_282_u32 = read_u32_chained!(field_281_u32);
                let field_283_u32 = read_u32_chained!(field_282_u32);
                let field_284_u32 = read_u32_chained!(field_283_u32);
                let field_285_u32 = read_u32_chained!(field_284_u32);
                let field_286_u32 = read_u32_chained!(field_285_u32);
                let field_287_u32 = read_u32_chained!(field_286_u32);
                let field_288_u32 = read_u32_chained!(field_287_u32);
                let field_289_u32 = read_u32_chained!(field_288_u32);
                let field_290_u32 = read_u32_chained!(field_289_u32);
                let field_291_u32 = read_u32_chained!(field_290_u32);
                let field_292_u32 = read_u32_chained!(field_291_u32);
                let field_293_u32 = read_u32_chained!(field_292_u32);
                let field_294_u32 = read_u32_chained!(field_293_u32);
                let field_295_u32 = read_u32_chained!(field_294_u32);
                let field_296_u32 = read_u32_chained!(field_295_u32);
                let field_297_u32 = read_u32_chained!(field_296_u32);
                let field_298_u32 = read_u32_chained!(field_297_u32);
                let field_299_u32 = read_u32_chained!(field_298_u32);
                let field_300_u32 = read_u32_chained!(field_299_u32);
                let field_301_u32 = read_u32_chained!(field_300_u32);
                let field_302_u32 = read_u32_chained!(field_301_u32);
                let field_303_u32 = read_u32_chained!(field_302_u32);
                let field_304_u32 = read_u32_chained!(field_303_u32);
                let field_305_u32 = read_u32_chained!(field_304_u32);
                let field_306_u32 = read_u32_chained!(field_305_u32);
                let field_307_u32 = read_u32_chained!(field_306_u32);
                let field_308_u32 = read_u32_chained!(field_307_u32);
                let field_309_u32 = read_u32_chained!(field_308_u32);
                let field_310_u32 = read_u32_chained!(field_309_u32);
                let field_311_u32 = read_u32_chained!(field_310_u32);
                let field_312_u32 = read_u32_chained!(field_311_u32);
                let field_313_u32 = read_u32_chained!(field_312_u32);
                let field_314_u32 = read_u32_chained!(field_313_u32);
                let field_315_u32 = read_u32_chained!(field_314_u32);
                let field_316_u32 = read_u32_chained!(field_315_u32);
                let field_317_u32 = read_u32_chained!(field_316_u32);
                let field_318_u32 = read_u32_chained!(field_317_u32);
                let field_319_u32 = read_u32_chained!(field_318_u32);
                let field_320_u32 = read_u32_chained!(field_319_u32);
                let field_321_u32 = read_u32_chained!(field_320_u32);
                let field_322_u32 = read_u32_chained!(field_321_u32);
                let field_323_u32 = read_u32_chained!(field_322_u32);
                let field_324_u32 = read_u32_chained!(field_323_u32);
                let field_325_u32 = read_u32_chained!(field_324_u32);
                let field_326_u32 = read_u32_chained!(field_325_u32);
                let field_327_u32 = read_u32_chained!(field_326_u32);
                let field_328_u32 = read_u32_chained!(field_327_u32);
                let field_329_u32 = read_u32_chained!(field_328_u32);
                let field_330_u32 = read_u32_chained!(field_329_u32);
                let field_331_u32 = read_u32_chained!(field_330_u32);
                let field_332_u32 = read_u32_chained!(field_331_u32);
                let field_333_u32 = read_u32_chained!(field_332_u32);
                let field_334_u32 = read_u32_chained!(field_333_u32);
                let field_335_u32 = read_u32_chained!(field_334_u32);
                let field_336_u32 = read_u32_chained!(field_335_u32);
                let field_337_u32 = read_u32_chained!(field_336_u32);
                let field_338_u32 = read_u32_chained!(field_337_u32);
                let field_339_u32 = read_u32_chained!(field_338_u32);
                let field_340_u32 = read_u32_chained!(field_339_u32);
                let field_341_u32 = read_u32_chained!(field_340_u32);
                let field_342_u32_count = read_u32_chained!(field_341_u32);
                let field_343_u8_flag = if field_342_u32_count.is_some() && probe + 1 <= entry_end {
                    let pre_ = probe;
                    match u8::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_344_u32 = if field_343_u8_flag.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let field_345_u32 = read_u32_chained!(field_344_u32);
                let field_346_u32 = read_u32_chained!(field_345_u32);
                let field_347_u32 = read_u32_chained!(field_346_u32);
                let field_348_u32 = read_u32_chained!(field_347_u32);
                let field_349_u32 = read_u32_chained!(field_348_u32);
                let field_350_u32 = read_u32_chained!(field_349_u32);
                let field_351_u32 = read_u32_chained!(field_350_u32);
                let field_352_u32 = read_u32_chained!(field_351_u32);
                let field_353_u32 = read_u32_chained!(field_352_u32);
                let field_354_u32 = read_u32_chained!(field_353_u32);
                let field_355_u32 = read_u32_chained!(field_354_u32);
                let field_356_u32 = read_u32_chained!(field_355_u32);
                let field_357_u32 = read_u32_chained!(field_356_u32);
                let field_358_u32 = read_u32_chained!(field_357_u32);
                let field_359_u32 = read_u32_chained!(field_358_u32);
                let field_360_u32 = read_u32_chained!(field_359_u32);
                let field_361_u32 = read_u32_chained!(field_360_u32);
                let field_362_u32 = read_u32_chained!(field_361_u32);
                let field_363_u32 = read_u32_chained!(field_362_u32);
                let field_364_u32 = read_u32_chained!(field_363_u32);
                let field_365_u32 = read_u32_chained!(field_364_u32);
                let field_366_u32 = read_u32_chained!(field_365_u32);
                let field_367_u32 = read_u32_chained!(field_366_u32);
                let field_368_u32 = read_u32_chained!(field_367_u32);
                let field_369_u32 = read_u32_chained!(field_368_u32);
                let field_370_u32 = read_u32_chained!(field_369_u32);
                let field_371_u32 = read_u32_chained!(field_370_u32);
                let field_372_u32 = read_u32_chained!(field_371_u32);
                let field_373_u32 = read_u32_chained!(field_372_u32);
                let field_374_u32 = read_u32_chained!(field_373_u32);
                let field_375_u32 = read_u32_chained!(field_374_u32);
                let field_376_u32 = read_u32_chained!(field_375_u32);
                let field_377_u32 = read_u32_chained!(field_376_u32);
                let field_378_u32 = read_u32_chained!(field_377_u32);
                let field_379_u32 = read_u32_chained!(field_378_u32);
                let field_380_u32 = read_u32_chained!(field_379_u32);
                let field_381_u32 = read_u32_chained!(field_380_u32);
                let field_382_u32 = read_u32_chained!(field_381_u32);
                let field_383_u32 = read_u32_chained!(field_382_u32);
                let field_384_u32 = read_u32_chained!(field_383_u32);
                let field_385_u32 = read_u32_chained!(field_384_u32);
                let field_386_u32 = read_u32_chained!(field_385_u32);
                let field_387_u32 = read_u32_chained!(field_386_u32);
                let field_388_u32 = read_u32_chained!(field_387_u32);
                let field_389_u32 = read_u32_chained!(field_388_u32);
                let field_390_u32 = read_u32_chained!(field_389_u32);
                let field_391_u32 = read_u32_chained!(field_390_u32);
                let field_392_u32 = read_u32_chained!(field_391_u32);
                let field_393_u32 = read_u32_chained!(field_392_u32);
                let field_394_u32 = read_u32_chained!(field_393_u32);
                let field_395_u32 = read_u32_chained!(field_394_u32);
                let field_396_u32 = read_u32_chained!(field_395_u32);
                let field_397_u32 = read_u32_chained!(field_396_u32);
                let field_398_u32 = read_u32_chained!(field_397_u32);
                let field_399_u32 = read_u32_chained!(field_398_u32);
                let field_400_u32 = read_u32_chained!(field_399_u32);
                let field_401_u32 = read_u32_chained!(field_400_u32);
                let field_402_u32 = read_u32_chained!(field_401_u32);
                let field_403_u32 = read_u32_chained!(field_402_u32);
                let field_404_u32 = read_u32_chained!(field_403_u32);
                let field_405_u32 = read_u32_chained!(field_404_u32);
                let field_406_u32 = read_u32_chained!(field_405_u32);
                let field_407_u32 = read_u32_chained!(field_406_u32);
                let field_408_u32 = read_u32_chained!(field_407_u32);
                let field_409_u32 = read_u32_chained!(field_408_u32);
                let field_410_u32 = read_u32_chained!(field_409_u32);
                let field_411_u32 = read_u32_chained!(field_410_u32);
                let field_412_u32 = read_u32_chained!(field_411_u32);
                let field_413_u32 = read_u32_chained!(field_412_u32);
                let field_414_u32 = read_u32_chained!(field_413_u32);
                let field_415_u32 = read_u32_chained!(field_414_u32);
                let field_416_u32 = read_u32_chained!(field_415_u32);
                let field_417_u32 = read_u32_chained!(field_416_u32);
                let field_418_u32 = read_u32_chained!(field_417_u32);
                let field_419_u32 = read_u32_chained!(field_418_u32);
                let field_420_u32 = read_u32_chained!(field_419_u32);
                let field_421_u32 = read_u32_chained!(field_420_u32);
                let field_422_u32 = read_u32_chained!(field_421_u32);
                let field_423_u32 = read_u32_chained!(field_422_u32);
                let field_424_u32 = read_u32_chained!(field_423_u32);
                let field_425_u32 = read_u32_chained!(field_424_u32);
                let field_426_u32 = read_u32_chained!(field_425_u32);
                let field_427_u32 = read_u32_chained!(field_426_u32);
                let field_428_u32 = read_u32_chained!(field_427_u32);
                let field_429_u32 = read_u32_chained!(field_428_u32);
                let field_430_u32 = read_u32_chained!(field_429_u32);
                let field_431_u32 = read_u32_chained!(field_430_u32);
                let field_432_u32 = read_u32_chained!(field_431_u32);
                let field_433_u32 = read_u32_chained!(field_432_u32);
                let field_434_u32 = read_u32_chained!(field_433_u32);
                let field_435_u32 = read_u32_chained!(field_434_u32);
                let field_436_u32 = read_u32_chained!(field_435_u32);
                let field_437_u32 = read_u32_chained!(field_436_u32);
                let field_438_u32 = read_u32_chained!(field_437_u32);
                let field_439_u32 = read_u32_chained!(field_438_u32);
                let field_440_u32 = read_u32_chained!(field_439_u32);
                let field_441_u32 = read_u32_chained!(field_440_u32);
                let field_442_u32 = read_u32_chained!(field_441_u32);
                let field_443_u32 = read_u32_chained!(field_442_u32);
                let field_444_u32 = read_u32_chained!(field_443_u32);
                let field_445_u32 = read_u32_chained!(field_444_u32);
                let field_446_u32 = read_u32_chained!(field_445_u32);
                let field_447_u32 = read_u32_chained!(field_446_u32);
                let field_448_u32 = read_u32_chained!(field_447_u32);
                let field_449_u32 = read_u32_chained!(field_448_u32);
                let field_450_u32 = read_u32_chained!(field_449_u32);
                let field_451_u32 = read_u32_chained!(field_450_u32);
                let field_452_u32 = read_u32_chained!(field_451_u32);
                let field_453_u32 = read_u32_chained!(field_452_u32);
                let field_454_u32 = read_u32_chained!(field_453_u32);
                let field_455_u32 = read_u32_chained!(field_454_u32);
                let field_456_u32 = read_u32_chained!(field_455_u32);
                let field_457_u32 = read_u32_chained!(field_456_u32);
                let field_458_u32 = read_u32_chained!(field_457_u32);
                let field_459_u32 = read_u32_chained!(field_458_u32);
                let field_460_u32 = read_u32_chained!(field_459_u32);
                let field_461_u32 = read_u32_chained!(field_460_u32);
                let field_462_u32 = read_u32_chained!(field_461_u32);
                let field_463_u32 = read_u32_chained!(field_462_u32);
                let field_464_u32 = read_u32_chained!(field_463_u32);
                let field_465_u32 = read_u32_chained!(field_464_u32);
                let field_466_u32 = read_u32_chained!(field_465_u32);
                let field_467_u32 = read_u32_chained!(field_466_u32);
                let field_468_u32 = read_u32_chained!(field_467_u32);
                let field_469_u32 = read_u32_chained!(field_468_u32);
                let field_470_u32 = read_u32_chained!(field_469_u32);
                let field_471_u32 = read_u32_chained!(field_470_u32);
                let field_472_u32 = read_u32_chained!(field_471_u32);
                let field_473_u32 = read_u32_chained!(field_472_u32);
                let field_474_u32 = read_u32_chained!(field_473_u32);
                let field_475_u32 = read_u32_chained!(field_474_u32);
                let field_476_u32 = read_u32_chained!(field_475_u32);
                let field_477_u32 = read_u32_chained!(field_476_u32);
                let field_478_u32 = read_u32_chained!(field_477_u32);
                let field_479_u32 = read_u32_chained!(field_478_u32);
                let field_480_u32 = read_u32_chained!(field_479_u32);
                let field_481_u32 = read_u32_chained!(field_480_u32);
                let field_482_u32 = read_u32_chained!(field_481_u32);
                let field_483_u32 = read_u32_chained!(field_482_u32);
                let field_484_u32 = read_u32_chained!(field_483_u32);
                let field_485_u32 = read_u32_chained!(field_484_u32);
                let field_486_u32 = read_u32_chained!(field_485_u32);
                let field_487_u32 = read_u32_chained!(field_486_u32);
                let field_488_u32 = read_u32_chained!(field_487_u32);
                let field_489_u32 = read_u32_chained!(field_488_u32);
                let field_490_u32 = read_u32_chained!(field_489_u32);
                let field_491_u32 = read_u32_chained!(field_490_u32);
                let field_492_u32 = read_u32_chained!(field_491_u32);
                let field_493_u32 = read_u32_chained!(field_492_u32);
                let field_494_u32 = read_u32_chained!(field_493_u32);
                let field_495_u32 = read_u32_chained!(field_494_u32);
                let field_496_u32 = read_u32_chained!(field_495_u32);
                let field_497_u32 = read_u32_chained!(field_496_u32);
                let field_498_u32 = read_u32_chained!(field_497_u32);
                let field_499_u32 = read_u32_chained!(field_498_u32);
                let field_500_u32 = read_u32_chained!(field_499_u32);
                let field_501_u32 = read_u32_chained!(field_500_u32);
                let field_502_u32 = read_u32_chained!(field_501_u32);
                let field_503_u32 = read_u32_chained!(field_502_u32);
                let field_504_u32 = read_u32_chained!(field_503_u32);
                let field_505_u32 = read_u32_chained!(field_504_u32);
                let field_506_u32 = read_u32_chained!(field_505_u32);
                let field_507_u32 = read_u32_chained!(field_506_u32);
                let field_508_u32 = read_u32_chained!(field_507_u32);
                let field_509_u32 = read_u32_chained!(field_508_u32);
                let field_510_u32 = read_u32_chained!(field_509_u32);
                let field_511_u32 = read_u32_chained!(field_510_u32);
                let field_512_u32 = read_u32_chained!(field_511_u32);
                let field_513_u32 = read_u32_chained!(field_512_u32);
                let field_514_u32 = read_u32_chained!(field_513_u32);
                let field_515_u32 = read_u32_chained!(field_514_u32);
                let field_516_u32 = read_u32_chained!(field_515_u32);
                let field_517_u32 = read_u32_chained!(field_516_u32);
                let field_518_u32 = read_u32_chained!(field_517_u32);
                let field_519_u32 = read_u32_chained!(field_518_u32);
                let field_520_u32 = read_u32_chained!(field_519_u32);
                let field_521_u32 = read_u32_chained!(field_520_u32);
                let field_522_u32 = read_u32_chained!(field_521_u32);
                let field_523_u32 = read_u32_chained!(field_522_u32);
                let field_524_u32 = read_u32_chained!(field_523_u32);
                let field_525_u32 = read_u32_chained!(field_524_u32);
                let field_526_u32 = read_u32_chained!(field_525_u32);
                let field_527_u32 = read_u32_chained!(field_526_u32);
                let field_528_u32 = read_u32_chained!(field_527_u32);
                let field_529_u32 = read_u32_chained!(field_528_u32);
                let field_530_u32 = read_u32_chained!(field_529_u32);
                let field_531_u32 = read_u32_chained!(field_530_u32);
                let field_532_u32 = read_u32_chained!(field_531_u32);
                let field_533_u32 = read_u32_chained!(field_532_u32);
                let field_534_u32 = read_u32_chained!(field_533_u32);
                let field_535_u32 = read_u32_chained!(field_534_u32);
                let field_536_u32 = read_u32_chained!(field_535_u32);
                let field_537_u32 = read_u32_chained!(field_536_u32);
                let field_538_u32 = read_u32_chained!(field_537_u32);
                let field_539_u32 = read_u32_chained!(field_538_u32);
                let field_540_u32 = read_u32_chained!(field_539_u32);
                let field_541_u32 = read_u32_chained!(field_540_u32);
                let field_542_u32 = read_u32_chained!(field_541_u32);
                let field_543_u32 = read_u32_chained!(field_542_u32);
                let field_544_u32 = read_u32_chained!(field_543_u32);
                let field_545_u32 = read_u32_chained!(field_544_u32);
                let field_546_u32 = read_u32_chained!(field_545_u32);
                let field_547_u32 = read_u32_chained!(field_546_u32);
                let field_548_u32 = read_u32_chained!(field_547_u32);
                let field_549_u32 = read_u32_chained!(field_548_u32);
                let field_550_u32 = read_u32_chained!(field_549_u32);
                let field_551_u32 = read_u32_chained!(field_550_u32);
                let field_552_u32 = read_u32_chained!(field_551_u32);
                let field_553_u32 = read_u32_chained!(field_552_u32);
                let field_554_u32 = read_u32_chained!(field_553_u32);
                let field_555_u32 = read_u32_chained!(field_554_u32);
                let field_556_u32 = read_u32_chained!(field_555_u32);
                let field_557_u32 = read_u32_chained!(field_556_u32);
                let field_558_u32 = read_u32_chained!(field_557_u32);
                let field_559_u32 = read_u32_chained!(field_558_u32);
                let field_560_u32 = read_u32_chained!(field_559_u32);
                let field_561_u32 = read_u32_chained!(field_560_u32);
                let field_562_u32 = read_u32_chained!(field_561_u32);
                let field_563_u32 = read_u32_chained!(field_562_u32);
                let field_564_u32 = read_u32_chained!(field_563_u32);
                let field_565_u32 = read_u32_chained!(field_564_u32);
                let field_566_u32 = read_u32_chained!(field_565_u32);
                let field_567_u32 = read_u32_chained!(field_566_u32);
                let field_568_u32 = read_u32_chained!(field_567_u32);
                let field_569_u32 = read_u32_chained!(field_568_u32);
                let field_570_u32 = read_u32_chained!(field_569_u32);
                let field_571_u32 = read_u32_chained!(field_570_u32);
                let field_572_u32 = read_u32_chained!(field_571_u32);
                let field_573_u32 = read_u32_chained!(field_572_u32);
                let field_574_u32 = read_u32_chained!(field_573_u32);
                let field_575_u32 = read_u32_chained!(field_574_u32);
                let field_576_u32 = read_u32_chained!(field_575_u32);
                let field_577_u32 = read_u32_chained!(field_576_u32);
                let field_578_u32 = read_u32_chained!(field_577_u32);
                let field_579_u32 = read_u32_chained!(field_578_u32);
                let field_580_u32 = read_u32_chained!(field_579_u32);
                let field_581_u32 = read_u32_chained!(field_580_u32);
                let field_582_u32 = read_u32_chained!(field_581_u32);
                let field_583_u32 = read_u32_chained!(field_582_u32);
                let field_584_u32 = read_u32_chained!(field_583_u32);
                let field_585_u32 = read_u32_chained!(field_584_u32);
                let field_586_u32 = read_u32_chained!(field_585_u32);
                let field_587_u32 = read_u32_chained!(field_586_u32);
                let field_588_u32 = read_u32_chained!(field_587_u32);
                let field_589_u32 = read_u32_chained!(field_588_u32);
                let field_590_u32 = read_u32_chained!(field_589_u32);
                let field_591_u32 = read_u32_chained!(field_590_u32);
                let field_592_u32 = read_u32_chained!(field_591_u32);
                let field_593_u32 = read_u32_chained!(field_592_u32);
                let field_594_u32 = read_u32_chained!(field_593_u32);
                let field_595_u32 = read_u32_chained!(field_594_u32);
                let field_596_u32 = read_u32_chained!(field_595_u32);
                let field_597_u32 = read_u32_chained!(field_596_u32);
                let field_598_u32 = read_u32_chained!(field_597_u32);
                let field_599_u32 = read_u32_chained!(field_598_u32);
                let field_600_u32 = read_u32_chained!(field_599_u32);
                let field_601_u32 = read_u32_chained!(field_600_u32);
                let field_602_u32 = read_u32_chained!(field_601_u32);
                let field_603_u32 = read_u32_chained!(field_602_u32);
                let field_604_u32 = read_u32_chained!(field_603_u32);
                let field_605_u32 = read_u32_chained!(field_604_u32);
                let field_606_u32 = read_u32_chained!(field_605_u32);
                let field_607_u32 = read_u32_chained!(field_606_u32);
                let field_608_u32 = read_u32_chained!(field_607_u32);
                let field_609_u32 = read_u32_chained!(field_608_u32);
                let field_610_u32 = read_u32_chained!(field_609_u32);
                let field_611_u32 = read_u32_chained!(field_610_u32);
                let field_612_u32 = read_u32_chained!(field_611_u32);
                let field_613_u32 = read_u32_chained!(field_612_u32);
                let field_614_u32 = read_u32_chained!(field_613_u32);
                let field_615_u32 = read_u32_chained!(field_614_u32);
                let field_616_u32 = read_u32_chained!(field_615_u32);
                let field_617_u32 = read_u32_chained!(field_616_u32);
                let field_618_u32 = read_u32_chained!(field_617_u32);
                let field_619_u32 = read_u32_chained!(field_618_u32);
                let field_620_u32 = read_u32_chained!(field_619_u32);
                let field_621_u32 = read_u32_chained!(field_620_u32);
                let field_622_u32 = read_u32_chained!(field_621_u32);
                let field_623_u32 = read_u32_chained!(field_622_u32);
                let field_624_u32 = read_u32_chained!(field_623_u32);
                let field_625_u32 = read_u32_chained!(field_624_u32);
                let field_626_u32 = read_u32_chained!(field_625_u32);
                let field_627_u32 = read_u32_chained!(field_626_u32);
                let field_628_u32 = read_u32_chained!(field_627_u32);
                let field_629_u32 = read_u32_chained!(field_628_u32);
                let field_630_u32 = read_u32_chained!(field_629_u32);
                let field_631_u32 = read_u32_chained!(field_630_u32);
                let field_632_u32 = read_u32_chained!(field_631_u32);
                let field_633_u32 = read_u32_chained!(field_632_u32);
                let field_634_u32 = read_u32_chained!(field_633_u32);
                let field_635_u32 = read_u32_chained!(field_634_u32);
                let field_636_u32 = read_u32_chained!(field_635_u32);
                let field_637_u32 = read_u32_chained!(field_636_u32);
                let field_638_u32 = read_u32_chained!(field_637_u32);
                let field_639_u32 = read_u32_chained!(field_638_u32);
                let field_640_u32 = read_u32_chained!(field_639_u32);
                let field_641_u32 = read_u32_chained!(field_640_u32);
                let field_642_u32 = read_u32_chained!(field_641_u32);
                let field_643_u32 = read_u32_chained!(field_642_u32);
                let field_644_u32 = read_u32_chained!(field_643_u32);
                let field_645_u32 = read_u32_chained!(field_644_u32);
                let field_646_u32 = read_u32_chained!(field_645_u32);
                let field_647_u32 = read_u32_chained!(field_646_u32);
                let field_648_u32 = read_u32_chained!(field_647_u32);
                let field_649_u32 = read_u32_chained!(field_648_u32);
                let field_650_u32 = read_u32_chained!(field_649_u32);
                let field_651_u32 = read_u32_chained!(field_650_u32);
                let field_652_u32 = read_u32_chained!(field_651_u32);
                let field_653_u32 = read_u32_chained!(field_652_u32);
                let field_654_u32 = read_u32_chained!(field_653_u32);
                let field_655_u32 = read_u32_chained!(field_654_u32);
                let field_656_u32 = read_u32_chained!(field_655_u32);
                let field_657_u32 = read_u32_chained!(field_656_u32);
                let field_658_u32 = read_u32_chained!(field_657_u32);
                let field_659_u32 = read_u32_chained!(field_658_u32);
                let field_660_u32 = read_u32_chained!(field_659_u32);
                let field_661_u32 = read_u32_chained!(field_660_u32);
                let field_662_u32 = read_u32_chained!(field_661_u32);
                let field_663_u32 = read_u32_chained!(field_662_u32);
                let field_664_u32 = read_u32_chained!(field_663_u32);
                // Alt-format detection: when trigger_event_handler_list (field 17) failed,
                // probe is still at tail_start. Try to read alt header (count + flag + name).
                let alt_trigger_count = if trigger_event_handler_list.is_none() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) if v < 1000 => Some(v), // sanity: count should be small
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let alt_trigger_flag = if alt_trigger_count.is_some() && probe + 1 <= entry_end {
                    let pre_ = probe;
                    match u8::read_from(data, &mut probe) {
                        Ok(v) => Some(v), _ => { probe = pre_; None }
                    }
                } else { None };
                let alt_trigger_name = if alt_trigger_flag.is_some() {
                    let pre_ = probe;
                    match CString::read_from(data, &mut probe) {
                        Ok(s) if probe <= entry_end => Some(s),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let alt_inner_count = if alt_trigger_name.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) if v < 1000 => Some(v),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let alt_inner_name = if alt_inner_count.is_some() {
                    let pre_ = probe;
                    match CString::read_from(data, &mut probe) {
                        Ok(s) if probe <= entry_end => Some(s),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let alt_inner_flag = if alt_inner_name.is_some() && probe + 4 <= entry_end {
                    let pre_ = probe;
                    match u32::read_from(data, &mut probe) {
                        Ok(v) => Some(v),
                        _ => { probe = pre_; None }
                    }
                } else { None };
                let alt_body_001 = read_u32_chained!(alt_inner_flag);
                let alt_body_002 = read_u32_chained!(alt_body_001);
                let alt_body_003 = read_u32_chained!(alt_body_002);
                let alt_body_004 = read_u32_chained!(alt_body_003);
                let alt_body_005 = read_u32_chained!(alt_body_004);
                let alt_body_006 = read_u32_chained!(alt_body_005);
                let alt_body_007 = read_u32_chained!(alt_body_006);
                let alt_body_008 = read_u32_chained!(alt_body_007);
                let alt_body_009 = read_u32_chained!(alt_body_008);
                let alt_body_010 = read_u32_chained!(alt_body_009);
                let alt_body_011 = read_u32_chained!(alt_body_010);
                let alt_body_012 = read_u32_chained!(alt_body_011);
                let alt_body_013 = read_u32_chained!(alt_body_012);
                let alt_body_014 = read_u32_chained!(alt_body_013);
                let alt_body_015 = read_u32_chained!(alt_body_014);
                let alt_body_016 = read_u32_chained!(alt_body_015);
                let alt_body_017 = read_u32_chained!(alt_body_016);
                let alt_body_018 = read_u32_chained!(alt_body_017);
                let alt_body_019 = read_u32_chained!(alt_body_018);
                let alt_body_020 = read_u32_chained!(alt_body_019);
                let alt_body_021 = read_u32_chained!(alt_body_020);
                let alt_body_022 = read_u32_chained!(alt_body_021);
                let alt_body_023 = read_u32_chained!(alt_body_022);
                let alt_body_024 = read_u32_chained!(alt_body_023);
                let alt_body_025 = read_u32_chained!(alt_body_024);
                let alt_body_026 = read_u32_chained!(alt_body_025);
                let alt_body_027 = read_u32_chained!(alt_body_026);
                let alt_body_028 = read_u32_chained!(alt_body_027);
                let alt_body_029 = read_u32_chained!(alt_body_028);
                let alt_body_030 = read_u32_chained!(alt_body_029);
                let alt_body_031 = read_u32_chained!(alt_body_030);
                let alt_body_032 = read_u32_chained!(alt_body_031);
                let alt_body_033 = read_u32_chained!(alt_body_032);
                let alt_body_034 = read_u32_chained!(alt_body_033);
                let alt_body_035 = read_u32_chained!(alt_body_034);
                let alt_body_036 = read_u32_chained!(alt_body_035);
                let alt_body_037 = read_u32_chained!(alt_body_036);
                let alt_body_038 = read_u32_chained!(alt_body_037);
                let alt_body_039 = read_u32_chained!(alt_body_038);
                let alt_body_040 = read_u32_chained!(alt_body_039);
                let alt_body_041 = read_u32_chained!(alt_body_040);
                let alt_body_042 = read_u32_chained!(alt_body_041);
                let alt_body_043 = read_u32_chained!(alt_body_042);
                let alt_body_044 = read_u32_chained!(alt_body_043);
                let alt_body_045 = read_u32_chained!(alt_body_044);
                let alt_body_046 = read_u32_chained!(alt_body_045);
                let alt_body_047 = read_u32_chained!(alt_body_046);
                let alt_body_048 = read_u32_chained!(alt_body_047);
                let alt_body_049 = read_u32_chained!(alt_body_048);
                let alt_body_050 = read_u32_chained!(alt_body_049);
                let alt_body_051 = read_u32_chained!(alt_body_050);
                let alt_body_052 = read_u32_chained!(alt_body_051);
                let alt_body_053 = read_u32_chained!(alt_body_052);
                let alt_body_054 = read_u32_chained!(alt_body_053);
                let alt_body_055 = read_u32_chained!(alt_body_054);
                let alt_body_056 = read_u32_chained!(alt_body_055);
                let alt_body_057 = read_u32_chained!(alt_body_056);
                let alt_body_058 = read_u32_chained!(alt_body_057);
                let alt_body_059 = read_u32_chained!(alt_body_058);
                let alt_body_060 = read_u32_chained!(alt_body_059);
                let alt_body_061 = read_u32_chained!(alt_body_060);
                let alt_body_062 = read_u32_chained!(alt_body_061);
                let alt_body_063 = read_u32_chained!(alt_body_062);
                let alt_body_064 = read_u32_chained!(alt_body_063);
                let alt_body_065 = read_u32_chained!(alt_body_064);
                let alt_body_066 = read_u32_chained!(alt_body_065);
                let alt_body_067 = read_u32_chained!(alt_body_066);
                let alt_body_068 = read_u32_chained!(alt_body_067);
                let alt_body_069 = read_u32_chained!(alt_body_068);
                let alt_body_070 = read_u32_chained!(alt_body_069);
                let alt_body_071 = read_u32_chained!(alt_body_070);
                let alt_body_072 = read_u32_chained!(alt_body_071);
                let alt_body_073 = read_u32_chained!(alt_body_072);
                let alt_body_074 = read_u32_chained!(alt_body_073);
                let alt_body_075 = read_u32_chained!(alt_body_074);
                let alt_body_076 = read_u32_chained!(alt_body_075);
                let alt_body_077 = read_u32_chained!(alt_body_076);
                let alt_body_078 = read_u32_chained!(alt_body_077);
                let alt_body_079 = read_u32_chained!(alt_body_078);
                let alt_body_080 = read_u32_chained!(alt_body_079);
                let alt_body_081 = read_u32_chained!(alt_body_080);
                let alt_body_082 = read_u32_chained!(alt_body_081);
                let alt_body_083 = read_u32_chained!(alt_body_082);
                let alt_body_084 = read_u32_chained!(alt_body_083);
                let alt_body_085 = read_u32_chained!(alt_body_084);
                let alt_body_086 = read_u32_chained!(alt_body_085);
                let alt_body_087 = read_u32_chained!(alt_body_086);
                let alt_body_088 = read_u32_chained!(alt_body_087);
                let alt_body_089 = read_u32_chained!(alt_body_088);
                let alt_body_090 = read_u32_chained!(alt_body_089);
                let alt_body_091 = read_u32_chained!(alt_body_090);
                let alt_body_092 = read_u32_chained!(alt_body_091);
                let alt_body_093 = read_u32_chained!(alt_body_092);
                let alt_body_094 = read_u32_chained!(alt_body_093);
                let alt_body_095 = read_u32_chained!(alt_body_094);
                let alt_body_096 = read_u32_chained!(alt_body_095);
                let alt_body_097 = read_u32_chained!(alt_body_096);
                let alt_body_098 = read_u32_chained!(alt_body_097);
                let alt_body_099 = read_u32_chained!(alt_body_098);
                let alt_body_100 = read_u32_chained!(alt_body_099);
                let alt_body_101 = read_u32_chained!(alt_body_100);
                let alt_body_102 = read_u32_chained!(alt_body_101);
                let alt_body_103 = read_u32_chained!(alt_body_102);
                let alt_body_104 = read_u32_chained!(alt_body_103);
                let alt_body_105 = read_u32_chained!(alt_body_104);
                let alt_body_106 = read_u32_chained!(alt_body_105);
                let alt_body_107 = read_u32_chained!(alt_body_106);
                let alt_body_108 = read_u32_chained!(alt_body_107);
                let alt_body_109 = read_u32_chained!(alt_body_108);
                let alt_body_110 = read_u32_chained!(alt_body_109);
                let alt_body_111 = read_u32_chained!(alt_body_110);
                let alt_body_112 = read_u32_chained!(alt_body_111);
                let alt_body_113 = read_u32_chained!(alt_body_112);
                let alt_body_114 = read_u32_chained!(alt_body_113);
                let alt_body_115 = read_u32_chained!(alt_body_114);
                let alt_body_116 = read_u32_chained!(alt_body_115);
                let alt_body_117 = read_u32_chained!(alt_body_116);
                let alt_body_118 = read_u32_chained!(alt_body_117);
                let alt_body_119 = read_u32_chained!(alt_body_118);
                let alt_body_120 = read_u32_chained!(alt_body_119);
                let alt_body_121 = read_u32_chained!(alt_body_120);
                let alt_body_122 = read_u32_chained!(alt_body_121);
                let alt_body_123 = read_u32_chained!(alt_body_122);
                let alt_body_124 = read_u32_chained!(alt_body_123);
                let alt_body_125 = read_u32_chained!(alt_body_124);
                let alt_body_126 = read_u32_chained!(alt_body_125);
                let alt_body_127 = read_u32_chained!(alt_body_126);
                let alt_body_128 = read_u32_chained!(alt_body_127);
                let alt_body_129 = read_u32_chained!(alt_body_128);
                let alt_body_130 = read_u32_chained!(alt_body_129);
                let alt_body_131 = read_u32_chained!(alt_body_130);
                let alt_body_132 = read_u32_chained!(alt_body_131);
                let alt_body_133 = read_u32_chained!(alt_body_132);
                let alt_body_134 = read_u32_chained!(alt_body_133);
                let alt_body_135 = read_u32_chained!(alt_body_134);
                let alt_body_136 = read_u32_chained!(alt_body_135);
                let alt_body_137 = read_u32_chained!(alt_body_136);
                let alt_body_138 = read_u32_chained!(alt_body_137);
                let alt_body_139 = read_u32_chained!(alt_body_138);
                let alt_body_140 = read_u32_chained!(alt_body_139);
                let alt_body_141 = read_u32_chained!(alt_body_140);
                let alt_body_142 = read_u32_chained!(alt_body_141);
                let alt_body_143 = read_u32_chained!(alt_body_142);
                let alt_body_144 = read_u32_chained!(alt_body_143);
                let alt_body_145 = read_u32_chained!(alt_body_144);
                let alt_body_146 = read_u32_chained!(alt_body_145);
                let alt_body_147 = read_u32_chained!(alt_body_146);
                let alt_body_148 = read_u32_chained!(alt_body_147);
                let alt_body_149 = read_u32_chained!(alt_body_148);
                let alt_body_150 = read_u32_chained!(alt_body_149);
                let alt_body_151 = read_u32_chained!(alt_body_150);
                let alt_body_152 = read_u32_chained!(alt_body_151);
                let alt_body_153 = read_u32_chained!(alt_body_152);
                let alt_body_154 = read_u32_chained!(alt_body_153);
                let alt_body_155 = read_u32_chained!(alt_body_154);
                let alt_body_156 = read_u32_chained!(alt_body_155);
                let alt_body_157 = read_u32_chained!(alt_body_156);
                let alt_body_158 = read_u32_chained!(alt_body_157);
                let alt_body_159 = read_u32_chained!(alt_body_158);
                let alt_body_160 = read_u32_chained!(alt_body_159);
                let alt_body_161 = read_u32_chained!(alt_body_160);
                let alt_body_162 = read_u32_chained!(alt_body_161);
                let alt_body_163 = read_u32_chained!(alt_body_162);
                let alt_body_164 = read_u32_chained!(alt_body_163);
                let alt_body_165 = read_u32_chained!(alt_body_164);
                let alt_body_166 = read_u32_chained!(alt_body_165);
                let alt_body_167 = read_u32_chained!(alt_body_166);
                let alt_body_168 = read_u32_chained!(alt_body_167);
                let alt_body_169 = read_u32_chained!(alt_body_168);
                let alt_body_170 = read_u32_chained!(alt_body_169);
                let alt_body_171 = read_u32_chained!(alt_body_170);
                let alt_body_172 = read_u32_chained!(alt_body_171);
                let alt_body_173 = read_u32_chained!(alt_body_172);
                let alt_body_174 = read_u32_chained!(alt_body_173);
                let alt_body_175 = read_u32_chained!(alt_body_174);
                let alt_body_176 = read_u32_chained!(alt_body_175);
                let alt_body_177 = read_u32_chained!(alt_body_176);
                let alt_body_178 = read_u32_chained!(alt_body_177);
                let alt_body_179 = read_u32_chained!(alt_body_178);
                let alt_body_180 = read_u32_chained!(alt_body_179);
                let alt_body_181 = read_u32_chained!(alt_body_180);
                let alt_body_182 = read_u32_chained!(alt_body_181);
                let alt_body_183 = read_u32_chained!(alt_body_182);
                let alt_body_184 = read_u32_chained!(alt_body_183);
                let alt_body_185 = read_u32_chained!(alt_body_184);
                let alt_body_186 = read_u32_chained!(alt_body_185);
                let alt_body_187 = read_u32_chained!(alt_body_186);
                let alt_body_188 = read_u32_chained!(alt_body_187);
                let alt_body_189 = read_u32_chained!(alt_body_188);
                let alt_body_190 = read_u32_chained!(alt_body_189);
                let alt_body_191 = read_u32_chained!(alt_body_190);
                let alt_body_192 = read_u32_chained!(alt_body_191);
                let alt_body_193 = read_u32_chained!(alt_body_192);
                let alt_body_194 = read_u32_chained!(alt_body_193);
                let alt_body_195 = read_u32_chained!(alt_body_194);
                let alt_body_196 = read_u32_chained!(alt_body_195);
                let alt_body_197 = read_u32_chained!(alt_body_196);
                let alt_body_198 = read_u32_chained!(alt_body_197);
                let alt_body_199 = read_u32_chained!(alt_body_198);
                let alt_body_200 = read_u32_chained!(alt_body_199);
                let alt_body_201 = read_u32_chained!(alt_body_200);
                let alt_body_202 = read_u32_chained!(alt_body_201);
                let alt_body_203 = read_u32_chained!(alt_body_202);
                let alt_body_204 = read_u32_chained!(alt_body_203);
                let alt_body_205 = read_u32_chained!(alt_body_204);
                let alt_body_206 = read_u32_chained!(alt_body_205);
                let alt_body_207 = read_u32_chained!(alt_body_206);
                let alt_body_208 = read_u32_chained!(alt_body_207);
                let alt_body_209 = read_u32_chained!(alt_body_208);
                let alt_body_210 = read_u32_chained!(alt_body_209);
                let alt_body_211 = read_u32_chained!(alt_body_210);
                let alt_body_212 = read_u32_chained!(alt_body_211);
                let alt_body_213 = read_u32_chained!(alt_body_212);
                let alt_body_214 = read_u32_chained!(alt_body_213);
                let alt_body_215 = read_u32_chained!(alt_body_214);
                let alt_body_216 = read_u32_chained!(alt_body_215);
                let alt_body_217 = read_u32_chained!(alt_body_216);
                let alt_body_218 = read_u32_chained!(alt_body_217);
                let alt_body_219 = read_u32_chained!(alt_body_218);
                let alt_body_220 = read_u32_chained!(alt_body_219);
                let alt_body_221 = read_u32_chained!(alt_body_220);
                let alt_body_222 = read_u32_chained!(alt_body_221);
                let alt_body_223 = read_u32_chained!(alt_body_222);
                let alt_body_224 = read_u32_chained!(alt_body_223);
                let alt_body_225 = read_u32_chained!(alt_body_224);
                let alt_body_226 = read_u32_chained!(alt_body_225);
                let alt_body_227 = read_u32_chained!(alt_body_226);
                let alt_body_228 = read_u32_chained!(alt_body_227);
                let alt_body_229 = read_u32_chained!(alt_body_228);
                let alt_body_230 = read_u32_chained!(alt_body_229);
                let alt_body_231 = read_u32_chained!(alt_body_230);
                let alt_body_232 = read_u32_chained!(alt_body_231);
                let alt_body_233 = read_u32_chained!(alt_body_232);
                let alt_body_234 = read_u32_chained!(alt_body_233);
                let alt_body_235 = read_u32_chained!(alt_body_234);
                let alt_body_236 = read_u32_chained!(alt_body_235);
                let alt_body_237 = read_u32_chained!(alt_body_236);
                let alt_body_238 = read_u32_chained!(alt_body_237);
                let alt_body_239 = read_u32_chained!(alt_body_238);
                let alt_body_240 = read_u32_chained!(alt_body_239);
                let alt_body_241 = read_u32_chained!(alt_body_240);
                let alt_body_242 = read_u32_chained!(alt_body_241);
                let alt_body_243 = read_u32_chained!(alt_body_242);
                let alt_body_244 = read_u32_chained!(alt_body_243);
                let alt_body_245 = read_u32_chained!(alt_body_244);
                let alt_body_246 = read_u32_chained!(alt_body_245);
                let alt_body_247 = read_u32_chained!(alt_body_246);
                let alt_body_248 = read_u32_chained!(alt_body_247);
                let alt_body_249 = read_u32_chained!(alt_body_248);
                let alt_body_250 = read_u32_chained!(alt_body_249);
                let alt_body_251 = read_u32_chained!(alt_body_250);
                let alt_body_252 = read_u32_chained!(alt_body_251);
                let alt_body_253 = read_u32_chained!(alt_body_252);
                let alt_body_254 = read_u32_chained!(alt_body_253);
                let alt_body_255 = read_u32_chained!(alt_body_254);
                let alt_body_256 = read_u32_chained!(alt_body_255);
                let alt_body_257 = read_u32_chained!(alt_body_256);
                let alt_body_258 = read_u32_chained!(alt_body_257);
                let alt_body_259 = read_u32_chained!(alt_body_258);
                let alt_body_260 = read_u32_chained!(alt_body_259);
                let alt_body_261 = read_u32_chained!(alt_body_260);
                let alt_body_262 = read_u32_chained!(alt_body_261);
                let alt_body_263 = read_u32_chained!(alt_body_262);
                let alt_body_264 = read_u32_chained!(alt_body_263);
                let alt_body_265 = read_u32_chained!(alt_body_264);
                let alt_body_266 = read_u32_chained!(alt_body_265);
                let alt_body_267 = read_u32_chained!(alt_body_266);
                let alt_body_268 = read_u32_chained!(alt_body_267);
                let alt_body_269 = read_u32_chained!(alt_body_268);
                let alt_body_270 = read_u32_chained!(alt_body_269);
                let alt_body_271 = read_u32_chained!(alt_body_270);
                let alt_body_272 = read_u32_chained!(alt_body_271);
                let alt_body_273 = read_u32_chained!(alt_body_272);
                let alt_body_274 = read_u32_chained!(alt_body_273);
                let alt_body_275 = read_u32_chained!(alt_body_274);
                let alt_body_276 = read_u32_chained!(alt_body_275);
                let alt_body_277 = read_u32_chained!(alt_body_276);
                let alt_body_278 = read_u32_chained!(alt_body_277);
                let alt_body_279 = read_u32_chained!(alt_body_278);
                let alt_body_280 = read_u32_chained!(alt_body_279);
                let alt_body_281 = read_u32_chained!(alt_body_280);
                let alt_body_282 = read_u32_chained!(alt_body_281);
                let alt_body_283 = read_u32_chained!(alt_body_282);
                let alt_body_284 = read_u32_chained!(alt_body_283);
                let alt_body_285 = read_u32_chained!(alt_body_284);
                let alt_body_286 = read_u32_chained!(alt_body_285);
                let alt_body_287 = read_u32_chained!(alt_body_286);
                let alt_body_288 = read_u32_chained!(alt_body_287);
                let alt_body_289 = read_u32_chained!(alt_body_288);
                let alt_body_290 = read_u32_chained!(alt_body_289);
                let alt_body_291 = read_u32_chained!(alt_body_290);
                let alt_body_292 = read_u32_chained!(alt_body_291);
                let alt_body_293 = read_u32_chained!(alt_body_292);
                let alt_body_294 = read_u32_chained!(alt_body_293);
                let alt_body_295 = read_u32_chained!(alt_body_294);
                let alt_body_296 = read_u32_chained!(alt_body_295);
                let alt_body_297 = read_u32_chained!(alt_body_296);
                let alt_body_298 = read_u32_chained!(alt_body_297);
                let alt_body_299 = read_u32_chained!(alt_body_298);
                let alt_body_300 = read_u32_chained!(alt_body_299);
                let alt_body_301 = read_u32_chained!(alt_body_300);
                let alt_body_302 = read_u32_chained!(alt_body_301);
                let alt_body_303 = read_u32_chained!(alt_body_302);
                let alt_body_304 = read_u32_chained!(alt_body_303);
                let alt_body_305 = read_u32_chained!(alt_body_304);
                let alt_body_306 = read_u32_chained!(alt_body_305);
                let alt_body_307 = read_u32_chained!(alt_body_306);
                let alt_body_308 = read_u32_chained!(alt_body_307);
                let alt_body_309 = read_u32_chained!(alt_body_308);
                let alt_body_310 = read_u32_chained!(alt_body_309);
                let alt_body_311 = read_u32_chained!(alt_body_310);
                let alt_body_312 = read_u32_chained!(alt_body_311);
                let alt_body_313 = read_u32_chained!(alt_body_312);
                let alt_body_314 = read_u32_chained!(alt_body_313);
                let alt_body_315 = read_u32_chained!(alt_body_314);
                let alt_body_316 = read_u32_chained!(alt_body_315);
                let alt_body_317 = read_u32_chained!(alt_body_316);
                let alt_body_318 = read_u32_chained!(alt_body_317);
                let alt_body_319 = read_u32_chained!(alt_body_318);
                let alt_body_320 = read_u32_chained!(alt_body_319);
                let field_665_u32 = read_u32_chained!(field_664_u32);
                let field_666_u32 = read_u32_chained!(field_665_u32);
                let field_667_u32 = read_u32_chained!(field_666_u32);
                let field_668_u32 = read_u32_chained!(field_667_u32);
                let field_669_u32 = read_u32_chained!(field_668_u32);
                let field_670_u32 = read_u32_chained!(field_669_u32);
                let field_671_u32 = read_u32_chained!(field_670_u32);
                let field_672_u32 = read_u32_chained!(field_671_u32);
                let field_673_u32 = read_u32_chained!(field_672_u32);
                let field_674_u32 = read_u32_chained!(field_673_u32);
                let field_675_u32 = read_u32_chained!(field_674_u32);
                let field_676_u32 = read_u32_chained!(field_675_u32);
                let field_677_u32 = read_u32_chained!(field_676_u32);
                let field_678_u32 = read_u32_chained!(field_677_u32);
                let field_679_u32 = read_u32_chained!(field_678_u32);
                let field_680_u32 = read_u32_chained!(field_679_u32);
                let field_681_u32 = read_u32_chained!(field_680_u32);
                let field_682_u32 = read_u32_chained!(field_681_u32);
                let field_683_u32 = read_u32_chained!(field_682_u32);
                let field_684_u32 = read_u32_chained!(field_683_u32);
                let field_685_u32 = read_u32_chained!(field_684_u32);
                let field_686_u32 = read_u32_chained!(field_685_u32);
                let field_687_u32 = read_u32_chained!(field_686_u32);
                let field_688_u32 = read_u32_chained!(field_687_u32);
                let field_689_u32 = read_u32_chained!(field_688_u32);
                let field_690_u32 = read_u32_chained!(field_689_u32);
                let field_691_u32 = read_u32_chained!(field_690_u32);
                let field_692_u32 = read_u32_chained!(field_691_u32);
                let field_693_u32 = read_u32_chained!(field_692_u32);
                let field_694_u32 = read_u32_chained!(field_693_u32);
                let field_695_u32 = read_u32_chained!(field_694_u32);
                let field_696_u32 = read_u32_chained!(field_695_u32);
                let field_697_u32 = read_u32_chained!(field_696_u32);
                let field_698_u32 = read_u32_chained!(field_697_u32);
                let field_699_u32 = read_u32_chained!(field_698_u32);
                let field_700_u32 = read_u32_chained!(field_699_u32);
                let field_701_u32 = read_u32_chained!(field_700_u32);
                let field_702_u32 = read_u32_chained!(field_701_u32);
                let field_703_u32 = read_u32_chained!(field_702_u32);
                let field_704_u32 = read_u32_chained!(field_703_u32);
                let field_705_u32 = read_u32_chained!(field_704_u32);
                let field_706_u32 = read_u32_chained!(field_705_u32);
                let field_707_u32 = read_u32_chained!(field_706_u32);
                let field_708_u32 = read_u32_chained!(field_707_u32);
                let field_709_u32 = read_u32_chained!(field_708_u32);
                let field_710_u32 = read_u32_chained!(field_709_u32);
                let field_711_u32 = read_u32_chained!(field_710_u32);
                let field_712_u32 = read_u32_chained!(field_711_u32);
                let field_713_u32 = read_u32_chained!(field_712_u32);
                let field_714_u32 = read_u32_chained!(field_713_u32);
                let field_715_u32 = read_u32_chained!(field_714_u32);
                let field_716_u32 = read_u32_chained!(field_715_u32);
                let field_717_u32 = read_u32_chained!(field_716_u32);
                let field_718_u32 = read_u32_chained!(field_717_u32);
                let field_719_u32 = read_u32_chained!(field_718_u32);
                let field_720_u32 = read_u32_chained!(field_719_u32);
                let field_721_u32 = read_u32_chained!(field_720_u32);
                let field_722_u32 = read_u32_chained!(field_721_u32);
                let field_723_u32 = read_u32_chained!(field_722_u32);
                let field_724_u32 = read_u32_chained!(field_723_u32);
                let field_725_u32 = read_u32_chained!(field_724_u32);
                let field_726_u32 = read_u32_chained!(field_725_u32);
                let field_727_u32 = read_u32_chained!(field_726_u32);
                let field_728_u32 = read_u32_chained!(field_727_u32);
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
                    field_108_u32,
                    field_109_u32,
                    field_110_u32,
                    field_111_u32,
                    field_112_u32,
                    field_113_u32,
                    field_114_u32,
                    field_115_u32,
                    field_116_u32,
                    field_117_u32,
                    field_118_u32,
                    field_119_u32,
                    field_120_u32,
                    field_121_u32,
                    field_122_u32,
                    field_123_u32,
                    field_124_u32,
                    field_125_u32,
                    field_126_u32,
                    field_127_u32,
                    field_128_u32,
                    field_129_u32,
                    field_130_u32,
                    field_131_u32,
                    field_132_u32,
                    field_133_u32,
                    field_134_u32,
                    field_135_u32,
                    field_136_u32,
                    field_137_u32,
                    field_138_u32,
                    field_139_u32,
                    field_140_u32,
                    field_141_u32,
                    field_142_u32,
                    field_143_u32,
                    field_144_u32,
                    field_145_u32,
                    field_146_u32,
                    field_147_u32,
                    field_148_u32,
                    field_149_u32,
                    field_150_u32,
                    field_151_u32,
                    field_152_u32,
                    field_153_u32,
                    field_154_u32,
                    field_155_u32,
                    field_156_u32,
                    field_157_u32,
                    field_158_u32,
                    field_159_u32,
                    field_160_u32,
                    field_161_u32,
                    field_162_u32,
                    field_163_u32,
                    field_164_u32,
                    field_165_u32,
                    field_166_u32,
                    field_167_u32,
                    field_168_u32,
                    field_169_u32,
                    field_170_u32,
                    field_171_u32,
                    field_172_u32,
                    field_173_u32,
                    field_174_u32,
                    field_175_u32,
                    field_176_u32,
                    field_177_u32,
                    field_178_u32,
                    field_179_u32,
                    field_180_u32,
                    field_181_u32,
                    field_182_u32,
                    field_183_u32,
                    field_184_u32,
                    field_185_u32,
                    field_186_u32,
                    field_187_u32,
                    field_188_u32,
                    field_189_u32,
                    field_190_u32,
                    field_191_u32,
                    field_192_u32,
                    field_193_u32,
                    field_194_u32,
                    field_195_u32,
                    field_196_u32,
                    field_197_u32,
                    field_198_u32,
                    field_199_u32,
                    field_200_u32,
                    field_201_u32,
                    field_202_u32,
                    field_203_u32,
                    field_204_u32,
                    field_205_u32,
                    field_206_u32,
                    field_207_u32,
                    field_208_u32,
                    field_209_u32,
                    field_210_u32,
                    field_211_u32,
                    field_212_u32,
                    field_213_u32,
                    field_214_u32,
                    field_215_u32,
                    field_216_u32,
                    field_217_u32,
                    field_218_u32,
                    field_219_u32,
                    field_220_u32,
                    field_221_u32,
                    field_222_u32,
                    field_223_u32,
                    field_224_u32,
                    field_225_u32,
                    field_226_u32,
                    field_227_u32,
                    field_228_u32,
                    field_229_u32,
                    field_230_u32,
                    field_231_u32,
                    field_232_u32,
                    field_233_u32,
                    field_234_u32,
                    field_235_u32,
                    field_236_u32,
                    field_237_u32,
                    field_238_u32,
                    field_239_u32,
                    field_240_u32,
                    field_241_u32,
                    field_242_u32,
                    field_243_u32,
                    field_244_u32,
                    field_245_u32,
                    field_246_u32,
                    field_247_u32,
                    field_248_u32,
                    field_249_u32,
                    field_250_u32,
                    field_251_u32,
                    field_252_u32,
                    field_253_u32,
                    field_254_u32,
                    field_255_u32,
                    field_256_u32,
                    field_257_u32,
                    field_258_u32,
                    field_259_u32,
                    field_260_u32,
                    field_261_u32,
                    field_262_u32,
                    field_263_u32,
                    field_264_u32,
                    field_265_u32,
                    field_266_u32,
                    field_267_u32,
                    field_268_u32,
                    field_269_u32,
                    field_270_u32,
                    field_271_u32,
                    field_272_u32,
                    field_273_u32,
                    field_274_u32,
                    field_275_u32,
                    field_276_u32,
                    field_277_u32,
                    field_278_u32,
                    field_279_u32,
                    field_280_u32,
                    field_281_u32,
                    field_282_u32,
                    field_283_u32,
                    field_284_u32,
                    field_285_u32,
                    field_286_u32,
                    field_287_u32,
                    field_288_u32,
                    field_289_u32,
                    field_290_u32,
                    field_291_u32,
                    field_292_u32,
                    field_293_u32,
                    field_294_u32,
                    field_295_u32,
                    field_296_u32,
                    field_297_u32,
                    field_298_u32,
                    field_299_u32,
                    field_300_u32,
                    field_301_u32,
                    field_302_u32,
                    field_303_u32,
                    field_304_u32,
                    field_305_u32,
                    field_306_u32,
                    field_307_u32,
                    field_308_u32,
                    field_309_u32,
                    field_310_u32,
                    field_311_u32,
                    field_312_u32,
                    field_313_u32,
                    field_314_u32,
                    field_315_u32,
                    field_316_u32,
                    field_317_u32,
                    field_318_u32,
                    field_319_u32,
                    field_320_u32,
                    field_321_u32,
                    field_322_u32,
                    field_323_u32,
                    field_324_u32,
                    field_325_u32,
                    field_326_u32,
                    field_327_u32,
                    field_328_u32,
                    field_329_u32,
                    field_330_u32,
                    field_331_u32,
                    field_332_u32,
                    field_333_u32,
                    field_334_u32,
                    field_335_u32,
                    field_336_u32,
                    field_337_u32,
                    field_338_u32,
                    field_339_u32,
                    field_340_u32,
                    field_341_u32,
                    field_342_u32_count,
                    field_343_u8_flag,
                    field_344_u32,
                    field_345_u32,
                    field_346_u32,
                    field_347_u32,
                    field_348_u32,
                    field_349_u32,
                    field_350_u32,
                    field_351_u32,
                    field_352_u32,
                    field_353_u32,
                    field_354_u32,
                    field_355_u32,
                    field_356_u32,
                    field_357_u32,
                    field_358_u32,
                    field_359_u32,
                    field_360_u32,
                    field_361_u32,
                    field_362_u32,
                    field_363_u32,
                    field_364_u32,
                    field_365_u32,
                    field_366_u32,
                    field_367_u32,
                    field_368_u32,
                    field_369_u32,
                    field_370_u32,
                    field_371_u32,
                    field_372_u32,
                    field_373_u32,
                    field_374_u32,
                    field_375_u32,
                    field_376_u32,
                    field_377_u32,
                    field_378_u32,
                    field_379_u32,
                    field_380_u32,
                    field_381_u32,
                    field_382_u32,
                    field_383_u32,
                    field_384_u32,
                    field_385_u32,
                    field_386_u32,
                    field_387_u32,
                    field_388_u32,
                    field_389_u32,
                    field_390_u32,
                    field_391_u32,
                    field_392_u32,
                    field_393_u32,
                    field_394_u32,
                    field_395_u32,
                    field_396_u32,
                    field_397_u32,
                    field_398_u32,
                    field_399_u32,
                    field_400_u32,
                    field_401_u32,
                    field_402_u32,
                    field_403_u32,
                    field_404_u32,
                    field_405_u32,
                    field_406_u32,
                    field_407_u32,
                    field_408_u32,
                    field_409_u32,
                    field_410_u32,
                    field_411_u32,
                    field_412_u32,
                    field_413_u32,
                    field_414_u32,
                    field_415_u32,
                    field_416_u32,
                    field_417_u32,
                    field_418_u32,
                    field_419_u32,
                    field_420_u32,
                    field_421_u32,
                    field_422_u32,
                    field_423_u32,
                    field_424_u32,
                    field_425_u32,
                    field_426_u32,
                    field_427_u32,
                    field_428_u32,
                    field_429_u32,
                    field_430_u32,
                    field_431_u32,
                    field_432_u32,
                    field_433_u32,
                    field_434_u32,
                    field_435_u32,
                    field_436_u32,
                    field_437_u32,
                    field_438_u32,
                    field_439_u32,
                    field_440_u32,
                    field_441_u32,
                    field_442_u32,
                    field_443_u32,
                    field_444_u32,
                    field_445_u32,
                    field_446_u32,
                    field_447_u32,
                    field_448_u32,
                    field_449_u32,
                    field_450_u32,
                    field_451_u32,
                    field_452_u32,
                    field_453_u32,
                    field_454_u32,
                    field_455_u32,
                    field_456_u32,
                    field_457_u32,
                    field_458_u32,
                    field_459_u32,
                    field_460_u32,
                    field_461_u32,
                    field_462_u32,
                    field_463_u32,
                    field_464_u32,
                    field_465_u32,
                    field_466_u32,
                    field_467_u32,
                    field_468_u32,
                    field_469_u32,
                    field_470_u32,
                    field_471_u32,
                    field_472_u32,
                    field_473_u32,
                    field_474_u32,
                    field_475_u32,
                    field_476_u32,
                    field_477_u32,
                    field_478_u32,
                    field_479_u32,
                    field_480_u32,
                    field_481_u32,
                    field_482_u32,
                    field_483_u32,
                    field_484_u32,
                    field_485_u32,
                    field_486_u32,
                    field_487_u32,
                    field_488_u32,
                    field_489_u32,
                    field_490_u32,
                    field_491_u32,
                    field_492_u32,
                    field_493_u32,
                    field_494_u32,
                    field_495_u32,
                    field_496_u32,
                    field_497_u32,
                    field_498_u32,
                    field_499_u32,
                    field_500_u32,
                    field_501_u32,
                    field_502_u32,
                    field_503_u32,
                    field_504_u32,
                    field_505_u32,
                    field_506_u32,
                    field_507_u32,
                    field_508_u32,
                    field_509_u32,
                    field_510_u32,
                    field_511_u32,
                    field_512_u32,
                    field_513_u32,
                    field_514_u32,
                    field_515_u32,
                    field_516_u32,
                    field_517_u32,
                    field_518_u32,
                    field_519_u32,
                    field_520_u32,
                    field_521_u32,
                    field_522_u32,
                    field_523_u32,
                    field_524_u32,
                    field_525_u32,
                    field_526_u32,
                    field_527_u32,
                    field_528_u32,
                    field_529_u32,
                    field_530_u32,
                    field_531_u32,
                    field_532_u32,
                    field_533_u32,
                    field_534_u32,
                    field_535_u32,
                    field_536_u32,
                    field_537_u32, field_538_u32, field_539_u32, field_540_u32,
                    field_541_u32, field_542_u32, field_543_u32, field_544_u32,
                    field_545_u32, field_546_u32, field_547_u32, field_548_u32,
                    field_549_u32, field_550_u32, field_551_u32, field_552_u32,
                    field_553_u32, field_554_u32, field_555_u32, field_556_u32,
                    field_557_u32, field_558_u32, field_559_u32, field_560_u32,
                    field_561_u32, field_562_u32, field_563_u32, field_564_u32,
                    field_565_u32, field_566_u32, field_567_u32, field_568_u32,
                    field_569_u32, field_570_u32, field_571_u32, field_572_u32,
                    field_573_u32, field_574_u32, field_575_u32, field_576_u32,
                    field_577_u32, field_578_u32, field_579_u32, field_580_u32,
                    field_581_u32, field_582_u32, field_583_u32, field_584_u32,
                    field_585_u32, field_586_u32, field_587_u32, field_588_u32,
                    field_589_u32, field_590_u32, field_591_u32, field_592_u32,
                    field_593_u32, field_594_u32, field_595_u32, field_596_u32,
                    field_597_u32, field_598_u32, field_599_u32, field_600_u32,
                    field_601_u32, field_602_u32, field_603_u32, field_604_u32,
                    field_605_u32, field_606_u32, field_607_u32, field_608_u32,
                    field_609_u32, field_610_u32, field_611_u32, field_612_u32,
                    field_613_u32, field_614_u32, field_615_u32, field_616_u32,
                    field_617_u32, field_618_u32, field_619_u32, field_620_u32,
                    field_621_u32, field_622_u32, field_623_u32, field_624_u32,
                    field_625_u32, field_626_u32, field_627_u32, field_628_u32,
                    field_629_u32, field_630_u32, field_631_u32, field_632_u32,
                    field_633_u32, field_634_u32, field_635_u32, field_636_u32,
                    field_637_u32, field_638_u32, field_639_u32, field_640_u32,
                    field_641_u32, field_642_u32, field_643_u32, field_644_u32,
                    field_645_u32, field_646_u32, field_647_u32, field_648_u32,
                    field_649_u32, field_650_u32, field_651_u32, field_652_u32,
                    field_653_u32, field_654_u32, field_655_u32, field_656_u32,
                    field_657_u32, field_658_u32, field_659_u32, field_660_u32,
                    field_661_u32, field_662_u32, field_663_u32, field_664_u32,
                    alt_trigger_count,
                    alt_trigger_flag,
                    alt_trigger_name,
                    alt_inner_count,
                    alt_inner_name,
                    alt_inner_flag,
                    alt_body_001, alt_body_002, alt_body_003, alt_body_004,
                    alt_body_005, alt_body_006, alt_body_007, alt_body_008,
                    alt_body_009, alt_body_010, alt_body_011, alt_body_012,
                    alt_body_013, alt_body_014, alt_body_015, alt_body_016,
                    alt_body_017, alt_body_018, alt_body_019, alt_body_020,
                    alt_body_021, alt_body_022, alt_body_023, alt_body_024,
                    alt_body_025, alt_body_026, alt_body_027, alt_body_028,
                    alt_body_029, alt_body_030, alt_body_031, alt_body_032,
                    alt_body_033, alt_body_034, alt_body_035, alt_body_036,
                    alt_body_037, alt_body_038, alt_body_039, alt_body_040,
                    alt_body_041, alt_body_042, alt_body_043, alt_body_044,
                    alt_body_045, alt_body_046, alt_body_047, alt_body_048,
                    alt_body_049, alt_body_050, alt_body_051, alt_body_052,
                    alt_body_053, alt_body_054, alt_body_055, alt_body_056,
                    alt_body_057, alt_body_058, alt_body_059, alt_body_060,
                    alt_body_061, alt_body_062, alt_body_063, alt_body_064,
                    alt_body_065, alt_body_066, alt_body_067, alt_body_068,
                    alt_body_069, alt_body_070, alt_body_071, alt_body_072,
                    alt_body_073, alt_body_074, alt_body_075, alt_body_076,
                    alt_body_077, alt_body_078, alt_body_079, alt_body_080,
                    alt_body_081, alt_body_082, alt_body_083, alt_body_084,
                    alt_body_085, alt_body_086, alt_body_087, alt_body_088,
                    alt_body_089, alt_body_090, alt_body_091, alt_body_092,
                    alt_body_093, alt_body_094, alt_body_095, alt_body_096,
                    alt_body_097, alt_body_098, alt_body_099, alt_body_100,
                    alt_body_101, alt_body_102, alt_body_103, alt_body_104,
                    alt_body_105, alt_body_106, alt_body_107, alt_body_108,
                    alt_body_109, alt_body_110, alt_body_111, alt_body_112,
                    alt_body_113, alt_body_114, alt_body_115, alt_body_116,
                    alt_body_117, alt_body_118, alt_body_119, alt_body_120,
                    alt_body_121, alt_body_122, alt_body_123, alt_body_124,
                    alt_body_125, alt_body_126, alt_body_127, alt_body_128,
                    alt_body_129, alt_body_130, alt_body_131, alt_body_132,
                    alt_body_133, alt_body_134, alt_body_135, alt_body_136,
                    alt_body_137, alt_body_138, alt_body_139, alt_body_140,
                    alt_body_141, alt_body_142, alt_body_143, alt_body_144,
                    alt_body_145, alt_body_146, alt_body_147, alt_body_148,
                    alt_body_149, alt_body_150, alt_body_151, alt_body_152,
                    alt_body_153, alt_body_154, alt_body_155, alt_body_156,
                    alt_body_157, alt_body_158, alt_body_159, alt_body_160,
                    alt_body_161, alt_body_162, alt_body_163, alt_body_164,
                    alt_body_165, alt_body_166, alt_body_167, alt_body_168,
                    alt_body_169, alt_body_170, alt_body_171, alt_body_172,
                    alt_body_173, alt_body_174, alt_body_175, alt_body_176,
                    alt_body_177, alt_body_178, alt_body_179, alt_body_180,
                    alt_body_181, alt_body_182, alt_body_183, alt_body_184,
                    alt_body_185, alt_body_186, alt_body_187, alt_body_188,
                    alt_body_189, alt_body_190, alt_body_191, alt_body_192,
                    alt_body_193, alt_body_194, alt_body_195, alt_body_196,
                    alt_body_197, alt_body_198, alt_body_199, alt_body_200,
                    alt_body_201, alt_body_202, alt_body_203, alt_body_204,
                    alt_body_205, alt_body_206, alt_body_207, alt_body_208,
                    alt_body_209, alt_body_210, alt_body_211, alt_body_212,
                    alt_body_213, alt_body_214, alt_body_215, alt_body_216,
                    alt_body_217, alt_body_218, alt_body_219, alt_body_220,
                    alt_body_221, alt_body_222, alt_body_223, alt_body_224,
                    alt_body_225, alt_body_226, alt_body_227, alt_body_228,
                    alt_body_229, alt_body_230, alt_body_231, alt_body_232,
                    alt_body_233, alt_body_234, alt_body_235, alt_body_236,
                    alt_body_237, alt_body_238, alt_body_239, alt_body_240,
                    alt_body_241, alt_body_242, alt_body_243, alt_body_244,
                    alt_body_245, alt_body_246, alt_body_247, alt_body_248,
                    alt_body_249, alt_body_250, alt_body_251, alt_body_252,
                    alt_body_253, alt_body_254, alt_body_255, alt_body_256,
                    alt_body_257, alt_body_258, alt_body_259, alt_body_260,
                    alt_body_261, alt_body_262, alt_body_263, alt_body_264,
                    alt_body_265, alt_body_266, alt_body_267, alt_body_268,
                    alt_body_269, alt_body_270, alt_body_271, alt_body_272,
                    alt_body_273, alt_body_274, alt_body_275, alt_body_276,
                    alt_body_277, alt_body_278, alt_body_279, alt_body_280,
                    alt_body_281, alt_body_282, alt_body_283, alt_body_284,
                    alt_body_285, alt_body_286, alt_body_287, alt_body_288,
                    alt_body_289, alt_body_290, alt_body_291, alt_body_292,
                    alt_body_293, alt_body_294, alt_body_295, alt_body_296,
                    alt_body_297, alt_body_298, alt_body_299, alt_body_300,
                    alt_body_301, alt_body_302, alt_body_303, alt_body_304,
                    alt_body_305, alt_body_306, alt_body_307, alt_body_308,
                    alt_body_309, alt_body_310, alt_body_311, alt_body_312,
                    alt_body_313, alt_body_314, alt_body_315, alt_body_316,
                    alt_body_317, alt_body_318, alt_body_319, alt_body_320,
                    field_665_u32, field_666_u32, field_667_u32, field_668_u32,
                    field_669_u32, field_670_u32, field_671_u32, field_672_u32,
                    field_673_u32, field_674_u32, field_675_u32, field_676_u32,
                    field_677_u32, field_678_u32, field_679_u32, field_680_u32,
                    field_681_u32, field_682_u32, field_683_u32, field_684_u32,
                    field_685_u32, field_686_u32, field_687_u32, field_688_u32,
                    field_689_u32, field_690_u32, field_691_u32, field_692_u32,
                    field_693_u32, field_694_u32, field_695_u32, field_696_u32,
                    field_697_u32, field_698_u32, field_699_u32, field_700_u32,
                    field_701_u32, field_702_u32, field_703_u32, field_704_u32,
                    field_705_u32, field_706_u32, field_707_u32, field_708_u32,
                    field_709_u32, field_710_u32, field_711_u32, field_712_u32,
                    field_713_u32, field_714_u32, field_715_u32, field_716_u32,
                    field_717_u32, field_718_u32, field_719_u32, field_720_u32,
                    field_721_u32, field_722_u32, field_723_u32, field_724_u32,
                    field_725_u32, field_726_u32, field_727_u32, field_728_u32,
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
                field_107_u32, field_108_u32, field_109_u32, field_110_u32,
                field_111_u32, field_112_u32, field_113_u32, field_114_u32,
                field_115_u32, field_116_u32, field_117_u32, field_118_u32,
                field_119_u32, field_120_u32, field_121_u32, field_122_u32,
                field_123_u32, field_124_u32, field_125_u32, field_126_u32,
                field_127_u32, field_128_u32, field_129_u32, field_130_u32,
                field_131_u32, field_132_u32, field_133_u32, field_134_u32,
                field_135_u32, field_136_u32, field_137_u32, field_138_u32,
                field_139_u32, field_140_u32, field_141_u32, field_142_u32,
                field_143_u32, field_144_u32, field_145_u32, field_146_u32,
                field_147_u32, field_148_u32, field_149_u32, field_150_u32,
                field_151_u32, field_152_u32, field_153_u32, field_154_u32,
                field_155_u32, field_156_u32, field_157_u32, field_158_u32,
                field_159_u32, field_160_u32, field_161_u32, field_162_u32,
                field_163_u32, field_164_u32, field_165_u32, field_166_u32,
                field_167_u32, field_168_u32, field_169_u32, field_170_u32,
                field_171_u32, field_172_u32, field_173_u32, field_174_u32,
                field_175_u32, field_176_u32, field_177_u32, field_178_u32,
                field_179_u32, field_180_u32, field_181_u32,
                field_182_u32, field_183_u32, field_184_u32, field_185_u32,
                field_186_u32, field_187_u32, field_188_u32, field_189_u32,
                field_190_u32, field_191_u32, field_192_u32, field_193_u32,
                field_194_u32, field_195_u32, field_196_u32, field_197_u32,
                field_198_u32, field_199_u32, field_200_u32, field_201_u32,
                field_202_u32, field_203_u32, field_204_u32, field_205_u32,
                field_206_u32, field_207_u32, field_208_u32, field_209_u32,
                field_210_u32, field_211_u32, field_212_u32, field_213_u32,
                field_214_u32, field_215_u32, field_216_u32, field_217_u32,
                field_218_u32, field_219_u32, field_220_u32, field_221_u32,
                field_222_u32, field_223_u32, field_224_u32, field_225_u32,
                field_226_u32, field_227_u32, field_228_u32, field_229_u32,
                field_230_u32, field_231_u32, field_232_u32, field_233_u32,
                field_234_u32, field_235_u32, field_236_u32, field_237_u32,
                field_238_u32, field_239_u32, field_240_u32, field_241_u32,
                field_242_u32, field_243_u32, field_244_u32, field_245_u32,
                field_246_u32, field_247_u32, field_248_u32, field_249_u32,
                field_250_u32, field_251_u32, field_252_u32, field_253_u32,
                field_254_u32, field_255_u32, field_256_u32, field_257_u32,
                field_258_u32, field_259_u32, field_260_u32, field_261_u32,
                field_262_u32, field_263_u32, field_264_u32, field_265_u32,
                field_266_u32, field_267_u32, field_268_u32, field_269_u32,
                field_270_u32, field_271_u32, field_272_u32, field_273_u32,
                field_274_u32, field_275_u32, field_276_u32, field_277_u32,
                field_278_u32, field_279_u32, field_280_u32, field_281_u32,
                field_282_u32, field_283_u32, field_284_u32, field_285_u32,
                field_286_u32, field_287_u32, field_288_u32, field_289_u32,
                field_290_u32, field_291_u32, field_292_u32, field_293_u32,
                field_294_u32, field_295_u32, field_296_u32, field_297_u32,
                field_298_u32, field_299_u32, field_300_u32, field_301_u32,
                field_302_u32, field_303_u32, field_304_u32, field_305_u32,
                field_306_u32, field_307_u32, field_308_u32, field_309_u32,
                field_310_u32, field_311_u32, field_312_u32, field_313_u32,
                field_314_u32, field_315_u32, field_316_u32, field_317_u32,
                field_318_u32, field_319_u32, field_320_u32, field_321_u32,
                field_322_u32, field_323_u32, field_324_u32, field_325_u32,
                field_326_u32, field_327_u32, field_328_u32, field_329_u32,
                field_330_u32, field_331_u32, field_332_u32, field_333_u32,
                field_334_u32, field_335_u32, field_336_u32, field_337_u32,
                field_338_u32, field_339_u32, field_340_u32, field_341_u32,
                field_342_u32_count, field_343_u8_flag, field_344_u32,
                field_345_u32, field_346_u32, field_347_u32, field_348_u32,
                field_349_u32, field_350_u32, field_351_u32, field_352_u32,
                field_353_u32, field_354_u32, field_355_u32, field_356_u32,
                field_357_u32, field_358_u32, field_359_u32, field_360_u32,
                field_361_u32, field_362_u32, field_363_u32, field_364_u32,
                field_365_u32, field_366_u32, field_367_u32, field_368_u32,
                field_369_u32, field_370_u32, field_371_u32, field_372_u32,
                field_373_u32, field_374_u32, field_375_u32, field_376_u32,
                field_377_u32, field_378_u32, field_379_u32, field_380_u32,
                field_381_u32, field_382_u32, field_383_u32, field_384_u32,
                field_385_u32, field_386_u32, field_387_u32, field_388_u32,
                field_389_u32, field_390_u32, field_391_u32, field_392_u32,
                field_393_u32, field_394_u32, field_395_u32, field_396_u32,
                field_397_u32, field_398_u32, field_399_u32, field_400_u32,
                field_401_u32, field_402_u32, field_403_u32, field_404_u32,
                field_405_u32, field_406_u32, field_407_u32, field_408_u32,
                field_409_u32, field_410_u32, field_411_u32, field_412_u32,
                field_413_u32, field_414_u32, field_415_u32, field_416_u32,
                field_417_u32, field_418_u32, field_419_u32, field_420_u32,
                field_421_u32, field_422_u32, field_423_u32, field_424_u32,
                field_425_u32, field_426_u32, field_427_u32, field_428_u32,
                field_429_u32, field_430_u32, field_431_u32, field_432_u32,
                field_433_u32, field_434_u32, field_435_u32, field_436_u32,
                field_437_u32, field_438_u32, field_439_u32, field_440_u32,
                field_441_u32, field_442_u32, field_443_u32, field_444_u32,
                field_445_u32, field_446_u32, field_447_u32, field_448_u32,
                field_449_u32, field_450_u32, field_451_u32, field_452_u32,
                field_453_u32, field_454_u32, field_455_u32, field_456_u32,
                field_457_u32, field_458_u32, field_459_u32, field_460_u32,
                field_461_u32, field_462_u32, field_463_u32, field_464_u32,
                field_465_u32, field_466_u32, field_467_u32, field_468_u32,
                field_469_u32, field_470_u32, field_471_u32, field_472_u32,
                field_473_u32, field_474_u32, field_475_u32, field_476_u32,
                field_477_u32, field_478_u32, field_479_u32, field_480_u32,
                field_481_u32, field_482_u32, field_483_u32, field_484_u32,
                field_485_u32, field_486_u32, field_487_u32, field_488_u32,
                field_489_u32, field_490_u32, field_491_u32, field_492_u32,
                field_493_u32, field_494_u32, field_495_u32, field_496_u32,
                field_497_u32, field_498_u32, field_499_u32, field_500_u32,
                field_501_u32, field_502_u32, field_503_u32, field_504_u32,
                field_505_u32, field_506_u32, field_507_u32, field_508_u32,
                field_509_u32, field_510_u32, field_511_u32, field_512_u32,
                field_513_u32, field_514_u32, field_515_u32, field_516_u32,
                field_517_u32, field_518_u32, field_519_u32, field_520_u32,
                field_521_u32, field_522_u32, field_523_u32, field_524_u32,
                field_525_u32, field_526_u32, field_527_u32, field_528_u32,
                field_529_u32, field_530_u32, field_531_u32, field_532_u32,
                field_533_u32, field_534_u32, field_535_u32, field_536_u32,
                field_537_u32, field_538_u32, field_539_u32, field_540_u32,
                field_541_u32, field_542_u32, field_543_u32, field_544_u32,
                field_545_u32, field_546_u32, field_547_u32, field_548_u32,
                field_549_u32, field_550_u32, field_551_u32, field_552_u32,
                field_553_u32, field_554_u32, field_555_u32, field_556_u32,
                field_557_u32, field_558_u32, field_559_u32, field_560_u32,
                field_561_u32, field_562_u32, field_563_u32, field_564_u32,
                field_565_u32, field_566_u32, field_567_u32, field_568_u32,
                field_569_u32, field_570_u32, field_571_u32, field_572_u32,
                field_573_u32, field_574_u32, field_575_u32, field_576_u32,
                field_577_u32, field_578_u32, field_579_u32, field_580_u32,
                field_581_u32, field_582_u32, field_583_u32, field_584_u32,
                field_585_u32, field_586_u32, field_587_u32, field_588_u32,
                field_589_u32, field_590_u32, field_591_u32, field_592_u32,
                field_593_u32, field_594_u32, field_595_u32, field_596_u32,
                field_597_u32, field_598_u32, field_599_u32, field_600_u32,
                field_601_u32, field_602_u32, field_603_u32, field_604_u32,
                field_605_u32, field_606_u32, field_607_u32, field_608_u32,
                field_609_u32, field_610_u32, field_611_u32, field_612_u32,
                field_613_u32, field_614_u32, field_615_u32, field_616_u32,
                field_617_u32, field_618_u32, field_619_u32, field_620_u32,
                field_621_u32, field_622_u32, field_623_u32, field_624_u32,
                field_625_u32, field_626_u32, field_627_u32, field_628_u32,
                field_629_u32, field_630_u32, field_631_u32, field_632_u32,
                field_633_u32, field_634_u32, field_635_u32, field_636_u32,
                field_637_u32, field_638_u32, field_639_u32, field_640_u32,
                field_641_u32, field_642_u32, field_643_u32, field_644_u32,
                field_645_u32, field_646_u32, field_647_u32, field_648_u32,
                field_649_u32, field_650_u32, field_651_u32, field_652_u32,
                field_653_u32, field_654_u32, field_655_u32, field_656_u32,
                field_657_u32, field_658_u32, field_659_u32, field_660_u32,
                field_661_u32, field_662_u32, field_663_u32, field_664_u32,
                alt_trigger_count, alt_trigger_flag, alt_trigger_name,
                alt_inner_count, alt_inner_name, alt_inner_flag,
                alt_body_001, alt_body_002, alt_body_003, alt_body_004,
                alt_body_005, alt_body_006, alt_body_007, alt_body_008,
                alt_body_009, alt_body_010, alt_body_011, alt_body_012,
                alt_body_013, alt_body_014, alt_body_015, alt_body_016,
                alt_body_017, alt_body_018, alt_body_019, alt_body_020,
                alt_body_021, alt_body_022, alt_body_023, alt_body_024,
                alt_body_025, alt_body_026, alt_body_027, alt_body_028,
                alt_body_029, alt_body_030, alt_body_031, alt_body_032,
                alt_body_033, alt_body_034, alt_body_035, alt_body_036,
                alt_body_037, alt_body_038, alt_body_039, alt_body_040,
                alt_body_041, alt_body_042, alt_body_043, alt_body_044,
                alt_body_045, alt_body_046, alt_body_047, alt_body_048,
                alt_body_049, alt_body_050, alt_body_051, alt_body_052,
                alt_body_053, alt_body_054, alt_body_055, alt_body_056,
                alt_body_057, alt_body_058, alt_body_059, alt_body_060,
                alt_body_061, alt_body_062, alt_body_063, alt_body_064,
                alt_body_065, alt_body_066, alt_body_067, alt_body_068,
                alt_body_069, alt_body_070, alt_body_071, alt_body_072,
                alt_body_073, alt_body_074, alt_body_075, alt_body_076,
                alt_body_077, alt_body_078, alt_body_079, alt_body_080,
                alt_body_081, alt_body_082, alt_body_083, alt_body_084,
                alt_body_085, alt_body_086, alt_body_087, alt_body_088,
                alt_body_089, alt_body_090, alt_body_091, alt_body_092,
                alt_body_093, alt_body_094, alt_body_095, alt_body_096,
                alt_body_097, alt_body_098, alt_body_099, alt_body_100,
                alt_body_101, alt_body_102, alt_body_103, alt_body_104,
                alt_body_105, alt_body_106, alt_body_107, alt_body_108,
                alt_body_109, alt_body_110, alt_body_111, alt_body_112,
                alt_body_113, alt_body_114, alt_body_115, alt_body_116,
                alt_body_117, alt_body_118, alt_body_119, alt_body_120,
                alt_body_121, alt_body_122, alt_body_123, alt_body_124,
                alt_body_125, alt_body_126, alt_body_127, alt_body_128,
                alt_body_129, alt_body_130, alt_body_131, alt_body_132,
                alt_body_133, alt_body_134, alt_body_135, alt_body_136,
                alt_body_137, alt_body_138, alt_body_139, alt_body_140,
                alt_body_141, alt_body_142, alt_body_143, alt_body_144,
                alt_body_145, alt_body_146, alt_body_147, alt_body_148,
                alt_body_149, alt_body_150, alt_body_151, alt_body_152,
                alt_body_153, alt_body_154, alt_body_155, alt_body_156,
                alt_body_157, alt_body_158, alt_body_159, alt_body_160,
                alt_body_161, alt_body_162, alt_body_163, alt_body_164,
                alt_body_165, alt_body_166, alt_body_167, alt_body_168,
                alt_body_169, alt_body_170, alt_body_171, alt_body_172,
                alt_body_173, alt_body_174, alt_body_175, alt_body_176,
                alt_body_177, alt_body_178, alt_body_179, alt_body_180,
                alt_body_181, alt_body_182, alt_body_183, alt_body_184,
                alt_body_185, alt_body_186, alt_body_187, alt_body_188,
                alt_body_189, alt_body_190, alt_body_191, alt_body_192,
                alt_body_193, alt_body_194, alt_body_195, alt_body_196,
                alt_body_197, alt_body_198, alt_body_199, alt_body_200,
                alt_body_201, alt_body_202, alt_body_203, alt_body_204,
                alt_body_205, alt_body_206, alt_body_207, alt_body_208,
                alt_body_209, alt_body_210, alt_body_211, alt_body_212,
                alt_body_213, alt_body_214, alt_body_215, alt_body_216,
                alt_body_217, alt_body_218, alt_body_219, alt_body_220,
                alt_body_221, alt_body_222, alt_body_223, alt_body_224,
                alt_body_225, alt_body_226, alt_body_227, alt_body_228,
                alt_body_229, alt_body_230, alt_body_231, alt_body_232,
                alt_body_233, alt_body_234, alt_body_235, alt_body_236,
                alt_body_237, alt_body_238, alt_body_239, alt_body_240,
                alt_body_241, alt_body_242, alt_body_243, alt_body_244,
                alt_body_245, alt_body_246, alt_body_247, alt_body_248,
                alt_body_249, alt_body_250, alt_body_251, alt_body_252,
                alt_body_253, alt_body_254, alt_body_255, alt_body_256,
                alt_body_257, alt_body_258, alt_body_259, alt_body_260,
                alt_body_261, alt_body_262, alt_body_263, alt_body_264,
                alt_body_265, alt_body_266, alt_body_267, alt_body_268,
                alt_body_269, alt_body_270, alt_body_271, alt_body_272,
                alt_body_273, alt_body_274, alt_body_275, alt_body_276,
                alt_body_277, alt_body_278, alt_body_279, alt_body_280,
                alt_body_281, alt_body_282, alt_body_283, alt_body_284,
                alt_body_285, alt_body_286, alt_body_287, alt_body_288,
                alt_body_289, alt_body_290, alt_body_291, alt_body_292,
                alt_body_293, alt_body_294, alt_body_295, alt_body_296,
                alt_body_297, alt_body_298, alt_body_299, alt_body_300,
                alt_body_301, alt_body_302, alt_body_303, alt_body_304,
                alt_body_305, alt_body_306, alt_body_307, alt_body_308,
                alt_body_309, alt_body_310, alt_body_311, alt_body_312,
                alt_body_313, alt_body_314, alt_body_315, alt_body_316,
                alt_body_317, alt_body_318, alt_body_319, alt_body_320,
                field_665_u32, field_666_u32, field_667_u32, field_668_u32,
                field_669_u32, field_670_u32, field_671_u32, field_672_u32,
                field_673_u32, field_674_u32, field_675_u32, field_676_u32,
                field_677_u32, field_678_u32, field_679_u32, field_680_u32,
                field_681_u32, field_682_u32, field_683_u32, field_684_u32,
                field_685_u32, field_686_u32, field_687_u32, field_688_u32,
                field_689_u32, field_690_u32, field_691_u32, field_692_u32,
                field_693_u32, field_694_u32, field_695_u32, field_696_u32,
                field_697_u32, field_698_u32, field_699_u32, field_700_u32,
                field_701_u32, field_702_u32, field_703_u32, field_704_u32,
                field_705_u32, field_706_u32, field_707_u32, field_708_u32,
                field_709_u32, field_710_u32, field_711_u32, field_712_u32,
                field_713_u32, field_714_u32, field_715_u32, field_716_u32,
                field_717_u32, field_718_u32, field_719_u32, field_720_u32,
                field_721_u32, field_722_u32, field_723_u32, field_724_u32,
                field_725_u32, field_726_u32, field_727_u32, field_728_u32, post_blob } => {
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
                if let Some(v) = field_108_u32 { v.write_to(w)?; }
                if let Some(v) = field_109_u32 { v.write_to(w)?; }
                if let Some(v) = field_110_u32 { v.write_to(w)?; }
                if let Some(v) = field_111_u32 { v.write_to(w)?; }
                if let Some(v) = field_112_u32 { v.write_to(w)?; }
                if let Some(v) = field_113_u32 { v.write_to(w)?; }
                if let Some(v) = field_114_u32 { v.write_to(w)?; }
                if let Some(v) = field_115_u32 { v.write_to(w)?; }
                if let Some(v) = field_116_u32 { v.write_to(w)?; }
                if let Some(v) = field_117_u32 { v.write_to(w)?; }
                if let Some(v) = field_118_u32 { v.write_to(w)?; }
                if let Some(v) = field_119_u32 { v.write_to(w)?; }
                if let Some(v) = field_120_u32 { v.write_to(w)?; }
                if let Some(v) = field_121_u32 { v.write_to(w)?; }
                if let Some(v) = field_122_u32 { v.write_to(w)?; }
                if let Some(v) = field_123_u32 { v.write_to(w)?; }
                if let Some(v) = field_124_u32 { v.write_to(w)?; }
                if let Some(v) = field_125_u32 { v.write_to(w)?; }
                if let Some(v) = field_126_u32 { v.write_to(w)?; }
                if let Some(v) = field_127_u32 { v.write_to(w)?; }
                if let Some(v) = field_128_u32 { v.write_to(w)?; }
                if let Some(v) = field_129_u32 { v.write_to(w)?; }
                if let Some(v) = field_130_u32 { v.write_to(w)?; }
                if let Some(v) = field_131_u32 { v.write_to(w)?; }
                if let Some(v) = field_132_u32 { v.write_to(w)?; }
                if let Some(v) = field_133_u32 { v.write_to(w)?; }
                if let Some(v) = field_134_u32 { v.write_to(w)?; }
                if let Some(v) = field_135_u32 { v.write_to(w)?; }
                if let Some(v) = field_136_u32 { v.write_to(w)?; }
                if let Some(v) = field_137_u32 { v.write_to(w)?; }
                if let Some(v) = field_138_u32 { v.write_to(w)?; }
                if let Some(v) = field_139_u32 { v.write_to(w)?; }
                if let Some(v) = field_140_u32 { v.write_to(w)?; }
                if let Some(v) = field_141_u32 { v.write_to(w)?; }
                if let Some(v) = field_142_u32 { v.write_to(w)?; }
                if let Some(v) = field_143_u32 { v.write_to(w)?; }
                if let Some(v) = field_144_u32 { v.write_to(w)?; }
                if let Some(v) = field_145_u32 { v.write_to(w)?; }
                if let Some(v) = field_146_u32 { v.write_to(w)?; }
                if let Some(v) = field_147_u32 { v.write_to(w)?; }
                if let Some(v) = field_148_u32 { v.write_to(w)?; }
                if let Some(v) = field_149_u32 { v.write_to(w)?; }
                if let Some(v) = field_150_u32 { v.write_to(w)?; }
                if let Some(v) = field_151_u32 { v.write_to(w)?; }
                if let Some(v) = field_152_u32 { v.write_to(w)?; }
                if let Some(v) = field_153_u32 { v.write_to(w)?; }
                if let Some(v) = field_154_u32 { v.write_to(w)?; }
                if let Some(v) = field_155_u32 { v.write_to(w)?; }
                if let Some(v) = field_156_u32 { v.write_to(w)?; }
                if let Some(v) = field_157_u32 { v.write_to(w)?; }
                if let Some(v) = field_158_u32 { v.write_to(w)?; }
                if let Some(v) = field_159_u32 { v.write_to(w)?; }
                if let Some(v) = field_160_u32 { v.write_to(w)?; }
                if let Some(v) = field_161_u32 { v.write_to(w)?; }
                if let Some(v) = field_162_u32 { v.write_to(w)?; }
                if let Some(v) = field_163_u32 { v.write_to(w)?; }
                if let Some(v) = field_164_u32 { v.write_to(w)?; }
                if let Some(v) = field_165_u32 { v.write_to(w)?; }
                if let Some(v) = field_166_u32 { v.write_to(w)?; }
                if let Some(v) = field_167_u32 { v.write_to(w)?; }
                if let Some(v) = field_168_u32 { v.write_to(w)?; }
                if let Some(v) = field_169_u32 { v.write_to(w)?; }
                if let Some(v) = field_170_u32 { v.write_to(w)?; }
                if let Some(v) = field_171_u32 { v.write_to(w)?; }
                if let Some(v) = field_172_u32 { v.write_to(w)?; }
                if let Some(v) = field_173_u32 { v.write_to(w)?; }
                if let Some(v) = field_174_u32 { v.write_to(w)?; }
                if let Some(v) = field_175_u32 { v.write_to(w)?; }
                if let Some(v) = field_176_u32 { v.write_to(w)?; }
                if let Some(v) = field_177_u32 { v.write_to(w)?; }
                if let Some(v) = field_178_u32 { v.write_to(w)?; }
                if let Some(v) = field_179_u32 { v.write_to(w)?; }
                if let Some(v) = field_180_u32 { v.write_to(w)?; }
                if let Some(v) = field_181_u32 { v.write_to(w)?; }
                if let Some(v) = field_182_u32 { v.write_to(w)?; }
                if let Some(v) = field_183_u32 { v.write_to(w)?; }
                if let Some(v) = field_184_u32 { v.write_to(w)?; }
                if let Some(v) = field_185_u32 { v.write_to(w)?; }
                if let Some(v) = field_186_u32 { v.write_to(w)?; }
                if let Some(v) = field_187_u32 { v.write_to(w)?; }
                if let Some(v) = field_188_u32 { v.write_to(w)?; }
                if let Some(v) = field_189_u32 { v.write_to(w)?; }
                if let Some(v) = field_190_u32 { v.write_to(w)?; }
                if let Some(v) = field_191_u32 { v.write_to(w)?; }
                if let Some(v) = field_192_u32 { v.write_to(w)?; }
                if let Some(v) = field_193_u32 { v.write_to(w)?; }
                if let Some(v) = field_194_u32 { v.write_to(w)?; }
                if let Some(v) = field_195_u32 { v.write_to(w)?; }
                if let Some(v) = field_196_u32 { v.write_to(w)?; }
                if let Some(v) = field_197_u32 { v.write_to(w)?; }
                if let Some(v) = field_198_u32 { v.write_to(w)?; }
                if let Some(v) = field_199_u32 { v.write_to(w)?; }
                if let Some(v) = field_200_u32 { v.write_to(w)?; }
                if let Some(v) = field_201_u32 { v.write_to(w)?; }
                if let Some(v) = field_202_u32 { v.write_to(w)?; }
                if let Some(v) = field_203_u32 { v.write_to(w)?; }
                if let Some(v) = field_204_u32 { v.write_to(w)?; }
                if let Some(v) = field_205_u32 { v.write_to(w)?; }
                if let Some(v) = field_206_u32 { v.write_to(w)?; }
                if let Some(v) = field_207_u32 { v.write_to(w)?; }
                if let Some(v) = field_208_u32 { v.write_to(w)?; }
                if let Some(v) = field_209_u32 { v.write_to(w)?; }
                if let Some(v) = field_210_u32 { v.write_to(w)?; }
                if let Some(v) = field_211_u32 { v.write_to(w)?; }
                if let Some(v) = field_212_u32 { v.write_to(w)?; }
                if let Some(v) = field_213_u32 { v.write_to(w)?; }
                if let Some(v) = field_214_u32 { v.write_to(w)?; }
                if let Some(v) = field_215_u32 { v.write_to(w)?; }
                if let Some(v) = field_216_u32 { v.write_to(w)?; }
                if let Some(v) = field_217_u32 { v.write_to(w)?; }
                if let Some(v) = field_218_u32 { v.write_to(w)?; }
                if let Some(v) = field_219_u32 { v.write_to(w)?; }
                if let Some(v) = field_220_u32 { v.write_to(w)?; }
                if let Some(v) = field_221_u32 { v.write_to(w)?; }
                if let Some(v) = field_222_u32 { v.write_to(w)?; }
                if let Some(v) = field_223_u32 { v.write_to(w)?; }
                if let Some(v) = field_224_u32 { v.write_to(w)?; }
                if let Some(v) = field_225_u32 { v.write_to(w)?; }
                if let Some(v) = field_226_u32 { v.write_to(w)?; }
                if let Some(v) = field_227_u32 { v.write_to(w)?; }
                if let Some(v) = field_228_u32 { v.write_to(w)?; }
                if let Some(v) = field_229_u32 { v.write_to(w)?; }
                if let Some(v) = field_230_u32 { v.write_to(w)?; }
                if let Some(v) = field_231_u32 { v.write_to(w)?; }
                if let Some(v) = field_232_u32 { v.write_to(w)?; }
                if let Some(v) = field_233_u32 { v.write_to(w)?; }
                if let Some(v) = field_234_u32 { v.write_to(w)?; }
                if let Some(v) = field_235_u32 { v.write_to(w)?; }
                if let Some(v) = field_236_u32 { v.write_to(w)?; }
                if let Some(v) = field_237_u32 { v.write_to(w)?; }
                if let Some(v) = field_238_u32 { v.write_to(w)?; }
                if let Some(v) = field_239_u32 { v.write_to(w)?; }
                if let Some(v) = field_240_u32 { v.write_to(w)?; }
                if let Some(v) = field_241_u32 { v.write_to(w)?; }
                if let Some(v) = field_242_u32 { v.write_to(w)?; }
                if let Some(v) = field_243_u32 { v.write_to(w)?; }
                if let Some(v) = field_244_u32 { v.write_to(w)?; }
                if let Some(v) = field_245_u32 { v.write_to(w)?; }
                if let Some(v) = field_246_u32 { v.write_to(w)?; }
                if let Some(v) = field_247_u32 { v.write_to(w)?; }
                if let Some(v) = field_248_u32 { v.write_to(w)?; }
                if let Some(v) = field_249_u32 { v.write_to(w)?; }
                if let Some(v) = field_250_u32 { v.write_to(w)?; }
                if let Some(v) = field_251_u32 { v.write_to(w)?; }
                if let Some(v) = field_252_u32 { v.write_to(w)?; }
                if let Some(v) = field_253_u32 { v.write_to(w)?; }
                if let Some(v) = field_254_u32 { v.write_to(w)?; }
                if let Some(v) = field_255_u32 { v.write_to(w)?; }
                if let Some(v) = field_256_u32 { v.write_to(w)?; }
                if let Some(v) = field_257_u32 { v.write_to(w)?; }
                if let Some(v) = field_258_u32 { v.write_to(w)?; }
                if let Some(v) = field_259_u32 { v.write_to(w)?; }
                if let Some(v) = field_260_u32 { v.write_to(w)?; }
                if let Some(v) = field_261_u32 { v.write_to(w)?; }
                if let Some(v) = field_262_u32 { v.write_to(w)?; }
                if let Some(v) = field_263_u32 { v.write_to(w)?; }
                if let Some(v) = field_264_u32 { v.write_to(w)?; }
                if let Some(v) = field_265_u32 { v.write_to(w)?; }
                if let Some(v) = field_266_u32 { v.write_to(w)?; }
                if let Some(v) = field_267_u32 { v.write_to(w)?; }
                if let Some(v) = field_268_u32 { v.write_to(w)?; }
                if let Some(v) = field_269_u32 { v.write_to(w)?; }
                if let Some(v) = field_270_u32 { v.write_to(w)?; }
                if let Some(v) = field_271_u32 { v.write_to(w)?; }
                if let Some(v) = field_272_u32 { v.write_to(w)?; }
                if let Some(v) = field_273_u32 { v.write_to(w)?; }
                if let Some(v) = field_274_u32 { v.write_to(w)?; }
                if let Some(v) = field_275_u32 { v.write_to(w)?; }
                if let Some(v) = field_276_u32 { v.write_to(w)?; }
                if let Some(v) = field_277_u32 { v.write_to(w)?; }
                if let Some(v) = field_278_u32 { v.write_to(w)?; }
                if let Some(v) = field_279_u32 { v.write_to(w)?; }
                if let Some(v) = field_280_u32 { v.write_to(w)?; }
                if let Some(v) = field_281_u32 { v.write_to(w)?; }
                if let Some(v) = field_282_u32 { v.write_to(w)?; }
                if let Some(v) = field_283_u32 { v.write_to(w)?; }
                if let Some(v) = field_284_u32 { v.write_to(w)?; }
                if let Some(v) = field_285_u32 { v.write_to(w)?; }
                if let Some(v) = field_286_u32 { v.write_to(w)?; }
                if let Some(v) = field_287_u32 { v.write_to(w)?; }
                if let Some(v) = field_288_u32 { v.write_to(w)?; }
                if let Some(v) = field_289_u32 { v.write_to(w)?; }
                if let Some(v) = field_290_u32 { v.write_to(w)?; }
                if let Some(v) = field_291_u32 { v.write_to(w)?; }
                if let Some(v) = field_292_u32 { v.write_to(w)?; }
                if let Some(v) = field_293_u32 { v.write_to(w)?; }
                if let Some(v) = field_294_u32 { v.write_to(w)?; }
                if let Some(v) = field_295_u32 { v.write_to(w)?; }
                if let Some(v) = field_296_u32 { v.write_to(w)?; }
                if let Some(v) = field_297_u32 { v.write_to(w)?; }
                if let Some(v) = field_298_u32 { v.write_to(w)?; }
                if let Some(v) = field_299_u32 { v.write_to(w)?; }
                if let Some(v) = field_300_u32 { v.write_to(w)?; }
                if let Some(v) = field_301_u32 { v.write_to(w)?; }
                if let Some(v) = field_302_u32 { v.write_to(w)?; }
                if let Some(v) = field_303_u32 { v.write_to(w)?; }
                if let Some(v) = field_304_u32 { v.write_to(w)?; }
                if let Some(v) = field_305_u32 { v.write_to(w)?; }
                if let Some(v) = field_306_u32 { v.write_to(w)?; }
                if let Some(v) = field_307_u32 { v.write_to(w)?; }
                if let Some(v) = field_308_u32 { v.write_to(w)?; }
                if let Some(v) = field_309_u32 { v.write_to(w)?; }
                if let Some(v) = field_310_u32 { v.write_to(w)?; }
                if let Some(v) = field_311_u32 { v.write_to(w)?; }
                if let Some(v) = field_312_u32 { v.write_to(w)?; }
                if let Some(v) = field_313_u32 { v.write_to(w)?; }
                if let Some(v) = field_314_u32 { v.write_to(w)?; }
                if let Some(v) = field_315_u32 { v.write_to(w)?; }
                if let Some(v) = field_316_u32 { v.write_to(w)?; }
                if let Some(v) = field_317_u32 { v.write_to(w)?; }
                if let Some(v) = field_318_u32 { v.write_to(w)?; }
                if let Some(v) = field_319_u32 { v.write_to(w)?; }
                if let Some(v) = field_320_u32 { v.write_to(w)?; }
                if let Some(v) = field_321_u32 { v.write_to(w)?; }
                if let Some(v) = field_322_u32 { v.write_to(w)?; }
                if let Some(v) = field_323_u32 { v.write_to(w)?; }
                if let Some(v) = field_324_u32 { v.write_to(w)?; }
                if let Some(v) = field_325_u32 { v.write_to(w)?; }
                if let Some(v) = field_326_u32 { v.write_to(w)?; }
                if let Some(v) = field_327_u32 { v.write_to(w)?; }
                if let Some(v) = field_328_u32 { v.write_to(w)?; }
                if let Some(v) = field_329_u32 { v.write_to(w)?; }
                if let Some(v) = field_330_u32 { v.write_to(w)?; }
                if let Some(v) = field_331_u32 { v.write_to(w)?; }
                if let Some(v) = field_332_u32 { v.write_to(w)?; }
                if let Some(v) = field_333_u32 { v.write_to(w)?; }
                if let Some(v) = field_334_u32 { v.write_to(w)?; }
                if let Some(v) = field_335_u32 { v.write_to(w)?; }
                if let Some(v) = field_336_u32 { v.write_to(w)?; }
                if let Some(v) = field_337_u32 { v.write_to(w)?; }
                if let Some(v) = field_338_u32 { v.write_to(w)?; }
                if let Some(v) = field_339_u32 { v.write_to(w)?; }
                if let Some(v) = field_340_u32 { v.write_to(w)?; }
                if let Some(v) = field_341_u32 { v.write_to(w)?; }
                if let Some(v) = field_342_u32_count { v.write_to(w)?; }
                if let Some(v) = field_343_u8_flag { v.write_to(w)?; }
                if let Some(v) = field_344_u32 { v.write_to(w)?; }
                if let Some(v) = field_345_u32 { v.write_to(w)?; }
                if let Some(v) = field_346_u32 { v.write_to(w)?; }
                if let Some(v) = field_347_u32 { v.write_to(w)?; }
                if let Some(v) = field_348_u32 { v.write_to(w)?; }
                if let Some(v) = field_349_u32 { v.write_to(w)?; }
                if let Some(v) = field_350_u32 { v.write_to(w)?; }
                if let Some(v) = field_351_u32 { v.write_to(w)?; }
                if let Some(v) = field_352_u32 { v.write_to(w)?; }
                if let Some(v) = field_353_u32 { v.write_to(w)?; }
                if let Some(v) = field_354_u32 { v.write_to(w)?; }
                if let Some(v) = field_355_u32 { v.write_to(w)?; }
                if let Some(v) = field_356_u32 { v.write_to(w)?; }
                if let Some(v) = field_357_u32 { v.write_to(w)?; }
                if let Some(v) = field_358_u32 { v.write_to(w)?; }
                if let Some(v) = field_359_u32 { v.write_to(w)?; }
                if let Some(v) = field_360_u32 { v.write_to(w)?; }
                if let Some(v) = field_361_u32 { v.write_to(w)?; }
                if let Some(v) = field_362_u32 { v.write_to(w)?; }
                if let Some(v) = field_363_u32 { v.write_to(w)?; }
                if let Some(v) = field_364_u32 { v.write_to(w)?; }
                if let Some(v) = field_365_u32 { v.write_to(w)?; }
                if let Some(v) = field_366_u32 { v.write_to(w)?; }
                if let Some(v) = field_367_u32 { v.write_to(w)?; }
                if let Some(v) = field_368_u32 { v.write_to(w)?; }
                if let Some(v) = field_369_u32 { v.write_to(w)?; }
                if let Some(v) = field_370_u32 { v.write_to(w)?; }
                if let Some(v) = field_371_u32 { v.write_to(w)?; }
                if let Some(v) = field_372_u32 { v.write_to(w)?; }
                if let Some(v) = field_373_u32 { v.write_to(w)?; }
                if let Some(v) = field_374_u32 { v.write_to(w)?; }
                if let Some(v) = field_375_u32 { v.write_to(w)?; }
                if let Some(v) = field_376_u32 { v.write_to(w)?; }
                if let Some(v) = field_377_u32 { v.write_to(w)?; }
                if let Some(v) = field_378_u32 { v.write_to(w)?; }
                if let Some(v) = field_379_u32 { v.write_to(w)?; }
                if let Some(v) = field_380_u32 { v.write_to(w)?; }
                if let Some(v) = field_381_u32 { v.write_to(w)?; }
                if let Some(v) = field_382_u32 { v.write_to(w)?; }
                if let Some(v) = field_383_u32 { v.write_to(w)?; }
                if let Some(v) = field_384_u32 { v.write_to(w)?; }
                if let Some(v) = field_385_u32 { v.write_to(w)?; }
                if let Some(v) = field_386_u32 { v.write_to(w)?; }
                if let Some(v) = field_387_u32 { v.write_to(w)?; }
                if let Some(v) = field_388_u32 { v.write_to(w)?; }
                if let Some(v) = field_389_u32 { v.write_to(w)?; }
                if let Some(v) = field_390_u32 { v.write_to(w)?; }
                if let Some(v) = field_391_u32 { v.write_to(w)?; }
                if let Some(v) = field_392_u32 { v.write_to(w)?; }
                if let Some(v) = field_393_u32 { v.write_to(w)?; }
                if let Some(v) = field_394_u32 { v.write_to(w)?; }
                if let Some(v) = field_395_u32 { v.write_to(w)?; }
                if let Some(v) = field_396_u32 { v.write_to(w)?; }
                if let Some(v) = field_397_u32 { v.write_to(w)?; }
                if let Some(v) = field_398_u32 { v.write_to(w)?; }
                if let Some(v) = field_399_u32 { v.write_to(w)?; }
                if let Some(v) = field_400_u32 { v.write_to(w)?; }
                if let Some(v) = field_401_u32 { v.write_to(w)?; }
                if let Some(v) = field_402_u32 { v.write_to(w)?; }
                if let Some(v) = field_403_u32 { v.write_to(w)?; }
                if let Some(v) = field_404_u32 { v.write_to(w)?; }
                if let Some(v) = field_405_u32 { v.write_to(w)?; }
                if let Some(v) = field_406_u32 { v.write_to(w)?; }
                if let Some(v) = field_407_u32 { v.write_to(w)?; }
                if let Some(v) = field_408_u32 { v.write_to(w)?; }
                if let Some(v) = field_409_u32 { v.write_to(w)?; }
                if let Some(v) = field_410_u32 { v.write_to(w)?; }
                if let Some(v) = field_411_u32 { v.write_to(w)?; }
                if let Some(v) = field_412_u32 { v.write_to(w)?; }
                if let Some(v) = field_413_u32 { v.write_to(w)?; }
                if let Some(v) = field_414_u32 { v.write_to(w)?; }
                if let Some(v) = field_415_u32 { v.write_to(w)?; }
                if let Some(v) = field_416_u32 { v.write_to(w)?; }
                if let Some(v) = field_417_u32 { v.write_to(w)?; }
                if let Some(v) = field_418_u32 { v.write_to(w)?; }
                if let Some(v) = field_419_u32 { v.write_to(w)?; }
                if let Some(v) = field_420_u32 { v.write_to(w)?; }
                if let Some(v) = field_421_u32 { v.write_to(w)?; }
                if let Some(v) = field_422_u32 { v.write_to(w)?; }
                if let Some(v) = field_423_u32 { v.write_to(w)?; }
                if let Some(v) = field_424_u32 { v.write_to(w)?; }
                if let Some(v) = field_425_u32 { v.write_to(w)?; }
                if let Some(v) = field_426_u32 { v.write_to(w)?; }
                if let Some(v) = field_427_u32 { v.write_to(w)?; }
                if let Some(v) = field_428_u32 { v.write_to(w)?; }
                if let Some(v) = field_429_u32 { v.write_to(w)?; }
                if let Some(v) = field_430_u32 { v.write_to(w)?; }
                if let Some(v) = field_431_u32 { v.write_to(w)?; }
                if let Some(v) = field_432_u32 { v.write_to(w)?; }
                if let Some(v) = field_433_u32 { v.write_to(w)?; }
                if let Some(v) = field_434_u32 { v.write_to(w)?; }
                if let Some(v) = field_435_u32 { v.write_to(w)?; }
                if let Some(v) = field_436_u32 { v.write_to(w)?; }
                if let Some(v) = field_437_u32 { v.write_to(w)?; }
                if let Some(v) = field_438_u32 { v.write_to(w)?; }
                if let Some(v) = field_439_u32 { v.write_to(w)?; }
                if let Some(v) = field_440_u32 { v.write_to(w)?; }
                if let Some(v) = field_441_u32 { v.write_to(w)?; }
                if let Some(v) = field_442_u32 { v.write_to(w)?; }
                if let Some(v) = field_443_u32 { v.write_to(w)?; }
                if let Some(v) = field_444_u32 { v.write_to(w)?; }
                if let Some(v) = field_445_u32 { v.write_to(w)?; }
                if let Some(v) = field_446_u32 { v.write_to(w)?; }
                if let Some(v) = field_447_u32 { v.write_to(w)?; }
                if let Some(v) = field_448_u32 { v.write_to(w)?; }
                if let Some(v) = field_449_u32 { v.write_to(w)?; }
                if let Some(v) = field_450_u32 { v.write_to(w)?; }
                if let Some(v) = field_451_u32 { v.write_to(w)?; }
                if let Some(v) = field_452_u32 { v.write_to(w)?; }
                if let Some(v) = field_453_u32 { v.write_to(w)?; }
                if let Some(v) = field_454_u32 { v.write_to(w)?; }
                if let Some(v) = field_455_u32 { v.write_to(w)?; }
                if let Some(v) = field_456_u32 { v.write_to(w)?; }
                if let Some(v) = field_457_u32 { v.write_to(w)?; }
                if let Some(v) = field_458_u32 { v.write_to(w)?; }
                if let Some(v) = field_459_u32 { v.write_to(w)?; }
                if let Some(v) = field_460_u32 { v.write_to(w)?; }
                if let Some(v) = field_461_u32 { v.write_to(w)?; }
                if let Some(v) = field_462_u32 { v.write_to(w)?; }
                if let Some(v) = field_463_u32 { v.write_to(w)?; }
                if let Some(v) = field_464_u32 { v.write_to(w)?; }
                if let Some(v) = field_465_u32 { v.write_to(w)?; }
                if let Some(v) = field_466_u32 { v.write_to(w)?; }
                if let Some(v) = field_467_u32 { v.write_to(w)?; }
                if let Some(v) = field_468_u32 { v.write_to(w)?; }
                if let Some(v) = field_469_u32 { v.write_to(w)?; }
                if let Some(v) = field_470_u32 { v.write_to(w)?; }
                if let Some(v) = field_471_u32 { v.write_to(w)?; }
                if let Some(v) = field_472_u32 { v.write_to(w)?; }
                if let Some(v) = field_473_u32 { v.write_to(w)?; }
                if let Some(v) = field_474_u32 { v.write_to(w)?; }
                if let Some(v) = field_475_u32 { v.write_to(w)?; }
                if let Some(v) = field_476_u32 { v.write_to(w)?; }
                if let Some(v) = field_477_u32 { v.write_to(w)?; }
                if let Some(v) = field_478_u32 { v.write_to(w)?; }
                if let Some(v) = field_479_u32 { v.write_to(w)?; }
                if let Some(v) = field_480_u32 { v.write_to(w)?; }
                if let Some(v) = field_481_u32 { v.write_to(w)?; }
                if let Some(v) = field_482_u32 { v.write_to(w)?; }
                if let Some(v) = field_483_u32 { v.write_to(w)?; }
                if let Some(v) = field_484_u32 { v.write_to(w)?; }
                if let Some(v) = field_485_u32 { v.write_to(w)?; }
                if let Some(v) = field_486_u32 { v.write_to(w)?; }
                if let Some(v) = field_487_u32 { v.write_to(w)?; }
                if let Some(v) = field_488_u32 { v.write_to(w)?; }
                if let Some(v) = field_489_u32 { v.write_to(w)?; }
                if let Some(v) = field_490_u32 { v.write_to(w)?; }
                if let Some(v) = field_491_u32 { v.write_to(w)?; }
                if let Some(v) = field_492_u32 { v.write_to(w)?; }
                if let Some(v) = field_493_u32 { v.write_to(w)?; }
                if let Some(v) = field_494_u32 { v.write_to(w)?; }
                if let Some(v) = field_495_u32 { v.write_to(w)?; }
                if let Some(v) = field_496_u32 { v.write_to(w)?; }
                if let Some(v) = field_497_u32 { v.write_to(w)?; }
                if let Some(v) = field_498_u32 { v.write_to(w)?; }
                if let Some(v) = field_499_u32 { v.write_to(w)?; }
                if let Some(v) = field_500_u32 { v.write_to(w)?; }
                if let Some(v) = field_501_u32 { v.write_to(w)?; }
                if let Some(v) = field_502_u32 { v.write_to(w)?; }
                if let Some(v) = field_503_u32 { v.write_to(w)?; }
                if let Some(v) = field_504_u32 { v.write_to(w)?; }
                if let Some(v) = field_505_u32 { v.write_to(w)?; }
                if let Some(v) = field_506_u32 { v.write_to(w)?; }
                if let Some(v) = field_507_u32 { v.write_to(w)?; }
                if let Some(v) = field_508_u32 { v.write_to(w)?; }
                if let Some(v) = field_509_u32 { v.write_to(w)?; }
                if let Some(v) = field_510_u32 { v.write_to(w)?; }
                if let Some(v) = field_511_u32 { v.write_to(w)?; }
                if let Some(v) = field_512_u32 { v.write_to(w)?; }
                if let Some(v) = field_513_u32 { v.write_to(w)?; }
                if let Some(v) = field_514_u32 { v.write_to(w)?; }
                if let Some(v) = field_515_u32 { v.write_to(w)?; }
                if let Some(v) = field_516_u32 { v.write_to(w)?; }
                if let Some(v) = field_517_u32 { v.write_to(w)?; }
                if let Some(v) = field_518_u32 { v.write_to(w)?; }
                if let Some(v) = field_519_u32 { v.write_to(w)?; }
                if let Some(v) = field_520_u32 { v.write_to(w)?; }
                if let Some(v) = field_521_u32 { v.write_to(w)?; }
                if let Some(v) = field_522_u32 { v.write_to(w)?; }
                if let Some(v) = field_523_u32 { v.write_to(w)?; }
                if let Some(v) = field_524_u32 { v.write_to(w)?; }
                if let Some(v) = field_525_u32 { v.write_to(w)?; }
                if let Some(v) = field_526_u32 { v.write_to(w)?; }
                if let Some(v) = field_527_u32 { v.write_to(w)?; }
                if let Some(v) = field_528_u32 { v.write_to(w)?; }
                if let Some(v) = field_529_u32 { v.write_to(w)?; }
                if let Some(v) = field_530_u32 { v.write_to(w)?; }
                if let Some(v) = field_531_u32 { v.write_to(w)?; }
                if let Some(v) = field_532_u32 { v.write_to(w)?; }
                if let Some(v) = field_533_u32 { v.write_to(w)?; }
                if let Some(v) = field_534_u32 { v.write_to(w)?; }
                if let Some(v) = field_535_u32 { v.write_to(w)?; }
                if let Some(v) = field_536_u32 { v.write_to(w)?; }
                if let Some(v) = field_537_u32 { v.write_to(w)?; }
                if let Some(v) = field_538_u32 { v.write_to(w)?; }
                if let Some(v) = field_539_u32 { v.write_to(w)?; }
                if let Some(v) = field_540_u32 { v.write_to(w)?; }
                if let Some(v) = field_541_u32 { v.write_to(w)?; }
                if let Some(v) = field_542_u32 { v.write_to(w)?; }
                if let Some(v) = field_543_u32 { v.write_to(w)?; }
                if let Some(v) = field_544_u32 { v.write_to(w)?; }
                if let Some(v) = field_545_u32 { v.write_to(w)?; }
                if let Some(v) = field_546_u32 { v.write_to(w)?; }
                if let Some(v) = field_547_u32 { v.write_to(w)?; }
                if let Some(v) = field_548_u32 { v.write_to(w)?; }
                if let Some(v) = field_549_u32 { v.write_to(w)?; }
                if let Some(v) = field_550_u32 { v.write_to(w)?; }
                if let Some(v) = field_551_u32 { v.write_to(w)?; }
                if let Some(v) = field_552_u32 { v.write_to(w)?; }
                if let Some(v) = field_553_u32 { v.write_to(w)?; }
                if let Some(v) = field_554_u32 { v.write_to(w)?; }
                if let Some(v) = field_555_u32 { v.write_to(w)?; }
                if let Some(v) = field_556_u32 { v.write_to(w)?; }
                if let Some(v) = field_557_u32 { v.write_to(w)?; }
                if let Some(v) = field_558_u32 { v.write_to(w)?; }
                if let Some(v) = field_559_u32 { v.write_to(w)?; }
                if let Some(v) = field_560_u32 { v.write_to(w)?; }
                if let Some(v) = field_561_u32 { v.write_to(w)?; }
                if let Some(v) = field_562_u32 { v.write_to(w)?; }
                if let Some(v) = field_563_u32 { v.write_to(w)?; }
                if let Some(v) = field_564_u32 { v.write_to(w)?; }
                if let Some(v) = field_565_u32 { v.write_to(w)?; }
                if let Some(v) = field_566_u32 { v.write_to(w)?; }
                if let Some(v) = field_567_u32 { v.write_to(w)?; }
                if let Some(v) = field_568_u32 { v.write_to(w)?; }
                if let Some(v) = field_569_u32 { v.write_to(w)?; }
                if let Some(v) = field_570_u32 { v.write_to(w)?; }
                if let Some(v) = field_571_u32 { v.write_to(w)?; }
                if let Some(v) = field_572_u32 { v.write_to(w)?; }
                if let Some(v) = field_573_u32 { v.write_to(w)?; }
                if let Some(v) = field_574_u32 { v.write_to(w)?; }
                if let Some(v) = field_575_u32 { v.write_to(w)?; }
                if let Some(v) = field_576_u32 { v.write_to(w)?; }
                if let Some(v) = field_577_u32 { v.write_to(w)?; }
                if let Some(v) = field_578_u32 { v.write_to(w)?; }
                if let Some(v) = field_579_u32 { v.write_to(w)?; }
                if let Some(v) = field_580_u32 { v.write_to(w)?; }
                if let Some(v) = field_581_u32 { v.write_to(w)?; }
                if let Some(v) = field_582_u32 { v.write_to(w)?; }
                if let Some(v) = field_583_u32 { v.write_to(w)?; }
                if let Some(v) = field_584_u32 { v.write_to(w)?; }
                if let Some(v) = field_585_u32 { v.write_to(w)?; }
                if let Some(v) = field_586_u32 { v.write_to(w)?; }
                if let Some(v) = field_587_u32 { v.write_to(w)?; }
                if let Some(v) = field_588_u32 { v.write_to(w)?; }
                if let Some(v) = field_589_u32 { v.write_to(w)?; }
                if let Some(v) = field_590_u32 { v.write_to(w)?; }
                if let Some(v) = field_591_u32 { v.write_to(w)?; }
                if let Some(v) = field_592_u32 { v.write_to(w)?; }
                if let Some(v) = field_593_u32 { v.write_to(w)?; }
                if let Some(v) = field_594_u32 { v.write_to(w)?; }
                if let Some(v) = field_595_u32 { v.write_to(w)?; }
                if let Some(v) = field_596_u32 { v.write_to(w)?; }
                if let Some(v) = field_597_u32 { v.write_to(w)?; }
                if let Some(v) = field_598_u32 { v.write_to(w)?; }
                if let Some(v) = field_599_u32 { v.write_to(w)?; }
                if let Some(v) = field_600_u32 { v.write_to(w)?; }
                if let Some(v) = field_601_u32 { v.write_to(w)?; }
                if let Some(v) = field_602_u32 { v.write_to(w)?; }
                if let Some(v) = field_603_u32 { v.write_to(w)?; }
                if let Some(v) = field_604_u32 { v.write_to(w)?; }
                if let Some(v) = field_605_u32 { v.write_to(w)?; }
                if let Some(v) = field_606_u32 { v.write_to(w)?; }
                if let Some(v) = field_607_u32 { v.write_to(w)?; }
                if let Some(v) = field_608_u32 { v.write_to(w)?; }
                if let Some(v) = field_609_u32 { v.write_to(w)?; }
                if let Some(v) = field_610_u32 { v.write_to(w)?; }
                if let Some(v) = field_611_u32 { v.write_to(w)?; }
                if let Some(v) = field_612_u32 { v.write_to(w)?; }
                if let Some(v) = field_613_u32 { v.write_to(w)?; }
                if let Some(v) = field_614_u32 { v.write_to(w)?; }
                if let Some(v) = field_615_u32 { v.write_to(w)?; }
                if let Some(v) = field_616_u32 { v.write_to(w)?; }
                if let Some(v) = field_617_u32 { v.write_to(w)?; }
                if let Some(v) = field_618_u32 { v.write_to(w)?; }
                if let Some(v) = field_619_u32 { v.write_to(w)?; }
                if let Some(v) = field_620_u32 { v.write_to(w)?; }
                if let Some(v) = field_621_u32 { v.write_to(w)?; }
                if let Some(v) = field_622_u32 { v.write_to(w)?; }
                if let Some(v) = field_623_u32 { v.write_to(w)?; }
                if let Some(v) = field_624_u32 { v.write_to(w)?; }
                if let Some(v) = field_625_u32 { v.write_to(w)?; }
                if let Some(v) = field_626_u32 { v.write_to(w)?; }
                if let Some(v) = field_627_u32 { v.write_to(w)?; }
                if let Some(v) = field_628_u32 { v.write_to(w)?; }
                if let Some(v) = field_629_u32 { v.write_to(w)?; }
                if let Some(v) = field_630_u32 { v.write_to(w)?; }
                if let Some(v) = field_631_u32 { v.write_to(w)?; }
                if let Some(v) = field_632_u32 { v.write_to(w)?; }
                if let Some(v) = field_633_u32 { v.write_to(w)?; }
                if let Some(v) = field_634_u32 { v.write_to(w)?; }
                if let Some(v) = field_635_u32 { v.write_to(w)?; }
                if let Some(v) = field_636_u32 { v.write_to(w)?; }
                if let Some(v) = field_637_u32 { v.write_to(w)?; }
                if let Some(v) = field_638_u32 { v.write_to(w)?; }
                if let Some(v) = field_639_u32 { v.write_to(w)?; }
                if let Some(v) = field_640_u32 { v.write_to(w)?; }
                if let Some(v) = field_641_u32 { v.write_to(w)?; }
                if let Some(v) = field_642_u32 { v.write_to(w)?; }
                if let Some(v) = field_643_u32 { v.write_to(w)?; }
                if let Some(v) = field_644_u32 { v.write_to(w)?; }
                if let Some(v) = field_645_u32 { v.write_to(w)?; }
                if let Some(v) = field_646_u32 { v.write_to(w)?; }
                if let Some(v) = field_647_u32 { v.write_to(w)?; }
                if let Some(v) = field_648_u32 { v.write_to(w)?; }
                if let Some(v) = field_649_u32 { v.write_to(w)?; }
                if let Some(v) = field_650_u32 { v.write_to(w)?; }
                if let Some(v) = field_651_u32 { v.write_to(w)?; }
                if let Some(v) = field_652_u32 { v.write_to(w)?; }
                if let Some(v) = field_653_u32 { v.write_to(w)?; }
                if let Some(v) = field_654_u32 { v.write_to(w)?; }
                if let Some(v) = field_655_u32 { v.write_to(w)?; }
                if let Some(v) = field_656_u32 { v.write_to(w)?; }
                if let Some(v) = field_657_u32 { v.write_to(w)?; }
                if let Some(v) = field_658_u32 { v.write_to(w)?; }
                if let Some(v) = field_659_u32 { v.write_to(w)?; }
                if let Some(v) = field_660_u32 { v.write_to(w)?; }
                if let Some(v) = field_661_u32 { v.write_to(w)?; }
                if let Some(v) = field_662_u32 { v.write_to(w)?; }
                if let Some(v) = field_663_u32 { v.write_to(w)?; }
                if let Some(v) = field_664_u32 { v.write_to(w)?; }
                if let Some(v) = alt_trigger_count { v.write_to(w)?; }
                if let Some(v) = alt_trigger_flag { v.write_to(w)?; }
                if let Some(s) = alt_trigger_name { s.write_to(w)?; }
                if let Some(v) = alt_inner_count { v.write_to(w)?; }
                if let Some(s) = alt_inner_name { s.write_to(w)?; }
                if let Some(v) = alt_inner_flag { v.write_to(w)?; }
                if let Some(v) = alt_body_001 { v.write_to(w)?; }
                if let Some(v) = alt_body_002 { v.write_to(w)?; }
                if let Some(v) = alt_body_003 { v.write_to(w)?; }
                if let Some(v) = alt_body_004 { v.write_to(w)?; }
                if let Some(v) = alt_body_005 { v.write_to(w)?; }
                if let Some(v) = alt_body_006 { v.write_to(w)?; }
                if let Some(v) = alt_body_007 { v.write_to(w)?; }
                if let Some(v) = alt_body_008 { v.write_to(w)?; }
                if let Some(v) = alt_body_009 { v.write_to(w)?; }
                if let Some(v) = alt_body_010 { v.write_to(w)?; }
                if let Some(v) = alt_body_011 { v.write_to(w)?; }
                if let Some(v) = alt_body_012 { v.write_to(w)?; }
                if let Some(v) = alt_body_013 { v.write_to(w)?; }
                if let Some(v) = alt_body_014 { v.write_to(w)?; }
                if let Some(v) = alt_body_015 { v.write_to(w)?; }
                if let Some(v) = alt_body_016 { v.write_to(w)?; }
                if let Some(v) = alt_body_017 { v.write_to(w)?; }
                if let Some(v) = alt_body_018 { v.write_to(w)?; }
                if let Some(v) = alt_body_019 { v.write_to(w)?; }
                if let Some(v) = alt_body_020 { v.write_to(w)?; }
                if let Some(v) = alt_body_021 { v.write_to(w)?; }
                if let Some(v) = alt_body_022 { v.write_to(w)?; }
                if let Some(v) = alt_body_023 { v.write_to(w)?; }
                if let Some(v) = alt_body_024 { v.write_to(w)?; }
                if let Some(v) = alt_body_025 { v.write_to(w)?; }
                if let Some(v) = alt_body_026 { v.write_to(w)?; }
                if let Some(v) = alt_body_027 { v.write_to(w)?; }
                if let Some(v) = alt_body_028 { v.write_to(w)?; }
                if let Some(v) = alt_body_029 { v.write_to(w)?; }
                if let Some(v) = alt_body_030 { v.write_to(w)?; }
                if let Some(v) = alt_body_031 { v.write_to(w)?; }
                if let Some(v) = alt_body_032 { v.write_to(w)?; }
                if let Some(v) = alt_body_033 { v.write_to(w)?; }
                if let Some(v) = alt_body_034 { v.write_to(w)?; }
                if let Some(v) = alt_body_035 { v.write_to(w)?; }
                if let Some(v) = alt_body_036 { v.write_to(w)?; }
                if let Some(v) = alt_body_037 { v.write_to(w)?; }
                if let Some(v) = alt_body_038 { v.write_to(w)?; }
                if let Some(v) = alt_body_039 { v.write_to(w)?; }
                if let Some(v) = alt_body_040 { v.write_to(w)?; }
                if let Some(v) = alt_body_041 { v.write_to(w)?; }
                if let Some(v) = alt_body_042 { v.write_to(w)?; }
                if let Some(v) = alt_body_043 { v.write_to(w)?; }
                if let Some(v) = alt_body_044 { v.write_to(w)?; }
                if let Some(v) = alt_body_045 { v.write_to(w)?; }
                if let Some(v) = alt_body_046 { v.write_to(w)?; }
                if let Some(v) = alt_body_047 { v.write_to(w)?; }
                if let Some(v) = alt_body_048 { v.write_to(w)?; }
                if let Some(v) = alt_body_049 { v.write_to(w)?; }
                if let Some(v) = alt_body_050 { v.write_to(w)?; }
                if let Some(v) = alt_body_051 { v.write_to(w)?; }
                if let Some(v) = alt_body_052 { v.write_to(w)?; }
                if let Some(v) = alt_body_053 { v.write_to(w)?; }
                if let Some(v) = alt_body_054 { v.write_to(w)?; }
                if let Some(v) = alt_body_055 { v.write_to(w)?; }
                if let Some(v) = alt_body_056 { v.write_to(w)?; }
                if let Some(v) = alt_body_057 { v.write_to(w)?; }
                if let Some(v) = alt_body_058 { v.write_to(w)?; }
                if let Some(v) = alt_body_059 { v.write_to(w)?; }
                if let Some(v) = alt_body_060 { v.write_to(w)?; }
                if let Some(v) = alt_body_061 { v.write_to(w)?; }
                if let Some(v) = alt_body_062 { v.write_to(w)?; }
                if let Some(v) = alt_body_063 { v.write_to(w)?; }
                if let Some(v) = alt_body_064 { v.write_to(w)?; }
                if let Some(v) = alt_body_065 { v.write_to(w)?; }
                if let Some(v) = alt_body_066 { v.write_to(w)?; }
                if let Some(v) = alt_body_067 { v.write_to(w)?; }
                if let Some(v) = alt_body_068 { v.write_to(w)?; }
                if let Some(v) = alt_body_069 { v.write_to(w)?; }
                if let Some(v) = alt_body_070 { v.write_to(w)?; }
                if let Some(v) = alt_body_071 { v.write_to(w)?; }
                if let Some(v) = alt_body_072 { v.write_to(w)?; }
                if let Some(v) = alt_body_073 { v.write_to(w)?; }
                if let Some(v) = alt_body_074 { v.write_to(w)?; }
                if let Some(v) = alt_body_075 { v.write_to(w)?; }
                if let Some(v) = alt_body_076 { v.write_to(w)?; }
                if let Some(v) = alt_body_077 { v.write_to(w)?; }
                if let Some(v) = alt_body_078 { v.write_to(w)?; }
                if let Some(v) = alt_body_079 { v.write_to(w)?; }
                if let Some(v) = alt_body_080 { v.write_to(w)?; }
                if let Some(v) = alt_body_081 { v.write_to(w)?; }
                if let Some(v) = alt_body_082 { v.write_to(w)?; }
                if let Some(v) = alt_body_083 { v.write_to(w)?; }
                if let Some(v) = alt_body_084 { v.write_to(w)?; }
                if let Some(v) = alt_body_085 { v.write_to(w)?; }
                if let Some(v) = alt_body_086 { v.write_to(w)?; }
                if let Some(v) = alt_body_087 { v.write_to(w)?; }
                if let Some(v) = alt_body_088 { v.write_to(w)?; }
                if let Some(v) = alt_body_089 { v.write_to(w)?; }
                if let Some(v) = alt_body_090 { v.write_to(w)?; }
                if let Some(v) = alt_body_091 { v.write_to(w)?; }
                if let Some(v) = alt_body_092 { v.write_to(w)?; }
                if let Some(v) = alt_body_093 { v.write_to(w)?; }
                if let Some(v) = alt_body_094 { v.write_to(w)?; }
                if let Some(v) = alt_body_095 { v.write_to(w)?; }
                if let Some(v) = alt_body_096 { v.write_to(w)?; }
                if let Some(v) = alt_body_097 { v.write_to(w)?; }
                if let Some(v) = alt_body_098 { v.write_to(w)?; }
                if let Some(v) = alt_body_099 { v.write_to(w)?; }
                if let Some(v) = alt_body_100 { v.write_to(w)?; }
                if let Some(v) = alt_body_101 { v.write_to(w)?; }
                if let Some(v) = alt_body_102 { v.write_to(w)?; }
                if let Some(v) = alt_body_103 { v.write_to(w)?; }
                if let Some(v) = alt_body_104 { v.write_to(w)?; }
                if let Some(v) = alt_body_105 { v.write_to(w)?; }
                if let Some(v) = alt_body_106 { v.write_to(w)?; }
                if let Some(v) = alt_body_107 { v.write_to(w)?; }
                if let Some(v) = alt_body_108 { v.write_to(w)?; }
                if let Some(v) = alt_body_109 { v.write_to(w)?; }
                if let Some(v) = alt_body_110 { v.write_to(w)?; }
                if let Some(v) = alt_body_111 { v.write_to(w)?; }
                if let Some(v) = alt_body_112 { v.write_to(w)?; }
                if let Some(v) = alt_body_113 { v.write_to(w)?; }
                if let Some(v) = alt_body_114 { v.write_to(w)?; }
                if let Some(v) = alt_body_115 { v.write_to(w)?; }
                if let Some(v) = alt_body_116 { v.write_to(w)?; }
                if let Some(v) = alt_body_117 { v.write_to(w)?; }
                if let Some(v) = alt_body_118 { v.write_to(w)?; }
                if let Some(v) = alt_body_119 { v.write_to(w)?; }
                if let Some(v) = alt_body_120 { v.write_to(w)?; }
                if let Some(v) = alt_body_121 { v.write_to(w)?; }
                if let Some(v) = alt_body_122 { v.write_to(w)?; }
                if let Some(v) = alt_body_123 { v.write_to(w)?; }
                if let Some(v) = alt_body_124 { v.write_to(w)?; }
                if let Some(v) = alt_body_125 { v.write_to(w)?; }
                if let Some(v) = alt_body_126 { v.write_to(w)?; }
                if let Some(v) = alt_body_127 { v.write_to(w)?; }
                if let Some(v) = alt_body_128 { v.write_to(w)?; }
                if let Some(v) = alt_body_129 { v.write_to(w)?; }
                if let Some(v) = alt_body_130 { v.write_to(w)?; }
                if let Some(v) = alt_body_131 { v.write_to(w)?; }
                if let Some(v) = alt_body_132 { v.write_to(w)?; }
                if let Some(v) = alt_body_133 { v.write_to(w)?; }
                if let Some(v) = alt_body_134 { v.write_to(w)?; }
                if let Some(v) = alt_body_135 { v.write_to(w)?; }
                if let Some(v) = alt_body_136 { v.write_to(w)?; }
                if let Some(v) = alt_body_137 { v.write_to(w)?; }
                if let Some(v) = alt_body_138 { v.write_to(w)?; }
                if let Some(v) = alt_body_139 { v.write_to(w)?; }
                if let Some(v) = alt_body_140 { v.write_to(w)?; }
                if let Some(v) = alt_body_141 { v.write_to(w)?; }
                if let Some(v) = alt_body_142 { v.write_to(w)?; }
                if let Some(v) = alt_body_143 { v.write_to(w)?; }
                if let Some(v) = alt_body_144 { v.write_to(w)?; }
                if let Some(v) = alt_body_145 { v.write_to(w)?; }
                if let Some(v) = alt_body_146 { v.write_to(w)?; }
                if let Some(v) = alt_body_147 { v.write_to(w)?; }
                if let Some(v) = alt_body_148 { v.write_to(w)?; }
                if let Some(v) = alt_body_149 { v.write_to(w)?; }
                if let Some(v) = alt_body_150 { v.write_to(w)?; }
                if let Some(v) = alt_body_151 { v.write_to(w)?; }
                if let Some(v) = alt_body_152 { v.write_to(w)?; }
                if let Some(v) = alt_body_153 { v.write_to(w)?; }
                if let Some(v) = alt_body_154 { v.write_to(w)?; }
                if let Some(v) = alt_body_155 { v.write_to(w)?; }
                if let Some(v) = alt_body_156 { v.write_to(w)?; }
                if let Some(v) = alt_body_157 { v.write_to(w)?; }
                if let Some(v) = alt_body_158 { v.write_to(w)?; }
                if let Some(v) = alt_body_159 { v.write_to(w)?; }
                if let Some(v) = alt_body_160 { v.write_to(w)?; }
                if let Some(v) = alt_body_161 { v.write_to(w)?; }
                if let Some(v) = alt_body_162 { v.write_to(w)?; }
                if let Some(v) = alt_body_163 { v.write_to(w)?; }
                if let Some(v) = alt_body_164 { v.write_to(w)?; }
                if let Some(v) = alt_body_165 { v.write_to(w)?; }
                if let Some(v) = alt_body_166 { v.write_to(w)?; }
                if let Some(v) = alt_body_167 { v.write_to(w)?; }
                if let Some(v) = alt_body_168 { v.write_to(w)?; }
                if let Some(v) = alt_body_169 { v.write_to(w)?; }
                if let Some(v) = alt_body_170 { v.write_to(w)?; }
                if let Some(v) = alt_body_171 { v.write_to(w)?; }
                if let Some(v) = alt_body_172 { v.write_to(w)?; }
                if let Some(v) = alt_body_173 { v.write_to(w)?; }
                if let Some(v) = alt_body_174 { v.write_to(w)?; }
                if let Some(v) = alt_body_175 { v.write_to(w)?; }
                if let Some(v) = alt_body_176 { v.write_to(w)?; }
                if let Some(v) = alt_body_177 { v.write_to(w)?; }
                if let Some(v) = alt_body_178 { v.write_to(w)?; }
                if let Some(v) = alt_body_179 { v.write_to(w)?; }
                if let Some(v) = alt_body_180 { v.write_to(w)?; }
                if let Some(v) = alt_body_181 { v.write_to(w)?; }
                if let Some(v) = alt_body_182 { v.write_to(w)?; }
                if let Some(v) = alt_body_183 { v.write_to(w)?; }
                if let Some(v) = alt_body_184 { v.write_to(w)?; }
                if let Some(v) = alt_body_185 { v.write_to(w)?; }
                if let Some(v) = alt_body_186 { v.write_to(w)?; }
                if let Some(v) = alt_body_187 { v.write_to(w)?; }
                if let Some(v) = alt_body_188 { v.write_to(w)?; }
                if let Some(v) = alt_body_189 { v.write_to(w)?; }
                if let Some(v) = alt_body_190 { v.write_to(w)?; }
                if let Some(v) = alt_body_191 { v.write_to(w)?; }
                if let Some(v) = alt_body_192 { v.write_to(w)?; }
                if let Some(v) = alt_body_193 { v.write_to(w)?; }
                if let Some(v) = alt_body_194 { v.write_to(w)?; }
                if let Some(v) = alt_body_195 { v.write_to(w)?; }
                if let Some(v) = alt_body_196 { v.write_to(w)?; }
                if let Some(v) = alt_body_197 { v.write_to(w)?; }
                if let Some(v) = alt_body_198 { v.write_to(w)?; }
                if let Some(v) = alt_body_199 { v.write_to(w)?; }
                if let Some(v) = alt_body_200 { v.write_to(w)?; }
                if let Some(v) = alt_body_201 { v.write_to(w)?; }
                if let Some(v) = alt_body_202 { v.write_to(w)?; }
                if let Some(v) = alt_body_203 { v.write_to(w)?; }
                if let Some(v) = alt_body_204 { v.write_to(w)?; }
                if let Some(v) = alt_body_205 { v.write_to(w)?; }
                if let Some(v) = alt_body_206 { v.write_to(w)?; }
                if let Some(v) = alt_body_207 { v.write_to(w)?; }
                if let Some(v) = alt_body_208 { v.write_to(w)?; }
                if let Some(v) = alt_body_209 { v.write_to(w)?; }
                if let Some(v) = alt_body_210 { v.write_to(w)?; }
                if let Some(v) = alt_body_211 { v.write_to(w)?; }
                if let Some(v) = alt_body_212 { v.write_to(w)?; }
                if let Some(v) = alt_body_213 { v.write_to(w)?; }
                if let Some(v) = alt_body_214 { v.write_to(w)?; }
                if let Some(v) = alt_body_215 { v.write_to(w)?; }
                if let Some(v) = alt_body_216 { v.write_to(w)?; }
                if let Some(v) = alt_body_217 { v.write_to(w)?; }
                if let Some(v) = alt_body_218 { v.write_to(w)?; }
                if let Some(v) = alt_body_219 { v.write_to(w)?; }
                if let Some(v) = alt_body_220 { v.write_to(w)?; }
                if let Some(v) = alt_body_221 { v.write_to(w)?; }
                if let Some(v) = alt_body_222 { v.write_to(w)?; }
                if let Some(v) = alt_body_223 { v.write_to(w)?; }
                if let Some(v) = alt_body_224 { v.write_to(w)?; }
                if let Some(v) = alt_body_225 { v.write_to(w)?; }
                if let Some(v) = alt_body_226 { v.write_to(w)?; }
                if let Some(v) = alt_body_227 { v.write_to(w)?; }
                if let Some(v) = alt_body_228 { v.write_to(w)?; }
                if let Some(v) = alt_body_229 { v.write_to(w)?; }
                if let Some(v) = alt_body_230 { v.write_to(w)?; }
                if let Some(v) = alt_body_231 { v.write_to(w)?; }
                if let Some(v) = alt_body_232 { v.write_to(w)?; }
                if let Some(v) = alt_body_233 { v.write_to(w)?; }
                if let Some(v) = alt_body_234 { v.write_to(w)?; }
                if let Some(v) = alt_body_235 { v.write_to(w)?; }
                if let Some(v) = alt_body_236 { v.write_to(w)?; }
                if let Some(v) = alt_body_237 { v.write_to(w)?; }
                if let Some(v) = alt_body_238 { v.write_to(w)?; }
                if let Some(v) = alt_body_239 { v.write_to(w)?; }
                if let Some(v) = alt_body_240 { v.write_to(w)?; }
                if let Some(v) = alt_body_241 { v.write_to(w)?; }
                if let Some(v) = alt_body_242 { v.write_to(w)?; }
                if let Some(v) = alt_body_243 { v.write_to(w)?; }
                if let Some(v) = alt_body_244 { v.write_to(w)?; }
                if let Some(v) = alt_body_245 { v.write_to(w)?; }
                if let Some(v) = alt_body_246 { v.write_to(w)?; }
                if let Some(v) = alt_body_247 { v.write_to(w)?; }
                if let Some(v) = alt_body_248 { v.write_to(w)?; }
                if let Some(v) = alt_body_249 { v.write_to(w)?; }
                if let Some(v) = alt_body_250 { v.write_to(w)?; }
                if let Some(v) = alt_body_251 { v.write_to(w)?; }
                if let Some(v) = alt_body_252 { v.write_to(w)?; }
                if let Some(v) = alt_body_253 { v.write_to(w)?; }
                if let Some(v) = alt_body_254 { v.write_to(w)?; }
                if let Some(v) = alt_body_255 { v.write_to(w)?; }
                if let Some(v) = alt_body_256 { v.write_to(w)?; }
                if let Some(v) = alt_body_257 { v.write_to(w)?; }
                if let Some(v) = alt_body_258 { v.write_to(w)?; }
                if let Some(v) = alt_body_259 { v.write_to(w)?; }
                if let Some(v) = alt_body_260 { v.write_to(w)?; }
                if let Some(v) = alt_body_261 { v.write_to(w)?; }
                if let Some(v) = alt_body_262 { v.write_to(w)?; }
                if let Some(v) = alt_body_263 { v.write_to(w)?; }
                if let Some(v) = alt_body_264 { v.write_to(w)?; }
                if let Some(v) = alt_body_265 { v.write_to(w)?; }
                if let Some(v) = alt_body_266 { v.write_to(w)?; }
                if let Some(v) = alt_body_267 { v.write_to(w)?; }
                if let Some(v) = alt_body_268 { v.write_to(w)?; }
                if let Some(v) = alt_body_269 { v.write_to(w)?; }
                if let Some(v) = alt_body_270 { v.write_to(w)?; }
                if let Some(v) = alt_body_271 { v.write_to(w)?; }
                if let Some(v) = alt_body_272 { v.write_to(w)?; }
                if let Some(v) = alt_body_273 { v.write_to(w)?; }
                if let Some(v) = alt_body_274 { v.write_to(w)?; }
                if let Some(v) = alt_body_275 { v.write_to(w)?; }
                if let Some(v) = alt_body_276 { v.write_to(w)?; }
                if let Some(v) = alt_body_277 { v.write_to(w)?; }
                if let Some(v) = alt_body_278 { v.write_to(w)?; }
                if let Some(v) = alt_body_279 { v.write_to(w)?; }
                if let Some(v) = alt_body_280 { v.write_to(w)?; }
                if let Some(v) = alt_body_281 { v.write_to(w)?; }
                if let Some(v) = alt_body_282 { v.write_to(w)?; }
                if let Some(v) = alt_body_283 { v.write_to(w)?; }
                if let Some(v) = alt_body_284 { v.write_to(w)?; }
                if let Some(v) = alt_body_285 { v.write_to(w)?; }
                if let Some(v) = alt_body_286 { v.write_to(w)?; }
                if let Some(v) = alt_body_287 { v.write_to(w)?; }
                if let Some(v) = alt_body_288 { v.write_to(w)?; }
                if let Some(v) = alt_body_289 { v.write_to(w)?; }
                if let Some(v) = alt_body_290 { v.write_to(w)?; }
                if let Some(v) = alt_body_291 { v.write_to(w)?; }
                if let Some(v) = alt_body_292 { v.write_to(w)?; }
                if let Some(v) = alt_body_293 { v.write_to(w)?; }
                if let Some(v) = alt_body_294 { v.write_to(w)?; }
                if let Some(v) = alt_body_295 { v.write_to(w)?; }
                if let Some(v) = alt_body_296 { v.write_to(w)?; }
                if let Some(v) = alt_body_297 { v.write_to(w)?; }
                if let Some(v) = alt_body_298 { v.write_to(w)?; }
                if let Some(v) = alt_body_299 { v.write_to(w)?; }
                if let Some(v) = alt_body_300 { v.write_to(w)?; }
                if let Some(v) = alt_body_301 { v.write_to(w)?; }
                if let Some(v) = alt_body_302 { v.write_to(w)?; }
                if let Some(v) = alt_body_303 { v.write_to(w)?; }
                if let Some(v) = alt_body_304 { v.write_to(w)?; }
                if let Some(v) = alt_body_305 { v.write_to(w)?; }
                if let Some(v) = alt_body_306 { v.write_to(w)?; }
                if let Some(v) = alt_body_307 { v.write_to(w)?; }
                if let Some(v) = alt_body_308 { v.write_to(w)?; }
                if let Some(v) = alt_body_309 { v.write_to(w)?; }
                if let Some(v) = alt_body_310 { v.write_to(w)?; }
                if let Some(v) = alt_body_311 { v.write_to(w)?; }
                if let Some(v) = alt_body_312 { v.write_to(w)?; }
                if let Some(v) = alt_body_313 { v.write_to(w)?; }
                if let Some(v) = alt_body_314 { v.write_to(w)?; }
                if let Some(v) = alt_body_315 { v.write_to(w)?; }
                if let Some(v) = alt_body_316 { v.write_to(w)?; }
                if let Some(v) = alt_body_317 { v.write_to(w)?; }
                if let Some(v) = alt_body_318 { v.write_to(w)?; }
                if let Some(v) = alt_body_319 { v.write_to(w)?; }
                if let Some(v) = alt_body_320 { v.write_to(w)?; }
                if let Some(v) = field_665_u32 { v.write_to(w)?; }
                if let Some(v) = field_666_u32 { v.write_to(w)?; }
                if let Some(v) = field_667_u32 { v.write_to(w)?; }
                if let Some(v) = field_668_u32 { v.write_to(w)?; }
                if let Some(v) = field_669_u32 { v.write_to(w)?; }
                if let Some(v) = field_670_u32 { v.write_to(w)?; }
                if let Some(v) = field_671_u32 { v.write_to(w)?; }
                if let Some(v) = field_672_u32 { v.write_to(w)?; }
                if let Some(v) = field_673_u32 { v.write_to(w)?; }
                if let Some(v) = field_674_u32 { v.write_to(w)?; }
                if let Some(v) = field_675_u32 { v.write_to(w)?; }
                if let Some(v) = field_676_u32 { v.write_to(w)?; }
                if let Some(v) = field_677_u32 { v.write_to(w)?; }
                if let Some(v) = field_678_u32 { v.write_to(w)?; }
                if let Some(v) = field_679_u32 { v.write_to(w)?; }
                if let Some(v) = field_680_u32 { v.write_to(w)?; }
                if let Some(v) = field_681_u32 { v.write_to(w)?; }
                if let Some(v) = field_682_u32 { v.write_to(w)?; }
                if let Some(v) = field_683_u32 { v.write_to(w)?; }
                if let Some(v) = field_684_u32 { v.write_to(w)?; }
                if let Some(v) = field_685_u32 { v.write_to(w)?; }
                if let Some(v) = field_686_u32 { v.write_to(w)?; }
                if let Some(v) = field_687_u32 { v.write_to(w)?; }
                if let Some(v) = field_688_u32 { v.write_to(w)?; }
                if let Some(v) = field_689_u32 { v.write_to(w)?; }
                if let Some(v) = field_690_u32 { v.write_to(w)?; }
                if let Some(v) = field_691_u32 { v.write_to(w)?; }
                if let Some(v) = field_692_u32 { v.write_to(w)?; }
                if let Some(v) = field_693_u32 { v.write_to(w)?; }
                if let Some(v) = field_694_u32 { v.write_to(w)?; }
                if let Some(v) = field_695_u32 { v.write_to(w)?; }
                if let Some(v) = field_696_u32 { v.write_to(w)?; }
                if let Some(v) = field_697_u32 { v.write_to(w)?; }
                if let Some(v) = field_698_u32 { v.write_to(w)?; }
                if let Some(v) = field_699_u32 { v.write_to(w)?; }
                if let Some(v) = field_700_u32 { v.write_to(w)?; }
                if let Some(v) = field_701_u32 { v.write_to(w)?; }
                if let Some(v) = field_702_u32 { v.write_to(w)?; }
                if let Some(v) = field_703_u32 { v.write_to(w)?; }
                if let Some(v) = field_704_u32 { v.write_to(w)?; }
                if let Some(v) = field_705_u32 { v.write_to(w)?; }
                if let Some(v) = field_706_u32 { v.write_to(w)?; }
                if let Some(v) = field_707_u32 { v.write_to(w)?; }
                if let Some(v) = field_708_u32 { v.write_to(w)?; }
                if let Some(v) = field_709_u32 { v.write_to(w)?; }
                if let Some(v) = field_710_u32 { v.write_to(w)?; }
                if let Some(v) = field_711_u32 { v.write_to(w)?; }
                if let Some(v) = field_712_u32 { v.write_to(w)?; }
                if let Some(v) = field_713_u32 { v.write_to(w)?; }
                if let Some(v) = field_714_u32 { v.write_to(w)?; }
                if let Some(v) = field_715_u32 { v.write_to(w)?; }
                if let Some(v) = field_716_u32 { v.write_to(w)?; }
                if let Some(v) = field_717_u32 { v.write_to(w)?; }
                if let Some(v) = field_718_u32 { v.write_to(w)?; }
                if let Some(v) = field_719_u32 { v.write_to(w)?; }
                if let Some(v) = field_720_u32 { v.write_to(w)?; }
                if let Some(v) = field_721_u32 { v.write_to(w)?; }
                if let Some(v) = field_722_u32 { v.write_to(w)?; }
                if let Some(v) = field_723_u32 { v.write_to(w)?; }
                if let Some(v) = field_724_u32 { v.write_to(w)?; }
                if let Some(v) = field_725_u32 { v.write_to(w)?; }
                if let Some(v) = field_726_u32 { v.write_to(w)?; }
                if let Some(v) = field_727_u32 { v.write_to(w)?; }
                if let Some(v) = field_728_u32 { v.write_to(w)?; }
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
                field_107_u32, field_108_u32, field_109_u32, field_110_u32,
                field_111_u32, field_112_u32, field_113_u32, field_114_u32,
                field_115_u32, field_116_u32, field_117_u32, field_118_u32,
                field_119_u32, field_120_u32, field_121_u32, field_122_u32,
                field_123_u32, field_124_u32, field_125_u32, field_126_u32,
                field_127_u32, field_128_u32, field_129_u32, field_130_u32,
                field_131_u32, field_132_u32, field_133_u32, field_134_u32,
                field_135_u32, field_136_u32, field_137_u32, field_138_u32,
                field_139_u32, field_140_u32, field_141_u32, field_142_u32,
                field_143_u32, field_144_u32, field_145_u32, field_146_u32,
                field_147_u32, field_148_u32, field_149_u32, field_150_u32,
                field_151_u32, field_152_u32, field_153_u32, field_154_u32,
                field_155_u32, field_156_u32, field_157_u32, field_158_u32,
                field_159_u32, field_160_u32, field_161_u32, field_162_u32,
                field_163_u32, field_164_u32, field_165_u32, field_166_u32,
                field_167_u32, field_168_u32, field_169_u32, field_170_u32,
                field_171_u32, field_172_u32, field_173_u32, field_174_u32,
                field_175_u32, field_176_u32, field_177_u32, field_178_u32,
                field_179_u32, field_180_u32, field_181_u32,
                field_182_u32, field_183_u32, field_184_u32, field_185_u32,
                field_186_u32, field_187_u32, field_188_u32, field_189_u32,
                field_190_u32, field_191_u32, field_192_u32, field_193_u32,
                field_194_u32, field_195_u32, field_196_u32, field_197_u32,
                field_198_u32, field_199_u32, field_200_u32, field_201_u32,
                field_202_u32, field_203_u32, field_204_u32, field_205_u32,
                field_206_u32, field_207_u32, field_208_u32, field_209_u32,
                field_210_u32, field_211_u32, field_212_u32, field_213_u32,
                field_214_u32, field_215_u32, field_216_u32, field_217_u32,
                field_218_u32, field_219_u32, field_220_u32, field_221_u32,
                field_222_u32, field_223_u32, field_224_u32, field_225_u32,
                field_226_u32, field_227_u32, field_228_u32, field_229_u32,
                field_230_u32, field_231_u32, field_232_u32, field_233_u32,
                field_234_u32, field_235_u32, field_236_u32, field_237_u32,
                field_238_u32, field_239_u32, field_240_u32, field_241_u32,
                field_242_u32, field_243_u32, field_244_u32, field_245_u32,
                field_246_u32, field_247_u32, field_248_u32, field_249_u32,
                field_250_u32, field_251_u32, field_252_u32, field_253_u32,
                field_254_u32, field_255_u32, field_256_u32, field_257_u32,
                field_258_u32, field_259_u32, field_260_u32, field_261_u32,
                field_262_u32, field_263_u32, field_264_u32, field_265_u32,
                field_266_u32, field_267_u32, field_268_u32, field_269_u32,
                field_270_u32, field_271_u32, field_272_u32, field_273_u32,
                field_274_u32, field_275_u32, field_276_u32, field_277_u32,
                field_278_u32, field_279_u32, field_280_u32, field_281_u32,
                field_282_u32, field_283_u32, field_284_u32, field_285_u32,
                field_286_u32, field_287_u32, field_288_u32, field_289_u32,
                field_290_u32, field_291_u32, field_292_u32, field_293_u32,
                field_294_u32, field_295_u32, field_296_u32, field_297_u32,
                field_298_u32, field_299_u32, field_300_u32, field_301_u32,
                field_302_u32, field_303_u32, field_304_u32, field_305_u32,
                field_306_u32, field_307_u32, field_308_u32, field_309_u32,
                field_310_u32, field_311_u32, field_312_u32, field_313_u32,
                field_314_u32, field_315_u32, field_316_u32, field_317_u32,
                field_318_u32, field_319_u32, field_320_u32, field_321_u32,
                field_322_u32, field_323_u32, field_324_u32, field_325_u32,
                field_326_u32, field_327_u32, field_328_u32, field_329_u32,
                field_330_u32, field_331_u32, field_332_u32, field_333_u32,
                field_334_u32, field_335_u32, field_336_u32, field_337_u32,
                field_338_u32, field_339_u32, field_340_u32, field_341_u32,
                field_342_u32_count, field_343_u8_flag, field_344_u32,
                field_345_u32, field_346_u32, field_347_u32, field_348_u32,
                field_349_u32, field_350_u32, field_351_u32, field_352_u32,
                field_353_u32, field_354_u32, field_355_u32, field_356_u32,
                field_357_u32, field_358_u32, field_359_u32, field_360_u32,
                field_361_u32, field_362_u32, field_363_u32, field_364_u32,
                field_365_u32, field_366_u32, field_367_u32, field_368_u32,
                field_369_u32, field_370_u32, field_371_u32, field_372_u32,
                field_373_u32, field_374_u32, field_375_u32, field_376_u32,
                field_377_u32, field_378_u32, field_379_u32, field_380_u32,
                field_381_u32, field_382_u32, field_383_u32, field_384_u32,
                field_385_u32, field_386_u32, field_387_u32, field_388_u32,
                field_389_u32, field_390_u32, field_391_u32, field_392_u32,
                field_393_u32, field_394_u32, field_395_u32, field_396_u32,
                field_397_u32, field_398_u32, field_399_u32, field_400_u32,
                field_401_u32, field_402_u32, field_403_u32, field_404_u32,
                field_405_u32, field_406_u32, field_407_u32, field_408_u32,
                field_409_u32, field_410_u32, field_411_u32, field_412_u32,
                field_413_u32, field_414_u32, field_415_u32, field_416_u32,
                field_417_u32, field_418_u32, field_419_u32, field_420_u32,
                field_421_u32, field_422_u32, field_423_u32, field_424_u32,
                field_425_u32, field_426_u32, field_427_u32, field_428_u32,
                field_429_u32, field_430_u32, field_431_u32, field_432_u32,
                field_433_u32, field_434_u32, field_435_u32, field_436_u32,
                field_437_u32, field_438_u32, field_439_u32, field_440_u32,
                field_441_u32, field_442_u32, field_443_u32, field_444_u32,
                field_445_u32, field_446_u32, field_447_u32, field_448_u32,
                field_449_u32, field_450_u32, field_451_u32, field_452_u32,
                field_453_u32, field_454_u32, field_455_u32, field_456_u32,
                field_457_u32, field_458_u32, field_459_u32, field_460_u32,
                field_461_u32, field_462_u32, field_463_u32, field_464_u32,
                field_465_u32, field_466_u32, field_467_u32, field_468_u32,
                field_469_u32, field_470_u32, field_471_u32, field_472_u32,
                field_473_u32, field_474_u32, field_475_u32, field_476_u32,
                field_477_u32, field_478_u32, field_479_u32, field_480_u32,
                field_481_u32, field_482_u32, field_483_u32, field_484_u32,
                field_485_u32, field_486_u32, field_487_u32, field_488_u32,
                field_489_u32, field_490_u32, field_491_u32, field_492_u32,
                field_493_u32, field_494_u32, field_495_u32, field_496_u32,
                field_497_u32, field_498_u32, field_499_u32, field_500_u32,
                field_501_u32, field_502_u32, field_503_u32, field_504_u32,
                field_505_u32, field_506_u32, field_507_u32, field_508_u32,
                field_509_u32, field_510_u32, field_511_u32, field_512_u32,
                field_513_u32, field_514_u32, field_515_u32, field_516_u32,
                field_517_u32, field_518_u32, field_519_u32, field_520_u32,
                field_521_u32, field_522_u32, field_523_u32, field_524_u32,
                field_525_u32, field_526_u32, field_527_u32, field_528_u32,
                field_529_u32, field_530_u32, field_531_u32, field_532_u32,
                field_533_u32, field_534_u32, field_535_u32, field_536_u32,
                field_537_u32, field_538_u32, field_539_u32, field_540_u32,
                field_541_u32, field_542_u32, field_543_u32, field_544_u32,
                field_545_u32, field_546_u32, field_547_u32, field_548_u32,
                field_549_u32, field_550_u32, field_551_u32, field_552_u32,
                field_553_u32, field_554_u32, field_555_u32, field_556_u32,
                field_557_u32, field_558_u32, field_559_u32, field_560_u32,
                field_561_u32, field_562_u32, field_563_u32, field_564_u32,
                field_565_u32, field_566_u32, field_567_u32, field_568_u32,
                field_569_u32, field_570_u32, field_571_u32, field_572_u32,
                field_573_u32, field_574_u32, field_575_u32, field_576_u32,
                field_577_u32, field_578_u32, field_579_u32, field_580_u32,
                field_581_u32, field_582_u32, field_583_u32, field_584_u32,
                field_585_u32, field_586_u32, field_587_u32, field_588_u32,
                field_589_u32, field_590_u32, field_591_u32, field_592_u32,
                field_593_u32, field_594_u32, field_595_u32, field_596_u32,
                field_597_u32, field_598_u32, field_599_u32, field_600_u32,
                field_601_u32, field_602_u32, field_603_u32, field_604_u32,
                field_605_u32, field_606_u32, field_607_u32, field_608_u32,
                field_609_u32, field_610_u32, field_611_u32, field_612_u32,
                field_613_u32, field_614_u32, field_615_u32, field_616_u32,
                field_617_u32, field_618_u32, field_619_u32, field_620_u32,
                field_621_u32, field_622_u32, field_623_u32, field_624_u32,
                field_625_u32, field_626_u32, field_627_u32, field_628_u32,
                field_629_u32, field_630_u32, field_631_u32, field_632_u32,
                field_633_u32, field_634_u32, field_635_u32, field_636_u32,
                field_637_u32, field_638_u32, field_639_u32, field_640_u32,
                field_641_u32, field_642_u32, field_643_u32, field_644_u32,
                field_645_u32, field_646_u32, field_647_u32, field_648_u32,
                field_649_u32, field_650_u32, field_651_u32, field_652_u32,
                field_653_u32, field_654_u32, field_655_u32, field_656_u32,
                field_657_u32, field_658_u32, field_659_u32, field_660_u32,
                field_661_u32, field_662_u32, field_663_u32, field_664_u32,
                alt_trigger_count, alt_trigger_flag, alt_trigger_name,
                alt_inner_count, alt_inner_name, alt_inner_flag,
                alt_body_001, alt_body_002, alt_body_003, alt_body_004,
                alt_body_005, alt_body_006, alt_body_007, alt_body_008,
                alt_body_009, alt_body_010, alt_body_011, alt_body_012,
                alt_body_013, alt_body_014, alt_body_015, alt_body_016,
                alt_body_017, alt_body_018, alt_body_019, alt_body_020,
                alt_body_021, alt_body_022, alt_body_023, alt_body_024,
                alt_body_025, alt_body_026, alt_body_027, alt_body_028,
                alt_body_029, alt_body_030, alt_body_031, alt_body_032,
                alt_body_033, alt_body_034, alt_body_035, alt_body_036,
                alt_body_037, alt_body_038, alt_body_039, alt_body_040,
                alt_body_041, alt_body_042, alt_body_043, alt_body_044,
                alt_body_045, alt_body_046, alt_body_047, alt_body_048,
                alt_body_049, alt_body_050, alt_body_051, alt_body_052,
                alt_body_053, alt_body_054, alt_body_055, alt_body_056,
                alt_body_057, alt_body_058, alt_body_059, alt_body_060,
                alt_body_061, alt_body_062, alt_body_063, alt_body_064,
                alt_body_065, alt_body_066, alt_body_067, alt_body_068,
                alt_body_069, alt_body_070, alt_body_071, alt_body_072,
                alt_body_073, alt_body_074, alt_body_075, alt_body_076,
                alt_body_077, alt_body_078, alt_body_079, alt_body_080,
                alt_body_081, alt_body_082, alt_body_083, alt_body_084,
                alt_body_085, alt_body_086, alt_body_087, alt_body_088,
                alt_body_089, alt_body_090, alt_body_091, alt_body_092,
                alt_body_093, alt_body_094, alt_body_095, alt_body_096,
                alt_body_097, alt_body_098, alt_body_099, alt_body_100,
                alt_body_101, alt_body_102, alt_body_103, alt_body_104,
                alt_body_105, alt_body_106, alt_body_107, alt_body_108,
                alt_body_109, alt_body_110, alt_body_111, alt_body_112,
                alt_body_113, alt_body_114, alt_body_115, alt_body_116,
                alt_body_117, alt_body_118, alt_body_119, alt_body_120,
                alt_body_121, alt_body_122, alt_body_123, alt_body_124,
                alt_body_125, alt_body_126, alt_body_127, alt_body_128,
                alt_body_129, alt_body_130, alt_body_131, alt_body_132,
                alt_body_133, alt_body_134, alt_body_135, alt_body_136,
                alt_body_137, alt_body_138, alt_body_139, alt_body_140,
                alt_body_141, alt_body_142, alt_body_143, alt_body_144,
                alt_body_145, alt_body_146, alt_body_147, alt_body_148,
                alt_body_149, alt_body_150, alt_body_151, alt_body_152,
                alt_body_153, alt_body_154, alt_body_155, alt_body_156,
                alt_body_157, alt_body_158, alt_body_159, alt_body_160,
                alt_body_161, alt_body_162, alt_body_163, alt_body_164,
                alt_body_165, alt_body_166, alt_body_167, alt_body_168,
                alt_body_169, alt_body_170, alt_body_171, alt_body_172,
                alt_body_173, alt_body_174, alt_body_175, alt_body_176,
                alt_body_177, alt_body_178, alt_body_179, alt_body_180,
                alt_body_181, alt_body_182, alt_body_183, alt_body_184,
                alt_body_185, alt_body_186, alt_body_187, alt_body_188,
                alt_body_189, alt_body_190, alt_body_191, alt_body_192,
                alt_body_193, alt_body_194, alt_body_195, alt_body_196,
                alt_body_197, alt_body_198, alt_body_199, alt_body_200,
                alt_body_201, alt_body_202, alt_body_203, alt_body_204,
                alt_body_205, alt_body_206, alt_body_207, alt_body_208,
                alt_body_209, alt_body_210, alt_body_211, alt_body_212,
                alt_body_213, alt_body_214, alt_body_215, alt_body_216,
                alt_body_217, alt_body_218, alt_body_219, alt_body_220,
                alt_body_221, alt_body_222, alt_body_223, alt_body_224,
                alt_body_225, alt_body_226, alt_body_227, alt_body_228,
                alt_body_229, alt_body_230, alt_body_231, alt_body_232,
                alt_body_233, alt_body_234, alt_body_235, alt_body_236,
                alt_body_237, alt_body_238, alt_body_239, alt_body_240,
                alt_body_241, alt_body_242, alt_body_243, alt_body_244,
                alt_body_245, alt_body_246, alt_body_247, alt_body_248,
                alt_body_249, alt_body_250, alt_body_251, alt_body_252,
                alt_body_253, alt_body_254, alt_body_255, alt_body_256,
                alt_body_257, alt_body_258, alt_body_259, alt_body_260,
                alt_body_261, alt_body_262, alt_body_263, alt_body_264,
                alt_body_265, alt_body_266, alt_body_267, alt_body_268,
                alt_body_269, alt_body_270, alt_body_271, alt_body_272,
                alt_body_273, alt_body_274, alt_body_275, alt_body_276,
                alt_body_277, alt_body_278, alt_body_279, alt_body_280,
                alt_body_281, alt_body_282, alt_body_283, alt_body_284,
                alt_body_285, alt_body_286, alt_body_287, alt_body_288,
                alt_body_289, alt_body_290, alt_body_291, alt_body_292,
                alt_body_293, alt_body_294, alt_body_295, alt_body_296,
                alt_body_297, alt_body_298, alt_body_299, alt_body_300,
                alt_body_301, alt_body_302, alt_body_303, alt_body_304,
                alt_body_305, alt_body_306, alt_body_307, alt_body_308,
                alt_body_309, alt_body_310, alt_body_311, alt_body_312,
                alt_body_313, alt_body_314, alt_body_315, alt_body_316,
                alt_body_317, alt_body_318, alt_body_319, alt_body_320,
                field_665_u32, field_666_u32, field_667_u32, field_668_u32,
                field_669_u32, field_670_u32, field_671_u32, field_672_u32,
                field_673_u32, field_674_u32, field_675_u32, field_676_u32,
                field_677_u32, field_678_u32, field_679_u32, field_680_u32,
                field_681_u32, field_682_u32, field_683_u32, field_684_u32,
                field_685_u32, field_686_u32, field_687_u32, field_688_u32,
                field_689_u32, field_690_u32, field_691_u32, field_692_u32,
                field_693_u32, field_694_u32, field_695_u32, field_696_u32,
                field_697_u32, field_698_u32, field_699_u32, field_700_u32,
                field_701_u32, field_702_u32, field_703_u32, field_704_u32,
                field_705_u32, field_706_u32, field_707_u32, field_708_u32,
                field_709_u32, field_710_u32, field_711_u32, field_712_u32,
                field_713_u32, field_714_u32, field_715_u32, field_716_u32,
                field_717_u32, field_718_u32, field_719_u32, field_720_u32,
                field_721_u32, field_722_u32, field_723_u32, field_724_u32,
                field_725_u32, field_726_u32, field_727_u32, field_728_u32, post_blob } => {
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
                m.insert("field_108_u32".to_string(), match field_108_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_109_u32".to_string(), match field_109_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_110_u32".to_string(), match field_110_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_111_u32".to_string(), match field_111_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_112_u32".to_string(), match field_112_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_113_u32".to_string(), match field_113_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_114_u32".to_string(), match field_114_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_115_u32".to_string(), match field_115_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_116_u32".to_string(), match field_116_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_117_u32".to_string(), match field_117_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_118_u32".to_string(), match field_118_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_119_u32".to_string(), match field_119_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_120_u32".to_string(), match field_120_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_121_u32".to_string(), match field_121_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_122_u32".to_string(), match field_122_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_123_u32".to_string(), match field_123_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                for (k, v) in [("field_124_u32", field_124_u32),
                               ("field_125_u32", field_125_u32),
                               ("field_126_u32", field_126_u32),
                               ("field_127_u32", field_127_u32),
                               ("field_128_u32", field_128_u32),
                               ("field_129_u32", field_129_u32),
                               ("field_130_u32", field_130_u32),
                               ("field_131_u32", field_131_u32),
                               ("field_132_u32", field_132_u32),
                               ("field_133_u32", field_133_u32),
                               ("field_134_u32", field_134_u32),
                               ("field_135_u32", field_135_u32),
                               ("field_136_u32", field_136_u32),
                               ("field_137_u32", field_137_u32),
                               ("field_138_u32", field_138_u32),
                               ("field_139_u32", field_139_u32),
                               ("field_140_u32", field_140_u32),
                               ("field_141_u32", field_141_u32),
                               ("field_142_u32", field_142_u32),
                               ("field_143_u32", field_143_u32),
                               ("field_144_u32", field_144_u32),
                               ("field_145_u32", field_145_u32),
                               ("field_146_u32", field_146_u32),
                               ("field_147_u32", field_147_u32),
                               ("field_148_u32", field_148_u32),
                               ("field_149_u32", field_149_u32),
                               ("field_150_u32", field_150_u32),
                               ("field_151_u32", field_151_u32),
                               ("field_152_u32", field_152_u32),
                               ("field_153_u32", field_153_u32),
                               ("field_154_u32", field_154_u32),
                               ("field_155_u32", field_155_u32),
                               ("field_156_u32", field_156_u32),
                               ("field_157_u32", field_157_u32),
                               ("field_158_u32", field_158_u32),
                               ("field_159_u32", field_159_u32),
                               ("field_160_u32", field_160_u32),
                               ("field_161_u32", field_161_u32),
                               ("field_162_u32", field_162_u32),
                               ("field_163_u32", field_163_u32),
                               ("field_164_u32", field_164_u32),
                               ("field_165_u32", field_165_u32),
                               ("field_166_u32", field_166_u32),
                               ("field_167_u32", field_167_u32),
                               ("field_168_u32", field_168_u32),
                               ("field_169_u32", field_169_u32),
                               ("field_170_u32", field_170_u32),
                               ("field_171_u32", field_171_u32),
                               ("field_172_u32", field_172_u32),
                               ("field_173_u32", field_173_u32),
                               ("field_174_u32", field_174_u32),
                               ("field_175_u32", field_175_u32),
                               ("field_176_u32", field_176_u32),
                               ("field_177_u32", field_177_u32),
                               ("field_178_u32", field_178_u32),
                               ("field_179_u32", field_179_u32),
                               ("field_180_u32", field_180_u32),
                               ("field_181_u32", field_181_u32),
                               ("field_182_u32", field_182_u32),
                               ("field_183_u32", field_183_u32),
                               ("field_184_u32", field_184_u32),
                               ("field_185_u32", field_185_u32),
                               ("field_186_u32", field_186_u32),
                               ("field_187_u32", field_187_u32),
                               ("field_188_u32", field_188_u32),
                               ("field_189_u32", field_189_u32),
                               ("field_190_u32", field_190_u32),
                               ("field_191_u32", field_191_u32),
                               ("field_192_u32", field_192_u32),
                               ("field_193_u32", field_193_u32),
                               ("field_194_u32", field_194_u32),
                               ("field_195_u32", field_195_u32),
                               ("field_196_u32", field_196_u32),
                               ("field_197_u32", field_197_u32),
                               ("field_198_u32", field_198_u32),
                               ("field_199_u32", field_199_u32),
                               ("field_200_u32", field_200_u32),
                               ("field_201_u32", field_201_u32),
                               ("field_202_u32", field_202_u32),
                               ("field_203_u32", field_203_u32),
                               ("field_204_u32", field_204_u32),
                               ("field_205_u32", field_205_u32),
                               ("field_206_u32", field_206_u32),
                               ("field_207_u32", field_207_u32),
                               ("field_208_u32", field_208_u32),
                               ("field_209_u32", field_209_u32),
                               ("field_210_u32", field_210_u32),
                               ("field_211_u32", field_211_u32),
                               ("field_212_u32", field_212_u32),
                               ("field_213_u32", field_213_u32),
                               ("field_214_u32", field_214_u32),
                               ("field_215_u32", field_215_u32),
                               ("field_216_u32", field_216_u32),
                               ("field_217_u32", field_217_u32),
                               ("field_218_u32", field_218_u32),
                               ("field_219_u32", field_219_u32),
                               ("field_220_u32", field_220_u32),
                               ("field_221_u32", field_221_u32),
                               ("field_222_u32", field_222_u32),
                               ("field_223_u32", field_223_u32),
                               ("field_224_u32", field_224_u32),
                               ("field_225_u32", field_225_u32),
                               ("field_226_u32", field_226_u32),
                               ("field_227_u32", field_227_u32),
                               ("field_228_u32", field_228_u32),
                               ("field_229_u32", field_229_u32),
                               ("field_230_u32", field_230_u32),
                               ("field_231_u32", field_231_u32),
                               ("field_232_u32", field_232_u32),
                               ("field_233_u32", field_233_u32),
                               ("field_234_u32", field_234_u32),
                               ("field_235_u32", field_235_u32),
                               ("field_236_u32", field_236_u32),
                               ("field_237_u32", field_237_u32),
                               ("field_238_u32", field_238_u32),
                               ("field_239_u32", field_239_u32),
                               ("field_240_u32", field_240_u32),
                               ("field_241_u32", field_241_u32),
                               ("field_242_u32", field_242_u32),
                               ("field_243_u32", field_243_u32),
                               ("field_244_u32", field_244_u32),
                               ("field_245_u32", field_245_u32),
                               ("field_246_u32", field_246_u32),
                               ("field_247_u32", field_247_u32),
                               ("field_248_u32", field_248_u32),
                               ("field_249_u32", field_249_u32),
                               ("field_250_u32", field_250_u32),
                               ("field_251_u32", field_251_u32),
                               ("field_252_u32", field_252_u32),
                               ("field_253_u32", field_253_u32),
                               ("field_254_u32", field_254_u32),
                               ("field_255_u32", field_255_u32),
                               ("field_256_u32", field_256_u32),
                               ("field_257_u32", field_257_u32),
                               ("field_258_u32", field_258_u32),
                               ("field_259_u32", field_259_u32),
                               ("field_260_u32", field_260_u32),
                               ("field_261_u32", field_261_u32),
                               ("field_262_u32", field_262_u32),
                               ("field_263_u32", field_263_u32),
                               ("field_264_u32", field_264_u32),
                               ("field_265_u32", field_265_u32),
                               ("field_266_u32", field_266_u32),
                               ("field_267_u32", field_267_u32),
                               ("field_268_u32", field_268_u32),
                               ("field_269_u32", field_269_u32),
                               ("field_270_u32", field_270_u32),
                               ("field_271_u32", field_271_u32),
                               ("field_272_u32", field_272_u32),
                               ("field_273_u32", field_273_u32),
                               ("field_274_u32", field_274_u32),
                               ("field_275_u32", field_275_u32),
                               ("field_276_u32", field_276_u32),
                               ("field_277_u32", field_277_u32),
                               ("field_278_u32", field_278_u32),
                               ("field_279_u32", field_279_u32),
                               ("field_280_u32", field_280_u32),
                               ("field_281_u32", field_281_u32),
                               ("field_282_u32", field_282_u32),
                               ("field_283_u32", field_283_u32),
                               ("field_284_u32", field_284_u32),
                               ("field_285_u32", field_285_u32),
                               ("field_286_u32", field_286_u32),
                               ("field_287_u32", field_287_u32),
                               ("field_288_u32", field_288_u32),
                               ("field_289_u32", field_289_u32),
                               ("field_290_u32", field_290_u32),
                               ("field_291_u32", field_291_u32),
                               ("field_292_u32", field_292_u32),
                               ("field_293_u32", field_293_u32),
                               ("field_294_u32", field_294_u32),
                               ("field_295_u32", field_295_u32),
                               ("field_296_u32", field_296_u32),
                               ("field_297_u32", field_297_u32),
                               ("field_298_u32", field_298_u32),
                               ("field_299_u32", field_299_u32),
                               ("field_300_u32", field_300_u32),
                               ("field_301_u32", field_301_u32),
                               ("field_302_u32", field_302_u32),
                               ("field_303_u32", field_303_u32),
                               ("field_304_u32", field_304_u32),
                               ("field_305_u32", field_305_u32),
                               ("field_306_u32", field_306_u32),
                               ("field_307_u32", field_307_u32),
                               ("field_308_u32", field_308_u32),
                               ("field_309_u32", field_309_u32),
                               ("field_310_u32", field_310_u32),
                               ("field_311_u32", field_311_u32),
                               ("field_312_u32", field_312_u32),
                               ("field_313_u32", field_313_u32),
                               ("field_314_u32", field_314_u32),
                               ("field_315_u32", field_315_u32),
                               ("field_316_u32", field_316_u32),
                               ("field_317_u32", field_317_u32),
                               ("field_318_u32", field_318_u32),
                               ("field_319_u32", field_319_u32),
                               ("field_320_u32", field_320_u32),
                               ("field_321_u32", field_321_u32),
                               ("field_322_u32", field_322_u32),
                               ("field_323_u32", field_323_u32),
                               ("field_324_u32", field_324_u32),
                               ("field_325_u32", field_325_u32),
                               ("field_326_u32", field_326_u32),
                               ("field_327_u32", field_327_u32),
                               ("field_328_u32", field_328_u32),
                               ("field_329_u32", field_329_u32),
                               ("field_330_u32", field_330_u32),
                               ("field_331_u32", field_331_u32),
                               ("field_332_u32", field_332_u32),
                               ("field_333_u32", field_333_u32),
                               ("field_334_u32", field_334_u32),
                               ("field_335_u32", field_335_u32),
                               ("field_336_u32", field_336_u32),
                               ("field_337_u32", field_337_u32),
                               ("field_338_u32", field_338_u32),
                               ("field_339_u32", field_339_u32),
                               ("field_340_u32", field_340_u32),
                               ("field_341_u32", field_341_u32),
                               ("field_342_u32_count", field_342_u32_count)] {
                    m.insert(k.to_string(), match v {
                        Some(val) => val.to_json_value(), None => Value::Null });
                }
                m.insert("field_343_u8_flag".to_string(), match field_343_u8_flag {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("field_344_u32".to_string(), match field_344_u32 {
                    Some(v) => v.to_json_value(), None => Value::Null });
                for (k, v) in [("field_345_u32", field_345_u32),
                               ("field_346_u32", field_346_u32),
                               ("field_347_u32", field_347_u32),
                               ("field_348_u32", field_348_u32),
                               ("field_349_u32", field_349_u32),
                               ("field_350_u32", field_350_u32),
                               ("field_351_u32", field_351_u32),
                               ("field_352_u32", field_352_u32),
                               ("field_353_u32", field_353_u32),
                               ("field_354_u32", field_354_u32),
                               ("field_355_u32", field_355_u32),
                               ("field_356_u32", field_356_u32),
                               ("field_357_u32", field_357_u32),
                               ("field_358_u32", field_358_u32),
                               ("field_359_u32", field_359_u32),
                               ("field_360_u32", field_360_u32),
                               ("field_361_u32", field_361_u32),
                               ("field_362_u32", field_362_u32),
                               ("field_363_u32", field_363_u32),
                               ("field_364_u32", field_364_u32),
                               ("field_365_u32", field_365_u32),
                               ("field_366_u32", field_366_u32),
                               ("field_367_u32", field_367_u32),
                               ("field_368_u32", field_368_u32),
                               ("field_369_u32", field_369_u32),
                               ("field_370_u32", field_370_u32),
                               ("field_371_u32", field_371_u32),
                               ("field_372_u32", field_372_u32),
                               ("field_373_u32", field_373_u32),
                               ("field_374_u32", field_374_u32),
                               ("field_375_u32", field_375_u32),
                               ("field_376_u32", field_376_u32),
                               ("field_377_u32", field_377_u32),
                               ("field_378_u32", field_378_u32),
                               ("field_379_u32", field_379_u32),
                               ("field_380_u32", field_380_u32),
                               ("field_381_u32", field_381_u32),
                               ("field_382_u32", field_382_u32),
                               ("field_383_u32", field_383_u32),
                               ("field_384_u32", field_384_u32),
                               ("field_385_u32", field_385_u32),
                               ("field_386_u32", field_386_u32),
                               ("field_387_u32", field_387_u32),
                               ("field_388_u32", field_388_u32),
                               ("field_389_u32", field_389_u32),
                               ("field_390_u32", field_390_u32),
                               ("field_391_u32", field_391_u32),
                               ("field_392_u32", field_392_u32),
                               ("field_393_u32", field_393_u32),
                               ("field_394_u32", field_394_u32),
                               ("field_395_u32", field_395_u32),
                               ("field_396_u32", field_396_u32),
                               ("field_397_u32", field_397_u32),
                               ("field_398_u32", field_398_u32),
                               ("field_399_u32", field_399_u32),
                               ("field_400_u32", field_400_u32),
                               ("field_401_u32", field_401_u32),
                               ("field_402_u32", field_402_u32),
                               ("field_403_u32", field_403_u32),
                               ("field_404_u32", field_404_u32),
                               ("field_405_u32", field_405_u32),
                               ("field_406_u32", field_406_u32),
                               ("field_407_u32", field_407_u32),
                               ("field_408_u32", field_408_u32),
                               ("field_409_u32", field_409_u32),
                               ("field_410_u32", field_410_u32),
                               ("field_411_u32", field_411_u32),
                               ("field_412_u32", field_412_u32),
                               ("field_413_u32", field_413_u32),
                               ("field_414_u32", field_414_u32),
                               ("field_415_u32", field_415_u32),
                               ("field_416_u32", field_416_u32),
                               ("field_417_u32", field_417_u32),
                               ("field_418_u32", field_418_u32),
                               ("field_419_u32", field_419_u32),
                               ("field_420_u32", field_420_u32),
                               ("field_421_u32", field_421_u32),
                               ("field_422_u32", field_422_u32),
                               ("field_423_u32", field_423_u32),
                               ("field_424_u32", field_424_u32),
                               ("field_425_u32", field_425_u32),
                               ("field_426_u32", field_426_u32),
                               ("field_427_u32", field_427_u32),
                               ("field_428_u32", field_428_u32),
                               ("field_429_u32", field_429_u32),
                               ("field_430_u32", field_430_u32),
                               ("field_431_u32", field_431_u32),
                               ("field_432_u32", field_432_u32),
                               ("field_433_u32", field_433_u32),
                               ("field_434_u32", field_434_u32),
                               ("field_435_u32", field_435_u32),
                               ("field_436_u32", field_436_u32),
                               ("field_437_u32", field_437_u32),
                               ("field_438_u32", field_438_u32),
                               ("field_439_u32", field_439_u32),
                               ("field_440_u32", field_440_u32),
                               ("field_441_u32", field_441_u32),
                               ("field_442_u32", field_442_u32),
                               ("field_443_u32", field_443_u32),
                               ("field_444_u32", field_444_u32),
                               ("field_445_u32", field_445_u32),
                               ("field_446_u32", field_446_u32),
                               ("field_447_u32", field_447_u32),
                               ("field_448_u32", field_448_u32),
                               ("field_449_u32", field_449_u32),
                               ("field_450_u32", field_450_u32),
                               ("field_451_u32", field_451_u32),
                               ("field_452_u32", field_452_u32),
                               ("field_453_u32", field_453_u32),
                               ("field_454_u32", field_454_u32),
                               ("field_455_u32", field_455_u32),
                               ("field_456_u32", field_456_u32),
                               ("field_457_u32", field_457_u32),
                               ("field_458_u32", field_458_u32),
                               ("field_459_u32", field_459_u32),
                               ("field_460_u32", field_460_u32),
                               ("field_461_u32", field_461_u32),
                               ("field_462_u32", field_462_u32),
                               ("field_463_u32", field_463_u32),
                               ("field_464_u32", field_464_u32),
                               ("field_465_u32", field_465_u32),
                               ("field_466_u32", field_466_u32),
                               ("field_467_u32", field_467_u32),
                               ("field_468_u32", field_468_u32),
                               ("field_469_u32", field_469_u32),
                               ("field_470_u32", field_470_u32),
                               ("field_471_u32", field_471_u32),
                               ("field_472_u32", field_472_u32),
                               ("field_473_u32", field_473_u32),
                               ("field_474_u32", field_474_u32),
                               ("field_475_u32", field_475_u32),
                               ("field_476_u32", field_476_u32),
                               ("field_477_u32", field_477_u32),
                               ("field_478_u32", field_478_u32),
                               ("field_479_u32", field_479_u32),
                               ("field_480_u32", field_480_u32),
                               ("field_481_u32", field_481_u32),
                               ("field_482_u32", field_482_u32),
                               ("field_483_u32", field_483_u32),
                               ("field_484_u32", field_484_u32),
                               ("field_485_u32", field_485_u32),
                               ("field_486_u32", field_486_u32),
                               ("field_487_u32", field_487_u32),
                               ("field_488_u32", field_488_u32),
                               ("field_489_u32", field_489_u32),
                               ("field_490_u32", field_490_u32),
                               ("field_491_u32", field_491_u32),
                               ("field_492_u32", field_492_u32),
                               ("field_493_u32", field_493_u32),
                               ("field_494_u32", field_494_u32),
                               ("field_495_u32", field_495_u32),
                               ("field_496_u32", field_496_u32),
                               ("field_497_u32", field_497_u32),
                               ("field_498_u32", field_498_u32),
                               ("field_499_u32", field_499_u32),
                               ("field_500_u32", field_500_u32),
                               ("field_501_u32", field_501_u32),
                               ("field_502_u32", field_502_u32),
                               ("field_503_u32", field_503_u32),
                               ("field_504_u32", field_504_u32),
                               ("field_505_u32", field_505_u32),
                               ("field_506_u32", field_506_u32),
                               ("field_507_u32", field_507_u32),
                               ("field_508_u32", field_508_u32),
                               ("field_509_u32", field_509_u32),
                               ("field_510_u32", field_510_u32),
                               ("field_511_u32", field_511_u32),
                               ("field_512_u32", field_512_u32),
                               ("field_513_u32", field_513_u32),
                               ("field_514_u32", field_514_u32),
                               ("field_515_u32", field_515_u32),
                               ("field_516_u32", field_516_u32),
                               ("field_517_u32", field_517_u32),
                               ("field_518_u32", field_518_u32),
                               ("field_519_u32", field_519_u32),
                               ("field_520_u32", field_520_u32),
                               ("field_521_u32", field_521_u32),
                               ("field_522_u32", field_522_u32),
                               ("field_523_u32", field_523_u32),
                               ("field_524_u32", field_524_u32),
                               ("field_525_u32", field_525_u32),
                               ("field_526_u32", field_526_u32),
                               ("field_527_u32", field_527_u32),
                               ("field_528_u32", field_528_u32),
                               ("field_529_u32", field_529_u32),
                               ("field_530_u32", field_530_u32),
                               ("field_531_u32", field_531_u32),
                               ("field_532_u32", field_532_u32),
                               ("field_533_u32", field_533_u32),
                               ("field_534_u32", field_534_u32),
                               ("field_535_u32", field_535_u32),
                               ("field_536_u32", field_536_u32),
                               ("field_537_u32", field_537_u32), ("field_538_u32", field_538_u32),
                               ("field_539_u32", field_539_u32), ("field_540_u32", field_540_u32),
                               ("field_541_u32", field_541_u32), ("field_542_u32", field_542_u32),
                               ("field_543_u32", field_543_u32), ("field_544_u32", field_544_u32),
                               ("field_545_u32", field_545_u32), ("field_546_u32", field_546_u32),
                               ("field_547_u32", field_547_u32), ("field_548_u32", field_548_u32),
                               ("field_549_u32", field_549_u32), ("field_550_u32", field_550_u32),
                               ("field_551_u32", field_551_u32), ("field_552_u32", field_552_u32),
                               ("field_553_u32", field_553_u32), ("field_554_u32", field_554_u32),
                               ("field_555_u32", field_555_u32), ("field_556_u32", field_556_u32),
                               ("field_557_u32", field_557_u32), ("field_558_u32", field_558_u32),
                               ("field_559_u32", field_559_u32), ("field_560_u32", field_560_u32),
                               ("field_561_u32", field_561_u32), ("field_562_u32", field_562_u32),
                               ("field_563_u32", field_563_u32), ("field_564_u32", field_564_u32),
                               ("field_565_u32", field_565_u32), ("field_566_u32", field_566_u32),
                               ("field_567_u32", field_567_u32), ("field_568_u32", field_568_u32),
                               ("field_569_u32", field_569_u32), ("field_570_u32", field_570_u32),
                               ("field_571_u32", field_571_u32), ("field_572_u32", field_572_u32),
                               ("field_573_u32", field_573_u32), ("field_574_u32", field_574_u32),
                               ("field_575_u32", field_575_u32), ("field_576_u32", field_576_u32),
                               ("field_577_u32", field_577_u32), ("field_578_u32", field_578_u32),
                               ("field_579_u32", field_579_u32), ("field_580_u32", field_580_u32),
                               ("field_581_u32", field_581_u32), ("field_582_u32", field_582_u32),
                               ("field_583_u32", field_583_u32), ("field_584_u32", field_584_u32),
                               ("field_585_u32", field_585_u32), ("field_586_u32", field_586_u32),
                               ("field_587_u32", field_587_u32), ("field_588_u32", field_588_u32),
                               ("field_589_u32", field_589_u32), ("field_590_u32", field_590_u32),
                               ("field_591_u32", field_591_u32), ("field_592_u32", field_592_u32),
                               ("field_593_u32", field_593_u32), ("field_594_u32", field_594_u32),
                               ("field_595_u32", field_595_u32), ("field_596_u32", field_596_u32),
                               ("field_597_u32", field_597_u32), ("field_598_u32", field_598_u32),
                               ("field_599_u32", field_599_u32), ("field_600_u32", field_600_u32),
                               ("field_601_u32", field_601_u32), ("field_602_u32", field_602_u32),
                               ("field_603_u32", field_603_u32), ("field_604_u32", field_604_u32),
                               ("field_605_u32", field_605_u32), ("field_606_u32", field_606_u32),
                               ("field_607_u32", field_607_u32), ("field_608_u32", field_608_u32),
                               ("field_609_u32", field_609_u32), ("field_610_u32", field_610_u32),
                               ("field_611_u32", field_611_u32), ("field_612_u32", field_612_u32),
                               ("field_613_u32", field_613_u32), ("field_614_u32", field_614_u32),
                               ("field_615_u32", field_615_u32), ("field_616_u32", field_616_u32),
                               ("field_617_u32", field_617_u32), ("field_618_u32", field_618_u32),
                               ("field_619_u32", field_619_u32), ("field_620_u32", field_620_u32),
                               ("field_621_u32", field_621_u32), ("field_622_u32", field_622_u32),
                               ("field_623_u32", field_623_u32), ("field_624_u32", field_624_u32),
                               ("field_625_u32", field_625_u32), ("field_626_u32", field_626_u32),
                               ("field_627_u32", field_627_u32), ("field_628_u32", field_628_u32),
                               ("field_629_u32", field_629_u32), ("field_630_u32", field_630_u32),
                               ("field_631_u32", field_631_u32), ("field_632_u32", field_632_u32),
                               ("field_633_u32", field_633_u32), ("field_634_u32", field_634_u32),
                               ("field_635_u32", field_635_u32), ("field_636_u32", field_636_u32),
                               ("field_637_u32", field_637_u32), ("field_638_u32", field_638_u32),
                               ("field_639_u32", field_639_u32), ("field_640_u32", field_640_u32),
                               ("field_641_u32", field_641_u32), ("field_642_u32", field_642_u32),
                               ("field_643_u32", field_643_u32), ("field_644_u32", field_644_u32),
                               ("field_645_u32", field_645_u32), ("field_646_u32", field_646_u32),
                               ("field_647_u32", field_647_u32), ("field_648_u32", field_648_u32),
                               ("field_649_u32", field_649_u32), ("field_650_u32", field_650_u32),
                               ("field_651_u32", field_651_u32), ("field_652_u32", field_652_u32),
                               ("field_653_u32", field_653_u32), ("field_654_u32", field_654_u32),
                               ("field_655_u32", field_655_u32), ("field_656_u32", field_656_u32),
                               ("field_657_u32", field_657_u32), ("field_658_u32", field_658_u32),
                               ("field_659_u32", field_659_u32), ("field_660_u32", field_660_u32),
                               ("field_661_u32", field_661_u32), ("field_662_u32", field_662_u32),
                               ("field_663_u32", field_663_u32), ("field_664_u32", field_664_u32),
                               ("alt_trigger_count", alt_trigger_count),
                               ("field_665_u32", field_665_u32), ("field_666_u32", field_666_u32),
                               ("field_667_u32", field_667_u32), ("field_668_u32", field_668_u32),
                               ("field_669_u32", field_669_u32), ("field_670_u32", field_670_u32),
                               ("field_671_u32", field_671_u32), ("field_672_u32", field_672_u32),
                               ("field_673_u32", field_673_u32), ("field_674_u32", field_674_u32),
                               ("field_675_u32", field_675_u32), ("field_676_u32", field_676_u32),
                               ("field_677_u32", field_677_u32), ("field_678_u32", field_678_u32),
                               ("field_679_u32", field_679_u32), ("field_680_u32", field_680_u32),
                               ("field_681_u32", field_681_u32), ("field_682_u32", field_682_u32),
                               ("field_683_u32", field_683_u32), ("field_684_u32", field_684_u32),
                               ("field_685_u32", field_685_u32), ("field_686_u32", field_686_u32),
                               ("field_687_u32", field_687_u32), ("field_688_u32", field_688_u32),
                               ("field_689_u32", field_689_u32), ("field_690_u32", field_690_u32),
                               ("field_691_u32", field_691_u32), ("field_692_u32", field_692_u32),
                               ("field_693_u32", field_693_u32), ("field_694_u32", field_694_u32),
                               ("field_695_u32", field_695_u32), ("field_696_u32", field_696_u32),
                               ("field_697_u32", field_697_u32), ("field_698_u32", field_698_u32),
                               ("field_699_u32", field_699_u32), ("field_700_u32", field_700_u32),
                               ("field_701_u32", field_701_u32), ("field_702_u32", field_702_u32),
                               ("field_703_u32", field_703_u32), ("field_704_u32", field_704_u32),
                               ("field_705_u32", field_705_u32), ("field_706_u32", field_706_u32),
                               ("field_707_u32", field_707_u32), ("field_708_u32", field_708_u32),
                               ("field_709_u32", field_709_u32), ("field_710_u32", field_710_u32),
                               ("field_711_u32", field_711_u32), ("field_712_u32", field_712_u32),
                               ("field_713_u32", field_713_u32), ("field_714_u32", field_714_u32),
                               ("field_715_u32", field_715_u32), ("field_716_u32", field_716_u32),
                               ("field_717_u32", field_717_u32), ("field_718_u32", field_718_u32),
                               ("field_719_u32", field_719_u32), ("field_720_u32", field_720_u32),
                               ("field_721_u32", field_721_u32), ("field_722_u32", field_722_u32),
                               ("field_723_u32", field_723_u32), ("field_724_u32", field_724_u32),
                               ("field_725_u32", field_725_u32), ("field_726_u32", field_726_u32),
                               ("field_727_u32", field_727_u32), ("field_728_u32", field_728_u32)] {
                    m.insert(k.to_string(), match v {
                        Some(val) => val.to_json_value(), None => Value::Null });
                }
                m.insert("alt_trigger_flag".to_string(), match alt_trigger_flag {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("alt_trigger_name".to_string(), match alt_trigger_name {
                    Some(s) => s.to_json_value(), None => Value::Null });
                m.insert("alt_inner_count".to_string(), match alt_inner_count {
                    Some(v) => v.to_json_value(), None => Value::Null });
                m.insert("alt_inner_name".to_string(), match alt_inner_name {
                    Some(s) => s.to_json_value(), None => Value::Null });
                m.insert("alt_inner_flag".to_string(), match alt_inner_flag {
                    Some(v) => v.to_json_value(), None => Value::Null });
                for (k, v) in [
                    ("alt_body_001", alt_body_001), ("alt_body_002", alt_body_002),
                    ("alt_body_003", alt_body_003), ("alt_body_004", alt_body_004),
                    ("alt_body_005", alt_body_005), ("alt_body_006", alt_body_006),
                    ("alt_body_007", alt_body_007), ("alt_body_008", alt_body_008),
                    ("alt_body_009", alt_body_009), ("alt_body_010", alt_body_010),
                    ("alt_body_011", alt_body_011), ("alt_body_012", alt_body_012),
                    ("alt_body_013", alt_body_013), ("alt_body_014", alt_body_014),
                    ("alt_body_015", alt_body_015), ("alt_body_016", alt_body_016),
                    ("alt_body_017", alt_body_017), ("alt_body_018", alt_body_018),
                    ("alt_body_019", alt_body_019), ("alt_body_020", alt_body_020),
                    ("alt_body_021", alt_body_021), ("alt_body_022", alt_body_022),
                    ("alt_body_023", alt_body_023), ("alt_body_024", alt_body_024),
                    ("alt_body_025", alt_body_025), ("alt_body_026", alt_body_026),
                    ("alt_body_027", alt_body_027), ("alt_body_028", alt_body_028),
                    ("alt_body_029", alt_body_029), ("alt_body_030", alt_body_030),
                    ("alt_body_031", alt_body_031), ("alt_body_032", alt_body_032),
                    ("alt_body_033", alt_body_033), ("alt_body_034", alt_body_034),
                    ("alt_body_035", alt_body_035), ("alt_body_036", alt_body_036),
                    ("alt_body_037", alt_body_037), ("alt_body_038", alt_body_038),
                    ("alt_body_039", alt_body_039), ("alt_body_040", alt_body_040),
                    ("alt_body_041", alt_body_041), ("alt_body_042", alt_body_042),
                    ("alt_body_043", alt_body_043), ("alt_body_044", alt_body_044),
                    ("alt_body_045", alt_body_045), ("alt_body_046", alt_body_046),
                    ("alt_body_047", alt_body_047), ("alt_body_048", alt_body_048),
                    ("alt_body_049", alt_body_049), ("alt_body_050", alt_body_050),
                    ("alt_body_051", alt_body_051), ("alt_body_052", alt_body_052),
                    ("alt_body_053", alt_body_053), ("alt_body_054", alt_body_054),
                    ("alt_body_055", alt_body_055), ("alt_body_056", alt_body_056),
                    ("alt_body_057", alt_body_057), ("alt_body_058", alt_body_058),
                    ("alt_body_059", alt_body_059), ("alt_body_060", alt_body_060),
                    ("alt_body_061", alt_body_061), ("alt_body_062", alt_body_062),
                    ("alt_body_063", alt_body_063), ("alt_body_064", alt_body_064),
                    ("alt_body_065", alt_body_065), ("alt_body_066", alt_body_066),
                    ("alt_body_067", alt_body_067), ("alt_body_068", alt_body_068),
                    ("alt_body_069", alt_body_069), ("alt_body_070", alt_body_070),
                    ("alt_body_071", alt_body_071), ("alt_body_072", alt_body_072),
                    ("alt_body_073", alt_body_073), ("alt_body_074", alt_body_074),
                    ("alt_body_075", alt_body_075), ("alt_body_076", alt_body_076),
                    ("alt_body_077", alt_body_077), ("alt_body_078", alt_body_078),
                    ("alt_body_079", alt_body_079), ("alt_body_080", alt_body_080),
                    ("alt_body_081", alt_body_081), ("alt_body_082", alt_body_082),
                    ("alt_body_083", alt_body_083), ("alt_body_084", alt_body_084),
                    ("alt_body_085", alt_body_085), ("alt_body_086", alt_body_086),
                    ("alt_body_087", alt_body_087), ("alt_body_088", alt_body_088),
                    ("alt_body_089", alt_body_089), ("alt_body_090", alt_body_090),
                    ("alt_body_091", alt_body_091), ("alt_body_092", alt_body_092),
                    ("alt_body_093", alt_body_093), ("alt_body_094", alt_body_094),
                    ("alt_body_095", alt_body_095), ("alt_body_096", alt_body_096),
                    ("alt_body_097", alt_body_097), ("alt_body_098", alt_body_098),
                    ("alt_body_099", alt_body_099), ("alt_body_100", alt_body_100),
                    ("alt_body_101", alt_body_101), ("alt_body_102", alt_body_102),
                    ("alt_body_103", alt_body_103), ("alt_body_104", alt_body_104),
                    ("alt_body_105", alt_body_105), ("alt_body_106", alt_body_106),
                    ("alt_body_107", alt_body_107), ("alt_body_108", alt_body_108),
                    ("alt_body_109", alt_body_109), ("alt_body_110", alt_body_110),
                    ("alt_body_111", alt_body_111), ("alt_body_112", alt_body_112),
                    ("alt_body_113", alt_body_113), ("alt_body_114", alt_body_114),
                    ("alt_body_115", alt_body_115), ("alt_body_116", alt_body_116),
                    ("alt_body_117", alt_body_117), ("alt_body_118", alt_body_118),
                    ("alt_body_119", alt_body_119), ("alt_body_120", alt_body_120),
                    ("alt_body_121", alt_body_121), ("alt_body_122", alt_body_122),
                    ("alt_body_123", alt_body_123), ("alt_body_124", alt_body_124),
                    ("alt_body_125", alt_body_125), ("alt_body_126", alt_body_126),
                    ("alt_body_127", alt_body_127), ("alt_body_128", alt_body_128),
                    ("alt_body_129", alt_body_129), ("alt_body_130", alt_body_130),
                    ("alt_body_131", alt_body_131), ("alt_body_132", alt_body_132),
                    ("alt_body_133", alt_body_133), ("alt_body_134", alt_body_134),
                    ("alt_body_135", alt_body_135), ("alt_body_136", alt_body_136),
                    ("alt_body_137", alt_body_137), ("alt_body_138", alt_body_138),
                    ("alt_body_139", alt_body_139), ("alt_body_140", alt_body_140),
                    ("alt_body_141", alt_body_141), ("alt_body_142", alt_body_142),
                    ("alt_body_143", alt_body_143), ("alt_body_144", alt_body_144),
                    ("alt_body_145", alt_body_145), ("alt_body_146", alt_body_146),
                    ("alt_body_147", alt_body_147), ("alt_body_148", alt_body_148),
                    ("alt_body_149", alt_body_149), ("alt_body_150", alt_body_150),
                    ("alt_body_151", alt_body_151), ("alt_body_152", alt_body_152),
                    ("alt_body_153", alt_body_153), ("alt_body_154", alt_body_154),
                    ("alt_body_155", alt_body_155), ("alt_body_156", alt_body_156),
                    ("alt_body_157", alt_body_157), ("alt_body_158", alt_body_158),
                    ("alt_body_159", alt_body_159), ("alt_body_160", alt_body_160),
                    ("alt_body_161", alt_body_161), ("alt_body_162", alt_body_162),
                    ("alt_body_163", alt_body_163), ("alt_body_164", alt_body_164),
                    ("alt_body_165", alt_body_165), ("alt_body_166", alt_body_166),
                    ("alt_body_167", alt_body_167), ("alt_body_168", alt_body_168),
                    ("alt_body_169", alt_body_169), ("alt_body_170", alt_body_170),
                    ("alt_body_171", alt_body_171), ("alt_body_172", alt_body_172),
                    ("alt_body_173", alt_body_173), ("alt_body_174", alt_body_174),
                    ("alt_body_175", alt_body_175), ("alt_body_176", alt_body_176),
                    ("alt_body_177", alt_body_177), ("alt_body_178", alt_body_178),
                    ("alt_body_179", alt_body_179), ("alt_body_180", alt_body_180),
                    ("alt_body_181", alt_body_181), ("alt_body_182", alt_body_182),
                    ("alt_body_183", alt_body_183), ("alt_body_184", alt_body_184),
                    ("alt_body_185", alt_body_185), ("alt_body_186", alt_body_186),
                    ("alt_body_187", alt_body_187), ("alt_body_188", alt_body_188),
                    ("alt_body_189", alt_body_189), ("alt_body_190", alt_body_190),
                    ("alt_body_191", alt_body_191), ("alt_body_192", alt_body_192),
                    ("alt_body_193", alt_body_193), ("alt_body_194", alt_body_194),
                    ("alt_body_195", alt_body_195), ("alt_body_196", alt_body_196),
                    ("alt_body_197", alt_body_197), ("alt_body_198", alt_body_198),
                    ("alt_body_199", alt_body_199), ("alt_body_200", alt_body_200),
                    ("alt_body_201", alt_body_201), ("alt_body_202", alt_body_202),
                    ("alt_body_203", alt_body_203), ("alt_body_204", alt_body_204),
                    ("alt_body_205", alt_body_205), ("alt_body_206", alt_body_206),
                    ("alt_body_207", alt_body_207), ("alt_body_208", alt_body_208),
                    ("alt_body_209", alt_body_209), ("alt_body_210", alt_body_210),
                    ("alt_body_211", alt_body_211), ("alt_body_212", alt_body_212),
                    ("alt_body_213", alt_body_213), ("alt_body_214", alt_body_214),
                    ("alt_body_215", alt_body_215), ("alt_body_216", alt_body_216),
                    ("alt_body_217", alt_body_217), ("alt_body_218", alt_body_218),
                    ("alt_body_219", alt_body_219), ("alt_body_220", alt_body_220),
                    ("alt_body_221", alt_body_221), ("alt_body_222", alt_body_222),
                    ("alt_body_223", alt_body_223), ("alt_body_224", alt_body_224),
                    ("alt_body_225", alt_body_225), ("alt_body_226", alt_body_226),
                    ("alt_body_227", alt_body_227), ("alt_body_228", alt_body_228),
                    ("alt_body_229", alt_body_229), ("alt_body_230", alt_body_230),
                    ("alt_body_231", alt_body_231), ("alt_body_232", alt_body_232),
                    ("alt_body_233", alt_body_233), ("alt_body_234", alt_body_234),
                    ("alt_body_235", alt_body_235), ("alt_body_236", alt_body_236),
                    ("alt_body_237", alt_body_237), ("alt_body_238", alt_body_238),
                    ("alt_body_239", alt_body_239), ("alt_body_240", alt_body_240),
                    ("alt_body_241", alt_body_241), ("alt_body_242", alt_body_242),
                    ("alt_body_243", alt_body_243), ("alt_body_244", alt_body_244),
                    ("alt_body_245", alt_body_245), ("alt_body_246", alt_body_246),
                    ("alt_body_247", alt_body_247), ("alt_body_248", alt_body_248),
                    ("alt_body_249", alt_body_249), ("alt_body_250", alt_body_250),
                    ("alt_body_251", alt_body_251), ("alt_body_252", alt_body_252),
                    ("alt_body_253", alt_body_253), ("alt_body_254", alt_body_254),
                    ("alt_body_255", alt_body_255), ("alt_body_256", alt_body_256),
                    ("alt_body_257", alt_body_257), ("alt_body_258", alt_body_258),
                    ("alt_body_259", alt_body_259), ("alt_body_260", alt_body_260),
                    ("alt_body_261", alt_body_261), ("alt_body_262", alt_body_262),
                    ("alt_body_263", alt_body_263), ("alt_body_264", alt_body_264),
                    ("alt_body_265", alt_body_265), ("alt_body_266", alt_body_266),
                    ("alt_body_267", alt_body_267), ("alt_body_268", alt_body_268),
                    ("alt_body_269", alt_body_269), ("alt_body_270", alt_body_270),
                    ("alt_body_271", alt_body_271), ("alt_body_272", alt_body_272),
                    ("alt_body_273", alt_body_273), ("alt_body_274", alt_body_274),
                    ("alt_body_275", alt_body_275), ("alt_body_276", alt_body_276),
                    ("alt_body_277", alt_body_277), ("alt_body_278", alt_body_278),
                    ("alt_body_279", alt_body_279), ("alt_body_280", alt_body_280),
                    ("alt_body_281", alt_body_281), ("alt_body_282", alt_body_282),
                    ("alt_body_283", alt_body_283), ("alt_body_284", alt_body_284),
                    ("alt_body_285", alt_body_285), ("alt_body_286", alt_body_286),
                    ("alt_body_287", alt_body_287), ("alt_body_288", alt_body_288),
                    ("alt_body_289", alt_body_289), ("alt_body_290", alt_body_290),
                    ("alt_body_291", alt_body_291), ("alt_body_292", alt_body_292),
                    ("alt_body_293", alt_body_293), ("alt_body_294", alt_body_294),
                    ("alt_body_295", alt_body_295), ("alt_body_296", alt_body_296),
                    ("alt_body_297", alt_body_297), ("alt_body_298", alt_body_298),
                    ("alt_body_299", alt_body_299), ("alt_body_300", alt_body_300),
                    ("alt_body_301", alt_body_301), ("alt_body_302", alt_body_302),
                    ("alt_body_303", alt_body_303), ("alt_body_304", alt_body_304),
                    ("alt_body_305", alt_body_305), ("alt_body_306", alt_body_306),
                    ("alt_body_307", alt_body_307), ("alt_body_308", alt_body_308),
                    ("alt_body_309", alt_body_309), ("alt_body_310", alt_body_310),
                    ("alt_body_311", alt_body_311), ("alt_body_312", alt_body_312),
                    ("alt_body_313", alt_body_313), ("alt_body_314", alt_body_314),
                    ("alt_body_315", alt_body_315), ("alt_body_316", alt_body_316),
                    ("alt_body_317", alt_body_317), ("alt_body_318", alt_body_318),
                    ("alt_body_319", alt_body_319), ("alt_body_320", alt_body_320),
                ] {
                    m.insert(k.to_string(), match v {
                        Some(val) => val.to_json_value(), None => Value::Null });
                }
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
                           "field_106_u32", "field_107_u32", "field_108_u32",
                           "field_109_u32", "field_110_u32", "field_111_u32",
                           "field_112_u32", "field_113_u32", "field_114_u32",
                           "field_115_u32", "field_116_u32", "field_117_u32",
                           "field_118_u32", "field_119_u32", "field_120_u32",
                           "field_121_u32", "field_122_u32", "field_123_u32",
                           "field_124_u32", "field_125_u32", "field_126_u32",
                           "field_127_u32", "field_128_u32", "field_129_u32",
                           "field_130_u32", "field_131_u32", "field_132_u32",
                           "field_133_u32", "field_134_u32", "field_135_u32",
                           "field_136_u32", "field_137_u32", "field_138_u32",
                           "field_139_u32", "field_140_u32", "field_141_u32",
                           "field_142_u32", "field_143_u32", "field_144_u32",
                           "field_145_u32", "field_146_u32", "field_147_u32",
                           "field_148_u32", "field_149_u32", "field_150_u32",
                           "field_151_u32", "field_152_u32", "field_153_u32",
                           "field_154_u32", "field_155_u32", "field_156_u32",
                           "field_157_u32", "field_158_u32", "field_159_u32",
                           "field_160_u32", "field_161_u32", "field_162_u32",
                           "field_163_u32", "field_164_u32", "field_165_u32",
                           "field_166_u32", "field_167_u32", "field_168_u32",
                           "field_169_u32", "field_170_u32", "field_171_u32",
                           "field_172_u32", "field_173_u32", "field_174_u32",
                           "field_175_u32", "field_176_u32", "field_177_u32",
                           "field_178_u32", "field_179_u32", "field_180_u32",
                           "field_181_u32", "field_182_u32", "field_183_u32",
                           "field_184_u32", "field_185_u32", "field_186_u32",
                           "field_187_u32", "field_188_u32", "field_189_u32",
                           "field_190_u32", "field_191_u32", "field_192_u32",
                           "field_193_u32", "field_194_u32", "field_195_u32",
                           "field_196_u32", "field_197_u32", "field_198_u32",
                           "field_199_u32", "field_200_u32", "field_201_u32",
                           "field_202_u32", "field_203_u32", "field_204_u32",
                           "field_205_u32", "field_206_u32", "field_207_u32",
                           "field_208_u32", "field_209_u32", "field_210_u32",
                           "field_211_u32", "field_212_u32", "field_213_u32",
                           "field_214_u32", "field_215_u32", "field_216_u32",
                           "field_217_u32", "field_218_u32", "field_219_u32",
                           "field_220_u32", "field_221_u32", "field_222_u32",
                           "field_223_u32", "field_224_u32", "field_225_u32",
                           "field_226_u32", "field_227_u32", "field_228_u32",
                           "field_229_u32", "field_230_u32", "field_231_u32",
                           "field_232_u32", "field_233_u32", "field_234_u32",
                           "field_235_u32", "field_236_u32", "field_237_u32",
                           "field_238_u32", "field_239_u32", "field_240_u32",
                           "field_241_u32", "field_242_u32", "field_243_u32",
                           "field_244_u32", "field_245_u32", "field_246_u32",
                           "field_247_u32", "field_248_u32", "field_249_u32",
                           "field_250_u32", "field_251_u32", "field_252_u32",
                           "field_253_u32", "field_254_u32", "field_255_u32",
                           "field_256_u32", "field_257_u32", "field_258_u32",
                           "field_259_u32", "field_260_u32", "field_261_u32",
                           "field_262_u32", "field_263_u32", "field_264_u32",
                           "field_265_u32", "field_266_u32", "field_267_u32",
                           "field_268_u32", "field_269_u32", "field_270_u32",
                           "field_271_u32", "field_272_u32", "field_273_u32",
                           "field_274_u32", "field_275_u32", "field_276_u32",
                           "field_277_u32", "field_278_u32", "field_279_u32",
                           "field_280_u32", "field_281_u32", "field_282_u32",
                           "field_283_u32", "field_284_u32", "field_285_u32",
                           "field_286_u32", "field_287_u32", "field_288_u32",
                           "field_289_u32", "field_290_u32", "field_291_u32",
                           "field_292_u32", "field_293_u32", "field_294_u32",
                           "field_295_u32", "field_296_u32", "field_297_u32",
                           "field_298_u32", "field_299_u32", "field_300_u32",
                           "field_301_u32", "field_302_u32", "field_303_u32",
                           "field_304_u32", "field_305_u32", "field_306_u32",
                           "field_307_u32", "field_308_u32", "field_309_u32",
                           "field_310_u32", "field_311_u32", "field_312_u32",
                           "field_313_u32", "field_314_u32", "field_315_u32",
                           "field_316_u32", "field_317_u32", "field_318_u32",
                           "field_319_u32", "field_320_u32", "field_321_u32",
                           "field_322_u32", "field_323_u32", "field_324_u32",
                           "field_325_u32", "field_326_u32", "field_327_u32",
                           "field_328_u32", "field_329_u32", "field_330_u32",
                           "field_331_u32", "field_332_u32", "field_333_u32",
                           "field_334_u32", "field_335_u32", "field_336_u32",
                           "field_337_u32", "field_338_u32", "field_339_u32",
                           "field_340_u32", "field_341_u32", "field_342_u32_count"] {
                    let v = json_get_field(obj, k)?;
                    if !v.is_null() {
                        <u32 as WriteJsonValue>::write_from_json(w, v)?;
                    }
                }
                let f343 = json_get_field(obj, "field_343_u8_flag")?;
                if !f343.is_null() {
                    <u8 as WriteJsonValue>::write_from_json(w, f343)?;
                }
                let f344 = json_get_field(obj, "field_344_u32")?;
                if !f344.is_null() {
                    <u32 as WriteJsonValue>::write_from_json(w, f344)?;
                }
                for k in &["field_345_u32", "field_346_u32", "field_347_u32",
                           "field_348_u32", "field_349_u32", "field_350_u32",
                           "field_351_u32", "field_352_u32", "field_353_u32",
                           "field_354_u32", "field_355_u32", "field_356_u32",
                           "field_357_u32", "field_358_u32", "field_359_u32",
                           "field_360_u32", "field_361_u32", "field_362_u32",
                           "field_363_u32", "field_364_u32", "field_365_u32",
                           "field_366_u32", "field_367_u32", "field_368_u32",
                           "field_369_u32", "field_370_u32", "field_371_u32",
                           "field_372_u32", "field_373_u32", "field_374_u32",
                           "field_375_u32", "field_376_u32", "field_377_u32",
                           "field_378_u32", "field_379_u32", "field_380_u32",
                           "field_381_u32", "field_382_u32", "field_383_u32",
                           "field_384_u32", "field_385_u32", "field_386_u32",
                           "field_387_u32", "field_388_u32", "field_389_u32",
                           "field_390_u32", "field_391_u32", "field_392_u32",
                           "field_393_u32", "field_394_u32", "field_395_u32",
                           "field_396_u32", "field_397_u32", "field_398_u32",
                           "field_399_u32", "field_400_u32", "field_401_u32",
                           "field_402_u32", "field_403_u32", "field_404_u32",
                           "field_405_u32", "field_406_u32", "field_407_u32",
                           "field_408_u32", "field_409_u32", "field_410_u32",
                           "field_411_u32", "field_412_u32", "field_413_u32",
                           "field_414_u32", "field_415_u32", "field_416_u32",
                           "field_417_u32", "field_418_u32", "field_419_u32",
                           "field_420_u32", "field_421_u32", "field_422_u32",
                           "field_423_u32", "field_424_u32", "field_425_u32",
                           "field_426_u32", "field_427_u32", "field_428_u32",
                           "field_429_u32", "field_430_u32", "field_431_u32",
                           "field_432_u32", "field_433_u32", "field_434_u32",
                           "field_435_u32", "field_436_u32", "field_437_u32",
                           "field_438_u32", "field_439_u32", "field_440_u32",
                           "field_441_u32", "field_442_u32", "field_443_u32",
                           "field_444_u32", "field_445_u32", "field_446_u32",
                           "field_447_u32", "field_448_u32", "field_449_u32",
                           "field_450_u32", "field_451_u32", "field_452_u32",
                           "field_453_u32", "field_454_u32", "field_455_u32",
                           "field_456_u32", "field_457_u32", "field_458_u32",
                           "field_459_u32", "field_460_u32", "field_461_u32",
                           "field_462_u32", "field_463_u32", "field_464_u32",
                           "field_465_u32", "field_466_u32", "field_467_u32",
                           "field_468_u32", "field_469_u32", "field_470_u32",
                           "field_471_u32", "field_472_u32",
                           "field_473_u32", "field_474_u32", "field_475_u32",
                           "field_476_u32", "field_477_u32", "field_478_u32",
                           "field_479_u32", "field_480_u32", "field_481_u32",
                           "field_482_u32", "field_483_u32", "field_484_u32",
                           "field_485_u32", "field_486_u32", "field_487_u32",
                           "field_488_u32", "field_489_u32", "field_490_u32",
                           "field_491_u32", "field_492_u32", "field_493_u32",
                           "field_494_u32", "field_495_u32", "field_496_u32",
                           "field_497_u32", "field_498_u32", "field_499_u32",
                           "field_500_u32", "field_501_u32", "field_502_u32",
                           "field_503_u32", "field_504_u32", "field_505_u32",
                           "field_506_u32", "field_507_u32", "field_508_u32",
                           "field_509_u32", "field_510_u32", "field_511_u32",
                           "field_512_u32", "field_513_u32", "field_514_u32",
                           "field_515_u32", "field_516_u32", "field_517_u32",
                           "field_518_u32", "field_519_u32", "field_520_u32",
                           "field_521_u32", "field_522_u32", "field_523_u32",
                           "field_524_u32", "field_525_u32", "field_526_u32",
                           "field_527_u32", "field_528_u32", "field_529_u32",
                           "field_530_u32", "field_531_u32", "field_532_u32",
                           "field_533_u32", "field_534_u32", "field_535_u32",
                           "field_536_u32",
                           "field_537_u32", "field_538_u32", "field_539_u32", "field_540_u32",
                           "field_541_u32", "field_542_u32", "field_543_u32", "field_544_u32",
                           "field_545_u32", "field_546_u32", "field_547_u32", "field_548_u32",
                           "field_549_u32", "field_550_u32", "field_551_u32", "field_552_u32",
                           "field_553_u32", "field_554_u32", "field_555_u32", "field_556_u32",
                           "field_557_u32", "field_558_u32", "field_559_u32", "field_560_u32",
                           "field_561_u32", "field_562_u32", "field_563_u32", "field_564_u32",
                           "field_565_u32", "field_566_u32", "field_567_u32", "field_568_u32",
                           "field_569_u32", "field_570_u32", "field_571_u32", "field_572_u32",
                           "field_573_u32", "field_574_u32", "field_575_u32", "field_576_u32",
                           "field_577_u32", "field_578_u32", "field_579_u32", "field_580_u32",
                           "field_581_u32", "field_582_u32", "field_583_u32", "field_584_u32",
                           "field_585_u32", "field_586_u32", "field_587_u32", "field_588_u32",
                           "field_589_u32", "field_590_u32", "field_591_u32", "field_592_u32",
                           "field_593_u32", "field_594_u32", "field_595_u32", "field_596_u32",
                           "field_597_u32", "field_598_u32", "field_599_u32", "field_600_u32",
                           "field_601_u32", "field_602_u32", "field_603_u32", "field_604_u32",
                           "field_605_u32", "field_606_u32", "field_607_u32", "field_608_u32",
                           "field_609_u32", "field_610_u32", "field_611_u32", "field_612_u32",
                           "field_613_u32", "field_614_u32", "field_615_u32", "field_616_u32",
                           "field_617_u32", "field_618_u32", "field_619_u32", "field_620_u32",
                           "field_621_u32", "field_622_u32", "field_623_u32", "field_624_u32",
                           "field_625_u32", "field_626_u32", "field_627_u32", "field_628_u32",
                           "field_629_u32", "field_630_u32", "field_631_u32", "field_632_u32",
                           "field_633_u32", "field_634_u32", "field_635_u32", "field_636_u32",
                           "field_637_u32", "field_638_u32", "field_639_u32", "field_640_u32",
                           "field_641_u32", "field_642_u32", "field_643_u32", "field_644_u32",
                           "field_645_u32", "field_646_u32", "field_647_u32", "field_648_u32",
                           "field_649_u32", "field_650_u32", "field_651_u32", "field_652_u32",
                           "field_653_u32", "field_654_u32", "field_655_u32", "field_656_u32",
                           "field_657_u32", "field_658_u32", "field_659_u32", "field_660_u32",
                           "field_661_u32", "field_662_u32", "field_663_u32", "field_664_u32",
                           "alt_trigger_count",
                           "field_665_u32", "field_666_u32", "field_667_u32", "field_668_u32",
                           "field_669_u32", "field_670_u32", "field_671_u32", "field_672_u32",
                           "field_673_u32", "field_674_u32", "field_675_u32", "field_676_u32",
                           "field_677_u32", "field_678_u32", "field_679_u32", "field_680_u32",
                           "field_681_u32", "field_682_u32", "field_683_u32", "field_684_u32",
                           "field_685_u32", "field_686_u32", "field_687_u32", "field_688_u32",
                           "field_689_u32", "field_690_u32", "field_691_u32", "field_692_u32",
                           "field_693_u32", "field_694_u32", "field_695_u32", "field_696_u32",
                           "field_697_u32", "field_698_u32", "field_699_u32", "field_700_u32",
                           "field_701_u32", "field_702_u32", "field_703_u32", "field_704_u32",
                           "field_705_u32", "field_706_u32", "field_707_u32", "field_708_u32",
                           "field_709_u32", "field_710_u32", "field_711_u32", "field_712_u32",
                           "field_713_u32", "field_714_u32", "field_715_u32", "field_716_u32",
                           "field_717_u32", "field_718_u32", "field_719_u32", "field_720_u32",
                           "field_721_u32", "field_722_u32", "field_723_u32", "field_724_u32",
                           "field_725_u32", "field_726_u32", "field_727_u32", "field_728_u32"] {
                    let v = json_get_field(obj, k)?;
                    if !v.is_null() {
                        <u32 as WriteJsonValue>::write_from_json(w, v)?;
                    }
                }
                let alt_flag = json_get_field(obj, "alt_trigger_flag")?;
                if !alt_flag.is_null() {
                    <u8 as WriteJsonValue>::write_from_json(w, alt_flag)?;
                }
                let alt_name = json_get_field(obj, "alt_trigger_name")?;
                if !alt_name.is_null() {
                    <CString as WriteJsonValue>::write_from_json(w, alt_name)?;
                }
                let aic = json_get_field(obj, "alt_inner_count")?;
                if !aic.is_null() {
                    <u32 as WriteJsonValue>::write_from_json(w, aic)?;
                }
                let ain = json_get_field(obj, "alt_inner_name")?;
                if !ain.is_null() {
                    <CString as WriteJsonValue>::write_from_json(w, ain)?;
                }
                let aif = json_get_field(obj, "alt_inner_flag")?;
                if !aif.is_null() {
                    <u32 as WriteJsonValue>::write_from_json(w, aif)?;
                }
                for k in &[
                    "alt_body_001", "alt_body_002", "alt_body_003", "alt_body_004",
                    "alt_body_005", "alt_body_006", "alt_body_007", "alt_body_008",
                    "alt_body_009", "alt_body_010", "alt_body_011", "alt_body_012",
                    "alt_body_013", "alt_body_014", "alt_body_015", "alt_body_016",
                    "alt_body_017", "alt_body_018", "alt_body_019", "alt_body_020",
                    "alt_body_021", "alt_body_022", "alt_body_023", "alt_body_024",
                    "alt_body_025", "alt_body_026", "alt_body_027", "alt_body_028",
                    "alt_body_029", "alt_body_030", "alt_body_031", "alt_body_032",
                    "alt_body_033", "alt_body_034", "alt_body_035", "alt_body_036",
                    "alt_body_037", "alt_body_038", "alt_body_039", "alt_body_040",
                    "alt_body_041", "alt_body_042", "alt_body_043", "alt_body_044",
                    "alt_body_045", "alt_body_046", "alt_body_047", "alt_body_048",
                    "alt_body_049", "alt_body_050", "alt_body_051", "alt_body_052",
                    "alt_body_053", "alt_body_054", "alt_body_055", "alt_body_056",
                    "alt_body_057", "alt_body_058", "alt_body_059", "alt_body_060",
                    "alt_body_061", "alt_body_062", "alt_body_063", "alt_body_064",
                    "alt_body_065", "alt_body_066", "alt_body_067", "alt_body_068",
                    "alt_body_069", "alt_body_070", "alt_body_071", "alt_body_072",
                    "alt_body_073", "alt_body_074", "alt_body_075", "alt_body_076",
                    "alt_body_077", "alt_body_078", "alt_body_079", "alt_body_080",
                    "alt_body_081", "alt_body_082", "alt_body_083", "alt_body_084",
                    "alt_body_085", "alt_body_086", "alt_body_087", "alt_body_088",
                    "alt_body_089", "alt_body_090", "alt_body_091", "alt_body_092",
                    "alt_body_093", "alt_body_094", "alt_body_095", "alt_body_096",
                    "alt_body_097", "alt_body_098", "alt_body_099", "alt_body_100",
                    "alt_body_101", "alt_body_102", "alt_body_103", "alt_body_104",
                    "alt_body_105", "alt_body_106", "alt_body_107", "alt_body_108",
                    "alt_body_109", "alt_body_110", "alt_body_111", "alt_body_112",
                    "alt_body_113", "alt_body_114", "alt_body_115", "alt_body_116",
                    "alt_body_117", "alt_body_118", "alt_body_119", "alt_body_120",
                    "alt_body_121", "alt_body_122", "alt_body_123", "alt_body_124",
                    "alt_body_125", "alt_body_126", "alt_body_127", "alt_body_128",
                    "alt_body_129", "alt_body_130", "alt_body_131", "alt_body_132",
                    "alt_body_133", "alt_body_134", "alt_body_135", "alt_body_136",
                    "alt_body_137", "alt_body_138", "alt_body_139", "alt_body_140",
                    "alt_body_141", "alt_body_142", "alt_body_143", "alt_body_144",
                    "alt_body_145", "alt_body_146", "alt_body_147", "alt_body_148",
                    "alt_body_149", "alt_body_150", "alt_body_151", "alt_body_152",
                    "alt_body_153", "alt_body_154", "alt_body_155", "alt_body_156",
                    "alt_body_157", "alt_body_158", "alt_body_159", "alt_body_160",
                    "alt_body_161", "alt_body_162", "alt_body_163", "alt_body_164",
                    "alt_body_165", "alt_body_166", "alt_body_167", "alt_body_168",
                    "alt_body_169", "alt_body_170", "alt_body_171", "alt_body_172",
                    "alt_body_173", "alt_body_174", "alt_body_175", "alt_body_176",
                    "alt_body_177", "alt_body_178", "alt_body_179", "alt_body_180",
                    "alt_body_181", "alt_body_182", "alt_body_183", "alt_body_184",
                    "alt_body_185", "alt_body_186", "alt_body_187", "alt_body_188",
                    "alt_body_189", "alt_body_190", "alt_body_191", "alt_body_192",
                    "alt_body_193", "alt_body_194", "alt_body_195", "alt_body_196",
                    "alt_body_197", "alt_body_198", "alt_body_199", "alt_body_200",
                    "alt_body_201", "alt_body_202", "alt_body_203", "alt_body_204",
                    "alt_body_205", "alt_body_206", "alt_body_207", "alt_body_208",
                    "alt_body_209", "alt_body_210", "alt_body_211", "alt_body_212",
                    "alt_body_213", "alt_body_214", "alt_body_215", "alt_body_216",
                    "alt_body_217", "alt_body_218", "alt_body_219", "alt_body_220",
                    "alt_body_221", "alt_body_222", "alt_body_223", "alt_body_224",
                    "alt_body_225", "alt_body_226", "alt_body_227", "alt_body_228",
                    "alt_body_229", "alt_body_230", "alt_body_231", "alt_body_232",
                    "alt_body_233", "alt_body_234", "alt_body_235", "alt_body_236",
                    "alt_body_237", "alt_body_238", "alt_body_239", "alt_body_240",
                    "alt_body_241", "alt_body_242", "alt_body_243", "alt_body_244",
                    "alt_body_245", "alt_body_246", "alt_body_247", "alt_body_248",
                    "alt_body_249", "alt_body_250", "alt_body_251", "alt_body_252",
                    "alt_body_253", "alt_body_254", "alt_body_255", "alt_body_256",
                    "alt_body_257", "alt_body_258", "alt_body_259", "alt_body_260",
                    "alt_body_261", "alt_body_262", "alt_body_263", "alt_body_264",
                    "alt_body_265", "alt_body_266", "alt_body_267", "alt_body_268",
                    "alt_body_269", "alt_body_270", "alt_body_271", "alt_body_272",
                    "alt_body_273", "alt_body_274", "alt_body_275", "alt_body_276",
                    "alt_body_277", "alt_body_278", "alt_body_279", "alt_body_280",
                    "alt_body_281", "alt_body_282", "alt_body_283", "alt_body_284",
                    "alt_body_285", "alt_body_286", "alt_body_287", "alt_body_288",
                    "alt_body_289", "alt_body_290", "alt_body_291", "alt_body_292",
                    "alt_body_293", "alt_body_294", "alt_body_295", "alt_body_296",
                    "alt_body_297", "alt_body_298", "alt_body_299", "alt_body_300",
                    "alt_body_301", "alt_body_302", "alt_body_303", "alt_body_304",
                    "alt_body_305", "alt_body_306", "alt_body_307", "alt_body_308",
                    "alt_body_309", "alt_body_310", "alt_body_311", "alt_body_312",
                    "alt_body_313", "alt_body_314", "alt_body_315", "alt_body_316",
                    "alt_body_317", "alt_body_318", "alt_body_319", "alt_body_320",
                ] {
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
