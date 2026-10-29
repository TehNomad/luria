"""Write the CDS-map tables the Milestone 8 view numbers itself with (D040,
D050, contract 8c).

These tables pin, for every CDS the view is expected to draw:

* ../cds-map-codons.tsv — every codon of every small fixture CDS, in
  **reading order**: `codon` counts codons from the first translated codon,
  `residue` is the residue number the view will print (D010: the terminal
  stop codon has none, so its row says `stop`), `bases_reading` is the codon
  read in the direction translation happens, and `bases_ref_ascending` is the
  same three bases listed in **ascending reference order**, which is the
  order D040 records interval columns. The two columns differ for every
  codon of a minus-strand CDS and for any codon split by the origin, and
  that difference is the point of the table (D050: residue_bases returns
  reading order, the interval columns stay reference order).
* ../cds-map-bases.tsv — every base of every small fixture CDS, with the
  codon it belongs to (`codon`, reading order), its `place` inside that codon
  (1..3 in reading order) and the `residue` it produces. This is
  `base_residue`'s whole expected output, and `residue_bases`' round trip
  follows from it.
* ../cds-map-boundaries.tsv — the first three and last three codons of `tet`
  and `bla` in `raw/J01749.1.gb`, with the totals in
  ../cds-map-counts.tsv. Those two CDSs are too long to list in full; the
  plan (§4.2) asks for their first, last and stop codons specifically,
  because they are the records a reader can check by eye against NCBI.
* ../cds-map-refused.tsv — the CDSs the view must not number at all, and the
  rule that refuses each: `ROP` in pBR322 (no id), `len13` in
  `genbank/synthetic.gb` (length not divisible by three), `mj1`/`mj2`
  (alternative start), and `fM` in `frames-62.gb` (`/codon_start=2`, M8-7).

How the positions are computed, and why that is independent of `crates/core`

Biopython parses the records (it shares no code with luria) and supplies each
CDS's interval parts and strand. The codon positions themselves come from a
walk written here in plain arithmetic: a plus-strand CDS reads its parts from
its first part upward in ascending reference coordinates, wrapping from the
record's last base to its first when a part abuts the origin; a minus-strand
CDS reads the **unique decreasing walk** over the same arc, which starts at
the top of the arc and therefore, for an origin-crossing arc, at the low
coordinate. `crates/core` reaches the same codons through
`codon_positions`/`codon_position_at` in `crates/core/src/mutation.rs`, whose
own expectations are pinned by `tests/codon-fixtures.rs` — a third, separate
route, and the reason the pBR322 table below also lists codons 2..4 and the
last three, where `tests/codon-fixtures.rs` records pBR322 codons 2..4.

The walk is cross-checked, not assumed: for every CDS that does not cross the
origin, this script asserts that its own codon list equals Biopython's
`location.extract()` translation of the same feature. A wrapped CDS cannot be
cross-checked that way, because Biopython sorts a compound location's parts
into ascending reference order before applying the complement; the wrapped
case is `fP` (plus strand, where both routes agree) and `fM` (minus strand,
where they do not — see ../README.md and M8-7). `fM` is therefore recorded as refused
(`/codon_start=2`) and gets no codon rows at all. Measured, Biopython's
`location.extract()` of that feature starts `TAC GGG CCC CGT ACC ATG ATT`
while the continuous descending walk over the same arc starts `ATG CCC GGG
GCA TGG TAC TAA`; neither is pinned as the answer here, because choosing one
would be choosing a coordinate rule, which is M8-7's job.

Run from the repository root with the pinned environment
(../../requirements.txt) and `python -I`:

    python -I tests/fixtures/view/reference/cds_map.py
    python -I tests/fixtures/view/reference/cds_map.py --check

Without --check, writes the five tables. With --check, regenerates them in
memory, compares each byte for byte with its tracked file, prints `OK <name>`
or `DIFFERS <name>` followed by the first differing line, exits 1 on any
difference, and writes nothing (D039).
"""
import os
import sys

