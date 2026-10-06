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
/// Both the display identifier and the content digest are reported so the
/// caller can see whether the name, the content, or both disagree.
#[derive(Clone, Debug, PartialEq)]
pub struct SequenceReferenceMismatchProblem {
    pub expected_identifier: String,
    pub actual_identifier: String,
    pub expected_digest: String,
    pub actual_digest: String,
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

/// Payload: a required CDS attribute was not supplied.
#[derive(Clone, Debug, PartialEq)]
pub struct MissingCdsAttributeProblem {
    pub cds: String,
    pub attribute: String,
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
            | DomainError::InvalidResiduePosition(_) => ErrorCategory::MalformedInput,
            DomainError::CoordinateOutOfBounds(_)
            | DomainError::SequenceReferenceMismatch(_)
            | DomainError::CdsLengthNotDivisibleByThree(_)
            | DomainError::AmbiguousCodon(_)
            | DomainError::InternalStopCodon(_)
            | DomainError::InvalidCodon(_)
            | DomainError::ResiduePositionOutsideCds(_)
            | DomainError::ResiduePositionWrongCds(_) => ErrorCategory::BiologicalValidation,
            DomainError::UnsupportedFeatureStructure(_)
            | DomainError::IncompleteCds(_)
            | DomainError::UnsupportedGeneticCode(_)
            | DomainError::UnsupportedNumberingScheme(_) => ErrorCategory::Unsupported,
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
                format!("the structure references sequence '{}' (digest {}) but sequence '{}' (digest {}) was supplied",
                    problem.actual_identifier, problem.actual_digest, problem.expected_identifier, problem.expected_digest),
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
            DomainError::MissingCdsAttribute(problem) =>
                format!("CDS '{}' does not supply {}", problem.cds, problem.attribute),
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
            });
        let content_differs =
            DomainError::SequenceReferenceMismatch(SequenceReferenceMismatchProblem {
                expected_identifier: "seq-a".to_string(),
                actual_identifier: "seq-a".to_string(),
                expected_digest: "a83f6d82".to_string(),
                actual_digest: "748c00c0".to_string(),
            });
        assert!(name_differs.category() == ErrorCategory::BiologicalValidation);
        assert!(content_differs.category() == ErrorCategory::BiologicalValidation);
        assert!(name_differs != content_differs);
    }
}
