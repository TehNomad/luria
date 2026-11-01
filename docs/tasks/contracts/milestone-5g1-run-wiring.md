# Milestone 5g1: `luria q5 design`, wired

Status: **Implemented 2026-10-17** by the local worker; accepted on review.
Review notes: the worker reported that §3.3 and §5 conflicted over 5d's stub
test; §5 was amended to delete it. On the unreachable non-clarification path
with no mutation, `run` unwraps `mutation()` as core's `design_run` does, so
that broken invariant would panic rather than exit 1. Accepted, because a
test pins that core never produces it.
Planner: Claude. Parent: `docs/tasks/contracts/milestone-5-cli.md` §3.2, §3.7 and §5
(5g1). Follows 5f1–5f4 (the result document) and 5e2 (`prepare`).

## 1. What and why

Everything the command needs now exists: `prepare` turns a request file into
core types (5e2), and `result.rs` turns core's answers into the result
document (5f1–5f4). `main.rs` still refuses `q5 design` with
`NOT_WIRED`. This contract joins the two: one function that runs a prepared
request through core and returns the document and the exit status.

5g2 adds `--write-edited`. 5g3 adds the plan §6 end-to-end tests. This
contract's tests check the wiring: which document, which state, which exit
status, and that the design in the document is the one `native.tsv` records.

## 2. Requirements this contract applies

- **Plan §3.2:** `q5 design` runs the whole workflow and prints one JSON
  result document to stdout.
- **Plan §3.7, the exit statuses:** 0 verified and every member designed;
  1 internal or I/O failure, with the document still written if it can be;
  3 `needs_clarification`; 4, 5, 6 `failed` by the error's category;
  7 verified but at least one member not designed.
- **G7 (D041, owner 2026-10-17):** an edit that is applied and verified but
  cannot be designed is reported like a one-member set — state `verified`,
  the member carries its mutation and its `designFailure`, exit 7. It is
  **not** a failed run, even though `NoPrimerDesignFits` is an `Unsupported`
  error that would otherwise be exit 6.
- **D005, D020:** a run that needs clarification performs no edit and
  designs nothing; the clarification is the answer. A result says `verified`
  only when core says the edit was verified.
- **D013:** this layer re-verifies nothing and re-derives nothing. Every
  value in the document comes from core through 5f's view functions.
- **D012:** the same input gives byte-identical output. Nothing here reads
  the clock, the environment or an absolute path.

## 3. The module: `crates/cli/src/run.rs` (new)

Add `pub mod run;` to `lib.rs`, after `pub mod result;`.

```rust
/// What one `luria q5 design` run produced: the document to print and the
/// status to exit with. Both are always present: a failure is a document
/// too (plan §3.7).
#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    pub document: ResultDocument,
    pub status: ExitStatus,
}

/// Run one request file through core and report it.
///
/// The only failure that is not a document is one this function cannot
/// reach: every path returns an `Outcome`.
pub fn run(request_path: &Path) -> Outcome;
```

### 3.1 What `run` does

1. `prepare(request_path)`. On `Err(failure)`:
   - the document is `failed_document(&failure)`;
   - the status is `ExitStatus::for_failure(category)`, where the category is
     `ErrorCategory::MalformedInput` for `RequestFailure::Request` and
     `error.category()` for `RequestFailure::Domain`. Write this as a small
     private helper, `fn failure_category(failure: &RequestFailure) -> ErrorCategory`.
2. On `Ok(prepared)`, match `prepared.operation` and take the path below.
   Every path that fails with a `DomainError` gives
   `failed_after(&prepared, domain_error_view(&error, None))` and
   `ExitStatus::for_failure(error.category())`. The path is `None`: the
   request was understood, so the error is not located in it.
3. The three paths:

**`Operation::Single { request, name }`** —
`q5::execute((**request).clone(), prepared.annotations.clone(), name.clone())`.
On `Ok(execution)`:
- `execution.needs_clarification()`: the document is
  `clarification_document(&prepared, &execution.scope())`, status
  `NeedsClarification`.
