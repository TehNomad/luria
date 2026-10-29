//! The view's drawing rules M8-33 and M8-34: what the view dims, the badge in
//! the title, the rule between two blocks, the colour an ambiguous base has, and
//! what `C` and `NO_COLOR` leave uncoloured (contract 8m9 §4).
//!
//! The columns come from the records, not from what the code draws. `pbr` is
//! `tests/fixtures/genbank/raw/J01749.1.gb`, which its own LOCUS line and ORIGIN
//! say is 4361 bp, circular (`d = 4`, four digits in its gutter), so at 80 x 24
//! its lines hold 66 bases: base `b` of a block starting at `s` stands at screen
//! column `6 + b - s`, and the bases run from column 6 to 71. `amb` is
//! `tests/fixtures/view/frames-amb.fa`: 33 bp, linear as an assumption, `d = 2`,
//! one line, so base `b` stands at column `4 + b`. Its ambiguous bases are the
//! letters the file holds, zero-based: `R` at 14, `N` at 17, 20 and 23 to 26, `S`
//! at 31 and `K` at 32 - columns 18, 21, 24, 27 to 30, 35 and 36.
//!
//! The rows are found by what they hold, never by a fixed number: the top row of
//! a block is the row whose gutter holds that block's first base number, and a
//! block's bottom row is the row under it, where `layout` puts its complements
//! (D055 order). Every block drawn here has its bases on one line.

use luria_cli::sequence_view::screen::draw;
use luria_cli::sequence_view::state::Action;
use luria_cli::sequence_view::state::Key;
use luria_cli::sequence_view::state::ViewState;
use luria_core::identity::Identifier;
use luria_core::view_record::open_record;
use luria_core::view_record::ViewRecord;
use ratatui::backend::TestBackend;
use ratatui::buffer::Cell;
use ratatui::style::{Color, Modifier};
use ratatui::Terminal;
use std::fs;
use std::path::Path;

/// The same size every other sequence-view test draws at: 80 columns, 24 rows.
const W: u16 = 80;
const H: u16 = 24;

/// The columns a `pbr` line's bases fill, first and last.
const FIRST: usize = 6;
const LAST: usize = 71;

/// The colour `layout` gives a base that is not one of the four DNA letters.
const AMBIGUOUS: Color = Color::Indexed(208);

fn fixture(path: &str) -> String {
    let file: std::path::PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"))
}

fn record(name: &str, path: &str) -> ViewRecord {
    open_record(Identifier::create(name.to_string()).unwrap(), fixture(path))
        .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
}

/// pBR322: 4361 bp and circular, as its LOCUS line states both.
fn pbr() -> ViewRecord {
    record("SYNPBR322", "../../tests/fixtures/genbank/raw/J01749.1.gb")
}

/// `framesamb`: 33 bp, linear as an assumption, and the one record here whose
/// bases hold IUPAC ambiguity codes.
fn amb() -> ViewRecord {
    record("framesamb", "../../tests/fixtures/view/frames-amb.fa")
}

/// The cells of the drawn screen, row by row - the `sequence_view_strands.rs`
/// reading of the test backend.
fn drawn(state: &ViewState) -> Vec<Vec<Cell>> {
    let mut terminal = Terminal::new(TestBackend::new(W, H)).unwrap();
    terminal.draw(|frame| draw(frame, state)).unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = usize::from(buffer.area.width);
    buffer
        .content
        .chunks(width)
        .map(|row: &[Cell]| row.to_vec())
        .collect()
}

fn symbol(cells: &[Vec<Cell>], row: usize, column: usize) -> String {
    cells[row][column].symbol().to_string()
}

/// The row's text from `first` to `last`, both columns inclusive.
fn slice(cells: &[Vec<Cell>], row: usize, first: usize, last: usize) -> String {
    cells[row][first..=last].iter().map(Cell::symbol).collect()
}

/// The whole row as text, the frame's border included.
fn line(cells: &[Vec<Cell>], row: usize) -> String {
    cells[row].iter().map(Cell::symbol).collect()
}

fn dim(cells: &[Vec<Cell>], row: usize, column: usize) -> bool {
    cells[row][column].modifier.contains(Modifier::DIM)
}

fn fg(cells: &[Vec<Cell>], row: usize, column: usize) -> Color {
    cells[row][column].fg
}

fn press(state: &mut ViewState, key: Key) {
    assert_eq!(
        state.handle(key),
        Action::Continue,
        "pressing {key:?} leaves"
    );
}

