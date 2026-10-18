#!/usr/bin/env python3
"""Expected luriaNative replacement designs under D033, derived without crates/core.

Restates D033 in plain Python and computes every Tm with Biopython, reusing
only the Tm and reverse-complement helpers of native-design.py (themselves
independent of crates/core). Writes ../native-replacement.tsv, read by
crates/core/tests/q5_primer_fixtures.rs.

Requires Biopython 1.88. Run from this directory:

    python3 native-replacement.py            rewrite ../native-replacement.tsv
    python3 native-replacement.py --check    compare, write nothing

D033, for a target region [s, e) of width w replaced by m bases, bound b, on a
linear template of length n, with t = max(w, m):

  t <= 6   INLINE, D030's geometry with the region in place of the
           substituted bases:
               B  = s - 10                 junction; forward 5' flank fixed
               v0 = max(10, b - 10)        forward 3' flank, start
               r0 = max(b, 15)             reverse length, start
               forward text = edited[B : e + v + (m - w)]
               forward site = template[B : e + v]
               reverse      = rc(template[B - r : B])
           Forward Tm basis: D030's hybrid when m == w; MATCHED when m != w,
           because primer and site are then different lengths and no
           position-by-position alignment exists (D033).

  t > 6    TAILS, D032's rule with the replacement text in place of the
           inserted text:
               forward anneals template[e : e + f], reverse rc(template[s - r : s])
               f0 = r0 = max(b, 15)
               forward tail = replacement[m//2 :]     (ceil(m/2) bases)
               reverse tail = rc(replacement[: m//2])
           Sizing uses the annealing portions only, matched basis; the
           whole-primer Tm is reported and never used for sizing.

Both paths: 55 C floor, G/C end below 63 C, forward first, 5 C balance,
Ta = min(72, round_half_up(lower Tm) + 1). Pair invariant: reverse end + w = forward start.
"""
import importlib.util
import os

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURES = os.path.dirname(HERE)

_spec = importlib.util.spec_from_file_location(
    "native_design", os.path.join(HERE, "native-design.py")
)
nd = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(nd)

FLOOR, CEILING, BALANCE = nd.FLOOR, nd.CEILING, nd.BALANCE
FLANK5 = nd.FLANK5
ACGT, rc, bio, ends_gc = nd.ACGT, nd.rc, nd.bio, nd.ends_gc

INLINE_MAX = 6


def grow(tm_at, text_at, start, available, side):
    """D030's floor-then-G/C growth. Returns (length, trace) or raises Stop."""
    trace = set()
    length = start
    while tm_at(length) < FLOOR:
        if length >= available:
            raise Stop(f"tm-floor {side}:{available}:{tm_at(length):.2f}")
        length += 1
        trace.add(f"{side}-floor-grew")
    floor_length = length
    while tm_at(length) < CEILING and length < available and not ends_gc(text_at(length)):
        length += 1
    if not ends_gc(text_at(length)):
        if length != floor_length:
            trace.add(f"{side}-gc-reverted")
        length = floor_length
    elif length != floor_length:
        trace.add(f"{side}-gc-extended")
    return length, trace


class Stop(Exception):
    pass


def balance(tmf, tmr, fwd_text, rev_text, f, r, avail3, avail5):
    """D030's balance step, growing whichever primer is colder."""
    trace = set()
    if tmf(f) - tmr(r) > BALANCE:
        while tmf(f) - tmr(r) > BALANCE and r < avail5:
            r += 1
            trace.add("balance-grew-rev")
        held, k = r, r
        if not ends_gc(rev_text(k)):
            while k < avail5 and tmr(k) < tmf(f) and not ends_gc(rev_text(k)):
                k += 1
            r = k if (ends_gc(rev_text(k)) and tmr(k) <= tmf(f)) else held
            if r == k:
                trace.add("balance-rev-gc")
    elif tmr(r) - tmf(f) > BALANCE:
        reverse_celsius = tmr(r)
        while reverse_celsius - tmf(f) > BALANCE and f < avail3:
            f += 1
            trace.add("balance-grew-fwd")
        held, k = f, f
        if not ends_gc(fwd_text(k)):
            while k < avail3 and tmf(k) < reverse_celsius and not ends_gc(fwd_text(k)):
                k += 1
            f = k if (ends_gc(fwd_text(k)) and tmf(k) <= reverse_celsius) else held
            if f == k:
                trace.add("balance-fwd-gc")
    return f, r, trace


