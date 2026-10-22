# Milestone 7m1: the review screens after the owner's hand test

Status: **Ready for the worker, 2026-10-22.** Planner: Claude. Parent:
`docs/tasks/milestone-7m-polish.md`, items M7-52 to M7-56 and M7-58's
keys (quoted in §1).

## 1. The owner's answers this contract applies

- M7-52: the Provenance screen goes. Its lines move to the bottom of
  Details, and the `p` key goes.
- M7-53: `?` shows the open screen's own keys, and the sequence legend
  only on the variant screen.
- M7-54: terminal colour names, mapped as follows. Edited bases are yellow
  bold, protein magenta, and annealing primer bases light blue. Primer
  tails are yellow, a stop and errors red bold, warnings yellow. Red and
  green are never the only difference between two things.
- M7-55: `nt` and `°C` are not coloured, and the annealing temperature is
  bold.
- M7-56: scrolling stops at the bottom on every scrolled screen, and the
  stale `c context` and `esc back` lines leave Details.
- M7-58 (keys only): on the export screen `w` or `o` writes and `esc`
  goes back. File names and overwrite are 7m2.

## 2. Changes

### 2.1 Scrolling stops at the bottom (`review/app.rs`, `review/proposal.rs`, `review/view.rs`, `review/terminal.rs`)

Today the app's `scroll` has no upper bound, and the view clamps only
what it draws (`first = scroll.min(len - rows)`). So after `j` past the
bottom, `k` has to undo every extra press before anything moves.

- `view.rs`: `pub fn max_scroll(app: &App, area: Rect) -> usize` and
  `pub fn proposal_max_scroll(app: &ProposalApp, area: Rect) -> usize`.
  Each gives `lines.len().saturating_sub(rows)` for the open screen, with
  the same lines and `rows` the draw uses: one shared private function,
  not a copy. Below the minimum size, both give 0.
- `App::clamp_scroll(&mut self, max: usize)` and
  `ProposalApp::clamp_scroll(&mut self, max: usize)` set `scroll` to
  `scroll.min(max)`.
- `terminal.rs`: after every key the app handles, both loops call
  `clamp_scroll` with the max for the terminal's current size
  (`terminal.size()`, as a `Rect` at 0,0).

### 2.2 Details (`review/text.rs`)

- `member()` no longer writes `c context` or `esc back`.
- The Details screen's lines are `text::member(document, index)`, then
  one empty line, then `text::provenance(document)`, unchanged. Add
  `pub fn details(document, index) -> Vec<String>` doing exactly that,
  and draw Details with it.

### 2.3 No Provenance screen (`review/app.rs`, `review/view.rs`)

Remove `Screen::Provenance`, the `p` key on every screen, and
`p provenance` from the overview's key line. `text::provenance` stays,
because Details uses it.

### 2.4 Help per screen (`review/text.rs`, `review/view.rs`, `review/app.rs`)

`text::help()` becomes `text::help(screen: Screen, problem: bool,
expandable: bool) -> Vec<String>`. The legend ("The sequence" and its
nine lines, then an empty line) comes first on `Member` only. Then
`Keys` and the open screen's keys, each as now: two spaces, the key
padded to 11, then the words.

| Screen | Keys, in order |
| --- | --- |
| Overview | `enter` open a variant; `j k` move; `PgUp PgDn` page; `e` the problem (only when `problem`); `x` export; `q` quit |
| Member | `d` details; `i` show or hide a long insertion (only when `expandable`); `j k` scroll; `PgUp PgDn` page; `esc` back; `q` quit |
| Details, Problem | `j k` scroll; `PgUp PgDn` page; `Home` top; `esc` back; `q` quit |
| Export | `w o` write the files; `esc` back; `q` quit |

`ctrl-c` is the last line on every screen: `  ctrl-c     quit`. `?` opens
help on every screen, Export included; `esc` or `?` closes it, as now.
The pop-up sizes itself to the lines, as now.

### 2.5 Export keys (`review/app.rs`, `review/view.rs`)

On `Export`: `w` or `o` gives `Action::Export` when there is a plan (as
`y` did), and `esc` or `Backspace` goes back to the overview. `y` and `n`
do nothing. The key line is `q quit  w write  esc back` with a plan, and
`q quit  esc back` without one, then `  ? help`.

### 2.6 Colours (`review/view.rs`)

Use `Color::` names only, never RGB:

