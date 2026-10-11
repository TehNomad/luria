#!/usr/bin/env python3
"""Cycle 4: quantify the effect of the thermodynamic-table changes.

Extracts, from a minified NEBaseChanger bundle, the namespace members that
`createTmCalc` depends on (trim / rev / revcomp / comp / dmsg) plus
`createTmCalc` itself, writes them as a standalone Node module, and evaluates
the SDM Tm (method 4, ct=0.5 uM, salt=150 mM) of fixed primer/compseq pairs.

Comparison A: live engine vs v2.8.4 engine (whole-engine drift).
Comparison B: live engine vs live engine with the changed nndhds / nnbulge
              keys reverted to their v2.8.4 values (table-only drift).

No network. Reads fixtures only; writes the generated module to
/tmp/neb_tm_mod.<build>.js for inspection.

Usage: python3 q4_tm_numeric.py
"""
import json
import re
import subprocess

BUILDS = {
    "live": "live-capture/004eae75971a-index.4330b0c7.js",
    "v2.8.4": "raw/index.a5da1636.v2.8.4.min.js",
}

# (label, primer seq, compseq or None => compseq = revcomp(seq))
# The first two are the SID-1 D404N primer pair recorded in
# docs/research/nebasechanger-polymerase-tm.md §5.3.
CASES = [
    ["D404N fwd primer", "GGGCTGATGAAACTGACAGCGAA", None],
    ["D404N rev primer", "AATCGCTTCAAGC", None],
    ["plain 24-mer", "AAGTCAAGTCAAGTCAAGTCAAGTCA", None],
    ["fwd primer vs 1-mismatch partner", "GGGCTGATGAAACTGACAGCGAA",
     "GGGCTGGTGAAACTGACAGCGAA"],
]

# nndhds / nnbulge keys that differ between v2.8.4 and live (from tm_tables_diff.py).
# Property names are the ones on the object returned by createTmCalc:
#   nndhds, loop, bulge  (nnbulge is exposed as `bulge`).
TABLE_PATCH = {
    "nndhds": {
        "aa/tc": {"dh": -9.1, "ds": -24.0},
        "ct/aa": {"dh": -9.1, "ds": -24.0},
        "ta/at": {"dh": -7.2, "ds": -20.4},
    },
    "bulge": {"25": {"dh": 0, "ds": -28.1}},
}


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


def extract(src, ns, name):
    m = re.search(r"\b" + ns + r"\." + name + r"\s*=", src)
    if not m:
        return None
    v = src.index("=", m.end() - 1) + 1
    while src[v] in " \t":
        v += 1
    if src.startswith("function", v):
        i = src.index("(", v)
        e = bal(src, i, "(", ")")
        j = src.index("{", e)
        return src[m.start():bal(src, j, "{", "}")]
    if src[v] in "{[":
        op, cl = ("{", "}") if src[v] == "{" else ("[", "]")
        return src[m.start():bal(src, v, op, cl)]
    k = v
    while k < len(src) and src[k] not in ",;":
        k += 1
    return src[m.start():k]


def build_module(build):
    src = open(BUILDS[build]).read()
    m = re.search(r"\bcreateTmCalc\s*=\s*function\s*\(", src)
    mm = re.search(r"\b([A-Za-z_$][\w$]{1,3})\.trim\b", src[m.start():m.start() + 500])
    ns = mm.group(1)
    parts = [f"var {ns} = {{}};"]
    for dep in ("trim", "rev", "revcomp", "comp", "dmsg"):
        t = extract(src, ns, dep)
        if t is None:
            raise SystemExit(f"missing {ns}.{dep} in {build}")
        parts.append(t + ";")
    parts.append(extract(src, ns, "createTmCalc") + ";")
    return "\n".join(parts), ns


def run(build, revert_tables=False):
    mod, ns = build_module(build)
    js = mod + f"""
{ns}.dmsg = function(){{}};
var cases = {json.dumps(CASES)};
var res = [];
for (var i = 0; i < cases.length; i++) {{
  var lbl = cases[i][0], s = cases[i][1], cp = cases[i][2];
  var c = {ns}.createTmCalc({{seq: s, ct: 0.5, salt: 150, compseq: cp}});
"""
    if revert_tables:
        js += f"""
  var p = {json.dumps(TABLE_PATCH)};
  for (var tk in p) {{
    var tbl = c[tk];
    for (var kk in p[tk]) tbl[kk] = p[tk][kk];
  }}
"""
    js += f"""
  c.setMethod(4);
  var t;
  try {{ t = c.Tm(); }} catch (err) {{ res.push([lbl, s, cp, "ERR", String(err) + " @ " + String(err.stack).split(String.fromCharCode(10))[1].trim()]); continue; }}
  res.push([lbl, s, cp, t.tm, t.dh, t.ds]);
}}
console.log(JSON.stringify(res));
"""
    path = f"/tmp/neb_tm_mod.{build}{'.patched' if revert_tables else ''}.js"
    open(path, "w").write(js)
    r = subprocess.run(["node", path], capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit(f"node failed for {build}: {r.stderr[:900]}")
    return json.loads(r.stdout)


def main():
    print("=== A. live engine vs v2.8.4 engine ===")
    live = run("live")
    old = run("v2.8.4")
    for a, b in zip(live, old):
        print(f"  {a[0]}: live={a[3]:.4f}  v2.8.4={b[3]:.4f}  delta={a[3]-b[3]:+.4f} C")
        print(f"     live dh={a[4]:.1f} ds={a[5]:.4f} | v2.8.4 dh={b[4]:.1f} ds={b[5]:.4f}")

    print("\n=== B. live engine, changed tables reverted to v2.8.4 values ===")
    patched = run("live", revert_tables=True)
    for a, b in zip(live, patched):
        print(f"  {a[0]}: live={a[3]:.4f}  patched={b[3]:.4f}  table-only delta={a[3]-b[3]:+.4f} C")
        if a[3] == b[3]:
            print("     (tables not exercised by this case)")


if __name__ == "__main__":
    main()
