---
type: Task Contract
title: "Milestone 7c3: drawing the result screens, with scrolling"
description: "7c1 and 7c2 build each screen's lines; 7b says which screen is open."
status: deprecated
tags: [milestone-7]
---
# Milestone 7c3: drawing the result screens, with scrolling

Status: **Implemented 2026-10-21** by the local worker; accepted after one
question (the 7a key-line assertion, approved below). Notes: a stale-row
concern was checked by the planner and is not a bug (ratatui resets its
back buffer on every swap, `ratatui-core` `swap_buffers`); `c` opens the
context screen for any member, and 7d shows the no-design line there.
Planner: Claude. Parent: [docs/tasks/contracts/milestone-7-tui.md](/tasks/contracts/milestone-7-tui.md) §3 rules 1 and
3, §5.1, §5.2, §6 (7c). Builds on 7a (`view.rs`), 7b (`app.rs`), 7c1 and
7c2 (`text.rs`).

## 1. What and why

7c1 and 7c2 build each screen's lines; 7b says which screen is open. This
contract draws them: the banner on every screen, then the open screen's
lines, scrolled when they are longer than the screen. The sequence-context
screen is 7d; until then it draws a placeholder.

## 2. Requirements this contract applies

- **Plan §3 rule 3:** the state and the experimental validation are on
  every screen, at the top, whatever the scroll position.
- **Plan §3 rule 1:** the view draws `text.rs`'s lines as they are; it
  adds only the banner, the key line and the scroll position.
- **7a's size rule** is unchanged: below 80x24, only the size message.

## 3. Changes

### 3.1 `crates/cli/src/review/app.rs`

1. `App` gains a private `scroll: usize`, 0 in `new`, and `pub fn
   scroll(&self) -> usize`.
2. `Key` gains `PageUp`, `PageDown`, `Home`.
3. `handle`, on every screen except `Overview`: `Down` or `Char('j')`
   adds 1 to `scroll`; `Up` or `Char('k')` subtracts 1, not below 0;
   `PageDown` adds 10; `PageUp` subtracts 10, not below 0; `Home` sets 0.
   On `Overview`, `Up`/`Down`/`j`/`k` keep moving the selection as in 7b,
   and `PageUp`, `PageDown` and `Home` scroll as above.
4. Every change of screen sets `scroll` to 0.
5. `scroll` has no upper bound here: the app does not know how many lines
   a screen has. The view clamps it (§3.2).

### 3.2 `crates/cli/src/review/view.rs`

At 80x24 or larger, the area is split into rows:
- rows 0 and 1: 7a's banner, `state: <state>` and `experimental
  validation: <experimental_validation>`, unchanged;
- row 2: blank;
- rows 3 to `h - 2`: the body;
- row `h - 1`: the key line.

The body's lines, by `app.screen()`:
- `Overview`: `text::overview(document, app.selected())`;
- `Member(i)`: `text::member(document, i)`;
- `Context(i)`: the single line `sequence context: not yet available
  (7d)`;
- `Provenance`: `text::provenance(document)`;
- `Problem`: `text::problem(document)`.

Scrolling: with `b` body rows and `n` lines, the first line drawn is
`min(app.scroll(), n.saturating_sub(b))`; the body shows lines from there.
A line longer than the width is cut at the width, not wrapped, so a
primer's row never moves.

The key line, by screen:
- `Overview`: `q quit  enter open  j/k move  p provenance`, followed by
  `  e problem` when the document has an error or clarifications;
- `Member`: `q quit  esc back  c context  j/k scroll`;
- `Context`: `q quit  esc back`;
- `Provenance` and `Problem`: `q quit  esc back  j/k scroll`.

When the body has more lines than rows, append to the key line
`  lines <first>-<last> of <n>`, 1-based and inclusive.

When the first line drawn is past the overview's selected row, the
selection is still correct but off screen; that is accepted for now.

### 3.3 `crates/cli/src/review/terminal.rs`

Key mapping only: `KeyCode::PageUp` → `Key::PageUp`, `KeyCode::PageDown`
→ `Key::PageDown`, `KeyCode::Home` → `Key::Home`.

## 4. Tests

`app.rs` unit tests (add):
1. `scrolling_moves_and_stops_at_zero`: on `Provenance`, `Down` ×3 → 3,
   `Up` ×5 → 0, `PageDown` → 10, `Home` → 0.
2. `changing_screen_resets_the_scroll`: scroll on `Member(0)`, `c`, then
   `scroll() == 0`.
3. `overview_keys_still_move_the_selection`: 7b's
   `selection_moves_and_stops_at_the_ends` still passes unchanged.

`crates/cli/tests/review_view.rs` (add; 7a's four tests must still pass
unchanged, except one assertion: the planner approved, 2026-10-21, that
`the_state_and_validation_are_on_screen`'s last-row check becomes the
overview's full key line, `q quit  enter open  j/k move  p provenance`,
because §3.2 replaces 7a's `q quit` line):
4. `every_screen_keeps_the_banner`: with the set document of 7c1 §4, open
   each of `Overview`, `Member(0)`, `Context(0)`, `Provenance`, through
   keys, and assert rows 0 and 1 are the banner, built from the
   document's fields.
5. `the_body_is_the_text_of_the_screen`: at 80x24, on `Provenance`, the
   body rows equal the first body-height lines of
   `text::provenance(document)`, each cut at 80 characters and trimmed.
6. `a_long_screen_scrolls_and_says_where`: the set document's
   `Member(0)` at 80x24, whose body is 20 rows (rows 3 to 22). First
   assert that `text::member(document, 0).len()` is more than 20; if it
   is not, stop and report. Press `PageDown`; the first body row is line
   `min(10, n - 20)` of `text::member`, cut and trimmed, and the key line
   ends with `lines <first>-<last> of <n>`, computed from that position.
7. `scrolling_past_the_end_stops_at_the_last_page`: `PageDown` ×100; the
   last body row is the screen's last line.
8. `the_key_line_offers_problem_only_with_a_problem`.

### 4.1 Rules that must be proven

1. The banner drawn only on the overview: test 4 fails.
2. No clamp on the scroll: test 7 fails.
3. A screen change that keeps the scroll: test 2 fails.

## 5. Do not

- Change only `app.rs`, `view.rs`, the key mapping in `terminal.rs`, and
  `tests/review_view.rs` (adding tests only).
- Do not change `text.rs` or its tests, any other file, fixtures, [docs/](/),
  any `Cargo.toml` or `vendor/`.
- Do not wrap lines, add colour as meaning, or draw the context screen.
- Do not add dependencies. Do not stage or commit.

## 6. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the proofs of §4.1, and any point where this contract
does not match the code; stop and report rather than guess.
