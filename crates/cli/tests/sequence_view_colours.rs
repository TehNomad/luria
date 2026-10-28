//! The residues coloured by Zappo group, and `C` turning them off and on
//! (contract 8m5 §4 rules 1 to 6, M8-29). Colours are drawing only: no block,
//! row or column changes, so every column here is one the layout tables and the
//! records' own coordinates already give.
//!
//! The columns come from the records, not from what the code draws. `f62` is
//! `tests/fixtures/view/frames-62.gb`, 62 bp, `d = 2`, so at 80 × 24 it lays out
//! as one 72-base line where base `b` stands at screen column `4 + b`; its `fP`
//! is `join(56..62,1..11)` with `/translation="MRTDH"`, so the zero-based codons
//! are M at 55 to 57, R at 58 to 60, T at 61 and 0 to 1 (split by the origin), D
//! at 2 to 4, H at 5 to 7 and the stop at 8 to 10. `pbr` is `J01749.1.gb`, 4558
//! bp, `d = 4`, so its lines hold 66 bases and base `b` of a line starting at
//! base `s` stands at column `6 + b - s`; frame 3 reads from base 2.
//!
//! A residue's letter stands over its codon's middle base, in black, with that
//! codon's group colour behind it; the codon's other two bases are spaces with
//! the same background; and a codon split between two lines is coloured on each
//! line for the bases that line holds, with the letter only where the middle
//! base is.

use luria_cli::sequence_view::layout::LaneMode;
use luria_cli::sequence_view::screen::draw;
use luria_cli::sequence_view::state::Action;
use luria_cli::sequence_view::state::Key;
use luria_cli::sequence_view::state::ViewState;
use luria_core::identity::Identifier;
use luria_core::view_record::open_record;
use luria_core::view_record::ViewRecord;
use ratatui::backend::TestBackend;
use ratatui::buffer::Cell;
use ratatui::style::Color;
use ratatui::Terminal;
use std::fs;
use std::path::Path;

/// The same size every other sequence-view test draws at: 80 columns, 24 rows.
const W: u16 = 80;
const H: u16 = 24;

/// The Zappo group colours, as `layout` maps them.
const ALIPHATIC: Color = Color::Indexed(217);
const POSITIVE: Color = Color::Indexed(63);
const NEGATIVE: Color = Color::Indexed(196);
const HYDROPHILIC: Color = Color::Indexed(46);

fn fixture(path: &str) -> String {
    let file: std::path::PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"))
}

fn record(name: &str, path: &str) -> ViewRecord {
    open_record(Identifier::create(name.to_string()).unwrap(), fixture(path))
        .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
}

fn pbr() -> ViewRecord {
    record("SYNPBR322", "../../tests/fixtures/genbank/raw/J01749.1.gb")
}

/// The 62 bp view fixture: `fP` = `join(56..62,1..11)`, `fM` =
/// `complement(join(53..62,1..8))` (D010's example).
fn f62() -> ViewRecord {
    record("f62", "../../tests/fixtures/view/frames-62.gb")
}

/// `text` typed one character per key, and the view never quits.
fn type_into(state: &mut ViewState, text: &str) {
    for ch in text.chars() {
        assert_eq!(
            state.handle(Key::Char(ch)),
            Action::Continue,
            "typing {ch:?} does not leave"
        );
    }
}

/// The keys, pressed one after the other.
fn press(state: &mut ViewState, keys: &[Key]) {
    for key in keys {
        assert_eq!(
            state.handle(*key),
            Action::Continue,
            "{key:?} does not leave"
        );
    }
}

/// The cells of the drawn screen, row by row — the `sequence_view_strands.rs`
/// reading of the test backend.
fn drawn(state: &ViewState) -> Vec<Vec<Cell>> {
    drawn_at(state, W, H)
}

/// The same reading at a screen of the given size, which `a_frame_codon_split_by_
/// a_block_edge` needs because 8m10's ruler line takes a row in every block.
fn drawn_at(state: &ViewState, width: u16, height: u16) -> Vec<Vec<Cell>> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| draw(frame, state)).unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = usize::from(buffer.area.width);
    buffer
        .content
        .chunks(width)
        .map(|row: &[Cell]| row.to_vec())
        .collect()
}

fn symbol(rows: &[Vec<Cell>], row: usize, column: usize) -> String {
    rows[row][column].symbol().to_string()
}

fn bg(rows: &[Vec<Cell>], row: usize, column: usize) -> Color {
    rows[row][column].bg
}

fn fg(rows: &[Vec<Cell>], row: usize, column: usize) -> Color {
    rows[row][column].fg
}

/// The columns of `row` that carry a background at all.
fn coloured(rows: &[Vec<Cell>], row: usize) -> Vec<usize> {
    (0..usize::from(W))
        .filter(|column| bg(rows, row, *column) != Color::Reset)
        .collect()
}

