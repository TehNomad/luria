//! The ORFs of `crates/core/src/orf.rs` (M8-40) against
//! `tests/fixtures/view/orfs.tsv`: the open reading frames of four records, read
//! circular and linear as the view can ask for either, with the minimum that
//! keeps them, on both strands.
//!
//! The table was written by `tests/fixtures/view/reference/orfs.py`, which reads
//! each reading with Biopython's own translation and finds the runs of residues
//! between a start codon and a stop codon; `tests/fixtures/view/README.md`
//! records those rules and the hand values the script checked its own rows
//! against (D051). The same hand values are repeated here as constants, so a
//! drifted table fails here too (D014). The Python checks are run from the
//! repository root, not by cargo (D039, D026):
//!
//! ```text
//! python3 -I tests/fixtures/view/reference/orfs.py --check
//! python3 -I tests/fixtures/verify.py
//! ```
//!
//! Every comparison is between core values (zero-based, half-open, D006) and a
//! fixture value converted once as the column is parsed; the one 1-based
//! conversion lives in `location_text`, which renders core's intervals the way
//! the table writes them. No rule core derives is re-derived here.

use luria_core::genbank::read_genbank;
use luria_core::genbank::GenBankRecord;
use luria_core::identity::Identifier;
use luria_core::orf::find_orfs;
use luria_core::orf::Orf;
use luria_core::sequence::is_dna_base;
use luria_core::sequence::normalize;
use luria_core::sequence::Sequence;
use luria_core::sequence::SequenceTopology;
use luria_core::strand::Strand;

const VIEW: &str = "../../tests/fixtures/view/";
const GENBANK: &str = "../../tests/fixtures/genbank/";
const TABLE: &str = "orfs.tsv";
/// The header row the table must have.
const HEADER: &str = "record\ttopology\tmin_residues\tstrand\tlocation\tresidues";
const COLUMNS: usize = 6;
/// 45 rows over nine cases, as the reference script's own check reports them:
/// 9/8/9 for `pbr322`, 4/4 for `synthetic`, 4/5 for `synthetic_linear`, 2/0 for
/// `frames62` (contract 8m17 §6).
const ROWS: usize = 45;

const PBR322_FILE: &str = "raw/J01749.1.gb";
const SYNTHETIC_FILE: &str = "synthetic.gb";
const LINEAR_FILE: &str = "synthetic-linear.gb";
const FRAMES62_FILE: &str = "frames-62.gb";

/// One case the table holds: a record, how the view asks for it to be read, the
/// minimum it keeps, and the rows the contract's §6 counts for it.
struct Case {
    record: &'static str,
    file: &'static str,
    topology: &'static str,
    min_residues: u64,
    rows: usize,
}

/// The nine cases. `pbr322` is read both ways because the view can flip what the
/// file itself says (contract 8m17 §5.2, and the test
/// `the_table_reads_both_ways_for_records_that_say_one`); `frames62` and
/// `synthetic_linear` appear in both readings to pin each of the two flips.
const CASES: [Case; 9] = [
    Case {
        record: "pbr322",
        file: PBR322_FILE,
        topology: "circular",
        min_residues: 75,
        rows: 9,
    },
    Case {
        record: "pbr322",
        file: PBR322_FILE,
        topology: "circular",
        min_residues: 81,
        rows: 8,
    },
    Case {
        record: "pbr322",
        file: PBR322_FILE,
        topology: "linear",
        min_residues: 75,
        rows: 9,
    },
    Case {
        record: "synthetic",
        file: SYNTHETIC_FILE,
        topology: "circular",
        min_residues: 2,
        rows: 4,
    },
    Case {
        record: "synthetic",
        file: SYNTHETIC_FILE,
        topology: "linear",
        min_residues: 2,
        rows: 4,
    },
    Case {
        record: "synthetic_linear",
        file: LINEAR_FILE,
        topology: "linear",
        min_residues: 10,
        rows: 4,
    },
    Case {
        record: "synthetic_linear",
        file: LINEAR_FILE,
        topology: "circular",
        min_residues: 10,
        rows: 5,
    },
    Case {
        record: "frames62",
        file: FRAMES62_FILE,
        topology: "circular",
        min_residues: 3,
        rows: 2,
    },
    Case {
        record: "frames62",
        file: FRAMES62_FILE,
        topology: "linear",
        min_residues: 3,
        rows: 0,
    },
];

