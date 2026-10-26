//! `match_cds`, `cds_names` and `cds_span` of
//! `crates/core/src/name_match.rs` (contract 8i1, D049's M8-15, D053, D056)
//! against the records already in `tests/fixtures/`: pBR322
//! (`genbank/raw/J01749.1.gb`) and `genbank/synthetic.gb`.
//!
//! Every expectation here was read off the GenBank text by hand and none was
//! taken from running luria. pBR322's CDS qualifiers are at
//! `J01749.1.gb:260-265` (`/gene="tet"`, `/protein_id="AAB59735.1"`),
//! `J01749.1.gb:321-325` (`/product="ROP protein"`, no `/gene`, `/locus_tag`
//! or D041 id, `/protein_id="AAB59736.1"`) and `J01749.1.gb:361-367`
//! (`/gene="bla"`, `/product="beta-lactamase"`,
//! `/protein_id="AAB59737.1"`); its spans are `86..1276`, `1915..2106` and
//! `complement(3293..4153)`, and the display names and their order follow
//! D053. synthetic's CDSs are at `synthetic.gb:12-74`; feature 0 is the
//! `source`, features 1 to 24 are the 24 CDSs and features 25 and 26 are a
//! `misc_feature` and a `gene`, so the index of a CDS in `record.cdss` is its
//! feature index minus one. The names the tests quote are the qualifiers on
//! those key lines — `m1` at `synthetic.gb:15-17`, `pj` at
//! `synthetic.gb:19-20`, the two `dup`s at `synthetic.gb:55-58` — and the
//! spans are the intervals in D036 order, origin joins included (D036):
//! `join(55..60,1..12)` is `(55, 12)`, `join(complement(1..3),
//! complement(55..60))` is `(55, 3)`, `<1..12` is drawn as `(1, 12)` by the
//! display parser (D049, D053) and `order(1..6,7..12)` is not drawn at all,
//! which is why its span is `None` and it has no list form.

use luria_core::identity::Identifier;
use luria_core::name_match::cds_span;
use luria_core::name_match::match_cds;
use luria_core::name_match::Matched;
use luria_core::name_match::NameMatch;
use luria_core::name_match::NameSource;
use luria_core::view_record::open_record;
use luria_core::view_record::ViewRecord;

const GENBANK: &str = "../../tests/fixtures/genbank/";
const PBR322_FILE: &str = "raw/J01749.1.gb";
const SYNTHETIC_FILE: &str = "synthetic.gb";

/// A fixture input, read as it sits on disk.
fn read_input(dir: &str, file: &str) -> String {
    let path: String = format!("{}{}", dir, file);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("cannot read {}: {}", path, error))
}

/// The identifier a record is read as: the caller supplies the identity, as in
/// every other reader (D007).
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

/// The names of the CDS `record` lists at index `item`, with a failure that
/// says which one was wanted.
fn names_of(record: &ViewRecord, file: &str, item: usize) -> Vec<(NameSource, String)> {
    record
        .cdss
        .get(item)
        .unwrap_or_else(|| {
            panic!(
                "{}: no CDS at index {} of {} CDSs",
                file,
                item,
                record.cdss.len()
            )
        })
        .names
        .clone()
}

/// The single CDS a value names, refusing anything else.
fn one(record: &ViewRecord, value: &str) -> NameMatch {
    match match_cds(record, value) {
        Matched::One(found) => found,
        Matched::Several(found) => panic!(
            "{:?}: expected One, got Several ({}) {:?}",
            value,
            found.len(),
            found
        ),
        Matched::Nothing => panic!("{:?}: expected One, got Nothing", value),
    }
}

/// The CDSs a value names, in the order they were reported.
fn several(record: &ViewRecord, value: &str) -> Vec<usize> {
    match match_cds(record, value) {
        Matched::Several(found) => found.iter().map(|found| found.item).collect(),
        Matched::One(found) => panic!("{:?}: expected Several, got One {:?}", value, found),
        Matched::Nothing => panic!("{:?}: expected Several, got Nothing", value),
    }
}

/// The value names no CDS at all.
fn nothing(record: &ViewRecord, value: &str) {
    match match_cds(record, value) {
        Matched::Nothing => {}
        other => panic!("{:?}: expected Nothing, got {:?}", value, other),
    }
}

/// A wanted `(source, name)` pair, as `ViewCds::names` holds it.
fn pair(source: NameSource, value: &str) -> (NameSource, String) {
    (source, value.to_string())
}