/// One row of the drawn screen as text, with the frame's border rule and the
/// padding it brings trimmed off.
fn row_text(cells: &[Cell]) -> String {
    cells
        .iter()
        .map(Cell::symbol)
        .collect::<String>()
        .trim_matches(|c| c == ' ' || c == '\u{2502}')
        .to_string()
}

/// The rows that name `label` first: the frame lines name their strand and
/// frame (`+3`), and the key list's rows name their keys.
fn labelled(rows: &[Vec<Cell>], label: &str) -> Vec<usize> {
    rows.iter()
        .enumerate()
        .filter(|(_row, cells)| row_text(cells).starts_with(label))
        .map(|(row, _cells)| row)
        .collect()
}

/// The one row that draws the selected CDS's residues: the row holding the
/// stop's `*`, which no other row of the view has.
fn stop_row(rows: &[Vec<Cell>]) -> usize {
    let found: Vec<usize> = rows
        .iter()
        .enumerate()
        .filter(|(_row, cells)| cells.iter().any(|cell| cell.symbol() == "*"))
        .map(|(row, _cells)| row)
        .collect();
    assert_eq!(found.len(), 1, "exactly one row draws the stop");
    found[0]
}

/// `fP`'s residue line, selected by `c` and `enter` on the first item: base `b`
/// of the one 72-base line stands at column `4 + b`, so the codons of M, R, T,
/// D and H are behind bases 55 to 57, 58 to 60, 61 and 0 to 1, 2 to 4 and 5 to 7
/// (§4 rule 1) and the stop's codon behind 8 to 10, with nothing behind it
/// (rule 2).
#[test]
fn a_cds_codon_is_coloured_across_its_bases() {
    let mut state: ViewState = ViewState::new(f62(), W, H);
    type_into(&mut state, "c");
    press(&mut state, &[Key::Home, Key::Enter]);
    assert_eq!(
        state.mode(),
        &LaneMode::Cds(0),
        "enter selects fP, the record's first CDS"
    );
    assert!(state.colours(), "the colours are on at open (rule 5)");

    let cells: Vec<Vec<Cell>> = drawn(&state);
    let row: usize = stop_row(&cells);

    // M, bases 55 to 57, the letter over base 56.
    for column in 59..=61 {
        assert_eq!(bg(&cells, row, column), ALIPHATIC, "M's codon at {column}");
    }
    assert_eq!(symbol(&cells, row, 60), "M", "the letter of M");
    assert_eq!(fg(&cells, row, 60), Color::Black, "in black");

    // R, bases 58 to 60, the letter over 59: positive, not hydrophilic (G4).
    for column in 62..=64 {
        assert_eq!(bg(&cells, row, column), POSITIVE, "R's codon at {column}");
    }
    assert_eq!(symbol(&cells, row, 63), "R", "the letter of R");

    // T: base 61 at the far end of the line and bases 0 and 1 at the other, the
    // letter over base 0. The codon is split by the origin, and this line holds
    // all three of its bases.
    assert_eq!(bg(&cells, row, 65), HYDROPHILIC, "T's first base");
    assert_eq!(bg(&cells, row, 4), HYDROPHILIC, "T's middle base");
    assert_eq!(bg(&cells, row, 5), HYDROPHILIC, "T's third base");
    assert_eq!(symbol(&cells, row, 4), "T");
    assert_eq!(symbol(&cells, row, 5), " ", "the third base is a space");
    assert_eq!(
        symbol(&cells, row, 65),
        " ",
        "and the first base is one too"
    );

    // D at bases 2 to 4 and H at 5 to 7.
    for column in 6..=8 {
        assert_eq!(bg(&cells, row, column), NEGATIVE, "D's codon at {column}");
    }
    assert_eq!(symbol(&cells, row, 7), "D");
    for column in 9..=11 {
        assert_eq!(bg(&cells, row, column), POSITIVE, "H's codon at {column}");
    }
    assert_eq!(symbol(&cells, row, 10), "H");

    // The stop's codon at bases 8 to 10 has no colour, and its `*` keeps the
    // look 8k1 gives it.
    for column in 12..=14 {
        assert_eq!(
            bg(&cells, row, column),
            Color::Reset,
            "nothing behind the stop at {column}"
        );
    }
    assert_eq!(symbol(&cells, row, 13), "*");
    assert_eq!(
        fg(&cells, row, 13),
        Color::Red,
        "the stop keeps its own colour"
    );

    // Nothing between the codons and after them: the gaps, and the bases the
    // CDS has no codon for.
    for column in 15..=58 {
        assert_eq!(
            bg(&cells, row, column),
            Color::Reset,
            "no background at {column}"
        );
    }
    assert_eq!(
        coloured(&cells, row).len(),
        15,
        "three bases behind each of the five residues, and nothing else"
    );
    for column in [4, 7, 10, 60, 63] {
        assert_eq!(
            fg(&cells, row, column),
            Color::Black,
            "the letter at {column}"
        );
    }
}

