//! The CDS catalogue of plan §3 and M6-4 (contract 6c): what the model is
//! shown about a GenBank template, and nothing else.
//!
//! One entry per CDS a user could name, in file order (D012), each with its
//! id, its `/gene`, its `/product` and whether luria can use it: either its
//! residue count, terminal stop excluded, and its strand, or the refusal's
//! code and rule. A CDS with no id cannot be named, so it is not listed
//! (D041 G2); two CDSs sharing an id are both listed, each with its own
//! refusal.
//!
//! The catalogue is a privacy boundary, decided here by construction rather
//! than by remembering to leave things out: these types can hold no
//! sequence and no coordinate. `CatalogueEntry` has no field for an
//! interval, a location text, a sequence, a codon or a translation, and
//! `Usability` has none for a length in nucleotides or a residue range.
//! Rendering the entries as prompt text is 6f; looking names up in them is
//! 6e.
//!
//! Usability is core's decision, never a second opinion here (D003): it is
//! exactly what `luria_core::genbank_cds::import_cds` gives, and the
//! refusal's code and rule are exactly what `luria q5 design` reports,
//! through `DomainError::code` and `crate::result::rule_code` (D041 G1).
//! None of `import_cds`'s rules is re-implemented. A refusal of one CDS is
//! that entry's `Refused`, not an error of the catalogue; the errors the
//! module returns are core's own.

use luria_core::error::DomainError;
use luria_core::error::GenBankCdsRule;
use luria_core::error::UnusableGenBankCdsProblem;
use luria_core::genbank::{GenBankFeature, GenBankRecord};
use luria_core::genbank_cds::{import_cds, GenBankImport, ImportedCds};
use luria_core::identity::Identifier;
use luria_core::strand::Strand;
use luria_core::translation::{translate, Translation};

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
pub fn catalogue(record: &GenBankRecord) -> Result<Vec<CatalogueEntry>, DomainError> {
    // §3.3 step 1: with no names every feature is not used, in file order,
    // each CDS with its id and the refusal importing it would give.
    let listing: GenBankImport = import_cds(record, &[])?;
    let features: Vec<GenBankFeature> = record.features();
    let mut entries: Vec<CatalogueEntry> = Vec::new();
    for feature in listing.unused().iter() {
        // §3.3 step 2: only a `CDS` feature can be named, and only one
        // whose id exists. A CDS with no id is not listed; two CDSs
        // sharing an id both appear, each with its own refusal.
        if feature.key() != "CDS" {
            continue;
        }
        let index: usize = feature.index();
        let id: Option<String> = feature.id();
        match id {
            None => {
                continue;
            }
            Some(name) => {
                // §3.3 step 3: the feature's own `/gene` and `/product`,
                // and nothing else - no `/note`, no location text.
                let usability: Usability = usability_of(record, name.clone(), feature.refusal())?;
                entries.push(CatalogueEntry {
                    id: name.clone(),
                    gene: qualifier_value(&features, index, "gene".to_string()),
                    product: qualifier_value(&features, index, "product".to_string()),
                    usability,
                });
            }
        }
    }
    Ok(entries)
}

/// The usability of one CDS (§3.3 step 4): core's refusal becomes `Refused`
/// with the error's code and rule as the result document reports them, and
/// no refusal is core's to give, so the CDS is imported by its id and its
/// residue count and strand come from that import.
fn usability_of(
    record: &GenBankRecord,
    id: String,
    refusal: Option<DomainError>,
) -> Result<Usability, DomainError> {
    match refusal {
        Some(error) => Ok(Usability::Refused {
            code: error.code().to_string(),
            rule: crate::result::rule_code(&error).map(|rule| rule.to_string()),
        }),
        None => {
            // The one CDS, imported by its id: its residue count and its
            // strand are core's, from `import_cds` and `translate`.
            let identifier: Identifier = Identifier::create(id.clone())?;
            let names: Vec<Identifier> = vec![identifier];
            let single: GenBankImport = import_cds(record, &names)?;
            let coding: Vec<ImportedCds> = single.cds();
            if coding.len() == 1 {
                let only: ImportedCds = coding[0].clone();
                let residues: Translation = translate(&record.sequence(), &only.cds())?;
                Ok(Usability::Usable {
                    residues: residues.residue_count(),
                    strand: only.cds().strand(),
                })
            } else {
                // `import_cds` answered for the id without importing it.
                // Its own steps (§4.1) refuse a name that selects no CDS,
                // or one that names two, before it can answer `Ok`, so an
                // import that gave none is a refusal reported as core's,
                // never as a panic (contract §5).
                Err(DomainError::UnusableGenBankCds(UnusableGenBankCdsProblem {
                    cds: id,
                    rule: GenBankCdsRule::NotFound,
                    line: None,
                    detail: None,
                }))
            }
        }
    }
}

/// The value of the qualifier `/name` as it first appears on the feature at
/// `index`, `None` when the qualifier is absent or carries no value (§3.3
/// step 3). Nothing else of the feature is read.
fn qualifier_value(features: &[GenBankFeature], index: usize, name: String) -> Option<String> {
    for pair in features.iter().enumerate() {
        if pair.0 != index {
            continue;
        }
        for each in pair.1.qualifiers().iter() {
            if each.name() == name {
                return each.value();
            }
        }
        return None;
    }
    None
}
