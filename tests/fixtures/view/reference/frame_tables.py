#!/usr/bin/env python3
"""Six-frame translation tables for the Milestone 8 view fixtures (8a1b).

Two tables:

* `frame-tables.tsv` - every complete codon of every frame on both strands,
  with the residue number that frame would show, the translated letter, and the
  codon's reference bases in ascending order.
* `frame-tails.tsv` - for each frame, how many bases the frame covers, how many
  complete codons that makes, and which 1 or 2 bases are left at the far end.
  View mode must not draw those bases at all (D050, M8-18), and must not
  translate them.

The rules are the ones in `docs/tasks/milestone-8-view.md` section 4.3, D052
and D050:

* Frames are numbered 1, 2, 3 on **either** strand. On the plus strand frame f
  starts at base f (1-based) and reads rightwards; on the minus strand frame f
  starts at the last base minus (f - 1) and reads leftwards.
* Frames are counted on the linear coordinates and **never run across the
  origin**, on a circular record exactly as on a linear one. A codon that would
  wrap is not drawn in frame mode. A CDS that crosses the origin is still one
  translation in CDS mode (D018) - that is `cds_map.py`'s business, not this
  script's.
* A frame's last codon is complete or absent. `?` in the `aa` column marks a
  *complete* codon that cannot be read (an ambiguity code, or a codon D010
  refuses to guess at); it never marks a leftover. No fixture here holds a base
  outside ACGT, so this table contains no `?` row - that case arrives with the
  ambiguity record in 8a1c.
* A frame residue is numbered from 1 within that frame and says nothing about a
  CDS residue. The CDS scale lives in `cds-map-bases.tsv`; the two must not be
  confused by a reader of either table.

The `aa` column is computed from this script's own codon table, and then each
frame's whole letter run is compared against Biopython translating the same
linear slice - so a wrong index mapping shows up as a mismatch, not as a wrong
fixture.

Run from the repository root:

    python -I tests/fixtures/view/reference/frame_tables.py --print
    python -I tests/fixtures/view/reference/frame_tables.py --check
"""

from Bio import SeqIO
from Bio.Seq import Seq
import hashlib
import itertools
import os
import sys

STD = "FFLLSSSSYY**CC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG"
CODONS = {
    "".join(t): STD[i]
    for i, t in enumerate(itertools.product("TCAG", "TCAG", "TCAG"))
}

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.dirname(HERE)                       # tests/fixtures/view
GENBANK = os.path.join(os.path.dirname(FIXTURE), "genbank")

FRAMES = [
    ("synthetic", os.path.join(GENBANK, "synthetic.gb")),
    ("synlin", os.path.join(GENBANK, "synthetic-linear.gb")),
    ("frames62", os.path.join(FIXTURE, "frames-62.gb")),
    ("frames61", os.path.join(FIXTURE, "frames-61.fa")),
    ("pbr322", os.path.join(GENBANK, "raw", "J01749.1.gb")),
]

# Records at or below this length store every codon of every frame. Longer ones
# store a sample: the first three codons of each frame, the last three, and
# every `SAMPLE_EVERY`-th codon. The script still *checks* every codon of every
# record against Biopython before it writes anything, so the sample is about how
# much data the repository carries, not about how much is verified. A shifted or
# rotation-based frame implementation breaks the sample anywhere in a long
# record, and the short records carry the interesting arithmetic (lengths 60,
# 393, 62 and 61: 0, 0, 2 and 1 modulo 3, so all three tail cases).
LONG_RECORD = 500
SAMPLE_EVERY = 100

# The README pins this digest by hand with the same pipeline
# `tests/fixtures/fasta/README.md` documents (`grep -v '>' | tr -d '\n\r \t' |
# tr a-z A-Z | sha256sum`). Asserting it here means the prose and the file
# cannot drift apart, and it is why `frames-61.fa` is the length it is: 61 is
# 1 mod 3, which the other records (60, 393, 62, 4361) do not cover.
CANONICAL_DIGEST = {
    "frames61":
        "60a3b3f88a4a09816494160a8b36694dcba03f23c76fa46a370b3db563b1143c",
}

TABLE_HEADER = [
    "record", "seq_len", "topology", "strand", "frame",
    "codon_reading", "residue", "aa", "codon", "bases_ref_ascending",
    "sampled",
]
TAIL_HEADER = [
    "record", "seq_len", "topology", "strand", "frame",
    "bases_in_frame", "complete_codons", "leftover_bases", "leftover",
]

