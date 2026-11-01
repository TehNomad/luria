"""Check the hand-written location text table (D039, contract 9c2 §5).

../locations.tsv gives, for one feature shape on one record, the GenBank
location text that shape is written as: the text `write_edit_location`
(contract 9c2) produces. The planner wrote every `text` cell by hand from
the rules of contract 9c2 §4, which ../README.md's "Location text (9c2)"
restates — M9-17 and M9-18, D006 and D036 — and no cell comes from running
luria.

This script checks each row a second time with Biopython alone: it builds a
minimal record of the row's length and topology with one `misc_feature` at
the row's text, reads it with `Bio.SeqIO.read`, and asks that the parts that
feature has are the shape's ranges — the same ranges, the same order, the
same strand on every part, and Biopython's `BeforePosition` and
`AfterPosition` exactly where the shape writes `<` and `>`. It shares no
code with `crates/`, never runs luria, and writes nothing.

Run from the repository root:

    python3 -I tests/fixtures/editor/reference/locations.py
    python3 -I tests/fixtures/editor/reference/locations.py --check

Both forms print `OK locations.tsv (<n> rows)` and exit 0, or print
`DIFFERS locations.tsv` with the first disagreement and exit 1.
--check is accepted so verify.py can run this like every other check.
"""
import io
import os
import sys

from Bio import SeqIO
from Bio.SeqFeature import BeforePosition, AfterPosition, SimpleLocation

HERE = os.path.dirname(os.path.abspath(__file__))
TABLE = os.path.join(HERE, "..", "locations.tsv")
COLUMNS = ["id", "length", "topology", "shape", "text"]
BASES_PER_LINE = 60   # the GenBank standard, ten bases to a group


class Interval:
    """One range, 1-based inclusive, with the marks written on its ends."""

    def __init__(self, low, high, low_mark, high_mark):
        self.low = low
        self.high = high
        self.low_mark = low_mark
        self.high_mark = high_mark


def parse_shape(text):
    strand, _, rest = text.partition(" ")
    assert strand in ("+", "-"), text
    intervals = []
    for part in rest.split(","):
        low, high = part.split("..")
        intervals.append(Interval(
            int(low.lstrip("<")), int(high.lstrip(">")),
            low.startswith("<"), high.startswith(">"),
        ))
    return strand, intervals


def record_text(length, text, circular):
    """A minimal GenBank record: `length` N bases, one source and one
    misc_feature at `text`, as text a GenBank reader can take."""
    topo = "circular" if circular else "linear"
    head = "LOCUS" + " " * 7 + "loctext"
    prefix = head + " " * (40 - len(head) - len(str(length))) + str(length)
    assert len(prefix) == 40, prefix
    seq = "N" * length
    lines = [
        prefix + " bp    DNA     " + topo.ljust(8) + " SYN 01-JAN-2026",
        "DEFINITION  A record for checking one location text.",
        "ACCESSION   LOCTEXT",
        "VERSION     LOCTEXT.1",
        "KEYWORDS    .",
        "SOURCE      synthetic construct",
        "  ORGANISM  synthetic construct",
        "FEATURES             Location/Qualifiers",
        "     source          1..%d" % length,
        '                     /organism="synthetic construct"',
        '                     /mol_type="other DNA"',
        "     misc_feature    " + text,
        "ORIGIN",
    ]
    for i in range(0, length, BASES_PER_LINE):
        block = seq[i:i + BASES_PER_LINE]
        grouped = " ".join(block[j:j + 10] for j in range(0, len(block), 10))
        lines.append("%9d %s" % (i + 1, grouped))
    lines.append("//")
    return "\n".join(lines) + "\n"


def biopython_feature(row):
    """The row's one misc_feature, as Biopython reads it on its record."""
    length = int(row["length"])
    circular = row["topology"] == "circular"
    handle = io.StringIO(record_text(length, row["text"], circular))
    record = SeqIO.read(handle, "genbank")
    features = [f for f in record.features if f.type == "misc_feature"]
    assert len(features) == 1, row["id"]
    return record, features[0]


def parts_in_shape_order(location, shape):
    """Every range of a location, as Biopython gives it, in the order the
    shape writes them. A plus join keeps the order the text writes them in;
    a minus join is listed 5' to 3', the highest range first, so it is
    reversed into the order `complement(join(…))` and the shape write them."""
    if isinstance(location, SimpleLocation):
        pieces = [location]
    else:
        pieces = list(location.parts)
    if shape == "-":
        pieces.reverse()
    return pieces


def ends_of(piece):
    """One range as 1-based inclusive coordinates, with Biopython's partial
    ends: `before` on a `<`, `after` on a `>`."""
    low = "before" if isinstance(piece.start, BeforePosition) else "exact"
    high = "after" if isinstance(piece.end, AfterPosition) else "exact"
    return int(piece.start) + 1, int(piece.end), low, high


def expected(interval):
    """The shape's range with its marks, in the form `ends_of` answers."""
    return (
        interval.low,
        interval.high,
        "before" if interval.low_mark else "exact",
        "after" if interval.high_mark else "exact",
    )


def disagreement(row):
    """What the row's text does not say, as a message; None if it does."""
    assert row["topology"] in ("linear", "circular"), row["id"]
    shape, intervals = parse_shape(row["shape"])
    record, feature = biopython_feature(row)
    length = int(row["length"])
    if len(record.seq) != length:
        return "the record Biopython reads is %d bases, the row says %d" % (
            len(record.seq), length,
        )
    location = feature.location
    pieces = parts_in_shape_order(location, shape)
    if len(pieces) != len(intervals):
        return "%s: %d range(s) read, the shape has %d" % (
            row["text"], len(pieces), len(intervals),
        )
    strand_code = 1 if shape == "+" else -1
    for piece, interval in zip(pieces, intervals):
        if piece.strand != strand_code or location.strand != strand_code:
            return "%s: a part reads %s, the shape is %s-strand" % (
                row["text"], piece.strand, shape,
            )
        got = ends_of(piece)
        want = expected(interval)
        if got != want:
            return (
                "%s: Biopython reads %d..%d as %s to %s, "
                "where the shape writes %d..%d as %s to %s"
                % (
                    row["text"], got[0], got[1], got[2], got[3],
                    want[0], want[1], want[2], want[3],
                )
            )
    return None


def main():
    with open(TABLE, encoding="utf-8") as handle:
        lines = handle.read().split("\n")
    assert lines[-1] == "", "the table must end with one newline"
    header = lines[0].split("\t")
    if header != COLUMNS:
        print("DIFFERS locations.tsv\n  header: {}".format(header))
        return 1
    ids = set()
    rows = 0
    for number, line in enumerate(lines[1:-1], start=2):
        fields = line.split("\t")
        assert len(fields) == len(COLUMNS), "line {}".format(number)
        row = dict(zip(COLUMNS, fields))
        assert row["id"] not in ids, "duplicate id " + row["id"]
        ids.add(row["id"])
        wrong = disagreement(row)
        if wrong is not None:
            print("DIFFERS locations.tsv")
            print("  line {} ({}): {}".format(number, row["id"], wrong))
            print("  text {:<40} shape {}".format(row["text"], row["shape"]))
            return 1
        rows += 1
    print("OK locations.tsv ({} rows)".format(rows))
    return 0


if __name__ == "__main__":
    sys.exit(main())