/// The rows of `orfs.tsv` past its header, refusing a table that is not the one
/// the reference script wrote.
fn table_rows() -> Vec<Vec<String>> {
    let path: String = format!("{}{}", VIEW, TABLE);
    let text: String = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
    let lines: Vec<&str> = text.lines().filter(|line| !line.is_empty()).collect();
    assert!(
        !lines.is_empty() && lines[0] == HEADER,
        "{}: first row {:?}, expected {:?}",
        path,
        lines.first().unwrap_or(&""),
        HEADER
    );
    let rows: Vec<Vec<String>> = lines[1..]
        .iter()
        .map(|line| line.split('\t').map(|field| field.to_string()).collect())
        .collect();
    for row in rows.iter() {
        assert!(
            row.len() == COLUMNS,
            "{}: row {:?} has {} columns, expected {}",
            TABLE,
            row,
            row.len(),
            COLUMNS
        );
    }
    assert!(
        rows.len() == ROWS,
        "{}: {} rows, expected {}",
        TABLE,
        rows.len(),
        ROWS
    );
    rows
}

/// The rows of one case, in the table's order.
fn case_rows(rows: &[Vec<String>], case: &Case) -> Vec<Vec<String>> {
    rows.iter()
        .filter(|row| {
            row[0] == case.record && row[1] == case.topology && number(&row[2]) == case.min_residues
        })
        .cloned()
        .collect()
}

/// The directory a case's fixture sits in: `frames62` is a view fixture created
/// for the view's frame tests (D032), the others are sequencing fixtures.
fn fixture_dir(record: &str) -> &'static str {
    if record == "frames62" {
        VIEW
    } else {
        GENBANK
    }
}

/// The sequence a case's record holds, read with the reader that owns its
/// format, named after the `record` column so the table and the sequence agree.
fn fixture_sequence(case: &Case) -> Sequence {
    let path: String = format!("{}{}", fixture_dir(case.record), case.file);
    let text: String = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
    let name: Identifier = Identifier::create(case.record.to_string()).unwrap_or_else(|error| {
        panic!("cannot use {:?} as an identifier: {:?}", case.record, error)
    });
    let record: GenBankRecord = read_genbank(name, text).unwrap_or_else(|error| {
        panic!(
            "{}: read_genbank refused it: {:?}",
            case.file,
            error.message()
        )
    });
    record.sequence()
}

/// Whether the case's `topology` column asks for a circular reading: the case
/// the view wants, never what the file says (D036).
fn case_is_circular(case: &Case) -> bool {
    match case.topology {
        "circular" => true,
        "linear" => false,
        other => panic!("{}: no topology called {:?}", case.record, other),
    }
}

/// A fixture number.
fn number(text: &str) -> u64 {
    text.parse::<u64>()
        .unwrap_or_else(|error| panic!("cannot read {:?} as a number: {}", text, error))
}

/// A strand as the table writes it.
fn strand_column(strand: &Strand) -> String {
    match strand {
        Strand::Plus => "+".to_string(),
        Strand::Minus => "-".to_string(),
    }
}

/// Core's intervals as the table's location text: 1-based inclusive, `join()`
/// for two parts in D036's order (`complement()` on the minus strand). The one
/// 1-based conversion of this file, applied as core's coordinates are written
/// out (D006).
fn location_text(orf: &Orf) -> String {
    let parts: Vec<String> = orf
        .intervals
        .iter()
        .map(|(start, end)| format!("{}..{}", start + 1, end))
        .collect();
    let body: String = if parts.len() == 1 {
        parts.join("")
    } else {
        format!("join({})", parts.join(","))
    };
    match orf.strand {
        Strand::Plus => body,
        Strand::Minus => format!("complement({})", body),
    }
}

