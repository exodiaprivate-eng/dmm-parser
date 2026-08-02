"""Classify a PA table-reader sub_ into a wire-width class by decompiling it.
FIX n      : single vtable read of n bytes
FWD fn     : thin wrapper -> forwards to another reader
CSTR/LOCSTR: string readers
LOOP       : has a count + loop (CArray) -> needs element size
COMPLEX    : anything else"""
import json, re, sys, urllib.request
def ida(method, **p):
    r=urllib.request.Request("http://localhost:13337/mcp",
        data=json.dumps({"jsonrpc":"2.0","id":1,"method":method,"params":p}).encode(),
        headers={"Content-Type":"application/json"})
    j=json.load(urllib.request.urlopen(r,timeout=180))
    if "error" in j: raise RuntimeError(j["error"])
    return j["result"]
def strip(t): return re.sub(r'/\*[^*]*\*/','',t)
def classify(fn):
    addr="0x"+fn.split("_")[1].lower()
    try: t=ida("decompile_function", address=addr)
    except Exception as e: return ("ERR", str(e)[:40], "")
    if isinstance(t,dict): t=t.get("result","")
    s=strip(t); flat=re.sub(r'\s+',' ',s)
    body=flat.split("{",1)[1] if "{" in flat else flat
    # single vtable read: return (*(...))(a1, a2, N LL);
    m=re.search(r'return \(\*\([^)]*\)\(\*\(_QWORD \*\)a1 \+ 16LL\)\)\(a1, a2, (\d+)LL\);\s*\}', flat)
    if m: return ("FIX", int(m.group(1)), t)
    # thin forwarder: return sub_X(a1, a2);
    m=re.search(r'return (sub_[0-9A-Fa-f]+)\(a1, a2\);\s*\}', flat)
    if m: return ("FWD", m.group(1), t)
    nvt=len(re.findall(r'\(a1, [^,]+, (\d+)LL\)', flat))
    has_loop = bool(re.search(r'\bwhile\b|\bfor \(', flat))
    if has_loop: return ("LOOP", nvt, t)
    return ("COMPLEX", nvt, t)
if __name__=="__main__":
    fns=json.load(open(sys.argv[1]))
    out={}
    for fn in fns:
        k,v,_=classify(fn)
        out[fn]=[k,v]
        print(f"{fn:<20} {k:<8} {v}")
    json.dump(out, open(sys.argv[2],"w"), indent=1)
