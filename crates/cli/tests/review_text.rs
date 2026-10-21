//! The text of `luria review`'s three result screens (milestone 7c1): the
//! overview, the problem screen and the provenance screen, as the lines
//! `luria_cli::review::text` builds from the result document (plan §5.2 screens
//! 2, 5 and 6).
//!
//! Every expected line is built from the document's own field, or is a fact
//! recorded in an independently verified fixture table:
//! `tests/fixtures/genbank/synthetic-linear-record.tsv` (LOCUS `synlin`, 393 bp,
//! `linear`, sha256 `a8b4ddcfa617179143d428dc103aabb7b13cf27f7b323407d11db8a01105bbd7`,
//! accession `SYN0002`, version `SYN0002.1`),
//! `tests/fixtures/genbank/synthetic-linear-cds.tsv` (`synA`, `[42, 345)`, `+`,
//! its id from `/gene`),
//! `tests/fixtures/genbank/synthetic-linear-residues.tsv` (residue 42 is `A`
//! with codon `GCA`, residue 87 is `W` with codon `TGG`),
//! `tests/fixtures/genbank/pbr322-cds.tsv` (`tet` and `bla`) and
//! `tests/fixtures/genbank/pbr322-record.tsv` (the digest below). No value is
//! copied from running the code: the documents' primer values are not
//! independently verified for `synthetic-linear.gb`, so no line of a design is
//! asserted here, and every other expected string names a fixture fact.
//!
//! The documents are `luria_cli::run::run` on a request in a directory under
//! `CARGO_TARGET_TMPDIR`, as `tests/review_view.rs` does: the two-variant set of
//! `variants-separate.json`, the brief's fail-closed clarification as
//! `tests/end_to_end.rs`'s `the_fail_closed_example_asks_and_edits_nothing` runs
//! it, the pBR322 `tet` `T99C` design failure (G7, D029) as
//! `a_fasta_and_a_genbank_template_give_the_same_result` builds it, and a
//! request path that is not in the directory.

use serde_json::{json, Value};

use luria_cli::result::{MutationView, ResultDocument};
use luria_cli::review::text;
use luria_cli::run::run;
use std::fs;
use std::path::PathBuf;

/// pBR322's digest, from `tests/fixtures/genbank/pbr322-record.tsv`.
const PBR322_DIGEST: &str = "536042a7485bdabdf49c4d21a306213a098d77d19bdba57a1c6adc76ce2b8372";

/// `synthetic-linear.gb`'s digest, from
/// `tests/fixtures/genbank/synthetic-linear-record.tsv`.
const SYNLIN_DIGEST: &str = "a8b4ddcfa617179143d428dc103aabb7b13cf27f7b323407d11db8a01105bbd7";

/// A fresh directory for one test, as `tests/review_view.rs`'s `workspace` is.
fn workspace(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("review-text-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// The design block `tests/end_to_end.rs`'s `design()` states: the same bound
/// and buffer, so the run is the one that fixture pins.
fn design() -> Value {
    json!({
        "policy": "luriaNative",
        "style": "q5_polymerase",
        "minAnnealingOverlap": 15,
        "placement": "inline",
        "buffer": "q5_reaction"
    })
}

/// The two-variant set: `variants-separate.json` beside a copy of
/// `synthetic-linear.gb`. `state` `verified` and two members are pinned here,
/// as 7a's test and `tests/fixtures/nl/README.md` pin them.
fn set(test: &str) -> ResultDocument {
    let dir = workspace(test);
    fs::copy(
        "../../tests/fixtures/genbank/synthetic-linear.gb",
        dir.join("synthetic-linear.gb"),
    )
    .unwrap();
    fs::copy(
        "../../tests/fixtures/nl/request/variants-separate.json",
        dir.join("request.json"),
    )
    .unwrap();
    let document: ResultDocument = run(&dir.join("request.json")).document;
    assert_eq!(document.state, "verified");
    assert_eq!(document.results.len(), 2);
    document
}

/// The brief's fail-closed example, as `tests/end_to_end.rs`'s
/// `the_fail_closed_example_asks_and_edits_nothing` runs it: a residue number
/// with no CDS named, on a template annotated with two, so the run asks and
/// edits nothing (D005).
fn clarification(test: &str) -> ResultDocument {
    let dir = workspace(test);
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
                "cds": ["tet", "bla"]
            },
            "edit": {
                "name": "G42",
                "residue": { "number": 42, "from": "A", "to": "G", "policy": "default" }
            },
            "design": design()
        })
        .to_string(),
    )
    .unwrap();
    let document: ResultDocument = run(&path).document;
    assert_eq!(document.state, "needs_clarification");
    assert_eq!(document.results.len(), 0);
    document
}

