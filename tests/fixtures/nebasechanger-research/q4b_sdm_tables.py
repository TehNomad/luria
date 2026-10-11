#!/usr/bin/env python3
"""Cycle 4 (part 2): compare the tables actually used by the SDM engine.

`tm_tables_diff.py` compares the FIRST occurrence of each table name, which is
the `TmCalc.prototype.*` table (the NEBuilder engine reached via
`createTmFunction`). The SDM engine (`createTmCalc`) keeps its own copies in
minified locals, aliased in the returned object as
`{dSSa, dHSa, dSBr, dHBr, nndhds, loop, bulge}`.

This script maps each returned-object key to its local variable inside
`createTmCalc`, extracts that object literal, and compares across builds.

Reads fixtures only; prints a table. No network.

Usage: python3 q4b_sdm_tables.py
"""
import re
import itertools

PATHS = {
    "v2.5.3": "raw/index.17c7600d.v2.5.3.min.js",
    "v2.7.2": "raw/index.dfcf8292.v2.7.2.min.js",
    "v2.8.4": "raw/index.a5da1636.v2.8.4.min.js",
    "live": "live-capture/004eae75971a-index.4330b0c7.js",
}

KEYS = ["dSSa", "dHSa", "dSBr", "dHBr", "nndhds", "loop", "bulge"]


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


def createTmCalc_body(src):
    m = re.search(r"\bcreateTmCalc\s*=\s*function\s*\(", src)
    i = src.index("(", m.end() - 1)
    e = bal(src, i, "(", ")")
    j = src.index("{", e)
    return src[m.start():bal(src, j, "{", "}")]


def returned_map(body):
    """Return {publicKey: localName} from the object literal containing
    `setMethod:`. The tail of createTmCalc is a comma expression
    `return k=…, E=…, {setSeq:R, …}`, so `return{` does not appear."""
    m = re.search(r"[{,]setMethod\s*:", body)
    if not m:
        return {}
    i = m.start()
    depth = 0
    while i >= 0:
        if body[i] in "}])":
            depth += 1
        elif body[i] in "{[(":
            if depth == 0:
                break
            depth -= 1
        i -= 1
    if i < 0 or body[i] != "{":
        return {}
    st = i
    en = bal(body, st, "{", "}")
    obj = body[st:en]
    out = {}
    for k in KEYS:
        mm = re.search(r"[{,]" + k + r":([A-Za-z_$][\w$]*)[,}]", obj)
        if mm:
            out[k] = mm.group(1)
    return out


def literal(body, local):
    """Extract `<local>={...}` object literal from the var declarations."""
    m = re.search(r"[,;]?\s*" + re.escape(local) + r"\s*=\s*\{", body)
    if not m:
        return None
    st = body.index("{", m.end() - 1)
    en = bal(body, st, "{", "}")
    return body[st:en] if en >= 0 else None


def main():
    tables = {}
    for build, path in PATHS.items():
        src = open(path).read()
        body = createTmCalc_body(src)
        mp = returned_map(body)
        print(f"{build}: createTmCalc len={len(body)} returned keys={sorted(mp)}")
        for k in KEYS:
            loc = mp.get(k)
            lit = literal(body, loc) if loc else None
            tables.setdefault(k, {})[build] = lit
            print(f"  {k} -> local {loc!r} literal len={len(lit) if lit else 'MISSING'}")

    print("\n=== SDM-engine table comparison (pairwise) ===")
    for k in KEYS:
        vals = tables[k]
        builds = list(PATHS)
        diffs = []
        for a, b in itertools.combinations(builds, 2):
            same = vals[a] == vals[b]
            diffs.append((a, b, same))
        print(f"\n{k}:")
        for a, b, same in diffs:
            print(f"  {a} vs {b}: byte_identical={same}")
        if any(not s for _, _, s in diffs):
            base = vals[builds[0]]
            for other in builds[1:]:
                if vals[other] != base:
                    pa = parse(base)
                    pb = parse(vals[other])
                    keys = set(pa) | set(pb)
                    dd = sorted(x for x in keys if pa.get(x) != pb.get(x))
                    print(f"  differing keys {builds[0]} vs {other}: {len(dd)} of {len(keys)}")
                    for x in dd[:10]:
                        print(f"    {x!r}: {builds[0]}={pa.get(x)} {other}={pb.get(x)}")


def parse(s):
    return {
        k: (dh, ds)
        for k, dh, ds in re.findall(
            r'"?([\w/\-]+)"?\s*:\s*\{dh:(-?[\d.]+),ds:(-?[\d.]+)\}', s or ""
        )
    }


if __name__ == "__main__":
    main()
