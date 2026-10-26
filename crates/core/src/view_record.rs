//! The one record the view and `luria find` build from a file's text
//! (plan §4.1 of `docs/tasks/milestone-8-view.md`; D049, D048, D041, D042,
//! D052, D053).
//!
//! `open_record` is the single entry point (§4.1, D048's "shared entry point"):
//! format detection, where the topology came from and the size ceiling cannot
//! drift apart between the view and a command-line search, because there is no
//! second way in. Like every other reader here it takes text, never a file —
//! opening a file belongs to the caller, and the byte-size ceiling that keeps a
//! gigabase FASTA from reaching the parser is the cli's (D048, 8h).
//!
//! The rules, in the order the contract fixes them:
//!
//! 1. **Format by content, never by name** (D049). The first line that is not
//!    blank — empty, or made only of spaces, tabs and `\r`, exactly as both
//!    readers count a blank line — decides everything: `LOCUS` starts a GenBank
//!    record, `>` starts a FASTA record, and anything else, a text with no such
//!    line included, is refused `unknown_record_format`. Nothing is guessed at
//!    (D005) and the file's extension is never looked at.
//! 2. **The readers, unchanged.** What the sniff chose, `read_genbank` or
//!    `read_fasta` reads, on the whole text as it came. A reader's refusal is
//!    returned as it is: a record that starts with `LOCUS` and is malformed
//!    further in is refused by the GenBank reader with the GenBank reader's own
//!    code, not by this module.
//! 3. **The ceiling** (D048, M8-14). More than `MAX_RECORD_BASES` bases is
//!    refused and the limit is named in the message; exactly 10 000 000 is
//!    accepted. It counts bases, not bytes, and it is checked on the sequence
//!    the reader produced, as soon as that reader returns and before any
//!    feature is parsed or any CDS imported (§4.1's order). So it is the same
//!    count `Sequence::length()` reports and the same one the view draws. There
//!    is no flag and no environment variable for it (plan §8).
//! 4. **FASTA** (§4.1, D049): no features, no CDSs, and the topology the reader
//!    was handed — `Linear`, marked `Assumed`. The view shows it as `linear
//!    (assumed)` and `T` switches it for the session; the assumption is named
//!    on screen, is never inferred, and cannot enter a design's arithmetic.
//! 5. **GenBank**: `topology` is `Stated`, because the LOCUS line said it
//!    (D036, D041). One `ViewFeature` per entry of `GenBankRecord::features()`,
//!    in file order, with its file position and key line. No feature is
//!    dropped and none is filtered by key: `source`, `gene` and every other key
//!    are listed, and a feature whose location this module cannot draw stays in
//!    the list carrying the refusal that keeps it from being drawn (D053).
//! 6. **Partial features are drawn** (D049, D053), by `parse_display_location`
//!    — never by `parse_genbank_location`, whose rules and refusals are what
//!    the science tier accepts and stay exactly as they are (D009, D036). A
//!    feature that only the display parser can read is drawn; it is still not
//!    importable, so D041 and D042 are unchanged and it can never be selected
//!    for residue numbering.
//! 7. **Every CDS**, in file order, is `Usable` with the `Cds` `import_cds`
//!    built or `Unusable` with the refusal `import_cds` gives (D041, D042,
//!    D052). No CDS rule is re-implemented here, and `import_cds` is called
//!    twice per record — once with nothing named, which prices every CDS, and
//!    once with every payable id — not once per CDS, which would re-evaluate
//!    every CDS for each one and turn quadratic on a genome-sized record.
//!
//! Two names, deliberately different (D053): a feature's **display name** is
//! the first of `/label`, `/gene`, `/locus_tag` and `/product` that has a value
//! that is not empty, else the feature key, and it is what the feature list
//! shows; a CDS's **id** is D041's — the first of `/locus_tag`, `/gene`,
//! `/label` — and it is what import, `--cds` and the not-used table speak. In
//! `tests/fixtures/genbank/synthetic.gb` the `m1` CDS is named `ignored-gene`
//! and id'd `m1`, and pBR322's ROP CDS is named `ROP protein` and has no id at
//! all. A refusal shown in the view is the existing `DomainError` and its
//! `code()` (`fuzzy_end`, `alternative_start`, `no_id`, …), never a sentence
//! invented here: 5a1 made those codes the CLI's contract, and one rule with
//! two codes is worse than a code the view has to explain.

