#!/usr/bin/env python3
"""Compare NEBaseChanger Tm / DMG lookup tables between the archived v2.8.4
bundle and the live build captured 2026-10-10.

Reads:  app-js/index.a5da1636.v2.8.4.js   (archive.org, version 2.8.4)
        live-capture/004eae75971a-index.4330b0c7.js  (live site)
Writes: stdout summary.
"""
import json
import re
import sys

APP = "app-js/index.a5da1636.v2.8.4.js"
LIVE = "live-capture/004eae75971a-index.4330b0c7.js"

NAMES = [
    "dSBr", "dSSa", "dHBr", "dHSa",   # Breslauer/SantaLucia delta-S / delta-H pairs
    "nnde", "nndhds", "nnloop", "nnbulge", "nntmm",  # Takahashi-Sagar DMG tables
    "mat",  # Needleman-Wunsch substitution scoring matrix
]


def balanced_end(s, i, op, cl):
    """Return the index just past the object that starts at s[i] == op."""
    depth = 0
    while i < len(s):
        if s[i] == op:
            depth += 1
        elif s[i] == cl:
            depth -= 1
            if depth == 0:
                return i + 1
        i += 1
    return -1


def pairs(s):
    """Parse {key: {dh:X,ds:Y}, ...} with unquoted or quoted keys."""
    return {
        k: (dh, ds)
        for k, dh, ds in re.findall(
            r'"?([\w/\-]+)"?\s*:\s*\{dh:(-?[\d.]+),ds:(-?[\d.]+)\}', s
        )
    }


def get_object(t, name):
    m = re.search(r"(?:\b|\.prototype\.)" + name + r"\s*=\s*\{", t)
    if not m:
        return None
    start = m.end() - 1
    end = balanced_end(t, start, "{", "}")
    return t[start:end] if end >= 0 else None


def main():
    app = open(APP).read()
    live = open(LIVE).read()
    for name in NAMES:
        a, b = get_object(app, name), get_object(live, name)
        if a is None or b is None:
            print(f"{name}: app={'ok' if a else 'MISSING'} live={'ok' if b else 'MISSING'}")
            continue
        same = a == b
        print(f"{name}: app_len={len(a)} live_len={len(b)} byte_identical={same}")
        if same:
            continue
        ja, jb = pairs(a), pairs(b)
        if not ja or not jb:
            ja, jb = json.loads(a), json.loads(b)
        keys = set(ja) | set(jb)
        diffs = sorted(k for k in keys if ja.get(k) != jb.get(k))
        print(f"  {len(diffs)} differing keys of {len(keys)}:")
        for k in diffs[:30]:
            print(f"    {k!r}: app={ja.get(k)} live={jb.get(k)}")
    complement_tables(app, live)


ALPHA = r"[ACGTUNSWMKRYVDHBacgtunswmkryvdhb \-]{20,40}"
PAIRED = re.compile(rf'"({ALPHA})"\s*,\s*[\w$]+\s*=\s*"({ALPHA})"')


def complement_tables(app, live):
    """Locate the paired src/dst alphabet strings used by comp/revcomp."""
    print("\n=== paired complement alphabets ===")
    for label, text in (("v2.8.4", app), ("live", live)):
        seen = {}
        for m in PAIRED.finditer(text):
            seen.setdefault((m.group(1), m.group(2)), []).append(m.start())
        for (src, dst), at in sorted(seen.items()):
            print(f"  {label}: {len(at)}x @ {at[:3]}  src={src} dst={dst}")
    print()
    for label, text in (("v2.8.4", app), ("live", live)):
        for m in PAIRED.finditer(text):
            src, dst = m.group(1), m.group(2)
            if "S" not in src or "W" not in src:
                continue
            print(f"  {label}: S->{dst[src.index('S')]} W->{dst[src.index('W')]}")


if __name__ == "__main__":
    main()
