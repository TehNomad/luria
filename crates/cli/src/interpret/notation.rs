//! The mutation notation of plan §5.3, §5.3a and M6-8: one token read as
//! a residue substitution, or a precise reason it is not one.
//!
//! The accepted forms, each optionally prefixed exactly `p.`:
//!
//! ```text
//! A42G        one-letter codes
//! Ala42Gly    three-letter codes
//! A?G         a part the user did not state
//! ```
//!
//! Any other token becomes `NotationError::Unsupported`, naming the text
//! exactly as given: a stop, a deletion, an insertion, a frameshift or a
//! DNA-level change is refused, never guessed (M6-8). Mixed notation
//! (`Ala42G`, `A42Gly`) is refused too, and so is a leading zero or a
//! number outside 1 to `i64::MAX`.
//!
//! A `?` in place of the original residue, the number or the target means
//! the user did not state that part (§5.3a); the parser reports which parts
//! in the order original, number, target and never gives a value for one.
//! A malformed token is `Unsupported` even when it also holds a `?`, so a
//! part is only ever reported unstated when the rest is well formed.
//!
//! Residue letters are normalized: any ASCII case is accepted, the result
//! is always `luria_core::genetic_code::Residue` (D002, D003) and
//! `Substitution.canonical` is the upper-case one-letter token.
//!
//! What this parser does not judge: whether the residue exists in a CDS,
//! whether `from` equals `to`, or whether the number is in range. Those are
//! core's, at `luria q5 design` (contract §2's last bullet), so `A42A` and
//! `A9999G` parse here.

use luria_core::genetic_code::residue_for_symbol;
use luria_core::genetic_code::Residue;

/// One of the twenty residues: its one-letter code, its three-letter code
/// and the English names a user may write for it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResidueName {
    pub letter: char,
    pub three: &'static str,
    pub names: &'static [&'static str],
}

/// The twenty residues of the standard genetic code, in the order of their
/// three-letter codes. Codes and names: IUPAC-IUB Joint Commission on
/// Biochemical Nomenclature, "Nomenclature and Symbolism for Amino Acids
/// and Peptides" (1983), 3AA-1. Checked against `luria-core` by test, so
/// the two cannot drift.
pub const RESIDUE_NAMES: [ResidueName; 20] = [
    ResidueName {
        letter: 'A',
        three: "Ala",
        names: &["alanine"],
    },
    ResidueName {
        letter: 'R',
        three: "Arg",
        names: &["arginine"],
    },
    ResidueName {
        letter: 'N',
        three: "Asn",
        names: &["asparagine"],
    },
    ResidueName {
        letter: 'D',
        three: "Asp",
        names: &["aspartate", "aspartic acid"],
    },
    ResidueName {
        letter: 'C',
        three: "Cys",
        names: &["cysteine"],
    },
    ResidueName {
        letter: 'Q',
        three: "Gln",
        names: &["glutamine"],
    },
    ResidueName {
        letter: 'E',
        three: "Glu",
        names: &["glutamate", "glutamic acid"],
    },
    ResidueName {
        letter: 'G',
        three: "Gly",
        names: &["glycine"],
    },
    ResidueName {
        letter: 'H',
        three: "His",
        names: &["histidine"],
    },
    ResidueName {
        letter: 'I',
        three: "Ile",
        names: &["isoleucine"],
    },
    ResidueName {
        letter: 'L',
        three: "Leu",
        names: &["leucine"],
    },
    ResidueName {
        letter: 'K',
        three: "Lys",
        names: &["lysine"],
    },
    ResidueName {
        letter: 'M',
        three: "Met",
        names: &["methionine"],
    },
    ResidueName {
        letter: 'F',
        three: "Phe",
        names: &["phenylalanine"],
    },
    ResidueName {
        letter: 'P',
        three: "Pro",
        names: &["proline"],
    },
    ResidueName {
        letter: 'S',
        three: "Ser",
        names: &["serine"],
    },
    ResidueName {
        letter: 'T',
        three: "Thr",
        names: &["threonine"],
    },
    ResidueName {
        letter: 'W',
        three: "Trp",
        names: &["tryptophan"],
    },
    ResidueName {
        letter: 'Y',
        three: "Tyr",
        names: &["tyrosine"],
    },
    ResidueName {
        letter: 'V',
        three: "Val",
        names: &["valine"],
    },
];

