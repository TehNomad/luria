//! A GenBank feature location read for drawing only (D049, D053; contract
//! 8b1 §4.1, M8-1).
//!
//! The shared science parser refuses every partial location (D041), and that
//! stays: a partial CDS is never selectable for numbering. The view still has
//! to draw the feature, so this module reads a location for display: a `<` or
//! `>` on an **outer** end is read by taking the stated base as the bound and
//! recording which end is open. Every other location gets exactly the answer
//! `parse_genbank_location` gives, and every refusal here is one of its
//! existing rules — no flag on the shared parser, no new rule, no new variant.
//!
//! The steps, in order; the first step that answers decides (§4):
//!
//! 1. The text is compacted with the shared parser's own whitespace rule
//!    (`genbank_location.rs:426-430`). With no `<` and no `>` in it, the
//!    answer is the shared parser's answer for the text as given: a
//!    `DisplayLocation` with both `open_*` fields `None`, or its error.
//! 2. More than one `<`, more than one `>`, or a marker not followed at once
//!    by a digit is not a partial this parser can read: the shared parser's
//!    answer for the text as given stands, which is today's refusal.
//! 3. The markers are removed and the shared parser reads what is left, so
//!    the refusal named is the reason the location is unusable without the
//!    marker (D053): `order(<1..10,20..30)` is `order`, not `fuzzy_end`, and
//!    `join(1..>5,8..10)` is `unsupported_join`, while
//!    `parse_genbank_location` keeps saying `fuzzy_end` for both. Its
//!    `UnusableGenBankLocation` payload names the marker-stripped text, so
//!    `location` is set back to the text as given. Any other error is
//!    returned as it is.
//! 4. Outer ends only (D053). The digits after each marker are read, and a
//!    `<` run must equal the **first interval's start + 1** and a `>` run the
//!    **last interval's end**, both 1-based and both in D036 order, which is
//!    the same for either strand. A marker on an inner boundary — `1..<12`,
//!    or `join(55..>60,1..5)`, whose `>` sits on the origin boundary — cannot
//!    be drawn honestly as one bar with an open end, and keeps its
//!    `fuzzy_end` refusal. So does a run that does not parse as `u64`. The
//!    open end recorded is `value - 1`, a reference position (D006):
//!    `complement(<1..12)` gives `open_before == Some(0)`. The strand is
//!    never consulted here.
//!
//! Comparing against the parsed intervals rather than the first and last
//! numbers written in the text matters for `join(complement(1..b),
//! complement(a..n))`, whose written numbers are the origin boundary while
//! its D036 intervals are `[a - 1, n)` then `[0, b)`.

use crate::error::DomainError;
use crate::error::GenBankLocationRule;
use crate::error::UnusableGenBankLocationProblem;
use crate::genbank_location::parse_genbank_location;
use crate::genbank_location::GenBankLocation;
use crate::sequence::SequenceTopology;

/// A location as the view draws it.
#[derive(Clone, Debug, PartialEq)]
pub struct DisplayLocation {
    /// The shape, exactly as `parse_genbank_location` reads the unmarked text.
    pub location: GenBankLocation,
    /// `<`: the zero-based base (D006) the feature may begin before.
    pub open_before: Option<u64>,
    /// `>`: the zero-based base the feature may continue past.
    pub open_after: Option<u64>,
}

