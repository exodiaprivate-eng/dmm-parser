//! Tier 1 — fully typed (no _tail_b64).
//!
//! Reader: `sub_1410E2100` in CrimsonDesert.exe (Win build).
//! Inner sub-readers (all decoded for the Tier 1.5 → 1 promotion):
//!   - sub_141103B30 (COptional<GameCondition>) — see
//!     `crate::binary::optional_game_condition`
//!   - sub_1410E25F0 (32-byte fixed EffectData; 31 wire bytes)
//!   - sub_1411144B0 → sub_1410E2740 (COptional<184-byte WeatherData>;
//!     45×u32 + 1×u8 = 181 wire bytes per WeatherData)
//!   - sub_1410E2E10 (88-byte fixed PostProcessData; 22×u32 = 88 wire bytes)
//!
//! Wire reads, in order (canonical names from Mac Korean error strings):
//!   1.  u16 key                                    (_key, pabgh format 2)
//!   2.  CString string_key                         (_stringKey)
//!   3.  u8 is_blocked                              (_isBlocked)
//!   4.  OptionalGameCondition condition            (_condition)
//!   5.  u32 projectile_key                         (_projectileKey)
//!   6.  u32 projectile_shot_key                    (_projectileShotKey)
//!   7.  u32 projectile_chase_physics_material_hash (_projectileChasePhysicsMaterialHash)
//!   8.  [u8; 12] projectile_shot_spread            (_projectileShotSpread)
//!   9.  [u8; 8] projectile_shot_interval           (_projectileShotInterval)
//!  10.  u32 projectile_height_offset               (_projectileHeightOffset)
//!  11.  u32 projectile_create_delay_time           (_projectileCreateDelayTime)
//!  12.  u8 projectile_hit_rate                     (_projectileHitRate)
//!  13.  u8 projectile_shot_count                   (_projectileShotCount)
//!  14.  Option<EffectData> effect_data             (_effectData,
//!       inline u8 presence + 32-byte struct)
//!  15.  Option<WeatherData> weather_data           (_weatherData,
//!       sub_1411144B0 = u8 presence + 184-byte struct)
//!  16.  Option<PostProcessData> post_process_data  (_postProcessData,
//!       inline u8 presence + 88-byte struct)
//!  17.  u8 is_advanced                             (_isAdvanced)

use crate::binary::optional_game_condition::OptionalGameCondition;
use crate::binary::*;
use crate::json_traits::{ToJsonValue, WriteJsonValue, get_field as json_get_field};
use crate::py_binary_struct;
use serde_json::{Map, Value};
use std::io::{self, Write};

py_binary_struct! {
    /// 32-byte fixed struct (31 wire bytes + 1 byte padding).
    /// Per sub_1410E25F0:
    ///   u32 lookup (read_u32_lookup_DA30 → qword_145F0DA30)
    ///   + 8 bytes raw + 8 bytes raw + 8 bytes raw + 3 trailing bytes.
    pub struct EffectData {
        pub lookup: u32,
        pub raw_a: [u8; 8],
        pub raw_b: [u8; 8],
        pub raw_c: [u8; 8],
        pub flag_a: u8,
        pub flag_b: u8,
        pub flag_c: u8,
    }
}

py_binary_struct! {
    /// 184-byte fixed struct (181 wire bytes + 3 padding).
    /// Per sub_1410E2740: 45 sequential u32 reads + 1 trailing u8.
    pub struct WeatherData {
        pub field_00: u32, pub field_04: u32, pub field_08: u32, pub field_0c: u32,
        pub field_10: u32, pub field_14: u32, pub field_18: u32, pub field_1c: u32,
        pub field_20: u32, pub field_24: u32, pub field_28: u32, pub field_2c: u32,
        pub field_30: u32, pub field_34: u32, pub field_38: u32, pub field_3c: u32,
        pub field_40: u32, pub field_44: u32, pub field_48: u32, pub field_4c: u32,
        pub field_50: u32, pub field_54: u32, pub field_58: u32, pub field_5c: u32,
        pub field_60: u32, pub field_64: u32, pub field_68: u32, pub field_6c: u32,
        pub field_70: u32, pub field_74: u32, pub field_78: u32, pub field_7c: u32,
        pub field_80: u32, pub field_84: u32, pub field_88: u32, pub field_8c: u32,
        pub field_90: u32, pub field_94: u32, pub field_98: u32, pub field_9c: u32,
        pub field_a0: u32, pub field_a4: u32, pub field_a8: u32, pub field_ac: u32,
        pub field_b0: u32,
        pub flag_b4: u8,
    }
}

py_binary_struct! {
    /// 88-byte fixed struct (22 sequential u32 reads).
    /// Per sub_1410E2E10.
    pub struct PostProcessData {
        pub field_00: u32, pub field_04: u32, pub field_08: u32, pub field_0c: u32,
        pub field_10: u32, pub field_14: u32, pub field_18: u32, pub field_1c: u32,
        pub field_20: u32, pub field_24: u32, pub field_28: u32, pub field_2c: u32,
        pub field_30: u32, pub field_34: u32, pub field_38: u32, pub field_3c: u32,
        pub field_40: u32, pub field_44: u32, pub field_48: u32, pub field_4c: u32,
        pub field_50: u32, pub field_54: u32,
    }
}

