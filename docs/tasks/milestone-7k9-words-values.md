# Milestone 7k9: words use the form's values; pasted line ends; a waiting line

Status: **Ready for the worker, 2026-10-22.** Planner: Claude. Follows
7k7 and 7k8. Findings from the planner's code review of the untested
terminal paths, with owner answers M7-41 and M7-42.

## 1. What and why

1. **M7-41 (owner, 2026-10-22): with words, the form's values are the
   design.** The form makes the user fill in Overlap and choose a codon
   policy, but 7k7 sent only a `--design` file to `interpret`. Without
   `--design`, every words proposal asked `design_unstated` and could not
   be approved. The form now gives `interpret` a profile built from what
   it shows: the overlap typed and placement `inline` (the only one, D029).
   The codon is `default` when "default policy" is chosen, and unstated
   when "typed per mutation" is, so the text must state one or the model
   path asks.
2. **M7-42 (owner, 2026-10-22): with words, residues count from the CDS
   start** unless the text says otherwise, the same rule the shorthand
   path uses (docs/cli.md §12.2).
3. **Pasted line ends.** Many terminals send a bracketed paste's line
   breaks as `\r`, not `\n`. `NewForm::paste` treats only `\n` as a line
   break, so a FASTA pasted from such a terminal becomes one line with
   `\r`s in it, and is refused or misread. `\r\n` and a lone `\r` must
   each be one line break.
4. **The screen during the model call.** The call can take up to 600
   seconds (`REQUEST_TIMEOUT_SECONDS`), and the screen shows nothing
   while it waits. One line now says the model is being asked.

## 2. Changes

### 2.1 `crates/cli/src/interpret/run.rs`: a profile given as a value

Add

```rust
pub fn interpret_with_profile(
    input: &InterpretInput,
    profile: ProfileDocument,
    backend: &mut dyn Backend,
) -> InterpretOutcome
```

It runs exactly as `interpret` does, except that step 2 (reading
`input.profile`) is skipped and `profile` is used in its place.
`input.profile` must be `None`; if it is not, panic with "a profile is
given twice". Do it by moving the body after step 2 into one private
function that both public functions call; `interpret`'s behaviour and
every existing test stay unchanged.

### 2.2 `crates/cli/src/new/form.rs`

- `FormAction::Interpret` gains a third field, `profile: ProfileDocument`.
- `pub fn words_profile(&self) -> ProfileDocument`:
  - `design`: `Some(DesignSpec { policy: LuriaNative, style:
    Q5Polymerase, min_annealing_overlap: <Overlap as typed>, placement:
    Inline, buffer: Q5Reaction })`, which are `new::build`'s own values
    (`crates/cli/src/new.rs`, the `DesignSpec` in `build`);
  - `codon`: `Some(CodonSpec::Default)` for `CodonChoice::DefaultPolicy`,
    `None` for `TypedPerMutation`;
  - `numbering`: `Some(NumberingSpec::FromCdsStart)`.
  `review()` reaches `Interpret` only after `form_blocker`, so Overlap is
  written and is digits only; parse it with `expect("the overlap is
  digits")`.
- `paste()`: a `\r` followed by `\n` is one line break, and a lone `\r`
  is one line break; both are handled exactly as `\n` is now, kept in
  multi-line fields and dropped in single-line ones.

### 2.3 `crates/cli/src/new/terminal.rs` and `crates/cli/src/main.rs`

- `run_new`'s `interpret` becomes `&mut dyn FnMut(&str, &str,
  ProfileDocument) -> InterpretationDocument`, and `FormAction::Interpret`
  passes its profile through it.
- Just before calling it, the loop draws the form once with
  `new::view::draw_waiting`, below.
- In `main.rs`, the closure calls `interpret_with_profile` with
  `profile: None` in the input and the given profile. With `--design`,
  the form already opened with the profile's overlap and codon (7k3), so
  the file is not read a second time. `type WordsModel` changes to match.

### 2.4 `crates/cli/src/new/view.rs`

`pub fn draw_waiting(frame: &mut Frame, form: &NewForm)`: exactly
`draw_form`, except that the key line is `asking the model; this screen
answers when it replies` and no cursor is set. Share the code with
`draw_form`; do not copy it.

### 2.5 Docs (authorized here)

- `docs/cli.md`, §12.1's `In words` row: after `not both.` add `The
  form's overlap and codon choice go with them, and residues count from
  the CDS's first codon unless your words say otherwise.`
- `docs/decisions.md`, D047's Words bullet: after `are not combined.` add
  `The form's overlap and codon choice are the design they are read
  with, and residues count from the CDS start (M7-41, M7-42).`
- Change the status line of D047 from `answers M7-31 to M7-40` to
  `answers M7-31 to M7-42`.

Each anchor text occurs once; if one does not, stop and ask.

## 3. Tests

In `crates/cli/tests/new_words.rs`:

1. `words_without_a_profile_use_the_forms_values`: corpus case
   `no-profile` (template `synthetic-linear.gb`, text
   `text/no-profile.txt`, reply `reply/no-profile.json`, which answers
   numbering `null`). Build a form with **no** `--design`, load the
   template, type the words, type `15` in Overlap, keep "default policy",
   and press `[ Review ]`. Run its `Interpret` through
   `interpret_with_profile` with `ScriptedBackend` and the case's reply.
   The state is `proposed`, there are no questions, and the request
   equals `request/edit-a42g.json` as JSON. That file is the corpus's
   expected request for the same edit under `profile/full.json`, whose
   values are overlap 15, codon default and numbering from the CDS start.
2. `a_typed_overlap_wins_over_the_profile`: a form opened with
   `profile/full.json` (overlap 15), Overlap changed to `20`: the
   `Interpret` profile's overlap is 20.
3. `typed_codons_leave_the_codon_unstated`: "typed per mutation" chosen:
   the `Interpret` profile's codon is `None`.

Change `words_with_a_genbank_file_ask_to_interpret` to compare the new
`profile` field as well (approved). `the_interpreted_request_is_the_corpus_request`
stays as it is.

In `crates/cli/tests/new_form.rs`:

4. `a_paste_with_carriage_returns_reads_as_lines`: the test's own
   `typed_paste()` text with each `\n` replaced by `\r`, and again by
   `\r\n`, gives the same `paste_text()` as the `\n` paste.

In `crates/cli/tests/new_view.rs`:

5. `waiting_names_the_model_call`: `draw_waiting` on the sid-1 form: the
   last row is the §2.4 key line, and every other row equals
   `draw_form`'s.

In `crates/cli/tests/interpret_run.rs`:

6. `a_profile_given_as_a_value_is_used`: case `one-sub-prose` with
   `profile: None` in the input, and `profile/full.json` parsed and passed
   to `interpret_with_profile`, gives the same document as `interpret`
   with the file.

### 3.1 Rules that must be proven

1. The form's overlap (put the profile's overlap back in place of the
   typed one, or a fixed 15): test 2 fails.
2. Numbering from the CDS start (set it to `None`): test 1 fails.
3. A lone `\r` is a line break (remove that rule): test 4 fails.

## 4. Do not

- Change only the files and sections named above: `interpret/run.rs`
  (§2.1), `new/form.rs`, `new/terminal.rs`, `new/view.rs`, `main.rs`,
  the four test files named in §3, and the §2.5 doc lines.
- No change to `interpret`'s behaviour, `build.rs`, the proposal screen,
  `review::approve`, `new::build`, luria-core, luria-nl, fixtures, Cargo
  files or `vendor/`. No network call in any test.
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
and anything that did not match.
