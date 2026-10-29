//! The editor's own reading of one GenBank feature location: 9b's
//! `EditLocation` from the location text (M9-17, M9-18; contract 9b2 §4).
//!
//! This is not the shared science parser (`genbank_location.rs`), whose rules
//! and refusals are what the science tier accepts; nothing here changes it.
//! The view, Q5 and CDS import keep refusing spliced joins and bare one-base
//! positions exactly as they do today (D009, D041). M9-17 and M9-18 say what
//! the *editor* reads, because a record whose feature it cannot read is a
//! record that cannot be edited at all (M9-12): pBR322's
//! `misc_difference 426` and a plain `join(10..20,30..40)` are both editable
//! text, and both are refused by that parser.
//!
//! Step 1 and the refusal rule names are the shared parser's, and so is their
//! order where the two overlap (`genbank_location.rs:1-60`). Every refusal is
//! `DomainError::UnusableGenBankLocation` with an existing rule and no new
//! variant: the text as given (before step 1), the rule, the length it was
//! read against, `cds: None` and `line: None`, since a location read here is
//! read on its own, outside a record import.
//!
//! Step 1, as the shared parser has it: space, tab, carriage return and
//! newline are removed, so a location continued over several lines reads as
//! one, and nothing else is whitespace. The rules, the first that applies
//! deciding:
//!
//! 1. Nothing left after step 1 is `Syntax`; a `:` is a reference into
//!    another record (`RemoteReference`); a `^` names a site between two
//!    bases (`BetweenBases`); `order(` joins parts with no stated connection
//!    (`Order`). These four are checked before the grammar, in this order, as
//!    the shared parser checks them. A `<` and a `>` are not among them:
//!    M9-17 reads them, on a join.
//! 2. The text must match the grammar below exactly, with nothing left over,
//!    or it is `Syntax`; so is a number that does not fit in `u64`. A mark on
//!    a bare position — `<426`, `>426` — is `FuzzyEnd`: M9-18 takes a bare
//!    position as the one base it names, and one base has no end to be open
//!    past.
//! 3. Every span, left to right, and within a span in this order: a position
//!    of 0 is `PositionZero`; an end before its start is
//!    `RangeEndsBeforeStart`; a position past `length` is `PastSequenceEnd`.
//!    A bare `N` is the one base `[N - 1, N)` (M9-18) and `a..b` is
//!    `[a - 1, b)` (D006).
//! 4. The strand, taken from the text and never inferred (D009):
//!    `complement(…)` around the whole location, or around every item of a
//!    `join`, is `Minus`; some items complemented and some not is
//!    `UnsupportedJoin`. When every item of a `join` is complemented the
//!    items are written 5′→3′, so they are reversed into coordinate order
//!    before rule 5 checks that order and rule 6 reads the marks. The ranges
//!    are stored in that order, which is D036 order for an origin join and
//!    low to high for every other join.
//! 5. The order. Each range must start after the start of the range before
//!    it; touching and overlapping ranges are kept, because ribosomal
//!    slippage writes them. One step back is an origin join: a range ending
//!    at `length` followed by one starting at 1, which is D036's two ranges
//!    in D036 order (D006, D036). On a linear record that is
//!    `OriginJoinOnLinear`; a second one, or any other step back, is
//!    `UnsupportedJoin`.
//! 6. The marks. A `<` stands only on the first range's start and a `>` only
//!    on the last range's end, counting in the order the ranges are stored —
//!    after rule 4's reversal, which is D036 order for an origin join. That
//!    is the feature's own 5′ and 3′ end either way, since a minus-strand
//!    join stores its high range first. Anything else is `FuzzyEnd`: a mark
//!    on an inner boundary, or a `>` on a start, or a `<` on an end. They set
//!    `open_start` and `open_end`, which belong to the feature's ends and not
//!    to the interval that carried them (README "Partial marks", Reading 1).
//!
//! The grammar, after step 1:
//!
//! ```text
//! location := span
//!           | "complement(" span ")"
//!           | "join(" item ("," item)* ")"
//!           | "complement(join(" span ("," span)* ")"
//! item     := span | "complement(" span ")"
//! span     := mark? INT [".." mark? INT]
//! mark     := "<" | ">"
//! INT      := one or more ASCII digits, fitting in u64
//! ```
//!
//! Keywords are lower case and matched exactly: nothing is repaired. Rule 2
//! refuses a mark on a bare `INT` as `FuzzyEnd`, and rule 6 refuses a mark
//! standing in the wrong place, whatever character it is.

