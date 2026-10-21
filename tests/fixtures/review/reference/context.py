#!/usr/bin/env python3
"""Expected sequence-context screens for `luria review` (milestone 7d0).

The sequence-context screen (docs/tasks/milestone-7-tui.md §5.3, contract 7d)
lays a designed primer pair out against the template. This script writes the
expected screen text for three designs whose geometry is already fixed by
independently verified tables, using only those tables and the template:

    context-native-01.txt  ../q5-primer/native.tsv row native-01
                           (replacement 1209:1212 GAT->AAC, bound 15)
    context-del-11.txt     ../q5-primer/native-deletion.tsv row del-11
    context-ins-14.txt     ../q5-primer/native-insertion.tsv row ins-14

The template is ../example-construct/input/sid-1_cDNA.fa. Nothing here reads
luria's output or shares code with crates/: the primers come from the tables'
spans, sequences and tails, and the bottom strand is complemented here.

Run from the repository root:

    python3 -I tests/fixtures/review/reference/context.py           rewrite
    python3 -I tests/fixtures/review/reference/context.py --check   compare

With --check each file's text is built in memory and compared byte for byte
with the tracked file; `OK <name>` or `DIFFERS <name>` and the first differing
line are printed, a difference exits 1, and nothing is written (D039).

The layout (contract 7d):

    sequence context: <member>, [<lo>, <hi>) of <template id>
    top strand 5'->3'; forward 5'->3' above it, reverse 3'->5' below it
    upper case anneals, lower case does not; ^ base edited or removed, + insertion point

    bases [<s>, <e>)
    template  <top strand, 60 bases a block>
    forward   <forward primer, each base in its template column>
    reverse   <reverse primer, written 3'->5', each base in its column>
    edit      <^ under each base of the edit span; + at an insertion point>

Blocks follow one another with a blank line between them. Every row is cut of
its trailing spaces. The window [lo, hi) runs from 5 bases before the leftmost
primer base to 5 after the rightmost, tails included, within the template.

A primer part with a template span and as many bases as the span is wide is
written in the span's columns: as it is for the forward primer, reversed for
the reverse primer (so each base sits over the template base it pairs with).
Annealing parts are upper case, parts that do not anneal (the written
mutation, a tail) lower case. A tail has no span: the forward primer's tail
sits in the columns just left of its first spanned part, the reverse primer's,
reversed, just right of its last. (A part whose base count differs from its
span's width, as in an unequal replacement, is written as `*` in its span and
named in a line after the blocks: `<role> <part role>: <sequence> at [s, e)`;
none of the three designs here has one.)
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
FIXTURES = os.path.dirname(os.path.dirname(HERE))
OUT = os.path.dirname(HERE)

WIDTH = 60
LABEL = 10
COMPLEMENT = {"A": "T", "T": "A", "G": "C", "C": "G"}


def load_template():
    path = os.path.join(FIXTURES, "example-construct", "input", "sid-1_cDNA.fa")
    with open(path, encoding="ascii") as f:
        lines = f.read().splitlines()
    assert lines[0].startswith(">")
    return "".join(line.strip() for line in lines[1:]).upper()


def row_of(table, case_id):
    with open(os.path.join(FIXTURES, "q5-primer", table), encoding="utf-8") as f:
        lines = f.read().splitlines()
    header = lines[0].split("\t")
    for line in lines[1:]:
        fields = dict(zip(header, line.split("\t")))
        if fields["case_id"] == case_id:
            return fields
    raise SystemExit("no row %s in %s" % (case_id, table))


def span(text):
    start, end = text.split(":")
    return int(start), int(end)


def revcomp(bases):
    return "".join(COMPLEMENT[b] for b in reversed(bases))


# Each design: (member name, template id, edit span, insertion point or None,
# forward parts, reverse parts). A part is (role, anneals, sequence, span or
# None), in primer order 5'->3', as D029/D031/D032 build them.


def native_01(t):
    r = row_of("native.tsv", "native-01")
    s, e, original, written = r["edit"].split(":")
    s, e = int(s), int(e)
    assert t[s:e] == original
    f0, f1 = span(r["forward_span"])
    r0, r1 = span(r["reverse_span"])
    left, right = (int(x) for x in r["flanks"].split(":"))
    assert f0 == s - left and f1 == e + right and r1 == f0
    assert r1 - r0 == int(r["reverse_length"])
    forward = [
        ("context", True, t[f0:s], (f0, s)),
        ("mutation", False, written, (s, e)),
        ("context", True, t[e:f1], (e, f1)),
    ]
    reverse = [("context", True, revcomp(t[r0:r1]), (r0, r1))]
    return "native-01", (s, e), None, forward, reverse


def del_11(t):
    r = row_of("native-deletion.tsv", "del-11")
    s, e, original = r["deletion"].split(":")
    s, e = int(s), int(e)
    assert t[s:e] == original
    f0, f1 = span(r["forward_span"])
    r0, r1 = span(r["reverse_span"])
    assert t[f0:f1] == r["forward"]
    assert revcomp(t[r0:r1]) == r["reverse"]
    forward = [("context", True, r["forward"], (f0, f1))]
    reverse = [("context", True, r["reverse"], (r0, r1))]
    return "del-11", (s, e), None, forward, reverse


def ins_14(t):
    r = row_of("native-insertion.tsv", "ins-14")
    point = int(r["point"])
    f0, f1 = span(r["forward_anneal_span"])
    r0, r1 = span(r["reverse_anneal_span"])
    assert f0 == point and r1 == point
    forward = []
    tail = r["forward_tail"]
    if tail != "none":
        forward.append(("tail", False, tail, None))
    forward.append(("context", True, t[f0:f1], (f0, f1)))
    assert "".join(p[2] for p in forward) == r["forward"]
    reverse = []
    tail = r["reverse_tail"]
    if tail != "none":
        reverse.append(("tail", False, tail, None))
    reverse.append(("context", True, revcomp(t[r0:r1]), (r0, r1)))
    assert "".join(p[2] for p in reverse) == r["reverse"]
    return "ins-14", (point, point), point, forward, reverse


def place(parts, reverse, columns, notes, role):
    """Write one primer's parts into `columns` (template position -> char)."""
    spanned = [p for p in parts if p[3] is not None]
    first = min(p[3][0] for p in spanned)
    last = max(p[3][1] for p in spanned)
    for part_role, anneals, sequence, where in parts:
        shown = sequence if anneals else sequence.lower()
        if where is None:
            if reverse:
                # Written 3'->5' left to right: the tail's 3' end joins the
                # annealing part, so the reversed tail starts at `last`.
                for i, base in enumerate(shown[::-1]):
                    columns[last + i] = base
            else:
                for i, base in enumerate(shown):
                    columns[first - len(shown) + i] = base
            continue
        s, e = where
        if len(sequence) != e - s:
            for k in range(s, e):
                columns[k] = "*"
            notes.append("%s %s: %s at [%d, %d)" % (role, part_role, sequence, s, e))
            continue
        written = shown[::-1] if reverse else shown
        for i, base in enumerate(written):
            columns[s + i] = base


