# Review M6 B-3, B-9: the numbering veto reads the user's sentence (D045)

Status: **Implemented 2026-10-20** by the local worker; accepted on planner review.
Planner: Claude. Parent: `docs/tasks/review-post-m6.md` B-3,
B-9. Owner answers, 2026-10-20: "Split by word kind", then "Split the
origin list" (see the decisions table there).

Files (no others):
- `crates/cli/src/interpret/evidence.rs`
- `crates/cli/src/interpret/build.rs`

## 1. Defect

`numbering_from_cds_start` (`evidence.rs`) applies `NUMBERING_VETO` to the
model's numbering words only. Text "In synA, change A42G. Do not number
residues from the start of the CDS; use UniProt numbering instead." with a
reply `"numbering": "from the start of the CDS"` is proposed: the quote is
in the text, and "not" and "UniProt" are outside it.

B-9: seven of the eighteen veto words and two of the nine origin phrases
can be deleted with every test green.

## 2. Required change

### 2.1 `evidence.rs`: three lists instead of `NUMBERING_VETO`

Replace `NUMBERING_VETO` with three public constants. Together they hold
exactly today's eighteen words; none is added or removed.

```rust
/// Negation and contrast: a veto in the sentence that holds the numbering
/// words. `t` is the end of a contraction such as "isn't" once normalized.
pub const NUMBERING_NEGATION: [&str; 8] =
    ["not", "never", "no", "t", "except", "instead", "rather", "but"];

/// Things a residue can be numbered along that are also ordinary nouns
/// ("in a single construct", "a His tag"): a veto in the sentence that
/// holds the numbering words only.
pub const NUMBERING_NOUNS: [&str; 4] = ["plasmid", "construct", "vector", "tag"];

/// Other numbering schemes: a veto anywhere in the text.
pub const NUMBERING_SCHEMES: [&str; 6] =
    ["mature", "precursor", "signal", "uniprot", "pdb", "structure"];
```

`numbering_from_cds_start(words)` keeps its signature and its behavior:
its veto loop runs over all three lists.

### 2.2 `evidence.rs`: a new public function

```rust
/// Whether the user's `text` states numbering from the start of the CDS
/// through its numbering `words` (D045): `numbering_from_cds_start(words)`,
/// no word of `NUMBERING_SCHEMES` anywhere in `text`, and no word of
/// `NUMBERING_NEGATION` or `NUMBERING_NOUNS` in any sentence of `text`
/// that holds `words`.
pub fn numbering_stated_from_cds_start(text: &str, words: &str) -> bool
```

Exactly:
1. If `numbering_from_cds_start(words)` is false, return false.
2. If `normalized_words(text)` holds any word of `NUMBERING_SCHEMES`,
   return false.
3. Work in chars (`Vec<char>` of `text` and of `words`), never bytes. A
   **boundary** is one of `.` `;` `!` `?` `\n`. For **every** start index
   `i` at which `words`' chars occur in `text`'s chars (overlapping
   occurrences included), the occurrence's **sentence** is the chars from
   just after the last boundary before `i` (or from 0) up to, not
   including, the first boundary at or after `i + words.len()` (or to the
   end). Boundaries inside the occurrence itself stay in the sentence. If
   `normalized_words` of any occurrence's sentence holds a word of
   `NUMBERING_NEGATION` or `NUMBERING_NOUNS`, return false.
4. If `words` occurs nowhere in `text`, return false. (Grounding already
   refuses that case; this keeps the function total and fail-closed.)
5. Otherwise return true.

Update the module doc's fourth bullet to describe both functions and the
three lists. Update the doc of `NUMBERING_VETO`'s replacements as given.

### 2.3 `build.rs`

In step 3e, call `numbering_stated_from_cds_start(text, words.as_str())`
instead of `numbering_from_cds_start(words.as_str())`, and import it
instead. The question and its message are unchanged. Add one sentence to
the step 3e comment: the user's sentence and text are read, not only the
model's words (D045).

## 3. Tests (write first; confirm each new one fails before the change)

### 3.1 `evidence.rs` test module

Existing tests are unchanged, and must pass.

1. `the_numbering_lists_are_pinned`: each of the three constants equals the
   literal array of §2.1 (write the literals in the test; do not build them
   from the constants), and `ORIGIN_PHRASES` equals its nine literal
   phrases as they are in the file today.