use crate::cds::Cds;
use crate::display_location::parse_display_location;
use crate::display_location::DisplayLocation;
use crate::error::DomainError;
use crate::error::RecordTooLongProblem;
use crate::error::UnknownRecordFormatProblem;
use crate::fasta::read_fasta;
use crate::fasta::FastaRecord;
use crate::genbank::read_genbank;
use crate::genbank::GenBankFeature;
use crate::genbank::GenBankQualifier;
use crate::genbank::GenBankRecord;
use crate::genbank_cds::import_cds;
use crate::genbank_cds::GenBankImport;
use crate::genbank_cds::UnusedFeature;
use crate::identity::Identifier;
use crate::name_match::NameSource;
use crate::sequence::Sequence;
use crate::sequence::SequenceTopology;

/// The ceiling on a record luria reads (D048, M8-14). Exactly this many bases
/// is accepted; one more is `DomainError::RecordTooLong`, whose message names
/// this number.
pub const MAX_RECORD_BASES: u64 = 10_000_000;

/// What a record's text turned out to be, decided by its content (D049).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordFormat {
    GenBank,
    Fasta,
}

/// Where the sequence's topology came from: the LOCUS line stated it, or the
/// view assumed linear for a FASTA record that states no topology (D049).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TopologySource {
    Stated,
    Assumed,
}

/// A feature as the view can use it: its shape, or the refusal that keeps it
/// from being drawn (D053). A refused feature stays in the feature list with
/// its refusal so the list can say why it is not drawn.
#[derive(Clone, Debug, PartialEq)]
pub enum FeatureShape {
    Drawn(DisplayLocation),
    NotDrawn(DomainError),
}

/// One entry of the feature table, as the feature list shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewFeature {
    /// Zero-based position in `GenBankRecord::features()`.
    pub index: usize,
    pub key: String,
    /// The first of `/label`, `/gene`, `/locus_tag`, `/product` whose value is
    /// present and not empty, in that order of preference; else the key.
    pub name: String,
    pub line: u64,
    pub shape: FeatureShape,
}

/// A CDS as `import_cds` answered: usable, or unusable with the refusal that
/// says so (D041, D042, D052). Only a `Usable` CDS may be selected for
/// translation and residue numbering.
#[derive(Clone, Debug, PartialEq)]
pub enum CdsStatus {
    Usable(Cds),
    Unusable(DomainError),
}

/// One CDS feature, in file order, with its D041 id and `import_cds`'s answer.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewCds {
    /// The `index` of its `ViewFeature`.
    pub feature: usize,
    /// Its D041 id (`UnusedFeature::id`), `None` when it has none.
    pub id: Option<String>,
    /// Every id D049 matches, in this order: the display name, then each
    /// non-empty `/gene`, `/locus_tag` and `/protein_id` value, each source in
    /// file order. Duplicates are kept: the first source is reported.
    pub names: Vec<(NameSource, String)>,
    pub status: CdsStatus,
}

/// A record as the view and `luria find` read it: the sequence, where its
/// topology came from, every feature, and every CDS.
#[derive(Clone, Debug, PartialEq)]
pub struct ViewRecord {
    pub format: RecordFormat,
    pub sequence: Sequence,
    pub topology: TopologySource,
    pub features: Vec<ViewFeature>,
    /// Every CDS feature, in file order.
    pub cdss: Vec<ViewCds>,
}

/// The qualifiers a display name is taken from, in order of preference (D053).
/// This is not the D041 id, which prefers `/locus_tag` first, then `/gene`,
/// then `/label`, and never `/product`.
const NAME_QUALIFIERS: [&str; 4] = ["label", "gene", "locus_tag", "product"];

/// The qualifiers D049's `--cds` matcher takes as ids, after the display name
/// and in the order D049 lists them, each with the `NameSource` it reports.
/// `/product` is not one of them: it names a feature for display (D053) and
/// reaches a CDS's ids only when it *is* that display name.
const ID_QUALIFIERS: [(&str, NameSource); 3] = [
    ("gene", NameSource::Gene),
    ("locus_tag", NameSource::LocusTag),
    ("protein_id", NameSource::ProteinId),
];

