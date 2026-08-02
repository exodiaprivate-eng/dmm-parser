"""Walk a .pabgb record in Python using a dmm-parser Rust struct as the layout
spec, reporting each field's byte span.

Purpose: when a table breaks on a new game build, a CArray-count blowup surfaces
hundreds of bytes AFTER the actual drift. This walks the same layout the Rust
parser uses and reports per-field offsets, so diffing an old vs new fixture
NAMES the field that moved.

Reusable for any py_binary_struct / pabgh_typed_blob_table table.
"""
import re
import struct

SCALARS = {
    "u8": 1, "i8": 1, "bool": 1,
    "u16": 2, "i16": 2,
    "u32": 4, "i32": 4, "f32": 4,
    "u64": 8, "i64": 8, "f64": 8,
}


def parse_structs(paths):
    """name -> [(field_name, type_str)] for every `pub struct` in the given .rs files."""
    out = {}
    for p in paths:
        try:
            src = open(p, encoding="utf-8").read()
        except OSError:
            continue
        for m in re.finditer(r"pub struct (\w+)(?:<[^>]*>)?\s*\{(.*?)\n\s*\}", src, re.S):
            name, body = m.group(1), m.group(2)
            # Strip line comments first: a type must never absorb `// ...` text.
            body_nc = "\n".join(re.sub(r"//.*$", "", ln) for ln in body.split("\n"))
            # Several `pub x: T,` can share one line, and a generic type may itself
            # contain commas — so split on `pub ` boundaries, not on commas.
            fields = []
            for mm in re.finditer(r"pub (\w+)\s*:\s*(.+?)(?=,\s*pub |,?\s*$)", body_nc, re.M):
                ty = mm.group(2).strip().rstrip(",").strip()
                if ty:
                    fields.append((mm.group(1), ty))
            if fields:
                out.setdefault(name, fields)
    return out


class Walker:
    def __init__(self, structs):
        self.s = structs
        self.rows = []          # (path, start, end, type_label)

    @staticmethod
    def norm(ty):
        return ty.replace("<'a>", "").replace("<'_>", "").strip()

    def field(self, d, p, ty, path):
        ty = self.norm(ty)

        if ty in SCALARS:
            n = SCALARS[ty]
            self.rows.append((path, p, p + n, ty))
            return p + n

        m = re.fullmatch(r"\[(\w+); *(\d+)\]", ty)
        if m:
            n = SCALARS[m.group(1)] * int(m.group(2))
            self.rows.append((path, p, p + n, ty))
            return p + n

        if ty.startswith("CString"):
            n = struct.unpack_from("<I", d, p)[0]
            if n > 8192:
                raise ValueError("%s: CString len %d" % (path, n))
            self.rows.append((path, p, p + 4 + n, "CString(%d)" % n))
            return p + 4 + n

        if ty.startswith("CBytes"):
            n = struct.unpack_from("<I", d, p)[0]
            if n > (1 << 22):
                raise ValueError("%s: CBytes len %d" % (path, n))
            self.rows.append((path, p, p + 4 + n, "CBytes(%d)" % n))
            return p + 4 + n

        if ty.startswith("LocalizableString"):
            n = struct.unpack_from("<I", d, p + 9)[0]
            if n > 8192:
                raise ValueError("%s: LocStr len %d" % (path, n))
            self.rows.append((path, p, p + 13 + n, "LocStr(%d)" % n))
            return p + 13 + n

        # VisioningData::effect_info is hand-written: present iff the preceding
        # visioning_type byte is 0. Not a COptional flag.
        if ty.startswith("Option<"):
            inner = ty[len("Option<"):-1]
            prev = self.rows[-1] if self.rows else None
            present = bool(prev and d[prev[1]] == 0)
            start = p
            if present:
                p = self.field(d, p, inner, path + "?")
            self.rows.append((path, start, p, "Option(%s)" % present))
            return p

        m = re.fullmatch(r"CArray<(.+)>", ty)
        if m:
            n = struct.unpack_from("<I", d, p)[0]
            if n > 1000000:
                raise ValueError("%s: CArray count %d" % (path, n))
            start = p
            p += 4
            for i in range(n):
                p = self.field(d, p, m.group(1), "%s[%d]" % (path, i))
            self.rows.append((path, start, p, "CArray(%d)" % n))
            return p

        m = re.fullmatch(r"COptional<(.+)>", ty)
        if m:
            flag = d[p]
            start = p
            p += 1
            if flag:
                p = self.field(d, p, m.group(1), path + "?")
            self.rows.append((path, start, p, "COptional(%d)" % flag))
            return p

        if ty in self.s:
            fields = self.s[ty]
            # Hand-written wrapper structs in dmm-parser follow two shapes:
            #   struct X { items: Vec<T> }   -> u32 count + count*T   (e.g.
            #                                   GimmickInteractionOverrideCArray)
            #   struct X { inner: Option<T> }-> u8 presence + T if set (e.g.
            #                                   OptionalGimmickInteractionOverrideData)
            # Treat them structurally so the walker doesn't need a per-type case.
            if len(fields) == 1:
                m2 = re.fullmatch(r"Vec<(.+)>", self.norm(fields[0][1]))
                if m2:
                    n = struct.unpack_from("<I", d, p)[0]
                    if n > 1000000:
                        raise ValueError("%s: %s count %d" % (path, ty, n))
                    start = p
                    p += 4
                    for i in range(n):
                        p = self.field(d, p, m2.group(1), "%s[%d]" % (path, i))
                    self.rows.append((path, start, p, "%s(%d)" % (ty, n)))
                    return p
                m2 = re.fullmatch(r"Option<(.+)>", self.norm(fields[0][1]))
                if m2:
                    flag = d[p]
                    start = p
                    p += 1
                    if flag:
                        p = self.field(d, p, m2.group(1), path + "?")
                    self.rows.append((path, start, p, "%s(%d)" % (ty, flag)))
                    return p
            start = p
            for fn, ft in fields:
                p = self.field(d, p, ft, "%s.%s" % (path, fn))
            self.rows.append((path, start, p, ty))
            return p

        raise ValueError("%s: unknown type %r" % (path, ty))

    def walk(self, d, root):
        p = 0
        for fn, ft in self.s[root]:
            p = self.field(d, p, ft, fn)
        return p


def top_level(rows):
    """Only the outermost fields (no nested paths) — for old-vs-new alignment."""
    return [r for r in rows if "." not in r[0] and "[" not in r[0]]
