#!/usr/bin/env python3
"""Cycle 1 (Q1): trace the runtime Tm call path in the live NEBaseChanger build.

Emits, for each build, the exact source text of:
  * the app-layer `tm` wrapper (Ge.tm / Ne.tm)
  * `createTmCalc` (including the closure `Tm` it builds)
  * every call site of the app-layer `tm`
  * every reference to `calcTm_nomm`

Reads only; writes nothing. Deterministic.

Usage:  python3 q1_tm_callpath.py
"""
import re
import sys

BUILDS = {
    "v2.5.3": "raw/index.17c7600d.v2.5.3.min.js",
    "v2.7.2": "raw/index.dfcf8292.v2.7.2.min.js",
    "v2.8.4": "raw/index.a5da1636.v2.8.4.min.js",
    "live-v2.11.0": "live-capture/004eae75971a-index.4330b0c7.js",
}


def balanced(src, start, op, cl):
    """Return end index just past the bracket opened at src[start] == op."""
    depth = 0
    i = start
    while i < len(src):
        c = src[i]
        if c == op:
            depth += 1
        elif c == cl:
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return -1


def find_fn(src, name):
    """Locate `name=function(` and return (start, end) of the whole function."""
    m = re.search(r"\b" + name + r"\s*=\s*function\s*\(", src)
    if not m:
        return None
    i = src.index("(", m.end() - 1)
    end = balanced(src, i, "(", ")")
    if end < 0:
        return None
    j = src.index("{", end)
    body_end = balanced(src, j, "{", "}")
    if body_end < 0:
        return None
    return m.start(), body_end


def main():
    for label, path in BUILDS.items():
        try:
            src = open(path).read()
        except FileNotFoundError:
            print(f"!! {label}: {path} MISSING")
            continue
        print(f"### {label}  ({path}, {len(src)} bytes)")

        # app-layer tm wrapper: Ge.tm= / Ne.tm= / t.tm=
        for m in re.finditer(r"([A-Za-z_$][\w$]*)\.tm\s*=\s*function\s*\(", src):
            i = src.index("(", m.end() - 1)
            end = balanced(src, i, "(", ")")
            j = src.index("{", end)
            body_end = balanced(src, j, "{", "}")
            ns = m.group(1)
            print(f"  [{ns}.tm @ {m.start()}..{body_end}] len={body_end-m.start()}")
            print("    " + src[m.start():min(body_end, m.start() + 700)])

        # createTmCalc
        m = re.search(r"createTmCalc\s*=\s*function\s*\(", src)
        if m:
            s, e = find_fn(src, "createTmCalc")
            print(f"  createTmCalc @ {s}..{e} len={e-s}")
            print("    " + src[s:min(e, s + 900)])

        # calcTm_nomm references
        refs = [m.start() for m in re.finditer(r"calcTm_nomm", src)]
        print(f"  calcTm_nomm occurrences: {len(refs)} at {refs}")
        for r in refs:
            print(f"    @{r}: ...{src[max(0,r-120):r+80]}...")

        # tm( call sites in app layer
        calls = [m.start() for m in re.finditer(r"\b[A-Za-z_$][\w$]*\.tm\(", src)]
        print(f"  <ns>.tm( call sites: {len(calls)}")
        for c in calls[:14]:
            print(f"    @{c}: {src[c:c+120]}")
        print()


if __name__ == "__main__":
    main()
