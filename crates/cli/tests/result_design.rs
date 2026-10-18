//! Design views from real designs, against the native-design fixtures.

use luria_cli::json::{parse_json, Json};
use luria_cli::prepare::{prepare, Operation, Prepared};
use luria_cli::result::{design_view, designed_member, member_view, undesigned_member, DesignView};
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
    assert_eq!(two(view.annealing_temperature), "64.00");
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
    assert_eq!(
        two(view.forward.whole_primer_tm.clone().unwrap().celsius),
        "67.15"
    );
    assert_eq!(view.reverse.sequence, "TTGCTCAACGAGCGG");
    assert_eq!(view.reverse.annealing.spans, vec![(1194, 1209)]);
    assert_eq!(view.reverse.tail, None);
    assert_eq!(two(view.reverse.tm.celsius), "62.38");
    assert_eq!(two(view.annealing_temperature), "63.00");
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
            "policy",
            "style",
            "algorithm",
            "minAnnealingOverlap",
            "placement",
            "editSpan",
            "forward",
            "reverse",
            "annealingTemperature",
            "tmBalanceMet",
            "pair",
            "unresolved",
        ]
    );
    let forward = serde_json::to_string(&view.forward).unwrap();
    assert_eq!(
        keys(&forward),
        vec![
            "role",
            "sequence",
            "length",
            "span",
            "editedSpan",
            "tail",
            "parts",
            "annealing",
            "tm",
            "wholePrimerTm",
            "warnings",
        ]
    );
    let tm = serde_json::to_string(&view.forward.tm).unwrap();
    assert_eq!(
        keys(&tm),
        vec![
            "celsius",
            "basis",
            "substitutedBecause",
            "enthalpyKcal",
            "entropyCal"
        ]
    );
    let pair = serde_json::to_string(&view.pair).unwrap();
    assert_eq!(
        keys(&pair),
        vec![
            "deletedWidth",
            "separation",
            "boundary",
            "abuts",
            "relationship"
        ]
    );
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
    let design = design_run(run, ready.design.clone())
        .unwrap()
        .design()
        .unwrap();

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
