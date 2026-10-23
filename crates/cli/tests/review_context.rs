//! The text of `luria review`'s sequence-context screen (milestone 7d, plan
//! §5.3, §5.2 screen 4): the lines `luria_cli::review::text::context` builds
//! from a run's result document and the template's bases, compared byte for byte
//! with the three fixtures of `tests/fixtures/review/`.
//!
//! The expectations are those fixtures, which
//! `tests/fixtures/review/reference/context.py` writes from independently
//! verified rows of `tests/fixtures/q5-primer/` (`native.tsv` row `native-01`,
//! `native-deletion.tsv` row `del-11`, `native-insertion.tsv` row `ins-14`) and
//! the SID-1 cDNA itself, by a script that shares no code with `crates/` (see
//! `tests/fixtures/review/README.md`). No line here is copied from running this
//! code. The three requests, bounds and edits are the ones
//! `tests/end_to_end.rs` already checks against those same rows
//! (`the_native_substitution_rows_match_the_table`,
//! `the_deletion_row_matches_the_table`, `the_insertion_row_matches_the_table`),
//! so the design the screen lays out is the design those tables pin, and the
//! template's bases are read again by `luria_cli::prepare::prepare` and accepted
//! only when their digest equals the document's, as D005 requires.

use serde_json::{json, Value};

use luria_cli::result::ResultDocument;
use luria_cli::review::matching_template;
use luria_cli::review::text;
use luria_cli::run::run;
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
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("review-context-{}", test));
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

/// The whole screen, as a fixture file holds it: every line followed by `\n`,
/// joined.
fn written(lines: &[String]) -> String {
    lines.iter().map(|line| format!("{}\n", line)).collect()
}

/// One fixture's whole text.
fn fixture(name: &str) -> String {
    fs::read_to_string(PathBuf::from("../../tests/fixtures/review").join(name)).unwrap()
}

/// Read the template again, and accept its bases only when their digest is the
/// document's: the screen's content when the template is the one the result was
/// computed on (D005).
fn bases(document: &ResultDocument, path: &Path) -> Vec<char> {
    let prepared = luria_cli::prepare::prepare(path).expect("the request prepares");
    matching_template(document, &prepared.template).expect("the template is the one the run used")
}

/// Run one request on SID-1 and lay its one member's design out against the
/// template.
fn context_of(test: &str, name: &str, edit: Value, bound: u64) -> String {
    let dir = workspace(test);
    let path = dir.join("request.json");
    fs::write(&path, request(name, edit, bound).to_string()).unwrap();
    let outcome = run(&path);
    let document: ResultDocument = outcome.document;
    assert_eq!(document.state, "verified", "{}: {}", test, name);
    assert_eq!(document.results.len(), 1, "{}: {}", test, name);
    written(&text::context(&document, 0, Some(&bases(&document, &path))))
}

/// The `template` column of one row of `tests/fixtures/q5-primer/native-insertion.tsv`,
/// found by its `case_id` column and read at test time, without its `seq:` prefix.
fn synthetic(row: &str) -> String {
    let text = fs::read_to_string("../../tests/fixtures/q5-primer/native-insertion.tsv").unwrap();
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
    panic!("no row {} in native-insertion.tsv", row)
}

/// The replacement `native.tsv` row `native-01` records.
fn native_edit() -> Value {
    json!({ "replacement": { "interval": [1209, 1212], "from": "GAT", "to": "AAC" } })
}

#[test]
fn the_native_01_context_is_the_fixture() {
    assert_eq!(
        context_of("native-01", "native-01", native_edit(), 15),
        fixture("context-native-01.txt"),
        "the screen lays the pair out as the table's own spans and tails place it"
    );
}

#[test]
fn the_del_11_context_is_the_fixture() {
    let edit = json!({ "deletion": { "interval": [1209, 1212], "from": "GAT" } });
    assert_eq!(
        context_of("del-11", "del-11", edit, 15),
        fixture("context-del-11.txt"),
        "a deletion's pair is laid out with the bases removed marked as edited"
    );
}

