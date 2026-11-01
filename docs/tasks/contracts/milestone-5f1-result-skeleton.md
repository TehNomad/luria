# Milestone 5f1: the result document's skeleton, provenance and failures

Status: **Implemented 2026-10-17** by the local worker, across two runs (the
first ended in a local inference error with clippy unrun); accepted on
review.
Planner: Claude. Parent: [docs/tasks/contracts/milestone-5-cli.md](/tasks/contracts/milestone-5-cli.md) §3.5 and §5 (5f,
split into 5f1 to 5f4).

## 1. What and why

`luria q5 design` prints one JSON document for success, clarification and
failure alike (plan §3.5). This contract builds that document's fixed shape,
its provenance, the echoed request, and complete **failed** documents. The
later contracts fill the rest of the same shape:
- 5f2: the target, mutation results, edit and expansion warnings, and the
  clarification document;
- 5f3: primer designs and design failures;
- 5f4: each error's `details`.

## 2. Requirements this contract applies

- **Plan §3.5:**
  - one document with `schema`, `state` (`verified`,
    `needs_clarification` or `failed`) and `experimentalValidation`, always
    `"not_performed"` (D020);
  - `provenance` (D021), the normalized `request` (D004, D016), `target`,
    clarification fields, `results` and `error`;
  - deterministic output: no timestamps, hostnames or absolute paths, keys
    in a fixed order, floats in shortest round-trip form;
  - intervals as `[start, end]` pairs.
- **5d:** every document is serde_json's pretty form plus one `\n`, built
  from `#[derive(Serialize)]` structs, never `Value` or `json!`. Field order
  is struct order.
- **D041 (G6):** the result gives the buffer's numbers, and the preset's
  name when one was used.
- **Decided here:**
  - **Fixed shape.** Every top-level key is always present: `null` or `[]`
    when it doesn't apply. A consumer never tests for a key's existence.
  - **The echoed request is itself a valid request.** `request` uses the
    request schema's own keys and order, omits absent optional fields as
    the input would, and states the buffer as the input did. So
    `parse_request` on the echo gives back the same `RequestDocument`. The
    preset's resolved numbers go in `provenance.buffer` instead.
  - **Category names** are `malformed_input`, `biological_validation` and
    `unsupported`.

## 3. The module: `crates/cli/src/result.rs`

Add `pub mod result;` to `lib.rs`, keeping the list alphabetical. Every view
type derives `Clone, Debug, PartialEq, Serialize` and has
`#[serde(rename_all = "camelCase")]` unless stated otherwise. `Option`
fields serialize as `null` (no `skip_serializing_if`) except in the request
echo (§3.3).

### 3.1 The document

```rust
pub struct ResultDocument {
    pub schema: &'static str,                 // RESULT_SCHEMA
    pub state: &'static str,                  // "verified" | "needs_clarification" | "failed"
    pub experimental_validation: &'static str, // "not_performed"
    pub provenance: Provenance,
    pub request: Option<RequestEcho>,
    pub target: Option<CdsView>,              // 5f2; None in 5f1
    pub clarifications: Vec<String>,          // 5f2
    pub candidates: Vec<String>,              // 5f2
    pub warnings: Vec<OperationWarning>,      // 5f2
    pub results: Vec<MemberView>,             // 5f2
    pub error: Option<ErrorView>,
}

/// The document as luria prints it: pretty JSON plus one `\n`.
pub fn render(document: &ResultDocument) -> String;
```

`CdsView`, `OperationWarning` and `MemberView` are defined in 5f2. For now,
declare each as an empty struct, `pub struct CdsView {}` and so on, with the
same derives. 5f2 fills them in. Only empty vectors and `None` are used in
5f1.

### 3.2 Provenance

