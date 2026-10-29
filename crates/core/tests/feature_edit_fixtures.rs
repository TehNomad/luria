//! The edit buffer in `luria_core::edit_buffer` against the table
//! `tests/fixtures/editor/feature-edits.tsv` (contract 9b).
//!
//! `tests/fixtures/editor/README.md` is the specification: what one edit of a
//! record's bases does to one feature — its intervals and partial marks, the
//! qualifiers that change, and what the status line names. Each row holds one
//! record: `length` bases with the row's topology, and one feature with the
//! row's key, shape and qualifiers. The row's edit is applied, and the
//! record's length, the surviving feature's shape, its qualifiers and what
//! the status line names must match what the table records. A row whose
//! feature keeps its shape checks both movement and non-movement, because the
//! shape column says which kind it is.
//!
//! The expectations are the table's own columns, which the planner wrote by
//! hand from M9-6, M9-13 and M9-14 and which `reference/feature_edits.py`
//! checks a second time. The table's 1-based inclusive coordinates are
//! converted here to the buffer's zero-based half-open ones (D006), by
//! `parse_shape` and `parse_edit`, and nowhere else.
//!
//! The Python check runs from the repository root, not by cargo:
//!
//! ```
//! python3 -I tests/fixtures/editor/reference/feature_edits.py --check
//! ```

use luria_core::edit_buffer::Edit;
use luria_core::edit_buffer::EditBuffer;
use luria_core::edit_buffer::EditFeature;
use luria_core::edit_buffer::EditLocation;
use luria_core::edit_buffer::EditOutcome;
use luria_core::identity::Identifier;
use luria_core::sequence::SequenceTopology;
use luria_core::strand::Strand;

const FIXTURE: &str = "../../tests/fixtures/editor/feature-edits.tsv";
const HEADER_ROW: &str = concat!(
    "id\trule\tlength\ttopology\tkey\tshape\tqualifiers\tedit\t",
    "length_after\tshape_after\tqualifiers_after\tnamed"
);

/// One row of the table.
struct Row {
    id: String,
    length: u64,
    topology: SequenceTopology,
    key: String,
    shape: String,
    qualifiers: String,
    edit: String,
    length_after: String,
    shape_after: String,
    qualifiers_after: String,
    named: String,
}

fn number(field: &str, what: &str) -> u64 {
    field
        .parse::<u64>()
        .unwrap_or_else(|_| panic!("{}: {} is {:?}, expected a number", FIXTURE, what, field))
}

/// The rows past the header, refusing a header it does not expect and a row
/// that is not twelve tab-separated fields.
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
            fields.len() == 12,
            "{}: row {:?} has {} fields, expected 12",
            FIXTURE,
            line,
            fields.len()
        );
        let id: String = fields[0].to_string();
        out.push(Row {
            topology: match fields[3] {
                "linear" => SequenceTopology::Linear,
                "circular" => SequenceTopology::Circular,
                other => panic!("{}: row '{}' has topology {:?}", FIXTURE, id, other),
            },
            length: number(fields[2], &format!("row {} length", id)),
            id,
            key: fields[4].to_string(),
            shape: fields[5].to_string(),
            qualifiers: fields[6].to_string(),
            edit: fields[7].to_string(),
            length_after: fields[8].to_string(),
            shape_after: fields[9].to_string(),
            qualifiers_after: fields[10].to_string(),
            named: fields[11].to_string(),
        });
    }
    out
}

/// The table's `low..high` intervals, 1-based inclusive, as zero-based
/// half-open ranges (D006): `(low - 1, high)`. `<` before a low end and `>`
/// before a high end mark the feature's ends, and appear only on the first
/// and last range (README "Partial marks", Reading 1).
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

/// The table's `A-B` as the column writes it: 1-based inclusive.
fn table_span(field: &str, id: &str) -> (u64, u64) {
    let parts: Vec<&str> = field.split('-').collect();
    assert!(
        parts.len() == 2,
        "{}: row '{}' range {:?} is not A-B",
        FIXTURE,
        id,
        field
    );
    (
        number(parts[0], &format!("row {} range low", id)),
        number(parts[1], &format!("row {} range high", id)),
    )
}

