#!/usr/bin/env python3
"""DNA-search tables for the Milestone 8 view fixtures (8a1c, section 4.4).

Three tables:

* `search-letters.tsv` - the matching rule itself, for every ordered pair of
  IUPAC letters: does a template letter match a pattern letter under luria's
  rule, what an intersection rule would answer, and whether the two agree at
  that pair. 225 rows, because the rule is about letters, not about a record.
* `search-dna-patterns.tsv` - one row per (record, pattern): the pattern as
  written, what it normalises to, its reverse complement, what the file says
  about topology, whether the search was allowed to wrap, how many hits it
  found, and what the row is there to pin. This is where a pattern that finds
  **nothing** is pinned; a hits table cannot pin an empty result.
* `search-dna-hits.tsv` - every hit of every pattern: its order within that
  pattern, strand, zero-based start, length, whether it wraps the origin, and
  the top-strand bases it covers.

The pattern as written is the join key, because three raw patterns - `GAATTC`,
`gaattc`, `GAUTTC` - normalise to one string, and a hits table keyed on the
normalised form could not say which of them produced a row.

The rules are `docs/tasks/milestone-8-view.md` section 4.4 and D052:

* **Containment, not intersection.** A template letter matches a pattern letter
  only when the pattern letter's base set contains *all* of the template
  letter's bases: template `A` matches patterns `A`, `R` and `N` among others,
  and template `N` matches pattern `N` alone. Containment implies intersection,
  never the reverse, and D052 rules intersection out. This script hand-codes
  containment, and the letter table prints both answers side by side, so the
  rejected rule is pinned as a difference rather than asserted as a comment
  (M8-7, D052).
* **Both strands are searched on the top strand.** A minus hit is found by
  searching the pattern's reverse complement and is reported with strand `-` at
  the same top-strand coordinates. Where one span matches on both strands it is
  reported once, as `both` (M8-8, D052).
* **Wrapping is a topology question.** A circular record can be matched at a
  start inside its last `len(pattern) - 1` bases, and such a hit wraps the
  origin. A linear record never wraps, and a pattern longer than the record
  finds nothing on either topology (section 4.4, D049). A FASTA record states no
  topology, so its `topology` column reads `unknown` - what the file says -
  while `wrap_used` records the assumption the view then applies, namely linear.
  Those two columns must not be collapsed into one value.
* **Every match position is reported**, overlaps included: no cap in core
  (M8-16, D050; the cap is presentation, section 5.4 and 5.5).
* **Order** is by start, ascending. One pattern has one length, so at a given
  start there is at most one plus hit and at most one minus hit, and when both
  occur they merge into one `both` hit; section 4.4's "`+` before `-`" therefore
  never has to break a tie for a single pattern. The tables pin that shape, and
  a wrapping hit, having a high start, sorts last.

What Biopython is good for here, and what it is not - measured on Biopython
1.88, `Bio.Data.IUPACData` and `Bio.Restriction`:

* Its letter table agrees with core's `bases_of` and with this script's, letter
  for letter, `N` listed in another order but the same set. So it is an oracle
  for the letter sets, and `search_letters_cross_check` proves that.
* Its `Restriction` search expands the degenerate letters in an **enzyme site**:
  `BstEII` (`GGTNACC`) is found in `GGTCACC`. Over a record whose bases are all
  concrete, that is the same statement as containment, so for the pBR322
  landmarks its site count is an independent count of what luria must find,
  wrapping included (`linear=False`).
* It is **not** an oracle for matching a record's own ambiguity codes, and it is
  not intersection-based in the way one might assume: `BamHI` (`GGATCC`) is not
  found in `GGAYCC` or `GGANCC`, and for an enzyme whose site holds `R` -
  `Dpi3090II` (`AAGRAG`), `SenSARA26III` (`ACRCAG`) - a template `R` at that
  position finds nothing, while `A` and `G` do. `BstEII` (`GGTNACC`) does match a
  template `R`, `Y`, even `X`, because its site `N` compiles to a regex `.`.
  What that leaves is a rule about how a **site** is compiled, not about how a
  **record** letter should be read: at a site `R`, Biopython refuses a record `R`
  that containment accepts, and at a site `N` it accepts a record `X` that no
  base set admits. So the `intersection_match` column describes the rule D052
  rejects, computed from Biopython's letter sets; it is not a measured Biopython
  answer, and the plan's M8-7 note, which attributes intersection behaviour to
  Biopython, is not supported by these measurements. Starts where the record
  itself holds an ambiguity code are compared against nothing, and are pinned by
  the letter grid and by the hand-written `purpose` column.

Run from the repository root:

    python -I tests/fixtures/view/reference/search_dna.py --print
    python -I tests/fixtures/view/reference/search_dna.py --check
"""

import hashlib
import os
import re
import sys
from typing import NamedTuple

from Bio import Restriction, SeqIO
from Bio.Data import IUPACData
from Bio.Seq import Seq

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURE = os.path.dirname(HERE)
GENBANK = os.path.join(os.path.dirname(FIXTURE), "genbank")

RECORDS = [
    ("synthetic", os.path.join(GENBANK, "synthetic.gb")),
    ("synlin", os.path.join(GENBANK, "synthetic-linear.gb")),
    ("frames62", os.path.join(FIXTURE, "frames-62.gb")),
    ("framesamb", os.path.join(FIXTURE, "frames-amb.fa")),
    ("pbr322", os.path.join(GENBANK, "raw", "J01749.1.gb")),
]

# The records whose file states no topology: the tables write `unknown` for them
# and search them under D049's linear assumption.
FASTA_RECORDS = {"framesamb"}

# The IUPAC DNA alphabet, as DNA_ALPHABET in crates/core/src/sequence.rs has it.
LETTERS = "ACGTRYSWKMBDHVN"

