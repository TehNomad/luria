//! The six reading frames in `crates/core/src/frames.rs` (8d, plan §4.3)
//! against `tests/fixtures/view/frame-tables.tsv` and
//! `tests/fixtures/view/frame-tails.tsv`: every complete codon of every frame
//! of both strands of six records, and per frame how many bases it covers, how
//! many complete codons that is, and which 1 or 2 leftover reference positions
//! frame mode must never draw (D050, D052).
//!
//! The two tables were written by `tests/fixtures/view/reference/frame_tables.py`
//! from Biopython's translation of each linear slice plus the arithmetic written
//! out there, and `tests/fixtures/view/README.md` records the rules and the two
//! spot values it checked by hand — including the codons Biopython answers
//! differently from luria (`GGN` → `G`, `CTN` → `L`, `TAR` → `*`, `NNN` → `X`),
//! which are recorded as a disagreement rather than followed (D010, D052). The
//! Python check is re-run from the repository root, not by cargo (D039, D026):
//!
//! ```
//! python3 -I tests/fixtures/verify.py
//! ```
//!
//! Every comparison here is between core values (zero-based, D006) and fixture
//! values converted by the single `zero_based()` helper as the column is parsed;
//! no rule that core also derives is re-derived in this file. `frames.rs` decides
//! where a codon sits and what letter it gives; the tables say whether that
//! agrees with what Biopython and the hand-derived rows say.

use luria_core::error::DomainError;
use luria_core::fasta::read_fasta;
use luria_core::fasta::FastaRecord;
use luria_core::frames::frame_residues;
use luria_core::frames::FrameResidue;
use luria_core::frames::ReadingFrame;
use luria_core::genbank::read_genbank;
use luria_core::genbank::GenBankRecord;
use luria_core::identity::Identifier;
use luria_core::sequence::normalize;
use luria_core::sequence::Sequence;
use luria_core::sequence::SequenceTopology;
use luria_core::strand::Strand;

const VIEW: &str = "../../tests/fixtures/view/";
const GENBANK: &str = "../../tests/fixtures/genbank/";
const TABLES_FILE: &str = "frame-tables.tsv";
const TAILS_FILE: &str = "frame-tails.tsv";
/// One row per complete codon of one frame, in that frame's own numbering.
const TABLES_HEADER: &str = "record\tseq_len\ttopology\tstrand\tframe\tcodon_reading\tresidue\taa\tcodon\tbases_ref_ascending\tsampled";
/// One row per record, strand and frame: what the frame covers and what it
/// leaves over.
const TAILS_HEADER: &str = "record\tseq_len\ttopology\tstrand\tframe\tbases_in_frame\tcomplete_codons\tleftover_bases\tleftover";
const TABLES_COLUMNS: usize = 11usize;
const TAILS_COLUMNS: usize = 9usize;
const TABLES_ROWS: usize = 1318usize;
const TAILS_ROWS: usize = 36usize;
const PBR322_FILE: &str = "raw/J01749.1.gb";
const SYNTHETIC_FILE: &str = "synthetic.gb";
const LINEAR_FILE: &str = "synthetic-linear.gb";
const FRAMES62_FILE: &str = "frames-62.gb";
const FRAMES61_FILE: &str = "frames-61.fa";
const FRAMESAMB_FILE: &str = "frames-amb.fa";
/// The `aa` column of `framesamb` plus frame 1 and minus frame 1, read off the
/// table by hand as its rows stand (11 codons each): `ATG TAT TAA TAG TAR AAN
/// GGN CTN NNN GTG TSK` and the complement-image reading of the same bases from
/// the far end. Every `?` position is the point — `GGN` and `CTN` are the rows
/// whose every expansion is one amino acid, which D010 refuses to guess at.
const AMBIGUOUS_PLUS: &str = "MY**?????V?";
const AMBIGUOUS_MINUS: &str = "?H?????LLIH";

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
/// record says nothing about topology, which is why its rows read `unknown`
/// while the view assumes linear (D049);
/// `a_fasta_record_reports_unknown_topology_in_the_table_and_is_read_as_linear`
/// pins both facts.
fn fixture_file(value: &str) -> (&'static str, &'static str, bool) {
    match value {
        "pbr322" => (GENBANK, PBR322_FILE, false),
        "synthetic" => (GENBANK, SYNTHETIC_FILE, false),
        "synlin" => (GENBANK, LINEAR_FILE, false),
        "frames62" => (VIEW, FRAMES62_FILE, false),
        "frames61" => (VIEW, FRAMES61_FILE, true),
        "framesamb" => (VIEW, FRAMESAMB_FILE, true),
        other => panic!("{}: no fixture record of that name", other),
    }
}

