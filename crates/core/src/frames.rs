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

    let letters: &[char] = sequence.letter_slice();
    let plus: bool = strand.is_plus();

    // The frame's offset: base `f` (1-based) on the plus strand, the last base
    // minus `f - 1` on the minus strand. Everything is worked out in `i128`,
    // because `window_start` and `window_end` are `u64` and a window may name
    // a position no frame codon stands at, even one above what an `i64` holds.
    let offset: i128 = frame.index() as i128 - 1;
    let length: i128 = letters.len() as i128;

    // A frame codon's middle base always stands inside the record: the first
    // codon of a plus frame spans `offset..offset + 3` and every later codon
    // sits further in, and a minus frame's codons lie below its first base. So
    // every middle lies in `0..length`, and clamping the window to the record
    // before solving for the first codon makes the arithmetic below
    // independent of how far outside the record the window's ends name.
    let window_start: i128 = (window_start as i128).min(length);
    let window_end: i128 = (window_end as i128).min(length);

    // Codons of the frame, counted from 0: on the plus strand codon `k` starts
    // at `offset + 3k` and reads up, and on the minus strand it starts at
    // `length - 1 - offset - 3k` and reads down. The middle base of codon `k`
    // is therefore `offset + 3k + 1` on the plus strand and
    // `length - offset - 2 - 3k` on the minus — increasing by 3 along a plus
    // frame, decreasing by 3 along a minus frame.
    //
    // `k` is the frame's own codon count from 0, so the residue number is
    // `k + 1`: the same number the walk from the frame's first codon gave the
    // codon (§4.3, D050). No codon outside the window is read, and no codon is
    // stepped over one at a time: the first codon the window can hold is found
    // by the division below, and the walk starts there.
    let first: i128 = if plus {
        // The first `k` with `offset + 3k + 1 >= window_start`, i.e.
        // `3k >= window_start - offset - 1`.
        ceil_div(window_start - offset - 1, 3)
    } else {
        // A minus frame reads down, so the walk skips the codons whose middle
        // lies at or above the window: the first `k` with
        // `length - offset - 2 - 3k < window_end`, i.e.
        // `3k >= length - offset - 1 - window_end`.
        ceil_div(length - offset - 1 - window_end, 3)
    };
    let mut k: i128 = if first < 0 { 0 } else { first };

    let mut out: Vec<FrameResidue> = Vec::new();
    loop {
        let middle: i128 = if plus {
            offset + 3 * k + 1
        } else {
            length - offset - 2 - 3 * k
        };
        // Half-open window, tested on the middle base (D006, §4.3). Both ends
        // are tested on both strands: a plus frame's middles rise and a minus
        // frame's fall, so the test that admits the first codon in the window
        // is answered by the other end at the first codon leaving it again.
        if middle < window_start || middle >= window_end {
            break;
        }
        // `middle` lies inside the record, so `position` is `middle` ± 1 and
        // cannot overflow the `i64` that `codon_at` takes.
        let position: i64 = if plus {
            (middle - 1) as i64
        } else {
            (middle + 1) as i64
        };
        // Reading ends where three whole bases do not stand: on a linear
        // coordinate line that is the end of the sequence, and the leftover 1
        // or 2 bases of a frame get no residue (D050, §4.3).
        let codon: String = match codon_at(letters, position, plus) {
            Some(codon) => codon,
            None => break,
        };
        out.push(FrameResidue {
            number: k as u64 + 1,
            symbol: codon_symbol(codon),
            middle: middle as u64,
        });
        k += 1;
    }

    Ok(out)
}

