//! DNA search: every place a pattern may match a record, on either strand
//! (plan §4.4; D052 M8-7, M8-8 and M8-16; D050 M8-16; D049).
//!
//! The rules, as §4.4 and the decisions give them:
//!
//! * **Containment, not intersection** (D052, M8-7). A record letter at an
//!   aligned position matches a pattern letter only when **every** base that
//!   record letter stands for lies in the base set `sequence::bases_of` gives
//!   for the pattern letter. A record `A` is matched by `A R W M D H V N`, a
//!   record `R` by `R D V N`, and a record `N` by `N` alone. `covers` is that
//!   rule, public so `search-letters.tsv` can pin all 225 letter pairs — 110 of
//!   which an intersection rule would answer the other way round. Intersection
//!   answers `N` against `A` yes; this rule answers no, and that is the point.
//! * **Patterns are folded before anything else**: ASCII case is dropped, then
//!   `U` is read as `T`. A pattern may hold `U` where a stored sequence may
//!   not, which is why the fold happens here and not in `sequence::normalize`.
//!   Folding comes before validation, so `u` is a legal pattern letter and
//!   `X` is not; the refusal names the letter as written and its zero-based
//!   position (§4.4).
//! * **Both strands**: the minus-strand hits are the top-strand hits of the
//!   pattern's reverse complement, reported with strand `-` at the same
//!   top-strand coordinates.
//! * **One hit per span** (D052, M8-8). A span that matches on both strands is
//!   reported once as `HitStrand::Both`, never as two hits. pBR322 makes that
//!   checkable: its EcoRI site `GAATTC` occurs exactly once in the whole
//!   record, at zero-based 4358, and only because the record is circular.
//! * **Wrapping follows the record's own topology** (D049): a circular record
//!   offers starts `0..=length - 1`, and a span running past the last base is
//!   read again from the first base; a linear record offers starts
//!   `0..=length - width` and nothing wraps. `wraps_origin` is *computed* as
//!   `start + width > length`, never inferred from a doubled copy of the
//!   record.
//! * **Every position, no cap** (D050, M8-16): overlapping matches are all
//!   reported — `AA` on `AAA` is three hits — and a pattern as long as the
//!   record matches at every rotation it matches on a circular record. A
//!   pattern longer than the record finds nothing, which is not an error.
//!   There is no cap in core; the 200-hit limit is presentation (D048).
//! * **Coordinates are zero-based and half-open** (D006): a hit covers
//!   `[start, start + length)` of the top strand, and both strands are
//!   reported in top-strand coordinates.
//!
//! `tests/fixtures/view/search-letters.tsv`, `search-dna-patterns.tsv` and
//! `search-dna-hits.tsv` are the answers, produced by
//! `tests/fixtures/view/reference/search_dna.py`, which shares no code with
//! this module (D039); `crates/core/tests/search_fixtures.rs` compares them.
//! Biopython is that script's oracle for the letter sets and for site counts
//! over records with no ambiguity codes, and for nothing else — the README
//! beside the tables records what it does not answer (D052).

use crate::error::DomainError;
use crate::error::InvalidSearchPatternCharactersProblem;
use crate::sequence::bases_of;
use crate::sequence::complement_letter;
use crate::sequence::Sequence;
use crate::sequence::SequenceTopology;

/// The four bases a mask can hold: bit 0 is `A`, bit 1 `C`, bit 2 `G`, bit 3
/// `T`, in the order `sequence::bases_of` lists a code's bases in.
const B_A: u8 = 1 << 0;
const B_C: u8 = 1 << 1;
const B_G: u8 = 1 << 2;
const B_T: u8 = 1 << 3;

/// What an IUPAC nucleotide code stands for, as a 4-bit mask over `A`, `C`,
/// `G`, `T`: the same base sets `sequence::bases_of` gives, and
/// `the_masks_agree_with_the_base_sets` says so for every pair of the alphabet.
///
/// A letter outside `sequence::DNA_ALPHABET` gives `0`, the empty set: it stands
/// for no base and matches nothing, which is how `bases_of` answers `None` for
/// it. `U` and `u` are not in the alphabet, and a pattern's `U` is read as `T`
/// by `fold_letter` before it is masked.
fn base_mask(letter: char) -> u8 {
    match letter {
        'A' => B_A,
        'C' => B_C,
        'G' => B_G,
        'T' => B_T,
        'R' => B_A | B_G,
        'Y' => B_C | B_T,
        'S' => B_C | B_G,
        'W' => B_A | B_T,
        'K' => B_G | B_T,
        'M' => B_A | B_C,
        'B' => B_C | B_G | B_T,
        'D' => B_A | B_G | B_T,
        'H' => B_A | B_C | B_T,
        'V' => B_A | B_C | B_G,
        'N' => B_A | B_C | B_G | B_T,
        _ => 0,
    }
}

/// §4.4's containment over masks: every base in the template's set is in the
/// pattern's set, which is `template & !pattern == 0` once the template's set
/// is non-empty — and never an intersection, which would answer `S` for `R`.
fn masks_cover(template: u8, pattern: u8) -> bool {
    template != 0 && template & !pattern == 0
}

