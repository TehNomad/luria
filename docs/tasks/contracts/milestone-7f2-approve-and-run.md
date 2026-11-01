# Milestone 7f2: review a proposal, approve it, and run it

Status: **Implemented 2026-10-21** by two local workers (the first retired
after steps 1–4 at context pressure); accepted on the first review.
Additions accepted: `proposal::approvable`, the one rule `handle`, the
key line and `approve` share; derives on `Approved`. Proof 3 differs:
`approve` re-checks the location first, so `File::create` alone is never
reached and test 6 passes; with the re-check also removed, test 6 fails.
`write_request_file`'s atomic `create_new` guards the race between check
and write, which no test reaches. `interpret`'s exit status is ignored
here: a failed interpretation opens on `proposal: failed`, and quitting
exits 0. The terminal loop and `emit_review_interpret` are verified by
compilation only, until the owner runs them.
Planner: Claude. Parent: [docs/tasks/contracts/milestone-7-tui.md](/tasks/contracts/milestone-7-tui.md) §1, §3 rules 3,
4 and 6, M7-1, M7-3, §6 (7f). Uses 7e's `text::proposal` and 7f1's
`request_location_problem`, `request_text` and `write_request_file`.

## 1. What and why

The second entry point of M7-1:

```
luria review --template FILE.gb (--text TEXT | --text-file FILE)
    --backend openai-compatible --endpoint URL --model NAME
    --request FILE.json [--design PROFILE.json] [--allow-remote]
```

It interprets the text, shows the proposal (7e's lines), and on the
user's explicit approval writes the request to `--request` and designs
from that file, then shows the result on the screens of 7a–7d. Quitting
before approval writes nothing and runs nothing.

`--request` is required, so the place the request will be written is
stated before anything happens, and is checked before the model is
called (plan §3 rule 4).

## 2. Requirements this contract applies

- **Plan §3 rule 4 (no silent action):** only `y` on the confirmation
  screen, reached by `a` from a `proposed` proposal, writes or runs
  anything. The confirmation names the file. The design runs **from the
  written file** (`run_with_output(path, None)`), so the result's request
  digest names a file on disk.
- **M7-3:** the TUI edits nothing; questions are read-only. To change the
  request, the user quits and runs again with other text or profile.
- **Plan §3 rule 6:** the remote gate, the key rule and the HTTP-only
  backend are `q5 interpret`'s, unchanged (`checked_endpoint`,
  `checked_key`, `LURIA_OPENAI_API_KEY`). The model is called once;
  approving does not call it again.
- **Plan §3 rule 3:** a proposal is never shown as a result: its banner
  says so.
