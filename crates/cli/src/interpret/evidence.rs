//! What a user's quote shows of one substitution: the evidence checks of
//! plan §5.3a and the numbering decision of §5.5 (contract 6e1), written as
//! pure functions over text. Nothing here sees the model, a catalogue, a
//! profile or a request; 6e2 calls these to decide between proposing a
//! request and asking a question.
//!
//! Every rule fails closed (D005): a doubtful case is *not* evidence, so
//! luria asks rather than assuming. The rules, exactly:
//!
//! - `digit_runs` lists every maximal run of ASCII digits in a quote, in
//!   order, as half-open char spans. A residue number is visible only when
//!   the quote holds exactly one run, its text equals the number's own text,
//!   and the run is joined to no ASCII letter except a residue's name: it
//!   starts at the quote's start, after a non-letter, or where a naming of
//!   the original ends (`A42G`, `Ala42Gly`), and it ends at the quote's end,
//!   before a non-letter, or where a naming of the target starts. So `42` is
//!   not visible in `420` or `142`, nor inside `ydj42` or `synA42`, and a
//!   number below 1 is never visible. A quote with two or more runs shows no
//!   number, and `build` asks `mutation_quote_ambiguous` about it (D045).
//! - A residue is named in a quote three ways: by its one-letter code, as
//!   an upper-case letter with no ASCII letter on either side and an ASCII
//!   digit immediately before or after it (so the article "A" and the
//!   pronoun "I" are not evidence); by its three-letter code, in any ASCII
//!   case, bounded by non-letters, except that all-lower-case `his`, `met`
//!   and `pro` are English words; or by one of its English names, in any
//!   ASCII case, bounded by non-letters. Mentions are half-open char spans,
//!   sorted by start then end, with exact repeats removed.
//! - When a number is visible, the original is in order only when a naming
//!   of the original ends at or before that run's start and a naming of the
//!   target starts at or after its end, so "residue 42: alanine to glycine"
//!   is refused. Where the number is not visible — no run, or one run joined
//!   to a name — the original is in order before the target when some
//!   mention of the original ends at or before the start of some mention of
//!   the target, so "glycine 42 to alanine" reported as `A42G` is refused;
//!   when both are the same residue, one mention cannot serve as both. Two
//!   or more runs are in order never, and `build` asks one question for such
//!   a quote (D045).
//! - The user's numbering words mean "from the start of the CDS" only when
//!   their normalized form — ASCII lower case, every character that is not
//!   an ASCII letter or digit treated as a space — holds one of
//!   `ORIGIN_PHRASES` as consecutive words and holds no word of
//!   `NUMBERING_VETO`, a negation, a contrast or another origin.
//!
//! All positions are char indices, as in `notation.rs`: nothing indexes a
//! `&str` by byte, and no input panics, non-ASCII characters included.

use crate::interpret::notation::Substitution;
use crate::interpret::notation::RESIDUE_NAMES;
use luria_core::genetic_code::Residue;

/// What a quote shows of one substitution (plan §5.3a).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Evidence {
    /// The residue number is written in the quote.
    pub number: bool,
    /// The original residue is named in the quote.
    pub original: bool,
    /// The target residue is named in the quote.
    pub target: bool,
    /// Some naming of the original ends at or before the start of some
    /// naming of the target. False whenever either is not named.
    pub in_order: bool,
}

/// The phrases, in normalized form (§3.3.4), that state numbering from the
/// start of the CDS.
pub const ORIGIN_PHRASES: [&str; 9] = [
    "from the start of the cds",
    "from the start of the annotated cds",
    "from the start of the coding sequence",
    "from the beginning of the cds",
    "from the beginning of the annotated cds",
    "from the beginning of the coding sequence",
    "from the cds start",
    "from the first codon of the cds",
    "from the start codon",
];

