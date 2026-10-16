#!/usr/bin/env python3
"""Reference behaviour for the Q5 SDM primer contract.

This file is NOT luria's primer-design algorithm and must not be read as one.
It is a transcription of the Q5 SDM primer path observed in the saved NEBaseChanger
research material, kept here so that every reference value in
`tests/fixtures/q5-primer/reference/` is reproducible rather than transcribed.
D008 and D011 state that NEBaseChanger behaviour is not the source of truth for
luria's algorithms; nothing in `crates/core` reads this file.

Self-contained: the Tm model it applies is inlined below, so the script runs
without the gitignored research corpus.

Transcribed structure of the REPLACE branch, in the script's names:

    pad   = 10                       # callsite constant `tt`; fixed left flank
    X     = max(minOverlap, 15)      # `ce = ce<15?15:ce`
    minTm = clamp(targetTm, 55, 72)  # `te=te<55?55:te; te=te>72?72:te`
    gcExt = 63                       # `ze=63`
    fwd   = template[s-pad : s] + replacement + template[e : e+O]
    rev   = revcomp(template[s-pad-R : s-pad])
    O     starts at pad, grows while Tm(fwd) < minTm
    R     starts at X, grows while Tm(rev) < minTm
    then a GC-terminal extension while Tm < gcExt and the primer does not end
    in G/C, then a revert to the last length that met minTm
    then a balance pass: the lower-Tm primer is extended while the gap exceeds
    5, and then while it is still below the other and does not end in G/C
    Ta    = min(round(Tm(fwd)), round(Tm(rev))) + 1, capped at 72

The window the reference tool designs against is the whole construct rotated so
that the forward primer starts at index 0; for a template with enough context on
both sides that rotation is indistinguishable from taking the span directly,
which is what this script does.
"""

import hashlib
import json
import math
import os

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURES = os.path.dirname(os.path.dirname(HERE))

# --- the reference Tm model, inlined -----------------------------------------
# NEBaseChanger's "method 4" at salt 150 mM and ct 0.5, as read from the saved
# material. Its salt assumption is what it is; the project has not adopted it.

NN_SA = {
    'aa': (-7.9, -22.2), 'tt': (-7.9, -22.2), 'at': (-7.2, -20.4), 'ta': (-7.2, -21.3),
    'ca': (-8.5, -22.7), 'tg': (-8.5, -22.7), 'gt': (-8.4, -22.4), 'ac': (-8.4, -22.4),
    'ct': (-7.8, -21.0), 'ag': (-7.8, -21.0), 'ga': (-8.2, -22.2), 'tc': (-8.2, -22.2),
    'cg': (-10.6, -27.2), 'gc': (-9.8, -24.4), 'gg': (-8.0, -19.9), 'cc': (-8.0, -19.9),
}
R_ = 1.987


def salt_ow(fgc, salt_m, div_m):
    s, i, d, a, n, r, p = -.911, 6.26, -48.2, 52.5, 3.92, 1.42, 8.31
    e = math.sqrt(div_m) / salt_m if salt_m > 0 else -1
    if 0 <= e < 0.22:
        return (4.29 * fgc - 3.95) * 1e-5 * math.log(salt_m) \
            + 9.4e-6 * math.log(salt_m) ** 2
    if 0.22 <= e < 6:
        n = 3.92 * (.843 - .352 * math.sqrt(salt_m) * math.log(salt_m))
        r = 1.42 * (1.279 - .00403 * math.log(salt_m) - .00803 * math.log(salt_m) ** 2)
        p = 8.31 * (.486 - .258 * math.log(salt_m) + .00525 * math.log(salt_m) ** 3)
    nn = None
    return (n + s * math.log(div_m)
            + fgc * (i + r * math.log(div_m))
            + .5 / (nn - 1) * (d + a * math.log(div_m) + p * math.log(div_m) ** 2)) * 1e-5


