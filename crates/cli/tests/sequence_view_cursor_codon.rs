//! The cursor's codon is reversed and bold on the selected CDS's residue line,
//! as well as underlined on the line of its bases (contract 8m8 §4, M8-32).
//!
//! `pbr` is `tests/fixtures/genbank/raw/J01749.1.gb` and `f62` is
//! `tests/fixtures/view/frames-62.gb`. The columns come from the records, not
//! from what the code draws: `pbr`'s `d = 4` gives 66-base lines, so base `p` of
//! its block 1 — bases 66 to 129, 8m4's — stands at screen column `6 + p - 66`,
//! and `f62`'s `d = 2` gives one line holding the 62 bp record, where base `b`
//! stands at column `4 + b` (8m5's geometry).
//!
//! The letters and the codons come from the files: `tet` is `86..1276` on the
//! plus strand with `/translation="MKSNNAL..."`, so its codon 1 is zero-based
//! bases 85, 86, 87 — the file's `atg` at 86 to 88 — and codon 2 is 88, 89, 90,
//! the file's `aaa` at 89 to 91, which is `K`. `fP` is `join(56..62,1..11)` with
//! `/translation="MRTDH"`, so its codon 1 is 55, 56, 57 (the `atg` at 56 to 58)
//! and its codon 3 is 61, 0 and 1, the two pieces the origin
//! splits: the base at 1-based 62, then the bases at 1-based 1 and 2.
//!
//! Core normalises a sequence's letters to upper case (D008), so the bases line
//! spells a codon's letters upper-cased: `tet`'s `atg` is drawn `ATG`.
//!
//! A residue letter stands over its codon's middle base and its codon's other
//! two bases are spaces, so the residue row is found by the letter and the two
//! spaces that make the codon, and the bases row by the codon's own three
//! letters; no fixed row number is used, since `layout` decides where a block's
//! lines fall.

use luria_cli::sequence_view::layout::layout;
use luria_cli::sequence_view::layout::LaneMode;
use luria_cli::sequence_view::layout::Layout;
use luria_cli::sequence_view::screen::draw;
use luria_cli::sequence_view::state::Action;
use luria_cli::sequence_view::state::Key;
use luria_cli::sequence_view::state::ViewState;
use luria_core::identity::Identifier;
use luria_core::view_record::open_record;
use luria_core::view_record::ViewRecord;
use ratatui::backend::TestBackend;
use ratatui::buffer::Cell;
use ratatui::style::Modifier;
use ratatui::Terminal;
use std::fs;
use std::path::Path;

/// Every test draws at this size: 80 columns and 24 rows, as every other
/// sequence-view test does.
const W: u16 = 80;
const H: u16 = 24;

/// The inner width `layout` is measured at: 80 columns less the frame's two
/// border columns (8k1 §4 rule 1).
const INNER: u16 = 78;

fn record(name: &str, path: &str) -> ViewRecord {
    let file: std::path::PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text: String =
        fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"));
    open_record(Identifier::create(name.to_string()).unwrap(), text)
        .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
}

fn pbr() -> ViewRecord {
    record("SYNPBR322", "../../tests/fixtures/genbank/raw/J01749.1.gb")
}

/// The 62 bp view fixture: `fP` = `join(56..62,1..11)` first, then `fM` =
/// `complement(join(53..62,1..8))`.
fn f62() -> ViewRecord {
    record("frames62", "../../tests/fixtures/view/frames-62.gb")
}

/// The keys, pressed one after the other, none of which leaves the view.
fn press(state: &mut ViewState, keys: &[Key]) {
    for key in keys {
        assert_eq!(
            state.handle(*key),
            Action::Continue,
            "key {key:?} does not leave"
        );
    }
}

/// `text` typed one character per key, then `enter` (8k2 §4 rule 2).
fn go_to(state: &mut ViewState, text: &str) {
    press(state, &[Key::Char('g')]);
    for ch in text.chars() {
        press(state, &[Key::Char(ch)]);
    }
    press(state, &[Key::Enter]);
}

/// `key` pressed `n` times.
fn walk(state: &mut ViewState, key: Key, n: usize) {
    for _ in 0..n {
        press(state, &[key]);
    }
}

/// The cells of the drawn screen, row by row, as the test backend holds them.
fn drawn(state: &ViewState) -> Vec<Vec<Cell>> {
    let mut terminal = Terminal::new(TestBackend::new(W, H)).unwrap();
    terminal.draw(|frame| draw(frame, state)).unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = usize::from(buffer.area.width);
    buffer
        .content()
        .chunks(width)
        .map(|row: &[Cell]| row.to_vec())
        .collect()
}

