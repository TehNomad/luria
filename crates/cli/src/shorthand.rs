//! `luria new`'s mutation shorthand (plan 7k §3, M7-35): one mutation per
//! line, in a form luria reads itself with no model. The line is read into
//! a request's own terms: zero-based half-open template coordinates (D006),
//! the bases as typed, upper case.
//!
//! What this parser does not judge: whether the bases match the template,
//! whether a position is in range, whether a codon translates to the
//! residue asked for. Those are 7k3's and `luria q5 design`'s, so
//! `1210 G>A` reads well here for any template.
//!
//! Residue lines reuse `crate::interpret::notation::parse_substitution`
//! (milestone 6) and re-implement nothing of it: the stop and `?` checks
//! come first, as the shorthand's own refusals, then the token without its
//! codon goes to 6's notation, and any refusal there is `Unreadable`.

use crate::interpret::notation::parse_substitution;
use luria_core::genetic_code::Residue;

/// What one line of shorthand asks for, in a request's own terms:
/// zero-based half-open template coordinates (D006).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading {
    /// `D404N`, `D404N:AAC`: `codon` is `None` for the form's codon choice.
    Residue {
        number: i64,
        from: char,
        to: char,
        codon: Option<String>,
    },
    /// `1210 G>A`, `1210-1212 GAT>AAC` (same length).
    Substitution {
        start: i64,
        end: i64,
        from: String,
        to: String,
    },
    /// `1210-1212 GAT>TT` (different length).
    Replacement {
        start: i64,
        end: i64,
        from: String,
        to: String,
    },
    /// `del 1210-1212`, `del 46`.
    Deletion { start: i64, end: i64 },
    /// `ins GAT after 1209`: `point` is the zero-based insertion point,
    /// equal to the 1-based base it follows.
    Insertion { point: i64, insert: String },
}

/// Why a line is not read; `code()` gives the fixture's code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    Empty,
    Unreadable,
    ZeroPosition,
    ReversedRange,
    LengthMismatch,
    NotDna,
    SameBases,
    BadCodon,
    Stop,
    Unstated,
}

/// The refusals, in the order `code()` lists them: for the message test.
#[cfg(test)]
const REFUSALS: [Refusal; 10] = [
    Refusal::Empty,
    Refusal::Unreadable,
    Refusal::ZeroPosition,
    Refusal::ReversedRange,
    Refusal::LengthMismatch,
    Refusal::NotDna,
    Refusal::SameBases,
    Refusal::BadCodon,
    Refusal::Stop,
    Refusal::Unstated,
];

impl Refusal {
    /// The snake-case code: `empty`, `unreadable`, `zero_position`,
    /// `reversed_range`, `length_mismatch`, `not_dna`, `same_bases`,
    /// `bad_codon`, `stop`, `unstated`.
    pub fn code(self) -> &'static str {
        match self {
            Refusal::Empty => "empty",
            Refusal::Unreadable => "unreadable",
            Refusal::ZeroPosition => "zero_position",
            Refusal::ReversedRange => "reversed_range",
            Refusal::LengthMismatch => "length_mismatch",
            Refusal::NotDna => "not_dna",
            Refusal::SameBases => "same_bases",
            Refusal::BadCodon => "bad_codon",
            Refusal::Stop => "stop",
            Refusal::Unstated => "unstated",
        }
    }

    /// One English sentence a user can act on, naming what to change;
    /// for example `Unstated`: "a part of the residue change is `?`;
    /// write the residue, number and new residue". ASCII or the screens'
    /// Unicode; no trailing full stop.
    pub fn message(self) -> &'static str {
        match self {
            Refusal::Empty => "the line holds nothing to read; write one mutation, such as `D404N` or `1210 G>A`",
            Refusal::Unreadable => "the line is not a mutation luria reads; write `D404N`, `1210 G>A`, `1210-1212 GAT>TT`, `del 1210-1212` or `ins GAT after 1209`",
            Refusal::ZeroPosition => "position 0 is not a template base; write a position from 1, and use `after 0` only to insert at the start",
            Refusal::ReversedRange => "the range runs backwards, its first position after its last; write the smaller position first",
            Refusal::LengthMismatch => "the bases written do not match the range's length; write one base for each base in the range",
            Refusal::NotDna => "the bases written are not DNA; write only A, C, G or T",
            Refusal::SameBases => "the new bases are the same as the bases already there; write bases that differ",
            Refusal::BadCodon => "the codon after `:` is not three DNA bases; write three of A, C, G or T, or write no codon",
            Refusal::Stop => "a stop is written as a residue; write the residue to put there, not `*`, `X` or `Ter`",
            Refusal::Unstated => "a part of the residue change is `?`; write the residue, number and new residue",
        }
    }
}