- otherwise the edit was verified. Build the member from
  `member_view(name, &mutation, &warnings)`, where `mutation` is
  `execution.mutation()` and `warnings` is `execution.warnings()`, then
  `q5_design::design_run(execution.clone(), prepared.design.clone())`:
  - `Ok(designed)` with `designed.design()` `Some(design)`:
    `designed_member(member, &design)`, status `Success`;
  - `Err(error)`: `undesigned_member(member, &error)`, status
    `PartialDesign` — **G7**;
  - `Ok(designed)` with `designed.design()` `None`: cannot happen on a
    verified run (§3.2). Use the member as `member_view` built it and status
    `Internal`, so a broken invariant is an internal error and never a
    document that claims a design.
- the document is `verified_document(&prepared, target, vec![member], vec![])`,
  with `target` `execution.scope().target()`.

**`Operation::Set(set)`** —
`q5::execute_set(set.clone(), prepared.annotations.clone())`. On
`Ok(execution)`:
- `execution.needs_clarification()`: `clarification_document(&prepared,
  &execution.scope())`, status `NeedsClarification`.
- otherwise `q5_design::design_set(execution.clone(), prepared.design.clone())`,
  then one member per `execution.results()`, **in that order** (request
  order, which `design_set` preserves). For each `result`, find the
  `Q5MemberDesign` whose `result_name()` equals `result.identifier()`:
  - found and `design()` is `Some(design)`: `designed_member(member, &design)`;
  - found and `failure()` is `Some(error)`: `undesigned_member(member, &error)`;
  - not found: status `Internal` for the run, the member as `member_view`
    built it. (`design_set` returns one outcome per member, so this is the
    same unreachable-invariant case as above.)
  where `member` is
  `member_view(&result.identifier(), &result.mutation(), &result.warnings())`.
- the status is `Success` when every member was designed, `PartialDesign`
  when at least one was not, `Internal` on the unreachable case.
- the document is `verified_document(&prepared, target, members, vec![])`,
  `target` from `execution.target()`.

**`Operation::Degenerate(expansion)`** — the same as `Operation::Set` with
`expansion.set()` as the set, and the operation warnings are
`expansion.warnings().iter().map(operation_warning_view).collect()` instead
of `vec![]`. The expansion itself was built in `prepare` (5e2), so no
expansion happens here.

### 3.2 The invariant this relies on

Core's `execute` and `execute_set` return `Ok` only for a run that needs
clarification or one whose edit was verified: a verification failure is
`MutationVerificationFailure`, an `Err`. So on the non-clarification path
`mutation()` is `Some` and `is_verified()` is true. §4.1's
`a_verified_run_is_verified_by_core` pins this for all three paths rather
than trusting it.

### 3.3 `main.rs`

- Replace the `Q5Design` arm's `not_wired()` with a function that calls
  `run`, writes `render(&outcome.document)` with the existing `write_out`,
  and returns `outcome.status`. A failed write returns `ExitStatus::Internal`
  (plan §3.7), and nothing goes to stderr on that path: the status is the
  report.
- `--write-edited` arrives in 5g2. Until then, when `write_edited` is
  `Some`, write
  `luria: '--write-edited' is not available in this build yet\n` to stderr
  and return `ExitStatus::Internal`, without running anything. Keep this as
  its own named const, so 5g2 removes a const and an arm.
- Delete `NOT_WIRED` and `not_wired`.

### 3.4 One doc line in `exit.rs`

`ExitStatus::PartialDesign`'s comment says "at least one variant-set
member". G7 widened it. Change that line to "at least one member, of a set
or a single edit, could not be designed". Change nothing else in `exit.rs`.

## 4. Tests: `crates/cli/tests/run.rs` (planner-written; add verbatim)

`cargo fmt` may reflow this; do not change its content. The expected design
values are `tests/fixtures/q5-primer/native.tsv` row `native-01`, which is
the independently verified D404N design, and row `native-07`, which records
that a substitution at `[20, 21)` of SID-1 cannot be designed at bound 15
because only 20 template bases lie 5′ of it and the pair needs 25.

