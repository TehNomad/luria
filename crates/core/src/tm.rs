//! Melting temperature by nearest-neighbour thermodynamics.
//!
//! The model is published, and this module names it rather than describing it
//! as "the" Tm (D014: a calculated thermodynamic value identifies its method):
//!
//! * **Watson–Crick and initiation parameters**: SantaLucia, J. Jr & Hicks, D.
//!   (2004), *Annu. Rev. Biophys. Biomol. Struct.* **33**:415–440, Table 1.
//! * **Single-mismatch parameters**: Allawi & SantaLucia (1997, 1998 ×3) and
//!   Peyret *et al.* (1999), as tabulated in the same review's Table 2.
//! * **Salt correction**: Owczarzy, R. *et al.* (2004), *Biochemistry*
//!   **43**:3537–3554, the monovalent branch.
//!
//! Both tables are reproduced from `tests/fixtures/tm/`, where every value
//! records its source and its verification; `tests/tm_tables.rs` checks that
//! the constants below still equal that fixture, so the fixture is the
//! authority and this file is a copy of it (D018).
//!
//! # What is computed
//!
//! For a primer `P` (5′→3′) annealing over a template site `W` (the same
//! strand, same span), the duplex is `P` against the complement of `W`:
//!
//! ```text
//!   5'-P[0] P[1] P[2] ...-3'
//!   3'-W'[0] W'[1] W'[2]...-5'      W'[i] = complement(W[i])
//! ```
//!
//! Where `P[i] == W[i]` that is a Watson–Crick pair; where they differ it is a
//! mismatch. Passing `W = P` therefore gives the **matched** duplex, which is
//! the primer against the product it creates, and passing the wild-type site
//! gives the **mismatch-aware** duplex, which is the primer against the
//! unedited template in the first PCR cycle. `TmBasis` records which was asked
//! for, and both are legitimate quantities describing different cycles.
//!
//! # The tandem-mismatch limit
//!
//! A doublet containing *two* mismatched pairs has no published parameters, so
//! a mismatch-aware Tm is unavailable whenever an edit changes two adjacent
//! bases — 74 % of residue-changing codon substitutions. That is reported as
//! `TmParametersUnavailable` rather than approximated, and the caller falls
//! back to the matched basis and says so (owner, 2026-10-14;
//! `tests/fixtures/tm/README.md`, "The tandem-mismatch gap").
//!
//! # Deliberately absent
//!
//! No divalent-cation branch (the Q5 condition is 0 mM, and the 2008 decision
//! tree is a different model), no dangling ends, no terminal-mismatch table,
//! no hairpin or dimer term, and no bulge or loop term. Each would need its own
//! sourced parameters.

use crate::error::DomainError;
use crate::error::EmptySequenceProblem;
use crate::error::InvalidBufferModelProblem;
use crate::error::TmParametersUnavailableProblem;
use crate::sequence::is_dna_base;

/// Identity of this Tm code path (D021, D014).
pub const TM_FUNCTION_NAME: &str = "nn-santalucia-hicks-2004-owczarzy-2004-monovalent";

/// Version of this Tm code path (D021).
pub const TM_FUNCTION_VERSION: &str = "0.1.0";

/// Gas constant, cal/(K mol), as the sources use it.
const R: f64 = 1.987;

/// 37 °C in kelvin, for the ΔG°37 cross-check in the tests.
pub const REFERENCE_KELVIN: f64 = 310.15;

const ZERO_CELSIUS_IN_KELVIN: f64 = 273.15;

/// Watson–Crick doublets: (motif, ΔH kcal/mol, ΔS cal/(K mol)).
/// SantaLucia & Hicks (2004) Table 1, via `tests/fixtures/tm/nn-matched.tsv`.
pub const MATCHED: [(&str, f64, f64); 10] = [
    ("AA/TT", -7.6, -21.3),
    ("AT/TA", -7.2, -20.4),
    ("TA/AT", -7.2, -21.3),
    ("CA/GT", -8.5, -22.7),
    ("GT/CA", -8.4, -22.4),
    ("CT/GA", -7.8, -21.0),
    ("GA/CT", -8.2, -22.2),
    ("CG/GC", -10.6, -27.2),
    ("GC/CG", -9.8, -24.4),
    ("GG/CC", -8.0, -19.9),
];

