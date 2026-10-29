"""Search for the 62 bp record `../frames-62.gb` holds, and check it still
satisfies what it claims (D036, D050, M8-18).

Why this record exists. Nothing else in the fixtures supplies two things the
Milestone 8 view needs:

* a reference whose length is **not** a multiple of 3, so a six-frame run ends
  with bases that get no codon (both existing synthetic records are: 60 and
  393). 62 % 3 == 2, which also gives M8-18's leftover-2-bases case; and
* a **usable minus-strand CDS that crosses the origin**. `mj1` and `mj2` in
  `tests/fixtures/genbank/synthetic.gb` are the only minus-strand origin joins
  in the fixtures and both are refused as `alternative_start`
  (`tests/fixtures/genbank/README.md`: their reverse complement starts `CAT`),
  so the CDS map would have no wrapped minus-strand codon to pin.

How a wrapped CDS reads, and where that comes from

`crates/core` reads a CDS as the joined arc along the molecule — parts that
abut across the origin are one arc, `base_length` followed by base 1 (D036) —
and on the minus strand takes the **reverse complement** of that arc text. That
is the rule `tests/fixtures/genbank/README.md` already verifies by hand for
`pj`, `mj1` and `mj2` ("The plus-strand text is ATGAAA + ATG. Its reverse
complement, CATTTTCAT, reads H F H"), and Biopython's own `location.extract()`
of the same features agrees with it, which is the cross-check this script also
runs. So a minus-strand CDS crossing the origin starts at the complement of the
arc's *lowest-numbered* base, and its codons are the complements of the
reference bases, taken in decreasing order. `mj1`/`mj2` start `CAT`; that is
the same walk.

Constraints, all from decisions already made:

* 62 bases, circular; 62 % 3 == 2 (M8-18's leftover bases, D050: they get no
  letter).
* `fP` at `join(56..62,1..11)` — 18 bases, 6 codons, plus strand, crossing the
  origin.
* `fM` at `complement(join(53..62,1..8))` — 18 bases, 6 codons, minus strand,
  crossing the origin. Its first codon is the complement of bases 8, 7 and 6,
  so it is `ATG` exactly where `fP` reads `GAT` (Asp): the two CDSs can share
  that codon without either getting an internal stop.
* each CDS: length divisible by 3, first codon ATG, no internal stop, last
  codon a stop — the rules that make a CDS importable (D042's
  `alternative_start`, `internal_stop_codon`, and
  `cds_length_not_divisible_by_three` as recorded in
  `tests/fixtures/genbank/synthetic-expected.tsv`).
* the record's `/translation` for each CDS is the residues of that reading,
  with no terminal stop (D010), because the import compares them (D042's
  `genbank_translation_mismatch`).
* the two CDSs overlap, which is what the M8-16 overlap case needs in a small
  file; neither contains an ambiguity code (ambiguity is `ambiguity.gb`'s job).

The search is deterministic: a fixed seed (20261024) and a fixed candidate
order. It never writes a file. Run from the repository root with the pinned
environment:

    python -I tests/fixtures/view/reference/make_frames_62.py --print
    python -I tests/fixtures/view/reference/make_frames_62.py --check

--print writes the chosen sequence, both codon lists and both proteins to
standard output. --check parses the tracked `../frames-62.gb` with Biopython
and asserts every constraint above against it, including that Biopython's own
extraction of each CDS matches the codon list this script computes.
"""
import os
import random
import re
import sys

from Bio import SeqIO

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.dirname(HERE)
RECORD = os.path.join(FIXTURE, "frames-62.gb")

LENGTH = 62
SEED = 20261024

# Spans as GenBank text, and as 1-based reference positions in molecule order
# (the order the arc lies along the circle, joining across the origin).
PLUS_TEXT = "join(56..62,1..11)"
MINUS_TEXT = "complement(join(53..62,1..8))"
PLUS_ARC = list(range(56, 63)) + list(range(1, 12))
MINUS_ARC = list(range(53, 63)) + list(range(1, 9))

STOPS = {"TAA", "TAG", "TGA"}
COMPLEMENT = {"A": "T", "T": "A", "G": "C", "C": "G"}
CODE = {
    "TTT": "F", "TTC": "F", "TTA": "L", "TTG": "L", "CTT": "L", "CTC": "L",
    "CTA": "L", "CTG": "L", "ATT": "I", "ATC": "I", "ATA": "I", "ATG": "M",
    "GTT": "V", "GTC": "V", "GTA": "V", "GTG": "V", "TCT": "S", "TCC": "S",
    "TCA": "S", "TCG": "S", "CCT": "P", "CCC": "P", "CCA": "P", "CCG": "P",
    "ACT": "T", "ACC": "T", "ACA": "T", "ACG": "T", "GCT": "A", "GCC": "A",
    "GCA": "A", "GCG": "A", "TAT": "Y", "TAC": "Y", "CAT": "H", "CAC": "H",
    "CAA": "Q", "CAG": "Q", "AAT": "N", "AAC": "N", "AAA": "K", "AAG": "K",
    "GAT": "D", "GAC": "D", "GAA": "E", "GAG": "E", "TGT": "C", "TGC": "C",
    "TGG": "W", "CGT": "R", "CGC": "R", "CGA": "R", "CGG": "R", "AGT": "S",
    "AGC": "S", "AGA": "R", "AGG": "R", "GGT": "G", "GGC": "G", "GGA": "G",
    "GGG": "G",
}


