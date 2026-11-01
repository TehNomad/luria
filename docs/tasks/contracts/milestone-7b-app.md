# Milestone 7b: the review state machine

Status: **Implemented 2026-10-21** by the local worker; accepted on the
first review. Notes: disabling `new`'s problem rule also fails tests 6, 7
and 9; `terminal.rs`'s new key mapping is unverified until run in a real
terminal. Planner: Claude. Parent:
[docs/tasks/contracts/milestone-7-tui.md](/tasks/contracts/milestone-7-tui.md) §5.1 (`review::app`, pure), §5.2 (the
result screens 2 to 6), §6 (7b). Builds on 7a
([docs/tasks/contracts/milestone-7a-review-skeleton.md](/tasks/contracts/milestone-7a-review-skeleton.md), implemented).

## 1. What and why

7a's `App` holds a result document and knows one rule: `q` and `Esc` quit.
The result screens of plan §5.2 (overview, member detail, sequence context,
provenance, problem) need a state machine that says which screen is open,
which member is selected, and what each key does. This contract builds it,
in `crates/cli/src/review/app.rs` only, pure and tested. It draws nothing
new: the screens are 7c and 7d, which read `App::screen()`.

## 2. Requirements this contract applies

- **Plan §5.1:** `review::app` is pure: no I/O, and no `ratatui` or
  `crossterm` type in a public signature.
- **Plan §3 rule 1:** the app reads only the `ResultDocument`; it computes
  nothing scientific. The only facts it uses are how many entries
  `results` has, whether `error` is `Some`, and whether `clarifications`
  is empty.
- **Plan §3 rule 3:** a design failure, a clarification or an error is
  never hidden: when the document has an error or clarifications, the
  first screen is the problem screen.

## 3. Changes, all in `crates/cli/src/review/app.rs`

### 3.1 `Key`

```rust
pub enum Key { Char(char), Up, Down, Enter, Backspace, Esc, Other }
```

Same derives as now.

### 3.2 `Screen` (new)

```rust
/// Which screen is open. A member index is an index into
/// `document.results`, always in range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Overview,
    Member(usize),
    Context(usize),
    Provenance,
    Problem,
}
```

### 3.3 `App`

Fields, private: the document, the open `Screen`, and `selected: usize`
(the member highlighted on the overview).

```rust
impl App {
    pub fn new(document: ResultDocument) -> App;
    pub fn document(&self) -> &ResultDocument;
    pub fn screen(&self) -> Screen;
    /// The member highlighted on the overview; 0 when there are none.
    pub fn selected(&self) -> usize;
    pub fn handle(&mut self, key: Key) -> Action;
}
```

`new`: `selected` is 0. The screen is `Problem` when `document.error` is
`Some` or `document.clarifications` is not empty; otherwise `Overview`.

`handle`, where `n` is `document.results.len()` and "back" is the screen
named in the last column:

| Screen | Key | Effect | Action |
| --- | --- | --- | --- |
| any | `Char('q')` | none | `Quit` |
| `Overview` | `Esc` | none | `Quit` |
| `Overview` | `Up` or `Char('k')` | `selected` decreases by 1, not below 0 | `Continue` |
| `Overview` | `Down` or `Char('j')` | `selected` increases by 1, not above `n - 1`; nothing when `n == 0` | `Continue` |
| `Overview` | `Enter` | `Member(selected)` when `n > 0`; nothing when `n == 0` | `Continue` |
| `Overview` | `Char('p')` | `Provenance` | `Continue` |
| `Overview` | `Char('e')` | `Problem` when `error` is `Some` or `clarifications` is not empty; nothing otherwise | `Continue` |
| `Member(i)` | `Char('c')` | `Context(i)` | `Continue` |
| `Member(i)` | `Esc` or `Backspace` | back to `Overview`, `selected` stays `i` | `Continue` |
| `Context(i)` | `Esc` or `Backspace` | back to `Member(i)` | `Continue` |
| `Provenance` | `Esc` or `Backspace` | back to `Overview` | `Continue` |
| `Problem` | `Esc` or `Backspace` | back to `Overview` | `Continue` |
| `Problem` | `Char('p')` | `Provenance` | `Continue` |

