#!/usr/bin/env python3
"""Expected readings of `luria new`'s mutation shorthand (plan 7k §3, M7-35).

Writes ../shorthand.tsv: for each input line, what luria must read it as,
or the code of the refusal. luria's `shorthand::parse_line` (contract 7k1)
must give exactly these. Independent of crates/: this is its own parser,
written from the plan's table, not from luria's code.

Positions on input are 1-based inclusive template bases (M7-24); the
reading is zero-based half-open, as a request holds them (D006):

* `N X>Y`             one base N        -> substitution [N-1, N) X -> Y
* `A-B X>Y`           bases A..B        -> substitution [A-1, B) when X and Y
                                            have the same length, else
                                            replacement [A-1, B)
* `del A-B`, `del N`  deletion          -> deletion [A-1, B)
* `ins S after N`     insertion         -> insertion at point N (between
                                            base N and base N+1); `after 0`
                                            means at the start
* `D404N`, `p.D404N`, `Asp404Asn`       -> residue 404 D -> N, codon by the
                                            form's choice (`policy`)
* `D404N:AAC`                           -> residue, codon AAC

Bases are A, C, G, T in either case, written upper case in the reading;
X must have exactly B-A+1 bases. Words and separators are matched
case-insensitively; runs of spaces count as one; leading and trailing
spaces are ignored. Refusal codes:

* `empty`            nothing but spaces
* `unreadable`       no form above matches
* `zero_position`    a position of 0 (except `after 0`), or a leading zero
* `reversed_range`   A > B
* `length_mismatch`  X's length is not the range's
* `not_dna`          a base other than A, C, G, T
* `same_bases`       X equals Y
* `bad_codon`        a codon that is not three of A, C, G, T
* `stop`             a residue edit naming a stop (`*`, `X`, `Ter`) as either
                     residue
* `unstated`         a residue edit with `?` for a part

A residue line that is neither of these and that 6's notation
(`notation.rs`) does not read, such as mixed codes or a number of 0 or
with a leading zero, is `unreadable`. A range may use `-` or an en dash
`–` (U+2013), as the screens write ranges.

Columns: line (as typed, with `\\t` never present), kind, start, end, from,
to, codon. Unused columns are `-`. For a refusal, kind is `refused` and
`from` holds the code. A residue's start is its number; end is `-`.

Run from the repository root:

    python3 -I tests/fixtures/new/reference/shorthand.py           rewrite
    python3 -I tests/fixtures/new/reference/shorthand.py --check   compare
"""
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.dirname(HERE)
NAME = "shorthand.tsv"

ONE = "ACDEFGHIKLMNPQRSTVWY"
THREE = {
    "ala": "A", "arg": "R", "asn": "N", "asp": "D", "cys": "C", "gln": "Q",
    "glu": "E", "gly": "G", "his": "H", "ile": "I", "leu": "L", "lys": "K",
    "met": "M", "phe": "F", "pro": "P", "ser": "S", "thr": "T", "trp": "W",
    "tyr": "Y", "val": "V",
}
STOPS = {"*", "x", "ter"}

LINES = [
    # residues
    "D404N", "d404n", "p.D404N", "Asp404Asn", "asp404asn", "D404N:AAC",
    "D404N:aac", "A42G", "D404*", "D404X", "Asp404Ter", "D404?", "?404N",
    "D0N", "D040N", "D404N:AAX", "D404N:AA", "Ala42G",
    # one base
    "1210 G>A", "1210 g>a", "  1210   G > A  ", "1 A>T", "0 G>A", "01210 G>A",
    "1210 G>G", "1210 GA>T", "1210 N>A",
    # ranges
    "1210-1212 GAT>AAC", "1210-1212 GAT>TT", "1210-1212 GAT>AACAAC",
    "1210–1212 GAT>AAC", "1212-1210 GAT>AAC", "1210-1212 GA>AAC",
    "1210-1210 G>A", "del 1210–1212",
    # deletions
    "del 1210-1212", "DEL 1210-1212", "del 46", "del 0", "del 1212-1210",
    "delete 1210-1212",
    # insertions
    "ins GAT after 1209", "INS gat AFTER 1209", "ins GAT after 0",
    "ins GATTACA after 50", "ins GNT after 1209", "ins after 1209",
    "ins GAT before 1209",
    # neither
    "", "   ", "hello", "c.1210G>A", "D404N A42G",
]


def bases(text):
    return text.upper()


def is_dna(text):
    return text != "" and all(c in "ACGTacgt" for c in text)