2. `every_origin_phrase_is_accepted`: for each of the nine phrases, written
   literally in the test, `numbering_from_cds_start(&format!("Residues are numbered {}.", phrase))`
   is true.
3. `every_veto_word_refuses_the_words_themselves`: for each of the eighteen
   words, written literally, `numbering_from_cds_start(&format!("from the start of the CDS {}", word))`
   is false.
4. `a_negation_or_noun_in_the_numbering_sentence_refuses_it`: for each of
   the twelve words of `NUMBERING_NEGATION` and `NUMBERING_NOUNS`, written
   literally, with
   `text = format!("Change A42G. Numbering from the CDS start, {} here.", word)`,
   `numbering_stated_from_cds_start(&text, "Numbering from the CDS start")`
   is false.
5. `a_negation_or_noun_in_another_sentence_does_not`: for the same twelve,
   with `text = format!("Change A42G, {} here. Numbering from the CDS start.", word)`,
   `numbering_stated_from_cds_start(&text, "Numbering from the CDS start")`
   is true.
6. `a_scheme_word_anywhere_refuses_it`: for each of the six
   `NUMBERING_SCHEMES` words, written literally, with
   `text = format!("Change A42G, {} here. Numbering from the CDS start.", word)`,
   `numbering_stated_from_cds_start(&text, "Numbering from the CDS start")`
   is false.
7. `the_sentence_is_found_around_every_occurrence`:
   - `numbering_stated_from_cds_start("Use numbering from the CDS start. Do not use numbering from the CDS start; never.", "numbering from the CDS start")`
     is false (the second occurrence's sentence holds "not");
   - `numbering_stated_from_cds_start("Change A42G, but only A42G; numbering from the CDS start.", "numbering from the CDS start")`
     is true (`;` ends the sentence holding "but");
   - `numbering_stated_from_cds_start("Change A42G; but numbering from the CDS start.", "numbering from the CDS start")`
     is false ("but" is after the `;`, in the numbering sentence);
   - `numbering_stated_from_cds_start("Change A42G. Numbering from the CDS start.", "Numbering from the CDS start.")`
     is true (a boundary inside the words);
   - `numbering_stated_from_cds_start("Change A42G.", "from the CDS start")`
     is false;
   - `numbering_stated_from_cds_start("\u{e9}. Numbering from the CDS start.", "Numbering from the CDS start")`
     is true (chars, not bytes).

Disabling proofs (once each, run, restore, report):
- delete `"precursor"` from `NUMBERING_SCHEMES` → tests 1 and 6 fail;
- delete `"from the first codon of the cds"` from `ORIGIN_PHRASES` → tests
  1 and 2 fail;
- skip step 2 (schemes anywhere) → test 6 fails;
- skip step 3 → test 4 fails;
- use only the first occurrence in step 3 → test 7's first case fails.

### 3.2 `build.rs` test module

Existing tests unchanged. New, with the existing helpers and
`profile(true, true)`, `syna()`, `facts()`, and the mutation
`{"token":"A42G","quote":"A42G","codon":null}`, `"cds":"synA"`,
composition null, unsupported `[]`:

1. `a_negated_numbering_sentence_is_not_from_the_cds_start`: text
   `"In synA, change A42G. Do not number residues from the start of the CDS."`,
   numbering `"from the start of the CDS"` → `asked` is
   `[("numbering_unsupported", "from the start of the CDS")]`.
2. `another_numbering_scheme_anywhere_is_asked_about`: text
   `"In synA, change A42G, numbering from the CDS start. The residue numbers follow UniProt."`,
   numbering `"numbering from the CDS start"` → `asked` is
   `[("numbering_unsupported", "numbering from the CDS start")]`.
3. `a_negation_elsewhere_still_proposes`: text
   `"In synA, change A42G in the plasmid. Do not change anything else. Numbering from the CDS start."`,
   numbering `"Numbering from the CDS start"` → `Built::Proposed`.

Disabling proof: restore the old call in step 3e → tests 1 and 2 fail.

### 3.3 The corpus

Every corpus case must keep its outcome; the planner has checked each text
(`two-combined`'s "single construct" is in another sentence). If one
changes, stop and report; do not edit a fixture.

## 4. Do not

- Add dependencies, or change any file not listed above.
- Add, remove or respell any veto word or origin phrase.
- Change any fixture or existing test.
- Change the numbering question, its code or its message.

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
