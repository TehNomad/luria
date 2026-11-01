//! The location text writer `luria_core::edit_location::write_edit_location`
//! and `read_edit_location` against the table
//! `tests/fixtures/editor/locations.tsv` (contract 9c2 §5 tests 1 and 2).
//!
//! Each row is one shape on one record — `length` bases with the row's
//! topology — and the GenBank location text the shape is written as. Test 1
//! (`every_location_writes_its_text`) asks the writer for the row's text;
//! test 2 (`every_text_reads_back`) asks the editor's reader, 9b2, to read
//! that text on the row's record and length back to the same shape. Writing
//! and reading are therefore two ends of one rule, and the rows pin it:
//! `join(…)` for several ranges, `complement(…)` around the whole text for a
//! minus location, a bare position for a one-base range, and `<` and `>` on
//! the feature's own ends (M9-17, M9-18, D006, D036).
//!
//! The expectations are the table's own `text` column, which the planner
//! wrote by hand from contract 9c2 §4 and which
//! `reference/locations.py` checks a second time with Biopython alone (D039).
//! The table's 1-based inclusive coordinates are converted here to the
//! zero-based half-open ones of `EditLocation` (D006), by `parse_shape` and
//! nowhere else.
//!
//! The Python check runs from the repository root, not by cargo:
//!
//! ```
//! python3 -I tests/fixtures/editor/reference/locations.py --check
//! ```

use luria_core::edit_buffer::EditLocation;
use luria_core::edit_location::read_edit_location;
use luria_core::edit_location::write_edit_location;
use luria_core::sequence::SequenceTopology;
use luria_core::strand::Strand;

const FIXTURE: &str = "../../tests/fixtures/editor/locations.tsv";
const HEADER_ROW: &str = "id\tlength\ttopology\tshape\ttext";

/// One row of the table.
struct Row {
    id: String,
    length: u64,
    topology: SequenceTopology,
    shape: String,
    text: String,
}

fn number(field: &str, what: &str) -> u64 {
    field
        .parse::<u64>()
        .unwrap_or_else(|_| panic!("{}: {} is {:?}, expected a number", FIXTURE, what, field))
}

/// The rows past the header, refusing a header it does not expect and a row
/// that is not five tab-separated fields.
fn rows() -> Vec<Row> {
    let text: String = std::fs::read_to_string(FIXTURE)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", FIXTURE, error));
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        !lines.is_empty() && lines[0] == HEADER_ROW,
        "{}: first row {:?}, expected {:?}",
        FIXTURE,
        if lines.is_empty() { "" } else { lines[0] },
        HEADER_ROW
    );
    let mut out: Vec<Row> = Vec::new();
    for line in lines[1..].iter() {
        let fields: Vec<&str> = line.split('\t').collect();
        assert!(
            fields.len() == 5,
            "{}: row {:?} has {} fields, expected 5",
            FIXTURE,
            line,
            fields.len()
        );
        let id: String = fields[0].to_string();
        out.push(Row {
            topology: match fields[2] {
                "linear" => SequenceTopology::Linear,
                "circular" => SequenceTopology::Circular,
                other => panic!("{}: row '{}' has topology {:?}", FIXTURE, id, other),
            },
            length: number(fields[1], &format!("row {} length", id)),
            id,
            shape: fields[3].to_string(),
            text: fields[4].to_string(),
        });
    }
    out
}

/// The table's shape: strand, then the `low..high` intervals, 1-based
/// inclusive, as zero-based half-open ranges (D006): `(low - 1, high)`. A
/// minus-strand join holds its ranges as `complement(join(…))` writes them,
/// and an origin-crossing feature lists the interval that ends at the
/// record's last base first (README "Columns", D036). `<` before a low end
/// and `>` before a high end mark the feature's ends, and appear only on the
/// first and last range (README "Partial marks", Reading 1).
fn parse_shape(field: &str, id: &str) -> EditLocation {
    let parts: Vec<&str> = field.split(' ').collect();
    assert!(
        parts.len() == 2,
        "{}: row '{}' shape {:?} is strand and intervals",
        FIXTURE,
        id,
        field
    );
    let strand: Strand = match parts[0] {
        "+" => Strand::Plus,
        "-" => Strand::Minus,
        other => panic!(
            "{}: row '{}' shape {:?} names {:?} as a strand",
            FIXTURE, id, field, other
        ),
    };
    let written: Vec<&str> = parts[1].split(',').collect();
    let mut ranges: Vec<(u64, u64)> = Vec::new();
    let mut open_start: bool = false;
    let mut open_end: bool = false;
    for (index, one) in written.iter().enumerate() {
        let ends: Vec<&str> = one.split("..").collect();
        assert!(
            ends.len() == 2,
            "{}: row '{}' interval {:?} is not low..high",
            FIXTURE,
            id,
            one
        );
        let bare_low: &str = ends[0].strip_prefix('<').unwrap_or(ends[0]);
        let bare_high: &str = ends[1].strip_prefix('>').unwrap_or(ends[1]);
        if ends[0] != bare_low {
            assert!(
                index == 0,
                "{}: row '{}' shape {:?} names `<` on an interval that is not the first",
                FIXTURE,
                id,
                field
            );
            open_start = true;
        }
        if ends[1] != bare_high {
            assert!(
                index + 1 == written.len(),
                "{}: row '{}' shape {:?} names `>` on an interval that is not the last",
                FIXTURE,
                id,
                field
            );
            open_end = true;
        }
        ranges.push((
            number(bare_low, &format!("row {} interval low", id)) - 1,
            number(bare_high, &format!("row {} interval high", id)),
        ));
    }
    EditLocation {
        strand,
        ranges,
        open_start,
        open_end,
    }
}

/// Test 1: every row's shape writes exactly the row's text, in one line
/// (9c3 wraps it, 9c2 writes it).
#[test]
fn every_location_writes_its_text() {
    for row in rows() {
        let shape: EditLocation = parse_shape(&row.shape, &row.id);
        let written: String = write_edit_location(&shape);
        assert_eq!(
            written,
            row.text,
            "{}: {} {} on a {}-base record writes {:?}, the table records {:?}",
            row.id,
            row.shape,
            row.topology.name(),
            row.length,
            written,
            row.text
        );
        assert!(
            !written.contains('\n'),
            "{}: {:?} is written on one line",
            row.id,
            written
        );
    }
}

/// Test 2: every row's text reads back to the row's shape, on the row's
/// record: 9b2's reader, the row's length and topology, and nothing else.
#[test]
fn every_text_reads_back() {
    for row in rows() {
        let want: EditLocation = parse_shape(&row.shape, &row.id);
        let found: EditLocation =
            read_edit_location(row.text.as_str(), row.length, row.topology.clone()).unwrap_or_else(
                |error| {
                    panic!(
                        "{}: {:?} on a {}-base {} record read_edit_location refused as {}: {}",
                        row.id,
                        row.text,
                        row.length,
                        row.topology.name(),
                        error.code(),
                        error.message()
                    )
                },
            );
        assert_eq!(
            found,
            want,
            "{}: {:?} on a {}-base {} record reads {:?}, the shape is {:?}",
            row.id,
            row.text,
            row.length,
            row.topology.name(),
            found,
            want
        );
    }
}