def tm_method4(seq, ct=0.5, salt=150.0, div=0.0):
    w = seq.lower().replace('u', 't')
    n = len(w)
    fgc = (w.count('g') + w.count('c')) / n
    a, p = -5.7, 200.0
    if w[0] in 'at':
        a += 6.9
        p += 2200
    if w[-1] in 'at':
        a += 6.9
        p += 2200
    dh = ds = 0.0
    for k in range(n - 1):
        h, s = NN_SA[w[k:k + 2]]
        dh += h
        ds += s
    sym = w == w.translate(str.maketrans('acgt', 'tgca'))[::-1]
    h = -1.4 if sym else 0.0
    primer_c = ct * 1e-6
    i = (dh * 1000.0 + p) / (a + h + ds + R_ * math.log(primer_c))
    m = i - 273.15
    salt_m = salt * 0.001
    i = 1.0 / (1.0 / i + salt_ow(fgc, salt_m, div * 0.001))
    return i - 273.15, m


TM_NAME = "reference method 4, salt 150 mM, ct 0.5 (NEBaseChanger)"


def revcomp(s: str) -> str:
    return s.translate(str.maketrans("ACGTacgt", "TGCAtgca"))[::-1]


def tm(seq: str) -> float:
    return tm_method4(seq)[0]


def digest(s: str) -> str:
    return hashlib.sha256(s.encode()).hexdigest()[:16]


def load_fa(p: str) -> str:
    with open(p) as fh:
        body = fh.read().split("\n", 1)[1]
    return "".join(body.split()).upper()