from Bio import SeqIO

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.dirname(HERE)
GENBANK = os.path.join(os.path.dirname(FIXTURE), "genbank")

RECORDS = [
    ("synthetic", os.path.join(GENBANK, "synthetic.gb")),
    ("synlin", os.path.join(GENBANK, "synthetic-linear.gb")),
    ("pbr322", os.path.join(GENBANK, "raw", "J01749.1.gb")),
    ("frames62", os.path.join(FIXTURE, "frames-62.gb")),
]

# The CDSs the view numbers, as `record:gene`. Usability comes from the
# already-verified GenBank fixtures, not from this script:
# genbank/synthetic-expected.tsv records p1, m1 and pj as imported (with
# their residues), and len13, mj1 and mj2 as refused; genbank/pbr322-cds.tsv
# and genbank/reference/pbr322.py record tet and bla as imported and the ROP
# CDS as having no id; frames-62.gb's fM carries /codon_start=2, which M8-7
# refuses. atg11 shares p1's coordinates and adds nothing, so it is left out.
USABLE = {
    "synthetic": ["p1", "m1", "pj"],
    "synlin": ["synA"],
    "pbr322": ["tet", "bla"],
    "frames62": ["fP", "fM"],
}

# CDSs listed in full, and CDSs listed only at their boundaries.
SMALL = {("synthetic", "p1"), ("synthetic", "m1"), ("synthetic", "pj"),
         ("synlin", "synA"), ("frames62", "fP"), ("frames62", "fM")}
BOUNDARY = {("pbr322", "tet"), ("pbr322", "bla")}

# Refused CDSs: name -> (rule, why), as the view must report it.
REFUSED = [
    ("pbr322", "ROP-feature", "unusable_genbank_cds:no_id",
     "the CDS at index 26 has no /gene, no /locus_tag and no /protein_id"),
    ("synthetic", "len13", "cds_length_not_divisible_by_three",
     "1..13 is 13 bases, and 13 is not divisible by three"),
    ("synthetic", "mj1", "unusable_genbank_cds:alternative_start",
     "complement(join(55..60,1..3)) starts CAT under any reading"),
    ("synthetic", "mj2", "unusable_genbank_cds:alternative_start",
     "join(complement(1..3),complement(55..60)) starts CAT under any reading"),
    ("synthetic", "fuzzy", "unusable_genbank_location:fuzzy_end",
     "the location parser keeps fuzzy ends (D049); the CDS import still refuses"),
    ("synthetic", "ord", "unusable_genbank_location:order",
     "order(...) is not a supported join (D036)"),
    ("synthetic", "exons", "unusable_genbank_location:unsupported_join",
     "join(1..6,7..12) does not abut across the origin (D036)"),
    ("synthetic", "single", "unusable_genbank_location:single_position",
     "a one-base feature is not a CDS interval"),
    ("synthetic", "remote", "unusable_genbank_location:remote_reference",
     "X00001.1:1..12 refers to another record (D036)"),
]

CODE = {
    "TTT": "F", "TTC": "F", "TTA": "L", "TTG": "L", "CTT": "L", "CTC": "L",
    "CTA": "L", "CTG": "L", "ATT": "I", "ATC": "I", "ATA": "I", "ATG": "M",
    "GTT": "V", "GTC": "V", "GTA": "V", "GTG": "V", "TCT": "S", "TCC": "S",
    "TCA": "S", "TCG": "S", "CCT": "P", "CCC": "P", "CCA": "P", "CCG": "P",
    "ACT": "T", "ACC": "T", "ACA": "T", "ACG": "T", "GCT": "A", "GCC": "A",
    "GCA": "A", "GCG": "A", "TAT": "Y", "TAC": "Y", "TAA": "*", "TAG": "*",
    "CAT": "H", "CAC": "H", "CAA": "Q", "CAG": "Q", "AAT": "N", "AAC": "N",
    "AAA": "K", "AAG": "K", "GAT": "D", "GAC": "D", "GAA": "E", "GAG": "E",
    "TGT": "C", "TGC": "C", "TGA": "*", "TGG": "W", "CGT": "R", "CGC": "R",
    "CGA": "R", "CGG": "R", "AGT": "S", "AGC": "S", "AGA": "R", "AGG": "R",
    "GGT": "G", "GGC": "G", "GGA": "G", "GGG": "G",
}
STOPS = {"TAA", "TAG", "TGA"}
COMPLEMENT = {"A": "T", "T": "A", "G": "C", "C": "G"}


