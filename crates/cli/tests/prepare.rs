//! `prepare`: a request file's path to everything the workflow needs, with
//! each failure at the JSON path it came from. The templates are the
//! committed fixtures, copied next to each test's request file.

use luria_cli::prepare::{
    prepare, Operation, PrepareFailure, Prepared, RequestFailure, TemplateSource,
};
use luria_cli::request::RequestErrorCode;
use luria_core::identity::sha256_hex;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

const SID1_DIGEST: &str = "54704740a9f04b79d17147ef95788ad9f0f11e5263843a7ed490924d43b55e3a";
const PBR322_DIGEST: &str = "536042a7485bdabdf49c4d21a306213a098d77d19bdba57a1c6adc76ce2b8372";

/// A fresh directory for one test, holding copies of both templates.
fn workspace(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("prepare-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::copy(
        "../../tests/fixtures/example-construct/input/sid-1_cDNA.fa",
        dir.join("sid-1_cDNA.fa"),
    )
    .unwrap();
    fs::copy(
        "../../tests/fixtures/genbank/raw/J01749.1.gb",
        dir.join("J01749.1.gb"),
    )
    .unwrap();
    dir
}

fn write(dir: &Path, request: &Value) -> PathBuf {
    let path = dir.join("request.json");
    fs::write(&path, request.to_string()).unwrap();
    path
}

fn sid1() -> Value {
    json!({
        "schema": "luria.q5-request/1",
        "coordinates": "zero-based-half-open",
        "template": {
            "id": "sid-1",
            "fasta": "sid-1_cDNA.fa",
            "topology": "linear",
            "cds": [{
                "id": "sid-1",
                "intervals": [[0, 3153]],
                "strand": "+",
                "completeness": "complete",
                "geneticCode": "standard",
                "numbering": "from-cds-start"
            }]
        },
        "scope": "sid-1",
        "edit": {
            "name": "D404N",
            "residue": { "cds": "sid-1", "number": 404, "from": "D", "to": "N", "codon": "AAC" }
        },
        "design": {
            "policy": "luriaNative",
            "style": "q5_polymerase",
            "minAnnealingOverlap": 15,
            "placement": "inline",
            "buffer": "q5_reaction"
        }
    })
}

fn pbr322() -> Value {
    json!({
        "schema": "luria.q5-request/1",
        "coordinates": "zero-based-half-open",
        "template": { "id": "pbr322", "genbank": "J01749.1.gb", "cds": ["tet", "bla"] },
        "variants": {
            "composition": "separate_variants",
            "members": [
                { "name": "s1", "substitution": { "interval": [99, 100], "from": "T", "to": "C" } },
                { "name": "d1", "deletion": { "interval": [99, 102], "from": "TGC" } },
                { "name": "i1", "insertion": { "point": 100, "before": "T", "after": "G", "insert": "GGG" } },
                { "name": "r1", "replacement": { "interval": [99, 102], "from": "TGC", "to": "TT" } },
                { "name": "k2r", "residue": { "cds": "tet", "number": 2, "from": "K", "to": "R", "policy": "default" } }
            ]
        },
        "design": {
            "policy": "luriaNative",
            "style": "q5_polymerase",
            "minAnnealingOverlap": 15,
            "placement": "inline",
            "buffer": { "monovalentMillimolar": 50, "primerNanomolar": 200, "dmsoPercent": 0 }
        }
    })
}

fn set(value: &mut Value, pointer: &str, new: Value) {
    *value.pointer_mut(pointer).unwrap() = new;
}

fn prepared(test: &str, request: &Value) -> Prepared {
    let dir = workspace(test);
    prepare(&write(&dir, request)).unwrap_or_else(|failure| panic!("{}: {:?}", test, failure))
}

fn failure(test: &str, request: &Value) -> RequestFailure {
    let dir = workspace(test);
    let path = write(&dir, request);
    match prepare(&path) {
        Err(PrepareFailure {
            failure,
            request_sha256,
            document,
        }) => {
            // Once the file has been read its digest is known, and once it
            // parsed, so is the document.
            assert_eq!(request_sha256, Some(sha256_hex(&fs::read(&path).unwrap())));
            if let RequestFailure::Domain { .. } = failure {
                assert!(
                    document.is_some(),
                    "{}: a domain failure keeps the document",
                    test
                );
            }
            failure
        }
        Ok(_) => panic!("{}: prepared", test),
    }
}

fn request_error(failure: RequestFailure) -> (RequestErrorCode, String, String) {
    match failure {
        RequestFailure::Request(error) => (error.code, error.path, error.message),
        other => panic!("expected a request error, got {:?}", other),
    }
}

fn domain_error(failure: RequestFailure) -> (String, String) {
    match failure {
        RequestFailure::Domain { path, error } => (path, error.code().to_string()),
        other => panic!("expected a domain error, got {:?}", other),
    }
}

fn pair(path: &str, code: &str) -> (String, String) {
    (path.to_string(), code.to_string())
}

