#!/usr/bin/env python3
"""Block-layout column tables for the Milestone 8 view fixtures (8a2, plan 5.1).

Two tables:

* `layout-blocks.tsv` - every block of every case: the record, the width the
  layout is given, the mode, the line width that width allows, and each
  block's first and last base (1-based, inclusive) and base count. This is
  where D055's shortened line is pinned.
* `layout-cells.tsv` - for chosen blocks, every item the layout places: the
  line it is on (0 at the top) and that line's role, what kind of item it is,
  the first column it occupies (0-based, from the left edge of the width
  given), how many columns it covers, and its text. Columns, not drawn
  screens (plan section 6): no border, glyph or style is pinned here.

The rules are `docs/tasks/milestone-8-view.md` section 5.1, D050 (M8-19) and
D055, as contract 8j states them:

* **Gutter.** `d = max(2, digits of the record length)`. A line's label (the
  first base's number, or a frame label such as `+1`) is right-aligned in
  columns `[0, d)`; column `d` is blank; bases start at `g = d + 1`.
* **Line width.** The largest multiple of 10 (no translation) or 30 (any
  translation mode) with `2d + 2 + N <= width`. The top strand's right number,
  the line's last base, starts one blank column after the last base.
* **Blocks.** Without a CDS selected, block `k` is bases `[kN, (k+1)N)`. With
  one, lines start at base 1, and a line that would end inside a codon of the
  selected CDS ends just before that codon (D055).
* **Lines, top to bottom.** Up to four feature lanes, then a `more` line if
  more features overlap; for a plus-strand CDS its residues then their numbers;
  for three or six frames `+3`, `+2`, `+1`; the top strand; the bottom strand;
  for a minus-strand CDS its numbers then its residues; for six frames `-1`,
  `-2`, `-3`.
* **Feature lanes.** Every drawn feature except `source` that has a base in
  the block, longest drawn length first, then file order; at most four; the
  rest counted on the `more` line. A lane covers the columns of the feature's
  bases in the block, as one cell run per stretch of consecutive bases.
* **Residues.** Each over its codon's middle base, on the line holding that
  base (D055). A CDS's stop codon is `*`; a frame's unreadable codon is `?`.
* **Residue numbers** (selected CDS only, D055). Every residue whose number is
  a multiple of 5, left to right, then the line's leftmost numbered residue;
  each centred on its residue (`start = column - (len - 1) // 2`), moved
  inward to stay over the line's bases, and dropped if it would overlap or
  touch a number already placed. The stop codon has no number (D010).

Independent of luria: the codon table is checked against Biopython's table 11;
a CDS's codons come from Biopython's reading of its location, its residues
must equal the file's /translation, and its codons must match
`cds-map-codons.tsv` where that table has rows; frames are checked against
every `frame-tables.tsv` row for the record; whether a synthetic feature is
drawn comes from `../genbank/synthetic-unused.tsv` (a location refusal other
than `fuzzy_end` is not drawn, D053).
"""

import hashlib
import itertools
import os
import sys

from Bio import SeqIO
from Bio.Data import CodonTable

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.dirname(HERE)
GENBANK = os.path.join(os.path.dirname(FIXTURE), "genbank")

RECORDS = {
    "synthetic": os.path.join(GENBANK, "synthetic.gb"),
    "pbr322": os.path.join(GENBANK, "raw", "J01749.1.gb"),
    "frames62": os.path.join(FIXTURE, "frames-62.gb"),
    "framesamb": os.path.join(FIXTURE, "frames-amb.fa"),
}
CANONICAL_DIGEST = {
    "framesamb":
        "dad6739778314aed63617363569469465d36df95353a236de7ca535254370469",
}

# (record, width, mode, blocks): `all`, or the zero-based bases whose blocks
# are tabled cell by cell. Every block of every case is in layout-blocks.tsv.
CASES = [
    ("synthetic", 78, "off", "all"),
    ("synthetic", 78, "cds:pj", "all"),
    ("synthetic", 78, "cds:m1", "all"),
    ("synthetic", 78, "six", "all"),
    ("frames62", 78, "off", "all"),
    ("frames62", 78, "cds:fP", "all"),
    ("frames62", 78, "cds:fM", "all"),
    ("frames62", 78, "six", "all"),
    ("framesamb", 78, "six", "all"),
    ("pbr322", 78, "off", [0, 4360]),
    ("pbr322", 78, "cds:tet", [85, 1275]),
    ("pbr322", 78, "cds:bla", [3292, 4152]),
    ("pbr322", 40, "cds:tet", [60, 88]),
    ("pbr322", 40, "cds:bla", [3292, 4152]),
    ("pbr322", 78, "three", [0]),
    ("pbr322", 118, "six", [4300]),
]

