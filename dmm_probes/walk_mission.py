import os,struct
D=r'C:\temp\GIT\CrimsonDesertUpdates\pabgb\2026-6-11'
data=open(os.path.join(D,'missioninfo.pabgb'),'rb').read()
ph=open(os.path.join(D,'missioninfo.pabgh'),'rb').read()
def parse_pabgh(b):
    c16=struct.unpack_from('<H',b,0)[0]
    if 2+c16*8==len(b): idx,cnt,ks,es=2,c16,4,8
    elif 2+c16*6==len(b): idx,cnt,ks,es=2,c16,2,6
    elif 2+c16*5==len(b): idx,cnt,ks,es=2,c16,1,5
    else: idx,cnt,ks,es=4,struct.unpack_from('<I',b,0)[0],4,8
    out=[]
    for i in range(cnt):
        pos=idx+i*es
        key=b[pos] if ks==1 else struct.unpack_from('<H',b,pos)[0] if ks==2 else struct.unpack_from('<I',b,pos)[0]
        out.append((key,struct.unpack_from('<I',b,pos+ks)[0]))
    return out
ents=parse_pabgh(ph); ents.sort(key=lambda e:e[1])
ranges=[(ents[i][0],ents[i][1],ents[i+1][1] if i+1<len(ents) else len(data)) for i in range(len(ents))]
class R:
    def __init__(s,o,end): s.o=o; s.end=end
    def u8(s): v=data[s.o]; s.o+=1; return v
    def u16(s): v=struct.unpack_from('<H',data,s.o)[0]; s.o+=2; return v
    def u32(s): v=struct.unpack_from('<I',data,s.o)[0]; s.o+=4; return v
    def u64(s): s.o+=8
    def f3(s): s.o+=12
    def quat(s): s.o+=16
    def cstr(s):
        n=s.u32()
        if n>5000 or s.o+n>len(data): raise ValueError(f'cstr len {n} @{s.o-4}')
        s.o+=n
    def carr(s,esz,name):
        n=s.u32()
        if n>20000: raise ValueError(f'{name} count {n} @{s.o-4}')
        if callable(esz):
            for _ in range(n): esz(s)
        else:
            s.o+=n*esz
def branch(s): s.o+=19
def exe(s): s.o+=18
def resdata(s):  # MissionResultData
    s.u8(); s.carr(4,'rd.list'); s.o+=28; s.u8(); s.u8()
def tvol(s):  # TriggerVolumeData
    s.u8(); s.o+=40; s.cstr(); s.cstr(); s.u8(); s.f3(); s.f3(); s.u8(); s.u8()
def uidesc(s):
    s.o+=12+12; s.carr(4,'ui.la'); s.carr(4,'ui.lb'); s.u32(); s.f3(); s.u32(); s.u32(); s.o+=6; s.u16()
def loc(s): s.u8(); s.u64(); s.cstr()
def rd2(s):
    s.u8(); s.cstr(); s.cstr(); s.u16(); s.u16(); s.u8(); s.cstr(); s.carr(4,'rd2.a'); s.carr(4,'rd2.b'); s.carr(4,'rd2.c'); s.u64(); s.u8()
def stage(s):
    s.u8(); s.u32(); s.u32(); s.u32(); s.carr(4,'st.list'); s.u32(); s.u16(); s.u8(); s.u8()
def walk(s,verbose=False):
    def L(n): 
        if verbose: print(f'  @{s.o-s0} ({s.o}) after {n}')
    s0=s.o
    s.u32(); s.cstr(); s.u8(); s.u32(); L('parent_quest')
    s.carr(4,'sub_mission'); L('sub_mission')
    s.carr(branch,'branch'); L('branch')
    s.carr(exe,'execute'); L('execute')
    s.carr(4,'start_player'); s.carr(4,'field_revive'); s.carr(4,'giveup'); L('revive lists')
    if s.u8()!=0: tvol(s)
    L('trigger_volume')
    s.carr(4,'reward'); s.carr(resdata,'result_data'); L('result_data')
    s.u16(); uidesc(s); L('ui_desc')
    loc(s); loc(s); loc(s); loc(s); L('labels')
    s.u32(); s.carr(rd2,'result_data_2'); L('result_data_2')
    s.carr(stage,'mission_stage'); L('mission_stage')
    s.u32(); s.u16(); s.u16(); s.u32(); s.o+=13; s.u32(); L('tail')
    return s.o
k,start,end=ranges[0]
print(f'e0 k=0x{k:x} start={start} end={end} size={end-start}')
try:
    o=walk(R(start,end),verbose=True)
    print(f'consumed to {o}, entry_end {end}, diff {end-o}')
except Exception as ex:
    print(f'FAIL: {ex}')
