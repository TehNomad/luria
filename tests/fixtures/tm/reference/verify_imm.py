"""Verify mismatch dH/dS against the open-access dG37 table.

Source A (dH, dS): Biopython Bio.SeqUtils.MeltingTemp DNA_IMM1, an independent
transcription of Allawi & SantaLucia 1997/1998 and Peyret et al. 1999.
Source B (dG37):   SantaLucia & Hicks 2004, Annu Rev, Table 2 (open access).

The NN model's strand-symmetry identity lets a doublet be read from either
strand: 5'-AB-3'/3'-A'B'-5' is the same motif as 5'-B'A'-3'/3'-BA-5'. The
review tabulates the WC pair first; Biopython stores whichever orientation the
paper used, so each key is normalised before lookup.

Requires Biopython 1.88 (see tests/fixtures/requirements.txt). Run from the
repository root:

    python -I tests/fixtures/tm/reference/verify_imm.py
    python -I tests/fixtures/tm/reference/verify_imm.py --check

Without --check, writes ../nn-mismatch.tsv as before. With --check, regenerates
the same text in memory, compares it
byte for byte with that tracked file, prints `OK nn-mismatch.tsv` or `DIFFERS
nn-mismatch.tsv` followed by the first differing line, exits 1 on a difference,
and writes nothing (D039).
"""
T = 310.15
WC = {("A", "T"), ("T", "A"), ("G", "C"), ("C", "G")}

IMM = {
 "AG/TT": (1.0, 0.9), "AT/TG": (-2.5, -8.3), "CG/GT": (-4.1, -11.7),
 "CT/GG": (-2.8, -8.0), "GG/CT": (3.3, 10.4), "GG/TT": (5.8, 16.3),
 "GT/CG": (-4.4, -12.3), "GT/TG": (4.1, 9.5), "TG/AT": (-0.1, -1.7),
 "TG/GT": (-1.4, -6.2), "TT/AG": (-1.3, -5.3), "AA/TG": (-0.6, -2.3),
 "AG/TA": (-0.7, -2.3), "CA/GG": (-0.7, -2.3), "CG/GA": (-4.0, -13.2),
 "GA/CG": (-0.6, -1.0), "GG/CA": (0.5, 3.2), "TA/AG": (0.7, 0.7),
 "TG/AA": (3.0, 7.4),
 "AC/TT": (0.7, 0.2), "AT/TC": (-1.2, -6.2), "CC/GT": (-0.8, -4.5),
 "CT/GC": (-1.5, -6.1), "GC/CT": (2.3, 5.4), "GT/CC": (5.2, 13.5),
 "TC/AT": (1.2, 0.7), "TT/AC": (1.0, 0.7),
 "AA/TC": (2.3, 4.6), "AC/TA": (5.3, 14.6), "CA/GC": (1.9, 3.7),
 "CC/GA": (0.6, -0.6), "GA/CC": (5.2, 14.2), "GC/CA": (-0.7, -3.8),
 "TA/AC": (3.4, 8.0), "TC/AA": (7.6, 20.2),
 "AA/TA": (1.2, 1.7), "CA/GA": (-0.9, -4.2), "GA/CA": (-2.9, -9.8),
 "TA/AA": (4.7, 12.9), "AC/TC": (0.0, -4.4), "CC/GC": (-1.5, -7.2),
 "GC/CC": (3.6, 8.9), "TC/AC": (6.1, 16.4), "AG/TG": (-3.1, -9.5),
 "CG/GG": (-4.9, -15.3), "GG/CG": (-6.0, -15.8), "TG/AG": (1.6, 3.6),
 "AT/TT": (-2.7, -10.8), "CT/GT": (-5.0, -15.8), "GT/CT": (-2.2, -8.4),
 "TT/AT": (0.2, -1.5),
}

# Review Table 2: dG37 by WC pair (first position), then mismatch X:Y.
REVIEW = {}
for wc, block in {
    "GC": {"A": {"A": 0.17, "C": 0.81, "G": -0.25},
           "C": {"A": 0.47, "C": 0.79, "T": 0.62},
           "G": {"A": -0.52, "G": -1.11, "T": 0.08},
           "T": {"C": 0.98, "G": -0.59, "T": 0.45}},
    "CG": {"A": {"A": 0.43, "C": 0.75, "G": 0.03},
           "C": {"A": 0.79, "C": 0.70, "T": 0.62},
           "G": {"A": 0.11, "G": -0.11, "T": -0.47},
           "T": {"C": 0.40, "G": -0.32, "T": -0.12}},
    "AT": {"A": {"A": 0.61, "C": 0.88, "G": 0.14},
           "C": {"A": 0.77, "C": 1.33, "T": 0.64},
           "G": {"A": 0.02, "G": -0.13, "T": 0.71},
           "T": {"C": 0.73, "G": 0.07, "T": 0.69}},
    "TA": {"A": {"A": 0.69, "C": 0.92, "G": 0.42},
           "C": {"A": 1.33, "C": 1.05, "T": 0.97},
           "G": {"A": 0.74, "G": 0.44, "T": 0.43},
           "T": {"C": 0.75, "G": 0.34, "T": 0.68}},
}.items():
    for x, inner in block.items():
        for y, dg in inner.items():
            REVIEW[f"{wc[0]}{x}/{wc[1]}{y}"] = dg

