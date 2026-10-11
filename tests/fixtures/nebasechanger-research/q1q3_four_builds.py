#!/usr/bin/env python3
"""Q1-Q3 headline claims re-verified across all four bundles.

Each check is a token/regex count, not a reading of a diff.  Output is a
table: claim x build -> observed.  Local only, no network.

Bundles (see manifest.json for provenance and sha256):
  v2.5.3       app-js/index.17c7600d.v2.5.3.js
  v2.7.2       app-js/index.dfcf8292.v2.7.2.js
  v2.8.4       app-js/index.a5da1636.v2.8.4.js
  live v2.11.0 live-capture/004eae75971a-index.4330b0c7.js
"""
import re

BUNDLES = [
    ("v2.5.3", "app-js/index.17c7600d.v2.5.3.js"),
    ("v2.7.2", "app-js/index.dfcf8292.v2.7.2.js"),
    ("v2.8.4", "app-js/index.a5da1636.v2.8.4.js"),
    ("live v2.11.0", "live-capture/004eae75971a-index.4330b0c7.js"),
]


def brace_span(src, open_idx):
    depth = 0
    for j in range(open_idx, len(src)):
        c = src[j]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return open_idx, j
    return open_idx, len(src)


def body_after(src, pattern):
    m = re.search(pattern, src)
    if not m:
        return None
    k = src.index("{", m.end())
    a, b = brace_span(src, k)
    return src[a : b + 1]


def main():
    for tag, path in BUNDLES:
        src = open(path, encoding="utf-8", errors="replace").read()
        print(f"### {tag}  ({path})")

        # --- Q1: which Tm engines exist, and which methods each supports
        tmcalc = body_after(src, r"TmCalc\.prototype\.calcTm\s*=\s*function")
        print("  Q1a TmCalc.prototype.calcTm:")
        if tmcalc is None:
            print("    ABSENT")
        else:
            print(f"    present, body {len(tmcalc)} chars")
            print(f"    cases: {sorted(set(re.findall(r'case (\d+):', tmcalc)))}")
            print(
                "    debug console.log: "
                + ("yes" if "console.log" in tmcalc else "no")
            )
            m8 = re.search(r"case 8:.{0,60}", tmcalc)
            m7 = re.search(r"case 7:.{0,110}", tmcalc)
            print("    case 8: " + (" ".join(m8.group(0).split()) if m8 else "absent"))
            print("    case 7: " + (" ".join(m7.group(0).split()) if m7 else "absent"))

        print("  Q1b TmCalc.prototype.calcTm_nomm:")
        nomm = body_after(src, r"TmCalc\.prototype\.calcTm_nomm\s*=\s*function")
        print("    " + (f"present, body {len(nomm)} chars" if nomm else "ABSENT"))

        print("  Q1c createTmCalc (the closure engine used by the SDM route):")
        ctc = body_after(src, r"createTmCalc\s*=\s*function")
        if ctc is None:
            print("    ABSENT")
        else:
            print(f"    present, body {len(ctc)} chars")
            print(f"    cases: {sorted(set(re.findall(r'case (\d+):', ctc)))}")

        print("  Q1d call sites of the engines:")
        for name in ("calcTm_nomm", "createTmCalc", "createTmFunction"):
            n = len(re.findall(re.escape(name) + r"\s*\(", src))
            print(f"    {name} invoked {n} time(s)")
        n = len(re.findall(r"calcTm(?!_)\s*\(", src))
        print(f"    calcTm invoked {n} time(s)  <- 0 means the engine is dead code")

        # --- Q2: ct units
        print("  Q2 ct unit handling:")
        for pat in (r"(\w+)=\1/1e3", r"(\w+)=\1/4\b", r"ct[:=]\s*[\d.]+"):
            hits = re.findall(pat, src)
            print(f"    {pat}: {len(hits)} -> {sorted(set(map(str, map(str, hits))))}")

        # --- Q3: primerMaxTm plumbing
        print("  Q3 primerMaxTm plumbing:")
        print(
            "    mis-named field `primerMaxTm=X.maxTm`: "
            + str(len(re.findall(r"primerMaxTm=\w+\.maxTm\b", src)))
        )
        print(
            "    `primerMaxTm||70` fallbacks: "
            + str(len(re.findall(r"primerMaxTm\|\|70", src)))
        )
        print(
            "    `primerMaxTm=72` clamps: "
            + str(len(re.findall(r"primerMaxTm=72", src)))
        )

        # --- dead code / removals
        print("  Removals (archived -> live):")
        for name in ("calculateSDMPrimers",):
            print(f"    {name}: {len(re.findall(name, src))}")
        print("  Input normalisation:")
        print(
            "    setSeq u->t: "
            + str(len(re.findall(r'replace\(/u/g,"t"\)', src)))
        )
        print("  Constants:")
        print(f"    R=1.987: {len(re.findall(r'prototype\.R=1\.987', src))}")
        print()


if __name__ == "__main__":
    main()
