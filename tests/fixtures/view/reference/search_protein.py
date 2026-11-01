#!/usr/bin/env python3
"""Protein-search tables for the Milestone 8 view fixtures (8a1c, section 4.5).

Two tables, with the headers contract 8f fixes:

* `search-protein-patterns.tsv` - one row per (record, target, pattern): the
  pattern as written, what it normalises to, its length, how many hits it
  found, and what the row is there to pin. A pattern that finds nothing is
  pinned here, because a hits table cannot pin an empty result.
* `search-protein-hits.tsv` - every hit: its order within its pattern, strand,
  frame (`-` for a CDS), zero-based start and length of the bases its codons
  cover, whether that run wraps the origin, the first and last residue numbers
  (`stop` for the stop codon), the target symbols it matched, and the
  top-strand bases it covers.

`target` is `frames` (all six frames of the record) or a CDS id (D041). The
rules are `docs/tasks/contracts/milestone-8-view.md` section 4.5, D050, D052 and D054:

* **Letters.** Pattern `X` matches every target symbol, `*` and `?` included
  (D054). `*` matches only `*`, and any other letter only itself, so an
  unreadable codon `?` is matched only by `X`. Case is ignored.
* **A CDS** is read along its own codons, its terminal stop as `*`. Residue
  numbers are the CDS's, and the stop codon has none (D010, D050).
* **A frame** is read as section 4.3 reads it: complete codons only, no
  wrapping across the origin whatever the topology (D052), a codon holding any
  ambiguity code read as `?`, and numbers counted within the frame (D050).
* **Every overlapping hit**, no cap (D050), **left to right by start**, plus
  strand first on a tie (D054). A hit's run is the bases its codons cover, in
  reference order; only a CDS that crosses the origin can give a run that wraps.

The codon table is hand-coded and checked against Biopython's table 11 for all
64 codons. Frames are rebuilt here and checked against every row
`frame-tables.tsv` holds for the record. A CDS's codons come from Biopython's
reading of the feature location, its residues must equal the file's own
/translation, and for synthetic every codon is checked against
`cds-map-codons.tsv`. Every hit is found twice: by this script's matcher and by
a regular expression over the same symbols. Every hit's `top_strand_bases` is
translated again on its own, and must give back `residues`.
"""

import hashlib
import os
import re
import sys
from typing import NamedTuple

from Bio import SeqIO
from Bio.Data import CodonTable

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.dirname(HERE)
GENBANK = os.path.join(os.path.dirname(FIXTURE), "genbank")

RECORDS = [
    ("synthetic", os.path.join(GENBANK, "synthetic.gb")),
    ("pbr322", os.path.join(GENBANK, "raw", "J01749.1.gb")),
    ("framesamb", os.path.join(FIXTURE, "frames-amb.fa")),
]

# As reference/search_dna.py and reference/frame_tables.py pin it.
CANONICAL_DIGEST = {
    "framesamb":
        "dad6739778314aed63617363569469465d36df95353a236de7ca535254370469",
}

# The usable CDSs of each record, by D041 id, as cds-map-counts.tsv lists them.
CDS_IDS = {"synthetic": ["p1", "m1", "pj"], "pbr322": ["tet", "bla"]}

PROTEIN_LETTERS = "ACDEFGHIKLMNPQRSTVWY"
BASES = "TCAG"
# The standard code, in TCAG order; checked against Biopython in codon_table().
AMINO = "FFLLSSSSYY**CC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG"
COMPLEMENT = {
    "A": "T", "T": "A", "C": "G", "G": "C",
    "R": "Y", "Y": "R", "S": "S", "W": "W", "K": "M", "M": "K",
    "B": "V", "V": "B", "D": "H", "H": "D", "N": "N",
}

PATTERN_HEADER = ["record", "target", "pattern", "normalized_pattern",
                  "pattern_length", "hit_count", "purpose"]
HIT_HEADER = ["record", "target", "pattern", "hit_index", "strand", "frame",
              "start", "length", "wraps_origin", "first", "last", "residues",
              "top_strand_bases"]

TABLES = [
    ("search-protein-patterns.tsv",
     os.path.join(FIXTURE, "search-protein-patterns.tsv")),
    ("search-protein-hits.tsv", os.path.join(FIXTURE, "search-protein-hits.tsv")),
]


