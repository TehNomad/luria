# Milestone 7a: `luria review REQUEST.json`, the skeleton

Status: **Implemented 2026-10-20** by the local worker; accepted on the
first review. Notes: ratatui 0.30's `restore()` returns nothing, so a failed
`try_init` returns its own error after `restore()`; `run` takes `mut app`;
rustfmt orders the `pub mod` lines alphabetically; `app.rs`'s unit tests
build a `ResultDocument` by hand, since `CARGO_TARGET_TMPDIR` exists only
for integration tests. The planner corrected one comment in
`review_view.rs` (the test that pins `edit-a42g.json`'s result).
`terminal.rs` is unverified until the owner runs `luria review` in a real
terminal. Planner: Claude. Parent:
`docs/tasks/contracts/milestone-7-tui.md` §3 (rules 1, 3, 7), §5.1, §6 (7a). Step
7a0 is done: `ratatui` 0.30 and `crossterm` 0.29 are already in
`crates/cli/Cargo.toml` and vendored.

## 1. What and why

The first runnable piece of the TUI:

```
luria review REQUEST.json
```

runs the request exactly as `luria q5 design` does, then opens a
full-screen terminal view showing the result's state and its experimental
validation, and quits on `q` or `Esc`. Every later screen (7b–7g) builds on
the module layout, the terminal session and the size rule set here.

Only the request form is in this contract. The interpret form
(`luria review --template …`) is 7e/7f; do not add any part of it.

## 2. Requirements this contract applies

- **Plan §3 rule 1 (no biology in the TUI):** the screen shows values from
  the `ResultDocument` only. The state and the experimental-validation line
  are the document's own `state` and `experimental_validation` strings,
  printed as they are, never a literal typed into the view.
- **Plan §3 rule 3:** every result screen shows the state and the
  experimental validation.
- **Plan §3 rule 7:** the terminal is restored on quit, on error and on
  panic.
- **Plan defaults (§9, approved):** the command is `luria review`; quitting
  exits 0; failures before the screen opens use the existing exit codes.
- **Usage errors, unchanged conventions (5d):** `luria: <message>`, a
  blank line, the usage text, all on stderr; nothing on stdout; exit 2; the
  first error found, left to right, is the one reported.

## 3. Changes

### 3.1 `crates/cli/src/args.rs`

1. `Command` gains a variant:

   ```rust
   Review { request: PathBuf },
   ```

2. `parse`: the word `review` goes to a new `parse_review(rest)`.
   `unknown command 'W'` is otherwise unchanged.

3. `parse_review(words)` reads its words left to right, like
   `parse_design` without `--write-edited`, first error wins:
   - a word exactly `--`, the first time, ends the options; it is not a
     path;
   - before `--`, any word starting with `-`: `unknown option 'W'`;
   - an empty word as the path: `an empty path is not a file`;
   - a second path `W`: `'review' takes one request file, got a second:
     'W'`;
   - no path at all: `'review' needs a request file`.

4. `USAGE` becomes exactly (trailing newline as now):

   ```
   usage:
     luria q5 design REQUEST.json [--write-edited DIR]
     luria q5 interpret --template FILE.gb (--text TEXT | --text-file FILE)
         --backend openai-compatible --endpoint URL --model NAME
         [--design PROFILE.json] [--allow-remote] [--write-request FILE.json]
     luria review REQUEST.json
     luria version
     luria help

   'q5 design' reads a Q5 site-directed mutagenesis request (JSON), runs the
   workflow and writes one JSON result document to stdout. 'q5 interpret'
   asks a language model what a request in words means and writes the
   proposed request, or its questions, as one JSON document. 'review' runs a
   request and shows its result in the terminal. See docs/specs/cli.md.
   ```

### 3.2 `crates/cli/src/review.rs` and `crates/cli/src/review/` (new)

Lay it out as `interpret.rs` and `interpret/` are. `lib.rs` gains
`pub mod review;`.

`review.rs` holds the module docs, `pub mod app; pub mod view; pub mod
terminal;`, and:

```rust
/// Why `luria review` cannot open its screen here, or `None` when it can
/// (plan §3 rule 7). Both stdin (keys) and stdout (the screen) must be
/// terminals. Decided before the request is read.
pub fn terminal_problem(stdin_is_terminal: bool, stdout_is_terminal: bool) -> Option<UsageError>;
```

It returns `None` only when both are `true`. Otherwise the message is
exactly: `'review' needs a terminal on stdin and stdout; 'q5 design' writes
the same result as JSON`.

`review/app.rs`: the state, pure, no I/O, no `ratatui` or `crossterm`
types in its public signatures:

```rust
/// A key, as the app sees it. `terminal` maps crossterm's events to it.
pub enum Key { Char(char), Esc, Other }

/// What the app asks its shell to do after a key.
pub enum Action { Continue, Quit }

pub struct App { /* the ResultDocument, private */ }

impl App {
    pub fn new(document: ResultDocument) -> App;
    pub fn document(&self) -> &ResultDocument;
    /// `q` and `Esc` quit; every other key continues.
    pub fn handle(&mut self, key: Key) -> Action;
}
```

Derive `Clone, Copy, Debug, PartialEq, Eq` on `Key` and `Action`, and
`Clone, Debug, PartialEq` on `App`.

`review/view.rs`: drawing, pure, no I/O:

```rust
/// The smallest screen the review draws on.
pub const MIN_WIDTH: u16 = 80;
pub const MIN_HEIGHT: u16 = 24;

/// Draw `app` into `frame`.
pub fn draw(frame: &mut ratatui::Frame, app: &App);
```

- If the frame's area is narrower than `MIN_WIDTH` or shorter than
  `MIN_HEIGHT`, draw only this line, top left, and nothing of the
  document: `the screen is WxH; luria review needs at least 80x24`, with
  `W` and `H` the area's width and height.
- Otherwise draw, top left, one line per item:
  - `state: <document.state>`
  - `experimental validation: <document.experimental_validation>`
  - and on the last row of the area: `q quit`.

  Nothing else. No colours or borders are required; if you use any, the
  text above must still appear exactly, on one row each.

`review/terminal.rs`: the shell, the only file with I/O:

```rust
/// Open the screen, draw `app` until it asks to quit, and restore the
/// terminal, whatever happens (plan §3 rule 7).
pub fn run(app: App) -> std::io::Result<()>;
```

- Open with `ratatui::try_init()` (it installs ratatui's panic hook, which
  restores the terminal on panic). If it fails, call `ratatui::restore()`
  and return its error.
- Loop: `terminal.draw(|frame| view::draw(frame, &app))`, then
  `crossterm::event::read()`. Only `Event::Key` with `KeyEventKind::Press`
  is a key: `KeyCode::Char(c)` is `Key::Char(c)`, `KeyCode::Esc` is
  `Key::Esc`, every other code is `Key::Other`. Other events (resize
  included) just redraw. Stop on `Action::Quit`.
- After the loop, or on any error inside it, call `ratatui::try_restore()`.
  Return the loop's error if there was one, else the restore's result.
- This file is not unit-tested (plan §5.1); keep it to the above.

### 3.3 `crates/cli/src/main.rs`

`Command::Review { request }` goes to a new `emit_review(request)`, which,
in order:
1. calls `terminal_problem(std::io::stdin().is_terminal(),
   std::io::stdout().is_terminal())` (`std::io::IsTerminal`); `Some` is
   `report_usage(problem)`, exit 2, before the request is read;
2. `run_with_output(&request, None)`; the outcome's `note` is ignored here,
   because nothing is written;
3. `review::terminal::run(App::new(outcome.document))`;
4. `Err(e)`: writes `luria: the terminal failed: <e>\n` on stderr with
   `write_err`, exit 1 (`ExitStatus::Internal`);
5. `Ok(())`: exit 0 (`ExitStatus::Success`), whatever the result's state.

Nothing else in `main.rs` changes.

## 4. Tests

All new; no existing test or fixture changes. A test that needs a result
document builds one with `luria_cli::run::run` on a request written into a
directory under `env!("CARGO_TARGET_TMPDIR")`, as `tests/end_to_end.rs`'s
`workspace` does.

### 4.1 `args.rs` unit tests (add to the existing `tests` module)

- `review r.json` → `Command::Review { request: "r.json" }`.
- `review -- -r.json` → request `-r.json`.
- Each message of §3.1 step 3, from: `review`; `review a.json b.json`;
  `review --write-edited d a.json` (unknown option `--write-edited`);
  `review ""`.

### 4.2 `review.rs` unit tests

- `review_needs_a_terminal_on_both_streams`: `(false, true)`, `(true,
  false)` and `(false, false)` give `Some` with the exact message;
  `(true, true)` gives `None`.

### 4.3 `review/app.rs` unit tests

- `q_and_esc_quit`: `Key::Char('q')` and `Key::Esc` give `Action::Quit`.
- `other_keys_continue`: `Key::Char('x')`, `Key::Char('Q')` and
  `Key::Other` give `Action::Continue`.

### 4.4 `crates/cli/tests/review_view.rs` (new)

Draw with `ratatui::Terminal::new(ratatui::backend::TestBackend::new(w,
h))` and read the screen as text: each row's cell symbols concatenated,
rows joined with `\n`.

Two documents:
- **verified**: copy `tests/fixtures/genbank/synthetic-linear.gb` and
  `tests/fixtures/nl/request/edit-a42g.json` into one directory and run the
  request. Assert first that the document's `state` is `"verified"` and its
  `experimental_validation` is `"not_performed"` (the CLI's
  `tests/end_to_end.rs` and `tests/fixtures/nl/README.md` already pin
  these).
- **failed**: `run` on a request path that does not exist in that
  directory. Assert first that its `state` is `"failed"`.

Tests:
- `the_state_and_validation_are_on_screen`: at 80x24, the verified screen
  has a row that is exactly `state: verified` once trailing spaces are
  trimmed, one that is `experimental validation: not_performed`, and the
  last row is `q quit`.
- `a_failed_result_shows_its_state`: at 80x24, the failed screen has a row
  `state: failed`, and no row containing `verified`.
- `a_small_screen_says_so_and_shows_nothing_else`: at 79x24 and 80x23, the
  first row is `the screen is 79x24; luria review needs at least 80x24`
  (respectively `80x23`), and no row contains `state:`.
- `the_screen_shows_the_document_not_a_literal`: take the verified
  document, set `state` to `"needs_clarification"` and
  `experimental_validation` to `"performed elsewhere"` (both are
  `&'static str`), draw it at 80x24, and assert rows `state:
  needs_clarification` and `experimental validation: performed elsewhere`.

### 4.5 `crates/cli/tests/cli.rs` (add)

- `review_without_a_terminal_is_a_usage_error`: run the binary
  (`env!("CARGO_BIN_EXE_luria")`) as `review missing.json` with
  `stdin(Stdio::null())` and stdout/stderr captured. Expect exit 2, empty
  stdout, and stderr exactly `luria: 'review' needs a terminal on stdin and
  stdout; 'q5 design' writes the same result as JSON\n\n` followed by
  `USAGE`. `missing.json` does not exist, so this also shows the check runs
  before the request is read.

### 4.6 Rules that must be proven

Disable each once, run the named tests, confirm they fail, restore, and
report what you saw:
1. `terminal_problem` always returning `None`:
   `review_needs_a_terminal_on_both_streams` and
   `review_without_a_terminal_is_a_usage_error` fail. (The binary test
   fails because crossterm cannot open a terminal in your shell; report
   the exit status you saw.)
2. The banner lines removed from `draw`:
   `the_state_and_validation_are_on_screen` fails.
3. The banner typing `verified` instead of reading `document.state`:
   `a_failed_result_shows_its_state` and
   `the_screen_shows_the_document_not_a_literal` fail.
4. The size check removed: `a_small_screen_says_so_and_shows_nothing_else`
   fails.

## 5. Do not

- Do not add dependencies or features, or edit any `Cargo.toml`,
  `Cargo.lock`, `.cargo/config.toml` or `vendor/`.
- Do not change files other than `crates/cli/src/args.rs`,
  `crates/cli/src/lib.rs`, `crates/cli/src/main.rs`,
  `crates/cli/src/review.rs`, `crates/cli/src/review/app.rs`,
  `crates/cli/src/review/view.rs`, `crates/cli/src/review/terminal.rs`,
  `crates/cli/tests/review_view.rs` and `crates/cli/tests/cli.rs` (adding
  the one test only).
- Do not change existing tests or fixtures, `docs/`, or `luria-core` or
  `luria-nl`.
- Do not add the interpret form of `review`, export, member or primer
  screens, colours as meaning, or mouse support.
- Do not run the binary's `review` command in a way that needs a terminal;
  your shell has none. Only the tests above.
- Do not stage or commit.

## 6. Checks

From the repository root:

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each command's result, the four proofs of §4.6, and any point where
this contract did not match the code (for example a ratatui 0.30 API that
differs from the names above); stop and report rather than guess.
