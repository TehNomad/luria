# Milestone 5f4: the details of each error

Status: **Implemented 2026-10-17** by the local worker; accepted on review.
Review note: §3's snippets were shorthand. The real code needs
`impl Serialize for Details`, `serde::ser::SerializeMap`, and `match &field`
for an `Option<String>`. `PrimerPairCoverage(_)` binds nothing, because all
three of its fields are left out and a bound name would be unused.
Planner: Claude. Parent: `docs/tasks/milestone-5-cli.md` §3.5, §3.6 and §5
(5f). Follows 5f3 (designs). Last of the 5f contracts.

## 1. What and why

Plan §3.5 says a failed result's `error` has `code`, `category`, `message`
and `details`, "the payload fields". 5f1 gave `ErrorView` everything but
`details`. This contract adds `details`: for a `DomainError`, the fields of
its problem struct as a JSON object; for an error the CLI raises itself
(`RequestError`), an empty object.

## 2. Requirements this contract applies

- **Plan §3.5:** `details` holds the payload fields. Output is
  deterministic, keys in a fixed order, floats in shortest round-trip form
  and never rounded.
- **Plan §3.5:** intervals are `[start, end]` JSON pairs and never appear
  in prose fields a consumer has to parse. Four core fields hold nothing
  but interval text written by core's `describe` (`PrimerTextAtSpanMismatchProblem.span`,
  `PrimerPairCoverageProblem.pair`, `.changed` and `.uncovered`), so they
  are **left out** of `details`. Their content is still in `message`.
  Changing core to carry them typed is outside this milestone. (Neither
  error can be caused by a request: the CLI never supplies primer text, so
  both mean a bug in core.)
- **5a1–5a3, 5b, 5c1–5c3:** a variant's rule is already the error's `rule`,
  so the six rule fields are left out of `details` too.
- **5f1:** values only from core; every match over a core enum exhaustive
  with no `_` arm.

## 3. The types and functions (in `crates/cli/src/result.rs`)

```rust
/// One value in an error's details. Untagged: `Text` is a JSON string,
/// `Count` and `Signed` are JSON numbers, `Absent` is `null` (an absent
/// `Option`; the key is still present, so each code has one fixed shape),
/// `Obstacles` is an array.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum DetailValue {
    Text(String),
    Count(u64),
    Signed(i64),
    Absent,
    Obstacles(Vec<ObstacleView>),
}

/// An error's details: (key, value) pairs in the problem struct's field
/// order. Serializes as a JSON object in that order, with a hand-written
/// `impl Serialize` that uses `serialize_map`. Not a `serde_json::Value` or
/// `Map`: without serde_json's `preserve_order` feature those sort their
/// keys.
#[derive(Clone, Debug, PartialEq)]
pub struct Details(pub Vec<(&'static str, DetailValue)>);

/// One reason a design did not fit (`DesignObstacle`), with core's
/// `describe()` text as `message`.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum ObstacleView {
    FivePrimeAvailability { available: u64, needed: u64, message: String },
    ThreePrimeAvailability { available: u64, needed: u64, message: String },
    AmbiguousTemplateBase { position: u64, base: String, message: String },
    FivePrimeTmFloorUnreachable { available: u64, reached_celsius: f64, message: String },
    ThreePrimeTmFloorUnreachable { available: u64, reached_celsius: f64, message: String },
}

/// The details of a domain error: one exhaustive match over `DomainError`,
/// no `_` arm.
pub fn error_details(error: &DomainError) -> Details;
```

The mapping, for every variant:
- `EmptyIdentifier` has no payload: `Details(vec![])`.
- Each other variant gives one entry per field of its problem struct, **in
  the struct's declaration order** (`crates/core/src/error.rs`), with the
  key the field's name in camelCase (`codon_number` → `codonNumber`,
  `expected_topology` → `expectedTopology`), except the fields left out in
  §2: the six `rule` fields and the four interval-text fields.
