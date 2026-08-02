"""
1.16 table-schema oracle — the NattKh method, adapted to the Mac binary.

For a table:
  1. field NAMES + order  <- Korean per-field error strings
  2. reader function      <- get_xrefs_to(first field string)
  3. field SEQUENCE       <- decompile reader, take reader-calls in order,
                             each carrying its destination MEM OFFSET (a2 + N)
  4. pair names <-> calls -> per-field (name, reader_fn, mem_offset)

Mem offsets are the in-memory struct layout, not wire sizes, but the DELTAS
pin field boundaries and the reader identity pins the wire type (one decompile
per distinct reader, cached in READERS below).

Usage: python schema116.py <TableName>
"""
import json, re, struct, subprocess, sys, os, urllib.request

BIN = r"C:\Users\justi\Desktop\Project\IDA Professional 9.0\1.16\CrimsonDesert_Steam.exe"

def ida(method, **params):
    req = urllib.request.Request(os.environ.get("IDA_MCP_URL", "http://localhost:%s/mcp" % os.environ.get("IDA_MCP_PORT", "13337")),
        data=json.dumps({"jsonrpc":"2.0","id":1,"method":method,"params":params}).encode(),
        headers={"Content-Type":"application/json"})
    r = json.load(urllib.request.urlopen(req, timeout=180))
    if "error" in r: raise RuntimeError(r["error"])
    return r["result"]

# ---- 1. field names in order -------------------------------------------------
def sections(d):
    ncmds=struct.unpack_from("<I",d,16)[0]; off=32; out=[]
    for _ in range(ncmds):
        cmd,cmdsize=struct.unpack_from("<II",d,off)
        if cmd==0x19:
            n=struct.unpack_from("<I",d,off+64)[0]; so=off+72
            for _i in range(n):
                addr,size=struct.unpack_from("<QQ",d,so+32)
                foff=struct.unpack_from("<I",d,so+48)[0]
                out.append((addr,size,foff)); so+=80
        off+=cmdsize
    return out

_PAT = re.compile(r"([A-Za-z0-9_]+)\uc758\s*(_[A-Za-z0-9_]+)\ub97c\s*\uc77d\uc5b4\ub4e4\uc774\ub294\ub370")

def field_names(table):
    d=open(BIN,"rb").read(); secs=sections(d)
    needle="\uc77d\uc5b4\ub4e4\uc774\ub294\ub370".encode("utf-8")
    hits=[]
    for m in re.finditer(re.escape(needle), d):
        fo=m.start()
        lo=d.rfind(b"\x00", max(0,fo-300), fo); lo=lo+1 if lo!=-1 else max(0,fo-300)
        hi=d.find(b"\x00", fo)
        if hi==-1 or hi-lo>400: continue
        try: s=d[lo:hi].decode("utf-8")
        except UnicodeDecodeError: continue
        mm=_PAT.search(s)
        if mm and mm.group(1)==table:
            va=None
            for a,sz,f in secs:
                if f<=lo<f+sz: va=a+(lo-f); break
            hits.append((mm.group(2), va, lo))
    hits.sort(key=lambda x:x[2])
    return hits

# ---- 2/3. reader + call sequence ---------------------------------------------
_CALL = re.compile(
    r'(?:(sub_[0-9A-Fa-f]+)|\)\s*)\(\s*a1\s*,\s*a2\s*(?:\+\s*(\d+))?\s*(?:,\s*(\d+)LL\s*)?\)')

def call_seq(pseudo):
    flat = re.sub(r'/\*[^*]*\*/', ' ', pseudo)      # strip /* line: N */ markers
    flat = re.sub(r'\s+', ' ', flat)                 # join continuations
    out=[]
    for m in _CALL.finditer(flat):
        fn = m.group(1) or "<vtable>"
        out.append((fn, int(m.group(2) or 0), int(m.group(3)) if m.group(3) else None))
    return out

def reader_for(field_va):
    xs = ida("get_xrefs_to", address=hex(field_va))
    for x in xs:
        f = x.get("function") or {}
        if f.get("name"): return f["name"], f["address"]
    return None, None

def schema(table):
    names = field_names(table)
    if not names: raise SystemExit(f"no Korean field strings for {table!r}")
    fn, addr = reader_for(names[0][1])
    if not fn: raise SystemExit("no reader xref")
    pseudo = ida("decompile_function", address=addr)
    if isinstance(pseudo, dict): pseudo = pseudo.get("result","")
    seq = call_seq(pseudo)
    return names, fn, addr, seq, pseudo

if __name__=="__main__":
    t=sys.argv[1]
    names, fn, addr, seq, pseudo = schema(t)
    open(f"_{t}_reader.txt","w",encoding="utf-8").write(pseudo)
    print(f"{t}: {len(names)} fields | reader {fn} @ {addr} | {len(seq)} reader-calls\n")
    print(f"{'#':>3} {'FIELD':<42} {'READER':<18} {'MEM':>6} {'d':>5}")
    prev=None
    for i in range(max(len(names), len(seq))):
        nm = names[i][0] if i < len(names) else "-"
        if i < len(seq):
            f,off,w = seq[i]
            d = "" if prev is None else str(off-prev); prev=off
            print(f"{i:>3} {nm:<42} {f:<18} {off:>6} {d:>5}" + (f"  w={w}" if w else ""))
        else:
            print(f"{i:>3} {nm:<42} {'-':<18}")
