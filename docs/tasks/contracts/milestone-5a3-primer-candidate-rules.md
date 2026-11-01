# Milestone 5a3: a `rule` on every `InvalidPrimerCandidate`

Status: **Implemented 2026-10-16** by the local worker; accepted on first review.
Planner: Claude. Parent: `docs/tasks/contracts/milestone-5-cli.md` §3.6 and §5 (5a3).
Decisions: D041; review item C8. Follows 5a2
(`docs/tasks/contracts/milestone-5a2-mutation-request-rules.md`, committed), and uses
the same pattern.

## 1. Why

`InvalidPrimerCandidate` is raised at 19 sites in `primer.rs` (the plan said
18) for 13 different reasons, told apart only by `reason` text. This contract
adds a typed `rule`. **No behavior changes:** every site refuses exactly what
it refuses today, with the same `primer` and `reason` text, the same variant
and category.

## 2. The rule enum

In `crates/core/src/error.rs`, directly above
`pub struct InvalidPrimerCandidateProblem`, add:

```rust
/// Which rule refused a primer or pair. Stable: `code()` is part of the
/// CLI's output contract (D041), and renaming one needs approval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimerCandidateRule {
    SpanOverlapsDeletion,
    SpanStraddlesInsertionPoint,
    SpanOverlapsReplacement,
    TextLengthNotSpanWidth,
    ForwardSpanDoesNotCoverReplacement,
    EditTakesNoTail,
    EmptyTail,
    RoleDoesNotMatchSlot,
    PrimersOnDifferentTemplates,
    PairNotOnEditTemplate,
    PairNotAbuttingEdit,
    MinimumOverlapBelowOne,
    TailsDoNotRebuild,
}
```

with `impl PrimerCandidateRule { pub fn code(self) -> &'static str }`, an
exhaustive `match` of literal strings, no `_` arm, written the same way as
`MutationRequestRule::code()`. Codes: `span_overlaps_deletion`,
`span_straddles_insertion_point`, `span_overlaps_replacement`,
`text_length_not_span_width`, `forward_span_does_not_cover_replacement`,
`edit_takes_no_tail`, `empty_tail`, `role_does_not_match_slot`,
`primers_on_different_templates`, `pair_not_on_edit_template`,
`pair_not_abutting_edit`, `minimum_overlap_below_one`,
`tails_do_not_rebuild`.

Change the payload to:

```rust
pub struct InvalidPrimerCandidateProblem {
    pub primer: String,
    pub rule: PrimerCandidateRule,
    pub reason: String,
}
```

Do not change `DomainError::message()`.

## 3. The 19 sites, all in `crates/core/src/primer.rs`

Identify each by function and current `reason` text, not line number. Add
`rule: PrimerCandidateRule::<Rule>,` between `primer` and `reason`, and add
`use crate::error::PrimerCandidateRule;` beside the existing
`use crate::error::InvalidPrimerCandidateProblem;`.

