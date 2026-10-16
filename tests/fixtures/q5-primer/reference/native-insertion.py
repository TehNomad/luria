#!/usr/bin/env python3
"""Expected luriaNative insertion designs under D032, derived without crates/core.

Restates D032 in plain Python and computes every Tm with Biopython, reusing
only the Tm and reverse-complement helpers of native-design.py (themselves
independent of crates/core). Writes ../native-insertion.tsv, read by
crates/core/tests/q5_primer_fixtures.rs.

Requires Biopython 1.88. Run from this directory:

    python3 native-insertion.py

D032, for an insertion of k bases at point p, bound b, linear template of
length n:
    f0 = r0 = max(b, 15)            starting annealing lengths, both primers
    forward anneals to t[p : p + f], reverse to rc(t[p - r : p])
    k <= 6:  forward tail = the whole insertion, reverse tail = ""
    k >  6:  forward tail = inserted[floor(k/2):]      (ceil(k/2) bases)
             reverse tail = rc(inserted[:floor(k/2)])
    primer = tail + annealing portion, 5' -> 3'
    no design possible     if f0 + r0 > n
    no design fits         if p < r0 (5'), n - p < f0 (3'), an ambiguity code
                           in the starting annealing portions, or a Tm floor
                           unreachable
    sizing uses the ANNEALING portion's Tm only (matched basis), D030
    otherwise unchanged: floor 55, G/C end below 63, forward first, balance 5,
    Ta = min(72, min Tm + 3)
    whole-primer Tm is also reported, labelled, and never used for sizing
    a primer longer than 100 nt gets a quality warning, never a refusal
"""
import importlib.util
import math
import os

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURES = os.path.dirname(HERE)

_spec = importlib.util.spec_from_file_location(
    "native_design", os.path.join(HERE, "native-design.py")
)
nd = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(nd)

FLOOR, CEILING, BALANCE = nd.FLOOR, nd.CEILING, nd.BALANCE
TA_OFFSET, TA_CAP = nd.TA_OFFSET, nd.TA_CAP
ACGT, rc, bio, ends_gc = nd.ACGT, nd.rc, nd.bio, nd.ends_gc

TAIL_WHOLE_MAX = 6
LENGTH_WARN = 100


def tails(inserted):
    """(forward tail, reverse tail) under D032's split rule."""
    k = len(inserted)
    if k <= TAIL_WHOLE_MAX:
        return inserted, ""
    half = k // 2
    return inserted[half:], rc(inserted[:half])


def design(t, p, inserted, b):
    trace = set()
    n, k = len(t), len(inserted)
    f0 = r0 = max(b, 15)
    fwd_tail, rev_tail = tails(inserted)
    if f0 + r0 > n:
        return {
            "expected": "error:NoPrimerDesignPossible",
            "failure": f"pair:{f0 + r0}>{n}",
        }, trace
    obstacles = []
    if p < r0:
        obstacles.append(f"5':{p}<{r0}")
    if n - p < f0:
        obstacles.append(f"3':{n - p}<{f0}")
    if obstacles:
        return {"expected": "error:NoPrimerDesignFits", "failure": " ".join(obstacles)}, trace
    for i in list(range(p - r0, p)) + list(range(p, p + f0)):
        if t[i] not in ACGT:
            obstacles.append(f"ambiguous:{i}={t[i]}")
    if obstacles:
        return {"expected": "error:NoPrimerDesignFits", "failure": " ".join(obstacles)}, trace

    avail3 = 0
    while p + avail3 < n and t[p + avail3] in ACGT:
        avail3 += 1
    avail5 = 0
    while p - avail5 - 1 >= 0 and t[p - avail5 - 1] in ACGT:
        avail5 += 1

    # Sizing reads the annealing portion only (D032, owner decision 4.5).
    anneal_f = lambda f: t[p:p + f]
    anneal_r = lambda r: rc(t[p - r:p])
    tmf = lambda f: bio(anneal_f(f), anneal_f(f))
    tmr = lambda r: bio(anneal_r(r), anneal_r(r))

    f = f0
    while tmf(f) < FLOOR:
        if f >= avail3:
            return {
                "expected": "error:NoPrimerDesignFits",
                "failure": f"tm-floor 3':{avail3}:{tmf(f):.2f}",
            }, trace
        f += 1
        trace.add("fwd-floor-grew")
    f_floor = f
    while tmf(f) < CEILING and f < avail3 and not ends_gc(anneal_f(f)):
        f += 1
    if not ends_gc(anneal_f(f)):
        if f != f_floor:
            trace.add("fwd-gc-reverted")
        f = f_floor
    elif f != f_floor:
        trace.add("fwd-gc-extended")

    r = r0
    while tmr(r) < FLOOR:
        if r >= avail5:
            return {
                "expected": "error:NoPrimerDesignFits",
                "failure": f"tm-floor 5':{avail5}:{tmr(r):.2f}",
            }, trace
        r += 1
        trace.add("rev-floor-grew")
    r_floor = r
    while tmr(r) < CEILING and r < avail5 and not ends_gc(anneal_r(r)):
        r += 1
    if not ends_gc(anneal_r(r)):
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
        r_bal, j = r, r
        if not ends_gc(anneal_r(j)):
            while j < avail5 and tmr(j) < tf and not ends_gc(anneal_r(j)):
                j += 1
            if ends_gc(anneal_r(j)) and tmr(j) <= tf:
                r = j
                trace.add("balance-rev-gc")
            else:
                r = r_bal
    elif tr - tf > BALANCE:
        while tmr(r) - tmf(f) > BALANCE and f < avail3:
            f += 1
            trace.add("balance-grew-fwd")
        f_bal, j = f, f
        if not ends_gc(anneal_f(j)):
            while j < avail3 and tmf(j) < tr and not ends_gc(anneal_f(j)):
                j += 1
            if ends_gc(anneal_f(j)) and tmf(j) <= tr:
                f = j
                trace.add("balance-fwd-gc")
            else:
                f = f_bal

    tf, tr = tmf(f), tmr(r)
    met = abs(tf - tr) <= BALANCE
    if not met:
        trace.add("balance-unmet")
    forward = fwd_tail + anneal_f(f)
    reverse = rev_tail + anneal_r(r)
    # The tails rebuild the insertion in the product: the forward tail is its
    # 3' part and the reverse tail the reverse complement of its 5' part.
    assert rc(rev_tail) + fwd_tail == inserted
    if len(forward) > LENGTH_WARN or len(reverse) > LENGTH_WARN:
        trace.add("length-warning")
    # The whole-primer Tm is reported, never used for sizing.
    whole_f, whole_r = bio(forward, forward), bio(reverse, reverse)
    return {
        "expected": "design",
        "forward_anneal_span": f"{p}:{p + f}",
        "reverse_anneal_span": f"{p - r}:{p}",
        "forward_tail": fwd_tail if fwd_tail else "none",
        "reverse_tail": rev_tail if rev_tail else "none",
        "forward": forward,
        "reverse": reverse,
        "forward_tm": f"{tf:.2f}",
        "reverse_tm": f"{tr:.2f}",
        "forward_whole_tm": f"{whole_f:.2f}",
        "reverse_whole_tm": f"{whole_r:.2f}",
        "annealing_temp": f"{min(TA_CAP, min(tf, tr) + TA_OFFSET):.2f}",
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
    "short-20": MIX,
    "with-n": MIX + GC + MIX[:5] + "N" + MIX[6:] + GC + MIX,
}

