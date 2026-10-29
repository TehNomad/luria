//! The residue ↔ coordinate map in `crates/core/src/cds_map.rs` (8c) against
//! `tests/fixtures/view/`: the five tables `reference/cds_map.py` writes for
//! the CDSs of `tests/fixtures/genbank/synthetic.gb` (`p1`, `m1`, `pj`),
//! `synthetic-linear.gb` (`synA`), `raw/J01749.1.gb` (pBR322: `tet`, `bla`) and
//! the hand-written 62 bp record `view/frames-62.gb` (`fP`, `fM`), plus
//! `cds-map-refused.tsv` for the features that must never get a residue scale
//! (D010, D036, D040, D042, D050).
//!
//! Every map here is built the way §5 of the contract says — `read_genbank`,
//! then `import_cds`, then `translate` of the imported CDS, then `CdsMap::new`
//! — so the CDS and the translation agree because one import produced both
//! (D042), not because the test made them agree.
//!
//! `reference/cds_map.py` re-derives every table from Biopython's
//! `location.extract()` plus the arithmetic written out in `reference/`, and
//! `tests/fixtures/genbank/README.md` derives the reading of a wrapped CDS by
//! hand. Those scripts run from the repository root, not from cargo, in the
//! pinned Python environment:
//!
//! ```
//! python -I tests/fixtures/verify.py
//! ```
//!
//! That Python check is not part of the required cargo checks (D039, D026).

use luria_core::cds::Cds;
use luria_core::cds_map::CdsMap;
use luria_core::cds_map::CodonPlace;
use luria_core::error::DomainError;
use luria_core::error::MutationRequestRule;
use luria_core::genbank::read_genbank;
use luria_core::genbank::GenBankRecord;
use luria_core::genbank_cds::import_cds;
use luria_core::genbank_cds::GenBankImport;
use luria_core::genbank_cds::UnusedFeature;
use luria_core::identity::Identifier;
use luria_core::sequence::Sequence;
use luria_core::translation::translate;
use luria_core::translation::Translation;

const VIEW: &str = "../../tests/fixtures/view/";
const GENBANK: &str = "../../tests/fixtures/genbank/";
/// `cds-map-codons.tsv` and `cds-map-boundaries.tsv` carry the same columns.
const CODON_HEADER: &str =
    "record\tcds\tstatus\tstrand\tcodon_reading\tresidue\tbases_reading\tbases_ref_ascending";
const BASES_HEADER: &str =
    "record\tcds\tstrand\tposition\tcodon_reading\tplace_reading\tresidue\tcodon";
const COUNTS_HEADER: &str = "record\tcds\tstrand\tnt\tcodons\tresidues\tterminal_stop\tfirst_codon_ref\tlast_codon_ref\tlast_residue_ref";
const REFUSED_HEADER: &str = "record\tcds\trule\tdetail";
const CODONS_FILE: &str = "cds-map-codons.tsv";
const BASES_FILE: &str = "cds-map-bases.tsv";
const COUNTS_FILE: &str = "cds-map-counts.tsv";
const BOUNDARIES_FILE: &str = "cds-map-boundaries.tsv";
const REFUSED_FILE: &str = "cds-map-refused.tsv";
const PBR322_FILE: &str = "raw/J01749.1.gb";
const SYNTHETIC_FILE: &str = "synthetic.gb";
const LINEAR_FILE: &str = "synthetic-linear.gb";
const FRAMES_FILE: &str = "frames-62.gb";
/// The pBR322 CDS feature with no `/gene`, `/locus_tag` or `/protein_id`;
/// `cds-map-refused.tsv` names its index in its detail column, and
/// `genbank_fixtures.rs` pins the same feature and its refusal.
const ROP_INDEX: usize = 26usize;

/// The rows of a fixture TSV under `tests/fixtures/view/` past its header row,
/// refusing a header row it does not have.
fn table(file: &str, header: &str) -> Vec<Vec<String>> {
    let path: String = format!("{}{}", VIEW, file);
    let text: String = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
    let lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    assert!(
        !lines.is_empty(),
        "{}: expected header row {:?}, none found",
        path,
        header
    );
    assert!(
        lines[0] == header,
        "{}: first row {:?}, expected {:?}",
        path,
        lines[0],
        header
    );
    lines[1..]
        .iter()
        .map(|line| line.split('\t').map(|field| field.to_string()).collect())
        .collect()
}

