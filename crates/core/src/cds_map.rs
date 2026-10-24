//! The residue ↔ coordinate map of one CDS, in both directions.
//!
//! The view has to answer two questions without doing coordinate arithmetic of
//! its own: which three reference bases encode a numbered residue, and which
//! residue the base under the cursor belongs to. Both answers come from this
//! module.
//!
//! A `Cds` alone cannot answer either one. Whether the last codon of a CDS is a
//! stop is a fact about the sequence, not about the annotation, and the residue
//! count is the translation's, never `total_length() / 3` (D010: the terminal
//! stop codon is a codon and not a residue, and a CDS with no terminal stop has
//! no codon to exclude). `CdsMap` is therefore built from a `Cds` *and* the
//! `Translation` of that same CDS: `import_cds` step 10 translates every CDS it
//! imports and passes translation errors through (D042), so the two arguments
//! of one import always agree, and this type does not re-check them.
//!
//! Coordinates are zero-based and half-open, as everywhere inside core (D006);
//! 1-based numbering appears only at the user-facing edge. Residue numbers are
//! 1-based by D010 and codon places are 1..=3.
//!
//! Two orders are in play and they are not the same one (D040):
//!
//! * The intervals a *result* records stay in reference order, which D040 says
//!   outright "is not the strand's reading order". That is a rule about records.
//! * `residue_bases` returns the three bases **in the order the codon is read**
//!   — ascending on the plus strand, descending on the minus strand — because
//!   that is what `mutation.rs::codon_positions` returns and what the view
//!   draws. The fixture tables name the order in every column header for the
//!   same reason (D050).
//!
//! The arithmetic itself is done in zero-based biological offsets inside the
//! CDS, measured in the order the intervals are stored: offset 0 is the first
//! base of the first stored interval, the same convention
//! `mutation.rs::reference_of` uses. A D036 origin-crossing CDS stores `[a, n)`
//! before `[0, b)`, so its offsets run through the origin and a codon can be
//! split between two intervals without any special case here. A minus-strand
//! CDS reads that stored arc backwards, which is what makes its first codon the
//! complement of the arc's lowest-numbered base when the arc wraps (D036 plus
//! the hand-derived `mj1`/`mj2` reading in `tests/fixtures/genbank/README.md`).
//!
//! Bases that are not part of a complete codon get no answer (D050): a position
//! in no interval, and a base of the incomplete final codon of a CDS whose
//! length is not a multiple of three, map to `None`. The second case cannot be
//! produced by an import, because D042 refuses such a CDS and `translate` treats
//! its length as validation; it is defensive arithmetic over a public type.
//!
//! `CdsMap` derives `Clone`, `Debug` and `PartialEq` and can derive no more:
//! `NucleotideInterval` and `Strand` are themselves `PartialEq`-only, so a
//! reflexive `Eq` over a map is not derivable without changing those types.
//! `CodonPlace` holds only integers, so it is `Copy` and `Eq`.

use crate::cds::Cds;
use crate::error::DomainError;
use crate::error::InvalidMutationRequestProblem;
use crate::error::MutationRequestRule;
use crate::interval::NucleotideInterval;
use crate::mutation::codon_positions;
use crate::strand::Strand;
use crate::translation::Translation;

/// Where a reference base sits inside a CDS.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CodonPlace {
    /// Which codon of the CDS, counted in reading order, starting at 1.
    pub codon: u64,
    /// The base's place within that codon, 1..=3 in reading order.
    pub place: u64,
    /// The residue number this codon produces, or `None` for the terminal
    /// stop codon (D010) and for a codon the CDS does not have.
    pub residue: Option<u64>,
}

/// The residue ↔ coordinate map of one CDS, built from the CDS and its
/// translation. `Cds` alone cannot answer either direction: whether the last
/// codon is a stop is a fact about the sequence, and the residue count is the
/// translation's, not `total_length() / 3` (D010).
#[derive(Clone, Debug, PartialEq)]
pub struct CdsMap {
    intervals: Vec<NucleotideInterval>,
    strand: Strand,
    total_length: u64,
    residues: u64,
    terminal_stop: bool,
}

