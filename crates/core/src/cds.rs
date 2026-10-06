//! A CDS: the annotated coding sequence of a record.
//!
//! A CDS carries everything translation needs explicitly: its intervals, its
//! strand, whether it is complete, the genetic code to use, and the residue
//! numbering origin. Nothing is defaulted. A record with no CDS is a missing
//! attribute, not an empty CDS.
//!
//! Genetic codes are validated when the CDS is built, not when it is
//! translated: an unsupported code never becomes a valid `Cds`, so no
//! downstream step can assume a supported code was requested.
//!
//! Residue positions are modelled by `ResiduePosition`, a one-based number
//! tied to a named CDS. There is no raw-index residue accessor, so zero-based
//! indexing cannot be confused with biological residue numbering.

use crate::error::DomainError;
use crate::error::MissingCdsAttributeProblem;
use crate::error::ResiduePositionProblem;
use crate::error::UnsupportedGeneticCodeProblem;
use crate::genetic_code::is_supported;
use crate::genetic_code::GeneticCode;
use crate::identity::Identifier;
use crate::identity::SequenceReference;
use crate::interval::validate_against;
use crate::interval::validate_structure;
use crate::interval::OrderedIntervals;
use crate::sequence::Sequence;
use crate::strand::Strand;

/// Whether a CDS reaches both ends of the coded region.
#[derive(Clone, Debug, PartialEq)]
pub enum Completeness {
    Complete,
    Incomplete,
}

/// Where residue numbering starts.
///
/// `FromCdsStart` means residue 1 is the first translated codon of the CDS.
/// `ExplicitMapping` means the record supplies its own residue numbering (for
/// example a fragment-based numbering); this milestone refuses to apply one
/// rather than assuming CDS-start numbering.
#[derive(Clone, Debug, PartialEq)]
pub enum Numbering {
    FromCdsStart,
    ExplicitMapping,
}

/// A CDS on a reference sequence.
#[derive(Clone, Debug, PartialEq)]
pub struct Cds {
    id: Identifier,
    reference: SequenceReference,
    intervals: OrderedIntervals,
    strand: Strand,
    completeness: Completeness,
    genetic_code: GeneticCode,
    numbering: Numbering,
}

/// Parse the completeness attribute.
///
/// An unrecognised value means no valid value was supplied, which is a
/// missing attribute rather than a guess.
pub fn parse_completeness(text: String) -> Result<Completeness, DomainError> {
    let lowered: String = text.to_lowercase();
    if lowered == "complete" {
        return Ok(Completeness::Complete);
    }
    if lowered == "incomplete" {
        return Ok(Completeness::Incomplete);
    }
    let problem = MissingCdsAttributeProblem {
        cds: text,
        attribute: "completeness".to_string(),
    };
    Err(DomainError::MissingCdsAttribute(problem))
}

/// Parse the residue numbering origin.
///
/// An unrecognised value means no valid value was supplied, which is a
/// missing attribute rather a guess.
pub fn parse_numbering(text: String) -> Result<Numbering, DomainError> {
    let lowered: String = text.to_lowercase();
    if lowered == "from-cds-start" {
        return Ok(Numbering::FromCdsStart);
    }
    if lowered == "explicit" {
        return Ok(Numbering::ExplicitMapping);
    }
    let problem = MissingCdsAttributeProblem {
        cds: text,
        attribute: "numbering".to_string(),
    };
    Err(DomainError::MissingCdsAttribute(problem))
}

impl Numbering {
    pub fn name(&self) -> String {
        match self {
            Numbering::FromCdsStart => "from-cds-start".to_string(),
            Numbering::ExplicitMapping => "explicit".to_string(),
        }
    }
}

impl GeneticCode {
    pub fn name(&self) -> String {
        match self {
            GeneticCode::Standard => "standard".to_string(),
        }
    }
}

impl Cds {
    /// Build a CDS, validating its intervals and its genetic code.
    pub fn create(
        sequence: &Sequence,
        id: Identifier,
        intervals: OrderedIntervals,
        strand: Strand,
        completeness: Completeness,
        genetic_code: String,
        numbering: Numbering,
    ) -> Result<Cds, DomainError> {
        let name: String = id.value().to_string();
        if !is_supported(genetic_code.clone()) {
            let problem = UnsupportedGeneticCodeProblem {
                cds: name,
                genetic_code,
            };
            return Err(DomainError::UnsupportedGeneticCode(problem));
        }
        validate_against(name.clone(), sequence, intervals.as_slice())?;
        validate_structure(name, sequence, intervals.as_slice())?;

        Ok(Cds {
            id,
            reference: sequence.reference(),
            intervals,
            strand,
            completeness,
            genetic_code: GeneticCode::Standard,
            numbering,
        })
    }

