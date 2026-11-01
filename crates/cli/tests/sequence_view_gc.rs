//! The overview strip's third line: the GC content of the bases every column
//! stands for, as one of eight block heights (contract 8m12 §6; D054, D056, D051).
//!
//! `pbr` is `tests/fixtures/genbank/raw/J01749.1.gb` — 4361 bp, circular by its own
//! declaration, and no CDS selected at open — and `amb` is
//! `tests/fixtures/view/frames-amb.fa`, the 33 bp
//! `ATGTATTAATAGTARAANGGNCTNNNNGTGTSK`. At 80 columns the frame's inner width is
//! 78, which is what `gc_line` is asked for: `pbr`'s strip is 71 cells wide, 78
//! minus the `1 ` and the ` 4361` (D054), and `amb`'s is 33 cells of one base each,
//! since a strip is never wider than the record has bases (D054).
//!
//! Every height here is the planner's, written out in §5 of
//! `docs/tasks/contracts/milestone-8m12-gc-sparkline.md`. The worker re-derived them from the
//! two files before writing these tests, with no help from the implementation: the
//! column ranges from D054's floor formula, `⌊i·L/w⌋` to `⌊(i+1)·L/w⌋`, the counts
//! from the files' own letters with `G`, `C`, `S` as GC and `A`, `T`, `W` as AT, and
//! the glyph from `⌊8·GC/(GC+AT)⌋` on the absolute 0–100 % scale of 8m12 §4 rule 2.
//! The four columns §5 names stand checked on top: `pbr`'s columns 0, 1, 35 and 70
//! hold 20/61, 31/61, 45/62 and 22/62 GC over bases 0-60, 61-121, 2149-2210 and
//! 4299-4360, and `amb`'s uncounted letters come out as spaces — `R` at 14, `N` at
//! 17, 20 and 23 to 26 and `K` at 32.
//!
//! The screen is read back off a `ratatui` test backend the way
//! `tests/sequence_view_overview.rs` reads it: the strip's three lines are rows 1,
//! 2 and 3 between the borders at x 1 to 78, and the blocks have what is left under
//! row 3 (8l2 §4 rule 2, 8m12 §4 rule 4).

use luria_cli::sequence_view::overview::gc_line;
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
use ratatui::style::Modifier;
use ratatui::Terminal;
use std::fs;
use std::path::Path;

/// Every test draws at this size, as `tests/sequence_view_overview.rs` does: 80
/// columns and 24 rows, so the frame's inner area is 78 columns wide, its first
/// three rows are the strip while it is drawn, and the blocks have 17 rows — 20
/// with the strip hidden (8l2 §4 rule 2, 8m12 §4 rule 4).
const W: u16 = 80;
const H: u16 = 24;

/// The frame's inner width, which is what the strip and the layout are built for:
/// the border takes one column at each side (8k1 §4 rule 1).
const INSIDE: u16 = W - 2;

/// The screen row the strip's third line is drawn on, and the first and last of
/// its cells — the columns line 1's strip cells and line 2's marks stand on, with
/// `1 ` at x 1 and 2 and ` 4361` at 74 to 78 (8l2 §4 rule 3, 8m12 §4 rule 3).
const THIRD_ROW: usize = 3;
const CELLS: (usize, usize) = (3, 73);

/// The strip's own columns are `pbr`'s 71 heights at this width, and `amb`'s 33:
/// §5's lines, with `▃ ▅ ▆ ▃` at columns 0, 1, 35 and 70 of `pbr`'s and all of
/// `amb`'s between its own marks. The spaces in `amb`'s line are its uncounted
/// letters, and `pbr`'s line has none, because every one of its columns holds both
/// kinds of base.
const PBR: &str = concat!(
    "▃▅▆▅▅▅▆▆▆▅▅▆▅▅▆▆▅▅▅▅▆▅▅▆▅▅▅▅▅▄",
    "▄▅▅▅▅▆▅▄▅▅▄▅▅▆▅▆▅▅▅▅▄▄▂▃▄▅▅▄▄",
    "▅▅▄▄▅▄▄▄▃▃▄▃",
);
const AMB: &str = concat!("▁▁█▁▁▁▁▁▁▁▁█▁▁ ▁▁", " ██ █▁    █▁█▁█ ",);

/// One fixture record, named as the CLI names it: the content decides the format,
/// never the name (D049), and `open_record` is the entry point the view and
/// `luria find` share (D048).
fn record(path: &str, name: &str) -> ViewRecord {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text: String =
        fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"));
    open_record(Identifier::create(name.to_string()).unwrap(), text)
        .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
}

