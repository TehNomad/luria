//! Structured domain errors.
//!
//! Every failure produced by the core domain model is a `DomainError`.
//! Callers classify failures with `ErrorCategory` and with the concrete
//! enum variant. Message text exists for human reporting only; it must
//! never be the basis for programmatic classification.
//!
//! The three failure classes are kept deliberately distinct:
//!
//! * `MalformedInput`: the request could not be represented at all, for
//!   example a negative coordinate, an empty interval, or a character
//!   outside the declared alphabet.
//! * `BiologicalValidation`: a well-formed request that violates a domain
//!   invariant or the contract of the operation, for example coordinates
//!   outside the referenced sequence, a structure pointing at the wrong
//!   sequence, or an internal stop codon.
//! * `Unsupported`: a well-formed request outside the supported scope of
//!   this milestone, for example a compound feature, an incomplete CDS, or
//!   a genetic code other than the standard one.

use crate::sequence::SequenceTopology;

/// The failure classes the product must keep distinguishable.
#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum ErrorCategory {
    MalformedInput,
    BiologicalValidation,
    Unsupported,
}

/// Payload for a rejected identifier.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidIdentifierProblem {
    pub identifier: String,
    pub reason: String,
}

/// Payload: a sequence record contains letters outside the declared alphabet.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidSequenceCharactersProblem {
    pub sequence: String,
    pub position: u64,
    pub characters: String,
}

/// Payload: a sequence record carries no bases at all.
#[derive(Clone, Debug, PartialEq)]
pub struct EmptySequenceProblem {
    pub sequence: String,
}

/// Payload: signed coordinates that are negative.
#[derive(Clone, Debug, PartialEq)]
pub struct NegativeCoordinateProblem {
    pub sequence: String,
    pub start: i64,
    pub end: i64,
}

/// Payload: an interval whose end precedes its start.
#[derive(Clone, Debug, PartialEq)]
pub struct ReversedIntervalProblem {
    pub sequence: String,
    pub start: u64,
    pub end: u64,
}

/// Payload: an interval that contains no nucleotide positions.
#[derive(Clone, Debug, PartialEq)]
pub struct EmptyIntervalProblem {
    pub sequence: String,
    pub position: u64,
}

/// Payload: an interval that does not fit the referenced sequence.
#[derive(Clone, Debug, PartialEq)]
pub struct CoordinateOutOfBoundsProblem {
    pub sequence: String,
    pub start: u64,
    pub end: u64,
    pub length: u64,
}

/// Payload: a structure references a sequence other than the one supplied.
///
/// The display identifier, the content digest, and the topology are reported
/// so the caller can see which of them disagree.
#[derive(Clone, Debug, PartialEq)]
pub struct SequenceReferenceMismatchProblem {
    pub expected_identifier: String,
    pub actual_identifier: String,
    pub expected_digest: String,
    pub actual_digest: String,
    pub expected_topology: SequenceTopology,
    pub actual_topology: SequenceTopology,
}

/// Payload: a codon contains something outside the DNA alphabet.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidCodonProblem {
    pub cds: String,
    pub codon_number: usize,
    pub codon: String,
}

/// Payload: a residue position that is not a positive one-based number.
#[derive(Clone, Debug, PartialEq)]
pub struct ResiduePositionProblem {
    pub cds: String,
    pub number: i64,
}

/// Payload: a residue position beyond the end of the translated CDS.
#[derive(Clone, Debug, PartialEq)]
pub struct ResiduePositionRangeProblem {
    pub cds: String,
    pub number: u64,
    pub residue_count: u64,
}

/// Payload: a residue position naming a CDS other than the one supplied.
#[derive(Clone, Debug, PartialEq)]
pub struct CdsMismatchProblem {
    pub expected: String,
    pub actual: String,
}

/// Payload: a residue numbering scheme this milestone refuses to apply.
#[derive(Clone, Debug, PartialEq)]
pub struct UnsupportedNumberingProblem {
    pub cds: String,
    pub numbering: String,
}

/// Payload: a feature or CDS with no intervals.
#[derive(Clone, Debug, PartialEq)]
pub struct FeatureWithoutIntervalsProblem {
    pub feature: String,
    pub sequence: String,
}

/// Payload: an interval list that is not in biological order.
#[derive(Clone, Debug, PartialEq)]
pub struct IntervalsOutOfOrderProblem {
    pub feature: String,
    pub sequence: String,
}

/// Payload: an interval list whose members overlap.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlappingIntervalsProblem {
    pub feature: String,
    pub sequence: String,
    pub start: u64,
    pub end: u64,
}

