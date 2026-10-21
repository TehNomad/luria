//! The export from the review screen (milestone 7g1, plan §3 rule 2, §5.4
//! "Writing", M7-4): the files `write_export` writes are the bytes the CLI
//! gives, and nothing is replaced.
//!
//! The SID-1 `del-11` + `ins-14` set is the builder of
//! `tests/review_order.rs`, copied: the designs and the template are the ones
//! `tests/fixtures/q5-primer/native-deletion.tsv` row `del-11` and
//! `native-insertion.tsv` row `ins-14` pin, and the order table is the fixture
//! `tests/fixtures/review/order-sid1-del11-ins14.tsv`, which
//! `reference/order.py` writes from those rows. The pBR322 `T99C` document is
//! `tests/review_text.rs`'s `design_failure` builder: a verified edit no primer
//! pair can be designed for (G7, D029), so its table is refused. The result and
//! FASTA bytes are what `render` and `run_with_output` give for the same
//! request, never pasted.

use serde_json::{json, Value};

use luria_cli::result::{render, ResultDocument};
use luria_cli::review::export::{export_plan, write_export, ExportPlan};
use luria_cli::review::order::OrderRefusal;
use luria_cli::run::{run, run_with_output};
use std::fs;
use std::path::{Path, PathBuf};

/// The SID-1 cDNA's digest, from `tests/fixtures/fasta/expected.tsv`, as
/// `tests/review_order.rs` declares it.
const SID1_DIGEST: &str = "54704740a9f04b79d17147ef95788ad9f0f11e5263843a7ed490924d43b55e3a";

/// pBR322's digest, from `tests/fixtures/genbank/pbr322-record.tsv`, as
/// `tests/review_text.rs` declares it.
const PBR322_DIGEST: &str = "536042a7485bdabdf49c4d21a306213a098d77d19bdba57a1c6adc76ce2b8372";

/// A fresh directory for one test, as `tests/review_order.rs`'s `sid1_workspace`
/// is, with a copy of the SID-1 cDNA beside the request.
fn sid1_workspace(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("review-export-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::copy(
        "../../tests/fixtures/example-construct/input/sid-1_cDNA.fa",
        dir.join("sid-1_cDNA.fa"),
    )
    .unwrap();
    dir
}

/// A fresh directory for one test, with no sequence file in it, as
/// `tests/review_order.rs`'s `workspace` is.
fn workspace(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("review-export-{}", test));
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

/// The SID-1 cDNA as a FASTA template, as `tests/review_order.rs`'s
/// `sid1_template()` declares it.
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

/// The two-member set of the fixture: `del-11` and `ins-14` on SID-1 at bound
/// 15, in that order, as `tests/review_order.rs`'s `set_request()` builds it.
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

/// `set.json`, the request file the export is named for, in `dir`, and the
/// document it runs to: `verified` with `del-11` then `ins-14`.
fn set_of(test: &str) -> (PathBuf, PathBuf, ResultDocument) {
    let dir: PathBuf = sid1_workspace(test);
    let request: PathBuf = dir.join("set.json");
    fs::write(&request, set_request().to_string()).unwrap();
    let document: ResultDocument = run(&request).document;
    assert_eq!(document.state, "verified");
    assert_eq!(
        document.results.len(),
        2,
        "del-11 then ins-14: {:?}",
        document
            .results
            .iter()
            .map(|m| m.name.clone())
            .collect::<Vec<String>>()
    );
    (dir, request, document)
}

/// The export directory, which does not exist yet, as §4.1 states.
fn out_of(dir: &Path) -> PathBuf {
    dir.join("out")
}

/// Every file in `dir`, as name and bytes, so a test can say the export left
/// what it found.
fn files(dir: &Path) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = fs::read_dir(dir)
        .expect("the export directory is readable")
        .map(|entry| {
            let path: PathBuf = entry.expect("an entry").path();
            (
                path.file_name()
                    .expect("a file name")
                    .to_string_lossy()
                    .into_owned(),
                fs::read_to_string(&path).expect("the file is readable"),
            )
        })
        .collect();
    found.sort();
    found
}

#[test]
fn the_export_is_what_the_cli_writes() {
    let (dir, request, document) = set_of("what_the_cli_writes");
    let out: PathBuf = out_of(&dir);
    assert!(!out.exists(), "the export directory does not exist yet");

    let plan: ExportPlan = export_plan(&document, &request, &out);
    assert_eq!(plan.result_file, "set.result.json");
    assert_eq!(plan.order_file, "set.order.tsv");
    assert_eq!(
        plan.constructs,
        vec!["del-11.fa".to_string(), "ins-14.fa".to_string()],
        "one FASTA name per member, in the document's order"
    );

    let written: Vec<String> =
        write_export(&plan, &document).expect("nothing is in the way, so the export writes");
    assert_eq!(
        written,
        vec![
            "del-11.fa".to_string(),
            "ins-14.fa".to_string(),
            "set.result.json".to_string(),
            "set.order.tsv".to_string(),
        ],
        "the constructs, then the result, then the table"
    );

    // The result file is what `luria q5 design set.json` prints.
    assert_eq!(
        fs::read_to_string(out.join("set.result.json")).unwrap(),
        render(&run(&request).document),
        "the result is `render` of the same document, byte for byte"
    );

    // The table is the fixture, byte for byte.
    assert_eq!(
        fs::read_to_string(out.join("set.order.tsv")).unwrap(),
        fs::read_to_string("../../tests/fixtures/review/order-sid1-del11-ins14.tsv").unwrap(),
        "the order file is the fixture's bytes"
    );

    // Every construct is the file `--write-edited` writes for that member, in a
    // second, empty directory, so the FASTA bytes are `write.rs`' own.
    let other: PathBuf = dir.join("other");
    run_with_output(&request, Some(&other));
    for name in ["del-11.fa", "ins-14.fa"] {
        assert_eq!(
            fs::read_to_string(out.join(name)).unwrap(),
            fs::read_to_string(other.join(name)).unwrap(),
            "{} is the file --write-edited writes for the same member",
            name
        );
    }
}

