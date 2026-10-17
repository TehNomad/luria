# Review B9: the wild-type skip compares against the template, not the request

Status: **Implemented 2026-10-16** by the local worker; accepted after the §8 amendment (the worker stopped on a defect in §2.1).
Planner: Claude. Parent: `docs/tasks/review-post-m4.md` B9.
File: `crates/core/src/q5_expand.rs` only.

## 1. Defect

`expand_replacement` skips a member as wild type when
`*sequence == expected_original`. That compares with the caller's claim
about the region, not with what the template holds there. If the claim is
wrong, the real error gets hidden. For example:

- **A length mismatch.** The region is 3 bases wide, `expected_original` is
  `"A"` and `degenerate` is `"A"`. The only member equals the claim, so it's
  skipped. Then every member is skipped, and the caller gets "the request
  would edit nothing" instead of the length error.
- **Wrong text.** The template holds `GCT`, `expected_original` is `"GAA"`
  and `degenerate` is `"GAA"`. The member is skipped as "wild type" although
  the template doesn't hold it, and the caller again gets "would edit
  nothing".

## 2. Required change

In `expand_replacement`, after the existing empty-`degenerate` check and
before `expand_degenerate`:

1. Validate the claim the same way every member's intent would. Call
   `NucleotideReplacementIntent::create(template.reference(),
   region.clone(), expected_original.clone(), expected_original.clone())?`.
   It returns the errors for a region on another sequence, a non-ACGT letter
   and a length mismatch, in that order. Keep the returned intent as
   `claim`. The second `expected_original` is only there because `create`
   needs a non-empty replacement; nothing is applied.
2. Call `region.within(&template)?`.
3. Read the template's letters over `[region.start(), region.end())`. If
   they differ from `claim.expected_original()`, return
   `DomainError::ExpectedNucleotideMismatch(ExpectedNucleotideMismatchProblem {
   sequence: template.display_name(), position: region.start(), expected:
   claim.expected_original(), actual })`. Build the problem exactly as
   `mutation::apply_replacement` does: read that function, and use the same
   text form for `actual`, uppercase canonical bases.
4. Skip a member when it equals the template's text over the region, which
   after step 3 is also `claim.expected_original()`. Compare with that
   value, not with the raw `expected_original` argument.

Update `expand_replacement`'s doc comment to say:
- the claim is checked against the template before anything is expanded;
- the wild-type member is the template's own text.

Keep the rest of the function as it is.

## 3. Tests: write them first, and confirm 1 and 2 fail before the change

Use the module's existing test helpers and template. For each region you
use, read the template's bases there by hand and write them down in a
comment.

1. `a_length_mismatch_is_reported_rather_than_hidden`:
   - a 3-base region, `expected_original` = `"A"`, `degenerate` = `"A"`;
   - the error is `InvalidMutationRequest`, and its reason contains
     `"the expected text is 1 letters; the region selects 3 nucleotides"`;
   - before the fix, the reason was the "would edit nothing" one.
2. `an_expected_original_the_template_does_not_hold_is_refused`:
   - a 3-base region whose template text is some `XYZ`;
   - `expected_original` and `degenerate` are both a different concrete
     3-base text;
   - the result is `ExpectedNucleotideMismatch`, with `position` = the region
     start, `expected` = the claimed text and `actual` = `XYZ`.
3. `the_wild_type_member_skipped_is_the_template_text`:
   - a correct claim, with a `degenerate` whose expansion includes the
     template text, e.g. one `N` over a 1-base region;
   - exactly that member is skipped (`WildTypeSkipped { sequence }` is the
     template's base), and the other three members remain.
   - This may pass before the change.

## 4. Stop and report if

- any existing test fails. An existing test might, for instance, rely on a
  claim that disagrees with the template;
- `NucleotideReplacementIntent::create` doesn't behave as described in
  §2.1.

## 5. Do not

- Edit any file other than `q5_expand.rs`.
- Change any existing test.
- Add dependencies.

## 6. Checks

The usual cargo checks with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 7. Handoff

Report:
- the changed file;
- which tests failed first, and how;
- the check results;
- any deviation.

## 8. Amendment, 2026-10-16

§2.1 was wrong: `NucleotideReplacementIntent::create` refuses a replacement
equal to its expected text, so passing `expected_original` twice rejects
every well-formed claim. The amended rule: pass `expected_original + "A"` as
the replacement. It is never applied. It is non-empty, valid ACGT, and
strictly longer than the claim, so the equality rule can't fire. The
reference, letter and length checks still run, in their own order. Comment
the reason at the call. Take `expected_original()` from the returned intent;
don't keep the intent itself.