```rust
pub struct Provenance {
    pub package: PackageView,              // { name: "luria-cli", version: CARGO_PKG_VERSION }
    pub algorithms: Vec<Algorithm>,        // crate::version::algorithms()
    pub request_sha256: Option<String>,
    pub operation: Option<&'static str>,   // "edit" | "variants" | "degenerate"
    pub template: Option<TemplateView>,
    pub buffer: Option<BufferView>,
}

pub struct PackageView { pub name: &'static str, pub version: &'static str }

pub struct TemplateView {
    pub id: String,
    pub sha256: String,      // the sequence's content digest
    pub length: u64,
    pub topology: String,    // SequenceTopology::name()
    pub source: SourceView,
    pub genbank: Option<GenBankView>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "format", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum SourceView {
    Fasta { path: String, header: String },
    Genbank { path: String, locus_name: String, accession: Option<String>, version: Option<String> },
}

pub struct GenBankView {
    pub imported_cds: Vec<ImportedCdsView>,
    pub unused_features: Vec<UnusedFeatureView>,
}

pub struct ImportedCdsView {
    pub id: String,
    pub id_qualifier: &'static str,      // CdsIdQualifier::name()
    pub feature_index: usize,
    pub line: u64,
    pub transl_table: Option<String>,
}

pub struct UnusedFeatureView {
    pub index: usize,
    pub key: String,
    pub line: u64,
    pub id: Option<String>,
    pub refusal: Option<RefusalView>,
}

pub struct RefusalView { pub code: &'static str, pub rule: Option<&'static str>, pub message: String }

pub struct BufferView {
    pub preset: Option<&'static str>,    // "q5_reaction" or None
    pub monovalent_millimolar: f64,
    pub primer_nanomolar: f64,
    pub dmso_percent: f64,
}

pub fn provenance(
    request_sha256: Option<String>,
    document: Option<&RequestDocument>,
    prepared: Option<&Prepared>,
) -> Provenance;
```

Rules:
- `operation` and `buffer` come from `document` (or from
  `prepared.document` when `prepared` is given). For `BufferSpec::Q5Reaction`
  the numbers are `BufferModel::q5_reaction()`'s accessors and the preset is
  `"q5_reaction"`; for `Values` they are the given numbers and the preset is
  `None`.
- `template` comes from `prepared` only. `genbank` is `Some` exactly for a
  GenBank source, built from its `GenBankImport`, in its order.
- A refusal's `code` is `DomainError::code()`, its `rule` is `rule_code`
  (§3.4), and its `message` is `message()`.

### 3.3 The request echo

```rust
pub fn request_echo(document: &RequestDocument) -> RequestEcho;
```

