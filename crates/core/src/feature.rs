//! Features: named intervals on a reference sequence, with an explicit strand.
//!
//! A feature is *not* a range of text. It points at a reference sequence by
//! identity, and its intervals are validated against that sequence's length
//! and topology. A feature that wraps the origin of a circular reference is
//! written as two abutting intervals; nothing is wrapped or joined silently.

use crate::error::DomainError;
use crate::identity::Identifier;
use crate::identity::SequenceReference;
use crate::interval::validate_against;
use crate::interval::validate_structure;
use crate::interval::OrderedIntervals;
use crate::sequence::Sequence;
use crate::strand::Strand;

/// A named feature on a reference sequence.
#[derive(Clone, Debug, PartialEq)]
pub struct Feature {
    id: Identifier,
    reference: SequenceReference,
    type_name: String,
    intervals: OrderedIntervals,
    strand: Strand,
}

impl Feature {
    /// Build a feature, validating every interval against `sequence`.
    pub fn create(
        sequence: &Sequence,
        id: Identifier,
        type_name: String,
        intervals: OrderedIntervals,
        strand: Strand,
    ) -> Result<Feature, DomainError> {
        let name: String = id.value().to_string();
        validate_against(name.clone(), sequence, intervals.as_slice())?;
        validate_structure(name, sequence, intervals.as_slice())?;

        Ok(Feature {
            id,
            reference: sequence.reference(),
            type_name,
            intervals,
            strand,
        })
    }

    /// The feature identifier.
    pub fn id(&self) -> Identifier {
        self.id.clone()
    }

    /// The reference sequence this feature points at, by identity.
    pub fn reference(&self) -> SequenceReference {
        self.reference.clone()
    }

    /// The feature type, for example `CDS` or `source_pub`.
    pub fn type_name(&self) -> String {
        self.type_name.clone()
    }

    /// The strand; always explicit.
    pub fn strand(&self) -> Strand {
        self.strand.clone()
    }

    /// The intervals, in biological order.
    pub fn intervals(&self) -> OrderedIntervals {
        self.intervals.clone()
    }

    /// Number of intervals.
    pub fn interval_count(&self) -> usize {
        self.intervals.len()
    }

    /// Total number of nucleotides selected.
    pub fn total_length(&self) -> u64 {
        let mut total: u64 = 0;
        for interval in self.intervals.iter() {
            total += interval.length();
        }
        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DomainError;
    use crate::error::ErrorCategory;
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

    fn circular_c() -> Sequence {
        normalize(
            Identifier::create("seq-c".to_string()).unwrap(),
            "GCTTGGAACGAACGAACGAACATG".to_string(),
            SequenceTopology::Circular,
        )
        .unwrap()
    }

    #[test]
    fn a_feature_points_at_its_reference() {
        let reference: SequenceReference = seq_a().reference();
        let intervals: OrderedIntervals = vec![create(&reference, 0, 12).unwrap()];
        let feature = Feature::create(
            &seq_a(),
            Identifier::create("feat-1".to_string()).unwrap(),
            "segment".to_string(),
            intervals,
            Strand::Plus,
        )
        .unwrap();
        assert!(feature.reference() == reference);
        assert!(feature.total_length() == 12);
        assert!(feature.interval_count() == 1);
    }

    // The intervals name `seq-a` but were built from a different artifact that
    // happens to share that name; Feature::create must still reject them.
    #[test]
    fn a_feature_rejects_a_same_name_different_content_reference() {
        let alias = normalize(
            Identifier::create("seq-a".to_string()).unwrap(),
            "ATTCCAAGCCATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Linear,
        );
        assert!(alias.is_ok());
        let intervals: OrderedIntervals = vec![create(&alias.unwrap().reference(), 0, 12).unwrap()];
        let feature = Feature::create(
            &seq_a(),
            Identifier::create("feat-1".to_string()).unwrap(),
            "segment".to_string(),
            intervals,
            Strand::Plus,
        );
        assert!(feature.is_err());
        let error: DomainError = feature.unwrap_err();
        assert!(matches!(error, DomainError::SequenceReferenceMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    #[test]
    fn a_feature_without_intervals_is_malformed() {
        let feature = Feature::create(
            &seq_a(),
            Identifier::create("feat-1".to_string()).unwrap(),
            "segment".to_string(),
            Vec::new(),
            Strand::Plus,
        )
        .unwrap_err();
        assert!(feature.category() == ErrorCategory::MalformedInput);
        assert!(matches!(feature, DomainError::FeatureWithoutIntervals(_)));
    }

    #[test]
    fn strand_is_carried_explicitly() {
        let reference: SequenceReference = seq_a().reference();
        let intervals: OrderedIntervals = vec![create(&reference, 0, 12).unwrap()];
        let feature = Feature::create(
            &seq_a(),
            Identifier::create("feat-1".to_string()).unwrap(),
            "segment".to_string(),
            intervals.clone(),
            Strand::Minus,
        )
        .unwrap();
        assert!(feature.strand() == Strand::Minus);
        assert!(feature.intervals() == intervals);
    }

    #[test]
    fn circular_features_wrap_the_origin_explicitly() {
        let reference: SequenceReference = circular_c().reference();
        let intervals: OrderedIntervals = vec![
            create(&reference, 21, 24).unwrap(),
            create(&reference, 0, 6).unwrap(),
        ];
        let feature = Feature::create(
            &circular_c(),
            Identifier::create("feat-1".to_string()).unwrap(),
            "segment".to_string(),
            intervals,
            Strand::Plus,
        )
        .unwrap();
        assert!(feature.interval_count() == 2);
        assert!(feature.total_length() == 9);
    }
}
