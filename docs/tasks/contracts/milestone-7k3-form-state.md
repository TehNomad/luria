# Milestone 7k3: the `luria new` form's state

Status: **Implemented 2026-10-22** by w7k3 (ran out of context before reporting) and w7k3b (proofs and checks). Proofs 1-3 failed as required; proof 2 was run by adding a preview cache refreshed only on typing, since previews are computed on demand. Planner: Claude. Parent:
`docs/tasks/contracts/milestone-7k-new.md` §1, §2 (owner answers M7-31 to M7-39).
Builds on 7k1 (`shorthand::parse_line`) and 7k2 (`new::build`). Next: 7k4
draws it, 7k5 wires it to a terminal and the proposal screen.

## 1. What and why

The form holds what the user has typed and chosen, moves focus, and on
`[ Review ]` turns itself into a `new::Form` and calls `new::build`. It is
pure: no terminal, no file I/O, no model. Reading a file is the loop's job
(7k5): the form asks for it with an action and is told the outcome.

## 2. Keys, `crates/cli/src/review/app.rs` and `terminal.rs`

`Key` gains `Tab`, `BackTab`, `Left`, `Right`, `Delete`; `terminal.rs`'s
`key_of` maps `KeyCode::Tab`, `BackTab`, `Left`, `Right`, `Delete` to them.
Nothing else in either file changes; the review app ignores the new keys
(they reach its existing `_ =>` arm). If an existing test fails, stop and
report.

## 3. The module `crates/cli/src/new/form.rs`

`crates/cli/src/new.rs` gains `pub mod form;` (Rust's `new.rs` +
`new/form.rs` layout).

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Source,      // file or paste
    File,        // the path typed (Source = file)
    Paste,       // the pasted text (Source = paste), multi-line
    Name,        // the template's name (FASTA and paste only)
    Shape,       // linear or circular (FASTA and paste only)
    Cds,         // whole, or bases (FASTA and paste); the CDS list (GenBank)
    First, Last, // 1-based bases (CdsChoice::Bases only)
    Strand,      // forward or reverse (FASTA and paste only)
    Mutations,   // multi-line
    Words,       // free text; skipped by focus without a backend
    Codon,       // default policy, or typed per mutation
    Overlap,     // digits
    SaveAs,      // the request file's name
    Review,      // the button
}

pub enum FormAction {
    Continue,
    Quit,
    /// The user left `File` with a path: the loop reads it and calls
    /// `NewForm::loaded`.
    Load(String),
    /// `[ Review ]` with every field valid.
    Review(Box<new::Built>),
}

pub struct NewForm { /* private */ }