/// Read one line of shorthand.
pub fn parse_line(line: &str) -> Result<Reading, Refusal> {
    let text: String = normalise(line);
    if text.is_empty() {
        return Err(Refusal::Empty);
    }
    let lowered: String = text.to_lowercase();

    // A residue line is one token, so it is tried first, and never on a
    // word that starts `del` or `ins` (M7-35's own order).
    if let Some(parts) = residue_form(&text) {
        if !lowered.starts_with("del") && !lowered.starts_with("ins") {
            return read_residue(parts);
        }
    }

    let words: Vec<&str> = text.split(' ').collect();
    let lowered_words: Vec<String> = words.iter().map(|word| word.to_lowercase()).collect();

    // `del 1210-1212`, `del 46`.
    if lowered_words.len() == 2 && lowered_words[0] == "del" {
        let (a_text, b_text) = split_range(words[1]);
        let a: i64 = position(a_text, false)?;
        let b: i64 = position(b_text, false)?;
        if a > b {
            return Err(Refusal::ReversedRange);
        }
        return Ok(Reading::Deletion {
            start: zero_based(a),
            end: b,
        });
    }

    // `ins GAT after 1209`; `after 0` is the start.
    if lowered_words.len() == 4 && lowered_words[0] == "ins" && lowered_words[2] == "after" {
        let insert: &str = words[1];
        if !is_letters(insert) {
            return Err(Refusal::Unreadable);
        }
        if !is_dna(insert) {
            return Err(Refusal::NotDna);
        }
        let point: i64 = position(words[3], true)?;
        return Ok(Reading::Insertion {
            point,
            insert: insert.to_ascii_uppercase(),
        });
    }

    // `<pos or range> X>Y`, spaces around the `>` allowed. The space after
    // the position is needed, the two around `>` are not, so `>` is inside
    // a word, is a word's first character, is its last, or is a word of its
    // own; the longest `X` wins, as the fixture's search takes it. In every
    // form both sides of the `>` are non-empty.
    let (where_text, x, y) = match words.len() {
        2 => match words[1].rfind('>') {
            Some(at) if at > 0 && at + 1 < words[1].len() => {
                (words[0], &words[1][..at], &words[1][at + 1..])
            }
            _ => return Err(Refusal::Unreadable),
        },
        3 if words[2].len() > 1 && words[2].starts_with('>') => {
            (words[0], words[1], &words[2][1..])
        }
        3 if words[1].len() > 1 && words[1].ends_with('>') => {
            (words[0], &words[1][..words[1].len() - 1], words[2])
        }
        4 if words[2] == ">" => (words[0], words[1], words[3]),
        _ => return Err(Refusal::Unreadable),
    };

    let (a_text, b_text) = split_range(where_text);
    let a: i64 = position(a_text, false)?;
    let b: i64 = position(b_text, false)?;
    if a > b {
        return Err(Refusal::ReversedRange);
    }
    if !is_letters(x) || !is_letters(y) {
        return Err(Refusal::Unreadable);
    }
    if !is_dna(x) || !is_dna(y) {
        return Err(Refusal::NotDna);
    }
    let span: i64 = match b.checked_sub(a).and_then(|gap| gap.checked_add(1)) {
        Some(length) => length,
        None => return Err(Refusal::Unreadable),
    };
    if x.chars().count() != span as usize {
        return Err(Refusal::LengthMismatch);
    }
    if x.eq_ignore_ascii_case(y) {
        return Err(Refusal::SameBases);
    }
    let start: i64 = zero_based(a);
    if x.chars().count() == y.chars().count() {
        return Ok(Reading::Substitution {
            start,
            end: b,
            from: x.to_ascii_uppercase(),
            to: y.to_ascii_uppercase(),
        });
    }
    Ok(Reading::Replacement {
        start,
        end: b,
        from: x.to_ascii_uppercase(),
        to: y.to_ascii_uppercase(),
    })
}

