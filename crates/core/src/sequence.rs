//! DNA sequence artifacts: canonical text, alphabet, topology, content hash.
//!
//! A `Sequence` is the *artifact*, not a display name. Two sequences with the
//! same canonical text have the same content hash and are the same sequence
//! for identity purposes; two sequences with the same display name but
//! different content are different sequences, and any operation that assumes
//! otherwise is rejected with `SequenceReferenceMismatch`. Structures never
//! carry a bare display name: `Sequence.reference()` yields a `SequenceReference`
//! (name plus content digest), and that is what intervals, features and CDSs
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
//! The supported alphabet is the DNA IUPAC set `A C G T N R Y S K F M V W`,
//! closed under complement. Letters outside that set are malformed input, not
//! a validation failure.
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
pub const DNA_ALPHABET: &str = "ACGTNRYSKFMVW";

/// Lowercase ASCII letters, the only characters normalization may fold.
const ASCII_LOWER: &str = "abcdefghijklmnopqrstuvwxyz";
const ASCII_UPPER: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

#[derive(Clone, Debug, PartialEq)]
pub enum SequenceTopology {
    Linear,
    Circular,
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
        'N' => Some('N'),
        'R' => Some('Y'),
        'Y' => Some('R'),
        'S' => Some('S'),
        'K' => Some('M'),
        'M' => Some('K'),
        'F' => Some('V'),
        'V' => Some('F'),
        'W' => Some('W'),
        _ => None,
    }
}

pub fn reverse_complement(letters: Vec<char>) -> Vec<char> {
    let mut out: Vec<char> = Vec::new();
    for index in 0..letters.len() {
        let ch = letters[letters.len() - 1 - index];
        out.push(complement_letter(ch).unwrap());
    }
    out
}

pub fn canonical_text(letters: Vec<char>) -> String {
    let mut text = String::new();
    for ch in letters {
        text.push(ch);
    }
    text
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

    /// Identity of this artifact: display name plus content digest.
    ///
    /// Intervals, features and CDSs carry this value rather than a bare
    /// display name, so applying them to a different artifact that happens
    /// to share a display name is rejected.
    pub fn reference(&self) -> SequenceReference {
        SequenceReference::create(self.name.clone(), self.content_hash.clone())
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
            assert!(complement_letter(ch).is_some());
        }
    }

    #[test]
    fn reverse_complement_of_a_known_subsequence() {
        // rc(ATGGCTTGGAA) = TTCCAAGCCAT, obtained by reversing the text
        // and letter-wise complementing it (A<->T, C<->G).
        let letters: Vec<char> = "ATGGCTTGGAA".chars().collect();
        let complemented = reverse_complement(letters.clone());
        assert!(canonical_text(complemented) == "TTCCAAGCCAT");
    }

    #[test]
    fn reverse_complement_is_an_involution() {
        let letters: Vec<char> = "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG".chars().collect();
        let twice = reverse_complement(reverse_complement(letters.clone()));
        assert!(canonical_text(twice) == canonical_text(letters));
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
}