/// A fixture input, read as it sits on disk.
fn read_input(dir: &str, file: &str) -> String {
    let path: String = format!("{}{}", dir, file);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("cannot read {}: {}", path, error))
}

/// The sequence identifier a fixture record is read as: the identity comes from
/// the caller, as in every other reader.
fn name(value: &str) -> Identifier {
    Identifier::create(value.to_string())
        .unwrap_or_else(|error| panic!("cannot use {:?} as an identifier: {:?}", value, error))
}

/// The identifiers of a request, in the order the request gives them.
fn request(values: Vec<&str>) -> Vec<Identifier> {
    values
        .iter()
        .map(|value| {
            Identifier::create(value.to_string()).unwrap_or_else(|error| {
                panic!("cannot use {:?} as an identifier: {:?}", value, error)
            })
        })
        .collect()
}

/// The record a fixture `record` column names, read with luria's own reader.
fn record_of(value: &str) -> GenBankRecord {
    let pair: (&str, &str) = match value {
        "pbr322" => (GENBANK, PBR322_FILE),
        "synthetic" => (GENBANK, SYNTHETIC_FILE),
        "synlin" => (GENBANK, LINEAR_FILE),
        "frames62" => (VIEW, FRAMES_FILE),
        other => panic!("{}: no fixture record of that name", other),
    };
    let record: GenBankRecord = read_genbank(name(pair.1), read_input(pair.0, pair.1))
        .unwrap_or_else(|error| {
            panic!("{}: read_genbank refused it: {:?}", pair.1, error.message())
        });
    record
}

/// The CDS a fixture row names and its map, built the way §5 says:
/// `read_genbank`, `import_cds`, then `translate` of the imported CDS, then
/// `CdsMap::new`. An import that refuses a CDS the tables call usable fails
/// here rather than quietly skipping rows.
fn subject(record_name: &str, cds_name: &str) -> (Cds, CdsMap) {
    let record: GenBankRecord = record_of(record_name);
    let sequence: Sequence = record.sequence();
    let import: GenBankImport =
        import_cds(&record, &request(vec![cds_name])).unwrap_or_else(|error| {
            panic!(
                "{}: import_cds refused the CDS {:?}: {:?}",
                record_name,
                cds_name,
                error.message()
            )
        });
    assert!(
        import.cds().len() == 1,
        "{}: importing {:?} gives {} CDSs, expected 1",
        record_name,
        cds_name,
        import.cds().len()
    );
    let cds: Cds = import.cds()[0].cds();
    let translation: Translation = translate(&sequence, &cds).unwrap_or_else(|error| {
        panic!(
            "{}: translate refused {:?}: {:?}",
            record_name, cds_name, error
        )
    });
    (cds.clone(), CdsMap::new(&cds, &translation))
}

/// The one 1-based → zero-based conversion (D006): every fixture coordinate is
/// GenBank-style 1-based inclusive, while every coordinate inside core is
/// zero-based and half-open. It is applied once, as a fixture column is parsed,
/// and no comparison in this file mixes the two conventions.
fn zero_based(one_based: u64) -> u64 {
    one_based
        .checked_sub(1)
        .unwrap_or_else(|| panic!("a fixture coordinate of 0 has no zero-based form"))
}

// Every comparison below is between core values (zero-based) and fixture values
// converted by `zero_based`; a failure message may print `position + 1` so the
// number in it is the one the fixture row writes.

/// A fixture number.
fn number(text: &str) -> u64 {
    text.parse::<u64>()
        .unwrap_or_else(|error| panic!("cannot read {:?} as a number: {}", text, error))
}

/// A reference column such as `bases_ref_ascending`: the 1-based positions it
/// lists, converted once here (D006) and compared only against core values.
fn fixture_positions(text: &str) -> Vec<u64> {
    text.split(',')
        .map(|field| zero_based(number(field)))
        .collect()
}

/// The residue column of a codon or base row: a residue number, or `stop` for
/// the terminal stop codon, which has no residue number at all (D010).
fn fixture_residue(text: &str) -> Option<u64> {
    if text == "stop" {
        return None;
    }
    Some(number(text))
}

/// The rows of a codon table for one CDS, in file order (which is reading order).
fn codon_rows(file: &str, record_name: &str, cds_name: &str) -> Vec<Vec<String>> {
    table(file, CODON_HEADER)
        .iter()
        .filter(|row| row[0] == record_name && row[1] == cds_name)
        .cloned()
        .collect()
}