def design(template: str, start: int, original: str, replacement: str,
           min_overlap: int, target_tm: float, pad: int = 10) -> dict:
    """The reference Q5 SDM REPLACE design. Coordinates are zero-based half-open."""
    end = start + len(original)
    assert template[start:end] == original, (start, original, template[start:end])

    x = max(min_overlap, 15)
    min_tm = min(max(target_tm, 55.0), 72.0)
    gc_ext = 63.0

    fwd_start = start - pad
    if fwd_start < 0:
        raise SystemExit("not enough left context for the reference pad")

    def fwd(o: int) -> str:
        return template[fwd_start:start] + replacement + template[end:end + o]

    def rev(r: int) -> str:
        return revcomp(template[fwd_start - r:fwd_start])

    o = pad
    f = fwd(o)
    t_f = tm(f)
    best_o = o
    while t_f < min_tm:
        o += 1
        f = fwd(o)
        t_f = tm(f)
        best_o = o
    while t_f < gc_ext and f[-1] not in "GCgc":
        o += 1
        f = fwd(o)
        t_f = tm(f)
    if f[-1] not in "GCgc":
        o = best_o
        f = fwd(o)
        t_f = tm(f)

    r = x
    g = rev(r)
    t_r = tm(g)
    best_r = r
    while t_r < min_tm:
        r += 1
        g = rev(r)
        t_r = tm(g)
        best_r = r
    while t_r < gc_ext and g[-1] not in "GCgc":
        r += 1
        g = rev(r)
        t_r = tm(g)
    if g[-1] not in "GCgc":
        r = best_r
        g = rev(r)
        t_r = tm(g)

    while t_f - t_r > 5:
        r += 1
        g = rev(r)
        t_r = tm(g)
    best_r = r
    while t_r < t_f and g[-1] not in "GCgc":
        r += 1
        g = rev(r)
        t_r = tm(g)
        if t_r < t_f:
            best_r = r
        else:
            g = rev(best_r)
            t_r = tm(g)
            r = best_r
            break

    while t_r - t_f > 5:
        o += 1
        f = fwd(o)
        t_f = tm(f)
    best_o = o
    while t_f < t_r and f[-1] not in "GCgc":
        o += 1
        f = fwd(o)
        t_f = tm(f)
        if t_f < t_r:
            best_o = o
        else:
            f = fwd(best_o)
            t_f = tm(f)
            o = best_o
            break

    fwd_span = (fwd_start, end + o)
    rev_span = (fwd_start - r, fwd_start)

    # ---------------------------------------------------------------------------
    # Decomposition. Everything below is derived from the construction above, not
    # inferred from the primer text, and not reported by the raw reference artifact:
    # raw/sdmsummary.txt reports primer sequence, Length, Tm and Ta only. It has no
    # overlap column, no span, no orientation, no tail/annealing split.
    #
    # The reference REPLACE path builds the forward primer as
    #     template[fwd_start:start] + replacement + template[end:end+o]
    # so its parts are known exactly. The reverse primer carries no mutated base.
    # ---------------------------------------------------------------------------
    left = template[fwd_start:start]
    right = template[end:end + o]
    assert left + replacement + right == f, "decomposition must rebuild the primer"
    assert len(left) == pad and len(right) == o

    edited = template[:start] + replacement + template[end:]
    fwd_edited_span = (fwd_start, fwd_start + len(f))
    assert edited[fwd_edited_span[0]:fwd_edited_span[1]] == f, "H3: text and coordinates must agree"

    # The template bases underneath each primer, in primer order.
    fwd_region = template[fwd_span[0]:fwd_span[1]]
    rev_region = revcomp(template[rev_span[0]:rev_span[1]])
    assert g == rev_region, "H3/I-1: reverse primer is the reverse complement of its span"

    # Primer positions that carry the mutation. For the forward primer these are
    # the replacement positions; the reverse primer has none.
    fwd_changed = list(range(pad, pad + len(original)))
    rev_changed: list[int] = []

    def anneal_idx(changed: list[int], n: int) -> list[int]:
        return [i for i in range(n) if i not in set(changed)]

    fwd_anneal = anneal_idx(fwd_changed, len(f))
    rev_anneal = anneal_idx(rev_changed, len(g))

    # The contract's quantity: overlap counts only annealing positions (H8, I-13).
    fwd_anneal_overlap = sum(1 for i in fwd_anneal if f[i] == fwd_region[i])
    rev_anneal_overlap = sum(1 for i in rev_anneal if g[i] == rev_region[i])

    # The quantity the existing script computed before this correction: a
    # position-by-position count over the WHOLE primer, mutated positions included.
    fwd_whole_match = sum(1 for a, b in zip(f, fwd_region) if a == b)
    rev_whole_match = sum(1 for a, b in zip(g, rev_region) if a == b)

    fwd_annealing_text = "".join(f[i] for i in fwd_anneal)
    rev_annealing_text = "".join(g[i] for i in rev_anneal)
    assert fwd_annealing_text == left + right, "annealing text must be the wild-type context"
    assert rev_annealing_text == g, "the reverse primer carries no mutated base"

    # H8: annealing and the mutation positions partition the primer.
    for name, seq, changed, anneal in (("forward", f, fwd_changed, fwd_anneal),
                                       ("reverse", g, rev_changed, rev_anneal)):
        assert len(anneal) + len(changed) == len(seq), f"{name}: H8 partition"
        assert sorted(anneal + changed) == list(range(len(seq))), f"{name}: H8 partition"

    ta = min(round(t_f), round(t_r)) + 1
    ta = min(ta, 72)

    def annealing_spans(span, changed: list[int], n: int):
        """Template spans of the annealing runs, derived from the primer's own parts.
        A primer read on the opposite strand covers one contiguous template region
        whose bases are the complement of the region, so its annealing run is the
        whole span."""
        if not changed:
            return [list(span)]
        runs, s = [], span[0]
        for i in range(n):
            if i in set(changed):
                if span[0] + i > s:
                    runs.append([s, span[0] + i])
                s = span[0] + i + 1
        if s < span[1]:
            runs.append([s, span[1]])
        return runs

    fwd_parts = [
        {"role": "context", "sequence": left, "span": [fwd_start, start],
         "primerIndices": [0, pad], "annealing": True},
        {"role": "mutation", "sequence": replacement, "span": [start, end],
         "primerIndices": [pad, pad + len(original)], "annealing": False},
        {"role": "context", "sequence": right, "span": [end, end + o],
         "primerIndices": [pad + len(original), len(f)], "annealing": True},
    ]
    rev_parts = [
        {"role": "context", "sequence": g, "span": list(rev_span),
         "primerIndices": [0, len(g)], "annealing": True},
    ]

    def build(part: str, seq: str, span, edited_span, region: str,
              changed: list[int], anneal: list[int], parts, whole_match: int,
              t: float) -> dict:
        return {
            "sequence": seq,
            "length": len(seq),
            "span": list(span),
            "oneBasedSpan": [span[0] + 1, span[1]],
            "editedSpan": list(edited_span) if edited_span else None,
            "orientation": "same" if part == "forward" else "opposite",
            "mutationPlacement": "inline" if changed else "none",
            "parts": parts,
            "tail": None,
            "tailReason": (
                "no 5' tail: the mutated positions sit at primer indices "
                f"{changed[0]}..{changed[-1] + 1}, not at the 5' end" if changed
                else "carries no mutated base: the whole primer is wild-type context"),
            "changedPositions": changed,
            "annealing": {
                "sequence": "".join(seq[i] for i in anneal),
                "spans": annealing_spans(span, changed, len(seq)),
                "length": len(anneal),
            },
            "annealingOverlap": sum(1 for i in anneal if seq[i] == region[i]),
            "wholePrimerMatchCount": whole_match,
            "tm": round(t, 2),
        }

    forward = build("forward", f, fwd_span, fwd_edited_span, fwd_region,
                    fwd_changed, fwd_anneal, fwd_parts, fwd_whole_match, t_f)
    reverse = build("reverse", g, rev_span, None, rev_region,
                    rev_changed, rev_anneal, rev_parts, rev_whole_match, t_r)

    h1 = {
        "minAnnealingOverlapSupplied": min_overlap,
        "forwardAnnealingOverlap": fwd_anneal_overlap,
        "reverseAnnealingOverlap": rev_anneal_overlap,
        "satisfied": (fwd_anneal_overlap >= min_overlap
                      and rev_anneal_overlap >= min_overlap),
        "forward": ("above" if fwd_anneal_overlap > min_overlap
                    else "at" if fwd_anneal_overlap == min_overlap else "below"),
        "reverse": ("above" if rev_anneal_overlap > min_overlap
                    else "at" if rev_anneal_overlap == min_overlap else "below"),
    }

    return {
        "request": {
            "minOverlapSupplied": min_overlap,
            "targetTm": target_tm,
            "pad": pad,
            "referenceParameterSlot": (
                "the reference tool's *minimum primer length*. The help text "
                "restricts that parameter to the annealing portion only when "
                "modifications are constrained to the 5' tails (13.1 item 5); "
                "these designs are inline, so the restriction does not apply "
                "and the script feeds the supplied value into that slot "
                "unchanged"),
            "contractField": (
                "minAnnealingOverlap (2.2). The two quantities are NOT established "
                "to be the same (15, question 4); the value is passed through "
                "unchanged and the comparison is reported separately."),
        },
        "edit": {"start": start, "end": end,
                 "original": original, "replacement": replacement},
        "forward": forward,
        "reverse": reverse,
        "relationship": {
            "abut": rev_span[1] == fwd_span[0],
            "boundary": fwd_span[0],
            "gap": fwd_span[0] - rev_span[1],
            "boundaryOffsetFromEdit": start - fwd_span[0],
            "contractGeometry": (
                "5 requires the pair's inner boundary at the edit; this design puts "
                f"it {start - fwd_span[0]} bases before it (inline placement)"),
        },
        "h1": h1,
        "ta": ta,
        "tmFunction": TM_NAME,
    }