/// pBR322: 4361 bp, circular, and no CDS selected on open.
fn pbr() -> ViewRecord {
    record("../../tests/fixtures/genbank/raw/J01749.1.gb", "SYNPBR322")
}

/// The 33 bp FASTA of the view fixtures, whose IUPAC letters leave columns with
/// nothing counted.
fn amb() -> ViewRecord {
    record("../../tests/fixtures/view/frames-amb.fa", "framesamb")
}

/// `key` pressed once, with a failure saying it leaves.
fn press(state: &mut ViewState, key: Key) {
    assert_eq!(
        state.handle(key),
        Action::Continue,
        "{key:?} does not leave"
    );
}

/// The screen drawn on a test backend: every row as the cells of its columns.
fn screen(state: &ViewState) -> Vec<Vec<char>> {
    let mut terminal = Terminal::new(TestBackend::new(W, H)).unwrap();
    terminal.draw(|frame| draw(frame, state)).unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = usize::from(buffer.area.width);
    buffer
        .content()
        .chunks(width)
        .map(|cells| cells.iter().map(|cell| one(cell.symbol())).collect())
        .collect()
}

/// The cell's symbol as the one character it is: every glyph this screen draws —
/// the frame's border, the strip's bars and blocks, the bases and the ruler —
/// stands in one column, so a cell's index is the column it sits at. A symbol
/// wider than that would shift the columns, and is a failure of the screen rather
/// than something the tests should work around.
fn one(symbol: &str) -> char {
    symbol.chars().next().expect("a cell holding one character")
}

/// The screen's cells, row by row, as the backend holds them.
fn drawn(state: &ViewState) -> Vec<Vec<Cell>> {
    let mut terminal = Terminal::new(TestBackend::new(W, H)).unwrap();
    terminal.draw(|frame| draw(frame, state)).unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = usize::from(buffer.area.width);
    buffer
        .content()
        .chunks(width)
        .map(|row| row.to_vec())
        .collect()
}

/// One row of the screen as its cells. Every glyph the strip draws is one cell
/// wide, so the index of a cell is the screen column it stands at.
fn row(state: &ViewState, which: usize) -> Vec<char> {
    screen(state)[which].clone()
}

/// One stretch of a row, `first` to `last` at both ends, as a string.
fn span(row: &[char], first: usize, last: usize) -> String {
    row[first..=last].iter().collect()
}

/// The frame's inner columns of a row that hold anything, as `(x, glyph)`: what a
/// line that marks, rather than spells, holds (8l2 §4 rule 5).
fn marks(row: &[char]) -> Vec<(usize, char)> {
    (1..79)
        .filter_map(|column| match row[column] {
            ' ' => None,
            glyph => Some((column, glyph)),
        })
        .collect()
}

/// The frame's inner columns of `row` that carry anything: an `fg`, a `bg` or a
/// modifier. The frame's border is dim, and the strip is not the border, so its
/// two columns are not asked about (8m12 §4 rule 3).
fn styled(state: &ViewState, which: usize) -> Vec<usize> {
    drawn(state)[which]
        .iter()
        .enumerate()
        .filter(|(column, cell)| {
            (1..=usize::from(INSIDE)).contains(column)
                && (cell.fg != Color::Reset
                    || cell.bg != Color::Reset
                    || cell.modifier != Modifier::empty())
        })
        .map(|(column, _cell)| column)
        .collect()
}

/// Whether a row holds a base: the block's base columns, which begin at x 6 after
/// the gutter's four and the blank beside them, hold ten sequence letters in a
/// run. A bar's name, a ruler tick, a lane's label and the strip's own glyphs never
/// do, so the rows the strip draws are told from the rows the blocks draw.
fn holds_base(row: &[char]) -> bool {
    let mut run: usize = 0;
    for letter in row.iter().skip(6) {
        match letter {
            'A' | 'C' | 'G' | 'T' => {
                run += 1;
                if run >= 10 {
                    return true;
                }
            }
            _ => run = 0,
        }
    }
    false
}

/// §6.1: `pbr`'s GC line at the frame's inner width is §5's 71 heights, and the
/// four columns §5 names stand at their own heights.
#[test]
fn the_gc_line_by_hand() {
    let line: String = gc_line(&pbr(), INSIDE);
    assert_eq!(line.chars().count(), 71, "one cell per strip column");
    let cells: Vec<char> = line.chars().collect();
    assert_eq!(
        cells,
        PBR.chars().collect::<Vec<char>>(),
        "the line §5 counts out, column by column"
    );
    assert_eq!(
        (cells[0], cells[1], cells[35], cells[70]),
        ('▃', '▅', '▆', '▃'),
        "20/61, 31/61, 45/62 and 22/62 GC over bases 0-60, 61-121, 2149-2210 and 4299-4360"
    );
}