# Hand-coded base sets, checked pair by pair against Biopython's table in
# search_letters_cross_check, so the sets are not this script's opinion alone.
# The matching *rule* is this script's own, and that is the part no library
# shares: Biopython's own search does something else again, as the module
# docstring records.
BASES = {
    "A": "A", "C": "C", "G": "G", "T": "T",
    "R": "AG", "Y": "CT", "S": "CG", "W": "AT", "K": "GT", "M": "AC",
    "B": "CGT", "D": "AGT", "H": "ACT", "V": "ACG", "N": "ACGT",
}

COMPLEMENT = {
    "A": "T", "T": "A", "C": "G", "G": "C",
    "R": "Y", "Y": "R", "S": "S", "W": "W", "K": "M", "M": "K",
    "B": "V", "V": "B", "D": "H", "H": "D", "N": "N",
}

# The FASTA record has no accession to identify it, so its canonical text is
# pinned by digest, exactly as reference/frame_tables.py and
# tests/fixtures/view/README.md pin it. A GenBank record is identified by the
# ACCESSION and VERSION in the file itself.
CANONICAL_DIGEST = {
    "framesamb":
        "dad6739778314aed63617363569469465d36df95353a236de7ca535254370469",
}

# The palindromic restriction sites D052 verified on pBR322, with the zero-based
# start the hits table must carry and whether that hit wraps the origin: 4359,
# 375, 29 and 2295 in the 1-based numbering D052 quotes.
LANDMARKS = {
    "GAATTC": (4358, True),
    "GGATCC": (374, False),
    "AAGCTT": (28, False),
    "CATATG": (2294, False),
}

# Constructions whose expected result follows from how they were built. Each one
# must appear in patterns_for, so a dropped pattern fails loudly instead of
# silently un-pinning a rule.
WRAPPING_HITS = {"synthetic": 2, "frames62": 5}
LINEAR_ZERO_WRAP = {"synlin": 4, "framesamb": 4}
TOO_LONG = {"synthetic": "A", "frames62": "G", "framesamb": "N"}
WHOLE_RECORD = {"synthetic", "synlin", "frames62"}
ROTATION_NO_HIT = {"synlin": 1}
# How far each circular record's rotation pattern is shifted, so the expected
# hit start follows from the construction.
CIRCULAR_ROTATION = {"synthetic": 1, "frames62": 5}

LETTER_HEADER = ["pattern_letter", "template_letter", "subset_match",
                 "intersection_match", "rules_agree"]
PATTERN_HEADER = ["record", "seq_len", "topology", "wrap_used", "pattern",
                  "normalized_pattern", "pattern_length", "rc_pattern",
                  "hit_count", "purpose"]
HIT_HEADER = ["record", "pattern", "hit_index", "strand", "start", "length",
              "wraps_origin", "top_strand_bases"]

TABLES = [
    ("search-letters.tsv", os.path.join(FIXTURE, "search-letters.tsv")),
    ("search-dna-patterns.tsv", os.path.join(FIXTURE, "search-dna-patterns.tsv")),
    ("search-dna-hits.tsv", os.path.join(FIXTURE, "search-dna-hits.tsv")),
]


def yes_no(value: bool) -> str:
    return "yes" if value else "no"


class PatternRow(NamedTuple):
    record: str
    seq_len: int
    topology: str
    wrap_used: bool
    pattern: str
    normalized: str
    rc_pattern: str
    purpose: str

    def key(self) -> tuple[str, str]:
        return (self.record, self.pattern)

    def row(self, hit_count: int) -> str:
        return "\t".join([self.record, str(self.seq_len), self.topology,
                          yes_no(self.wrap_used), self.pattern, self.normalized,
                          str(len(self.normalized)), self.rc_pattern,
                          str(hit_count), self.purpose])


class Hit(NamedTuple):
    record: str
    pattern: str
    index: int
    strand: str
    start: int
    length: int
    wraps_origin: bool
    bases: str

    def key(self) -> tuple[str, str]:
        return (self.record, self.pattern)

    def row(self) -> str:
        return "\t".join([self.record, self.pattern, str(self.index), self.strand,
                          str(self.start), str(self.length),
                          yes_no(self.wraps_origin), self.bases])


def normalize(text: str) -> str:
    """Fold a pattern the way section 4.4 says core does: ignore case, read `U`
    as `T`. A letter outside the IUPAC set and an empty pattern are errors that
    no table row can hold, so this script refuses them the same way; contract 8e
    pins those two refusals with unit tests."""
    folded: str = text.upper().replace("U", "T")
    for letter in folded:
        assert letter in BASES, f"pattern letter outside the IUPAC set: {letter!r}"
    assert folded != "", "an empty pattern is an error"
    return folded


def reverse_complement(text: str) -> str:
    return "".join(COMPLEMENT[letter] for letter in reversed(text))


def matches(template: str, pattern: str) -> bool:
    """D052's rule: every base the template letter stands for must lie in the
    pattern letter's set. Not "the sets intersect" - that is the rule D052
    rejects, and it lets a template `N` match a pattern `A`."""
    return all(base in BASES[pattern] for base in BASES[template])


def window(text: str, start: int, length: int, wraps: bool) -> str:
    if not wraps:
        return text[start:start + length]
    size: int = len(text)
    return "".join(text[(start + step) % size] for step in range(length))


def top_strand_hits(text: str, pattern: str, last: int) -> dict[int, bool]:
    """The starts 0 .. `last` where `pattern` matches, each with whether the hit
    runs past the end and has to be read from base 1 again."""
    found: dict[int, bool] = {}
    for start in range(0, last + 1):
        wraps: bool = start + len(pattern) > len(text)
        bases: str = window(text, start, len(pattern), wraps)
        if all(matches(base, letter) for base, letter in zip(bases, pattern)):
            found[start] = wraps
    return found


