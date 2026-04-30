//! Tier 1.5 — typed prefix + tail blob.
//!
//! Reader: `sub_1410E6FC0` in CrimsonDesert.exe (Win build). Massive
//! 7205-byte function, 100+ wire reads in the body.
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
//! Steps 1-6 are typed (6 fields). Field 7 blocks promotion: the
//! 144-byte GimmickInteractionOverrideData struct depends on at least 3
//! unresolved helpers (sub_141114FC0, sub_141E2C900, sub_141101AB0).
//! Body has 100+ wire reads.
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
use crate::pabgh_typed_blob_table;

pabgh_typed_blob_table! {
    pub struct GimmickInfo<'a> {
        pub key: u32,
        pub string_key: CString<'a>,
        pub is_blocked: u8,
        pub prefab_path: CString<'a>,
        pub gimmick_group_info: u32,
        pub breakable_object_info: u16,
    }
    tail: tail_blob;
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
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            items.push(
                GimmickInfo::read_with_size(&data, &mut c, e - s)
                    .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er)),
            );
            assert_eq!(c, *e);
        }
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
