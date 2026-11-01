//! Sequence-only quality warnings on a primer.
//!
//! Every threshold here is a published guideline, not a biological rule, so
//! breaking one is a **warning on a design that is still emitted** (owner,
//! 2026-10-14). The only hard thermodynamic requirement is D030's 55 °C floor.
//! `tests/fixtures/primer-quality/README.md` records the source of each
//! threshold, the one disagreement between sources, and why the length check
//! fires on designs that are nonetheless correct;
//! `tests/primer_quality_thresholds.rs` keeps the constants below equal to
//! that fixture (D018).
//!
//! There is no cap on insertion length, but any primer longer than
//! `MAX_ORDERABLE_LENGTH` is past the owner's practical limit for ordering a
//! standard oligo (D032, owner, 2026-10-15: the limit for ordering from IDT).
//! It is warned about and still emitted, like every other warning here.
//!
//! Hairpins, self-dimers and primer-dimers are **not** checked. They need
//! loop, bulge and dimer nearest-neighbour parameters that have not been
//! sourced, and inventing them is what `docs/decisions/` D014 forbids.
//!
//! Nothing here reads a template, an edit or a Tm: a warning is a property of
//! the primer text alone, so the same text always draws the same warnings.

/// Below this G+C percentage a primer is warned about.
pub const GC_LOW_PERCENT: f64 = 40.0;

/// Above this G+C percentage a primer is warned about.
pub const GC_HIGH_PERCENT: f64 = 60.0;

/// The longest acceptable run of one base.
pub const MAX_SINGLE_BASE_RUN: u64 = 3;

/// The most acceptable repeats of a two-base unit. Sources disagree on
/// whether four is acceptable; luria takes the permissive reading and the
/// fixture README records the stricter one.
pub const MAX_DINUCLEOTIDE_REPEATS: u64 = 4;

/// The longest acceptable run of G/C ending a primer. One is the intended GC
/// clamp that D030 extends toward; three or more invites primer-dimers.
pub const MAX_THREE_PRIME_GC_RUN: u64 = 2;

/// A primer shorter than this is warned about.
pub const LENGTH_LOW: u64 = 18;

/// A primer longer than this is warned about.
pub const LENGTH_HIGH: u64 = 30;

/// A primer longer than this is past the practical limit for ordering a
/// standard oligo (D032, owner, 2026-10-15: the owner's limit for what IDT
/// will take as a standard order). There is no cap on insertion length, so
/// this warns rather than refuses, and it is separate from `LENGTH_HIGH`: one
/// is a PCR guideline, the other a vendor limit.
pub const MAX_ORDERABLE_LENGTH: u64 = 100;

/// One thing worth telling the caller about a primer's sequence.
///
/// Each variant carries the measurement, not just the fact of the warning, so
/// a caller applying a stricter or looser threshold of its own can.
#[derive(Clone, Debug, PartialEq)]
pub enum PrimerWarning {
    /// G+C outside the 40–60 % band.
    GcContentOutsideBand { gc_percent: f64 },
    /// Four or more of one base in a row, which can cause primer slippage.
    SingleBaseRun { base: char, length: u64, start: u64 },
    /// Five or more repeats of a two-base unit, which can misprime.
    DinucleotideRepeat {
        unit: String,
        repeats: u64,
        start: u64,
    },
    /// Three or more G/C bases ending the primer.
    ThreePrimeGcRun { length: u64 },
    /// Length outside the 18–30 nt guideline. Under D030 a reverse primer may
    /// start at 15, and either primer may grow past 30 to reach the Tm floor,
    /// so this reports an unusual length rather than a defect.
    LengthOutsideGuideline { length: u64 },
    /// Longer than the 100 nt a standard oligo can be ordered at (D032). A
    /// long insertion can push a primer past it; the design is still emitted.
    AboveOrderingLimit { length: u64 },
}

impl PrimerWarning {
    /// Human-readable text. For reporting only, never for classification.
    pub fn describe(&self) -> String {
        match self {
            PrimerWarning::GcContentOutsideBand { gc_percent } => format!(
                "G+C is {:.1} %, outside the {:.0}–{:.0} % guideline",
                gc_percent, GC_LOW_PERCENT, GC_HIGH_PERCENT
            ),
            PrimerWarning::SingleBaseRun {
                base,
                length,
                start,
            } => format!(
                "a run of {} {} at position {}; more than {} can cause primer slippage",
                length, base, start, MAX_SINGLE_BASE_RUN
            ),
            PrimerWarning::DinucleotideRepeat {
                unit,
                repeats,
                start,
            } => format!(
                "{} repeated {} times at position {}; more than {} repeats can misprime",
                unit, repeats, start, MAX_DINUCLEOTIDE_REPEATS
            ),
            PrimerWarning::ThreePrimeGcRun { length } => format!(
                "the last {} bases are all G or C; more than {} invites primer-dimers",
                length, MAX_THREE_PRIME_GC_RUN
            ),
            PrimerWarning::LengthOutsideGuideline { length } => format!(
                "{} bases, outside the {}–{} nt guideline",
                length, LENGTH_LOW, LENGTH_HIGH
            ),
            PrimerWarning::AboveOrderingLimit { length } => format!(
                "{} bases, longer than the {} nt a standard oligo can be ordered at",
                length, MAX_ORDERABLE_LENGTH
            ),
        }
    }
}