def find_hits(record: str, raw: str, text: str, wrap_used: bool) -> list[Hit]:
    """Both strands' hits for one pattern, merged by span and ordered by start.

    A linear record offers starts 0 .. len - len(pattern); a circular one offers
    all len starts, and a start past len - len(pattern) wraps. A pattern longer
    than the record has no start at all, on either topology.
    """
    size: int = len(text)
    pattern: str = normalize(raw)
    width: int = len(pattern)
    assert all(letter in BASES for letter in text), (
        f"{record}: a record holds a letter outside the IUPAC alphabet")
    if width > size:
        return []
    last: int = size - 1 if wrap_used else size - width
    plus: dict[int, bool] = top_strand_hits(text, pattern, last)
    minus: dict[int, bool] = top_strand_hits(text, reverse_complement(pattern), last)
    hits: list[Hit] = []
    for index, start in enumerate(sorted(set(plus) | set(minus)), start=1):
        wraps: bool = start + width > size
        if start in plus and start in minus:
            strand: str = "both"
        elif start in plus:
            strand = "+"
        else:
            strand = "-"
        hits.append(Hit(record, raw, index, strand, start, width, wraps,
                        window(text, start, width, wraps)))
    return hits


def patterns_for(name: str, text: str) -> list[tuple[str, str]]:
    """(pattern, what it pins) for one record. Patterns built from the record's
    own text say so, because their expected hit follows from the construction and
    not from how the search works."""
    if name == "pbr322":
        return [
            ("GAATTC", "EcoRI: one hit at base 4359, reachable only by wrapping (D052)"),
            ("gaattc", "the same site with the case folded"),
            ("GAATUC", "the same site with U read as T, folding to GAATTC"),
            ("GGATCC", "BamHI at base 375, exactly one hit (D052)"),
            ("AAGCTT", "HindIII at base 29, exactly one hit (D052)"),
            ("CATATG", "NdeI at base 2295, exactly one hit (D052)"),
            ("GAATTT", "one base off EcoRI: must not match at base 4359"),
            ("AATT", "the inner four of EcoRI: hits the site and other places too"),
            ("RATY", "an ambiguity pattern equal to its own reverse complement: "
                      "the ends pair R with Y and the middle A with T"),
            ("N", "matches every letter a record can hold: one both hit per base"),
        ]
    if name == "synthetic":
        return [
            (text, "as long as the record: one hit at base 1, no wrap"),
            (text[1:] + text[0], "one rotation: a hit at base 2 that wraps"),
            (text[-2:] + text[:2], "built to match only by wrapping"),
            (text + TOO_LONG[name], "longer than the record: no hit at all"),
            (text[20:28], "built from bases 21-28: a plus hit at 21 by construction"),
            (reverse_complement(text[40:48]),
             "built as the reverse complement of bases 41-48: a minus hit at 41"),
            (text[0:12], "the record's first 12 bases, so its reverse complement "
                         "is a minus hit somewhere else"),
        ]
    if name == "synlin":
        return [
            (text, "as long as the record: exactly one hit, at base 1"),
            (text[1:] + text[0], "one rotation: no hit, because linear never wraps"),
            (text[-LINEAR_ZERO_WRAP[name]:] + text[:LINEAR_ZERO_WRAP[name]],
             "built to match only by wrapping: nothing on a linear record"),
            (text[100:112], "built from bases 101-112: a plus hit at 101 by construction"),
            (reverse_complement(text[200:212]),
             "built as the reverse complement of bases 201-212: a minus hit at 201"),
        ]
    if name == "frames62":
        return [
            (text[50:62] + text[0:5], "a 17-mer built across the origin"),
            (text[55:62] + text[0:11],
             "the 18-base join that the fP CDS is annotated as, bases 56-62 and "
             "1-11: a pattern that exists only because the record is circular"),
            (text[-5:] + text[:5],
             "the last five bases then the first five: reachable only by wrapping"),
            (text, "as long as the record: one hit at base 1"),
            (text[5:] + text[0:5], "five rotations: a hit at base 6 that wraps"),
            (text + TOO_LONG[name], "longer than the record: no hit at all"),
            (text[0:31] + "N" + text[32:],
             "a record-wide pattern with N at position 32: legal at all 62 "
             "rotations here, and matching at base 1 alone"),
        ]
    if name == "framesamb":
        return [
            ("AAG", "a concrete 3-mer this record does not contain at all, and "
                    "bases 16-18 are A A N: the zero-hit control"),
            ("AAR", "no hit anywhere: bases 16-18 are A A N, and N is not covered by R"),
            ("AAN", "the only pattern letter there that covers that N"),
            ("AAY", "no hit at 16: an intersection rule would match that N, and must not"),
            ("GTGTSK", "a pattern that says the record's own S and K exactly"),
            ("GTGTCK", "no hit at 28: template S stands for C G, so C alone is not enough"),
            ("GTGTKK", "no hit at 28: S is not covered by K, which stands for G T"),
            ("GTGTNN", "the only way to read across S and K"),
            ("N", "matches every base of this record as well"),
            ("R", "a two-fold pattern, never both: no letter lies inside R and Y"),
            ("S", "a pattern equal to its own reverse complement: both at every hit"),
            ("K", "the K at base 33, and every plain G and T"),
            (text[-4:] + text[:4],
             "the construction that gives a wrapping hit on a circular record: "
             "a FASTA record is searched as linear and has no such start"),
            (text + TOO_LONG[name],
             "one base longer than the record: no hit, and a record-wide pattern "
             "would need a wrap the linear assumption forbids"),
            (text[0:31] + "N" + text[32:],
             "a record-wide pattern with N at position 32: on a linear record "
             "base 1 is the only legal start, so this is one hit"),
        ]
    raise AssertionError("no pattern list for " + name)


