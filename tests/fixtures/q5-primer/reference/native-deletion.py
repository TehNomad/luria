#!/usr/bin/env python3
"""Expected luriaNative deletion designs under D031, derived without crates/core.

Restates D031 in plain Python and computes every Tm with Biopython, reusing
only the Tm and reverse-complement helpers of native-design.py (which are
themselves independent of crates/core). Writes ../native-deletion.tsv, read by
crates/core/tests/q5_primer_fixtures.rs.

Requires Biopython 1.88. Run from this directory:

    python3 native-deletion.py            rewrite ../native-deletion.tsv
    python3 native-deletion.py --check    compare, write nothing

D031, for a deletion [s, e), w = e - s, bound b, linear template length n:
    f0 = r0 = max(b, 15)            starting lengths, both primers
    forward  = t[e : e + f]         5' end at e, reads rightward
    reverse  = rc(t[s - r : s])     5' end at s, reads leftward
    no design possible     if w + f0 + r0 > n
    no design fits         if s < r0 (5'), n - e < f0 (3'), an ambiguity code
                           in the starting pair, or a Tm floor unreachable
    sizing: D030 unchanged (floor 55, G/C end below 63, forward first,
            balance 5, Ta = min(72, min Tm + 3)); both Tms matched
    pair invariant: reverse end + w == forward start
"""
import importlib.util
import os

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURES = os.path.dirname(HERE)

_spec = importlib.util.spec_from_file_location("native_design", os.path.join(HERE, "native-design.py"))
nd = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(nd)

FLOOR, CEILING, BALANCE, TA_OFFSET, TA_CAP = nd.FLOOR, nd.CEILING, nd.BALANCE, nd.TA_OFFSET, nd.TA_CAP
ACGT, rc, bio, ends_gc = nd.ACGT, nd.rc, nd.bio, nd.ends_gc


def design(t, s, e, b):
    trace = set()
    n, w = len(t), e - s
    f0 = r0 = max(b, 15)
    if w + f0 + r0 > n:
        return {"expected": "error:NoPrimerDesignPossible", "failure": f"pair:{w + f0 + r0}>{n}"}, trace
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

    fwd = lambda f: t[e:e + f]
    rev = lambda r: rc(t[s - r:s])
    tmf = lambda f: bio(fwd(f), fwd(f))
    tmr = lambda r: bio(rev(r), rev(r))

    # Forward: floor, then G/C. (D030's loops, with the 3' flank replaced by
    # the whole forward length.)
    f = f0
    while tmf(f) < FLOOR:
        if f >= avail3:
            return {"expected": "error:NoPrimerDesignFits",
                    "failure": f"tm-floor 3':{avail3}:{tmf(f):.2f}"}, trace
        f += 1
        trace.add("fwd-floor-grew")
    f_floor = f
    while tmf(f) < CEILING and f < avail3 and not ends_gc(fwd(f)):
        f += 1
    if not ends_gc(fwd(f)):
        if f != f_floor:
            trace.add("fwd-gc-reverted")
        f = f_floor
    elif f != f_floor:
        trace.add("fwd-gc-extended")

    r = r0
    while tmr(r) < FLOOR:
        if r >= avail5:
            return {"expected": "error:NoPrimerDesignFits",
                    "failure": f"tm-floor 5':{avail5}:{tmr(r):.2f}"}, trace
        r += 1
        trace.add("rev-floor-grew")
    r_floor = r
    while tmr(r) < CEILING and r < avail5 and not ends_gc(rev(r)):
        r += 1
    if not ends_gc(rev(r)):
        if r != r_floor:
            trace.add("rev-gc-reverted")
        r = r_floor
    elif r != r_floor:
        trace.add("rev-gc-extended")

    tf, tr = tmf(f), tmr(r)
    if tf - tr > BALANCE:
        while tmf(f) - tmr(r) > BALANCE and r < avail5:
            r += 1
            trace.add("balance-grew-rev")
        r_bal, k = r, r
        if not ends_gc(rev(k)):
            while k < avail5 and tmr(k) < tf and not ends_gc(rev(k)):
                k += 1
            if ends_gc(rev(k)) and tmr(k) <= tf:
                r = k
                trace.add("balance-rev-gc")
            else:
                r = r_bal
    elif tr - tf > BALANCE:
        while tmr(r) - tmf(f) > BALANCE and f < avail3:
            f += 1
            trace.add("balance-grew-fwd")
        f_bal, k = f, f
        if not ends_gc(fwd(k)):
            while k < avail3 and tmf(k) < tr and not ends_gc(fwd(k)):
                k += 1
            if ends_gc(fwd(k)) and tmf(k) <= tr:
                f = k
                trace.add("balance-fwd-gc")
            else:
                f = f_bal

    tf, tr = tmf(f), tmr(r)
    met = abs(tf - tr) <= BALANCE
    if not met:
        trace.add("balance-unmet")
    assert (s - r) + r + w == e + 0 and s + w == e  # reverse end + w == forward start
    return {
        "expected": "design",
        "forward_span": f"{e}:{e + f}",
        "reverse_span": f"{s - r}:{s}",
        "forward_tm": f"{tf:.2f}",
        "reverse_tm": f"{tr:.2f}",
        "annealing_temp": f"{min(TA_CAP, min(tf, tr) + TA_OFFSET):.2f}",
        "balance_met": "yes" if met else "no",
        "forward": fwd(f),
        "reverse": rev(r),
        "failure": "none",
    }, trace


