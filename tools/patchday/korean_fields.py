"""NattKh method: dump the per-FIELD Korean error strings
   "<Table>의 _<field>를 읽어들이는데 실패했다"
to get each table's exact FIELD LIST + ORDER from the binary.

Reads the **Mac** (Apple-silicon) build — unstripped Mach-O, so the assert strings carry the
real field names in reader order. That is the whole reason the Mac binary is kept.

★ The binary is an ARGUMENT, not a constant. It was pinned to the 1.16 build, so running it
on patch day reported field names from three patches ago — a wrong answer that looks exactly
like a right one, which is the worst kind. Defaults to the newest version directory under the
IDA workspace, so it follows whatever builds are actually on disk.

Usage:
    python tools/patchday/korean_fields.py                  # newest build on disk
    python tools/patchday/korean_fields.py --list           # which builds are available
    python tools/patchday/korean_fields.py --bin <path>     # a specific binary
    python tools/patchday/korean_fields.py iteminfo         # filter to one table
"""
import argparse
import os
import re
import struct
import sys

IDA_WORKSPACE = os.path.join(
    os.path.expanduser("~"), "Desktop", "Project", "IDA Professional 9.0"
)
MAC_NAME = "CrimsonDesert_Steam.exe"


def _vkey(name):
    """Sort '1.18' and '2.0' the way a human means — '2.0' is NEWER than '1.18'."""
    return [int(c) if c.isdigit() else -1 for c in re.split(r"[._]", name)]


def available_builds():
    if not os.path.isdir(IDA_WORKSPACE):
        return []
    out = []
    for d in sorted(os.listdir(IDA_WORKSPACE)):
        p = os.path.join(IDA_WORKSPACE, d, MAC_NAME)
        if os.path.isfile(p) and d[:1].isdigit():
            out.append((d, p))
    return sorted(out, key=lambda t: _vkey(t[0]))


def newest_build():
    b = available_builds()
    return b[-1][1] if b else None

NEEDLE = "\uc77d\uc5b4\ub4e4\uc774\ub294\ub370".encode("utf-8")  # 읽어들이는데

def sections(data):
    ncmds = struct.unpack_from("<I", data, 16)[0]
    off, out = 32, []
    for _ in range(ncmds):
        cmd, cmdsize = struct.unpack_from("<II", data, off)
        if cmd == 0x19:
            nsects = struct.unpack_from("<I", data, off + 64)[0]
            so = off + 72
            for _i in range(nsects):
                addr, size = struct.unpack_from("<QQ", data, so + 32)
                foff = struct.unpack_from("<I", data, so + 48)[0]
                out.append((addr, size, foff))
                so += 80
        off += cmdsize
    return out

def f2v(secs, fo):
    for addr, size, foff in secs:
        if foff <= fo < foff + size:
            return addr + (fo - foff)
    return None

_ap = argparse.ArgumentParser()
_ap.add_argument("table_filter", nargs="?", default=None)
_ap.add_argument("--bin", dest="binary", default=None, help="Mach-O to scan")
_ap.add_argument("--list", action="store_true", help="list builds and exit")
_args = _ap.parse_args()

if _args.list:
    for _name, _path in available_builds():
        print(f"  {_name:<8} {os.path.getsize(_path):>13,}  {_path}")
    sys.exit(0)

BIN = _args.binary or newest_build()
if not BIN:
    sys.exit(f"no Mac binary under {IDA_WORKSPACE}/<version>/{MAC_NAME}")
print(f"binary: {BIN}")

data = open(BIN, "rb").read()
secs = sections(data)
print(f"binary {len(data):,} bytes; scanning for the field-error pattern...")

pat = re.compile(
    r"([A-Za-z0-9_]+)\uc758\s*(_[A-Za-z0-9_]+)\ub97c\s*\uc77d\uc5b4\ub4e4\uc774\ub294\ub370"
)

hits = []
for m in re.finditer(re.escape(NEEDLE), data):
    fo = m.start()
    lo = data.rfind(b"\x00", max(0, fo - 300), fo)
    lo = lo + 1 if lo != -1 else max(0, fo - 300)
    hi = data.find(b"\x00", fo)
    if hi == -1 or hi - lo > 400:
        continue
    try:
        s = data[lo:hi].decode("utf-8")
    except UnicodeDecodeError:
        continue
    mm = pat.search(s)
    if mm:
        hits.append((mm.group(1), mm.group(2), f2v(secs, lo), lo))

tables = {}
for tbl, fld, va, fo in hits:
    tables.setdefault(tbl, []).append((fld, va, fo))

print(f"found {len(hits)} field-error strings across {len(tables)} tables\n")
filt = _args.table_filter.lower() if _args.table_filter else None
shown = 0
for tbl in sorted(tables):
    if filt and filt not in tbl.lower():
        continue
    flds = tables[tbl]
    flds.sort(key=lambda x: x[2])
    print(f"=== {tbl} ({len(flds)} fields) ===")
    for fld, va, fo in flds:
        print(f"   {fld:46} {'0x%x' % va if va else ''}")
    print()
    shown += 1
if not shown:
    print("tables:", ", ".join(sorted(tables)))
