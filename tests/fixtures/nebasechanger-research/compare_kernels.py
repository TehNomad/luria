#!/usr/bin/env python3
"""Compare NEBaseChanger algorithm kernels across bundle versions.

Extracts each kernel by a stable anchor, strips console logging, normalises
identifiers positionally, and reports whether the kernels are structurally
identical across versions.
"""
import re
import subprocess
import sys

VERSIONS = {
    "v2.5.3": "app-js/beautified/index.17c7600d.v2.5.3.beauty.js",
    "v2.7.2": "app-js/beautified/index.dfcf8292.v2.7.2.beauty.js",
    "v2.8.4": "app-js/beautified/index.a5da1636.v2.8.4.beauty.js",
}

# anchor -> regex per version (property names survive minification)
ANCHORS = [
    r"(?<!Primer\.)calcSDMPrimers = function",
    r"Primer\.calculatePrimers = function",
    r"TmCalc\.prototype\.calcTm = function",
    r"expandDNASeq = function",
    r"parseSeqFile = function",
    r"alignDNAglobal = function",
    r"terminalOverlapLen = function",
    r"NASeq\.prototype = \{",
]


def body(text, start):
    i = text.find("{", start)
    depth = 0
    k = i
    while k < len(text):
        c = text[k]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return text[start : k + 1]
        k += 1
    return text[start:]


def normalise(src):
    """Strip console logging and rename identifiers positionally."""
    src = re.sub(r"console\.\w+\([^()]*(?:\([^()]*\)[^()]*)*\)", "LOG", src)
    names = []

    def sub(m):
        n = m.group(0)
        if n not in names:
            names.append(n)
        return "@" + str(names.index(n))

    src = re.sub(r"\b[A-Za-z_$][A-Za-z0-9_$]*\b", sub, src)
    src = re.sub(r"\s+", " ", src)
    return src


def main():
    root = "tests/fixtures/nebasechanger-research"
    data = {v: open(p, encoding="utf-8").read() for v, p in VERSIONS.items()}
    for pat in ANCHORS:
        row = {}
        for v, d in data.items():
            m = re.search(pat, d)
            if not m:
                row[v] = None
                continue
            blk = body(d, m.start())
            row[v] = (d[: m.start()].count("\n") + 1, normalise(blk))
        vals = {v: r[1] for v, r in row.items() if r}
        same = len(set(vals.values())) == 1 if vals else False
        print(f"### {pat}")
        for v in VERSIONS:
            r = row.get(v)
            print(f"  {v}: line {r[0]}  ({'present' if r else 'ABSENT'})")
        print(f"  normalised-identical: {same}")
        if not same and vals:
            import difflib

            vs = list(vals.items())
            for i in range(len(vs) - 1):
                (a, sa), (b, sb) = vs[i], vs[i + 1]
                diff = list(difflib.ndiff(sa.split(), sb.split()))
                changed = [x for x in diff if x[0] in "+-"]
                print(f"  {a}->{b}: {len(changed)} changed tokens")
                print("   " + " ".join(changed[:40]))
        print()


main()
