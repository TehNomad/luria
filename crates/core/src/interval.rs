//! Nucleotide coordinates and intervals.
//!
//! Coordinates are **zero-based and half-open**: the interval below selects
//! the first four nucleotides and nothing else.
//!
//! ```text
//! NucleotideInterval(seq-a@a83f6d82, 0, 4)
//! ```
//!
//! An interval never has `start > end`. A reversed interval is malformed
//! input, not a strand encoding: strand is carried explicitly by the feature,
//! and a reverse-strand feature is the same coordinates plus `Strand::Minus`.
//!
//! Intervals are stored in **biological order**, that is, in order of
//! increasing coordinate on the reference. On a circular reference, a feature
//! that wraps the origin is written as two abutting intervals, the first
//! ending at the sequence length and the second starting at zero.
//!
//! The interval's reference is a `SequenceReference` (display name plus
//! content digest), not a bare display name, so an interval built against one
//! artifact cannot be applied to a different artifact that shares a name.
//!
//! The struct fields are private: `create` is the only way to obtain an
//! interval, and it rejects negative, reversed, and empty intervals. There is
//! therefore no public way to hold an interval with `start >= end`.

use crate::error::CoordinateOutOfBoundsProblem;
use crate::error::DomainError;
use crate::error::EmptyIntervalProblem;
use crate::error::FeatureWithoutIntervalsProblem;
use crate::error::IntervalsOutOfOrderProblem;
use crate::error::NegativeCoordinateProblem;
use crate::error::OverlappingIntervalsProblem;
use crate::error::ReversedIntervalProblem;
use crate::error::SequenceReferenceMismatchProblem;
use crate::error::UnsupportedFeatureStructureProblem;
use crate::identity::SequenceReference;
use crate::sequence::Sequence;
use crate::sequence::SequenceTopology;

/// Intervals of one feature, in biological order.
pub type OrderedIntervals = Vec<NucleotideInterval>;

/// An interval on a reference sequence.
///
/// Fields are private; `create` is the only constructor, so every value of
/// this type satisfies `start < end` and carries a `SequenceReference`.
#[derive(Clone, Debug, PartialEq)]
pub struct NucleotideInterval {
    sequence: SequenceReference,
    start: u64,
    end: u64,
}

/// Build an interval from signed coordinates.
///
/// Negative, reversed, and empty intervals are malformed input. Nothing is
/// silently corrected.
pub fn create(
    sequence: &SequenceReference,
    start: i64,
    end: i64,
) -> Result<NucleotideInterval, DomainError> {
    if start < 0 || end < 0 {
        let problem = NegativeCoordinateProblem {
            sequence: sequence.name().value().to_string(),
            start,
            end,
        };
        return Err(DomainError::NegativeCoordinate(problem));
    }

    let low: u64 = unsigned(start);
    let high: u64 = unsigned(end);
    if high < low {
        let problem = ReversedIntervalProblem {
            sequence: sequence.name().value().to_string(),
            start: low,
            end: high,
        };
        return Err(DomainError::ReversedInterval(problem));
    }
    if high == low {
        let problem = EmptyIntervalProblem {
            sequence: sequence.name().value().to_string(),
            position: low,
        };
        return Err(DomainError::EmptyInterval(problem));
    }

    Ok(NucleotideInterval {
        sequence: sequence.clone(),
        start: low,
        end: high,
    })
}

impl NucleotideInterval {
    /// The reference sequence this interval points at.
    pub fn reference(&self) -> SequenceReference {
        self.sequence.clone()
    }

    /// Zero-based start.
    pub fn start(&self) -> u64 {
        self.start
    }

    /// Exclusive end.
    pub fn end(&self) -> u64 {
        self.end
    }

    /// Number of nucleotides selected. Always positive for a constructed
    /// interval.
    pub fn length(&self) -> u64 {
        self.end() - self.start()
    }