BLOCK_HEADER = ["record", "width", "mode", "line_width", "block", "first_base",
                "last_base", "bases"]
CELL_HEADER = ["record", "width", "mode", "block", "line", "role", "kind",
               "column", "cells", "value"]
TABLES = [
    ("layout-blocks.tsv", os.path.join(FIXTURE, "layout-blocks.tsv")),
    ("layout-cells.tsv", os.path.join(FIXTURE, "layout-cells.tsv")),
]

BASES = "TCAG"
AMINO = "FFLLSSSSYY**CC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG"
CODONS = {a + b + c: AMINO[16 * i + 4 * j + k]
          for (i, a), (j, b), (k, c) in itertools.product(enumerate(BASES), repeat=3)}
COMPLEMENT = {
    "A": "T", "T": "A", "C": "G", "G": "C",
    "R": "Y", "Y": "R", "S": "S", "W": "W", "K": "M", "M": "K",
    "B": "V", "V": "B", "D": "H", "H": "D", "N": "N",
}


def check_codon_table() -> None:
    bio = CodonTable.unambiguous_dna_by_id[11]
    for codon, amino in CODONS.items():
        expected = "*" if codon in bio.stop_codons else bio.forward_table[codon]
        assert amino == expected, f"{codon}: {amino}, Biopython {expected}"


def translate(codon: str) -> str:
    return CODONS.get(codon, "?")   # any ambiguity code: unreadable (D052)


def read_on(text: str, strand: str, positions) -> str:
    codon = "".join(text[p] for p in positions)
    return codon if strand == "+" else "".join(COMPLEMENT[b] for b in codon)


def display_name(feature) -> str:
    for key in ("label", "gene", "locus_tag", "product"):
        for value in feature.qualifiers.get(key, []):
            if value.strip():
                return value
    return feature.type


def not_drawn(name: str) -> set[int]:
    """Feature indices whose location the view refuses (D053), from the
    synthetic fixture's own refusal table; other records refuse none."""
    if name != "synthetic":
        return set()
    out = set()
    with open(os.path.join(GENBANK, "synthetic-unused.tsv"), encoding="utf-8") as f:
        header = f.readline().rstrip("\n").split("\t")
        for line in f:
            row = dict(zip(header, line.rstrip("\n").split("\t")))
            refusal = row["refusal"]
            if (refusal.startswith("unusable_genbank_location:")
                    and refusal != "unusable_genbank_location:fuzzy_end"):
                out.add(int(row["index"]))
    assert out == {18, 19, 20, 21}, out
    return out


class Feature:
    def __init__(self, index, key, name, strand, positions):
        self.index, self.key, self.name = index, key, name
        self.strand, self.positions = strand, positions


def features(name: str, record) -> list[Feature]:
    refused = not_drawn(name)
    out = []
    for index, f in enumerate(record.features):
        if index in refused:
            continue
        positions = sorted({int(p) for p in f.location})
        strand = "-" if f.location.strand == -1 else "+"
        out.append(Feature(index, f.type, display_name(f), strand, positions))
    return out


class Codon:
    def __init__(self, positions, symbol, number):
        self.positions, self.symbol, self.number = positions, symbol, number

    @property
    def middle(self) -> int:
        return self.positions[1]


