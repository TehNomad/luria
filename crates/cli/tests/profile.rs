//! The design profile of milestone 6 (contract 6d): the corpus profiles,
//! the design block read exactly as a request's, and every refusal.

use luria_cli::request::parse_profile;
use luria_cli::request::parse_request;
use luria_cli::request::BufferSpec;
use luria_cli::request::CodonSpec;
use luria_cli::request::DesignSpec;
use luria_cli::request::NumberingSpec;
use luria_cli::request::PlacementSpec;
use luria_cli::request::PolicySpec;
use luria_cli::request::ProfileDocument;
use luria_cli::request::RequestError;
use luria_cli::request::RequestErrorCode;
use luria_cli::request::StyleSpec;
use serde_json::Value;

const NL: &str = "../../tests/fixtures/nl/";

fn read(file: &str) -> String {
    let path: String = format!("{}{}", NL, file);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("cannot read {}: {}", path, error))
}

fn profile(text: &str) -> ProfileDocument {
    parse_profile(text).unwrap_or_else(|error| panic!("{:?} was refused: {:?}", text, error))
}

fn refusal(text: &str) -> RequestError {
    match parse_profile(text) {
        Err(error) => error,
        Ok(found) => panic!("{:?} parsed as {:?}", text, found),
    }
}

/// The design every corpus profile and expected request states.
fn corpus_design() -> DesignSpec {
    DesignSpec {
        policy: PolicySpec::LuriaNative,
        style: StyleSpec::Q5Polymerase,
        min_annealing_overlap: 15,
        placement: PlacementSpec::Inline,
        buffer: BufferSpec::Q5Reaction,
    }
}

#[test]
fn the_corpus_profiles_read_as_written() {
    assert_eq!(
        profile(&read("profile/full.json")),
        ProfileDocument {
            design: Some(corpus_design()),
            codon: Some(CodonSpec::Default),
            numbering: Some(NumberingSpec::FromCdsStart),
        }
    );
    assert_eq!(
        profile(&read("profile/no-codon.json")),
        ProfileDocument {
            design: Some(corpus_design()),
            codon: None,
            numbering: Some(NumberingSpec::FromCdsStart),
        }
    );
    assert_eq!(
        profile(&read("profile/no-numbering.json")),
        ProfileDocument {
            design: Some(corpus_design()),
            codon: Some(CodonSpec::Default),
            numbering: None,
        }
    );
}

