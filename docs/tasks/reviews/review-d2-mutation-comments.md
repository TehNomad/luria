# Review cleanup D2: comments in `mutation.rs`, `q5_design.rs`, `q5_expand.rs`

Status: **Implemented 2026-10-16** by the local worker; accepted after one review round (two false claims and a vague citation fixed).
Planner: Claude. Parent: `docs/tasks/reviews/review-post-m4.md` §D.

**Comments only.** No code, string, test or behavior change.

## 1. Rules

Follow §2 of `docs/tasks/reviews/review-d1-q5-comments.md` exactly; its 11 rules
apply here unchanged. In short:
- describe the code as it is now;
- keep every D-number, spec section and owner decision that justifies a rule
  still in code;
- one-line item summaries;
- don't touch trailing comments, test modules or `Defensive check:`
  comments;
- make no unverified claims;
- accuracy over length.

Module-doc targets: `mutation.rs` about 30 lines; the other two as short as
the rules allow.

## 2. Facts (verified by the planner)

- **Intents.** `mutation.rs` has five intent types:
  - `NucleotideMutationIntent`
  - `ResidueMutationIntent`
  - `NucleotideDeletionIntent` (D031)
  - `NucleotideInsertionIntent` (D032)
  - `NucleotideReplacementIntent` (D033)

  It also has five matching `MutationResult` variants and five `apply_*`
  operations.
- **`verify`** checks length per kind:
  - substitution: unchanged;
  - deletion: `n − w`;
  - insertion: `n + k`;
  - replacement: `n − w + m`.

  It rebuilds the edited artifact from the record for deletions, insertions
  and replacements. For residue results it checks the recorded annotation
  (exact `Cds` equality), the source's residue and codon, and the edited
  codon. It re-translates **only** residue results (`docs/specs/q5-sdm.md` §8, open
  owner question A5).
- **`require_within_feature`** requires the whole interval inside one CDS
  interval. **`require_point_within_feature`** requires `start < p < end`
  for one interval.
- **`apply_replacement` refuses a region that is the whole sequence.** No
  source records why. Unlike a deletion, it would *not* leave an empty
  artifact, because a replacement writes at least one base.

## 3. Known false or stale statements

**`crates/core/src/mutation.rs`:**
- lines 8 and 17: "Three intents exist" and "All three". There are five
  (§2);
- around line 1328, `apply_replacement`'s doc: "replacing the whole sequence
  would leave an artifact of no bases at all". False. State the rule
  plainly, and say no source records a reason for it;
- `verify`'s doc: it is one long paragraph built up per milestone. Rewrite it
  as a short list of what is checked for each result kind (§2);
- module doc: remove milestone narration.

**`crates/core/src/q5_design.rs` and `crates/core/src/q5_expand.rs`:** remove
milestone and task narration. Check every claim against the code, and in
particular these two in `q5_design.rs`:
- per-member design outcomes;
- a set's edit failure is atomic, but its design failures are per member.

## 4. Guard

Run the §5 guard of `docs/tasks/reviews/review-d1-q5-comments.md`, with these file
arguments:

```
crates/core/src/mutation.rs crates/core/src/q5_design.rs crates/core/src/q5_expand.rs
```

It must print nothing and exit 0.

## 5. Do not

- Edit any other file.
- Change any line that isn't a whole-line comment.
- Touch the test modules.

## 6. Checks

Same as D1 §7: the guard, `cargo fmt --check`, `cargo test --workspace`,
`cargo clippy -D warnings`, `git diff --check` and `git status --short`, with
`TMPDIR=$PWD/target/tmp`.

## 7. Handoff

Report:
- comment line counts before and after, per file;
- each §3 item and how you fixed it;
- any citation you dropped, and why;
- the exact check results.