/// Duplex initiation, applied once.
pub const INITIATION: (f64, f64) = (0.2, -5.7);

/// Applied once for each duplex end closed by an A–T pair.
pub const TERMINAL_AT: (f64, f64) = (2.2, 6.9);

/// Applied only to a self-complementary duplex.
pub const SYMMETRY: (f64, f64) = (0.0, -1.4);

/// Doublets with one internal mismatch, Watson–Crick pair first.
/// Via `tests/fixtures/tm/nn-mismatch.tsv`; 48 motifs, the complete set.
pub const MISMATCHED: [(&str, f64, f64); 48] = [
    ("AA/TA", 1.2, 1.7),
    ("AA/TC", 2.3, 4.6),
    ("AA/TG", -0.6, -2.3),
    ("AC/TA", 5.3, 14.6),
    ("AC/TC", 0.0, -4.4),
    ("AC/TT", 0.7, 0.2),
    ("AG/TA", -0.7, -2.3),
    ("AG/TG", -3.1, -9.5),
    ("AG/TT", 1.0, 0.9),
    ("AT/TC", -1.2, -6.2),
    ("AT/TG", -2.5, -8.3),
    ("AT/TT", -2.7, -10.8),
    ("CA/GA", -0.9, -4.2),
    ("CA/GC", 1.9, 3.7),
    ("CA/GG", -0.7, -2.3),
    ("CC/GA", 0.6, -0.6),
    ("CC/GC", -1.5, -7.2),
    ("CC/GT", -0.8, -4.5),
    ("CG/GA", -4.0, -13.2),
    ("CG/GG", -4.9, -15.3),
    ("CG/GT", -4.1, -11.7),
    ("CT/GC", -1.5, -6.1),
    ("CT/GG", -2.8, -8.0),
    ("CT/GT", -5.0, -15.8),
    ("GA/CA", -2.9, -9.8),
    ("GA/CC", 5.2, 14.2),
    ("GA/CG", -0.6, -1.0),
    ("GC/CA", -0.7, -3.8),
    ("GC/CC", 3.6, 8.9),
    ("GC/CT", 2.3, 5.4),
    ("GG/CA", 0.5, 3.2),
    ("GG/CG", -6.0, -15.8),
    ("GG/CT", 3.3, 10.4),
    ("GT/CC", 5.2, 13.5),
    ("GT/CG", -4.4, -12.3),
    ("GT/CT", -2.2, -8.4),
    ("TA/AA", 4.7, 12.9),
    ("TA/AC", 3.4, 8.0),
    ("TA/AG", 0.7, 0.7),
    ("TC/AA", 7.6, 20.2),
    ("TC/AC", 6.1, 16.4),
    ("TC/AT", 1.2, 0.7),
    ("TG/AA", 3.0, 7.4),
    ("TG/AG", 1.6, 3.6),
    ("TG/AT", -0.1, -1.7),
    ("TT/AC", 1.0, 0.7),
    ("TT/AG", -1.3, -5.3),
    ("TT/AT", 0.2, -1.5),
];

/// Which duplex a Tm describes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TmBasis {
    /// The primer against its own perfect complement: the product it creates,
    /// and the duplex of every cycle after the first.
    MatchedComplement,
    /// The primer against the unedited template, mismatched at the edit: the
    /// first cycle's duplex.
    MismatchedWildTypeSite,
}

impl TmBasis {
    pub fn name(&self) -> String {
        match self {
            TmBasis::MatchedComplement => "matched_complement".to_string(),
            TmBasis::MismatchedWildTypeSite => "mismatched_wild_type_site".to_string(),
        }
    }
}