#[test]
fn every_member_but_the_schema_is_optional() {
    assert_eq!(
        profile(r#"{"schema": "luria.q5-design-profile/1"}"#),
        ProfileDocument {
            design: None,
            codon: None,
            numbering: None,
        }
    );
}

// The same design object, in a request and in a profile, reads the same,
// and is refused with the same path and message. Only the code differs.
#[test]
fn a_profile_design_reads_exactly_as_a_request_design() {
    let request_text: String = read("request/edit-a42g.json");
    let request: Value = serde_json::from_str(&request_text).expect("the corpus request is JSON");
    let design: Value = request["design"].clone();
    let profile_of = |design: &Value| -> String {
        serde_json::json!({ "schema": "luria.q5-design-profile/1", "design": design }).to_string()
    };
    let request_of = |design: &Value| -> String {
        let mut whole: Value = request.clone();
        whole["design"] = design.clone();
        whole.to_string()
    };

    let from_request: DesignSpec = parse_request(&request_text)
        .expect("the request parses")
        .design;
    assert_eq!(profile(&profile_of(&design)).design, Some(from_request));

    let values: Value = serde_json::json!({
        "monovalentMillimolar": 50, "primerNanomolar": 250, "dmsoPercent": 3
    });
    let faults: [(&str, Value); 7] = [
        ("minAnnealingOverlap", serde_json::json!(1.5)),
        ("policy", serde_json::json!("neb")),
        ("style", serde_json::json!(null)),
        ("placement", serde_json::json!("middle")),
        ("buffer", serde_json::json!("pcr")),
        (
            "buffer",
            serde_json::json!({ "monovalentMillimolar": 50, "primerNanomolar": 250 }),
        ),
        ("extra", serde_json::json!(true)),
    ];
    for fault in faults {
        let mut wrong: Value = design.clone();
        wrong[fault.0] = fault.1.clone();
        let in_request: RequestError = match parse_request(&request_of(&wrong)) {
            Err(error) => error,
            Ok(_) => panic!("the request accepted {} = {}", fault.0, fault.1),
        };
        let in_profile: RequestError = refusal(&profile_of(&wrong));
        assert_eq!(in_request.code, RequestErrorCode::Invalid, "{}", fault.0);
        assert_eq!(
            in_profile.code,
            RequestErrorCode::ProfileInvalid,
            "{}",
            fault.0
        );
        assert_eq!(
            in_profile.path, in_request.path,
            "{} = {}",
            fault.0, fault.1
        );
        assert_eq!(
            in_profile.message, in_request.message,
            "{} = {}",
            fault.0, fault.1
        );
    }

    let mut with_values: Value = design.clone();
    with_values["buffer"] = values;
    assert_eq!(
        profile(&profile_of(&with_values))
            .design
            .map(|found| found.buffer),
        Some(BufferSpec::Values {
            monovalent_millimolar: 50.0,
            primer_nanomolar: 250.0,
            dmso_percent: 3.0,
        })
    );
}

#[test]
fn each_refusal_is_profile_invalid_at_its_path() {
    let schema: &str = r#""schema": "luria.q5-design-profile/1""#;
    let cases: Vec<(String, &str, Option<&str>)> = vec![
        ("{".to_string(), "$", None),
        (format!("{{{}, {}}}", schema, schema), "$", None),
        (format!("{{{}}} {{}}", schema), "$", None),
        ("[]".to_string(), "$", Some("expected an object")),
        (
            "{}".to_string(),
            "$",
            Some("missing required field 'schema'"),
        ),
        (
            r#"{"schema": "luria.q5-request/1"}"#.to_string(),
            "$.schema",
            Some("expected \"luria.q5-design-profile/1\""),
        ),
        (
            format!("{{{}, \"buffer\": \"q5_reaction\"}}", schema),
            "$.buffer",
            Some("unknown field 'buffer'"),
        ),
        (
            format!("{{{}, \"codon\": \"GGC\"}}", schema),
            "$.codon",
            Some("expected \"default\""),
        ),
        (format!("{{{}, \"codon\": null}}", schema), "$.codon", None),
        (
            format!("{{{}, \"numbering\": \"from-protein-start\"}}", schema),
            "$.numbering",
            Some("expected \"from-cds-start\""),
        ),
        (
            format!("{{{}, \"numbering\": null}}", schema),
            "$.numbering",
            None,
        ),
        (
            format!("{{{}, \"design\": \"q5\"}}", schema),
            "$.design",
            Some("expected an object"),
        ),
        (
            format!("{{{}, \"design\": null}}", schema),
            "$.design",
            None,
        ),
    ];
    for case in cases {
        let error: RequestError = refusal(&case.0);
        assert_eq!(error.code, RequestErrorCode::ProfileInvalid, "{:?}", case.0);
        assert_eq!(error.path, case.1, "{:?}", case.0);
        match case.2 {
            Some(message) => assert_eq!(error.message, message, "{:?}", case.0),
            None => assert!(!error.message.is_empty(), "{:?}", case.0),
        }
    }
    // Syntax errors are reported as the request parser reports them.
    assert!(refusal("{").message.starts_with("invalid JSON: "));
}

#[test]
fn the_profile_code_is_pinned() {
    assert_eq!(RequestErrorCode::ProfileInvalid.code(), "profile_invalid");
    assert_eq!(luria_cli::PROFILE_SCHEMA, "luria.q5-design-profile/1");
}
