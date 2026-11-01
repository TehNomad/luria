# Milestone 5f2: targets, mutation results, warnings and clarifications

Status: **Implemented 2026-10-17** by the local worker; accepted on review.
Review note: the §4.2 "clarification has no target" proof only fails when
the literal `target: None` in `clarification_document` is broken. A gate
that needs clarification never carries a target, so breaking the accessor
fails nothing.
Planner: Claude. Parent: `docs/tasks/contracts/milestone-5-cli.md` §3.5 and §5 (5f).
Follows 5f1 (`result.rs`: the document's skeleton, provenance and failures).

## 1. What and why

5f1 declared `CdsView`, `OperationWarning` and `MemberView` as empty
placeholders. This contract defines them, adds the edit-warning and
mutation views, and builds the two remaining documents:
- `needs_clarification`: the brief's fail-closed case, with no edit;
- `verified`: one member per edit, each with its mutation and warnings.

Designs are 5f3. Here every member's `design` and `designFailure` are
`null`.

## 2. Requirements this contract applies

- **Plan §3.5:**
  - `target` is the resolved CDS, or `null`;
  - `clarifications` and `candidates` are filled when `state` is
    `needs_clarification`;
  - each result gives the mutation: kind, recorded coordinates (D040
    order), original and written text, codon and residues for a residue
    edit, and the edited construct's SHA-256 and length;
  - warnings are structured, each with a `code`;
  - intervals are `[start, end]` pairs;
  - the edited sequence text is not in the JSON (5g's `--write-edited`
    writes it).
- **D021:** a residue edit records the codon-selection policy and, for
  `default`, the table it consulted (`CodonSelectionPolicy::source`).
- **D020:** clarification is a result state, not an error. Nothing is
  edited.
- **5d/5f1:** serde structs and enums only, field order as listed, `Option`
  as `null`.

## 3. The views (in `crates/cli/src/result.rs`)

All derive `Clone, Debug, PartialEq, Serialize`, with camelCase field names.
The field order below is the JSON order.

```rust
pub struct CdsView {
    pub id: String,
    pub intervals: Vec<(u64, u64)>,  // [[start, end], ...], D036 order
    pub strand: &'static str,        // "+" or "-"
    pub completeness: String,        // Completeness::name()
    pub genetic_code: String,        // GeneticCode::name()
    pub numbering: String,           // Numbering::name()
    pub length: u64,                 // Cds::total_length()
}

pub struct EditedView { pub id: String, pub sha256: String, pub length: u64 }

#[serde(tag = "kind", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum MutationView {
    Substitution { state: String, interval: (u64, u64), original: String, written: String, coordinates: Vec<u64>, edited: EditedView },
    Residue {
        state: String, cds: String, residue: u64,
        original_residue: String, written_residue: String,
        original_codon: String, written_codon: String,
        codon_policy: String, codon_policy_source: Option<String>,
        coordinates: Vec<u64>, edited: EditedView,
    },
    Deletion { state: String, interval: (u64, u64), original: String, coordinates: Vec<u64>, edited: EditedView },
    Insertion { state: String, point: u64, before: String, after: String, written: String, coordinates: Vec<u64>, edited: EditedView },
    Replacement { state: String, interval: (u64, u64), original: String, written: String, coordinates: Vec<u64>, edited: EditedView },
}

#[serde(tag = "code", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum EditWarningView {
    Frameshift { cds: String, coding_bases_removed: u64, wholly_inside: bool, message: String },
    InsertionFrameshift { cds: String, bases_inserted: u64, message: String },
    ReplacementFrameshift { cds: String, bases_replaced: u64, bases_written: u64, wholly_inside: bool, message: String },
    PrematureStop { cds: String, codon_number: u64, message: String },
}

#[serde(tag = "code", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum OperationWarning {
    WildTypeSkipped { sequence: String, message: String },
    ManyDesigns { count: usize, threshold: usize, message: String },
}

/// 5f3 fills this in. Until then an empty struct.
pub struct DesignView {}

pub struct MemberView {
    pub name: String,
    pub mutation: MutationView,
    pub warnings: Vec<EditWarningView>,
    pub design: Option<DesignView>,
    pub design_failure: Option<ErrorView>,
}
```

Values, every one from a core accessor:
- `state` is `MutationState::name()`. Each view takes the **actual**
  originals (`actual_original`, `actual_deleted`, `actual_before`,
  `actual_after`), which equal the expected ones in any result core
  returns.
- `coordinates` is `MutationResult::target_coordinates()`, as core gives it.
- **Substitution:** `interval` is `(position.start(), position.end())`;
  `written` is `replacement()`.
- **Residue:** `cds` is `cds().value()`; `residue` is
  `position().number()`; residues are `Residue::symbol()`; `written_codon`
  is `selected_codon().text()`; `codon_policy` is `policy().name()`;
  `codon_policy_source` is `policy().source()`.
- **Deletion:** `original` is `actual_deleted()`.
- **Insertion:** `point` is `point().position()`, and `written` is
  `inserted()`.
- **Replacement:** `written` is `replacement()`.
- **`edited`:** the edited `Sequence`'s name, content digest and length.
- **Warning messages** are the core value's `describe()`.

### 3.1 Functions

```rust
pub fn cds_view(cds: &Cds) -> CdsView;
pub fn mutation_view(mutation: &MutationResult) -> MutationView;
pub fn edit_warning_view(warning: &Q5EditWarning) -> EditWarningView;
pub fn operation_warning_view(warning: &Q5ExpansionWarning) -> OperationWarning;
/// design and design_failure are None (5f3 adds them).
pub fn member_view(name: &Identifier, mutation: &MutationResult, warnings: &[Q5EditWarning]) -> MemberView;

/// state "needs_clarification": the gate's clarifications and candidates,
/// target None, no results, no error.
pub fn clarification_document(prepared: &Prepared, gate: &Q5WorkflowResult) -> ResultDocument;

/// state "verified": the target, the members in order, operation warnings.
pub fn verified_document(
    prepared: &Prepared,
    target: Option<&Cds>,
    members: Vec<MemberView>,
    warnings: Vec<OperationWarning>,
) -> ResultDocument;
```

Both documents carry `provenance(Some(prepared.request_sha256.clone()),
Some(&prepared.document), Some(prepared))` and
`request_echo(&prepared.document)`, as `failed_after` does.

All `match`es over core enums are exhaustive, with no `_` arm.

## 4. Tests (planner-written; add verbatim)

### 4.1 Integration test: `crates/cli/tests/result_members.rs`

Every expected value was derived by the planner, by hand and with Python's
`hashlib`. The core functions were not run to get them. The template is
`ATGGCTTGGAATCAGCAGCAGCAGCAGCAG` (30 bases: M A W N Q Q Q Q Q Q), with CDS
`cds-1` over `[0, 30)` on the plus strand:

| Edit | Edited text | Length | SHA-256 | Warning |
| --- | --- | --- | --- | --- |
| residue 4 N→M, default (`AAT`→`ATG`, the only M codon) | `ATGGCTTGGATGCAGCAGCAGCAGCAGCAG` | 30 | `8f6fc03b…c5e0` | none |
| `[3, 4)` G→C (codon 2 `GCT`→`CCT`) | `ATGCCTTGGAATCAGCAGCAGCAGCAGCAG` | 30 | `15c95947…8f74` | none |
| `[8, 9)` G→A (codon 3 `TGG`→`TGA`, a stop) | `ATGGCTTGAAATCAGCAGCAGCAGCAGCAG` | 30 | `40897b50…7f03` | premature stop at codon 3 (D037) |
| delete `[9, 10)` `A` | `ATGGCTTGGATCAGCAGCAGCAGCAGCAG` | 29 | `80509689…b1f8` | frameshift, 1 base, wholly inside |
| insert `GGG` at 12, between `T` and `C` | `ATGGCTTGGAATGGGCAGCAGCAGCAGCAGCAG` | 33 | `23d41897…328a` | none (3 bases keep the frame) |
| replace `[9, 12)` `AAT`→`GG` | `ATGGCTTGGGGCAGCAGCAGCAGCAGCAG` | 29 | `b782c619…6403` | replacement frameshift, 3 → 2 |

SID-1 D404N's edited construct must have the SHA-256 of NEBaseChanger's
output file (`tests/fixtures/fasta/expected.tsv`, `sdmfinalseq.fa`), an
independent source.

```rust
//! Member, target and clarification views, built from real core results
//! and compared with expectations derived by hand.

use luria_cli::prepare::{prepare, Operation, Prepared};
use luria_cli::result::{
    cds_view, clarification_document, member_view, operation_warning_view, render,
    verified_document,
};
use luria_core::cds::{Cds, Completeness, Numbering, ResiduePosition};
use luria_core::identity::Identifier;
use luria_core::interval::{create, create_point};
use luria_core::mutation::{
    CodonSelectionPolicy, NucleotideDeletionIntent, NucleotideInsertionIntent,
    NucleotideMutationIntent, NucleotideReplacementIntent, ResidueMutationIntent,
};
use luria_core::q5::{execute, Q5MutationIntent, Q5SdmRequest};
use luria_core::q5_expand::Q5ExpansionWarning;
use luria_core::sequence::{normalize, Sequence, SequenceTopology};
use luria_core::strand::Strand;
use std::fs;
use std::path::{Path, PathBuf};

const WILD_TYPE: &str = "ATGGCTTGGAATCAGCAGCAGCAGCAGCAG";

fn id(text: &str) -> Identifier {
    Identifier::create(text.to_string()).unwrap()
}

fn template() -> Sequence {
    normalize(id("seq-a"), WILD_TYPE.to_string(), SequenceTopology::Linear).unwrap()
}

fn cds(sequence: &Sequence) -> Cds {
    Cds::create(
        sequence,
        id("cds-1"),
        vec![create(&sequence.reference(), 0, 30).unwrap()],
        Strand::Plus,
        Completeness::Complete,
        "standard".to_string(),
        Numbering::FromCdsStart,
    )
    .unwrap()
}

/// One edit through the workflow, as the member view's compact JSON.
fn member(intent: Q5MutationIntent, name: &str) -> String {
    let sequence = template();
    let request = Q5SdmRequest::create(sequence.clone(), Some(id("cds-1")), intent).unwrap();
    let execution = execute(request, vec![cds(&sequence)], id(name)).unwrap();
    let view = member_view(&id(name), &execution.mutation().unwrap(), &execution.warnings());
    serde_json::to_string(&view).unwrap()
}

#[test]
fn a_residue_edit_gives_its_codon_policy_and_coordinates() {
    let intent = Q5MutationIntent::Residue(
        ResidueMutationIntent::create(
            ResiduePosition::create(id("cds-1"), 4).unwrap(),
            "N".to_string(),
            "M".to_string(),
            CodonSelectionPolicy::Default,
        )
        .unwrap(),
    );
    assert_eq!(
        member(intent, "res-n4m"),
        r#"{"name":"res-n4m","mutation":{"kind":"residue","state":"verified","cds":"cds-1","residue":4,"originalResidue":"N","writtenResidue":"M","originalCodon":"AAT","writtenCodon":"ATG","codonPolicy":"default","codonPolicySource":"Kazusa Codon Usage Database, Escherichia coli W3110 (K-12), species 316407, retrieved 2026-10-14","coordinates":[9,10,11],"edited":{"id":"res-n4m","sha256":"8f6fc03b83312d286d8ae34e341a24c4c6c357b1b917b22587c881efe5d0c5e0","length":30}},"warnings":[],"design":null,"designFailure":null}"#
    );
}

#[test]
fn substitutions_give_their_interval_and_any_premature_stop() {
    let plain = Q5MutationIntent::Nucleotide(
        NucleotideMutationIntent::create(
            template().reference(),
            create(&template().reference(), 3, 4).unwrap(),
            "G".to_string(),
            "C".to_string(),
        )
        .unwrap(),
    );
    assert_eq!(
        member(plain, "sub-g3c"),
        r#"{"name":"sub-g3c","mutation":{"kind":"substitution","state":"verified","interval":[3,4],"original":"G","written":"C","coordinates":[3],"edited":{"id":"sub-g3c","sha256":"15c95947f118c6a7223bafd3c1b07dcc066553baf50419c6e2a20ccb9b908f74","length":30}},"warnings":[],"design":null,"designFailure":null}"#
    );
    let stop = Q5MutationIntent::Nucleotide(
        NucleotideMutationIntent::create(
            template().reference(),
            create(&template().reference(), 8, 9).unwrap(),
            "G".to_string(),
            "A".to_string(),
        )
        .unwrap(),
    );
    assert_eq!(
        member(stop, "sub-stop"),
        r#"{"name":"sub-stop","mutation":{"kind":"substitution","state":"verified","interval":[8,9],"original":"G","written":"A","coordinates":[8],"edited":{"id":"sub-stop","sha256":"40897b503557add570317f2bd59b35300aa53aeb589ddad03fb1fdb5f9b17f03","length":30}},"warnings":[{"code":"premature_stop","cds":"cds-1","codonNumber":3,"message":"the edit puts a stop codon at codon 3 of cds-1, before its last codon"}],"design":null,"designFailure":null}"#
    );
}

#[test]
fn a_deletion_insertion_and_replacement_give_their_views_and_frameshifts() {
    let deletion = Q5MutationIntent::Deletion(
        NucleotideDeletionIntent::create(
            template().reference(),
            create(&template().reference(), 9, 10).unwrap(),
            "A".to_string(),
        )
        .unwrap(),
    );
    assert_eq!(
        member(deletion, "del-a9"),
        r#"{"name":"del-a9","mutation":{"kind":"deletion","state":"verified","interval":[9,10],"original":"A","coordinates":[9],"edited":{"id":"del-a9","sha256":"80509689043c1574eaa9d6c9d88103d3a6d28e3f4d809cde7a9ee3326148b1f8","length":29}},"warnings":[{"code":"frameshift","cds":"cds-1","codingBasesRemoved":1,"whollyInside":true,"message":"deleting 1 coding bases from cds-1 shifts its reading frame"}],"design":null,"designFailure":null}"#
    );
    let insertion = Q5MutationIntent::Insertion(
        NucleotideInsertionIntent::create(
            template().reference(),
            create_point(&template().reference(), 12).unwrap(),
            "T".to_string(),
            "C".to_string(),
            "GGG".to_string(),
        )
        .unwrap(),
    );
    assert_eq!(
        member(insertion, "ins-ggg"),
        r#"{"name":"ins-ggg","mutation":{"kind":"insertion","state":"verified","point":12,"before":"T","after":"C","written":"GGG","coordinates":[],"edited":{"id":"ins-ggg","sha256":"23d41897a0ead7259dbe96d883d9f2b9d7c842b61e4bb123f8429045ca88328a","length":33}},"warnings":[],"design":null,"designFailure":null}"#
    );
    let replacement = Q5MutationIntent::Replacement(
        NucleotideReplacementIntent::create(
            template().reference(),
            create(&template().reference(), 9, 12).unwrap(),
            "AAT".to_string(),
            "GG".to_string(),
        )
        .unwrap(),
    );
    assert_eq!(
        member(replacement, "rep-gg"),
        r#"{"name":"rep-gg","mutation":{"kind":"replacement","state":"verified","interval":[9,12],"original":"AAT","written":"GG","coordinates":[9,10,11],"edited":{"id":"rep-gg","sha256":"b782c619d032c629ec7d77c5ac88135125f69c6725b1ce9a03c26de2ac5b6403","length":29}},"warnings":[{"code":"replacement_frameshift","cds":"cds-1","basesReplaced":3,"basesWritten":2,"whollyInside":true,"message":"replacing 3 bases with 2 in cds-1 shifts its reading frame"}],"design":null,"designFailure":null}"#
    );
}

#[test]
fn a_cds_and_the_expansion_warnings_have_their_views() {
    assert_eq!(
        serde_json::to_string(&cds_view(&cds(&template()))).unwrap(),
        r#"{"id":"cds-1","intervals":[[0,30]],"strand":"+","completeness":"complete","geneticCode":"standard","numbering":"from-cds-start","length":30}"#
    );
    assert_eq!(
        serde_json::to_string(&operation_warning_view(&Q5ExpansionWarning::WildTypeSkipped {
            sequence: "GAT".to_string()
        }))
        .unwrap(),
        r#"{"code":"wild_type_skipped","sequence":"GAT","message":"GAT equals the bases already in the region and was skipped"}"#
    );
    assert_eq!(
        serde_json::to_string(&operation_warning_view(&Q5ExpansionWarning::ManyDesigns {
            count: 65,
            threshold: 64
        }))
        .unwrap(),
        r#"{"code":"many_designs","count":65,"threshold":64,"message":"65 designs, more than 64; every one is still emitted"}"#
    );
}

fn workspace(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("members-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn prepared(dir: &Path, request: &str) -> Prepared {
    let path = dir.join("request.json");
    fs::write(&path, request).unwrap();
    prepare(&path).unwrap_or_else(|failed| panic!("{:?}", failed.failure))
}

// The brief's fail-closed example: "mutate residue 42 to glycine" against a
// template with two CDSs. Nothing is chosen, nothing is edited, and both
// CDSs are offered.
#[test]
fn an_ambiguous_residue_request_gives_a_clarification_and_no_edit() {
    let dir = workspace("clarify");
    fs::write(dir.join("seq-a.fa"), format!(">seq-a\n{}\n", WILD_TYPE)).unwrap();
    let ready = prepared(
        &dir,
        r#"{"schema":"luria.q5-request/1","coordinates":"zero-based-half-open","template":{"id":"seq-a","fasta":"seq-a.fa","topology":"linear","cds":[{"id":"cds-1","intervals":[[0,30]],"strand":"+","completeness":"complete","geneticCode":"standard","numbering":"from-cds-start"},{"id":"cds-2","intervals":[[0,30]],"strand":"-","completeness":"complete","geneticCode":"standard","numbering":"from-cds-start"}]},"edit":{"name":"g42","residue":{"number":42,"from":"A","to":"G","policy":"default"}},"design":{"policy":"luriaNative","style":"q5_polymerase","minAnnealingOverlap":15,"placement":"inline","buffer":"q5_reaction"}}"#,
    );
    let (request, name) = match ready.operation.clone() {
        Operation::Single { request, name } => (*request, name),
        other => panic!("expected a single edit, got {:?}", other),
    };
    let execution = execute(request, ready.annotations.clone(), name).unwrap();
    assert!(execution.needs_clarification());
    assert!(execution.mutation().is_none());
    let document = clarification_document(&ready, &execution.scope());
    assert_eq!(document.state, "needs_clarification");
    assert_eq!(document.experimental_validation, "not_performed");
    assert_eq!(document.candidates, vec!["cds-1".to_string(), "cds-2".to_string()]);
    assert_eq!(document.clarifications.len(), 1);
    assert_eq!(document.target, None);
    assert!(document.results.is_empty());
    assert_eq!(document.error, None);
    assert!(render(&document).contains("\"state\": \"needs_clarification\""));
}

#[test]
fn the_sid1_edit_is_the_construct_nebasechanger_wrote() {
    let dir = workspace("sid1");
    fs::copy(
        "../../tests/fixtures/example-construct/input/sid-1_cDNA.fa",
        dir.join("sid-1_cDNA.fa"),
    )
    .unwrap();
    let ready = prepared(
        &dir,
        r#"{"schema":"luria.q5-request/1","coordinates":"zero-based-half-open","template":{"id":"sid-1","fasta":"sid-1_cDNA.fa","topology":"linear","cds":[{"id":"sid-1","intervals":[[0,3153]],"strand":"+","completeness":"complete","geneticCode":"standard","numbering":"from-cds-start"}]},"scope":"sid-1","edit":{"name":"D404N","residue":{"cds":"sid-1","number":404,"from":"D","to":"N","codon":"AAC"}},"design":{"policy":"luriaNative","style":"q5_polymerase","minAnnealingOverlap":15,"placement":"inline","buffer":"q5_reaction"}}"#,
    );
    let (request, name) = match ready.operation.clone() {
        Operation::Single { request, name } => (*request, name),
        other => panic!("expected a single edit, got {:?}", other),
    };
    let execution = execute(request, ready.annotations.clone(), name.clone()).unwrap();
    let member = member_view(&name, &execution.mutation().unwrap(), &execution.warnings());
    let target = execution.scope().target();
    let document = verified_document(&ready, target.as_ref(), vec![member], Vec::new());
    assert_eq!(document.state, "verified");
    assert_eq!(document.target, Some(cds_view(&ready.annotations[0])));
    let text = serde_json::to_string(&document.results).unwrap();
    assert_eq!(
        text,
        r#"[{"name":"D404N","mutation":{"kind":"residue","state":"verified","cds":"sid-1","residue":404,"originalResidue":"D","writtenResidue":"N","originalCodon":"GAT","writtenCodon":"AAC","codonPolicy":"supplied","codonPolicySource":null,"coordinates":[1209,1210,1211],"edited":{"id":"D404N","sha256":"6ab7d5a6fdd995692ed5cda5079db64c97a70935deb9edfd10fe402628b7ff50","length":3153}},"warnings":[],"design":null,"designFailure":null}]"#
    );
}
```

`cargo fmt` may reflow these; do not change content. If an expected value
differs from what the code produces, stop and report the test, the expected
text and the actual text. Do not change the test. The planner will check
whether the expectation or the code is wrong.

### 4.2 Proving the rules are load-bearing (AGENTS.md)

| Rule | Test that must fail when it is broken |
| --- | --- |
| warnings are carried (drop them) | `substitutions_give_their_interval_and_any_premature_stop` |
| the `default` policy names its table (make the source `None`) | `a_residue_edit_gives_its_codon_policy_and_coordinates` |
| a clarification has no target and no results (fill the target) | `an_ambiguous_residue_request_gives_a_clarification_and_no_edit` |
| `edited` is the edited sequence (use the template instead) | `the_sid1_edit_is_the_construct_nebasechanger_wrote` |

## 5. Do not

- Edit files other than `crates/cli/src/result.rs` and
  `crates/cli/tests/result_members.rs` (new).
- Run a workflow inside `result.rs`. Its functions take core results.
- Compute any value that a core accessor gives, put sequence text in a view,
  or add `_` arms.
- Change `luria-core` or the other CLI modules. If one gets in the way,
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

Report the changed and new files, the exact commands and results, the §4.2
results, and anything that did not match this contract.