# case_id, template, p, inserted, bound, note
CASES = [
    ("ins-01", "balanced", 50, "GAT", 15, "3 bases: one forward tail, no reverse tail"),
    ("ins-02", "balanced", 50, "G", 15, "a single base"),
    ("ins-03", "balanced", 50, "GATTAC", 15, "6 bases: the threshold, still one tail"),
    ("ins-04", "balanced", 50, "GATTACA", 15, "7 bases: split 3 reverse / 4 forward"),
    ("ins-05", "balanced", 50, "GATTACAGATTACA", 15, "14 bases: split 7 / 7"),
    ("ins-06", "balanced", 50, "GAT", 25, "bound 25 starts both annealing portions at 25"),
    ("ins-07", "at-upstream", 50, "GAT", 15, "AT-rich 5' side: the reverse primer grows"),
    ("ins-08", "at-downstream", 50, "GAT", 15, "AT-rich 3' side: the forward primer grows"),
    ("ins-09", "balanced", 10, "GAT", 15, "10 bases before the point; the reverse needs 15"),
    ("ins-10", "balanced", 85, "GAT", 15, "exactly 15 bases after the point: the forward primer just fits, cannot grow, and balance is left unmet"),
    ("ins-11", "short-20", 10, "GAT", 15, "20 bases cannot hold 15 + 15"),
    ("ins-12", "with-n", 50, "GAT", 15, "an N inside the starting reverse annealing portion"),
    ("ins-13", "balanced", 50, "GATTACA" * 30, 15, "210 bases inserted: both primers pass 100 nt, the owner's practical ordering limit"),
    ("ins-14", "../example-construct/input/sid-1_cDNA.fa", 1209, "GAT", 15, "a real template"),
    ("ins-15", "balanced", 90, "GAT", 15, "10 bases after the point; the forward primer needs 15"),
]

HEADER = [
    "case_id", "template", "point", "inserted", "min_overlap", "expected",
    "forward_anneal_span", "reverse_anneal_span", "forward_tail", "reverse_tail",
    "forward", "reverse", "forward_tm", "reverse_tm", "forward_whole_tm",
    "reverse_whole_tm", "annealing_temp", "balance_met", "failure", "note",
]


def main():
    rows = ["\t".join(HEADER)]
    for case_id, name, p, inserted, b, note in CASES:
        if name in SYNTHETIC:
            t, template_column = SYNTHETIC[name], "seq:" + SYNTHETIC[name]
        else:
            t, template_column = nd.load(name), name
        out, trace = design(t, p, inserted, b)
        row = [case_id, template_column, str(p), inserted, str(b), out["expected"]]
        row += [out.get(k, "none") for k in HEADER[6:18]]
        row += [out["failure"], note]
        rows.append("\t".join(row))
        shown = {k: v for k, v in out.items() if k not in ("forward", "reverse")}
        print(case_id, shown, sorted(trace))
    with open(os.path.join(FIXTURES, "native-insertion.tsv"), "w") as handle:
        handle.write("\n".join(rows) + "\n")


if __name__ == "__main__":
    main()
