//! luria's Tm against an independent implementation of the same published
//! model.
//!
//! The expected values below were produced by **Biopython 1.88**
//! (`Bio.SeqUtils.MeltingTemp.Tm_NN`), not by luria, with settings chosen to
//! name the same model `crates/core/src/tm.rs` implements:
//!
//! ```text
//! nn_table  = DNA_NN4    SantaLucia & Hicks (2004)
//! imm_table = DNA_IMM1   Allawi & SantaLucia 1997/1998, Peyret et al. 1999
//! saltcorr  = 6          Owczarzy et al. (2004), monovalent branch
//! Na = 150, dnac1 = 500, dnac2 = 0, selfcomp passed per sequence
//! ```
//!
//! This is a check of luria's arithmetic, not of its choice of parameters: two
//! implementations reading the same tables should agree, and a sign error, a
//! misread doublet, a wrong concentration convention or a missing correction
//! shows up as a disagreement. The generating script is `.tmp/tm/crosscheck.py`.
//!
//! Note on `selfcomp`: Biopython does not detect self-complementarity — it is a
//! caller-supplied flag defaulting to false — whereas luria detects it from the
//! sequence. The flag was passed to match. Note also that Biopython's docstring
//! records MELTING and Primer3Plus as using `C_T/4` by default, which would put
//! their values about 1 °C below both of these; a Tm is only comparable
//! alongside the convention that produced it, which is why
//! `MeltingTemperature` carries its buffer and basis.

use luria_core::tm::melting_temperature;
use luria_core::tm::BufferModel;
use luria_core::tm::MeltingTemperature;
use luria_core::tm::TmBasis;

/// A mixed-composition 18-mer. Self-complementary, as all three 18-mers here
/// are, which is what exercises the symmetry term.
const MIXED: &str = "ATGCATGCATGCATGCAT";

/// `MIXED` with index 9 changed, giving one T·T mismatch against the primer.
const ONE_MISMATCH_SITE: &str = "ATGCATGCAAGCATGCAT";

/// A 23-mer carrying the `q5p-01` edit pattern, `GAT` → `AAC`: indices 10 and
/// 12 differ and index 11 matches, so there are two single mismatches
/// separated by one Watson-Crick pair. Not self-complementary.
const PRIMER_23: &str = "ACGTACGTACAACGTACGTACGT";
const SITE_23: &str = "ACGTACGTACGATGTACGTACGT";

/// (label, primer, template site, Biopython's value in °C)
const EXPECTED: [(&str, &str, &str, f64); 6] = [
    ("matched mixed 18-mer", MIXED, MIXED, 62.126),
    (
        "matched GC-rich 18-mer",
        "GCGCGCGCGCGCGCGCGC",
        "GCGCGCGCGCGCGCGCGC",
        88.462,
    ),
    (
        "matched AT-rich 18-mer",
        "ATATATATATATATATAT",
        "ATATATATATATATATAT",
        29.723,
    ),
    ("one mismatch at index 9", MIXED, ONE_MISMATCH_SITE, 56.390),
    ("q5p-01 pattern, matched", PRIMER_23, PRIMER_23, 67.397),
    ("q5p-01 pattern, two mismatches", PRIMER_23, SITE_23, 56.750),
];

fn tm(primer: &str, site: &str) -> MeltingTemperature {
    let primer_letters: Vec<char> = primer.chars().collect();
    let site_letters: Vec<char> = site.chars().collect();
    melting_temperature(&primer_letters, &site_letters, &BufferModel::q5_reaction())
        .unwrap_or_else(|error| panic!("{} / {}: {:?}", primer, site, error))
}

#[test]
fn luria_agrees_with_biopython_on_every_cross_check_case() {
    for (label, primer, site, expected) in EXPECTED.iter() {
        let value: f64 = tm(primer, site).celsius();
        let difference: f64 = (value - expected).abs();
        assert!(
            difference < 0.005,
            "{}: luria {:.3} °C, Biopython {:.3} °C, difference {:.3}",
            label,
            value,
            expected,
            difference
        );
    }
}

/// The cases are not all the same shape, which is the point of the set: a
/// matched duplex, a self-complementary one, one mismatch and two.
#[test]
fn the_cross_check_set_covers_both_bases_and_both_mismatch_counts() {
    assert!(tm(MIXED, MIXED).basis() == TmBasis::MatchedComplement);
    assert!(tm(MIXED, ONE_MISMATCH_SITE).basis() == TmBasis::MismatchedWildTypeSite);
    assert!(tm(PRIMER_23, SITE_23).basis() == TmBasis::MismatchedWildTypeSite);

    // A mismatch destabilises, and two destabilise more than one.
    let matched: f64 = tm(PRIMER_23, PRIMER_23).celsius();
    let two: f64 = tm(PRIMER_23, SITE_23).celsius();
    assert!(two < matched, "{} {}", two, matched);
    let one_drop: f64 = tm(MIXED, MIXED).celsius() - tm(MIXED, ONE_MISMATCH_SITE).celsius();
    let two_drop: f64 = matched - two;
    assert!(
        one_drop > 0.0 && two_drop > one_drop,
        "{} {}",
        one_drop,
        two_drop
    );
}
