# Milestone 5e2: preparing a request for the workflow

Status: **Implemented 2026-10-17** by the local worker; accepted after two review rounds, both on planner errors the worker reported: the digest test could not add a member, and the types and one test helper were too large or too specific for clippy.
Planner: Claude. Parent: [docs/tasks/contracts/milestone-5-cli.md](/tasks/contracts/milestone-5-cli.md) §3.3, §3.4 and §5
(5e, split into 5e1 and 5e2). Follows 5e1 (`request.rs`, the strict
document).

## 1. What and why

5e1 turns request text into a `RequestDocument` without touching files or
core. This contract takes the next step: from a request file's path to
everything `luria q5 design` needs before it runs. That means the template
`Sequence` and its source metadata, the CDS annotations, the core request
for the operation, and the `PrimerDesignRequest`. Every value is built by a
`luria-core` constructor, so core's own rules and errors apply. Each failure
names the JSON path of the request element it came from.

Nothing is executed here: no `execute`, `execute_set` or design call. That
is 5g.

## 2. Requirements this contract applies

- **Plan §3.4:** paths are relative to the request file's directory.
  `scope` is optional; when it's absent, the library's target resolution
  applies. `combined_construct` is accepted by the parser and refused by the
  workflow, which here means `Q5VariantSetRequest::create`.
- **Plan §3.3, D041:** FASTA topology and CDSs come from the request. A
  GenBank template's topology comes from its LOCUS line, and its CDSs from
  `import_cds` with the requested ids. The template's identifier is the
  request's `template.id`, never the file's header or LOCUS name (D007).
