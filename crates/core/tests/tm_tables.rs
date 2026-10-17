//! The Tm parameter tables in `crates/core/src/tm.rs` are a copy of
//! `tests/fixtures/tm/`, and the fixture is the authority (D018). These tests
//! check that the copy is faithful and that the fixture's own verification
//! still holds, so a value cannot drift in the Rust without the fixture and
//! its recorded provenance changing too.

use luria_core::tm::MATCHED;
use luria_core::tm::MISMATCHED;
use luria_core::tm::REFERENCE_KELVIN;
use luria_core::tm::{INITIATION, SYMMETRY, TERMINAL_AT};

const FIXTURE_DIR: &str = "../../tests/fixtures/tm";

/// One parameter row: motif, ΔH, ΔS, and the published ΔG°37 where the fixture
/// carries one.
struct Row {
    motif: String,
    enthalpy: f64,
    entropy: f64,
    free_energy: Option<f64>,
}

fn rows(file: &str) -> Vec<Row> {
    let path: String = format!("{}/{}", FIXTURE_DIR, file);
    let text: String = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
    let mut out: Vec<Row> = Vec::new();
    for line in text.lines() {
        let trimmed: &str = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("motif\t") {
            continue;
        }
        let fields: Vec<&str> = trimmed.split('\t').collect();
        assert!(
            fields.len() >= 3,
            "{}: row {:?} has {} fields",
            file,
            fields.first(),
            fields.len()
        );
        let number = |text: &str, what: &str| -> f64 {
            text.parse::<f64>()
                .unwrap_or_else(|_| panic!("{}: {} {:?} is not a number", file, what, text))
        };
        out.push(Row {
            motif: fields[0].to_string(),
            enthalpy: number(fields[1], "dH"),
            entropy: number(fields[2], "dS"),
            free_energy: fields.get(3).map(|text| number(text, "dG37")),
        });
    }
    out
}

fn embedded(table: &[(&str, f64, f64)], motif: &str) -> Option<(f64, f64)> {
    table
        .iter()
        .find(|(name, _, _)| *name == motif)
        .map(|(_, enthalpy, entropy)| (*enthalpy, *entropy))
}

fn same(left: f64, right: f64) -> bool {
    (left - right).abs() < 1e-9
}

#[test]
fn the_embedded_watson_crick_table_equals_the_fixture() {
    let fixture: Vec<Row> = rows("nn-matched.tsv");
    let mut doublets: usize = 0;
    for row in fixture.iter() {
        let found: Option<(f64, f64)> = match row.motif.as_str() {
            "initiation" => Some(INITIATION),
            "terminal_AT" => Some(TERMINAL_AT),
            "symmetry" => Some(SYMMETRY),
            motif => {
                doublets += 1;
                embedded(&MATCHED, motif)
            }
        };
        let (enthalpy, entropy) =
            found.unwrap_or_else(|| panic!("{} is in the fixture but not in tm.rs", row.motif));
        assert!(
            same(enthalpy, row.enthalpy) && same(entropy, row.entropy),
            "{}: tm.rs has {}/{}, the fixture has {}/{}",
            row.motif,
            enthalpy,
            entropy,
            row.enthalpy,
            row.entropy
        );
    }
    assert!(
        doublets == MATCHED.len(),
        "the fixture has {} doublets, tm.rs has {}",
        doublets,
        MATCHED.len()
    );
}

#[test]
fn the_embedded_mismatch_table_equals_the_fixture() {
    let fixture: Vec<Row> = rows("nn-mismatch.tsv");
    assert!(
        fixture.len() == MISMATCHED.len(),
        "the fixture has {} motifs, tm.rs has {}",
        fixture.len(),
        MISMATCHED.len()
    );
    for row in fixture.iter() {
        let (enthalpy, entropy) = embedded(&MISMATCHED, &row.motif)
            .unwrap_or_else(|| panic!("{} is in the fixture but not in tm.rs", row.motif));
        assert!(
            same(enthalpy, row.enthalpy) && same(entropy, row.entropy),
            "{}: tm.rs has {}/{}, the fixture has {}/{}",
            row.motif,
            enthalpy,
            entropy,
            row.enthalpy,
            row.entropy
        );
    }
}

