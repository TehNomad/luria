//! Extracting the nucleotides a feature selects.
//!
//! Extraction validates first and reads second. A minus-strand feature is
//! extracted as the reverse complement of the selected text; the strand is
//! taken from the feature, never inferred from the coordinates.
//!
//! The entry points that a caller should use are `extract_feature` and
//! `extract_cds`: both read the strand and the reference out of the feature or
//! CDS they are given, so a caller cannot pass a strand that disagrees with the
//! annotation. `extract` is the lower-level, strand-agnostic read of an
//! interval list in biological order.

use crate::cds::Cds;
use crate::error::DomainError;
use crate::feature::Feature;
use crate::interval::require_sequence;
use crate::interval::validate_against;
use crate::interval::validate_structure;
use crate::interval::NucleotideInterval;
use crate::interval::OrderedIntervals;
use crate::sequence::canonical_text;
use crate::sequence::reverse_complement;
use crate::sequence::Sequence;
use crate::strand::Strand;

/// Collect the nucleotides selected by `intervals`, in biological order.
///
/// The strand is *not* applied; see `extract_feature` and `extract_cds`.
pub fn extract(
    sequence: &Sequence,
    feature: String,
    intervals: &[NucleotideInterval],
) -> Result<Vec<char>, DomainError> {
    validate_against(feature.clone(), sequence, intervals)?;
    validate_structure(feature, sequence, intervals)?;

    let letters: Vec<char> = sequence.letters();
    let mut selected: Vec<char> = Vec::new();
    for interval in intervals {
        let mut position: u64 = interval.start();
        while position < interval.end() {
            selected.push(letters[position as usize]);
            position += 1;
        }
    }

    Ok(selected)
}

/// Extracted nucleotides as canonical text, strand not applied.
fn extract_text(
    sequence: &Sequence,
    feature: String,
    intervals: &[NucleotideInterval],
) -> Result<String, DomainError> {
    let selected: Vec<char> = extract(sequence, feature, intervals)?;
    Ok(canonical_text(selected))
}

/// Extracted nucleotides on the given strand.
fn extract_on_strand(
    sequence: &Sequence,
    feature: String,
    intervals: &[NucleotideInterval],
    strand: Strand,
) -> Result<String, DomainError> {
    let text: String = extract_text(sequence, feature, intervals)?;
    apply_strand(text, strand)
}

/// Apply a strand to canonical text.
fn apply_strand(text: String, strand: Strand) -> Result<String, DomainError> {
    if strand.is_plus() {
        return Ok(text);
    }

    let letters: Vec<char> = text.chars().collect();
    Ok(canonical_text(reverse_complement(letters)?))
}

/// The nucleotides a feature selects, on the feature's own strand.
///
/// The feature must point at `sequence` by identity: name and content digest.
pub fn extract_feature(sequence: &Sequence, feature: &Feature) -> Result<String, DomainError> {
    let label: String = feature.id().value().to_string();
    require_sequence(feature.reference(), sequence)?;
    let intervals: OrderedIntervals = feature.intervals();
    extract_on_strand(sequence, label, &intervals, feature.strand())
}