/// Every warning a primer's sequence draws, in a fixed order: length, the
/// ordering limit, G+C, then the sequence features by position. The order is
/// part of the output, so two equal primers always report identically.
pub fn inspect(primer: &[char]) -> Vec<PrimerWarning> {
    let mut warnings: Vec<PrimerWarning> = Vec::new();
    if primer.is_empty() {
        return warnings;
    }

    let length: u64 = primer.len() as u64;
    if !(LENGTH_LOW..=LENGTH_HIGH).contains(&length) {
        warnings.push(PrimerWarning::LengthOutsideGuideline { length });
    }
    // A 120-nt primer therefore draws both: one warning is the PCR guideline,
    // the other the vendor limit, and they say different things.
    if length > MAX_ORDERABLE_LENGTH {
        warnings.push(PrimerWarning::AboveOrderingLimit { length });
    }

    let gc: usize = primer
        .iter()
        .filter(|base| **base == 'G' || **base == 'C')
        .count();
    let gc_percent: f64 = 100.0 * gc as f64 / primer.len() as f64;
    if gc_percent < GC_LOW_PERCENT || gc_percent > GC_HIGH_PERCENT {
        warnings.push(PrimerWarning::GcContentOutsideBand { gc_percent });
    }

    warnings.extend(single_base_runs(primer));
    warnings.extend(dinucleotide_repeats(primer));

    let mut tail: u64 = 0;
    while (tail as usize) < primer.len() {
        let base: char = primer[primer.len() - 1 - tail as usize];
        if base != 'G' && base != 'C' {
            break;
        }
        tail += 1;
    }
    if tail > MAX_THREE_PRIME_GC_RUN {
        warnings.push(PrimerWarning::ThreePrimeGcRun { length: tail });
    }

    warnings
}

/// Each maximal run of one base longer than the guideline, left to right.
fn single_base_runs(primer: &[char]) -> Vec<PrimerWarning> {
    let mut warnings: Vec<PrimerWarning> = Vec::new();
    let mut start: usize = 0;
    while start < primer.len() {
        let mut end: usize = start + 1;
        while end < primer.len() && primer[end] == primer[start] {
            end += 1;
        }
        let run: u64 = (end - start) as u64;
        if run > MAX_SINGLE_BASE_RUN {
            warnings.push(PrimerWarning::SingleBaseRun {
                base: primer[start],
                length: run,
                start: start as u64,
            });
        }
        start = end;
    }
    warnings
}

