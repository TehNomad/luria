#!/usr/bin/env python3
"""Research helper: rename short minifier locals to canonical names so that
two builds can be diffed apples-to-apples.

This is a comparison aid, NOT a semantics-preserving alpha-renamer: it renames
every identifier of length <= MAXLEN that is not preceded by '.', is not a JS
keyword, and is outside string/regex/template literals.  Property names, string
literals, numeric literals and long identifiers are left untouched.  Any
conclusion drawn from the diff is re-checked against the verbatim excerpts.

Usage: normalize_js.py <file> [--maxlen N]
"""
import re
import sys

KEYWORDS = set("""
var let const function return if else for while do break continue switch case
default new delete typeof in this of true false null undefined void instanceof
try catch finally throw class extends get set static yield await async import
from as export debugger with Object Array String Math Number Boolean JSON Error
Date RegExp Map Set Promise console Infinity NaN length charAt slice indexOf
split join replace replaceAll push pop forEach map filter reduce some every
find hasOwnProperty toUpperCase toLowerCase toString valueOf trim startsWith
endsWith padStart padEnd concat reverse sort includes indexof toFixed
""".split())


def mask(src):
    """Return (masked, mapping) where string/template/regex bodies are replaced
    by \\x00 markers so their contents are never renamed."""
    out = []
    store = []
    i = 0
    prev_sig = ""  # last non-space significant char, to disambiguate /
    while i < len(src):
        c = src[i]
        if c in "\"'`":
            j = i + 1
            while j < len(src):
                if src[j] == "\\":
                    j += 2
                    continue
                if src[j] == c:
                    break
                j += 1
            j = min(j + 1, len(src))
            store.append(src[i:j])
            out.append("\x00%d\x00" % (len(store) - 1))
            prev_sig = "x"
            i = j
            continue
        if c == "/":
            # regex literal only if previous significant token suggests a value
            if prev_sig in "=(!,:;[{&|?+" or prev_sig == "":
                j = i + 1
                ok = False
                while j < len(src):
                    if src[j] == "\\":
                        j += 2
                        continue
                    if src[j] == "[":
                        while j < len(src) and src[j] != "]":
                            if src[j] == "\\":
                                j += 2
                                continue
                            j += 1
                    elif src[j] == "/":
                        ok = True
                        break
                    elif src[j] == "\n":
                        break
                    j += 1
                if ok:
                    while j < len(src) and src[j].isalpha():
                        j += 1
                    store.append(src[i:j])
                    out.append("\x00%d\x00" % (len(store) - 1))
                    prev_sig = "x"
                    i = j
                    continue
        out.append(c)
        if not c.isspace():
            prev_sig = c
        i += 1
    return "".join(out), store


def main():
    path = sys.argv[1]
    maxlen = 3
    if "--maxlen" in sys.argv:
        maxlen = int(sys.argv[sys.argv.index("--maxlen") + 1])
    src = open(path, encoding="utf8", errors="replace").read()
    masked, store = mask(src)

    names = {}

    def repl(m):
        full = m.group(0)
        dot = full[:1]
        name = full[1:]
        if dot == "." or name in KEYWORDS or len(name) > maxlen:
            return full
        if name not in names:
            names[name] = "v%d_" % len(names)
        return dot + names[name]

    masked = re.sub(r"(?<![A-Za-z0-9_$\x00])(\.?)([A-Za-z_$][A-Za-z0-9_$]*)", repl, masked)

    def unmask(s):
        return re.sub(r"\x00(\d+)\x00", lambda m: store[int(m.group(1))], s)

    print(unmask(masked))


main()