#[test]
fn the_sid1_request_prepares_against_the_fasta_template() {
    let dir = workspace("sid1");
    let path = write(&dir, &sid1());
    let ready = prepare(&path).unwrap();
    assert_eq!(ready.request_sha256, sha256_hex(&fs::read(&path).unwrap()));
    assert_eq!(ready.template.length(), 3153);
    assert_eq!(ready.template.content_hash().hex_digest(), SID1_DIGEST);
    assert_eq!(ready.template.name().value(), "sid-1");
    assert_eq!(
        ready.source,
        TemplateSource::Fasta {
            path: "sid-1_cDNA.fa".to_string(),
            header: "3xFLAG-mVenus-sid-1 cDNA".to_string()
        }
    );
    assert_eq!(ready.annotations.len(), 1);
    assert_eq!(ready.annotations[0].id().value(), "sid-1");
    assert_eq!(ready.annotations[0].total_length(), 3153);
    match ready.operation {
        Operation::Single { request, name } => {
            assert_eq!(name.value(), "D404N");
            assert_eq!(request.template(), ready.template);
        }
        other => panic!("expected a single edit, got {:?}", other),
    }
    assert_eq!(ready.design.min_annealing_overlap(), 15);
}

#[test]
fn a_matching_digest_is_accepted_and_a_wrong_one_refused() {
    // `pointer_mut` cannot add a member, so the digest is inserted.
    let mut right = sid1();
    right
        .pointer_mut("/template")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("sha256".to_string(), json!(SID1_DIGEST));
    prepared("digest-right", &right);

    let mut wrong = sid1();
    wrong
        .pointer_mut("/template")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .insert("sha256".to_string(), json!(PBR322_DIGEST));
    assert_eq!(
        request_error(failure("digest-wrong", &wrong)),
        (
            RequestErrorCode::TemplateDigestMismatch,
            "$.template.sha256".to_string(),
            format!(
                "the template sequence's SHA-256 is {}, not {}",
                SID1_DIGEST, PBR322_DIGEST
            )
        )
    );
}

#[test]
fn the_pbr322_request_prepares_against_the_genbank_template() {
    let ready = prepared("pbr322", &pbr322());
    assert_eq!(ready.template.content_hash().hex_digest(), PBR322_DIGEST);
    assert_eq!(ready.template.name().value(), "pbr322");
    let ids: Vec<String> = ready
        .annotations
        .iter()
        .map(|cds| cds.id().value().to_string())
        .collect();
    assert_eq!(ids, vec!["tet".to_string(), "bla".to_string()]);
    match ready.source {
        TemplateSource::GenBank {
            path,
            locus_name,
            accession,
            version,
            import,
        } => {
            assert_eq!(path, "J01749.1.gb");
            assert_eq!(locus_name, "SYNPBR322");
            assert_eq!(accession, Some("J01749".to_string()));
            assert_eq!(version, Some("J01749.1".to_string()));
            assert_eq!(import.unused().len(), 48);
        }
        other => panic!("expected GenBank, got {:?}", other),
    }
    match ready.operation {
        Operation::Set(set) => assert_eq!(set.variants().len(), 5),
        other => panic!("expected a set, got {:?}", other),
    }
}

#[test]
fn a_degenerate_replacement_expands_without_the_wild_type() {
    let mut request = sid1();
    request.as_object_mut().unwrap().remove("edit");
    request.as_object_mut().unwrap().insert(
        "degenerate".to_string(),
        json!({ "name": "lib", "interval": [1209, 1212], "from": "GAT", "to": "NNK" }),
    );
    match prepared("degenerate", &request).operation {
        Operation::Degenerate(expansion) => {
            assert_eq!(expansion.set().variants().len(), 31);
            assert_eq!(expansion.warnings().len(), 1);
        }
        other => panic!("expected an expansion, got {:?}", other),
    }
}

#[test]
fn file_problems_are_the_clis_own_errors() {
    let dir = workspace("missing-request");
    match prepare(&dir.join("absent.json"))
        .map_err(|failed| (failed.failure, failed.request_sha256))
    {
        Err((RequestFailure::Request(error), None)) => {
            assert_eq!(error.code, RequestErrorCode::Unreadable);
            assert_eq!(error.path, "$");
            assert!(
                error.message.starts_with("cannot read the request file: "),
                "{}",
                error.message
            );
        }
        other => panic!("expected request_unreadable, got {:?}", other),
    }

    let latin = workspace("latin1");
    fs::write(latin.join("request.json"), [0x7b, 0xff, 0x7d]).unwrap();
    assert_eq!(
        request_error(prepare(&latin.join("request.json")).unwrap_err().failure),
        (
            RequestErrorCode::Unreadable,
            "$".to_string(),
            "the request file is not UTF-8".to_string()
        )
    );

    let mut absent = sid1();
    set(&mut absent, "/template/fasta", json!("absent.fa"));
    let (code, path, message) = request_error(failure("absent-template", &absent));
    assert_eq!(code, RequestErrorCode::TemplateUnreadable);
    assert_eq!(path, "$.template.fasta");
    assert!(
        message.starts_with("cannot read the template file 'absent.fa': "),
        "{}",
        message
    );

    let mut absolute = sid1();
    set(&mut absolute, "/template/fasta", json!("/etc/hostname"));
    assert_eq!(
        request_error(failure("absolute-template", &absolute)),
        (
            RequestErrorCode::Invalid,
            "$.template.fasta".to_string(),
            "expected a path relative to the request file".to_string()
        )
    );

    let mut bad_json = sid1();
    set(&mut bad_json, "/coordinates", json!("one-based"));
    assert_eq!(
        request_error(failure("bad-json", &bad_json)).0,
        RequestErrorCode::Invalid
    );
}

