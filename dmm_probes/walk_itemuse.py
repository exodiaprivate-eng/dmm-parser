#!/usr/bin/env python3
"""Data-driven wirewalk for itemuseinfo.pabgb (1.11). Decode base, group
residual (post-base) bytes by disc to infer each variant's extra layout."""
import struct, os
from collections import defaultdict, Counter

D = r"C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-6-11"
data = open(os.path.join(D, "itemuseinfo.pabgb"), "rb").read()
ph = open(os.path.join(D, "itemuseinfo.pabgh"), "rb").read()
cnt = struct.unpack_from("<I", ph, 0)[0]
ents = sorted(((struct.unpack_from("<II", ph, 4 + i*8)) for i in range(cnt)), key=lambda e: e[1])

class R:
    def __init__(s, b): s.b=b; s.o=0
    def u8(s): v=s.b[s.o]; s.o+=1; return v
    def u32(s): v=struct.unpack_from("<I",s.b,s.o)[0]; s.o+=4; return v
    def u64(s): v=struct.unpack_from("<Q",s.b,s.o)[0]; s.o+=8; return v
    def cstr(s):
        ln=s.u32()
        if ln>10000 or s.o+ln>len(s.b): raise ValueError(f"bad cstr len {ln} @{s.o}")
        v=s.b[s.o:s.o+ln]; s.o+=ln; return v
    def left(s): return len(s.b)-s.o

def read_elem(r):
    # BaseUseDataElem hypothesis: key u32 + flag u8 + LocalizableString(cat u8+index u64+CString) + trailing u32
    r.u32()            # key_lookup
    r.u8()             # flag
    r.u8()             # category
    r.u64()            # index
    r.cstr()           # default string
    r.u32()            # TRAILING u32 (hypothesis)

def read_base(r):
    # 18 fixed: 4*u8 + 2*u32 + 2*u8 + u32   then CArray<elem>
    for _ in range(4): r.u8()
    r.u32(); r.u32()
    r.u8(); r.u8()
    r.u32()
    n = r.u32()
    if n > 100000: raise ValueError(f"bad elem count {n}")
    for _ in range(n): read_elem(r)
    return n

residual_by_disc = defaultdict(list)
disc_count = Counter()
base_fail = 0
for i in range(cnt):
    o = ents[i][1]; e = ents[i+1][1] if i+1 < cnt else len(data)
    b = data[o:e]
    r = R(b)
    try:
        r.u32()                # key
        r.cstr()               # string_key
        r.u8()                 # is_blocked
        disc = r.u8()          # disc
        disc_count[disc]+=1
        base_start = r.o
        n = read_base(r)
        residual = r.left()
        residual_by_disc[disc].append((residual, b[r.o:].hex()[:64], i))
    except Exception as ex:
        base_fail += 1
        if base_fail <= 8:
            print(f"BASE FAIL rec{i} key=0x{ents[i][0]:x} disc={b[r.o-1] if r.o>0 else '?'}: {ex}")

print(f"\nbase_fail={base_fail}/{cnt}")
print("\n=== residual (post-base bytes) distribution per disc ===")
for disc in sorted(residual_by_disc):
    res = residual_by_disc[disc]
    rc = Counter(x[0] for x in res)
    print(f"disc {disc:2d} ({len(res)} recs): residual sizes -> {dict(sorted(rc.items()))}")
    # show a sample residual for the most common nonzero size
    nz = [x for x in res if x[0]>0]
    if nz:
        print(f"        sample residual: {nz[0][1]}  (rec{nz[0][2]}, {nz[0][0]}B)")