def search_letters_rows() -> list[list[str]]:
    """The 15 x 15 grid: luria's containment rule and the intersection rule side
    by side, for every ordered pair of IUPAC letters."""
    rows: list[list[str]] = []
    for pattern in LETTERS:
        for template in LETTERS:
            subset: bool = matches(template, pattern)
            intersection: bool = bool(set(BASES[pattern]) & set(BASES[template]))
            assert not subset or intersection, (
                f"containment must imply intersection: {pattern} / {template}")
            rows.append([pattern, template, yes_no(subset),
                         yes_no(intersection), yes_no(subset == intersection)])
    return rows


def search_letters_cross_check(rows: list[list[str]]) -> tuple[int, int]:
    """Check this script's letter sets against Biopython's, pair by pair, and
    make the boundary of that comparison explicit. Returns (pairs where the
    template letter is concrete, pairs where the two rules disagree).

    The `intersection_match` column is not a Biopython answer: it is what an
    intersection rule returns, computed from Biopython's own sets, and it is the
    rule D052 rejects. `restriction_cross_check` measures what Biopython's own
    search does, and that does something else again."""
    for letter in LETTERS:
        mine: set[str] = set(BASES[letter])
        theirs: set[str] = set(IUPACData.ambiguous_dna_values[letter])
        assert mine == theirs, (
            f"base set for {letter}: this script {''.join(sorted(mine))!r}, "
            f"Biopython {''.join(sorted(theirs))!r}")
    # The complement table is Biopython's too, so the reverse-complement search
    # of section 4.4 is not pinned to this script's arithmetic alone.
    pairs = IUPACData.ambiguous_dna_complement
    for letter in LETTERS:
        assert pairs[letter] == COMPLEMENT[letter], (
            f"complement of {letter}: this script {COMPLEMENT[letter]!r}, "
            f"Biopython {pairs[letter]!r}")
    concrete: int = 0
    disagree: int = 0
    for row in rows:
        pattern: str = row[0]
        template: str = row[1]
        theirs = bool(set(IUPACData.ambiguous_dna_values[pattern])
                      & set(IUPACData.ambiguous_dna_values[template]))
        assert yes_no(theirs) == row[3], (
            f"the intersection of Biopython's own sets for {row}: {theirs}")
        if template in "ACGT":
            # One concrete base: containment and intersection are one statement,
            # so the two rules must answer alike.
            assert row[4] == "yes", f"the rules must agree here: {row}"
            assert row[2] == row[3], f"luria and the intersection rule: {row}"
            concrete += 1
        else:
            # A wider template letter is the only place they can part, and they
            # do not always part: template R against pattern R matches under
            # both. What must hold everywhere is that luria's yes is also an
            # intersection yes.
            if row[2] == "yes":
                assert row[3] == "yes", f"containment implies intersection: {row}"
            if row[4] == "no":
                assert row[2] == "no" and row[3] == "yes", (
                    f"a disagreement must be intersection yes and luria no: {row}")
                disagree += 1
    assert concrete == 4 * len(LETTERS), concrete
    assert disagree > 0, "no pair where the rules differ: the grid is wrong"

    # D052 states its examples sampled, not exhaustively: template `A` is
    # covered by exactly eight pattern letters, and the seven that fail are
    # exactly the letters whose sets leave out A.
    assert matches("A", "A") and matches("A", "R") and matches("A", "N")
    for wrong in ("C", "G", "T", "Y", "S", "K", "B"):
        assert not matches("A", wrong), f"template A against pattern {wrong}"
    assert [letter for letter in LETTERS if matches("A", letter)] == [
        "A", "R", "W", "M", "D", "H", "V", "N"]
    assert matches("N", "N")
    for pattern in LETTERS:
        if pattern != "N":
            assert not matches("N", pattern), (
                f"template N must match pattern {pattern} alone")
    # Template R = {A G} is covered by exactly R, D, V and N.
    assert [letter for letter in LETTERS if matches("R", letter)] == [
        "R", "D", "V", "N"]
    # The alphabet is closed under complement, as sequence.rs says it is, and
    # every letter is the complement-image of its partner.
    flip = {"A": "T", "T": "A", "C": "G", "G": "C"}
    for letter in LETTERS:
        assert COMPLEMENT[COMPLEMENT[letter]] == letter, letter
        assert set(BASES[COMPLEMENT[letter]]) == {flip[base]
                                                 for base in BASES[letter]}, (
            f"{letter} is not the complement-image of {COMPLEMENT[letter]}")
    return concrete, disagree


# The helpers and base objects in Bio.Restriction that are not enzymes and are
# skipped when looking an enzyme up by its recognition site.
NOT_ENZYMES = {"Analyze", "Cadz1", "Cadz2", "Cadz3", "Comm", "CompareTo",
               "Constraint", "Custom", "CustomType", "Cdz", "CdzF", "Compiler",
               "DvH", "Enzymes", "Format", "Greedy", "Mixed", "NewComp",
               "OnlyComp", "OnlyNonComp", "Position", "PrintFormat",
               "Restriction", "Search", "Telomere", "Tth111I", "Variable",
               "Verify", "non_enzymatic", "Test", "TypeClass", "Base"}


def enzyme_names(site: str) -> list[str]:
    """The Biopython enzymes whose recognition site is exactly `site`. EcoRI is
    exposed as a class and R_Ecoli1 as an instance of it; both have the same
    site, and both are counted so the name list is honest about how many
    independent names agree."""
    names: list[str] = []
    for name in dir(Restriction):
        if name in NOT_ENZYMES:
            continue
        if getattr(getattr(Restriction, name, None), "site", None) == site:
            names.append(name)
    return names