/// Normalise a line: runs of whitespace to one space, leading and trailing
/// whitespace gone, and U+2013 `–`, the dash the screens write, to `-`.
fn normalise(line: &str) -> String {
    let words: Vec<&str> = line.split_whitespace().collect();
    words.join(" ").replace('\u{2013}', "-")
}

/// The words on a line, after normalisation: single spaces, so every word
/// is non-empty.
fn split_range(where_text: &str) -> (&str, &str) {
    match where_text.split_once('-') {
        Some((a_text, b_text)) => (a_text, b_text),
        None => (where_text, where_text),
    }
}

/// A decimal position as written: no sign, no leading zero, and a value
/// that fits `i64`. A position of 0 is refused, except where `allow_zero`
/// says `after 0` means the start.
fn position(text: &str, allow_zero: bool) -> Result<i64, Refusal> {
    if text.is_empty() || !text.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(Refusal::Unreadable);
    }
    if text.chars().count() > 1 && text.starts_with('0') {
        return Err(Refusal::ZeroPosition);
    }
    let value: i64 = match text.parse::<i64>() {
        Ok(found) => found,
        Err(_) => return Err(Refusal::Unreadable),
    };
    if value == 0 && !allow_zero {
        return Err(Refusal::ZeroPosition);
    }
    Ok(value)
}

/// The 1-based base `A` as the request's zero-based start, `[A-1, …)` (D006).
fn zero_based(one_based: i64) -> i64 {
    one_based
        .checked_sub(1)
        .expect("a position from 1 has a 1-less")
}

/// Text made only of letters, as the fixture's `isalpha` test is.
fn is_letters(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|ch| ch.is_alphabetic())
}

/// A non-empty run of DNA bases, A, C, G, T in either case.
fn is_dna(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|ch| matches!(ch, 'A' | 'C' | 'G' | 'T' | 'a' | 'c' | 'g' | 't'))
}

/// A residue named as a stop: `*`, `X` or `Ter`, in any case.
fn is_stop(part: &str) -> bool {
    let lowered: String = part.to_lowercase();
    lowered == "*" || lowered == "x" || lowered == "ter"
}

/// A residue as the one-letter code `Residue::symbol` writes it.
fn letter_of(residue: Residue) -> char {
    residue
        .symbol()
        .chars()
        .next()
        .expect("a residue symbol is one letter")
}

/// The residue line's parts, in `shorthand.py`'s order: the residue as
/// written, the number as written, the target as written, and the codon
/// after `:` when the line wrote one. `None` when the line is not in the
/// residue form.
fn read_residue(parts: (String, String, String, Option<String>)) -> Result<Reading, Refusal> {
    let (original, number, target, codon) = parts;

    // A stop first, as `shorthand.py` checks it.
    if is_stop(&original) || is_stop(&target) {
        return Err(Refusal::Stop);
    }
    // Then a part the user left unstated.
    if original == "?" || number == "?" || target == "?" {
        return Err(Refusal::Unstated);
    }

    // 6's notation reads the token without its codon, and knows the codes,
    // the number, and one notation only; nothing is judged twice.
    let token: String = format!("{}{}{}", original, number, target);
    let substitution = match parse_substitution(&token) {
        Ok(found) => found,
        Err(_) => return Err(Refusal::Unreadable),
    };

    // The codon, when the line wrote one, is three DNA bases, upper case.
    if let Some(written) = codon {
        if written.chars().count() != 3 || !is_dna(&written) {
            return Err(Refusal::BadCodon);
        }
        return Ok(Reading::Residue {
            number: substitution.number,
            from: letter_of(substitution.from),
            to: letter_of(substitution.to),
            codon: Some(written.to_ascii_uppercase()),
        });
    }
    Ok(Reading::Residue {
        number: substitution.number,
        from: letter_of(substitution.from),
        to: letter_of(substitution.to),
        codon: None,
    })
}

