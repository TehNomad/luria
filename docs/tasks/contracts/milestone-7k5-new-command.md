# Milestone 7k5: the `luria new` command

Status: **Implemented 2026-10-22.** Proofs 2 and 3 failed as required;
proof 1 fails only with both the location check and `create_new`
disabled, since the location check refuses first. Accepted:
`run_new(form, cwd, &mut written)` returns the request path for the
export plan; a failure returns to the Reading screen, as 7f2 does.
Known gaps: without a terminal, `luria new` prints the review's message
(it names `'review'`), to be fixed in a follow-up; the GenBank path and an
already-taken paste file name are untested; the terminal loop is the
owner's hand test. Planner: Claude. Parent:
`docs/tasks/contracts/milestone-7k-new.md` §1, §2 (rules 4 to 6), M7-31, M7-37.
Builds on 7k2 (`new::build`), 7k3 (`NewForm`), 7k4 (`draw_form`) and the
review's proposal pattern (7f2: nothing written before `a` then `y`).

Scope: the **shorthand** path end to end. The "In words" box stays
disabled in this contract whatever the command line says; free text with
`--backend` is contract 7k7, so this one stays reviewable.

## 1. What the user does

```
luria new [--design PROFILE.json]
```

The form opens (7k3/7k4). On `[ Review ]` with a valid form, a
confirmation screen shows the request exactly as it will be written and
the files it will write. `a` then `y` writes them and runs the design; the
result opens on the review screens (7i/7j). `--export DIR` works as in
`luria review`. Quitting at any point before `y` writes nothing.

## 2. Changes

### 2.1 `crates/cli/src/args.rs`

`Command::New { design: Option<PathBuf>, export: Option<PathBuf> }`,
parsed from `new [--design FILE] [--export DIR]`; any other word is a
usage error naming it. `USAGE` gains, after the two `luria review` lines,
`  luria new [--design PROFILE.json] [--export DIR]`, and its prose one
sentence: `'new' takes a template and mutations in a form and runs them
once you approve.`

### 2.2 The confirmation screen, `crates/cli/src/new/confirm.rs`

```rust
pub struct Planned {
    /// Where each file goes: for a template file, the request beside it
    /// (M7-37, as `--write-request` requires); for a paste, the pasted
    /// FASTA and the request both in the current directory.
    pub request: PathBuf,
    pub paste: Option<(PathBuf, String)>,
    pub built: new::Built,
}

pub fn plan(built: new::Built, template_dir: Option<&Path>, cwd: &Path, save_as: &str) -> Planned;

pub enum ConfirmScreen { Reading, Confirm }
pub struct ConfirmApp { /* private: planned, screen, scroll, failure */ }
impl ConfirmApp {
    pub fn new(planned: Planned) -> ConfirmApp;
    pub fn handle(&mut self, key: Key) -> ConfirmAction; // Continue, Back, Quit, Approve
    pub fn failed(&mut self, message: String);
}
pub fn confirm_lines(app: &ConfirmApp) -> Vec<String>;
pub fn draw_confirm(frame: &mut Frame, app: &ConfirmApp);
```

- `confirm_lines` (Reading): `This request will be written:`, the file's
  path, a blank line, the request exactly as `interpret::run::request_text`
  of `result::request_echo` gives it (plan §3 rule 2), then, for a paste,
  a blank line, `and the template:` and its path. Then any failure line.
- Keys (Reading): `a` → Confirm; `esc` → `Back` (to the form, its values
  kept); `q` → Quit; `j`/`k`/PageUp/PageDown scroll. (Confirm): `y` →
  `Approve`; `n`/`esc` → Reading.
- `draw_confirm`: 7j2's frame with the badge ` PROPOSAL `, the confirm
  question on the Confirm screen naming every file, as 7f2's does.

### 2.3 Writing and running, `crates/cli/src/new/confirm.rs`

