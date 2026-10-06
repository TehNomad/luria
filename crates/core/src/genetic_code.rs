//! The genetic code.
//!
//! Only the standard genetic code is supported. The table below is the
//! published NCBI/EMBL standard code, written as the 64 residues in codon
//! order, where each codon position runs over the bases in the order `T, C,
//! A, G`. The index of a codon is therefore:
//!
//! ```text
//! 16 * index(first) + 4 * index(second) + index(third)
//! ```
//!
//! A codon whose letters are DNA IUPAC symbols but not plain bases is
//! *ambiguous*: translating it would require guessing which base was meant,
//! so it is reported rather than resolved. For example `NGG` stands for the
//! four codons `AGG`, `CGG`, `GGG`, `TGG`, which give `R`, `R`, `G`, `W`;
//! there is no single residue for it.
//!
//! A codon lookup never returns a placeholder residue. `lookup` returns a
//! `CodonOutcome`, which distinguishes a residue from a stop, an ambiguous
//! codon, and an invalid codon; only the residue branch carries a `Residue`.

use crate::sequence::is_dna_letter;

/// The standard genetic code, 64 residues in codon order.
pub const STANDARD_GENETIC_CODE: &str =
    "FFLLSSSSYY**CC*WLLLLPPPPHHQQRRRRIIIMTTTTNNKKSSRRVVVVAAAADDEEGGGG";

/// Base order used by the table above.
pub const TRIPLET_BASES: &str = "TCAG";

/// The only genetic code this milestone supports.
pub const STANDARD_CODE_NAME: &str = "standard";

/// A residue of the standard genetic code.
#[derive(Clone, Debug, PartialEq)]
pub enum Residue {
    A,
    R,
    N,
    D,
    C,
    Q,
    E,
    G,
    H,
    I,
    L,
    T,
    F,
    P,
    S,
    W,
    V,
    K,
    M,
    Y,
}

impl Residue {
    /// The conventional one-letter symbol.
    pub fn symbol(&self) -> String {
        match self {
            Residue::A => "A".to_string(),
            Residue::R => "R".to_string(),
            Residue::N => "N".to_string(),
            Residue::D => "D".to_string(),
            Residue::C => "C".to_string(),
            Residue::Q => "Q".to_string(),
            Residue::E => "E".to_string(),
            Residue::G => "G".to_string(),
            Residue::H => "H".to_string(),
            Residue::I => "I".to_string(),
            Residue::L => "L".to_string(),
            Residue::T => "T".to_string(),
            Residue::F => "F".to_string(),
            Residue::P => "P".to_string(),
            Residue::S => "S".to_string(),
            Residue::W => "W".to_string(),
            Residue::V => "V".to_string(),
            Residue::K => "K".to_string(),
            Residue::M => "M".to_string(),
            Residue::Y => "Y".to_string(),
        }
    }
}

/// The genetic code in use. Only supported codes are representable, so an
/// unsupported request never survives into a `Cds` (see `cds.rs::create`).
#[derive(Clone, Debug, PartialEq)]
pub enum GeneticCode {
    Standard,
}

/// The outcome of looking a codon up in the genetic code.
#[derive(Clone, Debug, PartialEq)]
pub enum CodonOutcome {
    Residue(Residue),
    Stop,
    Ambiguous,
    Invalid,
}

impl CodonOutcome {
    /// True for the three stop codons of the standard code.
    pub fn is_stop(&self) -> bool {
        match self {
            CodonOutcome::Stop => true,
            CodonOutcome::Residue(_residue) => false,
            CodonOutcome::Ambiguous => false,
            CodonOutcome::Invalid => false,
        }
    }

    /// True when the codon uses IUPAC symbols that stand for several bases.
    pub fn is_ambiguous(&self) -> bool {
        match self {
            CodonOutcome::Ambiguous => true,
            CodonOutcome::Residue(_residue) => false,
            CodonOutcome::Stop => false,
            CodonOutcome::Invalid => false,
        }
    }

    /// True when the codon contains something outside the DNA alphabet.
    pub fn is_invalid(&self) -> bool {
        match self {
            CodonOutcome::Invalid => true,
            CodonOutcome::Residue(_residue) => false,
            CodonOutcome::Stop => false,
            CodonOutcome::Ambiguous => false,
        }
    }
}

/// Index of a base in the table's base order, or the length of that order
/// when the letter is not a plain base.
pub fn base_place(ch: char) -> usize {
    let bases: Vec<char> = TRIPLET_BASES.chars().collect();
    let count: usize = bases.len();
    let mut place: usize = 0;
    while place < count {
        if bases[place] == ch {
            return place;
        }
        place += 1;
    }
    count
}

/// Residue for a table symbol, or `None` for anything that is not a residue.
fn residue_for_symbol(symbol: char) -> Option<Residue> {
    match symbol {
        'A' => Some(Residue::A),
        'R' => Some(Residue::R),
        'N' => Some(Residue::N),
        'D' => Some(Residue::D),
        'C' => Some(Residue::C),
        'Q' => Some(Residue::Q),
        'E' => Some(Residue::E),
        'G' => Some(Residue::G),
        'H' => Some(Residue::H),
        'I' => Some(Residue::I),
        'L' => Some(Residue::L),
        'T' => Some(Residue::T),
        'F' => Some(Residue::F),
        'P' => Some(Residue::P),
        'S' => Some(Residue::S),
        'W' => Some(Residue::W),
        'V' => Some(Residue::V),
        'K' => Some(Residue::K),
        'M' => Some(Residue::M),
        'Y' => Some(Residue::Y),
        _ => None,
    }
}

