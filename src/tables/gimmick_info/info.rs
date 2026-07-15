//! Tier 1.5 — typed prefix + Decoded|Raw fallback tail.
//!
//! Reader (Tier IDA verified 2026-05-19 vs CrimsonDesert.exe md5
//! 3d614280…): `sub_1410C8D20` — GimmickInfo deserializer (via
//! "GimmickInfo" class block at 0x144AFED38+; size 0x1C92 = 7314 bytes).
//! ~176 field-ops in the reader (110 direct byte-reads + 66 sub-reader
//! calls) ≈ the 179 decoded fields (6 head + GimmickPostBody 139 +
//! intermediate lists). (Cited `sub_1410E6FC0` stale.)
//!
//! ⚠ VERSION NOTE: the 6 head fields (key, string_key, is_blocked,
//! prefab_path, gimmick_group_info, breakable_object_info) decode on the
//! 1.0.4 dump; but `GimmickPostBody` (the 139-field tail) is decoded for
//! the *current* binary and FALLS BACK to `GimmickTail::Raw(blob)` on
//! 1.0.4 data — the 1.0.4 post-body layout diverges (post_body_diag
//! shows CArray-count = ASCII string bytes, i.e. misalignment). So the
//! 18MB byte-exact roundtrip passes via Raw-blob preservation for the
//! tail, NOT by field-decoding the post-body on 1.0.4. Field-level
//! decode of GimmickPostBody is verified against the current binary
//! (reader identity + op-count); byte-roundtrip of the post-body needs
//! a current-version fixture. (Original cite below also stale.)
//!
//! [legacy header, addresses stale] Massive
//! 7205-byte function, 100+ wire reads in the body. Fields 1-18 are
//! typed when the Decoded probe succeeds; the remaining 80+ reads sit in
//! `post_blob`. All typed fields are Option<…> so a mid-sequence decode
//! failure lands cleanly in post_blob without corrupting the roundtrip.
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
//!      CArray<COptional<144-byte item via sub_1410DF770>>)
//!   8. u8 _useInteractionUISocket    (mem a2+64)
//!   9. u8 _useSubPartForInteraction  (mem a2+65)
//!  10. _propertyList                 (sub_141101AB0, CArray<u32>)
//!  11. u32 _gimmickNameHash          (mem a2+88)
//!  12. LocalizableString _gimmickName (mem a2+96)
//!  13. CString _emojiTextureID       (mem a2+128)
//!  14. CString _devMemo              (mem a2+136)
//!  15. sub_141104D20                 (CArray<{CString,CString}>)
//!  16. sub_141102990                 (CArray<{CString}>)
//!  17. sub_1411125E0                 (CArray<COptional<TGPEHD>>)
//!  18. sub_141C7F8B0                 (CArray<GimmickChartParameter>)
//!  19. CArray<COptional<CString>>   (alt-trigger name list, mem a2+208)
//!      Post-blob fields 20-179 are decoded by GimmickPostBody when
//!      F19 decodes cleanly and the probe is within the entry bounds.


// ─────────────────────────────────────────────────────────────────────────
// CANONICAL FIELD CATALOG — pa::GimmickInfo
// ─────────────────────────────────────────────────────────────────────────
//
// Schema source: NattKh/CrimsonDesertModdingTools `pabgb_complete_schema.json`
// (canonical PA names extracted from Korean error strings in CrimsonDesert.exe).
//
// Total canonical fields:  159
// Decoded by dmm-parser:   6
// Missing in this struct:  153
//
// ✅ = present in this struct (round-trips via shape='v3.1')
// ⏳ = in canonical schema but not yet decoded by dmm-parser
//
// ⏳ _blockNavigation (direct_13B, stream=13)
// ⏳ _registerAsPlatformOfSummonee (direct_13B, stream=13)
// ⏳ _checkAllyToBreak (direct_13B, stream=13)
// ⏳ _initialNavigation (direct_13B, stream=13)
// ⏳ _makeNaviVoxelSpecial (direct_13B, stream=13)
// ⏳ _checkAllyToBreakUseGimmickInfo (direct_13B, stream=13)
// ⏳ _generateEffectData (array_or_complex, stream=1)
// ⏳ _isBlockRoadSpawnStageObstacle (direct_13B, stream=13)
// ⏳ _canDisassemble (direct_13B, stream=13)
// ⏳ _gimmickOnTimeGroupDataList
// ⏳ _transmutationMaterialItemList (reader_4B, stream=4)
// ⏳ _transmutationMaterialGimmickList (reader_4B, stream=4)
// ⏳ _timerRandomInterval (direct_u64, stream=8)
// ⏳ _transmutationMaterialItemGroupList
// ⏳ _motionTypeAsPlatform (direct_13B, stream=13)
// ⏳ _movableNavigation (direct_13B, stream=13)
// ⏳ _emojiTextureID
// ⏳ _gimmickName (reader_8B, stream=8)
// ⏳ _gimmickChartParameterList
// ⏳ _devMemo
// ⏳ _triggerVolumeGroupDataList
// ⏳ _gimmickTagList
// ⏳ _elementalReceiverColliderGroupDataList
// ⏳ _triggerCheckTargetDataList
// ✅ _gimmickGroupInfo (reader_4B, stream=4)
// ✅ _prefabPath
// ⏳ _gimmickInteractionOverrideDataList
// ✅ _breakableObjectInfo
// ⏳ _useSubPartForInteraction (direct_13B, stream=13)
// ⏳ _useInteractionUISocket (direct_13B, stream=13)
// ⏳ _gimmickNameHash (direct_u32, stream=4)
// ⏳ _propertyList (reader_4B, stream=4)
// ✅ _key (direct_u32, stream=4)
// ✅ _isBlocked (direct_13B, stream=13)
// ✅ _stringKey
// ⏳ _vehicleInfo (reader_2B, stream=2)
// ⏳ _targetableRange (direct_13B, stream=13)
// ⏳ _hasObstacleUseType (direct_13B, stream=13)
// ⏳ _isUnique (direct_13B, stream=13)
// ⏳ _isHousingGimmick (direct_13B, stream=13)
// ⏳ _isHandCatchable (direct_13B, stream=13)
// ⏳ _isSavePresetTarget (direct_13B, stream=13)
// ⏳ _isPuzzleGimmick (direct_13B, stream=13)
// ⏳ _maxFertilizerAmount (direct_u32, stream=4)
// ⏳ _installOriginGimmickInfo (reader_4B, stream=4)
// ⏳ _propertyConditionStringListForDebug
// ⏳ _fertilizerIntakeAmount (direct_u32, stream=4)
// ⏳ _stickToObjectSocketList
// ⏳ _stickToObjectType (direct_13B, stream=13)
// ⏳ _interactionUIDistanceLv (direct_13B, stream=13)
// ⏳ _pushObjectSocketList
// ⏳ _summonGimmickDataList
// ⏳ _gimmickNodeData (array_or_complex, stream=4)
// ⏳ _summonItemDataList
// ⏳ _summonCharacterDataList
// ⏳ _impulseSurroundingDistance (direct_u32, stream=4)
// ⏳ _summonRandomDataList
// ⏳ _pageGimmickInfo (reader_4B, stream=4)
// ⏳ _inspectDataList
// ⏳ _gimmickAttachTargetDataList
// ⏳ _transformSetList
// ⏳ _eventKeyGuideList
// ⏳ _targetSealPartGimmickInfoList (reader_4B, stream=4)
// ⏳ _bodyMass (direct_u32, stream=4)
// ⏳ _remoteCatchPullInDurationTime (direct_u32, stream=4)
// ⏳ _isLevelGimmickQuickRespawn (direct_13B, stream=13)
// ⏳ _isTwoHandsRemoteCatch (direct_13B, stream=13)
// ⏳ _cogWheelSawToothCount (direct_u32, stream=4)
// ⏳ _constraintSpeedLevel (direct_13B, stream=13)
// ⏳ _dropRollCount (direct_u32, stream=4)
// ⏳ _cogWheelTriggerScale (direct_12B, stream=12)
// ⏳ _dropSetInfoList (reader_4B, stream=4)
// ⏳ _dropOffsetSocketName
// ⏳ _buyableDropItem (reader_1B, stream=1)
// ⏳ _dropInfoDataList
// ⏳ _pushableDirection (direct_13B, stream=13)
// ⏳ _sealCompleteCount (direct_u32, stream=4)
// ⏳ _snapDialData
// ⏳ _pendulumData
// ⏳ _hoveringData
// ⏳ _forceFieldTargetType (direct_13B, stream=13)
// ⏳ _triggerCheckTargetType (direct_13B, stream=13)
// ⏳ _keepClimbPointWhenBroken (direct_13B, stream=13)
// ⏳ _attackImpulseCompleteData
// ⏳ _boardKey (direct_u32, stream=4)
// ⏳ _batteryTotalCapacity (direct_u64, stream=8)
// ⏳ _batteryInitCapacity (direct_u64, stream=8)
// ⏳ _centerOfMass (direct_12B, stream=12)
// ⏳ _collisionBodyData (reader_1B, stream=1)
// ⏳ _physicsContactEventDeltaVelocityThreashold (direct_u32, stream=4)
// ⏳ _physicsBreakingDeltaVelocityThreashold (direct_u32, stream=4)
// ⏳ _growthDataList
// ⏳ _controlMaterialParamValueList
// ⏳ _convertItemInfo (reader_4B, stream=4)
// ⏳ _isInstallable (direct_13B, stream=13)
// ⏳ _allyGroupInfo
// ⏳ _uiMapTextureInfo
// ⏳ _detectCustomRenderIndex (direct_13B, stream=13)
// ⏳ _isTargetable (direct_13B, stream=13)
// ⏳ _saveLevelData (direct_13B, stream=13)
// ⏳ _additionalHeightOnCatched (direct_u32, stream=4)
// ⏳ _knowledgeInfo (reader_4B, stream=4)
// ⏳ _saveOption
// ⏳ _customVolumeGroupDataList
// ⏳ _elementalStatusInitialStatList
// ⏳ _defaultSpawnReasonHash (direct_u32, stream=4)
// ⏳ _defaultSpawnReasonData (reader_8B, stream=8)
// ⏳ _isBuyable (direct_13B, stream=13)
// ⏳ _isWild (direct_13B, stream=13)
// ⏳ _useRemoteCatchFishing (direct_13B, stream=13)
// ⏳ _excludeSequencerBoundary (direct_13B, stream=13)
// ⏳ _forceCursorAimTargetable (direct_13B, stream=13)
// ⏳ _autoSpawnEnviornmentDetailEffect (direct_13B, stream=13)
// ⏳ _gimmickFactionInoMode (direct_13B, stream=13)
// ⏳ _isAttachTargetOfOtherGimmick (direct_13B, stream=13)
// ⏳ _propagateSkillFromParentActor (direct_13B, stream=13)
// ⏳ _isShowInteractionByTrigger (direct_13B, stream=13)
// ⏳ _elementalMaterialInfoList (reader_4B, stream=4)
// ⏳ _respawnTimeSeconds (direct_u64, stream=8)
// ⏳ _applyOffsetByScreenSpaceCasting (direct_13B, stream=13)
// ⏳ _spawnableVisibleOnly (direct_13B, stream=13)
// ⏳ _gamePlayObjectShareData (array_or_complex, stream=4)
// ⏳ _initScale (direct_u32, stream=4)
// ⏳ _shaderMaterialEffectType (direct_13B, stream=13)
// ⏳ _craftToolData
// ⏳ _jamReactionType (direct_13B, stream=13)
// ⏳ _jammedLogoutEffectName
// ⏳ _collectFilter_Dev (direct_13B, stream=13)
// ⏳ _housingSupportPlaneScale (direct_u32, stream=4)
// ⏳ _physicsQualityPreset (direct_13B, stream=13)
// ⏳ _knowledgeExtractType (direct_13B, stream=13)
// ⏳ _equipDockingSpawnDistanceLevel (direct_u32, stream=4)
// ⏳ _spawnDistanceLevel (direct_u32, stream=4)
// ⏳ _useOnDemandCombination (direct_13B, stream=13)
// ⏳ _collisionGroupLayer (direct_u32, stream=4)
// ⏳ _physicsTriggerDataList
// ⏳ _elementalAreaWithMaterial
// ⏳ _miniGameDataList
// ⏳ _trafficBoxDataList (reader_4B, stream=4)
// ⏳ _factionStructure
// ⏳ _housingItemPlacementTypeFlag
// ⏳ _housingGimmickSpecialType (direct_13B, stream=13)
// ⏳ _housingStackableTypeFlag
// ⏳ _buoyancySubmersionRatio (direct_u32, stream=4)
// ⏳ _dialogDataList
// ⏳ _characterStepHeight (direct_u32, stream=4)
// ⏳ _breakDropOffsetDistance (direct_u32, stream=4)
// ⏳ _defaultAliasName
// ⏳ _breakTypeFromParent (direct_13B, stream=13)
// ⏳ _elementalAreaDataList
// ⏳ _weakPointEffectDataList
// ⏳ _isCollectOnlyGimmick (direct_13B, stream=13)
// ⏳ _isAlwaysSave (direct_13B, stream=13)
// ⏳ _useGroupingRemoteCatch (direct_13B, stream=13)
// ⏳ _isBlockSpawnOnAwayFromOriginTransform (direct_13B, stream=13)
// ⏳ _snowRatio (direct_u32, stream=4)
// ⏳ _applyGimmickStateToItem (direct_13B, stream=13)
// ⏳ _massLevel (direct_13B, stream=13)
// ⏳ _sealData (reader_1B, stream=1)

use crate::binary::*;
use crate::binary::variants::gimmick_interaction_override::GimmickInteractionOverrideCArray;
use crate::binary::variants::trigger_gameplay_event_handler_data::TriggerEventHandlerDataElement;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use crate::py_binary_struct;
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use pyo3::types::PyAnyMethods;
use serde_json::{Map, Value};
use std::io::{self, Write};

// ── Re-export GimmickHelperBlock from the TGPEHD variant ─────────────────────
use crate::binary::variants::trigger_gameplay_event_handler_data::GimmickHelperBlock;

// ── Leaf element types for GimmickInfo fields 1-18 ───────────────────────────

// GimmickChartParameter — element reader sub_14F0B2F40 (F18 list). TRUE layout:
// name:CString, tag:u8, value (WIDTH BY TAG: u32 for 0/2/3/4/6/7/8, u16 for 1/5/9,
// none otherwise), tail:u8. Was mis-modeled as fixed {u32,u8,u32,u8} — `field_a`
// was really the CString name, so non-empty names drifted (F18 under-consumed →
// F19 read mid-string → alt_trigger=None → post-body skipped). `value` is stored
// zero-extended in a u32; the wire width is re-derived from `tag` on write.
#[inline]
fn gcp_value_width(tag: u8) -> u8 {
    match tag { 0 | 2 | 3 | 4 | 6 | 7 | 8 => 4, 1 | 5 | 9 => 2, _ => 0 }
}