class Residue(NamedTuple):
    number: int | None   # None for a CDS's stop codon
    symbol: str
    positions: tuple[int, int, int]   # zero-based, reading order


class Hit(NamedTuple):
    record: str
    target: str
    pattern: str
    index: int
    strand: str
    frame: str
    start: int
    length: int
    wraps: bool
    first: int | None
    last: int | None
    residues: str
    bases: str

    def row(self) -> str:
        cells = [self.record, self.target, self.pattern, str(self.index),
                 self.strand, self.frame, str(self.start), str(self.length),
                 "yes" if self.wraps else "no", number(self.first),
                 number(self.last), self.residues, self.bases]
        return "\t".join(cells)


def number(value: int | None) -> str:
    return "stop" if value is None else str(value)


def codon_table() -> dict[str, str]:
    table = {a + b + c: AMINO[16 * i + 4 * j + k]
             for i, a in enumerate(BASES) for j, b in enumerate(BASES)
             for k, c in enumerate(BASES)}
    bio = CodonTable.unambiguous_dna_by_id[11]
    for codon, amino in table.items():
        expected = "*" if codon in bio.stop_codons else bio.forward_table[codon]
        assert amino == expected, f"{codon}: {amino}, Biopython {expected}"
    return table


CODONS = codon_table()


def translate(codon: str) -> str:
    # Any ambiguity code makes the codon unreadable (D052): `?`, never a guess.
    return CODONS.get(codon, "?")


def reverse_complement(text: str) -> str:
    return "".join(COMPLEMENT[base] for base in reversed(text))


def read_on(text: str, strand: str, positions: tuple[int, int, int]) -> str:
    codon = "".join(text[p] for p in positions)
    return codon if strand == "+" else "".join(COMPLEMENT[b] for b in codon)


def frames(text: str) -> list[tuple[str, int, list[Residue]]]:
    """The six frames, `+1 +2 +3 -1 -2 -3`. Frame `f` on `+` starts at base
    `f - 1`; on `-` its first codon ends at base `len - (f - 1)`."""
    out = []
    for strand in "+-":
        for frame in (1, 2, 3):
            residues: list[Residue] = []
            if strand == "+":
                first = frame - 1
                starts = range(first, len(text) - 2, 3)
                codons = [(s, s + 1, s + 2) for s in starts]
            else:
                end = len(text) - (frame - 1)
                codons = [(e - 1, e - 2, e - 3) for e in range(end, 2, -3)]
            for count, positions in enumerate(codons, 1):
                symbol = translate(read_on(text, strand, positions))
                residues.append(Residue(count, symbol, positions))
            out.append((strand, frame, residues))
    return out


def check_frames_against_table(name: str, text: str, read: list) -> int:
    """Every frame-tables.tsv row for this record must equal this script's."""
    mine = {}
    for strand, frame, residues in read:
        for residue in residues:
            bases = ",".join(str(p + 1) for p in sorted(residue.positions))
            codon = read_on(text, strand, residue.positions)
            mine[(strand, str(frame), str(residue.number))] = (
                residue.symbol, codon, bases)
    seen = 0
    with open(os.path.join(FIXTURE, "frame-tables.tsv"), encoding="utf-8") as f:
        header = f.readline().rstrip("\n").split("\t")
        for line in f:
            row = dict(zip(header, line.rstrip("\n").split("\t")))
            if row["record"] != name:
                continue
            key = (row["strand"], row["frame"], row["residue"])
            got = (row["aa"], row["codon"], row["bases_ref_ascending"])
            assert mine.get(key) == got, f"{name} {key}: table {got}, here {mine.get(key)}"
            seen += 1
    assert seen > 0, f"{name}: no frame-tables.tsv rows to check against"
    return seen


def cds_by_id(record, wanted: str):
    for feature in record.features:
        if feature.type != "CDS":
            continue
        q = feature.qualifiers
        ident = next((q[k][0] for k in ("locus_tag", "gene", "label") if k in q), None)
        if ident == wanted:
            return feature
    raise AssertionError(f"no CDS with id {wanted}")