class Refused(Exception):
    """A CDS the view must not number."""


def parse_record(path):
    """Biopython's parse of one record: the sequence and its CDS features."""
    record = SeqIO.read(path, "genbank")
    return str(record.seq).upper(), record.features


def arc(parts, length):
    """The CDS's bases as a list of 1-based reference positions in the order
    the CDS lies along the molecule, joining parts that abut (including across
    the origin). Parts come from Biopython in ascending reference order."""
    out = []
    for start, end in parts:  # 1-based inclusive
        out.extend(range(start, end + 1))
    if len(out) > 1 and out[0] == 1 and out[-1] == length:
        # The parts wrap: rotate so the arc runs continuously through the
        # origin (base length followed by base 1).
        cut = 0
        for index in range(1, len(out)):
            if out[index] != out[index - 1] + 1:
                cut = index
                break
        out = out[cut:] + out[:cut]
    return out


def reading(arc, strand):
    """The arc's 1-based positions in reading order. A plus CDS reads the
    continuous arc forward. A minus CDS reads it backward, so it starts at the
    complement of the arc's last base in molecule order - for an arc that
    crosses the origin, at the complement of its lowest-numbered base. That
    is the rule tests/fixtures/genbank/README.md verifies by hand for `pj`,
    `mj1` and `mj2` ("The plus-strand text is ATGAAA + ATG. Its reverse
    complement, CATTTTCAT, reads H F H"), and Biopython's extract() of the
    same features agrees with it, which cross_check below asserts."""
    return list(arc) if strand == 1 else list(reversed(arc))


def feature_parts(feature):
    """Biopython's parts of a location as (start, end) 1-based inclusive,
    ascending, with the strand."""
    loc = feature.location
    parts = [(int(p.start) + 1, int(p.end)) for p in loc.parts]
    return parts, int(loc.strand)


def codons_of(positions, seq, strand, translation):
    """Split reading-order positions into codons and check them against the
    feature's own /translation. A minus-strand codon is the complement of the
    reference bases it sits on; `positions` already runs in reading order."""
    if len(positions) % 3:
        raise Refused("cds_length_not_divisible_by_three")
    cs = []
    for index in range(0, len(positions), 3):
        codon = ""
        for position in positions[index:index + 3]:
            base = seq[position - 1]
            if base not in "ACGT":
                raise Refused("ambiguous_base:" + codon + base)
            codon += COMPLEMENT[base] if strand == -1 else base
        cs.append(codon)
    if cs[0] != "ATG":
        raise Refused("alternative_start:" + cs[0])
    stated = translation[0] if translation else None
    body = cs[:-1] if cs[-1] in STOPS else cs
    residues = "".join(CODE[c] for c in body)
    if stated is not None and residues != stated:
        raise Refused("translation_mismatch:%s!=%s" % (residues, stated))
    return cs


def map_cds(name, parts, strand, seq, length, translation):
    """The reading-order positions and codons of one CDS, as the view needs
    them: parts in ascending reference order joined into one arc that runs
    through the origin, then read in the direction translation happens."""
    forward = arc(parts, length)
    positions = reading(forward, strand)
    cs = codons_of(positions, seq, strand, translation)
    return positions, cs