/// Read one GenBank feature location for drawing. `sequence_length` and
/// `topology` are the ones the record's own location was read against, and
/// this function answers every question about them that
/// `parse_genbank_location` answers.
pub fn parse_display_location(
    text: &str,
    sequence_length: u64,
    topology: SequenceTopology,
) -> Result<DisplayLocation, DomainError> {
    // Step 1: the shared parser's own whitespace rule, applied before any
    // marker is looked for, so a location continued over several lines reads
    // as one. With no marker in it this parser has nothing to say: the answer
    // is the shared parser's, refusal included.
    let compact: String = compact(text);
    if !compact.contains('<') && !compact.contains('>') {
        return closed(parse_genbank_location(text, sequence_length, topology));
    }

    // Step 2: two markers on one end, or a marker with no base written after
    // it, is not a partial this parser reads: today's refusal stands.
    let markers: Vec<(char, String)> = marker_runs(&compact);
    let doubled: bool = count(&compact, '<') > 1 || count(&compact, '>') > 1;
    let unplaced: bool = markers.iter().any(|(_, run)| run.is_empty());
    if doubled || unplaced {
        return closed(parse_genbank_location(text, sequence_length, topology));
    }

    // Step 3: the markers go, and the shared parser reads what is left, so a
    // refusal names the rule that stands without the marker (D053). Its
    // payload names the marker-stripped text, which is not the feature's
    // location text: put the text as given back.
    let unmarked: String = compact
        .chars()
        .filter(|ch| *ch != '<' && *ch != '>')
        .collect();
    let location: GenBankLocation =
        match parse_genbank_location(&unmarked, sequence_length, topology) {
            Ok(location) => location,
            Err(error) => return Err(names_given_text(error, text)),
        };

    // Step 4: outer ends only (D053). The stated base after a `<` is the first
    // interval's start, and after a `>` the last interval's end, both 1-based
    // and both in D036 order, which is the same for either strand. A marker
    // elsewhere is an inner boundary and is not drawn.
    let intervals: Vec<(u64, u64)> = location.intervals();
    let (first, last): ((u64, u64), (u64, u64)) = match (intervals.first(), intervals.last()) {
        (Some(first), Some(last)) => (*first, *last),
        _ => return Err(fuzzy_end(text, sequence_length)),
    };
    let mut open_before: Option<u64> = None;
    let mut open_after: Option<u64> = None;
    for (marker, run) in markers.iter() {
        let stated: u64 = match run.parse::<u64>() {
            Ok(stated) => stated,
            Err(_) => return Err(fuzzy_end(text, sequence_length)),
        };
        let bound: Option<u64> = if *marker == '<' {
            first.0.checked_add(1)
        } else {
            Some(last.1)
        };
        if bound != Some(stated) {
            return Err(fuzzy_end(text, sequence_length));
        }
        // The open end is a reference position (D006), whatever the strand:
        // `complement(<1..12)` opens before base 0 of the reference.
        let reference: u64 = match stated.checked_sub(1) {
            Some(reference) => reference,
            None => return Err(fuzzy_end(text, sequence_length)),
        };
        if *marker == '<' {
            open_before = Some(reference);
        } else {
            open_after = Some(reference);
        }
    }
    Ok(DisplayLocation {
        location,
        open_before,
        open_after,
    })
}

/// Space, tab, carriage return and newline removed, as step 1 of
/// `parse_genbank_location` does it (`genbank_location.rs:426-430`). No other
/// character is whitespace here either.
fn compact(text: &str) -> String {
    text.chars()
        .filter(|ch| *ch != ' ' && *ch != '\t' && *ch != '\r' && *ch != '\n')
        .collect()
}

/// Each `<` or `>` of the compacted text, in the order written, with the digits
/// written at once after it. An empty run means the marker is not followed by a
/// digit, which step 2 refuses and step 4 cannot read.
fn marker_runs(text: &str) -> Vec<(char, String)> {
    let letters: Vec<char> = text.chars().collect();
    let mut found: Vec<(char, String)> = Vec::new();
    for (position, letter) in letters.iter().enumerate() {
        if *letter != '<' && *letter != '>' {
            continue;
        }
        let mut run: String = String::new();
        for next in letters[position + 1..].iter() {
            if !next.is_ascii_digit() {
                break;
            }
            run.push(*next);
        }
        found.push((*letter, run));
    }
    found
}

/// How many times `marker` is written in `text`.
fn count(text: &str, marker: char) -> usize {
    text.chars().filter(|ch| *ch == marker).count()
}

/// The shared parser's answer for a text with no marker to draw: the same
/// shape, and neither end open.
fn closed(found: Result<GenBankLocation, DomainError>) -> Result<DisplayLocation, DomainError> {
    match found {
        Ok(location) => Ok(DisplayLocation {
            location,
            open_before: None,
            open_after: None,
        }),
        Err(error) => Err(error),
    }
}

/// A refusal from the shared parser, on text this parser stripped: its payload
/// names the stripped text, so put the location text as given back into it
/// (D053). Any other error is returned as it is.
fn names_given_text(error: DomainError, text: &str) -> DomainError {
    match error {
        DomainError::UnusableGenBankLocation(mut problem) => {
            problem.location = text.to_string();
            DomainError::UnusableGenBankLocation(problem)
        }
        other => other,
    }
}