/// Stream-mode `u8 presence + (if presence: T)` wrapper for fixed-size T.
#[derive(Debug)]
pub struct OptionalFixed<T> {
    pub inner: Option<T>,
}

impl<'a, T: BinaryRead<'a> + BinaryWrite + ToJsonValue + WriteJsonValue> OptionalFixed<T> {
    pub fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let presence = u8::read_from(data, offset)?;
        let inner = if presence != 0 { Some(T::read_from(data, offset)?) } else { None };
        Ok(Self { inner })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        match &self.inner {
            Some(v) => {
                1u8.write_to(w)?;
                v.write_to(w)
            }
            None => 0u8.write_to(w),
        }
    }

    pub fn to_json_value(&self) -> Value {
        match &self.inner {
            Some(v) => v.to_json_value(),
            None => Value::Null,
        }
    }

    pub fn write_from_json(w: &mut Vec<u8>, v: &Value) -> io::Result<()> {
        if v.is_null() {
            w.push(0);
            Ok(())
        } else {
            w.push(1);
            T::write_from_json(w, v)
        }
    }
}

#[derive(Debug)]
pub struct GameGlobalEffectInfo<'a> {
    pub key: u16,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub condition: OptionalGameCondition<'a>,
    pub projectile_key: u32,
    pub projectile_shot_key: u32,
    pub projectile_chase_physics_material_hash: u32,
    pub projectile_shot_spread: [u8; 12],
    pub projectile_shot_interval: [u8; 8],
    pub projectile_height_offset: u32,
    pub projectile_create_delay_time: u32,
    pub projectile_hit_rate: u8,
    pub projectile_shot_count: u8,
    pub effect_data: OptionalFixed<EffectData>,
    pub weather_data: OptionalFixed<WeatherData>,
    pub post_process_data: OptionalFixed<PostProcessData>,
    pub is_advanced: u8,
}