fn type_into(state: &mut ViewState, text: &str) {
    for ch in text.chars() {
        press(state, Key::Char(ch));
    }
}

/// The row whose gutter - the frame's inner columns 1 to `gutter_end` - spells
/// `label`: the block that starts with that base number, or the frame line whose
/// label it is. The overview strip's bars line holds box-drawing characters and
/// its marks line is blank, so neither spells a label.
fn gutter_row(cells: &[Vec<Cell>], label: &str, gutter_end: usize) -> usize {
    let found: Vec<usize> = cells
        .iter()
        .enumerate()
        .filter(|(_row, cells)| {
            let text: String = cells[1..=gutter_end].iter().map(Cell::symbol).collect();
            text.trim() == label
        })
        .map(|(row, _cells)| row)
        .collect();
    assert_eq!(
        found.len(),
        1,
        "one row of the drawn view has {label} alone in its gutter, not {} rows",
        found.len()
    );
    found[0]
}

/// pbr's block that starts at base `first`: its top row, and the row of its
/// complements under it. That block's gutter ends at screen column 4, the width
/// `d = 4` takes.
fn pbr_block(cells: &[Vec<Cell>], first: u64) -> (usize, usize) {
    let top: usize = gutter_row(cells, &format!("{}", first + 1), 4);
    (top, top + 1)
}

/// Rule 1: every cell of a block's bottom row is dim, no base cell of its top row
/// is, and the gutter's number and the number after the last base are dim too.
/// After `t`, a frame's own label is dim.
#[test]
fn the_secondary_text_is_dim() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    let cells: Vec<Vec<Cell>> = drawn(&state);
    let (top, bottom): (usize, usize) = pbr_block(&cells, 0);
    assert_eq!(
        slice(&cells, top, FIRST, FIRST + 3),
        "TTCT",
        "the row the gutter names is the block's top row, and its first four bases \
         are the ORIGIN's first four"
    );

    for column in FIRST..=LAST {
        assert!(
            dim(&cells, bottom, column),
            "the bottom strand at column {column} is dim"
        );
    }
    for column in FIRST..=LAST {
        assert!(
            !dim(&cells, top, column),
            "the top strand at column {column} is not dim"
        );
    }

    assert!(dim(&cells, top, 4), "the gutter's 1 is dim");
    assert_eq!(
        slice(&cells, top, 73, 74),
        "66",
        "the number after the last base stands at columns 73 and 74"
    );
    assert!(dim(&cells, top, 73), "its first cell is dim");
    assert!(dim(&cells, top, 74), "and its last cell is dim");
    assert_eq!(
        symbol(&cells, top, 72),
        " ",
        "the column between the last base and the number is blank"
    );

    press(&mut state, Key::Char('t'));
    let cells: Vec<Vec<Cell>> = drawn(&state);
    let frame: usize = gutter_row(&cells, "+1", 4);
    assert!(dim(&cells, frame, 3), "the label +1 starts here and is dim");
    assert!(dim(&cells, frame, 4), "and ends here, dim");
}

/// Rule 2: the topology's badge in the title, and the dim border that holds it.
#[test]
fn the_title_has_a_topology_badge() {
    let cells: Vec<Vec<Cell>> = drawn(&ViewState::new(pbr(), W, H));
    let title: String = line(&cells, 0);
    assert!(
        title.contains("⟳ circular"),
        "a stated circular record shows its badge: {title:?}"
    );
    assert!(
        title.contains("4361 bp"),
        "and keeps the length its LOCUS states: {title:?}"
    );

    let amb_cells: Vec<Vec<Cell>> = drawn(&ViewState::new(amb(), W, H));
    let amb_title: String = line(&amb_cells, 0);
    assert!(
        amb_title.contains("— linear"),
        "an assumed linear record shows its badge: {amb_title:?}"
    );
    assert!(
        amb_title.contains("(assumed)"),
        "and keeps the word that says the topology was assumed: {amb_title:?}"
    );

    assert!(dim(&cells, 0, 0), "the frame's top-left corner is dim");
    assert!(dim(&amb_cells, 0, 0), "and in the other view too");
}