Every other key on every screen changes nothing and continues. Selection
does not wrap. `Q` (upper case) is not quit.

### 3.4 `terminal.rs`

The only change outside `app.rs`: the key mapping also gives `KeyCode::Up`
→ `Key::Up`, `KeyCode::Down` → `Key::Down`, `KeyCode::Enter` →
`Key::Enter`, `KeyCode::Backspace` → `Key::Backspace`. Nothing else in
`terminal.rs` changes.

## 4. Tests, in `app.rs`'s `tests` module

`q_and_esc_quit` changes: `Esc` quits only on the overview. Replace its
body with: on a fresh overview app, `Char('q')` quits; on another fresh
overview app, `Esc` quits. `other_keys_continue` stays as it is.

The test helper `document()` stays. Add a helper that returns it with `n`
entries in `results`. Build each entry as a `MemberView` with any
`MutationView` you like (for example a `Substitution` with empty strings
and zeros), `warnings` empty, `design` and `design_failure` `None`: the
app never reads a member's content, and nothing here is a scientific
value. Add a helper that returns the document with `error: Some(..)` (any
`ErrorView`, with `details: Details(vec![])`, `Details` being
`crate::result::Details`), and one with one clarification string.

New tests:
1. `the_first_screen_is_the_overview_without_a_problem`: `n = 2`, no
   error, no clarifications → `Overview`, `selected() == 0`.
2. `an_error_opens_on_the_problem_screen` and
   `a_clarification_opens_on_the_problem_screen`.
3. `selection_moves_and_stops_at_the_ends`: `n = 3`. `Up` at 0 stays 0;
   `Down` ×5 gives 2; `Char('k')` gives 1; `Char('j')` gives 2.
4. `selection_does_nothing_without_members`: `n = 0`, `Down` and `Enter`
   leave `selected() == 0` and `Overview`.
5. `enter_opens_the_selected_member_and_back_keeps_it`: `n = 3`, `Down`,
   `Enter` → `Member(1)`; `c` → `Context(1)`; `Esc` → `Member(1)`;
   `Backspace` → `Overview` with `selected() == 1`.
6. `provenance_opens_and_closes`: from the overview and from the problem
   screen, `p` → `Provenance`, `Esc` → `Overview`.
7. `e_opens_the_problem_screen_only_when_there_is_one`: no problem → `e`
   leaves `Overview`; with an error, `Esc` from `Problem` gives
   `Overview`, then `e` gives `Problem`.
8. `q_quits_from_every_screen`: for each of the five screens, reached by
   keys from a fresh app, `Char('q')` returns `Quit`.
9. `esc_goes_back_rather_than_quitting_below_the_overview`: on `Member`,
   `Context`, `Provenance` and `Problem`, `Esc` returns `Continue`.

### 4.1 Rules that must be proven

Disable each once, run the tests, confirm the named test fails, restore,
and report what you saw:
1. `new` always opening `Overview`: test 2 fails.
2. The clamp on `Down` removed (`selected += 1` without the bound): test 3
   fails.
3. `Esc` on `Member` quitting instead of going back: test 9 fails.
4. `Backspace` from `Member` resetting `selected` to 0: test 5 fails.

## 5. Do not

- Do not change files other than `crates/cli/src/review/app.rs` and the
  key mapping in `crates/cli/src/review/terminal.rs`.
- Do not change `view.rs`, `main.rs`, `args.rs`, any test file outside
  `app.rs`, any fixture, [docs/](/), any `Cargo.toml` or `vendor/`.
- Do not draw any new screen, or read any member field in `app.rs`.
- Do not add dependencies. Do not stage or commit.
- Your shell has no terminal: do not run `luria review` by hand.

## 6. Checks

From the repository root:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each command's result, the four proofs of §4.1, and any point
where this contract does not match the code; stop and report rather than
guess.
