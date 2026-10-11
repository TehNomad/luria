"""Q2 of the cycle-5 list: enumerate the nndhds key-set difference.

Two copies of nndhds exist in every bundle:
  A. TmCalc.prototype.nndhds   (the class copy)
  B. the local table inside createTmCalc (the closure copy)

Print both key sets, their sizes, and the keys present in only one copy.
Run: python3 q7_nndhds_keys.py [bundle]
Default bundle: the live capture.
"""
import re
import sys

DEFAULT = "live-capture/004eae75971a-index.4330b0c7.js"


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


def parse_table(src, open_idx):
    """nndhds is a flat object of {dh:..,ds:..} values; keys are unquoted
    or quoted slash-pairs."""
    a, b = brace_span(src, open_idx)
    body = src[a : b + 1]
    keys = re.findall(r'[{,]\s*"?([a-z]{2}/[a-z]{2})"?\s*:\s*\{', body)
    return keys, body


def main():
    path = sys.argv[1] if len(sys.argv) > 1 else DEFAULT
    src = open(path, encoding="utf-8", errors="replace").read()
    print(f"bundle: {path}")

    copies = {}
    m = re.search(r"TmCalc\.prototype\.nndhds\s*=\s*\{", src)
    if m:
        keys, body = parse_table(src, m.end() - 1)
        copies["A TmCalc.prototype.nndhds"] = (keys, body)

    m = re.search(r"createTmCalc\s*=\s*function", src)
    if m:
        k = src.index("{", m.end())
        a, b = brace_span(src, k)
        body = src[a : b + 1]
        # the closure copy is assigned to a minified variable, so find it by
        # its key signature (it contains dash keys, the class copy does not)
        mm = re.search(r'(\w+)=\{"[a-z-]{2}/[a-z-]{2}":\{dh:', body)
        if mm:
            st = body.index("{", mm.start() + len(mm.group(1)) + 1)
            c, d = brace_span(body, st)
            tbl = body[c : d + 1]
            keys = re.findall(r'"([a-z-]{2}/[a-z-]{2})"', tbl)
            copies[f"B createTmCalc local (var {mm.group(1)})"] = (keys, tbl)

    for name, (keys, body) in copies.items():
        print(f"\n{name}: {len(keys)} keys, table {len(body)} chars")
        dupes = len(keys) - len(set(keys))
        if dupes:
            from collections import Counter

            print(f"  duplicate keys: "
                  f"{[k for k, c in Counter(keys).items() if c > 1]}")

    names = list(copies)
    if len(names) == 2:
        ka, kb = set(copies[names[0]][0]), set(copies[names[1]][0])
        only_a, only_b = sorted(ka - kb), sorted(kb - ka)
        print(f"\nonly in A ({len(only_a)}): {only_a}")
        print(f"only in B ({len(only_b)}): {only_b}")
        print(f"shared: {len(ka & kb)}")

        # value comparison for shared keys
        va = extract_values(copies[names[0]][1], ka & kb)
        vb = extract_values(copies[names[1]][1], ka & kb)
        diff = [k for k in sorted(ka & kb) if va.get(k) != vb.get(k)]
        print(f"\nshared keys with different values ({len(diff)}):")
        for k in diff:
            print(f"  {k}: A={va[k]}  B={vb[k]}")


def extract_values(body, keys):
    out = {}
    for k in keys:
        m = re.search(r'[{,]\s*"?%s"?\s*:\s*\{([^{}]*)\}' % re.escape(k), body)
        if m:
            out[k] = dict(
                (kk, float(vv))
                for kk, vv in re.findall(r"(\w+)\s*:\s*(-?[\d.]+)", m.group(1))
            )
    return out


if __name__ == "__main__":
    sys.exit(main())