/// Payload: an interval structure this milestone refuses to interpret.
#[derive(Clone, Debug, PartialEq)]
pub struct UnsupportedFeatureStructureProblem {
    pub feature: String,
    pub sequence: String,
    pub interval_count: usize,
    pub reason: String,
}

/// Payload: a required CDS attribute was not supplied, or was supplied with
/// a value that is not one of the recognised values.
///
/// `value` is `None` when the attribute was absent and `Some(text)` when the
/// supplied text was not recognised.
#[derive(Clone, Debug, PartialEq)]
pub struct MissingCdsAttributeProblem {
    pub cds: String,
    pub attribute: String,
    pub value: Option<String>,
}

/// Payload: a CDS that is not explicitly complete.
#[derive(Clone, Debug, PartialEq)]
pub struct IncompleteCdsProblem {
    pub cds: String,
    pub completeness: String,
}

/// Payload: a genetic code outside the supported set.
#[derive(Clone, Debug, PartialEq)]
pub struct UnsupportedGeneticCodeProblem {
    pub cds: String,
    pub genetic_code: String,
}

/// Payload: extracted CDS length is not a multiple of three.
#[derive(Clone, Debug, PartialEq)]
pub struct CdsLengthNotDivisibleByThreeProblem {
    pub cds: String,
    pub length: u64,
}

/// Payload: a codon that cannot be translated without guessing.
#[derive(Clone, Debug, PartialEq)]
pub struct AmbiguousCodonProblem {
    pub cds: String,
    pub codon_number: usize,
    pub codon: String,
}

/// Payload: a stop codon that is not the final codon of the CDS.
#[derive(Clone, Debug, PartialEq)]
pub struct InternalStopCodonProblem {
    pub cds: String,
    pub codon_number: usize,
    pub codon: String,
}

/// Payload: a mutation request that cannot be represented.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidMutationRequestProblem {
    pub request: String,
    pub reason: String,
}

/// Payload: a residue mutation that did not say how to choose the codon.
#[derive(Clone, Debug, PartialEq)]
pub struct UnspecifiedCodonPolicyProblem {
    pub cds: String,
}

/// Payload: a codon-selection policy this milestone refuses to apply.
#[derive(Clone, Debug, PartialEq)]
pub struct UnsupportedMutationPolicyProblem {
    pub policy: String,
}

/// Payload: a composition mode of a multi-variant request that this
/// milestone refuses to execute. `composition` names the mode the caller
/// asked for and `variant_count` how many members it carried, so the
/// refusal says which request it is about; `reason` states why the mode is
/// out of scope. Nothing about the members was executed.
#[derive(Clone, Debug, PartialEq)]
pub struct UnsupportedCompositionProblem {
    pub composition: String,
    pub variant_count: usize,
    pub reason: String,
}

/// Payload: the nucleotide at the target is not the expected original.
#[derive(Clone, Debug, PartialEq)]
pub struct ExpectedNucleotideMismatchProblem {
    pub sequence: String,
    pub position: u64,
    pub expected: String,
    pub actual: String,
}

/// Payload: the translated residue is not the expected original residue.
#[derive(Clone, Debug, PartialEq)]
pub struct ExpectedResidueMismatchProblem {
    pub cds: String,
    pub number: u64,
    pub expected: String,
    pub actual: String,
}

/// Payload: the target coordinate lies outside the scoped feature or CDS.
#[derive(Clone, Debug, PartialEq)]
pub struct TargetOutsideFeatureProblem {
    pub feature: String,
    pub sequence: String,
    pub start: u64,
    pub end: u64,
}

/// Payload: a claimed mutation result does not follow from its source.
#[derive(Clone, Debug, PartialEq)]
pub struct MutationVerificationFailureProblem {
    pub result: String,
    pub reason: String,
}

/// Payload: a proposed primer cannot become a primer record at all.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidPrimerCandidateProblem {
    pub primer: String,
    pub reason: String,
}

/// Payload: primer text is not the sequence the contract says sits at its span.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimerTextAtSpanMismatchProblem {
    pub primer: String,
    pub span: String,
    pub expected: String,
    pub actual: String,
}

/// Payload: an annealing portion is not the wild-type template under its spans.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimerAnnealingNotTemplateProblem {
    pub primer: String,
    pub position: u64,
    pub expected: String,
    pub actual: String,
}

/// Payload: a primer's parts do not partition its text.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimerPartitionProblem {
    pub primer: String,
    pub reason: String,
}