impl NewForm {
    /// `overlap` and `codon` from `--design` when given (rule 2), else
    /// empty; `backend` says whether `Words` is enabled (M7-36).
    pub fn new(profile: Option<&ProfileDocument>, backend: bool) -> NewForm;
    pub fn handle(&mut self, key: Key) -> FormAction;
    /// Bracketed paste: the text goes into the focused text field as typed
    /// characters would, newlines kept in multi-line fields and dropped in
    /// single-line ones.
    pub fn paste(&mut self, text: &str);
    /// The loop's answer to `Load`: the file's name (no directory) and its
    /// text, or the operating system's reason it could not be read.
    pub fn loaded(&mut self, file_name: &str, outcome: Result<String, String>);
    // Read-only accessors for 7k4: focus(), the value of each field, the
    // GenBank CDS list with the selected index, `previews()` (below),
    // `blocker()` (the last refusal: field, line, message), `words_enabled()`.
}
```

### 3.1 Behaviour

- **Focus**: `Tab` and `Down` move to the next visible field, `BackTab`
  and `Up` to the previous one, wrapping. A field that does not apply is
  skipped: `Paste` with Source = file and `File` with Source = paste;
  `Name`, `Shape`, `Strand` with a GenBank file; `First`/`Last` unless CDS
  = bases; `Words` without a backend. Inside `Paste`, `Mutations` and
  `Words`, `Up`/`Down` move between lines instead, and focus leaves them
  only by `Tab`/`BackTab`.
- **Choices** (`Source`, `Shape`, `Cds`, `Strand`, `Codon`): `Left`/`Right`
  or space toggle. For a GenBank file, `Cds` is the catalogue
  (`interpret::catalogue::catalogue`); `Left`/`Right` move through its
  **usable** entries only.
- **Text**: printable characters insert at the cursor; `Backspace`,
  `Delete`, `Left`, `Right` edit; `Enter` in a multi-line field starts a
  new line, in a single-line field moves to the next field. `Overlap`,
  `First`, `Last` take digits only.
- **`File`**: leaving it (`Tab`, `BackTab`, `Enter`) with a non-empty
  path that differs from the last one loaded returns `Load(path)`.
  `loaded` then sets the template: text starting with `LOCUS` is GenBank
  (the CDS list from the catalogue; the request's `id` is the LOCUS name),
  text starting with `>` is FASTA (`Name` set to the file name's stem
  when empty); anything else, or an `Err`, is a blocker on `File` naming
  the reason.
- **Previews**: `previews()` gives, for each mutation line in order, the
  line's number and either the `Reading` or the `Refusal` from
  `shorthand::parse_line`; blank lines have none. It is recomputed on
  every edit, so the screen always shows what each line means.
- **`[ Review ]`** (`Enter` on `Review`): build a `new::Form` from the
  fields and call `new::build`. The form's own blockers, checked first,
  in this order, each a message and the field it moves focus to: no
  template loaded or pasted (`File` or `Paste`), no name (`Name`), CDS
  bases empty (`First`/`Last`), no overlap (`Overlap`), no request name
  (`SaveAs`), and a `--design` profile whose placement is not `inline`
  (`Overlap`, message saying `q5 design` supports inline only, M7-39).
  Then `build`'s refusal becomes a blocker on `Mutations` (with its line)
  or on the field it concerns (`MissingOverlap` on `Overlap`,
  `TemplateUnreadable` on `File`/`Paste`, `OutsideTemplate` without a
  line on `First`). Success returns `Review(built)`.
- **`SaveAs`**: filled with `<name>.request.json` whenever `Name` (or a
  GenBank LOCUS name) changes and the user has not typed in `SaveAs`; a
  paste's FASTA is saved as `<name>.fa` (shown read-only beside it).
- **Quit**: `Esc` returns `Quit` from any field.

## 4. Tests: `crates/cli/tests/new_form.rs` (new)

Drive the form with `Key` values and `paste`, as a user would; templates
are read from `tests/fixtures/` at test time and given through `loaded`.

1. `typing_the_sid1_set_form_builds_its_request`: type the path, answer
   `Load` with `loaded("sid-1_cDNA.fa", text)`, keep CDS whole and forward,
   type `requests.py`'s `sid1-set` lines into `Mutations` (with the blank
   line), choose the default codon policy, type `15` into `Overlap`, press
   `Enter` on `[ Review ]`: the action is `Review(built)` and `built`'s
   request, rendered with `request_echo`, equals
   `tests/fixtures/new/request-sid1-set.json` as JSON.
2. `a_pasted_template_builds_its_request`: Source = paste, `paste()` the
   `syn-paste` text, name `syn`, CDS bases `1` to `99`, its two lines, typed
   codon, overlap `15`: equals `request-syn-paste.json`, and `built.paste`
   is `("syn.fa", paste-syn-paste.fa's text)`.
3. `a_genbank_file_offers_its_usable_cds`: load
   `genbank/synthetic-linear.gb`: the CDS list is the catalogue's usable
   entries, `Name`/`Shape`/`Strand` are skipped by `Tab`, and lines `A42G`,
   `W87A` with overlap `20` give `request-synlin-genbank.json`.
4. `previews_follow_every_edit`: after typing `D404` the preview is a
   refusal; after `N` it is a residue reading; after `Backspace` a refusal
   again.
5. `review_without_an_overlap_names_the_field`: the sid1 form with no
   overlap: `Review` is not returned, `blocker()` names `Overlap`, and
   focus is on `Overlap`.
6. `a_refused_line_is_named`: lines `D404N` and `D404*`: the blocker is on
   `Mutations`, line 2.
7. `words_is_skipped_without_a_backend`: with `backend = false`, `Tab`
   never focuses `Words`; with `true`, it does.
8. `escape_quits`: `Esc` from `Mutations` returns `Quit`.

### 4.1 Rules that must be proven

1. Skipping `Name`/`Shape`/`Strand` for GenBank: test 3 fails.
2. Recomputing previews on `Backspace`: test 4 fails.
3. Moving focus to the blocker's field: test 5 fails.

## 5. Do not

- Change only `crates/cli/src/review/app.rs` (the `Key` variants),
  `crates/cli/src/review/terminal.rs` (`key_of`), `crates/cli/src/new.rs`
  (one `pub mod form;` line), the new `crates/cli/src/new/form.rs` and
  `crates/cli/tests/new_form.rs`.
- No drawing, no file reading, no terminal. No change to `new::build`,
  `shorthand.rs`, luria-core, fixtures, docs, Cargo files or `vendor/`.
- Do not stage or commit.

## 6. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the §4.1 proofs, and anything that did not match.
