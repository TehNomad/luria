//! The six reading frames of a record: three per strand, translated on the
//! record's own unchanged sequence text (plan §4.3; D052, D050).
//!
//! This module and `crate::frame` are different things, and the names are close
//! enough that the difference has to be written down. `frame.rs` reads a **CDS
//! through an edit on an edited construct**: it carries the CDS's first base
//! across the edit, walks codons in the CDS's frame, and stops at the first
//! stop codon, the first unreadable codon, or the moment the reading runs out
//! of construct. `frames.rs` reads **a record's frames on unchanged sequence
//! text**: there is no CDS and no edit, and it stops only at the end of the
//! sequence — a `*` or a `?` gets a residue and the frame carries on past it.
//!
//! The rules, as §4.3, D052 and D050 give them:
//!
//! * Frames are numbered 1, 2, 3 on either strand, from the base of that
//!   strand's own first codon: plus frame *f* starts at 1-based base *f*
//!   (zero-based position `f - 1`) and steps up by 3; minus frame *f* starts at
//!   the last base minus (*f* − 1) (zero-based `length - f`) and steps down by
//!   3. Six frames means three per strand, never a 4..6 numbering.
//! * A frame **never runs across the origin** (D052, M8-2), and `topology()` is
//!   never consulted here: a circular record is framed exactly like a linear
//!   record of the same text. `codon_at` answers `None` as soon as three whole
//!   bases do not stand at a position, and that *is* the no-wrap rule — no
//!   padding, no rotation, no reading base 1 after the last base.
//! * A leftover base is not drawn (D050, M8-18): when the length is not a
//!   multiple of 3 the 1 or 2 bases at the far end of a frame get no residue at
//!   all, because a codon that would use them is not a complete codon. `?` is
//!   reserved for a *complete* codon that cannot be read.
//! * The frame has its own numbering: `number` is 1 at the frame's first codon
//!   and counts every complete codon, stops and unreadable codons included
//!   (D050). It has nothing to do with a CDS residue number. `cds-map-bases.tsv`
//!   holds that other scale; the two columns look alike and are not the same
//!   thing.
//! * `middle` is a **reference position**, so it decreases as a minus frame
//!   reads. That is the same fact D040 records about reading order versus
//!   reference order: the order the frame walks in is not the order the
//!   coordinates run in, and a minus frame's residue list runs from the high
//!   end of the record to the low end.
//! * The window is half-open `[window_start, window_end)` in zero-based
//!   reference coordinates (D006) and is tested against the codon's **middle**
//!   base — not its first base and not its span. `window_end < window_start`
//!   is refused with `DomainError::ReversedInterval`, the same refusal
//!   `mutant_residues` builds; `window_start == window_end` is legal and empty.
//!
//! The letter itself is decided by `crate::frame::codon_symbol`, which is
//! `genetic_code::lookup` read one way: a letter, `*` for a stop, `?` for a
//! codon holding any base outside `A C G T` (D052 makes `N` always `?`). There
//! is no second mapping here, and an ambiguous codon is never collapsed to the
//! letter its expansions agree on — Biopython answers `G` for `GGN`, `L` for
//! `CTN` and `*` for `TAR`; D010 and D052 say there is no guess to make, and
//! `tests/fixtures/view/README.md` records that disagreement rather than
//! hiding it.

use crate::error::DomainError;
use crate::error::ReversedIntervalProblem;
use crate::frame::codon_at;
use crate::frame::codon_symbol;
use crate::sequence::Sequence;
use crate::strand::Strand;

// The two readings mean the same three things, so there is one residue type
// (§2 of contract 8d): an index counting from 1 in the context being read, the
// one-letter symbol, and the zero-based position of the codon's middle base.
pub use crate::frame::FrameResidue;

/// One of the three reading frames of a strand. Six frames means three per
/// strand, numbered from the base of that strand's own first codon (§4.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadingFrame {
    First,
    Second,
    Third,
}