#[test]
fn nothing_is_replaced() {
    let (dir, request, document) = set_of("replaced");
    let out: PathBuf = out_of(&dir);

    let plan: ExportPlan = export_plan(&document, &request, &out);
    write_export(&plan, &document).expect("the first export writes");
    let before: Vec<(String, String)> = files(&out);
    assert_eq!(
        before
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<String>>(),
        vec![
            "del-11.fa".to_string(),
            "ins-14.fa".to_string(),
            "set.order.tsv".to_string(),
            "set.result.json".to_string(),
        ],
        "the first export wrote exactly the plan's files: {:?}",
        before
    );

    // The second export finds a name taken, so nothing is written, and what is
    // on disk is untouched (G8, D041).
    let second: Result<Vec<String>, String> = write_export(&plan, &document);
    match second {
        Err(message) => {
            assert_eq!(
                message,
                "set.result.json already exists; luria never replaces a file; nothing was written"
                    .to_string(),
                "the refusal names the file it found, and says nothing was written"
            );
        }
        Ok(names) => panic!("an existing file must refuse the export: {:?}", names),
    }
    assert_eq!(
        files(&out),
        before,
        "every file's content is unchanged, and no new file appeared"
    );
}

#[test]
fn an_existing_construct_blocks_the_export() {
    let (dir, request, document) = set_of("existing_construct");
    let out: PathBuf = out_of(&dir);
    fs::create_dir_all(&out).unwrap();
    fs::write(out.join("del-11.fa"), "a construct already on disk\n").unwrap();

    let plan: ExportPlan = export_plan(&document, &request, &out);
    let outcome: Result<Vec<String>, String> = write_export(&plan, &document);
    match outcome {
        Err(message) => assert_eq!(
            message,
            "will not overwrite the existing file 'del-11.fa'; nothing was written".to_string(),
            "--write-edited's own refusal, as the screen line the contract names"
        ),
        Ok(names) => panic!("an existing construct must refuse the export: {:?}", names),
    }

    assert!(
        !out.join("set.result.json").exists(),
        "the result file is not written, because nothing is written"
    );
    assert!(
        !out.join("set.order.tsv").exists(),
        "the order file is not written either"
    );
    assert_eq!(
        fs::read_to_string(out.join("del-11.fa")).unwrap(),
        "a construct already on disk\n",
        "the file that was there is not written through or replaced"
    );
    assert_eq!(
        files(&out),
        vec![(
            "del-11.fa".to_string(),
            "a construct already on disk\n".to_string()
        )],
        "the directory holds only what was already in it"
    );
}

#[test]
fn a_changed_request_is_not_exported() {
    let (dir, request, document) = set_of("changed_request");
    let out: PathBuf = out_of(&dir);

    // The request file changed after the run the screen shows, so a fresh run
    // gives a different document: the export refuses, and the directory the
    // export would create is never created (D012).
    let mut request_json: Value = set_request();
    request_json["design"]["minAnnealingOverlap"] = json!(16);
    fs::write(&request, request_json.to_string()).unwrap();

    let plan: ExportPlan = export_plan(&document, &request, &out);
    assert_eq!(
        write_export(&plan, &document),
        Err("the request gives a different result now; nothing was written".to_string()),
        "the request no longer gives the document on screen"
    );
    assert!(!out.exists(), "the export directory holds nothing");
    // The request the run of the screen read is the one the export refuses.
    let fresh: ResultDocument = run(&request).document;
    assert_ne!(fresh, document, "a bound of 16 gives a different design");
}

#[test]
fn a_result_with_no_design_exports_without_a_table() {
    // The pBR322 `tet` `T99C` request of `tests/review_text.rs`'s
    // `design_failure`: a verified edit that no primer pair can be designed for
    // (G7, D029), so its member has no row and there is no table, while the
    // result file and the edited construct are written as for any run.
    let dir: PathBuf = workspace("no_table");
    fs::copy(
        "../../tests/fixtures/genbank/raw/J01749.1.gb",
        dir.join("J01749.1.gb"),
    )
    .unwrap();
    let request: PathBuf = dir.join("tet.json");
    fs::write(
        &request,
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
    let document: ResultDocument = run(&request).document;
    assert_eq!(document.state, "verified");
    assert_eq!(document.results.len(), 1);
    assert!(document.results[0].design.is_none());

    let out: PathBuf = out_of(&dir);
    let plan: ExportPlan = export_plan(&document, &request, &out);
    assert_eq!(
        plan.order,
        Err(OrderRefusal::NothingDesigned),
        "no member has a primer pair, so the plan says there is no table"
    );

    let written: Vec<String> =
        write_export(&plan, &document).expect("the result and the construct are written");
    assert_eq!(
        written,
        vec!["T99C.fa".to_string(), "tet.result.json".to_string()],
        "the result file and the construct, and no order file"
    );
    assert_eq!(
        fs::read_to_string(out.join("tet.result.json")).unwrap(),
        render(&run(&request).document),
        "the result is `render` of the same document, byte for byte"
    );
    let other: PathBuf = dir.join("other");
    run_with_output(&request, Some(&other));
    assert_eq!(
        fs::read_to_string(out.join("T99C.fa")).unwrap(),
        fs::read_to_string(other.join("T99C.fa")).unwrap(),
        "the construct is the file --write-edited writes for the same member"
    );
    assert!(
        !out.join("tet.order.tsv").exists(),
        "the refused table is not written"
    );
}
