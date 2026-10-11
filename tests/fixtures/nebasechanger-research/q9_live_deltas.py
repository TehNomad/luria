#!/usr/bin/env python3
"""Q9: per-function deltas between NEBaseChanger v2.8.4 (a5da1636) and the
live build 4330b0c7 (v2.11.0).

Works on the prettier-beautified bundles so the output is a readable unified
diff of each algorithm function.  Renamed minifier identifiers therefore show
up as real diff lines; read them together with the token-shape verdict from
q8_live_vs_284.py (SAME = renaming only).

Usage: q9_live_deltas.py [name ...]     (default: the agreed algorithm surface)
"""
import os
import re
import difflib
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
BEAUTY = {
    "v2.8.4": os.path.join(HERE, "app-js", "beautified", "index.a5da1636.v2.8.4.beauty.js"),
    "live": os.path.join(HERE, "live-js", "beautified", "index.4330b0c7.v2.11.0-live.beauty.js"),
}

NAMES = """DEGENERATES expandDNASeq alignDNAglobal alignDNAlocal createTmCalc
calcTm calculatePrimers calculateSDMPrimers calcSDMPrimers autogenPrimers
autogenSDMPrimers terminalOverlapLen simpleTm createTmFunction revcomp
translate cut matchesIUPAC verifyOp""".split()


def extract(src: str, name: str):
    """Yield (start_line, text) for each definition of `name` in a beautified bundle."""
    out = []
    lines = src.split("\n")
    pat = re.compile(rf"(?:\.\b)?{re.escape(name)}\s*=\s*(function\b|\{{)")
    for i, ln in enumerate(lines):
        if not pat.search(ln):
            continue
        # brace-match from this line
        depth = 0
        started = False
        buf = []
        for j in range(i, len(lines)):
            buf.append(lines[j])
            for ch in lines[j]:
                if ch == "{":
                    depth += 1
                    started = True
                elif ch == "}":
                    depth -= 1
            if started and depth == 0 and j > i:
                break
        out.append((i + 1, "\n".join(buf)))
    return out


def main():
    srcs = {k: open(v).read() for k, v in BEAUTY.items()}
    names = sys.argv[1:] or NAMES
    print("beautified bundles:", {k: (os.path.basename(v), len(open(v).read())) for k, v in BEAUTY.items()})
    for n in names:
        a = extract(srcs["v2.8.4"], n)
        b = extract(srcs["live"], n)
        print(f"\n######## {n}  v2.8.4:{len(a)}  live:{len(b)}")
        if not a and not b:
            print("  absent in both")
            continue
        if not a:
            print("  NEW IN LIVE (not present in v2.8.4):")
            for ln, body in b:
                print(f"  [live line {ln}]")
                print(body)
            continue
        if not b:
            print("  REMOVED IN LIVE (present only in v2.8.4):")
            for ln, body in a:
                print(f"  [v2.8.4 line {ln}]")
                print(body)
            continue
        for k in range(max(len(a), len(b))):
            ta = a[k][1] if k < len(a) else ""
            tb = b[k][1] if k < len(b) else ""
            if ta == tb:
                print(f"  occurrence {k}: IDENTICAL ({len(ta)} chars)")
                continue
            print(f"  occurrence {k}: DIFFERS  (v2.8.4 line {a[k][0]} / live line {b[k][0]})")
            d = difflib.unified_diff(
                ta.split("\n"), tb.split("\n"),
                fromfile=f"v2.8.4:{n}", tofile=f"live:{n}", lineterm="", n=3,
            )
            for row in d:
                print(row)


if __name__ == "__main__":
    main()
