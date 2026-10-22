# Milestone 7m3: the form's fields after the owner's hand test

Status: **Ready for the worker, 2026-10-22.** (One worker at a time: each runs the whole
test suite). Planner: Claude. Parent: `docs/tasks/milestone-7m-polish.md`,
M7-45, M7-46, M7-47 and M7-51.

## 1. The owner's answers this contract applies

- M7-45: Overlap is prefilled with `15`, shown and editable; a
  `--design` profile's value replaces it. Its place in the focus order is
  its place on screen.
- M7-46: no codon field. A line with `:CODON` uses that codon; a line
  without one uses the default codon policy.
- M7-47: one Mutations box. A line luria cannot read at all (refusal
  `unreadable`) makes the whole box words. With `--backend` the box goes
  to the model; without it, it is a blocker that names `--backend`. A line
  luria reads but refuses stays a red error and is never sent to the
  model. Words still need a GenBank template.
- M7-51: `PgUp`/`PgDn` inside the paste or mutation box move the cursor
  10 lines; elsewhere, focus jumps to the first or the last field.

## 2. Changes (`crates/cli/src/new/form.rs`, `new/text.rs`, `new/view.rs`)

### 2.1 Fields

- Remove `Field::Codon` and `Field::Words`, the `words` and `codon` values,
  `words()`, `codon()`, `words_enabled()` and their rows. `ORDER` becomes
  Source, File, Paste, Name, Shape, Cds, First, Last, Strand, Mutations,
  Overlap, SaveAs, Review.
- `build_form` always passes `CodonChoice::DefaultPolicy` to `new::build`,
  which reads `D404N:AAC` as codon `AAC` and `D404N` as the default policy
  (unchanged; `new.rs` is not touched).
- `NewForm::new`: Overlap is `15` unless the profile gives a design, whose
  `min_annealing_overlap` replaces it. The empty-overlap and too-large
  blockers stay (the user can delete or overwrite the value).
- `words_profile()`: codon is always `Some(CodonSpec::Default)`; the rest
  is unchanged.

### 2.2 Words in the Mutations box

`pub fn reads_as_words(&self) -> bool`: some non-blank mutation line's
`parse_line` is `Err(Refusal::Unreadable)`.

In `review()`, after `form_blocker`, in this order:

1. A non-blank line whose refusal is anything but `Unreadable` is a
   blocker on `Mutations` with that line's number and its refusal's own
   `message()` (as today's `line_refused`).
2. When `reads_as_words()`:
   - without a backend: a blocker on `Mutations`, message `line <n> is
     not shorthand; start luria new with --backend to read the mutations
     in words`, `<n>` the first unreadable line;
   - with one, a template that is not a loaded GenBank record: a blocker
     on `Mutations`, message `words need a GenBank template, whose CDS
     list the model is shown`;
   - otherwise `FormAction::Interpret { template, text, profile }`, `text`
     being every mutation line joined with `\n`.
3. Otherwise `new::build`, as now.

Previews (`text.rs`): when `reads_as_words()`, an `Unreadable` line's
preview is `… <n>  read in words by the model at [ Review ]` with a
backend, and `✗ <n>  not shorthand; start luria new with --backend to
read it in words` without one. Other previews are unchanged. With a
backend, words, and no loaded GenBank record, one more continued row
follows the previews: `needs a GenBank template` (dim, as now).

`view.rs`: a preview starting `…` is drawn dim. The `needs --backend`
and `In words` rules go.

### 2.3 The screen's rows

```
Mutations  <lines>
           <previews>
           [needs a GenBank template]

Primers    inline (confine to tail not supported yet)
Overlap    15 nt
Save as    <name>

           [ Review ]
```

### 2.4 Page keys

In `handle`, for every field: `PageDown` and `PageUp` in `Paste` or
`Mutations` move the cursor 10 lines down or up, clamped to the first and
last line, and the column clamped to the line's length. Anywhere else,
focus goes to the first visible field (`PageUp`) or to `Review`
(`PageDown`). Leaving `File` this way asks to load it, as `tab` does.

## 3. Tests

Existing tests in `crates/cli/tests/new_form.rs`, `new_text.rs`,
`new_view.rs`, `new_words.rs` and `new_confirm.rs` change only where they
use a removed field or value (`Field::Codon`, `Field::Words`, `codon()`,
`words()`, the `In words` and `Codon` rows), or an empty Overlap at
opening. Each such change is approved; report each one with the test's
name. Words tests type their text into `Mutations`. `new_text.rs`'s
28-line sid-1 form loses the `In words` and `Codon` rows, which is
approved. `new_build.rs` and every fixture are unchanged.

New tests, in `crates/cli/tests/new_form.rs` unless named:

1. `overlap_opens_at_15`, and with `profile/full.json` it opens at the
   profile's value.
2. `focus_order_is_screen_order`: tabbing from `Mutations` reaches
   `Overlap`, then `SaveAs`, then `Review`.
3. `a_typed_codon_is_used_and_none_is_the_default`: lines `D404N:AAC` and
   `D405A` build members whose codons are `Supplied("AAC")` and `Default`.
4. `new_words.rs`, `unreadable_lines_go_to_the_model`: the `two-separate`
   case's text typed into `Mutations` with a backend and
   `synthetic-linear.gb` loaded gives `Interpret` with that text.
5. `new_words.rs`, `unreadable_lines_without_a_backend_name_it`: the §2.2
   message, on `Mutations`.
6. `new_words.rs`, `a_refused_line_is_never_words`: lines `D404*` and
   `change alanine 42 to glycine`, with a backend: the blocker is the
   stop refusal on line 1, not `Interpret`.
7. `page_keys_move_ten_lines_or_to_the_ends`.

### 3.1 Rules that must be proven

1. A refused line is never words (drop §2.2 step 1): test 6 fails.
2. Overlap prefill (open it empty): test 1 fails.

## 4. Do not

- Change only `new/form.rs`, `new/text.rs`, `new/view.rs`,
  `new/terminal.rs` (only if a removed name forces it), `main.rs` (same),
  and the tests named in §3.
- No change to `new.rs`, `shorthand.rs`, `interpret`, the review screens,
  fixtures, docs, Cargo files or `vendor/`.
- Do not stage or commit.

## 5. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the §3.1 proofs, and every existing assertion
changed.