/// The bases a location text covers, read off the text itself. The table's
/// grammar (`complement()`, `join()`, `start..end`) is read here independently
/// of `genbank_location::parse_feature_location`, so a broken location cannot
/// cancel itself out in a comparison.
fn location_span(text: &str) -> u64 {
    let body: &str = match text.strip_prefix("complement(") {
        Some(rest) => rest.strip_suffix(')').unwrap_or(rest),
        None => text,
    };
    let inner: &str = match body.strip_prefix("join(") {
        Some(rest) => rest.strip_suffix(')').unwrap_or(rest),
        None => body,
    };
    inner.split(',').map(part_span).sum()
}

/// One `start..end` part as bases: inclusive on the page, so the count is the
/// end minus the start, plus one.
fn part_span(part: &str) -> u64 {
    let (start, end) = part_bounds(part);
    end - start + 1
}

/// The 1-based bounds of one location part.
fn part_bounds(part: &str) -> (u64, u64) {
    let mut sides = part.split("..");
    let start: u64 = match sides.next() {
        Some(value) => number(value),
        None => panic!("{:?}: a location part needs a start", part),
    };
    let end: u64 = match sides.next() {
        Some(value) => number(value),
        None => panic!("{:?}: a location part needs an end", part),
    };
    assert!(
        sides.next().is_none(),
        "{:?}: no second `..` in a part",
        part
    );
    (start, end)
}

/// The parts of a location text, in the order the text is written.
fn location_parts(text: &str) -> Vec<(u64, u64)> {
    let body: &str = match text.strip_prefix("complement(") {
        Some(rest) => rest.strip_suffix(')').unwrap_or(rest),
        None => text,
    };
    let inner: &str = match body.strip_prefix("join(") {
        Some(rest) => rest.strip_suffix(')').unwrap_or(rest),
        None => body,
    };
    inner
        .split(',')
        .map(part_bounds)
        .collect::<Vec<(u64, u64)>>()
}

/// The record's letters: every fixture record in this table is DNA with no
/// ambiguity code, which is what makes D010's residue arithmetic apply to every
/// row, and gives the record length that the span bound and D036 are measured
/// against.
fn fixture_length(record: &str) -> u64 {
    let case: &Case = CASES
        .iter()
        .find(|case| case.record == record)
        .unwrap_or_else(|| panic!("no case of a record called {:?}", record));
    let letters: Vec<char> = fixture_sequence(case).letters();
    assert!(
        letters.iter().all(|ch| is_dna_base(*ch)),
        "{}: a fixture record holds a letter that is not a base; the table's residue arithmetic assumes ACGT (D010)",
        record
    );
    letters.len() as u64
}

/// The ORFs of a case, as the table writes them.
fn case_orf_texts(case: &Case) -> Vec<String> {
    render(
        find_orfs(
            &fixture_sequence(case),
            case_is_circular(case),
            case.min_residues,
        )
        .as_slice(),
    )
}

/// T1: for each case, `find_orfs` rendered to the table's columns equals the
/// table's rows, in order (contract 8m17 §8).
#[test]
fn every_case_matches_the_table() {
    let rows: Vec<Vec<String>> = table_rows();
    for case in CASES.iter() {
        let wanted: Vec<Vec<String>> = case_rows(rows.as_slice(), case);
        assert!(
            wanted.len() == case.rows,
            "{} {} {}: the table holds {} rows for this case, expected {}",
            case.record,
            case.topology,
            case.min_residues,
            wanted.len(),
            case.rows
        );
        let given: Vec<String> = case_orf_texts(case);
        let expected: Vec<String> = wanted
            .iter()
            .map(|row| format!("{} {} {}", row[3], row[4], row[5]))
            .collect();
        assert!(
            given == expected,
            "{} {} {}:\n  got    {:?}\n  expected {:?}",
            case.record,
            case.topology,
            case.min_residues,
            given,
            expected
        );
    }
}