def restriction_cross_check(row: PatternRow, text: str, hits: list[Hit]) -> None:
    """Count the sites with Biopython's `Restriction` search, a second and
    unrelated implementation of "where is this site in this record".

    Valid only where the record's letters and the pattern's letters are all
    concrete: measured on Biopython 1.88, `Restriction` expands the degenerate
    letters of its own site (BstEII GGTNACC is found in GGTCACC) but never
    matches a degenerate letter in the record (BamHI GGATCC is not found in
    GGAYCC, and for the site AAGRAG a template R finds nothing). Over a concrete
    record the two rules say the same thing, so the counts must agree.

    Only a palindromic site is comparable: luria merges a span that matches on
    both strands into one `both` hit, which is one site, while a non-palindromic
    pattern's reverse complement is a different site that Biopython counts under
    a different enzyme name. `linear` is the opposite of `wrap_used`, so a
    circular record is searched circularly here too.
    """
    if row.normalized not in LANDMARKS:
        return
    assert set(row.normalized) <= set("ACGT") and set(text) <= set("ACGT"), (
        f"{row.record} {row.pattern}: the landmark oracle needs concrete bases")
    assert reverse_complement(row.normalized) == row.normalized, (
        f"{row.normalized} is not palindromic, so a site count is not comparable")
    names: list[str] = enzyme_names(row.normalized)
    assert names, f"Biopython knows no enzyme with site {row.normalized}"
    counts: dict[str, int] = {
        name: len(getattr(Restriction, name).search(
            Seq(text), linear=not row.wrap_used)) for name in names}
    assert len(set(counts.values())) == 1, (
        f"Biopython's enzymes disagree about {row.normalized}: {counts}")
    found: int = counts[names[0]]
    assert len(hits) == found, (
        f"{row.record} {row.pattern}: this script counts {len(hits)} hits, "
        f"Bio.Restriction counts {found} sites ({', '.join(sorted(names))})")


def find_occurrences(haystack: str, needle: str, last: int) -> set[int]:
    """Every start up to `last` where `needle` appears literally, overlaps
    included. Python's own search, sharing nothing with `matches`."""
    found: set[int] = set()
    position: int = haystack.find(needle)
    while position != -1 and position <= last:
        found.add(position)
        position = haystack.find(needle, position + 1)
    return found


def biopython_matching_measured() -> None:
    """Pin the measurements this file's honesty about Biopython rests on.

    `Bio.Restriction` compiles a regex out of the enzyme's own site and matches
    the record text literally: a site `N` becomes `.`, which is why `BstEII`
    answers for a record reading `GGTXACC` where `X` is no IUPAC letter at all,
    and a site `R` becomes `[AG]`, which is why `Dpi3090II` answers nothing for a
    record `R` at that position while luria's containment rule accepts it. Neither
    answer is intersection-based. If a Biopython upgrade changed one of these,
    the sentence above and the one in README.md would stop describing the
    installed library, so they are asserted rather than cited.
    """

    def answers(enzyme: object, texts: list[str]) -> dict[str, bool]:
        return {text: len(enzyme.search(Seq(text), linear=True)) > 0
                for text in texts}

    concrete: dict[str, bool] = answers(Restriction.BamHI,
                                        ["GGATCC", "GGAYCC", "GGANCC"])
    assert concrete == {"GGATCC": True, "GGAYCC": False, "GGANCC": False}, (
        f"BamHI over an ambiguous record now answers {concrete}")
    degenerate_r: dict[str, bool] = answers(Restriction.Dpi3090II,
                                            ["AAGAAG", "AAGGAG", "AAGRAG"])
    assert degenerate_r == {"AAGAAG": True, "AAGGAG": True, "AAGRAG": False}, (
        f"Dpi3090II at its own R now answers {degenerate_r}")
    degenerate_n: dict[str, bool] = answers(Restriction.BstEII,
                                            ["GGTCACC", "GGTRACC", "GGTXACC"])
    assert degenerate_n == {"GGTCACC": True, "GGTRACC": True, "GGTXACC": True}, (
        f"BstEII at its own N now answers {degenerate_n}")


def cross_check(row: PatternRow, text: str, hits: list[Hit]) -> int:
    """Re-find the hits without using this script's matcher. Two oracles, each
    honest about its own scope; returns the number of starts neither can speak
    about because the record holds an ambiguity code there.

    * A pattern of plain `A C G T` letters is looked for with Python's own
      substring search, over the record doubled by `len(pattern) - 1` bases when
      the record may wrap. That pins the start range and the wrapping rule
      independently: a hit that exists only by wrapping, like pBR322's EcoRI
      site, has to come out of the doubled text. A window holding an ambiguity
      code cannot match such a pattern under either rule, so every start of a
      concrete pattern is comparable.
    * A pattern carrying ambiguity letters is tested window by window with a
      regular expression whose character classes come from Biopython's
      `ambiguous_dna_values`. That compares the *pattern's* handling of its own
      ambiguity codes against Biopython's letter table, and it can only speak
      where the record's letters are concrete - which is the boundary the letter
      grid proves, since a wider template letter is where the rules part.
    """
    size: int = len(text)
    pattern: str = row.normalized
    width: int = len(pattern)
    if width > size:
        assert hits == [], f"{row.record}: {pattern} is longer than the record"
        return 0
    last: int = size - 1 if row.wrap_used else size - width
    mine: dict[str, set[int]] = {
        label: {hit.start for hit in hits if hit.strand in (label, "both")}
        for label in ("+", "-")}
    concrete_starts: list[int] = [
        start for start in range(0, last + 1)
        if set(window(text, start, width, start + width > size)) <= set("ACGT")]
    excluded: int = last + 1 - len(concrete_starts)
    for probe, label in ((pattern, "+"), (reverse_complement(pattern), "-")):
        found: set[int]
        if set(probe) <= set("ACGT"):
            haystack: str = text if not row.wrap_used else text + text[:width - 1]
            found = find_occurrences(haystack, probe, last)
            assert mine[label] == found, (
                f"{row.record} {label} {pattern}: this script "
                f"{sorted(mine[label])}, substring search {sorted(found)}")
            continue
        expression: str = "".join(
            "[" + IUPACData.ambiguous_dna_values[letter] + "]" for letter in probe)
        found = {start for start in concrete_starts
                 if re.fullmatch(expression,
                                 window(text, start, width,
                                       start + width > size))}
        here: set[int] = mine[label] & set(concrete_starts)
        assert here == found, (
            f"{row.record} {label} {pattern}: this script {sorted(here)}, "
            f"Biopython's letter table {sorted(found)} over "
            f"{len(concrete_starts)} concrete starts, {excluded} not comparable")
    return excluded


