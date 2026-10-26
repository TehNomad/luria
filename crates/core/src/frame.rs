//! The mutant reading frame: the residues a ribosome reads in the edited
//! construct (plan §5.3a, M7-18 to M7-21; contract 7i1).
//!
//! The TUI may not translate or compute coordinates (D003, amended by M7-21),
//! so this module does: given the edited construct, the CDS as annotated on the
//! template, and the edit, it returns each mutant residue whose codon's middle
//! base lies in a window of the edited construct, with its number, its symbol
//! and that middle base's position.
//!
//! The rules, as `tests/fixtures/review/reference/frame.py` applies them
//! (M7-20):
//!
//! * the CDS's first base on its own strand (`start` for `+`, `end - 1` for
//!   `-`) is carried into the edited construct: kept when it lies before the
//!   edit's start, moved by the edit's length change (`written length -
//!   (edit end - edit start)`) when it lies at or past the edit's end; a first
//!   base inside the edited region is refused;
//! * from there codons are read three bases at a time along the CDS's strand
//!   (for `-`, the complement of positions `p`, `p-1`, `p-2`), in the CDS's
//!   frame, on past any frameshift, whole codons only;
//! * the first stop codon gives `*` and ends the protein; a codon with a base
//!   that is not `A`, `C`, `G` or `T` gives `?` and ends it;
//! * reading also ends at the construct's end, or once a codon's middle base
//!   is past the window in the reading direction;
//! * a residue is returned when its middle base lies in `[window_start,
//!   window_end)` of the edited construct; numbers count from 1 at the CDS's
//!   first codon.
//!
//! The CDS is read for its coordinates only: its reference is never compared
//! with `edited`, because the annotation lives on the template and the reading
//! happens on the edited construct.

use crate::cds::Cds;
use crate::error::DomainError;
use crate::error::MutationVerificationFailureProblem;
use crate::error::ReversedIntervalProblem;
use crate::error::UnsupportedFeatureStructureProblem;
use crate::genetic_code::lookup;
use crate::genetic_code::CodonOutcome;
use crate::interval::NucleotideInterval;
use crate::interval::OrderedIntervals;
use crate::sequence::canonical_text;
use crate::sequence::complement_letter;
use crate::sequence::Sequence;
use crate::strand::Strand;

/// One residue laid on the sequence it was read from: the edited construct for
/// `mutant_residues`, a record for `crate::frames::frame_residues`. At both
/// call sites the three fields mean the same three things — an index counting
/// from 1 in the context being read, the one-letter symbol, and the position of
/// the codon's middle base on that same sequence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameResidue {
    /// 1 at the CDS's first codon.
    pub number: u64,
    /// The one-letter symbol, `*` for a stop, `?` for an unreadable codon.
    pub symbol: char,
    /// The zero-based position, within the sequence this residue was laid on,
    /// of the codon's middle base: inside the edited construct for
    /// `mutant_residues`, and a zero-based position of the record for
    /// `crate::frames::frame_residues`. Both are zero-based half-open
    /// spaces (D006), and they are not the same space.
    pub middle: u64,
}

/// The edit, in template coordinates: `[start, end)` replaced by
/// `written_length` bases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditSpan {
    pub start: u64,
    pub end: u64,
    pub written_length: u64,
}

/// The symbol a codon gives: the residue's letter, `*` for a stop, `?` for a
/// codon that cannot be read, that is one with a base outside `A`, `C`, `G`,
/// `T` — an IUPAC ambiguity code, which stands for several bases, or a letter
/// outside the DNA alphabet.
pub(crate) fn codon_symbol(codon: String) -> char {
    match lookup(codon) {
        CodonOutcome::Residue(residue) => {
            let text: String = residue.symbol();
            let mut letters: std::str::Chars<'_> = text.chars();
            letters.next().unwrap_or('?')
        }
        CodonOutcome::Stop => '*',
        CodonOutcome::Ambiguous => '?',
        CodonOutcome::Invalid => '?',
    }
}

/// The CDS's first base on its own strand, in template coordinates: the
/// `start` of the first interval for `+`, `end - 1` of the first for `-`.
///
/// The intervals are taken in their own order (D040), and the list must be
/// non-empty, which `Cds::create` guarantees.
fn first_base(cds: &Cds, intervals: &[NucleotideInterval]) -> u64 {
    let first: &NucleotideInterval = &intervals[0];
    match cds.strand() {
        Strand::Plus => first.start(),
        Strand::Minus => first.end() - 1,
    }
}

/// The codon read at `position` of `letters`, reading forward when `plus` and
/// backward otherwise, or `None` when fewer than three whole bases of the
/// construct stand at it.
///
/// A minus-strand codon is the complement of positions `p`, `p-1`, `p-2`, in
/// that order. A base outside the DNA alphabet cannot be complemented; it
/// becomes `N`, which the genetic code reports as unreadable.
pub(crate) fn codon_at(letters: &[char], position: i64, plus: bool) -> Option<String> {
    let length: i64 = letters.len() as i64;
    if plus {
        if position < 0 || position + 2 >= length {
            return None;
        }
        let start: usize = position as usize;
        return Some(canonical_text(letters[start..start + 3].to_vec()));
    }
    if position - 2 < 0 {
        return None;
    }
    let top: usize = position as usize;
    let mut text: String = String::new();
    let mut step: usize = 0;
    while step < 3 {
        let base: char = letters[top - step];
        match complement_letter(base) {
            Some(complement) => text.push(complement),
            None => text.push('N'),
        }
        step += 1;
    }
    Some(text)
}