def cds_residues(name: str, text: str, record, ident: str) -> tuple[str, list[Residue]]:
    feature = cds_by_id(record, ident)
    strand = "+" if feature.location.strand == 1 else "-"
    order = [int(p) for p in feature.location]   # biological order
    assert len(order) % 3 == 0, f"{ident}: {len(order)} bases"
    residues: list[Residue] = []
    for i in range(0, len(order), 3):
        positions = (order[i], order[i + 1], order[i + 2])
        residues.append(Residue(i // 3 + 1, translate(read_on(text, strand, positions)),
                                positions))
    assert residues[-1].symbol == "*", f"{ident}: no terminal stop"
    residues[-1] = residues[-1]._replace(number=None)
    protein = "".join(r.symbol for r in residues[:-1])
    assert protein == feature.qualifiers["translation"][0], f"{ident}: {protein}"
    assert str(feature.extract(record.seq)).upper() == "".join(
        read_on(text, strand, r.positions) for r in residues), f"{ident}: bases"
    return strand, residues


def check_cds_against_table(name: str, ident: str, strand: str,
                            residues: list[Residue]) -> int:
    seen = 0
    with open(os.path.join(FIXTURE, "cds-map-codons.tsv"), encoding="utf-8") as f:
        header = f.readline().rstrip("\n").split("\t")
        for line in f:
            row = dict(zip(header, line.rstrip("\n").split("\t")))
            if row["record"] != name or row["cds"] != ident:
                continue
            residue = residues[int(row["codon_reading"]) - 1]
            bases = ",".join(str(p + 1) for p in sorted(residue.positions))
            assert row["strand"] == strand and row["bases_ref_ascending"] == bases, (
                f"{name} {ident} codon {row['codon_reading']}: {bases}")
            seen += 1
    return seen


def normalize(raw: str) -> str:
    pattern = raw.upper()
    assert pattern and all(c in PROTEIN_LETTERS + "X*" for c in pattern), raw
    return pattern


def letter_matches(pattern_letter: str, symbol: str) -> bool:
    return pattern_letter == "X" or pattern_letter == symbol


def run_of(residues: list[Residue], strand: str, n: int) -> tuple[int, int, bool]:
    """Start, length and wrap of the bases a run of codons covers."""
    first, last = residues[0].positions, residues[-1].positions
    start = first[0] if strand == "+" else last[2]
    length = 3 * len(residues)
    return start, length, start + length > n


def window(text: str, start: int, length: int) -> str:
    doubled = text + text
    return doubled[start:start + length]


def find(record: str, target: str, raw: str, text: str,
         reads: list[tuple[str, str, list[Residue]]]) -> list[Hit]:
    pattern = normalize(raw)
    found = []
    for strand, frame, residues in reads:
        symbols = "".join(r.symbol for r in residues)
        offsets = [o for o in range(len(residues) - len(pattern) + 1)
                   if all(letter_matches(p, symbols[o + i])
                          for i, p in enumerate(pattern))]
        # The second finder: a regex, `X` as any character, everything else literal.
        regex = "".join("." if p == "X" else re.escape(p) for p in pattern)
        oracle = [m.start() for m in re.finditer(f"(?=({regex}))", symbols)]
        assert offsets == oracle, f"{record} {target} {raw}: {offsets} vs {oracle}"
        for o in offsets:
            run = residues[o:o + len(pattern)]
            start, length, wraps = run_of(run, strand, len(text))
            found.append((start, strand, frame, length, wraps, run))
    found.sort(key=lambda h: (h[0], h[1] == "-"))
    hits = []
    for index, (start, strand, frame, length, wraps, run) in enumerate(found, 1):
        hits.append(Hit(record, target, raw, index, strand, frame, start, length,
                        wraps, run[0].number, run[-1].number,
                        "".join(r.symbol for r in run), window(text, start, length)))
    return hits


def check_hit(hit: Hit, text: str) -> None:
    """Translate the hit's bases afresh: they must give back its residues."""
    n = len(text)
    assert hit.bases == window(text, hit.start, hit.length)
    assert hit.length == 3 * len(normalize(hit.pattern))
    assert hit.wraps == (hit.start + hit.length > n)
    if hit.target == "frames":
        assert not hit.wraps, f"{hit}: a frame hit wraps"
    reading = hit.bases if hit.strand == "+" else reverse_complement(hit.bases)
    again = "".join(translate(reading[i:i + 3]) for i in range(0, len(reading), 3))
    assert again == hit.residues, f"{hit.row()}: bases read {again}"
    assert all(letter_matches(p, s) for p, s in zip(normalize(hit.pattern), hit.residues))


