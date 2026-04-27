//! `Skill.pabgb` (SkillInfo) — fully field-decoded, **1952/1952 (100%) entries
//! round-trip byte-perfect**.
//!
//! Per IDA `sub_1410F8940` (1819 bytes), found via Korean error string
//! `'SkillInfo의 _allowSkillWithLowResource를 읽어들이는데 실패했다.'`.
//! All 34 fields per Mac symbols mapped.
//!
//! BuffData wrapper is `[u8 absent_flag][BuffData if absent_flag == 0]` per
//! sub_1419D9C70 (NO leading u32 like BuffInfo's BuffDataEntry). The 120-variant
//! BuffData family is shared with BuffInfo (same sub_1419D8B50 base reader,
//! same sub_1419D4FC0 allocator) — uses the existing
//! `binary::variants::buff_data::BuffData`. SummonBuffData (tag 10, previously
//! opaque) is now fully typed — see binary/variants/buff_data.rs.
//!
//! Helper sub-readers (decompiled from IDA):
//!   - GraphData (sub_141E2BB80):       8+8+8+4 = 28 stream bytes
//!   - ResourceStat (sub_1410F8830):    1+4+1+8+4+4 = 22 stream bytes
//!   - u32 lookups (sub_1410FF890, sub_1411077F0, sub_141107A20): u32 count + N×u32
//!   - u16 raw (sub_1410FEA90):         u32 count + N×u16
//!   - ResourceStat list (sub_141107900): u32 count + N×ResourceStat
//!   - read_u32_lookup_DA30:            u32 (4 stream bytes)
//!   - read_u32_lookup_DA10:            u32 (4 stream bytes)

use crate::binary::variants::buff_data::BuffData;
use crate::binary::*;
use std::io::{self, Write};

/// `[u8 absent_flag][BuffData if absent_flag == 0]` per sub_1419D9C70.
/// 1 = absent (skip), 0 = present (read BuffData). Inverted from typical COptional.
#[derive(Debug)]
pub struct BuffDataOptional<'a> {
    pub absent_flag: u8,
    pub data: Option<BuffData<'a>>,
}

impl<'a> BinaryRead<'a> for BuffDataOptional<'a> {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let absent_flag = u8::read_from(data, offset)?;
        let payload = if absent_flag == 0 {
            Some(BuffData::read_from(data, offset)?)
        } else {
            None
        };
        Ok(Self { absent_flag, data: payload })
    }
}

impl<'a> BinaryWrite for BuffDataOptional<'a> {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.absent_flag.write_to(w)?;
        if let Some(d) = &self.data {
            d.write_to(w)?;
        }
        Ok(())
    }
}

/// 28-byte GraphData per sub_141E2BB80: u64 + u64 + u64 + u32.
#[derive(Debug)]
pub struct GraphData {
    pub a: u64,
    pub b: u64,
    pub c: u64,
    pub d: u32,
}

impl<'a> BinaryRead<'a> for GraphData {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let a = u64::read_from(data, offset)?;
        let b = u64::read_from(data, offset)?;
        let c = u64::read_from(data, offset)?;
        let d = u32::read_from(data, offset)?;
        Ok(Self { a, b, c, d })
    }
}

impl BinaryWrite for GraphData {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.a.write_to(w)?;
        self.b.write_to(w)?;
        self.c.write_to(w)?;
        self.d.write_to(w)?;
        Ok(())
    }
}

/// 22-byte ResourceStat per sub_1410F8830: u8 + u32 + u8 + u64 + u32 + u32.
#[derive(Debug)]
pub struct ResourceStat {
    pub a: u8,
    pub lookup_b: u32,
    pub c: u8,
    pub d: u64,
    pub lookup_e: u32,
    pub lookup_f: u32,
}

impl<'a> BinaryRead<'a> for ResourceStat {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let a = u8::read_from(data, offset)?;
        let lookup_b = u32::read_from(data, offset)?;
        let c = u8::read_from(data, offset)?;
        let d = u64::read_from(data, offset)?;
        let lookup_e = u32::read_from(data, offset)?;
        let lookup_f = u32::read_from(data, offset)?;
        Ok(Self { a, lookup_b, c, d, lookup_e, lookup_f })
    }
}

impl BinaryWrite for ResourceStat {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.a.write_to(w)?;
        self.lookup_b.write_to(w)?;
        self.c.write_to(w)?;
        self.d.write_to(w)?;
        self.lookup_e.write_to(w)?;
        self.lookup_f.write_to(w)?;
        Ok(())
    }
}

