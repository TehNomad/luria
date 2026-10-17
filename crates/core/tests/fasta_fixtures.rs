//! The FASTA reader in `crates/core/src/fasta.rs` against
//! `tests/fixtures/fasta/expected.tsv`: for each single-record FASTA input in
//! `../example-construct/`, the header it carries as source metadata, the
//! sequence length, and the SHA-256 of the canonical text (D007).
//!
//! `reference/digests.py` computed all three columns with the Python standard
//! library only, sharing no code with `crates/core`; the digests were also
//! checked by hand (see `tests/fixtures/fasta/README.md`). The script is
//! rerun from the repository root, not by cargo:
//!
//! ```
//! python -I tests/fixtures/fasta/reference/digests.py --check
//! ```
//!
//! That Python check is not part of the required cargo checks (D039, D026).

use luria_core::fasta::read_fasta;
use luria_core::fasta::FastaRecord;
use luria_core::identity::Identifier;
use luria_core::sequence::SequenceTopology;

const FIXTURE: &str = "../../tests/fixtures/fasta/expected.tsv";
/// The FASTA inputs sit under here, and the `file` column of `expected.tsv`
/// continues the path, relative to the package directory.
const FIXTURES: &str = "../../tests/fixtures/";
const HEADER_ROW: &str = "file\theader\tlength\tsha256";
const DATA_ROWS: usize = 2;

/// One row of `expected.tsv`: the input file relative to `tests/fixtures/`,
/// its header as source metadata, its sequence length, and the SHA-256 of its
/// canonical text.
struct Row {
    file: String,
    header: String,
    length: u64,
    digest: String,
}

/// The rows of `expected.tsv` past its header row, refusing a header row it
/// does not have and a row that is not four tab-separated fields.
fn rows() -> Vec<Row> {
    let text: String = std::fs::read_to_string(FIXTURE)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", FIXTURE, error));
    let lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    assert!(
        !lines.is_empty(),
        "{}: expected header row {:?}, none found",
        FIXTURE,
        HEADER_ROW
    );
    assert!(
        lines[0] == HEADER_ROW,
        "{}: first row {:?}, expected {:?}",
        FIXTURE,
        lines[0],
        HEADER_ROW,
    );
    let mut out: Vec<Row> = Vec::new();
    for line in lines[1..].iter() {
        let fields: Vec<String> = line.split('\t').map(|field| field.to_string()).collect();
        assert!(
            fields.len() == 4,
            "{}: row {:?} has {} fields, expected 4",
            FIXTURE,
            line,
            fields.len(),
        );
        out.push(Row {
            file: fields[0].clone(),
            header: fields[1].clone(),
            length: fields[2]
                .parse::<u64>()
                .unwrap_or_else(|_| panic!("{}: row {:?} has no length", FIXTURE, line,)),
            digest: fields[3].clone(),
        });
    }
    out
}

/// The sequence identifier every fixture input is read as. The header line is
/// source metadata only (D007): the identity comes from the caller.
fn name() -> Identifier {
    Identifier::create("fixture".to_string()).unwrap()
}

#[test]
fn the_fixture_holds_exactly_its_header_row_and_two_data_rows() {
    let capture: Vec<Row> = rows();
    assert!(
        capture.len() == DATA_ROWS,
        "{}: {} data rows, expected {}",
        FIXTURE,
        capture.len(),
        DATA_ROWS,
    );
}

#[test]
fn the_reader_gives_the_record_the_fixture_records() {
    for row in rows().iter() {
        let path: String = format!("{}{}", FIXTURES, row.file);
        let text: String = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
        let record: FastaRecord = read_fasta(name(), text, SequenceTopology::Linear)
            .unwrap_or_else(|error| {
                panic!("{}: read_fasta refused it: {:?}", path, error.message())
            });
        assert!(
            record.header() == row.header,
            "{}: the reader gives header {:?}, the fixture records {:?}",
            path,
            record.header(),
            row.header,
        );
        assert!(
            record.sequence().length() == row.length,
            "{}: the reader gives length {}, the fixture records {}",
            path,
            record.sequence().length(),
            row.length,
        );
        assert!(
            record.sequence().content_hash().hex_digest() == row.digest,
            "{}: the reader gives digest {:?}, the fixture records {:?}",
            path,
            record.sequence().content_hash().hex_digest(),
            row.digest,
        );
    }
}