# ---------------------------------------------------------------------------
# Self-check: the derived fields must be consistent with the primer text and
# with the template. These are the fixture-level forms of H3, H4, H8, I-1, I-6,
# I-7 and I-13. A failure here means the record contradicts itself, not that a
# luria algorithm is wrong.
# ---------------------------------------------------------------------------


def check(template: str, d: dict) -> None:
    for role, primer in (("forward", d["forward"]), ("reverse", d["reverse"])):
        seq = primer["sequence"]
        s, e = primer["span"]
        assert e - s == primer["length"] == len(seq), f"{role}: length vs span"

        # parts must rebuild the primer, in order, with no gaps or overlap
        runs = [p["primerIndices"] for p in primer["parts"]]
        assert runs[0][0] == 0 and runs[-1][1] == len(seq), f"{role}: parts must cover the primer"
        for i in range(1, len(runs)):
            assert runs[i][0] == runs[i - 1][1], f"{role}: parts must abut"
        assert "".join(p["sequence"] for p in primer["parts"]) == seq, f"{role}: parts rebuild sequence"

        # every context part must equal the template under it
        for p in primer["parts"]:
            if p["annealing"]:
                under = revcomp(template[p["span"][0]:p["span"][1]]) if role == "reverse" else template[p["span"][0]:p["span"][1]]
                assert p["sequence"] == under, f"{role}: context part must equal the template under it"
        mut = [p for p in primer["parts"] if not p["annealing"]]
        for p in mut:
            assert p["sequence"] == d["edit"]["replacement"], f"{role}: mutation part"

        # H3 clause 1: primer text equals the edited sequence at its edited span
        if primer["editedSpan"]:
            es, ee = primer["editedSpan"]
            edited = template[:d["edit"]["start"]] + d["edit"]["replacement"] + template[d["edit"]["end"]:]
            assert edited[es:ee] == seq, f"{role}: H3 editedSpan must locate the primer"
            assert ee - es == len(seq), f"{role}: H3 editedSpan width"

        # I-1 / H3 clause 2: the reverse primer's annealing is the reverse
        # complement of the template region its annealing spans cover.
        anneal = primer["annealing"]
        covered = "".join(template[a:b] for a, b in anneal["spans"])
        want = revcomp(covered) if role == "reverse" else covered
        assert anneal["sequence"] == want, f"{role}: I-1 annealing must be reconstructible from its spans"
        assert sum(b - a for a, b in anneal["spans"]) == anneal["length"] == len(anneal["sequence"]), f"{role}: annealing length"

        # H8 / I-13: annealing and the mutated positions partition the primer,
        # and the reported overlap counts only annealing positions.
        changed = primer["changedPositions"]
        assert anneal["length"] + len(changed) == len(seq), f"{role}: H8 partition"
        region = revcomp(template[s:e]) if role == "reverse" else template[s:e]
        anneal_idx = [i for i in range(len(seq)) if i not in set(changed)]
        assert primer["annealingOverlap"] == sum(1 for i in anneal_idx if seq[i] == region[i]), f"{role}: I-13"
        assert primer["wholePrimerMatchCount"] == sum(1 for a, b in zip(seq, region) if a == b), f"{role}: whole-primer count"
        assert primer["annealingOverlap"] <= primer["wholePrimerMatchCount"], f"{role}: annealing <= whole"

        # H4: an orientation is stated, and the two primers' are opposite
        assert primer["orientation"] in ("same", "opposite"), f"{role}: H4 orientation"

    assert d["forward"]["orientation"] != d["reverse"]["orientation"], "H4: orientations must be opposite"

    # H2: the pair covers the changed region of its edit. The mutated template
    # span must be the edit's changed region, and must lie inside the pair's
    # covered span.
    s, e = d["edit"]["start"], d["edit"]["end"]
    mut_spans = [p["span"] for p in d["forward"]["parts"] if not p["annealing"]]
    assert mut_spans == [[s, e]], "H2: the mutated part must be exactly the changed region"
    covered = [p["span"] for pr in (d["forward"], d["reverse"]) for p in pr["parts"]]
    lo = min(a for a, _ in covered); hi = max(b for _, b in covered)
    assert lo <= s and e <= hi, "H2: the changed region must lie inside the pair's covered span"

    # H5: the pair's relationship to the edit is stated, not implied.
    rel = d["relationship"]
    assert {"abut", "boundary", "gap"} <= set(rel), "H5: relationship must be stated"
    assert rel["gap"] == rel["boundary"] - d["reverse"]["span"][1], "H5: gap must be derivable from the spans"
    assert rel["abut"] is (d["reverse"]["span"][1] == d["forward"]["span"][0]), "H5: abut must be derivable"
    assert d["builtAgainst"] == digest(template), "I-6: builtAgainst must name the template"
    assert d["h1"]["forwardAnnealingOverlap"] == d["forward"]["annealingOverlap"], "h1 must use the recorded quantity"
    assert d["h1"]["reverseAnnealingOverlap"] == d["reverse"]["annealingOverlap"], "h1 must use the recorded quantity"