/// The one row of the screen that spells `text` at the columns starting at
/// `first`: `layout` decides which row of a block a line falls on, and the
/// overview strip takes two rows above the blocks, so the rows are found by what
/// they hold rather than by a number. The spelling is unique: a residue's letter
/// and the spaces beside it, or a codon's three bases, appear nowhere else.
fn row_spelling(rows: &[Vec<Cell>], first: usize, text: &str) -> usize {
    let found: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_row, cells)| {
            first + text.chars().count() <= cells.len()
                && text
                    .chars()
                    .zip(cells[first..].iter())
                    .all(|(ch, cell)| cell.symbol().chars().eq([ch]))
        })
        .map(|(row, _cells)| row)
        .collect();
    assert_eq!(
        found.len(),
        1,
        "one row spells {text:?} at column {first}, not {found:?} in a screen that \
         also holds {:?}",
        rows.iter()
            .map(|row| row.iter().map(Cell::symbol).collect::<String>())
            .collect::<Vec<String>>()
    );
    found[0]
}

/// Whether a cell carries the M8-32 mark: `REVERSED` and `BOLD` together.
fn marked(rows: &[Vec<Cell>], row: usize, column: usize) -> bool {
    let modifier: Modifier = rows[row][column].modifier;
    modifier.contains(Modifier::REVERSED) && modifier.contains(Modifier::BOLD)
}

/// Every cell of the drawn screen that is reversed and bold.
fn marked_everywhere(rows: &[Vec<Cell>]) -> Vec<(usize, usize)> {
    rows.iter()
        .enumerate()
        .flat_map(|(row, cells)| {
            cells
                .iter()
                .enumerate()
                .filter(|(_column, cell)| {
                    cell.modifier.contains(Modifier::REVERSED)
                        && cell.modifier.contains(Modifier::BOLD)
                })
                .map(move |(column, _cell)| (row, column))
        })
        .collect()
}

/// `pbr`'s block 1 is bases 66 to 129 at 66 bases a line, so base `p` of it
/// stands at screen column `6 + p - 66` (8m4, 8m5's geometry).
fn pbr_column(p: u64) -> usize {
    usize::try_from(6 + p - 66).expect("a base of block 1 fits the 80-column screen")
}

/// `f62` is one line holding all 62 bases with a gutter of 2, so base `b` stands
/// at screen column `4 + b` (8m5's geometry).
fn f62_column(b: u64) -> usize {
    usize::try_from(4 + b).expect("a base of the one line fits the 80-column screen")
}

/// `c`, then `enter` on the first row of the CDS list, which selects `tet`: two
/// usable CDSs means M8-11 selects none at the open, so the pick is what makes
/// the residue line and the cursor's codon (8m1 §5).
fn select_tet(state: &mut ViewState) {
    press(state, &[Key::Char('c'), Key::Home, Key::Enter]);
    assert_eq!(state.selected(), Some(0), "tet is the file's first CDS");
    assert_eq!(
        state.cursor(),
        85,
        "the pick puts the cursor on tet's start codon"
    );
}

/// Rule 1 and 2: the codon's three cells on the residue line are reversed and
/// bold, and only those three; the bases line keeps the cursor's reversal and
/// the codon's underline as 8k1 draws them.
#[test]
fn the_cursor_codon_is_marked_on_the_residue_line() {
    let record: ViewRecord = pbr();
    let laid: Layout = layout(&record, INNER, &LaneMode::Cds(0)).expect("a layout at width 78");
    assert_eq!(
        laid.blocks[1],
        (66, 130),
        "block 1 is 8m4's, bases 67 to 130"
    );

    let mut state: ViewState = ViewState::new(record, W, H);
    select_tet(&mut state);
    assert_eq!(
        state.mode(),
        &LaneMode::Cds(0),
        "and the lanes show its residues"
    );

    // Codon 1 is bases 85, 86 and 87 — the file's `atg` at 86 to 88 — at columns
    // 25, 26 and 27, with `M` over the middle base at 26.
    let columns: [usize; 3] = [pbr_column(85), pbr_column(86), pbr_column(87)];
    assert_eq!(columns, [25, 26, 27], "the columns 8m4's block gives");
    let rows: Vec<Vec<Cell>> = drawn(&state);
    let residues: usize = row_spelling(&rows, columns[0], " M ");
    let bases: usize = row_spelling(&rows, columns[0], "ATG");
    assert_ne!(
        residues, bases,
        "the residue line and the bases line are two rows"
    );
    for column in columns {
        assert!(
            marked(&rows, residues, column),
            "codon 1's cell at column {column} on the residue line is reversed and bold"
        );
    }
    for column in [columns[0] - 1, columns[2] + 1] {
        assert!(
            !marked(&rows, residues, column),
            "column {column} beside the codon is not marked"
        );
    }

    // Rule 2 again after three bases: codon 2 is bases 88, 89, 90 — the file's
    // `aaa` at 89 to 91, which is `K`, and its letter stands over 89 — at
    // columns 28, 29 and 30, and codon 1 is not marked any more.
    walk(&mut state, Key::Char('l'), 3);
    assert_eq!(
        state.cursor(),
        88,
        "three bases is the next codon's first base"
    );
    let columns: [usize; 3] = [pbr_column(88), pbr_column(89), pbr_column(90)];
    assert_eq!(columns, [28, 29, 30], "the columns the same block gives");
    let rows: Vec<Vec<Cell>> = drawn(&state);
    let residues: usize = row_spelling(&rows, columns[0], " K ");
    let bases: usize = row_spelling(&rows, columns[0], "AAA");
    for column in columns {
        assert!(
            marked(&rows, residues, column),
            "codon 2's cell at column {column} is marked once the cursor is in it"
        );
    }
    for column in [25, 26, 27] {
        assert!(
            !marked(&rows, residues, column),
            "codon 1 at column {column} is not marked now the cursor has left it"
        );
    }

    // Rule 4: the bases line is what 8k1 draws it to be — the cursor's own cell
    // reversed, the whole codon underlined, nothing bold.
    assert!(
        rows[bases][28].modifier.contains(Modifier::REVERSED),
        "the cursor's own cell on the bases line is still reversed"
    );
    for column in [28, 29, 30] {
        assert!(
            rows[bases][column].modifier.contains(Modifier::UNDERLINED),
            "the codon's cell at column {column} on the bases line is still underlined"
        );
        assert!(
            !rows[bases][column].modifier.contains(Modifier::BOLD),
            "and nothing on the bases line is bold"
        );
    }
}