- `String` → `Text`; `u64` → `Count`; `usize` → `Count` (`as u64`);
  `i64` → `Signed`; `SequenceTopology` → `Text(topology.name())`;
  `Option<String>` and `Option<u64>` → the value, or `Absent` for `None`;
  `Vec<DesignObstacle>` → `Obstacles`, one `ObstacleView` per obstacle in
  order, through an exhaustive match over `DesignObstacle` with
  `message: obstacle.describe()`.

`ErrorView` gains one field, **last**, after `path`:

```rust
    pub details: Details,
```

- `domain_error_view` sets it to `error_details(error)`.
- `failure_view`'s `RequestFailure::Request` arm sets it to
  `Details(vec![])`. Its `RequestFailure::Domain` arm already goes through
  `domain_error_view`.

## 4. Tests

### 4.1 Change to one 5f1 test (planner-made)

In `crates/cli/src/result.rs`, test
`an_unreadable_request_gives_the_pinned_failed_document`, the expected
text ends its `error` object with `\"path\": \"$\"\n  }}`. Change that to
`\"path\": \"$\",\n    \"details\": {{}}\n  }}` (a `format!` string, so the
braces are doubled). Change nothing else in that test or in any other
existing test.

### 4.2 `crates/cli/tests/error_details.rs` (planner-written; add verbatim)

`cargo fmt` may reflow it; do not change its content. The expected objects
were generated from the problem structs in `crates/core/src/error.rs` by
the rules in §3, not from luria's output, and the 52 constructors were
checked to compile against core and to give 52 distinct codes.

