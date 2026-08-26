# -*- coding: utf-8 -*-
"""Recover a table's TRUE wire field order from the binary.

`korean_fields.py` tells you which fields a table has. It lists them in string
order, which is usually read order but is not the same thing — and a patch that
MOVES a field is exactly what string order fails to notice, because the record
stays the same length and only the values come out wrong.

This asks the code instead. Each `<Table>의 _<field>를 읽어들이는데 실패했다`
string is referenced from the one place that reports that field failing to read,
so sorting fields by the ADDRESS of the instruction referencing them gives the
order the reader actually reads them in.

Worked example — GamePlayTriggerInfo on 2.00.00, where `_isEnable` moved from
index 5 to index 14 and `_useTriggerEvent` was appended after it:

    0x10204ab40  _prefabName
    0x10204ab4c  _isEnable          <- moved here from position 5
    0x10204ab58  _useTriggerEvent   <- new

Needs IDA open on the matching binary with the IDA-MCP server on
http://localhost:13337 (see the ida-mcp-access memo). Read-only: it calls
get_metadata and get_xrefs_to, nothing else.

Usage:
    python tools/patchday/field_order.py GamePlayTriggerInfo
    python tools/patchday/field_order.py QuestInfo MissionInfo --bin <path>
"""
import argparse
import json
import os
import re
import struct
import sys
import urllib.error
import urllib.request

# Self-contained on purpose: korean_fields.py does its work at module scope, so
# importing it would run its CLI. The Mach-O walk below is small enough to carry.
IDA_WORKSPACE = os.path.join(
    os.path.expanduser("~"), "Desktop", "Project", "IDA Professional 9.0"
)
MAC_NAME = "CrimsonDesert_Steam.exe"
NEEDLE = "읽어들이는데".encode("utf-8")  # 읽어들이는데
PAT = re.compile(
    r"([A-Za-z0-9_]+)의\s*(_[A-Za-z0-9_]+)를\s*읽어들이는데"
)
MCP = "http://localhost:13337/mcp"


def _vkey(name):
    """'2.0' is NEWER than '1.18' — compare component-wise, not as text."""
    return [int(c) if c.isdigit() else -1 for c in re.split(r"[._]", name)]


def newest_build():
    if not os.path.isdir(IDA_WORKSPACE):
        return None
    cands = [
        (d, os.path.join(IDA_WORKSPACE, d, MAC_NAME))
        for d in os.listdir(IDA_WORKSPACE)
        if d[:1].isdigit() and os.path.isfile(os.path.join(IDA_WORKSPACE, d, MAC_NAME))
    ]
    return sorted(cands, key=lambda t: _vkey(t[0]))[-1][1] if cands else None


def _sections(data):
    ncmds = struct.unpack_from("<I", data, 16)[0]
    off, out = 32, []
    for _ in range(ncmds):
        cmd, cmdsize = struct.unpack_from("<II", data, off)
        if cmd == 0x19:  # LC_SEGMENT_64
            nsects = struct.unpack_from("<I", data, off + 64)[0]
            so = off + 72
            for _i in range(nsects):
                addr, size = struct.unpack_from("<QQ", data, so + 32)
                foff = struct.unpack_from("<I", data, so + 48)[0]
                out.append((addr, size, foff))
                so += 80
        off += cmdsize
    return out


def harvest_with_addrs(path):
    """{owner: [(field, string_va)]} — the VA is what get_xrefs_to needs."""
    data = open(path, "rb").read()
    secs = _sections(data)
    out = {}
    for m in re.finditer(re.escape(NEEDLE), data):
        fo = m.start()
        lo = data.rfind(b"\x00", max(0, fo - 300), fo)
        lo = lo + 1 if lo != -1 else max(0, fo - 300)
        hi = data.find(b"\x00", fo)
        if hi == -1 or hi - lo > 400:
            continue
        try:
            text = data[lo:hi].decode("utf-8")
        except UnicodeDecodeError:
            continue
        mm = PAT.search(text)
        if not mm:
            continue
        va = None
        for addr, size, foff in secs:
            if foff <= lo < foff + size:
                va = addr + (lo - foff)
                break
        if va is not None:
            out.setdefault(mm.group(1), []).append((mm.group(2), va))
    return out


def rpc(method, params):
    req = urllib.request.Request(
        MCP,
        data=json.dumps(
            {"jsonrpc": "2.0", "id": 1, "method": method, "params": params}
        ).encode(),
        headers={"Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=60) as r:
        body = json.load(r)
    if "error" in body:
        raise RuntimeError(body["error"])
    return body.get("result")


def code_ref(addr):
    """Address of the instruction referencing this string, or None.

    A field with no xref is not an error: not every assert survives inlining.
    Those are reported as unplaced rather than dropped, because silently
    omitting a field is how an order reconstruction goes quietly wrong.
    """
    try:
        refs = rpc("get_xrefs_to", {"address": "0x%x" % addr})
    except (urllib.error.URLError, RuntimeError, TimeoutError, OSError):
        return None
    if not refs:
        return None
    return min(int(r["address"], 16) for r in refs)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("tables", nargs="+")
    ap.add_argument("--bin", dest="binary", default=None,
                    help="Mach-O to read the strings from (default: newest)")
    args = ap.parse_args()

    try:
        meta = rpc("get_metadata", {})
    except Exception as e:  # noqa: BLE001 - the message is the whole point
        print(f"IDA-MCP unreachable at {MCP}: {e}", file=sys.stderr)
        print("Open the binary in IDA and confirm '[MCP] Server started'.",
              file=sys.stderr)
        return 2

    binary = args.binary or newest_build()
    if not binary:
        print(f"no Mac binary under {IDA_WORKSPACE}/<version>/{MAC_NAME}",
              file=sys.stderr)
        return 2

    print(f"IDA has:      {meta['path']}  (md5 {meta['md5']})")
    print(f"strings from: {binary}")
    if os.path.basename(os.path.dirname(binary)) not in meta["path"]:
        print("⚠ these may be different builds — addresses will not line up.")
    print()

    owners = harvest_with_addrs(binary)

    for table in args.tables:
        fields = owners.get(table)
        if not fields:
            near = [t for t in owners if table.lower() in t.lower()]
            print(f"=== {table}: no such owner"
                  + (f" — did you mean {', '.join(near[:4])}?" if near else ""))
            continue

        placed, unplaced = [], []
        for name, saddr in fields:
            ref = code_ref(saddr)
            (placed if ref is not None else unplaced).append((ref, name))
        placed.sort()

        print(f"=== {table}  ({len(placed)} placed"
              + (f", {len(unplaced)} UNPLACED" if unplaced else "") + ") ===")
        for i, (ref, name) in enumerate(placed):
            print(f"  {i:3d}  0x{ref:x}  {name}")
        for _, name in unplaced:
            print(f"    ?  {'(no xref)':12s}  {name}   — place by hand")
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
