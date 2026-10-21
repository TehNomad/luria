# Review screen fixtures

Expected text of `luria review`'s sequence-context screen (milestone 7,
`docs/tasks/milestone-7-tui.md` §5.3 and §7; contract 7d). The planner
wrote them in contract 7d0, on 2026-10-21.

## Files

| File | Design it lays out | Source of the design |
| --- | --- | --- |
| `context-native-01.txt` | replacement `[1209, 1212)` `GAT`->`AAC`, bound 15 | `../q5-primer/native.tsv` row `native-01` |
| `context-del-11.txt` | deletion `[1209, 1212)` `GAT`, bound 15 | `../q5-primer/native-deletion.tsv` row `del-11` |
| `context-ins-14.txt` | insertion of `GAT` at 1209, bound 15 | `../q5-primer/native-insertion.tsv` row `ins-14` |
| `reference/context.py` | writes the three files, and checks them with `--check` | |

The template is the SID-1 cDNA, `../example-construct/input/sid-1_cDNA.fa`,
named `sid-1` in the request. Each design's member is named by its row's
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

## Rerunning

```
python3 -I tests/fixtures/review/reference/context.py --check
```

prints `OK` for each file, and is part of `tests/fixtures/verify.py`. It
needs no third-party package.
