#!/usr/bin/env python3
"""Cycle-5 verification pass over the four minified bundles.

Re-checks, independently of the earlier cycles' scripts, the headline claims
behind objectives 1-3 and the two open questions about `Primer` defaults:

  V1  Ge/Ne.tm literals (ct, salt, polGroup, method) + the Phusion branch
  V2  createTmFunction nM conversion and the /4 for methods 1 and 5
  V3  TmCalc.init defaults, setCt / setMonosalt / setDisalt conversions
  V4  createTmCalc defaults (ct, salt, dmso, method)
  V5  every occurrence of the bare identifier `maxTm`
  V6  every occurrence of `primerMaxTm`, with the defineProperty offset
  V7  `new Primer(` / `Primer(` construction sites and whether options are passed
  V8  nndhds key sets: TmCalc.prototype copy vs createTmCalc copy

Nothing is written. Exit 0 = every check ran; verdicts are printed.
"""
import re
import ast
import json

BUILDS = {
    "v2.5.3": ("raw/index.17c7600d.v2.5.3.min.js", "ge"),
    "v2.7.2": ("raw/index.dfcf8292.v2.7.2.min.js", "Ee"),
    "v2.8.4": ("raw/index.a5da1636.v2.8.4.min.js", "Ge"),
    "live-v2.11.0": ("live-capture/004eae75971a-index.4330b0c7.js", "Ne"),
}


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


def jsobj(lit):
    j = re.sub(r'(?<!["A-Za-z0-9_.])([A-Za-z_][A-Za-z0-9_]*)\s*:', r'"\1":', lit)
    return ast.literal_eval(j)


def func_body(src, start):
    """Brace-balanced body of a `X=function...` starting at `start`."""
    j = src.index("{", start)
    return src[start:bal(src, j, "{", "}")]