/// Payload: a primer's annealing overlap is below the supplied bound.
#[derive(Clone, Debug, PartialEq)]
pub struct AnnealingOverlapBelowBoundProblem {
    pub primer: String,
    pub overlap: u64,
    pub bound: u64,
}

/// Payload: a pair does not cover the changed region of its edit.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimerPairCoverageProblem {
    pub pair: String,
    pub changed: String,
    pub uncovered: String,
}

/// Payload: the two primers do not state opposite orientations.
#[derive(Clone, Debug, PartialEq)]
pub struct PrimerOrientationConflictProblem {
    pub forward: String,
    pub reverse: String,
}

/// Payload: a primer placement the contract leaves open is requested.
#[derive(Clone, Debug, PartialEq)]
pub struct UnsupportedPrimerPlacementProblem {
    pub placement: String,
    pub reason: String,
}

/// Payload: a primer-design request that cannot be acted on as given, such as
/// a `minAnnealingOverlap` below 1 or a mutation result that was never
/// verified.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidPrimerDesignRequestProblem {
    pub field: String,
    pub reason: String,
}

/// Payload: an edit or template outside the design policy's scope (D029:
/// a contiguous replacement region of at most 3 bases on a linear template).
#[derive(Clone, Debug, PartialEq)]
pub struct UnsupportedPrimerDesignScopeProblem {
    pub policy: String,
    pub reason: String,
}

/// Payload: the template is shorter than the smallest pair the policy can
/// emit, so no placement of any abutting pair fits (D029).
#[derive(Clone, Debug, PartialEq)]
pub struct NoPrimerDesignPossibleProblem {
    pub template: String,
    pub template_length: u64,
    pub smallest_pair: u64,
}

/// One reason a design under D029/D030 does not fit the template.
#[derive(Clone, Debug, PartialEq)]
pub enum DesignObstacle {
    /// Too few template bases 5′ of the edit for the forward 5′ flank plus
    /// the reverse primer.
    FivePrimeAvailability { available: u64, needed: u64 },
    /// Too few template bases 3′ of the edit for the forward 3′ flank.
    ThreePrimeAvailability { available: u64, needed: u64 },
    /// The template holds an ambiguity code where a primer base would be
    /// (H6: no degenerate symbol in a primer).
    AmbiguousTemplateBase { position: u64, base: String },
    /// The reverse primer ran out of template 5′ of the junction before its
    /// Tm reached the 55 °C floor (D030). `reached_celsius` is its Tm at the
    /// longest length available.
    FivePrimeTmFloorUnreachable {
        available: u64,
        reached_celsius: f64,
    },
    /// The forward primer ran out of template 3′ of the edit before its Tm
    /// reached the floor.
    ThreePrimeTmFloorUnreachable {
        available: u64,
        reached_celsius: f64,
    },
}

impl DesignObstacle {
    pub fn describe(&self) -> String {
        match self {
            DesignObstacle::FivePrimeAvailability { available, needed } => format!(
                "5' side: {} template bases before the edit, {} needed",
                available, needed
            ),
            DesignObstacle::ThreePrimeAvailability { available, needed } => format!(
                "3' side: {} template bases after the edit, {} needed",
                available, needed
            ),
            DesignObstacle::AmbiguousTemplateBase { position, base } => format!(
                "template position {} is the ambiguity code {}",
                position, base
            ),
            DesignObstacle::FivePrimeTmFloorUnreachable {
                available,
                reached_celsius,
            } => format!(
                "5' side: the reverse primer reaches only {:.2} °C using all {} bases available",
                reached_celsius, available
            ),
            DesignObstacle::ThreePrimeTmFloorUnreachable {
                available,
                reached_celsius,
            } => format!(
                "3' side: the forward primer reaches only {:.2} °C using all {} bases available",
                reached_celsius, available
            ),
        }
    }
}

/// Payload: a pair of the policy's starting size fits the template, but not
/// around this edit, or not at a Tm the policy accepts. Another geometry might
/// fit; this policy keeps the junction fixed and does not shift (D029, D030).
#[derive(Clone, Debug, PartialEq)]
pub struct NoPrimerDesignFitsProblem {
    pub template: String,
    pub edit_start: u64,
    pub edit_end: u64,
    pub obstacles: Vec<DesignObstacle>,
}

/// Payload: a buffer model that cannot be used, such as a concentration of
/// zero where the Tm equation takes a logarithm.
#[derive(Clone, Debug, PartialEq)]
pub struct InvalidBufferModelProblem {
    pub field: String,
    pub reason: String,
}