/// The residues of the mutant protein whose codon's middle base lies in
/// `[window_start, window_end)` of `edited` (plan §5.3a, M7-20).
///
/// `cds` is the CDS as annotated on the **template**: its intervals give the
/// first base and its strand gives the reading direction, and its reference is
/// not compared with `edited`. `edit` is the edit in template coordinates.
///
/// Refusals: a CDS with more than one interval (a split CDS needs its own
/// rule), a first base inside the edited region, an edit whose `start > end`,
/// a window with `start > end`. Reading ends at the construct's end, at the
/// first stop or unreadable codon, or once the middle base runs past the
/// window; positions past the end of the construct never panic and never
/// underflow.
pub fn mutant_residues(
    edited: &Sequence,
    cds: &Cds,
    edit: EditSpan,
    window_start: u64,
    window_end: u64,
) -> Result<Vec<FrameResidue>, DomainError> {
    let label: String = cds.id().value().to_string();
    let sequence: String = edited.display_name();

    if edit.end < edit.start {
        let problem = ReversedIntervalProblem {
            sequence: sequence.clone(),
            start: edit.start,
            end: edit.end,
        };
        return Err(DomainError::ReversedInterval(problem));
    }
    if window_end < window_start {
        let problem = ReversedIntervalProblem {
            sequence: sequence.clone(),
            start: window_start,
            end: window_end,
        };
        return Err(DomainError::ReversedInterval(problem));
    }

    let intervals: OrderedIntervals = cds.intervals();
    if intervals.is_empty() {
        return Err(DomainError::MutationVerificationFailure(
            MutationVerificationFailureProblem {
                result: label,
                reason: "the CDS has no intervals to give a first base".to_string(),
            },
        ));
    }
    if intervals.len() > 1 {
        let problem = UnsupportedFeatureStructureProblem {
            feature: label,
            sequence: sequence.clone(),
            interval_count: intervals.len(),
            reason: "a split CDS needs its own reading rule".to_string(),
        };
        return Err(DomainError::UnsupportedFeatureStructure(problem));
    }

    let plus: bool = cds.strand().is_plus();
    let first: u64 = first_base(cds, intervals.as_slice());
    if first >= edit.start && first < edit.end {
        let problem = MutationVerificationFailureProblem {
            result: label,
            reason: format!(
                "the CDS's first base at template position {} lies inside the edited region [{}, {})",
                first, edit.start, edit.end
            ),
        };
        return Err(DomainError::MutationVerificationFailure(problem));
    }

    // The CDS's first base is kept before the edit, and moved by the edit's
    // length change at or past the edit's end.
    let delta: i64 = edit.written_length as i64 - (edit.end - edit.start) as i64;
    let mut position: i64 = if first < edit.start {
        first as i64
    } else {
        first as i64 + delta
    };

    let letters: Vec<char> = edited.letters();
    let lo: i64 = window_start as i64;
    let hi: i64 = window_end as i64;

    let mut out: Vec<FrameResidue> = Vec::new();
    let mut number: u64 = 0;

    while let Some(codon) = codon_at(letters.as_slice(), position, plus) {
        let middle: i64 = if plus { position + 1 } else { position - 1 };
        if (plus && middle >= hi) || (!plus && middle < lo) {
            break;
        }
        number += 1;
        let symbol: char = codon_symbol(codon);
        if middle >= lo && middle < hi {
            out.push(FrameResidue {
                number,
                symbol,
                middle: middle as u64,
            });
        }
        if symbol == '*' || symbol == '?' {
            break;
        }
        position += if plus { 3 } else { -3 };
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cds::Completeness;
    use crate::cds::Numbering;
    use crate::error::ErrorCategory;
    use crate::identity::Identifier;
    use crate::interval::create;
    use crate::sequence::normalize;
    use crate::sequence::SequenceTopology;

    fn seq(letters: &str) -> Sequence {
        normalize(
            Identifier::create("syn".to_string()).unwrap(),
            letters.to_string(),
            SequenceTopology::Linear,
        )
        .unwrap()
    }

    fn circular(letters: &str) -> Sequence {
        normalize(
            Identifier::create("syn".to_string()).unwrap(),
            letters.to_string(),
            SequenceTopology::Circular,
        )
        .unwrap()
    }

    fn cds_on(
        sequence: &Sequence,
        start: i64,
        end: i64,
        strand: Strand,
    ) -> Result<Cds, DomainError> {
        let intervals: OrderedIntervals = vec![create(&sequence.reference(), start, end).unwrap()];
        Cds::create(
            sequence,
            Identifier::create("cds-1".to_string()).unwrap(),
            intervals,
            strand,
            Completeness::Complete,
            "standard".to_string(),
            Numbering::FromCdsStart,
        )
    }

    fn edit(start: u64, end: u64, written_length: u64) -> EditSpan {
        EditSpan {
            start,
            end,
            written_length,
        }
    }

    fn text_of(residues: &[FrameResidue]) -> String {
        let mut parts: Vec<String> = Vec::new();
        for residue in residues {
            parts.push(format!(
                "{}:{}:{}",
                residue.number, residue.symbol, residue.middle
            ));
        }
        parts.join(";")
    }

    // A split CDS has no single first base to carry into the edited construct:
    // a join needs its own rule (D036), so it is refused rather than read as
    // its first interval alone. Only a circular sequence lets one be built.
    #[test]
    fn a_split_cds_is_refused() {
        // 24 bases, joined at the origin: `[21, 24)` then `[0, 6)`.
        let template: Sequence = circular("ATGGCTTGGAATCAGCAGCAGCAG");
        let joined: OrderedIntervals = vec![
            create(&template.reference(), 21, 24).unwrap(),
            create(&template.reference(), 0, 6).unwrap(),
        ];
        let cds: Cds = Cds::create(
            &template,
            Identifier::create("cds-1".to_string()).unwrap(),
            joined,
            Strand::Plus,
            Completeness::Complete,
            "standard".to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap();
        let edited: Sequence = seq("ATGGCTTGGAATCAGCAGCAGCAG");
        let error: DomainError =
            mutant_residues(&edited, &cds, edit(12, 13, 1), 0, 24).unwrap_err();
        assert!(matches!(error, DomainError::UnsupportedFeatureStructure(_)));
        assert!(error.category() == ErrorCategory::Unsupported);
        match error {
            DomainError::UnsupportedFeatureStructure(problem) => {
                assert!(problem.interval_count == 2);
                assert!(problem.feature == "cds-1");
            }
            other => panic!("expected UnsupportedFeatureStructure, got {:?}", other),
        }
    }

    // A first base inside the edited region is deleted by the edit, so the
    // mutant protein has no start to read from.
    #[test]
    fn a_start_inside_the_edit_is_refused() {
        let template: Sequence = seq("ATGGCTTGGAATCAGCAGCAGCAGCAGCAG");
        let cds: Cds = cds_on(&template, 3, 26, Strand::Plus).unwrap();
        let edited: Sequence = seq("ATGGCTTGGAATCAGCAGCAGCAGCAGCAG");
        let error: DomainError = mutant_residues(&edited, &cds, edit(3, 6, 3), 0, 26).unwrap_err();
        assert!(matches!(error, DomainError::MutationVerificationFailure(_)));
        assert!(error.category() == ErrorCategory::BiologicalValidation);
        match error {
            DomainError::MutationVerificationFailure(problem) => {
                assert!(problem.result == "cds-1");
                assert!(problem.reason.contains("inside the edited region"));
            }
            other => panic!("expected MutationVerificationFailure, got {:?}", other),
        }
    }

    #[test]
    fn an_empty_window_gives_nothing() {
        let sequence: Sequence = seq("ATGGCTTGGAATCAGCAGCAGCAGCAGCAG");
        let cds: Cds = cds_on(&sequence, 0, 24, Strand::Plus).unwrap();
        let residues: Vec<FrameResidue> =
            mutant_residues(&sequence, &cds, edit(24, 24, 0), 10, 10).unwrap();
        assert!(residues.is_empty());
        let minus: Cds = cds_on(&sequence, 0, 24, Strand::Minus).unwrap();
        let back: Vec<FrameResidue> =
            mutant_residues(&sequence, &minus, edit(24, 24, 0), 10, 10).unwrap();
        assert!(back.is_empty());
    }

    // A last codon the construct's end cuts is not read: whole codons only,
    // and the reading ends there without panicking.
    #[test]
    fn reading_stops_at_the_construct_end() {
        let sequence: Sequence = seq("ATGGCTTGGA");
        let cds: Cds = cds_on(&sequence, 0, 10, Strand::Plus).unwrap();
        let residues: Vec<FrameResidue> =
            mutant_residues(&sequence, &cds, edit(10, 10, 0), 0, 10).unwrap();
        assert!(text_of(residues.as_slice()) == "1:M:1;2:A:4;3:W:7");
        assert!(residues.len() == 3);

        // The same cut read from the minus strand: positions 9, 8, 7 are
        // whole, 6, 5, 4, then 3, 2, 1, then 0 needs -1 and -2, which are
        // outside the construct.
        let minus: Cds = cds_on(&sequence, 0, 10, Strand::Minus).unwrap();
        let back: Vec<FrameResidue> =
            mutant_residues(&sequence, &minus, edit(10, 10, 0), 0, 10).unwrap();
        assert!(back.len() == 3);
        assert!(back[0].number == 1 && back[0].middle == 8);
        assert!(back[2].number == 3 && back[2].middle == 2);
    }
}
