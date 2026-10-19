//! GenBank feature locations: the text between a feature's key and its
//! qualifiers, read as luria intervals and a strand (D041).
//!
//! The parser reads no files and knows nothing about records, features or
//! qualifiers: it turns one location text into intervals and a strand, and
//! refuses every location form luria does not represent, each with its own
//! rule. 5c2 hands it the text it has read; 5c3 builds the intervals and the
//! `Cds` from its result, and keeps its own checks there, so this module does
//! not restate overlap or CDS length.
//!
//! The rules, in the order the contract fixes them: the first that applies
//! decides, and every refusal carries the text as given and the length it
//! was read against.
//!
//! 1. Every ASCII whitespace character — space, tab, carriage return,
//!    newline — is removed, so a location continued over several lines
//!    reads as one. The text is kept as given for the payload.
//! 2. Nothing left after step 1 is `Syntax`.
//! 3. A `:` is a reference into another record: `RemoteReference`.
//! 4. A `<` or a `>` marks an incomplete feature: `FuzzyEnd`.
//! 5. A `^` names a site between two bases: `BetweenBases`.
//! 6. `order(` joins parts with no stated connection: `Order`.
//! 7. The text is parsed against the grammar below; anything it does not
//!    match exactly, with nothing left over, is `Syntax`; so is a number
//!    that does not fit in `u64`. Keywords are lower case and matched
//!    exactly: nothing is repaired.
//! 8. Every part, left to right, and within a part in this order: a bare
//!    position is `SinglePosition`; a position of 0 is `PositionZero`; a
//!    range whose end precedes its start is `RangeEndsBeforeStart`; an end
//!    past the length it was read against is `PastSequenceEnd`. `a..a` is a
//!    one-base range and is not refused.
//! 9. The shape. A single part gives one interval; a join is read only as an
//!    origin join, exactly two ranges `P1 = a..n` and `P2 = 1..b` where `n`
//!    is the sequence length, written plain, with `complement(…)` around the
//!    whole join, or with `complement(…)` around each part with `P2` first.
//!    Such a join on a linear template is `OriginJoinOnLinear`; every other
//!    join is `UnsupportedJoin` (D009: compound and spliced features are
//!    refused, never inferred).
//!
//! The grammar of step 7:
//!
//! ```text
//! location := part
//!           | "complement(" part ")"
//!           | "join(" item ("," item)* ")"
//!           | "complement(join(" part ("," part)* "))"
//! item     := part | "complement(" part ")"
//! part     := INT | INT ".." INT
//! INT      := one or more ASCII digits, fitting in u64
//! ```
//!
//! GenBank positions are one-based and inclusive, luria coordinates are zero-
//! based and half-open (D006): the range `a..b` becomes `[a - 1, b)`, and an
//! origin join becomes `[a - 1, n)` followed by `[0, b)`, in that order for
//! either strand (D036). Strand is taken from the text, never inferred
//! (D009).

use crate::error::DomainError;
use crate::error::GenBankLocationRule;
use crate::error::UnusableGenBankLocationProblem;
use crate::sequence::SequenceTopology;
use crate::strand::Strand;

/// A GenBank location as luria intervals: zero-based half-open `(start, end)`
/// pairs in D036 order, and the strand.
#[derive(Clone, Debug, PartialEq)]
pub struct GenBankLocation {
    intervals: Vec<(u64, u64)>,
    strand: Strand,
}

impl GenBankLocation {
    /// The intervals in D036 order: one for a linear feature, or `[a - 1, n)`
    /// followed by `[0, b)` for a circular one. They are plain pairs, not
    /// `NucleotideInterval`s: the parser has no `Sequence` to reference, and
    /// 5c3 builds the intervals and the `Cds` from these.
    pub fn intervals(&self) -> Vec<(u64, u64)> {
        self.intervals.clone()
    }

    /// The strand as written, never inferred (D009).
    pub fn strand(&self) -> Strand {
        self.strand.clone()
    }
}

/// A refusal: the rule, the text as given, and the length it was read
/// against (§3).
fn refusal(
    rule: GenBankLocationRule,
    text: String,
    sequence_length: u64,
) -> Result<GenBankLocation, DomainError> {
    Err(DomainError::UnusableGenBankLocation(
        UnusableGenBankLocationProblem {
            location: text,
            rule,
            sequence_length,
            cds: None,
            line: None,
        },
    ))
}

