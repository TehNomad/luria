//! DNA search in `crates/core/src/search.rs` (8e, plan §4.4) against
//! `tests/fixtures/view/search-letters.tsv`, `search-dna-patterns.tsv` and
//! `search-dna-hits.tsv`: which pattern letters may match a record letter at
//! an aligned position (all 225 pairs), and every hit of 44 patterns over five
//! records — 4542 hits, with strand, coordinates, wrapping and the bases each
//! span covers (D052 M8-7, M8-8 and M8-16; D050 M8-16; D049, D006).
//!
//! Unlike `cds_map_fixtures.rs`, this file applies **no 1-based → zero-based
//! conversion**. `search-dna-hits.tsv`'s `start` column is already zero-based,
//! as the file table in `tests/fixtures/view/README.md` states, so it is
//! compared with `DnaHit::start` exactly as it stands; adding the conversion
//! here would move every hit by one base.
//!
//! Hit rows are grouped by the pattern **as written**, never by the table's
//! `normalized_pattern` column: `GAATTC`, `gaattc` and `GAATUC` fold to the
//! same search string and would collapse into one group, which is exactly what
//! rule 2 of §4.4 says must not happen — each of the three rows is checked on
//! its own against its own hits.
//!
//! The tables are the answers. They come from
//! `tests/fixtures/view/reference/search_dna.py`, which shares no code with
//! `crates/` (D039) and hand-codes §4.4's containment rule; Biopython is its
//! oracle only for the letter sets, the complement table and site counts over
//! records with no ambiguity codes, and `tests/fixtures/view/README.md`
//! records that limit rather than arguing it away. The Python check runs from
//! the repository root, not from cargo (D039, D026):
//!
//! ```
//! python3 -I tests/fixtures/verify.py
//! ```
//!
//! Nothing here re-derives a rule: no second copy of the matching rule, no
//! reverse-complement arithmetic, no folded pattern. Core is asked what it
//! finds and the tables say whether that is right.

use luria_core::fasta::read_fasta;
use luria_core::fasta::FastaRecord;
use luria_core::genbank::read_genbank;
use luria_core::genbank::GenBankRecord;
use luria_core::identity::Identifier;
use luria_core::search::covers;
use luria_core::search::find_dna;
use luria_core::search::DnaHit;
use luria_core::sequence::bases_of;
use luria_core::sequence::Sequence;
use luria_core::sequence::SequenceTopology;
use luria_core::sequence::DNA_ALPHABET;

const VIEW: &str = "../../tests/fixtures/view/";
const GENBANK: &str = "../../tests/fixtures/genbank/";
const LETTERS_FILE: &str = "search-letters.tsv";
const PATTERNS_FILE: &str = "search-dna-patterns.tsv";
const HITS_FILE: &str = "search-dna-hits.tsv";
/// One row per ordered pair of IUPAC letters: what §4.4's containment rule
/// answers, what the rejected intersection rule would answer, and whether the
/// two rules agree. `pattern_letter` comes first, `template_letter` second —
/// the reverse of `covers(template, pattern)`'s argument order.
const LETTERS_HEADER: &str =
    "pattern_letter\ttemplate_letter\tsubset_match\tintersection_match\trules_agree";
/// One row per (record, pattern as written): the record it was searched over,
/// the pattern, its reverse complement as the reference script computed it, and
/// how many hit rows follow for that pair.
const PATTERNS_HEADER: &str = "record\tseq_len\ttopology\twrap_used\tpattern\tnormalized_pattern\tpattern_length\trc_pattern\thit_count\tpurpose";
/// One row per hit, in the order core is expected to return them. `start` is
/// already zero-based (D006), which is why this file has no `zero_based()`.
const HITS_HEADER: &str =
    "record\tpattern\thit_index\tstrand\tstart\tlength\twraps_origin\ttop_strand_bases";