/// Rule 4: `C` turns the backgrounds off and on again, and the letters stay
/// where they were; inside a list `C` is filter text, as every other printable
/// key there (8m1 §4 rule 3).
#[test]
fn c_turns_the_colours_off_and_on() {
    let mut state: ViewState = ViewState::new(f62(), W, H);
    type_into(&mut state, "c");
    press(&mut state, &[Key::Home, Key::Enter]);
    let before: Vec<Vec<Cell>> = drawn(&state);
    let row: usize = stop_row(&before);
    let letters: Vec<String> = before[row]
        .iter()
        .map(Cell::symbol)
        .map(str::to_string)
        .collect();

    assert_eq!(state.handle(Key::Char('C')), Action::Continue, "`C`");
    assert!(!state.colours(), "`C` turns the colours off");
    let cells: Vec<Vec<Cell>> = drawn(&state);
    let row: usize = stop_row(&cells);
    assert_eq!(
        cells[row]
            .iter()
            .map(Cell::symbol)
            .map(str::to_string)
            .collect::<Vec<String>>(),
        letters,
        "the same letters in the same columns with the colours off"
    );
    assert!(
        coloured(&cells, row).is_empty(),
        "no cell of the residue line carries a background"
    );

    assert_eq!(state.handle(Key::Char('C')), Action::Continue, "`C` again");
    assert!(state.colours(), "`C` again turns them on");
    let cells: Vec<Vec<Cell>> = drawn(&state);
    let row: usize = stop_row(&cells);
    assert_eq!(
        cells[row]
            .iter()
            .map(Cell::symbol)
            .map(str::to_string)
            .collect::<Vec<String>>(),
        letters,
        "the letters are back where they were"
    );
    for column in 59..=61 {
        assert_eq!(
            bg(&cells, row, column),
            ALIPHATIC,
            "the colour behind M at {column}"
        );
    }
    for column in [4, 5, 65] {
        assert_eq!(
            bg(&cells, row, column),
            HYDROPHILIC,
            "the colour behind T at {column}"
        );
    }

    type_into(&mut state, "f");
    type_into(&mut state, "C");
    assert_eq!(
        state.list().expect("`f` opens a list").filter(),
        "C",
        "the filter holds the letter that was typed"
    );
    assert!(
        state.colours(),
        "the list's `C` is not the key that turns colours off"
    );
}

/// Rule 2's frame half: a frame codon standing at the end of one line and the
/// start of the next is coloured on both lines, and the letter stands only on
/// the line that holds its middle base. `pbr` at 80 × 26, 2 rows above the 24 the
/// other tests draw at for the ruler line each of this test's two blocks takes,
/// with three frames showing and the overview strip hidden: frame 3 reads from
/// base 2, so its codon at bases 65 to 67 (`AAC`, `N`) has base 65 at the end of
/// the first line and bases 66 and 67 at the start of the second.
#[test]
fn a_frame_codon_split_by_a_block_edge() {
    // The ruler line 8m10 adds a row to each of the two blocks this test counts:
    // block 0's 10 lines and block 1's 9 become 11 and 10, and 11 + the gap + 10
    // = 22 rows fit only the 22 the body holds without the strip at this height,
    // 2 rows more than the 20 it had at 8l2's.
    let height: u16 = H + 2;
    let mut state: ViewState = ViewState::new(pbr(), W, height);
    type_into(&mut state, "to");
    let cells: Vec<Vec<Cell>> = drawn_at(&state, W, height);
    let rows: Vec<usize> = labelled(&cells, "+3");
    assert_eq!(
        rows.len(),
        2,
        "the +3 line of each drawn block, in drawing order"
    );

    let first: usize = rows[0];
    assert_eq!(
        bg(&cells, first, 71),
        HYDROPHILIC,
        "base 65, the codon's first base at the line's end"
    );
    assert_eq!(
        symbol(&cells, first, 71),
        " ",
        "and no letter for a codon whose middle base is on the next line"
    );

    let second: usize = rows[1];
    for column in 6..=7 {
        assert_eq!(
            bg(&cells, second, column),
            HYDROPHILIC,
            "the codon's bases at the start of the next line, {column}"
        );
    }
    assert_eq!(symbol(&cells, second, 6), "N");
    assert_eq!(symbol(&cells, second, 7), " ");
}

/// Rule 6: the `?` list names the key, right after `F`'s row.
#[test]
fn the_key_list_names_c() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert_eq!(state.handle(Key::Char('?')), Action::Continue);
    let cells: Vec<Vec<Cell>> = drawn(&state);
    let rows: Vec<usize> = labelled(&cells, "C");
    assert!(!rows.is_empty(), "some row of the list names the key");
    let text: String = row_text(&cells[rows[0]]);
    assert_eq!(
        text, "C  colour the residues and the features, and back",
        "the row names the key and what it does"
    );
}
