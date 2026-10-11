#!/usr/bin/env python3
"""Extract named JS functions from a beautified NEBaseChanger bundle.

Usage: extract.py <beauty.js> <anchor-regex> [<anchor-regex> ...]
Prints each matched function with its 1-based line range in the file.
"""
import re
import sys


def body_after_brace(text, start):
    """Return text[i:j+1] for the balanced {...} block beginning at/after start."""
    i = text.find("{", start)
    if i < 0:
        return None
    depth = 0
    in_str = None
    k = i
    while k < len(text):
        c = text[k]
        if in_str:
            if c == "\\":
                k += 2
                continue
            if c == in_str:
                in_str = None
        elif c in "\"'`":
            in_str = c
        elif c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return text[start : k + 1]
        k += 1
    return text[start : start + 8000]


def main():
    path = sys.argv[1]
    pats = sys.argv[2:]
    data = open(path, encoding="utf-8").read()
    lines = data.split("\n")

    def lineno(pos):
        return data.count("\n", 0, pos) + 1

    for pat in pats:
        for m in re.finditer(pat, data):
            blk = body_after_brace(data, m.start())
            if blk is None:
                continue
            a, b = lineno(m.start()), lineno(m.start() + len(blk))
            print(f"### {pat}  -> lines {a}..{b} ({len(blk)} chars)")
            print(blk)
            print()
            break


main()
