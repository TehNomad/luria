#!/usr/bin/env python3
r"""Open reading frames of the Milestone 8 view fixtures (8m17, M8-40).

One table, `tests/fixtures/view/orfs.tsv`: the open reading frames of nine
(record, topology, minimum) cases, as `core::orf::find_orfs` must report them
(D036 for the intervals, D010 for what a codon is).

The rules are contract 8m17 section 5, in this script's own words:

* A start codon is `ATG`. A stop is `TAA`, `TAG` or `TGA`. Any other codon -
  including one holding an IUPAC ambiguity code - is **neither**: it does not
  start an ORF and does not end one (D010). Biopython reads `TAR` as a stop and
  `NNN` as `X`, so this script asserts that every record it writes rows for
  holds only `A`, `C`, `G`, `T`; the ambiguous cases are exercised in Rust only
  (contract section 6, section 8 T5).
* One ORF per stop codon: the ORF runs back to the first `M` after the previous
  stop in that frame, which is the **furthest-back** start codon before that
  stop. The regex `M[^*]*\*` over a frame's translation takes exactly that: it
  starts at the leftmost `M` of the run and is greedy over non-stops, so it
  reaches back past any nearer `M`. Taking the `M` nearest the stop instead
  gives a shorter, wrong ORF (contract section 10 G1).
* The ORF's span includes its stop codon, as a CDS feature's does; its residue
  count does not (D010: "The terminal stop codon is not an amino-acid residue").
* On a circular record a codon may wrap the origin and an ORF may cross it, and
  an ORF's span may not exceed the record's length. Read linear, nothing wraps,
  and a reading that runs off either end without a stop is not an ORF. The
  topology is the case's parameter, not what the file states: the view's `T`
  key flips it, so one file appears under both `circular` and `linear`.
* The ORF is kept when its residue count is at least the case's minimum, which
  is inclusive.

How each case is found:

1. Read the strand: the plus strand is the record's own text, the minus strand
   is Biopython's `reverse_complement`. Call that reading `R`, length `L`.
2. Circular: translate `R` over all three frames of `R` repeated three times -
   `R + R + R` and the same coordinate line one base and two bases in. Codon
   `m` of frame `p` sits at coordinate `p + 3m`, and coordinates in `[L, 2L)`
   are the record's own codons: that is the middle copy, and a stop beginning
   there counts exactly once however the record's length wraps. Three frames
   are needed because a record whose length is not a multiple of three does not
   hold its codons in one frame of the repeated text. Linear: translate `R` in
   its three frames; a codon whose three bases do not all stand inside `R` is
   not a codon at all.
3. Find the `M[^*]*\*` matches in that translation and keep the ones whose stop
   codon begins in the middle copy. A circular match can be longer than `L`,
   which is a start codon out of reach across the origin: keep the first `M`
   whose span to the stop's last base fits in the last `L` bases before the
   stop's end, and no ORF at all when none fits.
4. Turn the reading interval into reference coordinates: the plus strand runs
   up from its first base; a minus strand interval starts one base past the
   last base of its span in reading terms, which lands it the other side of the
   reading coordinates. Origin-crossing means two intervals, the one ending at
   the record's end first (D036).

Rows come out in the order `find_orfs` orders them: by the first interval's
start, plus before minus at equal starts, then by that interval's end. The spot
values asserted below are the hand values of contract section 7; `--check`
compares the whole tracked table, so a row that drifts is caught there.

Run from the repository root:

    python -I tests/fixtures/view/reference/orfs.py --print
    python -I tests/fixtures/view/reference/orfs.py --check
"""

from Bio import SeqIO
from Bio.Seq import Seq
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.dirname(HERE)                       # tests/fixtures/view
GENBANK = os.path.join(os.path.dirname(FIXTURE), "genbank")

# (record, file, topology as scanned, minimum residue count, expected rows). A
# case scans the topology named here, which need not be what the file's LOCUS
# line says; the view flips it with `T`. The expected row counts are the
# contract section 6 table, and `crates/core/tests/orf_fixtures.rs` reads them
# back out of the table itself.
CASES = [
    ("pbr322", os.path.join(GENBANK, "raw", "J01749.1.gb"), "circular", 75, 9),
    ("pbr322", os.path.join(GENBANK, "raw", "J01749.1.gb"), "circular", 81, 8),
    ("pbr322", os.path.join(GENBANK, "raw", "J01749.1.gb"), "linear", 75, 9),
    ("synthetic", os.path.join(GENBANK, "synthetic.gb"), "circular", 2, 4),
    ("synthetic", os.path.join(GENBANK, "synthetic.gb"), "linear", 2, 4),
    ("synthetic_linear", os.path.join(GENBANK, "synthetic-linear.gb"),
     "linear", 10, 4),
    ("synthetic_linear", os.path.join(GENBANK, "synthetic-linear.gb"),
     "circular", 10, 5),
    ("frames62", os.path.join(FIXTURE, "frames-62.gb"), "circular", 3, 2),
    ("frames62", os.path.join(FIXTURE, "frames-62.gb"), "linear", 3, 0),
]