/// 12-byte ResourceItem per inline loop in sub_1410F8940: u32 lookup + u64.
#[derive(Debug)]
pub struct ResourceItem {
    pub lookup: u32,
    pub value: u64,
}

impl<'a> BinaryRead<'a> for ResourceItem {
    fn read_from(data: &'a [u8], offset: &mut usize) -> io::Result<Self> {
        let lookup = u32::read_from(data, offset)?;
        let value = u64::read_from(data, offset)?;
        Ok(Self { lookup, value })
    }
}

impl BinaryWrite for ResourceItem {
    fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.lookup.write_to(w)?;
        self.value.write_to(w)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct SkillInfo<'a> {
    pub key: u32,
    pub string_key: CString<'a>,
    pub is_blocked: u8,
    pub cooltime: u32,
    pub buff_level_list: CArray<CArray<BuffDataOptional<'a>>>,
    pub skill_group_key: u32,
    pub parent_skill: u32,
    pub learn_level: u32,
    pub apply_type: u8,
    pub icon_path: u32,
    pub need_upgrade_item_info: u32,
    pub need_upgrade_item_count_graph: GraphData,
    pub need_upgrade_experience_graph: GraphData,
    pub usable_character_info_list: CArray<u32>,
    pub usable_condition: CArray<u32>,
    pub learn_knowledge_info: u32,
    pub faction_info: u32,
    pub use_resource_stat_list: CArray<ResourceStat>,
    pub use_resource_item_list: CArray<ResourceItem>,
    pub use_driver_resource_stat_list: CArray<ResourceStat>,
    pub use_battery_stat: u64,
    pub is_ui_use_allowed: u8,
    pub is_learn_use_artifact: u8,
    pub allow_skill_with_low_resource: u8,
    pub is_use_child_pattern_description_buff_data: u8,
    pub damage_type: u8,
    pub ui_type: u8,
    pub reserve_slot_info_list: CArray<u32>,
    pub max_level: u32,
    pub skill_group_key_list: CArray<u16>,
    pub buff_sustain_flag: u32,
    pub dev_skill_name: CString<'a>,
    pub dev_skill_desc: CString<'a>,
    pub video_path: u32,
}

impl<'a> SkillInfo<'a> {
    pub fn read_with_size(
        data: &'a [u8],
        offset: &mut usize,
        _entry_size: usize,
    ) -> io::Result<Self> {
        let key = u32::read_from(data, offset)?;
        let string_key = CString::read_from(data, offset)?;
        let is_blocked = u8::read_from(data, offset)?;
        let cooltime = u32::read_from(data, offset)?;
        let buff_level_list = CArray::<CArray<BuffDataOptional>>::read_from(data, offset)?;
        let skill_group_key = u32::read_from(data, offset)?;
        let parent_skill = u32::read_from(data, offset)?;
        let learn_level = u32::read_from(data, offset)?;
        let apply_type = u8::read_from(data, offset)?;
        let icon_path = u32::read_from(data, offset)?;
        let need_upgrade_item_info = u32::read_from(data, offset)?;
        let need_upgrade_item_count_graph = GraphData::read_from(data, offset)?;
        let need_upgrade_experience_graph = GraphData::read_from(data, offset)?;
        let usable_character_info_list = CArray::<u32>::read_from(data, offset)?;
        let usable_condition = CArray::<u32>::read_from(data, offset)?;
        let learn_knowledge_info = u32::read_from(data, offset)?;
        let faction_info = u32::read_from(data, offset)?;
        let use_resource_stat_list = CArray::<ResourceStat>::read_from(data, offset)?;
        let use_resource_item_list = CArray::<ResourceItem>::read_from(data, offset)?;
        let use_driver_resource_stat_list = CArray::<ResourceStat>::read_from(data, offset)?;
        let use_battery_stat = u64::read_from(data, offset)?;
        let is_ui_use_allowed = u8::read_from(data, offset)?;
        let is_learn_use_artifact = u8::read_from(data, offset)?;
        let allow_skill_with_low_resource = u8::read_from(data, offset)?;
        let is_use_child_pattern_description_buff_data = u8::read_from(data, offset)?;
        let damage_type = u8::read_from(data, offset)?;
        let ui_type = u8::read_from(data, offset)?;
        let reserve_slot_info_list = CArray::<u32>::read_from(data, offset)?;
        let max_level = u32::read_from(data, offset)?;
        let skill_group_key_list = CArray::<u16>::read_from(data, offset)?;
        let buff_sustain_flag = u32::read_from(data, offset)?;
        let dev_skill_name = CString::read_from(data, offset)?;
        let dev_skill_desc = CString::read_from(data, offset)?;
        let video_path = u32::read_from(data, offset)?;

        Ok(Self {
            key, string_key, is_blocked, cooltime, buff_level_list,
            skill_group_key, parent_skill, learn_level, apply_type,
            icon_path, need_upgrade_item_info,
            need_upgrade_item_count_graph, need_upgrade_experience_graph,
            usable_character_info_list, usable_condition,
            learn_knowledge_info, faction_info,
            use_resource_stat_list, use_resource_item_list, use_driver_resource_stat_list,
            use_battery_stat, is_ui_use_allowed, is_learn_use_artifact,
            allow_skill_with_low_resource, is_use_child_pattern_description_buff_data,
            damage_type, ui_type,
            reserve_slot_info_list, max_level, skill_group_key_list,
            buff_sustain_flag, dev_skill_name, dev_skill_desc, video_path,
        })
    }

