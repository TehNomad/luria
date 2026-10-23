//! The mutant reading frame in `luria_core::frame` against
//! `tests/fixtures/review/frame.tsv` (plan §5.3a, M7-20; contract 7i1).
//!
//! Each row gives a template (a `seq:` column, or a FASTA file under
//! `tests/fixtures/`), the CDS as annotated on the **template**, the edit in
//! template coordinates, and a window of the edited construct; the `residues`
//! column records the expected mutant residues as `number:symbol:middle`,
//! `;`-joined, `-` for none.
//!
//! The expectations come from `reference/frame.py`, which splices the written
//! bases into the template and reads every codon with its own standard-code
//! table and complement map, sharing nothing with `crates/`. The driver here
//! builds the edited construct the same way, from the row's own numbers, and
//! calls only the public domain API: `normalize`, `read_fasta`, `create`,
//! `Cds::create` and `mutant_residues`. The CDS is built on the template and
//! passed with the edited construct, so the function must read the CDS's
//! coordinates only, never its reference.
//!
//! The Python check is rerun from the repository root, not by cargo:
//!
//! ```
//! python3 -I tests/fixtures/review/reference/frame.py --check
//! ```

use luria_core::cds::Cds;
use luria_core::cds::Completeness;
use luria_core::cds::Numbering;
use luria_core::fasta::read_fasta;
use luria_core::frame::mutant_residues;
use luria_core::frame::EditSpan;
use luria_core::frame::FrameResidue;
use luria_core::identity::Identifier;
use luria_core::interval::create;
use luria_core::interval::OrderedIntervals;
use luria_core::sequence::normalize;
use luria_core::sequence::Sequence;
use luria_core::sequence::SequenceTopology;
use luria_core::strand::Strand;

const FIXTURE: &str = "../../tests/fixtures/review/frame.tsv";
/// FASTA templates are read from here, by the row's own path.
const FIXTURES: &str = "../../tests/fixtures/";
const HEADER_ROW: &str = "case_id\ttemplate\tcds_start\tcds_end\tstrand\tedit_start\tedit_end\twritten\twindow_start\twindow_end\tresidues";

/// One row of `frame.tsv`.
struct Row {
    case_id: String,
    template: String,
    cds_start: u64,
    cds_end: u64,
    strand: Strand,
    edit_start: u64,
    edit_end: u64,
    written: String,
    window_start: u64,
    window_end: u64,
    residues: String,
}

fn number(field: &str, case_id: &str, column: &str) -> u64 {
    field
        .parse::<u64>()
        .unwrap_or_else(|_| panic!("{}: row '{}' has no {}", FIXTURE, case_id, column))
}

/// The rows of `frame.tsv` past its header row, refusing a header row it does
/// not have and a row that is not eleven tab-separated fields.
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
        HEADER_ROW
    );
    let mut out: Vec<Row> = Vec::new();
    for line in lines[1..].iter() {
        let fields: Vec<String> = line.split('\t').map(|field| field.to_string()).collect();
        assert!(
            fields.len() == 11,
            "{}: row {:?} has {} fields, expected 11",
            FIXTURE,
            line,
            fields.len()
        );
        let case_id: String = fields[0].clone();
        out.push(Row {
            case_id: case_id.clone(),
            template: fields[1].clone(),
            cds_start: number(&fields[2], &case_id, "cds_start"),
            cds_end: number(&fields[3], &case_id, "cds_end"),
            strand: match fields[4].as_str() {
                "+" => Strand::Plus,
                "-" => Strand::Minus,
                other => panic!("{}: row '{}' has strand {:?}", FIXTURE, case_id, other),
            },
            edit_start: number(&fields[5], &case_id, "edit_start"),
            edit_end: number(&fields[6], &case_id, "edit_end"),
            written: if fields[7] == "-" {
                "".to_string()
            } else {
                fields[7].clone()
            },
            window_start: number(&fields[8], &case_id, "window_start"),
            window_end: number(&fields[9], &case_id, "window_end"),
            residues: fields[10].clone(),
        });
    }
    out
}

/// The template's canonical text: the bases after `seq:`, or the single FASTA
/// record at the row's path under `tests/fixtures/`.
fn template_text(row: &Row) -> String {
    if row.template.starts_with("seq:") {
        return row.template["seq:".len()..].to_string();
    }
    let path: String = format!("{}{}", FIXTURES, row.template);
    let text: String = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
    let record = read_fasta(
        Identifier::create("template".to_string()).unwrap(),
        text,
        SequenceTopology::Linear,
    )
    .unwrap_or_else(|error| {
        panic!(
            "{}: read_fasta refused {}: {}",
            row.case_id,
            path,
            error.message()
        )
    });
    record.sequence().canonical_text()
}

