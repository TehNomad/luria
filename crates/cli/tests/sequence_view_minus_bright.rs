//! M8-36 (contract 8m14): while the lanes show a minus-strand CDS, the strand
//! that CDS reads is the one drawn as primary — the bottom line's letters are
//! bright and the top line's are dim, the reverse of 8m9, and the cursor's cell
//! and its codon's underline stand on the bottom row. In every other mode — no
//! translation, a plus-strand CDS, the three and six frames — the top line keeps
//! the look 8m9 gives it and the marks stay on the top row.
//!
//! `pbr` is `tests/fixtures/genbank/raw/J01749.1.gb`, 4361 bp and circular, so
//! its gutter is four columns wide and its lines hold 66 bases: a block's first
//! base stands at screen column `1 + 4 + 1 = 6` — the frame's left border, the
//! gutter's four columns — and base `p` of the block starting at `s` stands at
//! column `6 + p - s`. A block whose line ends inside a codon of the selected CDS
//! is 1 or 2 bases shorter (D055), so the columns its bases fill come from the
//! layout, as everything else here does.
//!
//! `bla` is CDS index 2, `complement(3293..4153)` — zero-based 3292 to 4152 —
//! and picking it out of the `c` list (`Home`, `Down` twice, `enter`) selects it,
//! turns its lane on and puts the cursor on 4152, its first reading base
//! (M8-21), whose codon is 4152, 4151, 4150. `tet` is CDS index 0, `86..1276` on
//! the plus strand, and its pick puts the cursor on 85 with the codon 85, 86, 87.
//!
//! The rows are found by what they hold, never by a fixed number: the top row of
//! a block is the row whose gutter holds that block's first base's number, as
//! `tests/sequence_view_styling.rs` finds them, and the row under it is where
//! `layout` puts the complements (D055 order). The blocks' ranges come from
//! `state.layout()` too: `bla`'s cursor block starts at zero-based 4090 and
//! `tet`'s at 66, because D050's phase makes the blocks snap to the selected
//! CDS's reading, and the three frames, which show no CDS, keep whole lines.

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

/// The column a block's first base stands on: the frame's left border, then the
/// gutter's four columns of pBR322's 4361 bp, then the bases.
const FIRST: usize = 6;

fn record(name: &str, path: &str) -> ViewRecord {
    let file: std::path::PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text: String =
        fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"));
    open_record(Identifier::create(name.to_string()).unwrap(), text)
        .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
}

