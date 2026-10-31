// §5 tests 1–3 of contract 9e: `rebuild` of `crates/core/src/view_record.rs`
// gives the record `open_record` would give the file that the edit describes,
// because `open_record`'s GenBank arm and the rebuilt record share the same
// feature, CDS and ceiling code (M9-6 i).
//
// Test 1 opens pBR322 (`genbank/raw/J01749.1.gb`) and the 62 bp origin-joint
// record `view/frames-62.gb`, makes a buffer with no edit, and rebuilds: the
// rebuilt record must equal the opened one in `sequence` and in the features
// (key, name, line, shape) and CDSs (feature, id, status). The expectations
// are the opened records' own values, which `view_record_fixtures.rs` and
// `frames_fixtures.rs` check against the fixture text and tables; `rebuild`'s
// output is not the source of any expectation here.
//
// Test 2 writes two records by hand: one with a usable CDS, the other the
// same record with one base inserted at base 5 and its CDS `1..13`. It
// checks that rebuilding the first's buffer after `Insert { at: 4 }`
// gives the CDS the status `open_record` gives the second: `Unusable`, with
// the same refusal.
//
// Test 3 builds a FASTA buffer at the ceiling, inserts one base, and checks
// `rebuild` gives the `RecordTooLong` `open_record` gives at 10 000 001 bases.

use luria_core::edit_buffer::buffer_for;
use luria_core::edit_buffer::Edit;
use luria_core::edit_buffer::EditBuffer;
use luria_core::error::DomainError;
use luria_core::identity::Identifier;
use luria_core::view_record::open_record;
use luria_core::view_record::rebuild;
use luria_core::view_record::CdsStatus;
use luria_core::view_record::FeatureShape;
use luria_core::view_record::ViewCds;
use luria_core::view_record::ViewFeature;
use luria_core::view_record::ViewRecord;
use luria_core::view_record::MAX_RECORD_BASES;

const GENBANK: &str = "../../tests/fixtures/genbank/";
const VIEW: &str = "../../tests/fixtures/view/";
const PBR322_FILE: &str = "raw/J01749.1.gb";
const FRAMES62_FILE: &str = "frames-62.gb";

/// §5 test 2's first record: 30 bp with a usable `CDS 1..12` (ATG ACG CTG TAA)
/// and a `misc_feature complement(21..24)` inside the `CTAG` repeat run.
const BEFORE: &str = r##"LOCUS       shift                     30 bp    DNA     linear   SYN 01-NOV-2026
FEATURES             Location/Qualifiers
     source          1..30
                     /mol_type="other DNA"
     CDS             1..12
                     /gene="shift"
     misc_feature    complement(21..24)
ORIGIN
        1 atgacgctgt aagctagcta gctagctagc
//
"##;

/// The same record as a file: one `C` written after the fourth base, inside
/// codon 2, the CDS grown to `1..13` (thirteen bases, not a whole of three
/// codons) and the repeat feature grown to `22..25`.
const AFTER: &str = r##"LOCUS       shift                     31 bp    DNA     linear   SYN 01-NOV-2026
FEATURES             Location/Qualifiers
     source          1..31
                     /mol_type="other DNA"
     CDS             1..13
                     /gene="shift"
     misc_feature    complement(22..25)
ORIGIN
        1 atgaccgctg taagctagct agctagctag c
//
"##;

/// A fixture input, read as it sits on disk.
fn read_input(dir: &str, file: &str) -> String {
    let path: String = format!("{}{}", dir, file);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("cannot read {}: {}", path, error))
}

/// The identifier a record is read as, as in every other reader (D007).
fn name(value: &str) -> Identifier {
    Identifier::create(value.to_string())
        .unwrap_or_else(|error| panic!("cannot use {:?} as an identifier: {:?}", value, error))
}

/// A record opened from a fixture, refusing out loud if it will not read.
fn open(dir: &str, file: &str) -> ViewRecord {
    let label: String = format!("{}{}", dir, file);
    open_record(name(label.as_str()), read_input(dir, file)).unwrap_or_else(|error| {
        panic!(
            "{} was refused as {}: {}",
            label,
            error.code(),
            error.message()
        )
    })
}