`RequestEcho` and its parts are serde structs and enums, written so that
the echo's JSON uses exactly the request schema of 5e1 §3.4: the same keys,
the same order (schema order), the fixed values (`schema`, `coordinates`),
and the same value spellings (`"+"`, `"linear"`, `"separate_variants"`,
`"luriaNative"`, `"q5_polymerase"`, `"inline"`, `"confine_to_tail"`,
`"q5_reaction"`). Absent optional fields (`sha256`, `scope`, a residue's
`cds`, and whichever of `codon` and `policy` wasn't given) are omitted with
`skip_serializing_if = "Option::is_none"`. Exactly one of `edit`,
`variants` and `degenerate` appears. A member is `name` followed by its one
kind key. A buffer is the string `"q5_reaction"` or the object of three
numbers. `serde(untagged)`, `serde(flatten)` and externally tagged enums
with `rename_all = "snake_case"` are all fine for this.

### 3.4 Errors

```rust
pub struct ErrorView {
    pub code: &'static str,
    pub rule: Option<&'static str>,
    pub category: &'static str,
    pub message: String,
    pub path: Option<String>,
}

pub fn category_name(category: ErrorCategory) -> &'static str;
/// The rule's code for the six variants that carry one
/// (InvalidMutationRequest, InvalidPrimerCandidate, MalformedFasta,
/// UnusableGenBankLocation, MalformedGenBank, UnusableGenBankCds); None
/// for every other variant. Exhaustive match, no `_` arm.
pub fn rule_code(error: &DomainError) -> Option<&'static str>;
pub fn domain_error_view(error: &DomainError, path: Option<String>) -> ErrorView;
/// A CLI error is `malformed_input` with no rule; a domain failure is
/// `domain_error_view` at its path.
pub fn failure_view(failure: &RequestFailure) -> ErrorView;
```

### 3.5 Failed documents

```rust
/// A request that could not be prepared.
pub fn failed_document(failure: &PrepareFailure) -> ResultDocument;
/// A prepared request whose run failed (5g calls this).
pub fn failed_after(prepared: &Prepared, error: ErrorView) -> ResultDocument;
```

Both give `state: "failed"`. `failed_document` uses
`provenance(failure.request_sha256, failure.document, None)` and
`request` is the echo of `failure.document` when it is `Some`. `failed_after` uses
`provenance(Some(prepared.request_sha256), Some(&prepared.document),
Some(prepared))` and echoes `prepared.document`. Everything else is `None`
or empty.

## 4. Tests

### 4.1 Unit tests in `result.rs` (planner-written; add verbatim)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::prepare::PrepareFailure;
    use crate::request::{parse_request, RequestError, RequestErrorCode};
    use luria_core::error::{InvalidMutationRequestProblem, MutationRequestRule};

    const SID1_REQUEST: &str = r#"{"schema":"luria.q5-request/1","coordinates":"zero-based-half-open","template":{"id":"sid-1","fasta":"sid-1_cDNA.fa","topology":"linear","cds":[{"id":"sid-1","intervals":[[0,3153]],"strand":"+","completeness":"complete","geneticCode":"standard","numbering":"from-cds-start"}]},"scope":"sid-1","edit":{"name":"D404N","residue":{"cds":"sid-1","number":404,"from":"D","to":"N","codon":"AAC"}},"design":{"policy":"luriaNative","style":"q5_polymerase","minAnnealingOverlap":15,"placement":"inline","buffer":"q5_reaction"}}"#;

    const SET_REQUEST: &str = r#"{"schema":"luria.q5-request/1","coordinates":"zero-based-half-open","template":{"id":"pbr322","genbank":"J01749.1.gb","sha256":"536042a7485bdabdf49c4d21a306213a098d77d19bdba57a1c6adc76ce2b8372","cds":["tet","bla"]},"variants":{"composition":"separate_variants","members":[{"name":"s1","substitution":{"interval":[99,100],"from":"T","to":"C"}},{"name":"d1","deletion":{"interval":[99,102],"from":"TGC"}},{"name":"i1","insertion":{"point":100,"before":"T","after":"G","insert":"GGG"}},{"name":"r1","replacement":{"interval":[99,102],"from":"TGC","to":"TT"}},{"name":"x1","residue":{"number":42,"from":"A","to":"G","policy":"default"}}]},"design":{"policy":"luriaNative","style":"q5_polymerase","minAnnealingOverlap":15,"placement":"confine_to_tail","buffer":{"monovalentMillimolar":50.0,"primerNanomolar":200.5,"dmsoPercent":0.0}}}"#;

    const DEGENERATE_REQUEST: &str = r#"{"schema":"luria.q5-request/1","coordinates":"zero-based-half-open","template":{"id":"sid-1","fasta":"sid-1_cDNA.fa","topology":"circular","cds":[]},"degenerate":{"name":"lib","interval":[1209,1212],"from":"GAT","to":"NNK"},"design":{"policy":"luriaNative","style":"q5_polymerase","minAnnealingOverlap":15,"placement":"inline","buffer":"q5_reaction"}}"#;

    fn algorithm_block(role: &str, name: &str, version: &str, last: bool) -> String {
        format!(
            "      {{\n        \"role\": \"{}\",\n        \"name\": \"{}\",\n        \"version\": \"{}\"\n      }}{}\n",
            role,
            name,
            version,
            if last { "" } else { "," }
        )
    }

    // The echo is the request itself, byte for byte when the input was
    // already in schema order and compact form, so it parses back to the
    // same document.
    #[test]
    fn the_echo_is_the_request_in_schema_order() {
        for text in [SID1_REQUEST, SET_REQUEST, DEGENERATE_REQUEST] {
            let document = parse_request(text).unwrap();
            let echo = serde_json::to_string(&request_echo(&document)).unwrap();
            assert_eq!(echo, text);
            assert_eq!(parse_request(&echo).unwrap(), document);
        }
    }

    #[test]
    fn the_buffer_is_resolved_in_provenance() {
        let preset = parse_request(SID1_REQUEST).unwrap();
        let view = provenance(None, Some(&preset), None);
        assert_eq!(view.operation, Some("edit"));
        assert_eq!(
            view.buffer,
            Some(BufferView {
                preset: Some("q5_reaction"),
                monovalent_millimolar: 150.0,
                primer_nanomolar: 500.0,
                dmso_percent: 0.0,
            })
        );
        let numbers = parse_request(SET_REQUEST).unwrap();
        let view = provenance(None, Some(&numbers), None);
        assert_eq!(view.operation, Some("variants"));
        assert_eq!(
            view.buffer,
            Some(BufferView {
                preset: None,
                monovalent_millimolar: 50.0,
                primer_nanomolar: 200.5,
                dmso_percent: 0.0,
            })
        );
        assert_eq!(view.template, None);
        let degenerate = parse_request(DEGENERATE_REQUEST).unwrap();
        assert_eq!(provenance(None, Some(&degenerate), None).operation, Some("degenerate"));
    }

    #[test]
    fn an_error_names_its_code_rule_category_and_path() {
        let error = DomainError::InvalidMutationRequest(InvalidMutationRequestProblem {
            request: "Z".to_string(),
            rule: MutationRequestRule::ResidueNotInStandardCode,
            reason: "the letter is not a residue of the standard genetic code".to_string(),
        });
        let view = domain_error_view(&error, Some("$.edit.residue".to_string()));
        assert_eq!(view.code, "invalid_mutation_request");
        assert_eq!(view.rule, Some("residue_not_in_standard_code"));
        assert_eq!(view.category, "malformed_input");
        assert_eq!(view.message, error.message());
        assert_eq!(view.path, Some("$.edit.residue".to_string()));
        assert_eq!(category_name(ErrorCategory::BiologicalValidation), "biological_validation");
        assert_eq!(category_name(ErrorCategory::Unsupported), "unsupported");
        assert_eq!(rule_code(&DomainError::EmptyIdentifier), None);
    }

    // The whole failed document is pinned, so the fixed shape, the key order
    // and the layout are too.
    #[test]
    fn an_unreadable_request_gives_the_pinned_failed_document() {
        let failure = PrepareFailure {
            failure: RequestFailure::Request(RequestError {
                code: RequestErrorCode::Unreadable,
                path: "$".to_string(),
                message: "cannot read the request file: No such file or directory (os error 2)"
                    .to_string(),
            }),
            request_sha256: None,
            document: None,
        };
        let algorithms: Vec<crate::version::Algorithm> = crate::version::algorithms();
        let blocks: String = algorithms
            .iter()
            .enumerate()
            .map(|(index, algorithm)| {
                algorithm_block(
                    algorithm.role,
                    algorithm.name,
                    algorithm.version,
                    index + 1 == algorithms.len(),
                )
            })
            .collect();
        let expected: String = format!(
            "{{\n  \"schema\": \"luria.q5-result/1\",\n  \"state\": \"failed\",\n  \"experimentalValidation\": \"not_performed\",\n  \"provenance\": {{\n    \"package\": {{\n      \"name\": \"luria-cli\",\n      \"version\": \"{}\"\n    }},\n    \"algorithms\": [\n{}    ],\n    \"requestSha256\": null,\n    \"operation\": null,\n    \"template\": null,\n    \"buffer\": null\n  }},\n  \"request\": null,\n  \"target\": null,\n  \"clarifications\": [],\n  \"candidates\": [],\n  \"warnings\": [],\n  \"results\": [],\n  \"error\": {{\n    \"code\": \"request_unreadable\",\n    \"rule\": null,\n    \"category\": \"malformed_input\",\n    \"message\": \"cannot read the request file: No such file or directory (os error 2)\",\n    \"path\": \"$\"\n  }}\n}}\n",
            env!("CARGO_PKG_VERSION"),
            blocks
        );
        assert_eq!(render(&failed_document(&failure)), expected);
    }

    #[test]
    fn a_failure_after_parsing_echoes_the_request() {
        let document = parse_request(SID1_REQUEST).unwrap();
        let failure = PrepareFailure {
            failure: RequestFailure::Domain {
                path: "$.edit.residue".to_string(),
                error: Box::new(DomainError::EmptyIdentifier),
            },
            request_sha256: Some("ab".repeat(32)),
            document: Some(Box::new(document.clone())),
        };
        let failed = failed_document(&failure);
        assert_eq!(failed.state, "failed");
        assert_eq!(failed.experimental_validation, "not_performed");
        assert_eq!(failed.request, Some(request_echo(&document)));
        assert_eq!(failed.provenance.request_sha256, Some("ab".repeat(32)));
        assert_eq!(failed.provenance.operation, Some("edit"));
        assert_eq!(failed.provenance.template, None);
        let error = failed.error.unwrap();
        assert_eq!(error.code, "empty_identifier");
        assert_eq!(error.path, Some("$.edit.residue".to_string()));
    }
}
```

### 4.2 Integration test: `crates/cli/tests/result.rs` (planner-written; add verbatim)

The feature lines 260 (`tet`) and 361 (`bla`) were read from
`tests/fixtures/genbank/raw/J01749.1.gb`. The other values come from that
fixture's README and `tests/fixtures/fasta/expected.tsv`.

```rust
//! Provenance built from prepared fixture requests.

