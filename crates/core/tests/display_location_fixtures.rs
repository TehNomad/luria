//! The display-only location parser of `crates/core/src/display_location.rs`
//! (contract 8b1, §4.1 of `docs/tasks/milestone-8-view.md`) against
//! `tests/fixtures/genbank/synthetic.gb`: the CDS whose location is `<1..12`
//! is drawn by `parse_display_location` and still refused for import by
//! `parse_genbank_location` (D049, D053, D041). The row was derived by the
//! planner from the GenBank text (1-based inclusive `a..b` is zero-based
//! `[a - 1, b)`, D006), not from running code.

use luria_core::display_location::parse_display_location;
use luria_core::display_location::DisplayLocation;
use luria_core::error::DomainError;
use luria_core::error::GenBankLocationRule;
use luria_core::genbank::read_genbank;
use luria_core::genbank::GenBankFeature;
use luria_core::genbank::GenBankRecord;
use luria_core::genbank_location::parse_genbank_location;
use luria_core::identity::Identifier;
use luria_core::sequence::SequenceTopology;
use luria_core::strand::Strand;

const FIXTURES: &str = "../../tests/fixtures/genbank/";
const SYNTHETIC_FILE: &str = "synthetic.gb";
/// The `fuzzy` CDS's key line in `synthetic.gb`, counted from 1.
const FUZZY_LINE: u64 = 59;

/// A fixture input, read as it sits on disk.
fn read_input(file: &str) -> String {
    let path: String = format!("{}{}", FIXTURES, file);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("cannot read {}: {}", path, error))
}

/// The sequence identifier a fixture record is read as: the identity comes
/// from the caller, as in every other reader.
fn name(value: &str) -> Identifier {
    Identifier::create(value.to_string())
        .unwrap_or_else(|error| panic!("cannot use {:?} as an identifier: {:?}", value, error))
}

/// The synthetic record, read from `synthetic.gb`.
fn synthetic() -> GenBankRecord {
    read_genbank(name(SYNTHETIC_FILE), read_input(SYNTHETIC_FILE)).unwrap_or_else(|error| {
        panic!(
            "{}: read_genbank refused it: {:?}",
            SYNTHETIC_FILE,
            error.message()
        )
    })
}

/// The CDS whose `/gene` is `fuzzy`.
fn fuzzy(record: &GenBankRecord) -> GenBankFeature {
    let found: Vec<GenBankFeature> = record
        .features()
        .into_iter()
        .filter(|feature| {
            feature.key() == "CDS"
                && feature.qualifiers().iter().any(|qualifier| {
                    qualifier.name() == "gene" && qualifier.value().as_deref() == Some("fuzzy")
                })
        })
        .collect();
    assert!(
        found.len() == 1,
        "expected one CDS with /gene=\"fuzzy\", found {}",
        found.len()
    );
    found[0].clone()
}

#[test]
fn the_synthetic_fuzzy_cds_is_drawn_and_still_refused_for_import() {
    let record: GenBankRecord = synthetic();
    let feature: GenBankFeature = fuzzy(&record);
    assert_eq!(
        feature.location(),
        "<1..12",
        "the fuzzy CDS's location text on line {}",
        feature.line()
    );
    assert_eq!(
        feature.line(),
        FUZZY_LINE,
        "the fuzzy CDS's key line, expected {}",
        FUZZY_LINE
    );
    let length: u64 = record.sequence().length();
    let topology: SequenceTopology = record.sequence().topology();
    assert_eq!(length, 60, "the synthetic record's length");
    assert!(
        topology.name() == "circular",
        "the synthetic record's topology is {:?}, expected circular",
        topology.name()
    );

    // Drawn: `<1..12` takes the stated base, so the bar runs over [0, 12) and
    // its left end is open at reference position 0 (D006, D053).
    let drawn: DisplayLocation =
        parse_display_location(&feature.location(), length, topology.clone()).unwrap_or_else(
            |error| {
                panic!(
                    "parse_display_location refuses {:?}: {:?}",
                    feature.location(),
                    error.message()
                )
            },
        );
    assert_eq!(drawn.location.intervals(), vec![(0, 12)], "the drawn shape");
    assert_eq!(
        drawn.location.strand(),
        Strand::Plus,
        "the drawn strand, expected +"
    );
    assert_eq!(drawn.open_before, Some(0), "open_before");
    assert_eq!(drawn.open_after, None, "open_after");

    // Imported: the shared parser still refuses it, unchanged (D041).
    match parse_genbank_location(&feature.location(), length, topology) {
        Err(DomainError::UnusableGenBankLocation(problem)) => {
            assert!(
                problem.rule == GenBankLocationRule::FuzzyEnd,
                "parse_genbank_location refuses {:?} with rule {:?}, expected fuzzy_end",
                feature.location(),
                problem.rule
            );
            assert_eq!(problem.location, "<1..12", "the refused location text");
        }
        Ok(location) => panic!(
            "parse_genbank_location accepts {:?}: intervals {:?}",
            feature.location(),
            location.intervals()
        ),
        Err(other) => panic!(
            "parse_genbank_location refuses {:?} as {:?}, expected unusable_genbank_location",
            feature.location(),
            other
        ),
    }
}
