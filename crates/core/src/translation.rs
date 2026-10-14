//! Translating a CDS into residues.
//!
//! The order of checks is fixed, because each one answers a different
//! question:
//!
//! 1. is the CDS explicitly complete?
//! 2. is the residue numbering one this milestone applies?
//! 3. do the intervals select nucleotides that exist on the referenced
//!    sequence?
//! 4. is the extracted length a multiple of three?
//! 5. does every codon have a residue in the standard genetic code?
//!
//! The genetic code itself is validated when the CDS is built (see
//! `cds.rs::create`), so an unsupported code never reaches translation.
//!
//! A terminal stop codon is *not* a residue: it marks the end of the CDS and
//! is reported separately. A stop codon anywhere else is a biological
//! validation failure, because the CDS claims residues continue past it.
//!
//! A `Translation` records the identity of the sequence artifact it was
//! produced from (name, content digest, and topology) and the CDS it translated, so a
//! result can be traced back to its inputs (decision D021).

use crate::cds::Cds;
use crate::cds::Completeness;
use crate::cds::Numbering;
use crate::cds::ResiduePosition;
use crate::error::AmbiguousCodonProblem;
use crate::error::CdsLengthNotDivisibleByThreeProblem;
use crate::error::CdsMismatchProblem;
use crate::error::DomainError;
use crate::error::IncompleteCdsProblem;
use crate::error::InternalStopCodonProblem;
use crate::error::InvalidCodonProblem;
use crate::error::ResiduePositionRangeProblem;
use crate::error::UnsupportedNumberingProblem;
use crate::extraction::extract_cds;
use crate::genetic_code::lookup;
use crate::genetic_code::CodonOutcome;
use crate::genetic_code::Residue;
use crate::identity::Identifier;
use crate::identity::SequenceReference;
use crate::sequence::Sequence;

/// The residues produced by translating a CDS, and what produced them.
#[derive(Clone, Debug, PartialEq)]
pub struct Translation {
    cds: Identifier,
    reference: SequenceReference,
    residues: Vec<Residue>,
    numbering: Numbering,
    terminal_stop: bool,
}

impl Translation {
    /// The CDS that was translated.
    pub fn cds(&self) -> Identifier {
        self.cds.clone()
    }

    /// The sequence artifact this translation was produced from.
    pub fn reference(&self) -> SequenceReference {
        self.reference.clone()
    }

    /// The residues, in order, excluding any terminal stop.
    pub fn residues(&self) -> Vec<Residue> {
        self.residues.clone()
    }

    /// Number of residues.
    pub fn residue_count(&self) -> usize {
        self.residues.len()
    }

    /// The residues concatenated, for reporting.
    pub fn residues_text(&self) -> String {
        let mut text: String = String::new();
        for residue in self.residues.iter() {
            text += residue.symbol().as_str();
        }
        text
    }

    /// True when the CDS ended with a stop codon.
    pub fn terminal_stop(&self) -> bool {
        self.terminal_stop
    }

    /// The residue numbering origin that was applied.
    pub fn numbering(&self) -> Numbering {
        self.numbering.clone()
    }

    /// The residue at a one-based, CDS-identified position.
    ///
    /// There is no zero-based accessor: a position that names a different CDS,
    /// or falls outside the translated CDS, is a domain error rather than a
    /// panic or a wrong residue.
    pub fn residue_at(&self, position: &ResiduePosition) -> Result<Residue, DomainError> {
        let name: String = self.cds.value().to_string();
        if position.cds() != self.cds {
            let problem = CdsMismatchProblem {
                expected: name,
                actual: position.cds().value().to_string(),
            };
            return Err(DomainError::ResiduePositionWrongCds(problem));
        }
        if self.numbering != Numbering::FromCdsStart {
            let problem = UnsupportedNumberingProblem {
                cds: name,
                numbering: self.numbering.name(),
            };
            return Err(DomainError::UnsupportedNumberingScheme(problem));
        }
        let count: usize = self.residues.len();
        if position.number() > count as u64 {
            let problem = ResiduePositionRangeProblem {
                cds: name,
                number: position.number(),
                residue_count: count as u64,
            };
            return Err(DomainError::ResiduePositionOutsideCds(problem));
        }
        Ok(self.residues[position.number() as usize - 1].clone())
    }
}