| Function | Current reason (start or key phrase) | Rule |
| --- | --- | --- |
| `construct_primer` | "the span … overlaps the deleted region [" | `SpanOverlapsDeletion` |
| `construct_primer` | "the span … straddles the insertion point at …" | `SpanStraddlesInsertionPoint` |
| `construct_primer` | "the span … overlaps the replaced region [ … a reverse primer of a replacement states the template…" | `SpanOverlapsReplacement` |
| `construct_primer` | "the text is N nucleotides and the span … selects M" + `shift_reason` | `TextLengthNotSpanWidth` |
| `construct_primer` | "a forward primer of a replacement reads the edited artifact over …" | `ForwardSpanDoesNotCoverReplacement` |
| `construct_tailed_primer` | "a 5′ tail carries inserted bases into a gap, and the edit … inserts nothing…" | `EditTakesNoTail` |
| `construct_tailed_primer` | "the replacement is [ … so a tailed primer's annealing portion lies outside the replaced region…" | `SpanOverlapsReplacement` |
| `construct_tailed_primer` | "the insertion point is at …, so a tailed primer's annealing portion may not straddle it…" | `SpanStraddlesInsertionPoint` |
| `construct_tailed_primer` | "there is no tail to place…" | `EmptyTail` |
| `construct_tailed_primer` | "the text is N nucleotides and the span … selects M" | `TextLengthNotSpanWidth` |
| `assemble_pair` | "the forward slot needs a primer whose role is forward" | `RoleDoesNotMatchSlot` |
| `assemble_pair` | "the reverse slot needs a primer whose role is reverse" | `RoleDoesNotMatchSlot` |
| `assemble_pair` | "the two primers were built against different templates: …" | `PrimersOnDifferentTemplates` |
| `assemble_pair` | "the pair was built against … and the edit against …" | `PairNotOnEditTemplate` |
| `assemble_pair` | "the deletion is [ …), so D031 requires the reverse primer to end at …" | `PairNotAbuttingEdit` |
| `assemble_pair` | "the insertion is at point …, so D032 requires …" | `PairNotAbuttingEdit` |
| `assemble_pair` | "the replacement is [ …), so the reverse primer's annealing portion must end at …" | `PairNotAbuttingEdit` |
| `require_design_constraints` | "§2.2 types it as an integer of at least 1; 0 is not a bound" | `MinimumOverlapBelowOne` |
| `require_tails_rebuild` | "the reverse primer's tail reads …, so the two tails do not rebuild the …" | `TailsDoNotRebuild` |

Before editing, grep `InvalidPrimerCandidateProblem {` in `crates/`.
Outside `error.rs` you should find exactly these 19, all in `primer.rs`. If
the count or any reason differs, stop and report.

## 4. Tests

### 4.1 Pinned rule codes (planner-written; add verbatim)

Add to `mod tests` in `error.rs`, after
`every_mutation_request_rule_has_its_pinned_code`:

```rust
    // The rule codes are the CLI's output contract (D041). This pins every
    // one, and checks that no two rules share a code.
    #[test]
    fn every_primer_candidate_rule_has_its_pinned_code() {
        use PrimerCandidateRule::*;
        let cases: [(PrimerCandidateRule, &str); 13] = [
            (SpanOverlapsDeletion, "span_overlaps_deletion"),
            (SpanStraddlesInsertionPoint, "span_straddles_insertion_point"),
            (SpanOverlapsReplacement, "span_overlaps_replacement"),
            (TextLengthNotSpanWidth, "text_length_not_span_width"),
            (
                ForwardSpanDoesNotCoverReplacement,
                "forward_span_does_not_cover_replacement",
            ),
            (EditTakesNoTail, "edit_takes_no_tail"),
            (EmptyTail, "empty_tail"),
            (RoleDoesNotMatchSlot, "role_does_not_match_slot"),
            (PrimersOnDifferentTemplates, "primers_on_different_templates"),
            (PairNotOnEditTemplate, "pair_not_on_edit_template"),
            (PairNotAbuttingEdit, "pair_not_abutting_edit"),
            (MinimumOverlapBelowOne, "minimum_overlap_below_one"),
            (TailsDoNotRebuild, "tails_do_not_rebuild"),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for (rule, code) in cases {
            assert_eq!(rule.code(), code, "{:?}", rule);
            assert!(seen.insert(code), "duplicate code {}", code);
        }
    }
```

### 4.2 The existing `error.rs` test that builds the payload

`every_variant_has_its_pinned_code` constructs `InvalidPrimerCandidateProblem`.
Add only `rule: PrimerCandidateRule::EmptyTail`.

### 4.3 A helper, then convert message-text classification

Add this helper to `mod tests` in `primer.rs`, near the other helpers at the
top of the module:

```rust
    fn primer_candidate_rule(error: DomainError) -> PrimerCandidateRule {
        match error {
            DomainError::InvalidPrimerCandidate(problem) => problem.rule,
            other => panic!("expected InvalidPrimerCandidate, got {:?}", other),
        }
    }
```