/// The rows of `cds-map-bases.tsv` for one CDS, in file order, which the
/// fixture README states is reading order.
fn base_rows(record_name: &str, cds_name: &str) -> Vec<Vec<String>> {
    table(BASES_FILE, BASES_HEADER)
        .iter()
        .filter(|row| row[0] == record_name && row[1] == cds_name)
        .cloned()
        .collect()
}

/// The one `cds-map-counts.tsv` row for one CDS.
fn counts_row(record_name: &str, cds_name: &str) -> Vec<String> {
    let rows: Vec<Vec<String>> = table(COUNTS_FILE, COUNTS_HEADER)
        .iter()
        .filter(|row| row[0] == record_name && row[1] == cds_name)
        .cloned()
        .collect();
    assert!(
        rows.len() == 1,
        "{}: {} rows for {:?} {:?}, expected 1",
        COUNTS_FILE,
        rows.len(),
        record_name,
        cds_name
    );
    rows[0].clone()
}

/// Every reference position a CDS selects, in reference order.
fn selected_positions(cds: &Cds) -> Vec<u64> {
    let mut found: Vec<u64> = Vec::new();
    for interval in cds.intervals() {
        let mut position: u64 = interval.start();
        while position < interval.end() {
            found.push(position);
            position += 1;
        }
    }
    found
}

/// The biological offset of a reference position inside a CDS: how many CDS
/// bases are stored before it, counted in the order the intervals are stored
/// (D036). This is the same offset `cds_map.rs` and `mutation.rs::reference_of`
/// use, and it is why the ascending column of the codon tables cannot be sorted
/// into reading order on its own: `fP`'s codon 3 sits on 62, 1 and 2, and
/// `mj1`/`mj2` in `tests/fixtures/genbank/README.md` derive that reading by hand.
fn fixture_offset(cds: &Cds, position: u64) -> u64 {
    let mut seen: u64 = 0;
    for interval in cds.intervals() {
        if position >= interval.start() && position < interval.end() {
            return seen + (position - interval.start());
        }
        seen += interval.length();
    }
    panic!(
        "the CDS {:?} selects no position {}",
        cds.id().value(),
        position
    );
}

/// The positions of one codon in the order the codon is read: the fixture's
/// ascending column ordered by biological offset, descending on the minus
/// strand (D036, and D040's warning that reference order is not reading order).
/// Reading order comes from the CDS's own intervals rather than from a sort of
/// the ascending column, because an origin-straddling codon reads `62, 1, 2`.
fn fixture_reading_order(cds: &Cds, ascending: &[u64]) -> Vec<u64> {
    let mut keyed: Vec<(u64, u64)> = ascending
        .iter()
        .map(|position| (*position, fixture_offset(cds, *position)))
        .collect();
    keyed.sort_by_key(|pair| pair.1);
    if !cds.strand().is_plus() {
        keyed.reverse();
    }
    keyed.iter().map(|pair| pair.0).collect()
}

/// The `rule` column of `cds-map-refused.tsv`: the error's code, or `code:rule`
/// when the error carries a rule — the same text `genbank_fixtures.rs` compares
/// against its own refusal columns.
fn refusal_text(refusal: DomainError) -> String {
    let code: String = refusal.code().to_string();
    match refusal {
        DomainError::UnusableGenBankCds(problem) => format!("{}:{}", code, problem.rule.code()),
        DomainError::UnusableGenBankLocation(problem) => {
            format!("{}:{}", code, problem.rule.code())
        }
        other => other.code().to_string(),
    }
}

/// The refusal of a residue number this CDS has no codon for: D010 gives the
/// terminal stop codon a codon and no residue, and `codon_positions` refuses the
/// same number, so `residue_bases` must refuse it too.
fn refused_number(record_name: &str, cds_name: &str, map: &CdsMap, asked: u64) {
    let outcome: Result<[u64; 3], DomainError> = map.residue_bases(asked);
    match outcome {
        Ok(bases) => panic!(
            "{} {}: residue_bases({}) gives {:?}, expected a refusal",
            record_name, cds_name, asked, bases
        ),
        Err(error) => match error {
            DomainError::InvalidMutationRequest(problem) => assert!(
                problem.rule == MutationRequestRule::CodonOutsideSequence,
                "{} {}: residue_bases({}) is refused by rule {:?}, expected codon_outside_sequence",
                record_name,
                cds_name,
                asked,
                problem.rule
            ),
            other => panic!(
                "{} {}: residue_bases({}) is refused as {:?}, expected invalid_mutation_request",
                record_name,
                cds_name,
                asked,
                other.code()
            ),
        },
    }
}

