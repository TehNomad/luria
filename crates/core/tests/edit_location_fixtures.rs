//! The editor's location reader and `buffer_for` against the records already
//! in `tests/fixtures/` (contract 9b2 §5 tests 3 and 4).
//!
//! Test 3 (`agrees_with_the_display_parser`) walks every feature of
//! `genbank/raw/J01749.1.gb`, `genbank/synthetic.gb`,
//! `genbank/synthetic-linear.gb` and `view/frames-62.gb` that the view's own
//! reader, `parse_display_location`, accepts, and asks that the editor's
//! reader give the same shape: the same strand, the same ranges, and the same
//! two open ends. It is the check that the editor reads what the view already
//! draws, and reads it the same way (M9-17, D036, D006); features the view
//! refuses — a bare position, an `order(…)` join, a plain exon join, a remote
//! reference — are the ones the editor exists for, and they are not compared.
//! The expectations are the display parser's own, which D053 and the shared
//! science parser pin, and are not derived from running luria.
//!
//! Test 4 (`pbr322_opens_for_editing`) asks the other question: a record the
//! view can open is a record the editor can open, feature by feature, keeping
//! each feature's location text exactly as the file wrote it (M9-7, M9-18).
//! pBR322's `misc_difference 426` (`J01749.1.gb:284`) is written as one bare
//! position and is the one base that position names (`genbank/README.md`).

use luria_core::display_location::parse_display_location;
use luria_core::display_location::DisplayLocation;
use luria_core::edit_buffer::buffer_for;
use luria_core::edit_buffer::EditBuffer;
use luria_core::edit_buffer::EditFeature;
use luria_core::edit_buffer::EditLocation;
use luria_core::edit_location::read_edit_location;
use luria_core::error::DomainError;
use luria_core::genbank::read_genbank;
use luria_core::genbank::GenBankFeature;
use luria_core::genbank::GenBankRecord;
use luria_core::identity::Identifier;
use luria_core::sequence::Sequence;
use luria_core::sequence::SequenceTopology;
use luria_core::view_record::open_record;
use luria_core::view_record::ViewRecord;

const GENBANK: &str = "../../tests/fixtures/genbank/";
const VIEW: &str = "../../tests/fixtures/view/";
const PBR322_FILE: &str = "raw/J01749.1.gb";

/// The four records test 3 walks, each with the directory it sits in.
const WALKED: [(&str, &str); 4] = [
    (GENBANK, "raw/J01749.1.gb"),
    (GENBANK, "synthetic.gb"),
    (GENBANK, "synthetic-linear.gb"),
    (VIEW, "frames-62.gb"),
];

/// A fixture input, read as it sits on disk.
fn read_input(dir: &str, file: &str) -> String {
    let path: String = format!("{}{}", dir, file);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("cannot read {}: {}", path, error))
}

/// The sequence identifier a fixture record is read as: the identity comes
/// from the caller, as in every other reader.
fn name(value: &str) -> Identifier {
    Identifier::create(value.to_string())
        .unwrap_or_else(|error| panic!("cannot use {:?} as an identifier: {:?}", value, error))
}

/// The record as `read_genbank` reads it, because test 3 compares features,
/// which the view keeps at one remove.
fn genbanked(dir: &str, file: &str) -> GenBankRecord {
    read_genbank(name(file), read_input(dir, file)).unwrap_or_else(|error| {
        panic!(
            "{}{}: read_genbank refused it as {}: {}",
            dir,
            file,
            error.code(),
            error.message()
        )
    })
}

/// The record `open_record` builds for a fixture file.
fn opened(dir: &str, file: &str) -> ViewRecord {
    open_record(name(file), read_input(dir, file)).unwrap_or_else(|error| {
        panic!(
            "{}{}: open_record refused it as {}: {}",
            dir,
            file,
            error.code(),
            error.message()
        )
    })
}

/// The rule that refused, named for the message a refusal prints.
fn rule_of(error: &DomainError) -> &'static str {
    match error {
        DomainError::UnusableGenBankLocation(problem) => problem.rule.code(),
        other => other.code(),
    }
}

/// What the editor's reader answers for one feature, named for the point at
/// which it failed.
fn read_for(feature: &GenBankFeature, length: u64, topology: &SequenceTopology) -> EditLocation {
    read_edit_location(feature.location().as_str(), length, topology.clone()).unwrap_or_else(
        |error| {
            panic!(
                "{} line {}: {:?} read for editing refused as {}: {}",
                feature.key(),
                feature.line(),
                feature.location(),
                rule_of(&error),
                error.message()
            )
        },
    )
}