/// T2: `pbr322` circular: `complement(323..565)` (80 residues) is kept at 80 and
/// dropped at 81, so the minimum is inclusive (contract 8m17 §5.2, §7, §8).
#[test]
fn the_minimum_is_inclusive() {
    let sequence: Sequence = fixture_sequence(&CASES[0]);
    let at_80: Vec<String> = find_orfs(&sequence, true, 80)
        .iter()
        .map(|orf| format!("{} {}", location_text(orf), orf.residues))
        .collect();
    let at_81: Vec<String> = find_orfs(&sequence, true, 81)
        .iter()
        .map(|orf| format!("{} {}", location_text(orf), orf.residues))
        .collect();
    assert!(
        at_80.iter().any(|row| row == "complement(323..565) 80"),
        "80 residues kept at a minimum of 80: {:?}",
        at_80
    );
    assert!(
        !at_81.iter().any(|row| row == "complement(323..565) 80"),
        "80 residues dropped at a minimum of 81: {:?}",
        at_81
    );
    // The rest of the case is unchanged at 80, and at 81 only that row goes: the
    // §7 hand value `86..1276` (396) and `complement(3293..4153)` (286) stand in
    // both readings, and §6 counts 9 rows at 75 and 8 at 81.
    assert!(
        at_80.len() == 9,
        "a minimum of 80 keeps 9 rows: {:?}",
        at_80
    );
    assert!(
        at_81.len() == 8,
        "a minimum of 81 keeps 8 rows: {:?}",
        at_81
    );
    for row in ["86..1276 396", "complement(3293..4153) 286"] {
        assert!(
            at_80.iter().any(|given| given == row) && at_81.iter().any(|given| given == row),
            "{} stands at both minimums",
            row
        );
    }
}

/// T3: §7's `synthetic` pair (contract 8m17 §8): the same stop gives
/// `join(55..60,1..12)` (5 residues) read circular and `1..12` (3) read linear,
/// because the ATG at 55 is reached only across the origin. This is the case
/// where taking the nearest ATG instead of the furthest back is visible in the
/// coordinates, not in the count, so it is the guard for that rule (G1).
#[test]
fn the_longest_orf_per_stop_crosses_the_origin() {
    let circular: Case = Case {
        record: "synthetic",
        file: SYNTHETIC_FILE,
        topology: "circular",
        min_residues: 2,
        rows: 4,
    };
    let linear: Case = Case {
        record: "synthetic",
        file: SYNTHETIC_FILE,
        topology: "linear",
        min_residues: 2,
        rows: 4,
    };
    let crossing: Vec<String> = case_orf_texts(&circular);
    let broken: Vec<String> = case_orf_texts(&linear);
    assert!(
        crossing.iter().any(|row| row == "+ join(55..60,1..12) 5"),
        "the origin-crossing ORF of §7: {:?}",
        crossing
    );
    assert!(
        broken.iter().any(|row| row == "+ 1..12 3"),
        "the same stop read linear gives 1..12 of 3 residues: {:?}",
        broken
    );
    // The two readings differ only in that row: read linear, the run the crossing
    // row starts in begins before the record and loses its start; read circular,
    // it reaches one. Every other row of one reading is a row of the other.
    for row in crossing.iter() {
        if row.starts_with("+ join(55..60,1..12)") {
            continue;
        }
        assert!(
            broken.iter().any(|given| given == row),
            "{:?} read circular is missing read linear",
            row
        );
    }
    for row in broken.iter() {
        if row.starts_with("+ 1..12") {
            continue;
        }
        assert!(
            crossing.iter().any(|given| given == row),
            "{:?} read linear is missing read circular",
            row
        );
    }
}