/// The buffer a Tm assumes. Named by the caller, never defaulted: the same
/// reference tool uses 150 mM / 500 nM at its SDM call site and 50 mM / 200 nM
/// in another default set, so a silent default would be a decision the caller
/// did not make (`docs/q5-primer-design.md` §2.2).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BufferModel {
    monovalent_millimolar: f64,
    primer_nanomolar: f64,
    dmso_percent: f64,
}

impl BufferModel {
    /// The Q5 site-directed-mutagenesis condition recorded in
    /// `docs/research/nebasechanger-client-investigation.md` §4.2: 150 mM
    /// monovalent, 0 mM divalent, 500 nM primer, no DMSO.
    pub fn q5_reaction() -> BufferModel {
        BufferModel {
            monovalent_millimolar: 150.0,
            primer_nanomolar: 500.0,
            dmso_percent: 0.0,
        }
    }

    /// A stated buffer. Divalent cations are not a parameter: the monovalent
    /// Owczarzy branch is the only one implemented, so a condition with
    /// magnesium cannot be represented rather than being approximated.
    pub fn create(
        monovalent_millimolar: f64,
        primer_nanomolar: f64,
        dmso_percent: f64,
    ) -> Result<BufferModel, DomainError> {
        let refuse = |field: &str, reason: String| -> DomainError {
            DomainError::InvalidBufferModel(InvalidBufferModelProblem {
                field: field.to_string(),
                reason,
            })
        };
        if !(monovalent_millimolar.is_finite() && monovalent_millimolar > 0.0) {
            return Err(refuse(
                "monovalent",
                format!(
                    "{} mM is not a usable concentration; the salt correction takes its logarithm",
                    monovalent_millimolar
                ),
            ));
        }
        if !(primer_nanomolar.is_finite() && primer_nanomolar > 0.0) {
            return Err(refuse(
                "primer",
                format!(
                    "{} nM is not a usable concentration; the Tm equation takes its logarithm",
                    primer_nanomolar
                ),
            ));
        }
        if !(dmso_percent.is_finite() && dmso_percent >= 0.0) {
            return Err(refuse(
                "dmso",
                format!("{} % is not a usable proportion", dmso_percent),
            ));
        }
        Ok(BufferModel {
            monovalent_millimolar,
            primer_nanomolar,
            dmso_percent,
        })
    }

    pub fn monovalent_millimolar(&self) -> f64 {
        self.monovalent_millimolar
    }

    pub fn primer_nanomolar(&self) -> f64 {
        self.primer_nanomolar
    }

    pub fn dmso_percent(&self) -> f64 {
        self.dmso_percent
    }

    pub fn describe(&self) -> String {
        format!(
            "{} mM monovalent, {} nM primer, {} % DMSO",
            self.monovalent_millimolar, self.primer_nanomolar, self.dmso_percent
        )
    }
}

/// A melting temperature, with everything needed to know what it means.
#[derive(Clone, Debug, PartialEq)]
pub struct MeltingTemperature {
    celsius: f64,
    basis: TmBasis,
    buffer: BufferModel,
    enthalpy_kcal: f64,
    entropy_cal: f64,
    /// Present when the basis is not the one the caller asked for.
    substituted_because: Option<String>,
}

impl MeltingTemperature {
    pub fn celsius(&self) -> f64 {
        self.celsius
    }

    pub fn basis(&self) -> TmBasis {
        self.basis
    }

    pub fn buffer(&self) -> BufferModel {
        self.buffer
    }

    /// Total duplex ΔH°, kcal/mol. Reported so the value is checkable.
    pub fn enthalpy_kcal(&self) -> f64 {
        self.enthalpy_kcal
    }

    /// Total duplex ΔS°, cal/(K mol).
    pub fn entropy_cal(&self) -> f64 {
        self.entropy_cal
    }

    /// Why this is not the basis that was requested, when it is not.
    pub fn substituted_because(&self) -> Option<String> {
        self.substituted_because.clone()
    }