/// Test 3: for every feature the view can draw, the editor reads the same
/// shape, and the open ends it records say the same thing the view's do.
#[test]
fn agrees_with_the_display_parser() {
    let mut compared: usize = 0;
    let mut not_drawn: usize = 0;

    for (dir, file) in WALKED {
        let record: GenBankRecord = genbanked(dir, file);
        let sequence: Sequence = record.sequence();
        let length: u64 = sequence.length();
        let topology: SequenceTopology = sequence.topology();

        for feature in record.features() {
            let display: DisplayLocation =
                match parse_display_location(feature.location().as_str(), length, topology.clone())
                {
                    Ok(display) => display,
                    // A feature the view cannot draw is one the editor's
                    // reader exists for, and says something else about: that
                    // is test 1's and test 5's business, not this test's.
                    Err(_) => {
                        not_drawn += 1;
                        continue;
                    }
                };

            let found: EditLocation = read_for(&feature, length, &topology);
            let ranges: Vec<(u64, u64)> = display.location.intervals();
            assert_eq!(
                found.strand,
                display.location.strand(),
                "{}{} line {}: {:?} strand",
                dir,
                file,
                feature.line(),
                feature.location()
            );
            assert_eq!(
                found.ranges,
                ranges,
                "{}{} line {}: {:?} ranges",
                dir,
                file,
                feature.line(),
                feature.location()
            );

            // The view records an open end as the base the mark names (D006,
            // D053); the editor records only whether an end is open, and on
            // which range the mark stood.
            let first: (u64, u64) = *ranges
                .first()
                .unwrap_or_else(|| panic!("{}{} line {}: no intervals", dir, file, feature.line()));
            let last: (u64, u64) = *ranges
                .last()
                .unwrap_or_else(|| panic!("{}{} line {}: no intervals", dir, file, feature.line()));
            assert_eq!(
                found.open_start,
                display.open_before == Some(first.0),
                "{}{} line {}: {:?} open_start",
                dir,
                file,
                feature.line(),
                feature.location()
            );
            assert_eq!(
                found.open_end,
                display.open_after == Some(last.1 - 1),
                "{}{} line {}: {:?} open_end",
                dir,
                file,
                feature.line(),
                feature.location()
            );

            compared += 1;
        }
    }

    assert!(compared > 0, "no location was compared at all");
    println!(
        "edit_location_fixtures: {} locations compared against the display parser, and {} refused by it",
        compared, not_drawn
    );
}

/// Test 4: pBR322 opens for editing, with every feature in file order and
/// each one's location text kept as the file wrote it (M9-7, M9-18).
#[test]
fn pbr322_opens_for_editing() {
    let record: ViewRecord = opened(GENBANK, PBR322_FILE);
    let genbank: &GenBankRecord = record
        .genbank
        .as_ref()
        .expect("a GenBank record opened from GenBank text");
    let buffer: EditBuffer = buffer_for(&record).unwrap_or_else(|blocked| {
        panic!(
            "pBR322 opens for editing, blocked at feature {} {} at line {}: {}",
            blocked.feature,
            blocked.key,
            blocked.line,
            rule_of(&blocked.error)
        )
    });

    assert_eq!(
        buffer.name(),
        record.sequence.name(),
        "the record's own name"
    );
    assert_eq!(buffer.length(), 4_361, "pBR322's length");
    assert_eq!(
        buffer.features().len(),
        genbank.features().len(),
        "every feature of the table"
    );

    for (index, feature) in genbank.features().iter().enumerate() {
        let found: &EditFeature = &buffer.features()[index];
        assert_eq!(found.key, feature.key(), "feature {}'s key", index);
        assert_eq!(
            found.location_text,
            Some(feature.location()),
            "feature {} keeps the location text the file wrote",
            index
        );
        assert_eq!(found.line, feature.line(), "feature {}'s line", index);
    }

    // `misc_difference 426` (`J01749.1.gb:284`) is written as one bare
    // position, and is the one base that position names (M9-18).
    let bare: Vec<&EditFeature> = buffer
        .features()
        .iter()
        .filter(|feature| {
            feature.key == "misc_difference" && feature.location_text.as_deref() == Some("426")
        })
        .collect();
    assert_eq!(bare.len(), 1, "one pBR322 misc_difference 426");
    assert_eq!(bare[0].location.ranges, vec![(425, 426)], "426 is one base");
    assert_eq!(bare[0].line, 284, "the line it is written on");
}