/// The strand a DNA hit was found on. `Both` is its own state, not a pair of
/// flags: a place that matches on both strands is one hit (M8-8, D052).
///
/// This is deliberately not `strand::Strand`: `Strand` has exactly `Plus` and
/// `Minus` and its variants are matched exhaustively across core, and
/// `Option<Strand>` for "both" is ambiguous with "unknown".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HitStrand {
    Plus,
    Minus,
    Both,
}

impl HitStrand {
    /// The symbol the tables and `luria find` use: `+`, `-`, `both`.
    pub fn symbol(self) -> String {
        match self {
            HitStrand::Plus => "+".to_string(),
            HitStrand::Minus => "-".to_string(),
            HitStrand::Both => "both".to_string(),
        }
    }
}

/// One place a pattern matches: zero-based, half-open, top-strand coordinates
/// on both strands (D006).
#[derive(Clone, Debug, PartialEq)]
pub struct DnaHit {
    /// Zero-based position of the first covered base on the top strand.
    pub start: u64,
    /// Number of bases covered, which is the folded pattern's length.
    pub length: u64,
    pub strand: HitStrand,
    /// True when the span runs past the end of a circular record.
    pub wraps_origin: bool,
}

/// §4.4's matching rule, public so the 225-row letter table can pin it:
/// a template letter matches a pattern letter only when every base the
/// template letter stands for is one of the pattern letter's bases.
///
/// Containment, never intersection: template `S` (`C`,`G`) and pattern `R`
/// (`A`,`G`) share `G` and do **not** match, and template `N` matches pattern
/// `N` alone. A letter outside `sequence::DNA_ALPHABET` stands for no known
/// set and matches nothing.
pub fn covers(template: char, pattern: char) -> bool {
    let template_mask: u8 = base_mask(template);
    let pattern_mask: u8 = base_mask(pattern);
    // Containment over the base sets as a mask test: the sets are four bits
    // wide, so `template & !pattern` is empty exactly when every base the
    // template letter stands for is one the pattern letter stands for. An
    // empty template set (`0`, a letter outside the alphabet) matches nothing,
    // as `bases_of`'s `None` did.
    masks_cover(template_mask, pattern_mask)
}

/// A pattern letter as §4.4 reads it, before anything else: ASCII case is
/// dropped, then `U` becomes `T`. ASCII-only folding, exactly as
/// `sequence::normalize` folds — a non-ASCII letter is never folded into a DNA
/// letter, it is refused. The fold is one character to one character, so an
/// index into the folded pattern is still an index into the pattern as written.
fn fold_letter(letter: char) -> char {
    let upper: char = letter.to_ascii_uppercase();
    if upper == 'U' {
        'T'
    } else {
        upper
    }
}

/// The span of `pattern` at `start`, read base by base off the top strand and
/// wrapped modulo the record length where it runs past the last base — never
/// from a doubled copy of the record (rule 6). `pattern` is the base mask of
/// each pattern letter, which `find_dna` builds once for the whole pattern.
fn span_matches(sequence: &Sequence, start: u64, pattern: &[u8]) -> bool {
    let letters: &[char] = sequence.letter_slice();
    let size: u64 = sequence.length();
    for (offset, mask) in pattern.iter().enumerate() {
        // The modulo keeps every position inside the record; it is only asked
        // for where the span runs past the last base, and `size` is at least
        // the pattern width wherever this runs, so a position in `start +
        // offset` is never out of range by more than one record length.
        let position: u64 = start + offset as u64;
        let index: usize = if position < size {
            position as usize
        } else {
            (position % size) as usize
        };
        // The checked read (D006): a span that cannot be read does not match,
        // rather than a panic.
        let template: char = match letters.get(index) {
            Some(template) => *template,
            None => return false,
        };
        if !masks_cover(base_mask(template), *mask) {
            return false;
        }
    }
    true
}

/// Every start in `0..=last` where `pattern` matches the top strand, ascending,
/// with every overlapping match kept: the scan steps one base, never the width
/// of the pattern (D050, M8-16).
fn top_strand_starts(sequence: &Sequence, pattern: &[u8], last: u64) -> Vec<u64> {
    let mut starts: Vec<u64> = Vec::new();
    let mut start: u64 = 0;
    while start <= last {
        if span_matches(sequence, start, pattern) {
            starts.push(start);
        }
        start += 1;
    }
    starts
}

/// One hit at `start`: the span always has the folded pattern's width, and
/// `wraps_origin` is computed from the coordinates (rule 6).
fn hit_at(start: u64, width: u64, size: u64, strand: HitStrand) -> DnaHit {
    DnaHit {
        start,
        length: width,
        strand,
        wraps_origin: start.saturating_add(width) > size,
    }
}