    pub fn function_name(&self) -> &'static str {
        TM_FUNCTION_NAME
    }

    pub fn function_version(&self) -> &'static str {
        TM_FUNCTION_VERSION
    }

    /// ΔG° at the given temperature, kcal/mol. The 2004 review's Table 1 and
    /// Table 2 print ΔG°37, so this is what the fixture check compares.
    pub fn free_energy_kcal(&self, kelvin: f64) -> f64 {
        self.enthalpy_kcal - kelvin * self.entropy_cal / 1000.0
    }

    pub fn describe(&self) -> String {
        format!("{:.2} °C ({})", self.celsius, self.basis.name())
    }
}

/// Look up a doublet, trying both strand orientations.
///
/// The nearest-neighbour model reads a doublet from either strand:
/// 5′-XY-3′/3′-X′Y′-5′ is the same motif as 5′-Y′X′-3′/3′-YX-5′. Tables list
/// one orientation, so both are tried before concluding a motif is absent.
fn lookup(
    table: &[(&str, f64, f64)],
    top: (char, char),
    bottom: (char, char),
) -> Option<(f64, f64)> {
    let direct: String = format!("{}{}/{}{}", top.0, top.1, bottom.0, bottom.1);
    let flipped: String = format!("{}{}/{}{}", bottom.1, bottom.0, top.1, top.0);
    for (motif, enthalpy, entropy) in table.iter() {
        if *motif == direct || *motif == flipped {
            return Some((*enthalpy, *entropy));
        }
    }
    None
}

fn complement(base: char) -> char {
    match base {
        'A' => 'T',
        'T' => 'A',
        'G' => 'C',
        'C' => 'G',
        other => other,
    }
}

fn is_watson_crick(top: char, bottom: char) -> bool {
    complement(top) == bottom
}