/// The refusal for a marker this parser does not draw (D053): the shared
/// parser's own rule, carrying the text as given.
fn fuzzy_end(text: &str, sequence_length: u64) -> DomainError {
    DomainError::UnusableGenBankLocation(UnusableGenBankLocationProblem {
        location: text.to_string(),
        rule: GenBankLocationRule::FuzzyEnd,
        sequence_length,
        cds: None,
        line: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::GenBankLocationRule;
    use crate::error::UnusableGenBankLocationProblem;
    use crate::strand::Strand;

    const N: u64 = 60;

    fn circular(text: &str) -> Result<DisplayLocation, DomainError> {
        parse_display_location(text, N, SequenceTopology::Circular)
    }

    fn linear(text: &str) -> Result<DisplayLocation, DomainError> {
        parse_display_location(text, N, SequenceTopology::Linear)
    }

    /// What the view draws: intervals, strand, and which ends are open.
    type Drawn = (Vec<(u64, u64)>, Strand, Option<u64>, Option<u64>);

    /// The four values a test compares, taken out of an accepted answer.
    fn drawn(found: Result<DisplayLocation, DomainError>) -> Drawn {
        let found: DisplayLocation =
            found.unwrap_or_else(|error| panic!("refused: {}", error.message()));
        (
            found.location.intervals(),
            found.location.strand(),
            found.open_before,
            found.open_after,
        )
    }

    /// The payload of a refusal.
    fn problem(found: Result<DisplayLocation, DomainError>) -> UnusableGenBankLocationProblem {
        match found {
            Err(DomainError::UnusableGenBankLocation(problem)) => problem,
            other => panic!("expected UnusableGenBankLocation, got {:?}", other),
        }
    }

    // Step 1: a location with no marker is the shared parser's, answer and
    // refusal alike, and neither end is open.
    #[test]
    fn a_location_without_markers_is_the_shared_parsers_answer() {
        let cases: [(&str, SequenceTopology); 5] = [
            ("1..12", SequenceTopology::Linear),
            ("complement(1..12)", SequenceTopology::Linear),
            ("order(1..5,8..10)", SequenceTopology::Linear),
            ("J00001.1:1..5", SequenceTopology::Linear),
            ("join(55..60,1..5)", SequenceTopology::Circular),
        ];
        for (text, topology) in cases {
            let shared: Result<GenBankLocation, DomainError> =
                parse_genbank_location(text, N, topology.clone());
            let found: Result<DisplayLocation, DomainError> =
                parse_display_location(text, N, topology);
            match (&shared, &found) {
                (Ok(location), Ok(display)) => {
                    assert_eq!(
                        display.location, *location,
                        "{:?}: the shape differs from the shared parser's",
                        text
                    );
                    assert_eq!(display.open_before, None, "{:?}: open_before", text);
                    assert_eq!(display.open_after, None, "{:?}: open_after", text);
                }
                (Err(shared_problem), Err(found_problem)) => assert_eq!(
                    found_problem, shared_problem,
                    "{:?}: the refusal differs from the shared parser's",
                    text
                ),
                (Ok(_), Err(error)) => panic!(
                    "{:?}: refused {} while the shared parser accepts it",
                    text,
                    error.message()
                ),
                (Err(_), Ok(display)) => panic!(
                    "{:?}: accepted {} interval(s) while the shared parser refuses it",
                    text,
                    display.location.intervals().len()
                ),
            }
        }
    }

    #[test]
    fn a_left_partial_takes_the_stated_base() {
        assert_eq!(
            drawn(linear("<1..12")),
            (vec![(0, 12)], Strand::Plus, Some(0), None)
        );
    }

    #[test]
    fn a_right_partial_takes_the_stated_base() {
        assert_eq!(
            drawn(linear("1..>12")),
            (vec![(0, 12)], Strand::Plus, None, Some(11))
        );
    }

    #[test]
    fn both_ends_open() {
        assert_eq!(
            drawn(linear("<3..>12")),
            (vec![(2, 12)], Strand::Plus, Some(2), Some(11))
        );
    }

    // The open ends are reference positions: `complement` changes the strand
    // of the shape, not which end is open or where it lies (D006).
    #[test]
    fn a_complement_partial_keeps_reference_coordinates() {
        assert_eq!(
            drawn(linear("complement(<1..12)")),
            (vec![(0, 12)], Strand::Minus, Some(0), None)
        );
        assert_eq!(
            drawn(linear("complement(1..>12)")),
            (vec![(0, 12)], Strand::Minus, None, Some(11))
        );
    }

    // D036 order, either strand, in each of the three origin-join spellings.
    #[test]
    fn a_partial_origin_join_in_each_spelling() {
        let both: Vec<(u64, u64)> = vec![(54, 60), (0, 5)];
        let (intervals, strand, before, after) = drawn(circular("join(<55..60,1..>5)"));
        assert_eq!(intervals, both, "join(<55..60,1..>5)");
        assert_eq!(strand, Strand::Plus, "join(<55..60,1..>5) strand");
        assert_eq!(before, Some(54), "join(<55..60,1..>5) open_before");
        assert_eq!(after, Some(4), "join(<55..60,1..>5) open_after");

        let (intervals, strand, before, after) = drawn(circular("complement(join(<55..60,1..>5))"));
        assert_eq!(intervals, both, "complement(join(<55..60,1..>5))");
        assert_eq!(
            strand,
            Strand::Minus,
            "complement(join(<55..60,1..>5)) strand"
        );
        assert_eq!(
            before,
            Some(54),
            "complement(join(<55..60,1..>5)) open_before"
        );
        assert_eq!(after, Some(4), "complement(join(<55..60,1..>5)) open_after");

        let text: &str = "join(complement(1..>5),complement(<55..60))";
        let (intervals, strand, before, after) = drawn(circular(text));
        assert_eq!(intervals, both, "{:?}", text);
        assert_eq!(strand, Strand::Minus, "{:?} strand", text);
        assert_eq!(before, Some(54), "{:?} open_before", text);
        assert_eq!(after, Some(4), "{:?} open_after", text);
    }

    // Step 4: a marker on an inner end, or a marker this parser does not
    // read, keeps the shared parser's `fuzzy_end`, naming the text as given.
    #[test]
    fn a_marker_on_an_inner_end_keeps_fuzzy_end() {
        for text in ["1..<12", ">1..12", "<<1..12", "<1..>>12", "<x1..12"].iter() {
            let problem: UnusableGenBankLocationProblem = problem(linear(text));
            assert_eq!(problem.rule, GenBankLocationRule::FuzzyEnd, "{:?}", text);
            assert_eq!(
                problem.location, *text,
                "{:?} names {:?}",
                text, problem.location
            );
        }
        for text in [
            "join(55..>60,1..5)",
            "join(55..60,<1..5)",
            "join(complement(<1..5),complement(55..>60))",
        ]
        .iter()
        {
            let problem: UnusableGenBankLocationProblem = problem(circular(text));
            assert_eq!(problem.rule, GenBankLocationRule::FuzzyEnd, "{:?}", text);
            assert_eq!(
                problem.location, *text,
                "{:?} names {:?}",
                text, problem.location
            );
        }
    }

    // Step 3: once the markers are gone, another rule may apply first, and
    // that is the reason the feature list gives (D053).
    #[test]
    fn the_shared_parsers_other_refusals_still_apply() {
        let cases: [(&str, GenBankLocationRule); 5] = [
            ("order(<1..10,20..30)", GenBankLocationRule::Order),
            ("join(1..>5,8..10)", GenBankLocationRule::UnsupportedJoin),
            ("<1..61", GenBankLocationRule::PastSequenceEnd),
            ("J00001.1:<1..5", GenBankLocationRule::RemoteReference),
            (
                "join(<55..60,1..>5)",
                GenBankLocationRule::OriginJoinOnLinear,
            ),
        ];
        for (text, wanted) in cases.iter() {
            let problem: UnusableGenBankLocationProblem = problem(linear(text));
            assert_eq!(problem.rule, *wanted, "{:?} rule", text);
            assert_eq!(
                problem.location, *text,
                "{:?} names {:?}",
                text, problem.location
            );
        }
    }

    // The whitespace rule matters where a marker and its base are split: step
    // 2 reads "not followed at once by a digit" after step 1, not before it.
    #[test]
    fn whitespace_inside_a_partial_location_is_ignored() {
        assert_eq!(
            drawn(linear("complement( <1..12 )")),
            drawn(linear("complement(<1..12)"))
        );
        for text in ["complement(< 1..12)", "complement(<\n1..12)"].iter() {
            assert_eq!(
                drawn(linear(text)),
                drawn(linear("complement(<1..12)")),
                "{:?}",
                text
            );
        }
        assert_eq!(
            drawn(linear("1..> 12")),
            (vec![(0, 12)], Strand::Plus, None, Some(11))
        );
    }
}
