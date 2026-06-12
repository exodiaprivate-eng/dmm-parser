#!/usr/bin/env python3
"""Localize skill.pabgb 1.11 drift. Parse pabgh like parse_pabgh_inline,
find e245 range, and walk SkillInfo header + buff_level_list framing."""
import os, struct

D = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-6-11"
data = open(os.path.join(D, "skill.pabgb"), "rb").read()
ph = open(os.path.join(D, "skill.pabgh"), "rb").read()

def parse_pabgh(b):
    c16 = struct.unpack_from("<H", b, 0)[0]
    c32 = struct.unpack_from("<I", b, 0)[0]
    if 2 + c16*8 == len(b): idx,cnt,ks,es = 2,c16,4,8
    elif 2 + c16*6 == len(b): idx,cnt,ks,es = 2,c16,2,6
    elif 2 + c16*5 == len(b): idx,cnt,ks,es = 2,c16,1,5
    elif 4 + c32*8 == len(b): idx,cnt,ks,es = 4,c32,4,8
    else: raise ValueError("unknown pabgh format")
    out=[]
    for i in range(cnt):
        pos = idx + i*es
        if ks==1: key=b[pos]
        elif ks==2: key=struct.unpack_from("<H",b,pos)[0]
        else: key=struct.unpack_from("<I",b,pos)[0]
        off=struct.unpack_from("<I",b,pos+ks)[0]
        out.append((key,off))
    return out

ents = parse_pabgh(ph)
ents.sort(key=lambda e: e[1])
ranges = [(ents[i][0], ents[i][1], ents[i+1][1] if i+1<len(ents) else len(data)) for i in range(len(ents))]

# find the records that fail
fail_keys = {0x1250b, 0xfdf1, 0xfdf2, 0xa18b, 0xa18d, 0xa18e, 0x16473}
for i,(k,s,e) in enumerate(ranges):
    if k in fail_keys:
        print(f"e{i} k=0x{k:x}  start={s} end={e} size={e-s}")

def dump(lo, hi):
    for o in range(lo, hi, 16):
        chunk = data[o:o+16]
        hexs = " ".join(f"{b:02x}" for b in chunk)
        asci = "".join(chr(b) if 32 <= b < 127 else "." for b in chunk)
        print(f"{o:7d}  {hexs:<48}  {asci}")

# locate e245 (key 0x1250b)
for i,(k,s,e) in enumerate(ranges):
    if k == 0x1250b:
        print(f"\n=== e{i} k=0x1250b record start..head (first 80 bytes) ===")
        dump(s, s+80)
        break