#[test]
fn the_map_counts_match_the_counts_table() {
    let rows: Vec<Vec<String>> = table(COUNTS_FILE, COUNTS_HEADER);
    for row in rows.iter() {
        let (cds, map): (Cds, CdsMap) = subject(&row[0], &row[1]);
        let symbol: String = cds.strand().symbol();
        assert!(
            symbol == row[2],
            "{} {}: the imported CDS is on strand {}, the fixture records {}",
            row[0],
            row[1],
            symbol,
            row[2]
        );
        assert!(
            cds.total_length() == number(&row[3]),
            "{} {}: the CDS is {} bases, the fixture records {}",
            row[0],
            row[1],
            cds.total_length(),
            row[3]
        );
        assert!(
            map.codons() == number(&row[4]),
            "{} {}: the map has {} codons, the fixture records {}",
            row[0],
            row[1],
            map.codons(),
            row[4]
        );
        assert!(
            map.residues() == number(&row[5]),
            "{} {}: the map has {} residues, the fixture records {}",
            row[0],
            row[1],
            map.residues(),
            row[5]
        );
        let stop: String = if map.terminal_stop() { "yes" } else { "no" }.to_string();
        assert!(
            stop == row[6],
            "{} {}: the map's terminal_stop is {}, the fixture records {}",
            row[0],
            row[1],
            stop,
            row[6]
        );
        // The three position columns are checked through `base_residue`: every
        // base of the first codon, of the last codon and of the last residue's
        // codon, collected in reference order, which is how those columns are
        // written.
        for column in [7usize, 8usize, 9usize] {
            let wanted: u64 = match column {
                7 => 1,
                8 => map.codons(),
                _ => map.residues(),
            };
            let found: Vec<u64> = selected_positions(&cds)
                .iter()
                .filter(|position| match map.base_residue(**position) {
                    None => false,
                    Some(place) => {
                        if column == 9 {
                            place.residue == Some(wanted)
                        } else {
                            place.codon == wanted
                        }
                    }
                })
                .cloned()
                .collect();
            let expected: Vec<u64> = fixture_positions(&row[column]);
            assert!(
                found == expected,
                "{} {}: the {} of codon/residue {} sits on {:?}, the fixture records {:?}",
                row[0],
                row[1],
                ["first codon", "last codon", "last residue"][column - 7],
                wanted,
                found,
                expected
            );
        }
    }
}

