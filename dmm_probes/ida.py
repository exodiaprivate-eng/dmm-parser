#!/usr/bin/env python3
"""Reusable IDA-MCP JSON-RPC helper (HTTP server at :13337).

Usage:
  python ida.py strings <substr>        # list strings containing substr
  python ida.py xrefs <hexaddr>         # xrefs to address (0x...)
  python ida.py decompile <hexaddr>     # pseudocode
  python ida.py func <name-or-hexaddr>  # find function by name / decompile by addr
  python ida.py err <TABLE> <_field>    # NattKh: find the Korean error-string reader xref
ASCII-filters output (cp1252 consoles choke on Korean).
"""
import json, sys, urllib.request

URL = "http://localhost:13337/mcp"

def rpc(method, **params):
    req = urllib.request.Request(
        URL,
        data=json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).encode(),
        headers={"Content-Type": "application/json"},
    )
    return json.load(urllib.request.urlopen(req, timeout=120)).get("result")

def asc(s):
    return "".join(c if 32 <= ord(c) < 127 else "." for c in str(s))

def all_strings(filt=None, cap=600000):
    out, off = [], 0
    while off < cap:
        r = rpc("list_strings", offset=off, count=20000)
        data = r.get("data") if isinstance(r, dict) else r
        if not data:
            break
        for s in data:
            st = s.get("string", "")
            if filt is None or filt in st:
                out.append((s.get("address"), st))
        no = r.get("next_offset") if isinstance(r, dict) else None
        if no is None or no <= off:
            break
        off = no
    return out

def main():
    if len(sys.argv) < 2:
        print(__doc__); return
    cmd = sys.argv[1]
    if cmd == "strings":
        for a, s in all_strings(sys.argv[2]):
            print(f"{a} | {asc(s)[:100]}")
    elif cmd == "xrefs":
        r = rpc("get_xrefs_to", address=sys.argv[2])
        print(json.dumps(r, indent=2)[:4000])
    elif cmd == "decompile":
        r = rpc("decompile_function", address=sys.argv[2])
        code = r.get("pseudocode", r) if isinstance(r, dict) else r
        print(asc(code)[:12000])
    elif cmd == "func":
        arg = sys.argv[2]
        if arg.startswith("0x"):
            r = rpc("decompile_function", address=arg)
            code = r.get("pseudocode", r) if isinstance(r, dict) else r
            print(asc(code)[:12000])
        else:
            r = rpc("list_functions", offset=0, count=100000)
            fns = r.get("data") if isinstance(r, dict) else r
            for f in fns or []:
                if arg.lower() in str(f.get("name", "")).lower():
                    print(f.get("address"), f.get("name"))
    elif cmd == "err":
        table, field = sys.argv[2], sys.argv[3]
        # NattKh: error string "TABLE의 _FIELD를 읽어들이는데 실패했다"
        for a, s in all_strings(field):
            if table in s:
                print(f"STRING {a} | {asc(s)[:90]}")
                xr = rpc("get_xrefs_to", address=a)
                print(json.dumps(xr, indent=0)[:2000])

if __name__ == "__main__":
    main()