/// pBR322 `tet`, `T99C`, the GenBank request of `tests/end_to_end.rs`'s
/// `a_fasta_and_a_genbank_template_give_the_same_result`: a verified edit that
/// no primer pair can be designed for, so its one member carries a
/// `designFailure` (G7, D029).
fn design_failure(test: &str) -> ResultDocument {
    let dir = workspace(test);
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
    assert!(document.results[0].design_failure.is_some());
    document
}

/// The document of a request file that is not in the directory: `failed`, with
/// no member and no design to show.
fn failed(test: &str) -> ResultDocument {
    let dir = workspace(test);
    let document: ResultDocument = run(&dir.join("no-such-request.json")).document;
    assert_eq!(document.state, "failed");
    document
}

/// The written codon a member's own residue mutation holds, read from the
/// document.
fn written_codon(mutation: &MutationView) -> String {
    match mutation {
        MutationView::Residue { written_codon, .. } => written_codon.clone(),
        other => panic!("a residue mutation: {:?}", other),
    }
}

/// A member's `designFailure` code, as the document holds it.
fn design_failure_code(document: &ResultDocument) -> &'static str {
    document.results[0]
        .design_failure
        .as_ref()
        .expect("a design failure")
        .code
}

/// The member rows: the marker (`> ` or `  `) is the only thing that makes a row
/// recognizable, so the lines with one are the rows.
fn member_rows(lines: &[String]) -> Vec<String> {
    lines
        .iter()
        .filter(|line| line.starts_with("> ") || line.starts_with("  "))
        .cloned()
        .collect()
}

/// How many times `expected` appears in `lines`, so "one line per entry" is
/// tested as one, not at least one.
fn count(lines: &[String], expected: &str) -> usize {
    lines.iter().filter(|line| *line == expected).count()
}

#[test]
fn the_overview_names_the_template_and_target() {
    let lines = text::overview(&set("overview"), 0);
    assert_eq!(lines[0], "template: synlin, linear, 393 bp");
    assert_eq!(lines[1], "target: synA, +, [42, 345)");
    assert_eq!(lines[2], "operation: variants");
}

#[test]
fn each_member_has_one_row_and_one_marker() {
    let document = set("rows");
    let lines = text::overview(&document, 1);
    let rows = member_rows(&lines);
    assert_eq!(rows.len(), 2, "one row per member: {:?}", lines);

    let marked: Vec<&String> = rows.iter().filter(|row| row.starts_with("> ")).collect();
    assert_eq!(marked.len(), 1, "exactly one marker: {:?}", rows);
    assert_eq!(
        *marked[0], rows[1],
        "the marker is on the member at `selected`"
    );

    let prefix: &str = "  A42G  synA A42G GCA->";
    assert!(rows[0].starts_with(prefix), "the first row: {:?}", rows[0]);
    assert!(
        rows[0][prefix.len()..].starts_with(&written_codon(&document.results[0].mutation)),
        "the written codon is the document's own: {:?}",
        rows[0]
    );
    assert!(
        rows[0].ends_with("designed  warnings: 0"),
        "the row ends with its outcome and its own warning count: {:?}",
        rows[0]
    );
    assert!(
        rows[1].starts_with("> W87A  synA W87A TGG->"),
        "{:?}",
        rows[1]
    );
}