    /// Check that the interval fits inside `sequence`.
    pub fn within(&self, sequence: &Sequence) -> Result<(), DomainError> {
        if self.end() > sequence.length() {
            let problem = CoordinateOutOfBoundsProblem {
                sequence: sequence.display_name(),
                start: self.start(),
                end: self.end(),
                length: sequence.length(),
            };
            return Err(DomainError::CoordinateOutOfBounds(problem));
        }
        Ok(())
    }
}

/// A point between two nucleotides: a coordinate, not an interval.
///
/// A point selects nothing; it names the gap before base `position`, that is,
/// the gap between base `position − 1` and base `position`. An insertion of
/// `k` bases at `p` occupies `[p, p + k)` of the edited artifact. Unlike an
/// interval a point has no length to order, and unlike `create` it does not
/// reject a coordinate at the end of the sequence: `within` accepts `position
/// == length`, which names the gap after the last base, and an edit that needs
/// a base on each side of the point refuses it there.
#[derive(Clone, Debug, PartialEq)]
pub struct InsertionPoint {
    sequence: SequenceReference,
    position: u64,
}

/// Build a point from a signed coordinate.
///
/// A negative coordinate is malformed input; nothing is silently corrected. A
/// coordinate past the end of a sequence is not rejected here, because a point
/// constructor never sees a sequence: `within` is what bounds the coordinate.
pub fn create_point(
    sequence: &SequenceReference,
    position: i64,
) -> Result<InsertionPoint, DomainError> {
    if position < 0 {
        let problem = NegativeCoordinateProblem {
            sequence: sequence.name().value().to_string(),
            start: position,
            end: position,
        };
        return Err(DomainError::NegativeCoordinate(problem));
    }

    Ok(InsertionPoint {
        sequence: sequence.clone(),
        position: unsigned(position),
    })
}

impl InsertionPoint {
    /// The reference sequence this point sits on.
    pub fn reference(&self) -> SequenceReference {
        self.sequence.clone()
    }

    /// Zero-based coordinate of the gap.
    pub fn position(&self) -> u64 {
        self.position
    }

    /// Check that the point fits inside `sequence`.
    ///
    /// `position == sequence.length()` fits: it names the gap after the last
    /// base. An edit that needs a base after the point refuses it there.
    pub fn within(&self, sequence: &Sequence) -> Result<(), DomainError> {
        if self.position() > sequence.length() {
            let problem = CoordinateOutOfBoundsProblem {
                sequence: sequence.display_name(),
                start: self.position(),
                end: self.position(),
                length: sequence.length(),
            };
            return Err(DomainError::CoordinateOutOfBounds(problem));
        }
        Ok(())
    }
}

/// True when both intervals select at least one shared nucleotide position.
pub fn overlaps(first: &NucleotideInterval, second: &NucleotideInterval) -> bool {
    first.start() < second.end() && second.start() < first.end()
}

/// True when some interval contains `position`, that is, `start <= position < end`.
pub(crate) fn holds_position(intervals: &[NucleotideInterval], position: u64) -> bool {
    for interval in intervals {
        if position >= interval.start() && position < interval.end() {
            return true;
        }
    }
    false
}

/// True when one interval contains the whole half-open region `[start, end)`.
pub(crate) fn holds_region(intervals: &[NucleotideInterval], start: u64, end: u64) -> bool {
    for interval in intervals {
        if start >= interval.start() && end <= interval.end() {
            return true;
        }
    }
    false
}

/// True when some interval has `start < position < end`: a base lies on each side.
pub(crate) fn holds_point(intervals: &[NucleotideInterval], position: u64) -> bool {
    for interval in intervals {
        if position > interval.start() && position < interval.end() {
            return true;
        }
    }
    false
}

/// Reject a reference that does not denote `sequence`.
///
/// Reference equality means the display identifier, the content digest, and
/// the topology all agree. This is the single place the rule is implemented; intervals, features
/// and CDSs all go through it.
pub fn require_sequence(
    reference: SequenceReference,
    sequence: &Sequence,
) -> Result<(), DomainError> {
    let expected: SequenceReference = sequence.reference();
    if reference != expected {
        let problem = SequenceReferenceMismatchProblem {
            expected_identifier: expected.name().value().to_string(),
            actual_identifier: reference.name().value().to_string(),
            expected_digest: expected.content_hash().hex_digest().to_string(),
            actual_digest: reference.content_hash().hex_digest().to_string(),
            expected_topology: expected.topology(),
            actual_topology: reference.topology(),
        };
        return Err(DomainError::SequenceReferenceMismatch(problem));
    }
    Ok(())
}