/// Words that, anywhere in the numbering words, make them doubtful: a
/// negation, a contrast, or another origin. `t` is the end of a
/// contraction such as "isn't" once normalized.
pub const NUMBERING_VETO: [&str; 18] = [
    "not",
    "never",
    "no",
    "t",
    "except",
    "instead",
    "rather",
    "but",
    "mature",
    "precursor",
    "signal",
    "tag",
    "plasmid",
    "construct",
    "vector",
    "uniprot",
    "pdb",
    "structure",
];

/// The end of the run of ASCII digits starting at `at`.
fn digits_end(chars: &[char], at: usize) -> usize {
    let mut end: usize = at;
    while end < chars.len() && chars[end].is_ascii_digit() {
        end += 1;
    }
    end
}

/// Whether the char just before `at` exists and is an ASCII letter: a
/// mention may only start at the quote's start or after a non-letter.
fn letter_just_before(chars: &[char], at: usize) -> bool {
    at > 0 && chars[at - 1].is_ascii_alphabetic()
}

/// Whether the char at `end` exists and is an ASCII letter: a mention may
/// only end at the quote's end or before a non-letter.
fn letter_at(chars: &[char], at: usize) -> bool {
    at < chars.len() && chars[at].is_ascii_alphabetic()
}

/// Whether an ASCII digit sits immediately before or after the one-letter
/// code at `at`, so `A42` and `W87` count and "A glycine" does not.
fn digit_alongside(chars: &[char], at: usize) -> bool {
    (at > 0 && chars[at - 1].is_ascii_digit())
        || (at + 1 < chars.len() && chars[at + 1].is_ascii_digit())
}

/// Every maximal run of ASCII digits in `quote`, as half-open char spans in
/// order: `digit_runs("A37G and A42S")` is `[(1, 3), (10, 12)]` (D045).
pub fn digit_runs(quote: &str) -> Vec<(usize, usize)> {
    let chars: Vec<char> = quote.chars().collect();
    let mut runs: Vec<(usize, usize)> = Vec::new();
    let mut at: usize = 0;
    while at < chars.len() {
        if !chars[at].is_ascii_digit() {
            at += 1;
            continue;
        }
        let end: usize = digits_end(&chars, at);
        runs.push((at, end));
        at = end;
    }
    runs
}

/// Whether `number` is written in `quote` as a maximal run of ASCII digits
/// (§3.3.1). Only whole runs are compared, so `42` is not visible in `420`
/// or `142`; a `number` below 1 is never visible.
pub fn number_visible(quote: &str, number: i64) -> bool {
    if number < 1 {
        return false;
    }
    let chars: Vec<char> = quote.chars().collect();
    let written: String = number.to_string();
    let mut at: usize = 0;
    while at < chars.len() {
        if !chars[at].is_ascii_digit() {
            at += 1;
            continue;
        }
        let end: usize = digits_end(&chars, at);
        let run: String = chars[at..end].iter().collect();
        if run == written {
            return true;
        }
        at = end;
    }
    false
}