/// T4: §7's `frames62` values, and no ORF anywhere in the table spans more than
/// its record (contract 8m17 §5 rule 2's bound, §8). The 62-base record holds an
/// ATG-to-stop run whose 21 codons span 63 bases of the record: refused.
#[test]
fn no_orf_is_longer_than_the_record() {
    let case: Case = Case {
        record: "frames62",
        file: FRAMES62_FILE,
        topology: "circular",
        min_residues: 3,
        rows: 2,
    };
    let found: Vec<String> = case_orf_texts(&case);
    // §7: the two origin-crossing CDSs that are ORFs, and nothing else.
    assert!(
        found
            .iter()
            .any(|row| row == "- complement(join(53..62,1..8)) 5")
            && found.iter().any(|row| row == "+ join(56..62,1..11) 5"),
        "§7's two frames62 rows: {:?}",
        found
    );
    assert!(
        found.len() == 2,
        "the 63-base run of 21 codons is not an ORF, leaving the two rows of §7: {:?}",
        found
    );
    // The bound holds for every case core scans, table or not: no ORF spans more
    // than the record it is read on.
    for case in CASES.iter() {
        let length: u64 = fixture_length(case.record);
        for orf in find_orfs(
            &fixture_sequence(case),
            case_is_circular(case),
            case.min_residues,
        ) {
            let span: u64 = orf.intervals.iter().map(|(start, end)| end - start).sum();
            assert!(
                span <= length,
                "{} {} {}: an ORF spans {} bases of a {} base record ({:?})",
                case.record,
                case.topology,
                case.min_residues,
                span,
                length,
                location_text(&orf)
            );
        }
    }
    // ...and for every row of the table, read off the location column.
    let rows: Vec<Vec<String>> = table_rows();
    for row in rows.iter() {
        let length: u64 = fixture_length(&row[0]);
        let span: u64 = location_span(&row[4]);
        assert!(
            span <= length,
            "{} {} {}: {:?} spans {} bases of a {} base record",
            row[0],
            row[1],
            row[2],
            row[4],
            span,
            length
        );
        // D010: the residues are the codons of the span, the stop codon of the
        // span not counted. Every fixture record here is plain ACGT (checked in
        // `fixture_length`), so the codon count divides exactly.
        assert!(
            span.is_multiple_of(3) && span / 3 - 1 == number(&row[5]),
            "{} {} {}: {:?} spans {} bases, which is {} codons including the stop, not {} residues",
            row[0],
            row[1],
            row[2],
            row[4],
            span,
            span / 3,
            number(&row[5]),
        );
        // D036: a join of two parts, in the order the interval that ends at the
        // record's length first, the one starting at origin second, and only two
        // parts ever.
        let parts: Vec<(u64, u64)> = location_parts(&row[4]);
        if parts.len() == 2 {
            assert!(
                parts[0].1 == length && parts[1].0 == 1,
                "{} {} {}: {:?} crosses the origin as two intervals, the first ending at {} and the second starting at 1",
                row[0],
                row[1],
                row[2],
                parts,
                length
            );
        } else {
            assert!(
                parts.len() == 1,
                "{} {} {}: {:?} is not one interval nor two: D036 gives two intervals for an ORF across the origin",
                row[0],
                row[1],
                row[2],
                row[4]
            );
        }
    }
}

/// T5: an ambiguous codon is neither a start nor a stop, whatever its reading
/// (contract 8m17 §5 rule 4, D010, §8). `TAR` sits inside a run, does not stop
/// it, and is not a start. Biopython answers `TAR` as a stop codon, which is
/// recorded as a disagreement in `tests/fixtures/view/README.md` and not copied.
#[test]
fn an_ambiguous_codon_is_not_a_stop() {
    let one: Sequence = sequence_of("one", "ATGAAATARAAATAA");
    let found: Vec<String> = render(find_orfs(&one, false, 1).as_slice());
    assert!(
        found.len() == 1 && found[0] == "+ 1..15 4",
        "ATG AAA TAR AAA TAA holds one ORF of four residues: {:?}",
        found
    );

    let three: Sequence = sequence_of("three", "ATGNNNTAA");
    let found: Vec<String> = render(find_orfs(&three, false, 2).as_slice());
    assert!(
        found.len() == 1 && found[0] == "+ 1..9 2",
        "ATG NNN TAA holds one ORF of two residues: {:?}",
        found
    );

    let starts_ambiguously: Sequence = sequence_of("start", "NTGAAATAA");
    let found: Vec<String> = render(find_orfs(&starts_ambiguously, false, 1).as_slice());
    assert!(
        found.is_empty(),
        "NTG AAA TAA is not a start codon, so no ATG opens it: {:?}",
        found
    );
}

