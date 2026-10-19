# Milestone 6c: the CDS catalogue

Status: **Implemented 2026-10-19** by the local worker; accepted on the first
review. The unreachable branch of §3.3 step 4 (an import that returns no CDS)
returns core's own `NotFound` refusal for the id; no test reaches it.
Planner: Claude. Parent: `docs/tasks/milestone-6-nl.md` §3 ("What leaves
the machine"), §5.5, M6-4, §6 (6c).

## 1. What and why

To name a CDS, the model has to know which CDSs the template offers. This
contract builds that list, the *catalogue*: one entry per CDS a user could
name, holding exactly what the model may be shown (M6-4). The prompt (6f)
renders it, and request building (6e) looks names up in it.

The catalogue is also a privacy boundary. Its types hold **no sequence and
no coordinate**: there is no field where one could go. What the model sees
is decided here, by construction, not by remembering to leave things out.

## 2. Requirements this contract applies

- **M6-4:** the model sees each CDS's id, `/gene`, `/product`, length in
  residues, strand, and whether luria can use it. Never the sequence, never
  a coordinate.
- **M6-7:** GenBank templates only. The catalogue takes a `GenBankRecord`.
- **D041 G2:** a CDS's id is the first of `/locus_tag`, `/gene` and
  `/label`. A CDS with none of them cannot be named, so it is not listed.
- **D003:** no second opinion on what is usable. Usability is exactly what
  `luria_core::genbank_cds::import_cds` decides; refusal codes are exactly
  what `luria q5 design` reports, through `error.code()` and
  `luria_cli::result::rule_code`.
- **D012:** the catalogue is in file order and deterministic.

## 3. Files

### 3.1 `crates/cli/src/interpret.rs`

Add `pub mod catalogue;` before `pub mod notation;`. Change nothing else.

### 3.2 `crates/cli/src/interpret/catalogue.rs` (new)

Module doc: what the catalogue is, that it is what the model is shown
(M6-4), that its types can hold no sequence or coordinate, and that
usability is core's decision, not this module's.

```rust
use luria_core::error::DomainError;
use luria_core::genbank::GenBankRecord;
use luria_core::strand::Strand;

/// One CDS a user could name, as the model is shown it (plan §3, M6-4).
/// It holds no sequence and no coordinate.
#[derive(Clone, Debug, PartialEq)]
pub struct CatalogueEntry {
    /// The CDS's id under D041 G2: the name a request would use.
    pub id: String,
    /// The value of the feature's first `/gene` qualifier, if it has one
    /// with a value.
    pub gene: Option<String>,
    /// The value of the feature's first `/product` qualifier, if it has
    /// one with a value.
    pub product: Option<String>,
    pub usability: Usability,
}

/// Whether luria can use the CDS, as `luria q5 design` would find.
#[derive(Clone, Debug, PartialEq)]
pub enum Usability {
    /// It imports: its residue count, terminal stop excluded, and its
    /// strand.
    Usable { residues: usize, strand: Strand },
    /// Importing it is refused: the error's code and, when the error has
    /// one, its rule, as the result document reports them.
    Refused { code: String, rule: Option<String> },
}

/// Every CDS of `record` that has an id, in file order.
pub fn catalogue(record: &GenBankRecord) -> Result<Vec<CatalogueEntry>, DomainError>;
```

### 3.3 What `catalogue` does

1. `let listing = luria_core::genbank_cds::import_cds(record, &[])?;` With
   no names, every feature is in `listing.unused()`, in file order, each
   CDS with its id and the refusal importing it would give.
2. For each `UnusedFeature` in that order, skip it unless its `key()` is
   exactly `CDS` and its `id()` is `Some`. A CDS with no id is not listed;
   two CDSs sharing an id are both listed, each with its own refusal.
3. `gene` and `product`: from `record.features()[feature.index()]`, the
   `value()` of the first qualifier whose `name()` is `gene`, respectively
   `product`. A qualifier with no value, or none at all, gives `None`.
   Take nothing else from the feature: no `/note`, no location text.
4. `usability`:
   - `refusal()` is `Some(error)`: `Refused { code: error.code().to_string(),
     rule: crate::result::rule_code(&error).map(|rule| rule.to_string()) }`;
   - `refusal()` is `None`: import that one CDS by its id,
     `import_cds(record, &[Identifier::create(id)?])?`, take its only
     `ImportedCds`, and report `Usable { residues, strand }` with
     `residues = translate(&record.sequence(), &cds)?.residue_count()` and
     `strand = cds.strand()`. If that import returns no CDS, return the
     error `import_cds` would give for it; do not panic.
5. Any `DomainError` from steps 1 or 4 is returned unchanged. A refusal of
   *one* CDS is not an error of the catalogue: it is that entry's
   `Refused`.

## 4. Tests (planner-written; add verbatim)

New file `crates/cli/tests/interpret_catalogue.rs`:

```rust
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
    let name: Identifier =
        Identifier::create("template".to_string()).expect("a valid identifier");
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
```

`cargo fmt` may reflow these; do not change content. If a test fails,
stop and report what `catalogue` returned. Do not change the test or a
fixture.

### 4.1 Proving the rules are load-bearing (AGENTS.md)

Disable each once, confirm the named test fails, restore, report both:

| Rule, disabled | Test that must fail |
| --- | --- |
| skip a CDS with no id (list it with an empty id instead) | `the_synthetic_record_lists_every_named_cds_with_its_refusal` |
| the refusal's rule (always `rule: None`) | `the_synthetic_record_lists_every_named_cds_with_its_refusal` |
| `/gene` from the qualifier (take the id instead) | `gene_and_product_are_the_features_own_qualifiers` |
| the terminal stop excluded (count it) | `the_linear_record_offers_syna_alone` |

## 5. Do not

- Add any field to `CatalogueEntry` or `Usability`, and in particular no
  interval, location, sequence, codon or translation text.
- Re-implement any part of `import_cds`'s rules, or decide usability any
  other way than §3.3 step 4.
- Render the catalogue as text for a prompt: that is 6f.
- Add a dependency, or make `luria-cli` depend on `luria-nl`.
- Change `luria-core`, `tests/fixtures/`, `docs/`, or any existing file
  under `crates/cli/` other than the one line in `interpret.rs`.
- Panic on any record. `catalogue` returns a `DomainError` instead.

## 6. Checks

From the repository root, with `TMPDIR=$PWD/target/tmp`:
- `cargo fmt --all -- --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `git diff --check`
- `git status --short`

## 7. Handoff

Report the new and changed files, the exact commands and their results,
the §4.1 results, and anything that did not match this contract. Quote
command output or the file for every claim; say plainly what you did not
check.
