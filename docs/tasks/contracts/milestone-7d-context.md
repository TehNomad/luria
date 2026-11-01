# Milestone 7d: the sequence-context screen

Status: **Implemented 2026-10-21** by the local worker; accepted on the
first review. Proof 2 (the reverse tail) could not fail: no fixture here
has a reverse tail. The planner added `context-ins-04.txt`, from
`native-insertion.tsv` row `ins-04`, which has tails on both primers;
contract 7d2 adds its test. Unverified: a layout longer than one 60-base
block, and a primer with no spanned part. Planner:
Claude. Parent: [docs/tasks/contracts/milestone-7-tui.md](/tasks/contracts/milestone-7-tui.md) §3 rules 1 and 4, §5.3,
§6 (7d0, 7d), §7. Builds on 7b (`app.rs`), 7c1/7c2 (`text.rs`) and 7c3
(`view.rs`). Step 7d0 is done: the expected screens are in
`tests/fixtures/review/` (see its `README.md`).

## 1. What and why

The sequence-context screen lays a designed primer pair out against the
template, so a person can see where each primer sits, which bases anneal,
and where the edit is. It is the one screen that needs the template's
bases, which the result document does not carry. This contract:
1. gives the app the template's bases, read again after the run and
   accepted only when their digest equals the document's;
2. builds the screen's lines in `text.rs`, from the document's primer
   parts and the template, with no biology: the bases are placed by the
   spans the document reports; nothing is complemented or translated;
3. draws them in place of 7c3's placeholder.

## 2. Requirements this contract applies

- **Plan §3 rule 1:** the only sequence operations are slicing the
  template at reported spans and reversing a reverse primer's text so it
  reads 3'->5'. No complement, no Tm, no coordinate is computed. The
  checks that the layout pairs correctly live in the fixture script, not
  here.
- **Fail closed (D005):** when the template cannot be read again, or its
  digest differs from `document.provenance.template.sha256`, the screen
  says so and draws no sequence.
- **The fixtures are the specification.** The three files in
  `tests/fixtures/review/` are the exact expected text, written by
  `tests/fixtures/review/reference/context.py` from independently verified
  rows of `tests/fixtures/q5-primer/`. Do not edit them. If your output
  differs, your code is wrong, or the contract is: stop and report.

## 3. Changes

### 3.1 `crates/cli/src/review.rs`

```rust
/// The template's bases when `sequence` is the one the result was computed
/// on: its digest equals `document.provenance.template.sha256`. `None` when
/// the document names no template, or the digests differ (D005).
pub fn matching_template(document: &ResultDocument, sequence: &Sequence) -> Option<Vec<char>>;
```

`sequence.content_hash().hex_digest()` is the digest to compare.

### 3.2 `crates/cli/src/review/app.rs`

`App` gains a private `template: Option<Vec<char>>`, `None` in `new`, and:

```rust
pub fn with_template(self, letters: Option<Vec<char>>) -> App;
pub fn template(&self) -> Option<&[char]>;
```

Nothing else in `app.rs` changes.

### 3.3 `crates/cli/src/main.rs`

`emit_review`, after `run_with_output` and before opening the screen:
`luria_cli::prepare::prepare(&request)`; on `Ok(prepared)`,
`matching_template(&outcome.document, &prepared.template)`; on `Err`,
`None`. Pass the result to `App::new(document).with_template(..)`.
Nothing else in `main.rs` changes.

### 3.4 `crates/cli/src/review/text.rs`

```rust
pub fn context(document: &ResultDocument, index: usize, template: Option<&[char]>) -> Vec<String>;
```

1. `index` out of range: `["no such member"]`.
2. The member has no `design`: `["no design, so no sequence context"]`.
3. `template` is `None`: `["sequence context unavailable: the template
   could not be read again, or it differs from the one this result was
   computed on"]`.
4. Otherwise the layout below, exactly as `tests/fixtures/review/reference/context.py`
   writes it (read that file's docstring and `place` and `screen`
   functions; they are the reference). In words:

**Placing a primer.** Columns are template positions. For each part of a
`PrimerView`, in order:
- the text is the part's `sequence`, upper case when `annealing` is true,
  lower case otherwise;
- a part with a `span` `(s, e)` and exactly `e - s` bases is written in
  columns `s..e`: as it is for the forward primer, reversed for the
  reverse primer;
