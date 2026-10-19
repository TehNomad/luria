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
    fs::copy(
        "../../tests/fixtures/genbank/raw/J01749.1.gb",
        dir.join("J01749.1.gb"),
    )
    .unwrap();
    dir
}

fn prepared(test: &str, request: &str) -> (Prepared, String) {
    let dir = workspace(test);
    let path = dir.join("request.json");
    fs::write(&path, request).unwrap();
    let digest = sha256_hex(&fs::read(&path).unwrap());
    (
        prepare(&path).unwrap_or_else(|failed| panic!("{:?}", failed.failure)),
        digest,
    )
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
    assert_eq!(
        template.sha256,
        "54704740a9f04b79d17147ef95788ad9f0f11e5263843a7ed490924d43b55e3a"
    );
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
        .map(|cds| {
            (
                cds.id.clone(),
                cds.id_qualifier,
                cds.feature_index,
                cds.line,
                cds.transl_table.clone(),
            )
        })
        .collect();
    assert_eq!(
        imported,
        vec![
            ("tet".to_string(), "gene", 10, 260, Some("11".to_string())),
            ("bla".to_string(), "gene", 38, 361, Some("11".to_string())),
        ]
    );
    // C-12: where each imported CDS sits, so a `needs_clarification` document's
    // candidates carry their locations. The values are `tests/fixtures/genbank/pbr322-cds.tsv`'s
    // `start`, `end` and `strand` columns for rows 10 (`tet`) and 38 (`bla`),
    // which Biopython verified.
    let tet = genbank
        .imported_cds
        .iter()
        .find(|cds| cds.id == "tet")
        .unwrap();
    assert_eq!(tet.intervals, vec![(85u64, 1276u64)]);
    assert_eq!(tet.strand, "+");
    let bla = genbank
        .imported_cds
        .iter()
        .find(|cds| cds.id == "bla")
        .unwrap();
    assert_eq!(bla.intervals, vec![(3292u64, 4153u64)]);
    assert_eq!(bla.strand, "-");
    assert_eq!(genbank.unused_features.len(), 48);
    let rop = genbank
        .unused_features
        .iter()
        .find(|feature| feature.index == 26)
        .unwrap();
    assert_eq!(rop.key, "CDS");
    assert_eq!(rop.id, None);
    let refusal = rop.refusal.clone().unwrap();
    assert_eq!(
        (refusal.code, refusal.rule),
        ("unusable_genbank_cds", Some("no_id"))
    );
    assert!(genbank
        .unused_features
        .iter()
        .filter(|feature| feature.key != "CDS")
        .all(|feature| feature.refusal.is_none() && feature.id.is_none()));
}

#[test]
fn a_failed_run_keeps_the_template_provenance() {
    let (ready, _) = prepared("failed-after", SID1);
    let error = luria_cli::result::domain_error_view(
        &luria_core::error::DomainError::EmptyIdentifier,
        None,
    );
    let failed = failed_after(&ready, error);
    assert_eq!(failed.state, "failed");
    assert!(failed.provenance.template.is_some());
    assert!(failed.request.is_some());
    assert_eq!(failed.error.unwrap().path, None);
}
