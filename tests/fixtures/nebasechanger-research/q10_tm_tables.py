#!/usr/bin/env python3
"""Q10: entry-by-entry comparison of the nearest-neighbour Tm tables in
NEBaseChanger v2.8.4 (a5da1636) vs the live build 4330b0c7 (v2.11.0).

Scans the createTmCalc closure in each prettier-beautified bundle and pulls out
every numeric table entry, keyed by the table it appears in (tables are
identified by their position in the source, in the order the closure declares
them: initiation, termination, consonance/br pairs, discontinuity (nndhds),
asymmetric loop, bulge).

Output: entries only in one build, and entries whose value changed.
"""
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
B = {
    "v2.8.4": os.path.join(HERE, "app-js", "beautified", "index.a5da1636.v2.8.4.beauty.js"),
    "live": os.path.join(HERE, "live-js", "beautified", "index.4330b0c7.v2.11.0-live.beauty.js"),
}

PAIR_RE = re.compile(r'"?([\w/+-]+)"?:\s*\{\s*dh:\s*([-\d.e+]+),\s*ds:\s*([-\d.e+]+)\s*\}')
SCALAR_RE = re.compile(r"(\w+):\s*([-\d.]+)")


def brace_text(src, start):
    d = 0
    for i in range(start, len(src)):
        if src[i] == "{":
            d += 1
        elif src[i] == "}":
            d -= 1
            if d == 0:
                return src[start:i + 1]
    return src[start:]


def tables(src):
    m = re.search(r"createTmCalc = function", src)
    body = brace_text(src, src.find("{", m.end()))
    out = []
    for tm in re.finditer(r"(\w+) = \{((?:\s*(?:\"[^\"]+\"|[\w/+-]+): \{[^}]*\},?)+)\s*\}", body):
        entries = {}
        for mm in PAIR_RE.finditer(tm.group(2)):
            k, dh, ds = mm.groups()
            entries[k] = (float(dh), float(ds))
        out.append(("nndhds:" + tm.group(1), entries))
    for tm in re.finditer(r"(\w+) = \{((?:\s*\w+: [-\d.]+,?)+\s*)\}", body):
        entries = {}
        for mm in SCALAR_RE.finditer(tm.group(2)):
            k, v = mm.groups()
            entries[k] = float(v)
        if len(entries) >= 4:
            out.append(("scalar:" + tm.group(1), entries))
    return out


def main():
    srcs = {k: open(v).read() for k, v in B.items()}
    tabs = {k: tables(s) for k, s in srcs.items()}
    for k, t in tabs.items():
        print(f"{k}: {[(n, len(d)) for n, d in t]}")
    print()
    a, b = tabs["v2.8.4"], tabs["live"]
    for i in range(max(len(a), len(b))):
        na, da = a[i] if i < len(a) else ("?", {})
        nb, db = b[i] if i < len(b) else ("?", {})
        only_a = {k: v for k, v in da.items() if k not in db}
        only_b = {k: v for k, v in db.items() if k not in da}
        changed = {k: (da[k], db[k]) for k in da if k in db and da[k] != db[k]}
        print(f"\ntable #{i}  v2.8.4={na} live={nb}  ({len(da)} vs {len(db)} entries)")
        print(f"  only v2.8.4: {only_a}")
        print(f"  only live:   {only_b}")
        print(f"  changed:     {changed}")


if __name__ == "__main__":
    main()
