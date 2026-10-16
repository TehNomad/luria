//! DNA sequence artifacts: canonical text, alphabet, topology, content hash.
//!
//! A `Sequence` is the *artifact*, not a display name. Two sequences with the
//! same canonical text have the same content hash and are the same sequence
//! for identity purposes; two sequences with the same display name but
//! different content are different sequences, and any operation that assumes
//! otherwise is rejected with `SequenceReferenceMismatch`. Structures never
//! carry a bare display name: `Sequence.reference()` yields a `SequenceReference`
//! (name, content digest, and topology), and that is what intervals, features and CDSs
//! store and compare.
//!
//! ```text
//! Sequence.create("seq-a", "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG", Linear)
//! Sequence.create("seq-a-copy", "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG", Linear)
//! ```
//!
//! Both artifacts above have the same content hash, so either may be used as
//! the reference for a feature that points at that content.
//!
//! The supported alphabet is the DNA IUPAC nucleotide set
//! `A C G T R Y S W K M B D H V N`, closed under complement. Letters outside
//! that set are malformed input, not a validation failure. `U` is RNA and is
//! not accepted.
//!
//! IUPAC ambiguity codes and their complements:
//!
//! ```text
//! R = A/G   <-> Y = C/T
//! S = C/G   <-> S
//! W = A/T   <-> W
//! K = G/T   <-> M = A/C
//! B = C/G/T <-> V = A/C/G
//! D = A/G/T <-> H = A/C/T
//! N = any   <-> N
//! ```
//!
//! Normalization is ASCII-only (decision D008: normalization never repairs
//! biological content). Lowercase ASCII `a-z` folds to `A-Z`; nothing else is
//! folded, so a non-ASCII character is reported rather than being mapped onto
//! a DNA letter by Unicode case folding.

use crate::error::CoordinateOutOfBoundsProblem;
use crate::error::DomainError;
use crate::error::EmptySequenceProblem;
use crate::error::InvalidSequenceCharactersProblem;
use crate::identity::sha256_hex;
use crate::identity::ContentHash;
use crate::identity::Identifier;
use crate::identity::SequenceReference;

/// The supported DNA IUPAC alphabet, closed under complement.
pub const DNA_ALPHABET: &str = "ACGTRYSWKMBDHVN";

/// The four unambiguous DNA bases.
pub const DNA_BASES: &str = "ACGT";

/// Lowercase ASCII letters, the only characters normalization may fold.
const ASCII_LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const ASCII_UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Clone, Debug, PartialEq)]
pub enum SequenceTopology {
    Linear,
    Circular,
}

