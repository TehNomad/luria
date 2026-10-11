#!/usr/bin/env python3
"""Q11: in-app help/manual prose deltas, NEBaseChanger v2.8.4 -> live v2.11.0.

The in-app manual is not one string: Svelte compiles it to many small template
literals interleaved with <br>/<i>/<a> markup.  We therefore compare the *set*
of prose fragments, after stripping markup and collapsing whitespace, and pair
near-matches with difflib so wording changes are visible rather than implied.

Usage:  python3 q11_help_diff.py            # summary to stdout
        python3 q11_help_diff.py --full     # verbatim fragment pairs

Durable output: live-js/diffs/q11-help-prose-deltas.txt
"""
import re
import sys
import difflib

LIVE = "live-js/beautified/index.4330b0c7.v2.11.0-live.beauty.js"
V284 = "app-js/beautified/index.a5da1636.v2.8.4.beauty.js"
OUT = "live-js/diffs/q11-help-prose-deltas.txt"


def prose(path):
    """Prose template literals, markup stripped, whitespace collapsed."""
    s = open(path).read()
    out = []
    for m in re.finditer(r"`([^`]*)`", s):
        t = m.group(1)
        if len(t) < 20:
            continue
        c = re.sub(r"<[^>]{0,160}>", " ", t)
        c = re.sub(r"&[a-z]{2,6};", " ", c)
        c = re.sub(r"\s+", " ", c).strip()
        if len(c) < 20:
            continue
        words = [w for w in c.split() if re.search(r"[A-Za-z]", w)]
        if len(words) < 4:
            continue
        alpha = sum(ch.isalpha() for ch in c)
        if alpha / max(1, len(c)) < 0.55:
            continue
        out.append(c)
    return out


def main():
    L = prose(LIVE)
    V = prose(V284)
    ls, vs = set(L), set(V)
    uniqL = [t for t in L if t not in vs]
    uniqV = [t for t in V if t not in ls]
    print(f"live prose fragments={len(L)}  v2.8.4={len(V)}")
    print(f"only in live={len(uniqL)}  only in v2.8.4={len(uniqV)}")
    print("near-match pairs (live vs v2.8.4):")
    for a in uniqL:
        if not uniqV:
            break
        best = max(uniqV, key=lambda b: difflib.SequenceMatcher(None, a, b).ratio())
        r = difflib.SequenceMatcher(None, a, best).ratio()
        if r > 0.5:
            print(f"--- ratio={r:.2f}")
            print("LIVE:", a)
            print("V284:", best)
            uniqV = [b for b in uniqV if b != best]
        elif "--full" in sys.argv:
            print("--- live-only, no v2.8.4 counterpart")
            print("LIVE:", a)
    if "--full" in sys.argv:
        for b in uniqV:
            print("--- v2.8.4-only")
            print("V284:", b)


if __name__ == "__main__":
    main()