# Hand values (contract section 7): the rows a case must hold, whatever else it
# holds. `pbr322`'s 80-residue row is the one min 81 drops. The `synthetic` pair
# is one and the same stop read circular and read linear: across the origin the
# walk reaches the `ATG` at 55 and read linear it cannot, so that stop gives a
# 3-residue ORF starting at base 1 instead. `frames62`'s two rows are that
# record's two origin-crossing CDSs; the record holds a minus-strand reading
# that would span 63 of its 62 bases too, which is no ORF - which is why the
# circular case holds only these two rows.
SPOT = {
    ("pbr322", "circular", 75): [
        ("+", "86..1276", 396),
        ("-", "complement(3293..4153)", 286),
        ("-", "complement(323..565)", 80),
    ],
    ("pbr322", "circular", 81): [
        ("+", "86..1276", 396),
        ("-", "complement(3293..4153)", 286),
    ],
    ("pbr322", "linear", 75): [("-", "complement(323..565)", 80)],
    ("synthetic", "circular", 2): [("+", "join(55..60,1..12)", 5)],
    ("synthetic", "linear", 2): [("+", "1..12", 3)],
    ("synthetic_linear", "linear", 10): [],
    ("synthetic_linear", "circular", 10): [("+", "join(284..393,1..34)", 47)],
    ("frames62", "circular", 3): [
        ("-", "complement(join(53..62,1..8))", 5),
        ("+", "join(56..62,1..11)", 5),
    ],
    ("frames62", "linear", 3): [],
}

HEADER = ["record", "topology", "min_residues", "strand", "location",
          "residues"]
TABLE = os.path.join(FIXTURE, "orfs.tsv")

# A protein run that is an ORF: a first M, no stop before it ends, and the stop
# that ends it. Left-anchored, greedy over non-stops.
ORF_RUN = re.compile(r"M[^*]*\*")


def stated_topology(rec) -> str:
    """What the file says about itself, printed as a note. Not a column of the
    table: the table's `topology` is the topology the case scans, and the note
    says which cases flip what the file states."""
    value = str(rec.annotations.get("topology", "")).lower()
    if value in ("circular", "linear"):
        return value
    if "circular" in rec.annotations:
        return "circular" if bool(rec.annotations["circular"]) else "linear"
    return "unknown"


def reading(seq: str, strand: str) -> str:
    """The reading `R` of one strand, in its own reading direction."""
    return seq if strand == "+" else str(Seq(seq).reverse_complement())