impl CdsMap {
    /// The map of one CDS, as the import that made it has it: the CDS's stored
    /// intervals, strand and selected length, and the residue count and
    /// terminal-stop flag of the translation taken from the same import (D042).
    ///
    /// Nothing is translated here, no `Sequence` is needed, and nothing can
    /// fail: both arguments are already validated values.
    pub fn new(cds: &Cds, translation: &Translation) -> CdsMap {
        CdsMap {
            intervals: cds.intervals(),
            strand: cds.strand(),
            total_length: cds.total_length(),
            residues: translation.residue_count() as u64,
            terminal_stop: translation.terminal_stop(),
        }
    }

    /// Residues, excluding a terminal stop codon (D010).
    pub fn residues(&self) -> u64 {
        self.residues
    }

    /// Complete codons: `total_length() / 3`, floored.
    pub fn codons(&self) -> u64 {
        self.total_length / 3
    }

    pub fn terminal_stop(&self) -> bool {
        self.terminal_stop
    }

    /// The three reference positions of residue `number`'s codon, in the
    /// order the codon is read: ascending on the plus strand, descending on
    /// the minus strand. Zero-based half-open coordinates (D006).
    ///
    /// A number with no residue — 0, or anything above `residues()`, which
    /// would address the terminal stop codon — is refused the way
    /// `codon_positions` refuses it: `DomainError::InvalidMutationRequest`
    /// with rule `CodonOutsideSequence`.
    pub fn residue_bases(&self, number: u64) -> Result<[u64; 3], DomainError> {
        // Checked before the walk so residue `residues + 1`, the terminal stop
        // codon, is refused rather than drawn: it has a codon and no residue
        // (D010). Every multiply and subtract under `codon_positions` is
        // checked there, so a huge number cannot wrap.
        if number == 0 || number > self.residues {
            return Err(no_codon_for_number());
        }
        let positions: Vec<u64> = codon_positions(
            self.intervals.clone(),
            self.strand.clone(),
            number,
            self.total_length,
        )?;
        // `codon_positions` builds exactly three positions or refuses, so a
        // list of any other length is a bug in the walk rather than an answer.
        if positions.len() != 3 {
            return Err(no_codon_for_number());
        }
        Ok([positions[0], positions[1], positions[2]])
    }

    /// Which codon holds this zero-based reference position, and what residue
    /// that codon produces. `None` for a position outside the CDS, and for a
    /// base of an incomplete final codon when the CDS's length is not a
    /// multiple of three (D050 gives leftover bases nothing).
    pub fn base_residue(&self, position: u64) -> Option<CodonPlace> {
        let offset: u64 = self.offset_of(position)?;
        // A minus-strand CDS reads the stored arc from its far end, the same
        // flip `codon_positions` applies to the offsets of a codon.
        let read: u64 = if self.strand.clone().is_plus() {
            offset
        } else {
            self.total_length.checked_sub(1)?.checked_sub(offset)?
        };
        let index: u64 = read / 3;
        // The leftover bases of a CDS whose length is not a multiple of three
        // are past the last complete codon, and D050 gives them nothing.
        if index >= self.codons() {
            return None;
        }
        // A codon at or past `residues` is the terminal stop codon: a codon,
        // with no residue number (D010).
        let residue: Option<u64> = if index < self.residues {
            Some(index.checked_add(1)?)
        } else {
            None
        };
        Some(CodonPlace {
            codon: index.checked_add(1)?,
            place: read % 3 + 1,
            residue,
        })
    }

    /// The biological offset of a reference position: how many CDS bases are
    /// stored before it, counted in the order the intervals are stored. This is
    /// the inverse of `mutation.rs::reference_of`. `None` when no interval
    /// selects the position.
    fn offset_of(&self, position: u64) -> Option<u64> {
        let mut seen: u64 = 0;
        for interval in self.intervals.iter() {
            if position >= interval.start() && position < interval.end() {
                return seen.checked_add(position.checked_sub(interval.start())?);
            }
            seen = seen.checked_add(interval.length())?;
        }
        None
    }
}

