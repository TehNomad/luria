# Contract 8m9 — cli: dimming, title badge, rule between blocks, ambiguous bases, colours off means none (M8-33, M8-34)

Planner: Claude. Status: **Implemented 2026-10-28** by worker `w-8m9` in
three rounds, reviewed and accepted. Tier: light (D051): every style here
can change at the hand test. Owner's answers:
`docs/tasks/milestone-8m-polish.md` §1, M8-33 (dimming) and M8-34. Findings
from review:

* Two gaps in this contract, both reported by the worker rather than
  worked around. 8k1's `the_title_names_the_topology_the_view_moves_on`
  pinned `"4361 bp circular"`; round 2 updated it (§3 item 5). Test 5 named
  hit 58, but selecting `tet` puts the cursor on 85, so the search goes to
  251. The planner checked the hits against the file (58, 251, 1319, …) and
  round 2 pinned 251 at columns 61–64. Round 3 allowed for the cursor's
  codon 250–252, which underlines column 60 on the top row.
* G1–G4 each failed the test §7 names. G5 fails only the `layout.rs` unit
  test: no stop is on screen in test 5's state. Accepted, since the unit
  test checks the stop directly.
* Colour inventory: stop red, Zappo backgrounds and their black letters,
  ambiguous 208, lane and strip colours, and the hit's yellow. Each is gated
  on `colours`. `overview.rs` needed no change.
* Choices accepted: the title reads `pBR322 · 4361 bp · ⟳ circular`; the
  rule spans a full line's width even before a shorter last block.

## 1. What this contract builds

Drawing only. No line, row, column or text of a block changes, except the
blank row between two blocks and the title.

1. The bottom strand, the gutter labels and the top line's right number are
   dim.
2. The title shows a topology badge, and the frame's border is dim.
3. The blank row between two blocks holds a dim dotted rule.
4. Ambiguous bases (anything but `A C G T`) are drawn in their own colour.
5. With colours off (`C`, or `NO_COLOR` at open), nothing in the view is
   coloured. The stop is bold only, and a search hit is marked bold and
   underlined instead of yellow.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `strand_cells`, `label_cell`, `frame_cells`, `residue_style`, `Drawing` | `crates/cli/src/sequence_view/layout.rs` | dimming and ambiguous bases |