use luria_cli::prepare::{prepare, Prepared};
use luria_cli::result::{failed_after, provenance, SourceView};
use luria_core::identity::sha256_hex;
use std::fs;
use std::path::PathBuf;

fn workspace(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("result-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::copy(
        "../../tests/fixtures/example-construct/input/sid-1_cDNA.fa",
        dir.join("sid-1_cDNA.fa"),
    )
    .unwrap();
    fs::copy("../../tests/fixtures/genbank/raw/J01749.1.gb", dir.join("J01749.1.gb")).unwrap();
    dir
}

fn prepared(test: &str, request: &str) -> (Prepared, String) {
    let dir = workspace(test);
    let path = dir.join("request.json");
    fs::write(&path, request).unwrap();
    let digest = sha256_hex(&fs::read(&path).unwrap());
    (prepare(&path).unwrap_or_else(|failed| panic!("{:?}", failed.failure)), digest)
}

const SID1: &str = r#"{"schema":"luria.q5-request/1","coordinates":"zero-based-half-open","template":{"id":"sid-1","fasta":"sid-1_cDNA.fa","topology":"linear","cds":[{"id":"sid-1","intervals":[[0,3153]],"strand":"+","completeness":"complete","geneticCode":"standard","numbering":"from-cds-start"}]},"scope":"sid-1","edit":{"name":"D404N","residue":{"cds":"sid-1","number":404,"from":"D","to":"N","codon":"AAC"}},"design":{"policy":"luriaNative","style":"q5_polymerase","minAnnealingOverlap":15,"placement":"inline","buffer":"q5_reaction"}}"#;