/// The refusal of a residue number this CDS has no codon for, as
/// `codon_positions` refuses one: the same problem and the same rule (D010).
fn no_codon_for_number() -> DomainError {
    let problem = InvalidMutationRequestProblem {
        request: "codon coordinates".to_string(),
        rule: MutationRequestRule::CodonOutsideSequence,
        reason: "the residue does not name a codon this CDS translates".to_string(),
    };
    DomainError::InvalidMutationRequest(problem)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cds::Cds;
    use crate::cds::Completeness;
    use crate::cds::Numbering;
    use crate::error::DomainError;
    use crate::error::MutationRequestRule;
    use crate::genetic_code::STANDARD_CODE_NAME;
    use crate::identity::Identifier;
    use crate::interval::create;
    use crate::interval::OrderedIntervals;
    use crate::sequence::normalize;
    use crate::sequence::Sequence;
    use crate::sequence::SequenceTopology;
    use crate::strand::Strand;
    use crate::translation::translate;

    fn sequence(name: &str, text: &str, topology: SequenceTopology) -> Sequence {
        normalize(
            Identifier::create(name.to_string()).unwrap(),
            text.to_string(),
            topology,
        )
        .unwrap()
    }

    fn map_of(sequence: &Sequence, intervals: OrderedIntervals, strand: Strand) -> CdsMap {
        let cds: Cds = Cds::create(
            sequence,
            Identifier::create("cds-1".to_string()).unwrap(),
            intervals,
            strand,
            Completeness::Complete,
            STANDARD_CODE_NAME.to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap();
        let translation: Translation = translate(sequence, &cds).unwrap();
        CdsMap::new(&cds, &translation)
    }

    // ATG GCT TGG TAA: M A W stop.
    fn plus_linear() -> (Sequence, CdsMap) {
        let seq: Sequence = sequence("seq-p", "ATGGCTTGGTAA", SequenceTopology::Linear);
        let intervals: OrderedIntervals = vec![create(&seq.reference(), 0, 12).unwrap()];
        let map: CdsMap = map_of(&seq, intervals, Strand::Plus);
        (seq, map)
    }

    // The plus-strand text of m1 in tests/fixtures/genbank/synthetic.gb: its
    // reverse complement reads ATG CAG CAG TAG, M Q Q stop.
    fn minus_linear() -> (Sequence, CdsMap) {
        let seq: Sequence = sequence("seq-m", "CTACTGCTGCAT", SequenceTopology::Linear);
        let intervals: OrderedIntervals = vec![create(&seq.reference(), 0, 12).unwrap()];
        let map: CdsMap = map_of(&seq, intervals, Strand::Minus);
        (seq, map)
    }

    // A 6-mer whose CDS arc is [2,6) then [0,2): codon 1 is bases 2,3,4 and
    // codon 2 is bases 5,0,1, which straddles the origin.
    //
    //
    // The literal is what it is because `translate` runs before any map can be
    // built: codon 1 is the arc's first three bases (`TGG`, Trp) and codon 2 is
    // arc offsets 3, 4 and 5 - reference bases 5, 0 and 1 - so codon 2 is both
    // the straddling codon and the terminal stop. Do not "tidy" the sequence:
    // an arc whose first three bases are a stop is refused as an internal stop
    // codon, and an arc whose stop is not its last codon pins nothing about the
    // origin.
    fn joined_circular() -> (Sequence, CdsMap) {
        let seq: Sequence = sequence("seq-j", "AATGGT", SequenceTopology::Circular);
        let intervals: OrderedIntervals = vec![
            create(&seq.reference(), 2, 6).unwrap(),
            create(&seq.reference(), 0, 2).unwrap(),
        ];
        let map: CdsMap = map_of(&seq, intervals, Strand::Plus);
        (seq, map)
    }

    #[test]
    fn a_plus_codon_is_returned_ascending() {
        let (_seq, map) = plus_linear();
        assert_eq!(map.residues(), 3);
        assert_eq!(map.codons(), 4);
        assert!(map.terminal_stop());
        assert_eq!(map.residue_bases(1).unwrap(), [0, 1, 2]);
        assert_eq!(map.residue_bases(3).unwrap(), [6, 7, 8]);
    }

    #[test]
    fn a_minus_codon_is_returned_descending() {
        let (_seq, map) = minus_linear();
        assert_eq!(map.residues(), 3);
        assert_eq!(map.residue_bases(1).unwrap(), [11, 10, 9]);
        assert_eq!(map.residue_bases(3).unwrap(), [5, 4, 3]);
    }

    #[test]
    fn a_joined_codon_straddles_the_origin() {
        let (_seq, map) = joined_circular();
        assert_eq!(map.codons(), 2);
        assert_eq!(map.residues(), 1);
        assert_eq!(map.residue_bases(1).unwrap(), [2, 3, 4]);
        // The stop codon's three bases: 5, then 0 and 1.
        for position in [5, 0, 1] {
            let place: CodonPlace = map.base_residue(position).unwrap();
            assert_eq!(place.codon, 2);
            assert_eq!(place.residue, None);
        }
        assert_eq!(
            map.base_residue(4).unwrap(),
            CodonPlace {
                codon: 1,
                place: 3,
                residue: Some(1)
            }
        );
    }

    #[test]
    fn the_terminal_stop_has_a_codon_and_no_residue() {
        let (_seq, map) = plus_linear();
        assert_eq!(
            map.base_residue(9).unwrap(),
            CodonPlace {
                codon: 4,
                place: 1,
                residue: None
            }
        );
        assert_eq!(
            map.base_residue(11).unwrap(),
            CodonPlace {
                codon: 4,
                place: 3,
                residue: None
            }
        );
        // D010: the stop codon is not residue 4.
        let error: DomainError = map.residue_bases(4).unwrap_err();
        assert!(matches!(
            error,
            DomainError::InvalidMutationRequest(problem)
                if problem.rule == MutationRequestRule::CodonOutsideSequence
        ));
        assert!(map.residue_bases(3).is_ok());
    }

    #[test]
    fn residue_zero_is_refused() {
        let (_seq, map) = plus_linear();
        assert!(matches!(
            map.residue_bases(0).unwrap_err(),
            DomainError::InvalidMutationRequest(problem)
                if problem.rule == MutationRequestRule::CodonOutsideSequence
        ));
    }

    // ATG GCT TGG: no stop codon at all. Nothing is excluded, so every codon
    // of this CDS is a residue (D010).
    #[test]
    fn a_cds_with_no_terminal_stop_numbers_every_codon() {
        let seq: Sequence = sequence("seq-n", "ATGGCTTGG", SequenceTopology::Linear);
        let intervals: OrderedIntervals = vec![create(&seq.reference(), 0, 9).unwrap()];
        let map: CdsMap = map_of(&seq, intervals, Strand::Plus);
        assert!(!map.terminal_stop());
        assert_eq!(map.codons(), 3);
        assert_eq!(map.residues(), 3);
        assert_eq!(map.residue_bases(3).unwrap(), [6, 7, 8]);
        assert_eq!(
            map.base_residue(8).unwrap(),
            CodonPlace {
                codon: 3,
                place: 3,
                residue: Some(3)
            }
        );
        assert!(map.residue_bases(4).is_err());
    }

    // 13 bases: the final codon is incomplete, and D050 gives leftover bases
    // nothing. `translate` refuses such a CDS and D042 refuses the feature, so
    // no import can produce this map; the type is public, so its rule is
    // tested by building the value directly.
    #[test]
    fn a_base_of_an_incomplete_final_codon_has_no_place() {
        let seq: Sequence = sequence("seq-l", "ATGGCTTGGAATCAG", SequenceTopology::Linear);
        let map: CdsMap = CdsMap {
            intervals: vec![create(&seq.reference(), 0, 13).unwrap()],
            strand: Strand::Plus,
            total_length: 13,
            residues: 4,
            terminal_stop: false,
        };
        assert_eq!(map.codons(), 4);
        assert_eq!(
            map.base_residue(11).unwrap(),
            CodonPlace {
                codon: 4,
                place: 3,
                residue: Some(4)
            }
        );
        assert_eq!(map.base_residue(12), None);
        assert_eq!(map.residue_bases(4).unwrap(), [9, 10, 11]);
        assert!(map.residue_bases(5).is_err());
    }

    #[test]
    fn a_position_outside_the_cds_maps_to_nothing() {
        let (_seq, map) = plus_linear();
        assert_eq!(map.base_residue(12), None);
        assert_eq!(map.base_residue(1000), None);
    }
}