/// The melting temperature of `primer` annealed over the template site `site`.
///
/// `site` is the template's own strand over the primer's span, so
/// `site == primer` asks for the matched duplex and the wild-type text asks
/// for the mismatch-aware one. Both must be plain A/C/G/T of equal length: an
/// ambiguity code has no nearest-neighbour parameters, and H6 already keeps
/// one out of a primer.
pub fn melting_temperature(
    primer: &[char],
    site: &[char],
    buffer: &BufferModel,
) -> Result<MeltingTemperature, DomainError> {
    let basis: TmBasis = if primer == site {
        TmBasis::MatchedComplement
    } else {
        TmBasis::MismatchedWildTypeSite
    };

    if primer.len() != site.len() {
        return Err(unavailable(
            "length",
            format!(
                "the primer is {} bases and the template site {}; a duplex needs them equal",
                primer.len(),
                site.len()
            ),
        ));
    }
    // Two bases is the shortest duplex with a nearest-neighbour doublet.
    if primer.len() < 2 {
        let problem = EmptySequenceProblem {
            sequence: "primer".to_string(),
        };
        return Err(DomainError::EmptySequenceContent(problem));
    }
    for (index, base) in primer.iter().chain(site.iter()).enumerate() {
        if !is_dna_base(*base) {
            return Err(unavailable(
                "alphabet",
                format!(
                    "{} at position {} is not A, C, G or T, and has no nearest-neighbour parameters",
                    base, index
                ),
            ));
        }
    }

    // The duplex: the primer over the complement of the template site.
    let bottom: Vec<char> = site.iter().map(|base| complement(*base)).collect();

    let mut enthalpy: f64 = INITIATION.0;
    let mut entropy: f64 = INITIATION.1;

    let mut index: usize = 0;
    while index + 1 < primer.len() {
        let top: (char, char) = (primer[index], primer[index + 1]);
        let base: (char, char) = (bottom[index], bottom[index + 1]);
        let first: bool = is_watson_crick(top.0, base.0);
        let second: bool = is_watson_crick(top.1, base.1);

        let table: &[(&str, f64, f64)] = match (first, second) {
            (true, true) => &MATCHED,
            (false, false) => {
                return Err(unavailable(
                    "tandem mismatch",
                    format!(
                        "positions {} and {} are both mismatched; no published parameters cover a doublet with two mismatches",
                        index,
                        index + 1
                    ),
                ));
            }
            _ => &MISMATCHED,
        };
        let found: Option<(f64, f64)> = lookup(table, top, base);
        if found.is_none() {
            return Err(unavailable(
                "motif",
                format!(
                    "no parameters for the doublet at positions {}-{}",
                    index,
                    index + 1
                ),
            ));
        }
        let (doublet_enthalpy, doublet_entropy) = found.unwrap();
        enthalpy += doublet_enthalpy;
        entropy += doublet_entropy;
        index += 1;
    }

    // One terminal A-T penalty per duplex end closed by an A-T pair. The
    // penalty is a property of the pair, so a mismatched end is not an A-T
    // pair and takes none.
    let last: usize = primer.len() - 1;
    for (top, base) in [(primer[0], bottom[0]), (primer[last], bottom[last])] {
        if is_watson_crick(top, base) && (top == 'A' || top == 'T') {
            enthalpy += TERMINAL_AT.0;
            entropy += TERMINAL_AT.1;
        }
    }

    let self_complementary: bool = primer
        .iter()
        .zip(primer.iter().rev())
        .all(|(left, right)| complement(*left) == *right);
    if self_complementary {
        enthalpy += SYMMETRY.0;
        entropy += SYMMETRY.1;
    }

    // Tm in 1 M salt: ΔH° / (ΔS° + R ln C_T), SantaLucia & Hicks (2004) eq. 3.
    // The primer is in large excess over the template in a PCR, so the
    // primer's own concentration is C_T. The source divides C_T by 4 for a
    // non-self-complementary duplex whose two strands are at equal
    // concentration; that case does not arise here, because the template
    // strand is not at the primer's concentration. A self-complementary primer
    // takes C_T undivided, plus the symmetry term applied above.
    //
    // Checked against Biopython's independent implementation on both
    // self-complementary and non-self-complementary synthetics, and against
    // the reference tool's recorded values on twelve real primers, none of
    // which is self-complementary (`tests/fixtures/tm/README.md`).
    let molar: f64 = buffer.primer_nanomolar * 1e-9;
    let one_molar_kelvin: f64 = (enthalpy * 1000.0) / (entropy + R * molar.ln());

    // Owczarzy 2004, monovalent branch. Requires the G+C fraction of the
    // primer, counted over the primer itself.
    let gc: f64 = primer
        .iter()
        .filter(|base| **base == 'G' || **base == 'C')
        .count() as f64
        / primer.len() as f64;
    let salt: f64 = buffer.monovalent_millimolar * 1e-3;
    let correction: f64 = (4.29 * gc - 3.95) * 1e-5 * salt.ln() + 9.40e-6 * salt.ln() * salt.ln();
    let corrected_kelvin: f64 = 1.0 / (1.0 / one_molar_kelvin + correction);

    // DMSO depresses Tm; 0.6 °C per percent is the factor the reference uses.
    let celsius: f64 = corrected_kelvin - ZERO_CELSIUS_IN_KELVIN - 0.6 * buffer.dmso_percent;

    Ok(MeltingTemperature {
        celsius,
        basis,
        buffer: *buffer,
        enthalpy_kcal: enthalpy,
        entropy_cal: entropy,
        substituted_because: None,
    })
}

/// The mismatch-aware Tm, falling back to the matched duplex where no
/// published parameters cover the mismatch pattern.
///
/// This is the hybrid the owner chose on 2026-10-14. The fallback is recorded
/// on the value, so a caller can always tell which duplex a number describes;
/// the two are never averaged or reported under one name.
pub fn melting_temperature_or_matched(
    primer: &[char],
    site: &[char],
    buffer: &BufferModel,
) -> Result<MeltingTemperature, DomainError> {
    match melting_temperature(primer, site, buffer) {
        Ok(value) => Ok(value),
        Err(DomainError::TmParametersUnavailable(problem)) => {
            let mut fallback: MeltingTemperature = melting_temperature(primer, primer, buffer)?;
            fallback.substituted_because = Some(format!(
                "the mismatch-aware basis is unavailable ({}): {}",
                problem.aspect, problem.reason
            ));
            Ok(fallback)
        }
        Err(other) => Err(other),
    }
}

