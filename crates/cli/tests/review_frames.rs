//! Every member's mutant protein, `luria_cli::review::member_frames`, against
//! `tests/fixtures/review/frame.tsv` (plan §5.3a, M7-18 to M7-21; contract
//! 7i2), and the run's own edited constructs as `run_with_constructs` returns
//! them.
//!
//! The expectations are the rows of `frame.tsv`, read at test time and parsed
//! as 7i1's test parses them (`number:symbol:middle`, `;`-joined), never
//! pasted. Those rows come from `tests/fixtures/review/reference/frame.py`,
//! which splices the written bases into the template and reads every codon with
//! its own standard-code table, sharing no code with `crates/` (see
//! `tests/fixtures/review/README.md`), so the protein a frame gives is checked
//! against a reading this repository does not contain.
//!
//! The requests are the ones `crates/cli/tests/review_context.rs` builds (its
//! `workspace`, `design`, `sid1_template`, `request` and `synthetic` helpers,
//! copied there, and that file is not changed), so the edits are the ones the
//! `tests/fixtures/q5-primer/` tables already pin: `native-01`
//! (`native.tsv`), `del-11` (`native-deletion.tsv`), `ins-14`
//! (`native-insertion.tsv`), the `D404N` residue `crates/cli/src/result.rs`
//! tests with, `del-02` (`native-deletion.tsv`), whose frameshift reads on
//! past the edit, and `ins-04` (`native-insertion.tsv`) read on a minus-strand
//! CDS, whose first base lies past the insertion and so moves by the seven
//! bases written. Each run goes through `run_with_constructs`, so the construct
//! read is the one core produced for that member, and `prepare` gives the
//! template and its annotations the CDS is taken from.

use serde_json::{json, Value};

use luria_cli::prepare::prepare;
use luria_cli::result::ResultDocument;
use luria_cli::review::member_frames;
use luria_cli::review::MemberFrame;
use luria_cli::run::run_with_constructs;
use luria_core::cds::Cds;
use luria_core::frame::FrameResidue;
use luria_core::sequence::{normalize, Sequence, SequenceTopology};
use std::fs;
use std::path::{Path, PathBuf};

/// The SID-1 cDNA's digest, from `tests/fixtures/fasta/expected.tsv`.
const SID1_DIGEST: &str = "54704740a9f04b79d17147ef95788ad9f0f11e5263843a7ed490924d43b55e3a";

/// Rule 4 of contract §2.2, exactly: the construct read is not the one the
/// result reports.
const NOT_THE_CONSTRUCT: &str = "the edited construct differs from the one this result reports";

/// Rule 1 of contract §2.2, exactly: the result names no CDS to read on.
const NO_TARGET: &str = "no target CDS, so no protein";

/// A fresh directory for one test, as `tests/end_to_end.rs`'s `workspace` is,
/// with a copy of the SID-1 cDNA beside the request.
fn workspace(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("review-frames-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::copy(
        "../../tests/fixtures/example-construct/input/sid-1_cDNA.fa",
        dir.join("sid-1_cDNA.fa"),
    )
    .unwrap();
    dir
}

/// The design block `tests/end_to_end.rs`'s `design()` states: the same buffer,
/// so the bound is the only number a test changes.
fn design() -> Value {
    json!({
        "policy": "luriaNative",
        "style": "q5_polymerase",
        "minAnnealingOverlap": 15,
        "placement": "inline",
        "buffer": "q5_reaction"
    })
}

/// The SID-1 cDNA as a FASTA template, as `tests/end_to_end.rs`'s
/// `sid1_template()` declares it: the same id, the same digest, the same CDS.
fn sid1_template() -> Value {
    json!({
        "id": "sid-1",
        "fasta": "sid-1_cDNA.fa",
        "topology": "linear",
        "sha256": SID1_DIGEST,
        "cds": [{
            "id": "sid-1",
            "intervals": [[0, 3153]],
            "strand": "+",
            "completeness": "complete",
            "geneticCode": "standard",
            "numbering": "from-cds-start"
        }]
    })
}

