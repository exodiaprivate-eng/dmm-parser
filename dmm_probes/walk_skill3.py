#!/usr/bin/env python3
"""For each failing skill record, scan its tail for the buff_level_list end
offset that yields a clean post-buff decode (rem==4 video_path). Report the
12 bytes ending at that offset, to confirm SummonBuffData is +N and inspect
the trailing bytes."""
import os, struct

D = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-6-11"
data = open(os.path.join(D, "skill.pabgb"), "rb").read()
ph = open(os.path.join(D, "skill.pabgh"), "rb").read()

def parse_pabgh(b):
    c16 = struct.unpack_from("<H", b, 0)[0]; c32 = struct.unpack_from("<I", b, 0)[0]
    if 2+c16*8==len(b): idx,cnt,ks,es=2,c16,4,8
    elif 2+c16*6==len(b): idx,cnt,ks,es=2,c16,2,6
    elif 2+c16*5==len(b): idx,cnt,ks,es=2,c16,1,5
    elif 4+c32*8==len(b): idx,cnt,ks,es=4,c32,4,8
    else: raise ValueError("fmt")
    out=[]
    for i in range(cnt):
        pos=idx+i*es
        key=b[pos] if ks==1 else struct.unpack_from("<H",b,pos)[0] if ks==2 else struct.unpack_from("<I",b,pos)[0]
        off=struct.unpack_from("<I",b,pos+ks)[0]; out.append((key,off))
    return out

ents=parse_pabgh(ph); ents.sort(key=lambda e:e[1])
ranges=[(ents[i][0],ents[i][1],ents[i+1][1] if i+1<len(ents) else len(data)) for i in range(len(ents))]
rmap={k:(s,e) for k,s,e in ranges}

class R:
    def __init__(s,b,o): s.b=b; s.o=o
    def u8(s): v=s.b[s.o]; s.o+=1; return v
    def u32(s): v=struct.unpack_from("<I",s.b,s.o)[0]; s.o+=4; return v
    def u64(s): v=struct.unpack_from("<Q",s.b,s.o)[0]; s.o+=8; return v
    def cstr(s):
        ln=s.u32()
        if ln>100000 or s.o+ln>len(s.b): raise ValueError("cstr")
        s.o+=ln
    def carr(s,esz):
        n=s.u32()
        if n>100000: raise ValueError("carr")
        s.o+=n*esz

def post_buff(buff_end, rec_end):
    r=R(data,buff_end)
    r.u32();r.u32();r.u32();r.u8();r.u32();r.u32()  # group,parent,learn,apply,icon,need
    r.o+=28;r.o+=28                                  # 2 graphs
    r.carr(4);r.carr(4)                              # usable_char, usable_cond
    r.u32();r.u32()                                  # learn_knowledge, faction
    r.carr(22);r.carr(12);r.carr(22)                 # res_stat, res_item, driver_stat
    r.u64()                                          # battery
    r.o+=6                                           # 6 flags
    r.carr(4);r.u32();r.carr(2);r.u32()              # reserve, maxlvl, grpkeylist, sustain
    r.cstr();r.cstr()                                # dev name, desc
    return rec_end - r.o

for k in [0x1250b,0xfdf1,0xfdf2,0xa18b,0xa18d,0xa18e,0x16473]:
    s,e=rmap[k]
    found=[]
    for be in range(s+30, e-4):
        try:
            if post_buff(be,e)==4: found.append(be)
        except Exception: pass
    for be in found:
        pre=data[be-12:be].hex()
        nxt=data[be:be+8].hex()
        print(f"k=0x{k:<6x} size={e-s:<5} buff_end={be}  pre12={pre}  next8={nxt}")
    if not found:
        print(f"k=0x{k:<6x} size={e-s:<5} NO CLEAN buff_end FOUND")