/// Look a codon up in the standard genetic code.
pub fn lookup(codon: String) -> CodonOutcome {
    if codon.len() != 3 {
        return CodonOutcome::Invalid;
    }

    let letters: Vec<char> = codon.chars().collect();
    for ch in letters.iter() {
        if !is_dna_letter(*ch) {
            return CodonOutcome::Invalid;
        }
    }

    let mut index: usize = 0;
    for ch in letters.iter() {
        let place: usize = base_place(*ch);
        if place >= TRIPLET_BASES.len() {
            return CodonOutcome::Ambiguous;
        }
        index = index * 4 + place;
    }

    let table: Vec<char> = STANDARD_GENETIC_CODE.chars().collect();
    let symbol: char = table[index];
    if symbol == '*' {
        return CodonOutcome::Stop;
    }
    let found: Option<Residue> = residue_for_symbol(symbol);
    if found.is_none() {
        return CodonOutcome::Invalid;
    }
    CodonOutcome::Residue(found.unwrap())
}

/// True when the named genetic code is one this milestone supports.
pub fn is_supported(name: String) -> bool {
    name == STANDARD_CODE_NAME
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sequence::is_dna_letter;
    use crate::sequence::DNA_ALPHABET;

    #[test]
    fn the_table_has_64_entries() {
        assert!(STANDARD_GENETIC_CODE.len() == 64);
    }

    #[test]
    fn known_codons_translate_to_published_residues() {
        assert!(lookup("TGG".to_string()) == CodonOutcome::Residue(Residue::W));
        assert!(lookup("TTT".to_string()) == CodonOutcome::Residue(Residue::F));
        assert!(lookup("ATT".to_string()) == CodonOutcome::Residue(Residue::I));
        assert!(lookup("ATG".to_string()) == CodonOutcome::Residue(Residue::M));
        assert!(lookup("AAA".to_string()) == CodonOutcome::Residue(Residue::K));
        assert!(lookup("AAT".to_string()) == CodonOutcome::Residue(Residue::N));
        assert!(lookup("CGT".to_string()) == CodonOutcome::Residue(Residue::R));
        assert!(lookup("TCA".to_string()) == CodonOutcome::Residue(Residue::S));
        assert!(lookup("GAA".to_string()) == CodonOutcome::Residue(Residue::E));
        assert!(lookup("GCT".to_string()) == CodonOutcome::Residue(Residue::A));
    }

    #[test]
    fn residue_symbols_round_trip() {
        let residues: Vec<Residue> =
            vec![Residue::M, Residue::A, Residue::W, Residue::N, Residue::F];
        let mut text: String = String::new();
        for residue in residues.iter() {
            text += residue.symbol().as_str();
        }
        assert!(text == "MAWNF");
    }

    #[test]
    fn the_three_stop_codons_are_recognised() {
        assert!(lookup("TAA".to_string()) == CodonOutcome::Stop);
        assert!(lookup("TAG".to_string()) == CodonOutcome::Stop);
        assert!(lookup("TGA".to_string()) == CodonOutcome::Stop);
        assert!(lookup("TGG".to_string()) == CodonOutcome::Residue(Residue::W));
    }

    // NGG expands to AGG, CGG, GGG, TGG -> R, R, G, W: no single residue.
    #[test]
    fn ambiguous_codons_are_reported_not_resolved() {
        assert!(lookup("NGG".to_string()) == CodonOutcome::Ambiguous);
        assert!(lookup("ANM".to_string()) == CodonOutcome::Ambiguous);
        assert!(lookup("NGG".to_string()).is_ambiguous());
    }

    #[test]
    fn letters_outside_the_alphabet_are_invalid() {
        assert!(lookup("UUA".to_string()) == CodonOutcome::Invalid);
        assert!(lookup("A#A".to_string()) == CodonOutcome::Invalid);
        assert!(lookup("GG".to_string()) == CodonOutcome::Invalid);
    }

    // There is no residue for an ambiguous or invalid codon: the residue
    // branch is the only one that carries a value, and it is never a
    // placeholder such as "?".
    #[test]
    fn only_residue_carrying_outcomes_have_a_symbol() {
        let ambiguous: CodonOutcome = lookup("NGG".to_string());
        let invalid: CodonOutcome = lookup("UUA".to_string());
        let outcomes: Vec<CodonOutcome> = vec![ambiguous, invalid];
        for outcome in outcomes.iter() {
            assert!(outcome.is_ambiguous() || outcome.is_invalid());
        }
    }

    #[test]
    fn only_the_standard_code_is_supported() {
        assert!(is_supported("standard".to_string()));
        assert!(!is_supported("iso1557".to_string()));
        assert!(!is_supported("standard ".to_string()));
        assert!(!is_supported("STANDARD".to_string()));
        assert!(!is_supported("iso-1557-1".to_string()));
    }

    #[test]
    fn dna_alphabet_is_shared_with_sequences() {
        assert!(DNA_ALPHABET.len() == 13);
        assert!(is_dna_letter('V'));
    }
}
