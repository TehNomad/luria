//! The GenBank reader in `crates/core/src/genbank.rs` (5c2) and the CDS
//! import in `crates/core/src/genbank_cds.rs` (5c3) against
//! `tests/fixtures/genbank/`: pBR322 (NCBI J01749.1) as Biopython reads it,
//! and a hand-written synthetic record with one CDS per accepted location
//! form and per refusal (D041 G1, G2, G3, G5; D006, D036).
//!
//! `reference/pbr322.py` reads the public record with Biopython 1.88 and
//! `reference/verify_synthetic.py` re-derives every biological column of the
//! synthetic tables; `tests/fixtures/genbank/README.md` records how each
//! value was derived and checked. The scripts run from the repository root,
//! not from cargo, in the pinned Python environment:
//!
//! ```
//! python -I tests/fixtures/verify.py
//! ```
//!
//! That Python check is not part of the required cargo checks (D039, D026).

use luria_core::cds::Cds;
use luria_core::cds::Completeness;
use luria_core::cds::Numbering;
use luria_core::error::DomainError;
use luria_core::error::GenBankCdsRule;
use luria_core::error::GenBankLocationRule;
use luria_core::extraction::extract_cds;
use luria_core::genbank::read_genbank;
use luria_core::genbank::GenBankFeature;
use luria_core::genbank::GenBankRecord;
use luria_core::genbank_cds::import_cds;
use luria_core::genbank_cds::GenBankImport;
use luria_core::genbank_cds::UnusedFeature;
use luria_core::genbank_location::parse_genbank_location;
use luria_core::genbank_location::GenBankLocation;
use luria_core::genetic_code::STANDARD_CODE_NAME;
use luria_core::identity::Identifier;
use luria_core::interval::create;
use luria_core::interval::OrderedIntervals;
use luria_core::sequence::Sequence;
use luria_core::strand::Strand;
use luria_core::translation::translate;

const FIXTURES: &str = "../../tests/fixtures/genbank/";
const RECORD_HEADER: &str = "locus_name\taccession\tversion\tlength\ttopology\tsha256\tfeatures";
const FEATURE_HEADER: &str = "index\tkey\tid\tid_source";
const CDS_HEADER: &str = "index\tid\tid_source\tstart\tend\tstrand\tcodon_start\ttransl_table\tstart_codon\tresidues\ttranslation_matches";
const EXPECTED_HEADER: &str =
    "name\terror\trule\tintervals\tstrand\tid_qualifier\ttransl_table\tresidues\tdetail\tline";
const UNUSED_HEADER: &str = "index\tkey\tid\trefusal";
const PBR322_FILE: &str = "raw/J01749.1.gb";
const SYNTHETIC_FILE: &str = "synthetic.gb";

/// The rows of a fixture TSV past its header row, refusing a header row it
/// does not have.
fn table(file: &str, header: &str) -> Vec<Vec<String>> {
    let path: String = format!("{}{}", FIXTURES, file);
    let text: String = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
    let lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    assert!(
        !lines.is_empty(),
        "{}: expected header row {:?}, none found",
        path,
        header
    );
    assert!(
        lines[0] == header,
        "{}: first row {:?}, expected {:?}",
        path,
        lines[0],
        header
    );
    lines[1..]
        .iter()
        .map(|line| line.split('\t').map(|field| field.to_string()).collect())
        .collect()
}

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

/// The identifiers of a request, in the order the request gives them.
fn request(values: Vec<String>) -> Vec<Identifier> {
    values
        .iter()
        .map(|value| {
            Identifier::create(value.clone()).unwrap_or_else(|error| {
                panic!("cannot use {:?} as an identifier: {:?}", value, error)
            })
        })
        .collect()
}

