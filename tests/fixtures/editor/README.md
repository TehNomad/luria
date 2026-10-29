# Fixture: how features follow an edit (Milestone 9, contract 9a)

`feature-edits.tsv` says what one edit of a record's bases does to one
feature: its intervals and partial marks, the qualifiers that change, and
what the status line names. It drives contract 9b's edit buffer. The rules
are the owner's answers M9-6, M9-13 and M9-14 in
`docs/tasks/milestone-9-editor.md` (approved 2026-10-29); this file states
them in the table's terms, with the readings the planner chose where the
answers leave a detail open (marked **Reading**).

## Provenance

The planner wrote every row, inputs and expected columns, by hand from the
rules below (2026-10-29). `reference/feature_edits.py` applies the same
rules a second time on plain lists of intervals and asserts that every
expected column agrees; it writes nothing, shares no code with `crates/`,
and never runs luria (D039). Each of its rules was disabled once and made
the named rows fail: the origin rule at the end (o1) and at the start (o2),
the moving mark (d4), the added `/codon_start` (s8), the minus-strand 5′
order (s6), the dropped `/translation` (g3), a position touched by an
insertion (t5) and by a deletion (t4), and growth inside an interval (b1).

```
python3 -I tests/fixtures/editor/reference/feature_edits.py --check
```

## Columns

