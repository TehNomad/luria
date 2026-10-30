//! Contract 9c3 §5 tests 1 and 2: `write_genbank` against the two pBR322
//! records `tests/fixtures/editor/reference/saved.py` builds from
//! `tests/fixtures/genbank/raw/J01749.1.gb`, byte for byte (D039).
//!
//! Test 1 (`an_unedited_save_is_the_file`) saves a buffer that no edit
//! touched: every feature comes back as the file wrote it, the `VERSION`
//! line is gone, the save's own COMMENT line is the last line of the
//! COMMENT block, and the LOCUS line carries the stamp's date (M9-7,
//! M9-11, M9-15).
//!
//! Test 2 (`a_substitution_rewrites_only_its_cds`) saves the same buffer
//! after one substitution inside the tet CDS: that CDS's `/translation`
//! qualifier has gone, so the CDS is the one block rewritten, and every
//! other feature is still verbatim. Test 3 (`an_edited_save_reads_back`)
//! checks that an edited save is read back as the buffer that wrote it,
//! and test 4 (`locus_line_forms`) pins the LOCUS line's four shapes.

use luria_core::edit_buffer::buffer_for;
use luria_core::edit_buffer::Edit;
use luria_core::edit_buffer::EditBuffer;
use luria_core::edit_buffer::EditFeature;
use luria_core::edit_buffer::EditLocation;
use luria_core::genbank::read_genbank;
use luria_core::genbank::GenBankRecord;
use luria_core::genbank_write::write_genbank;
use luria_core::genbank_write::SaveStamp;
use luria_core::identity::Identifier;
use luria_core::view_record::open_record;
use luria_core::view_record::ViewRecord;

const GENBANK: &str = "../../tests/fixtures/genbank/";
const EDITOR: &str = "../../tests/fixtures/editor/";
const PBR322_FILE: &str = "raw/J01749.1.gb";
const SAVED_FILE: &str = "pbr322-saved.gb";
const SUBSTITUTED_FILE: &str = "pbr322-substituted.gb";
const READBACK: &str = "pbr322-edited.gb";

/// A fixture input, read as it sits on disk.
fn read_input(dir: &str, file: &str) -> String {
    let path: String = format!("{}{}", dir, file);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("cannot read {}: {}", path, error))
}