/// Every hit of `pattern` on `sequence`, both strands, in the order §4.4 gives.
///
/// * An empty pattern is `DomainError::EmptySearchPattern`; a pattern letter
///   outside the IUPAC DNA alphabet **after folding** is
///   `DomainError::InvalidSearchPatternCharacters`, naming the letter as
///   written and its zero-based position.
/// * A pattern longer than the record, and a pattern that matches nowhere,
///   give an empty vector rather than an error (§4.4).
/// * Hits are ordered by `start` ascending; there is no cap.
pub fn find_dna(sequence: &Sequence, pattern: &str) -> Result<Vec<DnaHit>, DomainError> {
    let written: Vec<char> = pattern.chars().collect();
    let folded: Vec<char> = written.iter().map(|letter| fold_letter(*letter)).collect();
    if folded.is_empty() {
        return Err(DomainError::EmptySearchPattern);
    }
    // Validate **after** folding — `u` and `U` are legal pattern letters — in
    // one pass that also builds the reverse complement through
    // `sequence::complement_letter` (the alphabet is closed under it, so the
    // `None` arm cannot be reached; it answers with the same refusal rather
    // than inventing a complement). `sequence::reverse_complement` is not used:
    // it refuses `U`, and a pattern may hold `U`.
    let mut complemented: Vec<char> = Vec::with_capacity(folded.len());
    for (position, (letter, as_written)) in folded.iter().zip(written.iter()).enumerate() {
        match (bases_of(*letter), complement_letter(*letter)) {
            (Some(_), Some(complement)) => complemented.push(complement),
            (None, _) | (_, None) => {
                return Err(DomainError::InvalidSearchPatternCharacters(
                    InvalidSearchPatternCharactersProblem {
                        pattern: pattern.to_string(),
                        position: position as u64,
                        characters: as_written.to_string(),
                    },
                ));
            }
        }
    }
    complemented.reverse();

    // The pattern and its reverse complement are turned into base masks once
    // here, never per start: the scan reads a mask off the record's own
    // letters. Both were validated above, so no mask is `0`.
    let pattern_masks: Vec<u8> = folded.iter().map(|letter| base_mask(*letter)).collect();
    let complement_masks: Vec<u8> = complemented
        .iter()
        .map(|letter| base_mask(*letter))
        .collect();

    let width: u64 = folded.len() as u64;
    let size: u64 = sequence.length();
    // Rule 7: a pattern longer than the record has no legal start on either
    // topology, and that is an empty answer rather than a refusal (§4.4).
    if width > size {
        return Ok(Vec::new());
    }
    // Rule 6: what a record offers follows from the topology its caller gave it
    // (D049), not from its text. A circular record offers every start and a
    // span may run past the last base; a linear one stops at `size - width`.
    let last: u64 = if sequence.topology() == SequenceTopology::Circular {
        size - 1
    } else {
        size - width
    };
    let plus: Vec<u64> = top_strand_starts(sequence, &pattern_masks, last);
    // Rule 4: the minus-strand hits are the top-strand hits of the reverse
    // complement, at the same top-strand coordinates.
    let minus: Vec<u64> = top_strand_starts(sequence, &complement_masks, last);

    let mut candidates: Vec<(u64, HitStrand)> = Vec::new();
    for start in plus {
        candidates.push((start, HitStrand::Plus));
    }
    for start in minus {
        candidates.push((start, HitStrand::Minus));
    }
    // Rule 11: hits are ordered by start ascending. §4.4 adds "then plus
    // before minus", and that tie-break can never fire: one pattern has one
    // length, so a start carries at most one plus hit and at most one minus
    // hit, and where both match the span merges into a single `Both` hit just
    // below — no two hits in the returned list share a start.
    candidates.sort_by_key(|candidate| candidate.0);

    let mut hits: Vec<DnaHit> = Vec::new();
    for (start, strand) in candidates {
        match hits.last_mut() {
            // Rule 5 (D052, M8-8): a span found on both strands is one hit with
            // strand `Both`, never two. The two candidates for a start are
            // adjacent because the list is ordered by start.
            Some(previous) if previous.start == start => {
                if previous.strand != strand {
                    previous.strand = HitStrand::Both;
                }
            }
            _ => hits.push(hit_at(start, width, size, strand)),
        }
    }
    Ok(hits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCategory;
    use crate::genbank::read_genbank;
    use crate::identity::Identifier;
    use crate::sequence::normalize;
    use crate::sequence::DNA_ALPHABET;

    /// A record made in the test: text plus the topology its caller states
    /// (D049). No reader and no fixture file is involved.
    fn dna(text: &str, topology: SequenceTopology) -> Sequence {
        let name: Identifier = Identifier::create("unit".to_string())
            .unwrap_or_else(|error| panic!("cannot name a test record: {:?}", error));
        normalize(name, text.to_string(), topology)
            .unwrap_or_else(|error| panic!("{:?} cannot be stored as DNA: {:?}", text, error))
    }

    /// pBR322 as the fixtures hold it, read the way the view reads it:
    /// `tests/fixtures/genbank/raw/J01749.1.gb`, 4361 bp, circular by its own
    /// LOCUS line (D036). This is the record D052's landmark claims are about;
    /// the expectations themselves stay in this file. The path is relative to
    /// the package root, which is where cargo runs unit tests from.
    fn pbr322() -> Sequence {
        let path: &str = "../../tests/fixtures/genbank/raw/J01749.1.gb";
        let text: String = std::fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
        let name: Identifier = Identifier::create("J01749.1".to_string())
            .unwrap_or_else(|error| panic!("cannot name pBR322: {:?}", error));
        let record = read_genbank(name, text)
            .unwrap_or_else(|error| panic!("read_genbank refused {}: {:?}", path, error.message()));
        record.sequence()
    }

    /// An expected hit, written out by hand.
    fn want(strand: HitStrand, start: u64, length: u64, wraps_origin: bool) -> DnaHit {
        DnaHit {
            start,
            length,
            strand,
            wraps_origin,
        }
    }

    /// Hits as one short line each, so a failure message says which field of
    /// which hit differs, and a count is readable.
    fn row(hit: &DnaHit) -> String {
        format!(
            "{}@{}+{}{}",
            hit.strand.symbol(),
            hit.start,
            hit.length,
            if hit.wraps_origin { " wraps" } else { "" }
        )
    }

    fn rows(hits: &[DnaHit]) -> Vec<String> {
        hits.iter().map(row).collect()
    }

    /// The hits of a pattern core is expected to search, not refuse.
    fn hits_of(sequence: &Sequence, pattern: &str) -> Vec<DnaHit> {
        match find_dna(sequence, pattern) {
            Ok(hits) => hits,
            Err(error) => panic!(
                "find_dna refused {:?} ({}: {}), and the test expects it to be searched",
                pattern,
                error.code(),
                error.message()
            ),
        }
    }

    /// The hits of a pattern that must match nothing — while still being
    /// searched. "Finds nothing" and "is refused" are different answers
    /// (rules 7 and 12), and collapsing them is how a missing rule hides.
    fn finds_nothing(sequence: &Sequence, pattern: &str) {
        match find_dna(sequence, pattern) {
            Ok(hits) => assert!(
                hits.is_empty(),
                "{:?} gives {:?}, expected no hits",
                pattern,
                rows(&hits)
            ),
            Err(error) => panic!(
                "{:?} is refused ({}: {}) where §4.4 answers it with no hits",
                pattern,
                error.code(),
                error.message()
            ),
        }
    }

    /// Hits for a pattern whose whole point is the **fold**: if the fold goes
    /// missing, the pattern turns into an illegal letter, and contract §5's
    /// G6/G7 require that to show up as a missing hit rather than as a refusal
    /// travelling down an error path. The refusals themselves are pinned by
    /// `empty_pattern_is_refused` and `illegal_pattern_letter_names_the_letter`.
    fn folded_rows(sequence: &Sequence, pattern: &str) -> Vec<String> {
        match find_dna(sequence, pattern) {
            Ok(hits) => rows(&hits),
            Err(_) => Vec::new(),
        }
    }

    /// Rule 1 (D052 M8-7): containment, not intersection.
    #[test]
    fn covers_uses_containment_not_intersection() {
        let alphabet: Vec<char> = DNA_ALPHABET.chars().collect();
        // A record `A` is matched by exactly the pattern letters whose set
        // holds `A`, in alphabet order (the same list `search-letters.tsv`
        // and `tests/fixtures/view/README.md` state).
        let by_a: Vec<char> = alphabet
            .iter()
            .cloned()
            .filter(|pattern| covers('A', *pattern))
            .collect();
        assert!(
            by_a == "ARWMDHVN".chars().collect::<Vec<char>>(),
            "a record `A` is matched by {:?}, expected `A R W M D H V N`",
            by_a
        );
        // A record `R` (`A`,`G`) by exactly `R D V N`, and a record `N` by `N`.
        let by_r: Vec<char> = alphabet
            .iter()
            .cloned()
            .filter(|pattern| covers('R', *pattern))
            .collect();
        assert!(
            by_r == "RDVN".chars().collect::<Vec<char>>(),
            "a record `R` is matched by {:?}, expected `R D V N`",
            by_r
        );
        let by_n: Vec<char> = alphabet
            .iter()
            .cloned()
            .filter(|pattern| covers('N', *pattern))
            .collect();
        assert!(
            by_n == vec!['N'],
            "a record `N` is matched by {:?}, expected `N` alone",
            by_n
        );
        // The pairs that tell the two rules apart.
        assert!(covers('G', 'R'), "a record `G` is inside pattern `R`");
        assert!(
            !covers('R', 'A'),
            "record `R` (A,G) is not inside pattern `A`; intersection would say yes"
        );
        assert!(
            !covers('N', 'A'),
            "record `N` is not inside pattern `A`; that is the answer D052 M8-7 pins"
        );
        assert!(
            !covers('S', 'R'),
            "record `S` (C,G) and pattern `R` (A,G) share `G` and still do not match"
        );
        assert!(covers('A', 'N'), "any record letter is inside pattern `N`");
        assert!(
            !covers('C', 'G'),
            "containment is not set equality either: `C` does not match pattern `G`"
        );
    }

    /// Rule 2: `U` is read as `T` in a pattern, which a stored record may not hold.
    #[test]
    fn gaattuc_finds_the_ecori_site() {
        let sequence: Sequence = pbr322();
        assert!(
            sequence.length() == 4361 && sequence.topology() == SequenceTopology::Circular,
            "pBR322 as read here is {} bp {} ({}), expected 4361 bp circular",
            sequence.length(),
            sequence.topology().name(),
            sequence.display_name()
        );
        let expected: Vec<String> = rows(&[want(HitStrand::Both, 4358, 6, true)]);
        let found: Vec<String> = folded_rows(&sequence, "GAATUC");
        assert!(
            found == expected,
            "GAATUC on pBR322 gives {:?}, expected the one EcoRI hit at zero-based 4358 {:?}",
            found,
            expected
        );
    }

    /// Rule 2: case is dropped before the pattern is searched.
    #[test]
    fn lowercase_pattern_finds_the_same_hit() {
        let sequence: Sequence = pbr322();
        let upper: Vec<String> = folded_rows(&sequence, "GAATTC");
        let lower: Vec<String> = folded_rows(&sequence, "gaattc");
        assert!(
            upper == rows(&[want(HitStrand::Both, 4358, 6, true)]),
            "GAATTC on pBR322 gives {:?}, expected one hit at 4358 that wraps",
            upper
        );
        assert!(
            lower == upper,
            "gaattc gives {:?}, the same pattern upper case gives {:?}",
            lower,
            upper
        );
    }

    /// Rule 3: an empty pattern is refused.
    #[test]
    fn empty_pattern_is_refused() {
        let outcome: Result<Vec<DnaHit>, DomainError> =
            find_dna(&dna("ATGATGAATTC", SequenceTopology::Linear), "");
        let error: DomainError = match outcome {
            Ok(hits) => panic!(
                "an empty pattern gives {} hits ({:?}); §4.4 refuses it",
                hits.len(),
                rows(&hits)
            ),
            Err(error) => error,
        };
        assert!(
            error == DomainError::EmptySearchPattern,
            "the refusal is {:?}, expected EmptySearchPattern",
            error
        );
        assert!(
            error.category() == ErrorCategory::MalformedInput,
            "EmptySearchPattern is in category {:?}",
            error.category()
        );
        assert!(
            error.code() == "empty_search_pattern",
            "the code is {}, expected empty_search_pattern",
            error.code()
        );
        assert!(
            !error.message().is_empty(),
            "the refusal carries no message"
        );
    }

    /// Rule 3: an illegal letter is named, with its position, after folding —
    /// so the lowercase here is what makes the check non-trivial: validating
    /// before folding would report the `g` at position 0 instead.
    #[test]
    fn illegal_pattern_letter_names_the_letter() {
        let sequence: Sequence = dna("GAATTCATG", SequenceTopology::Linear);
        let pattern: &str = "gaatxc";
        let outcome: Result<Vec<DnaHit>, DomainError> = find_dna(&sequence, pattern);
        let error: DomainError = match outcome {
            Ok(hits) => panic!(
                "{:?} gives {} hits ({:?}); §4.4 refuses `x`",
                pattern,
                hits.len(),
                rows(&hits)
            ),
            Err(error) => error,
        };
        match &error {
            DomainError::InvalidSearchPatternCharacters(problem) => {
                assert!(
                    problem.pattern == pattern,
                    "the refusal names pattern {:?}, expected {:?} as written",
                    problem.pattern,
                    pattern
                );
                assert!(
                    problem.position == 4,
                    "the refusal points at zero-based {}, expected 4, where the `x` of the pattern sits: {:?}",
                    problem.position,
                    problem
                );
                assert!(
                    problem.characters == "x",
                    "the refusal names {:?}, expected the letter as written, \"x\"",
                    problem.characters
                );
            }
            other => panic!(
                "the refusal is {:?} ({}), expected InvalidSearchPatternCharacters",
                other,
                other.code()
            ),
        }
        assert!(
            error.code() == "invalid_search_pattern_characters",
            "the code is {}, expected invalid_search_pattern_characters",
            error.code()
        );
        assert!(
            error.category() == ErrorCategory::MalformedInput,
            "InvalidSearchPatternCharacters is in category {:?}",
            error.category()
        );
        let message: String = error.message();
        assert!(
            message.contains("'x'"),
            "the message {:?} does not name the offending letter in its own quotes, which §4.4 requires: naming the pattern is not enough",
            message
        );
        assert!(
            message.contains("position 4"),
            "the message {:?} does not give the letter's zero-based position, which §4.4 requires",
            message
        );
        assert!(
            message.contains(pattern),
            "the message {:?} does not name the pattern it refused",
            message
        );
    }

    /// Rule 4: a minus hit is the reverse complement's plus hit, at the same
    /// top-strand coordinates.
    ///
    /// `GACCTTTTAGGTC` holds `GACCT` at 0 and `AGGTC` at 8, and each is the
    /// other's reverse complement; every base is concrete, so a match is an
    /// exact substring and there is nothing else to find.
    #[test]
    fn minus_hit_of_a_constructed_pattern_sits_at_the_same_start_as_its_reverse_complement_plus_hit(
    ) {
        let sequence: Sequence = dna("GACCTTTTAGGTC", SequenceTopology::Linear);
        let pattern_hits: Vec<DnaHit> = hits_of(&sequence, "AGGTC");
        assert!(
            pattern_hits
                == vec![
                    want(HitStrand::Minus, 0, 5, false),
                    want(HitStrand::Plus, 8, 5, false)
                ],
            "AGGTC gives {:?}, expected a minus hit at 0 and a plus hit at 8",
            rows(&pattern_hits)
        );
        let complement_hits: Vec<DnaHit> = hits_of(&sequence, "GACCT");
        assert!(
            complement_hits
                == vec![
                    want(HitStrand::Plus, 0, 5, false),
                    want(HitStrand::Minus, 8, 5, false)
                ],
            "GACCT gives {:?}, expected a plus hit at 0 and a minus hit at 8",
            rows(&complement_hits)
        );
        let first: &DnaHit = &pattern_hits[0];
        let other: &DnaHit = &complement_hits[0];
        assert!(
            first.start == other.start
                && first.length == other.length
                && first.wraps_origin == other.wraps_origin,
            "the minus hit of AGGTC covers [{},{}) while the plus hit of GACCT covers [{},{})",
            first.start,
            first.start + first.length,
            other.start,
            other.start + other.length
        );
    }

    /// Rule 5 (D052 M8-8): a palindromic span is one `both` hit.
    #[test]
    fn a_palindromic_site_is_one_both_hit() {
        // `GAATTC` is its own reverse complement, written out by hand: it is
        // the EcoRI site, and D052 says a site like this is reported once.
        let hits: Vec<DnaHit> = hits_of(&dna("ATCGAATTCAT", SequenceTopology::Linear), "GAATTC");
        assert!(
            hits == vec![want(HitStrand::Both, 3, 6, false)],
            "GAATTC on linear ATCGAATTCAT gives {:?}, expected exactly one `both` hit at 3",
            rows(&hits)
        );
    }

    /// Rule 5 on the record D052 names: pBR322's four landmark sites occur once
    /// each, and EcoRI's is reachable only by wrapping.
    #[test]
    fn pbr322_gaattc_is_one_hit_not_two() {
        let hits: Vec<DnaHit> = hits_of(&pbr322(), "GAATTC");
        assert!(
            hits.len() == 1,
            "GAATTC on pBR322 gives {} hits ({:?}), expected the one EcoRI site D052 names",
            hits.len(),
            rows(&hits)
        );
        assert!(
            hits == vec![want(HitStrand::Both, 4358, 6, true)],
            "GAATTC on pBR322 gives {:?}, expected one `both` hit at 4358 that wraps the origin",
            rows(&hits)
        );
    }

    /// Rule 6: on a circular record a span may start in the last `width - 1`
    /// bases, and `wraps_origin` is computed, not inferred from a doubled string.
    #[test]
    fn circular_site_spanning_the_origin_is_one_wrapping_hit() {
        // `TTCAAGAA`: its last three bases `GAA` followed by its first three
        // `TTC` spell the EcoRI site across the origin, the same shape as
        // pBR322's site at 4358. 5 + 6 > 8 is what makes it wrap.
        let hits: Vec<DnaHit> = hits_of(&dna("TTCAAGAA", SequenceTopology::Circular), "GAATTC");
        assert!(
            hits == vec![want(HitStrand::Both, 5, 6, true)],
            "GAATTC on circular TTCAAGAA gives {:?}, expected one `both` hit at 5 that wraps",
            rows(&hits)
        );
    }

    /// Rule 6, the other side: nothing wraps on a linear record.
    #[test]
    fn the_same_pattern_on_a_linear_record_finds_nothing_there() {
        let circular: Vec<DnaHit> = hits_of(&dna("TTCAAGAA", SequenceTopology::Circular), "GAATTC");
        assert!(
            circular.len() == 1,
            "the circular record is expected to hold the spanning site first: {:?}",
            rows(&circular)
        );
        let linear: Vec<DnaHit> = hits_of(&dna("TTCAAGAA", SequenceTopology::Linear), "GAATTC");
        assert!(
            linear.is_empty(),
            "GAATTC on linear TTCAAGAA gives {:?}; a linear record offers only starts 0..=2 and never wraps",
            rows(&linear)
        );
    }

    /// Rule 7 (D050): a pattern longer than the record finds nothing, and that
    /// is not an error.
    #[test]
    fn pattern_longer_than_the_record_finds_nothing() {
        for topology in [SequenceTopology::Circular, SequenceTopology::Linear] {
            let sequence: Sequence = dna("TTCAAGAA", topology.clone());
            // The record does hold a 6-base site at 5..11 on a circular
            // record, so the zero answer below is about length and not about a
            // pattern that never occurs anywhere.
            if topology == SequenceTopology::Circular {
                assert!(
                    hits_of(&sequence, "GAATTC").len() == 1,
                    "the 6-base site should be there to begin with"
                );
            }
            finds_nothing(&sequence, "GAATTCGAAT");
            finds_nothing(&sequence, "GAATTCGAATTCGAATT");
        }
    }

    /// Rule 8: a pattern as long as the record hits at the rotation it matches,
    /// and any such start above 0 wraps.
    #[test]
    fn equal_length_pattern_hits_at_its_rotation_and_wraps() {
        let sequence: Sequence = dna("ACGTAG", SequenceTopology::Circular);
        let rotated: Vec<DnaHit> = hits_of(&sequence, "GTAGAC");
        assert!(
            rotated == vec![want(HitStrand::Plus, 2, 6, true)],
            "GTAGAC on circular ACGTAG gives {:?}, expected one hit at rotation 2, which wraps",
            rows(&rotated)
        );
        let exact: Vec<DnaHit> = hits_of(&sequence, "ACGTAG");
        assert!(
            exact == vec![want(HitStrand::Plus, 0, 6, false)],
            "ACGTAG on circular ACGTAG gives {:?}, expected one hit at 0, which does not wrap",
            rows(&exact)
        );
    }

    /// Rule 8 on a linear record: the only legal start is 0.
    #[test]
    fn equal_length_pattern_on_a_linear_record_hits_only_at_zero() {
        let sequence: Sequence = dna("ACGTAG", SequenceTopology::Linear);
        let exact: Vec<DnaHit> = hits_of(&sequence, "ACGTAG");
        assert!(
            exact == vec![want(HitStrand::Plus, 0, 6, false)],
            "ACGTAG on linear ACGTAG gives {:?}, expected one hit at 0",
            rows(&exact)
        );
        finds_nothing(&sequence, "GTAGAC");
    }

    /// Rule 9 (D050 M8-16): every match position, overlaps included.
    #[test]
    fn aa_on_aaa_is_three_plus_hits() {
        // No `T` anywhere, so `AA`'s reverse complement `TT` matches nothing
        // and all three hits are `+`.
        let circular: Vec<DnaHit> = hits_of(&dna("AAA", SequenceTopology::Circular), "AA");
        assert!(
            circular
                == vec![
                    want(HitStrand::Plus, 0, 2, false),
                    want(HitStrand::Plus, 1, 2, false),
                    want(HitStrand::Plus, 2, 2, true)
                ],
            "AA on circular AAA gives {:?}, expected three overlapping plus hits",
            rows(&circular)
        );
        let linear: Vec<DnaHit> = hits_of(&dna("AAA", SequenceTopology::Linear), "AA");
        assert!(
            linear
                == vec![
                    want(HitStrand::Plus, 0, 2, false),
                    want(HitStrand::Plus, 1, 2, false)
                ],
            "AA on linear AAA gives {:?}, expected two hits: 2 is not a legal start",
            rows(&linear)
        );
    }

    /// Rule 10: there is no cap in core.
    #[test]
    fn pattern_n_on_pbr322_returns_every_base() {
        let sequence: Sequence = pbr322();
        let hits: Vec<DnaHit> = hits_of(&sequence, "N");
        let first: Vec<String> = hits.iter().take(6).map(row).collect();
        assert!(
            hits.len() as u64 == sequence.length(),
            "N on pBR322 gives {} hits, expected one per base ({}): the first are {:?}",
            hits.len(),
            sequence.length(),
            first
        );
        let expected: Vec<DnaHit> = (0..sequence.length())
            .map(|start| want(HitStrand::Both, start, 1, false))
            .collect();
        assert!(
            hits == expected,
            "N on pBR322 does not give one `both` hit of length 1 at each of 0..={}: the first rows are {:?}",
            sequence.length() - 1,
            first
        );
    }

    /// Rule 11: ordered by start. §4.4's "then plus before minus" tie-break
    /// cannot fire — one pattern has one length, so at most one plus and one
    /// minus hit share a start, and where both match the span merges into a
    /// single `Both` hit.
    #[test]
    fn hits_are_ordered_by_start() {
        let sequence: Sequence = dna("GACCTTTTAGGTC", SequenceTopology::Linear);
        let hits: Vec<DnaHit> = hits_of(&sequence, "AGGTC");
        let starts: Vec<u64> = hits.iter().map(|hit| hit.start).collect();
        let mut sorted: Vec<u64> = starts.clone();
        sorted.sort();
        assert!(
            starts == sorted,
            "AGGTC hits come out with starts {:?}, which is not ascending",
            starts
        );
        assert!(
            starts == vec![0, 8],
            "AGGTC gives {:?}, expected the minus site at 0 before the plus site at 8: order is by start, not by strand",
            rows(&hits)
        );
    }

    /// Rule 12 (D052 M8-7): an ambiguity letter in the **record** is matched
    /// only by containment.
    #[test]
    fn aar_does_not_match_a_run_of_a_a_n() {
        let sequence: Sequence = dna("CGTGAANCGT", SequenceTopology::Linear);
        assert!(
            sequence.letter_at(4) == Ok('A')
                && sequence.letter_at(5) == Ok('A')
                && sequence.letter_at(6) == Ok('N'),
            "the test record must hold `A A N` at 4, 5 and 6"
        );
        finds_nothing(&sequence, "AAR");
        let hits: Vec<DnaHit> = hits_of(&sequence, "AAN");
        assert!(
            hits == vec![want(HitStrand::Plus, 4, 3, false)],
            "AAN on the same record gives {:?}, expected one hit at 4",
            rows(&hits)
        );
    }

    /// A letter in the failure messages of `the_masks_agree_with_the_base_sets`.
    fn quote(letter: char) -> String {
        format!("`{}`", letter)
    }

    fn yes(answer: bool) -> &'static str {
        if answer {
            "matches"
        } else {
            "does not match"
        }
    }

    /// §5 test 2: the masks `covers` compares are the base sets
    /// `sequence::bases_of` gives, not a second mapping of the alphabet. The
    /// containment below is written over `bases_of`'s own lists of bases,
    /// independently of the mask route `covers` takes, and every pair of the 15
    /// IUPAC codes — plus `X` and `U` on either side — is compared.
    #[test]
    fn the_masks_agree_with_the_base_sets() {
        let alphabet: Vec<char> = DNA_ALPHABET.chars().collect();
        assert!(
            alphabet.len() == 15,
            "DNA_ALPHABET holds {} letters, expected the 15 IUPAC codes",
            alphabet.len()
        );
        // `X` is a letter outside the alphabet outright, and `U` is a letter a
        // **record** may not hold (`normalize` refuses it, and a pattern folds
        // it to `T`). Neither has a base set; both go on either side of a pair.
        let outside: Vec<char> = vec!['X', 'U'];
        let letters: Vec<char> = alphabet
            .iter()
            .cloned()
            .chain(outside.iter().cloned())
            .collect();

        // Containment over the base sets, written out from `bases_of`: every
        // base the template letter stands for is one the pattern letter stands
        // for. A letter with no set matches nothing, and an empty set matches
        // nothing rather than everything — which is what `t != 0` says in
        // `covers`.
        let contains = |template: char, pattern: char| -> bool {
            let template_bases: Vec<char> = match bases_of(template) {
                Some(bases) => bases,
                None => return false,
            };
            let pattern_bases: Vec<char> = match bases_of(pattern) {
                Some(bases) => bases,
                None => return false,
            };
            !template_bases.is_empty()
                && template_bases
                    .iter()
                    .all(|base| pattern_bases.contains(base))
        };

        let mut compared: u64 = 0;
        for template in letters.iter() {
            for pattern in letters.iter() {
                let via_masks: bool = covers(*template, *pattern);
                let via_sets: bool = contains(*template, *pattern);
                assert!(
                    via_masks == via_sets,
                    "covers({}, {}) answers {}, while containment over bases_of's sets answers {}",
                    quote(*template),
                    quote(*pattern),
                    yes(via_masks),
                    yes(via_sets),
                );
                compared += 1;
            }
        }
        // Every pair was compared, so the loop bounds are the ones the test
        // names: 15 IUPAC codes and the two letters outside them, both ways.
        assert!(
            compared == 17 * 17,
            "{} letter pairs compared, expected 17 * 17 (15 IUPAC codes plus X and U either way)",
            compared
        );
    }

    /// §5 test 4, `#[ignore]`d: a 6-base pattern over a 10 Mb record is scanned
    /// by reading the record's own letters, not by building a `Vec` or a
    /// `String` per start. Timed on the release profile, and on its own
    /// (contract 8r2 §6):
    ///
    /// ```text
    /// cargo test -p luria-core --release -- --ignored a_search_of_a_long_record_is_quick
    /// ```
    #[test]
    #[ignore]
    fn a_search_of_a_long_record_is_quick() {
        let text: String = "ACGT".repeat(2_500_000);
        let sequence: Sequence = dna(text.as_str(), SequenceTopology::Linear);
        assert!(
            sequence.length() == 10_000_000,
            "the test record is {} bases, expected 10 000 000, the ceiling `MAX_RECORD_BASES` allows (D048, D056)",
            sequence.length()
        );
        // The record is built outside the timed part; only the search is timed.
        let began: std::time::Instant = std::time::Instant::now();
        let hits: Vec<DnaHit> = hits_of(&sequence, "GAATTC");
        let elapsed: u128 = began.elapsed().as_millis();
        // `ACGT` repeated holds no `GAATTC`, so the answer is empty while the
        // scan is asked at every start on both strands: the whole record is
        // read twice over, and the timing measures that work.
        assert!(
            hits.is_empty(),
            "GAATTC in `ACGT` repeated gives {} hits, expected none",
            hits.len()
        );
        assert!(
            elapsed < 500,
            "GAATTC over a 10 Mb record took {} ms: the scan reads the record's letters by slice, and the search must stay under 500 ms in --release",
            elapsed
        );
    }
}
