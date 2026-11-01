# Review screen fixtures

Expected text of `luria review`'s sequence-context screen (milestone 7,
`docs/tasks/contracts/milestone-7-tui.md` §5.3 and §7; contract 7d). The planner
wrote them in contract 7d0, on 2026-10-21.

## Files

| File | Design it lays out | Source of the design |
| --- | --- | --- |
| `context-native-01.txt` | replacement `[1209, 1212)` `GAT`->`AAC`, bound 15 | `../q5-primer/native.tsv` row `native-01` |
| `context-del-11.txt` | deletion `[1209, 1212)` `GAT`, bound 15 | `../q5-primer/native-deletion.tsv` row `del-11` |
| `context-ins-14.txt` | insertion of `GAT` at 1209, bound 15 | `../q5-primer/native-insertion.tsv` row `ins-14` |
| `context-ins-04.txt` | insertion of `GATTACA` at 50, bound 15, on the row's own synthetic template (named `syn`): a tail on both primers | `../q5-primer/native-insertion.tsv` row `ins-04` |
| `reference/context.py` | writes the three files, and checks them with `--check` | |
| `order-sid1-del11-ins14.tsv` | the IDT order table for a set of `del-11` and `ins-14` on `sid-1` (contract 7g2) | the rows' own `forward` and `reverse` columns |
| `reference/order.py` | writes it, and checks it with `--check` | |

The template is the SID-1 cDNA, `../example-construct/input/sid-1_cDNA.fa`,
named `sid-1` in the request, except for `ins-04`, whose 100-base template is
the `seq:` column of its row, named `syn`. `ins-04` was added after 7d's
review, because no other design here has a reverse tail; the planner checked
that a `q5 design` run of it gives the row's own two primers. Each design's member is named by its row's
case id (`native-01`, `del-11`, `ins-14`).

## How the expectations were derived

No value comes from luria. `reference/context.py` reads only the template
and the three table rows, which were themselves derived without
`crates/core` (`../q5-primer/README.md`), and which
`crates/cli/tests/end_to_end.rs` already checks luria's designs against
(`the_native_substitution_rows_match_the_table`,
`the_deletion_row_matches_the_table`,
`the_insertion_row_matches_the_table`).

For each design the script:
1. builds the primers' parts from the row: the template's own bases at the
   annealing spans, the written bases at the edit, the tail from the
   `forward_tail` / `reverse_tail` column;
2. asserts each assembled primer equals the row's own `forward` and
   `reverse` sequence where the row records them (`del-11`, `ins-14`), and
   that each annealing part equals the template (forward) or its reverse
   complement (reverse), complemented by a table in the script;
3. writes the layout of contract 7d: 60 template bases a block, the forward
   primer above in its template columns, the reverse primer below written
   3'->5' so each base sits under the base it pairs with, annealing bases
   upper case and the rest lower case, and `^` under each edited base or
   `+` at the insertion point.

The planner also checked each layout by eye against the template: every
upper-case reverse base is the complement of the base above it, and the
forward primer's lower-case bases are exactly the written edit or tail.

## The mutant reading frames

`reference/frame.py` writes `frame.tsv`: for ten cases, the mutant
protein's residues whose codon's middle base lies in a window of the
edited construct, each as `number:symbol:middle`. It splices the edit into
the template and reads codons with its own standard-code table and
complement map, sharing nothing with `crates/` (plan §5.3a, M7-18 to
M7-21). The cases cover a codon change, an in-frame deletion and
insertion (SID-1, from `native-01`, `del-11`, `ins-14`), a one-base
deletion that shifts the frame, a seven-base insertion, a replacement
that shifts the frame into a stop, an early stop, a minus-strand CDS, and
an edit upstream of the CDS that moves its start, and an insertion
before a minus-strand CDS's first base (its last template base), which
moves where reading starts by the inserted length. The planner checked
`native-01`'s residue 404 by hand: `AAC` at `[1209, 1212)` is `N`, over
base 1210.

## The redesigned member screen

`reference/member.py` writes `member-*.txt`, the sequence block the
redesigned member screen draws (plan §5.3a, M7-16 to M7-20): the forward
primer above the top strand, the mutant protein (from `frame.py`)
between the strands, the bottom strand, and the reverse primer below,
with edited bases and non-annealing primer bases in lower case. Its
docstring is the layout's specification. The designs are `native-01`,
`del-11` and `ins-14` on SID-1, and `del-02` (a frameshift), `ins-04`
(tails on both primers), `rep-06` (a replacement that writes more than it
removes, with tails) and `ins-13` (a 210-base insertion, collapsed and
expanded) on the rows' own synthetic template, named `syn`. The planner
checked each by eye: every upper-case primer base matches the template
or bottom strand beside it, and each residue sits over its codon's middle
base.

## The order table

`reference/order.py` writes the table by the rules of
`docs/tasks/contracts/milestone-7-tui.md` §5.4 (owner answers M7-7 to M7-13) from the
`forward` and `reverse` columns of `del-11` and `ins-14`, which were derived
without `crates/core`. All four primers are 15 to 19 nt, so every row is
`25nm` and `STD`. The planner checked by hand that a `q5 design` run of the
same two edits on SID-1 at bound 15 gives these four primers.

## Rerunning

```
python3 -I tests/fixtures/review/reference/context.py --check
python3 -I tests/fixtures/review/reference/order.py --check
python3 -I tests/fixtures/review/reference/frame.py --check
python3 -I tests/fixtures/review/reference/member.py --check
```

print `OK` for each file, and are part of `tests/fixtures/verify.py`. They
need no third-party package.