def rows_for(name, parts, strand, seq, length, translation):
    """(codon rows, base rows) for one usable CDS, all in reading order."""
    positions, cs = map_cds(name, parts, strand, seq, length, translation)
    terminal = cs[-1] in STOPS
    codon_rows = []
    base_rows = []
    count = len(cs) - 1 if terminal else len(cs)
    for index, codon in enumerate(cs, 1):
        residue = "stop" if (terminal and index == len(cs)) else str(index)
        trio = positions[(index - 1) * 3:(index - 1) * 3 + 3]
        codon_rows.append([
            name, "usable", "+" if strand == 1 else "-", str(index), residue,
            codon, ",".join(str(p) for p in sorted(trio)),
        ])
        for place, position in enumerate(trio, 1):
            base_rows.append([
                name, "+" if strand == 1 else "-", str(position), str(index),
                str(place), residue, codon,
            ])
    counts = [
        name, "+" if strand == 1 else "-", str(len(positions)), str(len(cs)),
        str(count), "yes" if terminal else "no",
        ",".join(str(p) for p in sorted(positions[0:3])),
        ",".join(str(p) for p in sorted(positions[-3:])),
        ",".join(str(p) for p in sorted(positions[(count - 1) * 3:count * 3]))
        if count else "-",
    ]
    return codon_rows, base_rows, counts, positions, cs


def cross_check(feature, record_seq, cs, name):
    """Assert this script's codons equal Biopython's own extraction of the same
    feature, for every CDS whose positions are local and exact. Biopython
    agrees with the arc walk on origin-crossing CDSs of both strands -
    including a wrapped minus-strand CDS, whose reading it takes the same way
    - so the wrapped case is checked rather than assumed. Remote references
    (X00001.1:1..12) and fuzzy bounds (`<1`) have no positions to extract, so
    they are skipped."""
    loc = feature.location
    if getattr(loc, "ref", None) is not None:
        return
    if any(type(p).__name__ != "ExactPosition" for p in loc.parts):
        return
    extracted = str(loc.extract(record_seq)).upper()
    bio = [extracted[i:i + 3] for i in range(0, len(extracted), 3)]
    if bio != cs:
        raise AssertionError(
            "%s: Biopython extracts %s, this walk gives %s" % (name, bio[:2], cs[:2]))


def luria_id(feature):
    """The id luria gives a CDS: the first of /locus_tag, /gene, /label
    (D041 G2, as recorded in genbank/pbr322-features.tsv)."""
    for qualifier in ("locus_tag", "gene", "label"):
        values = feature.qualifiers.get(qualifier)
        if values:
            return values[0]
    return "?"


def render():
    """Build every table. Only the CDSs named in USABLE get rows; every other
    CDS belongs to the GenBank fixtures, and ../cds-map-refused.tsv records
    the ones the view has to refuse."""
    codon_rows, base_rows, count_rows, boundary_rows = [], [], [], []
    for record_name, path in RECORDS:
        seq, features = parse_record(path)
        length = len(seq)
        seen = set()
        for feature in features:
            if feature.type != "CDS":
                continue
            name = luria_id(feature)
            if name not in USABLE[record_name] or name in seen:
                continue
            seen.add(name)
            parts, strand = feature_parts(feature)
            marker = "+" if strand == 1 else "-"
            try:
                cl, br, counts, positions, cs = rows_for(
                    name, parts, strand, seq, length,
                    feature.qualifiers.get("translation"))
            except Refused as problem:
                raise AssertionError(
                    "%s:%s should be usable but is not: %s"
                    % (record_name, name, problem))
            cross_check(feature, seq, cs, "%s:%s" % (record_name, name))
            count_rows.append([record_name] + counts)
            if (record_name, name) in BOUNDARY:
                boundary_rows.extend([[record_name] + row for row in cl[:3] + cl[-3:]])
            else:
                codon_rows.extend([[record_name] + row for row in cl])
                base_rows.extend([[record_name] + row for row in br])
    refused_rows = [list(row) for row in REFUSED]
    return codon_rows, base_rows, count_rows, boundary_rows, refused_rows


