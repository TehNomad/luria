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
    let view = member_view(
        &id(name),
        &execution.mutation().unwrap(),
        &execution.warnings(),
    );
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
        serde_json::to_string(&operation_warning_view(
            &Q5ExpansionWarning::WildTypeSkipped {
                sequence: "GAT".to_string()
            }
        ))
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
    assert_eq!(
        document.candidates,
        vec!["cds-1".to_string(), "cds-2".to_string()]
    );
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