// D049's id list, attached to the CDS itself: the display name first (D053),
// then each non-empty `/gene`, `/locus_tag` and `/protein_id` value. A CDS with
// no qualifier at all still has its display name, which is then the feature
// key, and `/product` reaches the list only as that display name.
#[test]
fn every_cds_carries_its_ids_and_their_sources() {
    let pbr322: ViewRecord = opened(GENBANK, PBR322_FILE);

    // `J01749.1.gb:260-265`: `/gene="tet"` is both the display name and an id,
    // and the `/protein_id` follows it.
    assert_eq!(
        names_of(&pbr322, PBR322_FILE, 0),
        vec![
            pair(NameSource::DisplayName, "tet"),
            pair(NameSource::Gene, "tet"),
            pair(NameSource::ProteinId, "AAB59735.1"),
        ],
        "{} CDS 0 (tet)",
        PBR322_FILE
    );
    // `J01749.1.gb:321-325`: no `/gene`, no `/locus_tag`, so the `/product`
    // names it for display and the `/protein_id` is its only id qualifier. This
    // is the CDS `import_cds` refuses `no_id` (D041, D053), and an unusable
    // CDS still carries its names, because the candidates are listed before
    // they are refused (D049, D056).
    assert_eq!(
        names_of(&pbr322, PBR322_FILE, 1),
        vec![
            pair(NameSource::DisplayName, "ROP protein"),
            pair(NameSource::ProteinId, "AAB59736.1"),
        ],
        "{} CDS 1 (ROP)",
        PBR322_FILE
    );
    // `J01749.1.gb:361-367`.
    assert_eq!(
        names_of(&pbr322, PBR322_FILE, 2),
        vec![
            pair(NameSource::DisplayName, "bla"),
            pair(NameSource::Gene, "bla"),
            pair(NameSource::ProteinId, "AAB59737.1"),
        ],
        "{} CDS 2 (bla)",
        PBR322_FILE
    );

    let synthetic: ViewRecord = opened(GENBANK, SYNTHETIC_FILE);

    // `synthetic.gb:15-17`: `/gene` is the display name (D053) and
    // `/locus_tag` is the D041 id, and both are ids D049 matches.
    assert_eq!(
        names_of(&synthetic, SYNTHETIC_FILE, 1),
        vec![
            pair(NameSource::DisplayName, "ignored-gene"),
            pair(NameSource::Gene, "ignored-gene"),
            pair(NameSource::LocusTag, "m1"),
        ],
        "{} CDS 1 (m1)",
        SYNTHETIC_FILE
    );
    // `synthetic.gb:19-20`: a `/label` names it and states no id, so the
    // display name is the only name.
    assert_eq!(
        names_of(&synthetic, SYNTHETIC_FILE, 2),
        vec![pair(NameSource::DisplayName, "pj")],
        "{} CDS 2 (pj)",
        SYNTHETIC_FILE
    );
    // `synthetic.gb:49-50`: a CDS with nothing to name it, displayed as the
    // feature key and refused `no_id` (D041).
    assert_eq!(
        names_of(&synthetic, SYNTHETIC_FILE, 12),
        vec![pair(NameSource::DisplayName, "CDS")],
        "{} CDS 12 (no id)",
        SYNTHETIC_FILE
    );
}

// D036 and D053: the span the list form carries is the first interval's start
// and the last interval's end in D036 order, for either strand, and a feature
// the display parser cannot draw has no span at all.
#[test]
fn a_span_is_read_in_d036_order() {
    let pbr322: ViewRecord = opened(GENBANK, PBR322_FILE);
    // `86..1276` and `complement(3293..4153)`, as `genbank/README.md:31-32`
    // and `genbank/pbr322-cds.tsv` read them: the strand changes nothing.
    assert_eq!(cds_span(&pbr322, 0), Some((86, 1276)), "tet");
    assert_eq!(cds_span(&pbr322, 2), Some((3293, 4153)), "bla");

    let synthetic: ViewRecord = opened(GENBANK, SYNTHETIC_FILE);
    // `join(55..60,1..12)`: the D036 intervals are `[54, 60)` then `[0, 12)`,
    // so the span runs 55 to 12 and the list form is `pj (55..12)`.
    assert_eq!(cds_span(&synthetic, 2), Some((55, 12)), "pj (2)");
    // `join(complement(1..3),complement(55..60))`, the same way round.
    assert_eq!(cds_span(&synthetic, 4), Some((55, 3)), "mj2 (4)");
    // `<1..12` is drawn by the display parser, at the stated base (D049,
    // D053), and the span it carries is the closed one.
    assert_eq!(cds_span(&synthetic, 16), Some((1, 12)), "fuzzy (16)");
    // `order(1..6,7..12)` is refused by the display parser as `order` (D053):
    // it is in the feature list, and it has no span to name it by.
    assert_eq!(cds_span(&synthetic, 17), None, "ord (17) is not drawn");
    // And an index past the CDSs is not a span either.
    assert_eq!(
        cds_span(&synthetic, synthetic.cdss.len()),
        None,
        "no CDS at that index"
    );
}

