# Milestone 5f3: primer designs and design failures in the result

Status: **Implemented 2026-10-17** by the local worker; accepted on review.
Review note: `primer_view` and `pair_view` are also `pub`, beyond the four
named functions; harmless, left as is. Was: **Contract, 2026-10-17.** Not implemented. Needs 5f2 committed
first.
Planner: Claude. Parent: `docs/tasks/milestone-5-cli.md` §3.5 and §5 (5f).
Follows 5f2 (members, targets, clarifications).

## 1. What and why

5f2 left `DesignView` empty and every member's `design` and
`designFailure` `null`. This contract defines the design view and attaches
it, or the member's design failure, to a member.

## 2. Requirements this contract applies

- **Plan §3.5:** `design` gives the policy, style, algorithm and version;
  both primers 5′→3′, with span, tail, length, annealing Tm with its basis
  and any `substitutedBecause`, whole-primer Tm and quality warnings; and
  the annealing temperature, balance, edit span and pair relationship.
  Floats print in shortest round-trip form and are never rounded.
- **`docs/q5-sdm.md` §12 (owner, 2026-10-14):** a set member that cannot be
  designed carries its own error, and the others are still designed.
- **D021, D028:** the design names its algorithm and version, and states
  what it left unresolved.
- **5d/5f1:** serde structs and enums, field order as listed, `Option` as
  `null`, exhaustive matches, values only from core accessors.

## 3. The views (in `crates/cli/src/result.rs`)

Replace the empty `DesignView` with the following. All derive
`Clone, Debug, PartialEq, Serialize`, camelCase. The field order is the JSON
order.

```rust
pub struct DesignView {
    pub policy: String,                 // request().policy().name()
    pub style: String,                  // request().style().name()
    pub algorithm: AlgorithmRef,        // algorithm_name(), algorithm_version()
    pub min_annealing_overlap: u64,     // request().min_annealing_overlap()
    pub placement: String,              // request().placement().name()
    pub edit_span: (u64, u64),          // edit_span()
    pub forward: PrimerView,
    pub reverse: PrimerView,
    pub annealing_temperature: f64,     // annealing_temperature()
    pub tm_balance_met: bool,           // tm_balance_met()
    pub pair: PairView,
    pub unresolved: Vec<UnresolvedView>,
}

pub struct AlgorithmRef { pub name: &'static str, pub version: &'static str }

pub struct PrimerView {
    pub role: String,                   // PrimerRole::name()
    pub sequence: String,               // 5'->3', Primer::sequence()
    pub length: u64,
    pub span: (u64, u64),               // span()
    pub edited_span: Option<(u64, u64)>,
    pub tail: Option<TailView>,         // tail(): its sequence, and tail_reason()
    pub parts: Vec<PartView>,
    pub annealing: AnnealingView,
    pub tm: TmView,                     // forward_tm() / reverse_tm()
    pub whole_primer_tm: Option<TmView>,// forward_whole_tm() / reverse_whole_tm()
    pub warnings: Vec<PrimerWarningView>, // forward_warnings() / reverse_warnings()
}

pub struct TailView { pub sequence: String, pub reason: String }

pub struct PartView {
    pub role: String,                   // PrimerPartRole::name()
    pub annealing: bool,
    pub sequence: String,
    pub span: Option<(u64, u64)>,
    pub primer_start: u64,              // primer_indices().0
    pub primer_end: u64,                // primer_indices().1
}

pub struct AnnealingView {
    pub sequence: String,               // annealing_sequence()
    pub length: u64,                    // annealing_length()
    pub overlap: u64,                   // annealing_overlap()
    pub spans: Vec<(u64, u64)>,         // annealing_spans()
}

pub struct TmView {
    pub celsius: f64,
    pub basis: String,                  // TmBasis::name()
    pub substituted_because: Option<String>,
    pub enthalpy_kcal: f64,
    pub entropy_cal: f64,
}

pub struct PairView {
    pub deleted_width: u64,
    pub separation: i64,
    pub boundary: Option<u64>,
    pub abuts: bool,
    pub relationship: String,
}

pub struct UnresolvedView { pub input: String, pub effect: String }

#[serde(tag = "code", rename_all = "snake_case", rename_all_fields = "camelCase")]
pub enum PrimerWarningView {
    GcContentOutsideBand { gc_percent: f64, message: String },
    SingleBaseRun { base: String, length: u64, start: u64, message: String },
    DinucleotideRepeat { unit: String, repeats: u64, start: u64, message: String },
    ThreePrimeGcRun { length: u64, message: String },
    LengthOutsideGuideline { length: u64, message: String },
    AboveOrderingLimit { length: u64, message: String },
}
```