impl SequenceTopology {
    pub fn name(&self) -> String {
        match self {
            SequenceTopology::Linear => "linear".to_string(),
            SequenceTopology::Circular => "circular".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sequence {
    name: Identifier,
    content_hash: ContentHash,
    letters: Vec<char>,
    length: u64,
    topology: SequenceTopology,
}

pub fn is_dna_letter(ch: char) -> bool {
    DNA_ALPHABET.contains(ch)
}

/// True for `A`, `C`, `G`, `T` only; false for every ambiguity code.
pub fn is_dna_base(ch: char) -> bool {
    DNA_BASES.contains(ch)
}

/// ASCII case folding only.
///
/// Lowercase ASCII `a-z` folds to `A-Z`; every other character is returned
/// unchanged and then fails the alphabet check. Unicode case folding is never
/// applied: `ß` must not become `S` (a valid DNA letter and a valid residue),
/// and `ſ` (U+017F, Latin small letter long s) must not become `S` either.
fn ascii_upper(ch: char) -> char {
    let lower: Vec<char> = ASCII_LOWER.chars().collect();
    let upper: Vec<char> = ASCII_UPPER.chars().collect();
    let mut index: usize = 0;
    while index < lower.len() {
        if lower[index] == ch {
            return upper[index];
        }
        index += 1;
    }
    ch
}

/// Complement of a DNA IUPAC letter. Returns `None` for anything outside
/// the supported alphabet, so the caller can report a domain error rather
/// than inventing a result.
pub fn complement_letter(ch: char) -> Option<char> {
    match ch {
        'A' => Some('T'),
        'T' => Some('A'),
        'G' => Some('C'),
        'C' => Some('G'),
        'R' => Some('Y'),
        'Y' => Some('R'),
        'S' => Some('S'),
        'W' => Some('W'),
        'K' => Some('M'),
        'M' => Some('K'),
        'B' => Some('V'),
        'V' => Some('B'),
        'D' => Some('H'),
        'H' => Some('D'),
        'N' => Some('N'),
        _ => None,
    }
}

/// The concrete bases an IUPAC nucleotide code stands for, in `A`, `C`, `G`,
/// `T` order. `None` for any character outside `DNA_ALPHABET`.
///
/// The sets are the standard IUPAC nucleotide sets, and each one is the
/// complement-image of its partner's set under `A`<->`T`, `C`<->`G` (for
/// example `B` = `C G T` complements to `V` = `A C G`), so the table below
/// agrees with `complement_letter` by construction.
pub fn bases_of(letter: char) -> Option<Vec<char>> {
    let bases: &str = match letter {
        'A' => "A",
        'C' => "C",
        'G' => "G",
        'T' => "T",
        'R' => "AG",
        'Y' => "CT",
        'S' => "CG",
        'W' => "AT",
        'K' => "GT",
        'M' => "AC",
        'B' => "CGT",
        'D' => "AGT",
        'H' => "ACT",
        'V' => "ACG",
        'N' => "ACGT",
        _ => return None,
    };
    Some(bases.chars().collect())
}

/// Reverse complement of DNA letters.
///
/// A letter outside the supported alphabet is `InvalidSequenceCharacters`,
/// with `position` the index of that letter in the input, rather than a
/// panic or an invented complement.
pub fn reverse_complement(letters: Vec<char>) -> Result<Vec<char>, DomainError> {
    let mut out: Vec<char> = Vec::new();
    for index in 0..letters.len() {
        let position: usize = letters.len() - 1 - index;
        let ch = letters[position];
        match complement_letter(ch) {
            Some(complement) => out.push(complement),
            None => {
                return Err(DomainError::InvalidSequenceCharacters(
                    InvalidSequenceCharactersProblem {
                        sequence: "reverse complement input".to_string(),
                        position: position as u64,
                        characters: ch.to_string(),
                    },
                ));
            }
        }
    }
    Ok(out)
}

pub fn canonical_text(letters: Vec<char>) -> String {
    let mut text = String::new();
    for ch in letters {
        text.push(ch);
    }
    text
}

/// Every concrete sequence a text of IUPAC codes denotes, in D033's order:
/// each position's bases are taken in `A`, `C`, `G`, `T` order and the
/// **leftmost position varies fastest**, so a replacement of `NS` gives
/// `AC, CC, GC, TC, AG, CG, GG, TG`.
///
/// The order is produced by counting: for expansion index `i` of `n` total,
/// each position's base is chosen from its own set with the leftmost
/// position's set as the innermost-varying digit, so the order does not
/// depend on the text's length. The number of expansions is the product of
/// the set sizes; it is not capped here, because the 64-design warning of
/// D033 belongs at the request layer, not in this function.
///
/// A character outside `DNA_ALPHABET` is `InvalidSequenceCharacters`, with
/// `sequence` the text `replacement` and `position` the zero-based index of
/// the first offending character. Nothing is case-folded here: callers pass
/// canonical text, so lowercase letters are reported rather than folded.
/// An empty text is not an error; it expands to no sequences, and the
/// caller decides whether an empty replacement is legal.
pub fn expand_degenerate(text: &str) -> Result<Vec<String>, DomainError> {
    let letters: Vec<char> = text.chars().collect();

    // The bases each position stands for, in `A`, `C`, `G`, `T` order.
    let mut sets: Vec<Vec<char>> = Vec::new();
    let mut index: usize = 0;
    while index < letters.len() {
        match bases_of(letters[index]) {
            Some(bases) => sets.push(bases),
            None => {
                return Err(DomainError::InvalidSequenceCharacters(
                    InvalidSequenceCharactersProblem {
                        sequence: "replacement".to_string(),
                        position: index as u64,
                        characters: letters[index].to_string(),
                    },
                ));
            }
        }
        index += 1;
    }
    if sets.is_empty() {
        return Ok(Vec::new());
    }

    // `strides[position]` is the product of the set sizes of the positions
    // to the left of `position`, which makes the leftmost position the
    // least significant digit and therefore the fastest varying one.
    let mut strides: Vec<usize> = Vec::new();
    let mut stride: usize = 1;
    let mut set_index: usize = 0;
    while set_index < sets.len() {
        strides.push(stride);
        stride *= sets[set_index].len();
        set_index += 1;
    }
    let count: usize = stride;

    let mut out: Vec<String> = Vec::new();
    let mut expansion: usize = 0;
    while expansion < count {
        let mut word: Vec<char> = Vec::new();
        let mut position: usize = 0;
        while position < sets.len() {
            let digit: usize = (expansion / strides[position]) % sets[position].len();
            word.push(sets[position][digit]);
            position += 1;
        }
        out.push(canonical_text(word));
        expansion += 1;
    }
    Ok(out)
}

/// Normalization and validation of FASTA-style sequence text.
///
/// * lines beginning with `;` are record headers and are ignored
/// * whitespace is ignored
/// * lowercase ASCII letters are uppercased; nothing else is folded
/// * every remaining character must be in the DNA IUPAC alphabet
/// * the content must be non-empty
///
/// `InvalidSequenceCharacters.position` is the zero-based index into the
/// canonical text of the offending character, not an offset into the raw
/// source text.
pub fn normalize(
    name: Identifier,
    text: String,
    topology: SequenceTopology,
) -> Result<Sequence, DomainError> {
    let mut letters: Vec<char> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(";") {
            continue;
        }
        for ch in trimmed.chars() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                continue;
            }
            let upper = ascii_upper(ch);
            if !is_dna_letter(upper) {
                return Err(DomainError::InvalidSequenceCharacters(
                    InvalidSequenceCharactersProblem {
                        sequence: name.value().to_string(),
                        position: letters.len() as u64,
                        characters: upper.to_string(),
                    },
                ));
            }
            letters.push(upper);
        }
    }
    if letters.is_empty() {
        return Err(DomainError::EmptySequenceContent(EmptySequenceProblem {
            sequence: name.value().to_string(),
        }));
    }
    let canonical = canonical_text(letters.clone());
    Ok(Sequence {
        name,
        content_hash: ContentHash::create(sha256_hex(canonical.as_bytes())),
        letters: letters.clone(),
        length: letters.len() as u64,
        topology,
    })
}

