# Review M6 B-6, B-8: nothing dropped, nothing repeated (D045)

Status: **Ready for the worker** (after `review-m6-b7-codon.md` is
committed). Planner: Claude. Parent: `docs/tasks/review-post-m6.md` B-6,
B-8. Owner answers, 2026-10-20: "Scan for notation tokens" and "Ask in
interpret".

Files (no others):
- `crates/cli/src/interpret/build.rs`
- `crates/cli/tests/documented_interpret.rs` (two lines, §2.4)
- `docs/cli.md` (two table rows of §10.6, and one sentence, §2.4)

## 1. Defects

- B-6: text "Using synA, design Q5 SDM primers for A42G and W87A as
  separate variants. …" with a reply listing only `A42G` is proposed as a
  single `edit`. The variant the user asked for is silently dropped.
- B-8: a reply listing `A42G` twice is proposed as a two-member `variants`
  set, which `q5 design` then refuses (exit 4, `duplicate_variant_name`).

## 2. Required change

### 2.1 Two new question codes

In `QuestionCode`, after `MutationQuoteAmbiguous` (from
`review-m6-b1-quote-locality.md`), in the enum and in `code()`:
- `MutationRepeated`, code `"mutation_repeated"`;
- `MutationNotExtracted`, code `"mutation_not_extracted"`.

### 2.2 Substitution words in the text (B-6)

A new private function:

```rust
/// The words of `text` that 6b's notation reads as a substitution, in text
/// order, each once (D045). A word is a maximal run of ASCII letters, ASCII
/// digits and `.`, with any `.` at its end removed; it counts when
/// `parse_substitution` reads it (`Ok`).
fn substitution_words(text: &str) -> Vec<String>
```

Chars, never bytes. "Each once": a word equal to one already listed is
skipped. So in "A42G and W87A." the words are `A42G` and `W87A`;
`p.A42G` is one word; `K12`, `Q5`, `W87` and `K12del` do not count.

A word is **covered** when some mutation's `quote` holds it as a whole
word: its chars at some position with no ASCII letter, ASCII digit or `.`
just before or just after (a `.` just after is allowed when it is the last
char of the quote or is followed by a char that is not an ASCII letter or
digit). Write this as a private function
`fn quote_holds_word(quote: &str, word: &str) -> bool`.

### 2.3 `build`, step 3b

After the existing loop over mutations (3b), and before 3c, in this order:

1. **Repeats (B-8).** Over the reads whose token parsed (`substitution` is
   `Some`), in extraction order: for each canonical name
   (`Substitution::canonical()`) that two or more reads share, push once,
   at its first appearance:
   ```rust
   Question {
       code: QuestionCode::MutationRepeated,
       subject: name.clone(),
       message: format!("{} is listed more than once", name.clone()),
   }
   ```
2. **Not extracted (B-6).** For each word of `substitution_words(text)`, in
   order, that no mutation's quote covers, push:
   ```rust
   Question {
       code: QuestionCode::MutationNotExtracted,
       subject: word.clone(),
       message: format!(
           "the text names {}, but the reply lists no substitution quoting it",
           word.clone(),
       ),
   }
   ```
   This runs with no mutations too, after 3a's `mutation_unstated`.

Update the step-3 comment and the module doc's paragraph "What is not
judged here": remove "whether two members repeat" from that list, and add
a sentence that repeats and uncovered substitution words are asked about
(D045), while a substitution written only in prose that the reply drops is
not detected.

### 2.4 Docs and the code table

- `crates/cli/tests/documented_interpret.rs`: add
  `QuestionCode::MutationRepeated,` and `QuestionCode::MutationNotExtracted,`
  right after `QuestionCode::MutationQuoteAmbiguous,`.
- `docs/cli.md` §10.6: right after the `mutation_quote_ambiguous` row, add
  exactly:
  ```
  | `mutation_repeated` | The same substitution is listed twice. |
  | `mutation_not_extracted` | Your text holds a substitution such as `A42G` that the reply does not list. |
  ```
- `docs/cli.md` §10.6: after the paragraph that starts "All the questions
  are asked at once.", add a paragraph, exactly:
  ```
  luria checks that every substitution written as a token (`A42G`,
  `Ala42Gly`, `p.A42G`) is in the reply. It cannot check that for one
  written only in words ("alanine 42 to glycine"): read the proposal's
  members before you run it.
  ```

## 3. Tests (write first; confirm each new one fails before the change)

In `build.rs`'s test module. Add the two codes to
`every_question_code_is_pinned` after `mutation_quote_ambiguous`. Other
existing tests unchanged. New, with `profile(true, true)`, `syna()`,
`facts()`, `"cds":"synA"`, unsupported `[]`:

1. `a_substitution_the_reply_drops_is_asked_about`: text
   `"Using synA, design Q5 SDM primers for A42G and W87A as separate variants. Number residues from the beginning of the annotated CDS."`,
   composition `"separate_variants"`, numbering
   `"from the beginning of the annotated CDS"`, one mutation
   `{"token":"A42G","quote":"A42G","codon":null}` → `asked` is
   `[("mutation_not_extracted", "W87A")]`.
2. `every_written_form_is_found`: text
   `"In synA, make p.A42G, ala87gly and W87A. Numbering from the CDS start."`,
   composition null, numbering `"Numbering from the CDS start"`,
   mutations `[]` → `asked` is
   `[("mutation_unstated", "mutations"), ("mutation_not_extracted", "p.A42G"), ("mutation_not_extracted", "ala87gly"), ("mutation_not_extracted", "W87A")]`.
3. `words_that_are_not_substitutions_are_not_asked_about`: text
   `"In synA with Q5, delete K12 and change W87 to alanine in HbA1c-free buffer. Numbering from the CDS start."`,
   composition null, numbering `"Numbering from the CDS start"`, one
   mutation `{"token":"W87A","quote":"W87 to alanine","codon":null}` →
   `Built::Proposed`.
4. `a_repeated_substitution_is_asked_about`: text
   `"In synA, make A42G and Ala42Gly as separate variants. Numbering from the CDS start."`,
   composition `"separate_variants"`, numbering
   `"Numbering from the CDS start"`, mutations
   `{"token":"A42G","quote":"A42G","codon":null}` and
   `{"token":"A42G","quote":"Ala42Gly","codon":null}` → `asked` is
   `[("mutation_repeated", "A42G")]`, `messages` is
   `["A42G is listed more than once"]`.

Before running, check each text against D045's other rules (one number
per quote, composition, numbering sentence, CDS name, codon); if a test
is asked something else, report it rather than changing the text.

Disabling proofs (once each, run, restore, report): skip item 1 → test 4
fails; skip item 2 → tests 1 and 2 fail; count every word of letters
and digits that holds a digit, whatever `parse_substitution` returns →
test 3 fails.

Every corpus case must keep its outcome (the planner has checked that no
corpus text holds an uncovered substitution word). If one changes, stop
and report; do not edit a fixture.

## 4. Do not

- Add dependencies, or change any file not listed above.
- Change `parse_substitution` or any notation rule.
- Change any fixture or existing test other than those named.

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
