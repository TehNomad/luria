"""Cross-check luria's Tm against Biopython on synthetic sequences.

Biopython is an independent implementation of the same published model, so
agreement checks luria's arithmetic, not its choice of parameters.

Settings match crates/core/src/tm.rs: DNA_NN4 (SantaLucia & Hicks 2004),
DNA_IMM1, Owczarzy 2004 monovalent salt correction, 150 mM Na, 500 nM primer
in excess.

Requires Biopython 1.88 (see tests/fixtures/requirements.txt). Run from the
repository root:

    python -I tests/fixtures/tm/reference/crosscheck.py
    python -I tests/fixtures/tm/reference/crosscheck.py --check

Without --check, prints the Biopython Tm values, as the generating script of
`crates/core/tests/tm_crosscheck.rs`. With --check, it additionally reads that
file's EXPECTED table, compares every label and value with the values computed
here (rounded to 3 decimals), prints `OK tm_crosscheck.rs` or `DIFFERS
tm_crosscheck.rs` followed by each difference, and exits 1 on any difference or
missing label. It writes nothing (D039).
"""
from Bio.SeqUtils import MeltingTemp as mt

COMP = {"A": "T", "T": "A", "G": "C", "C": "G"}

def selfcomp(seq):
    rc = "".join(COMP[b] for b in reversed(seq))
    return seq == rc

def bio_tm(primer, site):
    # Biopython does not detect self-complementarity; it is a caller flag,
    # default False. luria detects it, so it must be passed here to compare.
    comp = "".join(COMP[b] for b in site)   # 3'->5' bottom strand
    return mt.Tm_NN(primer, c_seq=comp, nn_table=mt.DNA_NN4, imm_table=mt.DNA_IMM1,
                    saltcorr=6, Na=150, dnac1=500, dnac2=0,
                    selfcomp=selfcomp(primer))

MIXED = "ATGCATGCATGCATGCAT"
one_mismatch_site = MIXED[:9] + "A" + MIXED[10:]     # primer has T at index 9
assert len(one_mismatch_site) == len(MIXED)

# The q5p-01 pattern on a synthetic 23-mer: GAT -> AAC, so indices 10 and 12
# differ and index 11 matches.
P23 = "ACGTACGTAC" + "AAC" + "GTACGTACGT"
W23 = "ACGTACGTAC" + "GAT" + "GTACGTACGT"
assert len(P23) == len(W23) == 23

CASES = [
    ("matched mixed 18-mer", MIXED, MIXED),
    ("matched GC-rich 18-mer", "GCGCGCGCGCGCGCGCGC", "GCGCGCGCGCGCGCGCGC"),
    ("matched AT-rich 18-mer", "ATATATATATATATATAT", "ATATATATATATATATAT"),
    ("one mismatch at index 9", MIXED, one_mismatch_site),
    ("q5p-01 pattern, matched", P23, P23),
    ("q5p-01 pattern, two mismatches", P23, W23),
]
values = {}
for label, primer, site in CASES:
    value = bio_tm(primer, site)
    values[label] = value
    print(f"{label:32} {value:9.3f}   selfcomp={selfcomp(primer)}")
print(f"\n{'drop from two mismatches':32} {bio_tm(P23, P23) - bio_tm(P23, W23):9.3f} C")

# ---- compare with the tracked Rust table (D039) -----------------------------
import os
import re
import sys

if "--check" in sys.argv[1:]:
    # The script sits at tests/fixtures/tm/reference/, so four levels up is the
    # repository root and the Rust test is a fixed path below it.
    root = os.path.dirname(os.path.abspath(__file__))
    for _ in range(4):
        root = os.path.dirname(root)
    rust_path = os.path.join(root, "crates", "core", "tests", "tm_crosscheck.rs")

    source = ""
    try:
        with open(rust_path) as f:
            source = f.read()
    except FileNotFoundError:
        print(f"DIFFERS tm_crosscheck.rs: no such file: {rust_path}")
        sys.exit(1)
    start = source.find("const EXPECTED")
    end = source.find("\n];", start)
    entries = []
    if start >= 0 and end > start:
        block = source[start:end]
        entries = re.findall(r'\(\s*"([^"]*)"[^()]*?(-?\d+(?:\.\d+)?)\s*,?\s*\)', block)

    problems = []
    if not entries:
        problems.append("  could not read the EXPECTED table from tm_crosscheck.rs")
    if len(entries) != len(CASES):
        problems.append(f"  {len(entries)} EXPECTED entries, expected {len(CASES)}")
    rust_labels = [label for label, _ in entries]
    for label, rust_value in entries:
        if label not in values:
            problems.append(f"  missing label: {label!r} is not computed here")
            continue
        mine = round(values[label], 3)
        if abs(mine - float(rust_value)) >= 0.0005:
            problems.append(f"  {label:32} rust {rust_value} != script {mine:.3f}")
    for label in values:
        if label not in rust_labels:
            problems.append(f"  missing label: {label!r} is not in the Rust table")

    if problems:
        print("DIFFERS tm_crosscheck.rs")
        for problem in problems:
            print(problem)
        sys.exit(1)
    print("OK tm_crosscheck.rs")