COMP = {"A": "T", "T": "A", "G": "C", "C": "G"}

def normalise(key):
    """Return the key with the WC pair first, or None if not a single mismatch."""
    top, bottom = key.split("/")
    p1, p2 = (top[0], bottom[0]), (top[1], bottom[1])
    first_wc, second_wc = p1 in WC, p2 in WC
    if first_wc == second_wc:
        return None                       # no WC pair, or two of them
    if first_wc:
        return key
    # Strand-symmetry identity: read the doublet from the other strand.
    return f"{bottom[1]}{bottom[0]}/{top[1]}{top[0]}"

ok, bad, skipped = [], [], []
for key, (dh, ds) in sorted(IMM.items()):
    norm = normalise(key)
    if norm is None or norm not in REVIEW:
        skipped.append((key, norm))
        continue
    calc = dh - T * ds / 1000.0
    printed = REVIEW[norm]
    delta = abs(calc - printed)
    (ok if delta <= 0.06 else bad).append((key, norm, calc, printed, delta))

print(f"verified {len(ok)}   disagree {len(bad)}   unmappable {len(skipped)}   total {len(IMM)}")
print(f"worst verified deviation {max(r[4] for r in ok):.3f} kcal/mol\n")
if bad:
    print("DISAGREEMENTS (dH/dS implies a different dG37 than the review prints):")
    for key, norm, calc, printed, delta in bad:
        print(f"  {key:8s} -> review {norm:8s}  implied {calc:+.3f}  printed {printed:+.2f}  delta {delta:.3f}")
print("\nnot checkable against Table 2 (tandem mismatches, no single WC pair):")
print("  " + ", ".join(k for k, _ in skipped))

# ---- emit the fixture -------------------------------------------------------
import os
import sys

check = "--check" in sys.argv[1:]
out = []
for key, (dh, ds) in IMM.items():
    norm = normalise(key)
    if norm is None or norm not in REVIEW:
        continue
    out.append((norm, dh, ds, REVIEW[norm]))
out.sort()
seen = {}
for norm, dh, ds, dg in out:
    if norm in seen and seen[norm] != (dh, ds):
        print(f"CONFLICT {norm}: {seen[norm]} vs {(dh, ds)}")
    seen[norm] = (dh, ds)
print(f"\ndistinct single-mismatch motifs: {len(seen)} (review Table 2 has 48)")
HERE = os.path.dirname(os.path.abspath(__file__))
path = os.path.join(os.path.dirname(HERE), "nn-mismatch.tsv")

text = """#
# Nearest-neighbour parameters for a DNA doublet containing ONE internal
# mismatch, in 1 M NaCl. Orientation: Watson-Crick pair first, mismatch second,
# as SantaLucia & Hicks (2004) Table 2 tabulates them. A doublet whose mismatch
# comes first is the same motif read from the other strand.
#
# dH, dS:  Allawi & SantaLucia 1997/1998 and Peyret et al. 1999, via the
#          independent transcription in Biopython's DNA_IMM1.
# dG37_published: SantaLucia & Hicks (2004) Table 2, open access. Carried here
#          as the verifier, not as an input: dH - 310.15*dS/1000 must reproduce
#          it within the rounding of the printed dH and dS (+/- 0.13 kcal/mol).
#
# This set is COMPLETE for single mismatches (48 of 48) and does NOT cover two
# adjacent mismatches. See README.md.
#
# dH: kcal/mol. dS: cal/(K mol). dG37: kcal/mol.
#
motif\tdH_kcal_per_mol\tdS_cal_per_K_mol\tdG37_published
"""
for norm in sorted(seen):
    dh, ds = seen[norm]
    text += f"{norm}\t{dh}\t{ds}\t{REVIEW[norm]}\n"

if check:
    try:
        with open(path, "rb") as f:
            tracked = f.read()
    except FileNotFoundError:
        tracked = None
    if tracked == text.encode():
        print(f"OK {os.path.basename(path)}")
    else:
        print(f"DIFFERS {os.path.basename(path)}")
        old = tracked.decode().splitlines(True) if tracked is not None else []
        new = text.splitlines(True)
        for i in range(min(len(old), len(new))):
            if old[i] != new[i]:
                print(f"  line {i + 1}: tracked {old[i]!r} != generated {new[i]!r}")
                break
        else:
            i = min(len(old), len(new))
            if len(old) == len(new):
                print("  line-level comparison found no difference")
            else:
                where = "tracked" if len(old) > len(new) else "generated"
                extra = (old if len(old) > len(new) else new)[i]
                print(f"  line {i + 1}: only in {where}: {extra!r}")
        sys.exit(1)
else:
    with open(path, "w") as f:
        f.write(text)
    print("wrote", os.path.abspath(path))