/// A location token: the two lower-case keywords, the punctuation, and a
/// number that fits in `u64`. A character that cannot start a token is not a
/// token at all, and a number too large for `u64` fails here too; both are
/// `Syntax` (step 7).
#[derive(Clone, Debug, PartialEq)]
enum Token {
    Complement,
    Join,
    Open,
    Close,
    Comma,
    Range,
    Number(u64),
}

/// The location split into tokens. `None` when the text holds something that
/// is not a token: an upper-case keyword, a word luria does not know, a lone
/// `.`, or a number that does not fit in `u64`.
fn tokenize(text: String) -> Option<Vec<Token>> {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens: Vec<Token> = Vec::new();
    let mut at: usize = 0;

    while at < chars.len() {
        let ch: char = chars[at];
        if ch == '(' {
            tokens.push(Token::Open);
            at += 1;
        } else if ch == ')' {
            tokens.push(Token::Close);
            at += 1;
        } else if ch == ',' {
            tokens.push(Token::Comma);
            at += 1;
        } else if ch == '.' {
            // A range separator is exactly two dots: a lone one is not a
            // token, and neither is a third dot after a separator.
            if at + 1 < chars.len() && chars[at + 1] == '.' {
                tokens.push(Token::Range);
                at += 2;
            } else {
                return None;
            }
        } else if ch.is_ascii_digit() {
            let from: usize = at;
            while at < chars.len() && chars[at].is_ascii_digit() {
                at += 1;
            }
            let digits: String = chars[from..at].iter().collect();
            match digits.parse::<u64>() {
                Ok(value) => {
                    tokens.push(Token::Number(value));
                }
                Err(_) => {
                    // A number that does not fit in `u64` is `Syntax`.
                    return None;
                }
            }
        } else if ch.is_ascii_alphabetic() {
            let from: usize = at;
            while at < chars.len() && chars[at].is_ascii_alphabetic() {
                at += 1;
            }
            let word: String = chars[from..at].iter().collect();
            // Keywords are lower case and matched exactly: `Complement` is
            // refused, not repaired.
            if word == "complement" {
                tokens.push(Token::Complement);
            } else if word == "join" {
                tokens.push(Token::Join);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    Some(tokens)
}

/// The token at `at`, as a comparison.
fn is_at(tokens: &[Token], at: usize, want: Token) -> bool {
    match tokens.get(at) {
        Some(token) => *token == want,
        None => false,
    }
}

/// The value of the `Number` token at `at`, when there is one.
fn number_at(tokens: &[Token], at: usize) -> Option<u64> {
    match tokens.get(at) {
        Some(Token::Number(value)) => Some(*value),
        _ => None,
    }
}

/// A parsed `part` (step 7): a bare position, or a range `a..b`.
#[derive(Clone, Debug, PartialEq)]
enum Part {
    Bare(u64),
    Range(u64, u64),
}

/// One parsed `item` (step 7): a part, and whether it is written as
/// `complement(part)`.
#[derive(Clone, Debug, PartialEq)]
struct Item {
    complement: bool,
    part: Part,
}

/// A parsed `location` (step 7): whether it is a join, whether the whole
/// location is wrapped in `complement(…)`, and its items in written order.
#[derive(Clone, Debug, PartialEq)]
struct Shape {
    join: bool,
    outer: bool,
    items: Vec<Item>,
}

/// A range part that passed step 8: its two positions, and whether it is
/// written as `complement(part)`.
#[derive(Clone, Debug, PartialEq)]
struct Ranged {
    complement: bool,
    start: u64,
    stop: u64,
}

/// Parse one `part`: `INT` or `INT .. INT`. On success, the part and the
/// index of the first token not consumed.
fn parse_part(tokens: &[Token], at: usize) -> Option<(Part, usize)> {
    match number_at(tokens, at) {
        None => None,
        Some(start) => {
            if is_at(tokens, at + 1, Token::Range) {
                number_at(tokens, at + 2).map(|stop| (Part::Range(start, stop), at + 3))
            } else {
                Some((Part::Bare(start), at + 1))
            }
        }
    }
}

/// Parse one `item`: `part` or `complement(part)`.
fn parse_item(tokens: &[Token], at: usize) -> Option<(Item, usize)> {
    if is_at(tokens, at, Token::Complement) && is_at(tokens, at + 1, Token::Open) {
        match parse_part(tokens, at + 2) {
            Some((part, next)) => {
                if is_at(tokens, next, Token::Close) {
                    let item: Item = Item {
                        complement: true,
                        part,
                    };
                    Some((item, next + 1))
                } else {
                    None
                }
            }
            None => None,
        }
    } else {
        parse_part(tokens, at).map(|(part, next)| {
            (
                Item {
                    complement: false,
                    part,
                },
                next,
            )
        })
    }
}

/// Parse `join( item ("," item)* ")"`, which takes at least one item.
fn parse_join(tokens: &[Token], at: usize) -> Option<(Vec<Item>, usize)> {
    if !is_at(tokens, at, Token::Join) || !is_at(tokens, at + 1, Token::Open) {
        return None;
    }

    let mut items: Vec<Item> = Vec::new();
    let mut next: usize = at + 2;

    loop {
        match parse_item(tokens, next) {
            Some((item, after)) => {
                items.push(item);
                next = after;
            }
            None => {
                return None;
            }
        }
        if is_at(tokens, next, Token::Comma) {
            next += 1;
        } else {
            break;
        }
    }

    if is_at(tokens, next, Token::Close) {
        Some((items, next + 1))
    } else {
        None
    }
}

/// Parse one `location`: `part`, `complement(part)`, `join(item, …)` or
/// `complement(join(part, …))`. In the last form the parts are never
/// themselves complemented; in `join(…)` items may be.
fn parse_location_at(tokens: &[Token], at: usize) -> Option<(Shape, usize)> {
    if is_at(tokens, at, Token::Join) {
        parse_join(tokens, at).map(|(items, next)| {
            (
                Shape {
                    join: true,
                    outer: false,
                    items,
                },
                next,
            )
        })
    } else if is_at(tokens, at, Token::Complement) {
        if !is_at(tokens, at + 1, Token::Open) {
            return None;
        }
        if is_at(tokens, at + 2, Token::Join) {
            // `complement(join(part, …))`: the parts are plain, so a
            // complemented part here is outside the grammar.
            match parse_join(tokens, at + 2) {
                Some((items, next)) => {
                    let plain: bool = items.iter().all(|item| !item.complement);
                    if plain && is_at(tokens, next, Token::Close) {
                        let shape: Shape = Shape {
                            join: true,
                            outer: true,
                            items,
                        };
                        Some((shape, next + 1))
                    } else {
                        None
                    }
                }
                None => None,
            }
        } else {
            // `complement(part)`.
            match parse_part(tokens, at + 2) {
                Some((part, next)) => {
                    if is_at(tokens, next, Token::Close) {
                        let item: Item = Item {
                            complement: false,
                            part,
                        };
                        let shape: Shape = Shape {
                            join: false,
                            outer: true,
                            items: vec![item],
                        };
                        Some((shape, next + 1))
                    } else {
                        None
                    }
                }
                None => None,
            }
        }
    } else {
        parse_part(tokens, at).map(|(part, next)| {
            let item: Item = Item {
                complement: false,
                part,
            };
            (
                Shape {
                    join: false,
                    outer: false,
                    items: vec![item],
                },
                next,
            )
        })
    }
}

/// Step 7 in full: the whole text must be one `location`, with nothing left
/// over. Anything else — including a number that does not fit in `u64` — is
/// `Syntax`.
fn parse_shape(text: String) -> Option<Shape> {
    match tokenize(text) {
        None => None,
        Some(tokens) => match parse_location_at(&tokens, 0) {
            Some((shape, next)) => {
                if next == tokens.len() {
                    Some(shape)
                } else {
                    None
                }
            }
            None => None,
        },
    }
}

/// Read one GenBank feature location as luria intervals and a strand.
/// `sequence_length` is the length the location was read against, and
/// `topology` says whether an origin join is representable at all (D036,
/// D041). The grammar and the shape are the contract's §4.1, applied in
/// order: the first rule that applies decides.
pub fn parse_genbank_location(
    text: &str,
    sequence_length: u64,
    topology: SequenceTopology,
) -> Result<GenBankLocation, DomainError> {
    // Every refusal carries the text as given, before step 1.
    let given: String = text.to_string();

    // Step 1: a location continued over several lines reads as one.
    let compact: String = text
        .chars()
        .filter(|ch| *ch != ' ' && *ch != '\t' && *ch != '\r' && *ch != '\n')
        .collect();

    // Step 2: nothing to read.
    if compact.is_empty() {
        return refusal(GenBankLocationRule::Syntax, given, sequence_length);
    }

    // Steps 3-6: the markers of location forms luria does not read, checked
    // before the grammar so that the first rule that applies decides.
    if compact.contains(":") {
        return refusal(GenBankLocationRule::RemoteReference, given, sequence_length);
    }
    if compact.contains("<") || compact.contains(">") {
        return refusal(GenBankLocationRule::FuzzyEnd, given, sequence_length);
    }
    if compact.contains("^") {
        return refusal(GenBankLocationRule::BetweenBases, given, sequence_length);
    }
    if compact.contains("order(") {
        return refusal(GenBankLocationRule::Order, given, sequence_length);
    }

    // Step 7: the grammar.
    let shape: Shape = match parse_shape(compact) {
        Some(found) => found,
        None => {
            return refusal(GenBankLocationRule::Syntax, given, sequence_length);
        }
    };

    // Step 8: every part, left to right, and within a part in the order the
    // contract fixes.
    let mut ranges: Vec<Ranged> = Vec::new();
    for item in shape.items {
        match item.part {
            Part::Bare(_) => {
                return refusal(GenBankLocationRule::SinglePosition, given, sequence_length);
            }
            Part::Range(start, stop) => {
                if start == 0 || stop == 0 {
                    return refusal(GenBankLocationRule::PositionZero, given, sequence_length);
                }
                if stop < start {
                    return refusal(
                        GenBankLocationRule::RangeEndsBeforeStart,
                        given,
                        sequence_length,
                    );
                }
                if stop > sequence_length {
                    return refusal(GenBankLocationRule::PastSequenceEnd, given, sequence_length);
                }
                ranges.push(Ranged {
                    complement: item.complement,
                    start,
                    stop,
                });
            }
        }
    }

    // Step 9: the shape. A single part is one interval, on the strand its
    // wrapper states.
    if !shape.join {
        let strand: Strand = if shape.outer {
            Strand::Minus
        } else {
            Strand::Plus
        };
        let part: Ranged = ranges[0].clone();
        return Ok(GenBankLocation {
            intervals: vec![(part.start - 1, part.stop)],
            strand,
        });
    }

    // A join is accepted only as an origin join: exactly two ranges, `P1 =
    // a..n` and `P2 = 1..b` with `n` the sequence length, in one of the three
    // spellings the contract lists.
    let forward: bool = ranges.len() == 2
        && !ranges[0].complement
        && !ranges[1].complement
        && ranges[0].stop == sequence_length
        && ranges[1].start == 1;
    let reversed: bool = ranges.len() == 2
        && ranges[0].complement
        && ranges[1].complement
        && ranges[0].start == 1
        && ranges[1].stop == sequence_length;

    if !forward && !reversed {
        return refusal(GenBankLocationRule::UnsupportedJoin, given, sequence_length);
    }

    // The join is an origin join, but a linear sequence has no origin to
    // cross (D036).
    if topology == SequenceTopology::Linear {
        return refusal(
            GenBankLocationRule::OriginJoinOnLinear,
            given,
            sequence_length,
        );
    }

    // D036: `[a - 1, n)` followed by `[0, b)`, for either strand.
    let first: u64 = if forward {
        ranges[0].start
    } else {
        ranges[1].start
    };
    let last: u64 = if forward {
        ranges[1].stop
    } else {
        ranges[0].stop
    };
    let strand: Strand = if shape.outer || ranges[0].complement {
        Strand::Minus
    } else {
        Strand::Plus
    };

    Ok(GenBankLocation {
        intervals: vec![(first - 1, sequence_length), (0, last)],
        strand,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCategory;
    use crate::error::GenBankLocationRule;
    use crate::error::GenBankLocationRule::*;

    const N: u64 = 100;

    fn circular(text: &str) -> Result<GenBankLocation, DomainError> {
        parse_genbank_location(text, N, SequenceTopology::Circular)
    }

    fn linear(text: &str) -> Result<GenBankLocation, DomainError> {
        parse_genbank_location(text, N, SequenceTopology::Linear)
    }

    fn accepted(found: Result<GenBankLocation, DomainError>) -> (Vec<(u64, u64)>, Strand) {
        let location = found.unwrap_or_else(|error| panic!("refused: {}", error.message()));
        (location.intervals(), location.strand())
    }

    fn rule(found: Result<GenBankLocation, DomainError>) -> GenBankLocationRule {
        match found {
            Err(DomainError::UnusableGenBankLocation(problem)) => {
                assert_eq!(problem.sequence_length, N);
                problem.rule
            }
            other => panic!("expected UnusableGenBankLocation, got {:?}", other),
        }
    }

    #[test]
    fn a_range_is_one_based_inclusive_converted_to_half_open() {
        assert_eq!(accepted(linear("1..99")), (vec![(0, 99)], Strand::Plus));
        assert_eq!(accepted(linear("1..100")), (vec![(0, 100)], Strand::Plus));
        assert_eq!(accepted(linear("5..5")), (vec![(4, 5)], Strand::Plus));
        assert_eq!(accepted(circular("10..30")), (vec![(9, 30)], Strand::Plus));
    }

    #[test]
    fn a_complement_is_the_same_interval_on_the_minus_strand() {
        assert_eq!(
            accepted(linear("complement(10..30)")),
            (vec![(9, 30)], Strand::Minus)
        );
    }

    // join(90..100,1..12) on a 100-base circle is 90..100 then 1..12:
    // [89, 100) then [0, 12), D036's order, for both strands.
    #[test]
    fn each_origin_join_spelling_gives_the_d036_intervals() {
        let both = vec![(89, 100), (0, 12)];
        assert_eq!(
            accepted(circular("join(90..100,1..12)")),
            (both.clone(), Strand::Plus)
        );
        assert_eq!(
            accepted(circular("complement(join(90..100,1..12))")),
            (both.clone(), Strand::Minus)
        );
        assert_eq!(
            accepted(circular("join(complement(1..12),complement(90..100))")),
            (both.clone(), Strand::Minus)
        );
    }

    #[test]
    fn whitespace_from_continuation_lines_is_removed() {
        assert_eq!(
            accepted(circular("join(90..100,\n                     1..12)")),
            (vec![(89, 100), (0, 12)], Strand::Plus)
        );
        assert_eq!(
            accepted(linear(" complement( 10 .. 30 ) ")),
            (vec![(9, 30)], Strand::Minus)
        );
    }

    #[test]
    fn a_reference_into_another_record_is_refused() {
        assert_eq!(rule(linear("J00194.1:100..202")), RemoteReference);
        assert_eq!(
            rule(linear("join(1..10,J00194.1:100..202)")),
            RemoteReference
        );
    }

    #[test]
    fn a_fuzzy_end_is_refused() {
        assert_eq!(rule(linear("<1..99")), FuzzyEnd);
        assert_eq!(rule(linear("1..>99")), FuzzyEnd);
        assert_eq!(rule(linear("complement(<1..99)")), FuzzyEnd);
    }

    #[test]
    fn a_site_between_bases_is_refused() {
        assert_eq!(rule(linear("10^11")), BetweenBases);
    }

    #[test]
    fn order_is_refused() {
        assert_eq!(rule(linear("order(1..10,20..30)")), Order);
        assert_eq!(rule(linear("complement(order(1..10,20..30))")), Order);
    }

    #[test]
    fn a_single_position_is_refused() {
        assert_eq!(rule(linear("467")), SinglePosition);
        assert_eq!(rule(linear("complement(46)")), SinglePosition);
        assert_eq!(rule(circular("join(46,1..12)")), SinglePosition);
    }

    #[test]
    fn position_zero_is_refused() {
        assert_eq!(rule(linear("0..10")), PositionZero);
        assert_eq!(rule(linear("complement(0..10)")), PositionZero);
    }

    #[test]
    fn a_reversed_range_is_refused() {
        assert_eq!(rule(linear("30..10")), RangeEndsBeforeStart);
        assert_eq!(rule(circular("95..5")), RangeEndsBeforeStart);
    }

    #[test]
    fn a_range_past_the_sequence_end_is_refused() {
        assert_eq!(rule(linear("1..101")), PastSequenceEnd);
        assert_eq!(rule(circular("join(90..101,1..12)")), PastSequenceEnd);
    }

    #[test]
    fn an_origin_join_on_a_linear_sequence_is_refused() {
        assert_eq!(rule(linear("join(90..100,1..12)")), OriginJoinOnLinear);
        assert_eq!(
            rule(linear("complement(join(90..100,1..12))")),
            OriginJoinOnLinear
        );
        assert_eq!(
            rule(linear("join(complement(1..12),complement(90..100))")),
            OriginJoinOnLinear
        );
    }

    #[test]
    fn every_other_join_is_refused() {
        for text in [
            "join(1..10,20..30)",                          // exons
            "join(1..12,90..100)",                         // origin parts, wrong order
            "join(complement(90..100),complement(1..12))", // minus, wrong order
            "join(90..100,complement(1..12))",             // mixed strands
            "join(80..89,90..100,1..12)",                  // three parts
            "join(1..99)",                                 // one part
            "join(90..99,1..12)",                          // does not reach n
            "join(90..100,2..12)",                         // does not start at 1
        ] {
            assert_eq!(rule(circular(text)), UnsupportedJoin, "{}", text);
        }
        assert_eq!(rule(linear("join(1..10,20..30)")), UnsupportedJoin);
    }

    #[test]
    fn text_outside_the_grammar_is_a_syntax_error() {
        for text in [
            "",
            "   ",
            "1..",
            "..5",
            "1...5",
            "complement(1..5",
            "complement(complement(1..5))",
            "complement(join(complement(1..5),complement(90..100)))",
            "Complement(1..5)",
            "bond(1,5)",
            "gap(10)",
            "1..5x",
            "1..5,6..9",
            "99999999999999999999..100",
        ] {
            assert_eq!(rule(linear(text)), Syntax, "{:?}", text);
        }
    }

    // Steps 3-6 run before the grammar, and step 8 before the shape: the
    // first rule that applies decides.
    #[test]
    fn the_first_rule_that_applies_decides() {
        assert_eq!(rule(linear("J00194.1:<1..10")), RemoteReference);
        assert_eq!(rule(linear("order(<1..10,20..30)")), FuzzyEnd);
        assert_eq!(rule(linear("join(0..10,20..30)")), PositionZero);
        assert_eq!(rule(linear("join(1..101,1..12)")), PastSequenceEnd);
    }

    #[test]
    fn a_refusal_read_on_its_own_names_no_cds_and_no_line() {
        let refusals: Vec<Result<GenBankLocation, DomainError>> = vec![
            linear("0..10"),
            linear("<1..99"),
            linear("join(90..100,1..12)"),
        ];
        for found in refusals.iter() {
            let problem = match found {
                Err(DomainError::UnusableGenBankLocation(problem)) => problem,
                other => panic!("expected UnusableGenBankLocation, got {:?}", other),
            };
            assert_eq!(problem.cds, None);
            assert_eq!(problem.line, None);
        }
    }

    #[test]
    fn each_rule_has_its_category() {
        let malformed = DomainError::UnusableGenBankLocation(UnusableGenBankLocationProblem {
            location: "0..10".to_string(),
            rule: PositionZero,
            sequence_length: N,
            cds: None,
            line: None,
        });
        assert!(malformed.category() == ErrorCategory::MalformedInput);
        assert!(linear("1..101").unwrap_err().category() == ErrorCategory::BiologicalValidation);
        assert!(linear("<1..99").unwrap_err().category() == ErrorCategory::Unsupported);
        assert_eq!(
            linear("<1..99").unwrap_err().code(),
            "unusable_genbank_location"
        );
    }

    #[test]
    fn the_message_quotes_the_location_as_given() {
        assert_eq!(
            linear("join(90..100,\n1..12)").unwrap_err().message(),
            "GenBank location 'join(90..100,\n1..12)' joins across the origin, but the sequence is linear"
        );
        assert_eq!(
            linear("1..101").unwrap_err().message(),
            "GenBank location '1..101' reaches past the end of the 100-base sequence"
        );
    }
}