/// A one-edit request on SID-1 at `bound`, as `tests/end_to_end.rs`'s `request`
/// writes it: `edit` is the one kind, with `name` added to it.
fn request(name: &str, edit: Value, bound: u64) -> Value {
    let mut spec = design();
    spec["minAnnealingOverlap"] = json!(bound);
    let mut edit_object = json!({ "name": name });
    for (key, value) in edit.as_object().expect("an edit object") {
        edit_object[key] = value.clone();
    }
    json!({
        "schema": "luria.q5-request/1",
        "coordinates": "zero-based-half-open",
        "template": sid1_template(),
        "scope": "sid-1",
        "edit": edit_object,
        "design": spec
    })
}

/// The `template` column of one row of one `tests/fixtures/q5-primer/` table,
/// found by its `case_id` column and read at test time, without its `seq:`
/// prefix. `ins-04` of `native-insertion.tsv` and `del-02` of
/// `native-deletion.tsv` give the same synthetic template.
fn synthetic(table: &str, row: &str) -> String {
    let path = format!("../../tests/fixtures/q5-primer/{}", table);
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
    let mut at: usize = 0;
    for (index, name) in text
        .lines()
        .next()
        .expect("a header")
        .split('\t')
        .enumerate()
    {
        if name == "template" {
            at = index;
        }
    }
    for line in text.lines().skip(1) {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields[0] == row {
            let column: &str = fields[at];
            assert!(
                column.starts_with("seq:"),
                "row {} gives a template file, not a sequence",
                row
            );
            return column["seq:".len()..].to_string();
        }
    }
    panic!("no row {} in {}", row, table)
}

/// The replacement `native.tsv` row `native-01` records.
fn native_edit() -> Value {
    json!({ "replacement": { "interval": [1209, 1212], "from": "GAT", "to": "AAC" } })
}

/// One run: the document it gave, the constructs core made for its members, and
/// the template and its annotations `prepare` reads beside the request.
struct Run {
    document: ResultDocument,
    constructs: Vec<(String, Sequence)>,
    template: Sequence,
    annotations: Vec<Cds>,
}

/// Run one request through `run_with_constructs` and prepare it, so the frames
/// are read on the constructs that run made and the template that run read.
/// Run one request file through `run_with_constructs` and prepare it, so the
/// frames are read on the constructs that run made and the template that run
/// read.
fn run_at(path: &Path) -> Run {
    let (outcome, constructs) = run_with_constructs(path);
    let prepared = prepare(path).expect("the request prepares");
    Run {
        document: outcome.document,
        constructs,
        template: prepared.template,
        annotations: prepared.annotations,
    }
}

fn run_request(test: &str, spec: Value) -> Run {
    let dir = workspace(test);
    let path = dir.join("request.json");
    fs::write(&path, spec.to_string()).unwrap();
    run_at(&path)
}

/// The same, on a synthetic FASTA template written beside the request, as
/// `tests/review_context.rs`'s `ins-04` test writes it.
fn run_synthetic(test: &str, template: &str, spec: Value) -> Run {
    let dir = workspace(test);
    fs::write(dir.join("syn.fa"), format!(">syn\n{}\n", template)).unwrap();
    let path = dir.join("request.json");
    fs::write(&path, spec.to_string()).unwrap();
    run_at(&path)
}

/// Every member's frame of a run, in `document.results` order.
fn frames_of(run: &Run) -> Vec<MemberFrame> {
    member_frames(
        &run.document,
        &run.template,
        &run.annotations,
        &run.constructs,
    )
}

/// One row of `frame.tsv`: its window in the edited construct, and the
/// residues it records.
struct Row {
    window_start: u64,
    window_end: u64,
    residues: String,
}

const HEADER_ROW: &str = "case_id\ttemplate\tcds_start\tcds_end\tstrand\tedit_start\tedit_end\twritten\twindow_start\twindow_end\tresidues";
const FIXTURE: &str = "../../tests/fixtures/review/frame.tsv";

