//! The thresholds in `crates/core/src/primer_quality.rs` are a copy of
//! `tests/fixtures/primer-quality/thresholds.tsv`, and the fixture is the
//! authority (D018): it records the published source of each one. This test
//! keeps the two equal, so a threshold cannot move in the Rust without the
//! fixture and its sources moving too.

use luria_core::primer_quality::GC_HIGH_PERCENT;
use luria_core::primer_quality::GC_LOW_PERCENT;
use luria_core::primer_quality::LENGTH_HIGH;
use luria_core::primer_quality::LENGTH_LOW;
use luria_core::primer_quality::MAX_DINUCLEOTIDE_REPEATS;
use luria_core::primer_quality::MAX_ORDERABLE_LENGTH;
use luria_core::primer_quality::MAX_SINGLE_BASE_RUN;
use luria_core::primer_quality::MAX_THREE_PRIME_GC_RUN;

const FIXTURE: &str = "../../tests/fixtures/primer-quality/thresholds.tsv";

/// The fixture's `name` and `value` columns.
fn rows() -> Vec<(String, f64)> {
    let text: String = std::fs::read_to_string(FIXTURE)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", FIXTURE, error));
    let mut out: Vec<(String, f64)> = Vec::new();
    for line in text.lines() {
        let trimmed: &str = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("name\t") {
            continue;
        }
        let fields: Vec<&str> = trimmed.split('\t').collect();
        assert!(
            fields.len() >= 2,
            "row {:?} has {} fields",
            fields.first(),
            fields.len()
        );
        let value: f64 = fields[1]
            .parse()
            .unwrap_or_else(|_| panic!("{}: {:?} is not a number", fields[0], fields[1]));
        out.push((fields[0].to_string(), value));
    }
    out
}

#[test]
fn every_threshold_equals_the_fixture() {
    let expected: Vec<(&str, f64)> = vec![
        ("GC_LOW_PERCENT", GC_LOW_PERCENT),
        ("GC_HIGH_PERCENT", GC_HIGH_PERCENT),
        ("MAX_SINGLE_BASE_RUN", MAX_SINGLE_BASE_RUN as f64),
        ("MAX_DINUCLEOTIDE_REPEATS", MAX_DINUCLEOTIDE_REPEATS as f64),
        ("MAX_THREE_PRIME_GC_RUN", MAX_THREE_PRIME_GC_RUN as f64),
        ("LENGTH_LOW", LENGTH_LOW as f64),
        ("LENGTH_HIGH", LENGTH_HIGH as f64),
        ("MAX_ORDERABLE_LENGTH", MAX_ORDERABLE_LENGTH as f64),
    ];
    let fixture: Vec<(String, f64)> = rows();
    assert!(
        fixture.len() == expected.len(),
        "the fixture has {} thresholds, primer_quality.rs has {}",
        fixture.len(),
        expected.len()
    );
    for (name, value) in expected.iter() {
        let found: &(String, f64) = fixture
            .iter()
            .find(|(row, _)| row == name)
            .unwrap_or_else(|| panic!("{} is not in the fixture", name));
        assert!(
            (found.1 - value).abs() < 1e-9,
            "{}: primer_quality.rs has {}, the fixture has {}",
            name,
            value,
            found.1
        );
    }
}

/// The bands have to be the right way round, or every primer warns. Checked
/// against the fixture's own values rather than the constants, which the
/// compiler already knows and clippy rightly calls a tautology.
#[test]
fn the_fixture_bands_are_coherent() {
    let fixture: Vec<(String, f64)> = rows();
    let value = |name: &str| -> f64 {
        fixture
            .iter()
            .find(|(row, _)| row == name)
            .unwrap_or_else(|| panic!("{} is not in the fixture", name))
            .1
    };
    assert!(value("GC_LOW_PERCENT") < value("GC_HIGH_PERCENT"));
    assert!(value("LENGTH_LOW") < value("LENGTH_HIGH"));
    // The ordering limit sits above the length guideline, so a primer past
    // either threshold still draws the guideline warning first (D032).
    assert!(value("LENGTH_HIGH") < value("MAX_ORDERABLE_LENGTH"));
    // A single G or C at the 3' end is the GC clamp D030 extends toward, so it
    // must stay unwarned.
    assert!(value("MAX_THREE_PRIME_GC_RUN") >= 1.0);
    // A run of one base is also a run of a two-base unit, so the single-base
    // threshold must not be the looser of the two.
    assert!(value("MAX_SINGLE_BASE_RUN") <= 2.0 * value("MAX_DINUCLEOTIDE_REPEATS"));
}