/// Smallest integer `n` with `divisor * n >= value` for a positive `divisor`
/// (here 3, the width of a codon): the least `k` a frame may start at.
///
/// `value / divisor` in Rust truncates towards 0, which is the ceiling only for
/// a negative `value` — `-4 / 3` is `-1`, and `ceil(-4/3)` is `-1` — so a
/// positive `value` with a remainder has one added.
fn ceil_div(value: i128, divisor: i128) -> i128 {
    debug_assert!(divisor > 0);
    let quotient: i128 = value / divisor;
    if value > 0 && quotient * divisor != value {
        quotient + 1
    } else {
        quotient
    }
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

    /// One residue as a short row: `number symbol@middle`.
    fn row(residue: &FrameResidue) -> String {
        format!("{}{}@{}", residue.number, residue.symbol, residue.middle)
    }

    fn rows(residues: &[FrameResidue]) -> Vec<String> {
        residues.iter().map(row).collect()
    }

    /// The walk `frame_residues` performed before contract 8r2: take the whole
    /// record, step codon by codon from the frame's first codon to the end of
    /// the sequence, and keep only the codons whose middle base lies in the
    /// window. Written out again here, in the test, because
    /// `a_window_gives_what_the_whole_frame_gives_inside_it` compares the
    /// reading under test against it, and that comparison must not depend on
    /// the code under test.
    fn whole_frame_filtered_by_window(
        sequence: &Sequence,
        strand: Strand,
        frame: ReadingFrame,
        window_start: u64,
        window_end: u64,
    ) -> Vec<FrameResidue> {
        let letters: Vec<char> = sequence.letters();
        let plus: bool = strand.is_plus();
        let length: i64 = letters.len() as i64;
        let offset: i64 = frame.index() as i64 - 1;
        let mut position: i64 = if plus { offset } else { length - 1 - offset };

        let mut out: Vec<FrameResidue> = Vec::new();
        let mut number: u64 = 0;
        while let Some(codon) = codon_at(letters.as_slice(), position, plus) {
            let middle: u64 = if plus {
                (position + 1) as u64
            } else {
                (position - 1) as u64
            };
            number += 1;
            if middle >= window_start && middle < window_end {
                out.push(FrameResidue {
                    number,
                    symbol: codon_symbol(codon),
                    middle,
                });
            }
            position += if plus { 3 } else { -3 };
        }
        out
    }

    /// §5 test 1: the windowed reading gives, residue for residue, what the
    /// whole-frame walk gives when its residues are filtered to the window. The
    /// comparison is with `whole_frame_filtered_by_window`, which is that walk
    /// written out again, so a window begun in the middle of a frame cannot be
    /// a codon off, cannot number itself from 1 inside the window, and cannot
    /// miss the leftover bases at either end.
    #[test]
    fn a_window_gives_what_the_whole_frame_gives_inside_it() {
        // The two fixture records that show how a base is left over: `frames61`
        // (61 bases, 61 = 3 * 20 + 1) and `frames62` (62 bases, 62 = 3 * 20 +
        // 2), whose texts agree up to the last two bases.
        const SIXTY_ONE: &str = "CAGACCATTAAAAGGTCCTTTGGGGGAATACGCCGAGCAGGTAATCATTAATTTAATGCGC";
        const SIXTY_TWO: &str = "CAGACCATTAAAAGGTCCTTTGGGGGAATACGCCGAGCAGGTAATCATTAATTTAATGCGCA";
        for text in [SIXTY_ONE, SIXTY_TWO] {
            let sequence: Sequence = record("window", text);
            let length: u64 = sequence.length();
            assert!(
                length == 61 || length == 62,
                "the test record is {} bases long, expected 61 or 62, as `frames61` and `frames62` are",
                length
            );
            for frame in ReadingFrame::ALL {
                for plus in [true, false] {
                    let strand: Strand = if plus { Strand::Plus } else { Strand::Minus };
                    let label: String = strand.clone().symbol();
                    // Every window inside the record, and past it: `length + 3`
                    // stands past the last codon any frame can complete, so a
                    // window wholly past the record is compared too.
                    let mut compared: u64 = 0;
                    for start in 0..=length + 3 {
                        for end in start..=length + 3 {
                            let windowed: Vec<FrameResidue> =
                                frame_residues(&sequence, strand.clone(), frame, start, end)
                                    .unwrap_or_else(|error| {
                                        panic!(
                                            "window [{}, {}) is refused: {:?}",
                                            start, end, error
                                        )
                                    });
                            let walked: Vec<FrameResidue> = whole_frame_filtered_by_window(
                                &sequence,
                                strand.clone(),
                                frame,
                                start,
                                end,
                            );
                            assert!(
                                windowed == walked,
                                "the {} base record on {} frame {} with window [{}, {}): the windowed reading gives {:?}, the whole-frame walk filtered to the window gives {:?}",
                                length,
                                label,
                                frame.index(),
                                start,
                                end,
                                rows(windowed.as_slice()),
                                rows(walked.as_slice()),
                            );
                            compared += 1;
                        }
                    }
                    // Every start and end pair in `0..=length + 3` was
                    // compared, so the loop bounds are the ones the test names.
                    let windows: u64 = (length + 4) * (length + 5) / 2;
                    assert!(
                        compared == windows,
                        "{} frame {} on the {} base record: {} windows compared, expected {} (every start and end in 0..={})",
                        frame.index(),
                        label,
                        length,
                        compared,
                        windows,
                        length + 3
                    );
                }
            }
        }
    }

    /// §5 test 3, `#[ignore]`d: the six frames' 60-base windows of a 10 Mb
    /// record are read in the window, not by walking the record. Timed on the
    /// release profile, and on its own (contract 8r2 §6):
    ///
    /// ```text
    /// cargo test -p luria-core --release -- --ignored a_window_of_a_long_record_is_quick
    /// ```
    #[test]
    #[ignore]
    fn a_window_of_a_long_record_is_quick() {
        let text: String = "ACGT".repeat(2_500_000);
        let sequence: Sequence = record("tenmeg", text.as_str());
        assert!(
            sequence.length() == 10_000_000,
            "the test record is {} bases, expected 10 000 000, the ceiling `MAX_RECORD_BASES` allows (D048, D056)",
            sequence.length()
        );
        // The record is built outside the timed part; only the six calls are
        // measured.
        let start: u64 = 9_000_000;
        let end: u64 = start + 60;
        let began: std::time::Instant = std::time::Instant::now();
        let mut residues: Vec<FrameResidue> = Vec::new();
        for frame in ReadingFrame::ALL {
            for plus in [true, false] {
                let strand: Strand = if plus { Strand::Plus } else { Strand::Minus };
                residues.extend(
                    frame_residues(&sequence, strand, frame, start, end).unwrap_or_else(|error| {
                        panic!(
                            "frame {} window [{}, {}) is refused: {:?}",
                            frame.index(),
                            start,
                            end,
                            error
                        )
                    }),
                );
            }
        }
        let elapsed: u128 = began.elapsed().as_micros();
        // A 60-base window holds 20 middles for every frame, so 120 residues
        // were read and the timing measures real work.
        assert!(
            residues.len() == 120,
            "the six frames give {} residues, expected 120: 20 middles in a 60-base window for each of six frames",
            residues.len()
        );
        assert!(
            elapsed < 5_000,
            "the six frames' 60-base windows of a 10 Mb record took {} us: a window is read by walking the window, and the six calls together must stay under 5 ms in --release",
            elapsed
        );
    }
}
