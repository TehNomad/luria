---
type: Task Contract
title: "Contract 8l2 — cli: the overview inset (§4.6, §5.1)"
description: "A two-line strip under the title: line 1 is the whole record scaled to the width with the longest features as bars; line 2 marks the part on screen, the cursor and the search hits."
status: deprecated
tags: [milestone-8]
---
# Contract 8l2 — cli: the overview inset (§4.6, §5.1)

Planner: Claude. Status: **Implemented 2026-10-27** by worker `w-8l2` in two
rounds, reviewed and accepted. Tier: light (D051): tests assert behaviour,
not wording; the hand test (8m) may change glyphs and rules. Plan row:
**8l2** in [docs/tasks/contracts/milestone-8-view.md](/tasks/contracts/milestone-8-view.md) §7. Findings from review:

* Round 1: tests 1–7 and G1–G7 behaved as §7 says. The planner checked
  the unit tests' strings by hand with D054, and a planner mutation
  (bars drawn over instead of skipped) fails tests 5 and 7.
* §5 rows 5 and 7 were the planner's error: "neither `─` nor a space"
  cannot hold, because rule 4's label puts a space inside the run. The
  worker reported it and pinned each whole run string instead.
* Two defects, found by a planner probe and fixed in round 2. First, `o`
  could leave the cursor off screen, because it did not call `follow`.
  Second, picking a CDS under the strip did the same in 463 of 1,869
  probed cases: `follow` ran while the list was open, and the blocks'
  rows then left out the strip. The blocks' rows now count the strip
  whenever it is on. Round 2 added tests 8–9 and G8–G9, and the planner
  re-ran G2 against the new `block_rows` (test 2 still fails).
* Choices left open and accepted: screen columns are the buffer's
  0-based `x`; `overview()` keeps its value under a list, and a prompt
  keeps the strip drawn; `o` sits before `q quit` on a 76-character key
  line; a feature with an interval outside the record gets no bar; with
  no room, `w` is 0.

## 1. What this contract builds

A two-line strip under the title: line 1 is the whole record scaled to
the width with the longest features as bars; line 2 marks the part on
screen, the cursor and the search hits. `o` hides and shows it. Display
only: nothing on it can be clicked or jumped to.

Rules that apply, already settled:
- Plan §4.6 and proposal §2.1 (`docs/tasks/proposals/proposal-sequence-view.md:
  70-94`): columns by core's `overview_columns`/`overview_column`; a
  feature crossing the origin is drawn at both ends; only the longest
  features when they do not all fit.
- D054 (`scripts/decision D054`): the floor formula, and a record shorter
  than the strip is drawn one column per base, the strip that much
  shorter.

## 2. What exists and must be reused

| Item | Where | Use here |
| --- | --- | --- |
| `overview_columns`, `overview_column` | `crates/core/src/overview.rs:34`, `:57` | every column a base is drawn in |
| `ViewState`, `visible`, `body_rows`, `follow`, `hits` | `crates/cli/src/sequence_view/state.rs` (8k1–8l1) | the strip's state, and the rows it takes from the blocks |
| `draw`, `draw_body` | `sequence_view/screen.rs` | the strip is drawn on the frame's first two inner rows |
| features as lanes see them | `ViewRecord::features`, `FeatureShape::Drawn` | which features have bars: drawn, not `source` |

## 3. Files the worker may touch

1. `crates/cli/src/sequence_view.rs`: add `pub mod overview;`.
2. `crates/cli/src/sequence_view/overview.rs`: new, with its `mod tests`.
3. `crates/cli/src/sequence_view/state.rs`, `screen.rs`.
4. `crates/cli/tests/sequence_view_overview.rs`: new.

## 4. The public surface, exactly

```rust
// overview.rs
/// The strip's two lines for an inner width: line 1 is `1 `, the strip,
/// ` {length}`; line 2 has the same columns. `shown` is the zero-based
/// first and last base on screen; `hits` are the shown hits' starts.
pub fn overview_lines(record: &ViewRecord, inner_width: u16, shown: (u64, u64),
    cursor: u64, hits: &[u64]) -> (String, String);
// state.rs
impl ViewState {
    pub fn overview(&self) -> bool;           // whether the strip is drawn
}
```

Rules (the §5 tests in brackets):

1. **When.** At open the strip is on when the layout has more than twice
   as many blocks as `visible()` holds with the strip off; otherwise off.
   `o` flips it (not inside a list or a prompt, where `o` is text).
   Nothing below 80 × 24. [1, 2]
2. **Rows.** With the strip on and no list open, the frame's first two
   inner rows are the strip and the blocks get `height − 6` rows: that is
   the row count `visible()` and `follow` measure. While a list is open the
   strip is not drawn and the list keeps `height − 4` rows (8k3). [2, 6]
3. **Columns.** The strip has `w = min(inner_width − len("1 ") −
   len(" {length}"), length)` columns; column `c` stands for
   `overview_columns(length, w)[c]`, and base `p` is drawn in
   `overview_column(length, w, p)`. Line 1 is `1 `, the `w` strip cells,
   ` {length}`. [3, 7]