/// The fixture's own verification, rerun: ΔH − TΔS must reproduce the ΔG°37
/// that SantaLucia & Hicks (2004) Table 2 prints. ΔH and ΔS are rounded to
/// 0.1 in the source, which propagates to about ±0.13 kcal/mol here.
#[test]
fn every_mismatch_parameter_still_reproduces_its_published_free_energy() {
    let fixture: Vec<Row> = rows("nn-mismatch.tsv");
    let mut checked: usize = 0;
    let mut worst: f64 = 0.0;
    for row in fixture.iter() {
        let published: f64 = row
            .free_energy
            .unwrap_or_else(|| panic!("{} carries no published dG37 to check against", row.motif));
        let derived: f64 = row.enthalpy - REFERENCE_KELVIN * row.entropy / 1000.0;
        let difference: f64 = (derived - published).abs();
        assert!(
            difference < 0.13,
            "{}: dH/dS imply dG37 {:.3}, the review prints {:.2}, difference {:.3}",
            row.motif,
            derived,
            published,
            difference
        );
        worst = worst.max(difference);
        checked += 1;
    }
    assert!(checked == 48, "checked {} motifs, expected 48", checked);
    assert!(worst < 0.13, "worst difference {:.3}", worst);
}

/// The fixture's own verification, rerun for the Watson-Crick table: ΔH − TΔS
/// must reproduce the ΔG°37 that SantaLucia 1998 prints, including the values
/// derived from its per-end initiation terms. ΔH and ΔS are rounded to 0.1 in
/// the source, which propagates to about ±0.13 kcal/mol here.
#[test]
fn every_watson_crick_parameter_still_reproduces_its_published_free_energy() {
    let fixture: Vec<Row> = rows("nn-matched.tsv");
    let mut checked: usize = 0;
    let mut worst: f64 = 0.0;
    for row in fixture.iter() {
        let published: f64 = row
            .free_energy
            .unwrap_or_else(|| panic!("{} carries no published dG37 to check against", row.motif));
        let derived: f64 = row.enthalpy - REFERENCE_KELVIN * row.entropy / 1000.0;
        let difference: f64 = (derived - published).abs();
        assert!(
            difference < 0.13,
            "{}: dH/dS imply dG37 {:.3}, the source prints {:.2}, difference {:.3}",
            row.motif,
            derived,
            published,
            difference
        );
        worst = worst.max(difference);
        checked += 1;
    }
    assert!(checked == 13, "checked {} rows, expected 13", checked);
    assert!(worst < 0.13, "worst difference {:.3}", worst);
}

/// The 48 motifs are every single-mismatch doublet: four Watson-Crick pairs,
/// each with the twelve mismatched pairs that are not Watson-Crick.
#[test]
fn the_mismatch_table_covers_every_single_mismatch_doublet() {
    let bases: [char; 4] = ['A', 'C', 'G', 'T'];
    let complement = |base: char| -> char {
        match base {
            'A' => 'T',
            'T' => 'A',
            'G' => 'C',
            _ => 'G',
        }
    };
    let mut expected: Vec<String> = Vec::new();
    for pair in bases.iter() {
        for top in bases.iter() {
            for bottom in bases.iter() {
                if complement(*top) == *bottom {
                    continue; // a Watson-Crick second pair is the matched table
                }
                expected.push(format!("{}{}/{}{}", pair, top, complement(*pair), bottom));
            }
        }
    }
    assert!(expected.len() == 48, "expected {} motifs", expected.len());
    for motif in expected.iter() {
        assert!(
            embedded(&MISMATCHED, motif).is_some(),
            "{} is a single-mismatch doublet with no parameters",
            motif
        );
    }
}