#[test]
fn the_pbr322_cds_map_matches_the_fixture() {
    for cds_name in ["tet", "bla"] {
        let (cds, map): (Cds, CdsMap) = subject("pbr322", cds_name);
        let counts: Vec<String> = counts_row("pbr322", cds_name);
        assert!(
            cds.total_length() == number(&counts[3]),
            "pbr322 {}: the CDS is {} bases, the fixture records {}",
            cds_name,
            cds.total_length(),
            counts[3]
        );
        assert!(
            map.codons() == number(&counts[4]),
            "pbr322 {}: the map has {} codons, the fixture records {}",
            cds_name,
            map.codons(),
            counts[4]
        );
        assert!(
            map.residues() == number(&counts[5]),
            "pbr322 {}: the map has {} residues, the fixture records {}",
            cds_name,
            map.residues(),
            counts[5]
        );
        let stop: String = if map.terminal_stop() { "yes" } else { "no" }.to_string();
        assert!(
            stop == counts[6],
            "pbr322 {}: the map's terminal_stop is {}, the fixture records {}",
            cds_name,
            stop,
            counts[6]
        );
        for row in codon_rows(BOUNDARIES_FILE, "pbr322", cds_name).iter() {
            let codon: u64 = number(&row[4]);
            let residue: Option<u64> = fixture_residue(&row[5]);
            let ascending: Vec<u64> = fixture_positions(&row[7]);
            let reading: Vec<u64> = fixture_reading_order(&cds, ascending.as_slice());
            assert!(
                row[2] == "usable",
                "cds-map-boundaries.tsv: the {} row {:?} is not usable; this test only maps usable CDSs",
                cds_name,
                row
            );
            // A row whose residue column says `stop` is not put through
            // `residue_bases`: the terminal stop codon has no residue number to
            // ask with (D010), and the refusal of `residues + 1` is what
            // `a_number_with_no_residue_is_refused` checks. Such a row is
            // checked through `base_residue`, which must give its codon with
            // `residue: None`.
            if residue.is_none() {
                assert!(
                    codon == map.codons(),
                    "pbr322 {}: the stop row is codon {}, but the map's last codon is {}",
                    cds_name,
                    codon,
                    map.codons()
                );
                refused_number("pbr322", cds_name, &map, map.residues() + 1);
            } else {
                let given: [u64; 3] = map.residue_bases(codon).unwrap_or_else(|error| {
                    panic!(
                        "pbr322 {}: residue_bases({}) is refused: {:?}",
                        cds_name,
                        codon,
                        error.message()
                    )
                });
                let mut sorted: Vec<u64> = given.to_vec();
                sorted.sort();
                assert!(
                    sorted == ascending,
                    "pbr322 {}: codon {} is on {:?}, the fixture records {:?}",
                    cds_name,
                    codon,
                    sorted,
                    ascending
                );
                assert!(
                    given.to_vec() == reading,
                    "pbr322 {}: residue_bases({}) reads {:?}, the fixture's reading order is {:?}",
                    cds_name,
                    codon,
                    given,
                    reading
                );
            }
            for indexed in reading.iter().enumerate() {
                let place_index: usize = indexed.0;
                let position: u64 = *indexed.1;
                let place: CodonPlace = map.base_residue(position).unwrap_or_else(|| {
                    panic!(
                        "pbr322 {}: base_residue({}) maps to nothing, the fixture gives it codon {} place {}",
                        cds_name,
                        position + 1,
                        codon,
                        place_index + 1
                    )
                });
                assert!(
                    place.codon == codon,
                    "pbr322 {}: base {} belongs to codon {}, the fixture gives codon {}",
                    cds_name,
                    position + 1,
                    place.codon,
                    codon
                );
                assert!(
                    place.place == (place_index as u64) + 1,
                    "pbr322 {}: base {} has place {} in codon {}, expected place {}",
                    cds_name,
                    position + 1,
                    place.place,
                    codon,
                    place_index + 1
                );
                assert!(
                    place.residue == residue,
                    "pbr322 {}: base {} of codon {} has residue {:?}, the fixture records {:?}",
                    cds_name,
                    position + 1,
                    codon,
                    place.residue,
                    residue
                );
            }
        }
    }
}

#[test]
fn every_base_of_a_small_fixture_cds_maps_to_its_row() {
    for pair in [
        ("synthetic", "p1"),
        ("synthetic", "m1"),
        ("synthetic", "pj"),
        ("synlin", "synA"),
        ("frames62", "fP"),
        ("frames62", "fM"),
    ] {
        let (cds, map): (Cds, CdsMap) = subject(pair.0, pair.1);
        let rows: Vec<Vec<String>> = base_rows(pair.0, pair.1);
        // The table is the whole map: its rows cover every base the CDS selects
        // exactly once, and no base that the CDS does not select.
        let listed: Vec<u64> = rows.iter().map(|row| zero_based(number(&row[3]))).collect();
        let mut given_positions: Vec<u64> = selected_positions(&cds);
        let mut listed_sorted: Vec<u64> = listed.clone();
        given_positions.sort();
        listed_sorted.sort();
        assert!(
            listed_sorted == given_positions,
            "{} {}: cds-map-bases.tsv lists {} bases of this CDS, and the CDS selects {:?}",
            pair.0,
            pair.1,
            listed_sorted.len(),
            given_positions.len()
        );
        for row in rows.iter() {
            let symbol: String = cds.strand().symbol();
            assert!(
                row[2] == symbol,
                "{} {}: the base row says strand {}, the imported CDS is on {}",
                pair.0,
                pair.1,
                row[2],
                symbol
            );
            let position: u64 = zero_based(number(&row[3]));
            let place: CodonPlace = map.base_residue(position).unwrap_or_else(|| {
                panic!(
                    "{} {}: base_residue({}) maps to nothing, cds-map-bases.tsv gives it codon {} place {}",
                    pair.0,
                    pair.1,
                    position + 1,
                    row[4],
                    row[5]
                )
            });
            assert!(
                place.codon == number(&row[4]),
                "{} {}: base {} is in codon {}, cds-map-bases.tsv records codon {}",
                pair.0,
                pair.1,
                position + 1,
                place.codon,
                row[4]
            );
            assert!(
                place.place == number(&row[5]),
                "{} {}: base {} has place {} in its codon, cds-map-bases.tsv records place {}",
                pair.0,
                pair.1,
                position + 1,
                place.place,
                row[5]
            );
            // A `stop` row's residue column has no number, and `base_residue`
            // answers `residue: None` for exactly that codon (D010).
            assert!(
                place.residue == fixture_residue(&row[6]),
                "{} {}: base {} has residue {:?}, cds-map-bases.tsv records {:?}",
                pair.0,
                pair.1,
                position + 1,
                place.residue,
                fixture_residue(&row[6])
            );
        }
    }
}

