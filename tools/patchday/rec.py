import struct, os
def _hdr(h):
    c16=struct.unpack_from("<H",h,0)[0]; c32=struct.unpack_from("<I",h,0)[0]
    for idx,cnt,ks,es in ((2,c16,4,8),(2,c16,2,6),(2,c16,1,5),(4,c32,4,8)):
        if idx+cnt*es==len(h): return idx,cnt,ks,es
    raise ValueError("unknown pabgh format")
def recs(D, base):
    d=open(os.path.join(D,base+".pabgb"),"rb").read()
    h=open(os.path.join(D,base+".pabgh"),"rb").read()
    idx,cnt,ks,es=_hdr(h)
    ents=[]
    for i in range(cnt):
        p=idx+i*es
        k=int.from_bytes(h[p:p+ks],"little"); off=struct.unpack_from("<I",h,p+ks)[0]
        ents.append((k,off))
    ents.sort(key=lambda e:e[1])
    return [(k,d[off:(ents[i+1][1] if i+1<cnt else len(d))]) for i,(k,off) in enumerate(ents)]