const LETTERS_COLUMNS: usize = 5usize;
const LETTERS_ROWS: usize = 225usize;
const LETTERS: usize = 15usize;
/// Rows where containment and intersection answer differently, counted from
/// the table's own columns and asserted against the table's own count, so an
/// accidentally narrower grid cannot pass (README: all 110 are
/// intersection-yes, luria-no).
const DISAGREEING_ROWS: usize = 110usize;
const PATTERNS_COLUMNS: usize = 10usize;
const PATTERNS_ROWS: usize = 44usize;
const HITS_COLUMNS: usize = 8usize;
const HITS_ROWS: usize = 4542usize;
const PBR322_FILE: &str = "raw/J01749.1.gb";
const SYNTHETIC_FILE: &str = "synthetic.gb";
const LINEAR_FILE: &str = "synthetic-linear.gb";
const FRAMES62_FILE: &str = "frames-62.gb";
const FRAMESAMB_FILE: &str = "frames-amb.fa";

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
/// record says nothing about topology (D049), which is why its `topology`
/// column reads `unknown` while the view — and this test — assume linear.
fn fixture_file(value: &str) -> (&'static str, &'static str, bool) {
    match value {
        "pbr322" => (GENBANK, PBR322_FILE, false),
        "synthetic" => (GENBANK, SYNTHETIC_FILE, false),
        "synlin" => (GENBANK, LINEAR_FILE, false),
        "frames62" => (VIEW, FRAMES62_FILE, false),
        "framesamb" => (VIEW, FRAMESAMB_FILE, true),
        other => panic!("{}: no fixture record of that name", other),
    }
}

/// The sequence a fixture `record` column names, read with luria's own reader:
/// `read_genbank` for a GenBank file, `read_fasta` with the linear assumption
/// D049 gives a FASTA — the same assumption that makes `framesamb`'s
/// `wrap_used` column read `no`. Each row reads its record from its own file,
/// so a table can never be compared against a record built some other way.
fn fixture_record(value: &str) -> Sequence {
    let file: (&'static str, &'static str, bool) = fixture_file(value);
    let text: String = read_input(file.0, file.1);
    if file.2 {
        let record: FastaRecord = read_fasta(name(file.1), text, SequenceTopology::Linear)
            .unwrap_or_else(|error| {
                panic!("{}: read_fasta refused it: {:?}", file.1, error.message())
            });
        return record.sequence();
    }
    let record: GenBankRecord = read_genbank(name(file.1), text).unwrap_or_else(|error| {
        panic!("{}: read_genbank refused it: {:?}", file.1, error.message())
    });
    record.sequence()
}

/// A fixture number. `start` is compared as it stands: it is already
/// zero-based, so there is no conversion to apply (D006).
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

/// A letter column that must hold exactly one character.
fn one_letter(text: &str) -> char {
    let letters: Vec<char> = text.chars().collect();
    assert!(
        letters.len() == 1,
        "{:?}: a letter column must hold exactly one character",
        text
    );
    letters[0]
}

/// The hit rows for one (record, pattern-as-written) pair, in file order.
fn group_of(hits: &[Vec<String>], record: &str, pattern: &str) -> Vec<Vec<String>> {
    hits.iter()
        .filter(|row| row[0] == record && row[1] == pattern)
        .cloned()
        .collect()
}

/// One hit row as one short line, for failure messages.
fn hit_row(row: &[String]) -> String {
    format!(
        "{}@{}+{}{}",
        row[3],
        row[4],
        row[5],
        if yes(&row[6]) { " wraps" } else { "" }
    )
}

/// One core hit in the same shape, so a failure prints both sides alike.
fn core_row(hit: &DnaHit) -> String {
    format!(
        "{}@{}+{}{}",
        hit.strand.symbol(),
        hit.start,
        hit.length,
        if hit.wraps_origin { " wraps" } else { "" }
    )
}

/// The bases of the span a hit covers, read back out of the `Sequence` itself
/// and wrapped modulo the record length exactly when the hit wraps. This is an
/// independent check of the coordinates, because the `top_strand_bases` column
/// came from the Python reference and this reads core's own stored record.
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