def translations(text: str, cap: int) -> list[str]:
    """`text` translated in each of its three frames, cut at the last whole
    codon ending at or before `cap`.

    Codon `m` of frame `p` sits at coordinate `p + 3m` of `text`, so a caller
    finds a codon's coordinate by arithmetic and never counts. The cut keeps
    every whole codon and no partial one: Biopython answers a partial codon `X`
    rather than leaving it out, and a partial codon is not a codon."""
    out = []
    for phase in (0, 1, 2):
        room = cap - phase
        end = phase + 3 * (room // 3) if room >= 3 else phase
        out.append(str(Seq(text[phase:end]).translate()))
    return out


def first_reachable_m(protein: str, first: int, stop: int, phase: int,
                      reach: int) -> int:
    """Protein index of the first `M` of a match that starts no earlier than
    `reach`, or -1 when the match holds none.

    `first` and `stop` are protein indices of the match's first `M` and its stop
    codon, and `reach` is a nucleotide coordinate: the earliest start an ORF may
    have and still fit under the record's length. The letters between the two
    ends of a match can be `M` - the regex is greedy over non-stops, not over
    non-`M`s - so this scans the match rather than a codon list."""
    for index in range(first + 1, stop + 1):
        if protein[index] != "M":
            continue
        if phase + 3 * index >= reach:
            return index
    return -1


def intervals_of(start: int, span: int, length: int) -> list[tuple[int, int]]:
    """Zero-based half-open reference intervals of a span starting at `start`,
    in D036 order: origin-crossing means the interval ending at the record's
    length comes first."""
    if start + span <= length:
        return [(start, start + span)]
    return [(start, length), (0, start + span - length)]


def location(start: int, span: int, length: int, strand: str) -> str:
    """GenBank 1-based inclusive location text for a reference span."""
    parts = ["%d..%d" % (low + 1, high) for low, high in
             intervals_of(start, span, length)]
    text = parts[0] if len(parts) == 1 else "join(" + ",".join(parts) + ")"
    return text if strand == "+" else "complement(" + text + ")"


def parse_location(text: str) -> tuple[str, list[tuple[int, int]]]:
    """The strand, and the 1-based inclusive spans a location text names, in the
    order they are written.

    This is written out by hand from the text `location` renders, so a
    rendering that loses a `join`, a `complement` or a wrapped half is caught by
    the bases it extracts and the protein they give."""
    strand = "+"
    body = text
    if body.startswith("complement(") and body.endswith(")"):
        strand = "-"
        body = body[len("complement("):-1]
    if body.startswith("join(") and body.endswith(")"):
        spans = body[len("join("):-1].split(",")
    else:
        spans = [body]
    parts = []
    for span in spans:
        low, dot, high = span.partition("..")
        assert dot == ".." and low.isdigit() and high.isdigit(), text
        parts.append((int(low), int(high)))
    return strand, parts


def extract(seq: str, parts: list[tuple[int, int]], strand: str) -> str:
    """The bases a location covers, in the order that strand reads them. GenBank
    reads `complement(join(a, b))` as the reverse complement of the parts
    written in order, and Biopython's `extract` does the same."""
    dna = "".join(seq[low - 1:high] for low, high in parts)
    return dna if strand == "+" else str(Seq(dna).reverse_complement())


def span_of(parts: list[tuple[int, int]]) -> int:
    return sum(high - low + 1 for low, high in parts)


def orfs(seq: str, circular: bool, minimum: int) -> list[tuple[str, int, int, int]]:
    """(strand, zero-based reference start, span in bases, residues) for every
    ORF of one record read at one minimum, in `find_orfs` order.

    One row per stop codon, never one per start codon: each row is the ORF that
    ends at that stop."""
    length = len(seq)
    found = []
    for strand in ("+", "-"):
        r = reading(seq, strand)
        # Three frames of `R` repeated three times for a circular record, and of
        # `R` itself for a linear one. The repeated text is what makes a
        # wrapping codon and an origin-crossing ORF visible as an ordinary run;
        # the middle copy of the coordinate line is what counts each of the
        # record's own codon starts exactly once.
        unrolled = r * 3 if circular else r
        cap = 3 * length if circular else length
        for phase, protein in enumerate(translations(unrolled, cap)):
            for match in ORF_RUN.finditer(protein):
                stop = match.end() - 1
                first = match.start()
                start = phase + 3 * first
                stop_at = phase + 3 * stop
                if circular:
                    if not length <= stop_at < 2 * length:
                        continue      # this stop belongs to another copy
                    if stop_at + 3 - start > length:
                        # The whole match is longer than the record: its first M
                        # is a start the walk cannot reach across the origin.
                        # Look instead at the Ms inside the window.
                        reach = first_reachable_m(protein, first, stop, phase,
                                                  stop_at + 3 - length)
                        if reach < 0:
                            continue
                        first = reach
                        start = phase + 3 * first
                residues = stop - first
                if residues < minimum:
                    continue
                span = stop_at + 3 - start
                assert not circular or span <= length, (start, span, length)
                reading_start = start % length
                if strand == "+":
                    reference = reading_start
                else:
                    # Reading base i of the reverse complement stands at
                    # reference (length - 1 - i) % length, so a minus interval
                    # begins one past the end of its span in reading
                    # coordinates, not at its first reading base.
                    reference = (length - reading_start - span) % length
                found.append((strand, reference, span, residues))
    found.sort(key=lambda row: (row[1], 0 if row[0] == "+" else 1,
                                min(length, row[1] + row[2])))
    return found


def rows_for(record: str, seq: str, topology: str, minimum: int) -> list[list[str]]:
    rows = []
    for strand, start, span, residues in orfs(seq, topology == "circular",
                                             minimum):
        rows.append([record, topology, str(minimum), strand,
                     location(start, span, len(seq), strand), str(residues)])
    return rows


def validate(rows: list[list[str]], sequences: dict[str, str]) -> None:
    """Every row read back: the location it renders, and what those bases
    translate to. Nothing here finds the ORF again - a row's own location text
    is parsed, its bases extracted from the record's raw text and translated,
    which is a different path from the regex that found the run."""
    seen = set()
    for row in rows:
        record, topology, minimum, strand, text, residues = row
        seq = sequences[record]
        length = len(seq)
        residues, minimum = int(residues), int(minimum)
        assert residues >= minimum, row
        assert strand in ("+", "-"), row
        assert topology in ("circular", "linear"), row
        parsed, parts = parse_location(text)
        assert parsed == strand, row
        assert all(1 <= low <= high <= length for low, high in parts), row
        span = span_of(parts)
        assert span % 3 == 0 and span // 3 - 1 == residues, row
        if topology == "circular":
            assert span <= length, row
        else:
            assert len(parts) == 1, row
        # D036: an origin-crossing feature is two intervals, the one ending at
        # the record's length first, and nothing else is ever split.
        if len(parts) == 2:
            assert parts[0][1] == length and parts[1][0] == 1, row
        dna = extract(seq, parts, strand)
        assert len(dna) == span, row
        protein = str(Seq(dna).translate())
        assert len(protein) == residues + 1, (row, protein)
        assert protein[0] == "M" and protein[-1] == "*", (row, protein)
        assert protein.count("*") == 1, (row, protein)
        assert protein.count("X") == 0, (row, protein)
        key = (record, topology, minimum, strand, text, residues)
        assert key not in seen, ("duplicate row", key)
        seen.add(key)


def build() -> tuple[str, list[list[str]]]:
    body = []
    summary = []
    sequences = {}
    for record, path, topology, minimum, expected in CASES:
        parser = "genbank" if path.endswith(".gb") else "fasta"
        records = list(SeqIO.parse(path, parser))
        assert len(records) == 1, "%s: %d records, expected 1" % (
            path, len(records))
        rec = records[0]
        seq = str(rec.seq).upper()
        # D010 answers an ambiguity code as neither a stop nor a residue, while
        # Biopython reads `TAR` as a stop and `NNN` as `X`: a record holding one
        # would not be read correctly here, so this asserts there are none. The
        # ambiguous cases arrive in Rust (T5) instead.
        assert set(seq) <= set("ACGT"), (record, sorted(set(seq)))
        sequences[record] = seq
        rows = rows_for(record, seq, topology, minimum)
        assert len(rows) == expected, (
            "%s %s %d: %d rows, expected %d (contract section 6)" % (
                record, topology, minimum, len(rows), expected))
        for strand, text, residues in SPOT[(record, topology, minimum)]:
            assert [strand, text, str(residues)] in [
                [row[3], row[4], row[5]] for row in rows], (
                "%s %s %d: no row %s %s %s" % (
                    record, topology, minimum, strand, text, residues))
        body += ["\t".join(row) for row in rows]
        stated = stated_topology(rec)
        summary.append([
            record, os.path.relpath(path, os.path.dirname(FIXTURE)),
            "len %d" % len(seq),
            "file states %s" % stated,
            "min %d" % minimum,
            "%d rows" % len(rows),
            "scanned topology flips what the file states"
            if stated != topology and stated != "unknown" else "",
        ])
    validate([line.split("\t") for line in body], sequences)
    table = "\t".join(HEADER) + "\n" + "\n".join(body) + "\n"
    return table, summary


def main() -> int:
    table, summary = build()
    rows = table.splitlines()[1:]
    if "--check" in sys.argv:
        tracked = open(TABLE).read()
        if tracked != table:
            where = next(
                (i for i, (a, b) in enumerate(zip(tracked.splitlines(),
                                                  table.splitlines()), 1)
                 if a != b), None)
            raise AssertionError(
                "%s: tracked and re-derived differ "
                "(%d bytes tracked, %d re-derived"
                % (TABLE, len(tracked), len(table))
                + ("; first difference on line %d:\n  tracked:    %s\n"
                   "  re-derived: %s" % (where, tracked.splitlines()[where - 1],
                                         table.splitlines()[where - 1])
                   if where else "; one file has more lines than the other")
                + ")")
        print("orfs.tsv re-derived: %d rows over %d cases" % (len(rows),
                                                              len(summary)))
        return 0
    if "--print" in sys.argv:
        print("orfs.tsv: %d rows over %d cases" % (len(rows), len(summary)))
        for note in summary:
            print("  %-16s %-24s %-8s %-22s %-8s %s%s"
                  % (note[0], note[1], note[2], note[3], note[4], note[5],
                     ("  <- " + note[6]) if note[6] else ""))
        for record, topology, minimum, strand, text, residues in [
                tuple(r.split("\t")) for r in rows]:
            print("    %-16s %-8s %-3s %-8s %-32s %s"
                  % (record, topology, minimum, strand, text, residues))
        print("every record here holds only ACGT, so Biopython's table and "
              "D010 answer every codon the same; the ambiguous codons are "
              "tested in Rust (T5)")
        return 0
    with open(TABLE, "w") as out:
        out.write(table)
    print("orfs.tsv written: %d rows over %d cases" % (len(rows), len(summary)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