| What | Style |
| --- | --- |
| template/bottom lower case (edited bases) | `Yellow` bold (as now) |
| protein letters | `Magenta` (was cyan) |
| protein `*` | `Red` bold (as now) |
| primer upper case (annealing) | `LightBlue` (was green) |
| primer lower case (tail) | `Yellow` (as now) |
| `[+N bp …]` marker | bold, no colour (was magenta bold, now the protein's colour) |

Warnings, errors and the state badges are unchanged.

**The primer table (M7-55).** `primer_table_line` rows start with
`forward` or `reverse`, so `row_label` takes them for sequence rows and
colours the `n` and `t` of `nt` and the `C` of `°C`. A primer table row
(its label, then `<n> nt`) is drawn with no colour. Its `{:.1} °C`
number is bold, its unit plain, and its sequence after the three spaces
is coloured as a primer row is.

## 3. Tests

Change existing assertions only as listed here. Each one changes only
because of a §2 rule:

- `p`/`Provenance` tests in `review/app.rs` and `tests/review_view.rs` /
  `tests/review_text.rs`: remove those about the screen; keep any that
  test `text::provenance`'s lines.
- `review_text.rs` 733, 905 and 920 (`c context`, `esc back`): now
  assert that neither line is in `member()`.
- The overview key line in `review_view.rs`: without `p provenance`.
- The export tests in `review/app.rs` (`y`, `n`): now `w`, `o` and `esc`.
  The test "`?` does nothing on the export screen" now asserts it opens
  help.
- Any test asserting `Color::Cyan` on protein or `Color::Green` on primers:
  the §2.6 colour.

New tests:

1. `review_view.rs`, `details_stops_scrolling_at_the_bottom`: on Details
   at 80x24, press `j` 200 times, clamping with `max_scroll` after each,
   as the loop does. Then one `k` moves the drawn body up one line.
2. `review_view.rs`, `details_end_with_the_provenance`: the Details body's
   last lines are `text::provenance`'s, and no `c context` line is
   drawn.
3. `review_text.rs`, `help_names_only_the_open_screens_keys`: the legend
   is on `Member` only, `x` is on Overview only, `w o` is on Export only,
   and `e` appears only when `problem`.
4. `review_view.rs`, `the_primer_table_colours_only_the_sequence`: on the
   del-11 member screen, the cells of `nt` and `°C` have no foreground
   colour, the Tm digits are bold, and an annealing base of the table's
   sequence is `LightBlue`.
5. `review/app.rs` unit test: `w` and `o` export, `esc` goes back.

### 3.1 Rules that must be proven

1. The clamp (remove the `clamp_scroll` call from the test's loop, i.e.
   test without it): test 1 fails.
2. The primer table rule (draw table rows as sequence rows again): test 4
   fails.

## 4. Do not

- Change only `crates/cli/src/review/{app,proposal,text,view,terminal}.rs`
  and the tests named in §3. `new/view.rs` uses `frame_block`, `key_line`
  and `too_small` from `review/view.rs`; keep their signatures.
- No change to a member fixture in `tests/fixtures/review/` (their text
  is unchanged; only styles change), luria-core, `new/`, docs, Cargo
  files or `vendor/`.
- Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the §3.1 proofs, and every existing assertion changed
with its §2 reason.

## 6. Planner answers to w7m1 (2026-10-22)

- `terminal.size()` does not exist in ratatui 0.30; reading the size with
  `terminal.backend().size()` and skipping the clamp when it cannot be read
  is accepted.
- Details has exactly one empty line between the member's lines and the
  provenance: `details()` adds the empty line only when `member()`'s last
  line is not already empty.
- Approved assertion changes beyond §3, each because Details is now
  `text::details` (§2.2) or help is per screen (§2.4):
  `tests/review_view.rs` `a_long_screen_scrolls_and_says_where`,
  `scrolling_past_the_end_stops_at_the_last_page` and
  `d_opens_the_details_and_esc_returns` count and compare against
  `text::details(&document, 0)`; `every_screen_keeps_the_banner` drops
  the provenance case; `the_body_is_the_text_of_the_screen` opens Details
  (`enter`, `d`) and compares with `text::details`; `help_is_a_pop_up`
  draws help on `Screen::Member(0)` with
  `text::help(Screen::Member(0), false, expandable)` and compares the rows
  outside the pop-up with that screen's body lines.
  `review/app.rs`'s `scrolling_moves_and_stops_at_zero` opening Details
  by `enter`, `d` is accepted.
