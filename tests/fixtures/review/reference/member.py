#!/usr/bin/env python3
"""Expected sequence block of the redesigned member screen (plan §5.3a).

Writes ../member-<case>.txt, the block `luria review` draws when a member
is opened (contract 7i3): the forward primer above the top strand, the
mutant protein between the strands, the bottom strand, the reverse primer
below it. Nothing is read from luria: the primers come from independently
verified rows of ../../q5-primer/, the protein from ./frame.py's own
splice and translation.

Run from the repository root:

    python3 -I tests/fixtures/review/reference/member.py           rewrite
    python3 -I tests/fixtures/review/reference/member.py --check   compare

The layout, in display columns:

* Template positions before the edit keep their columns. The edit's
  region takes R = max(e - s, w) columns (e - s template bases replaced by
  w written ones), or, for an insertion longer than 30 bases that is not
  expanded, the width of its marker. Template positions at or past the
  edit's end follow the region.
* `template` row: the template's bases, the region's template bases in
  lower case (for an insertion, the inserted bases in lower case), `-`
  where a replacement wrote more bases than it removed. A collapsed
  insertion shows `[+N bp <first 10>...<last 10>]`, its bases lower case.
* `bottom` row: the complement of each template-row base; blank under the
  marker, `-` under `-`.
* `protein` row: each mutant residue over the display column of its
  codon's middle base in the edited construct; none inside a collapsed
  marker.
* `forward` and `reverse` rows: each primer part over its columns, upper
  case when it anneals, lower case when not; a mutation part's written
  bases over the region from its first column; the reverse primer's text
  reversed, so it reads 3'->5'. A tail sits just left of the forward
  primer's first spanned column, or reversed just right of the reverse
  primer's last; none inside a collapsed marker.
* The window runs from 5 columns before the leftmost drawn primer column
  to 5 after the rightmost, within the display; 60 columns a block.

Lines:

    <member>  <summary>  <template id> bases <a>–<b>

where <summary> is the edit as requested, in 1-based inclusive
coordinates (plan §5.3b, M7-24): `X→Y at a–b` (or `at a` for one base)
for a substitution or replacement, `delete X at a–b` for a deletion,
`insert X after p` for an insertion at zero-based point p (`at the
start` for 0), each base text longer than 30 written as `<N> bp`; and
<a>–<b> the window's template bases, 1-based inclusive. The legend is
not here: it is the `?` help pop-up (M7-27).
    (blank)
    forward   ...
    template  ...
    protein   ...
    bottom    ...
    reverse   ...
    (blank, then the next block's five rows, ...)
    (blank, then one note per collapsed insertion and per hidden tail)

Every row is cut of trailing spaces; an empty row is its bare label.
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.dirname(HERE)
FIXTURES = os.path.dirname(OUT)
sys.path.insert(0, HERE)
import frame  # noqa: E402  this directory's own script

WIDTH = 60
LABEL = 10
INLINE_LIMIT = 30
COMPLEMENT = {"A": "T", "T": "A", "G": "C", "C": "G",
              "a": "t", "t": "a", "g": "c", "c": "g", "-": "-"}


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
    a, b = text.split(":")
    return int(a), int(b)


def revcomp(bases):
    return "".join(COMPLEMENT[b] for b in reversed(bases))


def template_of(column):
    if column.startswith("seq:"):
        return "syn", column[len("seq:"):]
    return "sid-1", frame.load(frame.SID1)


# A design: member, (template id, bases), (cds start, end, strand),
# (s, e, written), forward parts, reverse parts. A part is
# (role, anneals, sequence, template span or None).

def native_01():
    r = row_of("native.tsv", "native-01")
    tid, t = template_of(r["sequence_file"])
    s, e, original, written = r["edit"].split(":")
    s, e = int(s), int(e)
    assert t[s:e] == original
    f0, f1 = span(r["forward_span"])
    r0, r1 = span(r["reverse_span"])
    forward = [("context", True, t[f0:s], (f0, s)), ("mutation", False, written, (s, e)),
               ("context", True, t[e:f1], (e, f1))]
    reverse = [("context", True, revcomp(t[r0:r1]), (r0, r1))]
    return "native-01", (tid, t), (0, 3153, "+"), (s, e, written), forward, reverse


def deletion(case_id, cds):
    r = row_of("native-deletion.tsv", case_id)
    tid, t = template_of(r["template"])
    s, e, original = r["deletion"].split(":")
    s, e = int(s), int(e)
    assert t[s:e] == original
    f0, f1 = span(r["forward_span"])
    r0, r1 = span(r["reverse_span"])
    assert t[f0:f1] == r["forward"] and revcomp(t[r0:r1]) == r["reverse"]
    forward = [("context", True, r["forward"], (f0, f1))]
    reverse = [("context", True, r["reverse"], (r0, r1))]
    return case_id, (tid, t), cds, (s, e, ""), forward, reverse


def insertion(case_id, cds):
    r = row_of("native-insertion.tsv", case_id)
    tid, t = template_of(r["template"])
    point = int(r["point"])
    f0, f1 = span(r["forward_anneal_span"])
    r0, r1 = span(r["reverse_anneal_span"])
    forward = []
    if r["forward_tail"] != "none":
        forward.append(("tail", False, r["forward_tail"], None))
    forward.append(("context", True, t[f0:f1], (f0, f1)))
    assert "".join(p[2] for p in forward) == r["forward"]
    reverse = []
    if r["reverse_tail"] != "none":
        reverse.append(("tail", False, r["reverse_tail"], None))
    reverse.append(("context", True, revcomp(t[r0:r1]), (r0, r1)))
    assert "".join(p[2] for p in reverse) == r["reverse"]
    return case_id, (tid, t), cds, (point, point, r["inserted"]), forward, reverse


def replacement_inline(case_id, cds):
    r = row_of("native-replacement.tsv", case_id)
    assert r["path"] == "inline"
    tid, t = template_of(r["template"])
    s, e, original = r["region"].split(":")
    s, e = int(s), int(e)
    assert t[s:e] == original
    f0, f1 = span(r["forward_span"])
    r0, r1 = span(r["reverse_span"])
    written = r["replacement"]
    forward = [("context", True, t[f0:s], (f0, s)), ("mutation", False, written, (s, e)),
               ("context", True, t[e:f1], (e, f1))]
    reverse = [("context", True, revcomp(t[r0:r1]), (r0, r1))]
    assert "".join(p[2] for p in forward) == r["forward"], "".join(p[2] for p in forward)
    assert reverse[0][2] == r["reverse"]
    return case_id, (tid, t), cds, (s, e, written), forward, reverse


def replacement_tails(case_id, cds):
    r = row_of("native-replacement.tsv", case_id)
    assert r["path"] == "tails"
    tid, t = template_of(r["template"])
    s, e, original = r["region"].split(":")
    s, e = int(s), int(e)
    assert t[s:e] == original
    f0, f1 = span(r["forward_span"])
    r0, r1 = span(r["reverse_span"])
    forward = [("tail", False, r["forward_tail"], None), ("context", True, t[f0:f1], (f0, f1))]
    reverse = [("tail", False, r["reverse_tail"], None),
               ("context", True, revcomp(t[r0:r1]), (r0, r1))]
    assert "".join(p[2] for p in forward) == r["forward"]
    assert "".join(p[2] for p in reverse) == r["reverse"]
    return case_id, (tid, t), cds, (s, e, r["replacement"]), forward, reverse


SID_CDS = (0, 3153, "+")
SYN_CDS = (0, 99, "+")


def marker(written):
    return "[+%d bp %s...%s]" % (len(written), written[:10].lower(), written[-10:].lower())


def bases_text(bases):
    return "%d bp" % len(bases) if len(bases) > INLINE_LIMIT else bases


def bases_at(s, e):
    # zero-based half-open [s, e) as 1-based inclusive
    return "%d" % (s + 1) if e - s == 1 else "%d\u2013%d" % (s + 1, e)


def summary(t, s, e, written):
    if s == e:
        where = "at the start" if s == 0 else "after %d" % s
        return "insert %s %s" % (bases_text(written), where)
    if not written:
        return "delete %s at %s" % (bases_text(t[s:e]), bases_at(s, e))
    return "%s\u2192%s at %s" % (bases_text(t[s:e]), bases_text(written), bases_at(s, e))


def screen(design, expanded):
    name, (tid, t), cds, (s, e, w_bases), forward, reverse = design
    w = len(w_bases)
    insertion_ = e == s
    collapsed = insertion_ and w > INLINE_LIMIT and not expanded
    region = len(marker(w_bases)) if collapsed else max(e - s, w)
    total = len(t) - (e - s) + region

    def col(position):  # template position -> display column
        if position < s:
            return position
        if position < e:
            return s + (position - s)
        return s + region + (position - e)

    template_row = {}
    for p in range(len(t)):
        if not (s <= p < e):
            template_row[col(p)] = t[p]
    if collapsed:
        for i, ch in enumerate(marker(w_bases)):
            template_row[s + i] = ch
    else:
        shown = t[s:e].lower() if not insertion_ else w_bases.lower()
        for i in range(region):
            template_row[s + i] = shown[i] if i < len(shown) else "-"
    bottom_row = {}
    for c, ch in template_row.items():
        if collapsed and s <= c < s + region:
            continue
        bottom_row[c] = COMPLEMENT[ch]

    notes = []

    def place(parts, rev, role):
        cells = {}
        spanned = [p for p in parts if p[3] is not None]
        first = min(col(p[3][0]) for p in spanned)
        last = max(col(p[3][1] - 1) + 1 for p in spanned)
        for part_role, anneals, seq, where in parts:
            shown = seq if anneals else seq.lower()
            text = shown[::-1] if rev else shown
            if where is None and collapsed:
                # A tail is inserted bases: inside a collapsed insertion,
                # all of it is hidden.
                notes.append("%s tail: %d nt inside the insertion, i to expand" % (role, len(seq)))
                continue
            if where is None:
                start = last if rev else first - len(text)
                cols = list(range(start, start + len(text)))
            elif part_role == "mutation":
                cols = list(range(s, s + len(text)))
            else:
                cols = [col(p) for p in range(where[0], where[1])]
            hidden = False
            for c, ch in zip(cols, text):
                if collapsed and s <= c < s + region:
                    hidden = True
                    continue
                if 0 <= c < total:
                    cells[c] = ch
            if hidden:
                notes.append("%s %s: %d nt inside the insertion, i to expand" % (role, part_role, len(seq)))
        return cells

    top = place(forward, False, "forward")
    bot = place(reverse, True, "reverse")
    used = list(top) + list(bot)
    lo = max(0, min(used) - 5)
    hi = min(total, max(used) + 1 + 5)

    protein = {}
    # every residue of the construct; placed only inside the window below
    for item in frame.residues(t, cds, (s, e, w_bases), (0, len(t) - (e - s) + w)):
        number, symbol, middle = item.split(":")
        m = int(middle)
        if m < s:
            c = m
        elif m < s + w:
            if collapsed:
                continue
            c = s + (m - s)
        else:
            c = s + region + (m - s - w)
        protein[c] = symbol

    def template_position(c, at_end):
        # A display column, as a template position: a column inside the
        # edit's region is the region's start, or for the window's end, its
        # end; the window's end is exclusive, so `hi` maps to the position
        # after the last drawn base.
        if c <= s and not (at_end and c == s and s < e):
            return c
        if c < s + region:
            return e if at_end else s
        return e + (c - s - region)

    lines = [
        "%s  %s  %s bases %s" % (name, summary(t, s, e, w_bases), tid,
                                 bases_at(template_position(lo, False), template_position(hi, True))),
    ]
    # Blocks of WIDTH columns; a block never splits a collapsed marker: it
    # ends just before the marker, and the next block starts at it.
    blocks = []
    bs = lo
    while bs < hi:
        be = min(bs + WIDTH, hi)
        if collapsed and bs < s < be < s + region:
            be = s
        blocks.append((bs, be))
        bs = be
    for bs, be in blocks:
        lines.append("")
        for label, cells in (("forward", top), ("template", template_row),
                             ("protein", protein), ("bottom", bottom_row), ("reverse", bot)):
            body = "".join(cells.get(c, " ") for c in range(bs, be))
            lines.append((label.ljust(LABEL) + body).rstrip())
    if collapsed:
        notes.insert(0, "insertion: %d bp, i to expand" % w)
    if notes:
        lines.append("")
        lines.extend(notes)
    return "".join(line + "\n" for line in lines)


def emit(name, text):
    path = os.path.join(OUT, name)
    if "--check" not in sys.argv[1:]:
        with open(path, "w", encoding="utf-8") as f:
            f.write(text)
        print("wrote", name)
        return True
    try:
        with open(path, encoding="utf-8") as f:
            tracked = f.read()
    except FileNotFoundError:
        tracked = None
    if tracked == text:
        print("OK %s" % name)
        return True
    print("DIFFERS %s" % name)
    return False


def main():
    designs = [
        ("member-native-01.txt", native_01(), False),
        ("member-del-11.txt", deletion("del-11", SID_CDS), False),
        ("member-del-02.txt", deletion("del-02", SYN_CDS), False),
        ("member-ins-14.txt", insertion("ins-14", SID_CDS), False),
        ("member-ins-04.txt", insertion("ins-04", SYN_CDS), False),
        ("member-rep-02.txt", replacement_inline("rep-02", SYN_CDS), False),
        ("member-rep-06.txt", replacement_tails("rep-06", SYN_CDS), False),
        ("member-ins-13.txt", insertion("ins-13", SYN_CDS), False),
        ("member-ins-13-expanded.txt", insertion("ins-13", SYN_CDS), True),
    ]
    ok = True
    for name, design, expanded in designs:
        ok = emit(name, screen(design, expanded)) and ok
    if not ok:
        sys.exit(1)


if __name__ == "__main__":
    main()
