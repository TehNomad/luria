#!/usr/bin/env python3
"""Cycle 4 (part 3): make the changed nndhds / bulge keys actually fire, and
measure the Tm difference they cause.

Why the earlier numeric run showed no difference: when compseq is omitted,
createTmCalc sets compseq = revcomp(seq), so rev(compseq) == seq and every
pairmap key is "XY/XY". The changed keys ("aa/tc", "ct/aa", "ta/at") need a
genuine mismatch, which is exactly the SDM situation (the primer carries the
mutation, its partner is the template complement).

This script constructs primer/compseq pairs whose aligned bigrams hit each
changed key, evaluates method-4 Tm in the live engine and in the live engine
with the key reverted to its v2.8.4 value, and reports the delta.

No network. Reads fixtures only.

Usage: python3 q4c_changed_keys.py
"""
import json
import re
import subprocess

LIVE = "live-capture/004eae75971a-index.4330b0c7.js"

CHANGED = {
    "nndhds": {"aa/tc": {"dh": -9.1, "ds": -24.0},
               "ct/aa": {"dh": -9.1, "ds": -24.0},
               "ta/at": {"dh": -7.2, "ds": -20.4}},
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


def build_module():
    src = open(LIVE).read()
    m = re.search(r"\bcreateTmCalc\s*=\s*function\s*\(", src)
    mm = re.search(r"\b([A-Za-z_$][\w$]{1,3})\.trim\b", src[m.start():m.start() + 500])
    ns = mm.group(1)
    parts = [f"var {ns} = {{}};"]
    for dep in ("trim", "rev", "revcomp", "comp", "dmsg"):
        mm = re.search(r"\b" + ns + r"\." + dep + r"\s*=", src)
        if not mm:
            raise SystemExit(f"missing {ns}.{dep}")
        v = src.index("=", mm.end() - 1) + 1
        if src.startswith("function", v):
            i = src.index("(", v)
            e = bal(src, i, "(", ")")
            j = src.index("{", e)
            parts.append(src[mm.start():bal(src, j, "{", "}") + 1])
        else:
            parts.append(src[mm.start():bal(src, v, "{", "}") + 1])
    parts.append(extract(src, ns, "createTmCalc") + ";")
    return "\n".join(parts), ns


def extract(src, ns, name):
    m = re.search(r"\b" + ns + r"\." + name + r"\s*=", src)
    v = src.index("=", m.end() - 1) + 1
    i = src.index("(", v)
    e = bal(src, i, "(", ")")
    j = src.index("{", e)
    return src[m.start():bal(src, j, "{", "}")]


def cases():
    """Primer/compseq pairs whose aligned bigram is each changed key.

    he = seq.split(""), de = compseq.split("").reverse().
    Key = he[Y]+he[Y+1] + "/" + de[Y]+de[Y+1].
    So choose D = rev(compseq) with the wanted pair at Y, and seq with the
    other half at Y.
    """
    out = []
    # Realistic SDM cases: template span T, primer P carrying a 2-nt
    # substitution. compseq = revcomp(T), so rev(compseq) == T and the
    # aligned bigram key is P[Y..Y+1] / T[Y..Y+1].
    T = "aaagtcaagtc aa gtcaagtc aa gtca".replace(" ", "")
    for old, new in (("tc", "aa"), ("aa", "tc"), ("ta", "at"), ("at", "ta")):
        i = T.find(old, 3)
        if i < 0:
            continue
        P = T[:i] + new + T[i + 2:]
        out.append([f"SDM subst {old}->{new} in {T}", P, revcomp(T)])
    return out


def revcomp(s):
    m = str.maketrans("acgtACGT", "tgcaTGCA")
    return s.translate(m)[::-1]


def run(revert):
    mod, ns = build_module()
    js = mod + f"""
{ns}.dmsg = function() {{}};
var cases = {json.dumps(cases())};
var res = [];
for (var i = 0; i < cases.length; i++) {{
  var lbl = cases[i][0], s = cases[i][1], cp = cases[i][2];
  var c = {ns}.createTmCalc({{seq: s, ct: 0.5, salt: 150, compseq: cp}});
"""
    if revert:
        js += f"""
  var p = {json.dumps(CHANGED)};
  for (var tk in p) {{ var tbl = c[tk]; for (var kk in p[tk]) tbl[kk] = p[tk][kk]; }}
"""
    js += f"""
  c.setMethod(4);
  var t = c.Tm();
  res.push([lbl, s, cp, t.tm, t.dh, t.ds]);
}}
console.log(JSON.stringify(res));
"""
    path = f"/tmp/neb_changed_keys.{'.patched' if revert else ''}js"
    open(path, "w").write(js)
    r = subprocess.run(["node", path], capture_output=True, text=True)
    if r.returncode != 0:
        raise SystemExit(f"node failed: {r.stderr[:900]}")
    return json.loads(r.stdout)


def main():
    a = run(False)
    b = run(True)
    print("=== changed-key cases: live engine vs live engine with v2.8.4 values ===")
    for x, y in zip(a, b):
        print(f"  {x[0]}: seq={x[1]} compseq={x[2]}")
        print(f"     live tm={x[3]:.4f}  reverted tm={y[3]:.4f}  delta={x[3]-y[3]:+.4f} C")
        print(f"     live dh={x[4]:.1f} ds={x[5]:.4f} | reverted dh={y[4]:.1f} ds={y[5]:.4f}")
        if x[3] == y[3]:
            print("     NOTE: key not exercised by this construction")


if __name__ == "__main__":
    main()