impl Sequence {
    pub fn name(&self) -> Identifier {
        self.name.clone()
    }

    pub fn content_hash(&self) -> ContentHash {
        self.content_hash.clone()
    }

    /// Identity of this artifact: display name, content digest, and topology.
    ///
    /// Intervals, features and CDSs carry this value rather than a bare
    /// display name, so applying them to a different artifact that happens
    /// to share a display name is rejected.
    pub fn reference(&self) -> SequenceReference {
        SequenceReference::create(
            self.name.clone(),
            self.content_hash.clone(),
            self.topology.clone(),
        )
    }

    pub fn topology(&self) -> SequenceTopology {
        self.topology.clone()
    }

    pub fn length(&self) -> u64 {
        self.length
    }

    pub fn letters(&self) -> Vec<char> {
        self.letters.clone()
    }

    pub fn canonical_text(&self) -> String {
        canonical_text(self.letters.clone())
    }

    pub fn display_name(&self) -> String {
        self.name.value().to_string()
    }

    /// Zero-based position into the canonical text.
    pub fn letter_at(&self, position: u64) -> Result<char, DomainError> {
        if position >= self.length {
            return Err(DomainError::CoordinateOutOfBounds(
                CoordinateOutOfBoundsProblem {
                    sequence: self.name.value().to_string(),
                    start: position,
                    end: position + 1,
                    length: self.length,
                },
            ));
        }
        Ok(self.letters[position as usize])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::DomainError;

    #[test]
    fn fasta_noise_does_not_change_content() {
        let raw = ";seq-a\nATGGCTTGGAATCAGCAGCAGCAGCAGCAG\n";
        let sequence = normalize(
            crate::identity::Identifier::create("seq-a".to_string()).unwrap(),
            raw.to_string(),
            SequenceTopology::Linear,
        );
        assert!(sequence.is_ok());
        assert!(sequence.unwrap().canonical_text() == "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG");
    }

    #[test]
    fn lowercase_and_whitespace_are_normalized() {
        let identifier = crate::identity::Identifier::create("seq-a".to_string()).unwrap();
        let sequence = normalize(
            identifier,
            "atg gct\ntgg aat\ncag cag cag cag cag cag\n".to_string(),
            SequenceTopology::Linear,
        );
        assert!(sequence.is_ok());
        assert!(sequence.unwrap().canonical_text() == "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG");
    }

    #[test]
    fn characters_outside_the_alphabet_are_malformed_input() {
        let identifier = crate::identity::Identifier::create("seq-g".to_string()).unwrap();
        let sequence = normalize(
            identifier,
            "ATGGCTTGGA#T".to_string(),
            SequenceTopology::Linear,
        );
        let error = sequence.unwrap_err();
        assert!(matches!(error, DomainError::InvalidSequenceCharacters(_)));
        assert!(error.category() == crate::error::ErrorCategory::MalformedInput);
    }

    #[test]
    fn unsupported_letters_are_rejected_not_silently_kept() {
        let identifier = crate::identity::Identifier::create("seq-h".to_string()).unwrap();
        let sequence = normalize(
            identifier,
            "ATGGCTTGGAU".to_string(),
            SequenceTopology::Linear,
        );
        assert!(
            sequence.unwrap_err()
                == crate::error::DomainError::InvalidSequenceCharacters(
                    crate::error::InvalidSequenceCharactersProblem {
                        sequence: "seq-h".to_string(),
                        position: 10,
                        characters: "U".to_string(),
                    }
                )
        );
    }

    #[test]
    fn empty_content_is_malformed_input() {
        let identifier = crate::identity::Identifier::create("seq-i".to_string()).unwrap();
        let sequence = normalize(
            identifier,
            ";seq-i\n\n".to_string(),
            SequenceTopology::Linear,
        );
        assert!(matches!(
            sequence.unwrap_err(),
            DomainError::EmptySequenceContent(_)
        ));
    }

    #[test]
    fn the_alphabet_is_closed_under_complement() {
        for ch in DNA_ALPHABET.chars() {
            let complement = complement_letter(ch);
            assert!(complement.is_some());
            assert!(is_dna_letter(complement.unwrap()));
            assert!(complement_letter(complement.unwrap()) == Some(ch));
        }
    }

    // The IUPAC nucleotide codes (NC-IUB 1984): fifteen symbols, and the
    // complement of an ambiguity code is the code for the complemented set,
    // e.g. B = C/G/T complements to G/C/A = V.
    #[test]
    fn the_alphabet_is_the_iupac_nucleotide_set() {
        assert!(DNA_ALPHABET.len() == 15);
        let pairs: Vec<(char, char)> = vec![
            ('A', 'T'),
            ('C', 'G'),
            ('R', 'Y'),
            ('S', 'S'),
            ('W', 'W'),
            ('K', 'M'),
            ('B', 'V'),
            ('D', 'H'),
            ('N', 'N'),
        ];
        for (letter, complement) in pairs {
            assert!(complement_letter(letter) == Some(complement));
            assert!(complement_letter(complement) == Some(letter));
        }
    }

    #[test]
    fn letters_that_are_not_iupac_nucleotide_codes_are_rejected() {
        for text in ["ATGF", "ATGU", "ATGE", "ATG*"] {
            let sequence = normalize(
                Identifier::create("seq-x".to_string()).unwrap(),
                text.to_string(),
                SequenceTopology::Linear,
            );
            assert!(matches!(
                sequence.unwrap_err(),
                DomainError::InvalidSequenceCharacters(_)
            ));
        }
    }

    #[test]
    fn every_iupac_ambiguity_code_is_accepted() {
        let sequence = normalize(
            Identifier::create("seq-x".to_string()).unwrap(),
            "ryswkmbdhvn".to_string(),
            SequenceTopology::Linear,
        );
        assert!(sequence.unwrap().canonical_text() == "RYSWKMBDHVN");
    }

    #[test]
    fn reverse_complement_of_a_known_subsequence() {
        // rc(ATGGCTTGGAA) = TTCCAAGCCAT, obtained by reversing the text
        // and letter-wise complementing it (A<->T, C<->G).
        let letters: Vec<char> = "ATGGCTTGGAA".chars().collect();
        let complemented = reverse_complement(letters.clone()).unwrap();
        assert!(canonical_text(complemented) == "TTCCAAGCCAT");
    }

    #[test]
    fn reverse_complement_is_an_involution() {
        let letters: Vec<char> = "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG".chars().collect();
        let twice = reverse_complement(reverse_complement(letters.clone()).unwrap()).unwrap();
        assert!(canonical_text(twice) == canonical_text(letters));
    }

    #[test]
    fn reverse_complement_reports_a_letter_outside_the_alphabet() {
        let error = reverse_complement(vec!['A', 'U', 'G']).unwrap_err();
        assert!(
            error
                == DomainError::InvalidSequenceCharacters(InvalidSequenceCharactersProblem {
                    sequence: "reverse complement input".to_string(),
                    position: 1,
                    characters: "U".to_string(),
                })
        );
    }

    #[test]
    fn non_ascii_characters_are_never_folded_into_dna_letters() {
        // Unicode case folding would turn these into valid DNA letters:
        //   ß  (U+00DF) -> SS -> S   (a DNA letter, and the residue Serine)
        //   ſ  (U+017F) -> S
        //   É  (U+00C9) -> É        (unchanged, still not a DNA letter)
        // Normalization must reject all of them instead of repairing content.
        let built = Identifier::create("seq-u".to_string());
        assert!(built.is_ok());
        let identifier: Identifier = built.unwrap();
        let sharp_s = normalize(
            identifier.clone(),
            "ATGGCTTGGßA".to_string(),
            SequenceTopology::Linear,
        );
        assert!(
            sharp_s.unwrap_err()
                == crate::error::DomainError::InvalidSequenceCharacters(
                    crate::error::InvalidSequenceCharactersProblem {
                        sequence: "seq-u".to_string(),
                        position: 9,
                        characters: "ß".to_string(),
                    }
                )
        );

        let long_s = normalize(
            identifier.clone(),
            "ATGGCTTGGſA".to_string(),
            SequenceTopology::Linear,
        );
        assert!(
            long_s.unwrap_err()
                == crate::error::DomainError::InvalidSequenceCharacters(
                    crate::error::InvalidSequenceCharactersProblem {
                        sequence: "seq-u".to_string(),
                        position: 9,
                        characters: "ſ".to_string(),
                    }
                )
        );

        let accented = normalize(
            identifier.clone(),
            "ATGGCTTGGÉA".to_string(),
            SequenceTopology::Linear,
        );
        assert!(
            accented.unwrap_err()
                == crate::error::DomainError::InvalidSequenceCharacters(
                    crate::error::InvalidSequenceCharactersProblem {
                        sequence: "seq-u".to_string(),
                        position: 9,
                        characters: "É".to_string(),
                    }
                )
        );

        // The combining-diaereses case: Unicode folding of ẗ (U+1E97) yields
        // "T" plus a combining mark. Only the T would survive a naive fold,
        // silently inserting a base.
        let dotted = normalize(
            identifier.clone(),
            "ATGGCTTGGẗA".to_string(),
            SequenceTopology::Linear,
        );
        assert!(matches!(
            dotted.unwrap_err(),
            DomainError::InvalidSequenceCharacters(_)
        ));
    }

    #[test]
    fn ascii_folding_is_a_total_map_on_lowercase_letters() {
        let lowered = "atggccttggaa".to_string();
        let folded: Vec<char> = lowered.chars().collect();
        let mut index: usize = 0;
        while index < folded.len() {
            let upper = ascii_upper(folded[index]);
            assert!(is_dna_letter(upper));
            index += 1;
        }
        assert!(ascii_upper('ß') == 'ß');
        assert!(ascii_upper('ſ') == 'ſ');
        assert!(ascii_upper('z') == 'Z');
        assert!(ascii_upper('Z') == 'Z');
    }

    #[test]
    fn sequence_reference_pairs_name_and_content_digest() {
        let built = Identifier::create("seq-a".to_string());
        assert!(built.is_ok());
        let identifier: Identifier = built.unwrap();
        let sequence = normalize(
            identifier.clone(),
            "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG".to_string(),
            SequenceTopology::Linear,
        );
        assert!(sequence.is_ok());
        let artifact = sequence.unwrap();
        let reference = artifact.reference();
        assert!(reference.name() == artifact.name());
        assert!(reference.content_hash() == artifact.content_hash());
        assert!(
            reference.content_hash().hex_digest()
                == "a83f6d825c144cef563438c085821ecbd8986cdaf0f65c75f7994ed616a2fd80"
        );
        assert!(reference.describe() == "seq-a@a83f6d82");
    }

    // D033 (approved 2026-10-15): each position's bases are taken in `A`, `C`,
    // `G`, `T` order and the leftmost position varies fastest. The order below
    // is quoted from the reference tool's help text, so it is the authority for
    // `expand_degenerate`.
    #[test]
    fn the_quoted_expansion_of_ns_is_in_reference_order() {
        let expansion = expand_degenerate("NS").unwrap();
        let expected: Vec<String> = vec![
            "AC".to_string(),
            "CC".to_string(),
            "GC".to_string(),
            "TC".to_string(),
            "AG".to_string(),
            "CG".to_string(),
            "GG".to_string(),
            "TG".to_string(),
        ];
        assert!(expansion == expected);
    }

    #[test]
    fn unambiguous_bases_expand_to_the_text_itself() {
        assert!(expand_degenerate("ACGT").unwrap() == vec!["ACGT".to_string()]);
    }

    #[test]
    fn a_single_code_expands_to_the_bases_it_stands_for() {
        assert!(expand_degenerate("R").unwrap() == vec!["A".to_string(), "G".to_string()]);
        assert!(
            expand_degenerate("N").unwrap()
                == vec![
                    "A".to_string(),
                    "C".to_string(),
                    "G".to_string(),
                    "T".to_string(),
                ]
        );
    }

    #[test]
    fn the_leftmost_position_varies_fastest() {
        // RRA stands for A/G, A/G, A. The first position cycles on every
        // expansion, the second only every two, and the third never varies.
        assert!(
            expand_degenerate("RRA").unwrap()
                == vec![
                    "AAA".to_string(),
                    "GAA".to_string(),
                    "AGA".to_string(),
                    "GGA".to_string(),
                ]
        );
    }

    #[test]
    fn every_iupac_code_stands_for_its_bases_in_base_order() {
        // The standard IUPAC nucleotide sets, each written in `A`, `C`, `G`,
        // `T` order.
        let expected: Vec<(char, String)> = vec![
            ('A', "A".to_string()),
            ('C', "C".to_string()),
            ('G', "G".to_string()),
            ('T', "T".to_string()),
            ('R', "AG".to_string()),
            ('Y', "CT".to_string()),
            ('S', "CG".to_string()),
            ('W', "AT".to_string()),
            ('K', "GT".to_string()),
            ('M', "AC".to_string()),
            ('B', "CGT".to_string()),
            ('D', "AGT".to_string()),
            ('H', "ACT".to_string()),
            ('V', "ACG".to_string()),
            ('N', "ACGT".to_string()),
        ];
        assert!(expected.len() == 15);
        for pair in expected {
            assert!(is_dna_letter(pair.0));
            let bases = bases_of(pair.0);
            assert!(bases.is_some());
            assert!(canonical_text(bases.unwrap()) == pair.1);
        }
        assert!(bases_of('F').is_none());
        assert!(bases_of('a').is_none());
    }

    #[test]
    fn each_code_set_is_the_complement_image_of_its_partner() {
        // The table in `bases_of` must agree with `complement_letter`: the set
        // of the partner code equals the complements of the code's own bases.
        for ch in DNA_ALPHABET.chars() {
            let partner: char = complement_letter(ch).unwrap();
            let mut left: Vec<char> = bases_of(partner).unwrap();
            let mut right: Vec<char> = Vec::new();
            for base in bases_of(ch).unwrap() {
                right.push(complement_letter(base).unwrap());
            }
            left.sort();
            right.sort();
            assert!(left == right);
        }
    }

    #[test]
    fn the_number_of_expansions_is_the_product_of_the_set_sizes() {
        // `N` stands for four bases, so three and four of them denote 4^3 and
        // 4^4 concrete sequences, all distinct.
        let triples = expand_degenerate("NNN").unwrap();
        assert!(triples.len() == 64);
        let mut first: usize = 0;
        while first < triples.len() {
            let mut second: usize = first + 1;
            while second < triples.len() {
                assert!(triples[first] != triples[second]);
                second += 1;
            }
            first += 1;
        }
        assert!(expand_degenerate("NNNN").unwrap().len() == 256);
    }

    #[test]
    fn an_empty_text_expands_to_no_sequences() {
        assert!(expand_degenerate("").unwrap().is_empty());
    }

    #[test]
    fn a_character_outside_the_alphabet_is_reported_with_its_position() {
        let error = expand_degenerate("ACFG").unwrap_err();
        assert!(
            error
                == DomainError::InvalidSequenceCharacters(InvalidSequenceCharactersProblem {
                    sequence: "replacement".to_string(),
                    position: 2,
                    characters: "F".to_string(),
                })
        );
    }
}