/// True for a blank line: empty, or made only of spaces, tabs and `\r`. The
/// same line both readers call blank (`genbank.rs` rule 2, `fasta.rs` rule 3),
/// so the sniff and the reader never disagree about where a record starts.
fn is_blank(line: &str) -> bool {
    line.chars().all(|ch| ch == ' ' || ch == '\t' || ch == '\r')
}

/// Step 1 (D049): the format, from the first line that is not blank. `LOCUS`
/// starts a GenBank record and `>` a FASTA record; anything else is no format
/// at all, and so is a text with no content line — the format comes from the
/// content, and the file's extension is never consulted.
fn sniff(text: &str, identifier: &str) -> Result<RecordFormat, DomainError> {
    let line: &str = match text.lines().find(|line| !is_blank(line)) {
        Some(line) => line,
        None => {
            return Err(DomainError::UnknownRecordFormat(
                UnknownRecordFormatProblem {
                    sequence: identifier.to_string(),
                },
            ))
        }
    };
    if line.starts_with("LOCUS") {
        return Ok(RecordFormat::GenBank);
    }
    if line.starts_with('>') {
        return Ok(RecordFormat::Fasta);
    }
    Err(DomainError::UnknownRecordFormat(
        UnknownRecordFormatProblem {
            sequence: identifier.to_string(),
        },
    ))
}

/// The feature's display name (D053): the first of the four name qualifiers
/// that states a value, else the feature key. A qualifier written bare
/// (`/label`) or with an empty value (`/gene=""`) states nothing to name a
/// feature by, so it does not win over the one below it.
fn display_name(feature: &GenBankFeature) -> String {
    let qualifiers: Vec<GenBankQualifier> = feature.qualifiers();
    for wanted in NAME_QUALIFIERS.iter() {
        for qualifier in qualifiers.iter() {
            if qualifier.name() != *wanted {
                continue;
            }
            let value: Option<String> = qualifier.value();
            match value {
                Some(value) if !value.is_empty() => return value,
                _ => continue,
            }
        }
    }
    feature.key()
}

/// Step 5 (D049, D053): the shape, from the display parser — never from
/// `parse_genbank_location`, whose refusals are what the science tier accepts
/// (D009, D036). A feature the display parser refuses keeps the refusal that
/// refuses it, so the feature list can say why it is not drawn.
fn shape_of(feature: &GenBankFeature, length: u64, topology: SequenceTopology) -> FeatureShape {
    let location: String = feature.location();
    match parse_display_location(&location, length, topology) {
        Ok(display) => FeatureShape::Drawn(display),
        Err(error) => FeatureShape::NotDrawn(error),
    }
}

/// Step 5: one `ViewFeature` per entry of the feature table, in file order,
/// with its position in that table and its key line. No feature is dropped and
/// none is filtered by key: `source`, `gene` and every other key is listed.
fn view_features(
    record: &GenBankRecord,
    length: u64,
    topology: SequenceTopology,
) -> Vec<ViewFeature> {
    let features: Vec<GenBankFeature> = record.features();
    let mut views: Vec<ViewFeature> = Vec::new();
    for pair in features.iter().enumerate() {
        let index: usize = pair.0;
        let feature: &GenBankFeature = pair.1;
        views.push(ViewFeature {
            index,
            key: feature.key(),
            name: display_name(feature),
            line: feature.line(),
            shape: shape_of(feature, length, topology.clone()),
        });
    }
    views
}