/// A residue substitution read from a token.
#[derive(Clone, Debug, PartialEq)]
pub struct Substitution {
    pub from: Residue,
    /// 1-based, as written; at least 1.
    pub number: i64,
    pub to: Residue,
}

impl Substitution {
    /// The canonical one-letter token, upper-case: `A42G`. It is the
    /// member name a proposed request uses (plan §5.5).
    pub fn canonical(&self) -> String {
        let from: String = self.from.symbol();
        let to: String = self.to.symbol();
        format!("{}{}{}", from, self.number, to)
    }
}

/// A part of a substitution a `?` stood for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Original,
    Number,
    Target,
}

/// Why a token is not a substitution luria can make.
#[derive(Clone, Debug, PartialEq)]
pub enum NotationError {
    /// No form milestone 6 accepts. `token` is the text exactly as given.
    Unsupported { token: String },
    /// A well-formed substitution with `?` in place of these parts, in the
    /// order original, number, target. Never empty.
    Unstated { parts: Vec<Part> },
}

/// A side of a token as written: a lone `?`, or the characters that stood
/// there, possibly none. A side is text, never a value: the steps after it
/// decide what the text means.
#[derive(Clone, Debug, PartialEq)]
enum Side {
    /// A lone `?` stood here: the user did not state this part.
    Question,
    /// The characters of this side as written; empty when the token held
    /// nothing this side could take.
    Run(String),
}

/// Whether `ch` is an ASCII letter, the only characters a residue side may
/// hold (contract §3.4 step 2 and step 4).
fn is_side_letter(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}

/// The first position at or after `at` holding something that is not an
/// ASCII letter: the end of the longest run of letters at `at`.
fn run_end(chars: &[char], at: usize) -> usize {
    let mut end: usize = at;
    while end < chars.len() && is_side_letter(chars[end]) {
        end += 1;
    }
    end
}

/// Step 2 and 5's raw text: the side starting at `at`, and the position
/// just past it. A `?` standing alone is the whole side; otherwise the side
/// is the longest run of ASCII letters at `at`, which may be empty.
fn read_side(chars: &[char], at: usize) -> (Side, usize) {
    if at < chars.len() && chars[at] == '?' {
        return (Side::Question, at + 1);
    }
    let end: usize = run_end(chars, at);
    let run: String = chars[at..end].iter().collect();
    (Side::Run(run), end)
}

/// Step 3's raw text: the digits at `at`, and the position just past them.
/// A lone `?` is the whole field; otherwise it is the longest run of ASCII
/// digits, which may be empty.
fn read_digits(chars: &[char], at: usize) -> (Side, usize) {
    if at < chars.len() && chars[at] == '?' {
        return (Side::Question, at + 1);
    }
    let mut end: usize = at;
    while end < chars.len() && chars[end].is_ascii_digit() {
        end += 1;
    }
    let run: String = chars[at..end].iter().collect();
    (Side::Run(run), end)
}

/// Step 4: the target side is everything left, so a `?` stands for the part
/// only when it is all that is left.
fn read_rest(chars: &[char]) -> Side {
    let rest: String = chars.iter().collect();
    if rest == "?" {
        return Side::Question;
    }
    Side::Run(rest)
}

/// Step 3: the number a run of digits states. Non-empty, no leading `0`,
/// and a value from 1 to `i64::MAX`; anything else gives `None`.
fn read_number(digits: String) -> Option<i64> {
    let letters: Vec<char> = digits.chars().collect();
    if letters.is_empty() || letters[0] == '0' {
        return None;
    }
    digits.parse::<i64>().ok()
}

/// Steps 5 and 6's input: the residue a side of letters names, with the
/// notation it used (its length in letters), or `None` when it is not 1 or
/// 3 ASCII letters of a code that exists.
fn read_residue(letters: &[char]) -> Option<(Residue, usize)> {
    if letters.len() == 1 {
        return one_letter_code(letters[0]);
    }
    if letters.len() == 3 {
        return three_letter_code(letters);
    }
    None
}