/// The table's `edit` column as the buffer's `Edit`, with the table's 1-based
/// numbers converted (D006). Only lengths matter in this table, so the bases
/// an edit writes are counted rather than spelled out: `K` `A`s for `K`
/// bases, and `C`s for the bases a substitution overwrites.
fn parse_edit(field: &str, id: &str) -> Edit {
    let words: Vec<&str> = field.split(' ').collect();
    match words[0] {
        "ins" => {
            assert!(
                words.len() == 4 && words[2] == "after",
                "{}: row '{}' edit {:?} is ins K after N",
                FIXTURE,
                id,
                field
            );
            Edit::Insert {
                at: number(words[3], &format!("row {} insertion point", id)),
                bases: vec!['A'; number(words[1], &format!("row {} insertion count", id)) as usize],
            }
        }
        "del" => {
            assert!(
                words.len() == 2,
                "{}: row '{}' edit {:?} is del A-B",
                FIXTURE,
                id,
                field
            );
            let (low, high) = table_span(words[1], id);
            Edit::Delete {
                start: low - 1,
                end: high,
            }
        }
        "rep" => {
            assert!(
                words.len() == 3,
                "{}: row '{}' edit {:?} is rep A-B K",
                FIXTURE,
                id,
                field
            );
            let (low, high) = table_span(words[1], id);
            Edit::Replace {
                start: low - 1,
                end: high,
                bases: vec![
                    'A';
                    number(words[2], &format!("row {} replacement count", id)) as usize
                ],
            }
        }
        "sub" => {
            assert!(
                words.len() == 2,
                "{}: row '{}' edit {:?} is sub A-B",
                FIXTURE,
                id,
                field
            );
            let (low, high) = table_span(words[1], id);
            Edit::Substitute {
                start: low - 1,
                bases: vec!['C'; (high - low + 1) as usize],
            }
        }
        other => panic!(
            "{}: row '{}' edit {:?} has {} for a verb, not one of ins, del, rep, sub",
            FIXTURE, id, field, other
        ),
    }
}

/// The table's `qualifiers` column: `name=value` pairs `;`-joined, `-` for
/// none, split at the first `=`, a flag qualifier written as its name alone.
fn parse_qualifiers(field: &str) -> Vec<(String, Option<String>)> {
    if field == "-" {
        return Vec::new();
    }
    let mut out: Vec<(String, Option<String>)> = Vec::new();
    for part in field.split(';') {
        let mut pieces = part.splitn(2, '=');
        let name: String = match pieces.next() {
            Some(text) => text.to_string(),
            None => continue,
        };
        let value: Option<String> = pieces.next().map(|text| text.to_string());
        out.push((name, value));
    }
    out
}

/// The feature's shape as the table writes it: strand, then the buffer's
/// ranges in order as 1-based inclusive `low..high`, comma-separated, with
/// `<` before the first range's low end and `>` before the last range's high
/// end.
fn shape_text(feature: &EditFeature) -> String {
    let mut parts: Vec<String> = Vec::new();
    for (index, range) in feature.location.ranges.iter().enumerate() {
        let first: bool = index == 0;
        let last: bool = index + 1 == feature.location.ranges.len();
        let mark_before: &str = if first && feature.location.open_start {
            "<"
        } else {
            ""
        };
        let mark_after: &str = if last && feature.location.open_end {
            ">"
        } else {
            ""
        };
        parts.push(format!(
            "{}{}..{}{}",
            mark_before,
            range.0 + 1,
            mark_after,
            range.1
        ));
    }
    format!(
        "{} {}",
        feature.location.strand.clone().symbol(),
        parts.join(",")
    )
}

/// The feature's qualifiers as the table writes them: `name=value` pairs
/// `;`-joined, `-` for none, a flag qualifier as its name alone.
fn qualifiers_text(feature: &EditFeature) -> String {
    if feature.qualifiers.is_empty() {
        return "-".to_string();
    }
    let parts: Vec<String> = feature
        .qualifiers
        .iter()
        .map(|(name, value)| match value {
            Some(text) => format!("{}={}", name, text),
            None => name.to_string(),
        })
        .collect();
    parts.join(";")
}