def validate(records: dict[str, str], rows: list[PatternRow], hits: list[Hit],
             excluded: dict[tuple[str, str], int]) -> None:
    by_pattern: dict[tuple[str, str], list[Hit]] = {}
    for hit in hits:
        by_pattern.setdefault(hit.key(), []).append(hit)
    listed: set[tuple[str, str]] = {row.key() for row in rows}
    assert len(listed) == len(rows), "a raw pattern repeats within a record"
    counts: dict[tuple[str, str], int] = {key: len(group)
                                          for key, group in by_pattern.items()}

    for row in rows:
        assert row.purpose != "", row
        assert row.normalized == normalize(row.pattern), row
        assert len(row.normalized) == len(row.pattern), row
        assert row.rc_pattern == reverse_complement(row.normalized), row
        assert row.topology in ("circular", "linear", "unknown"), row
        assert row.wrap_used == (row.topology == "circular"), (
            f"wrap_used must be the assumption the topology forces: {row}")
        assert (row.topology == "unknown") == (row.record in FASTA_RECORDS), row
        assert row.seq_len == len(records[row.record]), row
        # A pattern with no hits has no row in the hits table at all, and that
        # absence is pinned here rather than left to the reader's eye.
        assert counts.get(row.key(), 0) == len(
            [hit for hit in by_pattern.get(row.key(), [])
             if hit.length == len(row.normalized)]), row

    for hit in hits:
        text: str = records[hit.record]
        assert 0 <= hit.start < len(text), hit
        assert len(hit.bases) == hit.length, hit
        assert hit.wraps_origin == (hit.start + hit.length > len(text)), hit
        assert hit.bases == window(text, hit.start, hit.length,
                                   hit.wraps_origin), hit
        assert hit.strand in ("+", "-", "both"), hit
        assert hit.index >= 1, hit
    for key, group in by_pattern.items():
        assert [hit.index for hit in group] == list(range(1, len(group) + 1)), key
        starts: list[int] = [hit.start for hit in group]
        assert starts == sorted(starts), f"hits must be ordered by start: {key}"
        assert len(set(starts)) == len(starts), (
            f"one pattern gives at most one hit per start: {key}")

    # pBR322's landmarks: D052's counts and 1-based positions, and the wrapping
    # EcoRI site that exists only because the record is circular.
    for pattern, (start, wraps) in LANDMARKS.items():
        group: list[Hit] = by_pattern[("pbr322", pattern)]
        assert len(group) == 1, f"pbr322 {pattern}: {len(group)} hits, expected 1"
        assert group[0].start == start and group[0].strand == "both", group[0]
        assert group[0].wraps_origin == wraps, group[0]
    for raw in ("gaattc", "GAATUC"):
        group = by_pattern[("pbr322", raw)]
        assert len(group) == 1 and group[0].start == 4358, group
        assert group[0].bases == "GAATTC", (
            f"a hit covers the folded bases, not the letters as written: {group[0]}")
    near: list[Hit] = by_pattern.get(("pbr322", "GAATTT"), [])
    assert all(hit.start != 4358 for hit in near), (
        f"GAATTT must not match the EcoRI site: {near}")
    aatt: list[Hit] = by_pattern[("pbr322", "AATT")]
    # The inner four bases of the EcoRI site start one base into it, at 4360
    # 1-based, and run two bases past the end, so this is the wrapping case.
    assert any(hit.start == 4359 and hit.wraps_origin for hit in aatt), (
        f"AATT must still hit the wrapping EcoRI site: {aatt}")
    assert len(aatt) > 1, f"AATT should hit elsewhere in 4361 bases too: {len(aatt)}"
    raty: list[Hit] = by_pattern[("pbr322", "RATY")]
    assert reverse_complement("RATY") == "RATY", (
        "RATY must be its own reverse complement for the merge claim below")
    assert reverse_complement("RAYT") != "RAYT", (
        "RAYT is the same four letters in the other order and is NOT self-"
        "complementary: the claim is about order, not about the letter set")
    assert raty, "RATY found nothing on pBR322"
    assert all(hit.strand == "both" for hit in raty), (
        "a pattern equal to its own reverse complement gives one hit per span")

    # Constructions, each pinned by how it was built rather than by the search.
    for record, overlap in WRAPPING_HITS.items():
        raw: str = records[record][-overlap:] + records[record][:overlap]
        assert (record, raw) in listed, (record, "wrapping", raw)
        found = by_pattern[(record, raw)]
        assert any(hit.wraps_origin for hit in found), (
            f"{record}: {raw!r} found no wrapping hit: {found}")
        assert any(hit.start == len(records[record]) - overlap and
                   hit.wraps_origin and hit.bases == raw for hit in found), (
            f"{record}: the instance built across the origin must be reported "
            f"at base {len(records[record]) - overlap + 1}: {found}")
    for record, overlap in LINEAR_ZERO_WRAP.items():
        text_local: str = records[record]
        raw = text_local[-overlap:] + text_local[:overlap]
        assert (record, raw) in listed, (record, "linear wrap", raw)
        found = by_pattern.get((record, raw), [])
        assert all(not hit.wraps_origin for hit in found), (
            f"{record}: no linear hit may wrap the origin: {found}")
        # The pattern really does occur in the record - one base further along,
        # where reading it would need to come back to base 1. Without this the
        # empty result would be vacuous: the pattern might simply not occur.
        width = len(raw)
        neighbour: int = len(text_local) - overlap
        assert all(matches(base, letter) for base, letter in
                   zip(window(text_local, neighbour, width, True), raw)), (
            f"{record}: {raw!r} must match at base {neighbour + 1} if a wrap "
            "were allowed - that is the hit the circular analogue gets - so an "
            "empty result pins the start range and not the pattern's absence")
        assert not any(hit.start >= neighbour for hit in found), (
            f"{record}: a linear record admits no start past base "
            f"{len(text_local) - width + 1}")
    for record, extra in TOO_LONG.items():
        raw = records[record] + extra
        assert (record, raw) in listed, (record, "too long", raw)
        assert not by_pattern.get((record, raw), []), (
            f"{record}: a pattern longer than the record finds nothing")
    for record in WHOLE_RECORD:
        raw = records[record]
        assert (record, raw) in listed, (record, "whole record")
        found = by_pattern[(record, raw)]
        assert found and found[0].start == 0 and not found[0].wraps_origin, found
        if record in ROTATION_NO_HIT:
            shift = ROTATION_NO_HIT[record]
            rotated: str = raw[shift:] + raw[:shift]
            assert (record, rotated) in listed, (record, "rotation", rotated)
            assert raw != rotated, f"{record} is its own rotation"
            assert raw != reverse_complement(raw), (
                f"{record} is its own reverse complement, so the minus strand "
                "would give this pattern a hit and the zero expectation is wrong")
            assert not by_pattern.get((record, rotated), []), (
                f"{record}: a rotation of a linear record finds nothing, because "
                "the only legal start for a pattern as long as the record is "
                "base 1 and the letters there are the record's own")
        else:
            shift = CIRCULAR_ROTATION[record]
            rotated = raw[shift:] + raw[:shift]
            assert (record, rotated) in listed, (record, "rotation", rotated)
            assert any(hit.start == shift and hit.wraps_origin and
                       hit.bases == rotated for hit in
                       by_pattern[(record, rotated)]), (
                f"{record}: a rotation by {shift} must hit at base {shift + 1}, "
                "wrapping")

    # Ambiguity in the record itself, where containment and intersection part.
    amb: str = records["framesamb"]
    assert amb[15:18] == "AAN" and amb[27:33] == "GTGTSK", (
        f"frames-amb.fa moved, and these rows depend on its letters: {amb!r}")
    assert not by_pattern.get(("framesamb", "AAR"), []), (
        "AAR must find nothing on framesamb")
    for raw in ("AAY", "GTGTCK", "GTGTKK"):
        found = by_pattern.get(("framesamb", raw), [])
        assert all(hit.start not in (15, 27) for hit in found), (
            f"framesamb {raw} must not match across the ambiguity there: {found}")
    assert excluded.get(("framesamb", "AAR"), 0) > 0, (
        "AAR on framesamb must have had starts no oracle could speak about")
    assert any(hit.start == 15 for hit in by_pattern[("framesamb", "AAN")]), (
        "AAN must hit the A A N at bases 16-18")
    assert any(hit.start == 27 and hit.strand == "+"
               for hit in by_pattern[("framesamb", "GTGTSK")]), (
        "GTGTSK must hit bases 28-33 on the plus strand")
    assert any(hit.start == 27 and hit.strand == "+"
               for hit in by_pattern[("framesamb", "GTGTNN")]), (
        "GTGTNN must read across S and K at bases 28-33")
    record_wide: list[Hit] = by_pattern[("framesamb", amb[0:31] + "N" + amb[32:])]
    assert len(record_wide) == 1, (
        f"base 1 is the only legal start for a pattern as long as a linear "
        f"record, so one span gives one row: {record_wide}")
    assert record_wide[0].start == 0 and not record_wide[0].wraps_origin, record_wide
    assert record_wide[0].length == len(amb), record_wide
    # The same construction on the 62-base circular record is legal at all 62
    # rotations, and matches at only one of them: a fact about this record, not
    # about the search, which is why the row is pinned by count.
    frames62_text: str = records["frames62"]
    # The fP join pattern must be the annotated span itself: bases 56-62 then
    # 1-11, which is 18 letters, not 17. A one-base slip in the slice would make
    # the purpose column a false statement about the fixture.
    fp_join: str = frames62_text[55:62] + frames62_text[0:11]
    assert len(fp_join) == 18, len(fp_join)
    fp_rows: list[Hit] = by_pattern[("frames62", fp_join)]
    assert fp_rows and all(hit.length == 18 for hit in fp_rows), fp_rows
    assert any(hit.bases == fp_join and hit.wraps_origin for hit in fp_rows), (
        f"the annotated join must be reported as a wrapping hit: {fp_rows}")
    rotated_wide: list[Hit] = by_pattern[
        ("frames62", frames62_text[0:31] + "N" + frames62_text[32:])]
    assert len(rotated_wide) == 1 and rotated_wide[0].start == 0, (
        f"frames62 must match its own record-wide pattern with N at 32 at base 1 "
        f"only, out of all {len(frames62_text)} rotations: {rotated_wide}")

    # `N` covers every letter a record can hold, and it is its own reverse
    # complement, so every hit is one single-base `both` hit.
    for row in rows:
        if row.normalized != "N":
            continue
        text = records[row.record]
        found = by_pattern[row.key()]
        assert len(found) == len(text), (
            f"{row.record}: N gave {len(found)} hits over {len(text)} bases")
        assert all(hit.strand == "both" and hit.length == 1
                   and hit.bases == text[hit.start] for hit in found), (
            f"{row.record}: every N hit is one base, reported once, both strands")

    # `S` equals its own reverse complement while `R` does not, and no IUPAC
    # letter's set lies inside both R and Y, so S is always `both` and R never
    # is (M8-8, D052). RATY is self-complementary as a word: its letters pair
    # with their complements from the far end, so its hits merge to `both` too.
    for hit in by_pattern[("framesamb", "S")]:
        assert hit.strand == "both", hit
    for hit in by_pattern[("framesamb", "R")]:
        assert hit.strand in ("+", "-"), hit
    assert by_pattern[("framesamb", "R")], "R found nothing"
    assert by_pattern[("framesamb", "K")][-1].start == len(amb) - 1, (
        "the K at the last base must be K's last hit on this linear record")


