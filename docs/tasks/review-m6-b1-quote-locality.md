# Review M6 B-1, B-2: one number per quote, in order (D045)

Status: **Ready for the worker.** Planner: Claude. Parent:
`docs/tasks/review-post-m6.md` B-1, B-2. Owner answer, 2026-10-20: "One
number, in order". Records D045 for the whole review.

Files (no others):
- `crates/cli/src/interpret/evidence.rs`
- `crates/cli/src/interpret/build.rs`
- `crates/cli/tests/documented_interpret.rs` (one line, §2.4)
- `docs/cli.md` (two table rows of §10.6, §2.4)
- `docs/decisions.md` (append D045, text in §2.5)

## 1. Defect

`evidence` (`evidence.rs:209`) accepts a token when its number, original and
target each appear *somewhere* in the quote, the original before the target.
So:
- B-1: text "make A37G and A42S as separate variants", reply
  `{"token": "A37S", "quote": "A37G and A42S"}` is proposed as `A37S`, an
  edit the user never asked for, and `q5 design` verifies it;
- B-2: "In ydj42, change the alanine to a glycine" with token `A42G` is
  proposed: the 42 is part of the CDS name, and the user stated no number.

## 2. Required change

### 2.1 `evidence.rs`: a new public function

```rust
/// Every maximal run of ASCII digits in `quote`, as half-open char spans in
/// order: `digit_runs("A37G and A42S")` is `[(1, 3), (9, 11)]`.
pub fn digit_runs(quote: &str) -> Vec<(usize, usize)>
```

Char indices, as everywhere in this file. Use the existing `digits_end`.

### 2.2 `evidence.rs`: the new rule in `evidence`

`Evidence`'s fields and `residue_mentions` are unchanged. `number_visible`
is unchanged and stays public. `evidence` computes `originals` and
`targets` as now, then `runs = digit_runs(quote)`, and:

- **No run.** `number` is `false`. `in_order` is today's rule: some
  original mention ends at or before the start of some target mention.
- **Exactly one run `(s, e)`.** `number` is `true` only when all three hold:
  1. `number_visible(run_text, substitution.number)`, where `run_text` is
     the run's chars collected into a `String`;
  2. left bound: `s == 0`, or the char at `s - 1` is not an ASCII letter,
     or some original mention ends exactly at `s` (`A42G`, `Ala42Gly`);
  3. right bound: `e` is the quote's char length, or the char at `e` is not
     an ASCII letter, or some target mention starts exactly at `e`.

  When `number` is `true`, `in_order` is: some original mention ends at or
  before `s` **and** some target mention starts at or after `e`. When
  `number` is `false`, `in_order` is today's rule (as with no run), so a
  quote whose only problem is its number earns only the number question.
- **Two or more runs.** `number` is `false` and `in_order` is `false`.
  (`build` asks one question instead, §2.3.)

`original` and `target` are unchanged in every case.

Rewrite the module doc's first and third bullets (the number rule and the
order rule) to state exactly the rule above, and add `digit_runs` to them.
Keep the rest of the doc.

### 2.3 `build.rs`

1. A new `QuestionCode::MutationQuoteAmbiguous`, code
   `"mutation_quote_ambiguous"`, placed right after
   `SubstitutionDirectionUnclear` in the enum and in `code()`.
2. In `read_mutation`'s `Ok(substitution)` arm: when
   `digit_runs(&quote).len() >= 2`, push one question and skip the number,
   original, target and direction checks for this mutation:
   ```rust
   Question {
       code: QuestionCode::MutationQuoteAmbiguous,
       subject: token.clone(),
       message: format!(
           "\"{}\" holds more than one number, so it does not show which is the residue number",
           quote.clone(),
       ),
   }
   ```
   Otherwise the existing checks run, unchanged in order. The codon check
   after them runs in both cases, unchanged.
3. The direction question's message becomes
   `format!("\"{}\" does not name the original residue, then the number, then the target", quote.clone())`.
4. Import `digit_runs` beside `evidence`. Add a sentence to the module doc's
   list of steps or to `read_mutation`'s doc saying a quote with more than
   one number is asked about as `mutation_quote_ambiguous` (D045).

### 2.4 The documented code table

- `crates/cli/tests/documented_interpret.rs`: add
  `QuestionCode::MutationQuoteAmbiguous,` right after
  `QuestionCode::SubstitutionDirectionUnclear,` in
  `the_question_table_lists_every_code_build_can_ask`. Change nothing else.
- `docs/cli.md` §10.6, the table after `<!-- luria:question-codes -->`:
  replace the `substitution_direction_unclear` row and add one row after it,
  exactly:
  ```
  | `substitution_direction_unclear` | Your words do not name the residue there now, then the number, then the residue to put there. |
  | `mutation_quote_ambiguous` | The words given for one substitution hold more than one number. |
  ```

### 2.5 `docs/decisions.md`

Append after D044, exactly:

