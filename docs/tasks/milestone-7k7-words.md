# Milestone 7k7: `luria new`'s "In words" box, through the model

Status: **Implemented, 2026-10-22.** Planner: Claude. Parent:
`docs/tasks/milestone-7k-new.md` (owner answers M7-33, M7-36, M7-40).
Builds on 7k3 to 7k5, on milestone 6's `interpret`, and on 7f2's proposal
screen, which it calls unchanged. Also fixes 7k5's known gap: `luria new`
without a terminal names `'review'`.

## 1. What and why

M7-40 (owner, 2026-10-22): words and shorthand are **either one, not
both**. When the "In words" box holds text, `[ Review ]` sends it through
`interpret` exactly as `luria review --template … --request …` does, and
the existing proposal screen (7e/7f2) shows the model's reading,
questions and quotes; `a`, `y` writes the request and runs it with
`review::approve` (7f1's file rules). The interpret path reads **GenBank**
templates only (M6-7: the model is shown the record's CDS list), so words
need a GenBank file.

## 2. Changes

### 2.1 `crates/cli/src/args.rs`

- `Command::New` gains `backend: Option<NewBackend>`, with
  `pub struct NewBackend { pub endpoint: String, pub model: String,
  pub allow_remote: bool }`.
- `new` accepts `--backend openai-compatible --endpoint URL --model NAME
  [--allow-remote]` with the interpret form's rules for these four
  options (`--backend`'s only value, repeated-option errors, empty
  values). All three of `--backend`, `--endpoint`, `--model`, or none,
  else a usage error naming the first one missing; `--allow-remote`
  without `--backend` is a usage error.
- `USAGE`'s `new` line becomes two lines:

  ```
    luria new [--design PROFILE.json] [--export DIR]
        [--backend openai-compatible --endpoint URL --model NAME [--allow-remote]]
  ```

### 2.2 The form, `crates/cli/src/new/form.rs`

- `FormAction` gains `Interpret { template: String, text: String }`:
  `template` is the loaded file's path as typed in `File`, `text` the
  words as typed.
- In `review()`, after the form's own blockers and only when `words()` is
  not blank (whitespace only counts as blank), in this order: a non-blank
  mutation line is a blocker on `Words`, message `write the mutations
  either in words or as lines, not both`; a template that is not a loaded
  GenBank file is a blocker on `Words`, message `words need a GenBank
  template, whose CDS list the model is shown`. Otherwise return
  `Interpret`, without calling `new::build`. When `words()` is blank,
  behaviour is unchanged.

### 2.3 The text, `crates/cli/src/new/text.rs`

With a backend, `In words` shows the text like `Mutations` (multi-line,
label on the first line). When the template is not a loaded GenBank file
and the box holds text, one more row follows it: 11 spaces then `needs a
GenBank template` (dim in the view, as `needs --backend` is: one rule
added to `new/view.rs`'s `styled_line` for a continued row starting with
that text, approved on w7k7's question, 2026-10-22, with a test
`a_genbank_need_is_dim` in `crates/cli/tests/new_view.rs`).

### 2.4 The loop, `crates/cli/src/new/terminal.rs`

`run_new` gains `interpret: &mut dyn FnMut(&str, &str) ->
InterpretationDocument` (template path, words). On
`FormAction::Interpret`, it calls it once and shows the proposal in the
same session: `review::proposal::ProposalApp::new(document, request)`
drawn with `review::view::draw_proposal`, where `request` is the path
`confirm::plan` would give for that template and the form's `SaveAs`.
`ProposalAction::Approve` calls `review::approve(document, template,
request)`; its `Ok(approved)` ends the loop as 7k5's approval does
(setting `written` to `request`), its `Err` is shown with
`approval_failed`. `ProposalAction::Quit` (`q` or `esc`) returns to the
form with its values kept; the model is not called again until the next
`[ Review ]`.

### 2.5 `crates/cli/src/main.rs`

- With `backend`: `checked_endpoint` and `checked_key` exactly as
  `emit_review_interpret` does, **before** the terminal check and before
  anything is read; a refusal is the same usage error. Build
  `OpenAiCompatible` once. The closure runs `interpret(&InterpretInput {
  template, text: TextSource::Inline(words), profile: design,
  write_request: None }, &mut backend)` and returns its `document`.
  `NewForm::new(profile, true)`.
- Without `backend`: unchanged; the closure is never called (pass one
  that panics with "no backend").

### 2.6 The terminal message, `crates/cli/src/review.rs`

`terminal_problem` gains a first parameter `command: &str`; its message
becomes `'<command>' needs a terminal on stdin and stdout; 'q5 design'
writes the same result as JSON`. `review`'s two callers pass `"review"`
(message unchanged); `new` passes `"new"`. The one `cli.rs` assertion
7k5 added for `new` changes to `'new'` (approved).

## 3. Tests

Copy the corpus helpers (`NL`, `GENBANK`, `read`, `cases`, `row_of`,
`workspace`, `text_of`) from `crates/cli/tests/review_proposal.rs`, and
the form helpers from `crates/cli/tests/new_form.rs`. Expected values come
from `tests/fixtures/nl/` (case `two-separate`: template
`synthetic-linear.gb`, text `text/two-separate.txt`, reply
`reply/two-separate.json`, profile `profile/full.json`, expected request
`request/variants-separate.json`), never pasted.

`crates/cli/tests/new_words.rs` (new):

1. `words_with_a_genbank_file_ask_to_interpret`: a form with a backend,
   `synthetic-linear.gb` loaded, the case's text typed into `Words`, no
   mutation lines, `[ Review ]`: the action is `Interpret` with that
   template path and text.
2. `words_and_lines_together_are_refused`: the same with one mutation
   line `A42G` as well: no `Interpret`, the blocker is on `Words` with the
   §2.2 message.
3. `words_with_a_fasta_file_are_refused`: `sid-1_cDNA.fa` loaded, words
   typed: the blocker is on `Words` with the §2.2 GenBank message.
4. `the_interpreted_request_is_the_corpus_request`: run `interpret` on the
   case through `ScriptedBackend` with the case's reply (as
   `review_proposal.rs` does), then `review::approve(document, template,
   request)` in the workspace: the written request equals
   `request/variants-separate.json` as JSON and the run is `verified`.
   (This pins that `luria new` reaches the same approve path; the loop
   itself is the owner's hand test.)
5. `blank_words_keep_the_shorthand_path`: words of only spaces and the
   sid1 lines: `[ Review ]` gives `Review(built)`, as 7k3.

In `crates/cli/tests/cli.rs`:

6. `new_with_a_backend_but_no_model_is_a_usage_error` and
   `new_allow_remote_without_backend_is_a_usage_error`, each naming the
   option.
7. `new_without_a_terminal_names_new`: the message is the §2.6 text with
   `'new'`.

### 3.1 Rules that must be proven

1. Either words or lines (remove the first §2.2 check): test 2 fails.
2. Words need GenBank (remove the second): test 3 fails.
3. Blank words are blank (treat any text, spaces included, as words):
   test 5 fails.

## 4. Do not

- Change only `args.rs`, `main.rs`, `review.rs` (§2.6 only),
  `new/form.rs`, `new/text.rs`, `new/terminal.rs`, `new/view.rs` (the
  one §2.3 dim rule), `crates/cli/tests/new_view.rs` (its one test), the new
  `crates/cli/tests/new_words.rs`, and `crates/cli/tests/cli.rs` (§3 tests
  6 and 7 and the one approved assertion).
- No change to `interpret`, the proposal screen, `review::approve`,
  `new::build`, luria-core, luria-nl, fixtures, docs, Cargo files or
  `vendor/`. No network call in any test.
- Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the §3.1 proofs, every existing assertion changed,
and anything that did not match. The loop with a real model is the
owner's hand test.

## 6. Implementation notes (planner, 2026-10-22)

- 1101 tests passed, 0 failed, 1 ignored; fmt, clippy, diff check clean;
  fixtures 19 of 19. Proofs 1 to 3 and the dim rule's proof failed as
  required.
- The 7k5 terminal test was renamed to `new_without_a_terminal_names_new`
  as §3 test 7 names it; its message assertion changed to `'new'`.
- `parse_new`'s unknown-word message still says `takes only --design and
  --export` (pinned by an existing test); the backend options are matched
  before it. Worth updating with 7k6.
- `terminal.rs`'s `request_path` repeats `confirm::plan`'s directory rule,
  since the words path has no `Built`.
- `main.rs` adds `type WordsModel` for clippy's `type_complexity`.
- The model call runs in the loop, so the screen waits while it answers.
- Untested: the proposal loop with a real model, `q` back to the form, and
  the dim row on a real terminal (owner's hand test).