/// The sequence identifier a fixture record is read as.
fn name(value: &str) -> Identifier {
    Identifier::create(value.to_string())
        .unwrap_or_else(|error| panic!("cannot use {:?} as an identifier: {:?}", value, error))
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

/// The record as `write_genbank` writes it, with the contract's stamp.
fn saved(buffer: &EditBuffer, opened: &GenBankRecord, edits: usize) -> String {
    let stamp: SaveStamp = SaveStamp {
        date: "29-OCT-2026".to_string(),
        version: "9.9.9".to_string(),
        edits,
    };
    write_genbank(buffer, opened, &stamp)
}

/// pBR322's buffer and the GenBank record it was built from.
fn pbr322() -> (EditBuffer, GenBankRecord) {
    let record: ViewRecord = opened(GENBANK, PBR322_FILE);
    let genbank: GenBankRecord = record
        .genbank
        .clone()
        .expect("a GenBank record opened from GenBank text");
    let buffer: EditBuffer = buffer_for(&record).unwrap_or_else(|blocked| {
        panic!(
            "pBR322 opens for editing, blocked at feature {} {} at line {}: {}",
            blocked.feature,
            blocked.key,
            blocked.line,
            blocked.error.code()
        )
    });
    (buffer, genbank)
}

/// The base at a zero-based position of an ORIGIN block, as the file wrote
/// it, upper-cased: every line after the `ORIGIN` line and before `//`
/// holds a line number and ten-letter groups, so the letters are collected
/// in order. The file writes lower case, while an edit's bases are IUPAC
/// upper case (§4.4).
fn base_at(text: &str, at: usize) -> char {
    let mut letters: Vec<char> = Vec::new();
    let mut in_origin: bool = false;
    for line in text.lines() {
        if line.starts_with("ORIGIN") {
            in_origin = true;
            continue;
        }
        if !in_origin {
            continue;
        }
        if line.starts_with("//") {
            break;
        }
        letters.extend(line.chars().filter(|letter| letter.is_ascii_alphabetic()));
    }
    letters[at].to_ascii_uppercase()
}

/// The message a mismatch prints: the first differing line, both versions,
/// and nothing else (§9). Never the whole record.
fn first_difference(expected: &str, written: &str) -> String {
    let want: Vec<&str> = expected.split('\n').collect();
    let got: Vec<&str> = written.split('\n').collect();
    for at in 0..want.len().min(got.len()) {
        if want[at] != got[at] {
            return format!(
                "first difference at line {}: expected {:?}, written {:?}\nexpected \
                 {} lines, written {} lines",
                at + 1,
                want[at],
                got[at],
                want.len(),
                got.len()
            );
        }
    }
    format!(
        "the first {} lines match; expected {} lines, written {} lines",
        want.len().min(got.len()),
        want.len(),
        got.len()
    )
}

/// Test 1: the file with no edits removed is what an unedited save writes.
#[test]
fn an_unedited_save_is_the_file() {
    let expected: String = read_input(EDITOR, SAVED_FILE);
    let (buffer, genbank) = pbr322();
    let written: String = saved(&buffer, &genbank, 0);
    if written != expected {
        panic!(
            "an unedited pBR322 save is {}:\n{}",
            SAVED_FILE,
            first_difference(&expected, &written)
        );
    }
}

/// Test 2: one substitution rewrites only the CDS whose `/translation` the
/// fixture lacks, every other feature staying verbatim.
#[test]
fn a_substitution_rewrites_only_its_cds() {
    let expected: String = read_input(EDITOR, SUBSTITUTED_FILE);
    // The file's base 100 (zero-based 99) is what the substitution puts in.
    let new_base: char = base_at(&expected, 99);
    let (mut buffer, genbank) = pbr322();
    buffer
        .apply(&Edit::Substitute {
            start: 99,
            bases: vec![new_base],
        })
        .unwrap_or_else(|error| {
            panic!(
                "substituting base 100 with {:?}: {} {}",
                new_base,
                error.code(),
                error.message()
            )
        });
    let written: String = saved(&buffer, &genbank, 1);
    if written != expected {
        panic!(
            "a substituted pBR322 save is {}:\n{}",
            SUBSTITUTED_FILE,
            first_difference(&expected, &written)
        );
    }
}

/// The bases of a buffer as one string, so a mismatch prints readably.
fn letters_of(buffer: &EditBuffer) -> String {
    buffer.letters().iter().collect()
}

/// The first base where two buffers' letters differ, with a little context:
/// the whole 4354-base record is never printed (§9).
fn base_difference(expected: &str, read: &str) -> String {
    let want: Vec<char> = expected.chars().collect();
    let have: Vec<char> = read.chars().collect();
    match (0..want.len().max(have.len())).find(|index| want.get(*index) != have.get(*index)) {
        Some(at) => format!(
            "first differing base at zero-based {}: expected {:?} around {:?}, read {:?} around \
             {:?}",
            at,
            want.get(at),
            want[at.saturating_sub(10)..(at + 10).min(want.len())]
                .iter()
                .collect::<String>(),
            have.get(at),
            have[at.saturating_sub(10)..(at + 10).min(have.len())]
                .iter()
                .collect::<String>()
        ),
        None => format!(
            "the letters agree as far as they go: expected {} bases, read {}",
            want.len(),
            have.len()
        ),
    }
}

/// One feature's key, its `EditLocation` and its qualifiers: the shape a save
/// has to reproduce (M9-7). `location_text` and `line` are left out on
/// purpose, because the read-back record keeps its own location text for the
/// same ranges and its own line numbers.
type FeatureShape = (String, EditLocation, Vec<(String, Option<String>)>);

/// The shape of every buffer feature, in buffer order.
fn feature_shapes(features: &[EditFeature]) -> Vec<FeatureShape> {
    features
        .iter()
        .map(|feature| {
            (
                feature.key.clone(),
                feature.location.clone(),
                feature.qualifiers.clone(),
            )
        })
        .collect()
}

/// Test 3: a save with an insert and a delete reads back as the buffer that
/// wrote it: the same letters, and the same features in the same order.
#[test]
fn an_edited_save_reads_back() {
    let (mut buffer, genbank) = pbr322();
    buffer
        .apply(&Edit::Insert {
            at: 99,
            bases: vec!['A', 'C', 'G'],
        })
        .unwrap_or_else(|error| {
            panic!(
                "inserting ACG before base 100: {} {}",
                error.code(),
                error.message()
            )
        });
    buffer
        .apply(&Edit::Delete {
            start: 2000,
            end: 2010,
        })
        .unwrap_or_else(|error| {
            panic!(
                "deleting bases [2000, 2010): {} {}",
                error.code(),
                error.message()
            )
        });
    let written: String = saved(&buffer, &genbank, 2);

    // The written text is read back twice: as the record the file holds
    // (`read_genbank`), and as the buffer the editor would see (`open_record`
    // and `buffer_for`).
    let record: GenBankRecord =
        read_genbank(name(READBACK), written.clone()).unwrap_or_else(|error| {
            panic!(
                "{}: the edited save was refused as {}: {}",
                READBACK,
                error.code(),
                error.message()
            )
        });
    let read: ViewRecord = open_record(name(READBACK), written).unwrap_or_else(|error| {
        panic!(
            "{}: the edited save was refused as {}: {}",
            READBACK,
            error.code(),
            error.message()
        )
    });
    let back: EditBuffer = buffer_for(&read).unwrap_or_else(|blocked| {
        panic!(
            "{}: the edited save cannot be edited, blocked at feature {} {} at line {}: {}",
            READBACK,
            blocked.feature,
            blocked.key,
            blocked.line,
            blocked.error.code()
        )
    });

    assert_eq!(
        record.sequence().length(),
        buffer.length(),
        "{}: the edited save states a length of {}, not {}",
        READBACK,
        record.sequence().length(),
        buffer.length()
    );
    assert_eq!(
        record.features().len(),
        buffer.features().len(),
        "{}: the edited save holds {} features, not {}",
        READBACK,
        record.features().len(),
        buffer.features().len()
    );
    if letters_of(&back) != letters_of(&buffer) {
        panic!(
            "{}: the read-back bases are not the edited buffer's: {}",
            READBACK,
            base_difference(&letters_of(&buffer), &letters_of(&back))
        );
    }
    // Every feature comes back with the key, location and qualifiers the
    // buffer held, in the same order.
    let want: Vec<FeatureShape> = feature_shapes(buffer.features());
    let have: Vec<FeatureShape> = feature_shapes(back.features());
    for index in 0..want.len().min(have.len()) {
        assert_eq!(
            have[index],
            want[index],
            "{}: feature {} reads back differently from the buffer it was written from",
            READBACK,
            index + 1
        );
    }
}

/// The first line of a save: the LOCUS line, for test 4.
fn locus_of(buffer: &EditBuffer, opened: &GenBankRecord) -> String {
    let text: String = saved(buffer, opened, 0);
    match text.lines().next() {
        Some(line) => line.to_string(),
        None => panic!("the writer wrote no LOCUS line"),
    }
}

/// A hand-written one-record GenBank text: `locus` is its LOCUS line and
/// `bases` its ORIGIN, so the reader sees only the parts test 4 needs.
fn mini(locus: &str, bases: &str) -> String {
    format!("{locus}\nDEFINITION  a synthetic record.\nORIGIN\n        1 {bases}\n//\n")
}

/// The buffer and the opened record for a hand-written GenBank text.
fn mini_record(text: &str) -> (EditBuffer, GenBankRecord) {
    let record: ViewRecord = open_record(name("mini"), text.to_string()).unwrap_or_else(|error| {
        panic!(
            "the hand-written record was refused as {}: {}",
            error.code(),
            error.message()
        )
    });
    let genbank: GenBankRecord = record
        .genbank
        .clone()
        .expect("a GenBank record opened from GenBank text");
    let buffer: EditBuffer = buffer_for(&record).unwrap_or_else(|blocked| {
        panic!(
            "the hand-written record cannot be edited, blocked at feature {} {}: {}",
            blocked.feature,
            blocked.key,
            blocked.error.code()
        )
    });
    (buffer, genbank)
}

/// Test 4: the four LOCUS line shapes of §4 rule 1, each pinned with a
/// literal written from that rule by hand.
#[test]
fn locus_line_forms() {
    // pBR322: the line the file has, but for the stamp's date.
    let (buffer, genbank) = pbr322();
    assert_eq!(
        locus_of(&buffer, &genbank),
        "LOCUS       SYNPBR322               4361 bp    DNA     circular SYN 29-OCT-2026"
    );

    // `ds-DNA` splits after the `-`, and the division is the file's own.
    let (buffer, genbank) = mini_record(&mini(
        "LOCUS       DSDNA1                  10 bp  ds-DNA      linear  PLN 01-JAN-2024",
        "atgcgcgtat",
    ));
    assert_eq!(
        locus_of(&buffer, &genbank),
        "LOCUS       DSDNA1                    10 bp ds-DNA     linear   PLN 29-OCT-2026"
    );

    // No molecule and no division: `DNA` and `UNK`.
    let (buffer, genbank) = mini_record(&mini(
        "LOCUS       NOBACT                    10 bp  linear",
        "gattacgcga",
    ));
    assert_eq!(
        locus_of(&buffer, &genbank),
        "LOCUS       NOBACT                    10 bp    DNA     linear   UNK 29-OCT-2026"
    );

    // A name past 16 characters pushes the rest right, keeping one space.
    let (buffer, genbank) = mini_record(&mini(
        "LOCUS       ABCDEFGHIJKLMNOP1234      12 bp  DNA       linear  SYN 01-JAN-2024",
        "atgcatgcatgc",
    ));
    assert_eq!(
        locus_of(&buffer, &genbank),
        "LOCUS       ABCDEFGHIJKLMNOP1234          12 bp    DNA     linear   SYN 29-OCT-2026"
    );
}
