# Contract 8m8 — cli: the cursor's codon highlighted on the residue line (M8-32)

Planner: Claude. Status: **Implemented 2026-10-28** by worker `w-8m8` in
two rounds, reviewed and accepted. Tier: light (D051). Owner's answer:
[docs/tasks/contracts/milestone-8m-polish.md](/tasks/contracts/milestone-8m-polish.md) §1, M8-32. Findings from review:

* G1–G4 each failed the test §7 names. For G2, widening the span alone
  also needed the loop's skip widened; the worker noted this.
* A gap in this contract: 8k1's `the_screen_shows_cursor_codon_and_status`
  asserted that exactly one cell on screen is reversed. Round 2 changed it
  to one cell reversed and not bold (the cursor), plus three reversed and
  bold. The planner checked those three columns by hand: `synlin`'s CDS
  `43..345` puts the cursor's codon at bases 42–44, columns 47–49.
* Not tested: a minus-strand CDS's residue line, which is found by the
  same role lookup.

## 1. What this contract builds

When the cursor stands in a codon of the selected CDS and that CDS's
residue line is drawn, the codon's three cells on that line are reversed
and bold. Today only the bases line marks it: the cursor's cell is
reversed and the codon is underlined. Nothing else changes.

M8-32's other half is already done: the status line shows the base, the
residue and its number, the codon's letters and span, and the features
under the cursor (8k1 §4 rule 13, `state.rs:716-757`).

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `cursor_codon` → `CursorCodon { positions, … }` | `crates/cli/src/sequence_view/state.rs:864` | the codon's three bases, including one split by the origin |
| `Marks`, `Rows`, `mark_bases`, how `top` and `bottom` are found by role | `crates/cli/src/sequence_view/screen.rs:167`, `:177`, `:363`, `:249-260` | find the residue row the same way, and mark it in the same loop |
| `LineRole::Residues` | `crates/cli/src/sequence_view/layout.rs` | the selected CDS's residue line, drawn only in `LaneMode::Cds` |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/screen.rs`.
2. `crates/cli/tests/sequence_view_cursor_codon.rs`: new.

## 4. Rules

1. **The row.** `Rows` gains `residues: Option<u16>`: the screen row of the
   block's `LineRole::Residues` line, found by role as `top` is, and `None`
   when the block has none.
2. **The mark.** For each base `p` of the block in `codon.positions`,
   `Modifier::REVERSED | Modifier::BOLD` is added to the cell at `p`'s
   column on the residue row. That column is the same as on the top row.
   The cell's own colours and letter stay. The mark goes only on the cells
   of the three positions, never on a span between them, so a codon split
   by the origin marks its two pieces and nothing between. [1, 2]
3. With no selected CDS, or a cursor that is in no codon of it, the residue
   row is not marked. `C` (colours off) does not remove the mark. [3]
4. The bases row is unchanged: the cursor is still reversed and the codon
   still underlined there.

## 5. Tests

`pbr` is `J01749.1.gb`. `c`, `Home`, `enter` selects `tet` (`86..1276`)
and puts the cursor on 85, its start codon (8m1). With `tet` selected, the
lines are 66 bases (8m4). Block 1 is bases 66–129 (zero-based), so base `p`
is at screen column `6 + p − 66`. `f62` is `frames-62.gb` (one line, base
`b` at column `4 + b`). There `c`, `Home`, `enter` selects `fP`
(`join(56..62,1..11)`). Its codon 3 is bases 61, 0, 1 (zero-based). Find the
residue row as the row whose cell at the expected letter column holds the
letter (`M` for codon 1 of either CDS), not by a fixed number. Read cells
from a `TestBackend` draw.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `the_cursor_codon_is_marked_on_the_residue_line` | `pbr`, select `tet`: on the residue row, columns 25, 26, 27 are reversed and bold, and 24 and 28 are not. `l` ×3 (cursor 88, codon 2, bases 88–90): 28, 29, 30 are marked, and 25 to 27 no longer are. On the top row, the cursor's cell (28) is still reversed |
| 2 | `a_codon_split_by_the_origin_marks_both_pieces` | `f62`, select `fP`, then `g`, `1`, `enter` (cursor 0): on the residue row, columns 4, 5 and 65 are reversed and bold. Columns 6 and 64 are not, and no column from 7 to 63 is |
| 3 | `no_codon_no_mark` | `pbr`, select `tet`, then `Home` (cursor 0, outside `tet`): no cell of any row is bold and reversed. Select `tet` again, then `C`: columns 25 to 27 of the residue row are still reversed and bold |

## 6. Checks

`scripts/check`, `python3 -I tests/fixtures/verify.py`, and every
`sequence_view*` target.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | only the cursor's own base is marked on the residue row | 1 |
| G2 | the span from the lowest to the highest position is marked | 2 |
| G3 | the mark goes on the top row instead of the residue row | 1 |
| G4 | the mark is skipped when colours are off | 3 |

## 8. Do not

- No new dependency. No change to `layout.rs`, `state.rs`, `crates/core`,
  any fixture or table, or any existing test.
- No change to the status line.

## 9. Report

Every guard with its failure line and md5 lines before and after, the check
results, and any choice §4 left open.