/// The sequence a fixture `record` column names, read with luria's own reader:
/// `read_genbank` for a GenBank file, `read_fasta` with the linear assumption
/// D049 gives a FASTA. The record is read from its own file every time, so a
/// table can never be compared against a record built some other way.
fn record_of(value: &str) -> Sequence {
    let pair: (&'static str, &'static str, bool) = fixture_file(value);
    let text: String = read_input(pair.0, pair.1);
    if pair.2 {
        let record: FastaRecord = read_fasta(name(pair.1), text, SequenceTopology::Linear)
            .unwrap_or_else(|error| {
                panic!("{}: read_fasta refused it: {:?}", pair.1, error.message())
            });
        return record.sequence();
    }
    let record: GenBankRecord = read_genbank(name(pair.1), text).unwrap_or_else(|error| {
        panic!("{}: read_genbank refused it: {:?}", pair.1, error.message())
    });
    record.sequence()
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

/// A fixture number.
fn number(text: &str) -> u64 {
    text.parse::<u64>()
        .unwrap_or_else(|error| panic!("cannot read {:?} as a number: {}", text, error))
}

/// A column of 1-based reference positions such as `bases_ref_ascending` or
/// `leftover_bases`, converted once here (D006) and compared only against core
/// values. `-` is the generator's marker for an empty list.
fn fixture_positions(text: &str) -> Vec<u64> {
    if text == "-" {
        return Vec::new();
    }
    text.split(',')
        .map(|field| zero_based(number(field)))
        .collect()
}

/// The middle of a codon's three reference positions in ascending order. The
/// columns are written ascending, and a frame's codons never wrap the origin
/// (D052), so a frame codon's three positions are contiguous and its middle
/// base is the middle of the ascending triple. This reads the fixture's own
/// column; it is not core's frame arithmetic.
fn fixture_middle(row: &[String], column: usize) -> u64 {
    let ascending: Vec<u64> = fixture_positions(&row[column]);
    assert!(
        ascending.len() == 3,
        "{} {} frame {}: `bases_ref_ascending` lists {:?}, expected three positions",
        row[0],
        row[3],
        row[4],
        ascending
    );
    ascending[1]
}

/// The strand a fixture `strand` column names.
fn fixture_strand(text: &str) -> Strand {
    match text {
        "+" => Strand::Plus,
        "-" => Strand::Minus,
        other => panic!("{:?}: not a strand the tables use", other),
    }
}

/// The frame a fixture `frame` column names, read the way core reads a frame
/// number rather than by a second mapping.
fn fixture_frame(text: &str) -> ReadingFrame {
    match ReadingFrame::from_index(number(text)) {
        None => panic!("{:?}: no such reading frame", text),
        Some(frame) => frame,
    }
}

/// The `aa` column: exactly one symbol character.
fn fixture_symbol(text: &str) -> char {
    let letters: Vec<char> = text.chars().collect();
    assert!(
        letters.len() == 1,
        "{:?}: an `aa` cell must hold one symbol",
        text
    );
    letters[0]
}

/// The residues of one frame over the whole record, the window being
/// `[0, length)`.
fn whole_frame(sequence: &Sequence, strand: Strand, frame: ReadingFrame) -> Vec<FrameResidue> {
    frame_residues(sequence, strand, frame, 0, sequence.length())
        .unwrap_or_else(|error| panic!("frame_residues refused a whole-record window: {:?}", error))
}

/// The middles of a residue list, in the order the frame produced them.
fn middles(residues: &[FrameResidue]) -> Vec<u64> {
    residues.iter().map(|residue| residue.middle).collect()
}

/// The symbols of a residue list, run together into one string.
fn symbols(residues: &[FrameResidue]) -> String {
    residues.iter().map(|residue| residue.symbol).collect()
}

/// The strand symbol, for a message about a frame this file built rather than
/// one a table row named.
fn row_symbol(plus: bool) -> String {
    if plus {
        return "+".to_string();
    }
    "-".to_string()
}

#[test]
fn the_frame_table_headers_and_row_counts_are_pinned() {
    let codon_rows: Vec<Vec<String>> = rows_of(TABLES_FILE, TABLES_HEADER, TABLES_COLUMNS);
    assert!(
        codon_rows.len() == TABLES_ROWS,
        "{}: {} codon rows, expected {}",
        TABLES_FILE,
        codon_rows.len(),
        TABLES_ROWS
    );
    let tail_rows: Vec<Vec<String>> = rows_of(TAILS_FILE, TAILS_HEADER, TAILS_COLUMNS);
    assert!(
        tail_rows.len() == TAILS_ROWS,
        "{}: {} tail rows, expected {}",
        TAILS_FILE,
        tail_rows.len(),
        TAILS_ROWS
    );
    // Six records, and the tail table is exactly those six records times two
    // strands times three frames.
    let mut records: Vec<String> = Vec::new();
    for row in tail_rows.iter() {
        if !records.contains(&row[0]) {
            records.push(row[0].clone());
        }
    }
    records.sort();
    let wanted: Vec<String> = [
        "frames61",
        "frames62",
        "framesamb",
        "pbr322",
        "synlin",
        "synthetic",
    ]
    .iter()
    .map(|value| value.to_string())
    .collect();
    assert!(
        records == wanted,
        "{}: records {:?}, expected {:?}",
        TAILS_FILE,
        records,
        wanted
    );
    assert!(
        records.len() * 6 == TAILS_ROWS,
        "{}: {} records times six frames is not {} rows",
        TAILS_FILE,
        records.len(),
        TAILS_ROWS
    );
}

#[test]
fn every_frame_row_of_a_short_record_matches_core() {
    let rows: Vec<Vec<String>> = rows_of(TABLES_FILE, TABLES_HEADER, TABLES_COLUMNS);
    let mut sequences: Vec<(String, Sequence)> = Vec::new();
    // Rows of one record, strand and frame are contiguous in the file, in that
    // frame's own codon order; `index` counts them and is compared with core's
    // residue number and position in the frame's list.
    let mut key: String = String::new();
    let mut index: usize = 0usize;
    let mut counted: usize = 0usize;
    for row in rows.iter() {
        if row[10] != "no" {
            continue;
        }
        counted += 1;
        let row_key: String = format!("{}\t{}\t{}", row[0], row[3], row[4]);
        if row_key != key {
            assert!(
                !sequences.iter().any(|pair| pair.0 == row_key),
                "{}: rows for {} are not contiguous in the file",
                TABLES_FILE,
                row_key
            );
            key = row_key;
            index = 0usize;
        }
        index += 1;

        let sequence: Sequence = match sequences.iter().find(|pair| pair.0 == key) {
            None => {
                let fresh: Sequence = record_of(&row[0]);
                sequences.push((key.clone(), fresh.clone()));
                fresh
            }
            Some(pair) => pair.1.clone(),
        };
        assert!(
            number(&row[1]) == sequence.length(),
            "{}: the table gives seq_len {} for a record core reads as {} bases",
            row[0],
            row[1],
            sequence.length()
        );

        let strand: Strand = fixture_strand(&row[3]);
        let frame: ReadingFrame = fixture_frame(&row[4]);
        let residues: Vec<FrameResidue> = whole_frame(&sequence, strand, frame);
        assert!(
            index <= residues.len(),
            "{} {} {} frame {}: row {} of this frame, but core gives only {} residues for it",
            row[0],
            row[3],
            row[4],
            row[4],
            index,
            residues.len()
        );
        let residue: &FrameResidue = &residues[index - 1];

        // The frame's own numbering: `codon_reading` and `residue` columns agree
        // with each other and with core's `number`, which counts every complete
        // codon including stops and unreadable codons (D050).
        assert!(
            residue.number == number(&row[5]),
            "{} {} frame {}: core numbers its residue {} where the table gives codon_reading {}",
            row[0],
            row[3],
            row[4],
            residue.number,
            row[5]
        );
        assert!(
            number(&row[5]) == number(&row[6]),
            "{} {} frame {}: codon_reading {} and residue {} are the same count in a frame (D050)",
            row[0],
            row[3],
            row[4],
            row[5],
            row[6]
        );
        assert!(
            residue.number == (index as u64),
            "{} {} frame {}: row {} of this frame is numbered {} by core, so the frame's rows must be in the frame's own order",
            row[0],
            row[3],
            row[4],
            index,
            residue.number
        );
        assert!(
            residue.symbol == fixture_symbol(&row[7]),
            "{} {} frame {}: codon {} is read {:?} by core, the table gives {:?}",
            row[0],
            row[3],
            row[4],
            row[8],
            residue.symbol,
            fixture_symbol(&row[7])
        );
        assert!(
            residue.middle == fixture_middle(row.as_slice(), 9),
            "{} {} frame {}: codon {} ({}) sits on {:?}, so its middle base is zero-based {}, but core gives middle {}",
            row[0],
            row[3],
            row[4],
            row[8],
            row[9],
            row[9],
            fixture_middle(row.as_slice(), 9),
            residue.middle
        );
    }
    assert!(counted > 0, "{}: no unsampled row was checked", TABLES_FILE);
}

#[test]
fn every_sampled_row_of_the_long_record_matches_core() {
    let rows: Vec<Vec<String>> = rows_of(TABLES_FILE, TABLES_HEADER, TABLES_COLUMNS);
    let sequence: Sequence = record_of("pbr322");
    let mut checked: usize = 0usize;
    let mut longest: u64 = 0u64;
    for row in rows.iter() {
        let length: u64 = number(&row[1]);
        if length > longest {
            longest = length;
        }
        // The sample marker cannot drift away from the generator's rule: a
        // record longer than 500 bases is sampled, one of 500 or fewer is not.
        let sampled: bool = row[10] == "yes";
        assert!(
            sampled == (length > 500),
            "{}: seq_len {} is marked sampled {:?}; the rule is that only a record longer than 500 bases is sampled",
            row[0],
            row[1],
            row[10]
        );
        if row[0] != "pbr322" {
            continue;
        }
        assert!(
            length == sequence.length(),
            "pbr322: the table gives seq_len {}, core reads {} bases",
            row[1],
            sequence.length()
        );
        let strand: Strand = fixture_strand(&row[3]);
        let frame: ReadingFrame = fixture_frame(&row[4]);
        let residues: Vec<FrameResidue> = whole_frame(&sequence, strand, frame);
        // A sampled row is found by the number it names, not by its position in
        // the file: the file holds the first three codons, the last three and
        // every hundredth of this frame.
        let asked: u64 = number(&row[5]);
        let found: Option<&FrameResidue> = residues.iter().find(|residue| residue.number == asked);
        match found {
            None => panic!(
                "pbr322 {} frame {}: core has no residue numbered {}; it gives {} residues with numbers from 1 to {:?}",
                row[3],
                row[4],
                asked,
                residues.len(),
                residues.last().map(|residue| residue.number)
            ),
            Some(residue) => {
                checked += 1;
                assert!(
                    residue.symbol == fixture_symbol(&row[7]),
                    "pbr322 {} frame {}: residue {} is read {:?} by core, the table gives {:?} for codon {}",
                    row[3],
                    row[4],
                    asked,
                    residue.symbol,
                    fixture_symbol(&row[7]),
                    row[8]
                );
                assert!(
                    residue.middle == fixture_middle(row.as_slice(), 9),
                    "pbr322 {} frame {}: residue {} sits on {} so its middle base is zero-based {}, but core gives middle {}",
                    row[3],
                    row[4],
                    asked,
                    row[9],
                    fixture_middle(row.as_slice(), 9),
                    residue.middle
                );
            }
        }
    }
    assert!(
        checked == 120,
        "pbr322: {} sampled rows checked, expected 120 (20 per frame over six frames)",
        checked
    );
    assert!(
        longest == sequence.length(),
        "the longest record in {} is {}, pbr322 reads {} bases",
        TABLES_FILE,
        longest,
        sequence.length()
    );
}

#[test]
fn the_tail_table_pins_the_codon_count_and_the_shape_of_every_frame() {
    for row in rows_of(TAILS_FILE, TAILS_HEADER, TAILS_COLUMNS).iter() {
        let sequence: Sequence = record_of(&row[0]);
        assert!(
            number(&row[1]) == sequence.length(),
            "{}: the tail table gives seq_len {}, core reads {} bases",
            row[0],
            row[1],
            sequence.length()
        );
        let strand: Strand = fixture_strand(&row[3]);
        let frame: ReadingFrame = fixture_frame(&row[4]);
        let plus: bool = strand.clone().is_plus();
        let residues: Vec<FrameResidue> = whole_frame(&sequence, strand, frame);
        // How many residues a frame produces is the table's codon count, and the
        // window here is the whole record.
        assert!(
            residues.len() as u64 == number(&row[6]),
            "{} {} frame {}: core produces {} residues, the table records {} complete codons for {} bases of frame",
            row[0],
            row[3],
            row[4],
            residues.len(),
            row[6],
            row[5]
        );

        // Codons step by three along the frame, so consecutive middles differ by
        // exactly three: up the reference on the plus strand, down it on the
        // minus strand (D040: the reading order is not the reference order).
        assert!(
            middles(residues.as_slice()).windows(2).all(|pair| {
                let step: i64 = pair[1] as i64 - pair[0] as i64;
                step == if plus { 3 } else { -3 }
            }),
            "{} {} frame {}: middles {:?} do not step by three in the {} direction",
            row[0],
            row[3],
            row[4],
            middles(residues.as_slice()),
            row[3]
        );
        let plus_shape: bool = middles(residues.as_slice())
            .windows(2)
            .all(|pair| pair[1] > pair[0]);
        let minus_shape: bool = middles(residues.as_slice())
            .windows(2)
            .all(|pair| pair[1] < pair[0]);
        assert!(
            if plus { plus_shape } else { minus_shape },
            "{} {} frame {}: middles {:?} are not strictly {} on the {} strand",
            row[0],
            row[3],
            row[4],
            middles(residues.as_slice()),
            if plus { "increasing" } else { "decreasing" },
            row[3]
        );

        // The leftover bases are never drawn (D050): a leftover base is not the
        // middle base of any codon the frame produces. The table names those
        // positions, converted once by `fixture_positions`, and this test does
        // not work out where they sit.
        let leftover: Vec<u64> = fixture_positions(&row[7]);
        assert!(
            (row[8] == "yes") == !leftover.is_empty(),
            "{} {} frame {}: `leftover` says {:?} while `leftover_bases` lists {:?}",
            row[0],
            row[3],
            row[4],
            row[8],
            row[7]
        );
        assert!(
            leftover.len() <= 2,
            "{} {} frame {}: {} leftover positions, a frame can leave over at most 2",
            row[0],
            row[3],
            row[4],
            leftover.len()
        );
        for residue in residues.iter() {
            assert!(
                !leftover.contains(&residue.middle),
                "{} {} frame {}: residue {} has middle {}, which the table records as a leftover base that must not be drawn (D050)",
                row[0],
                row[3],
                row[4],
                residue.number,
                residue.middle
            );
        }
    }
}

#[test]
fn topology_does_not_change_the_frames() {
    // The same 62 bases under both topologies (D052: frame numbering is
    // independent of topology, so switching a record between circular and
    // linear changes the overview and the search, never the frames).
    let text: String = record_of("frames62").canonical_text();
    assert!(
        text.chars().count() == 62,
        "frames-62.gb reads as {} bases, expected 62",
        text.chars().count()
    );
    let circular: Sequence = normalize(
        name("frames62-circular"),
        text.clone(),
        SequenceTopology::Circular,
    )
    .unwrap_or_else(|error| {
        panic!(
            "the 62 base text refused to normalise as circular: {:?}",
            error
        )
    });
    let linear: Sequence = normalize(name("frames62-linear"), text, SequenceTopology::Linear)
        .unwrap_or_else(|error| {
            panic!(
                "the 62 base text refused to normalise as linear: {:?}",
                error
            )
        });
    assert!(
        circular.length() == linear.length(),
        "the same text reads as {} bases circular and {} linear",
        circular.length(),
        linear.length()
    );

    for frame in ReadingFrame::ALL {
        for plus in [true, false] {
            let strand: Strand = if plus { Strand::Plus } else { Strand::Minus };
            let round: Vec<FrameResidue> = whole_frame(&circular, strand.clone(), frame);
            let straight: Vec<FrameResidue> = whole_frame(&linear, strand, frame);
            assert!(
                round == straight,
                "frame {} on the {} strand of the circular record gives {:?}, its linear twin gives {:?}",
                frame.index(),
                row_symbol(plus),
                round,
                straight
            );
            // A circular record is not longer at the far end: 62 bases hold 20
            // whole codons in a frame that starts on a base congruent to 1, and
            // a rotation would have added a 21st codon out of the last bases.
            assert!(
                round.len() == 20,
                "frame {} on the {} strand of a circular 62 base record gives {} residues, expected 20: the 21st would need a codon across the origin (D052)",
                frame.index(),
                row_symbol(plus),
                round.len()
            );
        }
    }

    // The two frames whose tail the table names: plus frame 1 leaves bases 61
    // and 62 over (zero-based 60 and 61), and a rotated codon there would have
    // its middle base on one of them. Minus frame 1 leaves bases 1 and 2 over.
    let far_plus: Vec<FrameResidue> = whole_frame(&circular, Strand::Plus, ReadingFrame::First);
    let far_minus: Vec<FrameResidue> = whole_frame(&circular, Strand::Minus, ReadingFrame::First);
    for residue in far_plus.iter() {
        assert!(
            residue.middle != 60 && residue.middle != 61,
            "circular plus frame 1 has a residue numbered {} with middle {}, a position the frame leaves over: a wrapped codon (D052)",
            residue.number,
            residue.middle
        );
    }
    for residue in far_minus.iter() {
        assert!(
            residue.middle != 0 && residue.middle != 1,
            "circular minus frame 1 has a residue numbered {} with middle {}, a position the frame leaves over: a wrapped codon (D052)",
            residue.number,
            residue.middle
        );
    }
    assert!(
        far_plus.last().map(|residue| residue.middle) == Some(58u64),
        "the last codon of circular plus frame 1 has middle {:?}, expected 58: reading stops at base 60 (zero-based 59), not past it",
        far_plus.last().map(|residue| residue.middle)
    );
    assert!(
        far_minus.last().map(|residue| residue.middle) == Some(3u64),
        "the last codon of circular minus frame 1 has middle {:?}, expected 3: the frame reads bases 6, 5 and 4 (1-based) and stops, because a codon at zero-based position 1 would need bases 1, 0 and -1 (D052)",
        far_minus.last().map(|residue| residue.middle)
    );
}

#[test]
fn a_fasta_record_reports_unknown_topology_in_the_table_and_is_read_as_linear() {
    // D049: the view assumes linear for a FASTA record and shows the
    // assumption. The tables report what the *file* says, and a FASTA record
    // says nothing, so their `topology` column reads `unknown`. Both facts are
    // asserted side by side; neither is reconciled into the other.
    for pair in [("frames61", FRAMES61_FILE), ("framesamb", FRAMESAMB_FILE)] {
        let record: FastaRecord = read_fasta(
            name(pair.1),
            read_input(VIEW, pair.1),
            SequenceTopology::Linear,
        )
        .unwrap_or_else(|error| panic!("{}: read_fasta refused it: {:?}", pair.1, error.message()));
        let sequence: Sequence = record.sequence();
        assert!(
            sequence.topology().name() == "linear",
            "{}: core reads the FASTA with the linear assumption D049 gives it, so topology() is {:?}",
            pair.0,
            sequence.topology().name()
        );

        for entry in [
            (TABLES_FILE, TABLES_HEADER, TABLES_COLUMNS),
            (TAILS_FILE, TAILS_HEADER, TAILS_COLUMNS),
        ] {
            let file: &str = entry.0;
            let found: Vec<Vec<String>> = rows_of(entry.0, entry.1, entry.2)
                .iter()
                .filter(|row| row[0] == pair.0)
                .cloned()
                .collect();
            assert!(
                !found.is_empty(),
                "{}: no row names record {}",
                file,
                pair.0
            );
            for row in found.iter() {
                assert!(
                    row[2] == "unknown",
                    "{}: the row for {} reports topology {:?}, the file's own statement, while core reads the same file as {:?} (D049 shows that assumption rather than writing it into the file)",
                    file,
                    pair.0,
                    row[2],
                    sequence.topology().name()
                );
                assert!(
                    number(&row[1]) == sequence.length(),
                    "{}: the row for {} gives seq_len {}, core reads {} bases from {}",
                    file,
                    pair.0,
                    row[1],
                    sequence.length(),
                    pair.1
                );
            }
        }

        // The record the other tests use is the one `read_fasta` gives, read
        // from the same file.
        assert!(
            sequence.canonical_text() == record_of(pair.0).canonical_text(),
            "{}: the FASTA record read here and the one read for the tables are different sequences",
            pair.0
        );
    }
}

#[test]
fn the_ambiguous_record_reads_letter_star_and_question_mark() {
    // The two runs of `framesamb` frame 1, on both strands, as the table's `aa`
    // column writes them and as the literals at the top of this file record
    // them. Every `?` position is the point: an ambiguous codon is never
    // collapsed to the letter all its expansions agree on (D010, D052), and a
    // frame does not stop at `*` or `?`.
    let sequence: Sequence = record_of("framesamb");
    for pair in [
        ("+", AMBIGUOUS_PLUS, Strand::Plus, ReadingFrame::First),
        ("-", AMBIGUOUS_MINUS, Strand::Minus, ReadingFrame::First),
    ] {
        let rows: Vec<Vec<String>> = rows_of(TABLES_FILE, TABLES_HEADER, TABLES_COLUMNS)
            .iter()
            .filter(|row| row[0] == "framesamb" && row[3] == pair.0 && row[4] == "1")
            .cloned()
            .collect();
        assert!(
            rows.len() == 11,
            "framesamb {} frame 1: {} rows in the table, expected 11 codons",
            pair.0,
            rows.len()
        );
        // Reading the column is not the same as calling core: the run below
        // comes out of the table's rows in the table's own order.
        let from_table: String = rows.iter().map(|row| fixture_symbol(&row[7])).collect();
        assert!(
            from_table == pair.1,
            "framesamb {} frame 1: the table's aa column reads {:?}, expected {:?}",
            pair.0,
            from_table,
            pair.1
        );
        let residues: Vec<FrameResidue> = whole_frame(&sequence, pair.2, pair.3);
        let from_core: String = symbols(residues.as_slice());
        assert!(
            residues.len() == 11,
            "framesamb {} frame 1: core produces {} residues ({:?}), expected 11 codons",
            pair.0,
            residues.len(),
            from_core
        );
        assert!(
            from_core == pair.1,
            "framesamb {} frame 1: core reads {:?}, expected {:?}",
            pair.0,
            from_core,
            pair.1
        );
        assert!(
            from_core == from_table,
            "framesamb {} frame 1: core reads {:?} where the table reads {:?}",
            pair.0,
            from_core,
            from_table
        );
        // Neither `*` nor `?` ends the frame: each run holds its markers and
        // still carries on past the last of them (D050).
        let unreadable: usize = from_core.chars().filter(|ch| *ch == '?').count();
        assert!(
            unreadable > 0 && from_core.chars().count() == 11,
            "framesamb {} frame 1: expected `?` codons inside an eleven symbol run, got {:?}",
            pair.0,
            from_core
        );
        let first_marker: Option<usize> = from_core
            .char_indices()
            .filter(|entry| entry.1 == '*' || entry.1 == '?')
            .map(|entry| entry.0)
            .next();
        match first_marker {
            None => panic!(
                "framesamb {} frame 1: {:?} holds no `*` and no `?`, so it pins nothing about where a frame ends",
                pair.0,
                from_core
            ),
            Some(index) => assert!(
                index + 1 < 11,
                "framesamb {} frame 1: {:?} has no codon after its first `*` or `?` at index {}; a frame carries on past both, which is how it differs from mutant_residues (D050)",
                pair.0,
                from_core,
                index
            ),
        }
    }
}

#[test]
fn a_reversed_window_is_refused() {
    let sequence: Sequence = record_of("frames62");
    for frame in ReadingFrame::ALL {
        for plus in [true, false] {
            let strand: Strand = if plus { Strand::Plus } else { Strand::Minus };
            let outcome: Result<Vec<FrameResidue>, DomainError> =
                frame_residues(&sequence, strand, frame, 30, 29);
            match outcome {
                Ok(residues) => panic!(
                    "frame {} on the {} strand: window [30, 29) gives {} residues, expected a refusal",
                    frame.index(),
                    row_symbol(plus),
                    residues.len()
                ),
                Err(error) => match error {
                    DomainError::ReversedInterval(problem) => {
                        assert!(
                            problem.start == 30 && problem.end == 29,
                            "frame {} on the {} strand: the refusal carries start {} end {}, expected 30 and 29",
                            frame.index(),
                            row_symbol(plus),
                            problem.start,
                            problem.end
                        );
                        assert!(
                            problem.sequence == sequence.display_name(),
                            "the refusal names {:?}, expected the record {:?}",
                            problem.sequence,
                            sequence.display_name()
                        );
                    }
                    other => panic!(
                        "frame {} on the {} strand: a reversed window is refused as {:?}, expected reversed_interval",
                        frame.index(),
                        row_symbol(plus),
                        other.code()
                    ),
                },
            }
        }
    }

    // An empty window is legal and gives nothing, on a record that has codons.
    for frame in ReadingFrame::ALL {
        let empty: Vec<FrameResidue> = frame_residues(&sequence, Strand::Plus, frame, 30, 30)
            .unwrap_or_else(|error| {
                panic!(
                    "frame {}: an empty window [30, 30) is refused: {:?}",
                    frame.index(),
                    error.message()
                )
            });
        assert!(
            empty.is_empty(),
            "frame {}: an empty window [30, 30) gives {:?}, expected nothing",
            frame.index(),
            middles(empty.as_slice())
        );
    }
}