/// Rule 3: the row between two blocks holds `┄` in the columns a full line's
/// bases stand in, and nowhere else in that row.
#[test]
fn a_dotted_rule_between_blocks() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    type_into(&mut state, "o");
    let cells: Vec<Vec<Cell>> = drawn(&state);
    let rule_rows: Vec<usize> = (0..usize::from(H))
        .filter(|row| (FIRST..=LAST).all(|column| symbol(&cells, *row, column) == "┄"))
        .collect();
    assert_eq!(
        rule_rows.len(),
        1,
        "one row of the view, and no other, holds a dotted rule"
    );
    let rule: usize = rule_rows[0];
    let anywhere: Vec<usize> = (0..usize::from(H) - 2)
        .filter(|row| (0..usize::from(W)).any(|column| symbol(&cells, *row, column) == "┄"))
        .collect();
    assert_eq!(
        anywhere,
        vec![rule],
        "no row of the drawn view holds a `┄` but the rule's own row"
    );

    let above: usize = gutter_row(&cells, "1", 4);
    let below: usize = gutter_row(&cells, "67", 4);
    assert!(
        above < rule && rule < below,
        "the rule's row {rule} lies between the block whose top row is {above} and \
         the block whose top row is {below}"
    );
    let letters: String = cells[rule - 1][FIRST..=FIRST + 3]
        .iter()
        .map(Cell::symbol)
        .collect();
    assert!(
        letters.chars().all(|ch| ch.is_ascii_alphabetic()),
        "the row above the rule holds a strand's letters, so the rule stands after a \
         block's last line: {letters:?}"
    );
    assert_ne!(
        symbol(&cells, rule + 1, FIRST),
        "┄",
        "the row after the rule is the next block's own row, not a second rule"
    );

    for column in FIRST..=LAST {
        assert_eq!(
            symbol(&cells, rule, column),
            "┄",
            "column {column} of the rule's row"
        );
        assert!(dim(&cells, rule, column), "column {column} is dim");
        assert_eq!(
            fg(&cells, rule, column),
            Color::Reset,
            "and the rule is uncoloured at column {column}"
        );
        assert_eq!(
            cells[rule][column].bg,
            Color::Reset,
            "and has no background at column {column}"
        );
    }
    for column in 1..=5 {
        assert_eq!(
            symbol(&cells, rule, column),
            " ",
            "the gutter's column {column} is blank"
        );
    }
    for column in 72..=78 {
        assert_eq!(
            symbol(&cells, rule, column),
            " ",
            "column {column} past the last base is blank"
        );
    }
}

/// Rule 4: an ambiguous base - anything but `A C G T` - in its own colour, on both
/// strands and nowhere else, and after `C` no column of either row has a colour.
#[test]
fn ambiguous_bases_have_their_own_colour() {
    let mut state: ViewState = ViewState::new(amb(), W, H);
    let cells: Vec<Vec<Cell>> = drawn(&state);
    let top: usize = gutter_row(&cells, "1", 2);
    let bottom: usize = top + 1;
    assert_eq!(
        slice(&cells, top, 4, 10),
        "ATGTATT",
        "the line's first base stands at column 4, as §5's comment says, and its \
         first seven letters are the file's"
    );

    for column in [18usize, 21, 24, 27, 28, 29, 30, 35, 36] {
        assert_eq!(
            fg(&cells, top, column),
            AMBIGUOUS,
            "an ambiguous top-strand base at column {column}"
        );
        assert_eq!(
            fg(&cells, bottom, column),
            AMBIGUOUS,
            "and its complement at column {column}"
        );
        assert!(
            dim(&cells, bottom, column),
            "the bottom row stays dim under the colour at column {column}"
        );
    }
    let coloured: Vec<usize> = (4..=36)
        .filter(|column| fg(&cells, top, *column) != Color::Reset)
        .collect();
    assert_eq!(
        coloured,
        vec![18, 21, 24, 27, 28, 29, 30, 35, 36],
        "the top row is coloured in §5's ambiguous columns and in no others"
    );
    let coloured: Vec<usize> = (4..=36)
        .filter(|column| fg(&cells, bottom, *column) != Color::Reset)
        .collect();
    assert_eq!(
        coloured,
        vec![18, 21, 24, 27, 28, 29, 30, 35, 36],
        "the bottom row is coloured in the same columns and no others"
    );

    type_into(&mut state, "C");
    assert!(!state.colours(), "C turns the colours off");
    let cells: Vec<Vec<Cell>> = drawn(&state);
    let top: usize = gutter_row(&cells, "1", 2);
    let bottom: usize = top + 1;
    for column in 4..=36 {
        assert_eq!(
            fg(&cells, top, column),
            Color::Reset,
            "with the colours off the top row is uncoloured at column {column}"
        );
        assert_eq!(
            fg(&cells, bottom, column),
            Color::Reset,
            "and the bottom row is uncoloured at column {column}"
        );
    }
}

