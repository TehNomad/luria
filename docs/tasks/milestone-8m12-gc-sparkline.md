# Contract 8m12 — cli: a GC-content sparkline as the strip's third line (M8-34)

Planner: Claude. Status: delegated to `w-8m12` 2026-10-28. Tier: light
(D051): the strip is display only, and the glyphs and scale can change at
the hand test. Owner's answer: `docs/tasks/milestone-8m-polish.md` §1,
M8-34: "a GC-content sparkline as a third strip line".

## 1. What this contract builds

The overview strip gains a third line under its two (8l2). Each strip column
shows the GC content of the bases it stands for, as one of eight block
heights. The strip now takes three rows, so with the strip shown the blocks
have one row less.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `overview_columns` (D054) | `crates/core/src/overview.rs:34` | the base range of each column |
| `strip`, `overview_lines`, `overview_drawn`, `HEAD` | `crates/cli/src/sequence_view/overview.rs:38`, `:82`, `:97` | the columns and `w` that line 1 uses |
| `draw_overview` | `crates/cli/src/sequence_view/screen.rs:128` | draw the third line |
| `OVERVIEW_ROWS`, `new`, `resize`, `relayout` | `crates/cli/src/sequence_view/state.rs:124`, `:313` | three rows; compute the line once per width |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/overview.rs`, `screen.rs`, `state.rs`.
2. `crates/cli/tests/sequence_view_gc.rs`: new.
3. Existing tests that fail **only** because the strip is one row taller
   (§9 says what to do with any other failure). Two kinds of change are
   allowed, and nothing else in those tests:
   - **Rows.** With the strip shown, every row below it is one lower. An
     expected row number may go up by exactly 1 for that reason, along with
     its comment's arithmetic.
   - **Fit.** A test whose point is which blocks fit with the strip shown
     keeps every expected value (`visible()` ranges, counts). It gets one
     more row of terminal height: a local height in that test only, or +1 on
     the local height it already has. Its comment's arithmetic is updated
     to match.

   List every such change in the report: the test, old → new, and why.

## 4. Rules

1. **The line.** `pub fn gc_line(record: &ViewRecord, inner_width: u16) ->
   String` in `overview.rs` returns the strip's `w` cells for that width:
   the same `w` and the same column ranges as line 1 (`overview_columns`).
   It holds only those cells, with no head and no tail. [1, 2]
2. **One cell.** Over the column's bases, count `G`, `C` and `S` as GC, and
   `A`, `T` and `W` as AT, in either case. Every other letter (`N`, `R`,
   `Y`, `K`, `M`, `B`, `D`, `H`, `V`) is left out of both counts. A column
   with no counted base is a space. Otherwise its glyph is
   `"▁▂▃▄▅▆▇█"[min(7, ⌊8·GC / (GC + AT)⌋)]`. The scale is absolute: 0 % is
   `▁` and 100 % is `█`, whatever the record's own range. [1, 2]
3. **Drawn.** `draw_overview` draws the third line on the inner row under
   the marks line: two blank cells under `1 `, then `gc_line`'s cells under
   line 1's strip cells, then ` GC`, clipped to the inner width. The line
   has no colour and no modifier. [3]
4. **Rows.** `OVERVIEW_ROWS` becomes 3. Everything that takes the strip's
   rows off the body already reads that constant. When `o` hides the strip,
   the third line goes with it. [4]
5. **Once per width.** The state computes the line in `new` and again
   whenever the width changes (`resize`), and keeps it. `draw_overview`
   reads the kept line and never counts bases while drawing, because a
   64 MiB record would make every frame slow (D056).

## 5. Hand values (the planner's, computed from the files independently)

`pbr` is `J01749.1.gb` (4361 bp). At inner width 78, `w = 71` (the module
doc of `overview.rs`). Column 0 is bases 0–60: 20 GC out of 61, ⌊8·20/61⌋ =
2, so `▃`. Column 1 is 61–121: 31/61, so `▅`. Column 35 is 2149–2210: 45/62,
so `▆`. Column 70 is 4299–4360: 22/62, so `▃`. The whole line is:

```
▃▅▆▅▅▅▆▆▆▅▅▆▅▅▆▆▅▅▅▅▆▅▅▆▅▅▅▅▅▄▄▅▅▅▅▆▅▄▅▅▄▅▅▆▅▆▅▅▅▅▄▄▂▃▄▅▅▄▄▅▅▄▄▅▄▄▄▃▃▄▃
```

`amb` is `tests/fixtures/view/frames-amb.fa`, 33 bp,
`ATGTATTAATAGTARAANGGNCTNNNNGTGTSK`. At inner width 78, `w = 33`, one base
per column. Its line (shown between `|` marks) is:

```
|▁▁█▁▁▁▁▁▁▁▁█▁▁ ▁▁ ██ █▁    █▁█▁█ |
```

R (14), N (17, 20, 23–26) and K (32) are spaces, and S (31) is `█`.

## 6. Tests (`tests/sequence_view_gc.rs`)

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `the_gc_line_by_hand` | `gc_line(pbr, 78)` equals §5's line, and its characters 0, 1, 35 and 70 are `▃ ▅ ▆ ▃` |
| 2 | `ambiguous_bases_are_left_out` | `gc_line(amb, 78)` equals §5's 33 cells |
| 3 | `the_strip_draws_it_third` | `pbr` view at 80 × 24, where the strip is open: screen row 3 holds §5's line at columns 3–73, columns 1–2 are blank, and columns 75–76 read `GC`. Rows 1 and 2 hold the strip's first two lines (row 1 starts with `1 `). No cell of row 3 has an `fg`, a `bg` or a modifier, with colours on and after `C` |
| 4 | `the_strip_takes_three_rows` | `pbr` at 80 × 24, strip open: the top row of block 0 (the row whose gutter holds `1` at column 4) is at least row 4, and no row above row 4 holds a base. After `o`, no cell of the frame holds any of `▁▂▃▄▅▆▇█` |

## 7. Checks

`scripts/check`, `python3 -I tests/fixtures/verify.py`, and every
`sequence_view*` target.

## 8. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | ambiguous letters counted as AT | 2 |
| G2 | the scale is the record's min to max instead of 0–100 % | 1 |
| G3 | `OVERVIEW_ROWS` stays 2 (the line overdrawn by the blocks) | 3 or 4 |
| G4 | the line is drawn with `o` off | 4 |

## 9. Do not

- No new dependency. No change to `crates/core`, `layout.rs`, any fixture
  or table, or any existing test except as §3 item 3 allows.
- No change to the strip's lines 1 and 2, or to `overview_lines`' signature.
- If an existing test fails for a reason other than §3 item 3's two, finish
  everything else, then stop and report it with its assertion, what it
  expected and what it got. Don't change it.
- Don't edit this contract.

## 10. Report

Every guard with its failure line and md5 lines before and after, every
change made under §3 item 3, the check results, and any choice §4 left open.