Each test below builds `let reason: String = match error { … problem.reason
… }`. Replace the `contains` that identifies *which rule* fired with a rule
assertion; keep each `contains` that checks a *reported value* (an interval,
the edit kind). Because `match error` consumes `error`, take the rule before
it, e.g. `assert_eq!(primer_candidate_rule(error.clone()), …)`.

| Test | Remove | Keep | Rule |
| --- | --- | --- | --- |
| `a_primer_may_not_cover_a_deleted_base` | `contains("deleted")` | `contains("the deleted region [9, 12)")` | `SpanOverlapsDeletion` |
| `a_forward_primer_away_from_the_deletion_edge_is_refused` | none | `contains("the deletion is [9, 12)")` | `PairNotAbuttingEdit` |
| `a_reverse_primer_away_from_the_deletion_edge_is_refused` | none | `contains("the deletion is [9, 12)")` | `PairNotAbuttingEdit` |
| `an_untailed_primer_may_not_read_across_the_insertion_point` | `contains("insertion point")` | | `SpanStraddlesInsertionPoint` |
| `a_tail_is_refused_for_a_substitution` | `contains("5′ tail")` | | `EditTakesNoTail` |
| `tails_that_do_not_rebuild_the_insertion_are_refused` | none | `contains("rebuild the insertion")` | `TailsDoNotRebuild` |
| `a_replacement_reverse_primer_may_not_cover_a_replaced_base` | `contains("replaced")` | `contains("the replaced region [9, 12)")` | `SpanOverlapsReplacement` |
| `tails_that_do_not_rebuild_the_replacement_are_refused` | none | `contains("rebuild the replacement")` | `TailsDoNotRebuild` |
| `a_replacement_pair_must_sit_at_the_regions_edges` | none | `contains("the replacement is [9, 12)")` | `PairNotAbuttingEdit` |
| `a_replacement_forward_primer_must_cover_the_whole_region` | `contains("covers the replaced region")` | | `ForwardSpanDoesNotCoverReplacement` |
| `a_tailed_replacement_primer_may_not_anneal_over_the_region` | `contains("replaced region")` | `contains("the replacement is [9, 12)")` | `SpanOverlapsReplacement` |
| `a_tailed_insertion_primer_may_not_straddle_the_point` | `contains("straddle")` | | `SpanStraddlesInsertionPoint` |

If the `reason` binding is left unused after a removal, remove that binding
and its `match` too; nothing else.

Leave `tail_reason()` assertions (`"part of the replacement"`, `"carries the
insertion"`, `"does not anneal"`) alone: they are not errors.

### 4.4 Strengthen tests that only match the variant

Keep each existing assertion and add a rule assertion:

| Test | Rule |
| --- | --- |
| `a_slot_holding_the_wrong_role_is_rejected` | `RoleDoesNotMatchSlot` |
| `h1_rejects_a_bound_of_zero` | `MinimumOverlapBelowOne` |
| `the_annealing_length_of_a_tailed_primer_must_match_its_span` | `TextLengthNotSpanWidth` |
| `an_annealing_portion_away_from_the_insertion_point_is_refused` | `PairNotAbuttingEdit` |
| `an_inline_replacement_pair_with_a_tailed_reverse_primer_is_refused` | `PairNotAbuttingEdit` |

**If any test in 4.3 or 4.4 produces a rule other than the one listed, do not
change the expectation or the site's rule. Stop and report the test, the
expected rule and the rule observed.**

### 4.5 New tests (planner-written; add verbatim)

Add after `the_annealing_length_of_a_tailed_primer_must_match_its_span`:

