"""Find and decompile a table's READER function via its Korean error strings.

The bytes tell you WIDTH; only the reader tells you ORDER and the exact type of
each field. The reader is easy to find even when the type has no symbol: it is the
function that references that type's `"<Type>의 _<field>를 …실패했다"` strings.

    python ida_reader.py StoreInfo                 # locate + decompile the reader
    python ida_reader.py StoreInfo --field maxSlot # anchor on one field only
    python ida_reader.py StoreInfo --strings-only  # just the string VAs

Requires the IDA-MCP HTTP server on :13337 with the matching DB loaded
(check with get_metadata — 1.18 Mac md5 9c1407310266528408e30c418e8b1d96).
"""
import argparse
import collections
import json
import struct
import subprocess
import sys

EXE = r"C:\Users\justi\Desktop\Project\IDA Professional 9.0\1.18\CrimsonDesert_Steam.exe"
URL = "http://localhost:13337/mcp"


def rpc(method, params):
    body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params})
    out = subprocess.run(
        ["curl", "-s", "-m", "90", "-X", "POST", URL,
         "-H", "Content-Type: application/json", "-d", body],
        capture_output=True)
    try:
        return json.loads(out.stdout.decode("utf-8", "replace")).get("result")
    except json.JSONDecodeError:
        print("!! bad RPC reply:", out.stdout[:200])
        sys.exit(1)


def segments(d):
    ncmds = struct.unpack_from("<I", d, 16)[0]
    off, segs = 32, []
    for _ in range(ncmds):
        cmd, sz = struct.unpack_from("<II", d, off)
        if cmd == 0x19:                                  # LC_SEGMENT_64
            vmaddr, _vmsize, fileoff, filesize = struct.unpack_from("<QQQQ", d, off + 24)
            segs.append((vmaddr, fileoff, filesize))
        off += sz
    return segs


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("type_name")
    ap.add_argument("--field")
    ap.add_argument("--exe", default=EXE)
    ap.add_argument("--strings-only", action="store_true")
    a = ap.parse_args()

    d = open(a.exe, "rb").read()
    segs = segments(d)

    def va(fo):
        for vmaddr, fileoff, filesize in segs:
            if fileoff <= fo < fileoff + filesize:
                return vmaddr + (fo - fileoff)
        return None

    needle = (a.type_name + "의 _").encode("utf-8")
    hits, i = [], 0
    while True:
        i = d.find(needle, i)
        if i < 0:
            break
        end = d.index(b"\x00", i)
        text = d[i:end].decode("utf-8", "replace")
        field = text.split("의 _", 1)[1].split("를", 1)[0]
        hits.append((field, va(i)))
        i = end
    if not hits:
        print(f"no reader strings for {a.type_name!r}")
        sys.exit(1)
    print(f"{len(hits)} field string(s) for {a.type_name} "
          f"(ADDRESS order — NOT field order):")
    for f, v in hits:
        print(f"   {v:#x}  {f}")
    if a.strings_only:
        return

    # Which function references them? The one they (almost all) share is the reader.
    fns = collections.Counter()
    for f, v in hits:
        if a.field and a.field.lower() not in f.lower():
            continue
        for x in rpc("get_xrefs_to", {"address": hex(v)}) or []:
            fn = x.get("function") or {}
            if fn.get("address"):
                fns[(fn["address"], fn.get("name", "?"))] += 1
    if not fns:
        print("\nno xrefs — the DB may not have analysed these, or wrong DB loaded")
        return
    print("\ncandidate readers (by how many of this type's strings they use):")
    for (addr, name), c in fns.most_common(5):
        print(f"   {addr}  {name}   x{c}")

    addr, name = fns.most_common(1)[0][0]
    print(f"\n=== decompile {name} @ {addr} ===\n")
    src = rpc("decompile_function", {"address": addr}) or "(none)"
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    print(src)


if __name__ == "__main__":
    main()
