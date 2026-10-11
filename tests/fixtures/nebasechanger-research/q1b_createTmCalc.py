#!/usr/bin/env python3
"""Cycle 1 (Q1, part 2): dump the full `createTmCalc` closure and the
`setMethod`/`Tm` internals for the live build, plus reachability of the
alternative Tm entry points.

Usage: python3 q1b_createTmCalc.py [build]
  build: live | v2.8.4 | v2.7.2 | v2.5.3   (default: live)
"""
import re
import sys

PATHS = {
    "v2.5.3": "raw/index.17c7600d.v2.5.3.min.js",
    "v2.7.2": "raw/index.dfcf8292.v2.7.2.min.js",
    "v2.8.4": "raw/index.a5da1636.v2.8.4.min.js",
    "live": "live-capture/004eae75971a-index.4330b0c7.js",
}


def balanced(src, start, op, cl):
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


def fn_range(src, name):
    m = re.search(r"\b" + name + r"\s*=\s*function\s*\(", src)
    if not m:
        return None
    i = src.index("(", m.end() - 1)
    end = balanced(src, i, "(", ")")
    j = src.index("{", end)
    return m.start(), balanced(src, j, "{", "}")


def main():
    which = sys.argv[1] if len(sys.argv) > 1 else "live"
    src = open(PATHS[which]).read()
    s, e = fn_range(src, "createTmCalc")
    body = src[s:e]
    print(f"createTmCalc @ {s}..{e}  len={len(body)}")

    # 1. parameter normalisation header
    print("\n--- header (first 700 chars) ---")
    print(body[:700])

    # 2. the returned object: find `return{` inside and list its keys
    print("\n--- returned object keys ---")
    m = re.search(r"return\s*\{", body)
    if m:
        end = balanced(body, body.index("{", m.start()), "{", "}")
        print(f"  return{{...}} @ {m.start()}..{end} len={end-m.start()}")
        keys = re.findall(r"([A-Za-z_$][\w$]*)\s*:", body[m.start():end])
        print("  keys:", keys)

    # 3. inner function definitions inside createTmCalc
    print("\n--- inner named functions / vars assigned functions ---")
    for mm in re.finditer(r"[,;]([A-Za-z_$][\w$]*)\s*=\s*function", body):
        print(f"  {mm.group(1)} = function @ {mm.start()}")

    # 4. setMethod / Tm definitions (they are on the returned object)
    print("\n--- setMethod / Tm / setSeq bodies ---")
    for name in ("setMethod", "Tm", "setSeq", "setCt", "setSalt", "setDMSO", "saltCorrect"):
        mm = re.search(r"\b" + name + r"\s*:\s*function", body)
        if not mm:
            mm = re.search(r"\b" + name + r"\s*=\s*function", body)
        if mm:
            i = body.index("function", mm.start())
            j = body.index("{", i)
            end = balanced(body, j, "{", "}")
            print(f"  {name} @ {mm.start()}..{end} len={end-mm.start()}")
            print("   " + body[mm.start():end])
        else:
            print(f"  {name}: NOT FOUND in createTmCalc")

    # 5. reachability of other Tm entry points
    print("\n--- reachability of alternative Tm entry points ---")
    for name in ("createTmFunction", "calcTm_nomm", "TmCalc", "simpleTm"):
        occ = [mm.start() for mm in re.finditer(r"\b" + name + r"\b", src)]
        print(f"  {name}: {len(occ)} occurrences {occ[:8]}")
        for o in occ[:12]:
            print(f"    @{o}: {src[max(0,o-90):o+110]}")


if __name__ == "__main__":
    main()
