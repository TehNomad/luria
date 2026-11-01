# Review M6 B-7: a codon belongs to its own quote; prompt v3 (D045)

Status: **Implemented 2026-10-20** by the local worker; accepted on planner
review. Prompt v3 measured live by the planner (research note, "Prompt v3
results"). Planner: Claude. Parent: `docs/tasks/reviews/review-post-m6.md` B-7.
Owner answer, 2026-10-20: "In its own quote, whole word", with the prompt
bump and the `codon-stated` reply change approved.

Files (no others):
- `crates/cli/src/interpret/build.rs`
- `crates/cli/tests/documented_interpret.rs` (one line, §2.5)
- `crates/nl/src/prompt.rs`
- `crates/nl/tests/prompt.rs`
- `docs/research/m6-local-model-extraction-2026-10-19.md` (append, §2.4)
- `docs/specs/cli.md` (one table row of §10.6, §2.5)
- `tests/fixtures/nl/reply/codon-stated.json` (one value, §2.3)
- `tests/fixtures/nl/README.md` (one sentence, §2.3)

## 1. Defect

A codon is grounded anywhere in the text, as a case-insensitive substring
(`stated_codon_in`, `build.rs`). So `GGC` inside "ggcx" is accepted, and two
Gly codons the text gives for two different mutations can be swapped by the
model and both pass; core only checks a codon encodes its target.

## 2. Required change

### 2.1 `build.rs`: the codon must be in its quote, as a whole word

1. Step 1 (grounding) is unchanged: a codon the text does not contain at
   all is still `Ungrounded { field: "mutations[i].codon" }`.
2. A new private function:
   ```rust
   /// Whether `quote` holds `codon` as a whole word (D045): its chars, ASCII
   /// case-insensitively, at some position with no ASCII letter just
   /// before or just after. A blank codon is never held.
   fn codon_in_quote(quote: &str, codon: &str) -> bool
   ```
   Chars, never bytes.
3. A new `QuestionCode::CodonNotInQuote`, code `"codon_not_in_quote"`,
   placed right after `CodonUnstated` in the enum and in `code()`.
4. In `read_mutation`'s `Ok(substitution)` arm, after the number, original,
   target, direction and quote-ambiguity questions and before the existing
   `codon_unstated` check: when the mutation's codon is `Some(codon)` and
   `!codon_in_quote(&quote, &codon)`, push
   ```rust
   Question {
       code: QuestionCode::CodonNotInQuote,
       subject: token.clone(),
       message: format!(
           "\"{}\" does not hold the codon {}, so luria cannot tell it is meant for this substitution",
           quote.clone(),
           codon.clone(),
       ),
   }
   ```
   The `codon_unstated` check (codon `None`, no profile policy) is
   unchanged.
5. Update the doc comment of `stated_codon_in` to say step 1 checks the
   text and step 3b checks the quote (D045).

### 2.2 `crates/nl/src/prompt.rs`: prompt v3

1. `PROMPT_VERSION` becomes `"luria.q5-extraction-prompt/3"`; update the
   pin test in this file's test module (today it asserts `/2`).
2. In `SYSTEM_PROMPT`, replace exactly this line:
   ```
     - quote: the exact words of the user's text that state this substitution, copied character for character.
   ```
   with exactly:
   ```
     - quote: the exact words of the user's text that state this substitution and no other, copied character for character. If the user names a codon for it, the quote includes the codon.
   ```
   Nothing else in the prompt, the schema or the user message changes.
3. The module doc and the doc of `SYSTEM_PROMPT` say "prompt v2" and give
   v2's scores: make them say prompt v3, from the note's "Prompt v3"
   section, and drop the v2 scores (the note keeps them).

### 2.3 The fixture

- `tests/fixtures/nl/reply/codon-stated.json`: the one mutation's
  `"quote"` value `"W87 to alanine"` becomes
  `"W87 to alanine using the codon GCG"`. Nothing else in the file
  changes. The text file is unchanged and already holds those words:
  `In synA, change W87 to alanine using the codon GCG. Numbering from the CDS start.`
- `tests/fixtures/nl/README.md`: after the sentence ending
  `` (`codon-stated`). `` in "The cases", add: `Its reply quotes the codon
  with its substitution, as D045 requires.`

The expected request (`edit-w87a-gcg.json`) is unchanged.

### 2.4 The research note

Append to the end of
`docs/research/m6-local-model-extraction-2026-10-19.md`:

1. A heading `## Prompt v3`, and one paragraph: "Post-M6 review, 2026-10-20
   (D045). One line of v2 changes, the quote's: it must state one
   substitution only, and include a codon the user names, because luria now
   asks when a quote holds more than one number or misses its codon. The
   schema and the user message are unchanged."
2. `System message:` followed by a fenced block (plain ```` ``` ````) holding
   the whole v3 `SYSTEM_PROMPT`, character for character.
3. `User message (catalogue, then the text):` and
   `` `response_format.json_schema.schema`: `` each followed by the same
   fenced block as in "Prompt v2" (copy them; the schema's fence is
   ```` ```json ````).
4. A line `Measured: see "Prompt v3 results" below.` and a heading
   `### Prompt v3 results` with the single line `To be recorded by the
   planner's live run.` The planner fills it in before the commit.

### 2.5 `crates/nl/tests/prompt.rs` and the code table

- `block` reads the `## Prompt v3` section instead of `## Prompt v2`
  (the split string, the `expect` and the panic text). Update its doc
  comment. Nothing else in the file changes.
- `crates/cli/tests/documented_interpret.rs`: add
  `QuestionCode::CodonNotInQuote,` right after `QuestionCode::CodonUnstated,`.
- `docs/specs/cli.md` §10.6: add one row right after `codon_unstated`, exactly:
  ```
  | `codon_not_in_quote` | A codon is in the text, but not in the words given for its substitution. |
  ```

## 3. Tests (write first; confirm each new one fails before the change)

In `build.rs`'s test module. Add
`(QuestionCode::CodonNotInQuote, "codon_not_in_quote")` to
`every_question_code_is_pinned` after the `codon_unstated` row. The test
`a_lower_case_codon_is_grounded_and_written_upper_case` must still pass
unchanged: change its reply's quote only if it fails, and then report it
(its text is "change A42 to glycine using codon ggc", quote "A42 to
glycine"; the planner expects it to need the quote
`"A42 to glycine using codon ggc"` — make exactly that change, nothing
else, and say so in the handoff).

New tests, with `profile(false, true)`, `syna()`, `facts()`, `"cds":"synA"`,
numbering `"Numbering from the CDS start"`, composition as stated,
unsupported `[]`:

1. `a_codon_outside_its_quote_is_asked_about`: text
   `"In synA, change W87 to alanine using the codon GCG. Numbering from the CDS start."`,
   mutation `{"token":"W87A","quote":"W87 to alanine","codon":"GCG"}` →
   `asked` is `[("codon_not_in_quote", "W87A")]`, and `messages` is
   `["\"W87 to alanine\" does not hold the codon GCG, so luria cannot tell it is meant for this substitution"]`.
2. `a_codon_inside_a_word_is_not_a_codon`: text
   `"In synA, change A42 to glycine with ggcx. Numbering from the CDS start."`,
   mutation `{"token":"A42G","quote":"A42 to glycine with ggcx","codon":"GGC"}`
   → `asked` is `[("codon_not_in_quote", "A42G")]`.
3. `codons_swapped_between_mutations_are_asked_about`: text
   `"In synA, make A42G with GGC and W87G with GGT as separate variants. Numbering from the CDS start."`,
   composition `"separate_variants"`, mutations
   `{"token":"A42G","quote":"A42G with GGC","codon":"GGT"}` and
   `{"token":"W87G","quote":"W87G with GGT","codon":"GGC"}` → `asked` is
   `[("codon_not_in_quote", "A42G"), ("codon_not_in_quote", "W87G")]`.
4. `a_codon_in_its_quote_is_used`: the text of test 1, mutation
   `{"token":"W87A","quote":"W87 to alanine using the codon GCG","codon":"gcg"}`
   → `Built::Proposed`, codon `CodonSpec::Supplied("GCG")`.

Disabling proof: skip §2.1 item 4 → tests 1, 2 and 3 fail.

Every corpus case must keep its outcome, `codon-stated` with its new reply
included. `crates/nl/tests/prompt.rs` must pass against the note's new
section. If anything else changes, stop and report.

## 4. Do not

- Add dependencies, or change any file not listed above.
- Change the schema, the user message layout or any other prompt line.
- Change the "Prompt v2" section or anything else already in the note.
- Change any other fixture or test than those named.
- Run the live test (`--ignored`); the planner does, because it uses the
  model server.

## 5. Checks

From the repository root, after `export TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report the changed files, each new test and whether it failed before the
change, the disabling proof and its result, whether
`a_lower_case_codon_is_grounded_and_written_upper_case` needed its quote
changed, the check results, and any deviation.
