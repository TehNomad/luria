"""Check the hand-derived feature-edit table (D039, plan 9 contract 9a).

../feature-edits.tsv says what one edit of a record's bases does to one
feature: its intervals, its partial marks, the qualifiers M9-6 (h) and
M9-13 change, and what the status line names. The planner wrote every
expected column by hand from the rules in docs/tasks/milestone-9-editor.md
(M9-6, M9-13, M9-14) as ../README.md states them. This script writes
nothing: it applies those rules again, on plain lists of intervals, and
asserts that every expected column agrees. It shares no code with
`crates/` and never runs luria.

Run from the repository root:

    python3 -I tests/fixtures/editor/reference/feature_edits.py
    python3 -I tests/fixtures/editor/reference/feature_edits.py --check

Both forms print `OK feature-edits.tsv (<n> rows)` and exit 0, or print
`DIFFERS feature-edits.tsv` with the first disagreement and exit 1.
--check is accepted so verify.py can run this like every other check.
"""
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TABLE = os.path.join(HERE, "..", "feature-edits.tsv")
COLUMNS = [
    "id", "rule", "length", "topology", "key", "shape", "qualifiers", "edit",
    "length_after", "shape_after", "qualifiers_after", "named",
]
POSITION_QUALIFIERS = ("transl_except", "anticodon")
POSITION = re.compile(r"pos:(complement\()?(\d+)(?:\.\.(\d+))?(\))?")


class Interval:
    """One interval, 1-based inclusive, with the marks written on its ends."""

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


def format_shape(strand, intervals):
    if not intervals:
        return "removed"
    parts = []
    for iv in intervals:
        parts.append("{}{}..{}{}".format(
            "<" if iv.low_mark else "", iv.low,
            ">" if iv.high_mark else "", iv.high,
        ))
    return strand + " " + ",".join(parts)


def parse_edit(text):
    """Return (kind, first, last, count) in the table's 1-based terms."""
    words = text.split()
    if words[0] == "ins":
        assert words[2] == "after", text
        return "ins", int(words[3]), None, int(words[1])
    first, last = (int(n) for n in words[1].split("-"))
    assert first <= last, text
    if words[0] == "del":
        return "del", first, last, None
    if words[0] == "rep":
        return "rep", first, last, int(words[2])
    assert words[0] == "sub", text
    return "sub", first, last, None


def crossing_pairs(intervals, length, circular):
    """Indexes i where interval i ends at the last base and i+1 starts at 1."""
    if not circular:
        return set()
    return {
        i for i in range(len(intervals) - 1)
        if intervals[i].high == length and intervals[i + 1].low == 1
    }


def insert(intervals, after, count, length, circular):
    """M9-6 (a), (b) and M9-14. Return whether any interval grew."""
    pairs = crossing_pairs(intervals, length, circular)
    grows_end = {i for i in pairs} if after == length else set()
    grows_start = {i + 1 for i in pairs} if after == 0 else set()
    grew = False
    for i, iv in enumerate(intervals):
        if i in grows_end:
            iv.high += count
            grew = True
        elif i in grows_start:
            iv.high += count
            grew = True
        elif after < iv.low:
            iv.low += count
            iv.high += count
        elif after < iv.high:
            iv.high += count
            grew = True
    return grew


def delete(intervals, first, last):
    """M9-6 (a), (c), (d). Return (kept intervals, whether any base went)."""
    removed = last - first + 1
    kept = []
    touched = False
    lost_low_mark = False
    lost_high_mark = False
    for i, iv in enumerate(intervals):
        if last < iv.low:
            iv.low -= removed
            iv.high -= removed
        elif first > iv.high:
            pass
        elif first <= iv.low and last >= iv.high:
            touched = True
            if i == 0 and iv.low_mark:
                lost_low_mark = True
            if i == len(intervals) - 1 and iv.high_mark:
                lost_high_mark = True
            continue
        elif first <= iv.low:
            touched = True
            iv.low = first
            iv.high -= removed
        elif last >= iv.high:
            touched = True
            iv.high = first - 1
        else:
            touched = True
            iv.high -= removed
        kept.append(iv)
    # A mark belongs to the feature's end: if its interval went, the
    # feature's new end carries it (README, "Partial marks").
    if kept and lost_low_mark:
        kept[0].low_mark = True
    if kept and lost_high_mark:
        kept[-1].high_mark = True
    return kept, touched


def five_prime_trim(strand, intervals, first, last):
    """The feature's bases a deletion takes from its 5' end, in order."""
    if strand == "+":
        bases = [b for iv in intervals for b in range(iv.low, iv.high + 1)]
    else:
        bases = [b for iv in reversed(intervals) for b in range(iv.high, iv.low - 1, -1)]
    count = 0
    for base in bases:
        if not first <= base <= last:
            break
        count += 1
    return count


