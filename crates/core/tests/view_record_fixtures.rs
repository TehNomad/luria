//! `open_record` of `crates/core/src/view_record.rs` (contract 8b2, §4.1 of
//! `docs/tasks/milestone-8-view.md`) against the records already in
//! `tests/fixtures/`: pBR322 (`genbank/raw/J01749.1.gb`), `genbank/synthetic.gb`
//! with the not-used table `genbank/synthetic-unused.tsv`, and
//! `view/frames-61.fa`.
//!
//! The expected values come from the GenBank text and the tables, read by hand
//! — synthetic's qualifiers at `synthetic.gb:16-20, 59, 75-78`, pBR322's at
//! `J01749.1.gb:232-241, 260-330`, and the coordinates and strands of the two
//! usable pBR322 CDSs from `genbank/pbr322-cds.tsv` and `genbank/README.md:31-32` —
//! and not from running luria. The statuses
//! and ids of synthetic's CDSs are the rows of `synthetic-unused.tsv`, which
//! 5c3 derived from the record (D041, D042, D052); the shapes and names are
//! D049's and D053's.

use luria_core::cds::Cds;
use luria_core::error::DomainError;
use luria_core::error::GenBankCdsRule;
use luria_core::error::GenBankLocationRule;
use luria_core::genbank::read_genbank;
use luria_core::genbank::GenBankFeature;
use luria_core::genbank::GenBankRecord;
use luria_core::identity::Identifier;
use luria_core::sequence::SequenceTopology;
use luria_core::strand::Strand;
use luria_core::view_record::open_record;
use luria_core::view_record::CdsStatus;
use luria_core::view_record::FeatureShape;
use luria_core::view_record::RecordFormat;
use luria_core::view_record::TopologySource;
use luria_core::view_record::ViewCds;
use luria_core::view_record::ViewFeature;
use luria_core::view_record::ViewRecord;

const GENBANK: &str = "../../tests/fixtures/genbank/";
const VIEW: &str = "../../tests/fixtures/view/";
const UNUSED_HEADER: &str = "index\tkey\tid\trefusal";
const PBR322_FILE: &str = "raw/J01749.1.gb";
const SYNTHETIC_FILE: &str = "synthetic.gb";
const UNUSED_TABLE: &str = "synthetic-unused.tsv";
const FRAMES_FILE: &str = "frames-61.fa";

/// A fixture input, read as it sits on disk.
fn read_input(dir: &str, file: &str) -> String {
    let path: String = format!("{}{}", dir, file);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("cannot read {}: {}", path, error))
}

/// The rows of a fixture TSV past its header row, refusing a header row it
/// does not have.
fn table(file: &str, header: &str) -> Vec<Vec<String>> {
    let text: String = read_input(GENBANK, file);
    let lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    assert!(
        !lines.is_empty(),
        "{}{}: expected header row {:?}, none found",
        GENBANK,
        file,
        header
    );
    assert!(
        lines[0] == header,
        "{}{}: first row {:?}, expected {:?}",
        GENBANK,
        file,
        lines[0],
        header
    );
    lines[1..]
        .iter()
        .map(|line| line.split('\t').map(|field| field.to_string()).collect())
        .collect()
}

/// The sequence identifier a fixture record is read as: the identity comes
/// from the caller, as in every other reader.
fn name(value: &str) -> Identifier {
    Identifier::create(value.to_string())
        .unwrap_or_else(|error| panic!("cannot use {:?} as an identifier: {:?}", value, error))
}

/// The record `open_record` builds for a fixture file, refusing to let a
/// refusal stand in for an answer the test has to compare.
fn opened(dir: &str, file: &str) -> ViewRecord {
    open_record(name(file), read_input(dir, file)).unwrap_or_else(|error| {
        panic!(
            "{}{}: open_record refused it as {}: {}",
            dir,
            file,
            error.code(),
            error.message()
        )
    })
}

/// `-` for nothing, the value itself otherwise, as the fixture tables write it.
fn or_dash(value: Option<String>) -> String {
    match value {
        None => "-".to_string(),
        Some(found) => found,
    }
}

/// The `refusal` column of the not-used tables: `-` for nothing, the error's
/// code, or `code:rule` when the error carries a rule.
fn refusal_text(refusal: Option<DomainError>) -> String {
    match refusal {
        None => "-".to_string(),
        Some(error) => {
            let code: String = error.code().to_string();
            match error {
                DomainError::UnusableGenBankCds(problem) => {
                    format!("{}:{}", code, problem.rule.code())
                }
                DomainError::UnusableGenBankLocation(problem) => {
                    format!("{}:{}", code, problem.rule.code())
                }
                other => other.code().to_string(),
            }
        }
    }
}

