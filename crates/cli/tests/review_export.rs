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
use luria_cli::review::export::{export_plan, taken, write_export, ExportPlan};
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
        vec!["sid-1_del-11.fa".to_string(), "sid-1_ins-14.fa".to_string()],
        "one FASTA name per member, named for the template, in the document's order"
    );

    let written: Vec<String> =
        write_export(&plan, &document, false).expect("nothing is in the way, so the export writes");
    assert_eq!(
        written,
        vec![
            "sid-1_del-11.fa".to_string(),
            "sid-1_ins-14.fa".to_string(),
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
    for (exported, written_name) in [
        ("sid-1_del-11.fa", "del-11.fa"),
        ("sid-1_ins-14.fa", "ins-14.fa"),
    ] {
        assert_eq!(
            fs::read_to_string(out.join(exported)).unwrap(),
            fs::read_to_string(other.join(written_name)).unwrap(),
            "{} is the file --write-edited writes for the same member",
            exported
        );
    }
}

#[test]
fn nothing_is_replaced() {
    let (dir, request, document) = set_of("replaced");
    let out: PathBuf = out_of(&dir);

    let plan: ExportPlan = export_plan(&document, &request, &out);
    write_export(&plan, &document, false).expect("the first export writes");
    let before: Vec<(String, String)> = files(&out);
    assert_eq!(
        before
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<String>>(),
        vec![
            "set.order.tsv".to_string(),
            "set.result.json".to_string(),
            "sid-1_del-11.fa".to_string(),
            "sid-1_ins-14.fa".to_string(),
        ],
        "the first export wrote exactly the plan's files: {:?}",
        before
    );

    // The second export finds a name taken, so nothing is written, and what is
    // on disk is untouched (G8, D041). The name it reports is the first of the
    // plan's that exists, which is a construct.
    let second: Result<Vec<String>, String> = write_export(&plan, &document, false);
    match second {
        Err(message) => {
            assert_eq!(
                message,
                "sid-1_del-11.fa already exists; luria never replaces a file; nothing was written"
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
    fs::write(out.join("sid-1_del-11.fa"), "a construct already on disk\n").unwrap();

    let plan: ExportPlan = export_plan(&document, &request, &out);
    let outcome: Result<Vec<String>, String> = write_export(&plan, &document, false);
    match outcome {
        Err(message) => assert_eq!(
            message,
            "sid-1_del-11.fa already exists; luria never replaces a file; nothing was written"
                .to_string(),
            "the taken name is named, and nothing was written"
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
        fs::read_to_string(out.join("sid-1_del-11.fa")).unwrap(),
        "a construct already on disk\n",
        "the file that was there is not written through or replaced"
    );
    assert_eq!(
        files(&out),
        vec![(
            "sid-1_del-11.fa".to_string(),
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
        write_export(&plan, &document, false),
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
        write_export(&plan, &document, false).expect("the result and the construct are written");
    assert_eq!(
        written,
        vec!["pbr322_T99C.fa".to_string(), "tet.result.json".to_string()],
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
        fs::read_to_string(out.join("pbr322_T99C.fa")).unwrap(),
        fs::read_to_string(other.join("T99C.fa")).unwrap(),
        "the construct is the file --write-edited writes for the same member"
    );
    assert!(
        !out.join("tet.order.tsv").exists(),
        "the refused table is not written"
    );
}

// M7-57: the template id names each construct, as the order table already names
// its primers (`sid-1_del-11_F`), so the export's names say which template the
// construct is of.
#[test]
fn constructs_are_named_by_template() {
    let (dir, request, document) = set_of("named_by_template");
    let plan: ExportPlan = export_plan(&document, &request, &out_of(&dir));
    assert_eq!(
        plan.constructs,
        vec!["sid-1_del-11.fa".to_string(), "sid-1_ins-14.fa".to_string()],
        "<template id>_<member>.fa, in the document's order"
    );
}

/// The bytes `--write-edited` writes for one member of the same request, in a
/// second, empty directory: `write.rs`' own writer, never copied from the export.
fn write_edited_bytes(dir: &Path, request: &Path, member: &str) -> Vec<u8> {
    let other: PathBuf = dir.join("write-edited");
    run_with_output(request, Some(&other));
    fs::read(other.join(member)).expect("--write-edited wrote the member's file")
}

// An export's construct is the file `luria q5 design --write-edited` writes for
// the same request, byte for byte, under its new name.
#[test]
fn a_written_construct_is_the_write_edited_bytes() {
    let (dir, request, document) = set_of("construct_bytes");
    let out: PathBuf = out_of(&dir);
    let plan: ExportPlan = export_plan(&document, &request, &out);
    write_export(&plan, &document, false).expect("nothing is in the way, so the export writes");

    assert_eq!(
        fs::read(out.join("sid-1_del-11.fa")).unwrap(),
        write_edited_bytes(&dir, &request, "del-11.fa"),
        "the exported construct is the bytes --write-edited writes, header and all"
    );
}

// M7-49: a name the export would write that exists refuses while `replace` is
// `false`, and is written over when the prompt was answered `y`.
#[test]
fn replace_writes_over_a_taken_name() {
    let (dir, request, document) = set_of("replace_taken");
    let out: PathBuf = out_of(&dir);
    fs::create_dir_all(&out).unwrap();
    fs::write(out.join("sid-1_del-11.fa"), "old\n").unwrap();

    let plan: ExportPlan = export_plan(&document, &request, &out);

    // Nothing is replaced, so nothing is written, and the file is as it was.
    let refused: Result<Vec<String>, String> = write_export(&plan, &document, false);
    assert!(
        refused.is_err(),
        "a taken name refuses the whole export: {:?}",
        refused
    );
    assert_eq!(
        fs::read_to_string(out.join("sid-1_del-11.fa")).unwrap(),
        "old\n",
        "the taken file is not written through or replaced"
    );
    assert!(
        !out.join("set.result.json").exists(),
        "a refused export writes nothing at all"
    );

    // `y` replaces the name it named, and the file is test 2's bytes.
    let written: Vec<String> =
        write_export(&plan, &document, true).expect("the answer y replaces the name");
    assert_eq!(
        written,
        vec![
            "sid-1_del-11.fa".to_string(),
            "sid-1_ins-14.fa".to_string(),
            "set.result.json".to_string(),
            "set.order.tsv".to_string(),
        ],
        "the whole export is written, in the plan's order"
    );
    assert_eq!(
        fs::read(out.join("sid-1_del-11.fa")).unwrap(),
        write_edited_bytes(&dir, &request, "del-11.fa"),
        "the replaced file is what --write-edited writes for the same member"
    );
}

// M7-49, C-1: a replace never writes through a link. The name is a link to a
// file outside the export directory, so `y` refuses, and the file the link points
// at is unchanged.
#[cfg(unix)]
#[test]
fn replace_never_writes_through_a_link() {
    use std::os::unix::fs::symlink;

    let (dir, request, document) = set_of("replace_link");
    let out: PathBuf = out_of(&dir);
    fs::create_dir_all(&out).unwrap();
    let outside: PathBuf = dir.join("outside.fa");
    fs::write(&outside, "the file the link points at\n").unwrap();
    symlink("../outside.fa", out.join("sid-1_del-11.fa")).unwrap();

    let plan: ExportPlan = export_plan(&document, &request, &out);
    let refused: Result<Vec<String>, String> = write_export(&plan, &document, true);
    assert!(
        refused.is_err(),
        "a link is never written through, so the export refuses: {:?}",
        refused
    );

    // The link is still a link, and its target is the bytes it was given: the
    // replace never went through it.
    assert!(
        fs::symlink_metadata(out.join("sid-1_del-11.fa"))
            .expect("the link is still there")
            .file_type()
            .is_symlink(),
        "the link itself is unchanged"
    );
    assert_eq!(
        fs::read_to_string(&outside).unwrap(),
        "the file the link points at\n",
        "nothing was written through the link"
    );
    assert!(
        !out.join("set.result.json").exists(),
        "a refused export writes nothing at all"
    );
    assert!(
        !out.join("sid-1_ins-14.fa").exists(),
        "the names after the refused one are not written either"
    );
}

// M7-49: the names the export would write that exist are listed, in the plan's
// order, so the prompt names them before `y` replaces them.
#[test]
fn taken_lists_existing_names_in_order() {
    let (dir, request, document) = set_of("taken_order");
    let out: PathBuf = out_of(&dir);
    let plan: ExportPlan = export_plan(&document, &request, &out);
    assert_eq!(
        taken(&plan),
        Vec::<String>::new(),
        "a directory nothing is in has no name taken"
    );

    fs::create_dir_all(&out).unwrap();
    fs::write(out.join("set.order.tsv"), "a table already on disk\n").unwrap();
    fs::write(out.join("sid-1_ins-14.fa"), "a construct already on disk\n").unwrap();
    fs::write(out.join("set.result.json"), "a result already on disk\n").unwrap();
    assert_eq!(
        taken(&plan),
        vec![
            "sid-1_ins-14.fa".to_string(),
            "set.result.json".to_string(),
            "set.order.tsv".to_string(),
        ],
        "constructs, then the result, then the table, in the plan's order: {:?}",
        taken(&plan)
    );

    // A table the plan refused is not a file the export writes, so its name is
    // not a name the export asks about.
    let tet: PathBuf = workspace("taken_no_table");
    fs::copy(
        "../../tests/fixtures/genbank/raw/J01749.1.gb",
        tet.join("J01749.1.gb"),
    )
    .unwrap();
    let tet_request: PathBuf = tet.join("tet.json");
    fs::write(
        &tet_request,
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
    let tet_document: ResultDocument = run(&tet_request).document;
    let tet_out: PathBuf = out_of(&tet);
    fs::create_dir_all(&tet_out).unwrap();
    fs::write(tet_out.join("tet.order.tsv"), "a table already on disk\n").unwrap();
    let tet_plan: ExportPlan = export_plan(&tet_document, &tet_request, &tet_out);
    assert_eq!(tet_plan.order, Err(OrderRefusal::NothingDesigned));
    assert_eq!(
        taken(&tet_plan),
        Vec::<String>::new(),
        "a plan with no table to write does not ask about its order file"
    );
}
