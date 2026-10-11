#!/usr/bin/env python3
"""Cycle 1 (Q1 reachability) + Cycle 3 (Q3 maxTm field).

Q1: which Tm entry point does the Q5 SDM workflow actually reach?
Q3: every definition/assignment/read of `maxTm` and `primerMaxTm`, and whether
    the `i.maxTm` read in the Primer constructor is reachable.

Usage: python3 q1q3_reach.py [build]     build: live|v2.8.4|v2.7.2|v2.5.3 (default live)
"""
import re
import sys

PATHS = {
    "v2.5.3": "raw/index.17c7600d.v2.5.3.min.js",
    "v2.7.2": "raw/index.dfcf8292.v2.7.2.min.js",
    "v2.8.4": "raw/index.a5da1636.v2.8.4.min.js",
    "live": "live-capture/004eae75971a-index.4330b0c7.js",
}


def bal(s, i, op, cl):
    d = 0
    while i < len(s):
        if s[i] == op:
            d += 1
        elif s[i] == cl:
            d -= 1
            if d == 0:
                return i + 1
        i += 1
    return -1


def fn_range(s, name):
    m = re.search(r"\b" + name + r"\s*=\s*function\s*\(", s)
    if not m:
        return None
    i = s.index("(", m.end() - 1)
    e = bal(s, i, "(", ")")
    j = s.index("{", e)
    return m.start(), bal(s, j, "{", "}")


def main():
    which = sys.argv[1] if len(sys.argv) > 1 else "live"
    s = open(PATHS[which]).read()
    print(f"build={which} file={PATHS[which]} bytes={len(s)}")

    print("\n===== Q1: SDM dispatch reachability =====")
    for name in ("autogenSDMPrimers", "autogenPrimers", "calcSDMPrimers",
                 "createTmFunction", "createTmCalc", "simpleTm"):
        m = fn_range(s, name)
        print(f"  {name}: defined {'yes' if m else 'NO'} @ {m[0] if m else '-'}")
        for mm in re.finditer(r"\b" + name + r"\s*\(", s):
            print(f"    call-ish @{mm.start()}: {s[mm.start():mm.start()+90]}")

    print("\n  --- Fragment.autogenSDMPrimers body ---")
    m = re.search(r"Fragment\.prototype\.autogenSDMPrimers\s*=\s*function", s)
    if m:
        i = s.index("function", m.start())
        j = s.index("{", i)
        e = bal(s, j, "{", "}")
        print("  " + s[m.start():min(e, m.start() + 2000)])

    print("\n  --- dispatcher: call site of calcSDMPrimers (context 900 before) ---")
    for mm in re.finditer(r"\.calcSDMPrimers\s*\(", s):
        print(f"  @{mm.start()}: ...{s[max(0,mm.start()-900):mm.start()+220]}")

    print("\n===== Q3: maxTm / primerMaxTm =====")
    for pat in (r"\bmaxTm\b", r"\bprimerMaxTm\b"):
        occ = list(re.finditer(pat, s))
        print(f"  {pat}: {len(occ)} occurrences")
        for mm in occ:
            print(f"    @{mm.start()}  {s[max(0,mm.start()-140):mm.start()+140]}")

    print("\n  --- Primer constructor body (first 1600) ---")
    m = re.search(r"\bPrimer\s*=\s*function", s)
    if m:
        i = s.index("function", m.start())
        j = s.index("{", i)
        e = bal(s, j, "{", "}")
        print("  " + s[m.start():min(e, m.start() + 1600)])

    print("\n  --- every read of this.primerMaxTm / this.maxTm ---")
    for mm in re.finditer(r"this\.(primerMaxTm|maxTm)", s):
        print(f"    @{mm.start()} {s[max(0,mm.start()-120):mm.start()+120]}")


if __name__ == "__main__":
    main()