```rust
//! `luria q5 design`, wired: which document, which state, which exit
//! status. The design values come from `tests/fixtures/q5-primer/`:
//! `native.tsv` rows `native-01` and `native-07`,
//! `native-deletion.tsv` row `del-11` and `native-insertion.tsv` row
//! `ins-14`. The member names and counts come from the requests.
//!
//! Every request against pBR322 names a `scope`, because a template with
//! two CDS annotations and no scope is a clarification whatever the edit
//! is (`q5.rs`: candidates are not narrowed by the edit) — which is what
//! `a_residue_edit_with_two_candidate_cdss_asks_and_edits_nothing` tests.

use luria_cli::exit::ExitStatus;
use luria_cli::result::{DetailValue, ResultDocument};
use luria_cli::run::{run, Outcome};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A fresh directory for one test, holding copies of both templates.
fn workspace(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("run-{}", test));
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

fn design() -> Value {
    json!({
        "policy": "luriaNative",
        "style": "q5_polymerase",
        "minAnnealingOverlap": 15,
        "placement": "inline",
        "buffer": "q5_reaction"
    })
}

/// The SID-1 cDNA as a FASTA template, with its one CDS declared.
fn sid1_template() -> Value {
    json!({
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
    })
}

/// pBR322, importing both CDSs by name.
fn pbr322_template() -> Value {
    json!({ "id": "pbr322", "genbank": "J01749.1.gb", "cds": ["tet", "bla"] })
}

fn ran(test: &str, request: &Value) -> Outcome {
    let dir = workspace(test);
    run(&write(&dir, request))
}

/// Two decimals, the precision `native.tsv` records.
fn two(value: f64) -> String {
    format!("{:.2}", value)
}

fn only_member(document: &ResultDocument) -> &luria_cli::result::MemberView {
    assert_eq!(document.results.len(), 1, "one member expected");
    &document.results[0]
}

#[test]
fn the_sid1_d404n_run_is_verified_and_designed() {
    let outcome = ran(
        "d404n",
        &json!({
            "schema": "luria.q5-request/1",
            "coordinates": "zero-based-half-open",
            "template": sid1_template(),
            "scope": "sid-1",
            "edit": {
                "name": "D404N",
                "residue": { "cds": "sid-1", "number": 404, "from": "D", "to": "N", "codon": "AAC" }
            },
            "design": design()
        }),
    );
    assert_eq!(outcome.status, ExitStatus::Success);
    assert_eq!(outcome.document.state, "verified");
    assert_eq!(outcome.document.experimental_validation, "not_performed");
    assert_eq!(outcome.document.clarifications, Vec::<String>::new());
    assert!(outcome.document.error.is_none());
    assert!(outcome.document.target.is_some());

    let member = only_member(&outcome.document);
    assert_eq!(member.name, "D404N");
    assert_eq!(member.design_failure, None);
    let view = member.design.clone().unwrap();
    // native.tsv row native-01.
    assert_eq!(view.forward.span, (1199, 1222));
    assert_eq!(view.reverse.span, (1180, 1199));
    assert_eq!(view.reverse.length, 19);
    assert_eq!(two(view.forward.tm.celsius), "67.72");
    assert_eq!(view.forward.tm.basis, "mismatched_wild_type_site");
    assert_eq!(two(view.reverse.tm.celsius), "62.90");
    assert_eq!(two(view.annealing_temperature), "65.90");
    assert!(view.tm_balance_met);
}

#[test]
fn an_edit_that_cannot_be_designed_is_verified_and_exits_7() {
    // native.tsv row native-07: 20 template bases lie 5' of [20, 21) and the
    // pair needs 25, so the edit is applied and verified but no pair fits.
    // G7: that is a member's design failure, not a failed run, even though
    // no_primer_design_fits is an Unsupported error.
    let outcome = ran(
        "undesignable",
        &json!({
            "schema": "luria.q5-request/1",
            "coordinates": "zero-based-half-open",
            "template": sid1_template(),
            "edit": {
                "name": "A20G",
                "substitution": { "interval": [20, 21], "from": "A", "to": "G" }
            },
            "design": design()
        }),
    );
    assert_eq!(outcome.status, ExitStatus::PartialDesign);
    assert_eq!(outcome.status.code(), 7);
    assert_eq!(outcome.document.state, "verified");
    let member = only_member(&outcome.document);
    assert_eq!(member.name, "A20G");
    assert_eq!(member.design, None);
    let failure = member.design_failure.clone().unwrap();
    assert_eq!(failure.code, "no_primer_design_fits");
    assert_eq!(failure.category, "unsupported");
    assert_eq!(failure.path, None);
    // 5f4: the details carry the payload, so the obstacle is readable.
    let keys: Vec<&str> = failure.details.0.iter().map(|(key, _)| *key).collect();
    assert_eq!(keys, vec!["template", "editStart", "editEnd", "obstacles"]);
    assert_eq!(
        failure.details.0[1].1,
        DetailValue::Count(20),
        "editStart"
    );
    // The run still failed nothing: there is no document-level error.
    assert!(outcome.document.error.is_none());
}

/// The three SID-1 edits at residue 404 that the fixtures record as
/// designable: `native-01` (the D404N residue edit), `del-11` (deleting
/// `GAT` at `[1209, 1212)`) and `ins-14` (inserting `GAT` at point 1209,
/// whose flanking bases are `A` at 1208 and `G` at 1209).
fn set_members() -> Value {
    json!([
        { "name": "s1", "residue": { "cds": "sid-1", "number": 404, "from": "D", "to": "N", "codon": "AAC" } },
        { "name": "d1", "deletion": { "interval": [1209, 1212], "from": "GAT" } },
        { "name": "i1", "insertion": { "point": 1209, "before": "A", "after": "G", "insert": "GAT" } }
    ])
}

#[test]
fn a_variant_set_gives_one_member_per_variant_in_request_order() {
    let outcome = ran(
        "set",
        &json!({
            "schema": "luria.q5-request/1",
            "coordinates": "zero-based-half-open",
            "template": sid1_template(),
            "scope": "sid-1",
            "variants": { "composition": "separate_variants", "members": set_members() },
            "design": design()
        }),
    );
    assert_eq!(outcome.status, ExitStatus::Success);
    assert_eq!(outcome.document.state, "verified");
    let names: Vec<String> = outcome
        .document
        .results
        .iter()
        .map(|member| member.name.clone())
        .collect();
    assert_eq!(names, vec!["s1", "d1", "i1"]);
    for member in outcome.document.results.iter() {
        assert_eq!(member.design_failure, None, "{}", member.name);
        assert!(member.design.is_some(), "{}", member.name);
    }
    assert_eq!(outcome.document.warnings.len(), 0);

    // ins-14: a tailed design, so the set carries each member's own design
    // and not one shared by all three.
    let insertion = outcome.document.results[2].design.clone().unwrap();
    assert_eq!(insertion.forward.sequence, "GATGATTTCGGGGCGCAC");
    assert_eq!(insertion.reverse.sequence, "TTGCTCAACGAGCGG");
    assert_eq!(insertion.forward.tail.clone().unwrap().sequence, "GAT");
    assert_eq!(two(insertion.forward.tm.celsius), "64.31");
    assert_eq!(
        two(insertion.forward.whole_primer_tm.clone().unwrap().celsius),
        "67.15"
    );
    assert_eq!(two(insertion.annealing_temperature), "65.38");

    // native-01, as the single-edit test checks it, so a member of a set and
    // a single edit give the same design for the same edit.
    let residue = outcome.document.results[0].design.clone().unwrap();
    assert_eq!(residue.forward.span, (1199, 1222));
    assert_eq!(residue.reverse.span, (1180, 1199));
}

#[test]
fn a_degenerate_expansion_skips_the_wild_type_and_says_so() {
    // SID-1 [1209, 1212) is GAT. "SAT" denotes CAT and GAT, and GAT is the
    // bases already there, so that member is left out with a warning
    // (Q5ExpansionWarning::WildTypeSkipped) and one member remains.
    let outcome = ran(
        "degenerate",
        &json!({
            "schema": "luria.q5-request/1",
            "coordinates": "zero-based-half-open",
            "template": sid1_template(),
            "degenerate": { "name": "lib", "interval": [1209, 1212], "from": "GAT", "to": "SAT" },
            "design": design()
        }),
    );
    assert_eq!(outcome.status, ExitStatus::Success);
    assert_eq!(outcome.document.state, "verified");
    let member = only_member(&outcome.document);
    assert_eq!(member.name, "lib-CAT");
    assert!(member.design.is_some());
    assert_eq!(outcome.document.warnings.len(), 1);
    let warning = serde_json::to_string(&outcome.document.warnings[0]).unwrap();
    assert!(
        warning.contains("\"code\":\"wild_type_skipped\""),
        "{}",
        warning
    );
    assert!(warning.contains("GAT"), "{}", warning);
}

#[test]
fn a_residue_edit_with_two_candidate_cdss_asks_and_edits_nothing() {
    // The brief's fail-closed case: a residue number with no CDS named, on a
    // template with two CDSs. D005: no sequence calculation is performed.
    let outcome = ran(
        "clarify",
        &json!({
            "schema": "luria.q5-request/1",
            "coordinates": "zero-based-half-open",
            "template": pbr322_template(),
            "edit": {
                "name": "G42",
                "residue": { "number": 42, "from": "A", "to": "G", "policy": "default" }
            },
            "design": design()
        }),
    );
    assert_eq!(outcome.status, ExitStatus::NeedsClarification);
    assert_eq!(outcome.status.code(), 3);
    assert_eq!(outcome.document.state, "needs_clarification");
    assert_eq!(outcome.document.results.len(), 0);
    assert_eq!(outcome.document.target, None);
    assert!(!outcome.document.clarifications.is_empty());
    assert_eq!(outcome.document.candidates.len(), 2);
    assert!(outcome.document.error.is_none());
}

#[test]
fn an_unreadable_request_fails_with_malformed_input() {
    let dir = workspace("unreadable");
    let outcome = run(&dir.join("absent.json"));
    assert_eq!(outcome.status, ExitStatus::MalformedInput);
    assert_eq!(outcome.status.code(), 4);
    assert_eq!(outcome.document.state, "failed");
    let error = outcome.document.error.clone().unwrap();
    assert_eq!(error.code, "request_unreadable");
    assert_eq!(error.path, Some("$".to_string()));
}

#[test]
fn a_wrong_expected_base_fails_with_biological_validation() {
    // [1209, 1210) of SID-1 is G, not A, so the request's claim is refused
    // and the category decides the status: 5, not 4.
    let outcome = ran(
        "mismatch",
        &json!({
            "schema": "luria.q5-request/1",
            "coordinates": "zero-based-half-open",
            "template": sid1_template(),
            "edit": {
                "name": "wrong",
                "substitution": { "interval": [1209, 1210], "from": "A", "to": "C" }
            },
            "design": design()
        }),
    );
    assert_eq!(outcome.status, ExitStatus::BiologicalValidation);
    assert_eq!(outcome.status.code(), 5);
    assert_eq!(outcome.document.state, "failed");
    let error = outcome.document.error.clone().unwrap();
    assert_eq!(error.code, "expected_nucleotide_mismatch");
    assert_eq!(error.category, "biological_validation");
    assert_eq!(outcome.document.results.len(), 0);
}

// §3.2: core returns Ok only for a clarification or a verified edit, so a
// document that is not `needs_clarification` always says `verified`. Every
// path above is one of the two.
#[test]
fn a_verified_run_is_verified_by_core() {
    let states: Vec<&str> = ["d404n-inv", "set-inv", "degenerate-inv"]
        .iter()
        .zip([
            json!({
                "schema": "luria.q5-request/1",
                "coordinates": "zero-based-half-open",
                "template": sid1_template(),
                "scope": "sid-1",
                "edit": {
                    "name": "D404N",
                    "residue": { "cds": "sid-1", "number": 404, "from": "D", "to": "N", "codon": "AAC" }
                },
                "design": design()
            }),
            json!({
                "schema": "luria.q5-request/1",
                "coordinates": "zero-based-half-open",
                "template": sid1_template(),
                "scope": "sid-1",
                "variants": { "composition": "separate_variants", "members": set_members() },
                "design": design()
            }),
            json!({
                "schema": "luria.q5-request/1",
                "coordinates": "zero-based-half-open",
                "template": sid1_template(),
                "degenerate": { "name": "lib", "interval": [1209, 1212], "from": "GAT", "to": "SAT" },
                "design": design()
            }),
        ])
        .map(|(test, request)| ran(test, &request).document.state)
        .collect();
    assert_eq!(states, vec!["verified", "verified", "verified"]);
}

// main.rs is wired, not just the library: the binary prints the document and
// exits with the outcome's status.
#[test]
fn the_binary_prints_the_document_and_exits_0() {
    let dir = workspace("binary");
    let path = write(
        &dir,
        &json!({
            "schema": "luria.q5-request/1",
            "coordinates": "zero-based-half-open",
            "template": sid1_template(),
            "scope": "sid-1",
            "edit": {
                "name": "D404N",
                "residue": { "cds": "sid-1", "number": 404, "from": "D", "to": "N", "codon": "AAC" }
            },
            "design": design()
        }),
    );
    let output = Command::new(env!("CARGO_BIN_EXE_luria"))
        .args(["q5", "design", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stderr, Vec::<u8>::new());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.ends_with("}\n"), "a document and a newline");
    let document: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(document["schema"], "luria.q5-result/1");
    assert_eq!(document["state"], "verified");
    assert_eq!(document["results"][0]["name"], "D404N");
    assert_eq!(document["results"][0]["design"]["forward"]["span"][0], 1199);
}

// A failed run still prints its document, and the status is the report.
#[test]
fn the_binary_prints_a_failed_document_and_exits_4() {
    let dir = workspace("binary-failed");
    let output = Command::new(env!("CARGO_BIN_EXE_luria"))
        .args(["q5", "design", dir.join("absent.json").to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    let document: Value = serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();
    assert_eq!(document["state"], "failed");
    assert_eq!(document["error"]["code"], "request_unreadable");
}

// 5g2 replaces this with the flag's real behaviour.
#[test]
fn write_edited_is_refused_in_this_build() {
    let dir = workspace("write-edited");
    let output = Command::new(env!("CARGO_BIN_EXE_luria"))
        .args([
            "q5",
            "design",
            dir.join("request.json").to_str().unwrap(),
            "--write-edited",
            dir.join("out").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, Vec::<u8>::new());
    let text = String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        text,
        "luria: '--write-edited' is not available in this build yet\n"
    );
}
```