```markdown
## D045: Tighter grounding of the model's extraction

Status: approved by the owner, 2026-10-20 (`docs/tasks/review-post-m6.md`,
owner decisions). Amends D044. Each rule fails closed: a doubtful case is
asked about, never proposed.

Decision:

* **One number per quote, in order** (B-1, B-2). A mutation's quote
  supports its token only when it holds exactly one run of digits, equal
  to the residue number and joined to no letter except a residue's name,
  with the original residue named before it and the target after it. A
  quote with more than one number is asked about
  (`mutation_quote_ambiguous`).
* **Numbering in context** (B-3). A negation or contrast word in the
  sentence holding the user's numbering words, or another numbering
  origin anywhere in the text, makes the numbering doubtful.
* **A CDS name is a whole name** (B-4). It must occur in the text bounded
  by characters that cannot be part of a name.
* **Composition is read by Rust** (B-5), from a fixed list of phrasings in
  the text. None is `composition_unstated`; a model's value that disagrees
  is asked about.
* **A codon belongs to its mutation** (B-7). It must occur, as a whole
  word, in that mutation's own quote.
* **Nothing dropped or repeated** (B-6, B-8). A substitution token in the
  text that no quote covers, and two mutations with the same canonical
  token, are asked about. A substitution written only in prose that the
  model drops is not detected: the user's review of the proposal is the
  control.
* **No key over plain HTTP** (M6-18). An API key with an endpoint that is
  not loopback is refused before anything is sent, until luria has TLS.

Reversibility:

Loosening any of these rules needs approval.
```

## 3. Tests (write first; confirm each new one fails before the change)

### 3.1 `evidence.rs` test module (existing tests unchanged)

They use the existing `token` and `shows(number, original, target,
in_order)` helpers.

1. `digit_runs_are_maximal_and_in_order`: `digit_runs("A37G and A42S")` is
   `[(1, 3), (9, 11)]`; `digit_runs("Q5 A42G")` is `[(1, 2), (4, 6)]`;
   `digit_runs("")` and `digit_runs("alanine")` are `[]`;
   `digit_runs("\u{e9}42")` is `[(1, 3)]`.
2. `a_quote_with_two_numbers_shows_no_number`:
   `evidence("A37G and A42S", &token("A37S"))` is
   `shows(false, true, true, false)`.
3. `a_number_joined_to_other_letters_is_not_the_residue_number`:
   - `evidence("In ydj42, change the alanine to a glycine", &token("A42G"))`
     is `shows(false, true, true, true)`;
   - `evidence("synA42 to glycine", &token("A42G"))` is
     `shows(false, false, true, false)`.
4. `the_original_is_before_the_number_and_the_target_after_it`:
   - `evidence("change the alanine to a glycine; see figure 42", &token("A42G"))`
     is `shows(true, true, true, false)`;
   - `evidence("residue 42: alanine to glycine", &token("A42G"))` is
     `shows(true, true, true, false)`.

Disabling proofs (do each once, run, restore, report the result):
- the "two or more runs" branch treated like "one run" on the first run →
  test 2 fails;
- the left and right bounds (§2.2 items 2 and 3) removed → test 3 fails;
- the one-run `in_order` replaced by today's rule → test 4 fails.

### 3.2 `build.rs` test module

The existing tests are unchanged except `a_swapped_token_is_asked_about_not_proposed`,
whose expected message becomes the new one of §2.3 item 3. Add
`(QuestionCode::MutationQuoteAmbiguous, "mutation_quote_ambiguous")` to
`every_question_code_is_pinned`, after the direction row (array length 16).
New tests, with the existing helpers (`syna`, `profile`, `facts`, `reply`,
`asked`, `messages`):

1. `a_quote_holding_two_substitutions_is_asked_about`: text
   `"In synA, make A37G and A42S as separate variants. Number residues from the beginning of the annotated CDS."`,
   reply one mutation `{"token":"A37S","quote":"A37G and A42S","codon":null}`,
   `"cds":"synA"`, `"composition":null`,
   `"numbering":"from the beginning of the annotated CDS"`,
   `"unsupported":[]`, `profile(true, true)`. `asked` is
   `[("mutation_quote_ambiguous", "A37S")]` and `messages` is
   `["\"A37G and A42S\" holds more than one number, so it does not show which is the residue number"]`.
2. `a_number_inside_a_cds_name_is_not_the_residue_number`: a one-entry
   catalogue `ydj42` (gene `Some("ydj42")`, product `None`, `Usable {
   residues: 200, strand: Strand::Plus }`); text
   `"In ydj42, change the alanine to a glycine. Number residues from the CDS start."`;
   reply `"cds":"ydj42"`, one mutation `{"token":"A42G","quote":"In ydj42, change the alanine to a glycine","codon":null}`,
   `"numbering":"from the CDS start"`, composition null, unsupported `[]`;
   `profile(true, true)`. `asked` is `[("residue_number_unstated", "A42G")]`.
3. `a_number_after_both_residues_is_asked_about`: text
   `"In synA, change the alanine to a glycine; see figure 42. Numbering from the CDS start."`;
   reply `"cds":"synA"`, one mutation
   `{"token":"A42G","quote":"change the alanine to a glycine; see figure 42","codon":null}`,
   `"numbering":"Numbering from the CDS start"`, composition null,
   unsupported `[]`; `profile(true, true)`. `asked` is
   `[("substitution_direction_unclear", "A42G")]`.

Disabling proof: remove §2.3 item 2's branch (run the old checks for every
quote) → test 1 fails.

### 3.3 Existing tests that must pass unchanged

Every other test, including `interpret_build.rs`, `interpret_end_to_end.rs`
and the corpus: the planner has checked that every corpus quote holds one
number, bounded, between its two residues. If any corpus case changes
outcome, stop and report it; do not edit a fixture.

## 4. Do not

- Add dependencies, or change any file not listed above.
- Change `number_visible`, `residue_mentions`, `Evidence`'s fields,
  `numbering_from_cds_start` or any notation rule.
- Change any fixture, or any existing test except the two named in §3.2.
- Touch the numbering, CDS, codon or composition rules (later contracts).

## 5. Checks

From the repository root, after `export TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report the changed files, each new test and whether it failed before the
change, each disabling proof and its result, the check results, and any
deviation.
