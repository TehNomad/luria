#!/usr/bin/env python3
"""Expected mutant reading frames for `luria review` (plan §5.3a, M7-18 to M7-21).

Writes ../frame.tsv: for each case, the residues of the mutant protein that a
ribosome reads from the CDS's start in the edited construct, whose codon's
middle base lies in a window of the edited construct. luria-core's
`frame::mutant_residues` (contract 7i1) must give exactly these.

Independent of crates/: the edited construct is built here by splicing the
written bases into the template, and every codon is read with this script's
own standard-code table and complement map.

Rules (M7-20):
* the CDS's start (its first base on its own strand) is carried into the
  edited construct: a start before the edit keeps its position, a start at or
  past the edit's end moves by the edit's length change;
* codons are read on the CDS's strand from that start, three bases at a time,
  whole codons only, in the CDS's frame and on past a frameshift;
* the first stop codon is reported as `*` and ends the protein; a codon with
  a base other than A, C, G or T is reported as `?` and ends it too;
* reading also ends at the sequence's end, or once a codon's middle base is
  past the window in the reading direction;
* a residue is reported as `number:symbol:middle`, `middle` being the edited
  construct position (zero-based) of its codon's middle base, for each residue
  whose middle lies in [window_start, window_end).

Columns: case_id, template (a file under tests/fixtures, or `seq:` and the
bases), cds_start, cds_end, strand, edit_start, edit_end, written (`-` for
none), window_start, window_end, residues (`;`-joined, `-` for none).

Run from the repository root:

    python3 -I tests/fixtures/review/reference/frame.py           rewrite
    python3 -I tests/fixtures/review/reference/frame.py --check   compare
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.dirname(HERE)
FIXTURES = os.path.dirname(OUT)
NAME = "frame.tsv"

BASES = "TCAG"
AMINO = "FFLLSSSSYY**CC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG"
CODE = {
    a + b + c: AMINO[16 * i + 4 * j + k]
    for i, a in enumerate(BASES)
    for j, b in enumerate(BASES)
    for k, c in enumerate(BASES)
}
COMPLEMENT = {"A": "T", "T": "A", "G": "C", "C": "G"}

SID1 = "example-construct/input/sid-1_cDNA.fa"
SYN = (
    "seq:ATGGCTTGGAATCAGCAGCAGCCGAGCGGCTCGCGGACGCATGGCTTGGAATCAGCAGCAGCCGAG"
    "CGGCTCGCGGACGCATGGCTTGGAATCAGCAGCA"
)

# case, template, (cds start, end, strand), (edit start, end, written), window
CASES = [
    ("native-01", SID1, (0, 3153, "+"), (1209, 1212, "AAC"), (1175, 1227)),
    ("del-11", SID1, (0, 3153, "+"), (1209, 1212, ""), (1185, 1229)),
    ("ins-14", SID1, (0, 3153, "+"), (1209, 1209, "GAT"), (1189, 1232)),
    ("syn-del-frameshift", SYN, (0, 99, "+"), (45, 46, ""), (25, 99)),
    ("syn-ins-04", SYN, (0, 99, "+"), (50, 50, "GATTACA"), (30, 77)),
    ("syn-rep-02", SYN, (0, 99, "+"), (50, 53, "TT"), (20, 70)),
    ("syn-stop", SYN, (0, 99, "+"), (3, 6, "TAA"), (0, 99)),
    ("syn-minus", SYN, (0, 99, "-"), (80, 81, "T"), (40, 99)),
    ("syn-edit-before-cds", SYN, (30, 99, "+"), (10, 13, ""), (20, 60)),
]


def load(template):
    if template.startswith("seq:"):
        return template[len("seq:"):]
    with open(os.path.join(FIXTURES, template), encoding="ascii") as f:
        lines = f.read().splitlines()
    assert lines[0].startswith(">")
    return "".join(line.strip() for line in lines[1:]).upper()


def residues(t, cds, edit, window):
    start, end, strand = cds
    s, e, written = edit
    edited = t[:s] + written + t[e:]
    delta = len(edited) - len(t)
    first = start if strand == "+" else end - 1
    assert not (s <= first < e), "the CDS start is inside the edit"
    begin = first if first < s else first + delta
    lo, hi = window
    out = []
    number = 0
    position = begin
    while True:
        if strand == "+":
            if position + 2 >= len(edited):
                break
            codon = edited[position:position + 3]
            middle = position + 1
            if middle >= hi:
                break
            step = 3
        else:
            if position - 2 < 0:
                break
            codon = "".join(COMPLEMENT.get(edited[p], "N") for p in (position, position - 1, position - 2))
            middle = position - 1
            if middle < lo:
                break
            step = -3
        number += 1
        symbol = CODE.get(codon, "?")
        if lo <= middle < hi:
            out.append("%d:%s:%d" % (number, symbol, middle))
        if symbol in "*?":
            break
        position += step
    return out


def main():
    lines = ["\t".join([
        "case_id", "template", "cds_start", "cds_end", "strand", "edit_start",
        "edit_end", "written", "window_start", "window_end", "residues",
    ])]
    for case, template, cds, edit, window in CASES:
        t = load(template)
        found = residues(t, cds, edit, window)
        lines.append("\t".join([
            case, template, str(cds[0]), str(cds[1]), cds[2], str(edit[0]),
            str(edit[1]), edit[2] or "-", str(window[0]), str(window[1]),
            ";".join(found) or "-",
        ]))
    text = "".join(line + "\n" for line in lines)
    path = os.path.join(OUT, NAME)
    if "--check" not in sys.argv[1:]:
        with open(path, "w", encoding="ascii") as f:
            f.write(text)
        print("wrote", NAME)
        return
    try:
        with open(path, encoding="ascii") as f:
            tracked = f.read()
    except FileNotFoundError:
        tracked = None
    if tracked == text:
        print("OK %s" % NAME)
        return
    print("DIFFERS %s" % NAME)
    sys.exit(1)


if __name__ == "__main__":
    main()