# (strand, frame) -> how many bases the frame covers, and where the leftover
# bases sit: at the high end for the plus strand, at the low end for the minus.
def covered(seq_len: int, strand: str, frame: int) -> int:
    return seq_len - frame + 1 if strand == "+" else seq_len - (frame - 1)


def leftover_span(seq_len: int, strand: str, frame: int) -> str:
    """The 1-based reference positions the frame cannot make a codon from."""
    count = covered(seq_len, strand, frame)
    left = count % 3
    if not left:
        return "-"
    low = seq_len - left + 1 if strand == "+" else 1
    return ",".join(str(p) for p in range(low, low + left))


def topology(rec) -> str:
    """How the file states its own topology. Biopython puts a GenBank LOCUS
    line's `circular`/`linear` in `annotations["topology"]` (older releases used
    a `circular` flag); a FASTA record says nothing, and that is recorded as
    `unknown`. D049 makes the *view* assume linear for a FASTA and show the
    assumption; a fixture table only reports what the file itself says."""
    stated = str(rec.annotations.get("topology", "")).lower()
    if stated in ("circular", "linear"):
        return stated
    if "circular" in rec.annotations:
        return "circular" if bool(rec.annotations["circular"]) else "linear"
    for feature in rec.features:
        if feature.type == "misc_feature":
            for note in feature.qualifiers.get("note", []):
                if "circular" in str(note).lower():
                    return "circular"
    return "unknown"


def positions(seq_len: int, strand: str, frame: int, index: int) -> list[int]:
    """The codon's reference positions (1-based) in the order it is read:
    ascending on the plus strand, descending on the minus strand."""
    if strand == "+":
        first = frame + 3 * (index - 1)
        return [first, first + 1, first + 2]
    start = seq_len - (frame - 1) - 3 * (index - 1)
    return [start, start - 1, start - 2]


def codon_of(seq: str, codon_positions: list[int], strand: str) -> str:
    """The codon **in the order it is read.** A minus-strand codon is the
    complement of each reference base, taken in decreasing position order —
    which is the reverse complement of the *ascending* span, not of the span
    written in reading order. Getting this backwards was a real bug in an
    earlier reference script: it silently mis-reads the first codon of every
    minus-strand feature, and `tests/fixtures/genbank/README.md`'s hand-derived
    `mj1`/`mj2` rows are what caught it there."""
    ascending = sorted(codon_positions)
    text = "".join(seq[p - 1] for p in ascending)
    return str(Seq(text).reverse_complement()) if strand == "-" else text


def stored(index: int, complete: int, seq_len: int) -> bool:
    """Whether this codon of a frame gets a row in the table."""
    if seq_len <= LONG_RECORD:
        return True
    return (index <= 3 or index > complete - 3 or index % SAMPLE_EVERY == 0)


def rows_for(name: str, rec) -> tuple[list[list[str]], list[list[str]]]:
    """(every codon row of every frame, the rows that get a table). The full
    list is what the cross-check compares against Biopython; the second is what
    goes on disk."""
    seq = str(rec.seq).upper()
    seq_len = len(seq)
    circle = topology(rec)
    all_rows: list[list[str]] = []
    kept: list[list[str]] = []
    for strand in ("+", "-"):
        for frame in (1, 2, 3):
            complete = covered(seq_len, strand, frame) // 3
            for index in range(1, complete + 1):
                place = positions(seq_len, strand, frame, index)
                codon = codon_of(seq, place, strand)
                row = [
                    name, str(seq_len), circle, strand, str(frame),
                    str(index), str(index), CODONS.get(codon, "?"), codon,
                    ",".join(str(p) for p in sorted(place)),
                    "yes" if seq_len > LONG_RECORD else "no",
                ]
                all_rows.append(row)
                if stored(index, complete, seq_len):
                    kept.append(row)
    return all_rows, kept


