---
type: Task Contract
title: "Contract 8m10 — cli: a ruler line over the top strand (M8-33)"
description: "Every block gains one line, the ruler, directly above its top strand."
status: deprecated
tags: [milestone-8]
---
# Contract 8m10 — cli: a ruler line over the top strand (M8-33)

Planner: Claude. Status: **Implemented 2026-10-28** by worker `w-8m10` in
three rounds, reviewed and accepted. Findings from review:

* The planner reran the reference script: `--check` is OK, with 616 blocks
  (unchanged) and 1087 cells (from 941). The script's `constructions`
  assert §5's values by name. G1–G4 each failed the test §8 names, and G2
  also failed the reference's `constructions`.
* 11 existing tests failed, all from the ruler's row, as §9 expected.
  Round 2 changed six of them (levels, search, strands) only in their
  expected rows or role lists: +1 row for each ruler above. The five that
  test what fits (scroll, the frame codon split across blocks, three
  overview tests) kept their expected values and got a taller local
  height. `picking_a_cds_keeps_the_cursor_on_screen` got `H + 1`, an exact
  fit, where the planner asked for one row per block. Accepted.
  `sequence_view_colours.rs` gained `drawn_at` so that one test can draw at
  its own height.
* Round 2 restored the frames in the reference docstring's line order, and
  round 3 corrected the order in the fixtures README.
* Process: in round 1 the worker added a draft to this contract and then
  removed it, leaving no diff. It was told not to edit contracts.

Tier: full
(D051): the ruler changes every block's lines, so it goes through the layout
tables and their reference. Owner's answer: [docs/tasks/contracts/milestone-8m-polish.md](/tasks/contracts/milestone-8m-polish.md)
§1, M8-33: "a ruler line with a tick every 10 bases and a number every 30,
next to the top strand". The planner reads "next to" as the line directly
above the top strand, in every mode. The hand test can move it.

## 1. What this contract builds