def read_positions(arc, strand):
    """The arc's 1-based positions in reading order. A plus CDS reads the arc
    forward; a minus CDS reads it backward, which on a wrapped arc starts at
    the arc's low end (the complement of its last base in molecule order)."""
    return list(arc) if strand == 1 else list(reversed(arc))


def codons(seq, arc, strand):
    """The codons of one CDS: `seq` is the 1-based-indexable record text."""
    positions = read_positions(arc, strand)
    out = []
    for index in range(0, len(positions), 3):
        codon = ""
        for position in positions[index:index + 3]:
            base = seq[position - 1]
            codon += COMPLEMENT[base] if strand == -1 else base
        out.append(codon)
    return out


def protein(cs):
    """The residues, without the terminal stop (D010)."""
    body = cs[:-1] if cs and cs[-1] in STOPS else cs
    return "".join(CODE[c] for c in body)


def usable(cs):
    """ATG start, no internal stop, terminal stop, length a multiple of three."""
    if len(cs) % 3:
        return False
    if cs[0] != "ATG" or cs[-1] not in STOPS:
        return False
    return all(codon not in STOPS for codon in cs[1:-1])


def search():
    """Deterministic search over the bases no codon pins. The pinned bases are
    `fP`'s start codon and stop codon and `fM`'s start codon and stop codon,
    translated into reference positions (and into complements for `fM`)."""
    rng = random.Random(SEED)
    pinned = {}
    for position, base in zip(read_positions(PLUS_ARC, 1)[:3], "ATG"):
        pinned[position] = base
    for position, base in zip(read_positions(PLUS_ARC, 1)[-3:], "TAA"):
        pinned[position] = base
    for position, base in zip(read_positions(MINUS_ARC, -1)[:3], "ATG"):
        pinned[position] = COMPLEMENT[base]
    for position, base in zip(read_positions(MINUS_ARC, -1)[-3:], "TAA"):
        pinned[position] = COMPLEMENT[base]
    free = sorted(set(range(1, LENGTH + 1)) - set(pinned))
    for _ in range(400000):
        seq = list("A" * LENGTH)
        for position, base in pinned.items():
            seq[position - 1] = base
        for position in free:
            seq[position - 1] = rng.choice("ACGT")
        text = "".join(seq)
        if usable(codons(text, PLUS_ARC, 1)) and usable(codons(text, MINUS_ARC, -1)):
            return text
    return None


def tracked_text():
    """The ORIGIN bases of the tracked record, upper case, one string."""
    lines = open(RECORD, encoding="ascii").read().splitlines()
    return "".join(
        "".join(line.split()[1:]) for line in lines
        if re.fullmatch(r"\s+\d+ +[A-Za-z ]+", line)
    ).upper()


def check_tracked():
    """Assert the tracked record still satisfies every constraint, with
    Biopython as the reader and this script's walk as the comparison."""
    record = SeqIO.read(RECORD, "genbank")
    seq = str(record.seq).upper()
    assert len(seq) == LENGTH, "record holds %d bases, not %d" % (len(seq), LENGTH)
    assert int(LENGTH) % 3 == 2, "62 must leave 2 bases, not %d" % (LENGTH % 3)
    seen = {}
    for feature in record.features:
        if feature.type != "CDS":
            continue
        gene = feature.qualifiers["gene"][0]
        loc = feature.location
        text = str(loc.extract(seq)).upper()
        arc = PLUS_ARC if gene == "fP" else MINUS_ARC
        strand = 1 if gene == "fP" else -1
        mine = codons(seq, arc, strand)
        bio = [text[i:i + 3] for i in range(0, len(text), 3)]
        assert bio == mine, "%s: Biopython extracts %s, the walk gives %s" % (
            gene, bio, mine)
        assert usable(mine), "%s is not an importable CDS: %s" % (gene, mine)
        stated = feature.qualifiers["translation"][0]
        assert stated == protein(mine), "%s states %s, reads %s" % (
            gene, stated, protein(mine))
        seen[gene] = len(mine)
        print("OK %s %s %s" % (gene, " ".join(mine), protein(mine)))
    assert set(seen) == {"fP", "fM"}, seen
    return 0


def main(argv):
    if "--check" in argv:
        return check_tracked()
    seq = search()
    if seq is None:
        print("no sequence found; the constraints as stated are unsatisfiable")
        return 1
    print(seq)
    print("fP %s  %s" % (PLUS_TEXT, " ".join(codons(seq, PLUS_ARC, 1))))
    print("fM %s  %s" % (MINUS_TEXT, " ".join(codons(seq, MINUS_ARC, -1))))
    print("fP /translation %s" % protein(codons(seq, PLUS_ARC, 1)))
    print("fM /translation %s" % protein(codons(seq, MINUS_ARC, -1)))
    print("length %d, %d %% 3 == %d" % (LENGTH, LENGTH, LENGTH % 3))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