#[test]
fn residue_bases_inverts_base_residue_for_every_fixture_base() {
    for pair in [
        ("synthetic", "p1"),
        ("synthetic", "m1"),
        ("synthetic", "pj"),
        ("synlin", "synA"),
        ("frames62", "fP"),
        ("frames62", "fM"),
    ] {
        let (cds, map): (Cds, CdsMap) = subject(pair.0, pair.1);
        let rows: Vec<Vec<String>> = base_rows(pair.0, pair.1);
        // Base → residue → base: every base row with a residue number must come
        // back from `residue_bases` of that number, and the three bases of one
        // residue are exactly the three rows that name it, in the reading order
        // the file lists them.
        for residue in 1..=map.residues() {
            let listed: Vec<u64> = rows
                .iter()
                .filter(|row| fixture_residue(&row[6]) == Some(residue))
                .map(|row| zero_based(number(&row[3])))
                .collect();
            assert!(
                listed.len() == 3,
                "{} {}: {} base rows name residue {}, expected 3",
                pair.0,
                pair.1,
                listed.len(),
                residue
            );
            let given: Vec<u64> = map
                .residue_bases(residue)
                .unwrap_or_else(|error| {
                    panic!(
                        "{} {}: residue_bases({}) is refused: {:?}",
                        pair.0,
                        pair.1,
                        residue,
                        error.message()
                    )
                })
                .to_vec();
            assert!(
                given == listed,
                "{} {}: residue_bases({}) reads {:?}, the base rows list {:?}",
                pair.0,
                pair.1,
                residue,
                given,
                listed
            );
            for position in listed.iter() {
                assert!(
                    given.contains(position),
                    "{} {}: residue_bases({}) does not contain base {}, which cds-map-bases.tsv gives that residue",
                    pair.0,
                    pair.1,
                    residue,
                    position + 1
                );
            }
            for position in given.iter() {
                let place: CodonPlace = map.base_residue(*position).unwrap_or_else(|| {
                    panic!(
                        "{} {}: base_residue({}) maps to nothing although residue_bases({}) returns it",
                        pair.0,
                        pair.1,
                        position + 1,
                        residue
                    )
                });
                assert!(
                    place.residue == Some(residue),
                    "{} {}: base {} of residue_bases({}) maps back to residue {:?}, expected Some({})",
                    pair.0,
                    pair.1,
                    position + 1,
                    residue,
                    place.residue,
                    residue
                );
            }
        }
        // The codon table, read the other way: `residue_bases` must give a
        // codon's three positions in reading order, and its ascending column is
        // the same three bases sorted (D040).
        for row in codon_rows(CODONS_FILE, pair.0, pair.1).iter() {
            let codon: u64 = number(&row[4]);
            let residue: Option<u64> = fixture_residue(&row[5]);
            let ascending: Vec<u64> = fixture_positions(&row[7]);
            let reading: Vec<u64> = fixture_reading_order(&cds, ascending.as_slice());
            // The base table lists the same codon's bases in reading order, and
            // it is in reading order because the fixture says so; the two
            // derivations of that order must agree.
            let from_bases: Vec<u64> = rows
                .iter()
                .filter(|base| number(&base[4]) == codon)
                .map(|base| zero_based(number(&base[3])))
                .collect();
            assert!(
                reading == from_bases,
                "{} {}: codon {} reads {:?} by its intervals, the base rows list {:?}",
                pair.0,
                pair.1,
                codon,
                reading,
                from_bases
            );
            // Settled ruling from the planner: a codon row whose residue column
            // says `stop` is not checked with `residue_bases` — the terminal
            // stop codon has no residue number to ask with (D010), and the
            // refusal of `residues() + 1` is what
            // `a_number_with_no_residue_is_refused` pins. Such a row is checked
            // through `base_residue`, which must report the codon with
            // `residue: None`. Only rows with a numeric residue go through
            // `residue_bases`.
            if residue.is_none() {
                assert!(
                    reading.len() == ascending.len(),
                    "{} {}: the stop codon {} lists {} reading positions and {} ascending ones",
                    pair.0,
                    pair.1,
                    codon,
                    reading.len(),
                    ascending.len()
                );
                for indexed in reading.iter().enumerate() {
                    let place_index: usize = indexed.0;
                    let position: u64 = *indexed.1;
                    let place: CodonPlace = map.base_residue(position).unwrap_or_else(|| {
                        panic!(
                            "{} {}: base_residue({}) maps to nothing, the stop codon {} covers {:?}",
                            pair.0,
                            pair.1,
                            position + 1,
                            codon,
                            ascending
                        )
                    });
                    assert!(
                        place.codon == codon && place.residue.is_none(),
                        "{} {}: the stop codon's base {} maps to {:?}, expected codon {} with residue None",
                        pair.0,
                        pair.1,
                        position + 1,
                        place,
                        codon
                    );
                    assert!(
                        place.place == (place_index as u64) + 1,
                        "{} {}: the stop codon's base {} has place {}, expected place {}",
                        pair.0,
                        pair.1,
                        position + 1,
                        place.place,
                        place_index + 1
                    );
                }
                continue;
            }
            // `residue_bases` asks by residue number. In these tables the
            // residue column equals the codon column for a numbered row,
            // because both count from the first translated codon and D010 only
            // removes a trailing stop codon; the test asserts that equality
            // rather than assuming it.
            let asked: u64 = match residue {
                None => continue,
                Some(value) => value,
            };
            assert!(
                codon == asked,
                "{} {}: the codon row numbers codon {} residue {}; this test asks residue_bases by residue",
                pair.0,
                pair.1,
                codon,
                asked
            );
            let given: Vec<u64> = map
                .residue_bases(asked)
                .unwrap_or_else(|error| {
                    panic!(
                        "{} {}: residue_bases({}) is refused: {:?}",
                        pair.0,
                        pair.1,
                        asked,
                        error.message()
                    )
                })
                .to_vec();
            assert!(
                given == reading,
                "{} {}: residue_bases({}) reads {:?}, the codon row's reading order is {:?}",
                pair.0,
                pair.1,
                asked,
                given,
                reading
            );
            let mut sorted: Vec<u64> = given.clone();
            sorted.sort();
            assert!(
                sorted == ascending,
                "{} {}: residue_bases({}) covers {:?}, the codon row's ascending column is {:?}",
                pair.0,
                pair.1,
                asked,
                sorted,
                ascending
            );
        }
    }
}