/// The `refusal` column for a `ViewCds`: `-` when `import_cds` would take it,
/// its refusal otherwise.
fn status_text(cds: &ViewCds) -> String {
    match &cds.status {
        CdsStatus::Usable(_) => "-".to_string(),
        CdsStatus::Unusable(error) => refusal_text(Some(error.clone())),
    }
}

/// The display name of the feature a `ViewCds` belongs to.
fn feature_name(record: &ViewRecord, cds: &ViewCds) -> String {
    record.features[cds.feature].name.clone()
}

/// The `GenBankCdsRule` of an unusable CDS, and a test failure for anything
/// else: an unusable CDS has to name the rule that refuses it (D052, D053).
fn cds_rule(cds: &ViewCds) -> GenBankCdsRule {
    match &cds.status {
        CdsStatus::Unusable(DomainError::UnusableGenBankCds(problem)) => problem.rule,
        other => panic!("expected an unusable CDS, got {:?}", other),
    }
}

/// The `GenBankLocationRule` of a feature that is not drawn.
fn location_rule(feature: &ViewFeature) -> GenBankLocationRule {
    match &feature.shape {
        FeatureShape::NotDrawn(DomainError::UnusableGenBankLocation(problem)) => problem.rule,
        other => panic!("expected an undrawn feature, got {:?}", other),
    }
}

/// Which ends of a drawn feature are open.
fn open_ends(feature: &ViewFeature) -> (Option<u64>, Option<u64>) {
    match &feature.shape {
        FeatureShape::Drawn(display) => (display.open_before, display.open_after),
        FeatureShape::NotDrawn(error) => panic!(
            "feature {} ({}) is not drawn: {}",
            feature.index,
            feature.key,
            error.message()
        ),
    }
}

/// The feature indexes of a record's CDSs.
fn cds_features(record: &ViewRecord) -> Vec<usize> {
    record.cdss.iter().map(|cds| cds.feature).collect()
}

/// The `Cds` a usable `ViewCds` carries, and a test failure for an unusable
/// one: these assertions ask what `import_cds` built, so a row that turns out
/// unusable deserves its own message (D052).
fn usable_cds(view: &ViewCds) -> Cds {
    match &view.status {
        CdsStatus::Usable(cds) => cds.clone(),
        CdsStatus::Unusable(error) => panic!(
            "CDS feature {}: unusable, {}",
            view.feature,
            refusal_text(Some(error.clone()))
        ),
    }
}

/// The id a `Cds` carries, as text (D007). A usable `ViewCds` has to carry the
/// CDS of its own feature, and the D041 id is how the two are checked against
/// each other: otherwise the view shows one CDS's translation under another's
/// name.
fn cds_id(cds: &Cds) -> String {
    cds.id().value().to_string()
}

/// The intervals a `Cds` selects, as `(start, end)` pairs, zero-based and
/// half-open: GenBank's `86..1276` is `(85, 1276)` and `complement(3293..4153)`
/// is `(3292, 4153)`, as `genbank/README.md:31-32` and
/// `genbank/pbr322-cds.tsv` record them.
fn pairs(cds: &Cds) -> Vec<(u64, u64)> {
    cds.intervals()
        .iter()
        .map(|interval| (interval.start(), interval.end()))
        .collect()
}

#[test]
fn pbr322_opens_as_a_stated_circular_genbank_record() {
    let text: String = read_input(GENBANK, PBR322_FILE);
    let record: ViewRecord = open_record(name(PBR322_FILE), text.clone()).unwrap_or_else(|error| {
        panic!(
            "{}: open_record refused it as {}: {}",
            PBR322_FILE,
            error.code(),
            error.message()
        )
    });

    // The LOCUS line states the topology (D036, D041), and the content decides
    // the format (D049).
    assert_eq!(record.format, RecordFormat::GenBank, "format");
    assert_eq!(record.topology, TopologySource::Stated, "topology source");
    assert_eq!(
        record.sequence.topology(),
        SequenceTopology::Circular,
        "topology"
    );
    assert_eq!(record.sequence.length(), 4361, "base count");

    // One `ViewFeature` per entry of `GenBankRecord::features()`, in file
    // order, with the same index, key and key line: no feature is dropped and
    // none is filtered by key (§8), drawn or not.
    let read: GenBankRecord = read_genbank(name(PBR322_FILE), text).unwrap_or_else(|error| {
        panic!(
            "{}: read_genbank refused it: {}",
            PBR322_FILE,
            error.message()
        )
    });
    let wanted: Vec<GenBankFeature> = read.features();
    assert_eq!(
        wanted.len(),
        50,
        "{}: the reader's feature count",
        PBR322_FILE
    );
    assert_eq!(
        record.features.len(),
        wanted.len(),
        "{}: open_record lists {} features, the reader reads {}",
        PBR322_FILE,
        record.features.len(),
        wanted.len()
    );
    for pair in wanted.iter().enumerate() {
        let index: usize = pair.0;
        let feature: &GenBankFeature = pair.1;
        let view: &ViewFeature = &record.features[index];
        assert_eq!(
            view.index, index,
            "feature {} carries index {}",
            index, view.index
        );
        assert_eq!(view.key, feature.key(), "feature {} key", index);
        assert_eq!(view.line, feature.line(), "feature {} key line", index);
    }
}