/// Translate a CDS against the sequence it references.
pub fn translate(sequence: &Sequence, cds: &Cds) -> Result<Translation, DomainError> {
    let name: String = cds.id().value().to_string();

    if cds.completeness() != Completeness::Complete {
        let problem = IncompleteCdsProblem {
            cds: name,
            completeness: cds.completeness().name(),
        };
        return Err(DomainError::IncompleteCds(problem));
    }

    if cds.numbering() != Numbering::FromCdsStart {
        let problem = UnsupportedNumberingProblem {
            cds: name,
            numbering: cds.numbering().name(),
        };
        return Err(DomainError::UnsupportedNumberingScheme(problem));
    }

    let selected: String = extract_cds(sequence, cds)?;

    if !selected.len().is_multiple_of(3) {
        let problem = CdsLengthNotDivisibleByThreeProblem {
            cds: name,
            length: selected.len() as u64,
        };
        return Err(DomainError::CdsLengthNotDivisibleByThree(problem));
    }

    let letters: Vec<char> = selected.chars().collect();
    let codon_count: usize = letters.len() / 3;
    let mut residues: Vec<Residue> = Vec::new();
    let mut terminal_stop: bool = false;

    let mut number: usize = 0;
    while number < codon_count {
        let codon: String = codon_text(letters.as_slice(), number);
        let outcome: CodonOutcome = lookup(codon.clone());
        match outcome {
            CodonOutcome::Residue(value) => residues.push(value),
            CodonOutcome::Stop => {
                if number + 1 == codon_count {
                    terminal_stop = true;
                } else {
                    let problem = InternalStopCodonProblem {
                        cds: name.clone(),
                        codon_number: number + 1,
                        codon,
                    };
                    return Err(DomainError::InternalStopCodon(problem));
                }
            }
            CodonOutcome::Ambiguous => {
                let problem = AmbiguousCodonProblem {
                    cds: name.clone(),
                    codon_number: number + 1,
                    codon,
                };
                return Err(DomainError::AmbiguousCodon(problem));
            }
            CodonOutcome::Invalid => {
                let problem = InvalidCodonProblem {
                    cds: name.clone(),
                    codon_number: number + 1,
                    codon,
                };
                return Err(DomainError::InvalidCodon(problem));
            }
        }
        number += 1;
    }

    Ok(Translation {
        cds: cds.id(),
        reference: sequence.reference(),
        residues,
        numbering: cds.numbering(),
        terminal_stop,
    })
}

/// Name of a completeness value, for error messages.
impl Completeness {
    pub fn name(&self) -> String {
        match self {
            Completeness::Complete => "complete".to_string(),
            Completeness::Incomplete => "incomplete".to_string(),
        }
    }
}

