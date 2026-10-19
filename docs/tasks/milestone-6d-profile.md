# Milestone 6d: the design profile

Status: **Ready for the worker, 2026-10-19.**
Planner: Claude. Parent: `docs/tasks/milestone-6-nl.md` §5.4 (and its
implementation note), M6-6, M6-11, §6 (6d).

## 1. What and why

Prose rarely states a primer-design policy, a buffer or a codon, and D028
forbids luria defaulting them. The user keeps those values in a small file
of their own, the *design profile* (M6-6). `luria q5 interpret` reads it
and fills in only what the text leaves out (6e).

This contract reads a profile. Its `design` block has exactly the shape of
a request's `design`, so it is read by the request parser's own
`read_design`, unchanged: one set of rules, one set of error messages.

## 2. Requirements this contract applies

- **Plan §5.4:** the schema is `luria.q5-design-profile/1`. Its members are
  `schema` (required), and `design`, `codon` and `numbering`, each
  optional. `codon`'s only value is `"default"`; `numbering`'s only value
  is `"from-cds-start"` (M6-11). An absent member is `None`, never a
  default: 6e asks about it.
- **The request parser's conventions (5e1), applied unchanged:** a JSON
  syntax error, a duplicate key or trailing text is refused at `$` with
  `invalid JSON: …`; an unknown member is refused at its own path; an
  optional member that is present must hold a value of its type, so
  `null` is refused, as for a request's `scope`; the first problem found
  is the one reported.
- **D028:** nothing has a default. There is no built-in profile.
- Every refusal has the code `profile_invalid` (a new `RequestErrorCode`),
  so a profile error is never mistaken for a request error.

## 3. Changes

### 3.1 `crates/cli/src/lib.rs`

After `REQUEST_SCHEMA`, add:

```rust
/// The design profile schema `luria q5 interpret` reads (milestone 6
/// plan §5.4).
pub const PROFILE_SCHEMA: &str = "luria.q5-design-profile/1";
```

### 3.2 `crates/cli/src/request.rs`

1. `RequestErrorCode` gains a last variant:

   ```rust
   /// A design profile is not valid JSON or breaks its schema (milestone 6,
   /// 6d). Produced only by `parse_profile`.
   ProfileInvalid,
   ```

   `code()` maps it to `"profile_invalid"`, and the doc comment listing the
   codes above `code()` gains `` `profile_invalid` ``.

2. Two new public types, after `BufferSpec`:

   ```rust
   /// The numbering origin a design profile may state (M6-11). From the
   /// start of the CDS is the only one luria supports.
   #[derive(Clone, Debug, PartialEq)]
   pub enum NumberingSpec {
       FromCdsStart,
   }

   /// A design profile (milestone 6 plan §5.4): values a user keeps for the
   /// requests whose text does not state them. Each is optional; an absent
   /// one is asked about, never defaulted (D028).
   #[derive(Clone, Debug, PartialEq)]
   pub struct ProfileDocument {
       pub design: Option<DesignSpec>,
       /// `CodonSpec::Default` when the profile says `"codon": "default"`.
       pub codon: Option<CodonSpec>,
       pub numbering: Option<NumberingSpec>,
   }
   ```

3. A new public function, after `parse_request`:

   ```rust
   /// Read a design profile from JSON text, refusing anything its schema
   /// does not allow and naming the JSON path of the first problem. Its
   /// `design` is read by `read_design`, as a request's is. Every refusal
   /// is `profile_invalid`.
   pub fn parse_profile(text: &str) -> Result<ProfileDocument, RequestError>;
   ```

   Built from the existing private helpers only, in this order:
   1. `crate::json::parse_json(text)`; on failure, `$` and
      `format!("invalid JSON: {}", problem)`, as `parse_request` does;
   2. `want_object` at `$`;
   3. `check_unknown` with `schema`, `design`, `codon`, `numbering`;
   4. `field_fixed` for `schema`, `crate::PROFILE_SCHEMA`;
   5. `design`: if `has_field`, `field_object` then `read_design` on the
      nested members, else `None`;
   6. `codon`: if `has_field`, `want_fixed` at `$.codon` with `"default"`,
      giving `Some(CodonSpec::Default)`, else `None`;
   7. `numbering`: if `has_field`, `want_fixed` at `$.numbering` with
      `"from-cds-start"`, giving `Some(NumberingSpec::FromCdsStart)`, else
      `None`.

   Every error from steps 1 to 7 is returned with its `path` and `message`
   unchanged and its `code` set to `RequestErrorCode::ProfileInvalid`.

Change nothing else in `request.rs`: not `read_design`, not any helper, not
`parse_request`, not any existing test. `parse_request`'s behaviour is
unchanged; its existing tests prove it.

## 4. Tests (planner-written; add verbatim)

New file `crates/cli/tests/profile.rs`:

```rust
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

    let from_request: DesignSpec = parse_request(&request_text).expect("the request parses").design;
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
        ("buffer", serde_json::json!({ "monovalentMillimolar": 50, "primerNanomolar": 250 })),
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
        assert_eq!(in_profile.code, RequestErrorCode::ProfileInvalid, "{}", fault.0);
        assert_eq!(in_profile.path, in_request.path, "{} = {}", fault.0, fault.1);
        assert_eq!(in_profile.message, in_request.message, "{} = {}", fault.0, fault.1);
    }

    let mut with_values: Value = design.clone();
    with_values["buffer"] = values;
    assert_eq!(
        profile(&profile_of(&with_values)).design.map(|found| found.buffer),
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
        ("{}".to_string(), "$", Some("missing required field 'schema'")),
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
        (format!("{{{}, \"numbering\": null}}", schema), "$.numbering", None),
        (
            format!("{{{}, \"design\": \"q5\"}}", schema),
            "$.design",
            Some("expected an object"),
        ),
        (format!("{{{}, \"design\": null}}", schema), "$.design", None),
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
```

`cargo fmt` may reflow these; do not change content. If a test fails,
stop and report what `parse_profile` returned. Do not change the test or a
fixture.

### 4.1 Proving the rules are load-bearing (AGENTS.md)

Disable each once, confirm the named test fails, restore, report both:

| Rule, disabled | Test that must fail |
| --- | --- |
| the code set to `ProfileInvalid` (leave the helpers' code) | `each_refusal_is_profile_invalid_at_its_path` |
| `check_unknown` at the root | `each_refusal_is_profile_invalid_at_its_path` |
| `codon` fixed to `"default"` (accept any string as `Default`) | `each_refusal_is_profile_invalid_at_its_path` |
| `design` read by `read_design` (always `None`) | `the_corpus_profiles_read_as_written` |

## 5. Do not

- Change `read_design`, any existing helper, `parse_request`, any
  existing test, or anything in `request.rs` beyond §3.2.
- Make any existing private item public. Only `parse_profile`,
  `ProfileDocument`, `NumberingSpec` and the new variant are new public
  items.
- Add a default value for any profile member, or a built-in profile.
- Read a file: `parse_profile` takes text. Reading the profile file is 6h.
- Add a dependency. Change `luria-core`, `luria-nl`, `tests/fixtures/` or
  `docs/`.

## 6. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 7. Handoff

Report the new and changed files, the exact commands and their results,
the §4.1 results, and anything that did not match this contract. Quote
command output or the file for every claim; say plainly what you did not
check.
