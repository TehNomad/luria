//! The open reading frames of a record: both strands, one ORF per stop codon
//! (M8-40, contract 8m17).
//!
//! An ORF is the span from a start codon to the stop codon that ends it, and
//! the rules are the ones `tests/fixtures/view/orfs.tsv` pins:
//!
//! * A start codon is `ATG` (`lookup` answers `Residue(M)`); a stop is `TAA`,
//!   `TAG` or `TGA` (`lookup` answers `Stop`). Every other codon — an ambiguous
//!   one included, `TAR` or `NNN` — is **neither** a start nor a stop: it does
//!   not end an ORF and does not start one (D010, which `genetic_code::lookup`
//!   implements). `frame_tables.py` records the same disagreement for the frame
//!   tables: Biopython answers `*` for `TAR` and `X` for `NNN`, and there is no
//!   guess to make here.
//! * One ORF per stop codon. Walking back from a stop one codon at a time, the
//!   walk ends at the previous stop codon in that frame (that stop ends its own
//!   ORF, not this one), at the start of the reading on a linear record, or —
//!   on a circular record, where the walk wraps — once its span would exceed
//!   the record's length. The ORF starts at the **furthest-back** start codon
//!   the walk met; a stop with no start codon in reach has no ORF. Taking the
//!   nearest one instead would give a shorter, wrong ORF.
//! * The span includes the stop codon, as a CDS feature's span does, while
//!   `residues` counts the codons from the `ATG` up to the stop and does not
//!   count the stop itself (D010: "The terminal stop codon is not an amino-acid
//!   residue").
//! * `min_residues` is inclusive: an ORF with exactly that many is kept.
//! * `circular` is a parameter, never `sequence.topology()`: the view's `T` key
//!   flips a record's topology without touching its text, and this is where the
//!   two readings part company. On a circular record a codon may wrap the
//!   origin and an ORF may cross it, which gives two intervals (D036); on a
//!   linear record nothing wraps, and a reading that runs off either end
//!   without a stop is not an ORF at all.
//! * `intervals` is in D036 order: one interval, or for an ORF across the
//!   origin two, the one ending at the record's length first.
//!
//! The result is ordered as §5.5 gives it: by `intervals[0].0`, plus before
//! minus at equal starts, then by `intervals[0].1`.
//!
//! The scan is one pass per strand over the record's own coordinate line, which
//! for a circular record is the reading repeated three times: a wrapping codon
//! and an origin-crossing ORF then read as ordinary runs, and a codon whose
//! start lies in `length .. 2 * length` is one of the record's own codons, each
//! exactly once however often the text repeats. Per frame phase the scan keeps
//! only the start codons of the current run that are still within the span an
//! ORF ending at a later stop may have, so nothing walks back per stop and a
//! 64 MiB record stays linear (D056).

use std::collections::VecDeque;

use crate::genetic_code::lookup;
use crate::genetic_code::CodonOutcome;
use crate::genetic_code::Residue;
use crate::sequence::complement_letter;
use crate::sequence::Sequence;
use crate::strand::Strand;

/// M8-40's minimum: an ORF has at least this many residues, the stop not
/// counted. `find_orfs` takes its minimum as a parameter, so a caller asking for
/// another one does not change this constant.
pub const ORF_MIN_RESIDUES: u64 = 75;

/// One open reading frame: where it is, on which strand, and how long it is.
#[derive(Clone, Debug, PartialEq)]
pub struct Orf {
    pub strand: Strand,
    /// Zero-based, half-open, in D036 order: one interval, or two for an ORF
    /// across the origin (the one ending at the record's length first). The
    /// ORF's span includes its stop codon, as a CDS feature's does.
    pub intervals: Vec<(u64, u64)>,
    /// Codons from the ATG up to the stop, the ATG counted, the stop not.
    pub residues: u64,
}