/// Rule 5: with the colours off, nothing in the view is coloured and a hit is
/// marked bold and underlined instead of coloured.
///
/// The file's `AATT` recurs at zero-based 58, 251, 1319 and further on: the
/// search takes the first hit at or after the cursor, and the key sequence leaves
/// the view showing the block whose gutter holds 197, where the hit at zero-based
/// 251 stands in columns 61 to 64 of both strand rows. `AATT` is its own
/// complement, so its hit marks the row of each strand (M8-24).
#[test]
fn colours_off_means_no_colour() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    type_into(&mut state, "c");
    press(&mut state, Key::Home);
    press(&mut state, Key::Enter);
    type_into(&mut state, "/AATT");
    press(&mut state, Key::Enter);
    assert!(state.hits().is_some(), "the search found its hits");
    type_into(&mut state, "C");
    assert!(!state.colours(), "C turns the colours off");

    let cells: Vec<Vec<Cell>> = drawn(&state);

    // Rule 5 first, and everywhere: no drawn cell has an `fg` or a `bg`.
    for (row, cells) in cells.iter().enumerate() {
        for (column, cell) in cells.iter().enumerate() {
            assert_eq!(
                cell.fg,
                Color::Reset,
                "row {row} column {column} has {:?} in front of {}",
                cell.fg,
                cell.symbol()
            );
            assert_eq!(
                cell.bg,
                Color::Reset,
                "row {row} column {column} has {:?} behind {}",
                cell.bg,
                cell.symbol()
            );
        }
    }

    // The columns just outside the hit: the hit's own marks reach neither.
    let top: usize = gutter_row(&cells, "197", 4);
    let bottom: usize = top + 1;
    for column in [61usize, 62, 63, 64] {
        for row in [top, bottom] {
            let cell: &Cell = &cells[row][column];
            assert!(
                cell.modifier.contains(Modifier::BOLD),
                "row {row} column {column} holds {} and is bold",
                cell.symbol()
            );
            assert!(
                cell.modifier.contains(Modifier::UNDERLINED),
                "row {row} column {column} holds {} and is underlined",
                cell.symbol()
            );
        }
    }
    // The top row checks `BOLD` only: the cursor's codon underline marks columns 60-62 there.
    for column in [60usize, 65] {
        let cell: &Cell = &cells[top][column];
        assert!(
            !cell.modifier.contains(Modifier::BOLD),
            "row {top} column {column} holds {} outside the hit and is not bold: {:?}",
            cell.symbol(),
            cell.modifier
        );
        let cell: &Cell = &cells[bottom][column];
        assert!(
            !cell.modifier.contains(Modifier::BOLD)
                && !cell.modifier.contains(Modifier::UNDERLINED),
            "row {bottom} column {column} holds {} outside the hit and carries neither \
             mark: {:?}",
            cell.symbol(),
            cell.modifier
        );
    }
}

/// Rule 5, from the open: `NO_COLOR` leaves the colours off, and what the feature
/// list and the key list draw is the same as ever - without a single colour.
#[test]
fn no_color_at_open_reaches_every_colour() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    state.set_colours(false);
    assert!(!state.colours(), "the view opens with the colours off");
    type_into(&mut state, "f");
    assert!(state.list().is_some(), "f opens the feature list");
    let cells: Vec<Vec<Cell>> = drawn(&state);
    for (row, cells) in cells.iter().enumerate() {
        for (column, cell) in cells.iter().enumerate() {
            assert_eq!(
                cell.fg,
                Color::Reset,
                "the feature list's row {row} column {column} has {:?} in front of {}",
                cell.fg,
                cell.symbol()
            );
            assert_eq!(
                cell.bg,
                Color::Reset,
                "the feature list's row {row} column {column} has {:?} behind {}",
                cell.bg,
                cell.symbol()
            );
        }
    }

    let mut state: ViewState = ViewState::new(pbr(), W, H);
    state.set_colours(false);
    type_into(&mut state, "?");
    assert!(state.help(), "the key list is open");
    let cells: Vec<Vec<Cell>> = drawn(&state);
    for (row, cells) in cells.iter().enumerate() {
        for (column, cell) in cells.iter().enumerate() {
            assert_eq!(
                cell.fg,
                Color::Reset,
                "the key list's row {row} column {column} has {:?} in front of {}",
                cell.fg,
                cell.symbol()
            );
            assert_eq!(
                cell.bg,
                Color::Reset,
                "the key list's row {row} column {column} has {:?} behind {}",
                cell.bg,
                cell.symbol()
            );
        }
    }
}