/// Step 6 (D041, D042, D052): every CDS feature, in file order, as `import_cds`
/// answers it. No CDS rule lives here, and `import_cds` runs twice for the
/// whole record — not once per CDS, because each call evaluates every CDS and
/// that is quadratic on a genome-sized record.
fn view_cdss(record: &GenBankRecord) -> Result<Vec<ViewCds>, DomainError> {
    // Pass 1: no name given, so every feature lands in `unused()` and every
    // CDS there carries the refusal importing it would give, `None` when it
    // would import (D052).
    let survey: GenBankImport = import_cds(record, &[])?;
    let unused: Vec<UnusedFeature> = survey.unused();

    // Pass 2: the CDSs that would import, named by their D041 id, in one call.
    // Such a CDS always has an id: a CDS with none is refused `no_id` and one
    // whose id is not an identifier is refused `invalid_identifier_characters`,
    // which is rows 13 and 24 of `synthetic-unused.tsv`.
    let mut names: Vec<Identifier> = Vec::new();
    for feature in unused.iter() {
        if feature.key() != "CDS" || feature.refusal().is_some() {
            continue;
        }
        let id: String = match feature.id() {
            Some(id) => id,
            None => unreachable!("a CDS that import_cds would take always states a D041 id"),
        };
        names.push(Identifier::create(id)?);
    }
    let import: GenBankImport = import_cds(record, &names)?;

    // The imported CDSs come back in the order their names were given, which is
    // not file order, so they are indexed by feature index once. Looking each
    // one up by scanning the returned list would pass over every imported CDS
    // once per CDS, which is quadratic in the CDS count on a genome-sized
    // record.
    let mut usable: std::collections::BTreeMap<usize, Cds> = std::collections::BTreeMap::new();
    for imported in import.cds() {
        usable.insert(imported.index(), imported.cds());
    }

    // `unused()` is in file order and covers every feature, so this lists the
    // CDSs in file order with their ids and their refusals.
    let features: &[GenBankFeature] = record.feature_slice();
    let mut cdss: Vec<ViewCds> = Vec::new();
    for feature in unused.iter() {
        if feature.key() != "CDS" {
            continue;
        }
        let status: CdsStatus = match feature.refusal() {
            Some(error) => CdsStatus::Unusable(error),
            None => match usable.get(&feature.index()) {
                Some(cds) => CdsStatus::Usable(cds.clone()),
                None => unreachable!(
                    "CDS feature {} has no refusal but is not imported",
                    feature.index()
                ),
            },
        };
        // Every id D049 matches for this CDS (M8-15), whatever `import_cds`
        // answered: an unusable CDS still has a name, because `find` and the
        // view list the candidates a value matched before they refuse them.
        // The display name leads, because it is what the list shows and what
        // the list form is built from (D053); the three id qualifiers follow,
        // each in the order they are written, and a bare qualifier or one with
        // an empty value states no id, exactly as it states no display name.
        // Duplicates are kept: `match_name` reports the first of an item's
        // matching names and never picks among them (D049, D056).
        let written: &GenBankFeature = &features[feature.index()];
        let qualifiers: Vec<GenBankQualifier> = written.qualifiers();
        let mut names: Vec<(NameSource, String)> =
            vec![(NameSource::DisplayName, display_name(written))];
        for pair in ID_QUALIFIERS.iter() {
            let (wanted, source): (&str, NameSource) = *pair;
            for qualifier in qualifiers.iter() {
                if qualifier.name() != wanted {
                    continue;
                }
                match qualifier.value() {
                    Some(value) if !value.is_empty() => names.push((source, value)),
                    _ => continue,
                }
            }
        }
        cdss.push(ViewCds {
            feature: feature.index(),
            id: feature.id(),
            names,
            status,
        });
    }
    Ok(cdss)
}

/// Step 3 (D048, M8-14): the ceiling, checked on the sequence a reader
/// produced. It counts bases, not bytes, exactly `MAX_RECORD_BASES` is
/// accepted, and the refusal names the limit.
///
/// Where the check sits is what §4.1 fixes: straight after the reader returns,
/// before anything else is done with the record. No test can see the position,
/// because a record over the limit is refused either way with the same payload;
/// what the order decides is how much work happens first. Checked after step 6,
/// a 50 000 000-base GenBank record would have every feature parsed and every
/// CDS imported, translated and evaluated twice before being refused.
fn check_ceiling(sequence: &Sequence, identifier: &str) -> Result<(), DomainError> {
    if sequence.length() > MAX_RECORD_BASES {
        return Err(DomainError::RecordTooLong(RecordTooLongProblem {
            sequence: identifier.to_string(),
            length: sequence.length(),
            limit: MAX_RECORD_BASES,
        }));
    }
    Ok(())
}

