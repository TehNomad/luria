#!/usr/bin/env python3
"""Expected luriaNative designs under D030, derived without crates/core.

Restates D029's scope and D030's Tm-driven sizing in plain Python, computes
every Tm with Biopython (an independent implementation of the same published
model; see tests/fixtures/tm/README.md), and writes ../native.tsv. It shares no
code with crates/core; crates/core/tests/q5_primer_fixtures.rs reads native.tsv.

Requires Biopython 1.88 (`Bio.SeqUtils.MeltingTemp`). Run from this directory:

    python3 native-design.py              rewrite ../native.tsv
    python3 native-design.py --synthetic  print unit-test expectations for the
                                          synthetic templates in SYNTHETIC

Prints no sequence from a fixture file. The --synthetic mode prints only the
synthetic templates defined in this file.

D030, for a replacement region [s, e), L = e - s, bound b, template length n:
    B  = s - 10                     junction; forward 5' flank is 10, fixed
    v0 = max(10, b - 10)            forward 3' flank, start
    r0 = max(b, 15)                 reverse length, start
    no design possible     if L + 10 + v0 + r0 > n
    no design fits         if s < 10 + r0 (5'), n - e < v0 (3'), an ambiguity
                           code in the starting pair, or a Tm floor unreachable
    grow forward 3' to Tm >= 55; then toward a G/C last base while Tm < 63,
        else back to the floor length
    the same for the reverse primer
    balance: grow the colder primer while the gap exceeds 5; then toward a G/C
        last base while its Tm stays below the other's, keeping that length only
        if it ends in G/C at or below the other's Tm; else the balanced length
    Ta = min(72, min(Tm_f, Tm_r) + 3)
Forward Tm: mismatch-aware against the wild-type site, or matched where two
mismatches are adjacent. Reverse Tm: matched.
"""
import json
import os
import sys

from Bio.SeqUtils import MeltingTemp as mt

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURES = os.path.dirname(HERE)

FLOOR, CEILING, BALANCE, TA_OFFSET, TA_CAP, FLANK5 = 55.0, 63.0, 5.0, 3.0, 72.0, 10
COMP = {"A": "T", "T": "A", "G": "C", "C": "G"}
ACGT = set("ACGT")


def rc(seq):
    return "".join(COMP[b] for b in reversed(seq))


def selfcomp(seq):
    return seq == rc(seq)


def bio(primer, site):
    comp = "".join(COMP[b] for b in site)
    return mt.Tm_NN(primer, c_seq=comp, nn_table=mt.DNA_NN4, imm_table=mt.DNA_IMM1,
                    saltcorr=6, Na=150, dnac1=500, dnac2=0, selfcomp=selfcomp(primer))


def forward_tm(primer, site):
    """Hybrid basis: mismatch-aware unless two mismatches are adjacent."""
    diff = [i for i in range(len(primer)) if primer[i] != site[i]]
    tandem = any(diff[k + 1] - diff[k] == 1 for k in range(len(diff) - 1))
    if not diff or tandem:
        return bio(primer, primer), "matched_complement"
    return bio(primer, site), "mismatched_wild_type_site"


def ends_gc(seq):
    return seq[-1] in "GC"


def design(t, s, rep, b):
    trace = set()
    n, L = len(t), len(rep)
    e = s + L
    v0, r0 = max(10, b - 10), max(b, 15)
    if L + FLANK5 + v0 + r0 > n:
        return {"expected": "error:NoPrimerDesignPossible",
                "failure": f"pair:{L + FLANK5 + v0 + r0}>{n}"}, trace
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

    # Availability: a primer cannot grow across an ambiguity code.
    avail3 = 0
    while e + avail3 < n and t[e + avail3] in ACGT:
        avail3 += 1
    avail5 = 0
    while B - avail5 - 1 >= 0 and t[B - avail5 - 1] in ACGT:
        avail5 += 1

    edited = t[:s] + rep + t[e:]
    fwd = lambda v: edited[B:e + v]
    site = lambda v: t[B:e + v]
    rev = lambda r: rc(t[B - r:B])
    tmf = lambda v: forward_tm(fwd(v), site(v))[0]
    tmr = lambda r: bio(rev(r), rev(r))

    # Forward: floor, then G/C.
    v = v0
    while tmf(v) < FLOOR:
        if v >= avail3:
            return {"expected": "error:NoPrimerDesignFits",
                    "failure": f"tm-floor 3':{avail3}:{tmf(v):.2f}"}, trace
        v += 1
        trace.add("fwd-floor-grew")
    v_floor = v
    while tmf(v) < CEILING and v < avail3 and not ends_gc(fwd(v)):
        v += 1
    if not ends_gc(fwd(v)):
        if v != v_floor:
            trace.add("fwd-gc-reverted")
        v = v_floor
    elif v != v_floor:
        trace.add("fwd-gc-extended")

    # Reverse: floor, then G/C.
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

    # Balance.
    tf, tr = tmf(v), tmr(r)
    if tf - tr > BALANCE:
        while tmf(v) - tmr(r) > BALANCE and r < avail5:
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
        while tmr(r) - tmf(v) > BALANCE and v < avail3:
            v += 1
            trace.add("balance-grew-fwd")
        v_bal, k = v, v
        if not ends_gc(fwd(k)):
            while k < avail3 and tmf(k) < tr and not ends_gc(fwd(k)):
                k += 1
            if ends_gc(fwd(k)) and tmf(k) <= tr:
                v = k
                trace.add("balance-fwd-gc")
            else:
                v = v_bal

    tf, basis = forward_tm(fwd(v), site(v))
    tr = tmr(r)
    met = abs(tf - tr) <= BALANCE
    if not met:
        trace.add("balance-unmet")
    if basis == "matched_complement" and fwd(v) != site(v):
        trace.add("tandem-fallback")
    return {
        "expected": "design",
        "forward_span": f"{B}:{e + v}",
        "reverse_span": f"{B - r}:{B}",
        "flanks": f"{FLANK5}:{v}",
        "reverse_length": str(r),
        "forward_tm": f"{tf:.2f}",
        "forward_tm_basis": basis,
        "reverse_tm": f"{tr:.2f}",
        "annealing_temp": f"{min(TA_CAP, min(tf, tr) + TA_OFFSET):.2f}",
        "balance_met": "yes" if met else "no",
        "failure": "none",
        "_forward": fwd(v), "_reverse": rev(r),
    }, trace