/// Validate an interval list against the sequence it references.
///
/// Checks, in order: that the list is not empty, that every interval points
/// at `sequence` (name, content digest, and topology), that every interval
/// fits inside it, that starts increase strictly, and that no two intervals
/// overlap.
///
/// Overlap is checked between every pair, not only neighbours: on a circular
/// reference the list `[21,24), [0,6), [6,22)` has no overlapping neighbours
/// but selects position 21 twice, which would make the feature longer than
/// the sequence.
pub fn validate_against(
    feature: String,
    sequence: &Sequence,
    intervals: &[NucleotideInterval],
) -> Result<(), DomainError> {
    if intervals.is_empty() {
        let problem = FeatureWithoutIntervalsProblem {
            feature,
            sequence: sequence.display_name(),
        };
        return Err(DomainError::FeatureWithoutIntervals(problem));
    }

    for interval in intervals {
        require_sequence(interval.reference(), sequence)?;
        interval.within(sequence)?;
    }

    let count: usize = intervals.len();
    let mut index: usize = 1;
    while index < count {
        let previous: NucleotideInterval = intervals[index - 1].clone();
        let current: NucleotideInterval = intervals[index].clone();
        let crossing: bool = sequence.topology() == SequenceTopology::Circular
            && previous.end() == sequence.length()
            && current.start() == 0;
        if !crossing && current.start() < previous.start() {
            let problem = IntervalsOutOfOrderProblem {
                feature,
                sequence: sequence.display_name(),
            };
            return Err(DomainError::IntervalsOutOfOrder(problem));
        }
        index += 1;
    }

    for later in 1..count {
        for earlier in 0..later {
            if overlaps(&intervals[earlier], &intervals[later]) {
                let problem = OverlappingIntervalsProblem {
                    feature,
                    sequence: sequence.display_name(),
                    start: intervals[later].start(),
                    end: intervals[later].end(),
                };
                return Err(DomainError::OverlappingIntervals(problem));
            }
        }
    }

    Ok(())
}

/// Validate the *shape* of an interval list against the topology.
///
/// A linear feature is a single interval. A circular feature (D036) is one
/// interval, or exactly two intervals that cross the origin: `[a, n)` followed
/// by `[0, b)`. Intervals that abut anywhere else are refused at construction,
/// so a feature never carries a join that the containment and warning checks
/// would treat as an edge.
pub fn validate_structure(
    feature: String,
    sequence: &Sequence,
    intervals: &[NucleotideInterval],
) -> Result<(), DomainError> {
    let count: usize = intervals.len();
    if count == 0 {
        let problem = FeatureWithoutIntervalsProblem {
            feature,
            sequence: sequence.display_name(),
        };
        return Err(DomainError::FeatureWithoutIntervals(problem));
    }

    let topology: SequenceTopology = sequence.topology();
    if topology == SequenceTopology::Linear && count > 1 {
        let problem = UnsupportedFeatureStructureProblem {
            feature,
            sequence: sequence.display_name(),
            interval_count: count,
            reason: "a linear feature is a single interval".to_string(),
        };
        return Err(DomainError::UnsupportedFeatureStructure(problem));
    }

    if topology == SequenceTopology::Circular {
        let length: u64 = sequence.length();
        let crossings: usize = origin_crossings(intervals, length);
        if crossings > 1 {
            let problem = UnsupportedFeatureStructureProblem {
                feature,
                sequence: sequence.display_name(),
                interval_count: count,
                reason: "the feature crosses the origin more than once".to_string(),
            };
            return Err(DomainError::UnsupportedFeatureStructure(problem));
        }
        let gap: usize = first_gap(intervals, length, topology);
        if gap < count {
            let problem = UnsupportedFeatureStructureProblem {
                feature,
                sequence: sequence.display_name(),
                interval_count: count,
                reason: "the intervals are not contiguous".to_string(),
            };
            return Err(DomainError::UnsupportedFeatureStructure(problem));
        }
        if abutting_split(intervals, length) {
            let problem = UnsupportedFeatureStructureProblem {
                feature,
                sequence: sequence.display_name(),
                interval_count: count,
                reason: "a circular feature is split only where it crosses the origin".to_string(),
            };
            return Err(DomainError::UnsupportedFeatureStructure(problem));
        }
    }

    Ok(())
}