/// A 1-letter side is a one-letter code, matched against
/// `RESIDUE_NAMES[..].letter` ignoring ASCII case (contract §3.4 step 5).
fn one_letter_code(letter: char) -> Option<(Residue, usize)> {
    let want: char = letter.to_ascii_lowercase();
    for entry in RESIDUE_NAMES.iter() {
        if entry.letter.to_ascii_lowercase() == want {
            match residue_for_symbol(entry.letter) {
                Some(residue) => {
                    return Some((residue, 1));
                }
                None => {
                    return None;
                }
            }
        }
    }
    None
}

/// A 3-letter side is a three-letter code, matched against
/// `RESIDUE_NAMES[..].three` ignoring ASCII case (contract §3.4 step 5).
fn three_letter_code(letters: &[char]) -> Option<(Residue, usize)> {
    let written: String = letters.iter().collect();
    for entry in RESIDUE_NAMES.iter() {
        if entry.three.eq_ignore_ascii_case(&written) {
            match residue_for_symbol(entry.letter) {
                Some(residue) => {
                    return Some((residue, 3));
                }
                None => {
                    return None;
                }
            }
        }
    }
    None
}

/// Read one mutation token: `A42G`, `p.Ala42Gly` or `A?G`.
pub fn parse_substitution(token: &str) -> Result<Substitution, NotationError> {
    let chars: Vec<char> = token.chars().collect();

    // Step 1: only an exactly `p.` prefix is removed; nothing else is, and
    // nothing is trimmed, so whitespace anywhere reaches the steps below.
    let body: &[char] = if chars.len() >= 2 && chars[0] == 'p' && chars[1] == '.' {
        &chars[2..]
    } else {
        &chars[..]
    };

    // Step 2: the original side, as written.
    let original_field: (Side, usize) = read_side(body, 0);
    let original: Side = original_field.0;
    let after_original: usize = original_field.1;

    // Step 3: the number's text, as written.
    let digits_field: (Side, usize) = read_digits(body, after_original);
    let digits: Side = digits_field.0;
    let after_digits: usize = digits_field.1;

    // Step 4: the target side is everything left.
    let target: Side = read_rest(&body[after_digits..]);

    // The residue each side names, and the notation it used: 1 for a
    // one-letter code, 3 for a three-letter code, 0 for a `?`.
    let mut from: Option<Residue> = None;
    let mut from_width: usize = 0;
    let mut number: Option<i64> = None;
    let mut to: Option<Residue> = None;
    let mut to_width: usize = 0;

    // The parts a `?` stood for, in the order original, number, target.
    // Malformed beats unstated: each step below refuses a bad field
    // outright, so `parts` is only ever returned once every field read
    // well, steps 5 and 6 included (contract §3.4 step 7 and step 8).
    let mut parts: Vec<Part> = [].to_vec();

    // Step 5: the residue the original side names.
    match original {
        Side::Question => {
            parts.push(Part::Original);
        }
        Side::Run(side) => {
            let letters: Vec<char> = side.chars().collect();
            match read_residue(&letters[..]) {
                Some((residue, width)) => {
                    from = Some(residue);
                    from_width = width;
                }
                None => {
                    return refuse(token);
                }
            }
        }
    }

    // Step 3: the number, or which part a `?` stood for.
    match digits {
        Side::Question => {
            parts.push(Part::Number);
        }
        Side::Run(text) => match read_number(text) {
            Some(value) => {
                number = Some(value);
            }
            None => {
                return refuse(token);
            }
        },
    }

    // Step 5: the residue the target side names.
    match target {
        Side::Question => {
            parts.push(Part::Target);
        }
        Side::Run(side) => {
            let letters: Vec<char> = side.chars().collect();
            match read_residue(&letters[..]) {
                Some((residue, width)) => {
                    to = Some(residue);
                    to_width = width;
                }
                None => {
                    return refuse(token);
                }
            }
        }
    }

    // Step 6: one notation. When both sides are letters, both are one
    // letter or both three; `Ala42G` and `A42Gly` are refused.
    if from_width > 0 && to_width > 0 && from_width != to_width {
        return refuse(token);
    }

    // Step 8: only a token that passed steps 1 to 6 can be `Unstated`.
    if !parts.is_empty() {
        return Err(NotationError::Unstated { parts });
    }

    // Every field read well and none was a `?`, so each holds a value.
    // These refusals cannot be reached; they keep this function total
    // rather than panicking (contract §5).
    let from_residue: Residue = match from {
        Some(residue) => residue,
        None => {
            return refuse(token);
        }
    };

    let value: i64 = match number {
        Some(found) => found,
        None => {
            return refuse(token);
        }
    };

    let to_residue: Residue = match to {
        Some(residue) => residue,
        None => {
            return refuse(token);
        }
    };

    Ok(Substitution {
        from: from_residue,
        number: value,
        to: to_residue,
    })
}

