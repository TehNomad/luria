#!/usr/bin/env python3
"""Objective 4: independent reproduction of the thermodynamic-table comparison.

For every build, locate each TmCalc table literal by brace-balanced scan of the
RAW minified bundle (no beautifier involved), record its exact byte offset, and
compare parsed values across builds.

Tables compared: dSBr dSSa dHBr dHSa nnde nndhds nnloop nnbulge nntmm init
saltCorrect R  (plus DEGENERATES / COMPLEMENTS for context).
"""
import ast
import json
import re

BUILDS = {
    "v2.5.3": "raw/index.17c7600d.v2.5.3.min.js",
    "v2.7.2": "raw/index.dfcf8292.v2.7.2.min.js",
    "v2.8.4": "raw/index.a5da1636.v2.8.4.min.js",
    "live-v2.11.0": "live-capture/004eae75971a-index.4330b0c7.js",
}
TABLES = ["dSBr", "dSSa", "dHBr", "dHSa", "nnde", "nndhds", "nnloop",
          "nnbulge", "nntmm", "DEGENERATES"]

# The paired alphabet strings used by the complement helpers.
COMPLEMENT_RE = re.compile(
    r'"(ACGT[A-Za-z\- ]{10,60})"\s*,\s*[A-Za-z_$][A-Za-z0-9_$]*\s*=\s*"(TGCA[A-Za-z\- ]{10,60})"')


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


def extract(src):
    out = {}
    for k in TABLES:
        for m in re.finditer(rf"(?<![A-Za-z0-9_]){k}\s*[:=]", src):
            v = m.group(0)[:-1]
            if v.endswith("="):
                continue
            i = m.end()
            if src[i] in "{[":
                end = bal(src, i, src[i], "}" if src[i] == "{" else "]")
                out[k] = (m.start(), src[i:end])
                break
    return out


def parse(lit):
    """JS object/array literal -> Python object via ast.literal_eval.
    Identifier keys are quoted; numeric and already-quoted keys are left alone.
    Python accepts JS-style numbers (-.7, 4.4) so no numeric rewriting is needed."""
    j = re.sub(r"(?<![\"A-Za-z0-9_.])([A-Za-z_][A-Za-z0-9_]*)\s*:", r'"\1":', lit)
    try:
        return ast.literal_eval(j)
    except Exception:
        return None


def version(src):
    m = re.search(r'return\["(2\.\d+\.\d+)"\]', src)
    return m.group(1) if m else None


def main():
    for b, p in BUILDS.items():
        src = open(p, encoding="utf8").read()
        print(f"{b}: file sha256-independent size {len(src)} bytes; "
              f"version constant return[\"...\"] = {version(src)}")
    tabs = {}
    offs = {}
    for b, p in BUILDS.items():
        src = open(p, encoding="utf8").read()
        e = extract(src)
        tabs[b] = {k: parse(v) for k, (o, v) in e.items()}
        offs[b] = {k: o for k, (o, v) in e.items()}
        print(f"== {b}: {len(e)}/{len(TABLES)} tables located")
        for k in TABLES:
            if k in e:
                print(f"   {k}: byte offset {offs[b][k]}, literal length {len(e[k][1])}")

    print()
    order = list(BUILDS)
    for k in TABLES:
        variants = {}
        for b in order:
            t = tabs[b].get(k)
            if t is None:
                continue
            key = json.dumps(t, sort_keys=True)
            variants.setdefault(key, []).append(b)
        if len(variants) == 1:
            print(f"{k}: IDENTICAL across all builds where present "
                  f"({', '.join(next(iter(variants.values())))})")
        else:
            print(f"{k}: DIFFERS")
            for key, bs in variants.items():
                print(f"   builds {', '.join(bs)}: {key[:200]}")

    print("\n=== per-key diff of DMG tables across builds ===")
    for k in ("nnde", "nndhds", "nnloop", "nnbulge", "nntmm"):
        keys = set()
        for b in order:
            t = tabs[b].get(k)
            if t:
                keys |= set(t.keys())
        print(f"\n{k}:")
        for kk in sorted(keys):
            row = []
            for b in order:
                t = tabs[b].get(k, {}).get(kk)
                row.append(json.dumps(t) if t is not None else "ABSENT")
            print(f"  {kk}: " + " | ".join(row) + ("   <-- SAME" if len(set(row)) == 1 else "   <-- CHANGED"))

    print("\n=== complement alphabet pairs (source -> target, char-wise) ===")
    comp = {}
    for b, p in BUILDS.items():
        src = open(p, encoding="utf8").read()
        seen = {}
        for m in COMPLEMENT_RE.finditer(src):
            a, c = m.group(1), m.group(2)
            seen.setdefault((a, c), []).append(m.start())
        comp[b] = seen
        print(f"  {b}: {len(seen)} distinct pairs at offsets:")
        for (a, c), offs in seen.items():
            print(f"    {a} -> {c}   x{len(offs)} at {offs}")
    print()
    for b in list(BUILDS)[1:]:
        a0 = set(comp[list(BUILDS)[0]])
        a1 = set(comp[b])
        if a0 != a1:
            print(f"  complement pairs DIFFER vs v2.5.3: only-in-{b}={a1 - a0}, "
                  f"only-in-v2.5.3={a0 - a1}")
    print("  per-pair char mapping diff (v2.8.4 vs live):")
    old = comp["v2.8.4"]
    new = comp["live-v2.11.0"]
    for (a, c) in set(old) | set(new):
        o = old.get((a, c))
        n = new.get((a, c))
        if o is None or n is None:
            print(f"    pair present only in one build: {a} -> {c}")
            continue
    # compare by source alphabet ordering
    bysrc = {a: c for (a, c) in old}
    bysrc2 = {a: c for (a, c) in new}
    for a in set(bysrc) | set(bysrc2):
        c1, c2 = bysrc.get(a), bysrc2.get(a)
        if c1 != c2:
            print(f"    source {a}: v2.8.4 target={c1}  live target={c2}")
            if c1 and c2 and len(c1) == len(c2):
                for i, ch in enumerate(a):
                    if c1[i] != c2[i]:
                        print(f"      char {ch}: v2.8.4 -> {c1[i]}   live -> {c2[i]}")

    print("\n=== DEGENERATES per key ===")
    for k in ("DEGENERATES",):
        keys = set()
        for b in order:
            t = tabs[b].get(k)
            if t:
                keys |= set(t.keys())
        print(f"\n{k}:")
        for kk in sorted(keys):
            row = []
            for b in order:
                t = tabs[b].get(k, {}).get(kk)
                row.append(str(t) if t is not None else "ABSENT")
            print(f"  {kk}: " + " | ".join(row) + ("   <-- SAME" if len(set(row)) == 1 else "   <-- CHANGED"))


main()
