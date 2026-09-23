"""Write tools/hashes_<ver>_baseline.json for the installed game: sha256 + size of every
group file (NNNN/0.pamt, NNNN/0.paz, ...), meta/*, and the exe. The per-mod builders gate on
this file so they never read a mounted game as vanilla.

Refuses a mounted game (dmm* folders present), same rule as prepatch_capture.py.
    python tools/patchday/hash_baseline.py            # version read from meta/0.paver
    python tools/patchday/hash_baseline.py --diff     # also diff against the previous baseline
"""
import argparse, hashlib, io, json, os, struct, sys, time
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
GAME = r"D:\SteamLibrary\steamapps\common\Crimson Desert"
TOOLS = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def sha(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for blk in iter(lambda: f.read(1 << 24), b""):
            h.update(blk)
    return h.hexdigest()

ap = argparse.ArgumentParser()
ap.add_argument("--diff", action="store_true")
a = ap.parse_args()

if any(x.lower().startswith("dmm") for x in os.listdir(GAME)):
    sys.exit("game is MOUNTED - unmount first, a baseline from a mounted game is worse than none")
pv = open(os.path.join(GAME, "meta", "0.paver"), "rb").read()
ver = "%d.%02d.%02d" % struct.unpack_from("<3H", pv, 0)
out = os.path.join(TOOLS, "hashes_%s_baseline.json" % ver)

files = {}
for root, dirs, fs in os.walk(GAME):
    rel = os.path.relpath(root, GAME).replace("\\", "/")
    top = rel.split("/")[0]
    if not (top.isdigit() or top in ("meta", "bin64") or rel == "."):
        dirs[:] = []
        continue
    for f in fs:
        if rel == ".":
            continue                                  # nothing at the root is game data
        if top == "bin64" and f != "CrimsonDesert.exe":
            continue                                  # the exe only: ASI signatures rot on it
        p = os.path.join(root, f)
        key = f if rel == "." else rel + "/" + f
        files[key] = {"sha256": sha(p), "size": os.path.getsize(p)}
doc = {"file_count": len(files), "files": files, "game_dir": GAME, "paver": pv.hex(),
       "timestamp": time.strftime("%Y-%m-%dT%H:%M:%S")}
io.open(out, "w", encoding="utf-8").write(json.dumps(doc, indent=1, sort_keys=True))
print("wrote %s  (%d files, game %s, paver %s)" % (out, len(files), ver, pv.hex()))

if a.diff:
    prev = sorted(x for x in os.listdir(TOOLS) if x.startswith("hashes_") and x.endswith("_baseline.json") and x != os.path.basename(out))
    if prev:
        pf = json.load(io.open(os.path.join(TOOLS, prev[-1]), encoding="utf-8"))["files"]
        ch = [k for k in files if k in pf and pf[k]["sha256"] != files[k]["sha256"]]
        add = [k for k in files if k not in pf]
        rem = [k for k in pf if k not in files]
        print("vs %s: %d changed / %d added / %d removed" % (prev[-1], len(ch), len(add), len(rem)))
        for k in sorted(ch):
            d = files[k]["size"] - pf[k]["size"]
            print("   %-28s %+d B" % (k, d) if d else "   %-28s same size" % k)
        for k in add: print("   ADDED   ", k)
        for k in rem: print("   REMOVED ", k)