/// Number of joins between consecutive intervals that cross the origin.
fn origin_crossings(intervals: &[NucleotideInterval], length: u64) -> usize {
    let mut crossings: usize = 0;
    let count: usize = intervals.len();
    let mut index: usize = 1;
    while index < count {
        let previous: NucleotideInterval = intervals[index - 1].clone();
        let current: NucleotideInterval = intervals[index].clone();
        if previous.end() == length && current.start() == 0 {
            crossings += 1;
        }
        index += 1;
    }
    crossings
}

/// True when some join between consecutive intervals abuts away from the
/// origin: the split is legal only at the origin, where an interval ending at
/// the sequence length is followed by one starting at zero (D036).
fn abutting_split(intervals: &[NucleotideInterval], length: u64) -> bool {
    let count: usize = intervals.len();
    let mut index: usize = 1;
    while index < count {
        let previous: NucleotideInterval = intervals[index - 1].clone();
        let current: NucleotideInterval = intervals[index].clone();
        let crossing: bool = previous.end() == length && current.start() == 0;
        if previous.end() == current.start() && !crossing {
            return true;
        }
        index += 1;
    }
    false
}

/// Index of the first join that is not contiguous, or `intervals.len()` when
/// every join is contiguous.
fn first_gap(intervals: &[NucleotideInterval], length: u64, topology: SequenceTopology) -> usize {
    let count: usize = intervals.len();
    let mut index: usize = 1;
    while index < count {
        let previous: NucleotideInterval = intervals[index - 1].clone();
        let current: NucleotideInterval = intervals[index].clone();
        let abutting: bool = previous.end() == current.start();
        let crossing: bool = topology == SequenceTopology::Circular
            && previous.end() == length
            && current.start() == 0;
        if !abutting && !crossing {
            return index;
        }
        index += 1;
    }
    count
}