#[test]
fn pbr322_has_tet_and_bla_usable_and_rop_unusable() {
    let record: ViewRecord = opened(GENBANK, PBR322_FILE);

    // Three CDS features, in file order: `tet` at 86..1276, the ROP CDS at
    // 1915..2106, `bla` at complement(3293..4153) — features 10, 26 and 38 of
    // the table, as `pbr322-features.tsv` numbers them.
    assert_eq!(
        cds_features(&record),
        vec![10, 26, 38],
        "the CDS features, in file order"
    );

    // `tet`: `/gene` names it and is its D041 id, and it imports.
    let tet: &ViewCds = &record.cdss[0];
    assert_eq!(
        feature_name(&record, tet),
        "tet",
        "the tet CDS's display name"
    );
    assert_eq!(or_dash(tet.id.clone()), "tet", "the tet CDS's D041 id");
    assert!(
        matches!(tet.status, CdsStatus::Usable(_)),
        "tet: {}",
        status_text(tet)
    );

    // The `Cds` attached to it is tet's own and not merely some imported CDS:
    // `[85, 1276)` on the plus strand, which is `86..1276` at
    // `J01749.1.gb:260`, as `genbank/pbr322-cds.tsv` row `10` and
    // `genbank/README.md:31` record it.
    let tet_cds: Cds = usable_cds(tet);
    assert_eq!(cds_id(&tet_cds), "tet", "the tet CDS's own id");
    assert_eq!(tet_cds.strand(), Strand::Plus, "the tet CDS's strand");
    assert_eq!(
        pairs(&tet_cds),
        vec![(85, 1276)],
        "the tet CDS's intervals, as (start, end)"
    );

    // The ROP CDS: no `/locus_tag`, no `/gene`, no `/label`, so it has no id
    // and `no_id` is the first rule that refuses it — even though it also has
    // a GTG start (D053). Its display name comes from `/product` (D053).
    let rop: &ViewCds = &record.cdss[1];
    assert_eq!(
        feature_name(&record, rop),
        "ROP protein",
        "the ROP CDS's display name"
    );
    assert_eq!(rop.id, None, "the ROP CDS's D041 id");
    assert_eq!(cds_rule(rop), GenBankCdsRule::NoId, "the ROP CDS's refusal");

    // `bla`: `/gene` again, and it imports.
    let bla: &ViewCds = &record.cdss[2];
    assert_eq!(
        feature_name(&record, bla),
        "bla",
        "the bla CDS's display name"
    );
    assert_eq!(or_dash(bla.id.clone()), "bla", "the bla CDS's D041 id");
    assert!(
        matches!(bla.status, CdsStatus::Usable(_)),
        "bla: {}",
        status_text(bla)
    );

    // `[3292, 4153)` on the minus strand, from `complement(3293..4153)` at
    // `J01749.1.gb:361`, as `genbank/pbr322-cds.tsv` row `38` and
    // `genbank/README.md:32` record it.
    let bla_cds: Cds = usable_cds(bla);
    assert_eq!(cds_id(&bla_cds), "bla", "the bla CDS's own id");
    assert_eq!(bla_cds.strand(), Strand::Minus, "the bla CDS's strand");
    assert_eq!(
        pairs(&bla_cds),
        vec![(3292, 4153)],
        "the bla CDS's intervals, as (start, end)"
    );

    // The two `source` features are listed, and named by their key: none of
    // `/label`, `/gene`, `/locus_tag`, `/product` has a value on them, and no
    // key is filtered out of the feature list.
    for index in [0, 1] {
        assert_eq!(
            record.features[index].key, "source",
            "feature {} key",
            index
        );
        assert_eq!(
            record.features[index].name, "source",
            "feature {} name",
            index
        );
    }
}