def load(relative):
    lines = open(os.path.join(FIXTURES, relative)).read().splitlines()
    return "".join(l.strip() for l in lines if not l.startswith((">", ";"))).upper()


def other_base(base):
    return "A" if base != "A" else "G"


SID1 = "../example-construct/input/sid-1_cDNA.fa"
MUT_A = "../mutation/mut-a.txt"

CASES = [
    ("native-01", SID1, 1209, 1212, "AAC", 15, "q5p-01", "the q5p-01 edit at bound 15"),
    ("native-02", SID1, 1209, 1212, "AAC", 24, "none", "q5p-02's bound; forward 3' flank starts at 14"),
    ("native-03", SID1, 1209, 1212, "AAC", 25, "none", "q5p-03's bound; forward 3' flank starts at 15"),
    ("native-04", SID1, 1503, 1506, "GAG", 15, "q5p-07/variant-b", "adjacent changes: forward Tm falls back to matched"),
    ("native-05", MUT_A, 9, 12, "AAC", 6, "none", "30 bases cannot hold 3 + 10 + 10 + 15"),
    ("native-06", MUT_A, 4, 5, "T", 31, "none", "q5p-09: bound 31 on a 30-base template"),
    ("native-07", SID1, 20, 21, None, 15, "none", "20 bases before the edit; the pair needs 25"),
    ("native-08", SID1, 3148, 3149, None, 15, "none", "4 bases after the edit; the forward primer needs 10"),
]

HEADER = ["case_id", "sequence_file", "edit", "min_overlap", "expected", "forward_span",
          "reverse_span", "flanks", "reverse_length", "forward_tm", "forward_tm_basis",
          "reverse_tm", "annealing_temp", "balance_met", "reference_design", "failure", "note"]


def reference_forward_span(key):
    d = json.load(open(os.path.join(HERE, "designs.json")))
    case, _, variant = key.partition("/")
    rec = d[case]
    rec = rec.get(variant, rec.get("design", rec)) if variant else rec.get("design", rec)
    return f"{rec['forward']['span'][0]}:{rec['forward']['span'][1]}"


def main():
    rows = ["\t".join(HEADER)]
    for case_id, path, s, e, replacement, b, reference, note in CASES:
        template = load(path)
        original = template[s:e]
        if replacement is None:
            replacement = other_base(original)
        assert len(replacement) == e - s and replacement != original, case_id
        out, trace = design(template, s, replacement, b)
        if reference != "none":
            same = out.get("forward_span") == reference_forward_span(reference)
            print(f"{case_id}: forward span equals {reference}: {same}")
            assert same, f"{case_id} no longer matches {reference}; drop the reference name"
        row = [case_id, path, f"{s}:{e}:{original}:{replacement}", str(b), out["expected"]]
        row += [out.get(k, "none") for k in HEADER[5:14]]
        row += [reference, out["failure"], note]
        rows.append("\t".join(row))
        shown = {k: v for k, v in out.items() if not k.startswith("_")}
        print(case_id, shown, sorted(trace))
    with open(os.path.join(FIXTURES, "native.tsv"), "w") as f:
        f.write("\n".join(rows) + "\n")


# ---------------------------------------------------------------------------
# Synthetic templates for the unit tests in crates/core/src/primer_design.rs.
# ---------------------------------------------------------------------------

def synthetic():
    import itertools
    # Building blocks: a GC-rich and an AT-rich 20-mer, and a mixed spacer.
    GC = "GCCGAGCGGCTCGCGGACGC"
    AT = "ATTAAATATTTAAATTATAT"
    MIX = "ATGGCTTGGAATCAGCAGCA"
    templates = {
        "balanced": MIX + GC + MIX + GC + MIX,
        "at-upstream": AT + AT + MIX + GC + GC,
        "at-downstream": GC + GC + MIX + AT + AT,
        "at-everywhere": AT * 5,
    }
    for name, t in templates.items():
        print(f'\n{name} ({len(t)}): "{t}"')
        for s in range(25, len(t) - 10):
            for rep_len in (1, 3):
                if s + rep_len > len(t):
                    continue
                orig = t[s:s + rep_len]
                rep = "".join(other_base(c) for c in orig)
                out, trace = design(t, s, rep, 15)
                if trace or out["expected"] != "design":
                    shown = {k: v for k, v in out.items()}
                    print(f"  s={s} L={rep_len} {orig}->{rep} {sorted(trace)} {shown}")


if __name__ == "__main__":
    if "--synthetic" in sys.argv:
        synthetic()
    else:
        main()