### 4.1 Proving the rules are load-bearing (AGENTS.md)

Break each rule in `run.rs`, run the named test, confirm it fails, restore.

| Rule | Test that must fail when it is broken |
| --- | --- |
| a design failure is a member's, not the run's (return `failed_after` and `for_failure` instead) | `an_edit_that_cannot_be_designed_is_verified_and_exits_7` |
| a clarification designs nothing (run the design path anyway) | `a_residue_edit_with_two_candidate_cdss_asks_and_edits_nothing` |
| members keep request order (reverse them) | `a_variant_set_gives_one_member_per_variant_in_request_order` |
| the status follows the error's category (always `MalformedInput`) | `a_wrong_expected_base_fails_with_biological_validation` |
| an expansion's warnings are carried (pass `vec![]`) | `a_degenerate_expansion_skips_the_wild_type_and_says_so` |

## 5. Do not

- Edit files other than `crates/cli/src/run.rs` (new), `crates/cli/src/lib.rs`
  (one `pub mod` line), `crates/cli/src/main.rs`, `crates/cli/src/exit.rs`
  (the one doc line of §3.4) and `crates/cli/tests/run.rs` (new).
- Change any existing test, with one exception: delete the 5d test
  `q5_design_is_not_wired_yet_and_says_so` from `crates/cli/tests/cli.rs`.
  It pins the stub this contract removes, and
  `the_binary_prints_the_document_and_exits_0` replaces it. (Planner
  amendment, 2026-10-17, after the worker reported the conflict.) If any
  other existing test fails, stop and report it.
- Write any file from `run`: `--write-edited` is 5g2. `run` reads the request
  and the template and writes nothing.
- Re-verify an edit, re-derive a design, round a float, or compute any value
  a core accessor or a 5f view function gives.
- Change `luria-core` or `result.rs`. If one gets in the way, stop and
  report.
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
results, and anything that did not match this contract. If an expected value
disagrees with what the code produces, report the test, the expected value
and the actual one rather than changing the test.
