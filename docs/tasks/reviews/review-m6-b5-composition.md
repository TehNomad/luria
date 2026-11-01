---
type: Task Contract
title: "Review M6 B-5: the composition is read from the text by Rust (D045)"
description: "composition is one of the schema's fixed values, so it is never grounded."
status: deprecated
tags: [milestone-6]
---
# Review M6 B-5: the composition is read from the text by Rust (D045)

Status: **Implemented 2026-10-20** by the local worker; accepted on planner
review. Planner: Claude. Parent: [docs/tasks/reviews/review-post-m6.md](/tasks/reviews/review-post-m6.md) B-5.
Owner answer, 2026-10-20: "Rust phrase list, must agree".

Files (no others):
- `crates/cli/src/interpret/evidence.rs`
- `crates/cli/src/interpret/build.rs`
- `crates/cli/tests/documented_interpret.rs` (one line, §2.4)
- [docs/specs/cli.md](/specs/cli.md) (one table row of §10.6, §2.4)

## 1. Defect

`composition` is one of the schema's fixed values, so it is never grounded.
With text "Using synA, design Q5 SDM primers for A42G and W87A. Number
residues from the beginning of the annotated CDS." (no composition stated)
and a reply claiming `separate_variants`, the request is proposed and
designs. Plan §5.5 says separate and combined are never assumed.

## 2. Required change

### 2.1 `evidence.rs`: the phrase lists

```rust
/// Phrases, in normalized form, that state separate variants.
pub const SEPARATE_PHRASES: [&str; 8] = [
    "separate variants",
    "separate constructs",
    "separate mutants",
    "separately",
    "individually",
    "individual variants",
    "individual mutants",
    "each on its own",
];

/// Phrases, in normalized form, that state one combined construct.
pub const COMBINED_PHRASES: [&str; 7] = [
    "combined",
    "together",
    "in combination",
    "single construct",
    "one construct",
    "same construct",
    "double mutant",
];
```

### 2.2 `evidence.rs`: what the text states

```rust
/// What the user's text states about composition (D045).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositionReading {
    /// No phrase of either list.
    Unstated,
    Separate,
    Combined,
    /// Phrases of both lists, or a phrase in a sentence with a negation.
    Unclear,
}

pub fn composition_stated(text: &str) -> CompositionReading
```

Exactly:
1. Split `text` into **sentences** at the boundary chars of
   `review-m6-b3-numbering-veto.md` (`.` `;` `!` `?` `\n`; the boundary
   chars themselves belong to no sentence). If `b3` added a helper for
   sentences or boundaries, reuse it; otherwise add a private one. Work in
   chars.
2. For each sentence, take `normalized_words` and look for every phrase of
   both lists as consecutive words (the existing `phrase_words` and
   `phrase_present`).
3. If a sentence holds a phrase of either list **and** a word of
   `NUMBERING_NEGATION`, the answer is `Unclear`.
4. Otherwise, phrases of both lists anywhere → `Unclear`; of
   `SEPARATE_PHRASES` only → `Separate`; of `COMBINED_PHRASES` only →
   `Combined`; none → `Unstated`.

Add a bullet to the module doc describing this rule and citing D045.

### 2.3 `build.rs`, step 3d

1. A new `QuestionCode::CompositionUnclear`, code `"composition_unclear"`,
   placed right after `CompositionUnstated` in the enum and in `code()`.
2. Step 3d, with two or more mutations, becomes (the model's
   `extraction.composition` is `Option<Composition>`):
   - `composition_stated(text)` is `Unstated` → `composition_unstated()`,
     whatever the model said;
   - `Unclear` → the new question below;
   - `Separate` or `Combined`, and the model's value is the same
     (`Some(SeparateVariants)` or `Some(CombinedConstruct)` respectively)
     → no question;
   - `Separate` or `Combined`, and the model's value differs, `None`
     included → the new question.

   ```rust
   Question {
       code: QuestionCode::CompositionUnclear,
       subject: "composition".to_string(),
       message: "luria cannot tell from the text whether the substitutions are separate variants or one combined construct".to_string(),
   }
   ```
   With one mutation nothing changes: no composition question, and the
   operation is `edit`.