    /// Build a CDS from optional attributes.
    ///
    /// Any absent attribute is reported as `MissingCdsAttribute`; nothing is
    /// defaulted.
    pub fn from_optional(
        sequence: &Sequence,
        id: Identifier,
        intervals: Option<OrderedIntervals>,
        strand: Option<Strand>,
        completeness: Option<String>,
        genetic_code: Option<String>,
        numbering: Option<String>,
    ) -> Result<Cds, DomainError> {
        let name: String = id.value().to_string();
        if intervals.is_none() {
            let problem = MissingCdsAttributeProblem {
                cds: name,
                attribute: "intervals".to_string(),
            };
            return Err(DomainError::MissingCdsAttribute(problem));
        }
        if strand.is_none() {
            let problem = MissingCdsAttributeProblem {
                cds: name,
                attribute: "strand".to_string(),
            };
            return Err(DomainError::MissingCdsAttribute(problem));
        }
        if completeness.is_none() {
            let problem = MissingCdsAttributeProblem {
                cds: name,
                attribute: "completeness".to_string(),
            };
            return Err(DomainError::MissingCdsAttribute(problem));
        }
        if genetic_code.is_none() {
            let problem = MissingCdsAttributeProblem {
                cds: name,
                attribute: "genetic code".to_string(),
            };
            return Err(DomainError::MissingCdsAttribute(problem));
        }
        if numbering.is_none() {
            let problem = MissingCdsAttributeProblem {
                cds: name,
                attribute: "numbering".to_string(),
            };
            return Err(DomainError::MissingCdsAttribute(problem));
        }

        let held_completeness: Option<String> = completeness;
        let parsed: Completeness = parse_completeness(held_completeness.unwrap())?;

        let held_numbering: Option<String> = numbering;
        let parsed_numbering: Numbering = parse_numbering(held_numbering.unwrap())?;

        let held_intervals: Option<OrderedIntervals> = intervals;
        let held_strand: Option<Strand> = strand;
        let held_code: Option<String> = genetic_code;
        Cds::create(
            sequence,
            id,
            held_intervals.unwrap(),
            held_strand.unwrap(),
            parsed,
            held_code.unwrap(),
            parsed_numbering,
        )
    }

    /// The CDS identifier.
    pub fn id(&self) -> Identifier {
        self.id.clone()
    }

    /// The reference sequence this CDS points at, by identity.
    pub fn reference(&self) -> SequenceReference {
        self.reference.clone()
    }

    /// The intervals, in biological order.
    pub fn intervals(&self) -> OrderedIntervals {
        self.intervals.clone()
    }

    /// The strand.
    pub fn strand(&self) -> Strand {
        self.strand.clone()
    }

    /// Completeness of the CDS.
    pub fn completeness(&self) -> Completeness {
        self.completeness.clone()
    }

    /// The validated genetic code.
    pub fn genetic_code(&self) -> GeneticCode {
        self.genetic_code.clone()
    }

    /// The residue numbering origin.
    pub fn numbering(&self) -> Numbering {
        self.numbering.clone()
    }

    /// Total number of nucleotides the CDS selects.
    pub fn total_length(&self) -> u64 {
        let mut total: u64 = 0;
        for interval in self.intervals.iter() {
            total += interval.length();
        }
        total
    }
}

/// A one-based residue position inside a named CDS.
///
/// The CDS name is part of the position so a position can never be applied to
/// the wrong CDS, and the number is one-based biological numbering, never a
/// zero-based index.
#[derive(Clone, Debug, PartialEq)]
pub struct ResiduePosition {
    cds: Identifier,
    number: u64,
}

impl ResiduePosition {
    /// Construct a residue position. Zero and negative numbers are rejected.
    pub fn create(cds: Identifier, number: i64) -> Result<ResiduePosition, DomainError> {
        if number < 1 {
            let problem = ResiduePositionProblem {
                cds: cds.value().to_string(),
                number,
            };
            return Err(DomainError::InvalidResiduePosition(problem));
        }
        Ok(ResiduePosition {
            cds,
            number: number as u64,
        })
    }

    pub fn cds(&self) -> Identifier {
        self.cds.clone()
    }