/// The residue form, matched on the whole line: an optional `p.`, a side of
/// 1 to 3 letters, `*` or `?`, digits or `?`, a side again, and an optional
/// `:codon` of non-spaces. Nothing in it holds a space, so a line with one
/// is not in this form. The search follows the fixture's order: the prefix
/// first, then the longest side, the longest number, and so on, backing
/// back when the rest does not read.
fn residue_form(text: &str) -> Option<(String, String, String, Option<String>)> {
    let chars: Vec<char> = text.chars().collect();
    for start in prefix_starts(&chars) {
        for (original, after_original) in letter_sides(&chars, start) {
            for (number, after_number) in digit_sides(&chars, after_original) {
                for (target, after_target) in letter_sides(&chars, after_number) {
                    if after_target == chars.len() {
                        return Some((original, number, target, None));
                    }
                    if chars[after_target] == ':' && after_target + 1 < chars.len() {
                        let codon: String = chars[after_target + 1..].iter().collect();
                        if codon.contains(' ') {
                            continue;
                        }
                        return Some((original, number, target, Some(codon)));
                    }
                }
            }
        }
    }
    None
}

/// Where a residue line's first side begins: past `p.` when the line opens
/// with exactly that prefix, which the fixture tries first, and otherwise
/// at the start.
fn prefix_starts(chars: &[char]) -> Vec<usize> {
    if chars.len() >= 2 && chars[0] == 'p' && chars[1] == '.' {
        vec![2, 0]
    } else {
        vec![0]
    }
}

/// A residue side at `at`, in the fixture's order: the longest run of
/// letters first (3, then 2, then 1), then `*`, then `?`.
fn letter_sides(chars: &[char], at: usize) -> Vec<(String, usize)> {
    let mut longest: usize = 0;
    while at + longest < chars.len() && chars[at + longest].is_ascii_alphabetic() {
        longest += 1;
    }
    let mut sides: Vec<(String, usize)> = Vec::new();
    for length in (1..=(longest.min(3))).rev() {
        sides.push((chars[at..at + length].iter().collect(), at + length));
    }
    if at < chars.len() && chars[at] == '*' {
        sides.push(("*".to_string(), at + 1));
    }
    if at < chars.len() && chars[at] == '?' {
        sides.push(("?".to_string(), at + 1));
    }
    sides
}

/// The number side at `at`, in the fixture's order: the longest run of
/// digits first, then shorter ones, then `?`.
fn digit_sides(chars: &[char], at: usize) -> Vec<(String, usize)> {
    let mut longest: usize = 0;
    while at + longest < chars.len() && chars[at + longest].is_ascii_digit() {
        longest += 1;
    }
    let mut sides: Vec<(String, usize)> = Vec::new();
    for length in (1..=longest).rev() {
        sides.push((chars[at..at + length].iter().collect(), at + length));
    }
    if at < chars.len() && chars[at] == '?' {
        sides.push(("?".to_string(), at + 1));
    }
    sides
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_refusal_has_a_message() {
        let mut messages: Vec<&str> = Vec::new();
        for refusal in REFUSALS {
            let text: &str = refusal.message();
            assert!(!text.is_empty(), "{} has no message", refusal.code());
            assert!(
                !text.ends_with('.'),
                "{} ends with a full stop",
                refusal.code()
            );
            for earlier in messages.iter() {
                assert_ne!(
                    *earlier,
                    text,
                    "{} and {} share a message",
                    refusal.code(),
                    earlier
                );
            }
            messages.push(text);
        }
        assert_eq!(messages.len(), REFUSALS.len());
    }

    #[test]
    fn a_huge_position_is_unreadable() {
        assert_eq!(
            parse_line("99999999999999999999 G>A"),
            Err(Refusal::Unreadable)
        );
    }
}