/// The nucleotides a CDS selects, on the CDS's own strand.
///
/// The CDS must point at `sequence` by identity: name and content digest.
pub fn extract_cds(sequence: &Sequence, cds: &Cds) -> Result<String, DomainError> {
    let label: String = cds.id().value().to_string();
    require_sequence(cds.reference(), sequence)?;
    let intervals: OrderedIntervals = cds.intervals();
    extract_on_strand(sequence, label, &intervals, cds.strand())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cds::Cds;
    use crate::cds::Completeness;
    use crate::cds::Numbering;
    use crate::error::DomainError;
    use crate::error::ErrorCategory;
    use crate::feature::Feature;
    use crate::identity::Identifier;
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

    fn seq_b() -> Sequence {
        normalize(
            Identifier::create("seq-b".to_string()).unwrap(),
            "ATTCCAAGCCATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Linear,
        )
        .unwrap()
    }

    fn circular_c() -> Sequence {
        normalize(
            Identifier::create("seq-c".to_string()).unwrap(),
            "GCTTGGAACGAACGAACGAACATG".to_string(),
            SequenceTopology::Circular,
        )
        .unwrap()
    }

    #[test]
    fn a_linear_interval_extracts_its_slice() {
        let reference = seq_a().reference();
        let intervals: OrderedIntervals = vec![create(&reference, 0, 12).unwrap()];
        let text = extract_text(&seq_a(), "feat-1".to_string(), intervals.as_slice());
        assert!(text.unwrap() == "ATGGCTTGGAAT");
    }

    #[test]
    fn a_minus_strand_feature_extracts_the_reverse_complement() {
        let feature = Feature::create(
            &seq_a(),
            Identifier::create("feat-1".to_string()).unwrap(),
            "segment".to_string(),
            vec![create(&seq_a().reference(), 0, 12).unwrap()],
            Strand::Minus,
        )
        .unwrap();
        let text = extract_feature(&seq_a(), &feature);
        assert!(text.unwrap() == "ATTCCAAGCCAT");
    }

    #[test]
    fn reverse_extraction_matches_a_written_reverse_sequence() {
        let feature = Feature::create(
            &seq_a(),
            Identifier::create("feat-1".to_string()).unwrap(),
            "segment".to_string(),
            vec![create(&seq_a().reference(), 0, 12).unwrap()],
            Strand::Minus,
        )
        .unwrap();
        let minus = extract_feature(&seq_a(), &feature).unwrap();
        let other = Feature::create(
            &seq_b(),
            Identifier::create("feat-2".to_string()).unwrap(),
            "segment".to_string(),
            vec![create(&seq_b().reference(), 0, 12).unwrap()],
            Strand::Plus,
        )
        .unwrap();
        let forward = extract_feature(&seq_b(), &other).unwrap();
        assert!(minus == forward);
    }

    #[test]
    fn an_origin_spanning_feature_extracts_both_sides() {
        let plus_feature = Feature::create(
            &circular_c(),
            Identifier::create("feat-1".to_string()).unwrap(),
            "segment".to_string(),
            vec![
                create(&circular_c().reference(), 21, 24).unwrap(),
                create(&circular_c().reference(), 0, 6).unwrap(),
            ],
            Strand::Plus,
        )
        .unwrap();
        let minus_feature = Feature::create(
            &circular_c(),
            Identifier::create("feat-1".to_string()).unwrap(),
            "segment".to_string(),
            vec![
                create(&circular_c().reference(), 21, 24).unwrap(),
                create(&circular_c().reference(), 0, 6).unwrap(),
            ],
            Strand::Minus,
        )
        .unwrap();
        assert!(extract_feature(&circular_c(), &plus_feature).unwrap() == "ATGGCTTGG");
        assert!(extract_feature(&circular_c(), &minus_feature).unwrap() == "CCAAGCCAT");
    }

    #[test]
    fn extraction_still_validates_coordinates() {
        let intervals: OrderedIntervals = vec![create(&seq_a().reference(), 0, 31).unwrap()];
        let error = extract(&seq_a(), "feat-1".to_string(), intervals.as_slice()).unwrap_err();
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        assert!(matches!(error, DomainError::CoordinateOutOfBounds(_)));
    }

    #[test]
    fn a_cds_is_extracted_on_its_own_strand() {
        let cds = Cds::create(
            &seq_a(),
            Identifier::create("cds-1".to_string()).unwrap(),
            vec![create(&seq_a().reference(), 0, 12).unwrap()],
            Strand::Minus,
            Completeness::Complete,
            "standard".to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap();
        assert!(extract_cds(&seq_a(), &cds).unwrap() == "ATTCCAAGCCAT");
    }

    // The feature was built against seq-a; extracting it from seq-b, which has
    // a different name and content, must fail.
    #[test]
    fn extraction_rejects_a_feature_from_another_sequence() {
        let feature = Feature::create(
            &seq_a(),
            Identifier::create("feat-1".to_string()).unwrap(),
            "segment".to_string(),
            vec![create(&seq_a().reference(), 0, 12).unwrap()],
            Strand::Plus,
        )
        .unwrap();
        let error = extract_feature(&seq_b(), &feature).unwrap_err();
        assert!(matches!(error, DomainError::SequenceReferenceMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn strand_is_never_inferred_from_coordinates() {
        let intervals: OrderedIntervals = vec![create(&seq_a().reference(), 0, 12).unwrap()];
        let feature = Feature::create(
            &seq_a(),
            Identifier::create("feat-1".to_string()).unwrap(),
            "segment".to_string(),
            intervals.clone(),
            Strand::Minus,
        )
        .unwrap();
        let biological: Vec<char> = extract(&seq_a(), "feat-1".to_string(), &intervals).unwrap();
        let on_strand: String = extract_feature(&seq_a(), &feature).unwrap();
        assert!(canonical_text(biological) == "ATGGCTTGGAAT");
        assert!(on_strand == "ATTCCAAGCCAT");
    }
}