/// Payload: the named Tm function has no published parameters for this duplex.
/// The commonest cause is two adjacent mismatches, for which no nearest-
/// neighbour set exists (`tests/fixtures/tm/README.md`).
#[derive(Clone, Debug, PartialEq)]
pub struct TmParametersUnavailableProblem {
    pub function: String,
    pub aspect: String,
    pub reason: String,
}

/// A structured domain failure.
#[derive(Clone, Debug, PartialEq)]
pub enum DomainError {
    EmptyIdentifier,
    InvalidIdentifierCharacters(InvalidIdentifierProblem),
    EmptySequenceContent(EmptySequenceProblem),
    InvalidSequenceCharacters(InvalidSequenceCharactersProblem),
    NegativeCoordinate(NegativeCoordinateProblem),
    ReversedInterval(ReversedIntervalProblem),
    EmptyInterval(EmptyIntervalProblem),
    CoordinateOutOfBounds(CoordinateOutOfBoundsProblem),
    SequenceReferenceMismatch(SequenceReferenceMismatchProblem),
    FeatureWithoutIntervals(FeatureWithoutIntervalsProblem),
    IntervalsOutOfOrder(IntervalsOutOfOrderProblem),
    OverlappingIntervals(OverlappingIntervalsProblem),
    UnsupportedFeatureStructure(UnsupportedFeatureStructureProblem),
    MissingCdsAttribute(MissingCdsAttributeProblem),
    IncompleteCds(IncompleteCdsProblem),
    UnsupportedGeneticCode(UnsupportedGeneticCodeProblem),
    CdsLengthNotDivisibleByThree(CdsLengthNotDivisibleByThreeProblem),
    AmbiguousCodon(AmbiguousCodonProblem),
    InvalidCodon(InvalidCodonProblem),
    InternalStopCodon(InternalStopCodonProblem),
    InvalidResiduePosition(ResiduePositionProblem),
    ResiduePositionOutsideCds(ResiduePositionRangeProblem),
    ResiduePositionWrongCds(CdsMismatchProblem),
    UnsupportedNumberingScheme(UnsupportedNumberingProblem),
    InvalidMutationRequest(InvalidMutationRequestProblem),
    UnspecifiedCodonPolicy(UnspecifiedCodonPolicyProblem),
    UnsupportedMutationPolicy(UnsupportedMutationPolicyProblem),
    UnsupportedComposition(UnsupportedCompositionProblem),
    ExpectedNucleotideMismatch(ExpectedNucleotideMismatchProblem),
    ExpectedResidueMismatch(ExpectedResidueMismatchProblem),
    TargetOutsideFeature(TargetOutsideFeatureProblem),
    MutationVerificationFailure(MutationVerificationFailureProblem),
    InvalidPrimerCandidate(InvalidPrimerCandidateProblem),
    PrimerTextAtSpanMismatch(PrimerTextAtSpanMismatchProblem),
    PrimerAnnealingNotTemplate(PrimerAnnealingNotTemplateProblem),
    PrimerPartition(PrimerPartitionProblem),
    AnnealingOverlapBelowBound(AnnealingOverlapBelowBoundProblem),
    PrimerPairCoverage(PrimerPairCoverageProblem),
    PrimerOrientationConflict(PrimerOrientationConflictProblem),
    UnsupportedPrimerPlacement(UnsupportedPrimerPlacementProblem),
    InvalidPrimerDesignRequest(InvalidPrimerDesignRequestProblem),
    UnsupportedPrimerDesignScope(UnsupportedPrimerDesignScopeProblem),
    NoPrimerDesignPossible(NoPrimerDesignPossibleProblem),
    NoPrimerDesignFits(NoPrimerDesignFitsProblem),
    InvalidBufferModel(InvalidBufferModelProblem),
    TmParametersUnavailable(TmParametersUnavailableProblem),
}

impl ErrorCategory {
    pub fn is_malformed_input(self) -> bool {
        self == ErrorCategory::MalformedInput
    }

    pub fn is_biological_validation(self) -> bool {
        self == ErrorCategory::BiologicalValidation
    }

    pub fn is_unsupported(self) -> bool {
        self == ErrorCategory::Unsupported
    }
}

