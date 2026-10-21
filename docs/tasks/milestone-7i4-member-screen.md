# Milestone 7i4: the redesigned member screen, its keys, and the trimmed overview

Status: **Ready for the worker, 2026-10-21.** Planner: Claude. Parent:
`docs/tasks/milestone-7-tui.md` §5.3a (owner answers M7-16, M7-17,
M7-19, M7-22). Builds on 7i2 (`App::frame`) and 7i3
(`text::member_sequence`).

## 1. What and why

The owner found the review screens too full. After this contract:

- `enter` on a member opens its **sequence screen**: 7i3's block, then
  one short line per primer. The technical lines (today's member screen)
  move behind `t`.
- `i` expands or collapses a long insertion; it starts collapsed every
  time the screen opens.
- The overview lists the template, the target and the members only.
- The old context screen and its key `c` go: the sequence screen
  replaces them.

## 2. Changes

### 2.1 `crates/cli/src/review/app.rs`

- `Screen`: `Context(usize)` is replaced by `Technical(usize)`.
  `Member(usize)` is now the sequence screen.
- `App` gains `expanded: bool`, `false` from `new`, and
  `pub fn expanded(&self) -> bool`.
- Keys, replacing the `Member`/`Context` rows of `handle`:

| screen | key | effect |
| --- | --- | --- |
| `Member(i)` | `t` | `Technical(i)` |
| `Member(i)` | `i` | `expanded = !expanded`, only when `expandable(document, i)`; otherwise nothing |
| `Member(i)` | `esc`, `backspace` | `Overview`, `selected = i` (as now) |
| `Technical(i)` | `esc`, `backspace` | `Member(i)` |
| `Member`, `Technical` | `j` `k` arrows `PageUp` `PageDown` `Home` | scroll, as `Member`/`Context` do now |

  `c` does nothing anywhere. `expanded` is set to `false` whenever the
  screen changes (where `scroll` is reset), so leaving to `t` and coming
  back starts collapsed too.
- `pub fn expandable(document: &ResultDocument, index: usize) -> bool`:
  the member exists, has a design, and its mutation is `Insertion` with
  `written.len() > 30` (7i3's `MEMBER_INLINE_LIMIT` in `text.rs`; make it
  `pub(crate)` and use it).

### 2.2 `crates/cli/src/review/text.rs`

- `pub fn member_screen(document, index, template: Option<&[char]>,
  frame: Option<&MemberFrame>, expanded: bool) -> Vec<String>`: 7i3's
  `member_sequence(...)` lines, then, when the member has a design:

  ```
  (blank)
  forward   5'-<sequence>-3'  <length> nt  Tm <celsius, one decimal> C
  reverse   5'-<sequence>-3'  <length> nt  Tm <celsius, one decimal> C
  anneal at <annealing_temperature with {}> C
  ```

  the label padded to 10 as in 7i3, two spaces between fields, `Tm` from
  `primer.tm.celsius` with `{:.1}` (the owner approved this layout in the
  §5.3a mock-up; the full value stays on the technical screen). Then,
  when there is any warning, a blank line, `warnings:`, and one line per
  edit warning `  <code>` and per primer warning `  <role> <code>`
  (codes by 7c2's helpers; messages are on the technical screen). A
  design failure: after the block, `design failed: <code>` and `  <message>`.
  Every value is a document field (plan §3 rule 1, rule 3: warnings and
  failures are listed, by code).
- `overview`: drop the `operation:` line, the `  warnings: <n>` suffix of
  each member row (the row ends with its outcome), and the body lines
  `e problem` and `p provenance` and the blank lines before them (the key
  line already offers both). Keep the template and target lines, the
  `warnings:` block of operation warnings when there is one (rule 3), the
  `members: <n>` line and one row per member.
- `text::member` is unchanged; it is now the technical screen's text.
  `text::context` and its tests stay (pure and tested); nothing calls it
  from the view any more.

### 2.3 `crates/cli/src/review/view.rs`

- `Screen::Member(i)` draws `text::member_screen(document, i,
  app.template(), app.frame(i), app.expanded())`; `Screen::Technical(i)`
  draws `text::member(document, i)`.
- Key lines: `Member`: `q quit  esc back  t technical  j/k scroll`, then
  `  i expand` when `expandable` and not expanded, `  i collapse` when
  expanded. `Technical`: `q quit  esc back  j/k scroll`. The scroll
  position suffix is added as now.

## 3. Existing assertions that change (approved deviation)

Only these, each to the new behaviour; report each one you change:

- `crates/cli/src/review/app.rs` unit tests that press `c` or expect
  `Screen::Context`: press `t` and expect `Screen::Technical` instead.
- `crates/cli/tests/review_view.rs`: `every_screen_keeps_the_banner`'s
  context row becomes `technical`, keys `[Enter, 't']`,
  `Screen::Technical(0)`; `a_long_screen_scrolls_and_says_where` and
  `scrolling_past_the_end_stops_at_the_last_page` open with `[Enter,
  't', ...]`, expect `Screen::Technical(0)` and the key line
  `q quit  esc back  j/k scroll`.
- `crates/cli/tests/review_text.rs`:
  `the_overview_names_the_template_and_target` drops its
  `operation: variants` assertion; `each_member_has_one_row_and_one_marker`
  expects rows ending `designed`;
  `the_problem_line_appears_only_with_a_problem` is replaced by a test
  that the overview body never holds `e problem` (the key line offers it:
  `the_key_line_offers_problem_only_with_a_problem` stays as is).

If any other existing test fails, stop and report.

## 4. New tests

In `crates/cli/tests/review_view.rs` (use 7i3's request builders from
`crates/cli/tests/review_member.rs` by copying, not importing):

1. `enter_opens_the_sequence`: native-01; after `[Enter]` the body's
   first row is the first line of `member-native-01.txt` and the key line
   is `q quit  esc back  t technical  j/k scroll` (no `i`).
2. `t_opens_the_technical_lines_and_esc_returns`: native-01; `[Enter,
   't']` shows `text::member`'s first line; `Esc` returns to
   `Screen::Member(0)`.
3. `i_expands_a_long_insertion_and_reopening_collapses_it`: row
   `ins-13`; `[Enter]` key line ends `i expand`; `[Enter, 'i']` the 20
   body rows are the first 20 lines of `member-ins-13-expanded.txt`
   (each as `view` shows a line at width 80), key line ends `i
   collapse`; then `Esc`,
   `Enter`: collapsed again.
4. `i_does_nothing_on_a_short_edit`: native-01, `[Enter, 'i']`:
   `expanded()` is `false`.
5. `the_short_primer_lines_are_the_documents`: native-01; the
   `member_screen` lines after the block are `forward   5'-<the
   document's forward sequence>-3'  <its length> nt  Tm <{:.1} of its
   celsius> C`, the same for reverse, and `anneal at <{}> C`; values read
   from the document, never pasted.
6. `the_overview_is_members_only`: on the two-member set
   `review_text.rs` uses, the overview is exactly the template line, the
   target line, `members: 2` and two rows.

### 4.1 Rules that must be proven

1. Resetting `expanded` on a screen change: test 3 fails.
2. `i` limited to `expandable`: test 4 fails.
3. `t` opening the technical screen: test 2 fails.

## 5. Do not

- Change only `crates/cli/src/review/app.rs`, `view.rs`, `text.rs`,
  and the tests named in §3 and §4.
- No change to luria-core, `run.rs`, `review.rs`, `main.rs`, fixtures,
  docs, Cargo files or `vendor/`. No new dependency.
- Do not stage or commit.

## 6. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, each assertion changed under §3, the proofs of
§4.1, and anything that did not match.
