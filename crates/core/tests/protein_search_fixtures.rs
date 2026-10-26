//! Protein search in `crates/core/src/protein_search.rs` (8f, plan §4.5) against
//! `tests/fixtures/view/search-protein-patterns.tsv` and
//! `search-protein-hits.tsv`: 40 (record, target, pattern-as-written) groups and
//! their 367 hits over `synthetic` (frames and its usable CDSs `p1`, `m1`, `pj`),
//! `pbr322` (frames and `tet`, `bla`) and `framesamb` (frames, read as linear),
//! with strand, frame, the base run each hit covers, whether it wraps, and the
//! residue or frame-codon numbers it carries (D054, D050 M8-16 and M8-17,
//! D052 M8-2, D010, D036, D006).
//!
//! Like `search_fixtures.rs`, this file applies **no 1-based → zero-based
//! conversion**: `search-protein-hits.tsv`'s `start` is already zero-based, and
//! `first`/`last` are already the CDS's residue numbers and the frame's codon
//! numbers, which is what D050 M8-17 fixes them to be.
//!
//! Groups are keyed by the pattern **as written**, never by the table's
//! `normalized_pattern` column: `MKM` and `mkm` on `pj` fold to one search string
//! and would collapse into one group of two rows — and §4.5 pins that each row is
//! checked against its own answer. `target` is `frames` or a CDS id; a CDS is
//! imported from its own file with `import_cds`, so its intervals, strand and
//! translation are core's reading of the record rather than coordinates written
//! into this test (D041, D042).
//!
//! The `residues` column — the symbols each hit matched — is compared by the
//! reference script itself, which translates every hit's own `top_strand_bases`
//! afresh and requires them to give back `residues`
//! (`reference/search_protein.py::check_hit`). Re-deriving it here would mean
//! writing a second matcher in the test. What this file checks instead is the
//! other half of those coordinates: each row's `top_strand_bases` must equal the
//! bases the `ProteinHit` core returns covers, read out of core's own record and
//! wrapped exactly when the hit says it wraps.
//!
//! The tables are the answers. They come from
//! `tests/fixtures/view/reference/search_protein.py`, which shares no code with
//! `crates/` (D039); its codon table is checked against Biopython's table 11 for
//! all 64 codons, its frames against every `frame-tables.tsv` row, a CDS's codons
//! against the file's own `/translation` and against `cds-map-codons.tsv`, and
//! every hit is found twice inside the script — by its own matcher and by a
//! regular expression over the same symbols. `tests/fixtures/view/README.md`
//! records what Biopython does not answer here: the `X`-matches-the-stop rule,
//! `?` as a complete unreadable codon, and the order hits come in (D054, D010,
//! D052). The Python check runs from the repository root, not from cargo (D039,
//! D026):
//!
//! ```
//! python3 -I tests/fixtures/verify.py
//! ```

use luria_core::cds::Cds;
use luria_core::fasta::read_fasta;
use luria_core::fasta::FastaRecord;
use luria_core::frames::ReadingFrame;
use luria_core::genbank::read_genbank;
use luria_core::genbank::GenBankRecord;
use luria_core::genbank_cds::import_cds;
use luria_core::genbank_cds::GenBankImport;
use luria_core::identity::Identifier;
use luria_core::protein_search::find_protein;
use luria_core::protein_search::ProteinHit;
use luria_core::protein_search::ProteinTarget;
use luria_core::sequence::Sequence;
use luria_core::sequence::SequenceTopology;
use luria_core::strand::Strand;

const VIEW: &str = "../../tests/fixtures/view/";
const GENBANK: &str = "../../tests/fixtures/genbank/";
const PATTERNS_FILE: &str = "search-protein-patterns.tsv";
const HITS_FILE: &str = "search-protein-hits.tsv";
/// One row per (record, target, pattern as written): what it folds to, its
/// length in letters, how many hit rows follow for that group, and what the row
/// is there to pin. A pattern that finds nothing can only be pinned here.
const PATTERNS_HEADER: &str =
    "record\ttarget\tpattern\tnormalized_pattern\tpattern_length\thit_count\tpurpose";