Each warning's `message` is its `describe()`. A `SingleBaseRun`'s `base`
is its `char` as a one-letter string. The pair fields come from
`pair()`'s accessors: `deleted_width`, `separation`, `boundary`, `abuts`,
`relationship`.

### 3.1 Functions

```rust
pub fn design_view(design: &PrimerDesign) -> DesignView;
pub fn primer_warning_view(warning: &PrimerWarning) -> PrimerWarningView;
/// The member with its design.
pub fn designed_member(member: MemberView, design: &PrimerDesign) -> MemberView;
/// The member with its design failure, `domain_error_view(error, None)`.
pub fn undesigned_member(member: MemberView, error: &DomainError) -> MemberView;
```

`designed_member` sets `design` and leaves `design_failure` `None`.
`undesigned_member` does the reverse.

## 4. Tests: `crates/cli/tests/result_design.rs` (planner-written; add verbatim)

The expected values come from fixtures that were verified independently:
- **D404N:** `tests/fixtures/q5-primer/native.tsv`, row `native-01`. The
  forward primer is exactly NEBaseChanger's `D404N1_fwd`,
  `cgttgagcaaaacttcggggcgc`
  (`tests/fixtures/example-construct/raw/sdmprimer.txt`). The planner
  derived the reverse primer from its span, as the reverse complement of
  template `[1180, 1199)`. It is the first 19 bases of NEBaseChanger's
  23-base reverse primer.
- **The tailed insertion:** `native-insertion.tsv`, row `ins-14`. Template
  base 1208 is `A` and base 1209 is `G`, read from `sid-1_cDNA.fa`.
- Tm values in the tables are rounded to 2 decimals. The test compares
  `format!("{:.2}", value)` with them, and the JSON itself carries the full
  value.