- a part with a `span` whose base count differs from `e - s` (an unequal
  replacement) is written as `*` in each column `s..e`, and a note line
  `<role> <part role>: <sequence> at [s, e)` is added (`<role>` is
  `forward` or `reverse`, `<part role>` the part's `role`);
- a part with no `span` (a tail): for the forward primer it is written
  in the columns just left of the primer's first spanned column, ending
  at `first - 1`; for the reverse primer it is written reversed, starting
  at the column just right of its last spanned column (`last`, the largest
  span end). Any of its columns before 0 or at or past the template's
  length is not drawn, and a note line `<role> tail: <sequence> extends
  past the template` is added. Use signed arithmetic for this; never
  underflow.

**The edit row.** `^` in each column of `design.edit_span`. When the
member's mutation is `MutationView::Insertion { point, .. }`, `+` in
column `point`.

**The window.** `lo` = max(0, smallest primer column − 5); `hi` =
min(template length, largest primer column + 1 + 5). Primer columns are
those written for either primer, tails included, within the template.

**Lines.**
1. `sequence context: <member name>, [<lo>, <hi>) of <template id>`,
   `<template id>` being `provenance.template.id`.
2. `top strand 5'->3'; forward 5'->3' above it, reverse 3'->5' below it`
3. `upper case anneals, lower case does not; ^ base edited or removed, +
   insertion point`
4. For each block of 60 columns from `lo` (the last one shorter): a blank
   line; `bases [<s>, <e>)`; then four rows, each a label padded to 10
   characters followed by one character per column, trailing spaces cut:
   `template` (the template's bases), `forward`, `reverse`, `edit`, with a
   space in every column a row does not fill. A row that is empty after
   cutting is the label alone, trimmed (`forward`, `reverse` or `edit`).
5. If there are note lines: a blank line, then the notes.

Check point 4's last sentence against the fixture script before relying
on it: the script cuts every row with `rstrip()`, so an empty row is the
bare label. If the script and this contract disagree, the script wins;
report the difference.

### 3.5 `crates/cli/src/review/view.rs`

The `Context(i)` body becomes `text::context(document, i, app.template())`
instead of 7c3's placeholder line. Nothing else changes.

## 4. Tests

### 4.1 `crates/cli/tests/review_context.rs` (new)

A helper, as `tests/end_to_end.rs` does: a workspace under
`env!("CARGO_TARGET_TMPDIR")` holding a copy of
`tests/fixtures/example-construct/input/sid-1_cDNA.fa`, and a request
written there with `end_to_end.rs`'s SID-1 template (the same `id`
`sid-1`, the same `sha256`, the same declared CDS), `scope` `sid-1`, its
`design()` with the bound given, and one `edit` named by the case id.
Run it with `luria_cli::run::run`, get the template with
`luria_cli::prepare::prepare(&path)` and `matching_template`, and compare
`text::context(&document, 0, Some(&letters))`, each line followed by
`\n` and joined, with the fixture file's whole text.

1. `the_native_01_context_is_the_fixture`: edit `{"replacement":
   {"interval": [1209, 1212], "from": "GAT", "to": "AAC"}}`, bound 15,
   name `native-01`; equals `tests/fixtures/review/context-native-01.txt`.
2. `the_del_11_context_is_the_fixture`: `{"deletion": {"interval": [1209,
   1212], "from": "GAT"}}`, bound 15, name `del-11`; equals
   `context-del-11.txt`.
3. `the_ins_14_context_is_the_fixture`: `{"insertion": {"point": 1209,
   "before": "A", "after": "G", "insert": "GAT"}}`, bound 15, name
   `ins-14`; equals `context-ins-14.txt`. (These three edits and bounds are
   the ones `end_to_end.rs` already checks against the same rows.)
4. `a_changed_template_gives_no_context`: run the native-01 request, then
   change one base of the workspace's FASTA file, `prepare` again, and
   assert `matching_template` is `None` and `text::context(.., None)` is
   the unavailable line.
5. `a_design_failure_has_no_context`: the pBR322 `T99C` document of
   `tests/review_text.rs` (copy its builder): the no-design line.

### 4.2 Unit tests in `text.rs` and `review.rs`

6. `an_unequal_part_is_starred_and_noted`: a hand-built design whose
   forward primer has a mutation part of 6 bases over a 3-column span:
   three `*` in those columns and the note line.
7. `a_tail_past_the_start_is_noted`: a forward tail longer than its first
   spanned column: no panic, the columns before 0 are not drawn, and the
   note is present.
8. `matching_template_needs_the_same_digest`: `Some` for the run's own
   sequence; `None` for a sequence with one base changed; `None` for a
   document with no template.

### 4.3 Rules that must be proven

1. The reverse primer not reversed: tests 1–3 fail.
2. The reverse tail placed left instead of right: test 3 fails.
3. Annealing case inverted (lower case for annealing parts): test 1
   fails.
4. `matching_template` ignoring the digest: tests 4 and 8 fail.

## 5. Do not

- Change only `review.rs`, `review/app.rs`, `review/text.rs`,
  `review/view.rs`, `main.rs` (§3.3 only), and the new
  `tests/review_context.rs`.
- Do not change any fixture, `tests/fixtures/review/` included, any other
  test, [docs/](/), any `Cargo.toml` or `vendor/`.
- Do not complement a base, compute a coordinate the document does not
  report, or call `luria_core`'s `reverse_complement`.
- Do not add dependencies. Do not stage or commit. Do not run `luria
  review` by hand.

## 6. Checks

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
git diff --check
git status --short
```

Report each result, the proofs of §4.3, and any point where this contract
and the fixture script disagree; stop and report rather than guess.