/// The open reading frames of `sequence`, read on both strands and kept when
/// they hold at least `min_residues` residues.
///
/// `circular` says whether the reading wraps; it is never taken from
/// `sequence.topology()`, because the view can flip that (M8-40, D036).
pub fn find_orfs(sequence: &Sequence, circular: bool, min_residues: u64) -> Vec<Orf> {
    let length: usize = sequence.letter_slice().len();
    let mut found: Vec<Orf> = Vec::new();
    for strand in [Strand::Plus, Strand::Minus] {
        let reading: Vec<char> = reading_of(sequence, strand.clone());
        let wrapped: bool = strand == Strand::Minus;
        for (start, span, residues) in scan_reading(reading.as_slice(), circular, min_residues) {
            // `start` counts in the reading's own coordinates, so where it
            // stands in the record depends on the strand that read it.
            let reading_start: usize = if circular { start % length } else { start };
            let reference: usize = if wrapped {
                reference_of_minus(reading_start, span, length, circular)
            } else {
                reading_start
            };
            found.push(Orf {
                strand: strand.clone(),
                intervals: intervals_of(reference, span, length),
                residues,
            });
        }
    }
    found.sort_by_key(order_key);
    found
}

/// §5.5: the first interval's start, plus before minus at equal starts, then
/// that interval's end.
fn order_key(orf: &Orf) -> (u64, u8, u64) {
    let (start, end) = match orf.intervals.first() {
        Some(interval) => *interval,
        None => (0, 0),
    };
    (start, if orf.strand == Strand::Plus { 0 } else { 1 }, end)
}

/// The reading of one strand, in its reading direction: the record's own letters
/// for the plus strand, the reverse complement for the minus strand.
///
/// A letter without a complement cannot stand in a normalized sequence, because
/// `sequence::normalize` refuses it, so the fallback is `N`: an ambiguous codon,
/// which D010 answers as neither a start nor a stop, and never a guessed one.
fn reading_of(sequence: &Sequence, strand: Strand) -> Vec<char> {
    let letters: Vec<char> = sequence.letters();
    if strand.is_plus() {
        return letters;
    }
    let mut reading: Vec<char> = Vec::with_capacity(letters.len());
    let mut offset: usize = letters.len();
    while offset > 0 {
        offset -= 1;
        reading.push(complement_letter(letters[offset]).unwrap_or('N'));
    }
    reading
}

/// The codon whose first base stands at `position` of the coordinate line being
/// scanned, in reading order: the bases of `reading`, wrapping at the origin when
/// the record is circular.
fn codon_text(reading: &[char], position: usize, circular: bool) -> String {
    let length: usize = reading.len();
    let mut text: String = String::new();
    let mut step: usize = 0;
    while step < 3 {
        let offset: usize = position + step;
        text.push(if circular {
            reading[offset % length]
        } else {
            reading[offset]
        });
        step += 1;
    }
    text
}

/// Half-open reference intervals of a span starting at `start`, in D036 order:
/// origin-crossing means two intervals, the one ending at the record's length
/// first, and a span of at most the record's length never needs more than two.
fn intervals_of(start: usize, span: usize, length: usize) -> Vec<(u64, u64)> {
    if start + span <= length {
        return vec![(start as u64, (start + span) as u64)];
    }
    vec![
        (start as u64, length as u64),
        (0, (start + span - length) as u64),
    ]
}

/// Where a minus reading span of `span` bases, starting at reading base
/// `reading_start`, begins in reference coordinates.
///
/// Reading base `i` of the reverse complement stands at reference
/// `(length - 1 - i) % length`, so a minus span begins one base past the end of
/// it, which lands it on the other side of the reading coordinates — and is why
/// an origin-crossing minus ORF and an origin-crossing plus ORF of the same
/// record have different first intervals. A linear minus reading never wraps, so
/// its span ends at the reading and the arithmetic stays inside the record.
fn reference_of_minus(reading_start: usize, span: usize, length: usize, circular: bool) -> usize {
    if !circular {
        return length - reading_start - span;
    }
    (length as i64 - (reading_start + span) as i64).rem_euclid(length as i64) as usize
}