impl<'a> GameGlobalEffectInfo<'a> {
    pub fn read_with_size(data: &'a [u8], offset: &mut usize, entry_size: usize) -> io::Result<Self> {
        let start = *offset;
        let item = Self::read_from(data, offset)?;
        let consumed = *offset - start;
        if consumed != entry_size {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("GameGlobalEffectInfo: consumed {} bytes, expected {}", consumed, entry_size)));
        }
        Ok(item)
    }

    pub fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let key = u16::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;
        let condition = OptionalGameCondition::read_from(data, offset)?;
        let projectile_key = u32::read_from(data, offset)?;
        let projectile_shot_key = u32::read_from(data, offset)?;
        let projectile_chase_physics_material_hash = u32::read_from(data, offset)?;
        let projectile_shot_spread = <[u8; 12]>::read_from(data, offset)?;
        let projectile_shot_interval = <[u8; 8]>::read_from(data, offset)?;
        let projectile_height_offset = u32::read_from(data, offset)?;
        let projectile_create_delay_time = u32::read_from(data, offset)?;
        let projectile_hit_rate = u8::read_from(data, offset)?;
        let projectile_shot_count = u8::read_from(data, offset)?;
        let effect_data = OptionalFixed::<EffectData>::read_from(data, offset)?;
        let weather_data = OptionalFixed::<WeatherData>::read_from(data, offset)?;
        let post_process_data = OptionalFixed::<PostProcessData>::read_from(data, offset)?;
        let is_advanced = u8::read_from(data, offset)?;
        Ok(Self {
            key, string_key, is_blocked, condition,
            projectile_key, projectile_shot_key, projectile_chase_physics_material_hash,
            projectile_shot_spread, projectile_shot_interval,
            projectile_height_offset, projectile_create_delay_time,
            projectile_hit_rate, projectile_shot_count,
            effect_data, weather_data, post_process_data, is_advanced,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.condition.write_to(w)?;
        self.projectile_key.write_to(w)?;
        self.projectile_shot_key.write_to(w)?;
        self.projectile_chase_physics_material_hash.write_to(w)?;
        self.projectile_shot_spread.write_to(w)?;
        self.projectile_shot_interval.write_to(w)?;
        self.projectile_height_offset.write_to(w)?;
        self.projectile_create_delay_time.write_to(w)?;
        self.projectile_hit_rate.write_to(w)?;
        self.projectile_shot_count.write_to(w)?;
        self.effect_data.write_to(w)?;
        self.weather_data.write_to(w)?;
        self.post_process_data.write_to(w)?;
        self.is_advanced.write_to(w)?;
        Ok(())
    }

    pub fn to_json_dict(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("key".to_string(), self.key.to_json_value());
        m.insert("string_key".to_string(), self.string_key.to_json_value());
        m.insert("is_blocked".to_string(), self.is_blocked.to_json_value());
        m.insert("condition".to_string(), self.condition.to_json_value());
        m.insert("projectile_key".to_string(), self.projectile_key.to_json_value());
        m.insert("projectile_shot_key".to_string(), self.projectile_shot_key.to_json_value());
        m.insert("projectile_chase_physics_material_hash".to_string(), self.projectile_chase_physics_material_hash.to_json_value());
        m.insert("projectile_shot_spread".to_string(), self.projectile_shot_spread.to_json_value());
        m.insert("projectile_shot_interval".to_string(), self.projectile_shot_interval.to_json_value());
        m.insert("projectile_height_offset".to_string(), self.projectile_height_offset.to_json_value());
        m.insert("projectile_create_delay_time".to_string(), self.projectile_create_delay_time.to_json_value());
        m.insert("projectile_hit_rate".to_string(), self.projectile_hit_rate.to_json_value());
        m.insert("projectile_shot_count".to_string(), self.projectile_shot_count.to_json_value());
        m.insert("effect_data".to_string(), self.effect_data.to_json_value());
        m.insert("weather_data".to_string(), self.weather_data.to_json_value());
        m.insert("post_process_data".to_string(), self.post_process_data.to_json_value());
        m.insert("is_advanced".to_string(), self.is_advanced.to_json_value());
        m
    }

    pub fn write_from_json_dict(w: &mut Vec<u8>, obj: &Map<String, Value>) -> io::Result<()> {
        <u16 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "key")?)?;
        <CString as WriteJsonValue>::write_from_json(w, json_get_field(obj, "string_key")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_blocked")?)?;
        OptionalGameCondition::write_from_json(w, json_get_field(obj, "condition")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "projectile_key")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "projectile_shot_key")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "projectile_chase_physics_material_hash")?)?;
        <[u8; 12] as WriteJsonValue>::write_from_json(w, json_get_field(obj, "projectile_shot_spread")?)?;
        <[u8; 8] as WriteJsonValue>::write_from_json(w, json_get_field(obj, "projectile_shot_interval")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "projectile_height_offset")?)?;
        <u32 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "projectile_create_delay_time")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "projectile_hit_rate")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "projectile_shot_count")?)?;
        OptionalFixed::<EffectData>::write_from_json(w, json_get_field(obj, "effect_data")?)?;
        OptionalFixed::<WeatherData>::write_from_json(w, json_get_field(obj, "weather_data")?)?;
        OptionalFixed::<PostProcessData>::write_from_json(w, json_get_field(obj, "post_process_data")?)?;
        <u8 as WriteJsonValue>::write_from_json(w, json_get_field(obj, "is_advanced")?)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};
    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gameglobaleffectinfo.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\gameglobaleffectinfo.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            let item = GameGlobalEffectInfo::read_from(&data, &mut c)
                .unwrap_or_else(|er| panic!("e{} k=0x{:x}: {}", i, k, er));
            assert_eq!(c, *e, "e{} k=0x{:x}: under/over-read {}/{}", i, k, c - s, e - s);
            items.push(item);
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "gameglobaleffectinfo roundtrip mismatch");
    }

    #[test]
    fn json_roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        for (i, (key, start, end)) in ranges.iter().enumerate() {
            let mut cursor = *start;
            let item = GameGlobalEffectInfo::read_from(&data, &mut cursor).unwrap();
            assert_eq!(cursor, *end, "entry {} key=0x{:x}: under/over-read", i, key);
            let dict = item.to_json_dict();
            let mut from_typed = Vec::new();
            item.write_to(&mut from_typed).unwrap();
            let mut from_json = Vec::new();
            GameGlobalEffectInfo::write_from_json_dict(&mut from_json, &dict)
                .unwrap_or_else(|e| panic!("entry {} key=0x{:x}: write_from_json_dict: {}", i, key, e));
            assert_eq!(
                from_json, from_typed,
                "entry {} key=0x{:x}: JSON round-trip diverges from typed write", i, key
            );
        }
    }

    #[test]
    fn fields_addressable() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let Some((_, s, _)) = ranges.first() else { eprintln!("SKIP: no entries"); return; };
        let mut c = *s;
        let item = GameGlobalEffectInfo::read_from(&data, &mut c).unwrap();
        let dict = item.to_json_dict();
        for f in [
            "key", "string_key", "is_blocked", "condition",
            "projectile_key", "projectile_shot_key",
            "projectile_chase_physics_material_hash",
            "projectile_shot_spread", "projectile_shot_interval",
            "projectile_height_offset", "projectile_create_delay_time",
            "projectile_hit_rate", "projectile_shot_count",
            "effect_data", "weather_data", "post_process_data",
            "is_advanced",
        ] {
            assert!(dict.contains_key(f), "missing field `{}` in JSON dict", f);
        }
        assert!(!dict.contains_key("_tail_b64"), "Tier 1.5 _tail_b64 leaked");
    }
}
