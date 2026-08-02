"""NattKh method: dump the per-FIELD Korean error strings
   "<Table>의 _<field>를 읽어들이는데 실패했다"
to get each table's exact FIELD LIST + ORDER from the binary.
Usage: python korean_fields.py [table_filter]"""
import re
import struct
import sys

BIN = r"C:\Users\justi\Desktop\Project\IDA Professional 9.0\1.16\CrimsonDesert_Steam.exe"
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
filt = sys.argv[1].lower() if len(sys.argv) > 1 else None
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