/// Every place `residue` is named in `quote`, as half-open char spans
/// sorted by start then end, without repeats (§3.3.2). One-letter codes
/// need an adjacent digit; all-lower-case `his`, `met` and `pro` are not
/// three-letter codes.
pub fn residue_mentions(quote: &str, residue: &Residue) -> Vec<(usize, usize)> {
    let chars: Vec<char> = quote.chars().collect();
    let symbol: String = residue.symbol();
    let mut spans: Vec<(usize, usize)> = Vec::new();
    for entry in RESIDUE_NAMES.iter() {
        if entry.letter.to_string() != symbol {
            continue;
        }

        let mut at: usize = 0;
        while at < chars.len() {
            if chars[at] == entry.letter
                && !letter_just_before(&chars, at)
                && !letter_at(&chars, at + 1)
                && digit_alongside(&chars, at)
            {
                spans.push((at, at + 1));
            }
            at += 1;
        }

        at = 0;
        while at + 3 <= chars.len() {
            let written: String = chars[at..at + 3].iter().collect();
            let english: bool = written == "his" || written == "met" || written == "pro";
            if entry.three.eq_ignore_ascii_case(&written)
                && !english
                && !letter_just_before(&chars, at)
                && !letter_at(&chars, at + 3)
            {
                spans.push((at, at + 3));
            }
            at += 1;
        }

        for name in entry.names.iter() {
            let width: usize = name.chars().count();
            if width == 0 || width > chars.len() {
                continue;
            }
            at = 0;
            while at + width <= chars.len() {
                let written: String = chars[at..at + width].iter().collect();
                if name.eq_ignore_ascii_case(&written)
                    && !letter_just_before(&chars, at)
                    && !letter_at(&chars, at + width)
                {
                    spans.push((at, at + width));
                }
                at += 1;
            }
        }
    }
    spans.sort();
    spans.dedup();
    spans
}

/// The order rule of plan §5.3a, used whenever the quote shows no number for
/// the residues to sit around: some mention of the original ends at or
/// before the start of some mention of the target.
fn mentions_in_order(originals: &[(usize, usize)], targets: &[(usize, usize)]) -> bool {
    for original in originals.iter() {
        for target in targets.iter() {
            if original.1 <= target.0 {
                return true;
            }
        }
    }
    false
}

/// What `quote` shows of `substitution` (§3.3.3, tightened by D045): the
/// number as the quote's one digit run, joined to no letter but a residue's
/// name, each residue as a naming, and the original named before that run
/// and the target after it. A quote with two or more runs shows neither the
/// number nor an order, so `build` asks which number is the residue number.
pub fn evidence(quote: &str, substitution: &Substitution) -> Evidence {
    let originals: Vec<(usize, usize)> = residue_mentions(quote, &substitution.from);
    let targets: Vec<(usize, usize)> = residue_mentions(quote, &substitution.to);
    let runs: Vec<(usize, usize)> = digit_runs(quote);
    let chars: Vec<char> = quote.chars().collect();
    let mut number: bool = false;
    let mut in_order: bool = false;
    match runs.len() {
        0 => {
            in_order = mentions_in_order(&originals, &targets);
        }
        1 => {
            let (start, end): (usize, usize) = runs[0];
            let run: String = chars[start..end].iter().collect();
            let bounded_left: bool = start == 0
                || !letter_just_before(&chars, start)
                || originals.iter().any(|original| original.1 == start);
            let bounded_right: bool = end == chars.len()
                || !letter_at(&chars, end)
                || targets.iter().any(|target| target.0 == end);
            if number_visible(&run, substitution.number) && bounded_left && bounded_right {
                number = true;
                in_order = originals.iter().any(|original| original.1 <= start)
                    && targets.iter().any(|target| target.0 >= end);
            } else {
                // The quote's only problem is how its number is written, so
                // it earns the number question alone and keeps today's rule.
                in_order = mentions_in_order(&originals, &targets);
            }
        }
        _ => {
            // D045: more than one run shows no number and no order; `build`
            // asks which number is the residue number (`mutation_quote_ambiguous`).
        }
    }
    Evidence {
        number,
        original: !originals.is_empty(),
        target: !targets.is_empty(),
        in_order,
    }
}

/// The words of the user's numbering words, normalized (§3.3.4): ASCII
/// lower case, every character that is not an ASCII letter or digit
/// treated as a space, empty pieces dropped.
fn normalized_words(words: &str) -> Vec<String> {
    let mut list: Vec<String> = Vec::new();
    let mut piece: String = String::new();
    for character in words.chars() {
        let letter: char = character.to_ascii_lowercase();
        if letter.is_ascii_alphabetic() || letter.is_ascii_digit() {
            piece.push(letter);
        } else if !piece.is_empty() {
            list.push(piece);
            piece = String::new();
        }
    }
    if !piece.is_empty() {
        list.push(piece);
    }
    list
}