/// (reading-coordinate start, span in bases, residues) for every ORF of
/// `reading` held at `min_residues`, in the order the scan met them.
///
/// The coordinate line is `reading` itself for a linear record and `reading`
/// repeated three times for a circular one, and the scan runs forward through it
/// once. A codon whose start lies in `length .. 2 * length` is one of the
/// record's own codon starts, each exactly once: the copies of the text before
/// and after that window are what make the run a stop began at visible, and the
/// record's own stops are the only ones that give a row.
///
/// Per frame phase the scan keeps the start codons of the run it is in, oldest
/// first, with the ones too far back for any ORF ending here already dropped. A
/// stop takes the oldest one left, which is the furthest-back start, and closes
/// the run, so no stop ever walks back over the sequence.
fn scan_reading(reading: &[char], circular: bool, min_residues: u64) -> Vec<(usize, usize, u64)> {
    let length: usize = reading.len();
    let mut found: Vec<(usize, usize, u64)> = Vec::new();
    if length == 0 {
        return found;
    }
    let limit: usize = if circular { 3 * length } else { length };
    // One run per frame phase: a stop closes only the run of its own phase, and
    // a wrapping codon belongs to the phase its start lies in. The walk steps
    // one base at a time, so each of the three phases of the reading is walked:
    // a codon belongs to the phase its own start lies in (`position % 3`).
    let mut runs: [VecDeque<usize>; 3] = [VecDeque::new(), VecDeque::new(), VecDeque::new()];
    let mut position: usize = 0;
    while position + 3 <= limit {
        let phase: usize = position % 3;
        let outcome: CodonOutcome = lookup(codon_text(reading, position, circular));
        if outcome == CodonOutcome::Residue(Residue::M) {
            runs[phase].push_back(position);
        } else if outcome.is_stop() {
            // A circular span of more than the record is no ORF: the starts
            // that far back hold no ORF of their own, since a later stop's
            // window reaches further still, and drop out here instead of a walk
            // back from this stop.
            if circular {
                // `position + 3` can fall below `length`, and the window bound
                // then asks for nothing: a start is never negative.
                let reach: usize = position.saturating_add(3).saturating_sub(length);
                while let Some(&oldest) = runs[phase].front() {
                    if oldest >= reach {
                        break;
                    }
                    runs[phase].pop_front();
                }
            }
            // The coordinate line holds the record's stops at three positions
            // each; the middle one is the record's own line.
            let own: bool = !circular || (position >= length && position < 2 * length);
            if own {
                if let Some(&start) = runs[phase].front() {
                    let residues: u64 = ((position - start) / 3) as u64;
                    if residues >= min_residues {
                        found.push((start, position + 3 - start, residues));
                    }
                }
            }
            runs[phase].clear();
        }
        position += 1;
    }
    found
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

    /// The plus-strand ORFs of a reading, as `start..end:residues`, the span
    /// summed over the intervals so an origin-crossing one is still one row.
    fn plus_orfs(letters: &str, circular: bool, min_residues: u64) -> String {
        find_orfs(&record("scan", letters), circular, min_residues)
            .iter()
            .filter(|orf| orf.strand == Strand::Plus)
            .map(|orf| {
                let span: u64 = orf.intervals.iter().map(|(start, end)| end - start).sum();
                format!(
                    "{}..{}:{}",
                    orf.intervals[0].0,
                    orf.intervals[0].0 + span,
                    orf.residues
                )
            })
            .collect::<Vec<String>>()
            .join(",")
    }

    // ATG ATG TAA: one ORF, running back to the first ATG; the nearer ATG would
    // give a 1-residue ORF, which is the wrong answer §5.4 warns of. The 9
    // bases are the two ATGs and the stop: the span holds the stop, D010 does
    // not count it.
    #[test]
    fn the_run_takes_the_furthest_back_start() {
        assert!(
            plus_orfs("ATGATGTAA", false, 1) == "0..9:2",
            "{}",
            plus_orfs("ATGATGTAA", false, 1)
        );
    }

    // ATG ATG TAA TAA: the second stop's run holds no start codon at all,
    // because the first stop closed it.
    #[test]
    fn a_stop_closes_its_run_so_the_previous_stop_is_not_reused() {
        assert!(
            plus_orfs("ATGATGTAATAA", false, 1) == "0..9:2",
            "{}",
            plus_orfs("ATGATGTAATAA", false, 1)
        );
    }

    // The span holds the stop codon, `residues` does not count it.
    #[test]
    fn an_orf_spans_its_stop_codon_and_does_not_count_it() {
        assert!(plus_orfs("ATGTAA", false, 1) == "0..6:1");
        assert!(plus_orfs("ATGAAATAA", false, 1) == "0..9:2");
    }

    // A stop with no start in its run gives no ORF, and an ambiguous codon is
    // neither a start nor a stop (D010): `NTG` does not open a run and `TAR`
    // does not close one.
    #[test]
    fn an_ambiguous_codon_neither_opens_nor_closes_a_run() {
        assert!(plus_orfs("AAATAGCATTAA", false, 1).is_empty());
        assert!(plus_orfs("NTGAAATAA", false, 1).is_empty());
        // ATG AAA TAR AAA TAA: TAR is no stop, so the run that the TAA closes
        // reaches back to the ATG at 0 and holds four residues.
        assert!(plus_orfs("ATGAAATARAAATAA", false, 1) == "0..15:4");
    }

    // The minimum is inclusive.
    #[test]
    fn the_minimum_residue_count_is_inclusive() {
        assert!(plus_orfs("ATGAAATAA", false, 2) == "0..9:2");
        assert!(plus_orfs("ATGAAATAA", false, 3).is_empty());
    }

    // The minimum the caller is handed when none is named is NEAT Finder's
    // default, 75 residues (contract section 9).
    #[test]
    fn the_default_minimum_is_neaets_default() {
        assert_eq!(ORF_MIN_RESIDUES, 75, "NEAT Finder's default minimum");
    }

    // A linear reading that runs off the end without a stop gives no ORF, while
    // the circular one crosses the origin: TAA TAC ATG CGC, read from its own
    // origin, is ATG CGC TAA and 9 bases wide, 2 residues.
    #[test]
    fn an_orf_across_the_origin_is_two_intervals() {
        let letters: &str = "TAATACATGCGC";
        assert!(
            plus_orfs(letters, false, 1).is_empty(),
            "a linear record wraps nothing"
        );
        let circular: Vec<Orf> = find_orfs(&record("wrap", letters), true, 1);
        let plus: Vec<&Orf> = circular
            .iter()
            .filter(|orf| orf.strand == Strand::Plus)
            .collect();
        assert!(plus.len() == 1, "{:?}", circular);
        assert!(plus[0].intervals == vec![(6, 12), (0, 3)], "{:?}", plus[0]);
        assert!(plus[0].residues == 2, "{:?}", plus[0]);
    }

    // A span of more than the record is no ORF. On these short records the
    // bound is what keeps the scan's spans inside the record; the place it bites
    // for certain - a reading whose furthest-back start reaches over more than
    // the record's length - is `synthetic_linear` circular and `frames62`
    // circular, and those are the fixture cases `orf_fixtures.rs` compares.
    #[test]
    fn an_orf_may_not_span_more_than_the_record() {
        assert!(
            plus_orfs("ATGTAA", true, 1) == "0..6:1",
            "{}",
            plus_orfs("ATGTAA", true, 1)
        );
        // ATG ATG GCT TAA: the record's own ORF spans the whole 12 bases, which
        // the bound allows, and it is the only ORF the reading gives - the
        // copies either repeat it or reach past the record's own stops.
        assert!(
            plus_orfs("ATGATGGCTTAA", true, 1) == "0..12:3",
            "{}",
            plus_orfs("ATGATGGCTTAA", true, 1)
        );
        assert!(
            plus_orfs("ATGATGGCTTAA", true, 4).is_empty(),
            "{}",
            plus_orfs("ATGATGGCTTAA", true, 4)
        );
    }

    // Two ORFs on one reading: `ATG TAA ATG TAA` gives the 6-base ORFs at 0 and
    // 6, and the reverse complement of that reading holds no ATG at all.
    #[test]
    fn two_orfs_on_one_reading_come_out_in_the_order_the_table_lists_them() {
        let found: Vec<Orf> = find_orfs(&record("order", "ATGTAAATGTAA"), false, 1);
        assert!(found.len() == 2, "{found:?}");
        assert!(found[0].intervals == vec![(0, 6)], "{:?}", found[0]);
        assert!(found[1].intervals == vec![(6, 12)], "{:?}", found[1]);
    }

    // The order §5.5 gives: by the first interval's start, plus before minus at
    // equal starts, then by that interval's end.
    #[test]
    fn the_order_puts_plus_before_minus_at_the_same_start() {
        let minus: Orf = Orf {
            strand: Strand::Minus,
            intervals: vec![(0, 6)],
            residues: 1,
        };
        let wide_plus: Orf = Orf {
            strand: Strand::Plus,
            intervals: vec![(0, 8)],
            residues: 1,
        };
        let narrow_plus: Orf = Orf {
            strand: Strand::Plus,
            intervals: vec![(0, 6)],
            residues: 1,
        };
        let mut rows: Vec<Orf> = vec![minus.clone(), wide_plus.clone(), narrow_plus.clone()];
        rows.sort_by_key(order_key);
        let keys: Vec<(u64, u8, u64)> = rows.iter().map(order_key).collect();
        assert!(keys == vec![(0, 0, 6), (0, 0, 8), (0, 1, 6)], "{keys:?}");
    }
}