#[derive(Debug)]
pub struct GimmickChartParameter<'a> {
    pub name: CString<'a>,
    pub tag: u8,
    pub value: u32,
    pub tail: u8,
}

impl<'a> BinaryRead<'a> for GimmickChartParameter<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let name = CString::read_from(data, offset)?;
        let tag = u8::read_from(data, offset)?;
        let value = match gcp_value_width(tag) {
            4 => u32::read_from(data, offset)?,
            2 => u16::read_from(data, offset)? as u32,
            _ => 0,
        };
        let tail = u8::read_from(data, offset)?;
        Ok(Self { name, tag, value, tail })
    }
}

impl<'a> crate::binary::BinaryReadTracked<'a> for GimmickChartParameter<'a> {
    fn read_tracked(data: &'a [u8], offset: &mut usize, path: &mut String,
                    ranges: &mut Vec<FieldRange>) -> io::Result<Self> {
        let start = *offset;
        let v = <Self as BinaryRead>::read_from(data, offset)?;
        ranges.push(FieldRange { path: path.clone(), start, end: *offset, ty: "GimmickChartParameter" });
        Ok(v)
    }
}

impl<'a> BinaryWrite for GimmickChartParameter<'a> {
    fn write_to(&self, w: &mut dyn std::io::Write) -> io::Result<()> {
        self.name.write_to(w)?;
        self.tag.write_to(w)?;
        match gcp_value_width(self.tag) {
            4 => self.value.write_to(w)?,
            2 => (self.value as u16).write_to(w)?,
            _ => {}
        }
        self.tail.write_to(w)?;
        Ok(())
    }
}

impl<'a> ToJsonValue for GimmickChartParameter<'a> {
    fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("name".into(), self.name.to_json_value());
        m.insert("tag".into(), self.tag.to_json_value());
        m.insert("value".into(), self.value.to_json_value());
        m.insert("tail".into(), self.tail.to_json_value());
        Value::Object(m)
    }
}

impl<'a> WriteJsonValue for GimmickChartParameter<'a> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "GimmickChartParameter: expected object"))?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "name")?)?;
        let tag = json_get_field(obj, "tag")?.as_u64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "GimmickChartParameter.tag"))? as u8;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "tag")?)?;
        let value = json_get_field(obj, "value")?.as_u64().unwrap_or(0) as u32;
        match gcp_value_width(tag) {
            4 => w.extend_from_slice(&value.to_le_bytes()),
            2 => w.extend_from_slice(&(value as u16).to_le_bytes()),
            _ => {}
        }
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "tail")?)?;
        Ok(())
    }
}

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

// F19 (alt-trigger) inner element: IDA sub_141AB03B0 reads, when the COptional
// flag is set, a CArray of these 45-byte elements (sub_14108B940 = [u8;12] +
// [u32;4] + [u8;12] = 40, then u32 + u8).
py_binary_struct! {
    pub struct GimmickF19InnerElem {
        pub a: [u8; 12],
        pub b: [u32; 4],
        pub c: [u8; 12],
        pub d: u32,
        pub e: u8,
    }
}


// ── Sub-types for post-blob fields (F20-F179) ─────────────────────────────────

// sub_140F68E80: reads {u8 + u64 + CString} = 32 mem bytes (variable wire).
py_binary_struct! {
    pub struct GimmickBlock32<'a> {
        pub flag: u8,
        pub value: u64,
        pub name: CString<'a>,
    }
}

// sub_1410DD140: composite reader for field 132's 264b element sub-struct.
// Wire: u32(DA48 lookup) + u32(EEE8) + u8 + u8 + u8 + u32 + u32 + CArray<u32>(DA48).
py_binary_struct! {
    pub struct GimmickDD140 {
        pub lookup_a: u32,
        pub lookup_b: u32,
        pub flag_a: u8,
        pub flag_b: u8,
        pub flag_c: u8,
        pub val_a: u32,
        pub val_b: u32,
        pub list: CArray<u32>,
    }
}

// sub_1410DD420: 264b-stride element for field 132 CArrays.
// Wire: 37 field reads (scalars + nested CArrays + CStrings).
py_binary_struct! {
    pub struct GimmickDD420Elem<'a> {
        pub f00: u8,
        pub f01: u32,
        pub f02: u32,
        pub f03: u32,
        pub f04: u32,
        pub f05: u32,
        pub f06: u8,
        pub f07: u32,
        pub f08: u16,
        pub f09: u32,
        pub f10: u8,
        pub f11: u8,
        pub sub_dd140: GimmickDD140,
        pub f12: u8,
        pub f13: u32,
        pub f14: u64,
        pub f15: u32,
        pub f16: u64,
        pub list_a: CArray<u32>,
        pub f17: u32,
        pub list_b: CArray<u32>,
        pub f18: u64,
        pub list_c: CArray<u32>,
        pub list_d: CArray<u16>,
        pub f19: [u8; 12],
        pub f20: u8,
        pub f21: u8,
        pub f22: u8,
        pub f23: u8,
        pub f24: u8,
        pub f25: u32,
        pub f26: u8,
        pub f27: u32,
        pub f28: u32,
        pub block: GimmickBlock32<'a>,
        pub name: CString<'a>,
        pub f29: u32,
    }
}

// F20 inner-inner element: 4× u32 (one "slot").
py_binary_struct! {
    pub struct GimmickF20SubElem {
        pub a: u32, pub b: u32, pub c: u32, pub d: u32,
    }
}

// F20 outer element: u32 + CArray<GimmickF20SubElem> + u8.
py_binary_struct! {
    pub struct GimmickF20Elem {
        pub outer: u32,
        pub inner: CArray<GimmickF20SubElem>,
        pub flag: u8,
    }
}

// F24 element: u16 + u32.
py_binary_struct! {
    pub struct GimmickF24Elem {
        pub lookup: u16,
        pub val: u32,
    }
}

// F34 element: u8+u8+f32+u8(tag)+CString(name)+u8(tag2)+[u8;16].
// `e` was mis-modeled as u32 (it read the CString length 18 as a scalar, then
// `f`+`g` ate 17 name bytes) — breaks any record with a populated F34 material-
// parameter list (shader params e.g. _emissiveIntensity/_emissiveProgressGauge).
py_binary_struct! {
    pub struct GimmickF34Elem<'a> {
        pub a: u8,
        pub b: u8,
        pub c: f32,
        pub d: u8,
        pub e: CString<'a>,
        pub f: u8,
        pub g: [u8; 16],
    }
}

// F35 element: u32×5+u8+u8.
py_binary_struct! {
    pub struct GimmickF35Elem {
        pub a: u32, pub b: u32, pub c: u32, pub d: u32, pub e: u32,
        pub f: u8, pub g: u8,
    }
}

// F46 optional inner: GimmickHelperBlock(40b) + u8+u8+u32+u8+u8+u8.
py_binary_struct! {
    pub struct GimmickF46Data {
        pub block: GimmickHelperBlock,
        pub a: u8,
        pub b: u8,
        pub c: u32,
        pub d: u8,
        pub e: u8,
        pub f: u8,
    }
}

// F75 / F166 / F167 element: u32+u32.
py_binary_struct! {
    pub struct GimmickF75Elem {
        pub a: u32,
        pub b: u32,
    }
}

// F78 inner element: u32+u32.
py_binary_struct! {
    pub struct GimmickF78SubElem {
        pub a: u32,
        pub b: u32,
    }
}

// F78 outer element: u32 + CArray<GimmickF78SubElem>.
py_binary_struct! {
    pub struct GimmickF78Elem {
        pub a: u32,
        pub inner: CArray<GimmickF78SubElem>,
    }
}

// F79 inner element (80-byte mem stride; variable wire): sub_1410E61F0.
// Wire: CArray<CString> + CArray<CString> + CBytes + CBytes
//       + u8 + [u32;3] + u32 + u32 + u8 + u8.
// str0/str1 use CBytes (sub_1410A9B70 raw-byte reader, not UTF-8 guaranteed).
py_binary_struct! {
    pub struct GimmickF79Inner<'a> {
        pub arr0: CArray<CString<'a>>,
        pub arr1: CArray<CString<'a>>,
        pub str0: CBytes<'a>,
        pub str1: CBytes<'a>,
        pub f48:  u8,
        pub f52:  [u32; 3],
        pub f64v: u32,
        pub f68v: u32,
        pub f72:  u8,
        pub f73:  u8,
    }
}

// F79 outer element: sub_141111CD0. Wire: u32 + u8×3 + CArray<GimmickF79Inner> + u8.
py_binary_struct! {
    pub struct GimmickF79Elem<'a> {
        pub f0:    u32,
        pub f4:    u8,
        pub f5:    u8,
        pub f6:    u8,
        pub inner: CArray<GimmickF79Inner<'a>>,
        pub tail:  u8,
    }
}

// F87 sub-element (sub_141109D60 per-element).
// Wire: u64 + u8 + u8 (10 bytes; 16-byte memory stride).
py_binary_struct! {
    pub struct GimmickF87Sub {
        pub f0: u64,
        pub f8: u8,
        pub f9: u8,
    }
}

// F87 inner element (128-byte mem stride): sub_1410F7F20.
// Wire: u32×2 + [u32;3]×2 + CBytes×5 (hash strings via sub_1410A9D40)
//       + CBytes (raw string via sub_1410A9B70) + [u32;4] (sub_141107700)
//       + u32 + u8 + [u32;4] (sub_1410AA0D0) + u8×6 + CArray<GimmickF87Sub>.
py_binary_struct! {
    pub struct GimmickF87Inner<'a> {
        pub f0:    u32,
        pub f1:    u32,
        pub f2:    [u32; 3],
        pub f3:    [u32; 3],
        pub hash0: CBytes<'a>,
        pub hash1: CBytes<'a>,
        pub hash2: CBytes<'a>,
        pub hash3: CBytes<'a>,
        pub hash4: CBytes<'a>,
        pub str0:  CBytes<'a>,
        pub arr0:  [u32; 4],
        pub f20:   u32,
        pub f21:   u8,
        pub arr1:  [u32; 4],
        pub f26:   u8,
        pub f26b:  u8,
        pub f26c:  u8,
        pub f26d:  u8,
        pub f27:   u8,
        pub f27b:  u8,
        pub subs:  CArray<GimmickF87Sub>,
    }
}

// F87 outer element: sub_141105260. Wire: GimmickF87Inner + u32 tail.
py_binary_struct! {
    pub struct GimmickF87Elem<'a> {
        pub inner: GimmickF87Inner<'a>,
        pub tail:  u32,
    }
}

// F88 sub-element for the initial CArray (sub_1411003E0 + sub_1410FF220).
// Each reads a u16 from wire and does an in-memory table lookup.
// Wire: u16 + u16.
py_binary_struct! {
    pub struct GimmickF88Sub1 {
        pub id0: u16,
        pub id1: u16,
    }
}

// F88 optional-field content (sub_141103B30 / sub_141CEA810).
// Wire: u8×3 (after the COptional flag byte).
py_binary_struct! {
    pub struct GimmickF88COptContent {
        pub b0: u8,
        pub b1: u8,
        pub b2: u8,
    }
}

// F88 sub-struct for sub_1410F6ED0 (memory offset 144 in GimmickF88Inner).
// Wire: u32 + CBytes (hash string) + u8 + u16 + u64.
py_binary_struct! {
    pub struct GimmickF88Sub3<'a> {
        pub f0:   u32,
        pub hash: CBytes<'a>,
        pub f8:   u8,
        pub f10:  u16,
        pub f16:  u64,
    }
}

// F88 inner element (232-byte mem stride): sub_1410F7440.
py_binary_struct! {
    pub struct GimmickF88Inner<'a> {
        // Initial CArray (sub_1411003E0 + sub_1410FF220 per element)
        pub arr0:   CArray<GimmickF88Sub1>,
        // COptional<{u8+u8+u8}> (sub_141103B30)
        pub opt0:   COptional<GimmickF88COptContent>,
        // scalar fields
        pub f24:    u32,
        pub f28:    u16,
        pub f32v:   [u32; 3],
        pub f44:    u32,
        pub f48:    u8,
        // raw string (sub_1410A9B70)
        pub str0:   CBytes<'a>,
        pub f64:    u8,
        pub f65:    u8,
        pub f66:    u8,
        pub f72:    u64,
        // sub_141107700
        pub arr1:   [u32; 4],
        pub f96:    u8,
        pub f97:    u8,
        // hash strings (sub_1410A9D40 wire)
        pub hash0:  CBytes<'a>,
        pub hash1:  CBytes<'a>,
        pub f108:   u32,
        // sub_141107700
        pub arr2:   [u32; 4],
        pub f128:   u32,
        pub f132:   u8,
        pub f136:   u32,
        // sub_1410F6ED0 sub-struct
        pub sub3:   GimmickF88Sub3<'a>,
        // second raw string (sub_1410A9B70)
        pub str1:   CBytes<'a>,
        pub f176:   u64,
        pub f184:   u8,
        pub f188:   u32,
        pub f192:   u32,
        pub f196:   u8,
        pub f197:   u8,
        pub f198:   u8,
        pub f199:   u8,
        pub f200:   u8,
        pub f204:   u64,
        // sub_141BD4120 = plain u32
        pub f212:   u32,
        pub f216:   u32,
        pub f220:   u16,
        pub f222:   u8,
        pub f223:   u8,
        pub f224:   u8,
        pub f225:   u8,
    }
}

// F88 outer element: sub_141105390. Wire: GimmickF88Inner + u32 tail.
py_binary_struct! {
    pub struct GimmickF88Elem<'a> {
        pub inner: GimmickF88Inner<'a>,
        pub tail:  u32,
    }
}

// F81 element: u32×4 + CArray<u32> + u32.
py_binary_struct! {
    pub struct GimmickF81Elem {
        // True element (IDA reader sub_1410C7630): 3 u32s, then a nested CArray<u32>
        // (sub_1410E2990 = the property_list reader), then a u32 (sub_1410E19E0,
        // enum-mapped). Previously had a spurious 4th u32 `d` + a flattened `inner:u32`
        // band-aid, which mis-aligned the inner count for records with non-empty f81.
        pub a: u32, pub b: u32, pub c: u32,
        pub inner: CArray<u32>,
        pub e: u32,
    }
}

// F89 element — a PER-ELEMENT VARIANT keyed on the first field `a`:
//   • a == 0  → "Common" 2-CString form (Common_Socket_01.. multi-element, 91-byte
//     stride): name1 + pre2[23] + name2:CString + post2[13].
//   • a != 0  → "Fx" scalar form (FX_01_Socket single-element, a = a real id like
//     1000799): name1 + e[u32;4] + f + g + h + i + j + list_id + k + l + m + m2 (37B,
//     no name2). Verified: a==0 for 84 elements, a!=0 for exactly the 3 FX records.
// Manual impl because the discriminator `a` is read before the variant body.
#[derive(Debug)]
pub enum GimmickF89Body<'a> {
    Common { pre2: [u8; 23], name2: CString<'a>, post2: [u8; 13] },
    Fx { e: [u32; 4], f: u32, g: u8, h: u8, i: u8, j: u32, list_id: u32, k: u16, l: u16, m: u8, m2: u8 },
}