def patterns_for(name: str, text: str, frame_reads, cds_reads) -> list[tuple[str, str, str]]:
    """(target, pattern, purpose). Constructed patterns are built from the
    targets, so what they must find follows from the construction."""
    rows: list[tuple[str, str, str]] = []
    plus1 = "".join(r.symbol for r in frame_reads[0][2])
    if name == "synthetic":
        wrap = plus1[-2:] + plus1[:2]
        rows += [
            ("frames", "X", "every codon of every frame, stops included (D054)"),
            ("frames", "*", "every stop codon of every frame"),
            ("frames", "M", "plus and minus hits interleaved by start"),
            ("frames", "XX", "every overlapping pair, in each frame"),
            ("frames", "W*", "a residue then a stop"),
            ("frames", "WX", "X after W: the W* hits and every other W"),
            ("frames", wrap, "built to match only if frame +1 wrapped: no hit (D052)"),
            ("pj", "MKM", "a run across the origin: wraps"),
            ("pj", "mkm", "case ignored: the MKM hit"),
            ("pj", "KM", "starts in the last six bases, wraps"),
            ("pj", "MAW", "after the origin, no wrap"),
            ("pj", "M", "base order, not reading order: residue 3 first (D054)"),
            ("pj", "X", "every codon, the stop at 9 with no number"),
            ("pj", "MKMAWX", "X matches the terminal stop (D054)"),
            ("pj", "MKMAW*", "the whole CDS, stop included"),
            ("pj", "MKMAWW", "as long as the target, last letter differs: no hit"),
            ("pj", "MKMAW*M", "longer than the target: no hit"),
            ("m1", "Q", "minus strand: residue 3 before residue 2 (D054)"),
            ("m1", "MQQ*", "minus strand: the run starts at the stop codon"),
            ("m1", "QX", "X matches the stop on the minus strand"),
            ("p1", "AW*", "a plain plus-strand CDS, ending on the stop"),
            ("p1", "MAWXX", "longer than the target: no hit"),
        ]
    elif name == "pbr322":
        tet = "".join(r.symbol for r in cds_reads["tet"][1])
        bla = "".join(r.symbol for r in cds_reads["bla"][1])
        rows += [
            ("tet", tet[:6], "tet's first six residues"),
            ("tet", tet[-4:], "tet's last three residues and its stop"),
            ("tet", "X*", "one stop, at the end: last is stop"),
            ("tet", "LL", "overlapping pairs in a real protein"),
            ("bla", bla[:6], "bla's first six residues, at its right-hand end"),
            ("bla", "M", "minus strand: residue 1 is the last hit by base (D054)"),
            ("bla", bla[-4:-1] + "X", "X matches bla's stop"),
            ("frames", tet[:6], "the same peptide in six frames: tet's own frame"),
            ("frames", bla[:6], "the same peptide in six frames: bla's own frame"),
            ("frames", "WW", "a pair, found across 4361 bases and six frames"),
        ]
    elif name == "framesamb":
        longest = max(len(r) for _, _, r in frame_reads)
        rows += [
            ("frames", "X", "every codon, `?` and `*` included"),
            ("frames", "**X", "X matches the unreadable TAR codon"),
            ("frames", "***", "TAR is `?`, not a stop, though every reading is one: no hit"),
            ("frames", "VX", "X after V: a residue, and the unreadable RAA and TSK codons"),
            ("frames", "G", "GGN is `?`: no hit, though every reading is G"),
            ("frames", "X" * longest, "as long as the longest frames: one hit in each"),
            ("frames", "X" * (longest + 1), "longer than every frame: no hit"),
            ("frames", "y", "case ignored, both strands"),
        ]
    return rows


