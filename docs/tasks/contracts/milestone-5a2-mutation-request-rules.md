---
type: Task Contract
title: "Milestone 5a2: a rule on every InvalidMutationRequest"
description: "InvalidMutationRequest is raised at 30 sites for 20 different reasons, and the only way to tell them apart is the reason text."
status: deprecated
tags: [milestone-5]
---
# Milestone 5a2: a `rule` on every `InvalidMutationRequest`

Status: **Implemented 2026-10-16** by the local worker; accepted on first review.
Planner: Claude. Parent: [docs/tasks/contracts/milestone-5-cli.md](/tasks/contracts/milestone-5-cli.md) §3.6 and §5 (5a2).
Decisions: D041 (the CLI prints each error's code and payload); review item
C8 (`InvalidMutationRequest` is a catch-all told apart only by message text).
Follows 5a1 (`DomainError::code()`, committed).

## 1. Why

`InvalidMutationRequest` is raised at 30 sites for 20 different reasons, and
the only way to tell them apart is the `reason` text. The CLI and the
Milestone 6 adapter need a stable, typed answer to "which rule refused this
request". This contract adds a `rule` field. **No behavior changes:** every
site still refuses exactly what it refuses today, with the same `request`
and `reason` text, the same variant and the same category.

The plan said 26 sites. That counted `mutation.rs` only; `q5.rs` and
`q5_expand.rs` have two each, and they are in scope too.

## 2. The rule enum

In `crates/core/src/error.rs`, directly above
`pub struct InvalidMutationRequestProblem`, add:

```rust
/// Which rule refused a mutation request. Stable: `code()` is part of the
/// CLI's output contract (D041), and renaming one needs approval.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MutationRequestRule {
    CodonIsStop,
    CodonIsAmbiguous,
    CodonNotThreeDnaLetters,
    SuppliedCodonSyntax,
    ResidueNotOneLetter,
    ResidueNotInStandardCode,
    TargetOnOtherSequence,
    SubstitutionNotOneNucleotide,
    ExpectedTextLengthMismatch,
    NothingWritten,
    NoChange,
    WholeSequence,
    InsertionAtSequenceEnd,
    SuppliedCodonWrongResidue,
    NoDefaultCodon,
    CodonOutsideSequence,
    BaseNotOneLetter,
    BaseNotDnaLetter,
    EmptyVariantSet,
    DuplicateVariantName,
}
```

and an `impl MutationRequestRule` with:

```rust
/// The rule's stable code: its name in snake_case.
pub fn code(self) -> &'static str
```

as an exhaustive `match` with literal strings and no `_` arm. The codes are
the names in snake_case: `codon_is_stop`, `codon_is_ambiguous`,
`codon_not_three_dna_letters`, `supplied_codon_syntax`,
`residue_not_one_letter`, `residue_not_in_standard_code`,
`target_on_other_sequence`, `substitution_not_one_nucleotide`,
`expected_text_length_mismatch`, `nothing_written`, `no_change`,
`whole_sequence`, `insertion_at_sequence_end`,
`supplied_codon_wrong_residue`, `no_default_codon`,
`codon_outside_sequence`, `base_not_one_letter`, `base_not_dna_letter`,
`empty_variant_set`, `duplicate_variant_name`.

Then change the payload to:

```rust
pub struct InvalidMutationRequestProblem {
    pub request: String,
    pub rule: MutationRequestRule,
    pub reason: String,
}
```

Do not change `DomainError::message()`; the rule is not printed in it.

## 3. The 30 sites

Identify each site by its function and its current `reason` text, not by
line number. Add `rule: MutationRequestRule::<Rule>,` between `request` and
`reason`. Change nothing else at the site.

| File | Function | Current reason (start of text) | Rule |
| --- | --- | --- | --- |
| `mutation.rs` | `Codon::create` | "the codon is a stop codon…" | `CodonIsStop` |
| `mutation.rs` | `Codon::create` | "the codon stands for more than one residue" | `CodonIsAmbiguous` |
| `mutation.rs` | `Codon::create` | "the codon is not three DNA letters" | `CodonNotThreeDnaLetters` |
| `mutation.rs` | `parse_codon_policy` | "a supplied codon is written supplied:XXX…" | `SuppliedCodonSyntax` |
| `mutation.rs` | `parse_residue` | "a residue is exactly one letter" | `ResidueNotOneLetter` |
| `mutation.rs` | `parse_residue` | "the letter is not a residue of the standard genetic code" | `ResidueNotInStandardCode` |
| `mutation.rs` | `NucleotideMutationIntent::create` | "the target coordinate names a sequence other than the source" | `TargetOnOtherSequence` |
| `mutation.rs` | `NucleotideMutationIntent::create` | "a nucleotide substitution selects exactly one nucleotide" | `SubstitutionNotOneNucleotide` |
| `mutation.rs` | `NucleotideMutationIntent::create` | "the replacement is the nucleotide that is already there" | `NoChange` |
| `mutation.rs` | `NucleotideDeletionIntent::create` | "the target coordinate names a sequence other than the source" | `TargetOnOtherSequence` |
| `mutation.rs` | `NucleotideDeletionIntent::create` | "the expected text is N letters; the region selects M nucleotides" | `ExpectedTextLengthMismatch` |
| `mutation.rs` | `NucleotideInsertionIntent::create` | "the target coordinate names a sequence other than the source" | `TargetOnOtherSequence` |
| `mutation.rs` | `NucleotideInsertionIntent::create` | "an insertion must insert at least one nucleotide" | `NothingWritten` |
| `mutation.rs` | `NucleotideReplacementIntent::create` | "the target coordinate names a sequence other than the source" | `TargetOnOtherSequence` |
| `mutation.rs` | `NucleotideReplacementIntent::create` | "the expected text is N letters; the region selects M nucleotides" | `ExpectedTextLengthMismatch` |
| `mutation.rs` | `NucleotideReplacementIntent::create` | "a replacement must write at least one nucleotide" | `NothingWritten` |
| `mutation.rs` | `NucleotideReplacementIntent::create` | "the replacement is the text that is already there" | `NoChange` |
| `mutation.rs` | `apply_residue` | "the codon the policy chose is the codon already at that residue" | `NoChange` |
| `mutation.rs` | `apply_deletion` | "a deletion may not remove the whole sequence" | `WholeSequence` |
| `mutation.rs` | `apply_insertion` | "an insertion needs a base on each side of the point…" | `InsertionAtSequenceEnd` |
| `mutation.rs` | `apply_replacement` | "a replacement may not replace the whole sequence" | `WholeSequence` |
| `mutation.rs` | `select_codon` | "the supplied codon does not encode the replacement residue" | `SuppliedCodonWrongResidue` |
| `mutation.rs` | `default_codon` | "the residue has no codon in the codon usage table" | `NoDefaultCodon` |
| `mutation.rs` | `codon_positions` | "the residue does not select nucleotides that exist" | `CodonOutsideSequence` |
| `mutation.rs` | `single_dna_base` | "… must be exactly one letter" | `BaseNotOneLetter` |
| `mutation.rs` | `single_dna_base` | "… must be one of A, C, G, T" | `BaseNotDnaLetter` |
| `q5.rs` | `Q5VariantSetRequest::create` | "a variant set must carry at least one variant" | `EmptyVariantSet` |
| `q5.rs` | `Q5VariantSetRequest::create` | "two variants are both named '…'…" | `DuplicateVariantName` |
| `q5_expand.rs` | `expand_replacement` | "a degenerate replacement must write at least one base" | `NothingWritten` |
| `q5_expand.rs` | `expand_replacement` | "every sequence the expansion could write is the text that is already there…" | `NoChange` |

Before editing, grep for `InvalidMutationRequestProblem {` in `crates/` and
`tests/`. Outside `error.rs` you should find exactly these 30. If the count
or any reason text differs from this table, stop and report.

## 4. Tests

### 4.1 Pinned rule codes (planner-written; add verbatim)

Add to `mod tests` in `error.rs`, after `every_variant_has_its_pinned_code`:

```rust
    // The rule codes are the CLI's output contract (D041). This pins every
    // one, and checks that no two rules share a code.
    #[test]
    fn every_mutation_request_rule_has_its_pinned_code() {
        use MutationRequestRule::*;
        let cases: [(MutationRequestRule, &str); 20] = [
            (CodonIsStop, "codon_is_stop"),
            (CodonIsAmbiguous, "codon_is_ambiguous"),
            (CodonNotThreeDnaLetters, "codon_not_three_dna_letters"),
            (SuppliedCodonSyntax, "supplied_codon_syntax"),
            (ResidueNotOneLetter, "residue_not_one_letter"),
            (ResidueNotInStandardCode, "residue_not_in_standard_code"),
            (TargetOnOtherSequence, "target_on_other_sequence"),
            (SubstitutionNotOneNucleotide, "substitution_not_one_nucleotide"),
            (ExpectedTextLengthMismatch, "expected_text_length_mismatch"),
            (NothingWritten, "nothing_written"),
            (NoChange, "no_change"),
            (WholeSequence, "whole_sequence"),
            (InsertionAtSequenceEnd, "insertion_at_sequence_end"),
            (SuppliedCodonWrongResidue, "supplied_codon_wrong_residue"),
            (NoDefaultCodon, "no_default_codon"),
            (CodonOutsideSequence, "codon_outside_sequence"),
            (BaseNotOneLetter, "base_not_one_letter"),
            (BaseNotDnaLetter, "base_not_dna_letter"),
            (EmptyVariantSet, "empty_variant_set"),
            (DuplicateVariantName, "duplicate_variant_name"),
        ];
        let mut seen = std::collections::BTreeSet::new();
        for (rule, code) in cases {
            assert_eq!(rule.code(), code, "{:?}", rule);
            assert!(seen.insert(code), "duplicate code {}", code);
        }
    }
```

### 4.2 Existing tests in `error.rs` that build the payload

Two existing tests construct `InvalidMutationRequestProblem` and will stop
compiling. Add only the `rule` field:
- `mutation_failures_are_classified_by_meaning`:
  `rule: MutationRequestRule::BaseNotDnaLetter` (its reason is "the
  replacement is not a DNA letter");
- `every_variant_has_its_pinned_code`: `rule: MutationRequestRule::NoChange`.

### 4.3 Convert message-text classification to rule assertions

In each test below, the `reason.contains(…)` that identifies *which rule*
fired is replaced by `assert_eq!(problem.rule, MutationRequestRule::<Rule>)`.
Where a test also checks that the message *reports a value* (counts, a
variant name), keep that `contains` and add the rule assertion beside it.

| File | Test | Remove | Keep | Assert rule |
| --- | --- | --- | --- | --- |
| `mutation.rs` | `a_deletion_that_asserts_the_wrong_number_of_bases_is_refused` | none | `contains("2")`, `contains("3")` | `ExpectedTextLengthMismatch` |
| `mutation.rs` | `a_deletion_asserts_only_the_bases_it_removes` | `contains("A, C, G, T")` | | `BaseNotDnaLetter` |
| `mutation.rs` | `an_insertion_of_no_bases_is_refused` | `contains("at least one")` | | `NothingWritten` |
| `mutation.rs` | `an_insertion_inserts_only_plain_bases` | `contains("A, C, G, T")` | | `BaseNotDnaLetter` |
| `mutation.rs` | `an_insertion_at_either_end_of_the_sequence_is_refused` | `contains("a base on each side")` | | `InsertionAtSequenceEnd` |
| `mutation.rs` | `a_replacement_that_asserts_the_wrong_number_of_bases_is_refused` | none | `contains("2")`, `contains("3")` | `ExpectedTextLengthMismatch` |
| `mutation.rs` | `a_replacement_asserts_and_writes_only_plain_bases` | `contains("A, C, G, T")` | | `BaseNotDnaLetter` |
| `mutation.rs` | `a_replacement_of_no_bases_is_refused` | `contains("at least one")` | | `NothingWritten` |
| `mutation.rs` | `a_replacement_that_writes_back_the_text_already_there_is_refused` | `contains("already there")` | | `NoChange` |
| `mutation.rs` | `a_replacement_may_not_replace_the_whole_sequence` | `contains("the whole sequence")` | | `WholeSequence` |
| `q5.rs` | `two_variants_sharing_a_result_name_are_refused` | none | `contains("res-a42g")` | `DuplicateVariantName` |
| `q5_expand.rs` | `an_expansion_with_nothing_left_is_refused` | `contains("already there")` | | `NoChange` |
| `q5_expand.rs` | `an_empty_or_invalid_replacement_text_is_refused` | `contains("at least one")` | | `NothingWritten` |
| `q5_expand.rs` | `a_length_mismatch_is_reported_rather_than_hidden` | none | the `contains("the expected text is 1 letters…")` | `ExpectedTextLengthMismatch` |

Leave every `reason.contains` on other variants (`MutationVerificationFailure`,
`InvalidPrimerCandidate`, `TmParametersUnavailable` and so on) untouched.

### 4.4 Strengthen tests that only match the variant

These tests reach `InvalidMutationRequest` but assert only
`matches!(…, DomainError::InvalidMutationRequest(_))`. Keep each existing
assertion and add a rule assertion for the error it already produces:

| File | Test | Error | Rule |
| --- | --- | --- | --- |
| `mutation.rs` | `a_substitution_writes_and_asserts_only_plain_bases` | both errors in the loop | `BaseNotDnaLetter` |
| `mutation.rs` | `an_unrepresentable_mutation_request_is_malformed_input` | `letter_error` | `BaseNotDnaLetter` |
| | | `stop_error` | `CodonIsStop` |
| | | the `contradictory` error | `SuppliedCodonWrongResidue` |
| `mutation.rs` | `a_deletion_cannot_target_another_sequence` | the error | `TargetOnOtherSequence` |
| `mutation.rs` | `a_deletion_may_not_remove_the_whole_sequence` | the error | `WholeSequence` |
| `mutation.rs` | `an_insertion_cannot_target_another_sequence` | the error | `TargetOnOtherSequence` |
| `mutation.rs` | `an_insertion_asserts_only_plain_bases_beside_the_point` | the error (`"N"` before the point) | `BaseNotDnaLetter` |
| `mutation.rs` | `a_replacement_cannot_target_another_sequence` | the error | `TargetOnOtherSequence` |
| `mutation.rs` | `a_residue_number_of_zero_selects_no_codon` | the error | `CodonOutsideSequence` |
| `mutation.rs` | `a_minus_strand_codon_past_the_cds_end_selects_no_codon` | both errors | `CodonOutsideSequence` |
| `mutation.rs` | `an_insertion_at_either_end_of_a_circular_sequence_is_refused` | both errors | `InsertionAtSequenceEnd` |
| `q5.rs` | `residue_letters_are_validated_when_the_request_is_built` | `stop_error` (`"*"`) and the `lower` error (`"g"`) | `ResidueNotInStandardCode` |
| `q5.rs` | `an_empty_variant_set_is_refused_at_construction` | the error | `EmptyVariantSet` |
| `q5.rs` | `a_member_whose_dna_edit_writes_no_change_is_refused_before_the_set_exists` | the error | `NoChange` |
| `q5_expand.rs` | `a_degenerate_member_cannot_be_built_directly` | the error | `BaseNotDnaLetter` |

**If any test in 4.3 or 4.4 produces a rule other than the one listed, do not
change the table's expectation or the site's rule to make it pass. Stop and
report the test, the expected rule and the rule observed.**

### 4.5 New tests for rules no test reaches (planner-written; add verbatim)

Add to `mod tests` in `mutation.rs`, after
`an_unrepresentable_mutation_request_is_malformed_input`. Import
`MutationRequestRule` in the test module if `use super::*` does not bring it
in.

```rust
    fn mutation_request_rule(error: DomainError) -> MutationRequestRule {
        match error {
            DomainError::InvalidMutationRequest(problem) => problem.rule,
            other => panic!("expected InvalidMutationRequest, got {:?}", other),
        }
    }

    // 5a2: rules that no other test reaches. Each asserts the rule, not the
    // message text. `NNN` is all ambiguity codes; `AC` is two letters.
    #[test]
    fn each_codon_and_letter_refusal_names_its_rule() {
        assert_eq!(
            mutation_request_rule(Codon::create("NNN".to_string()).unwrap_err()),
            MutationRequestRule::CodonIsAmbiguous
        );
        assert_eq!(
            mutation_request_rule(Codon::create("AC".to_string()).unwrap_err()),
            MutationRequestRule::CodonNotThreeDnaLetters
        );
        assert_eq!(
            mutation_request_rule(
                parse_codon_policy("supplied:GCT:GCC".to_string()).unwrap_err()
            ),
            MutationRequestRule::SuppliedCodonSyntax
        );
        assert_eq!(
            mutation_request_rule(parse_residue("AG".to_string()).unwrap_err()),
            MutationRequestRule::ResidueNotOneLetter
        );
    }

    // `seq_a` holds `G` at [3, 4). A two-base target and a two-letter base
    // are each refused by their own rule.
    #[test]
    fn each_substitution_shape_refusal_names_its_rule() {
        let two_bases = NucleotideMutationIntent::create(
            seq_a().reference(),
            at(3, 5),
            "G".to_string(),
            "C".to_string(),
        );
        assert_eq!(
            mutation_request_rule(two_bases.unwrap_err()),
            MutationRequestRule::SubstitutionNotOneNucleotide
        );
        let two_letters = NucleotideMutationIntent::create(
            seq_a().reference(),
            at(3, 4),
            "GG".to_string(),
            "C".to_string(),
        );
        assert_eq!(
            mutation_request_rule(two_letters.unwrap_err()),
            MutationRequestRule::BaseNotOneLetter
        );
    }
```

If any of these fails, or fails to compile because a function named here
has a different signature, stop and report. Do not alter the inputs or
expected rules.

### 4.6 Coverage

After this contract, every rule except `NoDefaultCodon` (unreachable: every
standard residue has a codon in the table) is asserted by at least one test.
In the handoff, list each of the 20 rules with the test(s) that assert it.

## 5. Do not

- Edit any file other than `crates/core/src/error.rs`,
  `crates/core/src/mutation.rs`, `crates/core/src/q5.rs` and
  `crates/core/src/q5_expand.rs`.
- Change any `request` or `reason` text, any message, any variant, any
  category, or which input is refused.
- Change the integration tests under `crates/core/tests/` or any fixture.
  They classify by variant name, which is unchanged.
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
- `grep -rn "InvalidMutationRequestProblem {" crates/ | grep -v "pub struct" | wc -l`
  must print 32: the 30 sites plus the 2 tests in `error.rs`. Report the
  real number.

## 7. Handoff

Report:
- the changed files;
- the exact commands run and their results;
- the rule-to-test table from 4.6;
- anything in §3 or §4 that did not match the code as found.