/// Rule 1, over the whole grid: `covers` answers exactly what the table's
/// `subset_match` column says, and the rejected intersection rule really does
/// disagree on the 110 rows the table marks as disagreeing.
#[test]
fn the_letter_table_pins_covers_over_every_pair_of_iupac_letters() {
    let rows: Vec<Vec<String>> = rows_of(LETTERS_FILE, LETTERS_HEADER, LETTERS_COLUMNS);
    assert!(
        rows.len() == LETTERS_ROWS,
        "{}: {} rows, expected {} (the 15 x 15 grid of IUPAC letters)",
        LETTERS_FILE,
        rows.len(),
        LETTERS_ROWS
    );
    let mut patterns: std::collections::BTreeSet<char> = std::collections::BTreeSet::new();
    let mut templates: std::collections::BTreeSet<char> = std::collections::BTreeSet::new();
    let mut disagreeing: usize = 0usize;
    for row in rows.iter() {
        let pattern: char = one_letter(&row[0]);
        let template: char = one_letter(&row[1]);
        let subset: bool = yes(&row[2]);
        let intersection: bool = yes(&row[3]);
        let agree: bool = yes(&row[4]);
        patterns.insert(pattern);
        templates.insert(template);
        assert!(
            bases_of(template).is_some() && bases_of(pattern).is_some(),
            "{}: {:?}/{:?} is outside the IUPAC alphabet core uses",
            LETTERS_FILE,
            template,
            pattern
        );
        // The table's `subset_match` is what core must answer.
        assert!(
            covers(template, pattern) == subset,
            "{}: covers(template {:?}, pattern {:?}) is {}, the table's subset_match is {} ({})",
            LETTERS_FILE,
            template,
            pattern,
            covers(template, pattern),
            subset,
            row.join("\t")
        );
        // `rules_agree` says whether the two rules part company, and core
        // answers as `subset_match`, so a disagreeing row is a row where the
        // intersection answer differs from what core does.
        assert!(
            (intersection != subset) == !agree,
            "{}: pattern {:?} template {:?} answers subset {} and intersection {} but rules_agree says {:?}",
            LETTERS_FILE,
            pattern,
            template,
            subset,
            intersection,
            agree
        );
        if !agree {
            disagreeing += 1;
        }
    }
    assert!(
        disagreeing == DISAGREEING_ROWS,
        "{}: {} rows disagree with an intersection rule, expected {}; a grid that quietly lost rows would pass the count above",
        LETTERS_FILE,
        disagreeing,
        DISAGREEING_ROWS
    );
    assert!(
        patterns.len() == LETTERS && templates.len() == LETTERS,
        "{}: the grid holds {} pattern letters and {} template letters, expected {} each ({})",
        LETTERS_FILE,
        patterns.len(),
        templates.len(),
        LETTERS,
        DNA_ALPHABET
    );
}

/// Rules 2, 3, 4, 5, 6, 7, 8, 9, 10 and 12 together: for every pattern row,
/// core's hits are exactly that group's rows — count, order, strand,
/// coordinates, wrapping — and no row is skipped.
#[test]
fn core_finds_exactly_the_hits_the_dna_tables_name() {
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
    // A hit row for a pair the patterns table does not name is a failure, not
    // an extra table to ignore.
    for row in hits.iter() {
        assert!(
            patterns
                .iter()
                .any(|pattern_row| pattern_row[0] == row[0] && pattern_row[4] == row[1]),
            "{}: no pattern row for record {:?} pattern {:?}",
            HITS_FILE,
            row[0],
            row[1]
        );
    }
    let mut grouped: usize = 0usize;
    for pattern_row in patterns.iter() {
        let record: &str = &pattern_row[0];
        // The pattern as written, never the `normalized_pattern` column:
        // `GAATTC`, `gaattc` and `GAATUC` fold alike and would collapse.
        let pattern: &str = &pattern_row[4];
        let group: Vec<Vec<String>> = group_of(&hits, record, pattern);
        let rows: Vec<String> = group.iter().map(|row| hit_row(row)).collect();
        assert!(
            number(&pattern_row[8]) == group.len() as u64,
            "{}: {} for record {:?} pattern {:?}, but {} hit rows carry that pair: {:?}",
            PATTERNS_FILE,
            pattern_row[8],
            record,
            pattern,
            group.len(),
            rows
        );
        // `hit_index` runs 1..=n in file order, so the file itself states the
        // order the hits must come back in.
        for (index, row) in group.iter().enumerate() {
            assert!(
                number(&row[2]) == (index + 1) as u64,
                "{}: record {:?} pattern {:?}: hit_index {:?} at row {} of its group",
                HITS_FILE,
                record,
                pattern,
                row[2],
                index + 1
            );
        }
        let sequence: Sequence = fixture_record(record);
        assert!(
            number(&pattern_row[1]) == sequence.length(),
            "{}: record {:?} says seq_len {}, the file on disk holds {} bases",
            PATTERNS_FILE,
            record,
            pattern_row[1],
            sequence.length()
        );
        // `wrap_used` is `yes` exactly where the record core was given is
        // circular (D049): a FASTA read with the linear assumption is not,
        // whatever its `topology` column says about what the file stated.
        let circular: bool = sequence.topology() == SequenceTopology::Circular;
        assert!(
            yes(&pattern_row[3]) == circular,
            "{}: record {:?} wrap_used {:?} but the record core was given is {}",
            PATTERNS_FILE,
            record,
            pattern_row[3],
            sequence.topology().name()
        );
        let found: Vec<DnaHit> = match find_dna(&sequence, pattern) {
            Ok(found) => found,
            Err(error) => panic!(
                "find_dna refused {:?} on record {:?} ({}: {}); the tables search it and name {} hit(s)",
                pattern,
                record,
                error.code(),
                error.message(),
                group.len()
            ),
        };
        let core_rows: Vec<String> = found.iter().map(core_row).collect();
        assert!(
            found.len() == group.len(),
            "record {:?} pattern {:?}: core gives {} hits ({}), the table names {} ({}); purpose: {}",
            record,
            pattern,
            found.len(),
            core_rows.join(", "),
            group.len(),
            rows.join(", "),
            pattern_row[9]
        );
        let mut previous: Option<u64> = None;
        for (index, row) in group.iter().enumerate() {
            let hit: &DnaHit = &found[index];
            assert!(
                hit.strand.symbol() == row[3],
                "record {:?} pattern {:?} hit {}: strand {:?}, the table says {:?}",
                record,
                pattern,
                index + 1,
                hit.strand.symbol(),
                row[3]
            );
            assert!(
                hit.start == number(&row[4]),
                "record {:?} pattern {:?} hit {}: zero-based start {}, the table says {}; core {} / table {}",
                record,
                pattern,
                index + 1,
                hit.start,
                row[4],
                core_row(hit),
                hit_row(row)
            );
            assert!(
                hit.length == number(&row[5]),
                "record {:?} pattern {:?} hit {}: length {}, the table says {}",
                record,
                pattern,
                index + 1,
                hit.length,
                row[5]
            );
            assert!(
                hit.wraps_origin == yes(&row[6]),
                "record {:?} pattern {:?} hit {}: wraps_origin {}, the table says {}",
                record,
                pattern,
                index + 1,
                hit.wraps_origin,
                row[6]
            );
            if let Some(seen) = previous {
                assert!(
                    seen < hit.start,
                    "record {:?} pattern {:?}: hits must be ordered by start, {} then {}",
                    record,
                    pattern,
                    seen,
                    hit.start
                );
            }
            previous = Some(hit.start);
        }
        grouped += group.len();
    }
    assert!(
        grouped == hits.len(),
        "{} of {} hit rows belong to a pattern row; every group must be checked, not skipped",
        grouped,
        hits.len()
    );
}