def build():
    patterns: list[tuple[str, str, str, str, int, str]] = []
    hits: list[Hit] = []
    stats = {"frame_rows": 0, "cds_rows": 0}
    for name, path in RECORDS:
        parser = "genbank" if path.endswith(".gb") else "fasta"
        parsed = list(SeqIO.parse(path, parser))
        assert len(parsed) == 1, path
        record = parsed[0]
        text = str(record.seq).upper()
        if name in CANONICAL_DIGEST:
            digest = hashlib.sha256(text.encode()).hexdigest()
            assert digest == CANONICAL_DIGEST[name], f"{path}: {digest}"
        frame_reads = frames(text)
        stats["frame_rows"] += check_frames_against_table(name, text, frame_reads)
        cds_reads = {}
        for ident in CDS_IDS.get(name, []):
            strand, residues = cds_residues(name, text, record, ident)
            stats["cds_rows"] += check_cds_against_table(name, ident, strand, residues)
            cds_reads[ident] = (strand, residues)
        for target, raw, purpose in patterns_for(name, text, frame_reads, cds_reads):
            if target == "frames":
                reads = [(s, str(f), r) for s, f, r in frame_reads]
            else:
                reads = [(cds_reads[target][0], "-", cds_reads[target][1])]
            found = find(name, target, raw, text, reads)
            for hit in found:
                check_hit(hit, text)
            pattern = normalize(raw)
            patterns.append((name, target, raw, pattern, len(pattern), purpose))
            hits.extend(found)
            assert len({(h.start, h.strand) for h in found}) == len(found), (
                f"{name} {target} {raw}: two hits share a start and strand")
            order = [(h.start, h.strand != "+") for h in found]
            assert order == sorted(order), (
                f"{name} {target} {raw}: hits must run by start, plus first on a tie")
        constructions(name, text, frame_reads, cds_reads, patterns, hits)
    assert stats["cds_rows"] > 0, "no cds-map-codons.tsv rows checked"
    keys = [(r[0], r[1], r[2]) for r in patterns]
    assert len(keys) == len(set(keys)), "a (record, target, pattern) row is repeated"
    build.stats = (len(patterns), len(hits), stats["frame_rows"], stats["cds_rows"])
    return patterns, hits


def group(hits: list[Hit], name: str, target: str, raw: str) -> list[Hit]:
    return [h for h in hits if (h.record, h.target, h.pattern) == (name, target, raw)]


def constructions(name, text, frame_reads, cds_reads, patterns, hits) -> None:
    """What the constructed rows must show, asserted, so a dropped or changed
    row fails here rather than silently un-pinning a rule."""
    rows = {(r[1], r[2]): r for r in patterns if r[0] == name}

    def got(target, raw):
        assert (target, raw) in rows, f"{name}: {target} {raw} is no longer a row"
        return group(hits, name, target, raw)

    def short(h):
        return (h.start, h.strand, h.frame, h.first, h.last, h.wraps)

    if name == "synthetic":
        plus1 = "".join(r.symbol for r in frame_reads[0][2])
        wrap = plus1[-2:] + plus1[:2]
        assert wrap in plus1 + plus1, "the wrap pattern must match a wrapped +1"
        assert got("frames", wrap) == [], f"{wrap}: a frame wrapped"
        everything = sum(len(r) for _, _, r in frame_reads)
        assert len(got("frames", "X")) == everything
        starts = [h.start for h in got("frames", "X")]
        assert any(starts.count(s) == 2 for s in starts), "no plus/minus tie to order"
        stops = sum(r.symbol == "*" for _, _, rs in frame_reads for r in rs)
        assert stops > 0 and len(got("frames", "*")) == stops
        stop_pairs = {short(h) for h in got("frames", "W*")}
        any_pairs = {short(h) for h in got("frames", "WX")}
        assert stop_pairs and stop_pairs < any_pairs, "X must cover W*'s hits and more"
        m = got("frames", "M")
        assert {h.strand for h in m} == {"+", "-"}, "M must hit both strands"
        assert [(h.start, h.strand) for h in got("pj", "MKM")] == [(54, "+")]
        assert [short(h) for h in got("pj", "mkm")] == [short(h) for h in got("pj", "MKM")]
        assert [short(h) for h in got("pj", "KM")] == [(57, "+", "-", 2, 3, True)]
        assert [short(h) for h in got("pj", "M")] == [
            (0, "+", "-", 3, 3, False), (54, "+", "-", 1, 1, False)]
        assert [h.start for h in got("pj", "X")] == [0, 3, 6, 9, 54, 57]
        assert [short(h) for h in got("pj", "MKMAWX")] == [(54, "+", "-", 1, None, True)]
        assert got("pj", "MKMAWW") == [] and got("pj", "MKMAW*M") == []
        assert [short(h) for h in got("m1", "Q")] == [
            (15, "-", "-", 3, 3, False), (18, "-", "-", 2, 2, False)]
        assert [short(h) for h in got("m1", "MQQ*")] == [(12, "-", "-", 1, None, False)]
        assert got("p1", "MAWXX") == []
    elif name == "pbr322":
        tet_strand, tet = cds_reads["tet"]
        bla_strand, bla = cds_reads["bla"]
        assert (tet_strand, bla_strand) == ("+", "-")
        peptide_t = "".join(r.symbol for r in tet[:6])
        peptide_b = "".join(r.symbol for r in bla[:6])
        cds_t = got("tet", peptide_t)
        cds_b = got("bla", peptide_b)
        assert [h.first for h in cds_t] == [1] and [h.first for h in cds_b] == [1]
        # The CDS hit's bases are a frame hit's bases: same start, same strand.
        for cds_hit, frame_hits in ((cds_t[0], got("frames", peptide_t)),
                                    (cds_b[0], got("frames", peptide_b))):
            same = [h for h in frame_hits
                    if (h.start, h.strand) == (cds_hit.start, cds_hit.strand)]
            assert len(same) == 1, f"{cds_hit.row()}: no frame hit at its bases"
        assert [h.last for h in got("tet", "X*")] == [None]
        m = got("bla", "M")
        assert m[-1].first == 1 and len(m) > 1, "bla: residue 1 must sort last"
        assert got("bla", "".join(r.symbol for r in bla[-4:-1]) + "X")[0].last is None
    elif name == "framesamb":
        longest = max(len(r) for _, _, r in frame_reads)
        full = [f"{s}{f}" for s, f, r in frame_reads if len(r) == longest]
        assert [f"{h.strand}{h.frame}" for h in got("frames", "X" * longest)] != []
        assert sorted(f"{h.strand}{h.frame}" for h in got("frames", "X" * longest)) == sorted(full)
        assert got("frames", "X" * (longest + 1)) == []
        # TAR's every reading is a stop and GGN's every reading is G, yet both read
        # `?` (D052): so `***` and `G` find nothing where a reading would.
        plus1 = frame_reads[0][2]
        codons = [read_on(text, "+", r.positions) for r in plus1]
        tar = codons.index("TAR")
        assert [r.symbol for r in plus1[tar - 2:tar + 1]] == ["*", "*", "?"]
        assert got("frames", "***") == []
        assert [h.first for h in got("frames", "**X") if h.strand == "+"] == [tar - 1]
        assert "GGN" in codons and got("frames", "G") == []
        assert len(got("frames", "X")) == sum(len(r) for _, _, r in frame_reads)