```rust
//! The `details` of every `DomainError` variant (5f4). Each expected
//! object was generated from the variant's problem struct in
//! `crates/core/src/error.rs` and the contract's rules, not from luria's
//! output: keys are the struct's fields in declaration order, in
//! camelCase; rule fields and the four interval-text fields are left out;
//! an absent `Option` is `null`.

use std::collections::BTreeSet;

use luria_cli::prepare::RequestFailure;
use luria_cli::request::{RequestError, RequestErrorCode};
use luria_cli::result::{error_details, failure_view};
use luria_core::error::*;
use luria_core::sequence::SequenceTopology;

fn cases() -> Vec<(DomainError, &'static str)> {
    vec![
        // EmptyIdentifier
        (
            DomainError::EmptyIdentifier,
            r#"{}"#,
        ),
        // InvalidIdentifierCharacters
        (
            DomainError::InvalidIdentifierCharacters(InvalidIdentifierProblem {
            identifier: "identifier-1".to_string(),
            reason: "reason-2".to_string(),
        }),
            r#"{"identifier":"identifier-1","reason":"reason-2"}"#,
        ),
        // EmptySequenceContent
        (
            DomainError::EmptySequenceContent(EmptySequenceProblem {
            sequence: "sequence-1".to_string(),
        }),
            r#"{"sequence":"sequence-1"}"#,
        ),
        // InvalidSequenceCharacters
        (
            DomainError::InvalidSequenceCharacters(InvalidSequenceCharactersProblem {
            sequence: "sequence-1".to_string(),
            position: 102,
            characters: "characters-3".to_string(),
        }),
            r#"{"sequence":"sequence-1","position":102,"characters":"characters-3"}"#,
        ),
        // NegativeCoordinate
        (
            DomainError::NegativeCoordinate(NegativeCoordinateProblem {
            sequence: "sequence-1".to_string(),
            start: -2,
            end: -3,
        }),
            r#"{"sequence":"sequence-1","start":-2,"end":-3}"#,
        ),
        // ReversedInterval
        (
            DomainError::ReversedInterval(ReversedIntervalProblem {
            sequence: "sequence-1".to_string(),
            start: 102,
            end: 103,
        }),
            r#"{"sequence":"sequence-1","start":102,"end":103}"#,
        ),
        // EmptyInterval
        (
            DomainError::EmptyInterval(EmptyIntervalProblem {
            sequence: "sequence-1".to_string(),
            position: 102,
        }),
            r#"{"sequence":"sequence-1","position":102}"#,
        ),
        // CoordinateOutOfBounds
        (
            DomainError::CoordinateOutOfBounds(CoordinateOutOfBoundsProblem {
            sequence: "sequence-1".to_string(),
            start: 102,
            end: 103,
            length: 104,
        }),
            r#"{"sequence":"sequence-1","start":102,"end":103,"length":104}"#,
        ),
        // SequenceReferenceMismatch
        (
            DomainError::SequenceReferenceMismatch(SequenceReferenceMismatchProblem {
            expected_identifier: "expected_identifier-1".to_string(),
            actual_identifier: "actual_identifier-2".to_string(),
            expected_digest: "expected_digest-3".to_string(),
            actual_digest: "actual_digest-4".to_string(),
            expected_topology: SequenceTopology::Linear,
            actual_topology: SequenceTopology::Circular,
        }),
            r#"{"expectedIdentifier":"expected_identifier-1","actualIdentifier":"actual_identifier-2","expectedDigest":"expected_digest-3","actualDigest":"actual_digest-4","expectedTopology":"linear","actualTopology":"circular"}"#,
        ),
        // FeatureWithoutIntervals
        (
            DomainError::FeatureWithoutIntervals(FeatureWithoutIntervalsProblem {
            feature: "feature-1".to_string(),
            sequence: "sequence-2".to_string(),
        }),
            r#"{"feature":"feature-1","sequence":"sequence-2"}"#,
        ),
        // IntervalsOutOfOrder
        (
            DomainError::IntervalsOutOfOrder(IntervalsOutOfOrderProblem {
            feature: "feature-1".to_string(),
            sequence: "sequence-2".to_string(),
        }),
            r#"{"feature":"feature-1","sequence":"sequence-2"}"#,
        ),
        // OverlappingIntervals
        (
            DomainError::OverlappingIntervals(OverlappingIntervalsProblem {
            feature: "feature-1".to_string(),
            sequence: "sequence-2".to_string(),
            start: 103,
            end: 104,
        }),
            r#"{"feature":"feature-1","sequence":"sequence-2","start":103,"end":104}"#,
        ),
        // UnsupportedFeatureStructure
        (
            DomainError::UnsupportedFeatureStructure(UnsupportedFeatureStructureProblem {
            feature: "feature-1".to_string(),
            sequence: "sequence-2".to_string(),
            interval_count: 103,
            reason: "reason-4".to_string(),
        }),
            r#"{"feature":"feature-1","sequence":"sequence-2","intervalCount":103,"reason":"reason-4"}"#,
        ),
        // MissingCdsAttribute
        (
            DomainError::MissingCdsAttribute(MissingCdsAttributeProblem {
            cds: "cds-1".to_string(),
            attribute: "attribute-2".to_string(),
            value: Some("value-3".to_string()),
        }),
            r#"{"cds":"cds-1","attribute":"attribute-2","value":"value-3"}"#,
        ),
        // IncompleteCds
        (
            DomainError::IncompleteCds(IncompleteCdsProblem {
            cds: "cds-1".to_string(),
            completeness: "completeness-2".to_string(),
        }),
            r#"{"cds":"cds-1","completeness":"completeness-2"}"#,
        ),
        // UnsupportedGeneticCode
        (
            DomainError::UnsupportedGeneticCode(UnsupportedGeneticCodeProblem {
            cds: "cds-1".to_string(),
            genetic_code: "genetic_code-2".to_string(),
        }),
            r#"{"cds":"cds-1","geneticCode":"genetic_code-2"}"#,
        ),
        // CdsLengthNotDivisibleByThree
        (
            DomainError::CdsLengthNotDivisibleByThree(CdsLengthNotDivisibleByThreeProblem {
            cds: "cds-1".to_string(),
            length: 102,
        }),
            r#"{"cds":"cds-1","length":102}"#,
        ),
        // AmbiguousCodon
        (
            DomainError::AmbiguousCodon(AmbiguousCodonProblem {
            cds: "cds-1".to_string(),
            codon_number: 102,
            codon: "codon-3".to_string(),
        }),
            r#"{"cds":"cds-1","codonNumber":102,"codon":"codon-3"}"#,
        ),
        // InvalidCodon
        (
            DomainError::InvalidCodon(InvalidCodonProblem {
            cds: "cds-1".to_string(),
            codon_number: 102,
            codon: "codon-3".to_string(),
        }),
            r#"{"cds":"cds-1","codonNumber":102,"codon":"codon-3"}"#,
        ),
        // InternalStopCodon
        (
            DomainError::InternalStopCodon(InternalStopCodonProblem {
            cds: "cds-1".to_string(),
            codon_number: 102,
            codon: "codon-3".to_string(),
        }),
            r#"{"cds":"cds-1","codonNumber":102,"codon":"codon-3"}"#,
        ),
        // InvalidResiduePosition
        (
            DomainError::InvalidResiduePosition(ResiduePositionProblem {
            cds: "cds-1".to_string(),
            number: -2,
        }),
            r#"{"cds":"cds-1","number":-2}"#,
        ),
        // ResiduePositionOutsideCds
        (
            DomainError::ResiduePositionOutsideCds(ResiduePositionRangeProblem {
            cds: "cds-1".to_string(),
            number: 102,
            residue_count: 103,
        }),
            r#"{"cds":"cds-1","number":102,"residueCount":103}"#,
        ),
        // ResiduePositionWrongCds
        (
            DomainError::ResiduePositionWrongCds(CdsMismatchProblem {
            expected: "expected-1".to_string(),
            actual: "actual-2".to_string(),
        }),
            r#"{"expected":"expected-1","actual":"actual-2"}"#,
        ),
        // UnsupportedNumberingScheme
        (
            DomainError::UnsupportedNumberingScheme(UnsupportedNumberingProblem {
            cds: "cds-1".to_string(),
            numbering: "numbering-2".to_string(),
        }),
            r#"{"cds":"cds-1","numbering":"numbering-2"}"#,
        ),
        // InvalidMutationRequest
        (
            DomainError::InvalidMutationRequest(InvalidMutationRequestProblem {
            request: "request-1".to_string(),
            rule: MutationRequestRule::CodonIsStop,
            reason: "reason-2".to_string(),
        }),
            r#"{"request":"request-1","reason":"reason-2"}"#,
        ),
        // UnspecifiedCodonPolicy
        (
            DomainError::UnspecifiedCodonPolicy(UnspecifiedCodonPolicyProblem {
            cds: "cds-1".to_string(),
        }),
            r#"{"cds":"cds-1"}"#,
        ),
        // UnsupportedMutationPolicy
        (
            DomainError::UnsupportedMutationPolicy(UnsupportedMutationPolicyProblem {
            policy: "policy-1".to_string(),
        }),
            r#"{"policy":"policy-1"}"#,
        ),
        // UnsupportedComposition
        (
            DomainError::UnsupportedComposition(UnsupportedCompositionProblem {
            composition: "composition-1".to_string(),
            variant_count: 102,
            reason: "reason-3".to_string(),
        }),
            r#"{"composition":"composition-1","variantCount":102,"reason":"reason-3"}"#,
        ),
        // ExpectedNucleotideMismatch
        (
            DomainError::ExpectedNucleotideMismatch(ExpectedNucleotideMismatchProblem {
            sequence: "sequence-1".to_string(),
            position: 102,
            expected: "expected-3".to_string(),
            actual: "actual-4".to_string(),
        }),
            r#"{"sequence":"sequence-1","position":102,"expected":"expected-3","actual":"actual-4"}"#,
        ),
        // ExpectedResidueMismatch
        (
            DomainError::ExpectedResidueMismatch(ExpectedResidueMismatchProblem {
            cds: "cds-1".to_string(),
            number: 102,
            expected: "expected-3".to_string(),
            actual: "actual-4".to_string(),
        }),
            r#"{"cds":"cds-1","number":102,"expected":"expected-3","actual":"actual-4"}"#,
        ),
        // TargetOutsideFeature
        (
            DomainError::TargetOutsideFeature(TargetOutsideFeatureProblem {
            feature: "feature-1".to_string(),
            sequence: "sequence-2".to_string(),
            start: 103,
            end: 104,
        }),
            r#"{"feature":"feature-1","sequence":"sequence-2","start":103,"end":104}"#,
        ),
        // MutationVerificationFailure
        (
            DomainError::MutationVerificationFailure(MutationVerificationFailureProblem {
            result: "result-1".to_string(),
            reason: "reason-2".to_string(),
        }),
            r#"{"result":"result-1","reason":"reason-2"}"#,
        ),
        // InvalidPrimerCandidate
        (
            DomainError::InvalidPrimerCandidate(InvalidPrimerCandidateProblem {
            primer: "primer-1".to_string(),
            rule: PrimerCandidateRule::SpanOverlapsDeletion,
            reason: "reason-2".to_string(),
        }),
            r#"{"primer":"primer-1","reason":"reason-2"}"#,
        ),
        // PrimerTextAtSpanMismatch
        (
            DomainError::PrimerTextAtSpanMismatch(PrimerTextAtSpanMismatchProblem {
            primer: "primer-1".to_string(),
            span: "span-2".to_string(),
            expected: "expected-3".to_string(),
            actual: "actual-4".to_string(),
        }),
            r#"{"primer":"primer-1","expected":"expected-3","actual":"actual-4"}"#,
        ),
        // PrimerAnnealingNotTemplate
        (
            DomainError::PrimerAnnealingNotTemplate(PrimerAnnealingNotTemplateProblem {
            primer: "primer-1".to_string(),
            position: 102,
            expected: "expected-3".to_string(),
            actual: "actual-4".to_string(),
        }),
            r#"{"primer":"primer-1","position":102,"expected":"expected-3","actual":"actual-4"}"#,
        ),
        // PrimerPartition
        (
            DomainError::PrimerPartition(PrimerPartitionProblem {
            primer: "primer-1".to_string(),
            reason: "reason-2".to_string(),
        }),
            r#"{"primer":"primer-1","reason":"reason-2"}"#,
        ),
        // AnnealingOverlapBelowBound
        (
            DomainError::AnnealingOverlapBelowBound(AnnealingOverlapBelowBoundProblem {
            primer: "primer-1".to_string(),
            overlap: 102,
            bound: 103,
        }),
            r#"{"primer":"primer-1","overlap":102,"bound":103}"#,
        ),
        // PrimerPairCoverage
        (
            DomainError::PrimerPairCoverage(PrimerPairCoverageProblem {
            pair: "pair-1".to_string(),
            changed: "changed-2".to_string(),
            uncovered: "uncovered-3".to_string(),
        }),
            r#"{}"#,
        ),
        // PrimerOrientationConflict
        (
            DomainError::PrimerOrientationConflict(PrimerOrientationConflictProblem {
            forward: "forward-1".to_string(),
            reverse: "reverse-2".to_string(),
        }),
            r#"{"forward":"forward-1","reverse":"reverse-2"}"#,
        ),
        // UnsupportedPrimerPlacement
        (
            DomainError::UnsupportedPrimerPlacement(UnsupportedPrimerPlacementProblem {
            placement: "placement-1".to_string(),
            reason: "reason-2".to_string(),
        }),
            r#"{"placement":"placement-1","reason":"reason-2"}"#,
        ),
        // InvalidPrimerDesignRequest
        (
            DomainError::InvalidPrimerDesignRequest(InvalidPrimerDesignRequestProblem {
            field: "field-1".to_string(),
            reason: "reason-2".to_string(),
        }),
            r#"{"field":"field-1","reason":"reason-2"}"#,
        ),
        // UnsupportedPrimerDesignScope
        (
            DomainError::UnsupportedPrimerDesignScope(UnsupportedPrimerDesignScopeProblem {
            policy: "policy-1".to_string(),
            reason: "reason-2".to_string(),
        }),
            r#"{"policy":"policy-1","reason":"reason-2"}"#,
        ),
        // NoPrimerDesignPossible
        (
            DomainError::NoPrimerDesignPossible(NoPrimerDesignPossibleProblem {
            template: "template-1".to_string(),
            template_length: 102,
            smallest_pair: 103,
        }),
            r#"{"template":"template-1","templateLength":102,"smallestPair":103}"#,
        ),
        // NoPrimerDesignFits
        (
            DomainError::NoPrimerDesignFits(NoPrimerDesignFitsProblem {
            template: "template-1".to_string(),
            edit_start: 102,
            edit_end: 103,
            obstacles: vec![
                DesignObstacle::FivePrimeAvailability { available: 20, needed: 25 },
                DesignObstacle::ThreePrimeAvailability { available: 4, needed: 10 },
                DesignObstacle::AmbiguousTemplateBase { position: 45, base: "N".to_string() },
                DesignObstacle::FivePrimeTmFloorUnreachable { available: 40, reached_celsius: 51.17 },
                DesignObstacle::ThreePrimeTmFloorUnreachable { available: 12, reached_celsius: 48.5 },
            ],
        }),
            r#"{"template":"template-1","editStart":102,"editEnd":103,"obstacles":[{"kind":"five_prime_availability","available":20,"needed":25,"message":"5' side: 20 template bases before the edit, 25 needed"},{"kind":"three_prime_availability","available":4,"needed":10,"message":"3' side: 4 template bases after the edit, 10 needed"},{"kind":"ambiguous_template_base","position":45,"base":"N","message":"template position 45 is the ambiguity code N"},{"kind":"five_prime_tm_floor_unreachable","available":40,"reachedCelsius":51.17,"message":"5' side: the reverse primer reaches only 51.17 °C using all 40 bases available"},{"kind":"three_prime_tm_floor_unreachable","available":12,"reachedCelsius":48.5,"message":"3' side: the forward primer reaches only 48.50 °C using all 12 bases available"}]}"#,
        ),
        // InvalidBufferModel
        (
            DomainError::InvalidBufferModel(InvalidBufferModelProblem {
            field: "field-1".to_string(),
            reason: "reason-2".to_string(),
        }),
            r#"{"field":"field-1","reason":"reason-2"}"#,
        ),
        // TmParametersUnavailable
        (
            DomainError::TmParametersUnavailable(TmParametersUnavailableProblem {
            function: "function-1".to_string(),
            aspect: "aspect-2".to_string(),
            reason: "reason-3".to_string(),
        }),
            r#"{"function":"function-1","aspect":"aspect-2","reason":"reason-3"}"#,
        ),
        // DegenerateExpansionTooLarge
        (
            DomainError::DegenerateExpansionTooLarge(DegenerateExpansionTooLargeProblem {
            replacement: "replacement-1".to_string(),
        }),
            r#"{"replacement":"replacement-1"}"#,
        ),
        // MalformedFasta
        (
            DomainError::MalformedFasta(MalformedFastaProblem {
            sequence: "sequence-1".to_string(),
            rule: FastaRule::NoRecord,
            line: None,
        }),
            r#"{"sequence":"sequence-1","line":null}"#,
        ),
        // UnusableGenBankLocation
        (
            DomainError::UnusableGenBankLocation(UnusableGenBankLocationProblem {
            location: "location-1".to_string(),
            rule: GenBankLocationRule::Syntax,
            sequence_length: 102,
        }),
            r#"{"location":"location-1","sequenceLength":102}"#,
        ),
        // MalformedGenBank
        (
            DomainError::MalformedGenBank(MalformedGenBankProblem {
            sequence: "sequence-1".to_string(),
            rule: GenBankRule::NoLocus,
            line: Some(102),
        }),
            r#"{"sequence":"sequence-1","line":102}"#,
        ),
        // UnusableGenBankCds
        (
            DomainError::UnusableGenBankCds(UnusableGenBankCdsProblem {
            cds: "cds-1".to_string(),
            rule: GenBankCdsRule::NotFound,
            line: None,
            detail: Some("detail-3".to_string()),
        }),
            r#"{"cds":"cds-1","line":null,"detail":"detail-3"}"#,
        ),
        // GenBankTranslationMismatch
        (
            DomainError::GenBankTranslationMismatch(GenBankTranslationMismatchProblem {
            cds: "cds-1".to_string(),
            line: 102,
            residue: 103,
            stated: None,
            translated: Some("translated-5".to_string()),
        }),
            r#"{"cds":"cds-1","line":102,"residue":103,"stated":null,"translated":"translated-5"}"#,
        ),
    ]
}

#[test]
fn every_variant_gives_its_payload_as_details() {
    for (error, expected) in cases() {
        let actual = serde_json::to_string(&error_details(&error)).unwrap();
        assert_eq!(actual, expected, "details of {}", error.code());
    }
}

#[test]
fn the_cases_cover_all_52_variants() {
    let codes: BTreeSet<&'static str> = cases().iter().map(|(error, _)| error.code()).collect();
    assert_eq!(codes.len(), 52);
}

#[test]
fn a_domain_error_view_carries_its_details() {
    let error = DomainError::CdsLengthNotDivisibleByThree(CdsLengthNotDivisibleByThreeProblem {
        cds: "tet".to_string(),
        length: 1190,
    });
    let view = luria_cli::result::domain_error_view(&error, Some("$.template".to_string()));
    assert_eq!(
        serde_json::to_string(&view.details).unwrap(),
        r#"{"cds":"tet","length":1190}"#
    );
}

#[test]
fn a_request_error_has_empty_details() {
    let failure = RequestFailure::Request(RequestError {
        code: RequestErrorCode::Invalid,
        path: "$.edit".to_string(),
        message: "an edit needs exactly one kind".to_string(),
    });
    let view = failure_view(&failure);
    assert_eq!(serde_json::to_string(&view.details).unwrap(), "{}");
}
```