impl DomainError {
    /// The failure class. Callers branch on this, never on message text.
    pub fn category(&self) -> ErrorCategory {
        match self {
            DomainError::EmptyIdentifier
            | DomainError::InvalidIdentifierCharacters(_)
            | DomainError::EmptySequenceContent(_)
            | DomainError::InvalidSequenceCharacters(_)
            | DomainError::NegativeCoordinate(_)
            | DomainError::ReversedInterval(_)
            | DomainError::EmptyInterval(_)
            | DomainError::FeatureWithoutIntervals(_)
            | DomainError::IntervalsOutOfOrder(_)
            | DomainError::OverlappingIntervals(_)
            | DomainError::MissingCdsAttribute(_)
            | DomainError::InvalidResiduePosition(_)
            | DomainError::InvalidMutationRequest(_)
            | DomainError::UnspecifiedCodonPolicy(_) => ErrorCategory::MalformedInput,
            DomainError::InvalidPrimerCandidate(_)
            | DomainError::InvalidPrimerDesignRequest(_)
            | DomainError::InvalidBufferModel(_) => ErrorCategory::MalformedInput,
            DomainError::CoordinateOutOfBounds(_)
            | DomainError::SequenceReferenceMismatch(_)
            | DomainError::CdsLengthNotDivisibleByThree(_)
            | DomainError::AmbiguousCodon(_)
            | DomainError::InternalStopCodon(_)
            | DomainError::InvalidCodon(_)
            | DomainError::ResiduePositionOutsideCds(_)
            | DomainError::ResiduePositionWrongCds(_)
            | DomainError::ExpectedNucleotideMismatch(_)
            | DomainError::ExpectedResidueMismatch(_)
            | DomainError::TargetOutsideFeature(_)
            | DomainError::MutationVerificationFailure(_)
            | DomainError::PrimerTextAtSpanMismatch(_)
            | DomainError::PrimerAnnealingNotTemplate(_)
            | DomainError::PrimerPartition(_)
            | DomainError::AnnealingOverlapBelowBound(_)
            | DomainError::PrimerPairCoverage(_)
            | DomainError::PrimerOrientationConflict(_)
            | DomainError::NoPrimerDesignPossible(_) => ErrorCategory::BiologicalValidation,
            DomainError::UnsupportedFeatureStructure(_)
            | DomainError::IncompleteCds(_)
            | DomainError::UnsupportedGeneticCode(_)
            | DomainError::UnsupportedNumberingScheme(_)
            | DomainError::UnsupportedMutationPolicy(_)
            | DomainError::UnsupportedComposition(_)
            | DomainError::UnsupportedPrimerPlacement(_)
            | DomainError::UnsupportedPrimerDesignScope(_)
            | DomainError::NoPrimerDesignFits(_)
            | DomainError::TmParametersUnavailable(_) => ErrorCategory::Unsupported,
        }
    }