```rust
/// Write the paste (if any), then the request, each as a new file never
/// replacing one (`interpret::run::write_request_file`), then run the
/// design from the request file exactly as `review::approve` does after its
/// write, giving `review::Approved`.
pub fn approve_new(planned: &Planned) -> Result<Approved, String>;
```

- Before writing: for a template file, `request_location_problem(template,
  request)` must be `None`. A paste and its request share the current
  directory, so the check passes by construction; still run it.
- If the paste is written and the request then fails, the message names
  the paste file as written (nothing is deleted).
- The design half: move `review::approve`'s code after its write into a
  `pub(crate) fn run_approved(request: &Path) -> Approved` in
  `review.rs`, used by both; `approve`'s behaviour and tests unchanged.

### 2.4 The loop, `crates/cli/src/new/terminal.rs`, and `main.rs`

`pub fn run_new(form: NewForm, ...) -> io::Result<Option<Approved>>`:
one terminal session (as `review::terminal::run_proposal`), drawing the
form, or the confirmation once `Review(built)` is returned; it answers
`Load(path)` by reading the file (relative to the current directory) and
calling `form.loaded`; it enables bracketed paste and passes
`Event::Paste(text)` to `form.paste`; on `Approve` it calls `approve_new`,
showing a failure on the confirmation screen. `main.rs`: `Command::New`
checks the terminal (7a's rule), reads `--design` with
`request::parse_profile` (a refusal is exit 4 with its message, before the
screen opens), runs `run_new`, and on `Some(approved)` opens
`review::terminal::run` with `approved` and the export plan, as
`emit_review_interpret` does.

## 3. Tests

`crates/cli/tests/new_confirm.rs` (new), in a fresh directory under
`CARGO_TARGET_TMPDIR`, with the fixtures' templates copied in:

1. `the_confirmation_shows_the_request_as_written`: build the
   `sid1-codon` form (7k3's helpers); `confirm_lines` holds, after the
   path and a blank line, exactly the lines of the request text, and that
   text parsed as JSON equals `request-sid1-codon.json`.
2. `approving_writes_the_request_beside_the_template_and_runs_it`:
   `approve_new` writes `<dir>/sid-1.request.json` with exactly the shown
   text, and `approved.document.state` is `verified` with one member
   `D404N` designed.
3. `approving_a_paste_writes_both_files`: the `syn-paste` form with cwd =
   the test directory: `syn.fa` equals `paste-syn-paste.fa`, the request
   equals `request-syn-paste.json` as JSON, and the run is `verified`.
4. `an_existing_file_is_never_replaced`: with `sid-1.request.json`
   already present (any content): `approve_new` is `Err` naming it, and
   the file's content is unchanged.
5. `quitting_writes_nothing`: `ConfirmApp` from the sid1 form; `q`
   returns `Quit` and the directory holds only the copied template.
6. `esc_returns_to_the_form`: `esc` on Reading returns `Back`; `a` then
   `n` returns to Reading; only `a` then `y` gives `Approve`.
7. In `crates/cli/tests/cli.rs` style: `luria new --bogus` is a usage
   error naming `--bogus`; `luria new` without a terminal is 7a's usage
   error.

### 3.1 Rules that must be proven

1. Never replacing a file: test 4 fails.
2. `y` only after `a`: test 6 fails.
3. The request beside the template (write it in cwd instead): test 2
   fails.

## 4. Do not

- Change only `args.rs`, `main.rs`, `review.rs` (the `run_approved`
  extraction), `new.rs` (`pub mod` lines), the new `new/confirm.rs` and
  `new/terminal.rs`, the new test file, and `cli.rs` tests for test 7.
- No change to `new::build`, `NewForm`, the review screens, luria-core,
  fixtures, docs, Cargo files or `vendor/`. No free text (7k7).
- Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

The terminal loop is checked by the owner by hand: `luria new` on
`sid-1_cDNA.fa` with `D404N`, then `a`, `y`, and the result screens.