def tails_for(name: str, rec) -> list[list[str]]:
    seq_len = len(rec.seq)
    circle = topology(rec)
    rows: list[list[str]] = []
    for strand in ("+", "-"):
        for frame in (1, 2, 3):
            count = covered(seq_len, strand, frame)
            left = count % 3
            rows.append([
                name, str(seq_len), circle, strand, str(frame), str(count),
                str(count // 3), leftover_span(seq_len, strand, frame),
                "yes" if left else "no",
            ])
    return rows


def cross_check(name: str, rec, rows: list[list[str]]) -> None:
    """Every frame's whole letter run against Biopython translating the same
    linear slice - all codons, stored in the table or not. If the mapping from a
    frame codon to a slice were wrong, the two runs would differ."""
    seq: Seq = rec.seq
    seq_len = len(seq)
    revcomp: Seq = seq.reverse_complement()
    for strand in ("+", "-"):
        for frame in (1, 2, 3):
            count = covered(seq_len, strand, frame)
            complete = count // 3
            slice_of = (seq[frame - 1:] if strand == "+"
                        else revcomp[frame - 1:])[: 3 * complete]
            bio = str(slice_of.translate())
            mine = "".join(row[7] for row in rows
                           if row[0] == name and row[3] == strand
                           and row[4] == str(frame))
            assert len(bio) == len(mine) == complete, (
                f"{name} {strand} frame {frame}: {len(bio)} Biopython letters, "
                f"{len(mine)} here, {complete} complete codons expected")
            assert bio == mine, (
                f"{name} {strand} frame {frame}: Biopython reads "
                f"{bio[:48]}..., this script reads {mine[:48]}...")


def validate(rows: list[list[str]], tails: list[list[str]]) -> None:
    for row in rows:
        assert len(row[8]) == 3, row
        assert row[5] == row[6], f"codon index and residue must agree: {row}"
        assert len(row[9].split(",")) == 3, row
        assert row[7] != "?", f"no fixture here holds a base outside ACGT: {row}"
    for row in tails:
        assert int(row[6]) * 3 + (0 if row[8] == "no" else len(row[7].split(","))) == int(row[5]), row


def build() -> tuple[str, str, int, int]:
    body: list[str] = []
    tail_body: list[str] = []
    for name, path in FRAMES:
        parser = "genbank" if path.endswith(".gb") else "fasta"
        records = list(SeqIO.parse(path, parser))
        assert len(records) == 1, f"{path}: {len(records)} records, expected 1"
        rec = records[0]
        if name in CANONICAL_DIGEST:
            text = str(rec.seq).upper()
            digest = hashlib.sha256(text.encode()).hexdigest()
            assert digest == CANONICAL_DIGEST[name], (
                f"{path}: canonical text hashes to {digest}, "
                f"README pins {CANONICAL_DIGEST[name]}")
        rows, kept = rows_for(name, rec)
        cross_check(name, rec, rows)
        body += ["\t".join(row) for row in kept]
        tail_body += ["\t".join(row) for row in tails_for(name, rec)]
    validate([line.split("\t") for line in body],
             [line.split("\t") for line in tail_body])
    table = "\t".join(TABLE_HEADER) + "\n" + "\n".join(body) + "\n"
    tail = "\t".join(TAIL_HEADER) + "\n" + "\n".join(tail_body) + "\n"
    return table, tail, len(body), len(tail_body)


TABLES = [
    os.path.join(FIXTURE, "frame-tables.tsv"),
    os.path.join(FIXTURE, "frame-tails.tsv"),
]


def main() -> int:
    table, tail, codon_rows, tail_rows = build()
    if "--check" in sys.argv:
        for path, text in zip(TABLES, (table, tail)):
            stored = open(path).read()
            assert stored == text, (
                f"{path}: {len(stored)} bytes tracked, {len(text)} re-derived")
        print(f"frame tables re-derived: {codon_rows} codon rows, "
              f"{tail_rows} tail rows")
        return 0
    if "--print" in sys.argv:
        print(f"frame-tables.tsv: {codon_rows} rows; frame-tails.tsv: {tail_rows} rows")
        wanted = [("frames62", "+", "1"), ("frames62", "-", "1"),
                  ("frames61", "+", "1"), ("pbr322", "+", "1"), ("pbr322", "-", "3")]
        for record, strand, frame in wanted:
            print(f"{record} strand {strand} frame {frame}:")
            shown = 0
            for line in table.splitlines()[1:]:
                fields = line.split("\t")
                if (fields[0], fields[3], fields[4]) == (record, strand, frame):
                    print("  ", line)
                    shown += 1
                    if shown == 3:
                        break
            print("   ...")
        print("tail rows (record, len, topology, strand, frame, bases in frame, "
              "complete codons, leftover bases, leftover):")
        for row in tail.splitlines()[1:]:
            print(" ", row)
        return 0
    for path, text in zip(TABLES, (table, tail)):
        with open(path, "w") as out:
            out.write(text)
    print(f"frame tables written: {codon_rows} codon rows, {tail_rows} tail rows")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