def main() -> None:
    sid = load_fa(f"{FIXTURES}/example-construct/input/sid-1_cDNA.fa")
    mut_a = load_fa(f"{FIXTURES}/mutation/mut-a.txt")

    out = {}

    for key, min_overlap in (("q5p-01", 15), ("q5p-02", 24), ("q5p-03", 25)):
        out[key] = design(sid, 1209, "GAT", "AAC", min_overlap, 55.0)

    out["q5p-04"] = {
        "note": "same design as q5p-01; this case is about the overlap count",
        "design": dict(out["q5p-01"]),
    }

    a = design(sid, 1209, "GAT", "AAC", 15, 55.0)
    b = design(sid, 1503, "CTG", "GAG", 15, 55.0)
    out["q5p-07"] = {
        "note": "two independent variants against one pristine template",
        "variant-a": a,
        "variant-b": b,
        "spansDisjoint": a["forward"]["span"][1] <= b["reverse"]["span"][0],
    }

    # Which fields the raw reference artifact actually reports, and which this
    # script derives. raw/sdmsummary.txt has columns ID, Description, Fwd Primer
    # ID, Fwd Primer Seq, Length, Tm Fwd, Rev Primer ID, Rev Primer Seq, Length,
    # Tm Rev, Ta. It reports no span, no orientation, no overlap, no tail/annealing
    # split. Only the D404N row corresponds to a design in this file.
    neb_row = "example-construct/raw/sdmsummary.txt (row D404N)"
    reported = ["forward.sequence", "forward.length", "reverse.sequence",
                "reverse.length", "ta"]
    derived = ["span", "oneBasedSpan", "editedSpan", "orientation", "parts",
               "tail", "tailReason", "changedPositions", "annealing",
               "annealingOverlap", "wholePrimerMatchCount", "tm", "h1"]

    def provenance(d, row):
        d["provenance"] = {
            "reportedBy": row or None,
            "reportedFields": reported if row else [],
            "reportedTm": ("74 (fwd), 70 (rev) as printed; this script computes "
                           "74.18 and 69.94 before rounding" if row else None),
            "derivedFields": derived,
            "note": ("raw/sdmsummary.txt reports no overlap column: both overlap "
                     "quantities here are derived by this script, not observed"),
        }

    for key in out:
        v = out[key]
        targets = [v] if "forward" in v else (
            [v["design"]] if "design" in v else [v["variant-a"], v["variant-b"]])
        for d in targets:
            row = neb_row if (key in ("q5p-01", "q5p-04") or
                              (key == "q5p-07" and d is v["variant-a"])) else None
            provenance(d, row)

    for v in out.values():
        for d in (v.get("design"), v.get("variant-a"), v.get("variant-b"), v):
            if d and "forward" in d:
                d["builtAgainst"] = digest(sid)
                check(sid, d)

    with open(f"{FIXTURES}/q5-primer/reference/designs.json", "w") as fh:
        fh.write(json.dumps(out, indent=2) + "\n")

    def show(tag, d):
        h = d["h1"]
        print(f"{tag}: bound={h['minAnnealingOverlapSupplied']} "
              f"fwd annealing={d['forward']['annealingOverlap']} ({h['forward']}) "
              f"rev annealing={d['reverse']['annealingOverlap']} ({h['reverse']}) "
              f"H1={'satisfied' if h['satisfied'] else 'VIOLATED'} "
              f"| whole-primer match counts fwd={d['forward']['wholePrimerMatchCount']} "
              f"rev={d['reverse']['wholePrimerMatchCount']} "
              f"| lengths fwd={d['forward']['length']} rev={d['reverse']['length']} "
              f"| ta={d['ta']} abut={d['relationship']['abut']}")

    for k in ("q5p-01", "q5p-02", "q5p-03"):
        show(k, out[k])
    show("q5p-04", out["q5p-04"]["design"])
    show("q5p-07a", out["q5p-07"]["variant-a"])
    show("q5p-07b", out["q5p-07"]["variant-b"])
    v = out["q5p-04"]["design"]
    print("q5p-04 forward: annealingOverlap", v["forward"]["annealingOverlap"],
          "vs wholePrimerMatchCount", v["forward"]["wholePrimerMatchCount"],
          "vs span width", v["forward"]["length"],
          "-- the shared middle base is a mutated position, so it is not annealing")
    v = out["q5p-07"]
    print("q5p-07 spansDisjoint", v["spansDisjoint"],
          "a fwd span", v["variant-a"]["forward"]["span"],
          "b rev span", v["variant-b"]["reverse"]["span"])
    print("mut-a.txt:", mut_a, "len", len(mut_a), "digest", digest(mut_a))
    print("sid-1 digest:", digest(sid))


main()