# The synthetic templates of crates/core/src/primer_design.rs's unit tests.
GC = "GCCGAGCGGCTCGCGGACGC"
AT = "ATTAAATATTTAAATTATAT"
MIX = "ATGGCTTGGAATCAGCAGCA"
SYNTHETIC = {
    "balanced": MIX + GC + MIX + GC + MIX,
    "at-upstream": AT + AT + MIX + GC + GC,
    "at-downstream": GC + GC + MIX + AT + AT,
    "short-40": (MIX + GC),
    "with-n": MIX + GC + MIX[:5] + "N" + MIX[6:] + GC + MIX,
    "at-head": AT + GC + GC + GC + GC,
    "at-tail": GC + GC + GC + GC + AT,
}

# case_id, template, s, e, bound, note
CASES = [
    ("del-01", "balanced", 50, 53, 15, "a codon from the middle of a balanced template"),
    ("del-02", "balanced", 45, 46, 15, "a single base"),
    ("del-03", "balanced", 30, 70, 15, "40 bases: no cap on width"),
    ("del-04", "balanced", 50, 53, 25, "bound 25 starts both primers at 25"),
    ("del-05", "at-upstream", 50, 53, 15, "AT-rich 5' side: the reverse primer grows for the floor"),
    ("del-06", "at-downstream", 50, 53, 15, "AT-rich 3' side: the forward primer grows for the floor"),
    ("del-07", "balanced", 10, 13, 15, "10 bases before the deletion; the reverse primer needs 15"),
    ("del-08", "balanced", 90, 93, 15, "7 bases after the deletion; the forward primer needs 15"),
    ("del-09", "short-40", 15, 26, 15, "40 bases cannot hold 11 + 15 + 15"),
    ("del-10", "with-n", 50, 53, 15, "an N inside the starting reverse primer"),
    ("del-11", "../example-construct/input/sid-1_cDNA.fa", 1209, 1212, 15, "the q5p-01 codon, deleted"),
    ("del-12", "at-head", 20, 23, 15, "only 20 AT bases 5' of the deletion: reverse floor unreachable"),
    ("del-13", "at-tail", 77, 80, 15, "only 20 AT bases 3' of the deletion: forward floor unreachable"),
]

HEADER = ["case_id", "template", "deletion", "min_overlap", "expected", "forward_span",
          "reverse_span", "forward_tm", "reverse_tm", "annealing_temp", "balance_met",
          "forward", "reverse", "failure", "note"]


def main():
    rows = ["\t".join(HEADER)]
    for case_id, name, s, e, b, note in CASES:
        if name in SYNTHETIC:
            t, template_column = SYNTHETIC[name], "seq:" + SYNTHETIC[name]
        else:
            t, template_column = nd.load(name), name
        out, trace = design(t, s, e, b)
        row = [case_id, template_column, f"{s}:{e}:{t[s:e]}", str(b), out["expected"]]
        row += [out.get(k, "none") for k in HEADER[5:13]]
        row += [out["failure"], note]
        rows.append("\t".join(row))
        print(case_id, {k: v for k, v in out.items() if k not in ("forward", "reverse")}, sorted(trace))
    nd.emit(os.path.join(FIXTURES, "native-deletion.tsv"), "\n".join(rows) + "\n")


if __name__ == "__main__":
    main()