fn unavailable(aspect: &str, reason: String) -> DomainError {
    DomainError::TmParametersUnavailable(TmParametersUnavailableProblem {
        function: TM_FUNCTION_NAME.to_string(),
        aspect: aspect.to_string(),
        reason,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCategory;

    fn letters(text: &str) -> Vec<char> {
        text.chars().collect()
    }

    fn matched_tm(text: &str) -> MeltingTemperature {
        let seq: Vec<char> = letters(text);
        melting_temperature(&seq, &seq, &BufferModel::q5_reaction()).unwrap()
    }

    // The published ΔG°37 for every Watson-Crick doublet, from the same table
    // the parameters come from, used here to check that this module's
    // bookkeeping reproduces the source's own arithmetic. A two-base duplex of
    // motif XY carries the doublet plus initiation plus any terminal A-T
    // penalties, so the doublet's own ΔG°37 is recovered by subtracting those.
    #[test]
    fn each_matched_doublet_reproduces_its_published_free_energy() {
        let published: [(&str, f64); 10] = [
            ("AA/TT", -1.00),
            ("AT/TA", -0.88),
            ("TA/AT", -0.58),
            ("CA/GT", -1.45),
            ("GT/CA", -1.44),
            ("CT/GA", -1.28),
            ("GA/CT", -1.30),
            ("CG/GC", -2.17),
            ("GC/CG", -2.24),
            ("GG/CC", -1.84),
        ];
        for (motif, expected) in published.iter() {
            let (enthalpy, entropy) = lookup(
                &MATCHED,
                (motif.chars().next().unwrap(), motif.chars().nth(1).unwrap()),
                (motif.chars().nth(3).unwrap(), motif.chars().nth(4).unwrap()),
            )
            .unwrap();
            let free_energy: f64 = enthalpy - REFERENCE_KELVIN * entropy / 1000.0;
            let difference: f64 = (free_energy - expected).abs();
            assert!(
                difference < 0.02,
                "{}: ΔG°37 {:.3} from ΔH/ΔS, published {:.2}",
                motif,
                free_energy,
                expected
            );
        }
    }

    // Published ΔG°37 for single mismatches, SantaLucia & Hicks (2004) Table 2.
    // The rounding of ΔH to 0.1 and ΔS to 0.1 propagates to about ±0.13.
    #[test]
    fn each_mismatched_doublet_reproduces_its_published_free_energy() {
        let published: [(&str, f64); 12] = [
            ("GA/CA", 0.17),
            ("GG/CG", -1.11),
            ("GT/CG", -0.59),
            ("GG/CT", 0.08),
            ("CT/GT", -0.12),
            ("CG/GG", -0.11),
            ("AG/TT", 0.71),
            ("AC/TC", 1.33),
            ("TC/AA", 1.33),
            ("TA/AA", 0.69),
            ("TT/AT", 0.68),
            ("AA/TA", 0.61),
        ];
        for (motif, expected) in published.iter() {
            let (enthalpy, entropy) = lookup(
                &MISMATCHED,
                (motif.chars().next().unwrap(), motif.chars().nth(1).unwrap()),
                (motif.chars().nth(3).unwrap(), motif.chars().nth(4).unwrap()),
            )
            .unwrap();
            let free_energy: f64 = enthalpy - REFERENCE_KELVIN * entropy / 1000.0;
            assert!(
                (free_energy - expected).abs() < 0.13,
                "{}: ΔG°37 {:.3} from ΔH/ΔS, published {:.2}",
                motif,
                free_energy,
                expected
            );
        }
    }

    #[test]
    fn a_doublet_reads_the_same_from_either_strand() {
        // 5'-AA-3'/3'-TT-5' is the only orientation in the table; the flipped
        // reading of 5'-TT-3'/3'-AA-5' must find it.
        assert!(lookup(&MATCHED, ('A', 'A'), ('T', 'T')) == Some((-7.6, -21.3)));
        assert!(lookup(&MATCHED, ('T', 'T'), ('A', 'A')) == Some((-7.6, -21.3)));
        // CA/GT and its flip TG/AC are one motif, distinct from GT/CA.
        assert!(lookup(&MATCHED, ('T', 'G'), ('A', 'C')) == Some((-8.5, -22.7)));
        assert!(lookup(&MATCHED, ('G', 'T'), ('C', 'A')) == Some((-8.4, -22.4)));
    }

    // GC-rich sequences melt higher than AT-rich ones of the same length.
    #[test]
    fn gc_content_raises_the_melting_temperature() {
        let at: f64 = matched_tm("ATATATATATATATATAT").celsius();
        let mixed: f64 = matched_tm("ATGCATGCATGCATGCAT").celsius();
        let gc: f64 = matched_tm("GCGCGCGCGCGCGCGCGC").celsius();
        assert!(at < mixed && mixed < gc, "{} {} {}", at, mixed, gc);
        // Sanity: an 18-mer of mixed composition is in the usual primer range.
        assert!(mixed > 40.0 && mixed < 80.0, "{}", mixed);
    }

    #[test]
    fn a_longer_duplex_melts_higher_than_a_prefix_of_itself() {
        let short: f64 = matched_tm("ATGCATGCATGCAT").celsius();
        let long: f64 = matched_tm("ATGCATGCATGCATGCATGC").celsius();
        assert!(short < long, "{} {}", short, long);
    }

    #[test]
    fn more_salt_raises_the_melting_temperature() {
        let seq: Vec<char> = letters("ATGCATGCATGCATGCAT");
        let low: MeltingTemperature =
            melting_temperature(&seq, &seq, &BufferModel::create(50.0, 500.0, 0.0).unwrap())
                .unwrap();
        let high: MeltingTemperature =
            melting_temperature(&seq, &seq, &BufferModel::create(150.0, 500.0, 0.0).unwrap())
                .unwrap();
        assert!(low.celsius() < high.celsius(), "{:?} {:?}", low, high);
    }

    #[test]
    fn dmso_depresses_the_melting_temperature_by_the_stated_factor() {
        let seq: Vec<char> = letters("ATGCATGCATGCATGCAT");
        let dry: f64 = melting_temperature(&seq, &seq, &BufferModel::q5_reaction())
            .unwrap()
            .celsius();
        let wet: f64 =
            melting_temperature(&seq, &seq, &BufferModel::create(150.0, 500.0, 5.0).unwrap())
                .unwrap()
                .celsius();
        assert!((dry - wet - 3.0).abs() < 1e-9, "{} {}", dry, wet);
    }

    // A mismatch destabilises, so the mismatch-aware Tm is below the matched
    // one for the same primer.
    #[test]
    fn one_mismatch_lowers_the_melting_temperature() {
        let primer: Vec<char> = letters("ATGCATGCATGCATGCAT");
        let mut site: Vec<char> = primer.clone();
        site[9] = 'A'; // the primer has C here, so C:T on the bottom strand
        let buffer: BufferModel = BufferModel::q5_reaction();
        let matched: MeltingTemperature = melting_temperature(&primer, &primer, &buffer).unwrap();
        let mismatched: MeltingTemperature = melting_temperature(&primer, &site, &buffer).unwrap();
        assert!(matched.basis() == TmBasis::MatchedComplement);
        assert!(mismatched.basis() == TmBasis::MismatchedWildTypeSite);
        assert!(
            mismatched.celsius() < matched.celsius(),
            "{} {}",
            mismatched.celsius(),
            matched.celsius()
        );
    }

    // Two adjacent mismatches put a doublet with two mismatched pairs in the
    // middle, which no published table covers.
    #[test]
    fn two_adjacent_mismatches_have_no_parameters() {
        let primer: Vec<char> = letters("ATGCATGCATGCATGCAT");
        let mut site: Vec<char> = primer.clone();
        site[9] = 'A';
        site[10] = 'C';
        let error: DomainError =
            melting_temperature(&primer, &site, &BufferModel::q5_reaction()).unwrap_err();
        assert!(error.category() == ErrorCategory::Unsupported);
        match error {
            DomainError::TmParametersUnavailable(problem) => {
                assert!(problem.aspect == "tandem mismatch");
                assert!(problem.function == TM_FUNCTION_NAME);
            }
            other => panic!("expected TmParametersUnavailable, got {:?}", other),
        }
    }

    #[test]
    fn the_hybrid_falls_back_to_the_matched_basis_and_says_so() {
        let primer: Vec<char> = letters("ATGCATGCATGCATGCAT");
        let buffer: BufferModel = BufferModel::q5_reaction();

        // One mismatch: the requested basis is available, so no substitution.
        let mut single: Vec<char> = primer.clone();
        single[9] = 'A';
        let kept: MeltingTemperature =
            melting_temperature_or_matched(&primer, &single, &buffer).unwrap();
        assert!(kept.basis() == TmBasis::MismatchedWildTypeSite);
        assert!(kept.substituted_because().is_none());

        // Two adjacent: the basis is substituted, with the reason recorded.
        let mut tandem: Vec<char> = primer.clone();
        tandem[9] = 'A';
        tandem[10] = 'C';
        let swapped: MeltingTemperature =
            melting_temperature_or_matched(&primer, &tandem, &buffer).unwrap();
        assert!(swapped.basis() == TmBasis::MatchedComplement);
        let reason: String = swapped.substituted_because().unwrap();
        assert!(reason.contains("tandem mismatch"), "{}", reason);
        // The fallback is exactly the matched Tm, not an adjusted one.
        assert!(
            swapped.celsius()
                == melting_temperature(&primer, &primer, &buffer)
                    .unwrap()
                    .celsius()
        );
    }

    #[test]
    fn an_ambiguity_code_has_no_parameters() {
        let primer: Vec<char> = letters("ATGCATGCNTGCATGCAT");
        let error: DomainError =
            melting_temperature(&primer, &primer, &BufferModel::q5_reaction()).unwrap_err();
        assert!(matches!(error, DomainError::TmParametersUnavailable(_)));
    }

    #[test]
    fn a_duplex_needs_two_bases_and_equal_strand_lengths() {
        let buffer: BufferModel = BufferModel::q5_reaction();
        let one: Vec<char> = letters("A");
        assert!(matches!(
            melting_temperature(&one, &one, &buffer).unwrap_err(),
            DomainError::EmptySequenceContent(_)
        ));
        let primer: Vec<char> = letters("ATGC");
        let site: Vec<char> = letters("ATG");
        assert!(matches!(
            melting_temperature(&primer, &site, &buffer).unwrap_err(),
            DomainError::TmParametersUnavailable(_)
        ));
    }

    #[test]
    fn a_buffer_concentration_of_zero_is_refused() {
        for (monovalent, primer, dmso) in
            [(0.0, 500.0, 0.0), (150.0, 0.0, 0.0), (150.0, 500.0, -1.0)]
        {
            let error: DomainError = BufferModel::create(monovalent, primer, dmso).unwrap_err();
            assert!(error.category() == ErrorCategory::MalformedInput);
            assert!(matches!(error, DomainError::InvalidBufferModel(_)));
        }
    }

    #[test]
    fn the_q5_condition_is_the_one_the_research_recorded() {
        let buffer: BufferModel = BufferModel::q5_reaction();
        assert!(buffer.monovalent_millimolar() == 150.0);
        assert!(buffer.primer_nanomolar() == 500.0);
        assert!(buffer.dmso_percent() == 0.0);
    }

    #[test]
    fn the_same_inputs_give_the_same_value() {
        assert!(matched_tm("ATGCATGCATGCATGCAT") == matched_tm("ATGCATGCATGCATGCAT"));
    }
}