#[test]
fn the_ins_14_context_is_the_fixture() {
    let edit =
        json!({ "insertion": { "point": 1209, "before": "A", "after": "G", "insert": "GAT" } });
    assert_eq!(
        context_of("ins-14", "ins-14", edit, 15),
        fixture("context-ins-14.txt"),
        "an insertion's tail sits left of its forward annealing span, and `+` marks the point"
    );
}

#[test]
fn the_ins_04_context_is_the_fixture() {
    // `native-insertion.tsv` row `ins-04`: seven bases inserted at 50 on the row's
    // own synthetic template, named `syn`. Seven is past the six a single tail
    // takes, so the forward primer keeps the last four bases as its tail and the
    // reverse primer takes the first three, written right of its annealing span.
    let template: String = synthetic("ins-04");
    let point: usize = 50;
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("review-context-ins-04");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("syn.fa"), format!(">syn\n{}\n", template)).unwrap();
    let path = dir.join("request.json");
    fs::write(
        &path,
        json!({
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
                "name": "ins-04",
                "insertion": {
                    "point": point,
                    "before": template[point - 1..point].to_string(),
                    "after": template[point..point + 1].to_string(),
                    "insert": "GATTACA"
                }
            },
            "design": design()
        })
        .to_string(),
    )
    .unwrap();
    let document: ResultDocument = run(&path).document;
    let letters: Vec<char> = bases(&document, &path);
    assert_eq!(letters.len(), 100, "the row's template column is 100 bases");
    assert_eq!(document.state, "verified", "ins-04");
    assert_eq!(document.results.len(), 1, "ins-04");
    assert_eq!(
        written(&text::context(&document, 0, Some(&letters))),
        fixture("context-ins-04.txt"),
        "both primers get a tail, the reverse one written right of its annealing span"
    );
}

#[test]
fn a_changed_template_gives_no_context() {
    let dir = workspace("changed-template");
    let path = dir.join("request.json");
    fs::write(&path, request("native-01", native_edit(), 15).to_string()).unwrap();
    let document: ResultDocument = run(&path).document;
    assert_eq!(document.state, "verified");
    assert_eq!(document.results.len(), 1);

    // One base of the FASTA in the workspace is changed, so the template there
    // is no longer the one this result was computed on.
    let fasta = dir.join("sid-1_cDNA.fa");
    let mut lines: Vec<String> = fs::read_to_string(&fasta)
        .unwrap()
        .lines()
        .map(|line| line.to_string())
        .collect();
    assert_eq!(
        &lines[1][0..1],
        "A",
        "the sequence starts at the second line"
    );
    lines[1] = format!("C{}", &lines[1][1..]);
    fs::write(&fasta, format!("{}\n", lines.join("\n"))).unwrap();

    // `prepare` refuses it, because the request declares the digest of the
    // sequence it names, and the changed sequence's digest is not that.
    assert!(
        luria_cli::prepare::prepare(&path).is_err(),
        "a template whose digest does not match the request's does not prepare"
    );
    let changed: Sequence = read_fasta(
        Identifier::create("sid-1".to_string()).expect("an identifier"),
        fs::read_to_string(&fasta).unwrap(),
        SequenceTopology::Linear,
    )
    .expect("the changed record reads")
    .sequence();
    assert_eq!(
        matching_template(&document, &changed),
        None,
        "the changed template's digest is not the document's"
    );
    assert_eq!(
        text::context(&document, 0, None),
        [UNAVAILABLE.to_string()],
        "without the template's bases the screen draws no sequence"
    );
}

#[test]
fn a_design_failure_has_no_context() {
    // The pBR322 `tet` `T99C` request of `tests/review_text.rs`'s
    // `design_failure`, whose one member carries a design failure (G7, D029):
    // there is no pair to lay out, so the screen says so even with the template
    // read again.
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("review-context-design-failure");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
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
    let document: ResultDocument = run(&path).document;
    assert_eq!(document.state, "verified");
    assert_eq!(document.results.len(), 1);
    assert!(document.results[0].design.is_none());
    assert_eq!(
        text::context(&document, 0, Some(&bases(&document, &path))),
        ["no design, so no sequence context".to_string()],
        "a member with no design has no sequence context to draw"
    );
}