#[derive(Debug)]
pub struct GimmickF89Elem<'a> {
    pub a: u32,
    pub b: u16,
    pub c: [u32; 3],
    pub d: [u32; 3],
    pub name: CString<'a>,
    pub body: GimmickF89Body<'a>,
}

impl<'a> BinaryRead<'a> for GimmickF89Elem<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let a = u32::read_from(data, offset)?;
        let b = u16::read_from(data, offset)?;
        let c = <[u32; 3]>::read_from(data, offset)?;
        let d = <[u32; 3]>::read_from(data, offset)?;
        let name = CString::read_from(data, offset)?;
        let body = if a == 0 {
            GimmickF89Body::Common {
                pre2: <[u8; 23]>::read_from(data, offset)?,
                name2: CString::read_from(data, offset)?,
                post2: <[u8; 13]>::read_from(data, offset)?,
            }
        } else {
            GimmickF89Body::Fx {
                e: <[u32; 4]>::read_from(data, offset)?,
                f: u32::read_from(data, offset)?,
                g: u8::read_from(data, offset)?,
                h: u8::read_from(data, offset)?,
                i: u8::read_from(data, offset)?,
                j: u32::read_from(data, offset)?,
                list_id: u32::read_from(data, offset)?,
                k: u16::read_from(data, offset)?,
                l: u16::read_from(data, offset)?,
                m: u8::read_from(data, offset)?,
                m2: u8::read_from(data, offset)?,
            }
        };
        Ok(Self { a, b, c, d, name, body })
    }
}

impl<'a> crate::binary::BinaryReadTracked<'a> for GimmickF89Elem<'a> {
    fn read_tracked(data: &'a [u8], offset: &mut usize, path: &mut String,
                    ranges: &mut Vec<FieldRange>) -> io::Result<Self> {
        let start = *offset;
        let v = <Self as BinaryRead>::read_from(data, offset)?;
        ranges.push(FieldRange { path: path.clone(), start, end: *offset, ty: "GimmickF89Elem" });
        Ok(v)
    }
}

impl<'a> BinaryWrite for GimmickF89Elem<'a> {
    fn write_to(&self, w: &mut dyn std::io::Write) -> io::Result<()> {
        self.a.write_to(w)?;
        self.b.write_to(w)?;
        self.c.write_to(w)?;
        self.d.write_to(w)?;
        self.name.write_to(w)?;
        match &self.body {
            GimmickF89Body::Common { pre2, name2, post2 } => {
                pre2.write_to(w)?; name2.write_to(w)?; post2.write_to(w)?;
            }
            GimmickF89Body::Fx { e, f, g, h, i, j, list_id, k, l, m, m2 } => {
                e.write_to(w)?; f.write_to(w)?; g.write_to(w)?; h.write_to(w)?; i.write_to(w)?;
                j.write_to(w)?; list_id.write_to(w)?; k.write_to(w)?; l.write_to(w)?;
                m.write_to(w)?; m2.write_to(w)?;
            }
        }
        Ok(())
    }
}

impl<'a> ToJsonValue for GimmickF89Elem<'a> {
    fn to_json_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("a".into(), self.a.to_json_value());
        m.insert("b".into(), self.b.to_json_value());
        m.insert("c".into(), self.c.to_json_value());
        m.insert("d".into(), self.d.to_json_value());
        m.insert("name".into(), self.name.to_json_value());
        match &self.body {
            GimmickF89Body::Common { pre2, name2, post2 } => {
                m.insert("pre2".into(), pre2.to_json_value());
                m.insert("name2".into(), name2.to_json_value());
                m.insert("post2".into(), post2.to_json_value());
            }
            GimmickF89Body::Fx { e, f, g, h, i, j, list_id, k, l, m: mm, m2 } => {
                m.insert("e".into(), e.to_json_value());
                m.insert("f".into(), f.to_json_value());
                m.insert("g".into(), g.to_json_value());
                m.insert("h".into(), h.to_json_value());
                m.insert("i".into(), i.to_json_value());
                m.insert("j".into(), j.to_json_value());
                m.insert("list_id".into(), list_id.to_json_value());
                m.insert("k".into(), k.to_json_value());
                m.insert("l".into(), l.to_json_value());
                m.insert("m".into(), mm.to_json_value());
                m.insert("m2".into(), m2.to_json_value());
            }
        }
        Value::Object(m)
    }
}

impl<'a> WriteJsonValue for GimmickF89Elem<'a> {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "GimmickF89Elem: expected object"))?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "a")?)?;
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "b")?)?;
        <[u32; 3] as WriteJsonValue>::write_from_json(w, json_get_field(obj, "c")?)?;
        <[u32; 3] as WriteJsonValue>::write_from_json(w, json_get_field(obj, "d")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "name")?)?;
        if obj.get("pre2").is_some() {
            <[u8; 23] as WriteJsonValue>::write_from_json(w, json_get_field(obj, "pre2")?)?;
            <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "name2")?)?;
            <[u8; 13] as WriteJsonValue>::write_from_json(w, json_get_field(obj, "post2")?)?;
        } else {
            <[u32; 4] as WriteJsonValue>::write_from_json(w, json_get_field(obj, "e")?)?;
            <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "f")?)?;
            <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "g")?)?;
            <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "h")?)?;
            <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "i")?)?;
            <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "j")?)?;
            <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "list_id")?)?;
            <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "k")?)?;
            <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "l")?)?;
            <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "m")?)?;
            <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "m2")?)?;
        }
        Ok(())
    }
}

impl<'a> crate::python_traits::ToPyValue for GimmickF89Elem<'a> {
    fn to_py_value(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        use crate::python_traits::ToPyValue as _;
        use pyo3::types::PyDictMethods;
        let d = pyo3::types::PyDict::new(py);
        d.set_item("a", self.a.to_py_value(py)?)?;
        d.set_item("b", self.b.to_py_value(py)?)?;
        d.set_item("c", self.c.to_py_value(py)?)?;
        d.set_item("d", self.d.to_py_value(py)?)?;
        d.set_item("name", self.name.to_py_value(py)?)?;
        match &self.body {
            GimmickF89Body::Common { pre2, name2, post2 } => {
                d.set_item("pre2", pre2.to_py_value(py)?)?;
                d.set_item("name2", name2.to_py_value(py)?)?;
                d.set_item("post2", post2.to_py_value(py)?)?;
            }
            GimmickF89Body::Fx { e, f, g, h, i, j, list_id, k, l, m: mm, m2 } => {
                d.set_item("e", e.to_py_value(py)?)?;
                d.set_item("f", f.to_py_value(py)?)?;
                d.set_item("g", g.to_py_value(py)?)?;
                d.set_item("h", h.to_py_value(py)?)?;
                d.set_item("i", i.to_py_value(py)?)?;
                d.set_item("j", j.to_py_value(py)?)?;
                d.set_item("list_id", list_id.to_py_value(py)?)?;
                d.set_item("k", k.to_py_value(py)?)?;
                d.set_item("l", l.to_py_value(py)?)?;
                d.set_item("m", mm.to_py_value(py)?)?;
                d.set_item("m2", m2.to_py_value(py)?)?;
            }
        }
        Ok(d.into_any().unbind())
    }
}

impl<'a> crate::python_traits::WritePyValue for GimmickF89Elem<'a> {
    fn write_from_py(w: &mut Vec<u8>, obj: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        use crate::python_traits::{WritePyValue, get_field};
        use pyo3::types::PyDictMethods;
        let d = obj.cast::<pyo3::types::PyDict>()?;
        <u32 as WritePyValue>::write_from_py(w, &get_field(&d, "a")?)?;
        <u16 as WritePyValue>::write_from_py(w, &get_field(&d, "b")?)?;
        <[u32; 3] as WritePyValue>::write_from_py(w, &get_field(&d, "c")?)?;
        <[u32; 3] as WritePyValue>::write_from_py(w, &get_field(&d, "d")?)?;
        <CString as WritePyValue>::write_from_py(w, &get_field(&d, "name")?)?;
        if d.contains("pre2")? {
            <[u8; 23] as WritePyValue>::write_from_py(w, &get_field(&d, "pre2")?)?;
            <CString as WritePyValue>::write_from_py(w, &get_field(&d, "name2")?)?;
            <[u8; 13] as WritePyValue>::write_from_py(w, &get_field(&d, "post2")?)?;
        } else {
            <[u32; 4] as WritePyValue>::write_from_py(w, &get_field(&d, "e")?)?;
            <u32 as WritePyValue>::write_from_py(w, &get_field(&d, "f")?)?;
            <u8 as WritePyValue>::write_from_py(w, &get_field(&d, "g")?)?;
            <u8 as WritePyValue>::write_from_py(w, &get_field(&d, "h")?)?;
            <u8 as WritePyValue>::write_from_py(w, &get_field(&d, "i")?)?;
            <u32 as WritePyValue>::write_from_py(w, &get_field(&d, "j")?)?;
            <u32 as WritePyValue>::write_from_py(w, &get_field(&d, "list_id")?)?;
            <u16 as WritePyValue>::write_from_py(w, &get_field(&d, "k")?)?;
            <u16 as WritePyValue>::write_from_py(w, &get_field(&d, "l")?)?;
            <u8 as WritePyValue>::write_from_py(w, &get_field(&d, "m")?)?;
            <u8 as WritePyValue>::write_from_py(w, &get_field(&d, "m2")?)?;
        }
        Ok(())
    }
}

// F90 sub-element: 19B lead + CString name ("SummonSocket_NN") + u32 trail.
py_binary_struct! {
    pub struct GimmickF90SubElem<'a> {
        pub a: u16, pub b: u16, pub c: u16,
        pub d: u64,
        pub e: u8,
        pub f: u32,
        pub name: CString<'a>,
        pub trail: u32,
    }
}

// F90 element: CString + CArray<GimmickF90SubElem> + u64+u8+u8+u32+u16.
py_binary_struct! {
    pub struct GimmickF90Elem<'a> {
        pub name: CString<'a>,
        pub inner: CArray<GimmickF90SubElem<'a>>,
        pub a: u64,
        pub b: u8,
        pub c: u8,
        pub d: u32,
        pub e: u32,
    }
}

// F92 element.
py_binary_struct! {
    pub struct GimmickF92Elem<'a> {
        // pre-name is 16B (a..f); g..k were mis-placed BEFORE name (made pre-name
        // 32B so `name` read its length from mid-data) — they belong AFTER name.
        pub a: u16, pub b: u16, pub c: u16,
        pub d: u32, pub e: u32, pub f: u16,
        pub name: CString<'a>,
        pub g: u32,
        pub h: u8, pub i: u16, pub j: u8,
        pub k: u64,
        pub l: u32, pub m: u8,
        // After m: a u8, then a SECOND CString (name2, e.g. event/dialog name like
        // "textdialog_gimmick_drop_..."), then a 16-byte tail. Was modeled as scalars
        // n,o,p,q,r,s,t,u_val (21B); x(1)+name2-empty(4)+tail(16)=21 ⇒ byte-equivalent
        // when name2 is empty, but reads the full name when populated.
        pub x: u8,
        pub name2: CString<'a>,
        pub tail: [u8; 28],
    }
}

// F97 element: 260-byte fixed buffer (Windows MAX_PATH string).
py_binary_struct! {
    pub struct GimmickF97Elem {
        pub data: [u8; 260],
    }
}

// F117 sub-element: u32(→u16 lookup) + [u8;8] + u32 + u32(→u16 lookup).
// Wire = 20 bytes; memory stride = 24 bytes (alignment).
py_binary_struct! {
    pub struct GimmickF117SubElem {
        pub lookup_a: u32,
        pub block_8b: [u8; 8],
        pub val_a: u32,
        pub lookup_b: u32,
    }
}

// F117 optional data: CArray<GimmickF117SubElem> + GimmickBlock32 + u32(→u16).
py_binary_struct! {
    pub struct GimmickF117Data<'a> {
        pub list: CArray<GimmickF117SubElem>,
        pub block: GimmickBlock32<'a>,
        pub val: u32,
    }
}

// F119 sub-sub-element: u16 + GimmickBlock32.
py_binary_struct! {
    pub struct GimmickF119SubSubElem<'a> {
        pub val: u16,
        pub block: GimmickBlock32<'a>,
    }
}

// F119 element: CArray<GimmickF119SubSubElem> + u8.
py_binary_struct! {
    pub struct GimmickF119Elem<'a> {
        pub inner: CArray<GimmickF119SubSubElem<'a>>,
        pub flag: u8,
    }
}

// F125 element: u32+u32+[u8;12]+[u8;12].
py_binary_struct! {
    pub struct GimmickF125Elem<'a> {
        // `b` was u32 but is really a CString (socket name e.g. "B_Canon_Acc_00")
        // per IDA reader sub_1410E3D90 (sub_14108B4D0=CString). Empty name == u32 0
        // so empty records stay byte-identical; non-empty names drifted before.
        pub a: u32,
        pub name: CString<'a>,
        pub c: [u8; 12],
        pub d: [u8; 12],
    }
}

// 10 consecutive u32 values (used in F126/F127/F168).
py_binary_struct! {
    pub struct U32x10 {
        pub v0: u32, pub v1: u32, pub v2: u32, pub v3: u32, pub v4: u32,
        pub v5: u32, pub v6: u32, pub v7: u32, pub v8: u32, pub v9: u32,
    }
}

// F126/F127 element: u8+u32+u8+u8+U32x10+u64+u32+u8×5+u32+u32+u8+u32+u32.
py_binary_struct! {
    pub struct GimmickF126Elem {
        pub a: u8,
        pub b: u32,
        pub c: u8,
        pub d: u8,
        pub ten_vals: U32x10,
        pub e: u64,
        pub f: u32,
        pub g0: u8, pub g1: u8, pub g2: u8, pub g3: u8, pub g4: u8,
        pub h: u32,
        pub i: u32,
        pub j: u8,
        pub k: u32,
        pub l: u32,
    }
}

// F128 element: CString + u32 + u32 + u32.
py_binary_struct! {
    pub struct GimmickF128Elem<'a> {
        pub name: CString<'a>,
        pub a: u32,
        pub b: u32,
        pub c: u32,
    }
}

// F131b element (IDA sub_1410F3800, CArray; element via sub_1410C6DF0 + tail).
//   sub_1410C6DF0: CString(sub_14108B300) + u32(sub_1410E2DC0, 4 wire→u16 RAM) +
//                  u32(sub_1410E7190, 4 wire→u16 RAM) + u32(@+12)
//   then: u8 flag · u32 hash(sub_14108B4D0) · [u8;12]
py_binary_struct! {
    pub struct GimmickF131bElem<'a> {
        pub name: CString<'a>,
        pub hash_a: u32,
        pub hash_b: u32,
        pub v12: u32,
        pub flag: u8,
        pub hash_c: u32,
        pub tail: [u8; 12],
    }
}