```rust
//! Design views from real designs, against the native-design fixtures.

use luria_cli::json::{parse_json, Json};
use luria_cli::prepare::{prepare, Operation, Prepared};
use luria_cli::result::{
    design_view, designed_member, member_view, undesigned_member, DesignView,
};
use luria_core::error::{DomainError, NoPrimerDesignPossibleProblem};
use luria_core::primer_design::{NATIVE_DESIGN_ALGORITHM_NAME, NATIVE_DESIGN_ALGORITHM_VERSION};
use luria_core::q5::execute;
use luria_core::q5_design::design_run;
use std::fs;
use std::path::PathBuf;

fn request(edit: &str) -> String {
    format!(
        r#"{{"schema":"luria.q5-request/1","coordinates":"zero-based-half-open","template":{{"id":"sid-1","fasta":"sid-1_cDNA.fa","topology":"linear","cds":[{{"id":"sid-1","intervals":[[0,3153]],"strand":"+","completeness":"complete","geneticCode":"standard","numbering":"from-cds-start"}}]}},"scope":"sid-1","edit":{},"design":{{"policy":"luriaNative","style":"q5_polymerase","minAnnealingOverlap":15,"placement":"inline","buffer":"q5_reaction"}}}}"#,
        edit
    )
}

fn designed(test: &str, edit: &str) -> (Prepared, DesignView) {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("design-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::copy(
        "../../tests/fixtures/example-construct/input/sid-1_cDNA.fa",
        dir.join("sid-1_cDNA.fa"),
    )
    .unwrap();
    let path = dir.join("request.json");
    fs::write(&path, request(edit)).unwrap();
    let ready = prepare(&path).unwrap_or_else(|failed| panic!("{:?}", failed.failure));
    let (single, name) = match ready.operation.clone() {
        Operation::Single { request, name } => (*request, name),
        other => panic!("expected a single edit, got {:?}", other),
    };
    let run = execute(single, ready.annotations.clone(), name).unwrap();
    let designed = design_run(run, ready.design.clone()).unwrap();
    let view = design_view(&designed.design().unwrap());
    (ready, view)
}

fn two(value: f64) -> String {
    format!("{:.2}", value)
}

fn keys(text: &str) -> Vec<String> {
    match parse_json(text).unwrap() {
        Json::Object(members) => members.into_iter().map(|(key, _)| key).collect(),
        other => panic!("expected an object, got {:?}", other),
    }
}

#[test]
fn the_d404n_design_is_native_01() {
    let (_, view) = designed(
        "d404n",
        r#"{"name":"D404N","residue":{"cds":"sid-1","number":404,"from":"D","to":"N","codon":"AAC"}}"#,
    );
    assert_eq!(view.policy, "luriaNative");
    assert_eq!(view.style, "q5_polymerase");
    assert_eq!(view.algorithm.name, NATIVE_DESIGN_ALGORITHM_NAME);
    assert_eq!(view.algorithm.version, NATIVE_DESIGN_ALGORITHM_VERSION);
    assert_eq!(view.min_annealing_overlap, 15);
    assert_eq!(view.placement, "inline");
    assert_eq!(view.edit_span, (1209, 1212));
    assert_eq!(view.forward.role, "forward");
    assert_eq!(view.forward.sequence, "CGTTGAGCAAAACTTCGGGGCGC");
    assert_eq!(view.forward.length, 23);
    assert_eq!(view.forward.span, (1199, 1222));
    assert_eq!(view.forward.tail, None);
    assert_eq!(two(view.forward.tm.celsius), "67.72");
    assert_eq!(view.forward.tm.basis, "mismatched_wild_type_site");
    assert_eq!(view.reverse.role, "reverse");
    assert_eq!(view.reverse.sequence, "AGCGGATTGAGCAGTTTTC");
    assert_eq!(view.reverse.length, 19);
    assert_eq!(view.reverse.span, (1180, 1199));
    assert_eq!(view.reverse.tail, None);
    assert_eq!(two(view.reverse.tm.celsius), "62.90");
    assert_eq!(two(view.annealing_temperature), "65.90");
    assert!(view.tm_balance_met);
}

#[test]
fn the_tailed_insertion_design_is_ins_14() {
    let (_, view) = designed(
        "ins14",
        r#"{"name":"ins-gat","insertion":{"point":1209,"before":"A","after":"G","insert":"GAT"}}"#,
    );
    assert_eq!(view.forward.sequence, "GATGATTTCGGGGCGCAC");
    assert_eq!(view.forward.annealing.spans, vec![(1209, 1224)]);
    assert_eq!(view.forward.annealing.sequence, "GATTTCGGGGCGCAC");
    assert_eq!(view.forward.tail.clone().unwrap().sequence, "GAT");
    assert_eq!(two(view.forward.tm.celsius), "64.31");
    assert_eq!(two(view.forward.whole_primer_tm.clone().unwrap().celsius), "67.15");
    assert_eq!(view.reverse.sequence, "TTGCTCAACGAGCGG");
    assert_eq!(view.reverse.annealing.spans, vec![(1194, 1209)]);
    assert_eq!(view.reverse.tail, None);
    assert_eq!(two(view.reverse.tm.celsius), "62.38");
    assert_eq!(two(view.annealing_temperature), "65.38");
    assert!(view.tm_balance_met);
}

// The field order is the contract's (§3): checked with luria's own JSON
// reader, which keeps members in document order.
#[test]
fn the_design_view_keeps_its_field_order() {
    let (_, view) = designed(
        "order",
        r#"{"name":"D404N","residue":{"cds":"sid-1","number":404,"from":"D","to":"N","codon":"AAC"}}"#,
    );
    let text = serde_json::to_string(&view).unwrap();
    assert_eq!(
        keys(&text),
        vec![
            "policy", "style", "algorithm", "minAnnealingOverlap", "placement", "editSpan",
            "forward", "reverse", "annealingTemperature", "tmBalanceMet", "pair", "unresolved",
        ]
    );
    let forward = serde_json::to_string(&view.forward).unwrap();
    assert_eq!(
        keys(&forward),
        vec![
            "role", "sequence", "length", "span", "editedSpan", "tail", "parts", "annealing",
            "tm", "wholePrimerTm", "warnings",
        ]
    );
    let tm = serde_json::to_string(&view.forward.tm).unwrap();
    assert_eq!(
        keys(&tm),
        vec!["celsius", "basis", "substitutedBecause", "enthalpyKcal", "entropyCal"]
    );
    let pair = serde_json::to_string(&view.pair).unwrap();
    assert_eq!(keys(&pair), vec!["deletedWidth", "separation", "boundary", "abuts", "relationship"]);
}

#[test]
fn a_member_carries_its_design_or_its_failure_not_both() {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("design-member");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::copy(
        "../../tests/fixtures/example-construct/input/sid-1_cDNA.fa",
        dir.join("sid-1_cDNA.fa"),
    )
    .unwrap();
    let path = dir.join("request.json");
    fs::write(
        &path,
        request(r#"{"name":"D404N","residue":{"cds":"sid-1","number":404,"from":"D","to":"N","codon":"AAC"}}"#),
    )
    .unwrap();
    let ready = prepare(&path).unwrap_or_else(|failed| panic!("{:?}", failed.failure));
    let (single, name) = match ready.operation.clone() {
        Operation::Single { request, name } => (*request, name),
        other => panic!("expected a single edit, got {:?}", other),
    };
    let run = execute(single, ready.annotations.clone(), name.clone()).unwrap();
    let mutation = run.mutation().unwrap();
    let warnings = run.warnings();
    let design = design_run(run, ready.design.clone()).unwrap().design().unwrap();

    let with_design = designed_member(member_view(&name, &mutation, &warnings), &design);
    assert_eq!(with_design.design, Some(design_view(&design)));
    assert_eq!(with_design.design_failure, None);

    let error = DomainError::NoPrimerDesignPossible(NoPrimerDesignPossibleProblem {
        template: "sid-1".to_string(),
        template_length: 3153,
        smallest_pair: 4000,
    });
    let without = undesigned_member(member_view(&name, &mutation, &warnings), &error);
    assert_eq!(without.design, None);
    let failure = without.design_failure.unwrap();
    assert_eq!(failure.code, "no_primer_design_possible");
    assert_eq!(failure.category, "biological_validation");
    assert_eq!(failure.path, None);
}
```

`cargo fmt` may reflow these; do not change content. If an expected value
differs from what the code produces, stop and report the test, the expected
value and the actual one. Do not change the test.

### 4.1 Proving the rules are load-bearing (AGENTS.md)

| Rule | Test that must fail when it is broken |
| --- | --- |
| the reverse primer's view uses `reverse_tm()` (use `forward_tm()`) | `the_d404n_design_is_native_01` |
| the tail is reported (always `None`) | `the_tailed_insertion_design_is_ins_14` |
| the whole-primer Tm is reported (always `None`) | `the_tailed_insertion_design_is_ins_14` |
| `designed_member` leaves `design_failure` `None` (set both) | `a_member_carries_its_design_or_its_failure_not_both` |

## 5. Do not

- Edit files other than `crates/cli/src/result.rs` and
  `crates/cli/tests/result_design.rs` (new).
- Run a design inside `result.rs`, round any float, or compute any value a
  core accessor gives.
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

Report the changed and new files, the exact commands and results, the §4.1
results, and anything that did not match this contract.