/// §6.2: `amb`'s line is §5's 33 heights, one base per column (D054), and its
/// uncounted letters are spaces: `R` at 14, `N` at 17, 20 and 23 to 26 and `K` at
/// 32 hold nothing counted, while `S` at 31 is `█`.
#[test]
fn ambiguous_bases_are_left_out() {
    let line: String = gc_line(&amb(), INSIDE);
    assert_eq!(line.chars().count(), 33, "one cell per base");
    let cells: Vec<char> = line.chars().collect();
    assert_eq!(
        cells,
        AMB.chars().collect::<Vec<char>>(),
        "the line §5 counts out, column by column"
    );
    for column in [14, 17, 20, 23, 24, 25, 26, 32] {
        assert_eq!(
            cells[column], ' ',
            "R, N and K leave column {column} with nothing counted"
        );
    }
    assert_eq!(cells[31], '█', "S counts as GC on its own");
    assert_eq!(cells[2], '█', "the G at base 2 is 100 % of one base");
    assert_eq!(cells[3], '▁', "and the T beside it is 0 %");
}

/// §6.3: the strip draws its line third, under its two, inside the frame's
/// borders. Row 1 is still `1 `, the strip and ` 4361`, row 2 still marks the
/// screen and the cursor, and row 3 holds §5's line at x 3 to 73 with the two blank
/// cells under `1 ` and ` GC` where the length's line has it. Nothing on the third
/// line carries a colour or a modifier, with the view's colours on and after `C`
/// turns them off (8m12 §4 rule 3).
#[test]
fn the_strip_draws_it_third() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert!(state.overview(), "4361 bp needs the strip");
    let first: Vec<char> = row(&state, 1);
    assert!(
        first.starts_with(&['│', '1', ' '][..]),
        "row 1 still starts with `1 `: {}",
        first.iter().collect::<String>()
    );
    assert!(
        first.ends_with(&['4', '3', '6', '1', '│'][..]),
        "row 1 still ends with ` 4361`"
    );
    assert_eq!(
        marks(&row(&state, 2)),
        vec![(3, '▲'), (4, '▀'), (5, '▀')],
        "row 2 still marks the screen and the cursor, x 3 to 5"
    );

    let third: Vec<char> = row(&state, THIRD_ROW);
    assert_eq!(
        span(&third, 1, 2),
        "  ",
        "the two cells under `1 ` are blank"
    );
    assert_eq!(
        span(&third, CELLS.0, CELLS.1),
        PBR,
        "the strip's 71 cells, under line 1's cells"
    );
    assert_eq!(
        span(&third, 74, 76),
        " GC",
        "a blank, then the label's two letters"
    );
    assert_eq!(third[0], '│', "the border is still the row's first cell");

    assert_eq!(
        styled(&state, THIRD_ROW),
        Vec::<usize>::new(),
        "no cell of row 3 carries a colour or a modifier, with colours on"
    );
    press(&mut state, Key::Char('C'));
    assert!(
        styled(&state, THIRD_ROW).is_empty(),
        "and none after `C` leaves the colours off"
    );
}

/// §6.4: the strip takes three rows, so block 0 begins under its third row, and
/// the row that numbers the block's first base in the gutter stands no higher.
/// After `o` the strip is gone, and its third line with it: no cell of the frame
/// holds one of the eight heights.
#[test]
fn the_strip_takes_three_rows() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert!(state.overview(), "4361 bp needs the strip");
    let rows: Vec<Vec<char>> = screen(&state);
    let top: usize = (0..rows.len())
        .find(|which| {
            let line: &Vec<char> = &rows[*which];
            span(line, 6, 14) == "TTCTCATGT" && line[4] == '1'
        })
        .expect("the row block 0's top strand stands on");
    assert!(
        top >= 4,
        "block 0 numbers its first base in the gutter at row {top}, under the strip"
    );
    for (which, line) in rows.iter().take(4).enumerate() {
        assert!(
            !holds_base(line),
            "row {which} holds a base: {}",
            line.iter().collect::<String>()
        );
    }

    press(&mut state, Key::Char('o'));
    assert!(!state.overview(), "o hides the strip");
    let spark: Vec<char> = "▁▂▃▄▅▆▇█".chars().collect();
    for (which, line) in screen(&state).iter().enumerate() {
        assert!(
            !line.iter().any(|glyph| spark.contains(glyph)),
            "row {which} still holds the strip's third line: {}",
            line.iter().collect::<String>()
        );
    }
}