// F129 element: u32+u32+[u32;3]+[u32;4]+[u32;3].
py_binary_struct! {
    pub struct GimmickF129Elem {
        pub a: u32, pub b: u32,
        pub c: [u32; 3],
        pub d: [u32; 4],
        pub e: [u32; 3],
    }
}

// F130 sub0 element (sub_141100E90): f32+[f32;2]+[f32;2]+[f32;2] = 28 bytes wire.
py_binary_struct! {
    pub struct GimmickF130Sub0Elem {
        // arr0/arr1 element for the REAL f130 body (IDA reader sub_1410E3510):
        // f32 + 3×8 bytes = 28 wire bytes. (sub_1410C0980's 14-byte element was
        // a RED HERRING — it belonged to the wrong f130 reader sub_1410C0A90.)
        pub v:  f32,
        pub a:  [f32; 2],
        pub b:  [f32; 2],
        pub c:  [f32; 2],
    }
}

// F130 sub1 element (sub_1410F27B0):
// COptional<GimmickF88COptContent> + u64 + u16 + u16 + u16 + COptional<u64>.
py_binary_struct! {
    pub struct GimmickF130Sub1Elem {
        pub opt0: COptional<GimmickF88COptContent>,
        pub t:    u64,
        pub a:    u16,
        pub b:    u16,
        pub c:    u16,
        pub opt1: COptional<u64>,
    }
}

// F130 sub2 element (sub_1410F2A30):
// COptional<GimmickF88COptContent> + u64 + u32 + u32.
py_binary_struct! {
    pub struct GimmickF130Sub2Elem {
        pub opt0: COptional<GimmickF88COptContent>,
        pub t:    u64,
        pub a:    u32,
        pub b:    u32,
    }
}

// F130 sub3 element (sub_1410F2B50):
// COptional<GimmickF88COptContent> + u64 + u32 + u32.
py_binary_struct! {
    pub struct GimmickF130Sub3Elem {
        pub opt0: COptional<GimmickF88COptContent>,
        pub t:    u64,
        pub a:    u32,
        pub b:    u32,
    }
}

// F130 optional body (sub_1410F2F90):
// COptional<GimmickF88COptContent> + CArray<sub1> + CArray<sub2> + CArray<sub3>.
py_binary_struct! {
    pub struct GimmickF130Sub0Body {
        pub opt0: COptional<GimmickF88COptContent>,
        pub arr1: CArray<GimmickF130Sub1Elem>,
        pub arr2: CArray<GimmickF130Sub2Elem>,
        pub arr3: CArray<GimmickF130Sub3Elem>,
    }
}

// F130 opt2 content (IDA reader sub_141E21DC0): u64 + u64 + u64 + u32 = 28 bytes.
py_binary_struct! {
    pub struct GimmickF130List2 {
        pub a: u64,
        pub b: u64,
        pub c: u64,
        pub d: u32,
    }
}

// F130 body (IDA reader sub_1410C7A90, the COptional content of the element):
// arr0/arr1 (CArray<sub0>, sub_1410E3510), u32, COptional<List1=Sub0Body>
// (sub_1410D5410), u32, COptional<List2> (sub_141E21DC0), u32-enum-lookup tail.
py_binary_struct! {
    pub struct GimmickF130Body {
        pub arr0: CArray<GimmickF130Sub0Elem>,
        pub arr1: CArray<GimmickF130Sub0Elem>,
        pub f40:  u32,
        pub opt1: COptional<GimmickF130Sub0Body>,
        pub f56:  u32,
        pub opt2: COptional<GimmickF130List2>,
        pub tail: u32,
    }
}

// F130 outer element (IDA reader sub_1410E7F50): u32 + COptional<GimmickF130Body>
// (sub_14110BF20 = presence u8 + sub_1410C7A90). ITER113's {a,b,c,arr,d,e,f,g} was
// RE'd from the WRONG reader (sub_1410C0A90); the true element is this.
py_binary_struct! {
    pub struct GimmickF130Elem {
        pub a:    u32,
        pub body: COptional<GimmickF130Body>,
    }
}

// F168/F169 optional inner: u32+u32+U32x10.
py_binary_struct! {
    pub struct GimmickF168Inner {
        pub a: u32, pub b: u32,
        pub vals: U32x10,
    }
}

// F170 sub-element: u64+u32.
py_binary_struct! {
    pub struct GimmickF170Elem {
        pub a: u64,
        pub b: u32,
    }
}

// F132 outer structure (IDA sub_1410C8960):
// LocalizableString×2 + u32(sub_14108B4D0) + u32(sub_1410E2D50 lookup, 4 wire→
// u16 RAM) + (CArray<u32>+CArray<GimmickDD420Elem 264b>)×2.
// `val` was mis-sized as u16 (2 wire) — the engine reads 4 wire bytes here.
py_binary_struct! {
    pub struct GimmickF132<'a> {
        pub block_a: GimmickBlock32<'a>,
        pub block_b: GimmickBlock32<'a>,
        // was `hash: u32` — actually a CString name (e.g. "Armory"): 0 for 12892
        // records (empty), small name-length for ~25, never a real hash.
        pub name: CString<'a>,
        pub val: u32,
        pub list_a_u32: CArray<u32>,
        pub list_a_264b: CArray<GimmickDD420Elem<'a>>,
        pub list_b_u32: CArray<u32>,
        pub list_b_264b: CArray<GimmickDD420Elem<'a>>,
    }
}

// ── F76 / F77: 128-byte tagged optional struct ────────────────────────────────

/// Inner 128-byte struct populated by `sub_141600210` (via `sub_141D03AA0`
/// COptional wrapper; CArray loop via `sub_141112050`).
/// Wire order (diverges from struct mem-layout):
///   u64(→+120) · u8 type_tag(→+112) · u32(→+4) · u32(→+6) · u32(→+8) ·
///   u32(→+12)  · u64(→+16) · u32(→+24) · u64(→+32) · u64(→+40) ·
///   u64(→+48)  · u16(→+56) · variant(→+64):
///     type ∈ {0,1,2,3,4,9}: u32 hash (4 bytes)
///     type 0xB            : nothing (0 bytes)
///     other               : unimplemented (parse fails → falls back to post_blob)
#[derive(Debug)]
pub struct GimmickF76Inner {
    pub f120:     u64,
    pub type_tag: u8,
    pub f4:       u32,
    pub f6:       u32,
    pub f8:       u32,
    pub f12:      u32,
    pub f16:      u64,
    pub f24:      u32,
    pub f32_val:  u64,
    pub f40:      u64,
    pub f48:      u64,
    pub f56:      u16,
    /// Some(hash u32) for types 0-3 and 9; None for type 0xB.
    pub f64_hash: Option<u32>,
}

impl<'a> BinaryRead<'a> for GimmickF76Inner {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let f120     = u64::read_from(data, offset)?;
        let type_tag = u8::read_from(data, offset)?;
        let f4       = u32::read_from(data, offset)?;
        let f6       = u32::read_from(data, offset)?;
        let f8       = u32::read_from(data, offset)?;
        let f12      = u32::read_from(data, offset)?;
        let f16      = u64::read_from(data, offset)?;
        let f24      = u32::read_from(data, offset)?;
        let f32_val  = u64::read_from(data, offset)?;
        let f40      = u64::read_from(data, offset)?;
        let f48      = u64::read_from(data, offset)?;
        let f56      = u16::read_from(data, offset)?;
        let f64_hash = match type_tag {
            0..=4 | 9 => Some(u32::read_from(data, offset)?),
            0xB       => None,
            t => return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("GimmickF76Inner: unknown type_tag=0x{:02X}", t))),
        };
        Ok(GimmickF76Inner { f120, type_tag, f4, f6, f8, f12, f16, f24, f32_val, f40, f48, f56, f64_hash })
    }
}

impl BinaryWrite for GimmickF76Inner {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.f120.write_to(w)?;
        self.type_tag.write_to(w)?;
        self.f4.write_to(w)?;
        self.f6.write_to(w)?;
        self.f8.write_to(w)?;
        self.f12.write_to(w)?;
        self.f16.write_to(w)?;
        self.f24.write_to(w)?;
        self.f32_val.write_to(w)?;
        self.f40.write_to(w)?;
        self.f48.write_to(w)?;
        self.f56.write_to(w)?;
        if let Some(h) = self.f64_hash { h.write_to(w)?; }
        Ok(())
    }
}

impl BinaryReadTracked<'_> for GimmickF76Inner {
    fn read_tracked(data: &[u8], offset: &mut usize,
        _path: &mut String, _ranges: &mut Vec<FieldRange>) -> io::Result<Self> {
        Self::read_from(data, offset)
    }
}

impl ToJsonValue for GimmickF76Inner {
    fn to_json_value(&self) -> Value {
        let mut m = serde_json::Map::new();
        m.insert("f120".into(),    self.f120.to_json_value());
        m.insert("type_tag".into(), self.type_tag.to_json_value());
        m.insert("f4".into(),      self.f4.to_json_value());
        m.insert("f6".into(),      self.f6.to_json_value());
        m.insert("f8".into(),      self.f8.to_json_value());
        m.insert("f12".into(),     self.f12.to_json_value());
        m.insert("f16".into(),     self.f16.to_json_value());
        m.insert("f24".into(),     self.f24.to_json_value());
        m.insert("f32_val".into(), self.f32_val.to_json_value());
        m.insert("f40".into(),     self.f40.to_json_value());
        m.insert("f48".into(),     self.f48.to_json_value());
        m.insert("f56".into(),     self.f56.to_json_value());
        m.insert("f64_hash".into(), match self.f64_hash {
            Some(h) => h.to_json_value(),
            None    => Value::Null,
        });
        Value::Object(m)
    }
}

impl WriteJsonValue for GimmickF76Inner {
    fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "GimmickF76Inner: expected JSON object"))?;
        let type_tag_v = json_get_field(obj, "type_tag")?;
        let type_tag = type_tag_v.as_u64()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                "GimmickF76Inner.type_tag: expected integer"))? as u8;
        u64::write_from_json(w, json_get_field(obj, "f120")?)?;
        u8::write_from_json(w,  type_tag_v)?;
        u32::write_from_json(w, json_get_field(obj, "f4")?)?;
        u32::write_from_json(w, json_get_field(obj, "f6")?)?;
        u32::write_from_json(w, json_get_field(obj, "f8")?)?;
        u32::write_from_json(w, json_get_field(obj, "f12")?)?;
        u64::write_from_json(w, json_get_field(obj, "f16")?)?;
        u32::write_from_json(w, json_get_field(obj, "f24")?)?;
        u64::write_from_json(w, json_get_field(obj, "f32_val")?)?;
        u64::write_from_json(w, json_get_field(obj, "f40")?)?;
        u64::write_from_json(w, json_get_field(obj, "f48")?)?;
        u16::write_from_json(w, json_get_field(obj, "f56")?)?;
        let hash_v = json_get_field(obj, "f64_hash")?;
        match type_tag {
            0..=4 | 9 => u32::write_from_json(w, hash_v)?,
            0xB       => {},
            t => return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("GimmickF76Inner: unknown type_tag=0x{:02X}", t))),
        }
        Ok(())
    }
}

impl crate::python_traits::ToPyValue for GimmickF76Inner {
    fn to_py_value(&self, py: pyo3::Python<'_>) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
        use pyo3::types::PyDict;
        use pyo3::IntoPyObjectExt;
        let d = PyDict::new(py);
        d.set_item("f120",     self.f120.to_py_value(py)?)?;
        d.set_item("type_tag", self.type_tag.to_py_value(py)?)?;
        d.set_item("f4",       self.f4.to_py_value(py)?)?;
        d.set_item("f6",       self.f6.to_py_value(py)?)?;
        d.set_item("f8",       self.f8.to_py_value(py)?)?;
        d.set_item("f12",      self.f12.to_py_value(py)?)?;
        d.set_item("f16",      self.f16.to_py_value(py)?)?;
        d.set_item("f24",      self.f24.to_py_value(py)?)?;
        d.set_item("f32_val",  self.f32_val.to_py_value(py)?)?;
        d.set_item("f40",      self.f40.to_py_value(py)?)?;
        d.set_item("f48",      self.f48.to_py_value(py)?)?;
        d.set_item("f56",      self.f56.to_py_value(py)?)?;
        d.set_item("f64_hash", match self.f64_hash {
            Some(h) => h.to_py_value(py)?,
            None    => py.None().into_py_any(py)?,
        })?;
        d.into_py_any(py)
    }
}

impl crate::python_traits::WritePyValue for GimmickF76Inner {
    fn write_from_py(w: &mut Vec<u8>, obj: &pyo3::Bound<'_, pyo3::PyAny>) -> pyo3::PyResult<()> {
        use pyo3::types::PyDict;
        use crate::python_traits::get_field as py_get_field;
        let d = obj.cast::<PyDict>()?;
        let type_tag: u8 = py_get_field(d, "type_tag")?.extract()?;
        u64::write_from_py(w, &py_get_field(d, "f120")?)?;
        u8::write_from_py(w,  &py_get_field(d, "type_tag")?)?;
        u32::write_from_py(w, &py_get_field(d, "f4")?)?;
        u32::write_from_py(w, &py_get_field(d, "f6")?)?;
        u32::write_from_py(w, &py_get_field(d, "f8")?)?;
        u32::write_from_py(w, &py_get_field(d, "f12")?)?;
        u64::write_from_py(w, &py_get_field(d, "f16")?)?;
        u32::write_from_py(w, &py_get_field(d, "f24")?)?;
        u64::write_from_py(w, &py_get_field(d, "f32_val")?)?;
        u64::write_from_py(w, &py_get_field(d, "f40")?)?;
        u64::write_from_py(w, &py_get_field(d, "f48")?)?;
        u16::write_from_py(w, &py_get_field(d, "f56")?)?;
        let hash_field = py_get_field(d, "f64_hash")?;
        match type_tag {
            0..=4 | 9 => u32::write_from_py(w, &hash_field)?,
            0xB       => {},
            t => return Err(pyo3::exceptions::PyValueError::new_err(
                format!("GimmickF76Inner: unknown type_tag=0x{:02X}", t))),
        }
        Ok(())
    }
}

// One element of F76's CArray (`sub_141112050`).
// Wire: COptional<GimmickF76Inner> (1-byte flag + optional inner) + u32.
py_binary_struct! {
    pub struct GimmickF76Elem {
        pub inner: COptional<GimmickF76Inner>,
        pub tag:   u32,
    }
}

// ── GimmickPostBody: fields F20-F179 ─────────────────────────────────────────