/// The codon at a zero-based codon index.
fn codon_text(letters: &[char], number: usize) -> String {
    let mut text: String = String::new();
    let mut offset: usize = 0;
    while offset < 3 {
        text += letters[3 * number + offset].to_string().as_str();
        offset += 1;
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cds::Cds;
    use crate::cds::Completeness;
    use crate::cds::Numbering;
    use crate::cds::ResiduePosition;
    use crate::error::DomainError;
    use crate::error::ErrorCategory;
    use crate::genetic_code::Residue;
    use crate::identity::Identifier;
    use crate::identity::SequenceReference;
    use crate::interval::create;
    use crate::interval::OrderedIntervals;
    use crate::sequence::normalize;
    use crate::sequence::Sequence;
    use crate::sequence::SequenceTopology;
    use crate::strand::Strand;

    fn sequence(name: String, text: String, topology: SequenceTopology) -> Sequence {
        normalize(
            Identifier::create(name.to_string()).unwrap(),
            text.to_string(),
            topology,
        )
        .unwrap()
    }

    fn linear_a() -> Sequence {
        sequence(
            "seq-a".to_string(),
            "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Linear,
        )
    }

    fn linear_d() -> Sequence {
        sequence(
            "seq-d".to_string(),
            "ATGGCTTGGAATTTTTAA".to_string(),
            SequenceTopology::Linear,
        )
    }

    fn linear_e() -> Sequence {
        sequence(
            "seq-e".to_string(),
            "ATGTAAGCTTGG".to_string(),
            SequenceTopology::Linear,
        )
    }

    fn linear_f() -> Sequence {
        sequence(
            "seq-f".to_string(),
            "ATGGCTNGGAAT".to_string(),
            SequenceTopology::Linear,
        )
    }

    fn whole(sequence: &Sequence, length: i64) -> OrderedIntervals {
        vec![create(&sequence.reference(), 0, length).unwrap()]
    }

    fn cds_of(sequence: &Sequence, length: i64, strand: Strand, completeness: Completeness) -> Cds {
        Cds::create(
            sequence,
            Identifier::create("cds-1".to_string()).unwrap(),
            whole(sequence, length),
            strand,
            completeness,
            "standard".to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap()
    }

    #[test]
    fn a_terminal_stop_is_not_a_residue() {
        let cds = cds_of(&linear_d(), 18, Strand::Plus, Completeness::Complete);
        let result = translate(&linear_d(), &cds).unwrap();
        assert!(result.residues_text() == "MAWNF");
        assert!(result.residue_count() == 5);
        assert!(result.terminal_stop());
    }

    // "Complete" is the caller's declaration about the CDS; it does not
    // require a start codon or a terminal stop. [3,15) of seq-a reads
    // GCT TGG AAT CAG: no ATG, no stop.
    #[test]
    fn a_complete_cds_needs_no_start_or_stop_codon() {
        let cds = Cds::create(
            &linear_a(),
            Identifier::create("cds-1".to_string()).unwrap(),
            vec![create(&linear_a().reference(), 3, 15).unwrap()],
            Strand::Plus,
            Completeness::Complete,
            "standard".to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap();
        let result = translate(&linear_a(), &cds).unwrap();
        assert!(result.residues_text() == "AWNQ");
        assert!(!result.terminal_stop());
    }

    #[test]
    fn an_internal_stop_is_a_validation_failure() {
        let cds = cds_of(&linear_e(), 12, Strand::Plus, Completeness::Complete);
        let error = translate(&linear_e(), &cds).unwrap_err();
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        assert!(matches!(error, DomainError::InternalStopCodon(_)));
    }

    #[test]
    fn an_ambiguous_codon_is_reported_not_guessed() {
        let cds = cds_of(&linear_f(), 12, Strand::Plus, Completeness::Complete);
        let error = translate(&linear_f(), &cds).unwrap_err();
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        assert!(matches!(error, DomainError::AmbiguousCodon(_)));
    }

    #[test]
    fn residues_are_addressed_by_one_based_positions() {
        let cds = cds_of(&linear_d(), 18, Strand::Plus, Completeness::Complete);
        let result = translate(&linear_d(), &cds).unwrap();
        let built = Identifier::create("cds-1".to_string());
        assert!(built.is_ok());
        let name: Identifier = built.unwrap();
        let first = ResiduePosition::create(name.clone(), 1);
        assert!(first.is_ok());
        assert!(result.residue_at(&first.unwrap()) == Ok(Residue::M));
        let zero = ResiduePosition::create(name.clone(), 0);
        assert!(zero.is_err());
        let error: DomainError = zero.unwrap_err();
        assert!(error.category() == ErrorCategory::MalformedInput);
    }

    #[test]
    fn a_position_outside_the_cds_is_a_domain_error() {
        let cds = cds_of(&linear_d(), 18, Strand::Plus, Completeness::Complete);
        let result = translate(&linear_d(), &cds).unwrap();
        let built = Identifier::create("cds-1".to_string());
        assert!(built.is_ok());
        let name: Identifier = built.unwrap();
        let beyond = ResiduePosition::create(name.clone(), 6);
        assert!(beyond.is_ok());
        let error = result.residue_at(&beyond.unwrap());
        assert!(error.is_err());
        let problem: DomainError = error.unwrap_err();
        assert!(matches!(problem, DomainError::ResiduePositionOutsideCds(_)));
        assert!(problem.category() == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn a_position_naming_another_cds_is_a_domain_error() {
        let cds = cds_of(&linear_d(), 18, Strand::Plus, Completeness::Complete);
        let result = translate(&linear_d(), &cds).unwrap();
        let built = Identifier::create("cds-2".to_string());
        assert!(built.is_ok());
        let position = ResiduePosition::create(built.unwrap(), 1);
        assert!(position.is_ok());
        let error = result.residue_at(&position.unwrap());
        assert!(matches!(
            error.unwrap_err(),
            DomainError::ResiduePositionWrongCds(_)
        ));
    }

    #[test]
    fn an_incomplete_cds_is_unsupported() {
        let cds = cds_of(&linear_d(), 18, Strand::Plus, Completeness::Incomplete);
        let error = translate(&linear_d(), &cds).unwrap_err();
        assert!(error.category() == ErrorCategory::Unsupported);
        assert!(matches!(error, DomainError::IncompleteCds(_)));
    }

    // The genetic code is rejected when the CDS is built, so no valid Cds
    // carries an unsupported code and translation is never reached.
    #[test]
    fn an_unsupported_genetic_code_never_reaches_translation() {
        let built = Cds::create(
            &linear_d(),
            Identifier::create("cds-1".to_string()).unwrap(),
            whole(&linear_d(), 18),
            Strand::Plus,
            Completeness::Complete,
            "iso1557".to_string(),
            Numbering::FromCdsStart,
        );
        assert!(built.is_err());
        let error: DomainError = built.unwrap_err();
        assert!(matches!(error, DomainError::UnsupportedGeneticCode(_)));
        assert!(error.category() == ErrorCategory::Unsupported);
    }

    #[test]
    fn an_explicit_numbering_mapping_is_unsupported() {
        let cds = Cds::create(
            &linear_d(),
            Identifier::create("cds-1".to_string()).unwrap(),
            whole(&linear_d(), 18),
            Strand::Plus,
            Completeness::Complete,
            "standard".to_string(),
            Numbering::ExplicitMapping,
        )
        .unwrap();
        let error = translate(&linear_d(), &cds).unwrap_err();
        assert!(matches!(error, DomainError::UnsupportedNumberingScheme(_)));
        assert!(error.category() == ErrorCategory::Unsupported);
    }

    #[test]
    fn a_length_that_is_not_a_multiple_of_three_is_validation() {
        let cds = cds_of(&linear_a(), 11, Strand::Plus, Completeness::Complete);
        let error = translate(&linear_a(), &cds).unwrap_err();
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        assert!(matches!(
            error,
            DomainError::CdsLengthNotDivisibleByThree(_)
        ));
    }

    #[test]
    fn a_minus_strand_cds_translates_its_reverse_complement() {
        let cds = cds_of(&linear_a(), 12, Strand::Minus, Completeness::Complete);
        let result = translate(&linear_a(), &cds).unwrap();
        assert!(result.residues_text() == "IPSH");
    }

    // The result records which artifact it came from, including its content
    // digest, so a result can be traced back to its inputs.
    #[test]
    fn a_translation_records_its_inputs() {
        let artifact = linear_d();
        let cds = cds_of(&artifact, 18, Strand::Plus, Completeness::Complete);
        let result = translate(&artifact, &cds).unwrap();
        assert!(result.cds() == cds.id());
        let expected: SequenceReference = artifact.reference();
        assert!(result.reference() == expected);
        assert!(result.numbering() == Numbering::FromCdsStart);
    }
}