```rust
    // 5a3: an untailed primer whose text is not its span's width. The text
    // is 9 letters and [0, 20) selects 20.
    #[test]
    fn an_untailed_primer_whose_text_is_not_its_span_width_is_refused() {
        let error: DomainError = construct_primer(
            &seq_a().clone(),
            edit().clone(),
            PrimerRole::Forward,
            PrimerPlacement::Inline,
            "ATGGCTTGG".to_string(),
            at(0, 20),
        )
        .unwrap_err();
        assert!(error.category() == ErrorCategory::MalformedInput);
        assert_eq!(
            primer_candidate_rule(error),
            PrimerCandidateRule::TextLengthNotSpanWidth
        );
    }

    // 5a3: a tailed primer with nothing in its tail. The insertion and the
    // annealing portion are the ones the accepted tailed tests use, so only
    // the empty tail is wrong.
    #[test]
    fn a_tailed_primer_with_an_empty_tail_is_refused() {
        let result: MutationResult = insertion(12, "T", "C", "GGG");
        let error: DomainError =
            tailed_forward(result.clone(), "", "CAGCAGCAGC", 12, 22).unwrap_err();
        assert!(error.category() == ErrorCategory::MalformedInput);
        assert_eq!(primer_candidate_rule(error), PrimerCandidateRule::EmptyTail);
    }
```

If either fails, stop and report; do not alter inputs or expected rules.

### 4.6 New tests (worker-written, from these exact scenarios)

Two rules in `assemble_pair` are not reached by any test. Write one test for
each, after `a_slot_holding_the_wrong_role_is_rejected`, using the existing
helpers (`seq_a`, `seq_b`, `edit`, `forward_primer`, `reverse_primer`, `at`,
`construct_primer`, and the `mutation` API the module already imports).

1. `primers_built_against_different_templates_are_refused`:
   - the forward primer is `forward_primer(12, 30, "CAGCAGCAGCAGCAGCAG")`,
     on `seq_a` with `edit()`, as in `h1_rejects_a_bound_of_zero`;
   - the reverse primer is built with `construct_primer` on `seq_b`, role
     `Reverse`, placement `Inline`, against a single-nucleotide substitution
     on `seq_b` placed inside the reverse span. Pick the span, the
     substitution and the primer text yourself, and state in a comment how
     each letter was derived from `seq_b`'s literal text (`seq_b` is
     `ATGGCTTGGAATCAGCAGCAGCAGCAGCAGCAG`);
   - `assemble_pair(forward, reverse, edit())` must fail with
     `PrimersOnDifferentTemplates`, category `MalformedInput`.
2. `a_pair_built_against_another_template_than_the_edit_is_refused`:
   - both primers built with `construct_primer` on `seq_b` against the same
     `seq_b` substitution, forward and reverse, each with a derivation
     comment;
   - `assemble_pair(forward, reverse, edit())` (an edit on `seq_a`) must fail
     with `PairNotOnEditTemplate`, category `MalformedInput`.

If either scenario fails earlier with a different error (for example while
building a primer), or cannot be built without a new helper of more than
about 15 lines, stop and report what you found instead of changing
production code.

### 4.7 Coverage

After this contract every one of the 13 rules is asserted by at least one
test. In the handoff, list each rule with the test(s) that assert it.

## 5. Do not

- Edit any file other than `crates/core/src/error.rs` and
  `crates/core/src/primer.rs`.
- Change any `primer` or `reason` text, any message, variant, category, or
  which input is refused.
- Change integration tests under `crates/core/tests/` or any fixture.
- Remove a `contains` listed under "Keep", or weaken any other assertion.
- Bump an algorithm version. Behavior is unchanged.
- Add dependencies.

## 6. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`
- `grep -rn "InvalidPrimerCandidateProblem {" crates/ | grep -v "pub struct" | wc -l`
  must print 20: the 19 sites plus the 1 test in `error.rs`.

## 7. Handoff

Report the changed files, the exact commands and results, the rule-to-test
table from 4.7, the derivation of the 4.6 test inputs, and anything in §3 or
§4 that did not match the code as found.