def position(text, allow_zero=False):
    """A decimal position: (value, None) or (None, refusal)."""
    if not text.isdigit():
        return None, "unreadable"
    if len(text) > 1 and text[0] == "0":
        return None, "zero_position"
    value = int(text)
    if value == 0 and not allow_zero:
        return None, "zero_position"
    return value, None


def residue_letter(text):
    lowered = text.lower()
    if lowered in STOPS:
        return None, "stop"
    if text == "?":
        return None, "unstated"
    if len(text) == 1 and text.upper() in ONE:
        return text.upper(), None
    if len(text) == 3 and lowered in THREE:
        return THREE[lowered], None
    return None, "unreadable"


def refused(code):
    return ("refused", "-", "-", code, "-", "-")


def read(line):
    text = " ".join(line.split()).replace("\u2013", "-")
    if text == "":
        return refused("empty")

    # residue: optional p., letter(s), number, letter(s), optional :codon
    m = re.fullmatch(r"(?:p\.)?([A-Za-z]{1,3}|\*|\?)(\d+|\?)([A-Za-z]{1,3}|\*|\?)(?::(\S+))?", text)
    if m and not text.lower().startswith(("del", "ins")):
        original, number, target, codon = m.groups()
        # a stop first, then an unstated part, then 6's notation
        for part in (original, target):
            if part.lower() in STOPS:
                return refused("stop")
        if "?" in (original, number, target):
            return refused("unstated")
        if (len(original) == 3) != (len(target) == 3):
            return refused("unreadable")
        a, problem = residue_letter(original)
        if problem:
            return refused("unreadable")
        b, problem = residue_letter(target)
        if problem:
            return refused("unreadable")
        n, problem = position(number)
        if problem:
            return refused("unreadable")
        if codon is not None:
            if len(codon) != 3 or not is_dna(codon):
                return refused("bad_codon")
            return ("residue", str(n), "-", a, b, codon.upper())
        return ("residue", str(n), "-", a, b, "policy")

    words = text.split(" ")
    lowered = [w.lower() for w in words]

    # deletion
    if lowered[0] == "del" and len(words) == 2:
        span = words[1]
        if "-" in span:
            a_text, b_text = span.split("-", 1)
        else:
            a_text, b_text = span, span
        a, problem = position(a_text)
        if problem:
            return refused(problem)
        b, problem = position(b_text)
        if problem:
            return refused(problem)
        if a > b:
            return refused("reversed_range")
        return ("deletion", str(a - 1), str(b), "-", "-", "-")

    # insertion
    if lowered[0] == "ins" and len(words) == 4 and lowered[2] == "after":
        insert = words[1]
        if not all(c.isalpha() for c in insert):
            return refused("unreadable")
        if not is_dna(insert):
            return refused("not_dna")
        p, problem = position(words[3], allow_zero=True)
        if problem:
            return refused(problem)
        return ("insertion", str(p), str(p), "-", bases(insert), "-")

    # substitution or replacement: "<pos or range> X>Y", spaces around > allowed
    m = re.fullmatch(r"(\S+) (\S+) ?> ?(\S+)", text)
    if m:
        where, x, y = m.groups()
        if "-" in where:
            a_text, b_text = where.split("-", 1)
        else:
            a_text, b_text = where, where
        a, problem = position(a_text)
        if problem:
            return refused(problem)
        b, problem = position(b_text)
        if problem:
            return refused(problem)
        if a > b:
            return refused("reversed_range")
        if not (x.isalpha() and y.isalpha()):
            return refused("unreadable")
        if not is_dna(x) or not is_dna(y):
            return refused("not_dna")
        if len(x) != b - a + 1:
            return refused("length_mismatch")
        if bases(x) == bases(y):
            return refused("same_bases")
        kind = "substitution" if len(x) == len(y) else "replacement"
        return (kind, str(a - 1), str(b), bases(x), bases(y), "-")

    return refused("unreadable")


def main():
    lines = ["\t".join(["line", "kind", "start", "end", "from", "to", "codon"])]
    for line in LINES:
        assert "\t" not in line
        lines.append("\t".join((line,) + read(line)))
    text = "".join(line + "\n" for line in lines)
    path = os.path.join(OUT, NAME)
    if "--check" not in sys.argv[1:]:
        with open(path, "w", encoding="utf-8") as f:
            f.write(text)
        print("wrote", NAME)
        return
    try:
        with open(path, encoding="utf-8") as f:
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