3. Step 4 builds `Variants` from the composition step 3d accepted, which
   by then equals the model's value; the existing `composition_of` and its
   guard stay as they are.
4. Update the step 3d comment and the module doc's "no silent choices"
   wording if it describes composition, citing D045.

### 2.4 The documented code table

- `crates/cli/tests/documented_interpret.rs`: add
  `QuestionCode::CompositionUnclear,` right after
  `QuestionCode::CompositionUnstated,`. Change nothing else.
- [docs/specs/cli.md](/specs/cli.md) §10.6: add one row right after the `composition_unstated`
  row, exactly:
  ```
  | `composition_unclear` | Your words say both, or negate it, or the model's reading differs from luria's. |
  ```

## 3. Tests (write first; confirm each new one fails before the change)

### 3.1 `evidence.rs` test module

1. `the_composition_phrases_are_pinned`: each list equals its literal array
   of §2.1, written out in the test.
2. `every_composition_phrase_is_read`: for each phrase of each list,
   written literally, `composition_stated(&format!("Make A42G and W87A {}.", phrase))`
   is `Separate` or `Combined` as its list says.
3. `what_the_text_states_about_composition`:
   - `"Make A42G and W87A."` → `Unstated`;
   - `"Make A42G and W87A as separate variants."` → `Separate`;
   - `"In synA, make both A42G and W87A in a single construct."` → `Combined`;
   - `"Make A42G and W87A separately, then combined."` → `Unclear`;
   - `"Make A42G and W87A, but not separately."` → `Unclear`;
   - `"Not now. Make A42G and W87A individually."` → `Separate`;
   - `"INDIVIDUALLY!"` → `Separate`;
   - `"\u{e9}. Make A42G and W87A together."` → `Combined`.
   - `"For synA, make A42G and W87A, each on its own."` → `Separate` (the
     research note's `each-own` paraphrase).

Disabling proofs: delete `"individual mutants"` → tests 1 and 2 fail; skip
step 3 (negation) → test 3's fifth case fails.

### 3.2 `build.rs` test module

Add `(QuestionCode::CompositionUnclear, "composition_unclear")` to
`every_question_code_is_pinned`, after the `composition_unstated` row.
Other existing tests unchanged. New, with `profile(true, true)`, `syna()`,
`facts()`, `"cds":"synA"`, mutations
`[{"token":"A42G","quote":"A42G","codon":null},{"token":"W87A","quote":"W87A","codon":null}]`,
numbering `"from the beginning of the annotated CDS"`, unsupported `[]`:

1. `a_composition_the_text_does_not_state_is_asked_about`: text
   `"Using synA, design Q5 SDM primers for A42G and W87A. Number residues from the beginning of the annotated CDS."`;
   for each composition `"separate_variants"` and `"combined_construct"`,
   `asked` is `[("composition_unstated", "composition")]`.
2. `a_composition_the_model_reads_differently_is_asked_about`: text
   `"Using synA, design Q5 SDM primers for A42G and W87A as separate variants. Number residues from the beginning of the annotated CDS."`;
   for each composition `"combined_construct"` and `null`, `asked` is
   `[("composition_unclear", "composition")]`.
3. `a_composition_both_read_is_proposed`: the text of test 2 with
   `"separate_variants"` → `Built::Proposed` whose operation is
   `OperationSpec::Variants` with `CompositionSpec::SeparateVariants` and
   members `A42G`, `W87A` in that order.

Disabling proofs: take the model's value when the text is `Unstated` → test
1 fails; skip the disagreement check → test 2 fails.

### 3.3 The corpus

Every case keeps its outcome: `two-separate` ("as separate variants") and
`two-combined` ("in a single construct") agree with their replies, and
`composition-unstated` states none. If one changes, stop and report; do
not edit a fixture.

## 4. Do not

- Add dependencies, or change any file not listed above.
- Change the extraction schema, the prompt, or `luria-nl`.
- Change the numbering lists or functions of `review-m6-b3`.
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