- **Order of checks before the screen opens** (each failure leaves the
  screen closed): arguments (exit 2); endpoint and key (exit 2);
  terminals (7a's `terminal_problem`, exit 2); the request location,
  `request_location_problem(template, request)` (`luria: <message>` on
  stderr, exit 4); then `interpret` with `write_request: None`.

## 3. Changes

### 3.1 `crates/cli/src/args.rs`

`Command` gains:

```rust
ReviewInterpret {
    template: PathBuf,
    text: TextSource,
    design: Option<PathBuf>,
    endpoint: String,
    model: String,
    allow_remote: bool,
    request: PathBuf,
},
```

`parse_review(words)`: when any word is exactly `--template`, the words
are read as the interpret form, by the same rules and messages as
`parse_interpret` (factor the shared part out rather than copy it), with
two differences: `--write-request` is not an option (`unknown option
'--write-request'`), and `--request` is a path option, required, whose
absence is `'review' needs --request`. Messages that name the command say
`'review'` where `parse_interpret` says `'q5 interpret'`. Otherwise
`parse_review` is 7a's request form, unchanged.

`USAGE` gains, after `luria review REQUEST.json`:

```
  luria review --template FILE.gb (--text TEXT | --text-file FILE)
      --backend openai-compatible --endpoint URL --model NAME
      --request FILE.json [--design PROFILE.json] [--allow-remote]
```

and the sentence about `review` becomes: `'review' runs a request, or
interprets a request in words and runs it once you approve, and shows the
result in the terminal.`

### 3.2 `crates/cli/src/review/proposal.rs` (new; `review.rs` gains `pub mod proposal;`)

Pure, no I/O:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProposalScreen { Reading, Confirm }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProposalAction { Continue, Quit, Approve }

#[derive(Clone, Debug, PartialEq)]
pub struct ProposalApp { /* document, request path, screen, scroll, failure: Option<String> */ }

impl ProposalApp {
    pub fn new(document: InterpretationDocument, request: PathBuf) -> ProposalApp;
    pub fn document(&self) -> &InterpretationDocument;
    pub fn request(&self) -> &Path;
    pub fn screen(&self) -> ProposalScreen;
    pub fn scroll(&self) -> usize;
    /// The last approval's failure, shown on the reading screen.
    pub fn failure(&self) -> Option<&str>;
    pub fn approval_failed(&mut self, message: String);
    pub fn handle(&mut self, key: Key) -> ProposalAction;
}
```

`handle` (`Key` is `app::Key`):
- any screen, `Char('q')`: `Quit`;
- `Reading`: `Esc` quits; `Char('a')` opens `Confirm` **only when the
  document's `state` is `proposed` and `request` is `Some`**, otherwise
  nothing; `Down`/`j`, `Up`/`k`, `PageDown`, `PageUp`, `Home` scroll as
  7c3's do;
- `Confirm`: `Char('y')` returns `Approve` (the screen stays `Confirm`
  until the shell reports); `Char('n')`, `Esc` and `Backspace` go back to
  `Reading`;
- every other key: nothing, `Continue`. A change of screen resets the
  scroll. `approval_failed` sets `failure` and returns to `Reading`.

### 3.3 `crates/cli/src/review/view.rs`

```rust
pub fn draw_proposal(frame: &mut ratatui::Frame, app: &ProposalApp);
```

The same size rule and row layout as `draw` (banner rows 0–1, blank row
2, body, key line on the last row, the same scrolling and `lines a-b of
n`). Banner: `proposal: <state>` and `nothing has run yet: this is a
proposal, not a result`. Body:
- `Reading`: when `failure` is `Some(m)`, first `approval failed: <m>`
  and a blank line; then `text::proposal(document)`.
- `Confirm`: exactly `write the request to <request path, as given> and
  run luria q5 design on it?`, a blank line, `y yes  n no`.

Key line: `Reading`: `q quit  j/k scroll`, followed by `  a approve` only
when `a` would open `Confirm`; `Confirm`: `q quit  y yes  n no`.

### 3.4 `crates/cli/src/review.rs`

```rust
/// What an approved proposal gave: the result and, for 7d, the template's
/// bases when they match the result.
pub struct Approved { pub document: ResultDocument, pub template: Option<Vec<char>> }

/// Write the proposed request at `request` and run it, as `luria q5
/// design` would run that file. Refuses, writing nothing, when the
/// document is not `proposed`, has no request, or the location is not
/// usable now (7f1's rules, checked again: the file may have appeared
/// since). The `Err` is one line for the screen.
pub fn approve(document: &InterpretationDocument, template: &Path, request: &Path) -> Result<Approved, String>;
```

In order: state and request check (`Err("there is no proposed request
to approve")`); `request_location_problem(template, request)` → `Err(<its
message>)`; `write_request_file(request, &request_text(echo))` → `Err("
could not write <file name>: <reason>")`; then
`run_with_output(request, None)` and `prepare(request)` +
`matching_template`, as 7d's `emit_review` does; `Ok(Approved { .. })`.

### 3.5 `crates/cli/src/review/terminal.rs`

```rust
/// Show the proposal until the user quits (`Ok(None)`) or an approval
/// succeeds (`Ok(Some(approved))`). `approve` is called on each
/// `ProposalAction::Approve`; its `Err` is shown and the user may try
/// again or quit. The terminal is restored before returning.
pub fn run_proposal(app: ProposalApp, approve: &mut dyn FnMut(&ProposalApp) -> Result<Approved, String>) -> std::io::Result<Option<Approved>>;
```

Same session handling, key mapping and restore rules as `run`.

### 3.6 `crates/cli/src/main.rs`

`Command::ReviewInterpret` → `emit_review_interpret`, in §2's order. Then
`terminal::run_proposal(ProposalApp::new(outcome.document, request),
&mut |app| review::approve(app.document(), &template, app.request()))`.
`Ok(None)` → exit 0. `Ok(Some(approved))` → `terminal::run(App::new(
approved.document).with_template(approved.template))`, then 7a's exit
rules. Any terminal `Err` → 7a's `luria: the terminal failed: <e>`, exit
1. `interpret`'s own `note` is ignored (it writes nothing here).

## 4. Tests

### 4.1 `args.rs` unit tests

- The full interpret form parses to `ReviewInterpret` with each field.
- `review --template t.gb --text x --backend openai-compatible
  --endpoint http://127.0.0.1:1/v1 --model m` (no `--request`): `'review'
  needs --request`.
- `--write-request r.json` in the interpret form: `unknown option
  '--write-request'`.
- `review r.json` still parses to `Review` (7a's tests pass unchanged).

### 4.2 `proposal.rs` unit tests (hand-built documents)

1. `a_opens_confirm_only_for_a_proposal`: `proposed` with a request:
   `a` → `Confirm`; `needs_clarification`: `a` → `Reading`; `proposed`
   with `request: None`: `Reading`.
2. `y_approves_and_n_goes_back`.
3. `q_quits_everywhere_and_esc_quits_only_when_reading`.
4. `a_failure_returns_to_reading_and_is_kept`.

### 4.3 `crates/cli/tests/review_approve.rs` (new)

Corpus cases through the scripted backend, as 7e's tests run them, with
`write_request: None`.

5. `approving_writes_the_request_and_designs_from_it`: case
   `two-separate`. `approve` → `Ok`. The written file's text equals
   `request_text` of the document's request; the `Approved.document`
   rendered with `result::render` equals the rendering of
   `run(request)` on that same file (the result `luria q5 design` would
   print), and its `state` is `verified`; `template` is `Some`.
6. `approving_never_replaces_a_file`: the request path already holds a
   file: `Err` naming it; the file's content is unchanged; no result.
7. `a_clarification_cannot_be_approved`: case `composition-unstated`:
   `Err("there is no proposed request to approve")`, and no file exists
   at the path.
8. `quitting_before_approval_writes_nothing`: build a `ProposalApp` for
   `two-separate`, send `q`: `Quit`, and no file exists at the path.

### 4.4 `crates/cli/tests/review_view.rs` (add)

9. `a_proposal_says_it_is_not_a_result`: rows 0 and 1 are the proposal
   banner, built from the document's state.
10. `the_confirmation_names_the_file`.
11. `approve_is_offered_only_for_a_proposal`: the key line has `a
    approve` for `two-separate`, not for `composition-unstated`.

### 4.5 `crates/cli/tests/cli.rs` (add)

12. `review_interpret_checks_the_request_location_first`: with stdin not
    a terminal the binary exits 2 before anything (7a's rule). (The
    location check cannot be reached from a test without a terminal;
    `approve`'s own checks are tests 6 and 7.)

### 4.6 Rules that must be proven

1. `a` opening `Confirm` for any state: tests 1 and 11 fail.
2. `approve` skipping the state check: test 7 fails.
3. `approve` using `File::create` instead of `write_request_file`: test 6
   fails.
4. `approve` running the echo in memory instead of the written file
   (for example by not writing first and running a temporary copy): test
   5's file assertion fails.

## 5. Do not

- Change only `args.rs`, `main.rs`, `review.rs`, `review/proposal.rs`,
  `review/view.rs`, `review/terminal.rs`, and the tests named in §4
  (adding tests; 7a's tests stay unchanged).
- Do not change `interpret/`, `luria-nl`, `text.rs`, fixtures, [docs/](/),
  any `Cargo.toml` or `vendor/`.
- Do not call a real model in a test; do not run `luria review` by hand.
- Do not add dependencies. Do not stage or commit.

## 6. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the proofs of §4.6, and any point where this contract
does not match the code; stop and report rather than guess.