#[test]
fn every_synthetic_cds_status_matches_the_not_used_table() {
    let record: ViewRecord = opened(GENBANK, SYNTHETIC_FILE);
    let rows: Vec<Vec<String>> = table(UNUSED_TABLE, UNUSED_HEADER);
    let cdss: Vec<Vec<String>> = rows.iter().filter(|row| row[1] == "CDS").cloned().collect();
    assert_eq!(cdss.len(), 24, "{}: CDS rows", UNUSED_TABLE);

    // Every CDS row of the table has its `ViewCds` at that feature index, in
    // file order, and there is no other `ViewCds`.
    let wanted: Vec<usize> = cdss.iter().map(|row| index(row)).collect();
    assert_eq!(
        cds_features(&record),
        wanted,
        "the CDS features, in file order"
    );

    for row in cdss.iter() {
        let feature: usize = index(row);
        let found: ViewCds = record
            .cdss
            .iter()
            .find(|cds| cds.feature == feature)
            .unwrap_or_else(|| panic!("{}: no ViewCds for CDS feature {}", UNUSED_TABLE, feature))
            .clone();
        assert_eq!(or_dash(found.id.clone()), row[2], "feature {} id", feature);
        assert_eq!(
            status_text(&found),
            row[3],
            "feature {} status: the record lists {}, the fixture records {}",
            feature,
            status_text(&found),
            row[3]
        );
        // A usable row also says which CDS the view carries: the `Cds` behind
        // `Usable` names itself with the id this row records, so a `ViewCds`
        // cannot be handed another feature's CDS (D041, D052).
        if row[3] == "-" && row[2] != "-" {
            let built: Cds = usable_cds(&found);
            assert_eq!(
                cds_id(&built),
                row[2],
                "feature {}: the usable CDS's own id",
                feature
            );
        }
    }
}

/// The feature index of a table row.
fn index(row: &[String]) -> usize {
    row[0]
        .parse()
        .unwrap_or_else(|_| panic!("{}: index {:?} is not a number", UNUSED_TABLE, row[0]))
}

#[test]
fn synthetic_features_are_drawn_or_kept_with_their_refusal() {
    let record: ViewRecord = opened(GENBANK, SYNTHETIC_FILE);
    assert_eq!(
        record.features.len(),
        27,
        "{}: feature count",
        SYNTHETIC_FILE
    );

    // `<1..12` (feature 17, `/gene="fuzzy"`) is drawn by the display parser of
    // 8b1, with the stated base 1 as the bound — zero-based 0 (D006) — and the
    // start open (D049, D053).
    assert_eq!(
        open_ends(&record.features[17]),
        (Some(0), None),
        "the fuzzy CDS's open ends"
    );

    // The four locations neither parser can read are kept, with the refusal
    // that keeps them from being drawn (D053), each named by the rule the
    // shared parser answers: `order`, `unsupported_join`, `single_position`,
    // `remote_reference`.
    let refused: [(usize, GenBankLocationRule); 4] = [
        (18, GenBankLocationRule::Order),
        (19, GenBankLocationRule::UnsupportedJoin),
        (20, GenBankLocationRule::SinglePosition),
        (21, GenBankLocationRule::RemoteReference),
    ];
    for pair in refused.iter() {
        assert_eq!(
            location_rule(&record.features[pair.0]),
            pair.1,
            "feature {}",
            pair.0
        );
    }

    // The names are D053's display names, not the D041 ids: feature 2 is named
    // by its `/gene` (`ignored-gene`) and id'd by its `/locus_tag` (`m1`);
    // feature 3 by its unquoted `/label`; feature 25 by its `/label`; and
    // feature 26, whose key is `gene`, by its `/gene` qualifier.
    let named: [(usize, &str); 5] = [
        (2, "ignored-gene"),
        (3, "pj"),
        (17, "fuzzy"),
        (25, "filler"),
        (26, "p1"),
    ];
    for pair in named.iter() {
        assert_eq!(
            record.features[pair.0].name, pair.1,
            "feature {} name",
            pair.0
        );
    }
}

#[test]
fn a_fasta_record_is_assumed_linear_with_nothing_else() {
    let record: ViewRecord = opened(VIEW, FRAMES_FILE);
    // FASTA carries no topology, so linear is assumed and marked as assumed
    // (D049); a FASTA record has no features and no CDSs (§4.1).
    assert_eq!(record.format, RecordFormat::Fasta, "format");
    assert_eq!(record.topology, TopologySource::Assumed, "topology source");
    assert_eq!(
        record.sequence.topology(),
        SequenceTopology::Linear,
        "topology"
    );
    assert_eq!(record.sequence.length(), 61, "base count");
    assert!(
        record.features.is_empty(),
        "features: {:?}",
        record.features
    );
    assert!(record.cdss.is_empty(), "cdss: {:?}", record.cdss);
}