def design(t, s, e, replacement, b):
    trace = set()
    n, w, m = len(t), e - s, len(replacement)
    inline = max(w, m) <= INLINE_MAX
    edited = t[:s] + replacement + t[e:]

    if inline:
        v0, r0 = max(10, b - FLANK5), max(b, 15)
        if w + FLANK5 + v0 + r0 > n:
            return {"expected": "error:NoPrimerDesignPossible",
                    "failure": f"pair:{w + FLANK5 + v0 + r0}>{n}"}, trace
        obstacles = []
        if s < FLANK5 + r0:
            obstacles.append(f"5':{s}<{FLANK5 + r0}")
        if n - e < v0:
            obstacles.append(f"3':{n - e}<{v0}")
        if obstacles:
            return {"expected": "error:NoPrimerDesignFits", "failure": " ".join(obstacles)}, trace
        B = s - FLANK5
        for i in range(B - r0, e + v0):
            if t[i] not in ACGT:
                obstacles.append(f"ambiguous:{i}={t[i]}")
        if obstacles:
            return {"expected": "error:NoPrimerDesignFits", "failure": " ".join(obstacles)}, trace

        avail3 = 0
        while e + avail3 < n and t[e + avail3] in ACGT:
            avail3 += 1
        avail5 = 0
        while B - avail5 - 1 >= 0 and t[B - avail5 - 1] in ACGT:
            avail5 += 1

        shift = m - w
        fwd = lambda v: edited[B:e + v + shift]
        site = lambda v: t[B:e + v]
        rev = lambda r: rc(t[B - r:B])

        def forward_pair(v):
            if m == w:
                return nd.forward_tm(fwd(v), site(v))
            # D033: different lengths, so no mismatch alignment exists.
            return bio(fwd(v), fwd(v)), "matched_complement"

        tmf = lambda v: forward_pair(v)[0]
        tmr = lambda r: bio(rev(r), rev(r))
        anneal_spans = None
    else:
        f0 = r0 = max(b, 15)
        if w + f0 + r0 > n:
            return {"expected": "error:NoPrimerDesignPossible",
                    "failure": f"pair:{w + f0 + r0}>{n}"}, trace
        obstacles = []
        if s < r0:
            obstacles.append(f"5':{s}<{r0}")
        if n - e < f0:
            obstacles.append(f"3':{n - e}<{f0}")
        if obstacles:
            return {"expected": "error:NoPrimerDesignFits", "failure": " ".join(obstacles)}, trace
        for i in list(range(s - r0, s)) + list(range(e, e + f0)):
            if t[i] not in ACGT:
                obstacles.append(f"ambiguous:{i}={t[i]}")
        if obstacles:
            return {"expected": "error:NoPrimerDesignFits", "failure": " ".join(obstacles)}, trace

        avail3 = 0
        while e + avail3 < n and t[e + avail3] in ACGT:
            avail3 += 1
        avail5 = 0
        while s - avail5 - 1 >= 0 and t[s - avail5 - 1] in ACGT:
            avail5 += 1

        v0 = f0
        fwd = lambda f: t[e:e + f]
        rev = lambda r: rc(t[s - r:s])
        tmf = lambda f: bio(fwd(f), fwd(f))
        tmr = lambda r: bio(rev(r), rev(r))
        anneal_spans = True

    try:
        v, grew = grow(tmf, fwd, v0, avail3, "3'")
        trace |= {x.replace("3'-", "fwd-") for x in grew}
        r, grew = grow(tmr, rev, r0, avail5, "5'")
        trace |= {x.replace("5'-", "rev-") for x in grew}
    except Stop as stop:
        return {"expected": "error:NoPrimerDesignFits", "failure": str(stop)}, trace

    v, r, grew = balance(tmf, tmr, fwd, rev, v, r, avail3, avail5)
    trace |= grew

    if inline:
        tf, basis = forward_pair(v)
        forward_text, reverse_text = fwd(v), rev(r)
        forward_span = f"{s - FLANK5}:{e + v}"
        reverse_span = f"{s - FLANK5 - r}:{s - FLANK5}"
        forward_tail = reverse_tail = "none"
        whole_f = whole_r = "none"
    else:
        tf, basis = bio(fwd(v), fwd(v)), "matched_complement"
        half = m // 2
        forward_tail, reverse_tail = replacement[half:], rc(replacement[:half])
        assert rc(reverse_tail) + forward_tail == replacement
        forward_text = forward_tail + fwd(v)
        reverse_text = reverse_tail + rev(r)
        forward_span = f"{e}:{e + v}"
        reverse_span = f"{s - r}:{s}"
        whole_f = f"{bio(forward_text, forward_text):.2f}"
        whole_r = f"{bio(reverse_text, reverse_text):.2f}"
        forward_tail = forward_tail or "none"
        reverse_tail = reverse_tail or "none"

    tr = tmr(r)
    met = abs(tf - tr) <= BALANCE
    if not met:
        trace.add("balance-unmet")
    return {
        "expected": "design",
        "path": "inline" if inline else "tails",
        "forward_span": forward_span,
        "reverse_span": reverse_span,
        "forward_tail": forward_tail,
        "reverse_tail": reverse_tail,
        "forward": forward_text,
        "reverse": reverse_text,
        "forward_tm": f"{tf:.2f}",
        "forward_tm_basis": basis,
        "reverse_tm": f"{tr:.2f}",
        "forward_whole_tm": whole_f,
        "reverse_whole_tm": whole_r,
        "annealing_temp": f"{nd.annealing_temp(tf, tr):.2f}",
        "balance_met": "yes" if met else "no",
        "failure": "none",
    }, trace