impl ReadingFrame {
    /// All three, in order — the view draws six lanes, this times two strands.
    pub const ALL: [ReadingFrame; 3] = [
        ReadingFrame::First,
        ReadingFrame::Second,
        ReadingFrame::Third,
    ];

    /// 1, 2 or 3. This is the number `frame-tables.tsv`'s `frame` column holds.
    pub fn index(self) -> u64 {
        match self {
            ReadingFrame::First => 1,
            ReadingFrame::Second => 2,
            ReadingFrame::Third => 3,
        }
    }

    /// `None` for 0 and for anything above 3.
    pub fn from_index(value: u64) -> Option<ReadingFrame> {
        match value {
            1 => Some(ReadingFrame::First),
            2 => Some(ReadingFrame::Second),
            3 => Some(ReadingFrame::Third),
            _ => None,
        }
    }
}

/// The residues of one frame of `sequence` whose codon's middle base lies in
/// `[window_start, window_end)` of that sequence.
///
/// Unlike `crate::frame::mutant_residues`, **a frame does not stop at a stop
/// codon or at `?`**: every complete codon of the frame gets a residue, and
/// `tests/fixtures/view/frame-tables.tsv` pins it — `framesamb` plus frame 1
/// keeps reading after both `TAA` and `AAN`. Reading ends only when
/// `codon_at` finds no three whole bases at the position, which on a linear
/// coordinate line is the end of the sequence (§4.3, D052).
///
/// `sequence.topology()` is never read: a circular record and a linear record
/// of the same text give the same residues (§4.3: "The numbering is the same on
/// a circular record"; D052). A window with `window_end < window_start` is
/// refused with `DomainError::ReversedInterval`; an empty window is legal.
/// A record shorter than three bases, or shorter than the frame's offset,
/// gives an empty vector rather than a panic or a wrapped position.
pub fn frame_residues(
    sequence: &Sequence,
    strand: Strand,
    frame: ReadingFrame,
    window_start: u64,
    window_end: u64,
) -> Result<Vec<FrameResidue>, DomainError> {
    if window_end < window_start {
        let problem = ReversedIntervalProblem {
            sequence: sequence.display_name(),
            start: window_start,
            end: window_end,
        };
        return Err(DomainError::ReversedInterval(problem));
    }

    let letters: Vec<char> = sequence.letters();
    let plus: bool = strand.is_plus();

    // The frame's start, zero-based: base `f` (1-based) on the plus strand,
    // the last base minus `f - 1` on the minus strand. The length is taken as
    // `i64` — the same idiom `codon_at` uses — so a record shorter than the
    // frame's offset gives a start outside the sequence rather than a wrapped
    // `u64`, and `codon_at` answers `None` for it at once.
    let length: i64 = letters.len() as i64;
    let offset: i64 = frame.index() as i64 - 1;
    let mut position: i64 = if plus { offset } else { length - 1 - offset };

    let mut out: Vec<FrameResidue> = Vec::new();
    let mut number: u64 = 0;

    while let Some(codon) = codon_at(letters.as_slice(), position, plus) {
        // `Some` from `codon_at` means three whole bases stand at `position`:
        // `position >= 0` on the plus strand and `position >= 2` on the minus
        // strand, so the middle base is a real reference position and the
        // conversion to `u64` cannot wrap.
        let middle: u64 = if plus {
            (position + 1) as u64
        } else {
            (position - 1) as u64
        };
        number += 1;
        // Half-open window, tested on the middle base (D006, §4.3).
        if middle >= window_start && middle < window_end {
            out.push(FrameResidue {
                number,
                symbol: codon_symbol(codon),
                middle,
            });
        }
        position += if plus { 3 } else { -3 };
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::Identifier;
    use crate::sequence::normalize;
    use crate::sequence::SequenceTopology;

    fn record(name: &str, letters: &str) -> Sequence {
        normalize(
            Identifier::create(name.to_string()).unwrap(),
            letters.to_string(),
            SequenceTopology::Linear,
        )
        .unwrap()
    }

    /// The middles of a residue list, in the order the frame produced them.
    fn middles(residues: &[FrameResidue]) -> Vec<u64> {
        residues.iter().map(|residue| residue.middle).collect()
    }

    /// The symbols of a residue list, run together.
    fn symbols(residues: &[FrameResidue]) -> String {
        residues.iter().map(|residue| residue.symbol).collect()
    }

    #[test]
    fn reading_frame_indexes_one_two_and_three() {
        let frames: [ReadingFrame; 3] = ReadingFrame::ALL;
        let numbers: Vec<u64> = frames.iter().map(|frame| frame.index()).collect();
        assert!(
            numbers == vec![1u64, 2, 3],
            "ReadingFrame::ALL gives {:?}, expected frames 1, 2 and 3",
            numbers
        );
        for frame in frames.iter() {
            assert!(
                ReadingFrame::from_index(frame.index()) == Some(*frame),
                "from_index({}) is not {:?}",
                frame.index(),
                frame
            );
        }
        assert!(
            ReadingFrame::from_index(0).is_none(),
            "frame 0 is not a frame"
        );
        assert!(
            ReadingFrame::from_index(4).is_none(),
            "frame 4 is not a frame"
        );
        assert!(
            ReadingFrame::from_index(u64::MAX).is_none(),
            "no number above 3 names a frame"
        );
    }

    // The window selects by middle base, not first base and not span: base 0 of
    // this record is codon 1's first base, and no window covering only it keeps
    // that residue.
    #[test]
    fn the_window_holds_residues_by_their_middle_base() {
        // ATG GCT TGG ATT: 12 bases, plus frame 1 reads codons at 0, 3, 6, 9.
        let sequence: Sequence = record("win", "ATGGCTTGGATT");
        let all: Vec<FrameResidue> =
            frame_residues(&sequence, Strand::Plus, ReadingFrame::First, 0, 12).unwrap();
        assert!(
            middles(all.as_slice()) == vec![1u64, 4, 7, 10],
            "plus frame 1 middles are {:?}, expected [1, 4, 7, 10]",
            middles(all.as_slice())
        );

        let middle_two: Vec<FrameResidue> =
            frame_residues(&sequence, Strand::Plus, ReadingFrame::First, 4, 8).unwrap();
        assert!(
            middles(middle_two.as_slice()) == vec![4u64, 7],
            "window [4, 8) gives {:?}, expected middles 4 and 7",
            middles(middle_two.as_slice())
        );

        // Half-open at the top: middle 7 lies at `window_end` and is outside.
        let open: Vec<FrameResidue> =
            frame_residues(&sequence, Strand::Plus, ReadingFrame::First, 4, 7).unwrap();
        assert!(
            middles(open.as_slice()) == vec![4u64],
            "window [4, 7) gives {:?}, expected the one residue with middle 4",
            middles(open.as_slice())
        );

        // [0, 1) holds base 0, which is a codon's first base and never a
        // middle, so nothing is in it.
        let first_base: Vec<FrameResidue> =
            frame_residues(&sequence, Strand::Plus, ReadingFrame::First, 0, 1).unwrap();
        assert!(
            first_base.is_empty(),
            "window [0, 1) gives {:?}, expected nothing: base 0 is a codon's first base",
            middles(first_base.as_slice())
        );
    }

    // A record shorter than one codon has frames whose start is past the end
    // (plus frames 2 and 3) or before the start (minus frames 2 and 3). Those
    // produce nothing, never a panic and never a wrapped position (D050).
    #[test]
    fn a_record_shorter_than_a_codon_gives_no_residues_on_either_strand_and_no_frame() {
        for text in ["A", "AT"] {
            let sequence: Sequence = record("short", text);
            let end: u64 = sequence.length();
            for frame in ReadingFrame::ALL {
                for plus in [true, false] {
                    let strand: Strand = if plus { Strand::Plus } else { Strand::Minus };
                    let label: String = strand.clone().symbol();
                    let residues: Vec<FrameResidue> =
                        frame_residues(&sequence, strand, frame, 0, end).unwrap();
                    assert!(
                        residues.is_empty(),
                        "the {} base record {:?} gives {} residues on {} frame {}: expected none",
                        text.len(),
                        text,
                        residues.len(),
                        label,
                        frame.index()
                    );
                }
            }
        }
    }

    // A frame is not the mutant reading of `frame.rs`: `?` and `*` get their
    // residue and the frame goes on (D050, and `frame-tables.tsv` rows 3..6 of
    // `framesamb` plus frame 1).
    #[test]
    fn an_unreadable_codon_does_not_end_the_frame() {
        let ambiguous: Sequence = record("amb", "ATNCAT");
        let residues: Vec<FrameResidue> =
            frame_residues(&ambiguous, Strand::Plus, ReadingFrame::First, 0, 6).unwrap();
        assert!(
            residues.len() == 2,
            "ATN CAT gives {} residues, expected 2: `?` does not end a frame",
            residues.len()
        );
        assert!(
            residues[0].symbol == '?' && residues[0].number == 1 && residues[0].middle == 1,
            "the first residue is {:?}, expected number 1, `?` at middle 1",
            residues[0]
        );
        assert!(
            residues[1].symbol == 'H' && residues[1].number == 2 && residues[1].middle == 4,
            "the second residue is {:?}, expected number 2, `H` at middle 4",
            residues[1]
        );

        let stopped: Sequence = record("stop", "TAACAT");
        let again: Vec<FrameResidue> =
            frame_residues(&stopped, Strand::Plus, ReadingFrame::First, 0, 6).unwrap();
        assert!(
            symbols(again.as_slice()) == "*H",
            "TAA CAT gives {:?}, expected `*` then `H`",
            symbols(again.as_slice())
        );
        assert!(
            again.len() == 2 && again[0].number == 1 && again[1].number == 2,
            "a stop codon must be counted and the frame must continue: {:?}",
            again
        );
    }

    // Minus frame 1 starts at the record's last base, not at its first, and
    // reads each codon's complement at decreasing positions: so its first
    // residue sits at the far end of the record and its numbering runs down the
    // reference, which is D040's fact about reading order versus reference order
    // seen in a frame.
    #[test]
    fn a_minus_frame_counts_from_the_far_end() {
        // ATG GGG: plus frame 1 reads ATG at 0 and GGG at 3, so Met then Gly at
        // middles 1 and 4. Minus frame 1 starts at position 5 and reads the
        // complements of 5, 4 and 3 (CCC = Pro), then of 2, 1 and 0
        // (CAT = His) at middles 4 and 1.
        let sequence: Sequence = record("two", "ATGGGG");
        let plus: Vec<FrameResidue> =
            frame_residues(&sequence, Strand::Plus, ReadingFrame::First, 0, 6).unwrap();
        let minus: Vec<FrameResidue> =
            frame_residues(&sequence, Strand::Minus, ReadingFrame::First, 0, 6).unwrap();
        assert!(
            symbols(plus.as_slice()) == "MG" && middles(plus.as_slice()) == vec![1u64, 4],
            "plus frame 1 gives {:?}, expected M at 1 then G at 4",
            plus
        );
        assert!(
            symbols(minus.as_slice()) == "PH" && middles(minus.as_slice()) == vec![4u64, 1],
            "minus frame 1 gives {:?}, expected P at 4 then H at 1",
            minus
        );
        assert!(
            plus[0].symbol != minus[0].symbol && plus[0].middle != minus[0].middle,
            "plus and minus frame 1 must disagree in symbol and position at their first codon: {:?} vs {:?}",
            plus[0],
            minus[0]
        );
        assert!(
            minus[0].number == 1 && minus[1].number == 2,
            "a minus frame numbers from its own far end: {:?}",
            minus
        );
    }
}