use crate::edit_buffer::EditLocation;
use crate::error::DomainError;
use crate::error::GenBankLocationRule;
use crate::error::UnusableGenBankLocationProblem;
use crate::sequence::SequenceTopology;
use crate::strand::Strand;

/// `<`, the mark on the feature's 5′ end (rule 6).
const OPEN_BEFORE: char = '<';

/// `>`, the mark on the feature's 3′ end (rule 6).
const OPEN_AFTER: char = '>';

/// The refusal: the rule, the text as given, and the length it was read
/// against, naming no CDS and pointing at no line.
fn refusal(rule: GenBankLocationRule, text: String, length: u64) -> DomainError {
    DomainError::UnusableGenBankLocation(UnusableGenBankLocationProblem {
        location: text,
        rule,
        sequence_length: length,
        cds: None,
        line: None,
    })
}

/// A location token: the two lower-case keywords, the punctuation, a mark and
/// the character it is, and a number that fits in `u64`. A character that
/// cannot start a token is not a token at all, and a number too large for
/// `u64` fails here too; both are `Syntax` (rule 2).
#[derive(Clone, Debug, PartialEq)]
enum Token {
    Complement,
    Join,
    Open,
    Close,
    Comma,
    Range,
    Mark(char),
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
        } else if ch == '<' || ch == '>' {
            // The character is kept, because rule 6 reads `<` only on a start
            // and `>` only on an end.
            tokens.push(Token::Mark(ch));
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
                Ok(value) => tokens.push(Token::Number(value)),
                Err(_) => return None,
            }
        } else if ch.is_ascii_alphabetic() {
            let from: usize = at;
            while at < chars.len() && chars[at].is_ascii_alphabetic() {
                at += 1;
            }
            let word: String = chars[from..at].iter().collect();
            // Keywords are lower case and matched exactly: `Complement` is
            // refused, not repaired. `order` is no keyword here, because rule
            // 1 has already refused `order(` and anything else holding the
            // word is outside the grammar.
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

/// The `Mark` token at `at`, as the character that wrote it, and `None` when
/// there is no mark there.
fn mark_at(tokens: &[Token], at: usize) -> Option<char> {
    match tokens.get(at) {
        Some(Token::Mark(ch)) => Some(*ch),
        _ => None,
    }
}

/// A parsed `span`: one one-based position, or a range with its second
/// position, and which mark went before each number. A bare `INT` has no mark
/// after its first number, because the parser reads no mark before a second
/// number unless it read `..`.
#[derive(Clone, Debug, PartialEq)]
struct Span {
    low: u64,
    low_mark: Option<char>,
    high: Option<u64>,
    high_mark: Option<char>,
}

/// One parsed `item`: a span, and whether it is written as
/// `complement(span)`.
#[derive(Clone, Debug, PartialEq)]
struct Item {
    complement: bool,
    span: Span,
}

/// A parsed `location`: whether it is a join, whether the whole location is
/// wrapped in `complement(…)`, and its items in written order.
#[derive(Clone, Debug, PartialEq)]
struct Shape {
    join: bool,
    outer: bool,
    items: Vec<Item>,
}

/// A span that passed rule 3: its range in the editor's coordinates
/// (zero-based half-open, D006) and the two marks written on it.
#[derive(Clone, Debug, PartialEq)]
struct Piece {
    range: (u64, u64),
    low_mark: Option<char>,
    high_mark: Option<char>,
}

/// Parse one `span`: `mark? INT`, or `mark? INT ".." mark? INT`. On success,
/// the span and the index of the first token not consumed.
fn parse_span(tokens: &[Token], at: usize) -> Option<(Span, usize)> {
    let low_mark: Option<char> = mark_at(tokens, at);
    let first: usize = if low_mark.is_some() { at + 1 } else { at };
    let low: u64 = number_at(tokens, first)?;
    if !is_at(tokens, first + 1, Token::Range) {
        let span: Span = Span {
            low,
            low_mark,
            high: None,
            high_mark: None,
        };
        return Some((span, first + 1));
    }
    let high_mark: Option<char> = mark_at(tokens, first + 2);
    let second: usize = if high_mark.is_some() {
        first + 3
    } else {
        first + 2
    };
    let high: u64 = number_at(tokens, second)?;
    let span: Span = Span {
        low,
        low_mark,
        high: Some(high),
        high_mark,
    };
    Some((span, second + 1))
}

/// Parse one `item`: `span` or `complement(span)`.
fn parse_item(tokens: &[Token], at: usize) -> Option<(Item, usize)> {
    if is_at(tokens, at, Token::Complement) && is_at(tokens, at + 1, Token::Open) {
        match parse_span(tokens, at + 2) {
            Some((span, next)) => {
                if is_at(tokens, next, Token::Close) {
                    let item: Item = Item {
                        complement: true,
                        span,
                    };
                    Some((item, next + 1))
                } else {
                    None
                }
            }
            None => None,
        }
    } else {
        parse_span(tokens, at).map(|(span, next)| {
            (
                Item {
                    complement: false,
                    span,
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
            None => return None,
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

/// Parse one `location`: `span`, `complement(span)`, `join(item, …)` or
/// `complement(join(span, …))`. In the last form the spans are never
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
            // `complement(join(span, …))`: the spans are plain, so a
            // complemented span here is outside the grammar.
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
            // `complement(span)`.
            match parse_span(tokens, at + 2) {
                Some((span, next)) => {
                    if is_at(tokens, next, Token::Close) {
                        let item: Item = Item {
                            complement: false,
                            span,
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
        parse_span(tokens, at).map(|(span, next)| {
            let item: Item = Item {
                complement: false,
                span,
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

/// Rule 2's grammar in full: the whole text must be one `location`, with
/// nothing left over. Anything else — including a number that does not fit in
/// `u64` — is `Syntax`.
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

/// Rule 2: a mark stands on a bare position, which has no `..` and so no end
/// to be open past (`<426`, `>426`).
fn marks_a_bare_position(shape: &Shape) -> bool {
    for item in shape.items.iter() {
        let marked: bool = item.span.low_mark.is_some() || item.span.high_mark.is_some();
        if item.span.high.is_none() && marked {
            return true;
        }
    }
    false
}

/// Rule 3: one span as the editor holds it, refusing in the order the
/// contract fixes. A bare `N` is the one base `[N - 1, N)` and `a..b` is
/// `[a - 1, b)` (D006, M9-18).
fn piece_of(span: &Span, length: u64, given: &str) -> Result<Piece, DomainError> {
    // A bare position has no second number, so it is its own range end.
    let high: u64 = match span.high {
        Some(high) => high,
        None => span.low,
    };
    if span.low == 0 || high == 0 {
        return Err(refusal(
            GenBankLocationRule::PositionZero,
            given.to_string(),
            length,
        ));
    }
    if high < span.low {
        return Err(refusal(
            GenBankLocationRule::RangeEndsBeforeStart,
            given.to_string(),
            length,
        ));
    }
    if high > length {
        return Err(refusal(
            GenBankLocationRule::PastSequenceEnd,
            given.to_string(),
            length,
        ));
    }
    Ok(Piece {
        range: (span.low - 1, high),
        low_mark: span.low_mark,
        high_mark: span.high_mark,
    })
}

/// Rule 5: the ranges in the order they are stored, refusing a step back that
/// is not D036's origin split, and an origin split on a record with no origin
/// to cross (D006, D009, D036). `length` is the record's length.
fn check_order(
    pieces: &[Piece],
    length: u64,
    topology: &SequenceTopology,
    given: &str,
) -> Result<(), DomainError> {
    let mut wrapped: bool = false;
    for pair in pieces.windows(2) {
        let before: &Piece = &pair[0];
        let next: &Piece = &pair[1];
        // Touching and overlapping ranges are kept, as ribosomal slippage
        // writes them: only a range that starts at or before the start of the
        // range before it is a step back.
        if next.range.0 > before.range.0 {
            continue;
        }
        // The one step back that is a feature: the range ending at the last
        // base of the record, and the one starting at base 1 (D036).
        let at_origin: bool = before.range.1 == length && next.range.0 == 0;
        if !at_origin {
            return Err(refusal(
                GenBankLocationRule::UnsupportedJoin,
                given.to_string(),
                length,
            ));
        }
        if topology == &SequenceTopology::Linear {
            return Err(refusal(
                GenBankLocationRule::OriginJoinOnLinear,
                given.to_string(),
                length,
            ));
        }
        if wrapped {
            return Err(refusal(
                GenBankLocationRule::UnsupportedJoin,
                given.to_string(),
                length,
            ));
        }
        wrapped = true;
    }
    Ok(())
}

/// Rule 6: the marks, counting in the order the ranges are stored, which is
/// D036 order for an origin-crossing feature. A `<` stands on the first
/// range's start and a `>` on the last range's end, and those two marks
/// belong to the feature's 5′ and 3′ ends (README "Partial marks",
/// Reading 1); anywhere else, or the other mark in that place, is a partial
/// the editor cannot read.
fn check_marks(pieces: &[Piece], given: &str, length: u64) -> Result<(bool, bool), DomainError> {
    let last: usize = pieces.len() - 1;
    let mut open_start: bool = false;
    let mut open_end: bool = false;
    for (index, piece) in pieces.iter().enumerate() {
        let wrong_start: bool = match piece.low_mark {
            Some(OPEN_BEFORE) => index != 0,
            Some(_) => true,
            None => false,
        };
        let wrong_end: bool = match piece.high_mark {
            Some(OPEN_AFTER) => index != last,
            Some(_) => true,
            None => false,
        };
        if wrong_start || wrong_end {
            return Err(refusal(
                GenBankLocationRule::FuzzyEnd,
                given.to_string(),
                length,
            ));
        }
        if index == 0 {
            open_start = piece.low_mark.is_some();
        }
        if index == last {
            open_end = piece.high_mark.is_some();
        }
    }
    Ok((open_start, open_end))
}

/// Read one location text for editing (M9-17, M9-18). `length` and
/// `topology` are the record's.
pub fn read_edit_location(
    text: &str,
    length: u64,
    topology: SequenceTopology,
) -> Result<EditLocation, DomainError> {
    // Every refusal carries the text as given, before step 1.
    let given: String = text.to_string();

    // Step 1, the shared parser's: a location continued over several lines
    // reads as one.
    let compact: String = text
        .chars()
        .filter(|ch| *ch != ' ' && *ch != '\t' && *ch != '\r' && *ch != '\n')
        .collect();

    // Rule 1: the forms the editor never reads. These four are checked before
    // the grammar and in this order, as the shared parser checks them. A `<`
    // and a `>` are not among them: rule 6 says where they stand.
    if compact.is_empty() {
        return Err(refusal(GenBankLocationRule::Syntax, given, length));
    }
    if compact.contains(":") {
        return Err(refusal(GenBankLocationRule::RemoteReference, given, length));
    }
    if compact.contains("^") {
        return Err(refusal(GenBankLocationRule::BetweenBases, given, length));
    }
    if compact.contains("order(") {
        return Err(refusal(GenBankLocationRule::Order, given, length));
    }

    // Rule 2: the grammar, and a mark on a bare position.
    let shape: Shape = match parse_shape(compact) {
        Some(found) => found,
        None => return Err(refusal(GenBankLocationRule::Syntax, given, length)),
    };
    if marks_a_bare_position(&shape) {
        return Err(refusal(GenBankLocationRule::FuzzyEnd, given, length));
    }

    // Rule 3: every span, left to right as written. The order the ranges are
    // stored in is rule 4's to settle, and rule 3 outlives it.
    let mut written: Vec<Piece> = Vec::new();
    for item in shape.items.iter() {
        written.push(piece_of(&item.span, length, given.as_str())?);
    }

    // Rule 4: the strand as written, never inferred (D009). In a `join`,
    // every item must be complemented or none; `complement(join(…))` leaves
    // its items plain and states the strand for the whole location.
    let complemented: usize = shape.items.iter().filter(|item| item.complement).count();
    let mixed: bool = complemented > 0 && complemented < shape.items.len();
    if shape.join && mixed {
        return Err(refusal(GenBankLocationRule::UnsupportedJoin, given, length));
    }
    let minus: bool = shape.outer || complemented > 0;

    // Every item of a join complemented: they are written 5′→3′, so they go
    // back into coordinate order before rule 5 checks it and rule 6 reads the
    // marks.
    let reversed: bool = shape.join && complemented > 0 && complemented == shape.items.len();
    let pieces: Vec<Piece> = if reversed {
        written.into_iter().rev().collect()
    } else {
        written
    };

    // Rules 5 and 6, in the order the ranges are stored.
    check_order(&pieces, length, &topology, given.as_str())?;
    let (open_start, open_end): (bool, bool) = check_marks(&pieces, given.as_str(), length)?;

    let ranges: Vec<(u64, u64)> = pieces.iter().map(|piece| piece.range).collect();
    let strand: Strand = if minus { Strand::Minus } else { Strand::Plus };

    Ok(EditLocation {
        strand,
        ranges,
        open_start,
        open_end,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DomainError;
    use crate::error::GenBankLocationRule;
    use crate::error::UnusableGenBankLocationProblem;
    use crate::sequence::SequenceTopology;
    use crate::strand::Strand;

    /// What a row of §5's table expects: the location, or the rule that
    /// refused it.
    type Expect = Result<EditLocation, GenBankLocationRule>;

    struct Row {
        text: &'static str,
        length: u64,
        topology: SequenceTopology,
        expect: Expect,
    }

    /// The length §5's table gives, unless the row says otherwise.
    const DEFAULT: u64 = 100;

    fn linear(text: &'static str, expect: Expect) -> Row {
        Row {
            text,
            length: DEFAULT,
            topology: SequenceTopology::Linear,
            expect,
        }
    }

    fn linear_on(text: &'static str, length: u64, expect: Expect) -> Row {
        Row {
            text,
            length,
            topology: SequenceTopology::Linear,
            expect,
        }
    }

    fn circular(text: &'static str, length: u64, expect: Expect) -> Row {
        Row {
            text,
            length,
            topology: SequenceTopology::Circular,
            expect,
        }
    }

    fn location(strand: Strand, ranges: &[(u64, u64)], open_start: bool, open_end: bool) -> Expect {
        Ok(EditLocation {
            strand,
            ranges: ranges.to_vec(),
            open_start,
            open_end,
        })
    }

    /// The rule that refused, and no other answer.
    fn refused(found: Result<EditLocation, DomainError>) -> GenBankLocationRule {
        match found {
            Ok(location) => panic!("the location was accepted as {:?}", location),
            Err(error) => match error {
                DomainError::UnusableGenBankLocation(problem) => problem.rule,
                other => panic!(
                    "the refusal is {}, not unusable_genbank_location",
                    other.code()
                ),
            },
        }
    }

    /// §5's table, row by row: the `EditLocation`, or the refusal's rule.
    #[test]
    fn reads_every_table_row() {
        use GenBankLocationRule::*;
        let rows: Vec<Row> = vec![
            linear("30..40", location(Strand::Plus, &[(29, 40)], false, false)),
            linear(
                "complement(30..40)",
                location(Strand::Minus, &[(29, 40)], false, false),
            ),
            linear_on(
                "426",
                4_361,
                location(Strand::Plus, &[(425, 426)], false, false),
            ),
            linear(
                "complement(27)",
                location(Strand::Minus, &[(26, 27)], false, false),
            ),
            linear("<30..40", location(Strand::Plus, &[(29, 40)], true, false)),
            linear("30..>40", location(Strand::Plus, &[(29, 40)], false, true)),
            linear(
                "complement(<30..>40)",
                location(Strand::Minus, &[(29, 40)], true, true),
            ),
            linear(
                "join(10..20,30..40)",
                location(Strand::Plus, &[(9, 20), (29, 40)], false, false),
            ),
            linear(
                "complement(join(10..20,30..40))",
                location(Strand::Minus, &[(9, 20), (29, 40)], false, false),
            ),
            linear(
                "join(complement(30..40),complement(10..20))",
                location(Strand::Minus, &[(9, 20), (29, 40)], false, false),
            ),
            linear(
                "join(<10..20,30..>40)",
                location(Strand::Plus, &[(9, 20), (29, 40)], true, true),
            ),
            linear(
                "complement(join(<10..20,30..40))",
                location(Strand::Minus, &[(9, 20), (29, 40)], true, false),
            ),
            linear(
                "join(10..20,20..30)",
                location(Strand::Plus, &[(9, 20), (19, 30)], false, false),
            ),
            linear(
                "join(10..20)",
                location(Strand::Plus, &[(9, 20)], false, false),
            ),
            circular(
                "join(56..62,1..11)",
                62,
                location(Strand::Plus, &[(55, 62), (0, 11)], false, false),
            ),
            linear_on("join(56..62,1..11)", 62, Err(OriginJoinOnLinear)),
            circular(
                "join(complement(1..8),complement(53..62))",
                62,
                location(Strand::Minus, &[(52, 62), (0, 8)], false, false),
            ),
            linear("join(30..40,10..20)", Err(UnsupportedJoin)),
            linear("join(10..20,complement(30..40))", Err(UnsupportedJoin)),
            linear("join(10..>20,30..40)", Err(FuzzyEnd)),
            linear_on("<426", 4_361, Err(FuzzyEnd)),
            linear("order(10..20,30..40)", Err(Order)),
            linear("10^11", Err(BetweenBases)),
            linear("J00194.1:100..202", Err(RemoteReference)),
            linear("0..10", Err(PositionZero)),
            linear("40..30", Err(RangeEndsBeforeStart)),
            linear("30..101", Err(PastSequenceEnd)),
            linear("join(10..20,30..40", Err(Syntax)),
        ];

        for row in rows.iter() {
            let answer: Expect =
                match read_edit_location(row.text, row.length, row.topology.clone()) {
                    Ok(found) => Ok(found),
                    Err(error) => Err(refused(Err(error))),
                };
            assert_eq!(
                answer,
                row.expect,
                "{:?} on length {} {}",
                row.text,
                row.length,
                row.topology.name()
            );
        }
    }

    /// §5 test 2: the payload names the text as given, with the whitespace
    /// step 1 removed still in it, the length it was read against, and no CDS
    /// and no line, because it was read on its own.
    #[test]
    fn refusal_payload_is_the_text_as_given() {
        let text: &str = " 30..\n101";
        let problem: UnusableGenBankLocationProblem =
            match read_edit_location(text, DEFAULT, SequenceTopology::Linear) {
                Err(DomainError::UnusableGenBankLocation(problem)) => problem,
                Ok(location) => panic!("the location was accepted as {:?}", location),
                Err(error) => panic!(
                    "the refusal is {}, not unusable_genbank_location",
                    error.code()
                ),
            };
        assert_eq!(problem.location, text, "the text as given, not step 1's");
        assert_eq!(problem.rule, GenBankLocationRule::PastSequenceEnd);
        assert_eq!(problem.sequence_length, DEFAULT);
        assert_eq!(problem.cds, None);
        assert_eq!(problem.line, None);
    }

    /// A mark on a bare position is `FuzzyEnd`, and both marks are read the
    /// same way there.
    #[test]
    fn a_mark_on_a_bare_position_is_fuzzy_end() {
        for text in ["<426", ">426", "join(10..20,<30)"] {
            assert_eq!(
                refused(read_edit_location(text, 4_361, SequenceTopology::Linear)),
                GenBankLocationRule::FuzzyEnd,
                "{:?}",
                text
            );
        }
    }

    /// Rule 1 comes before the grammar: `order(` is `Order` and not
    /// `Syntax`, and a `^` or a `:` in a text nothing else can read is
    /// refused by its own rule.
    #[test]
    fn the_indicators_are_refused_before_the_grammar() {
        use GenBankLocationRule::*;
        let rows: Vec<(&str, GenBankLocationRule)> = vec![
            ("order(", Order),
            ("1..", Syntax),
            (" \t\n", Syntax),
            ("^", BetweenBases),
            (":", RemoteReference),
            ("1..2..3", Syntax),
            ("Complement(10..20)", Syntax),
            ("jop(10..20)", Syntax),
            ("10..20)", Syntax),
            ("99999999999999999999999", Syntax),
        ];
        for (text, want) in rows {
            assert_eq!(
                refused(read_edit_location(text, DEFAULT, SequenceTopology::Linear)),
                want,
                "{:?}",
                text
            );
        }
    }

    /// The rules keep their order: rule 3 out of rule 4, and rule 5 out of
    /// rule 6, whichever way round the location is wrong.
    #[test]
    fn the_first_rule_that_applies_decides() {
        use GenBankLocationRule::*;
        let rows: Vec<(&'static str, GenBankLocationRule)> = vec![
            // The out-of-range span is refused before the mixed join is.
            ("join(10..200,complement(30..40))", PastSequenceEnd),
            // A step back out of order is refused before the mark on it is.
            ("join(30..40,10..>20)", UnsupportedJoin),
            // A mark in the wrong place, on a location that is in order.
            ("join(10..<20,30..40)", FuzzyEnd),
            ("join(10..20,50..60,30..40)", UnsupportedJoin),
        ];
        for (text, want) in rows {
            assert_eq!(
                refused(read_edit_location(text, DEFAULT, SequenceTopology::Linear)),
                want,
                "{:?}",
                text
            );
        }
    }

    /// Whitespace from a continuation line is step 1's: a location written
    /// over several lines reads as one.
    #[test]
    fn whitespace_from_continuation_lines_is_removed() {
        let found: EditLocation = read_edit_location(
            "join(10..20,\r\n\t30..40)",
            DEFAULT,
            SequenceTopology::Linear,
        )
        .unwrap_or_else(|error| panic!("the location is read: {}", error.message()));
        assert_eq!(found.ranges, vec![(9, 20), (29, 40)]);
        assert_eq!(found.strand, Strand::Plus);
        assert!(!found.open_start);
        assert!(!found.open_end);
    }

    /// A second mark, or one standing in a place the contract does not allow,
    /// is refused; and the two open ends belong to the feature, not to the
    /// interval that carried them.
    #[test]
    fn marks_stand_where_the_contract_allows_them() {
        use GenBankLocationRule::*;
        let rows: Vec<(&'static str, GenBankLocationRule)> = vec![
            ("join(10..20,30..>40,50..60)", FuzzyEnd),
            ("join(>10..20,30..40)", FuzzyEnd),
            ("join(10..<20,30..40)", FuzzyEnd),
            ("join(10..20,50..60,30..40)", UnsupportedJoin),
        ];
        for (text, want) in rows {
            assert_eq!(
                refused(read_edit_location(text, DEFAULT, SequenceTopology::Linear)),
                want,
                "{:?}",
                text
            );
        }
        let found: EditLocation = read_edit_location(
            "join(<10..20,30..40,50..>60)",
            DEFAULT,
            SequenceTopology::Linear,
        )
        .unwrap_or_else(|error| panic!("the location is read: {}", error.message()));
        assert_eq!(found.ranges, vec![(9, 20), (29, 40), (49, 60)]);
        assert!(found.open_start);
        assert!(found.open_end);
    }
}