// D049's rules on a real record: the exact tier first, `/protein_id` and
// `/locus_tag` as ids, the display name as the first name of an item, the list
// form whole-only, and a `/product` that is not a display name worth nothing.
#[test]
fn matching_on_pbr322() {
    let record: ViewRecord = opened(GENBANK, PBR322_FILE);

    // Exact, on the display name of the tet CDS. Its `/gene` states the same
    // name, and that is one item rather than two matches (D049).
    assert_eq!(
        one(&record, "TET"),
        NameMatch {
            item: 0,
            source: NameSource::DisplayName,
            name: "tet".to_string(),
        },
        "TET names the tet CDS once"
    );
    // Exact on a `/protein_id`, with a space written into the value: interior
    // whitespace is ignored (D049).
    assert_eq!(
        one(&record, "aab 59737.1"),
        NameMatch {
            item: 2,
            source: NameSource::ProteinId,
            name: "AAB59737.1".to_string(),
        },
        "a protein id with a space in the value"
    );
    // A prefix at the start of the display name of the ROP CDS, which has no
    // D041 id and is refused `no_id` by `import_cds`: the matcher still names
    // it, and what the refusal means is the cli's (D056).
    assert_eq!(
        one(&record, "rop"),
        NameMatch {
            item: 1,
            source: NameSource::DisplayName,
            name: "ROP protein".to_string(),
        },
        "rop names the ROP CDS by its display name"
    );
    // The list form, as the whole string only, and folded like every other
    // name: the display name and the span with no space between them is the
    // same span (D049).
    assert_eq!(
        one(&record, "tet (86..1276)"),
        NameMatch {
            item: 0,
            source: NameSource::ListForm,
            name: "tet (86..1276)".to_string(),
        },
        "the tet list form"
    );
    assert_eq!(
        one(&record, "bla(3293..4153)"),
        NameMatch {
            item: 2,
            source: NameSource::ListForm,
            name: "bla (3293..4153)".to_string(),
        },
        "the bla list form, span written without spaces"
    );
    // A prefix of three `/protein_id`s is several candidates, listed in
    // ascending CDS order and answered as none of them being chosen (D049).
    assert_eq!(
        several(&record, "AAB5973"),
        vec![0, 1, 2],
        "the shared protein id prefix"
    );
    // `/product` is not an id: `bla`'s product is `beta-lactamase`, and that
    // word is a name only for the feature list (D053). It reaches matching
    // only when it is the display name, which pBR322's ROP `/product` is.
    nothing(&record, "lactamase");
}

// The same rules on synthetic: two CDSs sharing an id are several, a prefix of
// two different ids is several, and a list form is a whole string.
#[test]
fn matching_on_synthetic() {
    let record: ViewRecord = opened(GENBANK, SYNTHETIC_FILE);

    // `synthetic.gb:55-58`: two CDSs, `1..12` and `complement(13..24)`, both
    // `/gene="dup"`.
    assert_eq!(
        several(&record, "dup"),
        vec![14, 15],
        "the two dup CDSs, in CDS order"
    );
    // `mj1` and `mj2` (`synthetic.gb:22-26`) share a prefix and neither is
    // matched exactly.
    assert_eq!(several(&record, "mj"), vec![3, 4], "mj1 and mj2");
    // `m1` is the `/locus_tag` of the CDS displayed as `ignored-gene`: the id
    // matches exactly, and it is the name reported (D053).
    assert_eq!(
        one(&record, "m1"),
        NameMatch {
            item: 1,
            source: NameSource::LocusTag,
            name: "m1".to_string(),
        },
        "m1 is a locus tag"
    );
    // A prefix of that display name is matched by the display name, which is
    // the first of the item's names and the one reported.
    assert_eq!(
        one(&record, "ignored"),
        NameMatch {
            item: 1,
            source: NameSource::DisplayName,
            name: "ignored-gene".to_string(),
        },
        "ignored-gene by prefix"
    );
    // The list form of a CDS whose span crosses the origin (D036).
    assert_eq!(
        one(&record, "pj (55..12)"),
        NameMatch {
            item: 2,
            source: NameSource::ListForm,
            name: "pj (55..12)".to_string(),
        },
        "the pj list form"
    );
}