GC = "GCCGAGCGGCTCGCGGACGC"
AT = "ATTAAATATTTAAATTATAT"
MIX = "ATGGCTTGGAATCAGCAGCA"
SYNTHETIC = {
    "balanced": MIX + GC + MIX + GC + MIX,
    "at-upstream": AT + AT + MIX + GC + GC,
    "at-downstream": GC + GC + MIX + AT + AT,
    "short-40": MIX + GC,
    "with-n": MIX + GC + MIX[:5] + "N" + MIX[6:] + GC + MIX,
}

# case_id, template, s, e, replacement, bound, note
CASES = [
    ("rep-01", "balanced", 50, 53, "TTT", 15, "3 for 3: an equal-length substitution, the D030 path"),
    ("rep-02", "balanced", 50, 53, "TT", 15, "3 for 2: shorter, inline, matched Tm basis"),
    ("rep-03", "balanced", 50, 53, "TTTTT", 15, "3 for 5: longer, inline, matched Tm basis"),
    ("rep-04", "balanced", 50, 56, "TTTTTT", 15, "6 for 6: the inline threshold"),
    ("rep-05", "balanced", 50, 57, "TTTTTTT", 15, "7 for 7: past the threshold, split across tails"),
    ("rep-06", "balanced", 50, 53, "TTTTTTT", 15, "3 for 7: m decides the path, so tails"),
    ("rep-07", "balanced", 50, 60, "TT", 15, "10 for 2: w decides the path, so tails"),
    ("rep-08", "balanced", 50, 53, "CCGGCCGGCCGG", 15, "12 inserted bases split 6 and 6"),
    ("rep-09", "at-upstream", 50, 53, "TTT", 15, "AT-rich 5' side: the reverse primer cannot reach the floor"),
    ("rep-10", "at-downstream", 50, 53, "TTT", 15, "AT-rich 3' side: the forward primer grows"),
    ("rep-11", "balanced", 10, 13, "TTT", 15, "10 bases before the region; the inline pair needs 25"),
    ("rep-12", "short-40", 15, 26, "TTTTTTTTTTTT", 15, "40 bases cannot hold the tail-path pair"),
    ("rep-13", "with-n", 50, 53, "TTT", 15, "an N inside the starting reverse primer"),
    ("rep-14", "../example-construct/input/sid-1_cDNA.fa", 1209, 1212, "AAC", 15, "the q5p-01 edit, as a replacement"),
    ("rep-15", "../example-construct/input/sid-1_cDNA.fa", 1209, 1212, "AACCCGGG", 15, "a real template, tail path"),
]

HEADER = ["case_id", "template", "region", "replacement", "min_overlap", "expected", "path",
          "forward_span", "reverse_span", "forward_tail", "reverse_tail", "forward", "reverse",
          "forward_tm", "forward_tm_basis", "reverse_tm", "forward_whole_tm", "reverse_whole_tm",
          "annealing_temp", "balance_met", "failure", "note"]


def main():
    rows = ["\t".join(HEADER)]
    for case_id, name, s, e, replacement, b, note in CASES:
        if name in SYNTHETIC:
            template, column = SYNTHETIC[name], "seq:" + SYNTHETIC[name]
        else:
            template, column = nd.load(name), name
        out, trace = design(template, s, e, replacement, b)
        row = [case_id, column, f"{s}:{e}:{template[s:e]}", replacement, str(b), out["expected"]]
        row += [out.get(k, "none") for k in HEADER[6:20]]
        row += [out["failure"], note]
        rows.append("\t".join(row))
        shown = {k: v for k, v in out.items() if k not in ("forward", "reverse")}
        print(case_id, shown, sorted(trace))
    nd.emit(os.path.join(FIXTURES, "native-replacement.tsv"), "\n".join(rows) + "\n")


if __name__ == "__main__":
    main()