- **Plan §3.6:** CLI-raised errors are `MalformedInput` with codes
  `request_unreadable`, `request_invalid`, `template_unreadable` and
  `template_digest_mismatch` (5e1's `RequestErrorCode`).
- **D021:** provenance includes the request file's SHA-256 (of its bytes).
- **Decided here:**
  - a template path must be relative. An absolute path gives
    `request_invalid`, so a result never echoes a machine-specific
    location;
  - two declared FASTA CDSs with the same id give `request_invalid`,
    matching GenBank's `RepeatedName`;
  - `template.sha256` is compared with the sequence's content digest, the
    one `Sequence::content_hash` gives and provenance reports.

## 3. The module: `crates/cli/src/prepare.rs`

Add `pub mod prepare;` to `lib.rs`, keeping the module list alphabetical.

```rust
/// Why a request could not be prepared.
#[derive(Clone, Debug, PartialEq)]
pub enum RequestFailure {
    /// Raised by the CLI itself (5e1's codes).
    Request(RequestError),
    /// Raised by a luria-core constructor or reader, at the JSON path of
    /// the request element it was built from. Boxed so the failure stays
    /// small (clippy `result_large_err`).
    Domain { path: String, error: Box<DomainError> },
}

/// Where the template came from (D007 source metadata).
#[derive(Clone, Debug, PartialEq)]
pub enum TemplateSource {
    Fasta { path: String, header: String },
    GenBank {
        path: String,
        locus_name: String,
        accession: Option<String>,
        version: Option<String>,
        import: GenBankImport,
    },
}

/// The operation, as the core entry point that will run it takes it.
#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    /// Boxed: a request carries its whole template (clippy
    /// `large_enum_variant`).
    Single { request: Box<Q5SdmRequest>, name: Identifier },
    Set(Q5VariantSetRequest),
    Degenerate(Q5Expansion),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prepared {
    /// SHA-256 of the request file's bytes, lower-case hex.
    pub request_sha256: String,
    pub document: RequestDocument,
    pub template: Sequence,
    pub source: TemplateSource,
    pub annotations: Vec<Cds>,
    pub operation: Operation,
    pub design: PrimerDesignRequest,
}

/// A failure, with whatever was known when it happened, so a failed
/// result can still echo the request (5f). `request_sha256` is `Some` once
/// the file has been read, and `document` once it has been parsed.
#[derive(Clone, Debug, PartialEq)]
pub struct PrepareFailure {
    pub failure: RequestFailure,
    pub request_sha256: Option<String>,
    pub document: Option<Box<RequestDocument>>,
}

pub fn prepare(request_path: &Path) -> Result<Prepared, PrepareFailure>;
```

*(Amended 2026-10-17, before implementation, so that a failed result can
echo the request. Amended again in review: the three `Box`es keep clippy's
size lints quiet without an `allow`.)*

If a core type used here lacks `PartialEq` (for example `Q5Expansion`),
stop and report rather than editing core.

### 3.1 Steps, in this order

The first failure is the result, wrapped in a `PrepareFailure` that
carries the request digest (after step 1) and the document (after step 2).
`doc` is the parsed document, `dir` is
`request_path.parent()` (or `.` if it has none), and `ref` is
`template.reference()`.

1. **Read** the request file's bytes with `std::fs::read`. On error:
   `Request(Unreadable)`, path `$`, message `cannot read the request file:
   {io error}`. Bytes that are not UTF-8: `Request(Unreadable)`, path `$`,
   message `the request file is not UTF-8`. `request_sha256` is
   `luria_core::identity::sha256_hex(&bytes)`.
2. **Parse** with `parse_request`. Its error is returned as
   `Request(error)`.
3. **Template path:** the `fasta` or `genbank` string. If
   `Path::new(text).is_absolute()`: `Request(Invalid)` at
   `$.template.fasta` (or `.genbank`), message `expected a path relative to
   the request file`. Otherwise read `dir.join(text)` with
   `std::fs::read_to_string`. On error: `Request(TemplateUnreadable)` at the
   same path, message `cannot read the template file '{text}': {io error}`.
4. **Identifier:** `Identifier::create(template.id)`, at `$.template.id`.
5. **Read the record:**
   - FASTA: `read_fasta(id, text, topology)`, at `$.template.fasta`;
   - GenBank: `read_genbank(id, text)`, at `$.template.genbank`.
6. **Digest:** if `sha256` is `Some(given)` and differs from
   `template.content_hash().hex_digest()`:
   `Request(TemplateDigestMismatch)` at `$.template.sha256`, message `the
   template sequence's SHA-256 is {actual}, not {given}`.
7. **Annotations:**
   - FASTA: for each declared CDS `i`, in order:
     - `Identifier::create(id)` at `$.template.cds[i].id`;
     - an id already declared at a lower index: `Request(Invalid)` at
       `$.template.cds[i].id`, message `CDS id '{id}' is declared twice`;
     - each interval `j`: `interval::create(&ref, start, end)` at
       `$.template.cds[i].intervals[j]`;
     - `Cds::from_optional(&template, id, Some(intervals), Some(strand),
       Some(completeness), Some(genetic_code), Some(numbering))` at
       `$.template.cds[i]`. `StrandSpec::Plus` is `Strand::Plus`.
   - GenBank: each id `i` through `Identifier::create` at
     `$.template.cds[i]`, then `import_cds(&record, &names)` at
     `$.template.cds`. The annotations are the imported CDSs, in order.
8. **Scope:** `Identifier::create` at `$.scope` when present.
9. **Operation.** For one member at path `P` (`$.edit` or
   `$.variants.members[i]`):
   - `Identifier::create(name)` at `P.name`;
   - the intent, at `P.<kind>` (for example `$.edit.residue`):
     - residue with `cds`: `ResiduePosition::create(Identifier(cds),
       number)`, then `ResidueMutationIntent::create(position, from, to,
       policy)`, giving `Q5MutationIntent::Residue`. Any `Identifier` error
       for `cds` is also reported at `P.residue`;
     - residue without `cds`: `ResidueIntentWithoutCds::create(number, from,
       to, policy)`, giving `ResidueWithoutCds`;
     - `policy`: `CodonSpec::Supplied(text)` is
       `CodonSelectionPolicy::Supplied(Codon::create(text)?)`, and
       `CodonSpec::Default` is `CodonSelectionPolicy::Default`;
     - substitution: `interval::create(&ref, s, e)`, then
       `NucleotideMutationIntent::create(ref, interval, from, to)`;
     - deletion: `NucleotideDeletionIntent::create(ref, region, from)`;
     - insertion: `create_point(&ref, point)`, then
       `NucleotideInsertionIntent::create(ref, point, before, after,
       insert)`;
     - replacement: `NucleotideReplacementIntent::create(ref, region, from,
       to)`;
   - `Q5SdmRequest::create(template.clone(), scope.clone(), intent)` at `P`.

   Then, by form:
   - `edit`: `Operation::Single { request, name }`;
   - `variants`: each member in order becomes `Q5Variant::create(request,
     name)`, then `Q5VariantSetRequest::create(composition, variants)` at
     `$.variants`;
   - `degenerate`: `Identifier::create(name)` at `$.degenerate.name`,
     `interval::create(&ref, s, e)` at `$.degenerate.interval`, then
     `expand_replacement(template.clone(), scope.clone(), region, from, to,
     name)` at `$.degenerate`.
10. **Design:** the buffer is `BufferModel::q5_reaction()` for the preset,
    or `BufferModel::create(m, p, d)` at `$.design.buffer`. Then
    `PrimerDesignRequest::create(DesignStyle::Q5Polymerase,
    DesignPolicy::LuriaNative, min_annealing_overlap, placement, buffer)` at
    `$.design`.

Write a module doc comment summarizing §2 and the step order.

## 4. Tests: `crates/cli/tests/prepare.rs` (planner-written; add verbatim)

The test inputs were read from the fixtures by the planner:
- SID-1 is one ORF over its whole 3153 bases: ATG, then a stop at codon
  1051 and nowhere else. Residue 404 is D, written `GAT` at `[1209,
  1212)`. Its content digest is in `tests/fixtures/fasta/expected.tsv`.
- pBR322 has `T` at 99 and `TGC` at `[99, 102)`. `tet`'s residue 2 is K
  (`AAA` at `[88, 91)`).
- `NNK` stands for 4 × 4 × 2 = 32 codons, and `GAT` is one of them (K is G
  or T). So the expansion of `GAT` → `NNK` has 31 members and skips the
  wild type.

```rust
//! `prepare`: a request file's path to everything the workflow needs, with
//! each failure at the JSON path it came from. The templates are the
//! committed fixtures, copied next to each test's request file.

use luria_cli::prepare::{prepare, Operation, PrepareFailure, Prepared, RequestFailure, TemplateSource};
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
    fs::copy("../../tests/fixtures/genbank/raw/J01749.1.gb", dir.join("J01749.1.gb")).unwrap();
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
        Err(PrepareFailure { failure, request_sha256, document }) => {
            // Once the file has been read its digest is known, and once it
            // parsed, so is the document.
            assert_eq!(request_sha256, Some(sha256_hex(&fs::read(&path).unwrap())));
            if let RequestFailure::Domain { .. } = failure {
                assert!(document.is_some(), "{}: a domain failure keeps the document", test);
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
    right.pointer_mut("/template").unwrap().as_object_mut().unwrap()
        .insert("sha256".to_string(), json!(SID1_DIGEST));
    prepared("digest-right", &right);

    let mut wrong = sid1();
    wrong.pointer_mut("/template").unwrap().as_object_mut().unwrap()
        .insert("sha256".to_string(), json!(PBR322_DIGEST));
    assert_eq!(
        request_error(failure("digest-wrong", &wrong)),
        (
            RequestErrorCode::TemplateDigestMismatch,
            "$.template.sha256".to_string(),
            format!("the template sequence's SHA-256 is {}, not {}", SID1_DIGEST, PBR322_DIGEST)
        )
    );
}

#[test]
fn the_pbr322_request_prepares_against_the_genbank_template() {
    let ready = prepared("pbr322", &pbr322());
    assert_eq!(ready.template.content_hash().hex_digest(), PBR322_DIGEST);
    assert_eq!(ready.template.name().value(), "pbr322");
    let ids: Vec<String> =
        ready.annotations.iter().map(|cds| cds.id().value().to_string()).collect();
    assert_eq!(ids, vec!["tet".to_string(), "bla".to_string()]);
    match ready.source {
        TemplateSource::GenBank { path, locus_name, accession, version, import } => {
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
    match prepare(&dir.join("absent.json")).map_err(|failed| (failed.failure, failed.request_sha256)) {
        Err((RequestFailure::Request(error), None)) => {
            assert_eq!(error.code, RequestErrorCode::Unreadable);
            assert_eq!(error.path, "$");
            assert!(error.message.starts_with("cannot read the request file: "), "{}", error.message);
        }
        other => panic!("expected request_unreadable, got {:?}", other),
    }

    let latin = workspace("latin1");
    fs::write(latin.join("request.json"), [0x7b, 0xff, 0x7d]).unwrap();
    assert_eq!(
        request_error(prepare(&latin.join("request.json")).unwrap_err().failure),
        (RequestErrorCode::Unreadable, "$".to_string(), "the request file is not UTF-8".to_string())
    );

    let mut absent = sid1();
    set(&mut absent, "/template/fasta", json!("absent.fa"));
    let (code, path, message) = request_error(failure("absent-template", &absent));
    assert_eq!(code, RequestErrorCode::TemplateUnreadable);
    assert_eq!(path, "$.template.fasta");
    assert!(message.starts_with("cannot read the template file 'absent.fa': "), "{}", message);

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
    assert_eq!(request_error(failure("bad-json", &bad_json)).0, RequestErrorCode::Invalid);
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
    twice.pointer_mut("/template/cds").unwrap().as_array_mut().unwrap().push(first);
    assert_eq!(
        request_error(failure("cds-twice", &twice)),
        (
            RequestErrorCode::Invalid,
            "$.template.cds[1].id".to_string(),
            "CDS id 'sid-1' is declared twice".to_string()
        )
    );

    let mut negative = sid1();
    set(&mut negative, "/template/cds/0/intervals/0", json!([-3, 3153]));
    assert_eq!(
        domain_error(failure("cds-negative", &negative)),
        pair("$.template.cds[0].intervals[0]", "negative_coordinate")
    );

    let mut code = sid1();
    set(&mut code, "/template/cds/0/geneticCode", json!("vertebrate_mitochondrial"));
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
    assert_eq!(domain_error(failure("edit-name", &name)), pair("$.edit.name", "empty_identifier"));

    let mut scope = sid1();
    set(&mut scope, "/scope", json!("a b"));
    assert_eq!(
        domain_error(failure("scope", &scope)),
        pair("$.scope", "invalid_identifier_characters")
    );

    let mut member = pbr322();
    set(&mut member, "/variants/members/2/insertion/point", json!(-1));
    assert_eq!(
        domain_error(failure("member-point", &member)),
        pair("$.variants.members[2].insertion", "negative_coordinate")
    );

    let mut combined = pbr322();
    set(&mut combined, "/variants/composition", json!("combined_construct"));
    assert_eq!(
        domain_error(failure("combined", &combined)),
        pair("$.variants", "unsupported_composition")
    );

    let mut empty = pbr322();
    set(&mut empty, "/variants/members", json!([]));
    assert_eq!(domain_error(failure("empty-set", &empty)), pair("$.variants", "invalid_mutation_request"));

    let mut salt = pbr322();
    set(&mut salt, "/design/buffer/monovalentMillimolar", json!(0));
    assert_eq!(domain_error(failure("salt", &salt)), pair("$.design.buffer", "invalid_buffer_model"));

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
    assert!(!message.contains(env!("CARGO_TARGET_TMPDIR")), "{}", message);
}
```

`cargo fmt` may reflow; do not change content. If a test fails, stop and
report the test and what `prepare` returned. Do not change the test.

### 4.1 Proving the rules are load-bearing (AGENTS.md)

| Rule | Test that must fail when it is disabled |
| --- | --- |
| an absolute template path is refused | `file_problems_are_the_clis_own_errors` |
| the digest is compared | `a_matching_digest_is_accepted_and_a_wrong_one_refused` |
| a declared CDS id may not repeat | `declared_cds_problems_fail_where_they_are_declared` |
| template paths are joined to the request's directory (resolve against the current directory instead) | `the_sid1_request_prepares_against_the_fasta_template` |
| the template's identifier is the request's id, not the header | `the_sid1_request_prepares_against_the_fasta_template` |

## 5. Do not

- Edit files other than `crates/cli/src/prepare.rs` (new),
  `crates/cli/src/lib.rs` and `crates/cli/tests/prepare.rs` (new).
- Run any workflow, design or edit; check the intent against the template
  beyond what the named constructors do; or repeat a core rule in the CLI.
- Change `luria-core`, `request.rs` or `json.rs`. If one gets in the way,
  stop and report.
- Add dependencies.

## 6. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 7. Handoff

Report the changed and new files, the exact commands and results, the §4.1
results, and anything that did not match this contract.