def build() -> tuple[list[list[str]], list[PatternRow], list[Hit],
                     dict[tuple[str, str], int]]:
    biopython_matching_measured()
    records: dict[str, str] = {}
    rows: list[PatternRow] = []
    hits: list[Hit] = []
    excluded: dict[tuple[str, str], int] = {}
    for name, path in RECORDS:
        parser: str = "genbank" if path.endswith(".gb") else "fasta"
        parsed = list(SeqIO.parse(path, parser))
        assert len(parsed) == 1, f"{path}: {len(parsed)} records, expected 1"
        text: str = str(parsed[0].seq).upper()
        records[name] = text
        if name in CANONICAL_DIGEST:
            digest: str = hashlib.sha256(text.encode()).hexdigest()
            assert digest == CANONICAL_DIGEST[name], (
                f"{path}: canonical text hashes to {digest}, "
                f"README pins {CANONICAL_DIGEST[name]}")
        stated: str = ("unknown" if name in FASTA_RECORDS
                       else str(parsed[0].annotations.get("topology", "")).lower())
        assert stated in ("circular", "linear", "unknown"), f"{name}: {stated}"
        for raw, purpose in patterns_for(name, text):
            normalized: str = normalize(raw)
            row = PatternRow(name, len(text), stated, stated == "circular", raw,
                             normalized, reverse_complement(normalized), purpose)
            found = find_hits(name, raw, text, row.wrap_used)
            excluded[(name, raw)] = cross_check(row, text, found)
            restriction_cross_check(row, text, found)
            rows.append(row)
            hits.extend(found)
    letters: list[list[str]] = search_letters_rows()
    concrete, disagree = search_letters_cross_check(letters)
    validate(records, rows, hits, excluded)
    build.stats = (len(letters), concrete, disagree, len(rows), len(hits),
                   sum(excluded.values()))
    return letters, rows, hits, excluded


