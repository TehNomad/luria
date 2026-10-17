# Milestone 5a1: a stable code for every `DomainError`

Status: **Implemented 2026-10-16** by the local worker; accepted on first review.
Planner: Claude. Parent: `docs/tasks/milestone-5-cli.md` §3.6 and §5 (5a1).
Decisions: D041 (the CLI's JSON output carries each error's `code`); review
item C8 (errors must be classifiable without message text).

## 1. Why

The CLI will print every failure as JSON with a `code` field. Consumers (the
Milestone 6 adapter, scripts, tests) branch on that code, so it has to be a
fixed string per variant that never depends on message text. This contract
adds the codes only. It does not touch `InvalidMutationRequest` or
`InvalidPrimerCandidate` payloads; those get rule enums in 5a2 and 5a3.

## 2. Required change

File: `crates/core/src/error.rs` only.

Add one method to `impl DomainError`, placed directly after `category()`:

```rust
/// A stable machine-readable code: the variant name in snake_case.
/// Codes are part of the CLI's output contract; renaming one needs approval.
pub fn code(&self) -> &'static str
```

It is an exhaustive `match` with one arm per variant (no `_` wildcard, so a
new variant fails to compile until it is given a code). The codes are exactly
the variant names converted to snake_case:

| Variant | Code |
| --- | --- |
| `EmptyIdentifier` | `empty_identifier` |
| `InvalidIdentifierCharacters` | `invalid_identifier_characters` |
| `EmptySequenceContent` | `empty_sequence_content` |
| `InvalidSequenceCharacters` | `invalid_sequence_characters` |
| `NegativeCoordinate` | `negative_coordinate` |
| `ReversedInterval` | `reversed_interval` |
| `EmptyInterval` | `empty_interval` |
| `CoordinateOutOfBounds` | `coordinate_out_of_bounds` |
| `SequenceReferenceMismatch` | `sequence_reference_mismatch` |
| `FeatureWithoutIntervals` | `feature_without_intervals` |
| `IntervalsOutOfOrder` | `intervals_out_of_order` |
| `OverlappingIntervals` | `overlapping_intervals` |
| `UnsupportedFeatureStructure` | `unsupported_feature_structure` |
| `MissingCdsAttribute` | `missing_cds_attribute` |
| `IncompleteCds` | `incomplete_cds` |
| `UnsupportedGeneticCode` | `unsupported_genetic_code` |
| `CdsLengthNotDivisibleByThree` | `cds_length_not_divisible_by_three` |
| `AmbiguousCodon` | `ambiguous_codon` |
| `InvalidCodon` | `invalid_codon` |
| `InternalStopCodon` | `internal_stop_codon` |
| `InvalidResiduePosition` | `invalid_residue_position` |
| `ResiduePositionOutsideCds` | `residue_position_outside_cds` |
| `ResiduePositionWrongCds` | `residue_position_wrong_cds` |
| `UnsupportedNumberingScheme` | `unsupported_numbering_scheme` |
| `InvalidMutationRequest` | `invalid_mutation_request` |
| `UnspecifiedCodonPolicy` | `unspecified_codon_policy` |
| `UnsupportedMutationPolicy` | `unsupported_mutation_policy` |
| `UnsupportedComposition` | `unsupported_composition` |
| `ExpectedNucleotideMismatch` | `expected_nucleotide_mismatch` |
| `ExpectedResidueMismatch` | `expected_residue_mismatch` |
| `TargetOutsideFeature` | `target_outside_feature` |
| `MutationVerificationFailure` | `mutation_verification_failure` |
| `InvalidPrimerCandidate` | `invalid_primer_candidate` |
| `PrimerTextAtSpanMismatch` | `primer_text_at_span_mismatch` |
| `PrimerAnnealingNotTemplate` | `primer_annealing_not_template` |
| `PrimerPartition` | `primer_partition` |
| `AnnealingOverlapBelowBound` | `annealing_overlap_below_bound` |
| `PrimerPairCoverage` | `primer_pair_coverage` |
| `PrimerOrientationConflict` | `primer_orientation_conflict` |
| `UnsupportedPrimerPlacement` | `unsupported_primer_placement` |
| `InvalidPrimerDesignRequest` | `invalid_primer_design_request` |
| `UnsupportedPrimerDesignScope` | `unsupported_primer_design_scope` |
| `NoPrimerDesignPossible` | `no_primer_design_possible` |
| `NoPrimerDesignFits` | `no_primer_design_fits` |
| `InvalidBufferModel` | `invalid_buffer_model` |
| `TmParametersUnavailable` | `tm_parameters_unavailable` |
| `DegenerateExpansionTooLarge` | `degenerate_expansion_too_large` |