    /// Human-readable text. For reporting only.
    pub fn message(&self) -> String {
        match self {
            DomainError::EmptyIdentifier =>
                "an identifier was not supplied".to_string(),
            DomainError::InvalidIdentifierCharacters(problem) =>
                format!("identifier '{}' is not a single printable ASCII token without whitespace",
                    problem.identifier),
            DomainError::EmptySequenceContent(problem) =>
                format!("sequence record '{}' contains no bases", problem.sequence),
            DomainError::InvalidSequenceCharacters(problem) =>
                format!("sequence record '{}' contains unsupported characters '{}' at zero-based position {}",
                    problem.sequence, problem.characters, problem.position),
            DomainError::NegativeCoordinate(problem) =>
                format!("interval [{}, {}) on '{}' uses negative coordinates",
                    problem.start, problem.end, problem.sequence),
            DomainError::ReversedInterval(problem) =>
                format!("interval [{}, {}) on '{}' ends before it starts",
                    problem.start, problem.end, problem.sequence),
            DomainError::EmptyInterval(problem) =>
                format!("interval [{}, {}) on '{}' selects no nucleotides",
                    problem.position, problem.position, problem.sequence),
            DomainError::CoordinateOutOfBounds(problem) =>
                format!("interval [{}, {}) exceeds the length {} of '{}'",
                    problem.start, problem.end, problem.length, problem.sequence),
            DomainError::SequenceReferenceMismatch(problem) =>
                format!("the structure references {} sequence '{}' (digest {}) but {} sequence '{}' (digest {}) was supplied",
                    problem.actual_topology.name(), problem.actual_identifier, problem.actual_digest,
                    problem.expected_topology.name(), problem.expected_identifier, problem.expected_digest),
            DomainError::InvalidResiduePosition(problem) =>
                format!("residue position {} of CDS '{}' is not a positive one-based number",
                    problem.number, problem.cds),
            DomainError::ResiduePositionOutsideCds(problem) =>
                format!("residue position {} of CDS '{}' is beyond its {} residues",
                    problem.number, problem.cds, problem.residue_count),
            DomainError::ResiduePositionWrongCds(problem) =>
                format!("the residue position names CDS '{}' but CDS '{}' was supplied",
                    problem.actual, problem.expected),
            DomainError::UnsupportedNumberingScheme(problem) =>
                format!("CDS '{}' requests numbering {}; only numbering from the CDS start is supported",
                    problem.cds, problem.numbering),
            DomainError::FeatureWithoutIntervals(problem) =>
                format!("feature '{}' on '{}' has no intervals",
                    problem.feature, problem.sequence),
            DomainError::IntervalsOutOfOrder(problem) =>
                format!("intervals of feature '{}' on '{}' are not in increasing biological order",
                    problem.feature, problem.sequence),
            DomainError::OverlappingIntervals(problem) =>
                format!("interval [{}, {}) of feature '{}' on '{}' overlaps the previous interval",
                    problem.start, problem.end, problem.feature, problem.sequence),
            DomainError::UnsupportedFeatureStructure(problem) =>
                format!("feature '{}' on '{}' has {} intervals and {}",
                    problem.feature, problem.sequence, problem.interval_count, problem.reason),
            DomainError::MissingCdsAttribute(problem) => match &problem.value {
                None => format!("CDS '{}' does not supply {}", problem.cds, problem.attribute),
                Some(value) => format!("CDS '{}' supplies {} '{}', which is not a recognised value",
                    problem.cds, problem.attribute, value),
            },
            DomainError::IncompleteCds(problem) =>
                format!("CDS '{}' is declared {}; only explicitly complete CDSs are supported",
                    problem.cds, problem.completeness),
            DomainError::UnsupportedGeneticCode(problem) =>
                format!("CDS '{}' requests genetic code '{}'; only the standard genetic code is supported",
                    problem.cds, problem.genetic_code),
            DomainError::CdsLengthNotDivisibleByThree(problem) =>
                format!("extracted CDS '{}' is {} nucleotides, which is not a multiple of three",
                    problem.cds, problem.length),
            DomainError::AmbiguousCodon(problem) =>
                format!("codon {} ('{}') of CDS '{}' is ambiguous; translation would require guessing",
                    problem.codon_number, problem.codon, problem.cds),
            DomainError::InvalidCodon(problem) =>
                format!("codon {} ('{}') of CDS '{}' is outside the DNA alphabet",
                    problem.codon_number, problem.codon, problem.cds),
            DomainError::InternalStopCodon(problem) =>
                format!("codon {} ('{}') of CDS '{}' is a stop codon but is not the final codon",
                    problem.codon_number, problem.codon, problem.cds),
            DomainError::InvalidMutationRequest(problem) =>
                format!("mutation request '{}' is not representable: {}", problem.request, problem.reason),
            DomainError::UnspecifiedCodonPolicy(problem) =>
                format!("the residue mutation on CDS '{}' does not say how to choose the replacement codon",
                    problem.cds),
            DomainError::UnsupportedMutationPolicy(problem) =>
                format!("codon policy '{}' is not supported; only default and supplied:XXX are supported",
                    problem.policy),
            DomainError::UnsupportedComposition(problem) =>
                format!("the request composes {} variants as {}; {}; no variant was executed",
                    problem.variant_count, problem.composition, problem.reason),
            DomainError::ExpectedNucleotideMismatch(problem) =>
                format!("the nucleotide at position {} of '{}' is {}, but the mutation expected {}",
                    problem.position, problem.sequence, problem.actual, problem.expected),
            DomainError::ExpectedResidueMismatch(problem) =>
                format!("residue {} of CDS '{}' is {}, but the mutation expected {}",
                    problem.number, problem.cds, problem.actual, problem.expected),
            DomainError::TargetOutsideFeature(problem) =>
                format!("interval [{}, {}) on '{}' lies outside feature '{}'",
                    problem.start, problem.end, problem.sequence, problem.feature),
            DomainError::MutationVerificationFailure(problem) =>
                format!("mutation result '{}' does not follow from its source: {}", problem.result, problem.reason),
            DomainError::InvalidPrimerCandidate(problem) =>
                format!("primer candidate '{}' cannot be represented: {}", problem.primer, problem.reason),
            DomainError::PrimerTextAtSpanMismatch(problem) =>
                format!("primer '{}' does not hold the sequence the contract puts at [{}): expected '{}', found '{}'",
                    problem.primer, problem.span, problem.expected, problem.actual),
            DomainError::PrimerAnnealingNotTemplate(problem) =>
                format!("primer '{}': its annealing portion at template position {} is '{}' where the template holds '{}'",
                    problem.primer, problem.position, problem.actual, problem.expected),
            DomainError::PrimerPartition(problem) =>
                format!("primer '{}': its parts do not partition its text: {}", problem.primer, problem.reason),
            DomainError::AnnealingOverlapBelowBound(problem) =>
                format!("primer '{}' anneals {} positions, below the supplied bound of {}",
                    problem.primer, problem.overlap, problem.bound),
            DomainError::PrimerPairCoverage(problem) =>
                format!("pair '{}' leaves the changed positions {} uncovered; it covers {}",
                    problem.pair, problem.changed, problem.uncovered),
            DomainError::PrimerOrientationConflict(problem) =>
                format!("primer pair states orientations {} and {}, which are not opposite", problem.forward, problem.reverse),
            DomainError::UnsupportedPrimerPlacement(problem) =>
                format!("primer placement {} is not supported: {}", problem.placement, problem.reason),
            DomainError::InvalidPrimerDesignRequest(problem) =>
                format!("the primer-design request's {} cannot be used: {}", problem.field, problem.reason),
            DomainError::UnsupportedPrimerDesignScope(problem) =>
                format!("design policy {} does not cover this request: {}", problem.policy, problem.reason),
            DomainError::NoPrimerDesignPossible(problem) =>
                format!("no primer pair fits '{}': it has {} bases and the smallest pair spans {}",
                    problem.template, problem.template_length, problem.smallest_pair),
            DomainError::InvalidBufferModel(problem) => format!(
                "the buffer model's {} cannot be used: {}",
                problem.field, problem.reason
            ),
            DomainError::TmParametersUnavailable(problem) => format!(
                "{} has no parameters for this duplex ({}): {}",
                problem.function, problem.aspect, problem.reason
            ),
            DomainError::NoPrimerDesignFits(problem) => {
                let reasons: Vec<String> =
                    problem.obstacles.iter().map(DesignObstacle::describe).collect();
                format!("no primer pair fits the edit [{}, {}) on '{}': {}",
                    problem.edit_start, problem.edit_end, problem.template, reasons.join("; "))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classification_does_not_need_message_text() {
        let mismatch = DomainError::SequenceReferenceMismatch(SequenceReferenceMismatchProblem {
            expected_identifier: "seq-a".to_string(),
            actual_identifier: "seq-b".to_string(),
            expected_digest: "a83f6d82".to_string(),
            actual_digest: "748c00c0".to_string(),
            expected_topology: SequenceTopology::Linear,
            actual_topology: SequenceTopology::Linear,
        });
        assert!(mismatch.category() == ErrorCategory::BiologicalValidation);
        assert!(!mismatch.category().is_malformed_input());
        assert!(!mismatch.category().is_unsupported());
    }

    #[test]
    fn malformed_and_unsupported_are_distinct() {
        assert!(
            DomainError::EmptyInterval(EmptyIntervalProblem {
                sequence: "seq-a".to_string(),
                position: 6
            })
            .category()
                == ErrorCategory::MalformedInput
        );
        assert!(
            DomainError::IncompleteCds(IncompleteCdsProblem {
                cds: "cds-1".to_string(),
                completeness: "incomplete".to_string()
            })
            .category()
                == ErrorCategory::Unsupported
        );
        assert!(
            DomainError::InternalStopCodon(InternalStopCodonProblem {
                cds: "cds-1".to_string(),
                codon_number: 2,
                codon: "TAA".to_string()
            })
            .category()
                == ErrorCategory::BiologicalValidation
        );
    }

    #[test]
    fn every_category_is_reachable() {
        let categories = [
            DomainError::EmptyIdentifier.category(),
            DomainError::CoordinateOutOfBounds(CoordinateOutOfBoundsProblem {
                sequence: "seq-a".to_string(),
                start: 0,
                end: 31,
                length: 30,
            })
            .category(),
            DomainError::UnsupportedGeneticCode(UnsupportedGeneticCodeProblem {
                cds: "cds-1".to_string(),
                genetic_code: "iso1557".to_string(),
            })
            .category(),
            DomainError::UnsupportedNumberingScheme(UnsupportedNumberingProblem {
                cds: "cds-1".to_string(),
                numbering: "explicit-mapping".to_string(),
            })
            .category(),
            DomainError::InvalidResiduePosition(ResiduePositionProblem {
                cds: "cds-1".to_string(),
                number: 0,
            })
            .category(),
            DomainError::ResiduePositionOutsideCds(ResiduePositionRangeProblem {
                cds: "cds-1".to_string(),
                number: 9,
                residue_count: 5,
            })
            .category(),
            DomainError::ResiduePositionWrongCds(CdsMismatchProblem {
                expected: "cds-1".to_string(),
                actual: "cds-2".to_string(),
            })
            .category(),
            DomainError::InvalidCodon(InvalidCodonProblem {
                cds: "cds-1".to_string(),
                codon_number: 2,
                codon: "UAA".to_string(),
            })
            .category(),
        ];
        assert!(categories[0] == ErrorCategory::MalformedInput);
        assert!(categories[1] == ErrorCategory::BiologicalValidation);
        assert!(categories[2] == ErrorCategory::Unsupported);
        assert!(categories[3] == ErrorCategory::Unsupported);
        assert!(categories[4] == ErrorCategory::MalformedInput);
        assert!(categories[5] == ErrorCategory::BiologicalValidation);
        assert!(categories[6] == ErrorCategory::BiologicalValidation);
        assert!(categories[7] == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn name_and_content_mismatch_are_the_same_failure_class() {
        let name_differs =
            DomainError::SequenceReferenceMismatch(SequenceReferenceMismatchProblem {
                expected_identifier: "seq-a".to_string(),
                actual_identifier: "seq-b".to_string(),
                expected_digest: "a83f6d82".to_string(),
                actual_digest: "748c00c0".to_string(),
                expected_topology: SequenceTopology::Linear,
                actual_topology: SequenceTopology::Linear,
            });
        let content_differs =
            DomainError::SequenceReferenceMismatch(SequenceReferenceMismatchProblem {
                expected_identifier: "seq-a".to_string(),
                actual_identifier: "seq-a".to_string(),
                expected_digest: "a83f6d82".to_string(),
                actual_digest: "748c00c0".to_string(),
                expected_topology: SequenceTopology::Linear,
                actual_topology: SequenceTopology::Linear,
            });
        assert!(name_differs.category() == ErrorCategory::BiologicalValidation);
        assert!(content_differs.category() == ErrorCategory::BiologicalValidation);
        assert!(name_differs != content_differs);
    }

    // A missing codon policy is something the caller can supply; an
    // unrecognised one is out of scope. They must not share a category.
    #[test]
    fn mutation_failures_are_classified_by_meaning() {
        let missing_policy = DomainError::UnspecifiedCodonPolicy(UnspecifiedCodonPolicyProblem {
            cds: "cds-1".to_string(),
        });
        let out_of_scope =
            DomainError::UnsupportedMutationPolicy(UnsupportedMutationPolicyProblem {
                policy: "minimal-edit".to_string(),
            });
        assert!(missing_policy.category() == ErrorCategory::MalformedInput);
        assert!(out_of_scope.category() == ErrorCategory::Unsupported);

        // The request is well-formed; the artifact disagrees with it.
        let residue_mismatch =
            DomainError::ExpectedResidueMismatch(ExpectedResidueMismatchProblem {
                cds: "cds-1".to_string(),
                number: 42,
                expected: "N".to_string(),
                actual: "D".to_string(),
            });
        assert!(residue_mismatch.category() == ErrorCategory::BiologicalValidation);

        let nucleotide_mismatch =
            DomainError::ExpectedNucleotideMismatch(ExpectedNucleotideMismatchProblem {
                sequence: "seq-a".to_string(),
                position: 1209,
                expected: "G".to_string(),
                actual: "A".to_string(),
            });
        assert!(nucleotide_mismatch.category() == ErrorCategory::BiologicalValidation);

        let outside_scope = DomainError::TargetOutsideFeature(TargetOutsideFeatureProblem {
            feature: "cds-1".to_string(),
            sequence: "seq-a".to_string(),
            start: 12,
            end: 13,
        });
        assert!(outside_scope.category() == ErrorCategory::BiologicalValidation);

        let verification =
            DomainError::MutationVerificationFailure(MutationVerificationFailureProblem {
                result: "mut-1".to_string(),
                reason: "the edited artifact differs at 2 positions, not 1".to_string(),
            });
        assert!(verification.category() == ErrorCategory::BiologicalValidation);

        // A bad replacement letter is malformed input, not a world disagreement.
        let bad_request = DomainError::InvalidMutationRequest(InvalidMutationRequestProblem {
            request: "nucleotide".to_string(),
            reason: "the replacement is not a DNA letter".to_string(),
        });
        assert!(bad_request.category() == ErrorCategory::MalformedInput);
    }
}