py_binary_struct! {
    /// All post-blob typed fields F20 through F179.
    /// Read via safe probe; on any failure the bytes stay in `post_blob`.
    pub struct GimmickPostBody<'a> {
        // F20: CArray<{u32+CArray<{u32,u32,u32,u32}>+u8}>
        pub f20: CArray<GimmickF20Elem>,
        // F21: u8
        pub f21: u8,
        // F22: CArray<u32> (u32→u16 DA38 lookup per element; raw u32 on wire)
        pub f22: CArray<u32>,
        // F23: CArray<u32> (DA00 lookup)
        pub f23: CArray<u32>,
        // F24: CArray<{u16+u32}> (u16→u16 DA20, u32→u16 DA00)
        pub f24: CArray<GimmickF24Elem>,
        // F25: u64
        pub f25: u64,
        // F26-F33: 8 u8 values (IDA: 8 individual reads a2+304..311 before
        // the {u32,u8,u8} block — was mis-sized as [u8;7], off by one).
        pub f26_32: [u8; 8],
        // F33: u32+u8+u8
        pub f33_a: u32,
        pub f33_b: u8,
        pub f33_c: u8,
        // F34: CArray<{u8+u8+f32+u8+u32+u8+[u8;16]}>
        pub f34: CArray<GimmickF34Elem<'a>>,
        // F35: CArray<{u32×5+u8+u8}>
        pub f35: CArray<GimmickF35Elem>,
        // F36: u8
        pub f36: u8,
        // F37: u32 (DA00 lookup)
        pub f37: u32,
        // F38: u32 (sub_1410E3380 lookup, 4 wire)
        pub f38: u32,
        // F39: u32. IDA f36-f42 looks like 15B (would make this phantom +4) but
        // removing it → with_body 0 (a compensating -4 under-read in f75-f179).
        // Kept until that -4 field is found. [[gimmick-f39-phantom]]
        pub f39: u32,
        // F40-F41: 2 u8 values
        pub f40_41: [u8; 2],
        // F42: u32
        pub f42: u32,
        // F43: u8 flag + CArray<u64> (both unconditionally read)
        pub f43_flag: u8,
        pub f43_list: CArray<u64>,
        // F44-F45: u64×2
        pub f44: u64,
        pub f45: u64,
        // F46: COptional<{GimmickHelperBlock+u8+u8+u32+u8+u8+u8}>
        pub f46: COptional<GimmickF46Data>,
        // F47: [u32;3]
        pub f47: [u32; 3],
        // F48-F50: u32×3
        pub f48: u32,
        pub f49: u32,
        pub f50: u32,
        // F51: u8
        pub f51: u8,
        // F52-F56: u32×5
        pub f52: u32,
        pub f53: u32,
        pub f54: u32,
        pub f55: u32,
        pub f56: u32,
        // F57: [u32;3]
        pub f57: [u32; 3],
        // F58-F61: u32×4 + u8
        pub f58: u32,
        pub f59: u32,
        pub f60: u32,
        pub f61: u32,
        pub f61b: u8,
        // F62: u8
        pub f62: u8,
        // F63-F67: u32×5
        pub f63: u32,
        pub f64: u32,
        pub f65: u32,
        pub f66: u32,
        pub f67: u32,
        // F68-F70: [u8;3]
        pub f68_70: [u8; 3],
        // F71: u32
        pub f71: u32,
        // F72: [u32;3]
        pub f72: [u32; 3],
        // F73: u32
        pub f73: u32,
        // F74: CString (socket name e.g. "Gimmick_Bag_00_Socket") — was mis-typed
        // u32; f74 is 0 for 12946 records (empty CString) and a real name length
        // for the rest, so reading it as u32 drifted any record with an f74 name.
        pub f74: CString<'a>,
        // F75: CArray<{u32+u32}> (DA 113C8 lookup per element)
        pub f75: CArray<GimmickF75Elem>,
        // F76: CArray<{COptional<GimmickF76Inner>+u32}> (sub_141112050)
        pub f76: CArray<GimmickF76Elem>,
        // F77: COptional<GimmickF76Inner> (sub_141D03AA0 → sub_141600210)
        pub f77: COptional<GimmickF76Inner>,
        // F78: CArray<{u32+CArray<{u32+u32}>}>
        pub f78: CArray<GimmickF78Elem>,
        // F79: CArray<{u32+u8×3+CArray<GimmickF79Inner>+u8}> (sub_141111CD0 / sub_1410E61F0)
        pub f79: CArray<GimmickF79Elem<'a>>,
        // F80: CArray<u32> (same as F22)
        pub f80: CArray<u32>,
        // F81: CArray<{u32×4+CArray<u32>+u32}>
        pub f81: CArray<GimmickF81Elem>,
        // F82-F83: u32×2
        pub f82: u32,
        pub f83: u32,
        // F84-F85: [u8;2]
        pub f84_85: [u8; 2],
        // F86: CString×2 + u32×3 (two CString hashes + 3 scalars)
        pub f86_str_a: CString<'a>,
        pub f86_str_b: CString<'a>,
        pub f86_a: u32,
        pub f86_b: u32,
        pub f86_c: u32,
        // F87: CArray<{GimmickF87Inner + u32}> (sub_141105260 / sub_1410F7F20)
        pub f87: CArray<GimmickF87Elem<'a>>,
        // F88: CArray<{GimmickF88Inner + u32}> (sub_141105390 / sub_1410F7440)
        pub f88: CArray<GimmickF88Elem<'a>>,
        // F89: CArray<{u32+u16+[u32;3]+[u32;3]+u32+[u32;4]+u32+u8+u8+u8+u32+CArray<u32>+u16+u16}>
        pub f89: CArray<GimmickF89Elem<'a>>,
        // F90: CArray<{CString+CArray<sub>+u64+u8+u8+u32+u16}>
        pub f90: CArray<GimmickF90Elem<'a>>,
        // F91: u32
        pub f91: u32,
        // F92: CArray<complex CString element>
        pub f92: CArray<GimmickF92Elem<'a>>,
        // F93-F94: u32×2 (DA lookup per item)
        pub f93: u32,
        pub f94: u32,
        // F95-F96: u32×2
        pub f95: u32,
        pub f96: u32,
        // F97: CArray<[u8;260]> (260-byte fixed path strings)
        pub f97: CArray<GimmickF97Elem>,
        // F98: u8
        pub f98: u8,
        // F99: u32
        pub f99: u32,
        // F100: CArray<CString> (bytes: count + N×["LungeSocket_NN" len-prefixed])
        pub f100: CArray<CString<'a>>,
        // F101: CArray<CString> (socket/bone names e.g. "CrankHandle01B") — was
        // mis-typed CArray<u32>, reading name lengths/bytes as u32 elements and
        // drifting any record with a populated f101 socket-name list.
        pub f101: CArray<CString<'a>>,
        // F102-F103: [u8;2]
        pub f102_103: [u8; 2],
        // F104: u16 (→u16 DA40 lookup; reads u16 on wire)
        pub f104: u16,
        // F105: u16 (→u16 DA18 lookup)
        pub f105: u16,
        // F106-F115: [u8;10]
        pub f106_115: [u8; 10],
        // F116: u32
        pub f116: u32,
        // F117: COptional<{CArray<20b sub-elem>+GimmickBlock32+u32}>
        pub f117: COptional<GimmickF117Data<'a>>,
        // F118: u8
        pub f118: u8,
        // F119: CArray<{CArray<{u16+GimmickBlock32}>+u8}>
        pub f119: CArray<GimmickF119Elem<'a>>,
        // F120-F122: u32×3
        pub f120: u32,
        pub f121: u32,
        pub f122: u32,
        // F123: u8
        pub f123: u8,
        // F124: CString
        pub f124: CString<'a>,
        // F125: CArray<{u32+u32+[u8;12]+[u8;12]}>
        pub f125: CArray<GimmickF125Elem<'a>>,
        // F126: CArray<{u8+u32+u8+u8+U32x10+u64+u32+u8×5+u32+u32+u8+u32}>
        pub f126: CArray<GimmickF126Elem>,
        // F127: CArray<same element type as F126>
        pub f127: CArray<GimmickF126Elem>,
        // F128: CArray<{CString+u32+u32+u32}>
        pub f128: CArray<GimmickF128Elem<'a>>,
        // F129: CArray<{u32+u32+[u32;3]+[u32;4]+[u32;3]}>
        pub f129: CArray<GimmickF129Elem>,
        // F130: CArray<GimmickF130Elem> (sub_1410E5E40)
        pub f130: CArray<GimmickF130Elem>,
        // F131: u32
        pub f131: u32,
        // F131b: CArray<GimmickF131bElem> (IDA sub_1410F3800 @ a2+1048) — sits
        // between f131 (@1040 u32) and f132 (sub_1410C8960 @1064). Was missing.
        pub f131b: CArray<GimmickF131bElem<'a>>,
        // F132: GimmickBlock32×2+u32+u16+(CArray<u32>+CArray<264b>)×2
        pub f132: GimmickF132<'a>,
        // F133: u32
        pub f133: u32,
        // F133b: u32 — IDA sub_1410C8960 (f132) ends at mem a2+1200, then
        // TWO u32 reads (a2+1200, a2+1204) precede the u8 at a2+1208. Rust
        // previously had only one u32 here; this second u32 was missing.
        pub f133b: u32,
        // F134: u8
        pub f134: u8,
        // F135: u32
        pub f135: u32,
        // F136-F138: [u8;3]
        pub f136_138: [u8; 3],
        // F139-F141: u32×3
        pub f139: u32,
        pub f140: u32,
        pub f141: u32,
        // F142-F144: [u8;3]
        pub f142_144: [u8; 3],
        // F145: u32
        pub f145: u32,
        // F146: u32×3+u8+u8
        pub f146_a: u32,
        pub f146_b: u32,
        pub f146_c: u32,
        pub f146_d: u8,
        pub f146_e: u8,
        // F147: u16 (→u16 17B68 lookup)
        pub f147: u16,
        // F148: CArray<u16> (u16→u16 15028 lookup per element)
        pub f148: CArray<u16>,
        // F149: u8
        pub f149: u8,
        // F150: u16 (→u16 DA18 lookup)
        pub f150: u16,
        // F151: u16
        pub f151: u16,
        // F152-F153: 2 u8 before CString (IDA: read @a2+1292 1B, @a2+1294 1B).
        // The old [u8;4] masked the missing f131b CArray (a +2/+4 upstream gap).
        pub f152_155: [u8; 2],
        // F154: CString
        pub f154: CString<'a>,
        // F155-F163: [u8;9]
        pub f155_163: [u8; 9],
        // F164: u32
        pub f164: u32,
        // F165: u64
        pub f165: u64,
        // F166: CArray<{u32+u32}>
        pub f166: CArray<GimmickF75Elem>,
        // F167: CArray<{u32+u32}>
        pub f167: CArray<GimmickF75Elem>,
        // F168: CArray<COptional<{u32+u32+U32x10}>>
        pub f168: CArray<COptional<GimmickF168Inner>>,
        // F169: CArray<COptional<{u32+u32+U32x10}>>
        pub f169: CArray<COptional<GimmickF168Inner>>,
        // F170: u32+u32+u32+CArray<{u64+u32}>
        pub f170_a: u32,
        pub f170_b: u32,
        pub f170_c: u32,
        pub f170_list: CArray<GimmickF170Elem>,
        // F171: u32
        pub f171: u32,
        // F172-F175: [u8;4]
        pub f172_175: [u8; 4],
        // F176: u32
        pub f176: u32,
        // F177: u8
        pub f177: u8,
        // F178: u32
        pub f178: u32,
        // F179: u32
        pub f179: u32,
    }
}

// ── GimmickTail ───────────────────────────────────────────────────────────────

/// Tail of GimmickInfo. When the field-7 CArray decode succeeds (and
/// the immediately-following stable scalar block parses cleanly) it
/// joins the typed prefix; the rest of the body (~85 fields) still
/// rides as `post_blob`. On any decode failure the entire post-prefix
/// region is captured as `Raw`.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum GimmickTail<'a> {
    Decoded {
        gimmick_interaction_override_list: GimmickInteractionOverrideCArray<'a>,
        use_interaction_ui_socket: u8,
        use_sub_part_for_interaction: u8,
        /// Present only for gimmick group 12026 (unique physics type): an extra
        /// f32 (raw u32 bits) before property_list. None for all other groups.
        group12026_extra: Option<u32>,
        property_list: CArray<u32>,
        gimmick_name_hash: u32,
        gimmick_name: Box<LocalizableString<'a>>,
        emoji_texture_id: Box<CString<'a>>,
        dev_memo: Box<CString<'a>>,
        hash_pair_list: CArray<GimmickHashPair<'a>>,
        hash_single_list: CArray<GimmickHashSingle<'a>>,
        /// sub_1411125E0 — `CArray<COptional<TriggerEventHandlerDataElement>>`.
        trigger_event_handler_list: Option<CArray<COptional<TriggerEventHandlerDataElement<'a>>>>,
        /// sub_141C7F8B0 — `CArray<GimmickChartParameter>`.
        gimmick_chart_parameter_list: Option<CArray<GimmickChartParameter<'a>>>,
        /// F19 — `CArray<COptional<CString>>` alt-trigger names.
        alt_trigger_list: Option<CArray<COptional<CArray<GimmickF19InnerElem>>>>,
        /// F20-F179 fully typed when alt_trigger_list decoded and count=0.
        post_body: Option<Box<GimmickPostBody<'a>>>,
        post_blob: Vec<u8>,
        /// Wire offset where the post-body (F20+) begins. Debug aid for RE.
        post_body_start: usize,
        /// True when the post-body could not be fully field-typed (a variant
        /// structure) and is preserved byte-exact in `post_blob` instead. The
        /// record still round-trips losslessly; these fields just aren't typed.
        post_body_raw: bool,
    },
    Raw(Vec<u8>),
}