/// Read one record's text into what the view needs: the format comes from the
/// content (D049), the sequence from the matching reader, the size ceiling is
/// applied to that sequence before anything else is done with it (D048), and a
/// GenBank record's features and CDSs are listed as §4.1 describes. The
/// identifier is the sequence's, supplied by the caller (D007).
pub fn open_record(name: Identifier, text: String) -> Result<ViewRecord, DomainError> {
    let identifier: String = name.value().to_string();

    // Step 1: the content decides, the name never does (D049).
    let format: RecordFormat = sniff(&text, &identifier)?;

    // Steps 2 to 4: the whole text, unchanged, goes to the reader the sniff
    // chose, and a reader's refusal is returned as it is. Each arm then applies
    // the ceiling to the sequence it got, and nothing else happens to a record
    // that is over it.
    let (sequence, topology, features, cdss): (
        Sequence,
        TopologySource,
        Vec<ViewFeature>,
        Vec<ViewCds>,
    ) = match format {
        // A FASTA record states no topology, so linear is assumed and named as
        // an assumption (D049), and it has no features and no CDSs (§4.1).
        RecordFormat::Fasta => {
            let record: FastaRecord = read_fasta(name, text, SequenceTopology::Linear)?;
            let sequence: Sequence = record.sequence();
            // Step 3, before step 4 lists anything (D048).
            check_ceiling(&sequence, &identifier)?;
            (sequence, TopologySource::Assumed, Vec::new(), Vec::new())
        }
        // A GenBank record's topology came from its LOCUS line (D036, D041).
        RecordFormat::GenBank => {
            let record: GenBankRecord = read_genbank(name, text)?;
            let sequence: Sequence = record.sequence();
            // Step 3, before steps 5 and 6 parse a feature or import a CDS
            // (D048).
            check_ceiling(&sequence, &identifier)?;
            let features: Vec<ViewFeature> =
                view_features(&record, sequence.length(), sequence.topology());
            let cdss: Vec<ViewCds> = view_cdss(&record)?;
            (sequence, TopologySource::Stated, features, cdss)
        }
    };

    Ok(ViewRecord {
        format,
        sequence,
        topology,
        features,
        cdss,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SYNTHETIC_LINEAR: &str = "synthetic-linear.gb";

    /// The record of §5 test 8, written here as a literal rather than read
    /// from a fixture: the point is the name rule, and no fixture pairs a
    /// `/label` against a `/gene` and a bare `/label` against a `/gene`.
    const NAME_RECORD: &str = r#"LOCUS       names                     12 bp    DNA     linear   SYN 26-OCT-2026
FEATURES             Location/Qualifiers
     misc_feature    1..3
                     /gene="g-only"
                     /label=lab
     misc_feature    4..6
                     /product="prod"
                     /locus_tag="tag"
     misc_feature    7..9
                     /note="nothing to name it by"
     misc_feature    10..12
                     /label
                     /gene="g"
ORIGIN
        1 atggcttggt aa
//
"#;

    /// A fixture record, read as it sits on disk. A test's working directory
    /// is the package root, which is the path the integration tests use.
    fn read_input(file: &str) -> String {
        let path: String = format!("../../tests/fixtures/genbank/{}", file);
        std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error))
    }

    /// The identifier a record is read as: the caller supplies the identity, as
    /// in every other reader (D007).
    fn name(value: &str) -> Identifier {
        Identifier::create(value.to_string())
            .unwrap_or_else(|error| panic!("cannot use {:?} as an identifier: {:?}", value, error))
    }

    /// The record's display names, in feature order.
    fn names(record: &ViewRecord) -> Vec<String> {
        record.features.iter().map(|f| f.name.clone()).collect()
    }

    // Step 1 (D049): the content of the text decides the format, and the file's
    // name never enters. A blank line before the record is skipped, because
    // both readers skip it, but `LOCUS` and `>` have to start the first
    // non-blank line rather than appear somewhere inside it.
    #[test]
    fn the_format_is_decided_by_content() {
        // A `>` header wins even when the header itself says `LOCUS`: the
        // format comes from what starts the line, and a FASTA header is free
        // text naming any legal identifier (D049, D007).
        for text in [">x\nACGT\n", "\n \t\n>x\nACGT\n", ">LOCUS\nACGT\n"] {
            let found: ViewRecord = open_record(name("seq-a"), text.to_string())
                .unwrap_or_else(|error| panic!("{:?}: refused {}", text, error.message()));
            assert_eq!(
                found.format,
                RecordFormat::Fasta,
                "{:?}: wrong format",
                text
            );
        }

        // Anything else is refused, not guessed at (D005): bases on their own,
        // no content line at all, a `;` comment line (D008 never repairs one),
        // `ORIGIN`, which is inside a GenBank record but never starts one, and
        // `LOCUS` moved off column 0, which is no key at all (D049).
        for text in [
            "ACGT\n",
            "",
            "\n\n",
            ";c\n>x\nA\n",
            "ORIGIN\n",
            " LOCUS       x\n",
        ] {
            match open_record(name("seq-a"), text.to_string()) {
                Err(DomainError::UnknownRecordFormat(problem)) => {
                    assert_eq!(
                        problem.sequence, "seq-a",
                        "{:?}: the refusal names {:?}",
                        text, problem.sequence
                    );
                }
                Ok(record) => panic!("{:?}: accepted as {:?}", text, record.format),
                Err(error) => panic!(
                    "{:?}: refused {} instead of unknown_record_format",
                    text,
                    error.code()
                ),
            }
        }

        // Two blank lines written before a whole GenBank record: the sniff
        // skips them and the GenBank reader skips them, so the record is read.
        let text: String = format!("\n\n{}", read_input(SYNTHETIC_LINEAR));
        let found: ViewRecord = open_record(name(SYNTHETIC_LINEAR), text)
            .unwrap_or_else(|error| panic!("{}: refused {}", SYNTHETIC_LINEAR, error.message()));
        assert_eq!(
            found.format,
            RecordFormat::GenBank,
            "blank lines before a LOCUS line: wrong format"
        );
    }

    // Step 3 (D048, M8-14): 10 000 000 bases at the boundary, and one base past
    // it, both named in the refusal.
    #[test]
    fn the_ceiling_is_ten_million_bases() {
        let text: String = format!(">big\n{}\n", "A".repeat(10_000_000));
        let found: ViewRecord = open_record(name("big"), text)
            .unwrap_or_else(|error| panic!("10 000 000 bases: refused {}", error.message()));
        assert_eq!(
            found.sequence.length(),
            10_000_000,
            "the accepted record's length"
        );
        assert_eq!(
            found.format,
            RecordFormat::Fasta,
            "the accepted record's format"
        );

        let text: String = format!(">big\n{}\n", "A".repeat(10_000_001));
        let error: DomainError = match open_record(name("big"), text) {
            Ok(record) => panic!(
                "10 000 001 bases: accepted {} bases",
                record.sequence.length()
            ),
            Err(error) => error,
        };
        assert_eq!(
            error.code(),
            "record_too_long",
            "message: {}",
            error.message()
        );
        // The limit is named in the message, grouped or not (D048).
        let message: String = error.message();
        assert!(
            message.contains("10000000") || message.contains("10 000 000"),
            "the message does not name the limit: {}",
            message
        );
        match error {
            DomainError::RecordTooLong(problem) => {
                assert_eq!(problem.sequence, "big");
                assert_eq!(problem.length, 10_000_001);
                assert_eq!(problem.limit, MAX_RECORD_BASES);
            }
            other => panic!("expected RecordTooLong, got {:?}", other),
        }
    }

    // D053: the display name is the first of `/label`, `/gene`, `/locus_tag`,
    // `/product` with a value that is present and not empty, and the feature
    // key when none of the four has one. It is not the D041 id.
    #[test]
    fn the_display_name_prefers_label_gene_locus_tag_product() {
        let found: ViewRecord =
            open_record(name("names"), NAME_RECORD.to_string()).unwrap_or_else(|error| {
                panic!(
                    "the record of §5 test 8 was refused as {}: {}",
                    error.code(),
                    error.message()
                )
            });
        // `/label` beats `/gene`; `/gene` beats `/locus_tag`, which beats
        // `/product`; a feature none of the four names is called by its key; a
        // bare `/label` states no value, so the `/gene` under it names it.
        assert_eq!(names(&found), vec!["lab", "tag", "misc_feature", "g"]);
    }
}