/// One row per hit, in the order core is expected to return them (D054). `start`
/// is already zero-based (D006); `frame` is `1`, `2`, `3` or `-` for a CDS hit;
/// `first` and `last` are `stop` for a CDS's stop codon, which is a codon and not
/// a residue (D010).
const HITS_HEADER: &str = "record\ttarget\tpattern\thit_index\tstrand\tframe\tstart\tlength\twraps_origin\tfirst\tlast\tresidues\ttop_strand_bases";
const PATTERNS_COLUMNS: usize = 7usize;
const PATTERNS_ROWS: usize = 40usize;
const HITS_COLUMNS: usize = 13usize;
const HITS_ROWS: usize = 367usize;
const PBR322_FILE: &str = "raw/J01749.1.gb";
const SYNTHETIC_FILE: &str = "synthetic.gb";
const FRAMESAMB_FILE: &str = "frames-amb.fa";
/// The `target` column value that means "the six frames of the record" rather
/// than a CDS id.
const FRAMES: &str = "frames";

/// The rows of a fixture TSV under `tests/fixtures/view/`, refusing a header row
/// it does not have.
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

/// The rows of one table, refusing a row that does not have its full columns.
fn rows_of(file: &str, header: &str, columns: usize) -> Vec<Vec<String>> {
    let rows: Vec<Vec<String>> = table(file, header);
    for row in rows.iter() {
        assert!(
            row.len() == columns,
            "{}: row {:?} has {} columns, expected {}",
            file,
            row,
            row.len(),
            columns
        );
    }
    rows
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

/// The file a fixture `record` column names, and whether it is FASTA. A FASTA
/// record says nothing about topology (D049) and holds no CDS, which is why
/// `framesamb` is read as linear here and appears in these tables with the
/// `frames` target only.
fn fixture_file(value: &str) -> (&'static str, &'static str, bool) {
    match value {
        "pbr322" => (GENBANK, PBR322_FILE, false),
        "synthetic" => (GENBANK, SYNTHETIC_FILE, false),
        "framesamb" => (VIEW, FRAMESAMB_FILE, true),
        other => panic!("{}: no fixture record of that name", other),
    }
}

/// The record a fixture `record` column names, read with luria's own reader, and
/// its GenBank record when the file is GenBank — the same source the CDS ids in
/// the `target` column come from, so a CDS can never be compared against a
/// sequence built some other way. `framesamb` comes through `read_fasta` with the
/// linear assumption D049 gives a FASTA.
fn fixture_record(value: &str) -> (Sequence, Option<GenBankRecord>) {
    let (dir, file, fasta) = fixture_file(value);
    let text: String = read_input(dir, file);
    if fasta {
        let record: FastaRecord = read_fasta(name(file), text, SequenceTopology::Linear)
            .unwrap_or_else(|error| {
                panic!("{}: read_fasta refused it: {:?}", file, error.message())
            });
        return (record.sequence(), None);
    }
    let record: GenBankRecord = read_genbank(name(file), text)
        .unwrap_or_else(|error| panic!("{}: read_genbank refused it: {:?}", file, error.message()));
    let sequence: Sequence = record.sequence();
    (sequence, Some(record))
}

/// The CDS a fixture `target` column names, imported the way the view imports a
/// selected CDS (D041, D042). An id the tables call usable and `import_cds`
/// refuses fails the test rather than quietly dropping its rows.
fn cds_of(record: &GenBankRecord, id: &str) -> Cds {
    let wanted: Identifier = name(id);
    let import: GenBankImport = import_cds(record, &[wanted])
        .unwrap_or_else(|error| panic!("import_cds refused {:?}: {:?}", id, error.message()));
    assert!(
        import.cds().len() == 1,
        "importing {:?} gives {} CDSs, expected 1",
        id,
        import.cds().len()
    );
    import.cds()[0].cds()
}

/// A fixture number. `start` and `length` are compared as they stand, and
/// `first`/`last` are the numbers core carries, with no conversion (D006).
fn number(text: &str) -> u64 {
    text.parse::<u64>()
        .unwrap_or_else(|error| panic!("cannot read {:?} as a number: {}", text, error))
}

/// A yes/no column. Anything else is a table this test does not understand.
fn yes(text: &str) -> bool {
    match text {
        "yes" => true,
        "no" => false,
        other => panic!("{:?}: expected \"yes\" or \"no\"", other),
    }
}

/// The `strand` column. A protein hit is never `both` (§4.5): a pattern of
/// residues is read on one strand at a time, unlike a DNA pattern.
fn strand(text: &str) -> Strand {
    match text {
        "+" => Strand::Plus,
        "-" => Strand::Minus,
        other => panic!("{:?}: a protein strand is + or -, never {:?}", text, other),
    }
}

/// The `frame` column: `1`, `2`, `3`, or `-` for a CDS hit, which has no frame.
fn frame(text: &str) -> Option<ReadingFrame> {
    match text {
        "-" => None,
        digits => match digits.parse::<u64>() {
            Ok(value) => Some(ReadingFrame::from_index(value).unwrap_or_else(|| {
                panic!(
                    "{:?}: no reading frame numbers a frame beyond 1, 2 and 3",
                    text
                )
            })),
            Err(error) => panic!("cannot read {:?} as a frame: {}", text, error),
        },
    }
}

/// A `first` or `last` column: a number, or `stop` for `None`, which is what a
/// CDS's terminal stop codon carries because it is not a residue (D010,
/// D050 M8-17).
fn residue_number(text: &str) -> Option<u64> {
    match text {
        "stop" => None,
        digits => Some(number(digits)),
    }
}

/// The hit rows of one (record, target, pattern-as-written) group, in file order.
fn group_of(hits: &[Vec<String>], record: &str, target: &str, pattern: &str) -> Vec<Vec<String>> {
    hits.iter()
        .filter(|row| row[0] == record && row[1] == target && row[2] == pattern)
        .cloned()
        .collect()
}

/// One hit row as one short line, for failure messages: strand, frame, base run,
/// and the two numbers — the same shape as the row itself.
fn hit_row(row: &[String]) -> String {
    format!(
        "{}{}@{}+{}#{}-{}{}",
        row[4],
        row[5],
        row[6],
        row[7],
        row[9],
        row[10],
        if yes(&row[8]) { " wraps" } else { "" }
    )
}

/// One core hit in the same shape, so a failure prints both sides alike.
fn core_row(hit: &ProteinHit) -> String {
    format!(
        "{}{}@{}+{}#{}-{}{}",
        hit.strand.clone().symbol(),
        match hit.frame {
            Some(value) => value.index().to_string(),
            None => "-".to_string(),
        },
        hit.start,
        hit.length,
        match hit.first {
            Some(value) => value.to_string(),
            None => "stop".to_string(),
        },
        match hit.last {
            Some(value) => value.to_string(),
            None => "stop".to_string(),
        },
        if hit.wraps_origin { " wraps" } else { "" }
    )
}

/// The bases of the span a hit covers, read back out of the `Sequence` core was
/// given and wrapped modulo the record length exactly when the hit wraps — the
/// independent check of the coordinates, because `top_strand_bases` came from the
/// reference script and this reads core's own record (D006, D036).
fn window_bases(sequence: &Sequence, start: u64, length: u64, wraps: bool) -> String {
    let size: u64 = sequence.length();
    let mut text: String = String::new();
    let mut offset: u64 = 0;
    while offset < length {
        let position: u64 = if wraps {
            (start + offset) % size
        } else {
            start + offset
        };
        let letter: char = sequence.letter_at(position).unwrap_or_else(|error| panic!(
            "the span at zero-based {} of '{}' (+{}{}) needs base {}, which the record does not have: {}",
            start,
            sequence.display_name(),
            length,
            if wraps { ", wrapping" } else { "" },
            position,
            error.message()
        ));
        text.push(letter);
        offset += 1;
    }
    text
}

/// Rules 1 to 6 together, over every group the tables hold: for every pattern
/// row, core's hits are exactly that group's rows — count, order, strand, frame,
/// coordinates, wrapping and residue numbers — and every row's `top_strand_bases`
/// are the bases the hit core returns actually covers.
#[test]
fn every_protein_pattern_row_matches() {
    let patterns: Vec<Vec<String>> = rows_of(PATTERNS_FILE, PATTERNS_HEADER, PATTERNS_COLUMNS);
    assert!(
        patterns.len() == PATTERNS_ROWS,
        "{}: {} rows, expected {}",
        PATTERNS_FILE,
        patterns.len(),
        PATTERNS_ROWS
    );
    let hits: Vec<Vec<String>> = rows_of(HITS_FILE, HITS_HEADER, HITS_COLUMNS);
    assert!(
        hits.len() == HITS_ROWS,
        "{}: {} rows, expected {}",
        HITS_FILE,
        hits.len(),
        HITS_ROWS
    );
    // A hit row for a group the patterns table does not name is a failure, not an
    // extra table to ignore.
    for row in hits.iter() {
        assert!(
            patterns.iter().any(|pattern_row| pattern_row[0] == row[0]
                && pattern_row[1] == row[1]
                && pattern_row[2] == row[2]),
            "{}: no pattern row for record {:?} target {:?} pattern {:?}",
            HITS_FILE,
            row[0],
            row[1],
            row[2]
        );
    }

    let mut grouped: usize = 0usize;
    let mut windows: usize = 0usize;
    for pattern_row in patterns.iter() {
        let record: &str = &pattern_row[0];
        let target: &str = &pattern_row[1];
        // The pattern as written, never the `normalized_pattern` column: `MKM` and
        // `mkm` fold to one search string and must still be checked apart.
        let pattern: &str = &pattern_row[2];
        let group: Vec<Vec<String>> = group_of(&hits, record, target, pattern);
        let want: Vec<String> = group.iter().map(|row| hit_row(row)).collect();
        // The group's size is pinned by the table's own count, and a group with no
        // rows at all fails here when the row claims hits — which is how a missing
        // group shows up rather than passing as an empty answer.
        assert!(
            number(&pattern_row[5]) == group.len() as u64,
            "{}: {} for record {:?} target {:?} pattern {:?}, but {} hit rows carry that triple: {:?}",
            PATTERNS_FILE,
            pattern_row[5],
            record,
            target,
            pattern,
            group.len(),
            want
        );
        assert!(
            number(&pattern_row[4]) == pattern.chars().count() as u64,
            "{}: pattern {:?} is {} letters, the table's pattern_length says {}",
            PATTERNS_FILE,
            pattern,
            pattern.chars().count(),
            pattern_row[4]
        );
        // `hit_index` runs 1..=n in file order, so the table states the order its
        // own rows must come back in (D054).
        for (index, row) in group.iter().enumerate() {
            assert!(
                number(&row[3]) == (index + 1) as u64,
                "{}: record {:?} target {:?} pattern {:?}: hit_index {:?} at row {} of its group",
                HITS_FILE,
                record,
                target,
                pattern,
                row[3],
                index + 1
            );
        }

        let (sequence, record_of_cds) = fixture_record(record);
        let selected: Option<Cds> = match target {
            FRAMES => None,
            id => Some(cds_of(
                record_of_cds.as_ref().unwrap_or_else(|| panic!(
                    "{}: target {:?} names a CDS, and the record {:?} it was read from holds no CDSs",
                    HITS_FILE, id, record
                )),
                id,
            )),
        };
        let chosen: ProteinTarget<'_> = match selected.as_ref() {
            Some(cds) => ProteinTarget::Cds(cds),
            None => ProteinTarget::SixFrames,
        };
        let found: Vec<ProteinHit> = match find_protein(&sequence, chosen, pattern) {
            Ok(found) => found,
            Err(error) => panic!(
                "find_protein refused {:?} on record {:?} target {:?} ({}: {}); the tables search it and name {} hit(s)",
                pattern,
                record,
                target,
                error.code(),
                error.message(),
                group.len()
            ),
        };
        let core_rows: Vec<String> = found.iter().map(core_row).collect();
        // A missing group fails here too: the count above has already pinned that
        // the table names hits for this triple, so an empty answer from core is a
        // count mismatch, and the purpose column says what was expected.
        assert!(
            found.len() == group.len(),
            "record {:?} target {:?} pattern {:?}: core gives {} hits ({}), the table names {} ({}); purpose: {}",
            record,
            target,
            pattern,
            found.len(),
            core_rows.join(", "),
            group.len(),
            want.join(", "),
            pattern_row[6]
        );

        let mut previous: Option<(u64, bool)> = None;
        for (index, row) in group.iter().enumerate() {
            let hit: &ProteinHit = &found[index];
            assert!(
                hit.strand == strand(&row[4]),
                "record {:?} target {:?} pattern {:?} hit {}: strand {:?}, the table says {:?}",
                record,
                target,
                pattern,
                index + 1,
                hit.strand.clone().symbol(),
                row[4]
            );
            assert!(
                hit.frame == frame(&row[5]),
                "record {:?} target {:?} pattern {:?} hit {}: frame {:?}, the table says {:?}",
                record,
                target,
                pattern,
                index + 1,
                hit.frame.map(|value| value.index()),
                row[5]
            );
            assert!(
                hit.start == number(&row[6]),
                "record {:?} target {:?} pattern {:?} hit {}: zero-based start {}, the table says {}; core {} / table {}",
                record,
                target,
                pattern,
                index + 1,
                hit.start,
                row[6],
                core_row(hit),
                hit_row(row)
            );
            assert!(
                hit.length == number(&row[7]),
                "record {:?} target {:?} pattern {:?} hit {}: length {}, the table says {}",
                record,
                target,
                pattern,
                index + 1,
                hit.length,
                row[7]
            );
            assert!(
                hit.wraps_origin == yes(&row[8]),
                "record {:?} target {:?} pattern {:?} hit {}: wraps_origin {}, the table says {}",
                record,
                target,
                pattern,
                index + 1,
                hit.wraps_origin,
                row[8]
            );
            assert!(
                hit.first == residue_number(&row[9]),
                "record {:?} target {:?} pattern {:?} hit {}: first {}, the table says {}",
                record,
                target,
                pattern,
                index + 1,
                match hit.first {
                    Some(value) => value.to_string(),
                    None => "stop".to_string(),
                },
                row[9]
            );
            assert!(
                hit.last == residue_number(&row[10]),
                "record {:?} target {:?} pattern {:?} hit {}: last {}, the table says {}",
                record,
                target,
                pattern,
                index + 1,
                match hit.last {
                    Some(value) => value.to_string(),
                    None => "stop".to_string(),
                },
                row[10]
            );

            // D054's order, checked on core's own hits rather than inferred from
            // the rows: ascending `start`, and a six-frame tie decided plus first.
            let key: (u64, bool) = (hit.start, hit.strand.clone() == Strand::Minus);
            if let Some(seen) = previous {
                assert!(
                    seen <= key,
                    "record {:?} target {:?} pattern {:?}: hits must come left to right by base, plus before minus on a tie, but {} came before {}",
                    record,
                    target,
                    pattern,
                    core_row(&found[index - 1]),
                    core_row(hit)
                );
            }
            previous = Some(key);

            // The coordinates, read back as bases: the run core reports covers
            // exactly what the reference script wrote into `top_strand_bases`.
            let bases: String = window_bases(&sequence, hit.start, hit.length, hit.wraps_origin);
            assert!(
                bases == row[12],
                "record {:?} target {:?} pattern {:?} hit {}: the run core reports ({}) covers {:?}, the table's top_strand_bases is {:?}",
                record,
                target,
                pattern,
                index + 1,
                core_row(hit),
                bases,
                row[12]
            );
            windows += 1;
        }
        grouped += group.len();
    }
    assert!(
        grouped == hits.len() && windows == HITS_ROWS,
        "{} of {} hit rows belong to a pattern row and {} windows were read back; every group must be checked, not skipped",
        grouped,
        hits.len(),
        windows
    );
}
