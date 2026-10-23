//! The text of the redesigned member screen's sequence block (milestone 7i3,
//! plan §5.3a): the lines `luria_cli::review::text::member_sequence` builds from
//! a run's result document, the template's bases, the member's mutant protein
//! and whether a long insertion is expanded, compared byte for byte with the
//! nine fixtures of `tests/fixtures/review/`.
//!
//! The expectations are those fixtures, which
//! `tests/fixtures/review/reference/member.py` writes from independently
//! verified rows of `tests/fixtures/q5-primer/` and the template sequences
//! themselves, by a script that shares no code with `crates/` (see
//! `tests/fixtures/review/README.md`). No line here is copied from running this
//! code. The requests, bounds, templates and edits are the ones
//! `tests/fixtures/q5-primer/` already pins and that
//! `tests/end_to_end.rs` checks, and every edit is built from the row's own
//! columns, never from a pasted value: the design the block lays out is the
//! design those tables pin, the template's bases are read again by `prepare` and
//! accepted only when their digest equals the document's (D005), and the protein
//! is the one core read on the construct `run_with_constructs` produced (7i2).

use serde_json::{json, Value};

use luria_cli::prepare::prepare;
use luria_cli::result::ResultDocument;
use luria_cli::review::matching_template;
use luria_cli::review::member_frames;
use luria_cli::review::text;
use luria_cli::review::MemberFrame;
use luria_cli::run::run_with_constructs;
use luria_core::fasta::read_fasta;
use luria_core::identity::Identifier;
use luria_core::sequence::{Sequence, SequenceTopology};
use std::fs;
use std::path::{Path, PathBuf};

/// The SID-1 cDNA's digest, from `tests/fixtures/fasta/expected.tsv`.
const SID1_DIGEST: &str = "54704740a9f04b79d17147ef95788ad9f0f11e5263843a7ed490924d43b55e3a";

/// pBR322's digest, from `tests/fixtures/genbank/pbr322-record.tsv`.
const PBR322_DIGEST: &str = "536042a7485bdabdf49c4d21a306213a098d77d19bdba57a1c6adc76ce2b8372";

/// The line the screen says when the template's bases are not there to draw
/// (D005), as `text::context` writes it.
const UNAVAILABLE: &str = "sequence context unavailable: the template could not be read again, \
                           or it differs from the one this result was computed on";