/// The coordinates of every hit, checked against the bases its row names: the
/// span core reports, read out of core's own record and wrapped when the hit
/// wraps, is the span the reference script wrote into `top_strand_bases`.
#[test]
fn every_hit_covers_the_bases_its_row_names() {
    let patterns: Vec<Vec<String>> = rows_of(PATTERNS_FILE, PATTERNS_HEADER, PATTERNS_COLUMNS);
    let hits: Vec<Vec<String>> = rows_of(HITS_FILE, HITS_HEADER, HITS_COLUMNS);
    let mut checked: usize = 0usize;
    for pattern_row in patterns.iter() {
        let record: &str = &pattern_row[0];
        let pattern: &str = &pattern_row[4];
        let group: Vec<Vec<String>> = group_of(&hits, record, pattern);
        if group.is_empty() {
            // A pattern with no hits has no window to read back; the count
            // itself is pinned in the test above, including the claim that it
            // matches nowhere.
            assert!(
                number(&pattern_row[8]) == 0,
                "{}: record {:?} pattern {:?} says hit_count {} but has no hit rows",
                PATTERNS_FILE,
                record,
                pattern,
                pattern_row[8]
            );
            continue;
        }
        let sequence: Sequence = fixture_record(record);
        let found: Vec<DnaHit> = match find_dna(&sequence, pattern) {
            Ok(found) => found,
            Err(error) => panic!(
                "find_dna refused {:?} on record {:?} ({}: {})",
                pattern,
                record,
                error.code(),
                error.message()
            ),
        };
        assert!(
            found.len() == group.len(),
            "record {:?} pattern {:?}: core gives {} hits, the table names {}",
            record,
            pattern,
            found.len(),
            group.len()
        );
        for (index, row) in group.iter().enumerate() {
            let hit: &DnaHit = &found[index];
            let bases: String = window_bases(&sequence, hit.start, hit.length, hit.wraps_origin);
            assert!(
                bases == row[7],
                "record {:?} pattern {:?} hit {}: the span core reports ({}@{}+{}) covers {:?}, the table's top_strand_bases is {:?}",
                record,
                pattern,
                index + 1,
                hit.strand.symbol(),
                hit.start,
                hit.length,
                bases,
                row[7]
            );
            checked += 1;
        }
    }
    assert!(
        checked == HITS_ROWS,
        "{} hit windows read back, the table holds {} hit rows",
        checked,
        HITS_ROWS
    );
}