That is 47 variants. If `error.rs` has a variant not in this table, or the
table names one that does not exist, stop and report; do not invent a code.

## 3. Test (written by the planner; add it verbatim)

Add this test to the existing `mod tests` in `error.rs`, after the last test.
It builds one value of every variant, so it also fails to compile if a
variant is added without updating it.

```rust
    // The codes are the CLI's output contract (D041). This pins every one,
    // and checks that no two variants share a code.
    #[test]
    fn every_variant_has_its_pinned_code() {
        let s = || "x".to_string();
        let cases: Vec<(DomainError, &str)> = vec![
            (DomainError::EmptyIdentifier, "empty_identifier"),
            (
                DomainError::InvalidIdentifierCharacters(InvalidIdentifierProblem {
                    identifier: s(),
                    reason: s(),
                }),
                "invalid_identifier_characters",
            ),
            (
                DomainError::EmptySequenceContent(EmptySequenceProblem { sequence: s() }),
                "empty_sequence_content",
            ),
            (
                DomainError::InvalidSequenceCharacters(InvalidSequenceCharactersProblem {
                    sequence: s(),
                    position: 0,
                    characters: s(),
                }),
                "invalid_sequence_characters",
            ),
            (
                DomainError::NegativeCoordinate(NegativeCoordinateProblem {
                    sequence: s(),
                    start: -1,
                    end: 0,
                }),
                "negative_coordinate",
            ),
            (
                DomainError::ReversedInterval(ReversedIntervalProblem {
                    sequence: s(),
                    start: 2,
                    end: 1,
                }),
                "reversed_interval",
            ),
            (
                DomainError::EmptyInterval(EmptyIntervalProblem {
                    sequence: s(),
                    position: 0,
                }),
                "empty_interval",
            ),
            (
                DomainError::CoordinateOutOfBounds(CoordinateOutOfBoundsProblem {
                    sequence: s(),
                    start: 0,
                    end: 2,
                    length: 1,
                }),
                "coordinate_out_of_bounds",
            ),
            (
                DomainError::SequenceReferenceMismatch(SequenceReferenceMismatchProblem {
                    expected_identifier: s(),
                    actual_identifier: s(),
                    expected_digest: s(),
                    actual_digest: s(),
                    expected_topology: SequenceTopology::Linear,
                    actual_topology: SequenceTopology::Linear,
                }),
                "sequence_reference_mismatch",
            ),
            (
                DomainError::FeatureWithoutIntervals(FeatureWithoutIntervalsProblem {
                    feature: s(),
                    sequence: s(),
                }),
                "feature_without_intervals",
            ),
            (
                DomainError::IntervalsOutOfOrder(IntervalsOutOfOrderProblem {
                    feature: s(),
                    sequence: s(),
                }),
                "intervals_out_of_order",
            ),
            (
                DomainError::OverlappingIntervals(OverlappingIntervalsProblem {
                    feature: s(),
                    sequence: s(),
                    start: 0,
                    end: 1,
                }),
                "overlapping_intervals",
            ),
            (
                DomainError::UnsupportedFeatureStructure(UnsupportedFeatureStructureProblem {
                    feature: s(),
                    sequence: s(),
                    interval_count: 2,
                    reason: s(),
                }),
                "unsupported_feature_structure",
            ),
            (
                DomainError::MissingCdsAttribute(MissingCdsAttributeProblem {
                    cds: s(),
                    attribute: s(),
                    value: None,
                }),
                "missing_cds_attribute",
            ),
            (
                DomainError::IncompleteCds(IncompleteCdsProblem {
                    cds: s(),
                    completeness: s(),
                }),
                "incomplete_cds",
            ),
            (
                DomainError::UnsupportedGeneticCode(UnsupportedGeneticCodeProblem {
                    cds: s(),
                    genetic_code: s(),
                }),
                "unsupported_genetic_code",
            ),
            (
                DomainError::CdsLengthNotDivisibleByThree(CdsLengthNotDivisibleByThreeProblem {
                    cds: s(),
                    length: 4,
                }),
                "cds_length_not_divisible_by_three",
            ),
            (
                DomainError::AmbiguousCodon(AmbiguousCodonProblem {
                    cds: s(),
                    codon_number: 1,
                    codon: s(),
                }),
                "ambiguous_codon",
            ),
            (
                DomainError::InvalidCodon(InvalidCodonProblem {
                    cds: s(),
                    codon_number: 1,
                    codon: s(),
                }),
                "invalid_codon",
            ),
            (
                DomainError::InternalStopCodon(InternalStopCodonProblem {
                    cds: s(),
                    codon_number: 1,
                    codon: s(),
                }),
                "internal_stop_codon",
            ),
            (
                DomainError::InvalidResiduePosition(ResiduePositionProblem {
                    cds: s(),
                    number: 0,
                }),
                "invalid_residue_position",
            ),
            (
                DomainError::ResiduePositionOutsideCds(ResiduePositionRangeProblem {
                    cds: s(),
                    number: 9,
                    residue_count: 5,
                }),
                "residue_position_outside_cds",
            ),
            (
                DomainError::ResiduePositionWrongCds(CdsMismatchProblem {
                    expected: s(),
                    actual: s(),
                }),
                "residue_position_wrong_cds",
            ),
            (
                DomainError::UnsupportedNumberingScheme(UnsupportedNumberingProblem {
                    cds: s(),
                    numbering: s(),
                }),
                "unsupported_numbering_scheme",
            ),
            (
                DomainError::InvalidMutationRequest(InvalidMutationRequestProblem {
                    request: s(),
                    reason: s(),
                }),
                "invalid_mutation_request",
            ),
            (
                DomainError::UnspecifiedCodonPolicy(UnspecifiedCodonPolicyProblem { cds: s() }),
                "unspecified_codon_policy",
            ),
            (
                DomainError::UnsupportedMutationPolicy(UnsupportedMutationPolicyProblem {
                    policy: s(),
                }),
                "unsupported_mutation_policy",
            ),
            (
                DomainError::UnsupportedComposition(UnsupportedCompositionProblem {
                    composition: s(),
                    variant_count: 2,
                    reason: s(),
                }),
                "unsupported_composition",
            ),
            (
                DomainError::ExpectedNucleotideMismatch(ExpectedNucleotideMismatchProblem {
                    sequence: s(),
                    position: 0,
                    expected: s(),
                    actual: s(),
                }),
                "expected_nucleotide_mismatch",
            ),
            (
                DomainError::ExpectedResidueMismatch(ExpectedResidueMismatchProblem {
                    cds: s(),
                    number: 1,
                    expected: s(),
                    actual: s(),
                }),
                "expected_residue_mismatch",
            ),
            (
                DomainError::TargetOutsideFeature(TargetOutsideFeatureProblem {
                    feature: s(),
                    sequence: s(),
                    start: 0,
                    end: 1,
                }),
                "target_outside_feature",
            ),
            (
                DomainError::MutationVerificationFailure(MutationVerificationFailureProblem {
                    result: s(),
                    reason: s(),
                }),
                "mutation_verification_failure",
            ),
            (
                DomainError::InvalidPrimerCandidate(InvalidPrimerCandidateProblem {
                    primer: s(),
                    reason: s(),
                }),
                "invalid_primer_candidate",
            ),
            (
                DomainError::PrimerTextAtSpanMismatch(PrimerTextAtSpanMismatchProblem {
                    primer: s(),
                    span: s(),
                    expected: s(),
                    actual: s(),
                }),
                "primer_text_at_span_mismatch",
            ),
            (
                DomainError::PrimerAnnealingNotTemplate(PrimerAnnealingNotTemplateProblem {
                    primer: s(),
                    position: 0,
                    expected: s(),
                    actual: s(),
                }),
                "primer_annealing_not_template",
            ),
            (
                DomainError::PrimerPartition(PrimerPartitionProblem {
                    primer: s(),
                    reason: s(),
                }),
                "primer_partition",
            ),
            (
                DomainError::AnnealingOverlapBelowBound(AnnealingOverlapBelowBoundProblem {
                    primer: s(),
                    overlap: 1,
                    bound: 2,
                }),
                "annealing_overlap_below_bound",
            ),
            (
                DomainError::PrimerPairCoverage(PrimerPairCoverageProblem {
                    pair: s(),
                    changed: s(),
                    uncovered: s(),
                }),
                "primer_pair_coverage",
            ),
            (
                DomainError::PrimerOrientationConflict(PrimerOrientationConflictProblem {
                    forward: s(),
                    reverse: s(),
                }),
                "primer_orientation_conflict",
            ),
            (
                DomainError::UnsupportedPrimerPlacement(UnsupportedPrimerPlacementProblem {
                    placement: s(),
                    reason: s(),
                }),
                "unsupported_primer_placement",
            ),
            (
                DomainError::InvalidPrimerDesignRequest(InvalidPrimerDesignRequestProblem {
                    field: s(),
                    reason: s(),
                }),
                "invalid_primer_design_request",
            ),
            (
                DomainError::UnsupportedPrimerDesignScope(UnsupportedPrimerDesignScopeProblem {
                    policy: s(),
                    reason: s(),
                }),
                "unsupported_primer_design_scope",
            ),
            (
                DomainError::NoPrimerDesignPossible(NoPrimerDesignPossibleProblem {
                    template: s(),
                    template_length: 10,
                    smallest_pair: 20,
                }),
                "no_primer_design_possible",
            ),
            (
                DomainError::NoPrimerDesignFits(NoPrimerDesignFitsProblem {
                    template: s(),
                    edit_start: 0,
                    edit_end: 1,
                    obstacles: Vec::new(),
                }),
                "no_primer_design_fits",
            ),
            (
                DomainError::InvalidBufferModel(InvalidBufferModelProblem {
                    field: s(),
                    reason: s(),
                }),
                "invalid_buffer_model",
            ),
            (
                DomainError::TmParametersUnavailable(TmParametersUnavailableProblem {
                    function: s(),
                    aspect: s(),
                    reason: s(),
                }),
                "tm_parameters_unavailable",
            ),
            (
                DomainError::DegenerateExpansionTooLarge(DegenerateExpansionTooLargeProblem {
                    replacement: s(),
                }),
                "degenerate_expansion_too_large",
            ),
        ];
        assert_eq!(cases.len(), 47);
        let mut seen = std::collections::BTreeSet::new();
        for (error, code) in &cases {
            assert_eq!(error.code(), *code, "{:?}", error);
            assert!(seen.insert(error.code()), "duplicate code {}", error.code());
            assert!(code
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'));
        }
    }
```

`cargo fmt` may reflow this test; that is fine. Do not change its content.

## 4. Do not

- Edit any file other than `crates/core/src/error.rs`.
- Change any existing variant, payload, category, message or test.
- Add dependencies, derive macros or a string-conversion helper that
  computes codes at run time. The codes are literal strings in the `match`.
- Use a `_` wildcard arm in `code()`.

## 5. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 6. Handoff

Report the changed file, the exact commands run with their results, and
anything in §2 that did not match `error.rs` as found.