impl<'a> GimmickTail<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_end: usize, group: u32) -> io::Result<Self> {
        let tail_start = *offset;
        let mut probe = tail_start;
        let try_decode = (|| -> io::Result<_> {
            let tr = std::env::var("RAWDIAG2").is_ok() && tail_start == 218058;
            macro_rules! tp { ($nm:expr) => { if tr { eprintln!("RAWDIAG2 {} ok @{}", $nm, probe); } } }
            let list = GimmickInteractionOverrideCArray::read_from(data, &mut probe)?; tp!("F1");
            if probe > entry_end { return Err(io::Error::new(io::ErrorKind::InvalidData, "overrun")); }
            let use_interaction_ui_socket = u8::read_from(data, &mut probe)?;
            let use_sub_part_for_interaction = u8::read_from(data, &mut probe)?;
            // Gimmick group 12026 (a unique physics gimmick type; every other
            // record is group 1000xxx) carries an extra f32 (e.g. 49.0, an angular
            // velocity / range) before the property_list. Stored as raw u32 bits
            // for foolproof byte-exact round-trip. Confirmed: this is the sole
            // F1-F19 difference for all 5 group-12026 records (raw 5→0).
            let group12026_extra = if group == 12026 {
                Some(u32::read_from(data, &mut probe)?)
            } else {
                None
            };
            let property_list = <CArray<u32>>::read_from(data, &mut probe)?; tp!("F4");
            let gimmick_name_hash = u32::read_from(data, &mut probe)?;
            let gimmick_name = LocalizableString::read_from(data, &mut probe)?; tp!("F6");
            let emoji_texture_id = Box::new(CString::read_from(data, &mut probe)?); tp!("F7");
            let dev_memo = Box::new(CString::read_from(data, &mut probe)?); tp!("F8");
            let hash_pair_list = <CArray<GimmickHashPair>>::read_from(data, &mut probe)?; tp!("F9");
            let hash_single_list = <CArray<GimmickHashSingle>>::read_from(data, &mut probe)?; tp!("F10");
            if probe > entry_end { return Err(io::Error::new(io::ErrorKind::InvalidData, "overrun")); }
            Ok((list, use_interaction_ui_socket, use_sub_part_for_interaction, group12026_extra,
                property_list, gimmick_name_hash, gimmick_name, emoji_texture_id, dev_memo,
                hash_pair_list, hash_single_list))
        })();
        match try_decode {
            Ok((list, ui, sp, group12026_extra, pl, gnh, gn, eti, dm, hpl, hsl)) => {
                // F17: CArray<COptional<TGPEHD>>; safe optional probe.
                let pre17 = probe;
                let trigger_event_handler_list = match <CArray<COptional<TriggerEventHandlerDataElement>>>::read_from(data, &mut probe) {
                    Ok(arr) if probe <= entry_end => Some(arr),
                    _ => {
                        if std::env::var("F17DIAG").is_ok() {
                            let c = if pre17+4<=data.len() { u32::from_le_bytes([data[pre17],data[pre17+1],data[pre17+2],data[pre17+3]]) } else {0};
                            eprintln!("F17FAIL group={} ts={} pre17={} f17count=0x{:x}", group, tail_start, pre17, c);
                        }
                        probe = pre17; None
                    }
                };
                // F18: gimmick_chart_parameter_list.
                let gimmick_chart_parameter_list = if trigger_event_handler_list.is_some() {
                    let pre18 = probe;
                    match <CArray<GimmickChartParameter>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => {
                            if std::env::var("F18DIAG").is_ok() {
                                let c = if pre18+4<=data.len() { u32::from_le_bytes([data[pre18],data[pre18+1],data[pre18+2],data[pre18+3]]) } else {0};
                                eprintln!("F18FAIL group={} ts={} pre18={} f18count=0x{:x}", group, tail_start, pre18, c);
                            }
                            probe = pre18; None
                        }
                    }
                } else {
                    None
                };
                // F19: CArray<COptional<CString>> alt-trigger name list.
                let alt_trigger_list = if gimmick_chart_parameter_list.is_some() {
                    let pre19 = probe;
                    match <CArray<COptional<CArray<GimmickF19InnerElem>>>>::read_from(data, &mut probe) {
                        Ok(arr) if probe <= entry_end => Some(arr),
                        _ => {
                            if std::env::var("F19DIAG").is_ok() {
                                let c = if pre19+4<=data.len() { u32::from_le_bytes([data[pre19],data[pre19+1],data[pre19+2],data[pre19+3]]) } else {0};
                                eprintln!("F19FAIL group={} ts={} pre19={} f19count=0x{:x}", group, tail_start, pre19, c);
                            }
                            probe = pre19; None
                        }
                    }
                } else {
                    None
                };
                // F20-F179: GimmickPostBody; only attempted when F19 decoded.
                static DIAG_CNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                let post_body_start = probe;
                let post_body = if alt_trigger_list.is_some() {
                    let pre_body = probe;
                    let result = GimmickPostBody::read_from(data, &mut probe);
                    match result {
                        Ok(body) if probe <= entry_end => Some(Box::new(body)),
                        Ok(_) => {
                            let n = DIAG_CNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            if n < 10 {
                                eprintln!("DIAG[{}] overshot: probe={} entry_end={} blob_remaining={} delta={}", n, probe, entry_end, entry_end.saturating_sub(pre_body), probe.saturating_sub(entry_end));
                            }
                            probe = pre_body; None
                        }
                        Err(e) => {
                            let n = DIAG_CNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            if n < 10 {
                                eprintln!("DIAG[{}] error: tail_start={} post_body_start={} probe={} entry_end={} blob_remaining={}: {}", n, tail_start, post_body_start, probe, entry_end, entry_end.saturating_sub(pre_body), e);
                            }
                            probe = pre_body; None
                        }
                    }
                } else {
                    None
                };
                let post_blob = data[probe..entry_end].to_vec();
                *offset = entry_end;
                // The post-body either typed cleanly (post_body=Some) or it hit a
                // variant region we don't yet field-type — in which case the bytes
                // ride byte-exact in post_blob. Either way the post-body is fully
                // accounted for (round-trips losslessly); mark the raw case.
                let post_body_raw = post_body.is_none() && !post_blob.is_empty();
                Ok(GimmickTail::Decoded {
                    gimmick_interaction_override_list: list,
                    use_interaction_ui_socket: ui,
                    use_sub_part_for_interaction: sp,
                    group12026_extra,
                    property_list: pl,
                    gimmick_name_hash: gnh,
                    gimmick_name: Box::new(gn),
                    emoji_texture_id: eti,
                    dev_memo: dm,
                    hash_pair_list: hpl,
                    hash_single_list: hsl,
                    trigger_event_handler_list,
                    gimmick_chart_parameter_list,
                    alt_trigger_list,
                    post_body,
                    post_blob,
                    post_body_start,
                    post_body_raw,
                })
            }
            Err(e) => {
                if std::env::var("RAWDIAG").is_ok() {
                    static RC: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                    let n = RC.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if n < 2000 { eprintln!("RAWDIAG[{}] tail_start={} entry_end={} err={}", n, tail_start, entry_end, e); }
                }
                if std::env::var("RAWTAG").is_ok() {
                    let t = crate::binary::variants::condition_data::LAST_ATTEMPTED_TAG.with(|c| c.get());
                    eprintln!("RAWTAG last_disc={:?}", t);
                }
                if std::env::var("RAWRANGE").is_ok() {
                    eprintln!("RAWRANGE {} {}", tail_start, entry_end);
                }
                let blob = data[tail_start..entry_end].to_vec();
                *offset = entry_end;
                Ok(GimmickTail::Raw(blob))
            }
        }
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match self {
            GimmickTail::Decoded {
                gimmick_interaction_override_list, use_interaction_ui_socket,
                use_sub_part_for_interaction, group12026_extra, property_list, gimmick_name_hash,
                gimmick_name, emoji_texture_id, dev_memo, hash_pair_list, hash_single_list,
                trigger_event_handler_list, gimmick_chart_parameter_list,
                alt_trigger_list, post_body, post_blob, post_body_start: _, post_body_raw: _,
            } => {
                gimmick_interaction_override_list.write_to(w)?;
                use_interaction_ui_socket.write_to(w)?;
                use_sub_part_for_interaction.write_to(w)?;
                if let Some(x) = group12026_extra { x.write_to(w)?; }
                property_list.write_to(w)?;
                gimmick_name_hash.write_to(w)?;
                gimmick_name.write_to(w)?;
                emoji_texture_id.write_to(w)?;
                dev_memo.write_to(w)?;
                hash_pair_list.write_to(w)?;
                hash_single_list.write_to(w)?;
                if let Some(arr) = trigger_event_handler_list { arr.write_to(w)?; }
                if let Some(arr) = gimmick_chart_parameter_list { arr.write_to(w)?; }
                if let Some(arr) = alt_trigger_list { arr.write_to(w)?; }
                if let Some(body) = post_body { body.write_to(w)?; }
                w.write_all(post_blob)
            }
            GimmickTail::Raw(b) => w.write_all(b),
        }
    }

    pub fn to_json_value(&self) -> Value {
        match self {
            GimmickTail::Decoded {
                gimmick_interaction_override_list, use_interaction_ui_socket,
                use_sub_part_for_interaction, group12026_extra, property_list, gimmick_name_hash,
                gimmick_name, emoji_texture_id, dev_memo, hash_pair_list, hash_single_list,
                trigger_event_handler_list, gimmick_chart_parameter_list,
                alt_trigger_list, post_body, post_blob, post_body_start: _, post_body_raw,
            } => {
                let mut m = Map::new();
                m.insert("kind".to_string(), Value::String("Decoded".to_string()));
                m.insert("post_body_raw".to_string(), Value::Bool(*post_body_raw));
                m.insert("gimmick_interaction_override_list".to_string(),
                         gimmick_interaction_override_list.to_json_value());
                m.insert("use_interaction_ui_socket".to_string(), use_interaction_ui_socket.to_json_value());
                m.insert("use_sub_part_for_interaction".to_string(), use_sub_part_for_interaction.to_json_value());
                m.insert("group12026_extra".to_string(), match group12026_extra { Some(x) => x.to_json_value(), None => Value::Null });
                m.insert("property_list".to_string(), property_list.to_json_value());
                m.insert("gimmick_name_hash".to_string(), gimmick_name_hash.to_json_value());
                m.insert("gimmick_name".to_string(), gimmick_name.to_json_value());
                m.insert("emoji_texture_id".to_string(), emoji_texture_id.to_json_value());
                m.insert("dev_memo".to_string(), dev_memo.to_json_value());
                m.insert("hash_pair_list".to_string(), hash_pair_list.to_json_value());
                m.insert("hash_single_list".to_string(), hash_single_list.to_json_value());
                m.insert("trigger_event_handler_list".to_string(), match trigger_event_handler_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null,
                });
                m.insert("gimmick_chart_parameter_list".to_string(), match gimmick_chart_parameter_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null,
                });
                m.insert("alt_trigger_list".to_string(), match alt_trigger_list {
                    Some(arr) => arr.to_json_value(), None => Value::Null,
                });
                m.insert("post_body".to_string(), match post_body {
                    Some(b) => b.to_json_value(), None => Value::Null,
                });
                m.insert("_post_blob_b64".to_string(), Value::String(B64.encode(post_blob)));
                // Reward overlay: expose the editable gather-count / friendship
                // values scanned out of the raw post-body blob (see drop_info_scan).
                m.insert("drop_info_data_list".to_string(),
                         super::drop_info_scan::reward_list_json(post_blob));
                Value::Object(m)
            }
            GimmickTail::Raw(b) => {
                let mut m = Map::new();
                m.insert("kind".to_string(), Value::String("Raw".to_string()));
                m.insert("_b64".to_string(), Value::String(B64.encode(b)));
                m.insert("drop_info_data_list".to_string(),
                         super::drop_info_scan::reward_list_json(b));
                Value::Object(m)
            }
        }
    }

    pub fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        let obj = v.as_object().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData, "GimmickTail: expected object",
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
                if let Some(v) = obj.get("group12026_extra").filter(|v| !v.is_null()) {
                    <u32 as WriteJsonValue>::write_from_json(w, v)?;
                }
                <CArray<u32> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "property_list")?)?;
                <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "gimmick_name_hash")?)?;
                <LocalizableString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "gimmick_name")?)?;
                <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "emoji_texture_id")?)?;
                <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "dev_memo")?)?;
                <CArray<GimmickHashPair> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "hash_pair_list")?)?;
                <CArray<GimmickHashSingle> as WriteJsonValue>::write_from_json(w, json_get_field(obj, "hash_single_list")?)?;
                let teh = json_get_field(obj, "trigger_event_handler_list")?;
                if !teh.is_null() {
                    <CArray<COptional<TriggerEventHandlerDataElement>> as WriteJsonValue>::write_from_json(w, teh)?;
                }
                let gcpl = json_get_field(obj, "gimmick_chart_parameter_list")?;
                if !gcpl.is_null() {
                    <CArray<GimmickChartParameter> as WriteJsonValue>::write_from_json(w, gcpl)?;
                }
                let atl = json_get_field(obj, "alt_trigger_list")?;
                if !atl.is_null() {
                    <CArray<COptional<CArray<GimmickF19InnerElem>>> as WriteJsonValue>::write_from_json(w, atl)?;
                }
                let pb = json_get_field(obj, "post_body")?;
                if !pb.is_null() {
                    GimmickPostBody::write_from_json(w, pb)?;
                }
                let b64 = json_get_field(obj, "_post_blob_b64")?.as_str()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                        "GimmickTail.Decoded._post_blob_b64: expected string"))?;
                let mut bytes = B64.decode(b64).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
                    format!("GimmickTail.Decoded._post_blob_b64: invalid base64: {}", e)))?;
                // Apply any reward-overlay edits back into the blob. Unedited
                // (patch-same-value) is byte-identical, so round-trip is preserved.
                if let Some(list) = obj.get("drop_info_data_list") {
                    super::drop_info_scan::patch_blob_from_json(&mut bytes, list);
                }
                w.extend_from_slice(&bytes);
                Ok(())
            }
            "Raw" => {
                let b64 = json_get_field(obj, "_b64")?.as_str()
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
                        "GimmickTail.Raw._b64: expected string"))?;
                let mut bytes = B64.decode(b64).map_err(|e| io::Error::new(io::ErrorKind::InvalidData,
                    format!("GimmickTail.Raw._b64: invalid base64: {}", e)))?;
                if let Some(list) = obj.get("drop_info_data_list") {
                    super::drop_info_scan::patch_blob_from_json(&mut bytes, list);
                }
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
        let tail = GimmickTail::read_with_size(data, offset, entry_end, gimmick_group_info)?;

        Ok(Self {
            key, string_key, is_blocked, prefab_path,
            gimmick_group_info, breakable_object_info, tail,
        })
    }

    pub fn read_tracked_with_size(
        data: &'a [u8],
        offset: &mut usize,
        entry_size: usize,
        path: &mut String,
        ranges: &mut Vec<FieldRange>,
    ) -> io::Result<Self> {
        let entry_end = *offset + entry_size;
        let key = track_read_field::<u32>(data, offset, path, ranges, "key", "u32")?;
        let string_key = track_read_field::<CString<'a>>(data, offset, path, ranges, "string_key", "CString")?;
        let is_blocked = track_read_field::<u8>(data, offset, path, ranges, "is_blocked", "u8")?;
        let prefab_path = track_read_field::<CString<'a>>(data, offset, path, ranges, "prefab_path", "CString")?;
        let gimmick_group_info = track_read_field::<u32>(data, offset, path, ranges, "gimmick_group_info", "u32")?;
        let breakable_object_info = track_read_field::<u16>(data, offset, path, ranges, "breakable_object_info", "u16")?;
        let tail = track_read_with(offset, path, ranges, "tail", "GimmickTail", |o| {
            GimmickTail::read_with_size(data, o, entry_end, gimmick_group_info)
        })?;
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
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets_from_bytes};

    fn find_fixture() -> Option<(Vec<u8>, Vec<u8>)> {
        let candidates: &[(&str, &str)] = &[
            (
                r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser\pabgb-dumps-1.07\gimmickinfo.pabgb",
                r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-parser\pabgb-dumps-1.07\gimmickinfo.pabgh",
            ),
            (
                "/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/gimmickinfo.pabgb",
                "/mnt/c/temp/GIT/CrimsonDesertUpdates/pabgb/2026-5-1/gimmickinfo.pabgh",
            ),
            (
                r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-5-1\gimmickinfo.pabgb",
                r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-5-1\gimmickinfo.pabgh",
            ),
        ];
        if let Ok(p) = std::env::var("DMM_PARSER_GIMMICKINFO_PABGB") {
            let q = std::env::var("DMM_PARSER_GIMMICKINFO_PABGH").ok()?;
            if let (Ok(d), Some(e)) = (std::fs::read(&p), std::fs::read(&q).ok()) {
                return Some((d, e));
            }
        }
        for (pb, pg) in candidates {
            if let (Ok(d), Ok(e)) = (std::fs::read(pb), std::fs::read(pg)) {
                return Some((d, e));
            }
        }
        None
    }

    macro_rules! load_or_skip {
        () => {
            match find_fixture() {
                Some(pair) => pair,
                None => { eprintln!("SKIP: gimmickinfo fixture not found"); return; }
            }
        };
    }


    #[test]
    fn roundtrip() {
        let (data, pabgh_data) = load_or_skip!();
        let Some(entries) = load_pabgh_offsets_from_bytes(&pabgh_data) else { eprintln!("SKIP: bad pabgh"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        let mut decoded = 0usize;
        let mut raw = 0usize;
        let mut with_body = 0usize;
        let mut raw_fallback = 0usize;
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            let item = GimmickInfo::read_with_size(&data, &mut c, e - s)
                .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er));
            assert_eq!(c, *e);
            match &item.tail {
                GimmickTail::Decoded { trigger_event_handler_list, gimmick_chart_parameter_list, alt_trigger_list, post_body, post_body_raw, .. } => {
                    decoded += 1;
                    if post_body.is_some() { with_body += 1; }
                    else if *post_body_raw { raw_fallback += 1; }
                    let _ = (trigger_event_handler_list, gimmick_chart_parameter_list, alt_trigger_list);
                }
                GimmickTail::Raw(_) => raw += 1,
            }
            items.push(item);
        }
        eprintln!("gimmickinfo: decoded={} raw={} with_body={} raw_fallback={} (total={})",
                  decoded, raw, with_body, raw_fallback, ranges.len());
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "gimmickinfo roundtrip mismatch");
    }

    #[test]
    fn post_body_diag() {
        let (data, pabgh_data) = load_or_skip!();
        let Some(entries) = load_pabgh_offsets_from_bytes(&pabgh_data) else { return; };
        let ranges = entry_ranges(&entries, data.len());
        use crate::binary::{BinaryRead, CArray, COptional, CString};
        'outer: for (_k, s, e) in &ranges {
            let mut probe = *s;
            let item = GimmickInfo::read_with_size(&data, &mut probe, e - s).unwrap();
            if let GimmickTail::Decoded { alt_trigger_list: Some(_), post_body: None, post_blob, .. } = &item.tail {
                let post_start = e - post_blob.len();
                eprintln!("post_start={} entry_end={} blob_len={}", post_start, e, post_blob.len());
                let p = &mut { post_start };
                macro_rules! rd {
                    ($t:ty, $p:expr, $name:expr) => {{
                        let lp = *$p;
                        let v = match <$t>::read_from(&data, $p) {
                            Ok(v) => v,
                            Err(e) => { eprintln!("  {} FAILED at off={}: {}", $name, lp, e); break 'outer; }
                        };
                        eprintln!("  {} [off={}]", $name, *$p);
                        v
                    }};
                }
                macro_rules! rdc {
                    ($p:expr, $name:expr) => {{
                        let len_pos = *$p;
                        match CString::read_from(&data, $p) {
                            Ok(s) => { eprintln!("  {} len={} [off={}]", $name, s.length, *$p); }
                            Err(e) => { eprintln!("  {} FAILED at len_pos={}: {}", $name, len_pos, e); break 'outer; }
                        }
                    }};
                }
                rd!(CArray<GimmickF20Elem>, p, "f20");
                rd!(u8, p, "f21");
                rd!(CArray<u32>, p, "f22");
                rd!(CArray<u32>, p, "f23");
                rd!(CArray<GimmickF24Elem>, p, "f24");
                rd!(u64, p, "f25");
                rd!([u8;8], p, "f26_32");
                rd!(u32, p, "f33_a");
                rd!(u8, p, "f33_b");
                rd!(u8, p, "f33_c");
                rd!(CArray<GimmickF34Elem>, p, "f34");
                rd!(CArray<GimmickF35Elem>, p, "f35");
                rd!(u8, p, "f36");
                rd!(u32, p, "f37");
                rd!(u32, p, "f38");
                rd!(u32, p, "f39");
                rd!([u8;2], p, "f40_41");
                rd!(u32, p, "f42");
                rd!(u8, p, "f43_flag");
                rd!(CArray<u64>, p, "f43_list");
                rd!(u64, p, "f44");
                rd!(u64, p, "f45");
                rd!(COptional<GimmickF46Data>, p, "f46");
                rd!([u32;3], p, "f47");
                rd!(u32, p, "f48");
                rd!(u32, p, "f49");
                rd!(u32, p, "f50");
                rd!(u8, p, "f51");
                rd!(u32, p, "f52");
                rd!(u32, p, "f53");
                rd!(u32, p, "f54");
                rd!(u32, p, "f55");
                rd!(u32, p, "f56");
                rd!([u32;3], p, "f57");
                rd!(u32, p, "f58");
                rd!(u32, p, "f59");
                rd!(u32, p, "f60");
                rd!(u32, p, "f61");
                rd!(u8, p, "f61b");
                rd!(u8, p, "f62");
                rd!(u32, p, "f63");
                rd!(u32, p, "f64");
                rd!(u32, p, "f65");
                rd!(u32, p, "f66");
                rd!(u32, p, "f67");
                rd!([u8;3], p, "f68_70");
                rd!(u32, p, "f71");
                rd!([u32;3], p, "f72");
                rd!(u32, p, "f73");
                rd!(u32, p, "f74");
                rd!(CArray<GimmickF75Elem>, p, "f75");
                { let cnt_pos=*p; let cnt=u32::read_from(&data,p).unwrap(); eprintln!("  f76(empty) count={} [cnt_pos={}]",cnt,cnt_pos); if cnt!=0 { eprintln!("  STOP: f76 non-zero"); return; } }
                { let fl_pos=*p; let fl=u8::read_from(&data,p).unwrap(); eprintln!("  f77(absent) flag={} [pos={}]",fl,fl_pos); if fl!=0 { eprintln!("  STOP: f77 non-zero"); return; } }
                rd!(CArray<GimmickF78Elem>, p, "f78");
                rd!(CArray<GimmickF79Elem>, p, "f79");
                rd!(CArray<u32>, p, "f80");
                rd!(CArray<GimmickF81Elem>, p, "f81");
                rd!(u32, p, "f82");
                rd!(u32, p, "f83");
                rd!([u8;2], p, "f84_85");
                rdc!(p, "f86_str_a");
                rdc!(p, "f86_str_b");
                rd!(u32, p, "f86_a");
                rd!(u32, p, "f86_b");
                rd!(u32, p, "f86_c");
                rd!(CArray<GimmickF87Elem>, p, "f87");
                rd!(CArray<GimmickF88Elem>, p, "f88");
                rd!(CArray<GimmickF89Elem>, p, "f89");
                rd!(CArray<GimmickF90Elem>, p, "f90");
                rd!(u32, p, "f91");
                rd!(CArray<GimmickF92Elem>, p, "f92");
                rd!(u32, p, "f93");
                rd!(u32, p, "f94");
                rd!(u32, p, "f95");
                rd!(u32, p, "f96");
                rd!(CArray<GimmickF97Elem>, p, "f97");
                rd!(u8, p, "f98");
                rd!(u32, p, "f99");
                rd!(CArray<CString>, p, "f100");
                rd!(CArray<u32>, p, "f101");
                rd!([u8;2], p, "f102_103");
                rd!(u16, p, "f104");
                rd!(u16, p, "f105");
                rd!([u8;10], p, "f106_115");
                rd!(u32, p, "f116");
                rd!(COptional<GimmickF117Data>, p, "f117");
                rd!(u8, p, "f118");
                rd!(CArray<GimmickF119Elem>, p, "f119");
                rd!(u32, p, "f120");
                rd!(u32, p, "f121");
                rd!(u32, p, "f122");
                rd!(u8, p, "f123");
                rdc!(p, "f124");
                rd!(CArray<GimmickF125Elem>, p, "f125");
                rd!(CArray<GimmickF126Elem>, p, "f126");
                rd!(CArray<GimmickF126Elem>, p, "f127");
                rd!(CArray<GimmickF128Elem>, p, "f128");
                rd!(CArray<GimmickF129Elem>, p, "f129");
                rd!(CArray<GimmickF130Elem>, p, "f130");
                rd!(u32, p, "f131");
                // f132 manually
                { let fp=*p; let fl=u8::read_from(&data,p).unwrap(); let fv=u64::read_from(&data,p).unwrap(); eprintln!("  f132.block_a flag={} val={} [off_before={}]",fl,fv,fp); rdc!(p, "f132.block_a.name"); }
                { let fp=*p; let fl=u8::read_from(&data,p).unwrap(); let fv=u64::read_from(&data,p).unwrap(); eprintln!("  f132.block_b flag={} val={} [off_before={}]",fl,fv,fp); rdc!(p, "f132.block_b.name"); }
                rd!(u32, p, "f132.hash");
                rd!(u32, p, "f132.val");
                { let cnt_pos=*p; let cnt=u32::read_from(&data,p).unwrap(); eprintln!("  f132.list_a_u32 count={} [cnt_pos={}]",cnt,cnt_pos); for _i in 0..cnt { u32::read_from(&data,p).unwrap(); } eprintln!("  f132.list_a_u32 done [off={}]", *p); }
                { let cnt_pos=*p; let cnt=u32::read_from(&data,p).unwrap(); eprintln!("  f132.list_a_264b count={} [cnt_pos={}]",cnt,cnt_pos); for i in 0..cnt { GimmickDD420Elem::read_from(&data,p).unwrap_or_else(|e| panic!("list_a_264b[{}]: {}", i, e)); } eprintln!("  f132.list_a_264b done [off={}]", *p); }
                { let cnt_pos=*p; let cnt=u32::read_from(&data,p).unwrap(); eprintln!("  f132.list_b_u32 count={} [cnt_pos={}]",cnt,cnt_pos); for _i in 0..cnt { u32::read_from(&data,p).unwrap(); } eprintln!("  f132.list_b_u32 done [off={}]", *p); }
                { let cnt_pos=*p; let cnt=u32::read_from(&data,p).unwrap(); eprintln!("  f132.list_b_264b count={} [cnt_pos={}]",cnt,cnt_pos); for i in 0..cnt { GimmickDD420Elem::read_from(&data,p).unwrap_or_else(|e| panic!("list_b_264b[{}]: {}", i, e)); } eprintln!("  f132.list_b_264b done [off={}]", *p); }
                eprintln!("  ALL fields through f132 OK at off={}", *p);
                rd!(u32, p, "f133");
                rd!(u32, p, "f133b");
                rd!(u8, p, "f134");
                rd!(u32, p, "f135");
                rd!([u8;3], p, "f136_138");
                rd!(u32, p, "f139");
                rd!(u32, p, "f140");
                rd!(u32, p, "f141");
                rd!([u8;3], p, "f142_144");
                rd!(u32, p, "f145");
                rd!(u32, p, "f146_a");
                rd!(u32, p, "f146_b");
                rd!(u32, p, "f146_c");
                rd!(u8, p, "f146_d");
                rd!(u8, p, "f146_e");
                rd!(u16, p, "f147");
                rd!(CArray<u16>, p, "f148");
                rd!(u8, p, "f149");
                rd!(u16, p, "f150");
                rd!(u16, p, "f151");
                rd!([u8;4], p, "f152_155");
                // Dump next 8 bytes before attempting f154
                if *p + 8 <= data.len() {
                    let peek: [u8;8] = data[*p..*p+8].try_into().unwrap();
                    eprintln!("  bytes at off={}: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}",
                        *p, peek[0],peek[1],peek[2],peek[3],peek[4],peek[5],peek[6],peek[7]);
                }
                rdc!(p, "f154");
                eprintln!("  ALL fields through f154 OK at off={}", *p);
                rd!([u8;9], p, "f155_163");
                rd!(u32, p, "f164");
                rd!(u64, p, "f165");
                rd!(CArray<GimmickF75Elem>, p, "f166");
                rd!(CArray<GimmickF75Elem>, p, "f167");
                rd!(CArray<COptional<GimmickF168Inner>>, p, "f168");
                rd!(CArray<COptional<GimmickF168Inner>>, p, "f169");
                rd!(u32, p, "f170_a");
                rd!(u32, p, "f170_b");
                rd!(u32, p, "f170_c");
                rd!(CArray<GimmickF170Elem>, p, "f170_list");
                rd!(u32, p, "f171");
                rd!([u8;4], p, "f172_175");
                rd!(u32, p, "f176");
                rd!(u8, p, "f177");
                rd!(u32, p, "f178");
                eprintln!("  ALL fields through f178 at off={}, entry_end={}", *p, e);
                break;
            }
        }

        // Second pass: trace a 692-byte blob entry
        eprintln!("\n--- 692-byte blob trace ---");
        'outer2: for (_k, s, e) in &ranges {
            let mut probe = *s;
            let item = GimmickInfo::read_with_size(&data, &mut probe, e - s).unwrap();
            if let GimmickTail::Decoded { alt_trigger_list: Some(_), post_body: None, post_blob, .. } = &item.tail {
                if post_blob.len() != 692 { continue; }
                let post_start = e - post_blob.len();
                eprintln!("post_start={} entry_end={} blob_len={}", post_start, e, post_blob.len());
                let p = &mut { post_start };
                macro_rules! rd2 {
                    ($t:ty, $p:expr, $name:expr) => {{
                        let lp = *$p;
                        let v = match <$t>::read_from(&data, $p) {
                            Ok(v) => v,
                            Err(e) => { eprintln!("  {} FAILED at off={}: {}", $name, lp, e); break 'outer2; }
                        };
                        eprintln!("  {} [off={}]", $name, *$p);
                        v
                    }};
                }
                macro_rules! rdc2 {
                    ($p:expr, $name:expr) => {{
                        let lp = *$p;
                        match CString::read_from(&data, $p) {
                            Ok(s) => { eprintln!("  {} len={} [off={}]", $name, s.length, *$p); }
                            Err(e) => { eprintln!("  {} FAILED at len_pos={}: {}", $name, lp, e); break 'outer2; }
                        }
                    }};
                }
                macro_rules! rdarr2 {
                    ($t:ty, $p:expr, $name:expr) => {{
                        let cnt_pos = *$p;
                        let cnt = u32::read_from(&data, $p).unwrap();
                        eprintln!("  {} count={} [cnt_pos={}]", $name, cnt, cnt_pos);
                        if cnt > 10000 { eprintln!("  {} STOP: count too large", $name); break 'outer2; }
                        for i in 0..cnt {
                            <$t>::read_from(&data, $p).unwrap_or_else(|e| panic!("{}[{}]: {}", $name, i, e));
                        }
                    }};
                }
                rd2!(CArray<GimmickF20Elem>, p, "f20");
                rd2!(u8, p, "f21");
                rd2!(CArray<u32>, p, "f22");
                rd2!(CArray<u32>, p, "f23");
                rd2!(CArray<GimmickF24Elem>, p, "f24");
                rd2!(u64, p, "f25");
                rd2!([u8;7], p, "f26_32");
                rd2!(u32, p, "f33_a");
                rd2!(u8, p, "f33_b");
                rd2!(u8, p, "f33_c");
                rd2!(CArray<GimmickF34Elem>, p, "f34");
                rd2!(CArray<GimmickF35Elem>, p, "f35");
                rd2!(u8, p, "f36");
                rd2!(u32, p, "f37");
                rd2!(u32, p, "f38");
                rd2!(u32, p, "f39");
                rd2!([u8;2], p, "f40_41");
                rd2!(u32, p, "f42");
                rd2!(u8, p, "f43_flag");
                rdarr2!(u64, p, "f43_list");
                rd2!(u64, p, "f44");
                rd2!(u64, p, "f45");
                rd2!(COptional<GimmickF46Data>, p, "f46");
                rd2!([u32;3], p, "f47");
                rd2!(u32, p, "f48");
                rd2!(u32, p, "f49");
                rd2!(u32, p, "f50");
                rd2!(u8, p, "f51");
                rd2!(u32, p, "f52");
                rd2!(u32, p, "f53");
                rd2!(u32, p, "f54");
                rd2!(u32, p, "f55");
                rd2!(u32, p, "f56");
                rd2!([u32;3], p, "f57");
                rd2!(u32, p, "f58");
                rd2!(u32, p, "f59");
                rd2!(u32, p, "f60");
                rd2!(u32, p, "f61");
                rd2!(u8, p, "f61b");
                rd2!(u8, p, "f62");
                rd2!(u32, p, "f63");
                rd2!(u32, p, "f64");
                rd2!(u32, p, "f65");
                rd2!(u32, p, "f66");
                rd2!(u32, p, "f67");
                rd2!([u8;3], p, "f68_70");
                rd2!(u32, p, "f71");
                rd2!([u32;3], p, "f72");
                rd2!(u32, p, "f73");
                rd2!(u32, p, "f74");
                rdarr2!(GimmickF75Elem, p, "f75");
                rdarr2!(GimmickF76Elem, p, "f76");
                rd2!(COptional<GimmickF76Inner>, p, "f77");
                rd2!(CArray<GimmickF78Elem>, p, "f78");
                rd2!(CArray<GimmickF79Elem>, p, "f79");
                rd2!(CArray<u32>, p, "f80");
                rd2!(CArray<GimmickF81Elem>, p, "f81");
                rd2!(u32, p, "f82");
                rd2!(u32, p, "f83");
                rd2!([u8;2], p, "f84_85");
                rdc2!(p, "f86_str_a");
                rdc2!(p, "f86_str_b");
                rd2!(u32, p, "f86_a");
                rd2!(u32, p, "f86_b");
                rd2!(u32, p, "f86_c");
                rd2!(CArray<GimmickF87Elem>, p, "f87");
                rd2!(CArray<GimmickF88Elem>, p, "f88");
                rd2!(CArray<GimmickF89Elem>, p, "f89");
                rd2!(CArray<GimmickF90Elem>, p, "f90");
                rd2!(u32, p, "f91");
                rd2!(CArray<GimmickF92Elem>, p, "f92");
                rd2!(u32, p, "f93");
                rd2!(u32, p, "f94");
                rd2!(u32, p, "f95");
                rd2!(u32, p, "f96");
                rd2!(CArray<GimmickF97Elem>, p, "f97");
                rd2!(u8, p, "f98");
                rd2!(u32, p, "f99");
                rd2!(CArray<CString>, p, "f100");
                rd2!(CArray<u32>, p, "f101");
                rd2!([u8;2], p, "f102_103");
                rd2!(u16, p, "f104");
                rd2!(u16, p, "f105");
                rd2!([u8;10], p, "f106_115");
                rd2!(u32, p, "f116");
                rd2!(COptional<GimmickF117Data>, p, "f117");
                rd2!(u8, p, "f118");
                rd2!(CArray<GimmickF119Elem>, p, "f119");
                rd2!(u32, p, "f120");
                rd2!(u32, p, "f121");
                rd2!(u32, p, "f122");
                rd2!(u8, p, "f123");
                rdc2!(p, "f124");
                rd2!(CArray<GimmickF125Elem>, p, "f125");
                rd2!(CArray<GimmickF126Elem>, p, "f126");
                rd2!(CArray<GimmickF126Elem>, p, "f127");
                rd2!(CArray<GimmickF128Elem>, p, "f128");
                rd2!(CArray<GimmickF129Elem>, p, "f129");
                rd2!(CArray<GimmickF130Elem>, p, "f130");
                rd2!(u32, p, "f131");
                { let fp=*p; let fl=u8::read_from(&data,p).unwrap(); let fv=u64::read_from(&data,p).unwrap(); eprintln!("  f132.block_a flag={} val={} [off_before={}]",fl,fv,fp); rdc2!(p, "f132.block_a.name"); }
                { let fp=*p; let fl=u8::read_from(&data,p).unwrap(); let fv=u64::read_from(&data,p).unwrap(); eprintln!("  f132.block_b flag={} val={} [off_before={}]",fl,fv,fp); rdc2!(p, "f132.block_b.name"); }
                rd2!(u32, p, "f132.hash");
                rd2!(u32, p, "f132.val");
                { let cnt_pos=*p; let cnt=u32::read_from(&data,p).unwrap(); eprintln!("  f132.list_a_u32 count={} [cnt_pos={}]",cnt,cnt_pos); if cnt>10000 { eprintln!("  STOP"); break 'outer2; } for _i in 0..cnt { u32::read_from(&data,p).unwrap(); } }
                { let cnt_pos=*p; let cnt=u32::read_from(&data,p).unwrap(); eprintln!("  f132.list_a_264b count={} [cnt_pos={}]",cnt,cnt_pos); if cnt>10000 { eprintln!("  STOP"); break 'outer2; } for i in 0..cnt { GimmickDD420Elem::read_from(&data,p).unwrap_or_else(|e| panic!("list_a_264b[{}]: {}", i, e)); } }
                { let cnt_pos=*p; let cnt=u32::read_from(&data,p).unwrap(); eprintln!("  f132.list_b_u32 count={} [cnt_pos={}]",cnt,cnt_pos); if cnt>10000 { eprintln!("  STOP"); break 'outer2; } for _i in 0..cnt { u32::read_from(&data,p).unwrap(); } }
                { let cnt_pos=*p; let cnt=u32::read_from(&data,p).unwrap(); eprintln!("  f132.list_b_264b count={} [cnt_pos={}]",cnt,cnt_pos); if cnt>10000 { eprintln!("  STOP"); break 'outer2; } for i in 0..cnt { GimmickDD420Elem::read_from(&data,p).unwrap_or_else(|e| panic!("list_b_264b[{}]: {}", i, e)); } }
                eprintln!("  f132 done [off={}]", *p);
                rd2!(u32, p, "f133");
                rd2!(u8, p, "f134");
                rd2!(u32, p, "f135");
                rd2!([u8;3], p, "f136_138");
                rd2!(u32, p, "f139");
                rd2!(u32, p, "f140");
                rd2!(u32, p, "f141");
                rd2!([u8;3], p, "f142_144");
                rd2!(u32, p, "f145");
                rd2!(u32, p, "f146_a");
                rd2!(u32, p, "f146_b");
                rd2!(u32, p, "f146_c");
                rd2!(u8, p, "f146_d");
                rd2!(u8, p, "f146_e");
                rd2!(u16, p, "f147");
                rdarr2!(u16, p, "f148");
                rd2!(u8, p, "f149");
                rd2!(u16, p, "f150");
                rd2!(u16, p, "f151");
                rd2!([u8;4], p, "f152_155");
                let peek8: [u8;8] = data[*p..*p+8].try_into().unwrap();
                eprintln!("  bytes at off={}: {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x} {:02x}", *p, peek8[0],peek8[1],peek8[2],peek8[3],peek8[4],peek8[5],peek8[6],peek8[7]);
                rdc2!(p, "f154");
                rd2!([u8;9], p, "f155_163");
                rd2!(u32, p, "f164");
                rd2!(u64, p, "f165");
                rdarr2!(GimmickF75Elem, p, "f166");
                rdarr2!(GimmickF75Elem, p, "f167");
                rdarr2!(COptional<GimmickF168Inner>, p, "f168");
                rdarr2!(COptional<GimmickF168Inner>, p, "f169");
                // Dump 61 bytes after f169 for wire layout analysis
                let tail_start = *p;
                let tail_end = e;
                let tail_len = tail_end - tail_start;
                eprint!("  tail bytes[{}] at off={}: ", tail_len, tail_start);
                for i in 0..tail_len.min(61) { eprint!("{:02x} ", data[tail_start+i]); }
                eprintln!();
                rd2!(u32, p, "f170_a");
                rd2!(u64, p, "f170_b");
                rd2!(u32, p, "f170_c");
                rdarr2!(GimmickF170Elem, p, "f170_list");
                rd2!(u32, p, "f171");
                rd2!([u8;4], p, "f172_175");
                rd2!(u32, p, "f176");
                rd2!(u8, p, "f177");
                rd2!(u32, p, "f178");
                eprintln!("  ALL fields at off={}, entry_end={}", *p, e);
                break 'outer2;
            }
        }
    }

    #[test]
    fn variant_diag() {
        // Print grouping by prefab prefix for post_body=None entries.
        // Also prints the min post_body size for successful entries.
        let (data, pabgh_data) = load_or_skip!();
        let Some(entries) = load_pabgh_offsets_from_bytes(&pabgh_data) else { return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut counts: std::collections::BTreeMap<Vec<u8>, (usize, usize)> = Default::default();
        let mut success_min = usize::MAX;
        let mut success_max = 0usize;
        for (_k, s, e) in &ranges {
            let mut probe = *s;
            let item = GimmickInfo::read_with_size(&data, &mut probe, e - s).unwrap();
            match &item.tail {
                GimmickTail::Decoded { post_body: None, post_blob, .. } => {
                    let key: Vec<u8> = item.prefab_path.data.as_bytes()
                        .iter().take(32).copied().collect();
                    let entry = counts.entry(key).or_insert((0, 0));
                    entry.0 += 1;
                    entry.1 = entry.1.max(post_blob.len());
                }
                GimmickTail::Decoded { post_body: Some(_), post_blob, .. } => {
                    // post_blob is the bytes consumed by alt_trigger_list
                    // probe is at entry_end so post_body size = entry_end - post_body_start
                    // We don't have post_body_start directly, but post_blob.len()==0
                    // means alt_trigger_list + post_body consumed all remaining bytes.
                    // Approximate: record entry size as proxy.
                    let _ = post_blob;
                    // Use entry size minus known-fixed prefix overhead (~350 bytes)
                    // as a rough proxy for the post-blob region.
                    let total = e - s;
                    success_min = success_min.min(total);
                    success_max = success_max.max(total);
                }
                _ => {}
            }
        }
        eprintln!("--- failing variant groups ({} total) ---",
                  counts.values().map(|(n,_)| n).sum::<usize>());
        for (k, (count, max_blob)) in &counts {
            let s = std::str::from_utf8(k).unwrap_or("(invalid)");
            eprintln!("  count={:4}  max_blob={:5}  prefab_prefix={:?}", count, max_blob, s);
        }
        eprintln!("--- successful with_body entry sizes: min={} max={} ---", success_min, success_max);
    }

    #[test]
    fn generated_blob_diag() {
        // Findings (4-24 dump):
        //   - All 1833 generated__/pointcontrol entries have exactly 738-byte post_blobs.
        //   - 1713/1833 are bitwise identical (default config); 120 non-default from
        //     abyssislandpipe_0018_phase00_00.
        //   - Only bytes 281-284 vary across all entries: a small u32 (≤ 0x1b45) likely
        //     a spline segment ID. All non-default entries have this field set; default = 0.
        //   - Blob contains CString("fx_pc_weapon_exp_b__logout.system.effect") at bytes
        //     596-639 (length u32 at 596, data at 600).
        //   - 00_common/spl entries (5, max_blob=738) do NOT share this blob — different variant.
        //   - Decoding requires IDA: sub_1410E6FC0 dispatches to a different post-body reader
        //     for generated__/pointcontrol gimmick classes than the standard GimmickPostBody.
        let (data, pabgh_data) = load_or_skip!();
        let Some(entries) = load_pabgh_offsets_from_bytes(&pabgh_data) else { return; };
        let ranges = entry_ranges(&entries, data.len());

        let mut first_blob: Option<Vec<u8>> = None;
        let mut total = 0usize;
        let mut identical = 0usize;
        let mut size_mismatch = 0usize;
        let mut non_default_count = 0usize;

        for (_k, s, e) in &ranges {
            let mut probe = *s;
            let item = GimmickInfo::read_with_size(&data, &mut probe, e - s).unwrap();
            if !item.prefab_path.data.contains("/generated__/") { continue; }
            let GimmickTail::Decoded { post_body: None, post_blob, .. } = &item.tail else { continue; };
            total += 1;
            if post_blob.len() != 738 { size_mismatch += 1; continue; }
            let var = &post_blob[281..285];
            match &first_blob {
                None => { first_blob = Some(post_blob.clone()); identical = 1; }
                Some(fb) => { if post_blob == fb { identical += 1; } }
            }
            if var != [0u8, 0, 0, 0] { non_default_count += 1; }
        }
        eprintln!("generated__ total={} size_mismatch={} identical={} non_default_var_field={}",
                  total, size_mismatch, identical, non_default_count);
    }

    #[test]
    fn json_roundtrip() {
        let (data, pabgh_data) = load_or_skip!();
        let Some(entries) = load_pabgh_offsets_from_bytes(&pabgh_data) else { eprintln!("SKIP: bad pabgh"); return; };
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