/// Each maximal run of a repeated two-base unit longer than the guideline.
///
/// A unit of two equal bases is skipped: `AAAA` is a single-base run, already
/// reported, and counting it twice would say the same thing twice. Runs are
/// maximal and non-overlapping, scanning left to right.
fn dinucleotide_repeats(primer: &[char]) -> Vec<PrimerWarning> {
    let mut warnings: Vec<PrimerWarning> = Vec::new();
    let mut start: usize = 0;
    while start + 1 < primer.len() {
        let (first, second) = (primer[start], primer[start + 1]);
        if first == second {
            start += 1;
            continue;
        }
        // The next unit occupies `start + 2 * repeats` and the index after it,
        // so both must be inside the primer before they are compared.
        let mut repeats: u64 = 1;
        loop {
            let next: usize = start + 2 * repeats as usize;
            if next + 1 >= primer.len() || primer[next] != first || primer[next + 1] != second {
                break;
            }
            repeats += 1;
        }
        if repeats > MAX_DINUCLEOTIDE_REPEATS {
            warnings.push(PrimerWarning::DinucleotideRepeat {
                unit: format!("{}{}", first, second),
                repeats,
                start: start as u64,
            });
            start += 2 * repeats as usize;
        } else {
            start += 1;
        }
    }
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;

    fn letters(text: &str) -> Vec<char> {
        text.chars().collect()
    }

    fn warnings(text: &str) -> Vec<PrimerWarning> {
        inspect(&letters(text))
    }

    /// A primer text of `length` bases: `ATCG` repeated and truncated, so a
    /// long primer stays inside every other guideline here.
    fn repeated(length: usize) -> String {
        let text: String = "ATCG".repeat(length / 4 + 1);
        text[..length].to_string()
    }

    // 22 bases, 50 % G+C, no run above 3, no long dinucleotide repeat, one C
    // at the 3′ end: the intended GC clamp.
    const CLEAN: &str = "ATCGATTGCAGTACCTGAATGC";

    #[test]
    fn a_primer_inside_every_guideline_draws_no_warning() {
        assert!(CLEAN.len() == 22);
        assert!(warnings(CLEAN).is_empty(), "{:?}", warnings(CLEAN));
    }

    #[test]
    fn gc_content_is_warned_about_outside_the_band() {
        // 20 bases, 4 G/C = 20 %.
        let low: Vec<PrimerWarning> = warnings("ATATTAATTAGCATTAATGC");
        assert!(low
            .iter()
            .any(|w| matches!(w, PrimerWarning::GcContentOutsideBand { gc_percent } if *gc_percent < 40.0)));

        // 20 bases, 16 G/C = 80 %.
        let high: Vec<PrimerWarning> = warnings("GCGCATGCGCGCATGCGCGC");
        assert!(high
            .iter()
            .any(|w| matches!(w, PrimerWarning::GcContentOutsideBand { gc_percent } if *gc_percent > 60.0)));

        // Exactly at each bound: 40 % and 60 % are inside the guideline.
        let at_low: Vec<PrimerWarning> = warnings("ATATATATATATGCGCGCGC");
        assert!(!at_low
            .iter()
            .any(|w| matches!(w, PrimerWarning::GcContentOutsideBand { .. })));
        let at_high: Vec<PrimerWarning> = warnings("ATATATATGCGCGCGCGCGC");
        assert!(!at_high
            .iter()
            .any(|w| matches!(w, PrimerWarning::GcContentOutsideBand { .. })));
    }

    #[test]
    fn a_run_of_one_base_is_warned_about_above_three() {
        // AAA is within the guideline.
        assert!(!warnings("ATCGAAAGTACCTGAATGCATC")
            .iter()
            .any(|w| matches!(w, PrimerWarning::SingleBaseRun { .. })));

        // AAAA is not, and the warning says where and how long.
        let found: Vec<PrimerWarning> = warnings("ATCGAAAAGTACCTGAATGCAT");
        assert!(found.contains(&PrimerWarning::SingleBaseRun {
            base: 'A',
            length: 4,
            start: 4
        }));
    }

    #[test]
    fn every_long_run_is_reported_in_order() {
        let found: Vec<PrimerWarning> = warnings("TTTTAGCATTCAGGGGGATCAT");
        let runs: Vec<&PrimerWarning> = found
            .iter()
            .filter(|w| matches!(w, PrimerWarning::SingleBaseRun { .. }))
            .collect();
        assert!(runs.len() == 2, "{:?}", runs);
        assert!(
            *runs[0]
                == PrimerWarning::SingleBaseRun {
                    base: 'T',
                    length: 4,
                    start: 0
                }
        );
        assert!(
            *runs[1]
                == PrimerWarning::SingleBaseRun {
                    base: 'G',
                    length: 5,
                    start: 12
                }
        );
    }

    #[test]
    fn a_dinucleotide_repeat_is_warned_about_above_four_units() {
        // ATATATAT is four units: acceptable on the permissive reading luria
        // takes, which the fixture README records.
        assert!(!warnings("GCATATATATCCTGAATGCATC")
            .iter()
            .any(|w| matches!(w, PrimerWarning::DinucleotideRepeat { .. })));

        // Five units is not.
        let found: Vec<PrimerWarning> = warnings("GCATATATATATCCTGAATGCA");
        assert!(found.contains(&PrimerWarning::DinucleotideRepeat {
            unit: "AT".to_string(),
            repeats: 5,
            start: 2
        }));
    }

    #[test]
    fn a_run_of_one_base_is_not_also_reported_as_a_dinucleotide_repeat() {
        // AAAAAAAAAA is one base repeated, not an "AA" repeat: saying both
        // would report the same feature twice.
        let found: Vec<PrimerWarning> = warnings("GCAAAAAAAAAACCTGAATGCA");
        assert!(found
            .iter()
            .any(|w| matches!(w, PrimerWarning::SingleBaseRun { .. })));
        assert!(!found
            .iter()
            .any(|w| matches!(w, PrimerWarning::DinucleotideRepeat { .. })));
    }

    #[test]
    fn a_gc_run_at_the_three_prime_end_is_warned_about_above_two() {
        // One G is the GC clamp D030 extends toward: no warning.
        assert!(!warnings("ATCGATTGCAGTACCTGAATAG")
            .iter()
            .any(|w| matches!(w, PrimerWarning::ThreePrimeGcRun { .. })));

        // Two is still acceptable.
        assert!(!warnings("ATCGATTGCAGTACCTGAATGC")
            .iter()
            .any(|w| matches!(w, PrimerWarning::ThreePrimeGcRun { .. })));

        // Three is not. This is the ending D030's G/C extension can produce.
        let found: Vec<PrimerWarning> = warnings("ATCGATTGCAGTACCTGAAGGC");
        assert!(found.contains(&PrimerWarning::ThreePrimeGcRun { length: 3 }));
    }

    #[test]
    fn length_is_warned_about_outside_the_guideline() {
        // 15 bases: D030's shortest reverse primer, below the guideline's 18.
        let short: Vec<PrimerWarning> = warnings("ATCGATTGCAGTACC");
        assert!(short.contains(&PrimerWarning::LengthOutsideGuideline { length: 15 }));

        // 31 bases: the length an AT-rich site can need to reach 55 °C.
        let long_primer: String = "ATCGATTGCAGTACCTGAATGCATCGATTGC".to_string();
        assert!(long_primer.len() == 31);
        assert!(
            warnings(&long_primer).contains(&PrimerWarning::LengthOutsideGuideline { length: 31 })
        );

        // Both bounds are inside the guideline.
        for text in ["ATCGATTGCAGTACCTGA", "ATCGATTGCAGTACCTGAATGCATCGATTG"] {
            assert!(!warnings(text)
                .iter()
                .any(|w| matches!(w, PrimerWarning::LengthOutsideGuideline { .. })));
        }
    }

    // D032: the owner's practical limit for ordering a standard oligo (owner,
    // 2026-10-15). At the limit a primer is orderable, so it draws no warning
    // here; one base past it it draws one, and is still inspected.
    #[test]
    fn a_primer_at_the_ordering_limit_draws_no_ordering_warning() {
        let primer: String = repeated(100_usize);
        assert!(primer.len() == 100);
        assert!(
            !warnings(&primer)
                .iter()
                .any(|w| matches!(w, PrimerWarning::AboveOrderingLimit { .. })),
            "{:?}",
            warnings(&primer)
        );
    }

    #[test]
    fn a_primer_one_base_past_the_ordering_limit_draws_it() {
        let primer: String = repeated(101_usize);
        assert!(primer.len() == 101);
        assert!(
            warnings(&primer).contains(&PrimerWarning::AboveOrderingLimit { length: 101 }),
            "{:?}",
            warnings(&primer)
        );
    }

    #[test]
    fn the_order_of_the_warnings_is_fixed() {
        // 16 bases, 12.5 % G+C, a run of 5 A, six AT units, no G/C tail.
        let found: Vec<PrimerWarning> = warnings("AAAAAATATATATATA");
        let kinds: Vec<&str> = found
            .iter()
            .map(|warning| match warning {
                PrimerWarning::LengthOutsideGuideline { .. } => "length",
                PrimerWarning::GcContentOutsideBand { .. } => "gc",
                PrimerWarning::SingleBaseRun { .. } => "run",
                PrimerWarning::DinucleotideRepeat { .. } => "repeat",
                PrimerWarning::ThreePrimeGcRun { .. } => "tail",
                PrimerWarning::AboveOrderingLimit { .. } => {
                    unreachable!("16 bases, under the ordering limit")
                }
            })
            .collect();
        assert!(
            kinds == vec!["length", "gc", "run", "repeat"],
            "{:?}",
            kinds
        );
        assert!(found == warnings("AAAAAATATATATATA"));
    }

    #[test]
    fn an_empty_primer_draws_no_warning() {
        // A primer layer that can produce an empty primer has a defect of its
        // own; this layer does not divide by zero reporting it.
        assert!(inspect(&[]).is_empty());
    }

    // Each description carries the measurement, so a report is actionable
    // without the caller re-deriving it from the sequence.
    #[test]
    fn every_warning_describes_the_measurement_it_made() {
        let found: Vec<PrimerWarning> = warnings("AAAAAATATATATATA");
        assert!(found.len() == 4, "{:?}", found);
        for warning in found {
            let text: String = warning.describe();
            let expected: &str = match &warning {
                PrimerWarning::LengthOutsideGuideline { .. } => "16",
                PrimerWarning::GcContentOutsideBand { .. } => "0.0 %",
                PrimerWarning::SingleBaseRun { .. } => "6 A",
                PrimerWarning::DinucleotideRepeat { .. } => "AT repeated 5",
                PrimerWarning::ThreePrimeGcRun { .. } => unreachable!("no G/C tail here"),
                PrimerWarning::AboveOrderingLimit { .. } => {
                    unreachable!("16 bases, under the ordering limit")
                }
            };
            assert!(text.contains(expected), "{:?} -> {:?}", warning, text);
        }
    }
}