/// pBR322: 4361 bp and circular, as its LOCUS line states both.
fn pbr() -> ViewRecord {
    record("SYNPBR322", "../../tests/fixtures/genbank/raw/J01749.1.gb")
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

/// The letter drawn in one cell, which is one of the record's bases or one of
/// their complements.
fn letter(rows: &[Vec<Cell>], row: usize, column: usize) -> char {
    let symbol: String = rows[row][column].symbol().chars().collect();
    assert_eq!(
        symbol.chars().count(),
        1,
        "column {column} of row {row} holds one letter, not {symbol:?}"
    );
    symbol.chars().next().expect("one letter")
}

/// The base its complement on the other strand spells, as the view draws the
/// letters upper-cased (D008).
fn complement(letter: char) -> char {
    match letter {
        'A' => 'T',
        'T' => 'A',
        'G' => 'C',
        'C' => 'G',
        other => other,
    }
}

/// Whether the cell at `row`, `column` carries `modifier`.
fn has(rows: &[Vec<Cell>], row: usize, column: usize, modifier: Modifier) -> bool {
    rows[row][column].modifier.contains(modifier)
}

fn dim(rows: &[Vec<Cell>], row: usize, column: usize) -> bool {
    has(rows, row, column, Modifier::DIM)
}

/// Every cell of the drawn screen that is reversed and bold — the cursor's codon
/// on the residue line (M8-32).
fn marked(rows: &[Vec<Cell>]) -> Vec<(usize, usize)> {
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

/// The text of one row's columns, `first` and `last` inclusive.
fn slice(rows: &[Vec<Cell>], row: usize, first: usize, last: usize) -> String {
    rows[row][first..=last].iter().map(Cell::symbol).collect()
}

/// The columns a block's bases fill, first and last, from the block's range: base
/// `p` of the block `[s, e)` stands at column `6 + p - s`, and the last base of a
/// block 1 or 2 bases shorter than a whole line is still the last of them.
fn span(block: (u64, u64)) -> std::ops::RangeInclusive<usize> {
    let last: usize = FIRST + (block.1 - block.0) as usize - 1;
    FIRST..=last
}

/// The screen column a base stands on, from the block's range.
fn column(block: (u64, u64), base: u64) -> usize {
    usize::try_from(6 + base - block.0).expect("the base stands inside the block")
}

/// The block `state.layout()` holds `base` in, checked against the contract §5
/// hand value for its start, which the test reads from the layout rather than
/// trusting the number.
fn block(state: &ViewState, base: u64, hand: u64) -> (u64, u64) {
    let built: &Layout = state
        .layout()
        .expect("the view draws a layout at 80 columns and 24 rows");
    let found: Vec<(u64, u64)> = built
        .blocks
        .iter()
        .copied()
        .filter(|(first, last)| *first <= base && base < *last)
        .collect();
    assert_eq!(
        found.len(),
        1,
        "one block of the layout holds base {base}, not {found:?}"
    );
    let held: (u64, u64) = found[0];
    assert_eq!(
        held.0, hand,
        "the block holding base {base} starts at zero-based {hand}, as contract 8m14 \
         §5 gives it"
    );
    held
}

/// The two rows of the block that starts at zero-based `first`, `span` giving the
/// columns its bases stand on: the row whose gutter holds that base's 1-based
/// number, and the row under it, where `layout` puts the complements (D055
/// order). pBR322's gutter is four columns wide, so a four-digit number fills the
/// frame's inner columns 1 to 4.
///
/// The two rows found are checked to hold a base and its complement in every
/// column of the block, which is what makes them the block's two strand rows.
fn block_rows(
    rows: &[Vec<Cell>],
    first: u64,
    bases: std::ops::RangeInclusive<usize>,
) -> (usize, usize) {
    let label: String = format!("{}", first + 1);
    let found: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_row, row)| {
            let gutter: String = row[1..=4].iter().map(Cell::symbol).collect();
            gutter.trim() == label
        })
        .map(|(row, _row)| row)
        .collect();
    assert_eq!(
        found.len(),
        1,
        "one row of the drawn view holds {label} alone in its gutter, not {found:?} \
         in a screen that holds {:?}",
        rows.iter()
            .map(|row| row.iter().map(Cell::symbol).collect::<String>())
            .collect::<Vec<String>>(),
    );
    let top: usize = found[0];
    let bottom: usize = top + 1;
    for column in bases {
        assert_eq!(
            complement(letter(rows, top, column)),
            letter(rows, bottom, column),
            "the row under the {label} row is its complement row, so the rows found \
             are the block's two strand rows"
        );
    }
    (top, bottom)
}

/// Rule 1 for one block: `bright` is the row the mode reads and `other` the row
/// it does not — every base cell of the reading row is not dim and every base
/// cell of the other row is dim, across the block's own columns.
fn assert_bright(
    rows: &[Vec<Cell>],
    bright: usize,
    other: usize,
    bases: std::ops::RangeInclusive<usize>,
) {
    for column in bases {
        assert!(
            !dim(rows, bright, column),
            "the base at column {column} of the row the mode reads is not dim"
        );
        assert!(
            dim(rows, other, column),
            "the base at column {column} of the other strand's row is dim"
        );
    }
}

