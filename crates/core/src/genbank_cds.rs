//! Importing the CDSs a request names from a GenBank record (D041 G1, G2,
//! G3, G5; D006, D036).
//!
//! A request names the CDSs it wants by id. This module turns each named
//! feature into a `Cds`, checked against every rule luria applies, and lists
//! every other feature as *not used*. For an unused CDS the list also gives
//! the refusal luria would have given it, so a problem in a feature nobody
//! asked for never fails the run (D041 G1).
//!
//! §4.1, the request:
//!
//! 1. A name given more than once is refused before anything else is
//!    examined: `RepeatedName`, with no line and no detail.
//! 2. Each name, in the order the request gives it, selects the features
//!    whose key is exactly `CDS` and whose id text equals it. None is
//!    `NotFound`, two or more is `AmbiguousName` with the second feature's
//!    line, and one is evaluated below; its error is the result.
//! 3. Every feature not imported is listed in file order. A `CDS` feature
//!    carries its id text, when it has one, and the refusal its evaluation
//!    gives when that fails; evaluating an unused feature never makes the
//!    import fail. An empty request imports nothing and lists everything.
//!
//! §4.2, evaluating one CDS feature: the steps run in this order and the
//! first that fails decides. `{id}` is the id text step 1 found, and `line`
//! is the feature's key line.
//!
//! 1. The id (G2): the first of `/locus_tag`, `/gene` and `/label` that
//!    appears among the qualifiers, in that priority order and not in file
//!    order, and the value of its first occurrence. A feature with none of
//!    the three, or whose first occurrence carries no value, has no id:
//!    `NoId`, with an empty `cds`.
//! 2. `Identifier::create` on that text; its errors are passed through.
//! 3. Two or more `CDS` features in the record with this id text is
//!    `AmbiguousName`, with the second such feature's line. A named import
//!    already reported this at §4.1 step 2; the step matters for the not-used
//!    list.
//! 4. The id qualifier's own name, then `/codon_start`, `/transl_table` and
//!    `/translation`: the first that occurs more than once is
//!    `RepeatedQualifier`, with that name as the detail.
//! 5. `/pseudo`, then `/pseudogene`: the first present, with or without a
//!    value, is `Pseudo`, with that name as the detail. Such a feature is not
//!    a coding sequence luria edits.
//! 6. `/codon_start` with a value other than 1 is `CodonStart`, with that
//!    value as the detail (`(no value)` for a flag).
//! 7. `/transl_table` other than 1 or 11 is `TranslTable`, with that value
//!    as the detail. G5: 1 is the standard code, and 11 is accepted only as
//!    an ATG start, because tables 1 and 11 give every codon the same residue
//!    and differ only in their alternative starts. The CDS is imported as the
//!    standard code and the record's value kept for provenance.
//! 8. `parse_genbank_location` (5c1) on the location text, against the
//!    sequence's length and topology; its errors are passed through (D036).
//! 9. The pairs as intervals on the sequence's reference (D006) and `Cds::create`
//!    with the location's strand, explicitly complete, the standard code and
//!    numbering from the CDS start; its errors are passed through.
//! 10. `translate`; its errors are passed through.
//! 11. Under `/transl_table=11`, a first codon other than `ATG` is
//!     `AlternativeStart`, with those three letters as the detail.
//! 12. A `/translation` that differs from the translation without its
//!     terminal stop is `GenBankTranslationMismatch` (G3) at the first
//!     differing residue, counted from 1, or at the shorter length + 1 when
//!     one is a prefix of the other. A wrapped quoted value keeps the spaces
//!     the reader joined with, so both texts are compared without any space.
//! 13. Otherwise the feature is imported, and the qualifier its id came from,
//!     the feature's position and key line, and the record's `/transl_table`
//!     are recorded with it.
//!
//! Matching is exact and case-sensitive. Nothing is repaired or defaulted
//! beyond what these steps state: coordinates come from 5c1 as zero-based
//! half-open pairs in D036 order (D006), and a feature's strand is as its
//! location was written, never inferred.