/// The template as a `Sequence`, named `template`.
fn template_sequence(row: &Row) -> Sequence {
    normalize(
        Identifier::create("template".to_string()).unwrap(),
        template_text(row),
        SequenceTopology::Linear,
    )
    .unwrap_or_else(|error| {
        panic!(
            "{}: the template is refused: {}",
            row.case_id,
            error.message()
        )
    })
}

/// The edited construct: `written` spliced into `[edit_start, edit_end)` of
/// the template, normalized as a `Sequence` named `edited`.
fn edited_sequence(row: &Row) -> Sequence {
    let bases: String = template_text(row);
    let chars: Vec<char> = bases.chars().collect();
    let start: usize = row.edit_start as usize;
    let end: usize = row.edit_end as usize;
    assert!(
        end <= chars.len() && start <= end,
        "{}: the edit [{}, {}) is outside the {}-base template",
        row.case_id,
        row.edit_start,
        row.edit_end,
        chars.len()
    );
    let mut letters: Vec<char> = Vec::new();
    letters.extend_from_slice(&chars[..start]);
    for ch in row.written.chars() {
        letters.push(ch);
    }
    letters.extend_from_slice(&chars[end..]);
    let text: String = letters.iter().collect();
    normalize(
        Identifier::create("edited".to_string()).unwrap(),
        text,
        SequenceTopology::Linear,
    )
    .unwrap_or_else(|error| {
        panic!(
            "{}: the edited construct is refused: {}",
            row.case_id,
            error.message()
        )
    })
}

/// The CDS as annotated on the template, complete, standard code, numbering
/// from the CDS start.
fn annotated_cds(row: &Row) -> Cds {
    let template: Sequence = template_sequence(row);
    let intervals: OrderedIntervals = vec![create(
        &template.reference(),
        row.cds_start as i64,
        row.cds_end as i64,
    )
    .unwrap_or_else(|error| {
        panic!(
            "{}: the CDS interval [{}, {}) is refused: {}",
            row.case_id,
            row.cds_start,
            row.cds_end,
            error.message()
        )
    })];
    Cds::create(
        &template,
        Identifier::create(row.case_id.clone()).unwrap(),
        intervals,
        row.strand.clone(),
        Completeness::Complete,
        "standard".to_string(),
        Numbering::FromCdsStart,
    )
    .unwrap_or_else(|error| panic!("{}: the CDS is refused: {}", row.case_id, error.message()))
}

/// The residues as the fixture writes them: `number:symbol:middle`,
/// `;`-joined, `-` for none. For the failure message only.
fn fixture_text(residues: &[FrameResidue]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for residue in residues {
        parts.push(format!(
            "{}:{}:{}",
            residue.number, residue.symbol, residue.middle
        ));
    }
    if parts.is_empty() {
        return "-".to_string();
    }
    parts.join(";")
}

/// The `residues` column parsed as `number:symbol:middle`, `;`-joined; `-` is
/// none.
fn parse_residues(field: &str, case_id: &str) -> Vec<FrameResidue> {
    if field == "-" {
        return Vec::new();
    }
    let mut out: Vec<FrameResidue> = Vec::new();
    for part in field.split(';') {
        let bits: Vec<&str> = part.split(':').collect();
        assert!(
            bits.len() == 3,
            "{}: residue {:?} is not number:symbol:middle",
            case_id,
            part
        );
        let symbol: Vec<char> = bits[1].chars().collect();
        out.push(FrameResidue {
            number: bits[0]
                .parse::<u64>()
                .unwrap_or_else(|_| panic!("{}: residue {:?} has no number", case_id, part)),
            symbol: match symbol.as_slice() {
                [ch] => *ch,
                other => panic!(
                    "{}: residue {:?} has symbol {:?}, expected one letter",
                    case_id, part, other
                ),
            },
            middle: bits[2]
                .parse::<u64>()
                .unwrap_or_else(|_| panic!("{}: residue {:?} has no middle", case_id, part)),
        });
    }
    out
}

#[test]
fn every_frame_row_matches() {
    let mut mismatches: Vec<String> = Vec::new();
    for row in rows().iter() {
        let edited: Sequence = edited_sequence(row);
        let cds: Cds = annotated_cds(row);
        let edit: EditSpan = EditSpan {
            start: row.edit_start,
            end: row.edit_end,
            written_length: row.written.len() as u64,
        };
        let found: Vec<FrameResidue> =
            mutant_residues(&edited, &cds, edit, row.window_start, row.window_end).unwrap_or_else(
                |error| {
                    panic!(
                        "{}: mutant_residues refused it: {}",
                        row.case_id,
                        error.message()
                    )
                },
            );
        let wanted: Vec<FrameResidue> = parse_residues(&row.residues, &row.case_id);
        if found != wanted {
            mismatches.push(format!(
                "{}: core gives [{}], the fixture records [{}]",
                row.case_id,
                fixture_text(found.as_slice()),
                row.residues
            ));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}
