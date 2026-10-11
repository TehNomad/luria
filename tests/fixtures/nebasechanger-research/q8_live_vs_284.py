#!/usr/bin/env python3
"""Q8: compare NEBaseChanger v2.8.4 (a5da1636) vs the live build (4330b0c7).

For every named algorithm function, extract the minified body from both builds,
reduce it to a canonical token stream (identifiers -> ID, so minifier renaming
is invisible), and report SAME / DIFFERENT.  For DIFFERENT functions, print a
unified diff of the beautified bodies.

Usage: q8_live_vs_284.py [--names "a,b,c"]
"""
import os
import re
import difflib
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
BUILDS = {
    "v2.8.4": os.path.join(HERE, "app-js", "index.a5da1636.v2.8.4.js"),
    "live": os.path.join(HERE, "live-capture", "004eae75971a-index.4330b0c7.js"),
}

# algorithm surface agreed for comparison (Q3/Q4/Q5/Q7 surface)
NAMES = """expandDNASeq DEGENERATES alignDNAglobal alignDNAlocal createTmCalc
calcTm calculatePrimers calculateSDMPrimers calcSDMPrimers autogenPrimers
autogenSDMPrimers terminalOverlapLen simpleTm createTmFunction revcomp
translate cut matchesIUPAC verifyOp""".split()


def brace_span(src, start):
    d = 0
    for i in range(start, len(src)):
        if src[i] == "{":
            d += 1
        elif src[i] == "}":
            d -= 1
            if d == 0:
                return src[start : i + 1]
    return src[start:]


def extract(src, name):
    """Return the minified definition text for `name` (method or global)."""
    out = []
    for m in re.finditer(rf"(?:\.\b)?{re.escape(name)}\s*=\s*function\b", src):
        body_start = src.find("{", m.end())
        if body_start == -1:
            continue
        out.append(f"{name} = " + brace_span(src, body_start))
    # object-literal tables (DEGENERATES = { ... })
    for m in re.finditer(rf"(?:\.\b)?{re.escape(name)}\s*=\s*\{{", src):
        out.append(f"{name} = " + brace_span(src, m.end() - 1))
    return out


def canon(body: str) -> str:
    """Token stream that ignores minifier renaming and whitespace."""
    toks = re.findall(r"[A-Za-z_$][A-Za-z0-9_$]*|\d+\.?\d*|<<<|>>>|>=|<=|==|!=|[-+*/%(){}[\];,.:?]", body)
    out = []
    for t in toks:
        if re.fullmatch(r"[A-Za-z_$][A-Za-z0-9_$]*", t):
            # keep known domain identifiers, mask local minifier names
            out.append(t if t.isupper() or "_" in t or t in (
                "DEGENERATES", "expandDNASeq", "alignDNAglobal", "alignDNAlocal",
                "createTmCalc", "calcTm", "calculatePrimers", "calculateSDMPrimers",
                "calcSDMPrimers", "autogenPrimers", "autogenSDMPrimers",
                "terminalOverlapLen", "simpleTm", "createTmFunction", "revcomp",
                "translate", "cut", "matchesIUPAC", "verifyOp",
                "seq", "seqid", "start", "end", "repseq", "selstart", "selend",
                "primerMinTm", "primerMaxTm", "primerMinLen", "primerMethod",
                "primerMonosalt", "primerCt", "polGroup", "confine5prime",
                "fwdprimer", "revprimer", "fwdPrimer", "revPrimer",
            ) else "ID")
        else:
            out.append(t)
    return " ".join(out)


def main():
    srcs = {k: open(v).read() for k, v in BUILDS.items()}
    names = NAMES
    for a in sys.argv[1:]:
        if a.startswith("--names"):
            names = a.split("=", 1)[1].split(",") if "=" in a else names
    print("builds:", {k: (v, len(open(v).read())) for k, v in BUILDS.items()})
    same, diff, missing = [], [], []
    detail = {}
    for n in names:
        sigs = {}
        for b, s in srcs.items():
            ext = extract(s, n)
            sigs[b] = [canon(x) for x in ext]
            if not ext:
                missing.append((b, n))
        a, l = sigs["v2.8.4"], sigs["live"]
        tag = "SAME" if a == l else "DIFFERENT"
        (same if a == l else diff).append(n)
        detail[n] = (len(a), len(l))
        print(f"{n:>22}  v2.8.4:{len(a)}  live:{len(l)}  {tag}")
    print()
    print("same:", same)
    print("different:", diff)
    print("missing:", missing)
    return diff


if __name__ == "__main__":
    main()
