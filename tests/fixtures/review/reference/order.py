#!/usr/bin/env python3
"""Expected IDT order table for `luria review` (milestone 7, contract 7g2).

Writes ../order-sid1-del11-ins14.tsv, the primer table luria must export for
a two-member set on the SID-1 cDNA (template id `sid-1`): member `del-11`,
the deletion of ../../q5-primer/native-deletion.tsv row del-11, and member
`ins-14`, the insertion of ../../q5-primer/native-insertion.tsv row ins-14,
both at bound 15. The primers are the rows' own `forward` and `reverse`
columns, derived without crates/core (../../q5-primer/README.md). Nothing
here reads luria's output.

The rules are docs/tasks/milestone-7-tui.md §5.4 (owner answers M7-7 to
M7-13): a header `Name Sequence Scale Purification`, tab-separated; one row
per primer, forward then reverse, members in order; the name
`<template id>_<member>_F` or `_R`; the scale by full primer length (15-60
`25nm`, 61-90 `100nm`, 91-100 `250nm`, 101-200 `4nmU`); purification `STD`;
`\\n` after every row.

Run from the repository root:

    python3 -I tests/fixtures/review/reference/order.py           rewrite
    python3 -I tests/fixtures/review/reference/order.py --check   compare
"""
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.dirname(HERE)
FIXTURES = os.path.dirname(OUT)
NAME = "order-sid1-del11-ins14.tsv"


def row_of(table, case_id):
    with open(os.path.join(FIXTURES, "q5-primer", table), encoding="utf-8") as f:
        lines = f.read().splitlines()
    header = lines[0].split("\t")
    for line in lines[1:]:
        fields = dict(zip(header, line.split("\t")))
        if fields["case_id"] == case_id:
            return fields
    raise SystemExit("no row %s in %s" % (case_id, table))


def scale(length):
    if 15 <= length <= 60:
        return "25nm"
    if 61 <= length <= 90:
        return "100nm"
    if 91 <= length <= 100:
        return "250nm"
    if 101 <= length <= 200:
        return "4nmU"
    raise SystemExit("no scale for %d nt" % length)


def main():
    members = [
        ("del-11", row_of("native-deletion.tsv", "del-11")),
        ("ins-14", row_of("native-insertion.tsv", "ins-14")),
    ]
    lines = ["Name\tSequence\tScale\tPurification"]
    for member, row in members:
        assert row["min_overlap"] == "15" and row["expected"] == "design"
        for suffix, column in (("F", "forward"), ("R", "reverse")):
            sequence = row[column]
            assert sequence and set(sequence) <= set("ACGT"), (member, column)
            lines.append("\t".join(
                ["sid-1_%s_%s" % (member, suffix), sequence, scale(len(sequence)), "STD"]
            ))
    text = "".join(line + "\n" for line in lines)
    path = os.path.join(OUT, NAME)
    if "--check" not in sys.argv[1:]:
        with open(path, "w", encoding="ascii") as f:
            f.write(text)
        print("wrote", NAME)
        return
    try:
        with open(path, encoding="ascii") as f:
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