def summary() -> str:
    letters, concrete, disagree, patterns, hits, not_comparable = build.stats
    return (f"{letters} letter pairs ({concrete} where the template letter is "
            f"concrete, {disagree} where the two rules disagree), {patterns} "
            f"patterns, {hits} hits, {not_comparable} starts where the record "
            f"holds an ambiguity code and no oracle speaks")


def check_or_write(argv: list[str]) -> int:
    letters, patterns, hits, _ = build()
    texts = [
        table_text(LETTER_HEADER, ["\t".join(row) for row in letters]),
        table_text(PATTERN_HEADER,
                   [row.row(sum(1 for hit in hits if hit.key() == row.key()))
                    for row in patterns]),
        table_text(HIT_HEADER, [hit.row() for hit in hits]),
    ]
    if "--print" in argv:
        print(texts[1])
        print()
        for row in patterns:
            shown = [hit.row() for hit in hits if hit.key() == row.key()][:4]
            print(f"# {row.record} {row.pattern} - {row.purpose}")
            print("\n".join(shown))
        print()
        print("# the letter grid, for four template letters")
        print(texts[0].split("\n")[0])
        for row in letters:
            if row[1] in ("A", "R", "S", "N"):
                print("\t".join(row))
        print()
        print(summary())
        return 0
    if "--check" in argv:
        bad: bool = False
        for (name, path), text in zip(TABLES, texts):
            if not os.path.isfile(path):
                print(f"MISSING {name}")
                bad = True
                continue
            with open(path, encoding="utf-8") as handle:
                tracked: str = handle.read()
            if tracked == text:
                print(f"OK {name} {os.path.getsize(path)} bytes")
                continue
            bad = True
            print(f"DIFFERS {name}")
            old_lines: list[str] = tracked.split("\n")
            new_lines: list[str] = text.split("\n")
            if len(old_lines) == len(new_lines):
                for number, (left, right) in enumerate(zip(old_lines, new_lines), 1):
                    if left != right:
                        print(f"  line {number}:\n    tracked    {left}\n"
                              f"    re-derived {right}")
                        break
            else:
                print(f"  {len(old_lines)} lines tracked, {len(new_lines)} re-derived")
        print(summary())
        return 1 if bad else 0
    for (name, path), text in zip(TABLES, texts):
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
        print(f"wrote {name} {os.path.getsize(path)} bytes")
    print(summary())
    return 0


def table_text(header: list[str], body: list[str]) -> str:
    return "\t".join(header) + "\n" + "\n".join(body) + "\n"


if __name__ == "__main__":
    raise SystemExit(check_or_write(sys.argv[1:]))
