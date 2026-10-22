# Milestone 7m1c: a coloured run's first character takes its colour

Status: **Implemented, 2026-10-22.** Planner: Claude. Found by
w7m1b during 7m1.

## 1. The bug

`crates/cli/src/review/view.rs`'s `runs` starts with `style =
Style::default()` and changes style only when `index > 0`. So the first
character of the text is pushed under the default style, whatever
`style_of` says for it, and a row whose body starts with a coloured base
draws that base plain. `crates/cli/src/new/view.rs`'s `option_runs` does
the same with `base`, so a choice row's leading `◉` is not green.

## 2. Change

In both functions, the first character's style is its own: set `style`
from the first character before the loop (or set it when `run` is
empty), so every character is drawn in exactly the style its function
gives it. No other change; the text drawn is unchanged.

## 3. Tests

1. `crates/cli/tests/review_view.rs`, `the_first_base_of_a_row_is_coloured`:
   on the del-11 member screen, the first base of the primer table's
   forward sequence is `LightBlue`, as test
   `the_primer_table_colours_only_the_sequence`'s other bases are. If it
   is a tail base (lower case), assert `Yellow` instead; say which.
2. `crates/cli/tests/new_view.rs`, `a_leading_choice_mark_is_green`: on a
   choice row whose text starts with `◉` (as `option_runs` receives it),
   the `◉` cell is green.

Proof: restore the old start (`index > 0` with default/base style) once
in each function; each test fails. Then put the fix back.

## 4. Do not

Change only `runs`, `option_runs` and the two new tests. No other code,
test, fixture, doc, Cargo file or `vendor/`. Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```