/// Rule 2 for one block: the cursor's cell and its codon's three cells stand on
/// `reading`, the row the mode reads; the other strand's row holds no mark of
/// theirs at all, across the block's own columns; and the residue line holds
/// exactly the codon's three cells, reversed and bold.
fn assert_marks(
    rows: &[Vec<Cell>],
    reading: usize,
    other: usize,
    columns: [usize; 3],
    cursor: usize,
    bases: std::ops::RangeInclusive<usize>,
) {
    for column in columns {
        assert!(
            has(rows, reading, column, Modifier::UNDERLINED),
            "the codon's cell at column {column} is underlined on the row the mode reads"
        );
    }
    assert!(
        has(rows, reading, cursor, Modifier::REVERSED),
        "the cursor's own cell is reversed on the row the mode reads"
    );
    assert!(
        !has(rows, reading, cursor, Modifier::BOLD),
        "and reversed alone, not bold as the residue line's cells are"
    );
    for column in bases {
        assert!(
            !has(rows, other, column, Modifier::UNDERLINED),
            "column {column} of the other strand's row is not underlined"
        );
        assert!(
            !has(rows, other, column, Modifier::REVERSED),
            "column {column} of the other strand's row is not reversed"
        );
    }
    let marks: Vec<(usize, usize)> = marked(rows);
    assert_eq!(
        marks.len(),
        3,
        "the residue line holds exactly three reversed and bold cells: {marks:?}"
    );
    let mut lines: Vec<usize> = marks.iter().map(|(line, _column)| *line).collect();
    lines.dedup();
    assert_eq!(
        lines.len(),
        1,
        "and those three stand on one residue line, not {lines:?}"
    );
    assert_eq!(
        marks
            .iter()
            .map(|(_line, column)| *column)
            .collect::<Vec<usize>>(),
        columns.to_vec(),
        "at the codon's own three columns"
    );
}

/// The keys that pick `bla` out of the CDS list: `c` opens it, `Home` puts the
/// highlight on tet's row, `Down` twice steps over the ROP row to bla's, and
/// `enter` picks it (8k3 §4, contract 8m14 §5).
fn select_bla(state: &mut ViewState) {
    press(
        state,
        &[Key::Char('c'), Key::Home, Key::Down, Key::Down, Key::Enter],
    );
    assert_eq!(
        state.selected(),
        Some(2),
        "bla is CDS index 2, third of the list's rows"
    );
    assert_eq!(state.mode(), &LaneMode::Cds(2), "and its lane is drawn");
    assert_eq!(
        state.cursor(),
        4152,
        "the pick puts the cursor on bla's first reading base (M8-21)"
    );
}

/// The keys that pick `tet`: `c`, `Home`, `enter`, tet's row being the list's
/// first.
fn select_tet(state: &mut ViewState) {
    press(state, &[Key::Char('c'), Key::Home, Key::Enter]);
    assert_eq!(state.selected(), Some(0), "tet is CDS index 0");
    assert_eq!(state.mode(), &LaneMode::Cds(0), "and its lane is drawn");
    assert_eq!(
        state.cursor(),
        85,
        "the pick puts the cursor on tet's start codon (M8-21)"
    );
}

/// Rule 1 and rule 3: `bla` picked, and the bottom row's base cells are bright
/// while the top row's are dim — the gutter's two numbers dim as ever, and the
/// same look again with the colours off.
#[test]
fn a_minus_cds_brightens_the_bottom_strand() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    select_bla(&mut state);
    let held: (u64, u64) = block(&state, 4152, 4090);
    let bases: std::ops::RangeInclusive<usize> = span(held);
    assert_eq!(
        *bases.end(),
        71,
        "a whole line's bases fill columns 6 to 71"
    );
    let rows: Vec<Vec<Cell>> = drawn(&state);
    let (top, bottom): (usize, usize) = block_rows(&rows, held.0, bases.clone());
    assert_eq!(
        slice(&rows, top, 1, 4),
        "4091",
        "the gutter holds the block's first base number"
    );
    assert_eq!(
        slice(&rows, top, 72, 76),
        " 4156",
        "the number after the block's last base stands at 73, past a blank column"
    );
    for column in 1..=4 {
        assert!(
            dim(&rows, top, column),
            "the gutter's number at column {column} stays dim"
        );
    }
    for column in 73..=76 {
        assert!(
            dim(&rows, top, column),
            "the number after the last base, at column {column}, stays dim"
        );
    }
    assert_bright(&rows, bottom, top, bases.clone());

    press(&mut state, &[Key::Char('C')]);
    assert!(!state.colours(), "C turns the colours off");
    let rows: Vec<Vec<Cell>> = drawn(&state);
    let (top, bottom): (usize, usize) = block_rows(&rows, held.0, bases.clone());
    assert_bright(&rows, bottom, top, bases);
}