    pub fn write_to(&self, w: &mut dyn Write) -> io::Result<()> {
        self.key.write_to(w)?;
        self.string_key.write_to(w)?;
        self.is_blocked.write_to(w)?;
        self.cooltime.write_to(w)?;
        self.buff_level_list.write_to(w)?;
        self.skill_group_key.write_to(w)?;
        self.parent_skill.write_to(w)?;
        self.learn_level.write_to(w)?;
        self.apply_type.write_to(w)?;
        self.icon_path.write_to(w)?;
        self.need_upgrade_item_info.write_to(w)?;
        self.need_upgrade_item_count_graph.write_to(w)?;
        self.need_upgrade_experience_graph.write_to(w)?;
        self.usable_character_info_list.write_to(w)?;
        self.usable_condition.write_to(w)?;
        self.learn_knowledge_info.write_to(w)?;
        self.faction_info.write_to(w)?;
        self.use_resource_stat_list.write_to(w)?;
        self.use_resource_item_list.write_to(w)?;
        self.use_driver_resource_stat_list.write_to(w)?;
        self.use_battery_stat.write_to(w)?;
        self.is_ui_use_allowed.write_to(w)?;
        self.is_learn_use_artifact.write_to(w)?;
        self.allow_skill_with_low_resource.write_to(w)?;
        self.is_use_child_pattern_description_buff_data.write_to(w)?;
        self.damage_type.write_to(w)?;
        self.ui_type.write_to(w)?;
        self.reserve_slot_info_list.write_to(w)?;
        self.max_level.write_to(w)?;
        self.skill_group_key_list.write_to(w)?;
        self.buff_sustain_flag.write_to(w)?;
        self.dev_skill_name.write_to(w)?;
        self.dev_skill_desc.write_to(w)?;
        self.video_path.write_to(w)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::variant::{entry_ranges, load_pabgh_offsets};

    const PABGB: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\skill.pabgb";
    const PABGH: &str = r"C:\Users\corin\Desktop\CD DUMPING TOOLS\dmm-pabgb-aio\vanilla_dumps\skill.pabgh";

    #[test]
    fn roundtrip() {
        let Ok(data) = std::fs::read(PABGB) else { eprintln!("SKIP"); return; };
        let Some(entries) = load_pabgh_offsets(PABGH) else { eprintln!("SKIP"); return; };
        let ranges = entry_ranges(&entries, data.len());
        let mut items = Vec::new();
        let mut pass = 0;
        let mut fails: Vec<String> = Vec::new();
        for (i, (k, s, e)) in ranges.iter().enumerate() {
            let mut c = *s;
            match SkillInfo::read_with_size(&data, &mut c, e - s) {
                Ok(item) => {
                    if c == *e {
                        items.push(item);
                        pass += 1;
                    } else {
                        fails.push(format!("e{} k=0x{:x}: under/over-consumed {}/{}", i, k, c - *s, e - *s));
                    }
                }
                Err(err) => {
                    fails.push(format!("e{} k=0x{:x}: {}", i, k, err));
                }
            }
        }
        if !fails.is_empty() {
            panic!("SkillInfo roundtrip: {} pass, {} fail (total {}).\nFailures:\n  {}",
                pass, fails.len(), ranges.len(), fails.join("\n  "));
        }
        let mut out = Vec::with_capacity(data.len());
        for it in &items { it.write_to(&mut out).unwrap(); }
        assert_eq!(out, data, "SkillInfo roundtrip bytes mismatch");
    }
}