def move_position(value, edit, length):
    """M9-13 (a): the qualifier's new value, or None when the edit touches it."""
    match = POSITION.search(value)
    assert match, value
    low = int(match.group(2))
    high = int(match.group(3)) if match.group(3) else low
    steps = []
    kind, first, last, count = edit
    if kind == "ins":
        steps.append(("ins", first, count))
    elif kind == "del":
        steps.append(("del", first, last))
    elif kind == "sub":
        steps.append(("sub", first, last))
    else:
        steps.append(("del", first, last))
        steps.append(("ins", first - 1, count))
    for step in steps:
        if step[0] == "ins":
            after, n = step[1], step[2]
            if low <= after < high:
                return None
            if after < low:
                low += n
                high += n
        else:
            a, b = step[1], step[2]
            if a <= high and b >= low:
                return None
            if step[0] == "del" and b < low:
                low -= b - a + 1
                high -= b - a + 1
    text = str(low) if not match.group(3) else "{}..{}".format(low, high)
    if match.group(1):
        text = "complement(" + text + ")"
    return value[:match.start()] + "pos:" + text + value[match.end():]


def parse_qualifiers(text):
    if text == "-":
        return []
    return [tuple(q.split("=", 1)) for q in text.split(";")]


def format_qualifiers(qualifiers):
    if not qualifiers:
        return "-"
    return ";".join(name + "=" + value for name, value in qualifiers)


def apply(row):
    length = int(row["length"])
    circular = row["topology"] == "circular"
    assert row["topology"] in ("linear", "circular"), row["id"]
    strand, intervals = parse_shape(row["shape"])
    qualifiers = parse_qualifiers(row["qualifiers"])
    edit = parse_edit(row["edit"])
    kind, first, last, count = edit
    if kind == "ins":
        assert 0 <= first <= length, row["id"]
    else:
        assert 1 <= first <= last <= length, row["id"]

    changed = False
    trimmed = 0
    if kind == "ins":
        changed = insert(intervals, first, count, length, circular)
        length_after = length + count
    elif kind == "sub":
        changed = any(first <= iv.high and last >= iv.low for iv in intervals)
        length_after = length
    else:
        trimmed = five_prime_trim(strand, intervals, first, last)
        intervals, changed = delete(intervals, first, last)
        length_after = length - (last - first + 1)
        if kind == "rep":
            # M9-6 (e): a replacement is a deletion, then an insertion at
            # the same place, on the record the deletion left.
            grew = insert(intervals, first - 1, count, length_after, circular)
            changed = changed or grew
            length_after += count

    if not intervals:
        return str(length_after), "removed", "-", "feature"

    named = []
    result = []
    for name, value in qualifiers:
        if name == "translation" and row["key"] == "CDS" and changed:
            continue
        if name in POSITION_QUALIFIERS:
            moved = move_position(value, edit, length)
            if moved is None:
                named.append(name)
                continue
            value = moved
        result.append((name, value))

    if row["key"] == "CDS" and trimmed:
        names = [name for name, _ in result]
        old = int(dict(result)["codon_start"]) if "codon_start" in names else 1
        new = (old - 1 - trimmed) % 3 + 1
        if "codon_start" in names:
            result = [(n, str(new) if n == "codon_start" else v) for n, v in result]
        elif new != 1:
            result.append(("codon_start", str(new)))

    return (
        str(length_after),
        format_shape(strand, intervals),
        format_qualifiers(result),
        ",".join(named) if named else "-",
    )


def main():
    with open(TABLE, encoding="utf-8") as handle:
        lines = handle.read().split("\n")
    assert lines[-1] == "", "the table must end with one newline"
    header = lines[0].split("\t")
    if header != COLUMNS:
        print("DIFFERS feature-edits.tsv\n  header: {}".format(header))
        return 1
    ids = set()
    rows = 0
    for number, line in enumerate(lines[1:-1], start=2):
        fields = line.split("\t")
        assert len(fields) == len(COLUMNS), "line {}".format(number)
        row = dict(zip(COLUMNS, fields))
        assert row["id"] not in ids, "duplicate id " + row["id"]
        ids.add(row["id"])
        expected = (row["length_after"], row["shape_after"], row["qualifiers_after"], row["named"])
        got = apply(row)
        if got != expected:
            print("DIFFERS feature-edits.tsv")
            print("  line {} ({}): expected {}".format(number, row["id"], "\t".join(expected)))
            print("  {} rules give {}".format(" " * len("line {} ({}):".format(number, row["id"])), "\t".join(got)))
            return 1
        rows += 1
    print("OK feature-edits.tsv ({} rows)".format(rows))
    return 0


if __name__ == "__main__":
    sys.exit(main())