use crate::cds::Cds;
use crate::cds::Completeness;
use crate::cds::Numbering;
use crate::error::DomainError;
use crate::error::GenBankCdsRule;
use crate::error::GenBankTranslationMismatchProblem;
use crate::error::UnusableGenBankCdsProblem;
use crate::extraction::extract_cds;
use crate::genbank::GenBankFeature;
use crate::genbank::GenBankRecord;
use crate::genbank_location::parse_genbank_location;
use crate::genbank_location::GenBankLocation;
use crate::genetic_code::STANDARD_CODE_NAME;
use crate::identity::Identifier;
use crate::identity::SequenceReference;
use crate::interval::create;
use crate::interval::NucleotideInterval;
use crate::interval::OrderedIntervals;
use crate::sequence::Sequence;
use crate::translation::translate;
use crate::translation::Translation;

/// Which qualifier a CDS's id came from (D041 G2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CdsIdQualifier {
    LocusTag,
    Gene,
    Label,
}

impl CdsIdQualifier {
    /// `locus_tag`, `gene` or `label`, as the qualifier is written.
    pub fn name(self) -> &'static str {
        use CdsIdQualifier::*;
        match self {
            LocusTag => "locus_tag",
            Gene => "gene",
            Label => "label",
        }
    }
}

/// A CDS imported from a GenBank feature, and where it came from.
#[derive(Clone, Debug, PartialEq)]
pub struct ImportedCds {
    cds: Cds,
    id_qualifier: CdsIdQualifier,
    index: usize,
    line: u64,
    transl_table: Option<String>,
}

impl ImportedCds {
    pub fn cds(&self) -> Cds {
        self.cds.clone()
    }

    pub fn id_qualifier(&self) -> CdsIdQualifier {
        self.id_qualifier
    }

    /// The feature's position in `GenBankRecord::features()`, from 0.
    pub fn index(&self) -> usize {
        self.index
    }

    /// The feature's key line.
    pub fn line(&self) -> u64 {
        self.line
    }

    /// The record's `/transl_table` value as written, `None` when absent.
    pub fn transl_table(&self) -> Option<String> {
        self.transl_table.clone()
    }
}

/// A feature the request did not import.
#[derive(Clone, Debug, PartialEq)]
pub struct UnusedFeature {
    index: usize,
    key: String,
    line: u64,
    id: Option<String>,
    refusal: Option<DomainError>,
}

impl UnusedFeature {
    pub fn index(&self) -> usize {
        self.index
    }

    pub fn key(&self) -> String {
        self.key.clone()
    }

    pub fn line(&self) -> u64 {
        self.line
    }

    /// For a CDS, its id under G2 when it has one; `None` for other keys.
    pub fn id(&self) -> Option<String> {
        self.id.clone()
    }