#[test]
fn a_number_with_no_residue_is_refused() {
    for row in table(COUNTS_FILE, COUNTS_HEADER).iter() {
        let (_cds, map): (Cds, CdsMap) = subject(&row[0], &row[1]);
        let residues: u64 = number(&row[5]);
        assert!(
            map.residues() == residues,
            "{} {}: the map has {} residues, the fixture records {}",
            row[0],
            row[1],
            map.residues(),
            row[5]
        );
        // One past the last residue is the terminal stop codon: it is a codon
        // and not a residue (D010), so the view must not number it. This is
        // also the standing check for every `stop` row of the codon and base
        // tables, which have no residue number to call `residue_bases` with.
        refused_number(&row[0], &row[1], &map, residues + 1);
        refused_number(&row[0], &row[1], &map, 0);
        assert!(
            map.residue_bases(residues).is_ok(),
            "{} {}: residue_bases({}) is the last residue's codon and must answer",
            row[0],
            row[1],
            residues
        );
        // A number far past the residue count must also be refused rather than
        // wrap around into a codon (§4: every multiply and subtract is checked).
        refused_number(&row[0], &row[1], &map, u64::MAX);
    }
}

#[test]
fn frames62_reads_and_both_wrapped_cdss_map() {
    let record: GenBankRecord = record_of("frames62");
    assert!(
        record.sequence().topology().name() == "circular",
        "frames-62.gb reads as {} topology, expected circular",
        record.sequence().topology().name()
    );
    let import: GenBankImport =
        import_cds(&record, &request(vec!["fP", "fM"])).unwrap_or_else(|error| {
            panic!(
                "frames-62.gb: import_cds refused fP and fM: {:?}",
                error.message()
            )
        });
    assert!(
        import.cds().len() == 2,
        "frames-62.gb: the import gives {} CDSs, expected 2",
        import.cds().len()
    );
    // The four spot values of §4, in core's zero-based coordinates (D006).
    let (_m1_cds, m1): (Cds, CdsMap) = subject("synthetic", "m1");
    let given: Vec<u64> = m1
        .residue_bases(1)
        .unwrap_or_else(|error| panic!("m1: residue_bases(1) is refused: {:?}", error.message()))
        .to_vec();
    assert!(
        given == vec![23u64, 22, 21],
        "m1: residue_bases(1) gives {:?}, expected [23, 22, 21]",
        given
    );
    let (_fp_cds, fp): (Cds, CdsMap) = subject("frames62", "fP");
    let given: Vec<u64> = fp
        .residue_bases(3)
        .unwrap_or_else(|error| panic!("fP: residue_bases(3) is refused: {:?}", error.message()))
        .to_vec();
    // The origin jump inside one codon: base 62, then bases 1 and 2.
    assert!(
        given == vec![61u64, 0, 1],
        "fP: residue_bases(3) gives {:?}, expected [61, 0, 1]",
        given
    );
    let (_fm_cds, fm): (Cds, CdsMap) = subject("frames62", "fM");
    let given: Vec<u64> = fm
        .residue_bases(1)
        .unwrap_or_else(|error| panic!("fM: residue_bases(1) is refused: {:?}", error.message()))
        .to_vec();
    // A wrapped minus-strand CDS starts at the complement of the arc's
    // lowest-numbered base, exactly the reading derived by hand for `mj1` and
    // `mj2` in tests/fixtures/genbank/README.md.
    assert!(
        given == vec![7u64, 6, 5],
        "fM: residue_bases(1) gives {:?}, expected [7, 6, 5]",
        given
    );
    let (_tet_cds, tet): (Cds, CdsMap) = subject("pbr322", "tet");
    assert!(
        tet.residues() == 396,
        "pbr322 tet: the map has {} residues, expected 396",
        tet.residues()
    );
    let given: Vec<u64> = tet
        .residue_bases(396)
        .unwrap_or_else(|error| panic!("tet: residue_bases(396) is refused: {:?}", error.message()))
        .to_vec();
    assert!(
        given == vec![1270u64, 1271, 1272],
        "tet: residue_bases(396) gives {:?}, expected [1270, 1271, 1272]",
        given
    );
    refused_number("pbr322", "tet", &tet, 397);
}