def screen(t, design):
    name, (es, ee), point, forward, reverse = design
    top = {}
    bottom = {}
    notes = []
    place(forward, False, top, notes, "forward")
    place(reverse, True, bottom, notes, "reverse")
    used = list(top) + list(bottom)
    lo = max(0, min(used) - 5)
    hi = min(len(t), max(used) + 1 + 5)
    edit = {}
    for k in range(es, ee):
        edit[k] = "^"
    if point is not None:
        edit[point] = "+"
    lines = [
        "sequence context: %s, [%d, %d) of %s" % (name, lo, hi, "sid-1"),
        "top strand 5'->3'; forward 5'->3' above it, reverse 3'->5' below it",
        "upper case anneals, lower case does not; ^ base edited or removed, + insertion point",
    ]
    for bs in range(lo, hi, WIDTH):
        be = min(bs + WIDTH, hi)
        lines.append("")
        lines.append("bases [%d, %d)" % (bs, be))
        for label, cells in (("template", None), ("forward", top), ("reverse", bottom), ("edit", edit)):
            if cells is None:
                body = t[bs:be]
            else:
                body = "".join(cells.get(k, " ") for k in range(bs, be))
            lines.append((label.ljust(LABEL) + body).rstrip())
    if notes:
        lines.append("")
        lines.extend(notes)
    return "".join(line + "\n" for line in lines)


def check_pairs(t, design):
    """Independent check of the layout: every upper-case reverse base pairs
    with the template base above it, and every upper-case forward base equals
    it (an annealing part of either primer matches the template)."""
    name, _, _, forward, reverse = design
    for _, anneals, sequence, where in forward:
        if anneals:
            assert t[where[0]:where[1]] == sequence, name
    for _, anneals, sequence, where in reverse:
        if anneals:
            assert revcomp(t[where[0]:where[1]]) == sequence, name


def emit(name, text):
    path = os.path.join(OUT, name)
    if "--check" not in sys.argv[1:]:
        with open(path, "w", encoding="ascii") as f:
            f.write(text)
        print("wrote", name)
        return True
    try:
        with open(path, "rb") as f:
            tracked = f.read().decode("ascii")
    except FileNotFoundError:
        tracked = None
    if tracked == text:
        print("OK %s" % name)
        return True
    print("DIFFERS %s" % name)
    old = tracked.splitlines() if tracked is not None else []
    new = text.splitlines()
    for i in range(max(len(old), len(new))):
        a = old[i] if i < len(old) else None
        b = new[i] if i < len(new) else None
        if a != b:
            print("  line %d: tracked %r, derived %r" % (i + 1, a, b))
            break
    return False


def main():
    t = load_template()
    ok = True
    for name, build in (
        ("context-native-01.txt", native_01),
        ("context-del-11.txt", del_11),
        ("context-ins-14.txt", ins_14),
    ):
        design = build(t)
        check_pairs(t, design)
        ok = emit(name, screen(t, design)) and ok
    if not ok:
        sys.exit(1)


if __name__ == "__main__":
    main()