#[test]
fn a_template_that_is_not_one_record_fails_at_its_path() {
    let dir = workspace("two-records");
    fs::write(dir.join("two.fa"), ">a\nACGT\n>b\nACGT\n").unwrap();
    let mut request = sid1();
    set(&mut request, "/template/fasta", json!("two.fa"));
    assert_eq!(
        domain_error(prepare(&write(&dir, &request)).unwrap_err().failure),
        pair("$.template.fasta", "malformed_fasta")
    );
}

#[test]
fn declared_cds_problems_fail_where_they_are_declared() {
    let mut twice = sid1();
    let first = twice.pointer("/template/cds/0").unwrap().clone();
    twice
        .pointer_mut("/template/cds")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .push(first);
    assert_eq!(
        request_error(failure("cds-twice", &twice)),
        (
            RequestErrorCode::Invalid,
            "$.template.cds[1].id".to_string(),
            "CDS id 'sid-1' is declared twice".to_string()
        )
    );

    let mut negative = sid1();
    set(
        &mut negative,
        "/template/cds/0/intervals/0",
        json!([-3, 3153]),
    );
    assert_eq!(
        domain_error(failure("cds-negative", &negative)),
        pair("$.template.cds[0].intervals[0]", "negative_coordinate")
    );

    let mut code = sid1();
    set(
        &mut code,
        "/template/cds/0/geneticCode",
        json!("vertebrate_mitochondrial"),
    );
    assert_eq!(
        domain_error(failure("cds-code", &code)),
        pair("$.template.cds[0]", "unsupported_genetic_code")
    );

    let mut not_found = pbr322();
    set(&mut not_found, "/template/cds/1", json!("rop"));
    assert_eq!(
        domain_error(failure("cds-not-found", &not_found)),
        pair("$.template.cds", "unusable_genbank_cds")
    );
}

#[test]
fn edit_and_design_problems_fail_where_they_are_written() {
    let mut letter = sid1();
    set(&mut letter, "/edit/residue/from", json!("Z"));
    assert_eq!(
        domain_error(failure("edit-letter", &letter)),
        pair("$.edit.residue", "invalid_mutation_request")
    );

    let mut name = sid1();
    set(&mut name, "/edit/name", json!(""));
    assert_eq!(
        domain_error(failure("edit-name", &name)),
        pair("$.edit.name", "empty_identifier")
    );

    let mut scope = sid1();
    set(&mut scope, "/scope", json!("a b"));
    assert_eq!(
        domain_error(failure("scope", &scope)),
        pair("$.scope", "invalid_identifier_characters")
    );

    let mut member = pbr322();
    set(
        &mut member,
        "/variants/members/2/insertion/point",
        json!(-1),
    );
    assert_eq!(
        domain_error(failure("member-point", &member)),
        pair("$.variants.members[2].insertion", "negative_coordinate")
    );

    let mut combined = pbr322();
    set(
        &mut combined,
        "/variants/composition",
        json!("combined_construct"),
    );
    assert_eq!(
        domain_error(failure("combined", &combined)),
        pair("$.variants", "unsupported_composition")
    );

    let mut empty = pbr322();
    set(&mut empty, "/variants/members", json!([]));
    assert_eq!(
        domain_error(failure("empty-set", &empty)),
        pair("$.variants", "invalid_mutation_request")
    );

    let mut salt = pbr322();
    set(&mut salt, "/design/buffer/monovalentMillimolar", json!(0));
    assert_eq!(
        domain_error(failure("salt", &salt)),
        pair("$.design.buffer", "invalid_buffer_model")
    );

    let mut overlap = sid1();
    set(&mut overlap, "/design/minAnnealingOverlap", json!(0));
    assert_eq!(
        domain_error(failure("overlap", &overlap)),
        pair("$.design", "invalid_primer_design_request")
    );
}

// A message names the path as the request gave it, never the directory the
// test happens to run in.
#[test]
fn no_message_names_the_test_directory() {
    let mut absent = sid1();
    set(&mut absent, "/template/fasta", json!("absent.fa"));
    let (_, _, message) = request_error(failure("no-absolute", &absent));
    assert!(
        !message.contains(env!("CARGO_TARGET_TMPDIR")),
        "{}",
        message
    );
}