#[test]
fn a_refused_cds_never_gets_a_map() {
    for row in table(REFUSED_FILE, REFUSED_HEADER).iter() {
        let record: GenBankRecord = record_of(&row[0]);
        // The pBR322 CDS with no id cannot be asked for by name, so its refusal
        // is read where the import records it: the list of features it did not
        // use. `genbank_fixtures.rs` pins the same feature, index and rule.
        if row[1] == "ROP-feature" {
            let import: GenBankImport = import_cds(&record, &request(vec!["tet", "bla"]))
                .unwrap_or_else(|error| {
                    panic!(
                        "{}: import_cds refused tet and bla: {:?}",
                        row[0],
                        error.message()
                    )
                });
            let found: Vec<UnusedFeature> = import
                .unused()
                .iter()
                .filter(|feature| feature.index() == ROP_INDEX)
                .cloned()
                .collect();
            assert!(
                found.len() == 1,
                "{}: the unused list has {} features at index {}, expected 1",
                row[0],
                found.len(),
                ROP_INDEX
            );
            let rop: UnusedFeature = found[0].clone();
            assert!(
                rop.key() == "CDS" && rop.id().is_none(),
                "{}: the feature at index {} is {:?} with id {:?}, expected a CDS with no id",
                row[0],
                ROP_INDEX,
                rop.key(),
                rop.id()
            );
            match rop.refusal() {
                None => panic!(
                    "{}: the CDS at index {} gives no refusal, expected {}",
                    row[0], ROP_INDEX, row[2]
                ),
                Some(refusal) => {
                    let text: String = refusal_text(refusal);
                    assert!(
                        text == row[2],
                        "{}: the CDS at index {} is refused as {}, the fixture records {}",
                        row[0],
                        ROP_INDEX,
                        text,
                        row[2]
                    );
                }
            }
            continue;
        }
        let outcome: Result<GenBankImport, DomainError> =
            import_cds(&record, &request(vec![row[1].as_str()]));
        match outcome {
            Ok(imported) => panic!(
                "{}: import_cds({:?}) succeeds with {} CDSs; the fixture says {:?} is refused by {}",
                row[0],
                row[1],
                imported.cds().len(),
                row[1],
                row[2]
            ),
            Err(error) => {
                let text: String = refusal_text(error);
                assert!(
                    text == row[2],
                    "{}: importing {:?} is refused as {}, the fixture records {}",
                    row[0],
                    row[1],
                    text,
                    row[2]
                );
            }
        }
    }
}