| Column | Meaning |
| --- | --- |
| `id` | Row name; the letter is the rule group |
| `rule` | The plan row the case pins |
| `length`, `topology` | The record before the edit: its length in bases, `linear` or `circular` |
| `key` | The feature key. Only `CDS` loses `/translation` or changes `/codon_start` |
| `shape` | Strand (`+` or `-`), a space, then the intervals in the file's order, 1-based inclusive `low..high`, comma-separated, with `<` before a low end and `>` before a high end as GenBank writes them. A minus-strand join lists intervals as `complement(join(…))` does; an origin-crossing feature lists the interval that ends at the record's last base first (D036) |
| `qualifiers` | `name=value` pairs joined by `;`, in file order, or `-` |
| `edit` | `ins K after N` (K bases between base N and N+1; `after 0` is before base 1), `del A-B` (bases A to B), `rep A-B K` (bases A to B replaced by K bases), `sub A-B` (bases A to B replaced by as many, M9-6 e's `r`). Only lengths matter here, so inserted bases are counted, not spelled |
| `length_after`, `shape_after`, `qualifiers_after` | The record's length and the feature after the edit; `removed` when no interval is left (qualifiers then `-`) |
| `named` | What the status line names: `feature` for a removed feature, the qualifier's name for one M9-13 (a) removes, or `-` |

## Rules, in the table's terms

For an interval `a..b`, an insertion `ins k after N` and a deletion `del A-B`
of `m = B - A + 1` bases:

- **Insertion** (M9-6 a, b): `N < a` shifts the interval to `a+k..b+k`;
  `a ≤ N < b` grows it to `a..b+k`; `N ≥ b` leaves it. So an insertion at
  either edge never grows it, and a one-base interval never grows.
- **Deletion** (M9-6 a, c, d): `B < a` shifts to `a-m..b-m`; `A > b`
  leaves it; `A ≤ a` and `B ≥ b` removes it; `A ≤ a ≤ B < b` trims the
  low end to `A..b-m`; `a < A ≤ b ≤ B` trims the high end to `a..A-1`;
  otherwise it shrinks to `a..b-m`. A feature with no interval left is
  removed.
- **Replacement** (M9-6 e): the deletion, then `ins K after A-1` on the
  record the deletion left. Bases that replace a feature's edge therefore
  land outside it (e2, s11), and replacing all of a feature removes it (e3).
  `sub` moves nothing.
- **Joins** (M9-6 f): each interval on its own, so an insertion between two
  touching intervals opens a gap (f2).
- **The origin** (M9-6 g, M9-14): on a circular record, when an interval
  ending at the last base is followed in the feature by one starting at
  base 1, that junction counts as inside the feature: `ins k after L`
  grows the first to `…..L+k`, and `ins k after 0` grows the second to
  `1..y+k` while every other interval shifts. Anywhere else, and on a linear
  record, `after 0` and `after L` are edges.
- **Partial marks** (M9-6 d): a trimmed end keeps its mark. **Reading:** a
  mark belongs to the feature's end, not to one interval, so when a deletion
  removes the interval that carried it, the feature's new first (`<`) or
  last (`>`) interval carries it (d4, s10).
- **`/translation`** (M9-6 h): a `CDS` loses it when the edit changed any
  of its bases: a deletion or `sub` over any of its intervals, or an
  insertion that grows one. A shift or an edge insertion keeps it (h1, h4,
  h5).
- **Positions** (M9-13 a): the `pos:` of `/transl_except` and `/anticodon`
  (`low..high` or one base, inside `complement(…)` or not) shifts as an
  interval would. An edit that touches it, a deletion or `sub` over one of
  its bases or an insertion strictly inside it, removes the qualifier and
  names it. The origin rule does not apply to positions.
- **`/codon_start`** (M9-13 b): k is the number of the feature's own bases
  the deletion removes before the first base that stays, read 5′ to 3′:
  for `+` from the low end of the first interval, for `-` from the high end
  of the last. If k > 0 the value becomes `((codon_start − 1 − k) mod 3) + 1`.
  Bases between intervals do not count (s10). **Reading:** an absent
  `/codon_start` counts as 1, and when the new value is not 1 it is added
  after the last qualifier (s8, s9), so the frame of a 5′-partial CDS stays
  right.

## Shapes

Most rows use a 100-base record. Rows g1–g3 and o1–o3 use the two
origin-crossing CDS shapes of `../view/frames-62.gb` (62 bases, circular);
o4–o5 use the 4361-base length of pBR322 with the shape the plan's M9-14
names. M9-12 (records whose locations the view refuses) and M9-6 (i) (CDS
checks after an edit) are not in this table: 9b and 9e test them.

## Location text (9c2)

`locations.tsv` gives, for one feature shape on one record, the GenBank
location text that shape is written as. Its test 1 asks `write_edit_location`
for the row's text and its test 2 asks 9b2's `read_edit_location` to read
that text back to the row's shape, on the row's length and topology; both run
in `crates/core/tests/location_text_fixtures.rs`. The planner wrote every
`text` cell by hand from the rules below (2026-11-06), and nothing was filled
in by running luria. `reference/locations.py` builds a minimal record of the
row's length and topology, puts one `misc_feature` at the row's text, reads it
with Biopython 1.88, and asks that the parts that feature has are the shape's
ranges — reversing them first, because Biopython lists a minus join 5′→3′.

| Column | Meaning |
| --- | --- |
| `id` | Row name |
| `length`, `topology` | The record the text is read against: its length in bases, `linear` or `circular` |
| `shape` | As in "Columns": strand and intervals. A minus-strand join lists them as `complement(join(…))` writes them, and an origin-crossing feature lists the interval that ends at the record's last base first (D036) |
| `text` | The location text, on one line (9c3 wraps it) |

Rows l1–l14 use the 100-base record; l3 uses pBR322's 4361-base length, and
l15–l16 the two origin-crossing shapes of `../view/frames-62.gb`. l17 uses
4366, a length of its own to put an origin join at the end of.

The rules, in the table's terms (contract 9c2 §4):

- **One interval** `(s, e)`, zero-based half-open (D006), writes `s+1..e`. A
  one-base interval, `e == s + 1`, writes its bare position: the number `e`
  on its own (M9-18). If a mark stands on it, it writes a range with the mark,
  `<s+1..e` or `s+1..>e`, because a bare position has no end to be open past.
- **Marks** (M9-17): `<` stands before the first interval's low end when the
  feature's 5′ end is open, and `>` before the last interval's high end when its
  3′ end is open, counting in the order the shape lists them. They belong to the
  feature's ends, not to the interval that carries them ("Partial marks",
  Reading 1).
- **Joins** (M9-17): one interval writes itself; several are `,`-joined inside
  `join(…)` in the order the shape lists them, whatever order that is (l15 is
  D036 order, and l13 keeps the touching intervals ribosomal slippage writes).
- **The strand** (M9-18): a minus location wraps the whole text in
  `complement(…)`, so a minus join reads `complement(join(…))` and never
  `join(complement(…),…)`. l16 is the minus origin join: its text reads back as
  the D036 shape the table records.

The command, from the repository root:

```
python3 -I tests/fixtures/editor/reference/locations.py --check
```

It prints `OK locations.tsv (17 rows)` or `DIFFERS locations.tsv` with the row,
and exits 1 on a difference; it writes nothing and never runs luria.
`tests/fixtures/verify.py` runs it with every other check.