    pub fn number(&self) -> u64 {
        self.number
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DomainError;
    use crate::error::ErrorCategory;
    use crate::genetic_code::GeneticCode;
    use crate::identity::Identifier;
    use crate::identity::SequenceReference;
    use crate::interval::create;
    use crate::interval::OrderedIntervals;
    use crate::sequence::normalize;
    use crate::sequence::Sequence;
    use crate::sequence::SequenceTopology;
    use crate::strand::Strand;

    fn seq_a() -> Sequence {
        normalize(
            Identifier::create("seq-a".to_string()).unwrap(),
            "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Linear,
        )
        .unwrap()
    }

    #[test]
    fn a_complete_cds_carries_every_attribute() {
        let reference: SequenceReference = seq_a().reference();
        let intervals: OrderedIntervals = vec![create(&reference, 0, 18).unwrap()];
        let cds = Cds::create(
            &seq_a(),
            Identifier::create("cds-1".to_string()).unwrap(),
            intervals,
            Strand::Plus,
            Completeness::Complete,
            "standard".to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap();
        assert!(cds.total_length() == 18);
        assert!(cds.completeness() == Completeness::Complete);
        assert!(cds.genetic_code() == GeneticCode::Standard);
        assert!(cds.numbering() == Numbering::FromCdsStart);
        assert!(cds.reference() == reference);
    }

    // The genetic code is rejected when the CDS is built, so no valid Cds ever
    // holds an unsupported code.
    #[test]
    fn an_unsupported_genetic_code_never_becomes_a_cds() {
        let reference: SequenceReference = seq_a().reference();
        let intervals: OrderedIntervals = vec![create(&reference, 0, 18).unwrap()];
        let cds = Cds::create(
            &seq_a(),
            Identifier::create("cds-1".to_string()).unwrap(),
            intervals,
            Strand::Plus,
            Completeness::Complete,
            "iso1557".to_string(),
            Numbering::FromCdsStart,
        );
        assert!(cds.is_err());
        let error: DomainError = cds.unwrap_err();
        assert!(matches!(error, DomainError::UnsupportedGeneticCode(_)));
        assert!(error.category() == ErrorCategory::Unsupported);
    }

    #[test]
    fn near_matches_are_not_accepted() {
        assert!(Cds::create(
            &seq_a(),
            Identifier::create("cds-1".to_string()).unwrap(),
            vec![create(&seq_a().reference(), 0, 18).unwrap()],
            Strand::Plus,
            Completeness::Complete,
            "STANDARD".to_string(),
            Numbering::FromCdsStart,
        )
        .is_err());
        assert!(Cds::create(
            &seq_a(),
            Identifier::create("cds-1".to_string()).unwrap(),
            vec![create(&seq_a().reference(), 0, 18).unwrap()],
            Strand::Plus,
            Completeness::Complete,
            "iso-1557-1".to_string(),
            Numbering::FromCdsStart,
        )
        .is_err());
    }

    #[test]
    fn an_absent_attribute_is_a_missing_attribute() {
        let reference: SequenceReference = seq_a().reference();
        let intervals: OrderedIntervals = vec![create(&reference, 0, 18).unwrap()];
        let error = Cds::from_optional(
            &seq_a(),
            Identifier::create("cds-1".to_string()).unwrap(),
            Some(intervals),
            None,
            Some("complete".to_string()),
            Some("standard".to_string()),
            Some("from-cds-start".to_string()),
        )
        .unwrap_err();
        assert!(error.category() == ErrorCategory::MalformedInput);
        assert!(matches!(error, DomainError::MissingCdsAttribute(_)));
    }

    #[test]
    fn an_absent_cds_is_not_an_empty_cds() {
        let error = Cds::from_optional(
            &seq_a(),
            Identifier::create("cds-1".to_string()).unwrap(),
            None,
            Some(Strand::Plus),
            Some("complete".to_string()),
            Some("standard".to_string()),
            Some("from-cds-start".to_string()),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::MissingCdsAttribute(_)));
    }

    // The intervals name `seq-a` but were built from a different artifact that
    // shares that name; the CDS must still be rejected.
    #[test]
    fn a_cds_rejects_a_same_name_different_content_reference() {
        let alias: Sequence = normalize(
            Identifier::create("seq-a".to_string()).unwrap(),
            "ATTCCAAGCCATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Linear,
        )
        .unwrap();
        let intervals: OrderedIntervals = vec![create(&alias.reference(), 0, 18).unwrap()];
        let error = Cds::from_optional(
            &seq_a(),
            Identifier::create("cds-1".to_string()).unwrap(),
            Some(intervals),
            Some(Strand::Plus),
            Some("complete".to_string()),
            Some("standard".to_string()),
            Some("from-cds-start".to_string()),
        )
        .unwrap_err();
        assert!(matches!(error, DomainError::SequenceReferenceMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn completeness_is_parsed_not_assumed() {
        assert!(parse_completeness("complete".to_string()).unwrap() == Completeness::Complete);
        assert!(parse_completeness("Incomplete".to_string()).unwrap() == Completeness::Incomplete);
        let error = parse_completeness("maybe".to_string()).unwrap_err();
        assert!(matches!(error, DomainError::MissingCdsAttribute(_)));
    }

    #[test]
    fn numbering_is_parsed_not_assumed() {
        assert!(parse_numbering("from-cds-start".to_string()).unwrap() == Numbering::FromCdsStart);
        assert!(parse_numbering("Explicit".to_string()).unwrap() == Numbering::ExplicitMapping);
        let error = parse_numbering("auto".to_string()).unwrap_err();
        assert!(matches!(error, DomainError::MissingCdsAttribute(_)));
    }

    #[test]
    fn residue_positions_are_one_based() {
        let built = Identifier::create("cds-1".to_string());
        assert!(built.is_ok());
        let name: Identifier = built.unwrap();
        assert!(ResiduePosition::create(name.clone(), 1).is_ok());
        let zero = ResiduePosition::create(name.clone(), 0);
        assert!(zero.is_err());
        let error: DomainError = zero.unwrap_err();
        assert!(error.category() == ErrorCategory::MalformedInput);
        assert!(matches!(error, DomainError::InvalidResiduePosition(_)));
        let position = ResiduePosition::create(name.clone(), 3).unwrap();
        assert!(position.number() == 3);
        assert!(position.cds() == name);
    }
}