/// A fresh directory for one test, as `tests/end_to_end.rs`'s `workspace` is,
/// with a copy of the SID-1 cDNA beside the request.
fn workspace(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("review-member-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// A fresh SID-1 workspace, as `tests/review_context.rs`'s `workspace` is.
fn sid1_workspace(test: &str) -> PathBuf {
    let dir = workspace(test);
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

/// The synthetic template as `tests/review_frames.rs` declares the rows' `seq:`
/// templates: id `syn`, file `syn.fa`, one CDS over its whole 100 bases.
fn syn_template() -> Value {
    json!({
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
    })
}

/// A one-edit request on SID-1 at `bound`, as `tests/end_to_end.rs`'s `request`
/// writes it: `edit` is the one kind, with `name` added to it.
fn sid1_request(name: &str, edit: Value, bound: u64) -> Value {
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

/// A one-edit request on the synthetic template, as `tests/review_frames.rs`
/// writes its `seq:` rows: the same design, with `name` added to the edit.
fn syn_request(name: &str, edit: Value) -> Value {
    let mut edit_object = json!({ "name": name });
    for (key, value) in edit.as_object().expect("an edit object") {
        edit_object[key] = value.clone();
    }
    json!({
        "schema": "luria.q5-request/1",
        "coordinates": "zero-based-half-open",
        "template": syn_template(),
        "edit": edit_object,
        "design": design()
    })
}

/// The whole block, as a fixture file holds it: every line followed by `\n`,
/// joined.
fn written(lines: &[String]) -> String {
    lines.iter().map(|line| format!("{}\n", line)).collect()
}

/// One fixture's whole text.
fn fixture(name: &str) -> String {
    fs::read_to_string(PathBuf::from("../../tests/fixtures/review").join(name)).unwrap()
}

/// One column of one row of one `tests/fixtures/q5-primer/` table, found by its
/// `case_id` column and read at test time.
fn column_of(table: &str, case_id: &str, column: &str) -> String {
    let path = format!("../../tests/fixtures/q5-primer/{}", table);
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
    let lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    let header: Vec<String> = lines[0]
        .split('\t')
        .map(|field| field.to_string())
        .collect();
    for line in lines[1..].iter() {
        let fields: Vec<String> = line.split('\t').map(|field| field.to_string()).collect();
        if fields[0] == case_id {
            for (index, name) in header.iter().enumerate() {
                if name == column {
                    return fields[index].clone();
                }
            }
            panic!("no column {} in {}", column, table);
        }
    }
    panic!("no row {} in {}", case_id, table);
}

/// One `s:e:X` field as its two numbers, read from the row's own text.
fn span_of(field: &str) -> (u64, u64) {
    let parts: Vec<&str> = field.split(':').collect();
    assert!(
        parts.len() >= 2 && parts.len() <= 3,
        "field {:?} is not s:e or s:e:bases",
        field
    );
    (
        parts[0].parse::<u64>().unwrap(),
        parts[1].parse::<u64>().unwrap(),
    )
}

/// The bases one `s:e:X` field names.
fn bases_of(field: &str) -> String {
    let parts: Vec<&str> = field.split(':').collect();
    assert!(parts.len() == 3, "field {:?} names no bases", field);
    parts[2].to_string()
}

/// The replacement `native.tsv` row `native-01` records, from its `edit` column
/// (`s:e:from:to`).
fn native_edit() -> Value {
    let field: String = column_of("native.tsv", "native-01", "edit");
    let parts: Vec<&str> = field.split(':').collect();
    assert!(parts.len() == 4, "edit {:?} is not s:e:from:to", field);
    json!({ "replacement": {
        "interval": [parts[0].parse::<u64>().unwrap(), parts[1].parse::<u64>().unwrap()],
        "from": parts[2],
        "to": parts[3]
    } })
}

/// The deletion one `native-deletion.tsv` row states in its `deletion` column
/// (`s:e:X`).
fn row_deletion(row: &str) -> Value {
    let field: String = column_of("native-deletion.tsv", row, "deletion");
    json!({ "deletion": {
        "interval": [span_of(&field).0, span_of(&field).1],
        "from": bases_of(&field)
    } })
}

/// The replacement one `native-replacement.tsv` row states, from its `region`
/// column (`s:e:from`) and its `replacement` column.
fn row_replacement(row: &str) -> Value {
    let region: String = column_of("native-replacement.tsv", row, "region");
    json!({ "replacement": {
        "interval": [span_of(&region).0, span_of(&region).1],
        "from": bases_of(&region),
        "to": column_of("native-replacement.tsv", row, "replacement")
    } })
}

/// An insertion as one `native-insertion.tsv` row states it: its `point` and
/// `inserted` columns, with `before` and `after` the template's own bases at
/// `point - 1` and `point`.
fn row_insertion(row: &str, template: &str) -> Value {
    let point: u64 = column_of("native-insertion.tsv", row, "point")
        .parse::<u64>()
        .unwrap();
    json!({ "insertion": {
        "point": point,
        "before": template[(point - 1) as usize..point as usize].to_string(),
        "after": template[point as usize..point as usize + 1].to_string(),
        "insert": column_of("native-insertion.tsv", row, "inserted")
    } })
}

/// The `template` column of one row of one `tests/fixtures/q5-primer/` table,
/// without its `seq:` prefix.
fn synthetic(table: &str, row: &str) -> String {
    let column: String = column_of(table, row, "template");
    assert!(
        column.starts_with("seq:"),
        "row {} of {} gives a template file, not a sequence",
        row,
        table
    );
    column["seq:".len()..].to_string()
}

/// The SID-1 cDNA's bases, read at test time as the template the rows naming
/// `sid-1_cDNA.fa` are drawn on.
fn sid1_bases() -> String {
    let text = fs::read_to_string("../../tests/fixtures/example-construct/input/sid-1_cDNA.fa")
        .expect("the SID-1 cDNA reads");
    let sequence: Sequence = read_fasta(
        Identifier::create("sid-1".to_string()).expect("an identifier"),
        text,
        SequenceTopology::Linear,
    )
    .expect("the record reads")
    .sequence();
    sequence.letters().iter().collect()
}

/// One run: the document it gave, the template's bases as `matching_template`
/// accepts them, and every member's frame as `member_frames` gives it.
struct Run {
    document: ResultDocument,
    template: Vec<char>,
    frames: Vec<MemberFrame>,
}

/// Run one request file through `run_with_constructs` and prepare it, so the
/// bases are the template that run read and the frames are read on the
/// constructs that run made. The run is checked before the screen is built.
fn run_at(path: &Path, test: &str) -> Run {
    let (outcome, constructs) = run_with_constructs(path);
    let prepared = prepare(path).expect("the request prepares");
    assert_eq!(
        outcome.document.state, "verified",
        "{}: the run is verified",
        test
    );
    assert_eq!(outcome.document.results.len(), 1, "{}: one member", test);
    let template: Vec<char> = matching_template(&outcome.document, &prepared.template)
        .expect("the template is the one the run used");
    let frames: Vec<MemberFrame> = member_frames(
        &outcome.document,
        &prepared.template,
        &prepared.annotations,
        &constructs,
    );
    Run {
        document: outcome.document,
        template,
        frames,
    }
}

/// The sequence block of one request, written into `dir`, for member 0, with
/// `expanded` asked for a long insertion.
fn block_of(dir: &Path, spec: Value, expanded: bool, test: &str) -> String {
    let path = dir.join("request.json");
    fs::write(&path, spec.to_string()).unwrap();
    let run: Run = run_at(&path, test);
    written(&text::member_sequence(
        &run.document,
        0,
        Some(&run.template),
        run.frames.first(),
        expanded,
    ))
}

/// The block of a request on a synthetic FASTA template written beside the
/// request, as `tests/review_frames.rs` writes its `seq:` rows.
fn syn_block(table: &str, row: &str, edit: Value, expanded: bool, test: &str) -> String {
    let template: String = synthetic(table, row);
    let dir: PathBuf = workspace(test);
    fs::write(dir.join("syn.fa"), format!(">syn\n{}\n", template)).unwrap();
    block_of(&dir, syn_request(row, edit), expanded, test)
}

#[test]
fn native_01_is_the_fixture() {
    let dir: PathBuf = sid1_workspace("native-01");
    assert_eq!(
        block_of(
            &dir,
            sid1_request("native-01", native_edit(), 15),
            false,
            "native-01"
        ),
        fixture("member-native-01.txt"),
        "the replacement's written bases sit between the forward primer's context parts, \
         over the lower case bases removed, with the mutant protein between the strands"
    );
}

#[test]
fn del_11_is_the_fixture() {
    let dir: PathBuf = sid1_workspace("del-11");
    assert_eq!(
        block_of(
            &dir,
            sid1_request("del-11", row_deletion("del-11"), 15),
            false,
            "del-11"
        ),
        fixture("member-del-11.txt"),
        "a deletion's region is all lower case, and the protein past it is shifted by the bases removed"
    );
}

#[test]
fn ins_14_is_the_fixture() {
    let template: String = sid1_bases();
    let dir: PathBuf = sid1_workspace("ins-14");
    assert_eq!(
        block_of(
            &dir,
            sid1_request("ins-14", row_insertion("ins-14", &template), 15),
            false,
            "ins-14"
        ),
        fixture("member-ins-14.txt"),
        "an insertion's written bases are drawn lower case in its region, its tail left of the \
         forward annealing span, and the protein past the insertion shifted by them"
    );
}

#[test]
fn del_02_is_the_fixture() {
    assert_eq!(
        syn_block(
            "native-deletion.tsv",
            "del-02",
            row_deletion("del-02"),
            false,
            "del-02"
        ),
        fixture("member-del-02.txt"),
        "a single base deleted shifts the frame of the synthetic CDS past the edit"
    );
}

#[test]
fn ins_04_is_the_fixture() {
    let template: String = synthetic("native-insertion.tsv", "ins-04");
    assert_eq!(
        syn_block(
            "native-insertion.tsv",
            "ins-04",
            row_insertion("ins-04", &template),
            false,
            "ins-04"
        ),
        fixture("member-ins-04.txt"),
        "both primers take a tail from the seven bases inserted, the reverse one written right of \
         its annealing span, reversed"
    );
}

#[test]
fn rep_02_is_the_fixture() {
    assert_eq!(
        syn_block(
            "native-replacement.tsv",
            "rep-02",
            row_replacement("rep-02"),
            false,
            "rep-02"
        ),
        fixture("member-rep-02.txt"),
        "a replacement shorter than the region it removes leaves the mutation part's bases \
         written from the region's start, with a space in the forward row"
    );
}

#[test]
fn rep_06_is_the_fixture() {
    assert_eq!(
        syn_block(
            "native-replacement.tsv",
            "rep-06",
            row_replacement("rep-06"),
            false,
            "rep-06"
        ),
        fixture("member-rep-06.txt"),
        "a replacement longer than the region it removes writes the region lower case then `-`, \
         and its tails sit outside the annealing spans"
    );
}

#[test]
fn ins_13_collapsed_is_the_fixture() {
    let template: String = synthetic("native-insertion.tsv", "ins-13");
    assert_eq!(
        syn_block(
            "native-insertion.tsv",
            "ins-13",
            row_insertion("ins-13", &template),
            false,
            "ins-13-collapsed"
        ),
        fixture("member-ins-13.txt"),
        "210 bases inserted are a marker, no protein inside it, both tails hidden, and no block \
         split inside the marker"
    );
}

#[test]
fn ins_13_expanded_is_the_fixture() {
    let template: String = synthetic("native-insertion.tsv", "ins-13");
    assert_eq!(
        syn_block(
            "native-insertion.tsv",
            "ins-13",
            row_insertion("ins-13", &template),
            true,
            "ins-13-expanded"
        ),
        fixture("member-ins-13-expanded.txt"),
        "expanded, every base of the insertion is written, the tails drawn, and the protein reads \
         through all five blocks"
    );
}

#[test]
fn a_frame_error_says_so() {
    let dir: PathBuf = sid1_workspace("frame-error");
    let path = dir.join("request.json");
    fs::write(
        &path,
        sid1_request("native-01", native_edit(), 15).to_string(),
    )
    .unwrap();
    let run: Run = run_at(&path, "frame-error");

    // The fixture's block, with the protein row the bare label and the reason as
    // the last note: the block is `member-native-01.txt` without its protein row
    // and with the note added.
    let broken: MemberFrame = Err("x".to_string());
    let lines: Vec<String> =
        text::member_sequence(&run.document, 0, Some(&run.template), Some(&broken), false);
    let protein_rows: Vec<String> = lines
        .iter()
        .filter(|line| line.starts_with("protein"))
        .cloned()
        .collect();
    assert_eq!(
        protein_rows,
        vec!["protein".to_string(), "protein: x".to_string()],
        "the protein row is the bare label in every block, and the reason is the last note: {:?}",
        lines
    );
    assert_eq!(
        lines.last().expect("a block has lines").as_str(),
        "protein: x",
        "the last line names the reason"
    );
    assert_eq!(
        lines.iter().filter(|line| *line == "protein").count(),
        fixture("member-native-01.txt")
            .lines()
            .filter(|line| line.starts_with("protein"))
            .count(),
        "one protein row per block, as the fixture has blocks"
    );
    assert_eq!(
        lines
            .iter()
            .filter(|line| line.starts_with("template"))
            .count(),
        fixture("member-native-01.txt")
            .lines()
            .filter(|line| line.starts_with("template"))
            .count(),
        "the strands are drawn as the fixture draws them"
    );

    // The same with no frame at all: the label alone, and `protein: not available`
    // as the last note.
    let lines: Vec<String> =
        text::member_sequence(&run.document, 0, Some(&run.template), None, false);
    assert_eq!(
        lines.last().expect("a block has lines").as_str(),
        "protein: not available",
        "with no frame the note says so: {:?}",
        lines
    );
    assert_eq!(
        lines.iter().filter(|line| *line == "protein").count(),
        fixture("member-native-01.txt")
            .lines()
            .filter(|line| line.starts_with("protein"))
            .count(),
        "one bare protein row, one block, as the fixture has"
    );

    // Case 1: an index past the last member says so, with the template and the
    // frame there to draw.
    assert_eq!(
        text::member_sequence(
            &run.document,
            1,
            Some(&run.template),
            run.frames.first(),
            false
        ),
        vec!["no such member".to_string()],
        "an index out of range has no member to draw"
    );
}

#[test]
fn no_design_and_no_template() {
    // Case 2: a member with no design has no sequence to lay out, with the
    // template read again. The pBR322 `tet` `T99C` request of
    // `tests/review_context.rs`'s `a_design_failure_has_no_context` (G7, D029).
    let dir: PathBuf = workspace("design-failure");
    fs::copy(
        "../../tests/fixtures/genbank/raw/J01749.1.gb",
        dir.join("J01749.1.gb"),
    )
    .unwrap();
    let path = dir.join("request.json");
    fs::write(
        &path,
        json!({
            "schema": "luria.q5-request/1",
            "coordinates": "zero-based-half-open",
            "template": {
                "id": "pbr322",
                "genbank": "J01749.1.gb",
                "sha256": PBR322_DIGEST,
                "cds": ["tet"]
            },
            "scope": "tet",
            "edit": {
                "name": "T99C",
                "substitution": { "interval": [99, 100], "from": "T", "to": "C" }
            },
            "design": design()
        })
        .to_string(),
    )
    .unwrap();
    let run: Run = run_at(&path, "design-failure");
    assert!(
        run.document.results[0].design.is_none(),
        "the member has no design"
    );
    assert_eq!(
        written(&text::member_sequence(
            &run.document,
            0,
            Some(&run.template),
            run.frames.first(),
            false
        )),
        "no design, so no sequence\n",
        "a member with no design has no sequence to draw"
    );

    // Case 3: the template's bases are not the ones this result was computed on,
    // so the block is the one line `text::context` says, with no layout.
    let dir: PathBuf = sid1_workspace("no-template");
    let path = dir.join("request.json");
    fs::write(
        &path,
        sid1_request("native-01", native_edit(), 15).to_string(),
    )
    .unwrap();
    let run: Run = run_at(&path, "no-template");
    assert_eq!(
        written(&text::member_sequence(
            &run.document,
            0,
            None,
            run.frames.first(),
            false
        )),
        format!("{}\n", UNAVAILABLE),
        "without the template's bases the block draws no sequence"
    );
}
