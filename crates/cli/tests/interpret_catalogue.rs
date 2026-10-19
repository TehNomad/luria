//! The CDS catalogue of milestone 6 (contract 6c) against the GenBank
//! fixtures. Every expected value comes from a fixture table derived
//! independently of luria (`tests/fixtures/genbank/README.md`): the linear
//! record's tables, `synthetic-unused.tsv`, `synthetic-expected.tsv` and the
//! pBR322 tables read by Biopython.

use luria_cli::interpret::catalogue::catalogue;
use luria_cli::interpret::catalogue::CatalogueEntry;
use luria_cli::interpret::catalogue::Usability;
use luria_core::genbank::read_genbank;
use luria_core::genbank::GenBankRecord;
use luria_core::identity::Identifier;
use luria_core::strand::Strand;

const GENBANK: &str = "../../tests/fixtures/genbank/";

fn record(file: &str) -> GenBankRecord {
    let path: String = format!("{}{}", GENBANK, file);
    let text: String = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
    let name: Identifier = Identifier::create("template".to_string()).expect("a valid identifier");
    read_genbank(name, text).unwrap_or_else(|error| panic!("{}: {}", file, error.message()))
}

fn entries(file: &str) -> Vec<CatalogueEntry> {
    catalogue(&record(file)).unwrap_or_else(|error| panic!("{}: {}", file, error.message()))
}

/// The rows of a fixture table past its header, refusing a header it does
/// not have.
fn rows(file: &str, header: &str) -> Vec<Vec<String>> {
    let path: String = format!("{}{}", GENBANK, file);
    let text: String = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
    let lines: Vec<&str> = text.lines().collect();
    assert!(
        !lines.is_empty() && lines[0] == header,
        "{}: first row {:?}, expected {:?}",
        file,
        lines.first(),
        header
    );
    lines[1..]
        .iter()
        .map(|line| line.split('\t').map(|field| field.to_string()).collect())
        .collect()
}

/// A usability as `synthetic-unused.tsv` writes a refusal: `code:rule`,
/// `code`, or `-` for a CDS luria can use.
fn refusal_text(usability: &Usability) -> String {
    match usability {
        Usability::Usable { .. } => "-".to_string(),
        Usability::Refused {
            code,
            rule: Some(rule),
        } => format!("{}:{}", code, rule),
        Usability::Refused { code, rule: None } => code.clone(),
    }
}

// synthetic-linear-cds.tsv: synA, [42, 345) on the plus strand, 100
// residues; the record states /gene="synA" and /product="synthetic
// protein A".
#[test]
fn the_linear_record_offers_syna_alone() {
    assert_eq!(
        entries("synthetic-linear.gb"),
        vec![CatalogueEntry {
            id: "synA".to_string(),
            gene: Some("synA".to_string()),
            product: Some("synthetic protein A".to_string()),
            usability: Usability::Usable {
                residues: 100,
                strand: Strand::Plus,
            },
        }]
    );
}

// Every CDS of synthetic.gb with an id, in file order, refused exactly as
// synthetic-unused.tsv says: the two CDSs sharing `dup` both appear, and
// the CDS with no id does not.
#[test]
fn the_synthetic_record_lists_every_named_cds_with_its_refusal() {
    let expected: Vec<(String, String)> = rows("synthetic-unused.tsv", "index\tkey\tid\trefusal")
        .into_iter()
        .filter(|row| row[1] == "CDS" && row[2] != "-")
        .map(|row| (row[2].clone(), row[3].clone()))
        .collect();
    assert_eq!(expected.len(), 23, "synthetic-unused.tsv has 23 named CDSs");
    let found: Vec<(String, String)> = entries("synthetic.gb")
        .iter()
        .map(|entry| (entry.id.clone(), refusal_text(&entry.usability)))
        .collect();
    assert_eq!(found, expected);
}

// The four usable CDSs of synthetic.gb, against synthetic-expected.tsv:
// the residue count is the length of its `residues` column, and the
// strand its `strand` column.
#[test]
fn each_usable_synthetic_cds_has_the_length_and_strand_of_the_fixture() {
    let expected: Vec<Vec<String>> = rows(
        "synthetic-expected.tsv",
        "name\terror\trule\tintervals\tstrand\tid_qualifier\ttransl_table\tresidues\tdetail\tline",
    );
    let mut checked: Vec<String> = Vec::new();
    for entry in entries("synthetic.gb") {
        if let Usability::Usable { residues, strand } = entry.usability {
            let row: &Vec<String> = expected
                .iter()
                .find(|row| row[0] == entry.id)
                .unwrap_or_else(|| panic!("synthetic-expected.tsv has no row {}", entry.id));
            assert_eq!(residues, row[7].len(), "{}: residues", entry.id);
            let fixture_strand: Strand = if row[4] == "plus" {
                Strand::Plus
            } else {
                Strand::Minus
            };
            assert_eq!(strand, fixture_strand, "{}: strand", entry.id);
            checked.push(entry.id.clone());
        }
    }
    assert_eq!(checked, vec!["p1", "m1", "pj", "atg11"]);
}

// /gene is reported as a qualifier, not confused with the id: m1 takes its
// id from /locus_tag and also has /gene="ignored-gene"; pj is named by
// /label and has no /gene. synthetic.gb states no /product anywhere.
#[test]
fn gene_and_product_are_the_features_own_qualifiers() {
    let found: Vec<CatalogueEntry> = entries("synthetic.gb");
    let entry = |id: &str| -> CatalogueEntry {
        found
            .iter()
            .find(|entry| entry.id == id)
            .unwrap_or_else(|| panic!("no entry {}", id))
            .clone()
    };
    assert_eq!(entry("m1").gene, Some("ignored-gene".to_string()));
    assert_eq!(entry("pj").gene, None);
    assert_eq!(entry("p1").gene, Some("p1".to_string()));
    assert!(found.iter().all(|entry| entry.product.is_none()));
}

// pBR322 (pbr322-cds.tsv, read by Biopython): tet, plus, 396 residues;
// bla, minus, 286 residues; the ROP CDS has no id and is not listed. The
// products are the record's /product values.
#[test]
fn pbr322_offers_tet_and_bla() {
    assert_eq!(
        entries("raw/J01749.1.gb"),
        vec![
            CatalogueEntry {
                id: "tet".to_string(),
                gene: Some("tet".to_string()),
                product: Some("tetracycline resistance protein".to_string()),
                usability: Usability::Usable {
                    residues: 396,
                    strand: Strand::Plus,
                },
            },
            CatalogueEntry {
                id: "bla".to_string(),
                gene: Some("bla".to_string()),
                product: Some("beta-lactamase".to_string()),
                usability: Usability::Usable {
                    residues: 286,
                    strand: Strand::Minus,
                },
            },
        ]
    );
}