/// One row of `frame.tsv`, found by its `case_id` column, read at test time.
fn frame_row(case_id: &str) -> Row {
    let text = fs::read_to_string(FIXTURE)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", FIXTURE, error));
    let lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    assert!(
        !lines.is_empty() && lines[0] == HEADER_ROW,
        "{}: expected header row {:?}, found {:?}",
        FIXTURE,
        HEADER_ROW,
        lines.first()
    );
    for line in lines[1..].iter() {
        let fields: Vec<&str> = line.split('\t').collect();
        assert!(
            fields.len() == 11,
            "{}: row {:?} has {} fields, expected 11",
            FIXTURE,
            line,
            fields.len()
        );
        if fields[0] == case_id {
            return Row {
                window_start: fields[8].parse::<u64>().unwrap_or_else(|_| {
                    panic!("{}: row '{}' has no window_start", FIXTURE, case_id)
                }),
                window_end: fields[9]
                    .parse::<u64>()
                    .unwrap_or_else(|_| panic!("{}: row '{}' has no window_end", FIXTURE, case_id)),
                residues: fields[10].to_string(),
            };
        }
    }
    panic!("{}: no row '{}'", FIXTURE, case_id)
}

/// The `residues` column parsed as 7i1's test parses it:
/// `number:symbol:middle`, `;`-joined; `-` is none.
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

/// The residues whose codon's middle base lies in the row's window
/// `[start, end)`, as `frame.tsv` states it for each case.
fn in_window(residues: &[FrameResidue], start: u64, end: u64) -> Vec<FrameResidue> {
    residues
        .iter()
        .filter(|residue| residue.middle >= start && residue.middle < end)
        .cloned()
        .collect()
}

/// The frame of the first member, as `member_frames` gave it.
fn first_frame(frames: &[MemberFrame]) -> Vec<FrameResidue> {
    frames
        .first()
        .expect("the run has one member")
        .as_ref()
        .expect("the frame is read")
        .clone()
}

/// The frame the fixture's row `case_id` records, against the run of `spec`,
/// each compared in that row's own window.
fn assert_frame(test: &str, case_id: &str, spec: Value) {
    let run: Run = run_request(test, spec);
    assert_eq!(
        run.document.state, "verified",
        "{}: the run is verified",
        case_id
    );
    assert_eq!(run.document.results.len(), 1, "{}: one member", case_id);
    let row: Row = frame_row(case_id);
    let frames: Vec<MemberFrame> = frames_of(&run);
    assert_eq!(
        in_window(&first_frame(&frames), row.window_start, row.window_end),
        parse_residues(&row.residues, case_id),
        "row {} of {}: the residues core read, in the row's window",
        case_id,
        FIXTURE
    );
}

#[test]
fn native_01_gives_the_fixture_frame() {
    assert_frame(
        "native-01",
        "native-01",
        request("native-01", native_edit(), 15),
    );
}

#[test]
fn del_11_gives_the_fixture_frame() {
    let edit = json!({ "deletion": { "interval": [1209, 1212], "from": "GAT" } });
    assert_frame("del-11", "del-11", request("del-11", edit, 15));
}

#[test]
fn ins_14_gives_the_fixture_frame() {
    let edit =
        json!({ "insertion": { "point": 1209, "before": "A", "after": "G", "insert": "GAT" } });
    assert_frame("ins-14", "ins-14", request("ins-14", edit, 15));
}

#[test]
fn a_residue_request_uses_its_codon() {
    // The `SID1_REQUEST` `crates/cli/src/result.rs` tests with: the residue
    // `D404N` of CDS `sid-1`, written as the codon `AAC`. Its edit spans the
    // three bases of codon 404, so it makes the same construct as `native-01`
    // and must read the same protein.
    let spec = json!({ "residue": {
        "cds": "sid-1",
        "number": 404,
        "from": "D",
        "to": "N",
        "codon": "AAC"
    } });
    assert_frame("residue-404", "native-01", request("D404N", spec, 15));
}

#[test]
fn a_frameshift_reads_on() {
    // `native-deletion.tsv` row `del-02`: one base deleted at 45 on the row's
    // own synthetic template, which shifts the frame of CDS `[[0, 99]]` `+`.
    // The row of `frame.tsv` that records it reads on past the edit, so the
    // frame is not cut at the deletion.
    let template: String = synthetic("native-deletion.tsv", "del-02");
    let spec = json!({
        "schema": "luria.q5-request/1",
        "coordinates": "zero-based-half-open",
        "template": {
            "id": "syn",
            "fasta": "syn.fa",
            "topology": "linear",
            "cds": [{
                "id": "syn",
                "intervals": [[0, 99]],
                "strand": "+",
                "completeness": "complete",
                "geneticCode": "standard",
                "numbering": "from-cds-start"
            }]
        },
        "edit": {
            "name": "del-02",
            "deletion": { "interval": [45, 46], "from": "T" }
        },
        "design": design()
    });
    let run: Run = run_synthetic("del-02", &template, spec);
    assert_eq!(run.document.state, "verified", "del-02 is verified");
    assert_eq!(run.document.results.len(), 1, "del-02 has one member");
    let row: Row = frame_row("syn-del-frameshift");
    let frames: Vec<MemberFrame> = frames_of(&run);
    assert_eq!(
        in_window(&first_frame(&frames), row.window_start, row.window_end),
        parse_residues(&row.residues, "syn-del-frameshift"),
        "row syn-del-frameshift of {}: a frameshift reads on past the deletion",
        FIXTURE
    );
}