/// The buffer of a record, refusing out loud if it cannot be edited.
fn make_buffer(record: &ViewRecord) -> EditBuffer {
    match buffer_for(record) {
        Ok(buffer) => buffer,
        Err(blocked) => panic!(
            "the feature {} on line {} cannot be an edit location: {}",
            blocked.key,
            blocked.line,
            blocked.error.code()
        ),
    }
}

/// The CDS that carries the given D041 id, refusing out loud with the ids the
/// record does give.
fn find_cds(record: &ViewRecord, id: &str) -> ViewCds {
    for cds in record.cdss.iter() {
        if cds.id.as_deref() == Some(id) {
            return cds.clone();
        }
    }
    let given: Vec<String> = record
        .cdss
        .iter()
        .filter_map(|cds| cds.id.clone())
        .collect();
    panic!("no CDS id {id:?}: the record ids {:?}", given);
}

/// §5 test 1: the feature list, feature for feature, as §5's columns ask.
fn check_features_unchanged(opened: &ViewRecord, rebuilt: &ViewRecord, label: &str) {
    assert_eq!(
        opened.features.len(),
        rebuilt.features.len(),
        "{label}: the rebuild changed the number of features"
    );
    for pair in opened.features.iter().zip(rebuilt.features.iter()) {
        let (want, got): (&ViewFeature, &ViewFeature) = (pair.0, pair.1);
        let item: String = format!("{}, feature {}", label, want.index);
        assert_eq!(got.index, want.index, "{item}: the index changed");
        assert_eq!(got.key, want.key, "{item}: the key changed");
        assert_eq!(got.name, want.name, "{item}: the display name changed");
        assert_eq!(got.line, want.line, "{item}: the key line changed");
        assert_eq!(got.shape, want.shape, "{item}: the shape changed");
    }
}

/// §5 test 1: the CDS list, CDS for CDS, as §5's columns ask.
fn check_cdss_unchanged(opened: &ViewRecord, rebuilt: &ViewRecord, label: &str) {
    assert_eq!(
        opened.cdss.len(),
        rebuilt.cdss.len(),
        "{label}: the rebuild changed the number of CDSs"
    );
    for pair in opened.cdss.iter().zip(rebuilt.cdss.iter()) {
        let (want, got): (&ViewCds, &ViewCds) = (pair.0, pair.1);
        let id: String = match want.id.as_ref() {
            Some(id) => id.clone(),
            None => format!("feature {}", want.feature),
        };
        let item: String = format!("{}, CDS {}", label, id);
        assert_eq!(
            got.feature, want.feature,
            "{item}: the feature index changed"
        );
        assert_eq!(got.id, want.id, "{item}: the D041 id changed");
        assert_eq!(got.status, want.status, "{item}: the status changed");
    }
}

#[test]
fn an_unedited_rebuild_is_the_opened_record() {
    let files: [(&str, &str); 2] = [(GENBANK, PBR322_FILE), (VIEW, FRAMES62_FILE)];
    for (dir, file) in files {
        let opened: ViewRecord = open(dir, file);
        let buffer: EditBuffer = make_buffer(&opened);
        let rebuilt: ViewRecord = rebuild(&buffer, &opened).unwrap_or_else(|error| {
            panic!(
                "{file} rebuilt record was refused as {}: {}",
                error.code(),
                error.message()
            )
        });
        let label: String = file.to_string();

        assert_eq!(rebuilt.format, opened.format, "{label}: the format changed");
        assert_eq!(
            rebuilt.topology, opened.topology,
            "{label}: the topology source changed"
        );
        assert_eq!(
            rebuilt.sequence, opened.sequence,
            "{label}: the sequence changed"
        );
        assert!(
            rebuilt.genbank.is_none(),
            "{label}: a rebuilt record keeps a GenBank record nothing reads"
        );
        check_features_unchanged(&opened, &rebuilt, &label);
        check_cdss_unchanged(&opened, &rebuilt, &label);
    }
}