/// Rule 1 and rule 3 again: a plus-strand CDS keeps the top row bright, and so do
/// the three frames with a minus-strand CDS picked — where the cursor's cell
/// stays on the top row as well.
#[test]
fn the_other_modes_keep_the_top_bright() {
    // `tet`: a plus-strand CDS, so the top row is the primary one, as 8m9 gives
    // it. Its cursor block ends inside tet's codon 15, so its line is two bases
    // short of a whole one (D055).
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    select_tet(&mut state);
    let held: (u64, u64) = block(&state, 85, 66);
    let bases: std::ops::RangeInclusive<usize> = span(held);
    let rows: Vec<Vec<Cell>> = drawn(&state);
    let (top, bottom): (usize, usize) = block_rows(&rows, held.0, bases.clone());
    assert_bright(&rows, top, bottom, bases.clone());

    // `bla`, then `t`: the frames are shown, so the top row is bright again and
    // the cursor's cell stands on it, not on the complements' row.
    select_bla(&mut state);
    press(&mut state, &[Key::Char('t')]);
    assert_eq!(
        state.mode(),
        &LaneMode::ThreeFrames,
        "t shows the three forward frames"
    );
    assert_eq!(
        state.selected(),
        Some(2),
        "the selection outlives the mode, so a minus-strand CDS is still picked"
    );
    assert_eq!(state.cursor(), 4152, "and the cursor stays where it is");
    // The frames keep whole lines, because no selected CDS snaps their edges.
    let frames: (u64, u64) = block(&state, 4152, 4092);
    let bases: std::ops::RangeInclusive<usize> = span(frames);
    let rows: Vec<Vec<Cell>> = drawn(&state);
    let (top, bottom): (usize, usize) = block_rows(&rows, frames.0, bases.clone());
    assert_bright(&rows, top, bottom, bases.clone());
    let cursor: usize = column(frames, 4152);
    assert!(
        has(&rows, top, cursor, Modifier::REVERSED),
        "the cursor's cell is reversed on the top row"
    );
    assert!(
        !has(&rows, bottom, cursor, Modifier::REVERSED),
        "and not on the complements' row"
    );
}

/// Rule 2 and rule 3: `bla` picked, the cursor's cell and its codon's underline
/// stand on the bottom row and nowhere else of the block, with the colours on and
/// off; the residue line still has its exactly three marked cells.
#[test]
fn a_minus_cds_marks_its_cursor_on_the_bottom_strand() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    select_bla(&mut state);
    let held: (u64, u64) = block(&state, 4152, 4090);
    let bases: std::ops::RangeInclusive<usize> = span(held);
    let columns: [usize; 3] = [column(held, 4150), column(held, 4151), column(held, 4152)];
    assert_eq!(
        columns,
        [66, 67, 68],
        "the columns contract 8m14 §5 gives the codon's three bases at"
    );
    let rows: Vec<Vec<Cell>> = drawn(&state);
    let (top, bottom): (usize, usize) = block_rows(&rows, held.0, bases.clone());
    assert_marks(&rows, bottom, top, columns, columns[2], bases.clone());

    // Rule 3: the same marks with the colours off, where the bottom row is the
    // bright one and the top row holds none of them.
    press(&mut state, &[Key::Char('C')]);
    assert!(!state.colours(), "C turns the colours off");
    let rows: Vec<Vec<Cell>> = drawn(&state);
    let (top, bottom): (usize, usize) = block_rows(&rows, held.0, bases.clone());
    assert_marks(&rows, bottom, top, columns, columns[2], bases);
}

/// Rule 2 again: `tet` picked, so the cursor's cell is reversed on the top row
/// and its codon underlined there, while the complements' row holds neither.
#[test]
fn a_plus_cds_marks_its_cursor_on_the_top_strand() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    select_tet(&mut state);
    let held: (u64, u64) = block(&state, 85, 66);
    let bases: std::ops::RangeInclusive<usize> = span(held);
    let columns: [usize; 3] = [column(held, 85), column(held, 86), column(held, 87)];
    assert_eq!(
        columns,
        [25, 26, 27],
        "the columns the cursor's block gives its codon's three bases at"
    );
    let rows: Vec<Vec<Cell>> = drawn(&state);
    let (top, bottom): (usize, usize) = block_rows(&rows, held.0, bases.clone());
    assert_marks(&rows, top, bottom, columns, columns[0], bases);
}
