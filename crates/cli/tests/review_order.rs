//! The IDT order table `luria review` exports (milestone 7g2, plan §5.4,
//! M7-4, M7-7 to M7-13): the bytes `luria_cli::review::order::order_table`
//! builds from a run's result document, compared byte for byte with the fixture
//! `tests/fixtures/review/order-sid1-del11-ins14.tsv`.
//!
//! The expectation is that fixture, which
//! `tests/fixtures/review/reference/order.py` writes from the `forward` and
//! `reverse` columns of `tests/fixtures/q5-primer/native-deletion.tsv` row
//! `del-11` and `tests/fixtures/q5-primer/native-insertion.tsv` row `ins-14`,
//! rows derived without `crates/core` (see `tests/fixtures/q5-primer/README.md`
//! and `tests/fixtures/review/README.md`). No byte here is copied from running
//! this code. The two edits, bound and template are the ones
//! `tests/end_to_end.rs` already checks against those same rows
//! (`the_deletion_row_matches_the_table`,
//! `the_insertion_row_matches_the_table`), so the designs the table lists are
//! the designs those tables pin, and the template id in every row name is the
//! `sid-1` the request declares.

use serde_json::{json, Value};

use luria_cli::result::ResultDocument;
use luria_cli::review::order::{order_table, OrderRefusal, OrderTable};
use luria_cli::run::run;
use std::fs;
use std::path::{Path, PathBuf};

/// The SID-1 cDNA's digest, from `tests/fixtures/fasta/expected.tsv`, as
/// `tests/review_context.rs` declares it.
const SID1_DIGEST: &str = "54704740a9f04b79d17147ef95788ad9f0f11e5263843a7ed490924d43b55e3a";

/// pBR322's digest, from `tests/fixtures/genbank/pbr322-record.tsv`, as
/// `tests/review_text.rs` declares it.
const PBR322_DIGEST: &str = "536042a7485bdabdf49c4d21a306213a098d77d19bdba57a1c6adc76ce2b8372";

/// A fresh directory for one test, as `tests/review_context.rs`'s `workspace`
/// is, with a copy of the SID-1 cDNA beside the request.
fn sid1_workspace(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("review-order-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::copy(
        "../../tests/fixtures/example-construct/input/sid-1_cDNA.fa",
        dir.join("sid-1_cDNA.fa"),
    )
    .unwrap();
    dir
}

/// A fresh directory for one test, with no sequence file in it.
fn workspace(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("review-order-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
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
/// The id is the one every row name carries.
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

/// The two-member set of the fixture: `del-11`, the deletion of
/// `native-deletion.tsv` row `del-11`, and `ins-14`, the insertion of
/// `native-insertion.tsv` row `ins-14`, on SID-1 at bound 15, in that order.
fn set_request() -> Value {
    json!({
        "schema": "luria.q5-request/1",
        "coordinates": "zero-based-half-open",
        "template": sid1_template(),
        "scope": "sid-1",
        "variants": {
            "composition": "separate_variants",
            "members": [
                { "name": "del-11", "deletion": { "interval": [1209, 1212], "from": "GAT" } },
                { "name": "ins-14", "insertion": { "point": 1209, "before": "A", "after": "G",
                                                    "insert": "GAT" } }
            ]
        },
        "design": design()
    })
}

/// Run one request in `dir` and report its document.
fn run_in(dir: &Path, name: &str) -> ResultDocument {
    let path: PathBuf = dir.join(name);
    run(&path).document
}

#[test]
fn the_table_is_the_fixture() {
    let dir = sid1_workspace("set");
    fs::write(dir.join("request.json"), set_request().to_string()).unwrap();
    let document: ResultDocument = run_in(&dir, "request.json");
    assert_eq!(document.state, "verified");
    assert_eq!(document.results.len(), 2, "del-11 then ins-14");

    let table: OrderTable = order_table(&document).expect("both members are designed");
    let expected: String =
        fs::read_to_string("../../tests/fixtures/review/order-sid1-del11-ins14.tsv").unwrap();
    assert_eq!(
        table.text, expected,
        "the table is the fixture's bytes, header included, every row followed by a line end"
    );
    assert_eq!(
        table.left_out,
        Vec::<String>::new(),
        "every member is designed, so nothing is left out"
    );
}

#[test]
fn a_design_failure_is_left_out_and_nothing_else_is_ordered() {
    // The pBR322 `tet` `T99C` request of `tests/review_text.rs`'s
    // `design_failure`: a verified edit that no primer pair can be designed
    // for (G7, D029), so its one member has no row, and there is no row at all
    // to write.
    let dir = workspace("design-failure");
    fs::copy(
        "../../tests/fixtures/genbank/raw/J01749.1.gb",
        dir.join("J01749.1.gb"),
    )
    .unwrap();
    fs::write(
        dir.join("request.json"),
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
    let document: ResultDocument = run_in(&dir, "request.json");
    assert_eq!(document.state, "verified");
    assert_eq!(document.results.len(), 1);
    assert!(document.results[0].design.is_none());

    assert_eq!(
        order_table(&document),
        Err(OrderRefusal::NothingDesigned),
        "no member has a primer pair, so there is no table"
    );
}

#[test]
fn a_failed_result_is_not_ordered() {
    // The request file that is not in the directory, as `tests/review_text.rs`'s
    // `failed` runs it: the document says `failed`, so no order is written even
    // though nothing else about it is designed.
    let dir = workspace("failed");
    let document: ResultDocument = run_in(&dir, "no-such-request.json");
    assert_eq!(document.state, "failed");

    assert_eq!(
        order_table(&document),
        Err(OrderRefusal::NotVerified {
            state: "failed".to_string()
        }),
        "a result that is not verified orders nothing"
    );
}