4. **Line 1, features.** Drawn features other than `source`, longest first
   (bases covered; ties in file order). Each takes the columns of its
   intervals, one run per interval (so a feature across the origin has two
   runs); a feature any of whose columns is already taken is skipped. A
   run shows the name, a space and the arrow (`▸` plus, `◂` minus) and
   then `═`, when it is at least two cells longer than the name; otherwise
   only `═`. Untaken strip cells are `─`. [5, 7]
5. **Line 2, marks.** `▀` in the columns of the bases on screen (from the
   first base of `visible().start` to the last base of the last visible
   block); `·` in the column of every shown hit's start; `▲` in the
   cursor's column. Where two fall in one column, `▲` wins, then `·`, then
   `▀`. Everything else is a space. No colour or modifier on either line.
   [3, 4]
6. The normal key line adds `o` and still fits: 8l1's test
   `the_key_line_fits_eighty_columns` (at most 76 characters at 80 × 24,
   `q quit` included) must still pass.

## 5. Tests the worker writes (`tests/sequence_view_overview.rs`)

`pbr` is `tests/fixtures/genbank/raw/J01749.1.gb` (4361 bp); at 80 × 24 its
inner width is 78, so `w = 78 − 2 − 5 = 71` and the strip is screen
columns 3–73, with `4361` at columns 75–78. `f62` is
`tests/fixtures/view/frames-62.gb` (62 bp: `w = min(78 − 2 − 3, 62) = 62`,
strip columns 3–64; `fP` = `join(56..62,1..11)`, `fM` =
`complement(join(53..62,1..8))`, both 18 bases). `w16` is the FASTA
`>w\nTTCAAAAAAAAAAGAA\n`. 80 × 24. Columns below are screen columns. The
planner derived them with the D054 formula `⌊((p + 1)·w − 1) / L⌋`.

| # | Test | Asserts |
| --- | --- | --- |
| 1 | `the_strip_opens_on_a_long_record` | `pbr`: `overview()` true; row 1 starts `│1 ` and ends `4361│`. `w16` and `f62`: `overview()` false |
| 2 | `o_flips_it_and_it_takes_two_rows` | `pbr`, `t` (three frames; block 0 is 10 lines, block 1 is 9): `visible() == 0..1`; `o` → `overview()` false, `visible() == 0..2` (10 + 1 + 9 = 20 rows fit only without the strip); `o` → true |
| 3 | `line_two_marks_the_screen_and_the_cursor` | `pbr` at open: row 2 has `▲` at column 3, `▀` at column 4, and no other non-space cell inside the frame. `End` → `▲` at column 73 |
| 4 | `hits_are_dotted` | `pbr`, `/ AATT` (cursor 58): row 2 has `▲` at 3, `▀` at 4, and `·` at exactly 7, 24, 55, 60, 64 and 73 (hits at 251; 1319 and 1333; 3233; 3539; 3794; 4359 — 58 is under `▲`) |
| 5 | `the_longest_features_are_bars` | `pbr`: row 1 contains `mobile_element` and `tet`; every cell in columns 54–73 (`mobile_element`, 3148..4361) and 4–23 (`tet` gene, 86..1276) is neither `─` nor a space |
| 6 | `a_list_hides_the_strip` | `pbr`, `f`: row 1 is the first list row (contains `source`) |
| 7 | `a_feature_across_the_origin_has_two_runs` | `f62`, `o` → `overview()` true; row 1: columns 3–13 and 58–64 (`fP`'s two runs) are neither `─` nor a space, columns 14 and 57 are `─`, and `62` ends at column 67 (the strip is narrow, D054); `fM` overlaps `fP` and is skipped |

Plus unit tests in `overview.rs` for `overview_lines` on a hand-built case
of your choice, with the expected strings written out.

## 6. Checks

`scripts/check`, plus `cargo test -p luria-cli --test
sequence_view_overview` and the other four `sequence_view*` targets.

## 7. Proving the tests bite

| # | Change | Must fail |
| --- | --- | --- |
| G1 | The strip is never on at open | 1 |
| G2 | The blocks keep `height − 4` rows with the strip on | 2 |
| G3 | No `▀` marks | 3 |
| G4 | No `·` marks | 4 |
| G5 | Features in file order instead of longest first | 5 |
| G6 | The strip is drawn while a list is open | 6 |
| G7 | A feature's runs use only its first interval | 7 |

## 8. Do not

- No new dependency; no change to `crates/core`, `layout.rs`, `list.rs`,
  `goto.rs`, `search.rs`, `review`, or any existing test or fixture, except
  this one expectation: in `tests/sequence_view_search.rs`,
  `every_shown_hit_is_marked`'s bases rows move from 6 and 13 to 8 and 15
  (the strip takes the frame's first two inner rows); its columns stay.
- No jumping by the strip, no mouse.
- No file is ever written.

## 9. Report

Every guard experiment with its failure line and md5 lines, and every
choice §4 left open.