def main():
    for b, (path, ns) in BUILDS.items():
        src = open(path, encoding="utf8").read()
        print("=" * 72)
        print(f"BUILD {b}  file={path}  bytes={len(src)}  app-namespace={ns}")

        # ---- V1 -----------------------------------------------------------------
        m = re.search(rf"(?<![A-Za-z0-9_]){ns}\.tm\s*=\s*function", src)
        if m:
            body = func_body(src, m.start())
            print(f"[V1] {ns}.tm @ {m.start()} len={len(body)}")
            print("     " + body)
            print(f"     occurrences of 'Phusion' inside body: {body.count('Phusion')}")
            # is the polGroup variable reassigned anywhere inside the body?
            mv = re.search(r"var\s+([A-Za-z_$][\w$]*)\s*=\s*[\"']Phusion", body)
            if mv:
                v = mv.group(1)
                uses = re.findall(rf"(?<![\w$.]){v}(?![\w$])", body)
                assigns = re.findall(rf"(?<![\w$.]){v}\s*=[^=]", body)
                print(f"     polGroup local = {v!r}: {len(uses)} mentions, "
                      f"{len(assigns)} assignment(s) -> "
                      f"{'CONSTANT, switch arm is dead' if len(assigns) == 1 else 'REASSIGNED'}")
        else:
            print(f"[V1] {ns}.tm NOT FOUND")

        # ---- V2 -----------------------------------------------------------------
        m = re.search(r"(?<![A-Za-z0-9_.])createTmFunction\s*=\s*function", src)
        if m:
            body = func_body(src, m.start())
            print(f"[V2] createTmFunction @ {m.start()} len={len(body)}")
            print("     " + body)
            print(f"     '/1e3' present: {'/1e3' in body}; '/4' present: "
                  f"{'/4' in body}; method guards: "
                  f"{sorted(set(re.findall(r'h===\\d+|\\(h===\\d+\\)', body)))}")
        else:
            print("[V2] createTmFunction NOT FOUND")

        # ---- V3 -----------------------------------------------------------------
        for name in ("init", "setCt", "setMonosalt", "setDisalt", "setMethod", "setDMSO"):
            mm = re.search(rf"prototype\.{name}\s*=\s*function", src)
            if mm:
                body = func_body(src, mm.start())
                print(f"[V3] prototype.{name} @ {mm.start()} len={len(body)}")
                print("     " + body)
            else:
                print(f"[V3] prototype.{name} NOT FOUND")

        # ---- V4 -----------------------------------------------------------------
        m = re.search(r"(?<![A-Za-z0-9_.])createTmCalc\s*=\s*function", src)
        if m:
            head = src[m.start():m.start() + 420]
            print(f"[V4] createTmCalc @ {m.start()} header:")
            print("     " + head[:400])
            d = re.search(r"var\s+([A-Za-z_$][\w$]*)\s*=\s*([0-9.]+)\s*\|\|\s*([0-9.]+)", head)
            print("     defaults seen in header: " +
                  ", ".join(f"{k}={v}" for k, v in
                            re.findall(r"([A-Za-z_$][\w$]*)\s*=\s*([A-Za-z0-9.]+)\|\|([A-Za-z0-9.]+)", head)))

        # ---- V5 -----------------------------------------------------------------
        occ = [mm.start() for mm in re.finditer(r"(?<![\w$.])maxTm(?![\w$])", src)]
        print(f"[V5] bare 'maxTm' occurrences: {len(occ)} at {occ}")
        for o in occ:
            print(f"     context @{o}: {src[max(0,o-90):o+60]!r}")

        # ---- V6 -----------------------------------------------------------------
        pm = [mm.start() for mm in re.finditer(r"(?<![\w$.])primerMaxTm(?![\w$])", src)]
        dp = [mm.start() for mm in re.finditer(r'defineProperty\([^)]*"primerMaxTm"', src)]
        print(f"[V6] 'primerMaxTm' occurrences: {len(pm)}; "
              f'defineProperty("primerMaxTm") at {dp}')
        for o in pm:
            ctx = src[max(0, o - 60):o + 40]
            if "i.maxTm" in ctx or "maxTm)" in ctx:
                print(f"     write candidate @{o}: {ctx!r}")

        # ---- V7 -----------------------------------------------------------------
        print("[V7] Primer construction sites:")
        for mm in re.finditer(r"new\s+[A-Za-z_$][\w$]*\.Primer\s*\(", src):
            arg = src[mm.end():mm.end() + 60]
            print(f"     @ {mm.start()}  arg head: {arg[:60]!r}")
        for mm in re.finditer(r"new\s+[A-Za-z_$][\w$]*\.BuilderPrimer\s*\(", src):
            arg = src[mm.end():mm.end() + 60]
            print(f"     BuilderPrimer @ {mm.start()}  arg head: {arg[:60]!r}")

        # ---- V8 -----------------------------------------------------------------
        proto = re.search(r"prototype\.nndhds\s*=\s*\{", src)
        proto_keys = None
        if proto:
            lit = src[proto.end() - 1:bal(src, proto.end() - 1, "{", "}")]
            proto_keys = set(jsobj(lit))
        loc = None
        for mm in re.finditer(r"nndhds\s*:\s*([A-Za-z_$][\w$]*)[,}]", src):
            loc = mm.group(1)
            break
        extra = None
        if loc:
            lm = re.search(rf"(?<![\w$.]){loc}\s*=\s*\{{", src)
            if lm:
                lit = src[lm.end() - 1:bal(src, lm.end() - 1, "{", "}")]
                loc_keys = set(jsobj(lit))
                print(f"[V8] TmCalc.prototype.nndhds keys={len(proto_keys)}  "
                      f"createTmCalc local {loc!r} keys={len(loc_keys)}")
                only_loc = sorted(loc_keys - proto_keys)
                only_proto = sorted(proto_keys - loc_keys)
                print(f"     only in createTmCalc copy ({len(only_loc)}): "
                      + " ".join(f"{k}={jsobj(lit)[k]}" for k in only_loc))
                print(f"     only in TmCalc.prototype copy ({len(only_proto)}): "
                      + " ".join(only_proto))
        print()


main()
