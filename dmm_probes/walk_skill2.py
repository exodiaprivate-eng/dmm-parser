#!/usr/bin/env python3
"""Forward-decode SkillInfo post-buff fields for e245, scanning candidate
buff_level_list end offsets to find the one that consumes cleanly to the
record end (166143). That tells us the TRUE SummonBuffData size."""
import os, struct

D = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-6-11"
data = open(os.path.join(D, "skill.pabgb"), "rb").read()
REC_END = 166143

class R:
    def __init__(s, b, o): s.b=b; s.o=o
    def u8(s): v=s.b[s.o]; s.o+=1; return v
    def u16(s): v=struct.unpack_from("<H",s.b,s.o)[0]; s.o+=2; return v
    def u32(s): v=struct.unpack_from("<I",s.b,s.o)[0]; s.o+=4; return v
    def u64(s): v=struct.unpack_from("<Q",s.b,s.o)[0]; s.o+=8; return v
    def cstr(s):
        ln=s.u32()
        if ln>100000 or s.o+ln>len(s.b): raise ValueError(f"bad cstr len {ln} @{s.o-4}")
        v=s.b[s.o:s.o+ln]; s.o+=ln; return v
    def carr(s, esz, name):
        n=s.u32()
        if n>100000: raise ValueError(f"bad {name} count {n} @{s.o-4}")
        s.o += n*esz
        return n
    def graph(s): s.o += 28

def decode_post_buff(buff_end, verbose=False):
    r = R(data, buff_end)
    log = []
    def L(name):
        if verbose: log.append(f"  @{r.o} before {name}")
    r.u32()  # skill_group_key
    r.u32()  # parent_skill
    r.u32()  # learn_level
    r.u8()   # apply_type
    r.u32()  # icon_path
    r.u32()  # need_upgrade_item_info
    r.graph(); r.graph()
    r.carr(4, "usable_character_info_list")
    r.carr(4, "usable_condition")
    r.u32(); r.u32()  # learn_knowledge_info, faction_info
    r.carr(22, "use_resource_stat_list")
    r.carr(12, "use_resource_item_list")
    r.carr(22, "use_driver_resource_stat_list")
    r.u64()  # use_battery_stat
    for _ in range(6): r.u8()  # 6 flag bytes
    r.carr(4, "reserve_slot_info_list")
    r.u32()  # max_level
    r.carr(2, "skill_group_key_list")
    r.u32()  # buff_sustain_flag
    r.cstr() # dev_skill_name
    r.cstr() # dev_skill_desc
    rem = REC_END - r.o
    return rem, r.o

# Our parser ends buff at 165935. Scan a window around it.
print("buff_end -> (remaining-after-2-devstrings, offset). want remaining==4 (video_path) and clean")
for be in range(165930, 165950):
    try:
        rem, end = decode_post_buff(be)
        flag = "  <== CLEAN (rem==4 video_path)" if rem == 4 else ""
        print(f"buff_end={be}: rem={rem} end_off={end}{flag}")
    except Exception as ex:
        print(f"buff_end={be}: ERR {ex}")