/// Rule 2's last sentence: a codon the origin splits is marked on its three
/// bases, which are at the two ends of the line, and nowhere between them.
#[test]
fn a_codon_split_by_the_origin_marks_both_pieces() {
    let record: ViewRecord = f62();
    let laid: Layout = layout(&record, INNER, &LaneMode::Cds(0)).expect("a layout at width 78");
    assert_eq!(laid.blocks.len(), 1, "the 62 bp record is one line (8m5)");
    assert_eq!(laid.blocks[0], (0, 62), "the one block is the record whole");

    let mut state: ViewState = ViewState::new(record, W, H);
    press(&mut state, &[Key::Char('c'), Key::Home, Key::Enter]);
    assert_eq!(state.selected(), Some(0), "fP is the file's first CDS");
    assert_eq!(
        state.cursor(),
        55,
        "the pick puts the cursor on fP's start codon"
    );
    go_to(&mut state, "1");
    assert_eq!(state.cursor(), 0, "base 1 of the record, zero-based 0");
    assert_eq!(
        state.selected(),
        Some(0),
        "and the jump keeps the selection"
    );

    // Codon 3 is the origin's: bases 61, 0 and 1, at columns 65, 4 and 5. Codon
    // 1, bases 55, 56 and 57 — the `atg` at 56 to 58 — still spells M over
    // columns 59 to 61, and gives the row.
    let pieces: [usize; 3] = [f62_column(61), f62_column(0), f62_column(1)];
    assert_eq!(
        pieces,
        [65, 4, 5],
        "the split codon's two pieces and its middle base"
    );
    let rows: Vec<Vec<Cell>> = drawn(&state);
    let residues: usize = row_spelling(&rows, f62_column(55), " M ");
    for column in pieces {
        assert!(
            marked(&rows, residues, column),
            "the split codon's cell at column {column} is reversed and bold"
        );
    }
    for column in [f62_column(2), f62_column(60)] {
        assert!(
            !marked(&rows, residues, column),
            "column {column} beside either piece is not marked"
        );
    }
    let between: Vec<usize> = (7..=63)
        .filter(|column| marked(&rows, residues, *column))
        .collect();
    assert!(
        between.is_empty(),
        "nothing between the two pieces is marked: {between:?}"
    );
}

/// Rule 3: with the cursor outside the selected CDS nothing is marked, and `C`
/// does not remove the mark.
#[test]
fn no_codon_no_mark() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    select_tet(&mut state);
    press(&mut state, &[Key::Home]);
    assert_eq!(
        state.cursor(),
        0,
        "Home takes the cursor to the record's first base"
    );
    assert_eq!(state.selected(), Some(0), "and keeps tet selected");
    let rows: Vec<Vec<Cell>> = drawn(&state);
    assert!(
        marked_everywhere(&rows).is_empty(),
        "no cell of any row is reversed and bold, but {:?}",
        marked_everywhere(&rows)
    );

    // The same with the colours off: the mark is on the cells of the codon's
    // three positions, which is not what the colours do.
    select_tet(&mut state);
    press(&mut state, &[Key::Char('C')]);
    assert!(!state.colours(), "C turns the colours off");
    let rows: Vec<Vec<Cell>> = drawn(&state);
    let residues: usize = row_spelling(&rows, 25, " M ");
    for column in [25, 26, 27] {
        assert!(
            marked(&rows, residues, column),
            "the residue line's cell at column {column} is marked with the colours off"
        );
    }
}