/// The reason a token is not a substitution, naming it exactly as given.
fn refuse(token: &str) -> Result<Substitution, NotationError> {
    Err(NotationError::Unsupported {
        token: token.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parsed(token: &str) -> Substitution {
        parse_substitution(token)
            .unwrap_or_else(|error| panic!("{:?} was refused: {:?}", token, error))
    }

    fn refused(token: &str) -> NotationError {
        match parse_substitution(token) {
            Err(error) => error,
            Ok(found) => panic!("{:?} parsed as {:?}", token, found),
        }
    }

    #[test]
    fn every_accepted_form_reads_the_same_substitution() {
        let tokens: [&str; 8] = [
            "A42G",
            "a42g",
            "p.A42G",
            "p.a42g",
            "Ala42Gly",
            "ala42gly",
            "ALA42GLY",
            "p.Ala42Gly",
        ];
        for token in tokens {
            let found: Substitution = parsed(token);
            assert_eq!(found.from, Residue::A, "{}", token);
            assert_eq!(found.number, 42, "{}", token);
            assert_eq!(found.to, Residue::G, "{}", token);
            assert_eq!(found.canonical(), "A42G", "{}", token);
        }
    }

    #[test]
    fn the_corpus_tokens_read_as_written() {
        let cases: [(&str, Residue, i64, Residue); 3] = [
            ("W87A", Residue::W, 87, Residue::A),
            ("V1A", Residue::V, 1, Residue::A),
            ("A420G", Residue::A, 420, Residue::G),
        ];
        for case in cases {
            let found: Substitution = parsed(case.0);
            assert_eq!(found.from, case.1, "{}", case.0);
            assert_eq!(found.number, case.2, "{}", case.0);
            assert_eq!(found.to, case.3, "{}", case.0);
            assert_eq!(found.canonical(), case.0, "{}", case.0);
        }
    }

    // Not this parser's business (contract §2): a same-residue token and a
    // number past any CDS parse, and core judges them at design.
    #[test]
    fn biology_is_not_judged_here() {
        assert_eq!(parsed("A42A").canonical(), "A42A");
        assert_eq!(parsed("A9999G").number, 9999);
        assert_eq!(parsed("A9223372036854775807G").number, i64::MAX);
    }

    #[test]
    fn every_residue_reads_in_both_notations() {
        for entry in RESIDUE_NAMES.iter() {
            let expected: Residue = residue_for_symbol(entry.letter)
                .unwrap_or_else(|| panic!("core has no residue {}", entry.letter));
            let one: String = format!("{}7{}", entry.letter, entry.letter);
            let three: String = format!("{}7{}", entry.three, entry.three);
            for token in [one.clone(), three.clone(), three.to_ascii_lowercase()] {
                let found: Substitution = parsed(&token);
                assert_eq!(found.from, expected.clone(), "{}", token);
                assert_eq!(found.to, expected.clone(), "{}", token);
                assert_eq!(found.canonical(), one, "{}", token);
            }
        }
    }

    #[test]
    fn a_question_mark_reports_each_unstated_part_in_order() {
        let cases: [(&str, Vec<Part>); 7] = [
            ("?42G", vec![Part::Original]),
            ("A?G", vec![Part::Number]),
            ("A42?", vec![Part::Target]),
            ("??G", vec![Part::Original, Part::Number]),
            ("???", vec![Part::Original, Part::Number, Part::Target]),
            ("p.?42Gly", vec![Part::Original]),
            ("Ala?Gly", vec![Part::Number]),
        ];
        for case in cases {
            assert_eq!(
                refused(case.0),
                NotationError::Unstated { parts: case.1 },
                "{}",
                case.0
            );
        }
    }

    #[test]
    fn every_other_form_is_unsupported_and_named() {
        let tokens: [&str; 37] = [
            "",
            "A42",
            "42G",
            "AG",
            "A42*",
            "Ala42Ter",
            "*101Q",
            "K12del",
            "A42_G43insK",
            "A42fs",
            "A42=",
            "c.123A>G",
            "P.A42G",
            "pA42G",
            " A42G",
            "A42G ",
            "A 42 G",
            "A042G",
            "A0G",
            "A-4G",
            "A9223372036854775808G",
            "Ala42G",
            "A42Gly",
            "Xaa42Gly",
            "B42G",
            "Z42G",
            "X42G",
            "U42G",
            "O42G",
            "AL42G",
            "Alan42G",
            "\u{c4}42G",
            "A42\u{11e}",
            "A4\u{ff12}G",
            "A42G?",
            "Ala?G",
            "A?42G",
        ];
        for token in tokens {
            assert_eq!(
                refused(token),
                NotationError::Unsupported {
                    token: token.to_string()
                },
                "{:?}",
                token
            );
        }
    }

    // The table and core must name the same twenty residues (plan §5.3a).
    #[test]
    fn the_table_holds_exactly_cores_twenty_residues() {
        let mut letters: Vec<char> = RESIDUE_NAMES.iter().map(|entry| entry.letter).collect();
        letters.sort();
        letters.dedup();
        assert_eq!(letters.len(), 20, "a letter repeats in RESIDUE_NAMES");
        for entry in RESIDUE_NAMES.iter() {
            let residue: Residue = residue_for_symbol(entry.letter)
                .unwrap_or_else(|| panic!("core has no residue {}", entry.letter));
            assert_eq!(residue.symbol(), entry.letter.to_string());
        }
        for letter in 'A'..='Z' {
            assert_eq!(
                residue_for_symbol(letter).is_some(),
                letters.contains(&letter),
                "core and the table disagree about {}",
                letter
            );
        }
    }

    #[test]
    fn three_letter_codes_and_names_are_distinct() {
        let mut threes: Vec<String> = RESIDUE_NAMES
            .iter()
            .map(|entry| entry.three.to_ascii_lowercase())
            .collect();
        threes.sort();
        threes.dedup();
        assert_eq!(threes.len(), 20, "a three-letter code repeats");
        let mut names: Vec<&str> = RESIDUE_NAMES
            .iter()
            .flat_map(|entry| entry.names.iter().copied())
            .collect();
        let count: usize = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), count, "a residue name repeats");
        assert_eq!(count, 22, "20 residues, two with a second name");
    }

    // The sourced values themselves (IUPAC-IUB 1983, 3AA-1), pinned so that
    // an edit to a code or a name cannot pass unnoticed: every other test
    // reads the table back to itself.
    #[test]
    fn the_table_holds_the_iupac_codes_and_names() {
        let expected: [(char, &str, &[&str]); 20] = [
            ('A', "Ala", &["alanine"]),
            ('R', "Arg", &["arginine"]),
            ('N', "Asn", &["asparagine"]),
            ('D', "Asp", &["aspartate", "aspartic acid"]),
            ('C', "Cys", &["cysteine"]),
            ('Q', "Gln", &["glutamine"]),
            ('E', "Glu", &["glutamate", "glutamic acid"]),
            ('G', "Gly", &["glycine"]),
            ('H', "His", &["histidine"]),
            ('I', "Ile", &["isoleucine"]),
            ('L', "Leu", &["leucine"]),
            ('K', "Lys", &["lysine"]),
            ('M', "Met", &["methionine"]),
            ('F', "Phe", &["phenylalanine"]),
            ('P', "Pro", &["proline"]),
            ('S', "Ser", &["serine"]),
            ('T', "Thr", &["threonine"]),
            ('W', "Trp", &["tryptophan"]),
            ('Y', "Tyr", &["tyrosine"]),
            ('V', "Val", &["valine"]),
        ];
        assert_eq!(RESIDUE_NAMES.len(), expected.len());
        for (entry, want) in RESIDUE_NAMES.iter().zip(expected.iter()) {
            assert_eq!(
                (entry.letter, entry.three, entry.names),
                (want.0, want.1, want.2),
                "the table's entry for {}",
                want.0
            );
        }
    }
}