| `title`, `topology_words` | `crates/cli/src/sequence_view/state.rs:662` | the badge |
| `draw`, `frame_block`, the loop placing blocks with one blank row between them, `mark_bases` | `crates/cli/src/sequence_view/screen.rs:52-65`, `:186-270`, `:363` | the border, the rule and the hit mark |
| `colours()` | `state.rs` (8m5) | the one switch for every colour |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view/layout.rs`, `state.rs`, `screen.rs`.
2. `crates/cli/src/sequence_view/overview.rs`: only to apply rule 5, if it
   draws a colour that rule 5 must turn off.
3. `layout.rs`'s unit test at about `:1306`: it asserts the stop is red
   with `colours: false`. Pass `colours: true` there instead, and add the
   rule 5 assertion for `false`.
4. `crates/cli/tests/sequence_view_styling.rs`: new.
5. `state.rs`'s `the_title_names_the_topology_the_view_moves_on` (round 2):
   its `"4361 bp circular"` and `"393 bp linear"` become
   `"4361 bp · ⟳ circular"` and `"393 bp · — linear"`.

## 4. Rules

1. **Dim** (`Modifier::DIM`, whatever `colours` is):
   - every cell of the `Bottom` line;
   - every gutter label (`label_cell`: a line's first base number, or a
     frame label such as `+1`);
   - the top line's right number.

   The top strand's bases, the residues and the lanes are not dim. [1]
2. **Title.** The topology words get a badge before them: `⟳` for circular
   and `—` for linear, for example `pBR322 · 4361 bp · ⟳ circular · no CDS
   selected`. Keep every word `title` has today, including `(assumed)`.
   The frame's border cells are dim; the title text is not. [2]
3. **The rule.** In the one blank row between two drawn blocks, the
   columns of a full line's bases hold `┄`, dim and uncoloured. Those are
   the inner columns `d + 1` to `d + N` (gutter `d`, line width `N`). Every
   other column of that row stays blank. No row is added or removed, and
   nothing is drawn above the first block or below the last. [3]
4. **Ambiguous bases.** With colours on, a cell of the `Top` or `Bottom`
   line whose letter is not one of `A C G T` gets `fg(Color::Indexed(208))`.
   On the bottom line it is dim as well. [4]
5. **Colours off.** When `colours()` is false:
   - no cell of the drawn view has an `fg` or `bg` other than
     `Color::Reset`, in the frame, the strip, the blocks, a list, the key
     list or the status and key lines;
   - the stop keeps `BOLD` and loses its red;
   - a search hit's cells get `BOLD | UNDERLINED` on the row its strand
     names (M8-24) instead of the yellow background;
   - modifiers (reversed, bold, underlined, dim) all stay.

   Find every `Color` the sequence view draws, and list them in the report
   with how each obeys this rule. [5, 6]

## 5. Tests (`tests/sequence_view_styling.rs`)

Read cells from a `TestBackend` draw at 80 × 24. Columns are screen
columns.
- `pbr` is `J01749.1.gb`: `d = 4`, 66-base lines, so a base `b` of the
  block starting at `s` is at column `6 + b − s`, and the bases run from
  column 6 to 71.
- `amb` is `tests/fixtures/view/frames-amb.fa`: 33 bp, linear assumed,
  `d = 2`, one line, so base `b` is at column `4 + b`. The planner read its
  ambiguous bases from the file, zero-based: R at 14, N at 17, 20 and
  23–26, S at 31, K at 32. Their columns are 18, 21, 24, 27–30, 35 and 36.

Find rows by content, not by a fixed number. For example, a top row is the
row whose gutter holds the block's first base number.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `the_secondary_text_is_dim` | `pbr` at open, block 0. Every cell of its bottom row from 6 to 71 is dim, and no top-row base cell (6–71) is. The gutter label `1` (column 4) and the right number `66` are dim. After `t`, the cell holding `+1` is dim |
| 2 | `the_title_has_a_topology_badge` | `pbr`'s row 0 contains `⟳ circular`. `amb`'s contains `— linear` and `(assumed)`. The border's corner cell (0, 0) is dim |
| 3 | `a_dotted_rule_between_blocks` | `pbr` with `o` (strip off), where blocks 0 and 1 are drawn. Exactly one row between block 0's top row and block 1's top row holds `┄`. In it, columns 6 to 71 are `┄` and dim, columns 1 to 5 and 72 to 78 are blank, and no other row of the body holds `┄` |
| 4 | `ambiguous_bases_have_their_own_colour` | `amb`: on the top row, columns 18, 21, 24, 27, 28, 29, 30, 35, 36 have fg 208, and columns 4 to 17 do not. The same columns on the bottom row are fg 208. After `C`, none has an fg |
| 5 | `colours_off_means_no_colour` | `pbr`: `c`, `Home`, `enter` (`tet`), `/`, type `AATT`, `enter`, then `C`. No cell of the buffer has an fg or bg other than `Color::Reset`. Selecting `tet` puts the cursor on 85, so the search goes to the first hit after it, 251 (zero-based; the hits are 58, 251, 1319, …). Its block starts at base 197 (1-based), so the hit is at columns 61–64, strand `both`. Those cells have `BOLD` and `UNDERLINED` on both the block's top and bottom rows, Columns 60 and 65 have neither on the bottom row, and are not bold on the top row, where the cursor's codon 250–252 underlines 60 (rounds 2–3) |
| 6 | `no_color_at_open_reaches_every_colour` | `set_colours(false)` on a fresh `pbr` view, then `f` (the feature list) and `?`: in each, no cell has an fg or bg other than `Reset` |

## 6. Checks

`scripts/check`, `python3 -I tests/fixtures/verify.py`, `layout_fixtures`
and every `sequence_view*` target.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | the bottom line is not dim | 1 |
| G2 | the rule spans the whole inner width | 3 |
| G3 | ambiguous bases coloured with colours off | 4 or 5 |
| G4 | search hits stay yellow with colours off | 5 |
| G5 | the stop stays red with colours off | 5, and the `layout.rs` unit test |

## 8. Do not

- No new dependency. No change to `crates/core`, any fixture or table, or
  any existing test except §3 item 3.
- No text change other than the badge, and no row change other than the
  rule. If another existing test fails, stop and report it with the
  assertion. Don't change it.

## 9. Report

Every guard with its failure line and md5 lines before and after, the
colour inventory rule 5 asks for, the check results, and any choice §4
left open.