    /// For a CDS, the error importing it would give; `None` if it could have
    /// been imported, and for every other key.
    pub fn refusal(&self) -> Option<DomainError> {
        self.refusal.clone()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GenBankImport {
    cds: Vec<ImportedCds>,
    unused: Vec<UnusedFeature>,
}

impl GenBankImport {
    /// In the order the request named them.
    pub fn cds(&self) -> Vec<ImportedCds> {
        self.cds.clone()
    }

    /// Every other feature, in file order.
    pub fn unused(&self) -> Vec<UnusedFeature> {
        self.unused.clone()
    }
}

/// A CDS refusal payload: the rule, the requested name or the feature's id
/// (empty for `NoId`), the feature's key line and the offending value (§3).
fn problem(
    rule: GenBankCdsRule,
    cds: String,
    line: Option<u64>,
    detail: Option<String>,
) -> DomainError {
    DomainError::UnusableGenBankCds(UnusableGenBankCdsProblem {
        cds,
        rule,
        line,
        detail,
    })
}

/// The refusal of the feature under evaluation (§4.2).
fn refuse(
    rule: GenBankCdsRule,
    cds: String,
    line: Option<u64>,
    detail: Option<String>,
) -> Result<ImportedCds, DomainError> {
    Err(problem(rule, cds, line, detail))
}

/// The refusal of the request itself (§4.1).
fn unusable(
    rule: GenBankCdsRule,
    cds: String,
    line: Option<u64>,
    detail: Option<String>,
) -> Result<GenBankImport, DomainError> {
    Err(problem(rule, cds, line, detail))
}

/// How many times the qualifier `/name` appears on the feature.
fn count(feature: &GenBankFeature, name: String) -> usize {
    feature
        .qualifiers()
        .iter()
        .filter(|each| each.name() == name)
        .count()
}

/// The value of the qualifier's first occurrence; `None` when the qualifier
/// is absent or carries no value.
fn first_value(feature: &GenBankFeature, name: String) -> Option<String> {
    for each in feature.qualifiers().iter() {
        if each.name() == name {
            return each.value();
        }
    }
    None
}

/// The id of a CDS feature under G2: the first of `locus_tag`, `gene` and
/// `label` that appears, in priority order rather than file order, with the
/// value of its first occurrence. `None` when none of the three appears, or
/// when that occurrence carries no value (G2).
fn id_of(feature: &GenBankFeature) -> Option<(CdsIdQualifier, String)> {
    let preferred: [(CdsIdQualifier, String); 3] = [
        (CdsIdQualifier::LocusTag, "locus_tag".to_string()),
        (CdsIdQualifier::Gene, "gene".to_string()),
        (CdsIdQualifier::Label, "label".to_string()),
    ];
    for pair in preferred.iter() {
        for each in feature.qualifiers().iter() {
            if each.name() == pair.1 {
                match each.value() {
                    Some(text) => {
                        return Some((pair.0, text));
                    }
                    None => {
                        return None;
                    }
                }
            }
        }
    }
    None
}

/// The id text of a CDS feature, without the qualifier it came from.
fn id_text(feature: &GenBankFeature) -> Option<String> {
    id_of(feature).map(|(_qualifier, text)| text)
}

/// True when the feature is a `CDS` (G1) whose id text is `wanted` (§4.1
/// step 2 and §4.2 step 3).
fn is_named_cds(feature: &GenBankFeature, wanted: String) -> bool {
    if feature.key() != "CDS" {
        return false;
    }
    match id_text(feature) {
        Some(text) => text == wanted,
        None => false,
    }
}

/// The key lines of the CDS features whose id text is `wanted`, in file
/// order (§4.2 step 3).
fn shared_lines(features: &[GenBankFeature], wanted: String) -> Vec<u64> {
    let mut found: Vec<u64> = Vec::new();
    for feature in features.iter() {
        if is_named_cds(feature, wanted.clone()) {
            found.push(feature.line());
        }
    }
    found
}

/// The letters of a text, one string each, so a residue and a codon can be
/// compared one at a time.
fn letters(text: String) -> Vec<String> {
    text.chars().map(|ch| ch.to_string()).collect()
}

/// The text with every space removed, as a wrapped quoted value needs (§4.2
/// step 12).
fn compact_text(text: String) -> String {
    let out: String = text.chars().filter(|ch| *ch != ' ').collect();
    out
}

/// The first three letters of a CDS's coded text, or the whole text when it
/// holds fewer (§4.2 step 11).
fn first_codon(coding: String) -> String {
    let out: String = coding.chars().take(3).collect();
    out
}

/// The first residue where two texts differ, counted from 1, with the letters
/// there (`None` where one text has already ended). `None` when they are
/// equal (§4.2 step 12).
fn mismatch_at(
    stated: String,
    translated: String,
) -> Option<(u64, Option<String>, Option<String>)> {
    let first: Vec<String> = letters(stated);
    let second: Vec<String> = letters(translated);
    let length: usize = if first.len() > second.len() {
        first.len()
    } else {
        second.len()
    };
    for index in 0..length {
        let there: Option<String> = letter_at(&first, index);
        let here: Option<String> = letter_at(&second, index);
        if there != here {
            return Some((index as u64 + 1, there, here));
        }
    }
    None
}

/// The letter at `index`, `None` when the text is shorter.
fn letter_at(letters: &[String], index: usize) -> Option<String> {
    if index < letters.len() {
        Some(letters[index].clone())
    } else {
        None
    }
}

/// The location's pairs as intervals on `sequence` (D006).
fn intervals_of(
    sequence: SequenceReference,
    pairs: Vec<(u64, u64)>,
) -> Result<OrderedIntervals, DomainError> {
    let mut found: OrderedIntervals = Vec::new();
    for pair in pairs.iter() {
        let interval: NucleotideInterval = create(&sequence, pair.0 as i64, pair.1 as i64)?;
        found.push(interval);
    }
    Ok(found)
}

/// §4.2: the CDS feature at `index`, checked against every rule luria applies.
/// The first step that fails gives the refusal.
fn evaluate(record: &GenBankRecord, index: usize) -> Result<ImportedCds, DomainError> {
    let features: Vec<GenBankFeature> = record.features();
    let feature: GenBankFeature = features[index].clone();
    let seq: Sequence = record.sequence();
    let line: u64 = feature.line();

    // Step 1: the id, and the qualifier it came from.
    let (qualifier, text): (CdsIdQualifier, String) = match id_of(&feature) {
        Some(found) => found,
        None => return refuse(GenBankCdsRule::NoId, "".to_string(), Some(line), None),
    };

    // Step 2: the id as a luria identifier.
    let id: Identifier = Identifier::create(text.to_string())?;

    // Step 3: two CDSs with this id cannot be told apart.
    let lines: Vec<u64> = shared_lines(&features, text.clone());
    if lines.len() >= 2 {
        return refuse(
            GenBankCdsRule::AmbiguousName,
            text.clone(),
            Some(lines[1]),
            None,
        );
    }

    // Step 4: a qualifier that appears twice has no agreed value.
    let checked: Vec<String> = vec![
        qualifier.name().to_string(),
        "codon_start".to_string(),
        "transl_table".to_string(),
        "translation".to_string(),
    ];
    for name in checked.iter() {
        if count(&feature, name.to_string()) > 1 {
            return refuse(
                GenBankCdsRule::RepeatedQualifier,
                text,
                Some(line),
                Some(name.to_string()),
            );
        }
    }

    // Step 5: a pseudo feature is not a coding sequence luria edits.
    let pseudo_names: Vec<String> = vec!["pseudo".to_string(), "pseudogene".to_string()];
    for name in pseudo_names.iter() {
        if count(&feature, name.to_string()) > 0 {
            return refuse(
                GenBankCdsRule::Pseudo,
                text,
                Some(line),
                Some(name.to_string()),
            );
        }
    }

    // Step 6: only a codon start of 1 is read.
    if count(&feature, "codon_start".to_string()) > 0 {
        let stated: Option<String> = first_value(&feature, "codon_start".to_string());
        if stated != Some("1".to_string()) {
            return refuse(GenBankCdsRule::CodonStart, text.clone(), Some(line), stated);
        }
    }

    // Step 7: only tables 1 and 11 are read at all, and 11 only for an ATG
    // start, which step 11 checks.
    let transl_table: Option<String> = first_value(&feature, "transl_table".to_string());
    if count(&feature, "transl_table".to_string()) > 0
        && transl_table != Some("1".to_string())
        && transl_table != Some("11".to_string())
    {
        return refuse(
            GenBankCdsRule::TranslTable,
            text.clone(),
            Some(line),
            transl_table,
        );
    }

    // Step 8: the location, as 5c1 reads it.
    let location_text: String = feature.location();
    let location: GenBankLocation =
        parse_genbank_location(&location_text, seq.length(), seq.topology())?;

    // Step 9: the intervals, and the CDS they carry.
    let intervals: OrderedIntervals = intervals_of(seq.reference(), location.intervals())?;
    let cds: Cds = Cds::create(
        &seq,
        id,
        intervals,
        location.strand(),
        Completeness::Complete,
        STANDARD_CODE_NAME.to_string(),
        Numbering::FromCdsStart,
    )?;

    // Step 10: the residues, whose terminal stop is not one.
    let residues: Translation = translate(&seq, &cds)?;

    // Step 11: table 11 is the standard code only from ATG (G5).
    if transl_table == Some("11".to_string()) {
        let coding: String = extract_cds(&seq, &cds)?;
        let start: String = first_codon(coding);
        if start != "ATG" {
            return refuse(
                GenBankCdsRule::AlternativeStart,
                text,
                Some(line),
                Some(start),
            );
        }
    }

    // Step 12: a stated translation must be luria's own (G3).
    if count(&feature, "translation".to_string()) > 0 {
        let stated: String = match first_value(&feature, "translation".to_string()) {
            Some(value) => value,
            None => "".to_string(),
        };
        let translated: String = residues.residues_text();
        match mismatch_at(compact_text(stated), translated) {
            None => (),
            Some((residue, there, here)) => {
                return Err(DomainError::GenBankTranslationMismatch(
                    GenBankTranslationMismatchProblem {
                        cds: text,
                        line,
                        residue,
                        stated: there,
                        translated: here,
                    },
                ));
            }
        }
    }

    // Step 13: imported, with where it came from.
    Ok(ImportedCds {
        cds,
        id_qualifier: qualifier,
        index,
        line,
        transl_table,
    })
}

/// Import the CDSs `names` selects from `record`, and list every other
/// feature as not used (§4.1).
pub fn import_cds(
    record: &GenBankRecord,
    names: &[Identifier],
) -> Result<GenBankImport, DomainError> {
    // §4.1 step 1: a name the request gives more than once.
    for position in 0..names.len() {
        let name: Identifier = names[position].clone();
        let repeated: bool = names
            .iter()
            .skip(position + 1)
            .any(|other| other.value() == name.value());
        if repeated {
            return unusable(
                GenBankCdsRule::RepeatedName,
                name.value().to_string(),
                None,
                None,
            );
        }
    }

    let features: Vec<GenBankFeature> = record.features();
    let mut imported: Vec<ImportedCds> = Vec::new();
    let mut used: Vec<usize> = Vec::new();

    // §4.1 step 2: each name, in the order the request gives it.
    for name in names.iter() {
        let wanted: String = name.value().to_string();
        let mut matches: Vec<usize> = Vec::new();
        for pair in features.iter().enumerate() {
            if is_named_cds(pair.1, wanted.clone()) {
                matches.push(pair.0);
            }
        }
        if matches.is_empty() {
            return unusable(GenBankCdsRule::NotFound, wanted, None, None);
        }
        if matches.len() >= 2 {
            let second: u64 = features[matches[1]].line();
            return unusable(GenBankCdsRule::AmbiguousName, wanted, Some(second), None);
        }
        {
            let imported_cds: ImportedCds = evaluate(record, matches[0])?;
            imported.push(imported_cds);
            used.push(matches[0]);
        }
    }

    // §4.1 step 3: every feature the request did not import, in file order.
    let mut unused: Vec<UnusedFeature> = Vec::new();
    for pair in features.iter().enumerate() {
        let position: usize = pair.0;
        if used.contains(&position) {
            continue;
        }
        let feature: GenBankFeature = pair.1.clone();
        let is_cds: bool = feature.key() == "CDS";
        let id: Option<String> = if is_cds { id_text(&feature) } else { None };
        let refusal: Option<DomainError> = if is_cds {
            evaluate(record, position).err()
        } else {
            None
        };
        unused.push(UnusedFeature {
            index: position,
            key: feature.key(),
            line: feature.line(),
            id,
            refusal,
        });
    }

    Ok(GenBankImport {
        cds: imported,
        unused,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cds::Completeness;
    use crate::cds::Numbering;
    use crate::error::GenBankCdsRule;
    use crate::error::UnusableGenBankCdsProblem;
    use crate::genbank::read_genbank;
    use crate::genetic_code::GeneticCode;

    fn key(name: &str, location: &str) -> String {
        format!("     {:<16}{}", name, location)
    }

    fn q(text: &str) -> String {
        format!("{:21}{}", "", text)
    }

    fn record(features: &[String]) -> GenBankRecord {
        let mut lines: Vec<String> = vec![
            "LOCUS       tiny                      12 bp    DNA     linear   SYN 17-OCT-2026"
                .to_string(),
            "FEATURES             Location/Qualifiers".to_string(),
        ];
        lines.extend(features.iter().cloned());
        lines.push("ORIGIN".to_string());
        lines.push("        1 atggcttggt aa".to_string());
        lines.push("//".to_string());
        read_genbank(
            Identifier::create("tiny".to_string()).unwrap(),
            lines.join("\n"),
        )
        .unwrap()
    }

    fn names(list: &[&str]) -> Vec<Identifier> {
        list.iter()
            .map(|name| Identifier::create(name.to_string()).unwrap())
            .collect()
    }

    fn cds_problem(error: DomainError) -> UnusableGenBankCdsProblem {
        match error {
            DomainError::UnusableGenBankCds(problem) => problem,
            other => panic!("expected UnusableGenBankCds, got {:?}", other),
        }
    }

    #[test]
    fn a_pseudogene_qualifier_is_refused_by_its_name() {
        let found = record(&[
            key("CDS", "1..12"),
            q("/gene=\"g\""),
            q("/pseudogene=\"unitary\""),
        ]);
        let problem = cds_problem(import_cds(&found, &names(&["g"])).unwrap_err());
        assert_eq!(problem.rule, GenBankCdsRule::Pseudo);
        assert_eq!(problem.detail, Some("pseudogene".to_string()));
        assert_eq!(problem.line, Some(3));
    }

    #[test]
    fn a_codon_start_without_a_value_is_refused() {
        let found = record(&[key("CDS", "1..12"), q("/gene=\"g\""), q("/codon_start")]);
        let problem = cds_problem(import_cds(&found, &names(&["g"])).unwrap_err());
        assert_eq!(problem.rule, GenBankCdsRule::CodonStart);
        assert_eq!(problem.detail, None);
    }

    #[test]
    fn table_1_is_the_standard_code_and_is_recorded() {
        let found = record(&[key("CDS", "1..12"), q("/gene=\"g\""), q("/transl_table=1")]);
        let imported = import_cds(&found, &names(&["g"])).unwrap();
        assert_eq!(imported.cds()[0].transl_table(), Some("1".to_string()));
    }

    #[test]
    fn a_wrapped_translation_is_compared_without_its_spaces() {
        let found = record(&[
            key("CDS", "1..12"),
            q("/gene=\"g\""),
            q("/translation=\"MA"),
            q("W\""),
        ]);
        assert!(import_cds(&found, &names(&["g"])).is_ok());
    }

    #[test]
    fn a_valueless_id_qualifier_cannot_name_a_cds() {
        let found = record(&[key("CDS", "1..12"), q("/gene")]);
        let problem = cds_problem(import_cds(&found, &names(&["g"])).unwrap_err());
        assert_eq!(problem.rule, GenBankCdsRule::NotFound);
        let unused = import_cds(&found, &[]).unwrap().unused();
        let refusal = cds_problem(unused[0].refusal().unwrap());
        assert_eq!(refusal.rule, GenBankCdsRule::NoId);
        assert_eq!(refusal.cds, "");
        assert_eq!(unused[0].id(), None);
    }

    #[test]
    fn the_imported_cds_is_complete_standard_and_numbered_from_its_start() {
        let found = record(&[key("CDS", "1..12"), q("/label=g")]);
        let imported = import_cds(&found, &names(&["g"])).unwrap().cds();
        let cds = imported[0].cds();
        assert_eq!(cds.id().value(), "g");
        assert_eq!(cds.reference(), found.sequence().reference());
        assert_eq!(cds.completeness(), Completeness::Complete);
        assert_eq!(cds.genetic_code(), GeneticCode::Standard);
        assert_eq!(cds.numbering(), Numbering::FromCdsStart);
        assert_eq!(imported[0].id_qualifier(), CdsIdQualifier::Label);
        assert_eq!(imported[0].id_qualifier().name(), "label");
        assert_eq!(imported[0].line(), 3);
        assert_eq!(imported[0].index(), 0);
        assert_eq!(imported[0].transl_table(), None);
    }

    #[test]
    fn each_message_reads_as_the_contract_states() {
        let found = record(&[
            key("CDS", "1..12"),
            q("/gene=\"g\""),
            q("/translation=\"MAV\""),
        ]);
        assert_eq!(
            import_cds(&found, &names(&["g"])).unwrap_err().message(),
            "GenBank CDS 'g' states a /translation that differs from luria's at residue 3: the record has V, luria has W (line 3)"
        );
        assert_eq!(
            import_cds(&found, &names(&["h"])).unwrap_err().message(),
            "GenBank CDS 'h' is not the id of any CDS feature in the record"
        );
        let anonymous = record(&[key("CDS", "1..12")]);
        let unused = import_cds(&anonymous, &[]).unwrap().unused();
        assert_eq!(
            unused[0].refusal().unwrap().message(),
            "a GenBank CDS has none of /locus_tag, /gene or /label with a value, so it cannot be named (line 3)"
        );
        let table = record(&[key("CDS", "1..12"), q("/gene=\"g\""), q("/transl_table=4")]);
        assert_eq!(
            import_cds(&table, &names(&["g"])).unwrap_err().message(),
            "GenBank CDS 'g' has /transl_table 4; only 1, and 11 with an ATG start, are supported (line 3)"
        );
    }
}