/// Convert a non-negative signed integer into an unsigned one.
///
/// Callers check the sign first; this function only performs the conversion.
fn unsigned(value: i64) -> u64 {
    if value < 0 {
        return 0;
    }
    value as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DomainError;
    use crate::error::ErrorCategory;
    use crate::identity::Identifier;
    use crate::sequence::normalize;
    use crate::sequence::Sequence;
    use crate::sequence::SequenceTopology;

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
    fn intervals_are_half_open() {
        let interval: NucleotideInterval = create(&seq_a().reference(), 0, 4).unwrap();
        assert!(interval.length() == 4);
        let letters: Vec<char> = seq_a().letters();
        assert!(letters[0] == 'A');
        assert!(letters[3] == 'G');
        assert!(letters[4] == 'C');
    }

    #[test]
    fn negative_coordinates_are_malformed() {
        let error = create(&seq_a().reference(), -1, 4).unwrap_err();
        assert!(error.category() == ErrorCategory::MalformedInput);
        assert!(matches!(error, DomainError::NegativeCoordinate(_)));
    }

    #[test]
    fn reversed_intervals_are_malformed() {
        let error = create(&seq_a().reference(), 8, 4).unwrap_err();
        assert!(error.category() == ErrorCategory::MalformedInput);
        assert!(matches!(error, DomainError::ReversedInterval(_)));
    }

    #[test]
    fn empty_intervals_are_malformed() {
        let error = create(&seq_a().reference(), 4, 4).unwrap_err();
        assert!(error.category() == ErrorCategory::MalformedInput);
        assert!(matches!(error, DomainError::EmptyInterval(_)));
    }

    #[test]
    fn out_of_bounds_is_validation() {
        let interval: NucleotideInterval = create(&seq_a().reference(), 0, 31).unwrap();
        let error = interval.within(&seq_a()).unwrap_err();
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        assert!(matches!(error, DomainError::CoordinateOutOfBounds(_)));
    }

    #[test]
    fn wrong_reference_is_rejected() {
        let intervals: OrderedIntervals = vec![create(&seq_b().reference(), 0, 4).unwrap()];
        let error =
            validate_against("feat-1".to_string(), &seq_a(), intervals.as_slice()).unwrap_err();
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        assert!(matches!(error, DomainError::SequenceReferenceMismatch(_)));
    }

    // The display names are equal; only the content digest differs. A name-only
    // comparison would accept this interval.
    #[test]
    fn same_name_different_content_is_rejected() {
        let alias = normalize(
            Identifier::create("seq-a".to_string()).unwrap(),
            "ATTCCAAGCCATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Linear,
        );
        assert!(alias.is_ok());
        let intervals: OrderedIntervals = vec![create(&alias.unwrap().reference(), 0, 4).unwrap()];
        let error =
            validate_against("feat-1".to_string(), &seq_a(), intervals.as_slice()).unwrap_err();
        assert!(matches!(error, DomainError::SequenceReferenceMismatch(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
    }

    // The fields are private and `create` is the only public constructor, so
    // every interval value in the program satisfies the ordering invariant.
    #[test]
    fn constructed_intervals_obey_the_ordering_invariant() {
        let pairs: Vec<Vec<i64>> = vec![
            vec![0, 1],
            vec![0, 30],
            vec![29, 30],
            vec![4, 4],
            vec![8, 4],
            vec![-1, 4],
            vec![0, -1],
        ];
        for pair in pairs {
            let built: Result<NucleotideInterval, DomainError> =
                create(&seq_a().reference(), pair[0], pair[1]);
            match built {
                Ok(interval) => {
                    assert!(interval.start() < interval.end());
                    assert!(interval.length() > 0);
                    assert!(interval.reference() == seq_a().reference());
                }
                Err(error) => assert!(error.category() == ErrorCategory::MalformedInput),
            }
        }
    }

    #[test]
    fn overlapping_intervals_are_malformed() {
        let name: SequenceReference = seq_a().reference();
        let intervals: OrderedIntervals =
            vec![create(&name, 0, 6).unwrap(), create(&name, 4, 10).unwrap()];
        let error =
            validate_against("feat-1".to_string(), &seq_a(), intervals.as_slice()).unwrap_err();
        assert!(error.category() == ErrorCategory::MalformedInput);
        assert!(matches!(error, DomainError::OverlappingIntervals(_)));
    }

    #[test]
    fn unsorted_intervals_are_malformed() {
        let name: SequenceReference = seq_a().reference();
        let intervals: OrderedIntervals =
            vec![create(&name, 10, 14).unwrap(), create(&name, 0, 4).unwrap()];
        let error =
            validate_against("feat-1".to_string(), &seq_a(), intervals.as_slice()).unwrap_err();
        assert!(error.category() == ErrorCategory::MalformedInput);
        assert!(matches!(error, DomainError::IntervalsOutOfOrder(_)));
    }

    #[test]
    fn linear_features_take_one_interval() {
        let name: SequenceReference = seq_a().reference();
        let intervals: OrderedIntervals =
            vec![create(&name, 0, 4).unwrap(), create(&name, 6, 10).unwrap()];
        let error =
            validate_structure("feat-1".to_string(), &seq_a(), intervals.as_slice()).unwrap_err();
        assert!(error.category() == ErrorCategory::Unsupported);
        assert!(matches!(error, DomainError::UnsupportedFeatureStructure(_)));
    }

    #[test]
    fn circular_features_may_wrap_once() {
        let name: SequenceReference = circular_c().reference();
        let intervals: OrderedIntervals =
            vec![create(&name, 21, 24).unwrap(), create(&name, 0, 6).unwrap()];
        assert!(
            validate_structure("feat-1".to_string(), &circular_c(), intervals.as_slice()).is_ok()
        );
    }

    // D036: the only legal join is the origin. `[0,6)` then `[6,24)` abuts at
    // 6, which hides a join the containment and warning checks treat as an edge.
    #[test]
    fn a_circular_feature_split_away_from_the_origin_is_refused() {
        let name: SequenceReference = circular_c().reference();
        let intervals: OrderedIntervals =
            vec![create(&name, 0, 6).unwrap(), create(&name, 6, 24).unwrap()];
        let error: DomainError =
            validate_structure("feat-1".to_string(), &circular_c(), intervals.as_slice())
                .unwrap_err();
        assert!(error.category() == ErrorCategory::Unsupported);
        match error {
            DomainError::UnsupportedFeatureStructure(problem) => {
                assert!(problem.feature == "feat-1");
                assert!(problem.sequence == "seq-c");
                assert!(problem.interval_count == 2);
                assert!(
                    problem.reason
                        == "a circular feature is split only where it crosses the origin"
                );
            }
            other => panic!("expected UnsupportedFeatureStructure, got {:?}", other),
        }
    }

    // One origin crossing at the 24/0 join, but a second join at 3: still
    // refused, and refused as a split away from the origin rather than as a
    // second crossing or a gap.
    #[test]
    fn a_circular_feature_split_at_the_origin_and_elsewhere_is_refused() {
        let name: SequenceReference = circular_c().reference();
        let intervals: OrderedIntervals = vec![
            create(&name, 20, 24).unwrap(),
            create(&name, 0, 3).unwrap(),
            create(&name, 3, 6).unwrap(),
        ];
        let error: DomainError =
            validate_structure("feat-1".to_string(), &circular_c(), intervals.as_slice())
                .unwrap_err();
        assert!(error.category() == ErrorCategory::Unsupported);
        match error {
            DomainError::UnsupportedFeatureStructure(problem) => {
                assert!(problem.feature == "feat-1");
                assert!(problem.sequence == "seq-c");
                assert!(problem.interval_count == 3);
                assert!(
                    problem.reason
                        == "a circular feature is split only where it crosses the origin"
                );
            }
            other => panic!("expected UnsupportedFeatureStructure, got {:?}", other),
        }
    }

    // No neighbouring pair overlaps, but [21,24) and [6,22) share position 21:
    // the feature would select 25 positions of a 24-base sequence.
    #[test]
    fn a_circular_feature_cannot_select_a_position_twice() {
        let name: SequenceReference = circular_c().reference();
        let intervals: OrderedIntervals = vec![
            create(&name, 21, 24).unwrap(),
            create(&name, 0, 6).unwrap(),
            create(&name, 6, 22).unwrap(),
        ];
        let error = validate_against("feat-1".to_string(), &circular_c(), intervals.as_slice())
            .unwrap_err();
        assert!(error.category() == ErrorCategory::MalformedInput);
        assert!(
            error
                == DomainError::OverlappingIntervals(OverlappingIntervalsProblem {
                    feature: "feat-1".to_string(),
                    sequence: "seq-c".to_string(),
                    start: 6,
                    end: 22,
                })
        );
    }

    // A wrap that covers the whole circle exactly once is still accepted.
    #[test]
    fn a_circular_feature_may_cover_the_whole_sequence_once() {
        let name: SequenceReference = circular_c().reference();
        let intervals: OrderedIntervals = vec![
            create(&name, 21, 24).unwrap(),
            create(&name, 0, 21).unwrap(),
        ];
        assert!(
            validate_against("feat-1".to_string(), &circular_c(), intervals.as_slice()).is_ok()
        );
    }

    // Same name, same bases, other topology: a different artifact.
    #[test]
    fn same_content_other_topology_is_rejected() {
        let linear = normalize(
            Identifier::create("seq-c".to_string()).unwrap(),
            "GCTTGGAACGAACGAACGAACATG".to_string(),
            SequenceTopology::Linear,
        )
        .unwrap();
        assert!(linear.content_hash() == circular_c().content_hash());
        let intervals: OrderedIntervals = vec![create(&circular_c().reference(), 0, 6).unwrap()];
        let error =
            validate_against("feat-1".to_string(), &linear, intervals.as_slice()).unwrap_err();
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        match error {
            DomainError::SequenceReferenceMismatch(problem) => {
                assert!(problem.expected_topology == SequenceTopology::Linear);
                assert!(problem.actual_topology == SequenceTopology::Circular);
                assert!(problem.expected_digest == problem.actual_digest);
            }
            other => panic!("expected SequenceReferenceMismatch, got {:?}", other),
        }
    }

    // ---- insertion points --------------------------------------------------

    #[test]
    fn a_negative_point_is_malformed() {
        let error: DomainError = create_point(&seq_a().reference(), -1).unwrap_err();
        assert!(error.category() == ErrorCategory::MalformedInput);
        assert!(matches!(error, DomainError::NegativeCoordinate(_)));
    }

    // The gap after the last base is a point on the sequence; one past it is
    // not a gap at all.
    #[test]
    fn a_point_may_sit_at_the_end_but_not_past_it() {
        let at_end: InsertionPoint = create_point(&seq_a().reference(), 30).unwrap();
        assert!(at_end.position() == 30);
        assert!(at_end.within(&seq_a()).is_ok());

        let past: InsertionPoint = create_point(&seq_a().reference(), 31).unwrap();
        let error: DomainError = past.within(&seq_a()).unwrap_err();
        assert!(matches!(error, DomainError::CoordinateOutOfBounds(_)));
        match error {
            DomainError::CoordinateOutOfBounds(problem) => {
                assert!(problem.start == 31);
                assert!(problem.end == 31);
                assert!(problem.length == 30);
            }
            other => panic!("expected CoordinateOutOfBounds, got {:?}", other),
        }
    }

    #[test]
    fn circular_gaps_are_unsupported() {
        let name: SequenceReference = circular_c().reference();
        let intervals: OrderedIntervals =
            vec![create(&name, 21, 24).unwrap(), create(&name, 1, 6).unwrap()];
        let error = validate_structure("feat-1".to_string(), &circular_c(), intervals.as_slice())
            .unwrap_err();
        assert!(error.category() == ErrorCategory::Unsupported);
        assert!(matches!(error, DomainError::UnsupportedFeatureStructure(_)));
    }

    // ---- containment over a list of intervals ------------------------------

    // A position is held from `start` up to `end − 1`, the bases an interval
    // selects; `end` names the first base it does not select.
    #[test]
    fn a_position_is_held_at_the_start_and_the_last_base() {
        let name: SequenceReference = seq_a().reference();
        let intervals: OrderedIntervals = vec![create(&name, 10, 14).unwrap()];
        assert!(holds_position(intervals.as_slice(), 10));
        assert!(holds_position(intervals.as_slice(), 13));
        assert!(!holds_position(intervals.as_slice(), 14));
        assert!(!holds_position(intervals.as_slice(), 9));
    }

    // A region is held only when one interval covers all of it: an interval
    // equal to the region is held, a region one wider on either side is not.
    #[test]
    fn a_region_is_held_only_when_one_interval_contains_it() {
        let name: SequenceReference = seq_a().reference();
        let intervals: OrderedIntervals = vec![create(&name, 10, 14).unwrap()];
        assert!(holds_region(intervals.as_slice(), 10, 14));
        assert!(holds_region(intervals.as_slice(), 11, 13));
        assert!(!holds_region(intervals.as_slice(), 9, 14));
        assert!(!holds_region(intervals.as_slice(), 10, 15));
    }

    // A point selects no nucleotide, so it is inside only with a base on each
    // side of it: both `start` and `end` are outside, `start + 1` is inside.
    #[test]
    fn a_point_is_held_only_strictly_inside_an_interval() {
        let name: SequenceReference = seq_a().reference();
        let intervals: OrderedIntervals = vec![create(&name, 10, 14).unwrap()];
        assert!(holds_point(intervals.as_slice(), 11));
        assert!(holds_point(intervals.as_slice(), 13));
        assert!(!holds_point(intervals.as_slice(), 10));
        assert!(!holds_point(intervals.as_slice(), 14));
    }
}