### 4.3 Proving the rules are load-bearing (AGENTS.md)

Break each rule in `result.rs`, run the named test, confirm it fails,
restore.

| Rule | Test that must fail when it is broken |
| --- | --- |
| every field is carried (drop `actualTopology` from `SequenceReferenceMismatch`) | `every_variant_gives_its_payload_as_details` |
| declaration order (swap `start` and `end` in `NegativeCoordinate`) | `every_variant_gives_its_payload_as_details` |
| `None` is `null`, key kept (leave `None` fields out) | `every_variant_gives_its_payload_as_details` |
| obstacle `reachedCelsius` is not rounded (round it to one decimal) | `every_variant_gives_its_payload_as_details` |
| `domain_error_view` carries details (use `Details(vec![])`) | `a_domain_error_view_carries_its_details` |
| a request error's details are empty (give it any entry) | `a_request_error_has_empty_details` |

## 5. Do not

- Edit files other than `crates/cli/src/result.rs` and
  `crates/cli/tests/error_details.rs` (new). In `result.rs`, change no
  existing test except as §4.1 says.
- Use `serde_json::Value`, `serde_json::Map` or `json!` in `result.rs`
  outside its tests.
- Change `luria-core`, the other CLI modules, any message text or the
  order of `ErrorView`'s existing fields. If something gets in the way,
  stop and report.
- Add dependencies.

## 6. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 7. Handoff

Report the changed and new files, the exact commands and results, the §4.3
results, and anything that did not match this contract.
