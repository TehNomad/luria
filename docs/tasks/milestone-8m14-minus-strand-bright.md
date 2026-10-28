# Contract 8m14 — cli: a minus-strand CDS's strand drawn bright (M8-36)

Planner: Claude. Status: written 2026-10-28, to run after 8m13. Tier:
light (D051): drawing only. Owner's answer: `docs/tasks/milestone-8m-polish.md`
§1, M8-36.

## 1. What this contract builds

While the lanes show a minus-strand CDS (`LaneMode::Cds(i)` where CDS `i` is
on the minus strand), the strand that CDS reads is the one drawn as primary:
- the bottom strand's letters are bright and the top strand's are dim, the
  reverse of 8m9;
- the cursor's cell and its codon's underline go on the bottom strand's row
  instead of the top's.

In every other mode (off, a plus-strand CDS, three or six frames) nothing
changes.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `strand_cells`, `base_runs`, `base_style` | `crates/cli/src/sequence_view/layout.rs:587`, `:561`, `:547` | which strand is dim |
| `block_lines`, `usable_cds`, the `LaneMode::Cds` arm | `layout.rs:1026`, `:266`, `:1099` | the CDS's strand is known there |
| `mark_bases`, `Rows` | `crates/cli/src/sequence_view/screen.rs:434` | the cursor and codon row |
| `ViewState::mode`, `ViewState::record` | `crates/cli/src/sequence_view/state.rs:541` | the mode the screen draws |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/layout.rs`, `screen.rs`, `state.rs`.
2. `crates/cli/tests/sequence_view_minus_bright.rs`: new.
3. Nothing else. No existing test should change: every test that checks the
   dimming or the cursor marks today uses no CDS or a plus-strand one (`tet`,
   `synA`, `fP`). If one fails, §9 says what to do.

## 4. Rules

1. **Which strand is bright.** `strand_cells` learns which strand is the
   primary one. When it is the bottom: the top strand's base cells are
   `DIM`, and the bottom strand's base cells are not. The two gutter numbers
   stay dim as they are. The ambiguous-letter colour (8m9) stays on either
   strand. `block_lines` makes the bottom primary only in the `Cds` arm for
   a minus-strand CDS. [1]
2. **Which row the marks go on.** In `mark_bases`, the cursor's `REVERSED`
   and the codon's `UNDERLINED` go on the bottom row when the state's mode is
   `Cds(i)` with CDS `i` on the minus strand, and on the top row otherwise.
   Add one `pub fn` on `ViewState` that answers this (for example
   `reads_bottom(&self) -> bool`), and use it rather than repeating the test
   in the screen. The residue line's codon mark, the hit marks (M8-24) and
   their colours-off look do not change. [2, 3]
3. **Colours off.** Rules 1 and 2 hold with the colours on and off. [1, 2]

## 5. Hand values (the planner's)

`pbr` is `J01749.1.gb` at 80 × 24. `bla` is CDS index 2,
`complement(3293..4153)`; picking it from the `c` list (`Home`, `Down` × 2,
`enter`) selects it, turns its lane on and puts the cursor on 4152 (M8-21),
whose codon is 4152, 4151, 4150. In `cds:bla` the block holding 4152 is
one-based 4091–4156 (zero-based start 4090; D050's phase moves the blocks),
so base `p` of that block is drawn at screen column `6 + p − 4090`: 4150,
4151 and 4152 at 66, 67 and 68. Tests read the block's start from
`state.layout()` rather than trusting this number, and assert it is 4090.

`tet` is CDS index 0, `86..1276`, plus strand.

## 6. Tests (`tests/sequence_view_minus_bright.rs`)

Find the top and bottom rows of a block by reading `state.lines(block)` roles,
as `screen.rs` does, or by the gutter number as `sequence_view_styling.rs`
does.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `a_minus_cds_brightens_the_bottom_strand` | `bla` picked: in the block holding 4152, every base cell of the bottom row is not `DIM` and every base cell of the top row is `DIM`. Again after `C` (colours off) |
| 2 | `the_other_modes_keep_the_top_bright` | `tet` picked: top bases not `DIM`, bottom `DIM`. `bla` picked, then `t` (three frames): the same, and the cursor's cell is `REVERSED` on the top row, not the bottom |
| 3 | `a_minus_cds_marks_its_cursor_on_the_bottom_strand` | `bla` picked: on the bottom row, columns 66–68 are `UNDERLINED` and column 68 is `REVERSED`; on the top row of that block no cell is `UNDERLINED` or `REVERSED`. The residue line still has exactly three cells `REVERSED` and `BOLD` |
| 4 | `a_plus_cds_marks_its_cursor_on_the_top_strand` | `tet` picked (cursor on 85, codon 85–87): the cursor's cell is `REVERSED` on the top row and its codon `UNDERLINED` there, and no cell of the bottom row of that block is either |

## 7. Checks

`scripts/check`, `python3 -I tests/fixtures/verify.py`, `layout_fixtures`
and every `sequence_view*` target.

## 8. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | `block_lines` never makes the bottom primary | 1 |
| G2 | the bottom is primary for every CDS, plus strand too | 2 |
| G3 | `reads_bottom` always false | 3 |
| G4 | `reads_bottom` true in three-frame mode too (any minus CDS selected) | 2 |

## 9. Do not

- No new dependency. No change to `crates/core`, any fixture or table, or
  any existing test.
- No change to the layout tables' text, kinds or columns: only styles change.
- If an existing test fails, finish everything else, then stop and report it
  with its assertion, what it expected and what it got. Don't change it.
- Don't edit this contract.

## 10. Report

Every guard with its failure line and md5 lines before and after, the check
results, any existing test that failed, and any choice §4 left open.
