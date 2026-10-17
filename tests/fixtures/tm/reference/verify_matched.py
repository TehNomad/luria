"""Verify nn-matched.tsv against Biopython's DNA_NN4.

Source A: tests/fixtures/tm/nn-matched.tsv (SantaLucia & Hicks 2004, Table 1;
see README.md for how the values were read and why this set was chosen).
Source B: Biopython Bio.SeqUtils.MeltingTemp.DNA_NN4, an independent
transcription of the same paper.

DNA_NN4's key spelling is read from Biopython's own source (Bio/SeqUtils/
MeltingTemp.py): a duplex key is "XY/X'Y'" with values (dH kcal/mol, dS cal/(K
mol)), the initiation term is "init", the terminal A/T term is "init_A/T" and
the symmetry term is "sym".

This checks the transcription in the fixture, not the arithmetic: every dH and
dS must equal DNA_NN4's tuple for that key.

Requires Biopython 1.88 (see tests/fixtures/requirements.txt). Run from the
repository root:

    python -I tests/fixtures/tm/reference/verify_matched.py
    python -I tests/fixtures/tm/reference/verify_matched.py --check

Reads ../nn-matched.tsv and writes nothing; --check is accepted and means the
same thing (D039). Prints one OK line, or every mismatch, and exits 1 on any
mismatch.
"""
import os
import sys

from Bio.SeqUtils import MeltingTemp as mt

# The ten Watson-Crick doublets of Table 1, in the fixture's order.
WC = ["AA/TT", "AT/TA", "TA/AT", "CA/GT", "GT/CA", "CT/GA", "GA/CT", "CG/GC",
      "GC/CG", "GG/CC"]

# Fixture row -> DNA_NN4 key, as spelled in Biopython's source.
TERMS = {"initiation": "init", "terminal_AT": "init_A/T", "symmetry": "sym"}

for arg in sys.argv[1:]:
    if arg != "--check":
        print("usage: verify_matched.py [--check]   (writes nothing either way)")
        sys.exit(2)

HERE = os.path.dirname(os.path.abspath(__file__))
path = os.path.join(os.path.dirname(HERE), "nn-matched.tsv")

rows = {}
with open(path) as f:
    for line in f:
        line = line.rstrip("\n")
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 4:
            print("DIFFERS nn-matched.tsv: malformed row")
            print(f"  {line!r}")
            sys.exit(1)
        motif, dh, ds, dg = fields
        if motif == "motif":
            continue
        rows[motif] = (float(dh), float(ds))

expected = [(motif, motif) for motif in WC] + sorted(TERMS.items())

problems = []
for fixture_key, nn_key in expected:
    if fixture_key not in rows:
        problems.append(f"  {fixture_key}: row missing from nn-matched.tsv")
        continue
    if nn_key not in mt.DNA_NN4:
        problems.append(f"  {fixture_key}: key {nn_key!r} missing from DNA_NN4")
        continue
    want = (float(mt.DNA_NN4[nn_key][0]), float(mt.DNA_NN4[nn_key][1]))
    if rows[fixture_key] != want:
        problems.append(
            f"  {fixture_key:12} fixture {rows[fixture_key][0]:g} / "
            f"{rows[fixture_key][1]:g} != DNA_NN4 {want[0]:g} / {want[1]:g}"
        )

for motif in sorted(rows):
    if motif not in dict(expected):
        problems.append(f"  {motif}: row not in the expected set "
                        "(ten Watson-Crick doublets plus the three terms)")

if problems:
    print(f"DIFFERS nn-matched.tsv: {len(problems)} mismatch(es)")
    for problem in problems:
        print(problem)
    sys.exit(1)
print("OK nn-matched.tsv")
