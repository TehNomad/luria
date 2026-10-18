//! Plan §6: run the executable, parse its JSON, and compare each
//! scientific field with a fixture verified before luria could produce it.
//!
//! The expectations come from `tests/fixtures/q5-primer/native.tsv`,
//! `native-deletion.tsv`, `native-insertion.tsv` and
//! `native-replacement.tsv`, whose generating scripts are independent of
//! `crates/` (see that directory's README), and from
//! `tests/fixtures/fasta/expected.tsv` and
//! `tests/fixtures/genbank/pbr322-record.tsv` for the templates.

use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The SID-1 cDNA's digest, from `tests/fixtures/fasta/expected.tsv`.
const SID1_DIGEST: &str = "54704740a9f04b79d17147ef95788ad9f0f11e5263843a7ed490924d43b55e3a";

/// pBR322's digest, from `tests/fixtures/genbank/pbr322-record.tsv`.
const PBR322_DIGEST: &str = "536042a7485bdabdf49c4d21a306213a098d77d19bdba57a1c6adc76ce2b8372";

fn workspace(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("e2e-{}", test));
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

/// Run the executable on `request` and return its exit code and stdout.
fn luria(dir: &Path, request: &Value) -> (i32, String) {
    let path = dir.join("request.json");
    fs::write(&path, request.to_string()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_luria"))
        .args(["q5", "design", path.to_str().unwrap()])
        .output()
        .unwrap();
    (
        output.status.code().unwrap(),
        String::from_utf8(output.stdout).unwrap(),
    )
}

/// Run the executable and return the document of a run that must exit with
/// `expected`.
fn document_at(test: &str, request: &Value, expected: i32) -> Value {
    let dir = workspace(test);
    let (code, text) = luria(&dir, request);
    assert_eq!(code, expected, "{}: {}", test, text);
    serde_json::from_str(&text).unwrap()
}

/// Run the executable and return the document of a run that must succeed
/// with every member designed.
fn document(test: &str, request: &Value) -> Value {
    document_at(test, request, 0)
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
        "sha256": SID1_DIGEST,
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

/// A one-edit request on SID-1 at the given bound. `edit` is the one kind
/// member, such as `{"deletion": {...}}`; the name is added to it.
fn request(name: &str, edit: Value, bound: u64) -> Value {
    let mut spec = design();
    spec["minAnnealingOverlap"] = json!(bound);
    let mut edit_object = json!({ "name": name });
    for (key, value) in edit.as_object().expect("an edit object") {
        edit_object[key] = value.clone();
    }
    json!({
        "schema": "luria.q5-request/1",
        "coordinates": "zero-based-half-open",
        "template": sid1_template(),
        "scope": "sid-1",
        "edit": edit_object,
        "design": spec
    })
}

/// The one member's design.
fn design_of(document: &Value) -> &Value {
    assert_eq!(document["state"], "verified");
    assert_eq!(document["results"].as_array().unwrap().len(), 1);
    &document["results"][0]["design"]
}

/// A `[start, end]` pair as a tuple.
fn span(value: &Value) -> (u64, u64) {
    (
        value[0].as_u64().expect("a start"),
        value[1].as_u64().expect("an end"),
    )
}

/// A temperature at the two decimals the tables record.
fn two(value: &Value) -> String {
    format!("{:.2}", value.as_f64().expect("a temperature"))
}

fn text(value: &Value) -> &str {
    value.as_str().expect("a string")
}

/// One `native.tsv` row, as this test reads it: the case, the bound, the
/// forward and reverse spans, the reverse primer's length, and the forward,
/// reverse and annealing temperatures at two decimals.
type SubstitutionRow = (
    &'static str,
    u64,
    (u64, u64),
    (u64, u64),
    u64,
    &'static str,
    &'static str,
    &'static str,
);

// native.tsv rows native-01, native-02 and native-03: the same edit at
// three bounds, so the pair grows with the bound.
#[test]
fn the_native_substitution_rows_match_the_table() {
    let edit = json!({ "replacement": { "interval": [1209, 1212], "from": "GAT", "to": "AAC" } });
    let rows: [SubstitutionRow; 3] = [
        (
            "native-01",
            15,
            (1199, 1222),
            (1180, 1199),
            19,
            "67.72",
            "62.90",
            "64.00",
        ),
        (
            "native-02",
            24,
            (1199, 1226),
            (1175, 1199),
            24,
            "72.08",
            "71.19",
            "72.00",
        ),
        (
            "native-03",
            25,
            (1199, 1227),
            (1174, 1199),
            25,
            "73.02",
            "71.66",
            "72.00",
        ),
    ];
    for (case, bound, forward, reverse, reverse_length, forward_tm, reverse_tm, annealing) in rows {
        let document = document(case, &request(case, edit.clone(), bound));
        let view = design_of(&document);
        assert_eq!(span(&view["forward"]["span"]), forward, "{} forward", case);
        assert_eq!(span(&view["reverse"]["span"]), reverse, "{} reverse", case);
        assert_eq!(
            view["reverse"]["length"].as_u64().unwrap(),
            reverse_length,
            "{} reverse length",
            case
        );
        assert_eq!(
            two(&view["forward"]["tm"]["celsius"]),
            forward_tm,
            "{} forward Tm",
            case
        );
        // Every one of these rows reports the mismatched-site basis.
        assert_eq!(
            text(&view["forward"]["tm"]["basis"]),
            "mismatched_wild_type_site",
            "{} basis",
            case
        );
        assert_eq!(
            two(&view["reverse"]["tm"]["celsius"]),
            reverse_tm,
            "{} reverse Tm",
            case
        );
        assert_eq!(
            two(&view["annealingTemperature"]),
            annealing,
            "{} annealing",
            case
        );
        assert_eq!(view["tmBalanceMet"], true, "{} balance", case);
        assert_eq!(
            view["minAnnealingOverlap"].as_u64().unwrap(),
            bound,
            "{} bound",
            case
        );
        assert_eq!(view["placement"], "inline", "{} placement", case);
        assert_eq!(view["policy"], "luriaNative", "{} policy", case);
    }
}

// native.tsv row native-04: the edit's two changed bases are adjacent, so
// the forward Tm falls back to the matched basis.
#[test]
fn the_adjacent_change_row_falls_back_to_the_matched_basis() {
    let document = document(
        "native-04",
        &request(
            "native-04",
            json!({ "replacement": { "interval": [1503, 1506], "from": "CTG", "to": "GAG" } }),
            15,
        ),
    );
    let view = design_of(&document);
    assert_eq!(span(&view["forward"]["span"]), (1493, 1516));
    assert_eq!(span(&view["reverse"]["span"]), (1469, 1493));
    assert_eq!(view["reverse"]["length"].as_u64().unwrap(), 24);
    assert_eq!(two(&view["forward"]["tm"]["celsius"]), "63.36");
    assert_eq!(text(&view["forward"]["tm"]["basis"]), "matched_complement");
    assert_eq!(two(&view["reverse"]["tm"]["celsius"]), "59.45");
    assert_eq!(two(&view["annealingTemperature"]), "60.00");
    assert_eq!(view["tmBalanceMet"], true);
}

// native-deletion.tsv row del-11: the deleted codon's own primers.
#[test]
fn the_deletion_row_matches_the_table() {
    let document = document(
        "del-11",
        &request(
            "del-11",
            json!({ "deletion": { "interval": [1209, 1212], "from": "GAT" } }),
            15,
        ),
    );
    let view = design_of(&document);
    assert_eq!(span(&view["forward"]["span"]), (1212, 1227));
    assert_eq!(span(&view["reverse"]["span"]), (1190, 1209));
    assert_eq!(text(&view["forward"]["sequence"]), "TTCGGGGCGCACAGA");
    assert_eq!(text(&view["reverse"]["sequence"]), "TTGCTCAACGAGCGGATTG");
    assert_eq!(two(&view["forward"]["tm"]["celsius"]), "67.61");
    assert_eq!(two(&view["reverse"]["tm"]["celsius"]), "65.77");
    assert_eq!(two(&view["annealingTemperature"]), "67.00");
    assert_eq!(view["tmBalanceMet"], true);
    // A deletion carries no tail on either primer.
    assert_eq!(view["forward"]["tail"], Value::Null);
    assert_eq!(view["reverse"]["tail"], Value::Null);
    // The construct is three bases shorter.
    assert_eq!(
        document["results"][0]["mutation"]["edited"]["length"]
            .as_u64()
            .unwrap(),
        3150
    );
}

// native-insertion.tsv row ins-14: the forward primer carries the inserted
// bases as its 5' tail, so it has a whole-primer Tm as well.
#[test]
fn the_insertion_row_matches_the_table() {
    let document = document(
        "ins-14",
        &request(
            "ins-14",
            json!({
                "insertion": { "point": 1209, "before": "A", "after": "G", "insert": "GAT" }
            }),
            15,
        ),
    );
    let view = design_of(&document);
    assert_eq!(
        span(&view["forward"]["annealing"]["spans"][0]),
        (1209, 1224)
    );
    assert_eq!(
        span(&view["reverse"]["annealing"]["spans"][0]),
        (1194, 1209)
    );
    assert_eq!(text(&view["forward"]["sequence"]), "GATGATTTCGGGGCGCAC");
    assert_eq!(text(&view["reverse"]["sequence"]), "TTGCTCAACGAGCGG");
    assert_eq!(text(&view["forward"]["tail"]["sequence"]), "GAT");
    assert_eq!(view["reverse"]["tail"], Value::Null);
    assert_eq!(two(&view["forward"]["tm"]["celsius"]), "64.31");
    assert_eq!(two(&view["reverse"]["tm"]["celsius"]), "62.38");
    assert_eq!(two(&view["forward"]["wholePrimerTm"]["celsius"]), "67.15");
    assert_eq!(two(&view["reverse"]["wholePrimerTm"]["celsius"]), "62.38");
    assert_eq!(two(&view["annealingTemperature"]), "63.00");
    assert_eq!(view["tmBalanceMet"], true);
    assert_eq!(
        document["results"][0]["mutation"]["edited"]["length"]
            .as_u64()
            .unwrap(),
        3156
    );
}

// native-replacement.tsv rows rep-14 and rep-15: the inline path, and the
// tail path when the written text is longer than the region.
#[test]
fn the_replacement_rows_match_the_table() {
    let inline = document(
        "rep-14",
        &request(
            "rep-14",
            json!({ "replacement": { "interval": [1209, 1212], "from": "GAT", "to": "AAC" } }),
            15,
        ),
    );
    let view = design_of(&inline);
    assert_eq!(span(&view["forward"]["span"]), (1199, 1222));
    assert_eq!(span(&view["reverse"]["span"]), (1180, 1199));
    assert_eq!(
        text(&view["forward"]["sequence"]),
        "CGTTGAGCAAAACTTCGGGGCGC"
    );
    assert_eq!(text(&view["reverse"]["sequence"]), "AGCGGATTGAGCAGTTTTC");
    assert_eq!(two(&view["forward"]["tm"]["celsius"]), "67.72");
    assert_eq!(
        text(&view["forward"]["tm"]["basis"]),
        "mismatched_wild_type_site"
    );
    assert_eq!(two(&view["reverse"]["tm"]["celsius"]), "62.90");
    assert_eq!(two(&view["annealingTemperature"]), "64.00");

    let tailed = document(
        "rep-15",
        &request(
            "rep-15",
            json!({
                "replacement": { "interval": [1209, 1212], "from": "GAT", "to": "AACCCGGG" }
            }),
            15,
        ),
    );
    let view = design_of(&tailed);
    assert_eq!(
        span(&view["forward"]["annealing"]["spans"][0]),
        (1212, 1227)
    );
    assert_eq!(
        span(&view["reverse"]["annealing"]["spans"][0]),
        (1190, 1209)
    );
    assert_eq!(text(&view["forward"]["tail"]["sequence"]), "CGGG");
    assert_eq!(text(&view["reverse"]["tail"]["sequence"]), "GGTT");
    assert_eq!(text(&view["forward"]["sequence"]), "CGGGTTCGGGGCGCACAGA");
    assert_eq!(
        text(&view["reverse"]["sequence"]),
        "GGTTTTGCTCAACGAGCGGATTG"
    );
    assert_eq!(two(&view["forward"]["tm"]["celsius"]), "67.61");
    assert_eq!(text(&view["forward"]["tm"]["basis"]), "matched_complement");
    assert_eq!(two(&view["reverse"]["tm"]["celsius"]), "65.77");
    assert_eq!(two(&view["forward"]["wholePrimerTm"]["celsius"]), "76.65");
    assert_eq!(two(&view["reverse"]["wholePrimerTm"]["celsius"]), "69.32");
    assert_eq!(two(&view["annealingTemperature"]), "67.00");
    assert_eq!(view["tmBalanceMet"], true);
}

// §3: the same edit at the same bound appears in two tables, so the two
// documents' designs must be identical.
#[test]
fn the_same_edit_in_two_tables_gives_one_design() {
    let edit = json!({ "replacement": { "interval": [1209, 1212], "from": "GAT", "to": "AAC" } });
    let first = document("same-a", &request("e", edit.clone(), 15));
    let second = document("same-b", &request("e", edit, 15));
    assert_eq!(
        first["results"][0]["design"],
        second["results"][0]["design"]
    );
}

// The brief's fail-closed example, through the executable: a residue number
// with no CDS named, on a template annotated with two. D005: no sequence
// calculation is performed, and the run asks instead.
#[test]
fn the_fail_closed_example_asks_and_edits_nothing() {
    let dir = workspace("fail-closed");
    let (code, out) = luria(
        &dir,
        &json!({
            "schema": "luria.q5-request/1",
            "coordinates": "zero-based-half-open",
            "template": {
                "id": "pbr322",
                "genbank": "J01749.1.gb",
                "sha256": PBR322_DIGEST,
                "cds": ["tet", "bla"]
            },
            "edit": {
                "name": "G42",
                "residue": { "number": 42, "from": "A", "to": "G", "policy": "default" }
            },
            "design": design()
        }),
    );
    assert_eq!(code, 3, "{}", out);
    let document: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(document["state"], "needs_clarification");
    assert_eq!(document["experimentalValidation"], "not_performed");
    // Both CDSs are offered, and nothing was chosen or edited.
    let candidates: Vec<&str> = document["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    assert_eq!(candidates, vec!["tet", "bla"]);
    assert!(!document["clarifications"].as_array().unwrap().is_empty());
    assert_eq!(document["results"].as_array().unwrap().len(), 0);
    assert_eq!(document["target"], Value::Null);
    assert_eq!(document["error"], Value::Null);
    // The clarification names the residue and both CDSs.
    let asked = document["clarifications"][0].as_str().unwrap();
    assert!(asked.contains("42"), "{}", asked);
    assert!(asked.contains("tet") && asked.contains("bla"), "{}", asked);
}

// G9, as amended on 2026-10-17: the same template in either format gives
// the same result. pBR322 is circular, and D029 scopes `luriaNative` to a
// linear template, so neither run designs a pair: each reports the same
// edit, verified, with the same `designFailure`, at exit 7 (G7). What is
// proven is that the format changes nothing but the provenance.
//
// The FASTA is written here from the GenBank record's own sequence, and its
// digest is checked against the independently verified one, so the two
// requests are known to carry the same artifact.
#[test]
fn a_fasta_and_a_genbank_template_give_the_same_result() {
    let dir = workspace("formats");

    // The FASTA file, from the GenBank record's sequence.
    let record = luria_core::genbank::read_genbank(
        luria_core::identity::Identifier::create("pbr322".to_string()).unwrap(),
        fs::read_to_string(dir.join("J01749.1.gb")).unwrap(),
    )
    .unwrap();
    let sequence = record.sequence();
    assert_eq!(
        sequence.content_hash().hex_digest(),
        PBR322_DIGEST,
        "the FASTA carries the verified pBR322 sequence"
    );
    let name = luria_core::identity::Identifier::create("pbr322".to_string()).unwrap();
    fs::write(
        dir.join("pbr322.fa"),
        luria_core::fasta::render_fasta(&name, &sequence),
    )
    .unwrap();

    // tet, from `tests/fixtures/genbank/pbr322-cds.tsv`: [85, 1276), plus
    // strand, an ATG start and 396 residues.
    let edit = json!({
        "name": "T99C",
        "substitution": { "interval": [99, 100], "from": "T", "to": "C" }
    });
    let from_genbank = document_at(
        "formats-genbank",
        &json!({
            "schema": "luria.q5-request/1",
            "coordinates": "zero-based-half-open",
            "template": {
                "id": "pbr322",
                "genbank": "J01749.1.gb",
                "sha256": PBR322_DIGEST,
                "cds": ["tet"]
            },
            "scope": "tet",
            "edit": edit.clone(),
            "design": design()
        }),
        7,
    );

    let dir_fasta = workspace("formats-fasta");
    fs::write(
        dir_fasta.join("pbr322.fa"),
        fs::read_to_string(dir.join("pbr322.fa")).unwrap(),
    )
    .unwrap();
    let path = dir_fasta.join("request.json");
    fs::write(
        &path,
        json!({
            "schema": "luria.q5-request/1",
            "coordinates": "zero-based-half-open",
            "template": {
                "id": "pbr322",
                "fasta": "pbr322.fa",
                "topology": "circular",
                "sha256": PBR322_DIGEST,
                "cds": [{
                    "id": "tet",
                    "intervals": [[85, 1276]],
                    "strand": "+",
                    "completeness": "complete",
                    "geneticCode": "standard",
                    "numbering": "from-cds-start"
                }]
            },
            "scope": "tet",
            "edit": edit,
            "design": design()
        })
        .to_string(),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_luria"))
        .args(["q5", "design", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(7));
    let from_fasta: Value =
        serde_json::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap();

    // The science is the same: the same mutation, the same outcome per
    // member, the same warnings, the same resolved target.
    assert_eq!(from_fasta["state"], from_genbank["state"]);
    assert_eq!(from_fasta["results"], from_genbank["results"]);
    assert_eq!(from_fasta["target"], from_genbank["target"]);
    assert_eq!(from_fasta["warnings"], from_genbank["warnings"]);

    // Both are verified edits that no pair can be designed for, because
    // D029's native scope is a linear template and pBR322 is circular. The
    // edit itself is still reported, with its own construct (G7).
    assert_eq!(from_genbank["state"], "verified");
    let member = &from_genbank["results"][0];
    assert_eq!(member["design"], Value::Null);
    assert_eq!(
        member["designFailure"]["code"],
        "unsupported_primer_design_scope"
    );
    assert_eq!(member["designFailure"]["category"], "unsupported");
    assert!(
        text(&member["designFailure"]["message"]).contains("circular"),
        "{}",
        member["designFailure"]["message"]
    );
    // The edit was applied, so the construct is one base changed from the
    // 4361-base template.
    assert_eq!(
        member["mutation"]["edited"]["length"].as_u64().unwrap(),
        4361
    );
    assert_ne!(member["mutation"]["edited"]["sha256"], json!(PBR322_DIGEST));

    // The provenance differs, and says which format it was.
    assert_eq!(
        from_fasta["provenance"]["template"]["source"]["format"],
        "fasta"
    );
    assert_eq!(
        from_genbank["provenance"]["template"]["source"]["format"],
        "genbank"
    );
    assert_ne!(
        from_fasta["provenance"]["template"],
        from_genbank["provenance"]["template"]
    );
    // Only the GenBank request reports imported CDSs and unused features.
    assert_ne!(
        from_genbank["provenance"]["template"]["genbank"],
        Value::Null
    );
    assert_eq!(from_fasta["provenance"]["template"]["genbank"], Value::Null);
    // Both carry the same artifact, by digest.
    assert_eq!(
        from_fasta["provenance"]["template"]["sha256"],
        json!(PBR322_DIGEST)
    );
    assert_eq!(
        from_genbank["provenance"]["template"]["sha256"],
        json!(PBR322_DIGEST)
    );
}

// D012: the same input twice gives byte-identical output, for a verified
// run, a clarification and a failure alike.
#[test]
fn every_case_is_byte_identical_when_run_twice() {
    let cases: [(&str, Value, i32); 3] = [
        (
            "twice-verified",
            request(
                "D404N",
                json!({
                    "replacement": { "interval": [1209, 1212], "from": "GAT", "to": "AAC" }
                }),
                15,
            ),
            0,
        ),
        (
            "twice-clarify",
            json!({
                "schema": "luria.q5-request/1",
                "coordinates": "zero-based-half-open",
                "template": {
                    "id": "pbr322",
                    "genbank": "J01749.1.gb",
                    "cds": ["tet", "bla"]
                },
                "edit": {
                    "name": "G42",
                    "residue": { "number": 42, "from": "A", "to": "G", "policy": "default" }
                },
                "design": design()
            }),
            3,
        ),
        (
            "twice-failed",
            request(
                "wrong",
                json!({
                    "substitution": { "interval": [1209, 1210], "from": "A", "to": "C" }
                }),
                15,
            ),
            5,
        ),
    ];
    for (case, spec, expected) in cases {
        let dir = workspace(case);
        let (first_code, first) = luria(&dir, &spec);
        let (second_code, second) = luria(&dir, &spec);
        assert_eq!(first_code, expected, "{}: {}", case, first);
        assert_eq!(second_code, expected, "{}", case);
        assert_eq!(first, second, "{} is not deterministic", case);
        assert!(!first.is_empty());
    }
}