Every block gains one line, the ruler, directly above its top strand. It
marks every tenth base with a tick and every thirtieth with that base's
number. Nothing else on any line moves; only the lines below the ruler move
down by one.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `LineRole`, `block_lines`, `strand_cells`, `Cell` | `crates/cli/src/sequence_view/layout.rs:136`, `:940-1045` | the new role and its line |
| `line_count`, `drawn_lines` | `crates/cli/src/sequence_view/state.rs:1027-1050` | rows already follow `block_lines`; no change expected |
| `lay_out`, `validate_cells`, `constructions`, `CASES` | `tests/fixtures/view/reference/layout_columns.py:308`, `:389`, `:444`, `:79` | the reference rule and its hand checks |
| `role_for`, the row counts | `crates/cli/tests/layout_fixtures.rs:217`, `:668`, `:680` | read the new role |
| "Block layout" | `tests/fixtures/view/README.md:446` | record the rule and the checks |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/layout.rs`, `state.rs`, `screen.rs` (the
   last two only if something there needs the new role).
2. `tests/fixtures/view/reference/layout_columns.py`: the rule in `lay_out`,
   its docstring, the new `CASES` entry (rule 5), and new checks in
   `constructions`.
3. `tests/fixtures/view/layout-blocks.tsv`, `layout-cells.tsv`: regenerated
   by the script, never edited by hand.
4. `tests/fixtures/view/README.md`, the "Block layout" section only.
5. `crates/cli/tests/layout_fixtures.rs`: `role_for` learns `"ruler"`, and
   the two row counts and the module comment's counts become the
   regenerated tables' counts.
6. `layout.rs`'s unit test `the_gutter_is_at_least_two_columns`: insert
   `LineRole::Ruler` before `LineRole::Top` in its roles list. The rows after
   it move down by one (top `rows[4]`, bottom `rows[5]`, the minus frames
   from `rows[6]`). Assert that the ruler row is blank: a 5-base record has
   no tenth base.
7. `crates/cli/tests/sequence_view_ruler.rs`: new.

## 4. Rules

1. **The role.** `LineRole` gains `Ruler`. `block_lines` puts it directly
   before `Top` in every mode:
   - off: ruler, top, bottom;
   - a plus-strand CDS: residues, numbers, ruler, top, bottom;
   - a minus-strand CDS: ruler, top, bottom, numbers, residues;
   - frames: `+3`, `+2`, `+1`, ruler, top, bottom, then `-1` to `-3` for six.

   The lanes and the `+N more` line stay above everything. The ruler line
   is there in every block, even when it holds nothing. [tables, 1]
2. **What it holds.** Take each base `p` of the block `[s, e)` (zero-based),
   with `n = p + 1`, at column `g + p − s` (`g = d + 1`):
   - If `n` is a multiple of 30, the number `n` is written right-aligned so
     that its last digit stands at that column. It is not drawn if it would
     start left of `g`, and a tick goes in that column instead.
   - Otherwise, if `n` is a multiple of 10, a tick `╷` goes in that column.

   Every other column of the line is blank, including the gutter: the
   ruler has no label. A number is at most 8 digits (D056's 64 MiB), and
   ticks are 10 columns apart, so a number never covers a tick. The
   reference's `validate_cells` still checks it. [tables, 1, 2]
3. **Style.** Every tick and number is `Modifier::DIM` with no `fg` or
   `bg`, whether the colours are on or off (the secondary text, M8-33). [3]
4. **Table rows.** In `layout-cells.tsv` the ruler's line has role `ruler`.
   A tick is kind `tick` with value `╷` and cells 1. A number is kind
   `ruler_number` with its digits as the value. A block's ruler with nothing
   on it has no rows, but its line number still counts.
5. **A new case.** Change `("pbr322", 40, "cds:tet", [60, 88])` to
   `[60, 88, 118]` so the block 119–148 is tabled. It is the block where a
   number has no room (rule 2).

## 5. Hand values (the planner's, from the rules and the file)

These go into `constructions` in the reference, and into the Rust tests.
Columns are layout columns: 0 at the width's left edge, no border.

- `pbr322`, width 78, off, block 0 (bases 1–66, `d = 4`, `g = 5`): ticks at
  14, 24, 44, 54 (bases 10, 20, 40, 50); `30` at 33–34; `60` at 63–64.
- The same, block 1 (67–132, `s = 66`): ticks at 8, 18, 38, 48, 68 (bases
  70, 80, 100, 110, 130); `90` at 27–28; `120` at 56–58.
- `pbr322`, width 40, `cds:tet` (30-base lines; blocks 61–88, 89–118,
  119–148, which 8m4's table gives). In block 89–118 (`s = 88`), `90` is at
  5–6 and the ticks are at 16 and 26. In block 119–148 (`s = 118`), base 120
  is at column 6, and its number would start at 4, left of `g = 5`, so the
  line holds only ticks, at 6, 16 and 26, and no digit.
- `frames62`, width 78 (`d = 2`, `g = 3`, one line 1–62): ticks at 12, 22,
  42, 52; `30` at 31–32; `60` at 61–62. Six frames: the roles are `+3 +2
  +1 ruler top bottom -1 -2 -3`.

## 6. Tests (`tests/sequence_view_ruler.rs`)

Use `block_lines` and read symbols and styles per column, as
`sequence_view_lane_colours.rs` does. Find the ruler line by its role.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `the_ruler_stands_over_the_top_strand` | `pbr`, width 78, `LaneMode::Off`, blocks 0 and 1: the line right before `Top` is `Ruler`, and its non-blank columns are exactly §5's. With `tet` selected (`Cds`), the order around it is numbers, ruler, top |
| 2 | `a_number_without_room_is_a_tick` | `pbr`, width 40, `tet` selected, block 119–148: the ruler holds `╷` at 6, 16, 26 and nothing else. Block 89–118: `90` at 5–6, ticks at 16 and 26 |
| 3 | `the_ruler_is_dim_and_uncoloured` | the test 1 block with colours on, and again with them off: every non-blank ruler cell is dim, with no `fg` or `bg` |
| 4 | `the_screen_draws_the_ruler` | `pbr` view at 80 × 24: the row above block 0's top row holds `30` at screen columns 34–35 (layout column + 1) and `╷` at 15 |

## 7. Checks

`scripts/check`; `python3 -I tests/fixtures/view/reference/layout_columns.py
--check` (after regenerating); `python3 -I tests/fixtures/verify.py`;
`layout_fixtures` and every `sequence_view*` target.

## 8. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | the ruler goes after the top strand | 1 and `layout_fixtures` |
| G2 | a number starts at its base instead of ending there | 1, and a `constructions` check in the reference (change the reference the same way, run `--check`, and restore) |
| G3 | a number without room is dropped with no tick | 2 |
| G4 | the ruler is not dim | 3 |

## 9. Do not

- No new dependency. No change to `crates/core`, any other fixture or table,
  or any existing test except §3 items 5 and 6.
- No change to any other line's text or column.
- Don't write a table row by hand or from luria's output; the tables come
  from the reference script.
- If any other existing test fails, finish everything else, then stop and
  report each failing test with its assertion, what it expected and what it
  got, and whether the only difference is the ruler's row. Don't change it.

## 10. Report

The regenerated tables' row counts and the script's `--print` summary,
every guard with its failure line and md5 lines before and after, the check
results, any existing test that failed, and any choice §4 left open.