def cds_codons(name: str, record, text: str, ident: str) -> tuple[str, list[Codon]]:
    for f in record.features:
        q = f.qualifiers
        found = next((q[k][0] for k in ("locus_tag", "gene", "label") if k in q), None)
        if f.type == "CDS" and found == ident:
            break
    else:
        raise AssertionError(f"{name}: no CDS {ident}")
    strand = "-" if f.location.strand == -1 else "+"
    order = [int(p) for p in f.location]
    codons = []
    for i in range(0, len(order), 3):
        positions = tuple(order[i:i + 3])
        codons.append(Codon(positions, translate(read_on(text, strand, positions)), i // 3 + 1))
    assert codons[-1].symbol == "*", f"{ident}: no terminal stop"
    codons[-1].number = None
    protein = "".join(c.symbol for c in codons[:-1])
    assert protein == f.qualifiers["translation"][0], f"{ident}: {protein}"
    check_against_cds_map(name, ident, strand, codons)
    return strand, codons


def check_against_cds_map(name: str, ident: str, strand: str, codons: list[Codon]) -> None:
    with open(os.path.join(FIXTURE, "cds-map-codons.tsv"), encoding="utf-8") as f:
        header = f.readline().rstrip("\n").split("\t")
        for line in f:
            row = dict(zip(header, line.rstrip("\n").split("\t")))
            if row["record"] != name or row["cds"] != ident:
                continue
            codon = codons[int(row["codon_reading"]) - 1]
            bases = ",".join(str(p + 1) for p in sorted(codon.positions))
            assert (row["strand"], row["bases_ref_ascending"]) == (strand, bases), (
                f"{name} {ident} codon {row['codon_reading']}")


def frame_codons(text: str) -> dict[str, list[Codon]]:
    """`+1 +2 +3 -1 -2 -3`: complete codons only, no wrapping (D052)."""
    out = {}
    n = len(text)
    for frame in (1, 2, 3):
        out[f"+{frame}"] = [
            Codon((s, s + 1, s + 2), translate(read_on(text, "+", (s, s + 1, s + 2))), k)
            for k, s in enumerate(range(frame - 1, n - 2, 3), 1)]
        out[f"-{frame}"] = [
            Codon((e - 1, e - 2, e - 3), translate(read_on(text, "-", (e - 1, e - 2, e - 3))), k)
            for k, e in enumerate(range(n - (frame - 1), 2, -3), 1)]
    return out


def check_frames(name: str, text: str, frames: dict[str, list[Codon]]) -> int:
    mine = {}
    for label, codons in frames.items():
        for c in codons:
            mine[(label[0], label[1], str(c.number))] = (
                c.symbol, ",".join(str(p + 1) for p in sorted(c.positions)))
    seen = 0
    with open(os.path.join(FIXTURE, "frame-tables.tsv"), encoding="utf-8") as f:
        header = f.readline().rstrip("\n").split("\t")
        for line in f:
            row = dict(zip(header, line.rstrip("\n").split("\t")))
            if row["record"] != name:
                continue
            key = (row["strand"], row["frame"], row["residue"])
            assert mine.get(key) == (row["aa"], row["bases_ref_ascending"]), (name, key)
            seen += 1
    assert seen > 0, name
    return seen


def gutter(length: int) -> int:
    return max(2, len(str(length)))


def line_width(length: int, width: int, mode: str) -> int:
    step = 10 if mode == "off" else 30
    d = gutter(length)
    n = (width - 2 * d - 2) // step * step
    assert n >= step, f"width {width} leaves no line for {length} bases in {mode}"
    return n


def blocks(length: int, n: int, codons: list[Codon] | None) -> list[tuple[int, int]]:
    codon_of = {}
    for i, c in enumerate(codons or []):
        for p in c.positions:
            codon_of[p] = i
    out, start = [], 0
    while start < length:
        end = min(start + n, length)
        if codons and end < length:
            left, right = codon_of.get(end - 1), codon_of.get(end)
            if left is not None and left == right:
                end = min(p for p in codons[left].positions if start <= p < end)
        out.append((start, end))
        start = end
    return out


def place_numbers(residues: list[tuple[int, Codon]], g: int, n: int) -> list[tuple[int, str]]:
    numbered = [(col, c) for col, c in residues if c.number is not None]
    order = [rc for rc in numbered if rc[1].number % 5 == 0]
    if numbered:
        leftmost = min(numbered, key=lambda rc: rc[0])
        if leftmost not in order:
            order.append(leftmost)
    kept: list[tuple[int, str]] = []
    for col, c in order:
        text = str(c.number)
        start = col - (len(text) - 1) // 2
        start = max(g, min(start, g + n - len(text)))
        end = start + len(text)
        if all(start > k + len(t) or end < k for k, t in kept):
            kept.append((start, text))
    return sorted(kept)


def lay_out(name, text, feats, mode, cds, n, block) -> list[tuple]:
    """Cells of one block: (line, role, kind, column, cells, value)."""
    s, e = block
    d = gutter(len(text))
    g = d + 1
    lines: list[tuple[str, list[tuple]]] = []

    lanes = [f for f in feats if f.key != "source" and any(s <= p < e for p in f.positions)]
    lanes.sort(key=lambda f: (-len(f.positions), f.index))
    for number, f in enumerate(lanes[:4], 1):
        inside = [p for p in f.positions if s <= p < e]
        runs, first = [], inside[0]
        for a, b in zip(inside, inside[1:] + [None]):
            if b != a + 1:
                runs.append((first, a))
                first = b
        cells = [("feature", g + a - s, b - a + 1, f"{f.name} ({f.strand})") for a, b in runs]
        lines.append((f"lane{number}", cells))
    if len(lanes) > 4:
        lines.append(("more", [("more", g, len(f"+{len(lanes) - 4} more"),
                                f"+{len(lanes) - 4} more")]))

    def residue_line(label, codons, with_label):
        cells = [("label", d - len(label), len(label), label)] if with_label else []
        on = [(g + c.middle - s, c) for c in codons if s <= c.middle < e]
        cells += [("residue", col, 1, c.symbol) for col, c in sorted(on, key=lambda x: x[0])]
        return cells, on

    def number_line(on):
        return [("residue_number", col, len(t), t) for col, t in place_numbers(on, g, e - s)]

    top = [("left_number", d - len(str(s + 1)), len(str(s + 1)), str(s + 1)),
           ("bases", g, e - s, text[s:e]),
           ("right_number", g + (e - s) + 1, len(str(e)), str(e))]
    bottom = [("complement", g, e - s, "".join(COMPLEMENT[b] for b in text[s:e]))]

    if mode.startswith("cds:"):
        strand, codons = cds
        cells, on = residue_line("", codons, False)
        if strand == "+":
            lines += [("residues", cells), ("numbers", number_line(on)), ("top", top),
                      ("bottom", bottom)]
        else:
            lines += [("top", top), ("bottom", bottom), ("numbers", number_line(on)),
                      ("residues", cells)]
    elif mode in ("three", "six"):
        frames = cds
        for label in ("+3", "+2", "+1"):
            lines.append((label, residue_line(label, frames[label], True)[0]))
        lines += [("top", top), ("bottom", bottom)]
        if mode == "six":
            for label in ("-1", "-2", "-3"):
                lines.append((label, residue_line(label, frames[label], True)[0]))
    else:
        lines += [("top", top), ("bottom", bottom)]

    out = []
    for line, (role, cells) in enumerate(lines):
        for kind, column, width, value in cells:
            out.append((line, role, kind, column, width, value))
    return out


def validate_case(name, text, width, mode, n, all_blocks, cds_codon_list) -> None:
    length = len(text)
    assert all_blocks[0][0] == 0 and all_blocks[-1][1] == length
    assert all(a[1] == b[0] for a, b in zip(all_blocks, all_blocks[1:]))
    short = [blk for blk in all_blocks[:-1] if blk[1] - blk[0] != n]
    if cds_codon_list is None:
        assert not short, f"{name} {mode}: short blocks without a CDS {short}"
        return
    assert all(n - 2 <= blk[1] - blk[0] < n for blk in short), (name, mode, short)
    starts = {blk[0] for blk in all_blocks}
    for c in cds_codon_list:
        ps = sorted(c.positions)
        if ps[-1] - ps[0] != 2:
            continue   # the codon the origin splits: across the record's ends, not a block edge
        assert not any(ps[0] < st <= ps[-1] for st in starts), (
            f"{name} {mode}: a block starts inside the codon at {ps}")


def validate_cells(name, text, width, rows) -> None:
    by_line: dict[tuple, list] = {}
    for line, role, kind, column, cells, value in rows:
        assert 0 <= column and column + cells <= width, (name, line, role, kind, column)
        assert cells == len(value) or kind in ("feature",), (name, kind, value)
        by_line.setdefault(line, []).append((column, column + cells, kind))
    for line, spans in by_line.items():
        spans.sort()
        for (a0, a1, k0), (b0, b1, k1) in zip(spans, spans[1:]):
            assert a1 <= b0, f"{name}: line {line} {k0} [{a0},{a1}) overlaps {k1} [{b0},{b1})"


def build():
    check_codon_table()
    block_rows, cell_rows = [], []
    stats = {"frame_rows": 0}
    loaded = {}
    for name, path in RECORDS.items():
        record = SeqIO.read(path, "genbank" if path.endswith(".gb") else "fasta")
        text = str(record.seq).upper()
        if name in CANONICAL_DIGEST:
            assert hashlib.sha256(text.encode()).hexdigest() == CANONICAL_DIGEST[name], name
        frames = frame_codons(text)
        stats["frame_rows"] += check_frames(name, text, frames)
        feats = features(name, record) if path.endswith(".gb") else []
        loaded[name] = (record, text, frames, feats)
    for name, width, mode, chosen in CASES:
        record, text, frames, feats = loaded[name]
        n = line_width(len(text), width, mode)
        cds = None
        codon_list = None
        if mode.startswith("cds:"):
            cds = cds_codons(name, record, text, mode[4:])
            codon_list = cds[1]
        elif mode in ("three", "six"):
            cds = frames
        all_blocks = blocks(len(text), n, codon_list)
        validate_case(name, text, width, mode, n, all_blocks, codon_list)
        for index, (s, e) in enumerate(all_blocks):
            block_rows.append([name, str(width), mode, str(n), str(index), str(s + 1),
                               str(e), str(e - s)])
        wanted = (range(len(all_blocks)) if chosen == "all" else
                  sorted({i for i, (s, e) in enumerate(all_blocks)
                          for p in chosen if s <= p < e}))
        for index in wanted:
            rows = lay_out(name, text, feats, mode, cds, n, all_blocks[index])
            validate_cells(name, text, width, rows)
            for line, role, kind, column, cells, value in rows:
                cell_rows.append([name, str(width), mode, str(index), str(line), role, kind,
                                  str(column), str(cells), value])
    constructions(block_rows, cell_rows)
    build.stats = (len(block_rows), len(cell_rows), stats["frame_rows"])
    return block_rows, cell_rows


def constructions(block_rows, cell_rows) -> None:
    """Values checked by hand, so a change to a rule fails here by name."""
    def blocks_of(record, width, mode):
        return [(int(r[5]), int(r[6])) for r in block_rows
                if (r[0], r[1], r[2]) == (record, str(width), mode)]

    # D055's own example: tet on 30-base lines, the line from 61 ends at 88.
    tet30 = blocks_of("pbr322", 40, "cds:tet")
    assert (61, 88) in tet30 and (89, 118) in tet30, tet30[:5]
    assert tet30[:2] == [(1, 30), (31, 60)]
    # frames-62: both origin CDSs put a block edge at 58, not 60 (codon 59-61).
    assert blocks_of("frames62", 78, "cds:fP") == [(1, 58), (59, 62)]
    assert blocks_of("frames62", 78, "cds:fM") == [(1, 58), (59, 62)]
    assert blocks_of("frames62", 78, "six") == [(1, 60), (61, 62)]
    assert blocks_of("synthetic", 78, "off") == [(1, 60)]
    # pBR322 without translation: 60-base lines (68 columns are free at width 78).
    assert blocks_of("pbr322", 78, "off")[:2] == [(1, 60), (61, 120)]

    def cells(record, width, mode, block, role=None, kind=None):
        return [r for r in cell_rows
                if (r[0], r[1], r[2], r[3]) == (record, str(width), mode, str(block))
                and (role is None or r[5] == role) and (kind is None or r[6] == kind)]

    # tet's M sits over base 87, the middle of its ATG at 86-88, and is
    # numbered 1, on the 61-88 line at width 40: column 5 + (86 - 60) = 31.
    block = tet30.index((61, 88))
    m = cells("pbr322", 40, "cds:tet", block, "residues", "residue")
    assert [(r[7], r[9]) for r in m] == [("31", "M")], m
    numbers = cells("pbr322", 40, "cds:tet", block, "numbers")
    assert [(r[7], r[9]) for r in numbers] == [("31", "1")], numbers
    # pBR322, tet, width 78: lines of 60 from base 1; the line from 61 ends at
    # 118 (tet's codon 119-121 would straddle 120), and from 119 every line is
    # 60 again, so bases 1259-1318 are a block. Residue 392 (codon 1259-1261)
    # is the leftmost, over 1260 at column 5 + 1 = 6, its number moved inward
    # from 5 to 5; residue 395 (1268-1270) is over 1269 at column 15, "395"
    # centred from 14. The stop codon (1274-1276) is `*` at column 21, with no
    # number.
    tet60 = blocks_of("pbr322", 78, "cds:tet")
    assert tet60[1] == (61, 118) and (1259, 1318) in tet60, tet60[:3]
    block = tet60.index((1259, 1318))
    numbers = cells("pbr322", 78, "cds:tet", block, "numbers")
    assert [(r[7], r[9]) for r in numbers] == [("5", "392"), ("14", "395")], numbers
    stop = [r for r in cells("pbr322", 78, "cds:tet", block, "residues") if r[9] == "*"]
    assert [r[7] for r in stop] == ["21"], stop
    # Lanes, longest first then file order: on the 61-118 line the tet gene and
    # the tet CDS (1191 bases each, gene first in the file) come before the two
    # 4-base misc_binding sites at 67-70 and 80-83.
    lanes = [(r[5], r[7], r[8], r[9])
             for r in cells("pbr322", 78, "cds:tet", 1, kind="feature")]
    assert lanes == [("lane1", "30", "33", "tet (+)"), ("lane2", "30", "33", "tet (+)"),
                     ("lane3", "11", "4", "misc_binding (+)"),
                     ("lane4", "24", "4", "misc_binding (+)")], lanes
    # synthetic pj crosses the origin of a 60-base record: codon 1 (55-57)
    # puts M at column 3 + 55 = 58 and codon 3 (1-3) another M at column 4;
    # its lane is two runs on the one line.
    pj = cells("synthetic", 78, "cds:pj", 0, "residues", "residue")
    assert [(r[7], r[9]) for r in pj] == [
        ("4", "M"), ("7", "A"), ("10", "W"), ("13", "*"), ("58", "M"), ("61", "K")], pj
    # frames-62's fP codon 3 is split by the origin (bases 62, 1, 2): its T sits
    # over base 1, its middle, at column 3 of block 0; base 62 is on block 1.
    fp = cells("frames62", 78, "cds:fP", 0, "residues", "residue")
    assert ("3", "T") in [(r[7], r[9]) for r in fp], fp
    lane = [r for r in cells("synthetic", 78, "off", 0, kind="feature") if r[9] == "pj (+)"]
    assert [(r[7], r[8]) for r in lane] == [("3", "12"), ("57", "6")], lane
    # a minus CDS: numbers then residues below the strands, numbers falling left to right.
    m1 = cells("synthetic", 78, "cds:m1", 0)
    roles = []
    for r in m1:
        if r[5] not in roles:
            roles.append(r[5])
    assert roles[-4:] == ["top", "bottom", "numbers", "residues"], roles
    # synthetic has more drawn features at base 1 than four lanes hold.
    more = cells("synthetic", 78, "off", 0, "more")
    assert len(more) == 1 and more[0][9].startswith("+"), more
    # framesamb six frames: the unreadable TAR codon is `?` over base 14.
    plus1 = cells("framesamb", 78, "six", 0, "+1", "residue")
    assert ("16", "?") in [(r[7], r[9]) for r in plus1], plus1


def summary() -> str:
    blocks_n, cells_n, frame_rows = build.stats
    return (f"{len(CASES)} cases, {blocks_n} blocks, {cells_n} cells; frames checked "
            f"against {frame_rows} frame-tables.tsv rows")


def table_text(header, rows) -> str:
    return "\t".join(header) + "\n" + "\n".join("\t".join(r) for r in rows) + "\n"


def check_or_write(argv: list[str]) -> int:
    block_rows, cell_rows = build()
    texts = [table_text(BLOCK_HEADER, block_rows), table_text(CELL_HEADER, cell_rows)]
    if "--print" in argv:
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
            for number, (left, right) in enumerate(
                    zip(tracked.split("\n"), text.split("\n")), 1):
                if left != right:
                    print(f"  line {number}:\n    tracked    {left}\n    re-derived {right}")
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