/// The words of one `ORIGIN_PHRASES` entry, written as words separated by
/// spaces.
fn phrase_words(phrase: &str) -> Vec<String> {
    phrase
        .split(" ")
        .filter(|word| !word.is_empty())
        .map(|word| word.to_string())
        .collect()
}

/// Whether `phrase` appears in `list` as consecutive words.
fn phrase_present(list: &[String], phrase: &[String]) -> bool {
    if phrase.is_empty() || phrase.len() > list.len() {
        return false;
    }
    let mut at: usize = 0;
    while at + phrase.len() <= list.len() {
        let window: &[String] = &list[at..at + phrase.len()];
        if window == phrase {
            return true;
        }
        at += 1;
    }
    false
}

/// Whether the user's numbering words state numbering from the start of the
/// CDS (M6-11, §3.3.4): only a phrase of `ORIGIN_PHRASES`, as consecutive
/// normalized words, and no word of `NUMBERING_VETO`. Anything off the
/// lists is refused, so luria asks.
pub fn numbering_from_cds_start(words: &str) -> bool {
    let list: Vec<String> = normalized_words(words);
    if list.is_empty() {
        return false;
    }
    for veto in NUMBERING_VETO.iter() {
        if list.iter().any(|word| word == veto) {
            return false;
        }
    }
    for phrase in ORIGIN_PHRASES.iter() {
        if phrase_present(&list, &phrase_words(phrase)) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpret::notation::parse_substitution;

    fn token(text: &str) -> Substitution {
        parse_substitution(text).unwrap_or_else(|error| panic!("{}: {:?}", text, error))
    }

    fn shows(number: bool, original: bool, target: bool, in_order: bool) -> Evidence {
        Evidence {
            number,
            original,
            target,
            in_order,
        }
    }

    /// One residue-naming case: the quote, the residue, the spans expected.
    type MentionCase = (&'static str, Residue, Vec<(usize, usize)>);

    #[test]
    fn a_number_is_visible_only_as_a_whole_digit_run() {
        let cases: [(&str, i64, bool); 12] = [
            ("alanine 42 to glycine", 42, true),
            ("A42G", 42, true),
            ("Ala42Gly", 42, true),
            ("V1A", 1, true),
            ("A9223372036854775807G", i64::MAX, true),
            ("alanine 420 to glycine", 42, false),
            ("alanine 142 to glycine", 42, false),
            ("alanine 4 2 to glycine", 42, false),
            ("alanine to glycine", 42, false),
            ("A42G", 4, false),
            ("A\u{ff14}\u{ff12}G", 42, false),
            ("residue 0", 0, false),
        ];
        for case in cases {
            assert_eq!(
                number_visible(case.0, case.1),
                case.2,
                "{:?} {}",
                case.0,
                case.1
            );
        }
    }

    // D045 (B-1): the digit runs of a quote are the numbers it holds.
    #[test]
    fn digit_runs_are_maximal_and_in_order() {
        let cases: [(&str, Vec<(usize, usize)>); 5] = [
            ("A37G and A42S", vec![(1, 3), (10, 12)]),
            ("Q5 A42G", vec![(1, 2), (4, 6)]),
            ("", vec![]),
            ("alanine", vec![]),
            ("\u{e9}42", vec![(1, 3)]),
        ];
        for case in cases {
            assert_eq!(digit_runs(case.0), case.1, "{:?}", case.0);
        }
    }

    #[test]
    fn each_way_of_naming_a_residue_is_found_where_it_is_written() {
        let cases: [MentionCase; 14] = [
            ("A42G", Residue::A, vec![(0, 1)]),
            ("A42G", Residue::G, vec![(3, 4)]),
            ("p.A42G", Residue::A, vec![(2, 3)]),
            ("Ala42Gly", Residue::A, vec![(0, 3)]),
            ("Ala42Gly", Residue::G, vec![(5, 8)]),
            ("ALA-42-gly", Residue::G, vec![(7, 10)]),
            ("change alanine 42 to glycine", Residue::A, vec![(7, 14)]),
            ("change alanine 42 to glycine", Residue::G, vec![(21, 28)]),
            ("Trp87\u{2192}Ala", Residue::W, vec![(0, 3)]),
            ("Trp87\u{2192}Ala", Residue::A, vec![(6, 9)]),
            ("W87 to alanine", Residue::W, vec![(0, 1)]),
            (
                "aspartic acid 102 or Aspartate",
                Residue::D,
                vec![(0, 13), (21, 30)],
            ),
            ("\u{e9}alanine 42", Residue::A, vec![(1, 8)]),
            ("alanine 42 to alanine", Residue::A, vec![(0, 7), (14, 21)]),
        ];
        for case in cases {
            assert_eq!(
                residue_mentions(case.0, &case.1),
                case.2,
                "{:?} {:?}",
                case.0,
                case.1
            );
        }
    }

    // Ordinary English is not evidence (plan §5.3a, tightenings).
    #[test]
    fn words_that_only_look_like_codes_are_not_mentions() {
        let none: [(&str, Residue); 10] = [
            ("A glycine at 42", Residue::A),
            ("I need it", Residue::I),
            ("In synA42", Residue::A),
            ("his residue 42", Residue::H),
            ("we met at 42", Residue::M),
            ("pro 42", Residue::P),
            ("glycyl 42", Residue::G),
            ("alanine", Residue::L),
            ("Alanine", Residue::N),
            ("AG", Residue::A),
        ];
        for case in none {
            assert_eq!(
                residue_mentions(case.0, &case.1),
                vec![],
                "{:?} {:?}",
                case.0,
                case.1
            );
        }
        assert_eq!(residue_mentions("His57", &Residue::H), vec![(0, 3)]);
        assert_eq!(residue_mentions("HIS 57", &Residue::H), vec![(0, 3)]);
        assert_eq!(residue_mentions("Met1", &Residue::M), vec![(0, 3)]);
        assert_eq!(residue_mentions("Pro12", &Residue::P), vec![(0, 3)]);
        assert_eq!(residue_mentions("histidine 57", &Residue::H), vec![(0, 9)]);
        // "alanine" holds "ala" but not as a code: the letter after it.
        assert_eq!(residue_mentions("alanine", &Residue::A), vec![(0, 7)]);
    }

    #[test]
    fn the_corpus_quotes_show_what_the_corpus_says() {
        let cases: [(&str, &str, Evidence); 11] = [
            (
                "alanine 42 to glycine",
                "A42G",
                shows(true, true, true, true),
            ),
            ("Ala42Gly", "A42G", shows(true, true, true, true)),
            ("A42G", "A42G", shows(true, true, true, true)),
            ("W87 to alanine", "W87A", shows(true, true, true, true)),
            ("valine 1 to alanine", "V1A", shows(true, true, true, true)),
            ("alanine to glycine", "A42G", shows(false, true, true, true)),
            (
                "alanine 420 to glycine",
                "A42G",
                shows(false, true, true, true),
            ),
            (
                "residue 42 to glycine",
                "A42G",
                shows(true, false, true, false),
            ),
            (
                "alanine 42 to something better",
                "A42G",
                shows(true, true, false, false),
            ),
            ("delete K12", "K12A", shows(true, true, false, false)),
            (
                "the alanine at position 42 of synA to a glycine",
                "A42G",
                shows(true, true, true, true),
            ),
        ];
        for case in cases {
            assert_eq!(
                evidence(case.0, &token(case.1)),
                case.2,
                "{:?} {}",
                case.0,
                case.1
            );
        }
    }

    // A swapped token must not pass: synA's residue 42 really is alanine,
    // so core's expected-residue check would not catch it (plan §5.3a).
    #[test]
    fn the_original_must_be_named_before_the_target() {
        let cases: [(&str, &str, Evidence); 4] = [
            (
                "glycine 42 to alanine",
                "A42G",
                shows(true, true, true, false),
            ),
            (
                "make 42 a glycine instead of the alanine",
                "A42G",
                shows(true, true, true, false),
            ),
            (
                "alanine 42 to alanine",
                "A42A",
                shows(true, true, true, true),
            ),
            ("alanine 42", "A42A", shows(true, true, true, false)),
        ];
        for case in cases {
            assert_eq!(
                evidence(case.0, &token(case.1)),
                case.2,
                "{:?} {}",
                case.0,
                case.1
            );
        }
    }

    // B-1 (D045): a quote holding two numbers shows no residue number.
    #[test]
    fn a_quote_with_two_numbers_shows_no_number() {
        assert_eq!(
            evidence("A37G and A42S", &token("A37S")),
            shows(false, true, true, false)
        );
    }

    // B-2 (D045): a digit run joined to a name is not a residue number, and
    // the run must be joined to the residues, not sit beside a name.
    #[test]
    fn a_number_joined_to_other_letters_is_not_the_residue_number() {
        assert_eq!(
            evidence("In ydj42, change the alanine to a glycine", &token("A42G")),
            shows(false, true, true, true)
        );
        assert_eq!(
            evidence("synA42 to glycine", &token("A42G")),
            shows(false, false, true, false)
        );
        assert_eq!(
            evidence("alanine 42nd to glycine", &token("A42G")),
            shows(false, true, true, true)
        );
    }

    // D045: with the number visible, the original is before it and the
    // target after it; a quote that fails only this earns the number alone.
    #[test]
    fn the_original_is_before_the_number_and_the_target_after_it() {
        assert_eq!(
            evidence(
                "change the alanine to a glycine; see figure 42",
                &token("A42G")
            ),
            shows(true, true, true, false)
        );
        assert_eq!(
            evidence("residue 42: alanine to glycine", &token("A42G")),
            shows(true, true, true, false)
        );
    }

    #[test]
    fn numbering_words_are_accepted_only_from_the_list() {
        let accepted: [&str; 11] = [
            "from the start of the CDS",
            "from the CDS start",
            "from the beginning of the annotated CDS",
            "Number residues from the start of the CDS.",
            "Residues are numbered from the CDS start.",
            "counting from the start of the CDS",
            "numbering from the CDS start",
            "(numbered from the CDS start)",
            "Number from the beginning of the CDS.",
            "numbered from the start codon",
            "FROM THE START OF THE CODING SEQUENCE",
        ];
        for words in accepted {
            assert!(numbering_from_cds_start(words), "{:?} was refused", words);
        }
        let refused: [&str; 13] = [
            "",
            "from the start of the plasmid",
            "from the start",
            "from residue 1",
            "1-based",
            "UniProt numbering",
            "from the CDS start, not the mature protein",
            "from the start of the mature CDS",
            "from the CDS start except the signal peptide",
            "this isn't from the CDS start",
            "from the CDS start in the PDB structure",
            "from the CDS start but after the tag",
            "fromthe CDS start",
        ];
        for words in refused {
            assert!(!numbering_from_cds_start(words), "{:?} was accepted", words);
        }
    }

    #[test]
    fn no_input_panics() {
        let inputs: [&str; 6] = [
            "",
            "\u{2192}",
            "\u{e9}\u{e9}\u{e9}",
            "4",
            "A",
            "\u{1f9ec}42\u{1f9ec}",
        ];
        for input in inputs {
            let _ = number_visible(input, 42);
            let _ = residue_mentions(input, &Residue::A);
            let _ = evidence(input, &token("A42G"));
            let _ = numbering_from_cds_start(input);
        }
    }
}