const PBR322: &str = r#"{"schema":"luria.q5-request/1","coordinates":"zero-based-half-open","template":{"id":"pbr322","genbank":"J01749.1.gb","cds":["tet","bla"]},"scope":"tet","edit":{"name":"s1","substitution":{"interval":[99,100],"from":"T","to":"C"}},"design":{"policy":"luriaNative","style":"q5_polymerase","minAnnealingOverlap":15,"placement":"inline","buffer":"q5_reaction"}}"#;

#[test]
fn a_fasta_template_is_described_by_its_digest_and_header() {
    let (ready, digest) = prepared("sid1", SID1);
    let view = provenance(Some(ready.request_sha256.clone()), None, Some(&ready));
    assert_eq!(view.request_sha256, Some(digest));
    let template = view.template.unwrap();
    assert_eq!(template.id, "sid-1");
    assert_eq!(template.sha256, "54704740a9f04b79d17147ef95788ad9f0f11e5263843a7ed490924d43b55e3a");
    assert_eq!(template.length, 3153);
    assert_eq!(template.topology, "linear");
    assert_eq!(
        template.source,
        SourceView::Fasta {
            path: "sid-1_cDNA.fa".to_string(),
            header: "3xFLAG-mVenus-sid-1 cDNA".to_string()
        }
    );
    assert_eq!(template.genbank, None);
}