/// The four columns the edit is checked against: the record's length, the
/// feature's shape, its qualifiers, and what the status line names. A feature
/// left with no range reads `removed`, and its qualifiers go with it.
fn found_columns(buffer: &EditBuffer, outcome: &EditOutcome) -> Vec<String> {
    assert!(
        outcome.removed.len() <= 1,
        "the row's record holds one feature, and {} left it",
        outcome.removed.len()
    );
    let removed: bool = !outcome.removed.is_empty();
    if removed {
        assert!(
            buffer.features().is_empty(),
            "a feature with no range stays out of the buffer"
        );
    } else {
        assert!(
            buffer.features().len() == 1,
            "the row's record holds one feature in it"
        );
    }
    let shape_after: String = if removed {
        "removed".to_string()
    } else {
        shape_text(&buffer.features()[0])
    };
    let qualifiers_after: String = if removed {
        "-".to_string()
    } else {
        qualifiers_text(&buffer.features()[0])
    };
    let named: String = if removed {
        "feature".to_string()
    } else if outcome.dropped.is_empty() {
        "-".to_string()
    } else {
        let names: Vec<String> = outcome
            .dropped
            .iter()
            .map(|(_, name)| name.to_string())
            .collect();
        names.join(",")
    };
    vec![
        buffer.length().to_string(),
        shape_after,
        qualifiers_after,
        named,
    ]
}

/// The record the row describes: the row's `length` `A`s, its topology, and
/// one feature with the row's key, shape and qualifiers, whose location text
/// is `Some("x")` because nothing has moved it yet.
fn record(row: &Row) -> EditBuffer {
    let feature: EditFeature = EditFeature {
        key: row.key.clone(),
        location: parse_shape(&row.shape, &row.id),
        qualifiers: parse_qualifiers(row.qualifiers.as_str()),
        location_text: Some("x".to_string()),
    };
    EditBuffer::new(
        Identifier::create("record".to_string()).unwrap(),
        row.topology.clone(),
        vec!['A'; row.length as usize],
        vec![feature],
    )
    .unwrap_or_else(|error| {
        panic!(
            "{}: the row's record is refused: {}",
            row.id,
            error.message()
        )
    })
}

/// The record after the row's edit, and what the edit did.
fn applied(row: &Row) -> (EditBuffer, EditOutcome) {
    let mut buffer: EditBuffer = record(row);
    let edit: Edit = parse_edit(&row.edit, &row.id);
    let outcome: EditOutcome = buffer.apply(&edit).unwrap_or_else(|error| {
        panic!(
            "{}: the row's edit {:?} is refused: {}",
            row.id,
            row.edit,
            error.message()
        )
    });
    (buffer, outcome)
}

#[test]
fn every_feature_edit_row_matches() {
    let mut mismatches: Vec<String> = Vec::new();
    for row in rows().iter() {
        let (buffer, outcome) = applied(row);
        let found: Vec<String> = found_columns(&buffer, &outcome);
        let wanted: Vec<String> = vec![
            row.length_after.clone(),
            row.shape_after.clone(),
            row.qualifiers_after.clone(),
            row.named.clone(),
        ];
        if found != wanted {
            let topology: &str = if row.topology == SequenceTopology::Circular {
                "circular"
            } else {
                "linear"
            };
            mismatches.push(format!(
                "{}: {} {} on a {} {}-base record with {} at {}, the edit gives length {}, shape {}, qualifiers {}, status line [{}], the table records length {}, shape {}, qualifiers {}, status line [{}]",
                row.id,
                row.key,
                row.edit,
                topology,
                row.length,
                row.qualifiers,
                row.shape,
                found[0],
                found[1],
                found[2],
                found[3],
                wanted[0],
                wanted[1],
                wanted[2],
                wanted[3]
            ));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn location_text_follows_moves() {
    // `location_text` is kept as written exactly when no edit moved the
    // location, and the table's `shape` and `shape_after` columns say which
    // rows moved. A feature left with no range is out of the buffer as it was
    // in it, location text and all.
    let mut wrong: Vec<String> = Vec::new();
    for row in rows().iter() {
        let (buffer, outcome) = applied(row);
        for feature in outcome.removed.iter() {
            if feature.location_text != Some("x".to_string()) {
                wrong.push(format!(
                    "{}: {} removed the feature at {}, holding location_text {:?}",
                    row.id, row.edit, row.shape, feature.location_text
                ));
            }
        }
        if !outcome.removed.is_empty() {
            continue;
        }
        let moved: bool = row.shape_after != row.shape;
        let text: Option<String> = buffer.features()[0].location_text.clone();
        let kept: bool = text == Some("x".to_string());
        if moved != !kept {
            wrong.push(format!(
                "{}: {} on {} leaves shape {}, so the feature {} location_text {:?}",
                row.id,
                row.edit,
                row.shape,
                row.shape_after,
                if kept { "kept its" } else { "cleared" },
                text
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