#[test]
fn an_insertion_before_a_minus_strand_start_moves_it() {
    // `native-insertion.tsv` row `ins-04`: seven bases inserted at 50, read on
    // the same template with CDS `[[0, 99]]` `-`. The CDS's first base on its
    // own strand is template 98, which lies past the insertion, so the edit
    // moves it by the seven bases written; the row counts its residues from
    // that moved start, so an insertion whose span is not the point alone
    // gives a different protein.
    let template: String = synthetic("native-insertion.tsv", "ins-04");
    let spec = json!({
        "schema": "luria.q5-request/1",
        "coordinates": "zero-based-half-open",
        "template": {
            "id": "syn",
            "fasta": "syn.fa",
            "topology": "linear",
            "cds": [{
                "id": "syn",
                "intervals": [[0, 99]],
                "strand": "-",
                "completeness": "complete",
                "geneticCode": "standard",
                "numbering": "from-cds-start"
            }]
        },
        "edit": {
            "name": "ins-04",
            "insertion": {
                "point": 50,
                "before": template[49..50].to_string(),
                "after": template[50..51].to_string(),
                "insert": "GATTACA"
            }
        },
        "design": design()
    });
    let run: Run = run_synthetic("ins-04-minus", &template, spec);
    assert_eq!(run.document.state, "verified", "ins-04 is verified");
    assert_eq!(run.document.results.len(), 1, "ins-04 has one member");
    assert!(
        run.document.results[0].design.is_some(),
        "the insertion is designed, so the run is verified with a design"
    );
    let row: Row = frame_row("syn-ins-minus");
    let frames: Vec<MemberFrame> = frames_of(&run);
    assert_eq!(
        in_window(&first_frame(&frames), row.window_start, row.window_end),
        parse_residues(&row.residues, "syn-ins-minus"),
        "row syn-ins-minus of {}: the moved first base reads as the row records",
        FIXTURE
    );
}

#[test]
fn a_changed_construct_gives_no_frame() {
    let run: Run = run_request("changed-construct", request("native-01", native_edit(), 15));
    let frames: Vec<MemberFrame> = frames_of(&run);
    assert_eq!(frames.len(), 1, "the run has one member");
    assert!(
        !first_frame(&frames).is_empty(),
        "the construct as core made it gives a frame"
    );

    // One base of the construct core made is changed, so its digest is no
    // longer the one the result reports for that member (D005, fail closed).
    let mut letters: Vec<char> = run.constructs[0].1.letters();
    letters[0] = if letters[0] == 'A' { 'C' } else { 'A' };
    let text: String = letters.iter().collect();
    let changed: Sequence = normalize(run.constructs[0].1.name(), text, SequenceTopology::Linear)
        .expect("the changed construct normalizes");

    let mut changed_constructs: Vec<(String, Sequence)> = run.constructs.clone();
    changed_constructs[0].1 = changed;
    let frames: Vec<MemberFrame> = member_frames(
        &run.document,
        &run.template,
        &run.annotations,
        &changed_constructs,
    );
    assert_eq!(
        frames[0],
        Err(NOT_THE_CONSTRUCT.to_string()),
        "a construct whose digest is not the one the member reports gives no frame"
    );
}

#[test]
fn no_target_gives_no_frame() {
    let mut run: Run = run_request("no-target", request("native-01", native_edit(), 15));
    run.document.target = None;
    let frames: Vec<MemberFrame> = frames_of(&run);
    assert_eq!(
        frames[0],
        Err(NO_TARGET.to_string()),
        "a result naming no target CDS has no protein to read"
    );
}