#[test]
fn a_genbank_template_lists_its_imported_and_unused_features() {
    let (ready, _) = prepared("pbr322", PBR322);
    let template = provenance(Some(ready.request_sha256.clone()), None, Some(&ready))
        .template
        .unwrap();
    assert_eq!(template.topology, "circular");
    assert_eq!(
        template.source,
        SourceView::Genbank {
            path: "J01749.1.gb".to_string(),
            locus_name: "SYNPBR322".to_string(),
            accession: Some("J01749".to_string()),
            version: Some("J01749.1".to_string())
        }
    );
    let genbank = template.genbank.unwrap();
    let imported: Vec<(String, &str, usize, u64, Option<String>)> = genbank
        .imported_cds
        .iter()
        .map(|cds| (cds.id.clone(), cds.id_qualifier, cds.feature_index, cds.line, cds.transl_table.clone()))
        .collect();
    assert_eq!(
        imported,
        vec![
            ("tet".to_string(), "gene", 10, 260, Some("11".to_string())),
            ("bla".to_string(), "gene", 38, 361, Some("11".to_string())),
        ]
    );
    assert_eq!(genbank.unused_features.len(), 48);
    let rop = genbank.unused_features.iter().find(|feature| feature.index == 26).unwrap();
    assert_eq!(rop.key, "CDS");
    assert_eq!(rop.id, None);
    let refusal = rop.refusal.clone().unwrap();
    assert_eq!((refusal.code, refusal.rule), ("unusable_genbank_cds", Some("no_id")));
    assert!(genbank
        .unused_features
        .iter()
        .filter(|feature| feature.key != "CDS")
        .all(|feature| feature.refusal.is_none() && feature.id.is_none()));
}

#[test]
fn a_failed_run_keeps_the_template_provenance() {
    let (ready, _) = prepared("failed-after", SID1);
    let error = luria_cli::result::domain_error_view(&luria_core::error::DomainError::EmptyIdentifier, None);
    let failed = failed_after(&ready, error);
    assert_eq!(failed.state, "failed");
    assert!(failed.provenance.template.is_some());
    assert!(failed.request.is_some());
    assert_eq!(failed.error.unwrap().path, None);
}
```

`cargo fmt` may reflow these; do not change content. If a test fails, stop
and report what the code returned. Do not change the test.

### 4.3 Proving the rules are load-bearing (AGENTS.md)

| Rule | Test that must fail when it is broken |
| --- | --- |
| absent optional fields are omitted from the echo (remove one `skip_serializing_if`) | `the_echo_is_the_request_in_schema_order` |
| the preset resolves to `BufferModel::q5_reaction()` (resolve it to `None` numbers or zeros) | `the_buffer_is_resolved_in_provenance` |
| the top-level keys keep struct order (swap two fields) | `an_unreadable_request_gives_the_pinned_failed_document` |
| `genbank` is `None` for FASTA | `a_fasta_template_is_described_by_its_digest_and_header` |

## 5. Do not

- Edit files other than `crates/cli/src/result.rs` (new),
  `crates/cli/src/lib.rs` and `crates/cli/tests/result.rs` (new).
- Use `serde_json::Value` or `json!` outside tests, or compute any value
  that a core accessor already gives.
- Change `luria-core`, `request.rs`, `json.rs` or `prepare.rs`. If one gets
  in the way, stop and report.
- Add dependencies.

## 6. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 7. Handoff

Report the changed and new files, the exact commands and results, the §4.3
results, and anything that did not match this contract.
