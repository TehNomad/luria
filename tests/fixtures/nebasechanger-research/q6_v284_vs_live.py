#!/usr/bin/env python3
"""Q6: verified v2.8.4 -> live (v2.11.0) deltas in the primer-design layer.

Every claim below is a regex/byte check against the two bundles, not a
reading of a diff.  Local only, no network.

Bundles:
  app-js/beautified/index.a5da1636.v2.8.4.beauty.js   (archived v2.8.4)
  live-js/beautified/index.4330b0c7.v2.11.0-live.beauty.js  (live v2.11.0)

Both are prettier-formatted from the minified originals; formatting does not
change tokens, so counts of identifiers and numeric literals are valid.
"""
import re
import sys

V284 = "app-js/beautified/index.a5da1636.v2.8.4.beauty.js"
LIVE = "live-js/beautified/index.4330b0c7.v2.11.0-live.beauty.js"
HELP = "../../../docs/research/nbc_help_v284.txt"


def load(path):
    with open(path, encoding="utf-8") as fh:
        return fh.read()


def main():
    a = load(V284)
    b = load(LIVE)

    print("== 1. dead-code removal: Primer.calculateSDMPrimers")
    for tag, src in (("v2.8.4", a), ("live", b)):
        hits = [m.start() for m in re.finditer(r"calculateSDMPrimers", src)]
        print(f"  {tag}: {len(hits)} occurrence(s)")
        for h in hits:
            line = src[:h].count("\n") + 1
            print(f"    line {line}: {' '.join(src[h-40:h+120].split())}")

    print()
    print("== 2. Fragment.autogen* option defaults")
    pat = re.compile(
        r"(Fragment\.prototype\.(?:autogenPrimers|autogenSDMPrimers)) = function"
    )
    for tag, src in (("v2.8.4", a), ("live", b)):
        for m in pat.finditer(src):
            name = m.group(1)
            head = src[m.start() : m.start() + 700]
            pairs = re.findall(r"\((\w+(?:\.\w+)*) = [\w.]+ \|\| ([^,)]+)", head)
            print(f"  {tag} {name}:")
            for k, v in pairs:
                print(f"    {k} -> {v}")
            # does it copy the caller's options object?
            print(
                "    Object.assign copy: "
                + ("yes" if "Object.assign({}, " in head else "no")
            )
            # which engine does it call?
            eng = re.findall(r"Primer\.calculate\w+", head)
            print(f"    engine called: {sorted(set(eng))}")

    print()
    print("== 3. three default sets inside Primer / Primer.calculatePrimers")
    for tag, src in (("v2.8.4", a), ("live", b)):
        m = re.search(r"Primer = function \(([^)]*)\)", src)
        if not m:
            print(f"  {tag}: Primer ctor not found")
            continue
        blk = src[m.start() : m.start() + 1400]
        lit = re.search(r"\{[^{}]*primerMethod[^{}]*\}", blk, re.S)
        print(f"  {tag} Primer ctor default object literal:")
        print(
            "    " + " ".join(lit.group(0).split()) if lit else "    (none)"
        )
        fb = re.findall(r"_modelPrimer\.(\w+) = [\w.]+ \|\| ([^,)]+)", blk)
        print(f"  {tag} _modelPrimer fallbacks:")
        for k, v in fb:
            print(f"    {k} -> {v}")

        m2 = re.search(r"Primer\.calculatePrimers = function", src)
        blk2 = src[m2.start() : m2.start() + 900]
        print(f"  {tag} calculatePrimers fallbacks:")
        for k, v in re.findall(r"\((\w+) = \1 \|\| ([^,)]+)", blk2):
            print(f"    {k} -> {v}")
        for k in ("primerMinLen", "primerMinTm", "primerMaxTm"):
            mm = re.search(rf"{k} \|\| (\d+)", blk2)
            if mm:
                print(f"    {k} -> {mm.group(1)}")

    print()
    print("== 4. in-app manual statements (verbatim, with byte offsets)")
    try:
        h = load(HELP)
    except FileNotFoundError:
        print("  nbc_help_v284.txt not found; skipping")
        return
    needles = [
        "at least 10 nucleotides that are 100% complementary",
        "preferred primer concentration",
        "corrected for buffer composition",
        "minimum primer length and minimum primer Tm with default values",
        "anchored in the center of the region",
    ]
    for n in needles:
        i = h.find(n)
        if i < 0:
            print(f"  NOT FOUND: {n!r}")
            continue
        s = max(0, i - 240)
        e = min(len(h), i + 320)
        print(f"  @{i}:")
        print("    " + " ".join(h[s:e].split()))


if __name__ == "__main__":
    sys.exit(main())
