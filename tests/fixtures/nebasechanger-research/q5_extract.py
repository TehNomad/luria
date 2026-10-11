#!/usr/bin/env python3
"""Cycle-5 extraction: the primer-generation + validation passes, live build.

Prints brace-balanced function bodies verbatim (with byte offsets) for:
  Primer.calculatePrimers        (search loop + acceptance test)
  Fragment.autogenPrimers        (whole-fragment route + post-checks)
  Fragment.autogenSDMPrimers     (SDM-named route)
  Fragment.setFwdPrimer / setRevPrimer (argument validation)
  Primer.locate_primer           (diagnostics)
  Ne.calcSDMPrimers              (inline-mutation route)
  Ne.revcomp / Ne.comp           (app-layer complement helpers)
plus every `throw new Error(...)` message and every `push("...")` diagnostic
inside the app-layer dispatcher region.

Usage: q5_extract.py [build]      build in {live, v2.8.4, v2.7.2, v2.5.3}
"""
import re
import sys

PATHS = {
    "live": "live-capture/004eae75971a-index.4330b0c7.js",
    "v2.8.4": "raw/index.a5da1636.v2.8.4.min.js",
    "v2.7.2": "raw/index.dfcf8292.v2.7.2.min.js",
    "v2.5.3": "raw/index.17c7600d.v2.5.3.min.js",
}
NS = {"live": "Ne", "v2.8.4": "Ge", "v2.7.2": "Ee", "v2.5.3": "ge"}


def bal(s, i, op, cl):
    d = 0
    while True:
        if s[i] == op:
            d += 1
        elif s[i] == cl:
            d -= 1
            if d == 0:
                return i + 1
        i += 1


def body(src, start):
    j = src.index("{", start)
    return src[start:bal(src, j, "{", "}")]


def main():
    which = sys.argv[1] if len(sys.argv) > 1 else "live"
    src = open(PATHS[which], encoding="utf8").read()
    ns = NS[which]
    pats = [
        r"(?<![\w$])Primer\.calculatePrimers\s*=\s*function",
        r"prototype\.autogenPrimers\s*=\s*function",
        r"prototype\.autogenSDMPrimers\s*=\s*function",
        r"prototype\.setFwdPrimer\s*=\s*function",
        r"prototype\.setRevPrimer\s*=\s*function",
        r"(?<![\w$])Primer\.locate_primer\s*=\s*function",
        rf"(?<![A-Za-z0-9_.]){ns}\.calcSDMPrimers\s*=\s*function",
        rf"(?<![A-Za-z0-9_.]){ns}\.revcomp\s*=\s*function",
        rf"(?<![A-Za-z0-9_.]){ns}\.comp\s*=\s*function",
        r"prototype\.updateTm\s*=\s*function",
        r"prototype\.terminalOverlapLen\s*=\s*function",
    ]
    for p in pats:
        for m in re.finditer(p, src):
            b = body(src, m.start())
            print(f"### {p}  @ {m.start()}  len={len(b)}")
            print(b)
            print()

    print("### throw/return-error messages in the app layer (offset > 300000)")
    for m in re.finditer(r'(throw new Error|push)\(\s*"([^"]{4,120})"', src):
        if m.start() > 300000:
            print(f"  @{m.start()} {m.group(1)}: {m.group(2)!r}")

    print()
    print("### createTmFunction call sites (all)")
    for m in re.finditer(r"createTmFunction\(", src):
        print(f"  @{m.start()}: {src[m.start():m.start()+90]!r}")

    print()
    print("### createTmCalc call sites (all)")
    for m in re.finditer(r"createTmCalc\(", src):
        print(f"  @{m.start()}: {src[max(0,m.start()-60):m.start()+110]!r}")

    print()
    print("### Ne.tm call sites (all)")
    for m in re.finditer(rf"(?<![\w$.]){ns}\.tm\(", src):
        print(f"  @{m.start()}: {src[max(0,m.start()-80):m.start()+60]!r}")


main()