#[test]
fn a_design_failure_is_on_the_overview_row() {
    let document = design_failure("failure_row");
    let rows = member_rows(&text::overview(&document, 0));
    assert_eq!(rows.len(), 1);
    assert!(
        rows[0].contains(&format!(
            "design failed: {}",
            design_failure_code(&document)
        )),
        "the row shows the document's own design failure code: {:?}",
        rows[0]
    );
    assert!(
        !rows[0].contains("designed"),
        "a design failure is shown as a failure, never as a design: {:?}",
        rows[0]
    );
}

#[test]
fn the_problem_line_appears_only_with_a_problem() {
    let lines = text::overview(&set("problem_line_set"), 0);
    assert!(
        !lines.iter().any(|line| line.as_str() == "e problem"),
        "a verified run with no clarification offers no problem screen: {:?}",
        lines
    );
    for (test, lines) in [
        (
            "problem_line_clarification",
            text::overview(&clarification("problem_line_clarification"), 0),
        ),
        (
            "problem_line_failed",
            text::overview(&failed("problem_line_failed"), 0),
        ),
    ] {
        assert!(
            lines.iter().any(|line| line.as_str() == "e problem"),
            "{} has a problem, so the line is there: {:?}",
            test,
            lines
        );
    }
}

#[test]
fn the_problem_screen_lists_the_clarification_and_candidates() {
    let document = clarification("clarification_lines");
    let lines = text::problem(&document);
    let expected: Vec<String> = vec![
        "clarifications:".to_string(),
        format!("  {}", document.clarifications[0]),
        "candidates:".to_string(),
        "  tet".to_string(),
        "  bla".to_string(),
    ];
    let positions: Vec<usize> = expected
        .iter()
        .map(|line| {
            lines
                .iter()
                .position(|row| *row == *line)
                .unwrap_or_else(|| panic!("{} is listed: {:?}", line, lines))
        })
        .collect();
    for pair in positions.windows(2) {
        assert!(
            pair[0] < pair[1],
            "the groups come in this order: {:?}",
            lines
        );
    }
    for line in &expected {
        assert_eq!(count(&lines, line), 1, "{:?} is listed once", lines);
    }
}

#[test]
fn the_problem_screen_shows_the_error() {
    let document = failed("failed_error");
    let error = document.error.as_ref().expect("an error");
    let lines = text::problem(&document);
    assert_eq!(lines[0], format!("error: {}", error.code));
    assert_eq!(lines[1], "category: malformed_input");
    assert_eq!(lines[2], format!("message: {}", error.message));
}

#[test]
fn the_provenance_screen_shows_the_record() {
    let document = set("provenance");
    let lines = text::provenance(&document);

    for expected in [
        "template: synlin".to_string(),
        format!("  sha256: {}", SYNLIN_DIGEST),
        "  length: 393 bp".to_string(),
        "  topology: linear".to_string(),
        "  locus: synlin".to_string(),
        "  accession: SYN0002".to_string(),
        "  version: SYN0002.1".to_string(),
    ] {
        assert_eq!(
            count(&lines, &expected),
            1,
            "{} is on the screen once",
            expected
        );
    }

    // The imported CDS, as `tests/fixtures/genbank/synthetic-linear-cds.tsv`
    // records it, with the line number the document itself holds.
    let cds = document
        .provenance
        .template
        .as_ref()
        .expect("a template")
        .genbank
        .as_ref()
        .expect("a GenBank import")
        .imported_cds[0]
        .clone();
    assert_eq!(
        count(
            &lines,
            &format!("    synA  +  [42, 345)  from /gene  line {}", cds.line)
        ),
        1,
        "{:?}",
        lines
    );

    // One line per algorithm the document names, from its own fields.
    for algorithm in &document.provenance.algorithms {
        assert_eq!(
            count(
                &lines,
                &format!(
                    "  {}: {} {}",
                    algorithm.role, algorithm.name, algorithm.version
                )
            ),
            1,
            "{} is named once",
            algorithm.role
        );
    }
    assert_eq!(count(&lines, "algorithms:"), 1);
}