/// A normalized linear sequence of the test's own letters.
fn sequence_of(name: &str, letters: &str) -> Sequence {
    normalize(
        Identifier::create(name.to_string()).expect("a test sequence name"),
        letters.to_uppercase(),
        SequenceTopology::Linear,
    )
    .expect("a normalized test sequence")
}

/// Core's ORFs as the table writes them.
fn render(orfs: &[Orf]) -> Vec<String> {
    orfs.iter()
        .map(|orf| {
            format!(
                "{} {} {}",
                strand_column(&orf.strand),
                location_text(orf),
                orf.residues
            )
        })
        .collect()
}

/// The table's topology column is what the view asks for, never what the file
/// says: four records state one reading on their LOCUS line and the table asks
/// for both readings of them (D036, contract 8m17 §5.2).
#[test]
fn the_table_reads_both_ways_for_records_that_say_one() {
    let mut flipped: usize = 0;
    let mut stated: usize = 0;
    for case in CASES.iter() {
        let file: String = fixture_sequence(case).topology().name();
        let asked: &str = case.topology;
        if asked != file {
            flipped += 1;
        } else {
            stated += 1;
        }
        let both: bool = CASES
            .iter()
            .filter(|other| other.record == case.record)
            .any(|other| other.topology != asked);
        assert!(
            both,
            "{}: the table reads {} as {:?} while the file itself says {:?}",
            case.record, case.file, asked, file
        );
    }
    assert!(
        flipped > 0 && stated > 0,
        "the table must hold at least one reading the file contradicts ({}) and at least one it states itself ({})",
        flipped,
        stated
    );
}

// Contract section 7's hand values, repeated here as constants so that a
// drifted table fails in Rust as well as in Python (D014). Each is a row the
// contract counts by hand and `reference/orfs.py` verifies against the record's
// own bases.
#[test]
fn the_hand_values_of_section_7_are_repeated_as_constants() {
    const HAND: [(&str, &str, &str, &str, &str, &str); 7] = [
        (
            "pbr322",
            "circular",
            "75",
            "-",
            "complement(3293..4153)",
            "286",
        ),
        ("pbr322", "circular", "75", "+", "86..1276", "396"),
        ("synthetic", "circular", "2", "+", "join(55..60,1..12)", "5"),
        ("synthetic", "linear", "2", "+", "1..12", "3"),
        (
            "synthetic_linear",
            "circular",
            "10",
            "+",
            "join(284..393,1..34)",
            "47",
        ),
        ("frames62", "circular", "3", "+", "join(56..62,1..11)", "5"),
        (
            "frames62",
            "circular",
            "3",
            "-",
            "complement(join(53..62,1..8))",
            "5",
        ),
    ];
    for (record, topology, min, strand, location, residues) in HAND {
        let rows: Vec<Vec<String>> = table_rows();
        let found: Vec<&Vec<String>> = rows
            .iter()
            .filter(|row| {
                row[0] == record
                    && row[1] == topology
                    && number(&row[2]) == number(min)
                    && row[3] == strand
                    && row[4] == location
            })
            .collect();
        assert!(
            found.len() == 1 && found[0][5] == residues,
            "{} {} {} {} {}: expected {} residues, the table says {:?}",
            record,
            topology,
            min,
            strand,
            location,
            residues,
            found
        );
    }

    // D010 as these rows apply it: what a location spans is the codons of the
    // ORF including its terminal stop, and the residues are one fewer. The two
    // `pbr322` counts are the hand ones of section 7, not the frame's: 3293..4153
    // spans 861 bases and 86..1276 spans 1191 of a 4361 base record.
    for (location, span, residues) in [
        ("complement(3293..4153)", 861u64, 286u64),
        ("86..1276", 1191, 396),
        ("join(55..60,1..12)", 18, 5),
        ("join(284..393,1..34)", 144, 47),
        ("join(56..62,1..11)", 18, 5),
        ("complement(join(53..62,1..8))", 18, 5),
        ("1..12", 12, 3),
    ] {
        assert!(
            location_span(location) == span && span / 3 - 1 == residues,
            "{} spans {} bases, which is not {} residues",
            location,
            span,
            residues
        );
    }
}