/// pBR322, read from `raw/J01749.1.gb`.
fn pbr322() -> GenBankRecord {
    read_genbank(name(PBR322_FILE), read_input(PBR322_FILE)).unwrap_or_else(|error| {
        panic!(
            "{}: read_genbank refused it: {:?}",
            PBR322_FILE,
            error.message()
        )
    })
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

/// The id text of a feature under D041 G2: the first of `/locus_tag`, `/gene`
/// and `/label` that appears among the qualifiers, in that priority order, and
/// the value of its first occurrence. `-` is what the fixture tables write for
/// a feature that has none. A first occurrence with no value means no id:
/// the search stops at the first qualifier of the three that appears, as
/// luria's `id_of` does, and does not fall through to the next name.
fn fixture_id(feature: GenBankFeature) -> String {
    for wanted in ["locus_tag", "gene", "label"] {
        for qualifier in feature.qualifiers().iter() {
            if qualifier.name() == wanted {
                match qualifier.value() {
                    None => return "-".to_string(),
                    Some(value) => return value,
                }
            }
        }
    }
    "-".to_string()
}

/// The value of the first occurrence of the qualifier `/name`, or `-` as the
/// fixture tables write it when the qualifier is absent or carries no value.
fn first_qualifier(feature: GenBankFeature, name: String) -> String {
    for qualifier in feature.qualifiers().iter() {
        if qualifier.name() == name {
            return or_dash(qualifier.value());
        }
    }
    "-".to_string()
}

/// The intervals of a CDS as `synthetic-expected.tsv` writes them: `start-end`
/// pairs joined by `,`.
fn intervals_text(cds: &Cds) -> String {
    let parts: Vec<String> = cds
        .intervals()
        .iter()
        .map(|interval| format!("{}-{}", interval.start(), interval.end()))
        .collect();
    parts.join(",")
}

/// The strand as `synthetic-expected.tsv` writes it.
fn strand_text(cds: &Cds) -> String {
    let strand: Strand = cds.strand();
    if strand.is_plus() {
        "plus".to_string()
    } else {
        "minus".to_string()
    }
}

/// The value an option carries, or `-` as the fixture tables write it.
fn or_dash(value: Option<String>) -> String {
    match value {
        None => "-".to_string(),
        Some(found) => found,
    }
}

/// The four columns a refusal fills in `synthetic-expected.tsv` (§5.1): the
/// error's code, the rule that refused it where a rule exists, the detail and
/// the line, each `-` where the table writes `-`.
fn error_columns(error: DomainError) -> (String, String, String, String) {
    let code: String = error.code().to_string();
    match error {
        DomainError::UnusableGenBankCds(problem) => {
            let line: String = match problem.line {
                None => "-".to_string(),
                Some(found) => found.to_string(),
            };
            (
                code,
                problem.rule.code().to_string(),
                or_dash(problem.detail),
                line,
            )
        }
        DomainError::UnusableGenBankLocation(problem) => (
            code,
            problem.rule.code().to_string(),
            "-".to_string(),
            "-".to_string(),
        ),
        DomainError::GenBankTranslationMismatch(problem) => {
            let stated: String = or_dash(problem.stated);
            let translated: String = or_dash(problem.translated);
            (
                code,
                "-".to_string(),
                format!("{}:{}:{}", problem.residue, stated, translated),
                problem.line.to_string(),
            )
        }
        DomainError::InternalStopCodon(problem) => (
            code,
            "-".to_string(),
            problem.codon_number.to_string(),
            "-".to_string(),
        ),
        DomainError::CdsLengthNotDivisibleByThree(problem) => (
            code,
            "-".to_string(),
            problem.length.to_string(),
            "-".to_string(),
        ),
        other => (
            other.code().to_string(),
            "-".to_string(),
            "-".to_string(),
            "-".to_string(),
        ),
    }
}

/// The `refusal` column of the not-used tables: `-` for nothing, the error's
/// code, or `code:rule` when the error carries a rule.
fn refusal_text(refusal: Option<DomainError>) -> String {
    match refusal {
        None => "-".to_string(),
        Some(error) => {
            let code: String = error.code().to_string();
            match error {
                DomainError::UnusableGenBankCds(problem) => {
                    format!("{}:{}", code, problem.rule.code())
                }
                DomainError::UnusableGenBankLocation(problem) => {
                    format!("{}:{}", code, problem.rule.code())
                }
                other => other.code().to_string(),
            }
        }
    }
}

/// luria's translation of the named CDS feature, built the way §4.2 steps 8 to
/// 10 build it: the location parsed against the record's sequence, the pairs as
/// intervals, and `translate` under the standard code. An import of that
/// feature refuses at its `/translation` (§4.2 step 12), so this is how the
/// test asks luria what the CDS reads as; the fixture, not this test, states
/// the answer.
fn luria_residues(record: GenBankRecord, wanted: String) -> String {
    let sequence: Sequence = record.sequence();
    for feature in record.features().iter() {
        if feature.key() != "CDS" {
            continue;
        }
        if fixture_id(feature.clone()) != wanted {
            continue;
        }
        let location: GenBankLocation =
            parse_genbank_location(&feature.location(), sequence.length(), sequence.topology())
                .unwrap_or_else(|error| {
                    panic!(
                        "the CDS {:?} has location {:?}, which luria refuses: {:?}",
                        wanted,
                        feature.location(),
                        error.message()
                    )
                });
        let intervals: OrderedIntervals = location
            .intervals()
            .iter()
            .map(|pair| {
                create(&sequence.reference(), pair.0 as i64, pair.1 as i64)
                    .unwrap_or_else(|error| panic!("cannot build the interval: {:?}", error))
            })
            .collect();
        let cds: Cds = Cds::create(
            &sequence,
            name(wanted.as_str()).clone(),
            intervals,
            location.strand(),
            Completeness::Complete,
            STANDARD_CODE_NAME.to_string(),
            Numbering::FromCdsStart,
        )
        .unwrap_or_else(|error| panic!("cannot build the CDS: {:?}", error));
        return translate(&sequence, &cds)
            .unwrap_or_else(|error| panic!("cannot translate the CDS: {:?}", error))
            .residues_text();
    }
    panic!("{}: no CDS feature has the id {:?}", SYNTHETIC_FILE, wanted)
}

#[test]
fn pbr322_reads_as_biopython_reads_it() {
    let record: GenBankRecord = pbr322();
    let rows: Vec<Vec<String>> = table("pbr322-record.tsv", RECORD_HEADER);
    assert!(
        rows.len() == 1,
        "{}: {} rows, expected 1",
        "pbr322-record.tsv",
        rows.len()
    );
    let row: Vec<String> = rows[0].clone();
    assert!(
        record.locus_name() == row[0],
        "the reader gives locus name {:?}, the fixture records {:?}",
        record.locus_name(),
        row[0]
    );
    assert!(
        or_dash(record.accession()) == row[1],
        "the reader gives accession {}, the fixture records {}",
        or_dash(record.accession()),
        row[1]
    );
    assert!(
        or_dash(record.version()) == row[2],
        "the reader gives version {}, the fixture records {}",
        or_dash(record.version()),
        row[2]
    );
    assert!(
        record.sequence().length().to_string() == row[3],
        "the reader gives length {}, the fixture records {}",
        record.sequence().length(),
        row[3]
    );
    assert!(
        record.sequence().topology().name() == row[4],
        "the reader gives topology {:?}, the fixture records {:?}",
        record.sequence().topology().name(),
        row[4]
    );
    assert!(
        record.sequence().content_hash().hex_digest() == row[5],
        "the reader gives digest {:?}, the fixture records {:?}",
        record.sequence().content_hash().hex_digest(),
        row[5]
    );
    let features: Vec<GenBankFeature> = record.features();
    assert!(
        features.len().to_string() == row[6],
        "the reader gives {} features, the fixture records {}",
        features.len(),
        row[6]
    );

    let feature_rows: Vec<Vec<String>> = table("pbr322-features.tsv", FEATURE_HEADER);
    assert!(
        feature_rows.len() == features.len(),
        "{}: {} rows, expected {} as the reader gives",
        "pbr322-features.tsv",
        feature_rows.len(),
        features.len()
    );
    for position in 0..features.len() {
        assert!(
            feature_rows[position][0] == position.to_string(),
            "{}: row {:?} is not index {}",
            "pbr322-features.tsv",
            feature_rows[position],
            position
        );
        assert!(
            features[position].key() == feature_rows[position][1],
            "the feature at index {} has key {:?}, the fixture records {:?}",
            position,
            features[position].key(),
            feature_rows[position][1]
        );
    }
}

// Row 26 (the ROP CDS) is a row of `pbr322-cds.tsv` this test does not
// compare: luria refuses that feature at §4.2 step 1 (`NoId`), so it has no
// imported CDS of its own. The row's `start_codon` (GTG) and `translation_matches`
// (no) are Biopython's facts about the record with no luria counterpart.
#[test]
fn pbr322_imports_tet_and_bla_as_biopython_reads_them() {
    let record: GenBankRecord = pbr322();
    let wanted: Vec<String> = vec!["tet".to_string(), "bla".to_string()];
    let import: GenBankImport =
        import_cds(&record, &request(wanted.clone())).unwrap_or_else(|error| {
            panic!(
                "{}: the import refused it: {:?}",
                PBR322_FILE,
                error.message()
            )
        });
    assert!(
        import.cds().len() == wanted.len(),
        "the import gives {} CDSs, expected {}",
        import.cds().len(),
        wanted.len()
    );
    let rows: Vec<Vec<String>> = table("pbr322-cds.tsv", CDS_HEADER);
    for pair in wanted.iter().enumerate() {
        let position: usize = pair.0;
        let name: String = pair.1.to_string();
        let row: Vec<String> = rows
            .iter()
            .find(|candidate| candidate[1] == name)
            .unwrap()
            .clone();
        let imported = import.cds()[position].clone();
        assert!(
            imported.index().to_string() == row[0],
            "the CDS at request position {} has index {}, the fixture records {}",
            position,
            imported.index(),
            row[0]
        );
        let intervals: OrderedIntervals = imported.cds().intervals();
        assert!(
            intervals.len() == 1,
            "the CDS {} has {} intervals, expected 1",
            wanted[position],
            intervals.len()
        );
        assert!(
            intervals[0].start().to_string() == row[3] && intervals[0].end().to_string() == row[4],
            "the CDS {} runs [{}, {}), the fixture records [{}, {})",
            wanted[position],
            intervals[0].start(),
            intervals[0].end(),
            row[3],
            row[4]
        );
        let fixture_strand: String = if row[5] == "+" {
            "plus".to_string()
        } else {
            "minus".to_string()
        };
        assert!(
            strand_text(&imported.cds()) == fixture_strand,
            "the CDS {} is on strand {}, the fixture records {:?}",
            wanted[position],
            strand_text(&imported.cds()),
            row[5]
        );
        assert!(
            imported.id_qualifier().name() == row[2],
            "the CDS {} takes its id from {}, the fixture records {}",
            wanted[position],
            imported.id_qualifier().name(),
            row[2]
        );
        assert!(
            or_dash(imported.transl_table()) == row[7],
            "the CDS {} keeps /transl_table {}, the fixture records {}",
            wanted[position],
            or_dash(imported.transl_table()),
            row[7]
        );
        let count: usize = translate(&record.sequence(), &imported.cds())
            .unwrap_or_else(|error| panic!("cannot translate the CDS: {:?}", error))
            .residue_count();
        assert!(
            count.to_string() == row[9],
            "the CDS {} reads {} residues, the fixture records {}",
            wanted[position],
            count,
            row[9]
        );
        let feature: GenBankFeature = record.features()[imported.index()].clone();
        let codon_start: String = first_qualifier(feature.clone(), "codon_start".to_string());
        assert!(
            codon_start == row[6],
            "{}: the CDS {} reads /codon_start {}, the fixture records {}",
            "pbr322-cds.tsv",
            name,
            codon_start,
            row[6]
        );
        let coding: String = extract_cds(&record.sequence(), &imported.cds())
            .unwrap_or_else(|error| panic!("cannot extract the CDS: {:?}", error));
        let start: String = coding.chars().take(3).collect();
        assert!(
            start == row[8],
            "{}: the CDS {} starts its coding text with {}, the fixture records {}",
            "pbr322-cds.tsv",
            name,
            start,
            row[8]
        );
        assert!(
            row[10] == "yes",
            "{}: the imported CDS {} has translation_matches {}, the only value an imported CDS can have is yes",
            "pbr322-cds.tsv",
            name,
            row[10]
        );
        let states_translation: bool = feature
            .qualifiers()
            .iter()
            .any(|each| each.name() == "translation");
        assert!(
            states_translation,
            "{}: the CDS {} is imported with translation_matches {}; its record feature states no /translation",
            "pbr322-cds.tsv",
            name,
            row[10]
        );
    }
}

#[test]
fn pbr322_lists_every_other_feature_as_unused() {
    let record: GenBankRecord = pbr322();
    let import: GenBankImport = import_cds(
        &record,
        &request(vec!["tet".to_string(), "bla".to_string()]),
    )
    .unwrap_or_else(|error| {
        panic!(
            "{}: the import refused it: {:?}",
            PBR322_FILE,
            error.message()
        )
    });
    let feature_rows: Vec<Vec<String>> = table("pbr322-features.tsv", FEATURE_HEADER);
    let imported: Vec<usize> = import.cds().iter().map(|cds| cds.index()).collect();
    assert!(
        imported == vec![10usize, 38usize],
        "the import gives indices {:?}, expected [10, 38]",
        imported
    );
    let expected: Vec<usize> = (0..feature_rows.len())
        .filter(|position| *position != 10usize && *position != 38usize)
        .collect();
    let unused: Vec<UnusedFeature> = import.unused();
    let given: Vec<usize> = unused.iter().map(|feature| feature.index()).collect();
    assert!(
        given == expected,
        "the unused list gives indices {:?}, expected {:?}",
        given,
        expected
    );
    for feature_item in unused.iter() {
        let feature: UnusedFeature = feature_item.clone();
        assert!(
            feature.key() == feature_rows[feature.index()][1],
            "the feature at index {} has key {:?}, the fixture records {:?}",
            feature.index(),
            feature.key(),
            feature_rows[feature.index()][1]
        );
        if feature.key() != "CDS" {
            assert!(feature.id().is_none() && feature.refusal().is_none(), "the unused non-CDS {:?} at index {} gives id {:?} and refusal {:?}, expected neither", feature.key(), feature.index(), feature.id(), feature.refusal());
        }
    }
    for row_item in feature_rows.iter().enumerate() {
        let position: usize = row_item.0;
        let row: Vec<String> = row_item.1.clone();
        if imported.contains(&position) {
            let imported_cds = import
                .cds()
                .iter()
                .find(|each| each.index() == position)
                .unwrap()
                .clone();
            assert!(
                imported_cds.cds().id().value() == row[2]
                    && imported_cds.id_qualifier().name() == row[3],
                "{}: the imported feature at index {} has id {} from {}, the fixture records id {} from {}",
                "pbr322-features.tsv",
                position,
                imported_cds.cds().id().value(),
                imported_cds.id_qualifier().name(),
                row[2],
                row[3]
            );
            continue;
        }
        let unused_feature = unused
            .iter()
            .find(|each| each.index() == position)
            .unwrap()
            .clone();
        assert!(
            or_dash(unused_feature.id()) == row[2],
            "{}: the unused feature at index {} has id {}, the fixture records {}",
            "pbr322-features.tsv",
            position,
            or_dash(unused_feature.id()),
            row[2]
        );
        assert!(
            row[3] == "-",
            "{}: the unused feature at index {} gives no id qualifier, the fixture records id_source {:?}",
            "pbr322-features.tsv",
            position,
            row[3]
        );
    }
    let found_rop: Vec<UnusedFeature> = unused
        .iter()
        .filter(|feature| feature.index() == 26usize)
        .cloned()
        .collect();
    assert!(
        found_rop.len() == 1,
        "the unused list has {} features at index 26, expected 1",
        found_rop.len()
    );
    let rop: UnusedFeature = found_rop[0].clone();
    assert!(
        rop.key() == "CDS" && rop.id().is_none(),
        "the feature at index 26 is {:?} with id {:?}, expected a CDS with no id",
        rop.key(),
        rop.id()
    );
    match rop.refusal() {
        None => panic!("the ROP CDS at index 26 gives no refusal"),
        Some(DomainError::UnusableGenBankCds(problem)) => {
            assert!(
                problem.rule == GenBankCdsRule::NoId,
                "the ROP CDS is refused by rule {:?}, expected {:?}",
                problem.rule,
                GenBankCdsRule::NoId
            );
            assert!(
                problem.cds.is_empty(),
                "the ROP CDS refusal names cds {:?}, expected nothing",
                problem.cds
            );
            assert!(
                problem.line == Some(rop.line()),
                "the ROP CDS refusal points at line {:?}, expected its own key line {:?}",
                problem.line,
                rop.line()
            );
        }
        Some(other) => panic!(
            "the ROP CDS at index 26 is refused as {:?}, expected unusable_genbank_cds",
            other
        ),
    }
}

#[test]
fn the_synthetic_record_imports_each_name_as_expected() {
    let record: GenBankRecord = synthetic();
    for row in table("synthetic-expected.tsv", EXPECTED_HEADER).iter() {
        let outcome: Result<GenBankImport, DomainError> =
            import_cds(&record, &request(vec![row[0].clone()]));
        match outcome {
            Ok(imported) => {
                assert!(
                    row[1] == "-",
                    "{}: importing {:?} succeeds; the fixture records error {:?}",
                    "synthetic-expected.tsv",
                    row[0],
                    row[1]
                );
                assert!(
                    imported.cds().len() == 1,
                    "{}: importing {:?} gives {} CDSs, expected 1",
                    "synthetic-expected.tsv",
                    row[0],
                    imported.cds().len()
                );
                let imported_cds = imported.cds()[0].clone();
                assert!(
                    row[2] == "-",
                    "{}: importing {:?} succeeds; the fixture records rule {:?}",
                    "synthetic-expected.tsv",
                    row[0],
                    row[2]
                );
                assert!(
                    intervals_text(&imported_cds.cds()) == row[3],
                    "{}: importing {:?} gives intervals {}, the fixture records {}",
                    "synthetic-expected.tsv",
                    row[0],
                    intervals_text(&imported_cds.cds()),
                    row[3]
                );
                assert!(
                    strand_text(&imported_cds.cds()) == row[4],
                    "{}: importing {:?} gives strand {}, the fixture records {}",
                    "synthetic-expected.tsv",
                    row[0],
                    strand_text(&imported_cds.cds()),
                    row[4]
                );
                assert!(
                    imported_cds.id_qualifier().name() == row[5],
                    "{}: importing {:?} takes its id from {}, the fixture records {}",
                    "synthetic-expected.tsv",
                    row[0],
                    imported_cds.id_qualifier().name(),
                    row[5]
                );
                assert!(
                    or_dash(imported_cds.transl_table()) == row[6],
                    "{}: importing {:?} keeps /transl_table {}, the fixture records {}",
                    "synthetic-expected.tsv",
                    row[0],
                    or_dash(imported_cds.transl_table()),
                    row[6]
                );
                let text: String = translate(&record.sequence(), &imported_cds.cds())
                    .unwrap_or_else(|error| {
                        panic!(
                            "{}: importing {:?} cannot translate the CDS: {:?}",
                            "synthetic-expected.tsv",
                            row[0],
                            error.message()
                        )
                    })
                    .residues_text();
                assert!(
                    text == row[7],
                    "{}: importing {:?} reads {}, the fixture records {}",
                    "synthetic-expected.tsv",
                    row[0],
                    text,
                    row[7]
                );
                assert!(
                    row[8] == "-" && row[9] == "-",
                    "{}: importing {:?} succeeds; the fixture records detail {:?} and line {:?}",
                    "synthetic-expected.tsv",
                    row[0],
                    row[8],
                    row[9]
                );
            }
            Err(error) => {
                let columns: (String, String, String, String) = error_columns(error.clone());
                let code: String = columns.0;
                let rule: String = columns.1;
                let detail: String = columns.2;
                let line: String = columns.3;
                assert!(
                    code == row[1],
                    "{}: importing {:?} is refused as {}, the fixture records {}",
                    "synthetic-expected.tsv",
                    row[0],
                    code,
                    row[1]
                );
                assert!(
                    rule == row[2],
                    "{}: importing {:?} is refused by rule {}, the fixture records {}",
                    "synthetic-expected.tsv",
                    row[0],
                    rule,
                    row[2]
                );
                assert!(row[3] == "-" && row[4] == "-" && row[5] == "-" && row[6] == "-", "{}: importing {:?} is refused; the fixture records values for a success ({:?})", "synthetic-expected.tsv", row[0], vec![row[3].clone(), row[4].clone(), row[5].clone(), row[6].clone()]);
                let residues: String = if row[1] == "genbank_translation_mismatch" {
                    luria_residues(record.clone(), row[0].clone())
                } else {
                    "-".to_string()
                };
                assert!(
                    residues == row[7],
                    "{}: importing {:?} reads {}, the fixture records {}",
                    "synthetic-expected.tsv",
                    row[0],
                    residues,
                    row[7]
                );
                assert!(
                    detail == row[8],
                    "{}: importing {:?} is refused with detail {}, the fixture records {}",
                    "synthetic-expected.tsv",
                    row[0],
                    detail,
                    row[8]
                );
                assert!(
                    line == row[9],
                    "{}: importing {:?} is refused at line {}, the fixture records {}",
                    "synthetic-expected.tsv",
                    row[0],
                    line,
                    row[9]
                );
            }
        }
    }
}

#[test]
fn the_synthetic_record_lists_every_feature_as_unused() {
    let record: GenBankRecord = synthetic();
    let empty: Vec<Identifier> = Vec::new();
    let import: GenBankImport = import_cds(&record, &empty).unwrap_or_else(|error| {
        panic!(
            "{}: the empty import refused it: {:?}",
            SYNTHETIC_FILE,
            error.message()
        )
    });
    let rows: Vec<Vec<String>> = table("synthetic-unused.tsv", UNUSED_HEADER);
    assert!(
        import.unused().len() == rows.len(),
        "{}: the unused list has {} features, the fixture records {}",
        "synthetic-unused.tsv",
        import.unused().len(),
        rows.len()
    );
    for pair in rows.iter().enumerate() {
        let position: usize = pair.0;
        let feature: UnusedFeature = import.unused()[position].clone();
        let row: Vec<String> = pair.1.clone();
        assert!(
            feature.index().to_string() == row[0],
            "{}: row {:?} is not index {}",
            "synthetic-unused.tsv",
            row,
            feature.index()
        );
        assert!(
            feature.key() == row[1],
            "the feature at index {} has key {:?}, the fixture records {:?}",
            feature.index(),
            feature.key(),
            row[1]
        );
        assert!(
            or_dash(feature.id()) == row[2],
            "the feature at index {} has id {}, the fixture records {}",
            feature.index(),
            or_dash(feature.id()),
            row[2]
        );
        assert!(
            refusal_text(feature.refusal()) == row[3],
            "the feature at index {} is refused as {}, the fixture records {}",
            feature.index(),
            refusal_text(feature.refusal()),
            row[3]
        );
    }
}

#[test]
fn imported_cdss_keep_the_request_order_and_leave_the_unused_list() {
    let record: GenBankRecord = synthetic();
    let wanted: Vec<String> = vec!["m1".to_string(), "p1".to_string()];
    let import: GenBankImport =
        import_cds(&record, &request(wanted.clone())).unwrap_or_else(|error| {
            panic!(
                "{}: the import refused it: {:?}",
                SYNTHETIC_FILE,
                error.message()
            )
        });
    assert!(
        import.cds().len() == 2,
        "the import gives {} CDSs, expected 2",
        import.cds().len()
    );
    let order: Vec<String> = import
        .cds()
        .iter()
        .map(|cds| cds.cds().id().value().to_string())
        .collect();
    assert!(
        order == wanted,
        "the import gives CDSs in order {:?}, expected {:?}",
        order,
        wanted
    );
    let indices: Vec<usize> = import
        .unused()
        .iter()
        .map(|feature| feature.index())
        .collect();
    assert!(
        indices.len() == 25,
        "the unused list has {} features, expected 25",
        indices.len()
    );
    assert!(
        !indices.contains(&1usize) && !indices.contains(&2usize),
        "the unused list keeps a requested CDS: {:?}",
        indices
    );
}

#[test]
fn a_name_repeated_in_the_request_is_refused() {
    let record: GenBankRecord = synthetic();
    let repeated: Vec<String> = vec!["p1".to_string(), "m1".to_string(), "p1".to_string()];
    match import_cds(&record, &request(repeated)) {
        Err(DomainError::UnusableGenBankCds(problem)) => {
            assert!(
                problem.rule == GenBankCdsRule::RepeatedName,
                "a repeated name is refused by rule {:?}, expected {:?}",
                problem.rule,
                GenBankCdsRule::RepeatedName
            );
            assert!(
                problem.cds == "p1",
                "the refusal names cds {:?}, expected the repeated name {:?}",
                problem.cds,
                "p1".to_string()
            );
            assert!(problem.line.is_none() && problem.detail.is_none(), "the refusal gives line {:?} and detail {:?}, expected neither: no feature had been examined", problem.line, problem.detail);
        }
        Ok(imported) => panic!(
            "a repeated name imports anyway: {} CDSs, expected a refusal",
            imported.cds().len()
        ),
        Err(other) => panic!(
            "a repeated name is refused as {:?}, expected unusable_genbank_cds",
            other
        ),
    }
}

#[test]
fn origin_joins_are_refused_on_a_linear_copy() {
    let text: String = read_input(SYNTHETIC_FILE);
    let linear: String = text.replacen("circular", "linear", 1);
    let record: GenBankRecord = read_genbank(name(SYNTHETIC_FILE), linear)
        .unwrap_or_else(|error| panic!("the linear copy is refused: {:?}", error.message()));
    assert!(
        record.sequence().topology().name() == "linear",
        "the copy reads as topology {:?}, expected linear",
        record.sequence().topology().name()
    );
    for wanted in ["pj", "mj1", "mj2"] {
        match import_cds(&record, &request(vec![wanted.to_string()])) {
            Err(DomainError::UnusableGenBankLocation(problem)) => {
                assert!(problem.rule == GenBankLocationRule::OriginJoinOnLinear, "the linear copy refuses the origin join {:?} with rule {:?}, expected {:?}", wanted, problem.rule, GenBankLocationRule::OriginJoinOnLinear);
            }
            Ok(imported) => panic!("the linear copy imports the origin join {:?}: {} CDSs, expected a refusal", wanted, imported.cds().len()),
            Err(other) => panic!("the linear copy refuses the origin join {:?} as {:?}, expected unusable_genbank_location", wanted, other)
        }
    }
    let import: GenBankImport = import_cds(&record, &request(vec!["p1".to_string()]))
        .unwrap_or_else(|error| panic!("the linear copy refuses p1: {:?}", error.message()));
    assert!(
        import.cds().len() == 1,
        "the linear copy imports {} CDSs for p1, expected 1",
        import.cds().len()
    );
    assert!(
        import.cds()[0].cds().reference().topology().name() == "linear",
        "the imported CDS references topology {:?}, expected linear",
        import.cds()[0].cds().reference().topology().name()
    );
}
