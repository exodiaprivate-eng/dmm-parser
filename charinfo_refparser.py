#!/usr/bin/env python3
# CharacterInfo reference parser. Walks a record via the deserializer's TRUE
# source-order read list (charinfo_reads.json) + reader widths (charinfo_widths.json).
# WORKFLOW (loop): run -> stops at first UNKNOWN reader -> decompile that sub_ in IDA
# (width = vtable ,N size arg; resolved StaticInfoWrapper keys are 4B wire unless the
# inner reader's size arg says 2; CArray readers loop -> add to CARR with elem wire
# size; CString=sub_100D39448; LocalizableString=sub_100D5D6D8) -> add to
# charinfo_widths.json (FIX or CARR) -> rerun. When key=1 FULLY WALKS, run --all to
# verify across records. Then port ordered field list to Rust info.rs.
import json, struct, sys
B=r'C:\Users\Coding\CrimsonDesertModding\dmm-parser-src'
P=r'C:\Users\Coding\CrimsonDesertModding\extractedpaz\0008_112'
seq=json.load(open(B+r'\charinfo_reads.json'))
W=json.load(open(B+r'\charinfo_widths.json')); FIX=W['FIX']; CARR=W['CARR']
def consume(data,p,fn,sz=0):
    if fn=='INLINEREAD': return p+sz
    # CharacterInfoOverridableFlags struct: aliveSkillInfoList CArray<{SkillKey u32,u32}>(8B) + 4 u8
    if fn=='sub_101F95934':
        c=struct.unpack_from('<I',data,p)[0]
        if c>100000: raise ValueError(f'ovf count{c}')
        return p+4+c*8+4
    if fn in FIX: return p+FIX[fn]
    if fn=='sub_100D39448': return p+4+struct.unpack_from('<I',data,p)[0]
    if fn=='sub_100D5D6D8': return p+1+8+4+struct.unpack_from('<I',data,p+9)[0]
    if fn in CARR:
        c=struct.unpack_from('<I',data,p)[0]
        if c>100000: raise ValueError(f'count{c}')
        return p+4+c*CARR[fn]
    return None
def records():
    pb=open(P+r'\characterinfo.pabgb','rb').read(); ph=open(P+r'\characterinfo.pabgh','rb').read()
    e=[]; q=2
    while q+8<=len(ph): e.append((struct.unpack_from('<I',ph,q)[0],struct.unpack_from('<I',ph,q+4)[0])); q+=8
    mono=[e[0]]
    for k,o in e[1:]:
        if o>=mono[-1][1] and o<=len(pb): mono.append((k,o))
        else: break
    out=[]
    for i in range(len(mono)-1): out.append((mono[i][0], pb[mono[i][1]:mono[i+1][1]]))
    out.append((mono[-1][0], pb[mono[-1][1]:]))
    return out
def walk(data):
    p=0
    for i,rec in enumerate(seq):
        fn=rec[0]; off=rec[1]; arg=rec[2]; sz=rec[3] if len(rec)>3 else 0
        try: np=consume(data,p,fn,sz)
        except Exception as ex: return ('ERR',i,fn,arg,p,str(ex))
        if np is None: return ('UNK',i,fn,arg,p,None)
        if np>len(data): return ('OVR',i,fn,arg,p,None)
        p=np
    return ('OK',len(seq),None,None,p,None)
if __name__=='__main__':
    recs=records()
    if '--all' in sys.argv:
        ok=0; stops={}
        for k,d in recs:
            r=walk(d)
            if r[0]=='OK' and r[4]==len(d): ok+=1
            else: stops[(r[0],r[1],r[2])]=stops.get((r[0],r[1],r[2]),0)+1
        print(f"FULLY WALKED {ok}/{len(recs)} records")
        for s,c in sorted(stops.items(),key=lambda x:-x[1])[:5]: print("  stop",s,"x",c)
    else:
        k,d=recs[0]; r=walk(d)
        if r[0]=='OK' and r[4]==len(d): print(f"key={k} FULLY WALKED, consumed {r[4]}/{len(d)} -- run --all")
        else: print(f"key={k} STOP {r[0]} idx{r[1]} {r[2]} arg={r[3]} wire@{r[4]} {r[5] or ''} (of {len(d)})")
