---
type: Task Contract
title: "Milestone 7j2: borders, the title badge, colour and the help pop-up"
description: "The owner asked for the screens to look like popular ratatui apps."
status: deprecated
tags: [milestone-7]
---
# Milestone 7j2: borders, the title badge, colour and the help pop-up

Status: **Implemented 2026-10-21** by w7j2 (ran out of context before
clippy) and w7j2b (two test lints). The planner fixed two
`needless_borrow` lints in `view.rs` (`&document.state` and
`&document.experimental_validation` passed without `&`). Proofs 1 and 2
failed as required. Test indices now read the body inside the border:
`BODY_ROWS` 21, inner width 78. Planner: Claude. Parent:
[docs/tasks/contracts/milestone-7-tui.md](/tasks/contracts/milestone-7-tui.md) §5.3b (owner answers M7-23, M7-27 and the
styling paragraph). Changes **drawing only**: every text line 7j1 builds
stays byte for byte what it is, so the string tests still hold.

## 1. What and why

The owner asked for the screens to look like popular ratatui apps. The
state and "not tested at the bench" move from two body rows into one
title badge (plan §3 rule 3 still holds), the body sits in a rounded
border, colour marks what matters, and `?` opens the help as a pop-up
over the screen.

## 2. Layout, `crates/cli/src/review/view.rs`

For `draw` (result screens), at 80x24 and up:

- Rows `0..height-1`: one `Block` with `Borders::ALL`,
  `BorderType::Rounded`. Its title, left, is:
  ` luria review ` then the badge then ` · not tested at the bench `.
  - The badge, by `document.state`: `verified` → ` ✓ verified ` (black on
    green), `failed` → ` ✗ failed ` (white on red),
    `needs_clarification` → ` ? needs clarification ` (black on yellow),
    any other value → ` <state> ` (reverse video). The words come from the
    state, never assumed.
  - `not tested at the bench` is the document's
    `experimental_validation` when it is `not_performed`; any other value
    is written as itself (` · experimental validation: <value> `).
- The body is the block's inner area (`height - 3` rows: two border
  rows and the key row). Scrolling, the `lines a-b of n` suffix and the
  minimum size check work on that height.
- Row `height - 1`: the key line, outside the border, in a dim style
  (`Modifier::DIM`), each key letter in bold: render as spans, splitting
  the existing key text on two spaces, bold up to the first space of each
  item.

`draw_proposal`: the same frame, with the badge ` PROPOSAL ` (black on
cyan) and ` · nothing has run yet ` in place of the state and
validation. Its body text does not change.

## 3. Colour rules, by line content

Applied in `view.rs` only, as spans over the unchanged text:

| where | rule |
| --- | --- |
| overview | the selected row (`▶ `) in reverse video, full width; `no primer pair` and `no design` in red |
| overview, details | `Template`, `Target`, `Variants (n)`, `Warnings` headings bold |
| member block | `template` and `bottom` rows: lower-case letters (edited bases) bold yellow; `protein` row: letters cyan, `*` bold red; `forward` and `reverse` rows: upper case green, lower case (non-annealing) yellow; row labels dim |
| member, below | the column header dim; `Warnings` bold; each warning line yellow; `No primer pair could be designed` bold red |
| problem | the first line bold red |
| everywhere | the `[+N bp …]` marker bold magenta |

A row is recognised by its label (the first 10 characters) or its exact
heading text. No rule changes a character.

## 4. The help pop-up (M7-27)

While `app.help()` is true, draw the screen as usual, then a centred
`Clear` + rounded `Block` titled ` Help `, sized to `text::help()`'s
longest line + 4 by its line count + 2 (clamped to the frame), holding
`text::help()`. 7j1's interim "help replaces the body" goes; the key line while help
is open stays `q quit  esc close`.

## 5. Tests, `crates/cli/tests/review_view.rs`

Existing tests read rows of a `TestBackend` buffer. Shift their row
indices to the new layout (body starts at row 1, column 2; key line at
row `height - 1`) and report each changed index. New:

1. `the_badge_names_the_state`: verified, failed and needs-clarification
   documents: row 0 contains `✓ verified`, `✗ failed`, `? needs
   clarification`, each with `not tested at the bench`.
2. `the_badge_is_coloured`: the buffer cell under the badge's first
   letter has the background colour §2 names, for each state.
3. `the_proposal_badge_is_not_a_result`: row 0 holds `PROPOSAL` and not
   `verified`.
4. `edited_bases_are_highlighted`: native-01's member screen: the cells of
   `gat` on the template row are bold yellow; an upper-case template
   cell is not.
5. `help_is_a_pop_up`: with help open, the body's first row still
   starts with the screen's own text outside the pop-up's columns, and
   the pop-up's rows hold `text::help()`'s lines.

### 5.1 Rules that must be proven

1. The badge taken from the state (hard-code `verified`): test 1 fails on
   the failed document.
2. Clearing under the pop-up: test 5 fails.

## 6. Do not

- Change only `crates/cli/src/review/view.rs` and
  `crates/cli/tests/review_view.rs`. No text line changes; no new
  dependency (ratatui 0.30's `Block`, `BorderType`, `Clear`, `Span`,
  `Style` are enough).
- Do not stage or commit.

## 7. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```