def summary() -> str:
    patterns, hits, frame_rows, cds_rows = build.stats
    return (f"{patterns} patterns, {hits} hits; checked against {frame_rows} "
            f"frame-tables.tsv rows and {cds_rows} cds-map-codons.tsv rows")


def table_text(header: list[str], body: list[str]) -> str:
    return "\t".join(header) + "\n" + "\n".join(body) + "\n"


def check_or_write(argv: list[str]) -> int:
    patterns, hits = build()
    texts = [
        table_text(PATTERN_HEADER, [
            "\t".join([r[0], r[1], r[2], r[3], str(r[4]),
                       str(len(group(hits, r[0], r[1], r[2]))), r[5]])
            for r in patterns]),
        table_text(HIT_HEADER, [h.row() for h in hits]),
    ]
    if "--print" in argv:
        print(texts[0])
        print(summary())
        return 0
    if "--check" in argv:
        bad = False
        for (name, path), text in zip(TABLES, texts):
            if not os.path.isfile(path):
                print(f"MISSING {name}")
                bad = True
                continue
            with open(path, encoding="utf-8") as handle:
                tracked = handle.read()
            if tracked == text:
                print(f"OK {name} {os.path.getsize(path)} bytes")
                continue
            bad = True
            print(f"DIFFERS {name}")
            for number_, (left, right) in enumerate(
                    zip(tracked.split("\n"), text.split("\n")), 1):
                if left != right:
                    print(f"  line {number_}:\n    tracked    {left}\n    re-derived {right}")
                    break
        print(summary())
        return 1 if bad else 0
    for (name, path), text in zip(TABLES, texts):
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
        print(f"wrote {name} {os.path.getsize(path)} bytes")
    print(summary())
    return 0


if __name__ == "__main__":
    raise SystemExit(check_or_write(sys.argv[1:]))