CODON_HEADER = ["record", "cds", "status", "strand", "codon_reading",
                "residue", "bases_reading", "bases_ref_ascending"]
BASE_HEADER = ["record", "cds", "strand", "position", "codon_reading",
               "place_reading", "residue", "codon"]
COUNT_HEADER = ["record", "cds", "strand", "nt", "codons", "residues",
                "terminal_stop", "first_codon_ref", "last_codon_ref",
                "last_residue_ref"]
REFUSED_HEADER = ["record", "cds", "rule", "detail"]


def validate(codon_rows, base_rows, count_rows):
    """Internal consistency the tables must show before they are written:
    every codon counted once in reading order, residue numbers running 1..n
    with `stop` in the last row exactly when terminal_stop says yes, and the
    base rows covering each CDS's bases exactly once, one row per base, with
    place 1..3 in reading order."""
    by_cds = {}
    for row in codon_rows:
        by_cds.setdefault(row[1], {"codons": [], "name": row[0]})["codons"].append(row)
    for row in base_rows:
        by_cds.setdefault(row[1], {"name": row[0]})
    counts = {row[1]: row for row in count_rows}
    for cds, bundle in by_cds.items():
        counts_row = counts.get(cds)
        assert counts_row, "%s has no counts row" % cds
        codons = bundle.get("codons", [])
        if not codons:
            continue
        assert len(codons) == int(counts_row[4]), "%s: %d rows, %s codons" % (
            cds, len(codons), counts_row[4])
        residues = int(counts_row[5])
        seen = [row[5] for row in codons]
        want = [str(n) for n in range(1, residues + 1)]
        if counts_row[6] == "yes":
            want.append("stop")
        assert seen == want, "%s: residues %s, expected %s" % (cds, seen, want)
        positions = [row[3] for row in base_rows if row[1] == cds]
        assert len(positions) == len(set(positions)), "%s: a base row repeats" % cds
        assert len(positions) == int(counts_row[3]), "%s: %d base rows, %s bases" % (
            cds, len(positions), counts_row[3])
        places = [row[5] for row in base_rows if row[1] == cds]
        assert places == [str((index % 3) + 1) for index in range(len(places))], (
            "%s: places are not 1,2,3 in reading order" % cds)


def tables():
    codon_rows, base_rows, count_rows, boundary_rows, refused_rows = render()
    validate(codon_rows, base_rows, count_rows)
    return [
        ("cds-map-codons.tsv", CODON_HEADER, codon_rows),
        ("cds-map-bases.tsv", BASE_HEADER, base_rows),
        ("cds-map-counts.tsv", COUNT_HEADER, count_rows),
        ("cds-map-boundaries.tsv", CODON_HEADER, boundary_rows),
        ("cds-map-refused.tsv", REFUSED_HEADER, refused_rows),
    ]


def main(argv):
    check = "--check" in argv
    failed = False
    for name, header, rows in tables():
        path = os.path.join(FIXTURE, name)
        text = "\n".join(
            ["\t".join(header)] + ["\t".join(row) for row in rows] + [""])
        if not check:
            with open(path, "w", encoding="ascii", newline="") as handle:
                handle.write(text)
            continue
        with open(path, "r", encoding="ascii", newline="") as handle:
            tracked = handle.read()
        if tracked == text:
            print("OK %s %d rows" % (name, len(rows)))
            continue
        failed = True
        for number, (want, have) in enumerate(
                zip(text.splitlines(), tracked.splitlines()), 1):
            if want != have:
                print("DIFFERS %s line %d: %r != %r" % (name, number, have, want))
                break
        else:
            print("DIFFERS %s: line counts differ" % name)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