#[test]
fn a_frameshift_matches_open() {
    let before: ViewRecord = open_record(name("shift"), BEFORE.to_string())
        .unwrap_or_else(|error| panic!("BEFORE refused as {}: {}", error.code(), error.message()));
    let after: ViewRecord = open_record(name("shift"), AFTER.to_string())
        .unwrap_or_else(|error| panic!("AFTER refused as {}: {}", error.code(), error.message()));

    let before_cds: ViewCds = find_cds(&before, "shift");
    let after_cds: ViewCds = find_cds(&after, "shift");
    assert!(
        matches!(before_cds.status, CdsStatus::Usable(_)),
        "BEFORE's CDS should be usable: {:?}",
        before_cds.status
    );
    assert!(
        matches!(after_cds.status, CdsStatus::Unusable(_)),
        "AFTER's CDS should be unusable: {:?}",
        after_cds.status
    );

    let mut buffer: EditBuffer = make_buffer(&before);
    buffer
        .apply(&Edit::Insert {
            at: 4,
            bases: vec!['C'],
        })
        .unwrap_or_else(|error| panic!("the insert was refused as {}", error.code()));

    let rebuilt: ViewRecord = rebuild(&buffer, &before).unwrap_or_else(|error| {
        panic!(
            "the rebuild was refused as {}: {}",
            error.code(),
            error.message()
        )
    });
    let rebuilt_cds: ViewCds = find_cds(&rebuilt, "shift");
    let rebuilt_shape: &FeatureShape = &rebuilt.features[rebuilt_cds.feature].shape;
    let after_shape: &FeatureShape = &after.features[after_cds.feature].shape;

    assert!(
        matches!(rebuilt_cds.status, CdsStatus::Unusable(_)),
        "the rebuilt CDS should be unusable: {:?}",
        rebuilt_cds.status
    );
    match (&rebuilt_cds.status, &after_cds.status) {
        (CdsStatus::Unusable(got), CdsStatus::Unusable(want)) => {
            assert_eq!(
                got.code(),
                want.code(),
                "the rebuild refused with {} , AFTER's open refused with {}",
                got.code(),
                want.code()
            );
            assert_eq!(
                got.message(),
                want.message(),
                "the rebuild's refusal message differs from the open refusal's"
            );
        }
        (got, want) => panic!("expected two Unusable statuses, got {got:?} and {want:?}"),
    }
    assert_eq!(
        after_shape, rebuilt_shape,
        "the rebuilt CDS location shape differs from the open record's"
    );
}

#[test]
fn rebuild_keeps_the_ceiling() {
    let bases: String = "A".repeat(MAX_RECORD_BASES as usize);
    let at_ceiling: ViewRecord = open_record(name("ceiling"), format!(">ceiling\n{bases}\n"))
        .unwrap_or_else(|error| panic!("10 000 000 bases refused as {}", error.code()));
    assert_eq!(
        at_ceiling.sequence.length(),
        MAX_RECORD_BASES,
        "the test record is not at the ceiling"
    );

    let mut buffer: EditBuffer = make_buffer(&at_ceiling);
    buffer
        .apply(&Edit::Insert {
            at: 0,
            bases: vec!['A'],
        })
        .unwrap_or_else(|error| panic!("the insert was refused as {}", error.code()));
    assert_eq!(
        buffer.length(),
        MAX_RECORD_BASES + 1,
        "the buffer is not one base past the ceiling"
    );

    let rebuilt_error: DomainError = match rebuild(&buffer, &at_ceiling) {
        Ok(record) => panic!(
            "a rebuild of {} bases was accepted",
            record.sequence.length()
        ),
        Err(error) => error,
    };
    let open_error: DomainError =
        match open_record(name("ceiling"), format!(">ceiling\n{bases}A\n")) {
            Ok(record) => panic!(
                "open_record of {} bases was accepted",
                record.sequence.length()
            ),
            Err(error) => error,
        };
    assert_eq!(
        open_error.code(),
        "record_too_long",
        "the open ceiling refusal changed"
    );
    assert_eq!(
        rebuilt_error.code(),
        open_error.code(),
        "rebuild refused with {}, open refused with {}",
        rebuilt_error.code(),
        open_error.code()
    );
    assert_eq!(
        rebuilt_error.message(),
        open_error.message(),
        "rebuild and open name the refusal differently"
    );
}
