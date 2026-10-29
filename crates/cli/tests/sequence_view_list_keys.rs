//! The feature list's rows are drawn in their feature's colour, and the list's
//! filter matches a feature's key as well as its names (contract 8m16 §6, M8-39;
//! 8m7's `feature_colour` and D056's one matcher).
//!
//! `pbr` is `tests/fixtures/genbank/raw/J01749.1.gb`: 4361 bp, 50 features in
//! file order, and the file carries no `ApEinfo_fwdcolor` or
//! `ApEinfo_revcolor`, so every row's colour is its key's — contract §5's hand
//! values: `CDS` 214, `gene` 179, `misc_binding` 168, and `source` has none. The
//! features this test names are 0 `source 1..4361`, 2 `misc_binding 24..27`
//! (named `misc_binding`, since it has no name qualifier), 9 the `gene` 86..1276
//! `/gene="tet"`, 10 the `CDS` 86..1276 `tet`, and the three `CDS` features are
//! 10, 26, 38 with the two `gene` features 9 and 37, as
//! `tests/fixtures/genbank/pbr322-features.tsv` numbers them.
//!
//! The colour expectations are §5's key colours off 8m7 §4 rule 2's table and
//! not what the code answers; the filtered rows are the keys and the `/gene`
//! values of the fixture read by hand. Go-to is unchanged by this contract: it
//! matches names only, and its status line for no match is what 8k2 words today.

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

/// Every test draws at this size: 80 columns and 24 rows, which leaves the body
/// 20 rows (`height - 4`) for the list's rows, starting at screen row 1
/// (`tests/sequence_view_list.rs`'s size).
const W: u16 = 80;
const H: u16 = 24;

/// The screen row the list's first row is drawn on: the frame's first inner row
/// (8k3 §4 rule 7).
const LIST_ROW: usize = 1;

/// The number of rows the body holds at this size, and so the number of list
/// rows drawn.
const BODY: usize = 20;

/// The body's rows, and its inner columns: the frame's border takes x 0 and x 79.
const ROWS: std::ops::Range<usize> = LIST_ROW..LIST_ROW + BODY;
const COLUMNS: (usize, usize) = (1, 78);

/// 8m7 §4 rule 2's key colours, and 8m7 §5's note that pBR322 has no ApE
/// colours, so the key decides (contract 8m16 §5).
const CDS_COLOUR: Color = Color::Indexed(214);
const GENE_COLOUR: Color = Color::Indexed(179);
const BINDING_COLOUR: Color = Color::Indexed(168);

fn pbr() -> ViewRecord {
    let path: &str = "../../tests/fixtures/genbank/raw/J01749.1.gb";
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text: String =
        fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"));
    open_record(Identifier::create("SYNPBR322".to_string()).unwrap(), text)
        .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
}

/// `text` typed into an open list, one key per character (8k3 §4 rule 3).
fn type_into(state: &mut ViewState, text: &str) {
    for ch in text.chars() {
        assert_eq!(
            state.handle(Key::Char(ch)),
            Action::Continue,
            "typing {ch:?} does not leave"
        );
    }
}

/// `key` pressed `times` times.
fn walk(state: &mut ViewState, key: Key, times: usize) {
    for _ in 0..times {
        assert_eq!(
            state.handle(key),
            Action::Continue,
            "{key:?} does not leave"
        );
    }
}

/// The cells of the drawn screen, row by row, with their colours — the way
/// `tests/sequence_view_colours.rs` reads the test backend.
fn drawn_cells(state: &ViewState) -> Vec<Vec<Cell>> {
    let mut terminal = Terminal::new(TestBackend::new(W, H)).unwrap();
    terminal.draw(|frame| draw(frame, state)).unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = usize::from(buffer.area.width);
    buffer
        .content
        .chunks(width)
        .map(|cells: &[Cell]| cells.to_vec())
        .collect()
}

/// The foreground of one cell.
fn fg(cells: &[Cell], column: usize) -> Color {
    cells[column].fg
}

/// One drawn row's text, from the frame's first inner column to its last
/// non-space cell — the padding trimmed as `tests/sequence_view_colours.rs`'s
/// `row_text` trims it.
fn row_text(cells: &[Cell]) -> String {
    let line: String = cells[COLUMNS.0..=COLUMNS.1]
        .iter()
        .map(Cell::symbol)
        .collect();
    line.trim().to_string()
}

/// The column of a drawn row's last text cell: the row's text runs from the
/// body's first column (x 1) to its last non-space cell inside the frame, and a
/// row longer than the body is clipped there (`draw_list` breaks at
/// `body.right()`). The cell at x `n` holds the `n`th character.
fn last_text_column(cells: &[Cell]) -> usize {
    let line: String = cells[COLUMNS.0..=COLUMNS.1]
        .iter()
        .map(Cell::symbol)
        .collect();
    let count: usize = line.trim_end().chars().count();
    assert!(count > 0, "the row has text to colour: {line:?}");
    // The text's first cell is x 1, so its `count`th is x `count`.
    count
}

/// The body row the feature `item` is drawn on while the filter is empty: the
/// rows come from the body's first row down in file order (8k3 §4 rule 7).
fn body_row(item: usize) -> usize {
    LIST_ROW + item
}

/// §6 test 1 (8m16 §4 rule 1): every feature row carries its lane colour as its
/// foreground, from its first text cell to its last, and a feature with no colour
/// of its own keeps the screen's.
#[test]
fn rows_have_their_lane_colour() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert!(state.colours(), "the view opens with its colours on");
    assert_eq!(
        state.handle(Key::Char('f')),
        Action::Continue,
        "f opens the feature list"
    );

    let cells: Vec<Vec<Cell>> = drawn_cells(&state);

    // Feature 10, the tet CDS: the `CDS` colour down the row.
    let row: usize = body_row(10);
    assert_eq!(row_text(&cells[row]), "tet CDS 86..1276 (+)");
    assert_eq!(
        fg(&cells[row], COLUMNS.0),
        CDS_COLOUR,
        "the tet CDS row starts in the CDS colour"
    );
    assert_eq!(
        fg(&cells[row], last_text_column(&cells[row])),
        CDS_COLOUR,
        "and ends in it"
    );

    // Feature 9, the tet gene: the `gene` colour.
    let row: usize = body_row(9);
    assert_eq!(row_text(&cells[row]), "tet gene 86..1276 (+)");
    assert_eq!(fg(&cells[row], COLUMNS.0), GENE_COLOUR, "the tet gene row");
    assert_eq!(
        fg(&cells[row], last_text_column(&cells[row])),
        GENE_COLOUR,
        "to the end of the tet gene row"
    );

    // Feature 2, `misc_binding 24..27`: the `misc_binding` colour, which is also
    // the colour of the two `regulatory` features either side of it.
    let row: usize = body_row(2);
    assert_eq!(
        row_text(&cells[row]),
        "misc_binding misc_binding 24..27 (+)",
        "a feature with no name qualifier is named by its key"
    );
    assert_eq!(
        fg(&cells[row], COLUMNS.0),
        BINDING_COLOUR,
        "the misc_binding row"
    );
    assert_eq!(
        fg(&cells[row], last_text_column(&cells[row])),
        BINDING_COLOUR,
        "to the end of the misc_binding row"
    );

    // Feature 0, `source`: a key with no colour, so the screen's own.
    let row: usize = body_row(0);
    assert_eq!(row_text(&cells[row]), "source source 1..4361 (+)");
    assert_eq!(
        fg(&cells[row], COLUMNS.0),
        Color::Reset,
        "the source row has no colour"
    );
    assert_eq!(
        fg(&cells[row], last_text_column(&cells[row])),
        Color::Reset,
        "and keeps it to the end of the row"
    );
}

/// §6 test 2 (8m16 §4 rule 2): with the colours off no list row has a colour, and
/// the CDS picker never has one, whatever the colours do.
#[test]
fn colours_off_rows_have_none() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert_eq!(
        state.handle(Key::Char('C')),
        Action::Continue,
        "`C` away from a list turns the colours off"
    );
    assert!(!state.colours(), "`C` turns the colours off");
    assert_eq!(
        state.handle(Key::Char('f')),
        Action::Continue,
        "f opens the feature list"
    );

    let cells: Vec<Vec<Cell>> = drawn_cells(&state);
    // The same rows test 1 colours, and no cell of the body carries a colour.
    assert_eq!(row_text(&cells[body_row(10)]), "tet CDS 86..1276 (+)");
    assert_eq!(
        row_text(&cells[body_row(2)]),
        "misc_binding misc_binding 24..27 (+)"
    );
    for row in ROWS {
        for column in COLUMNS.0..=COLUMNS.1 {
            assert_eq!(
                fg(&cells[row], column),
                Color::Reset,
                "row {row} column {column} has no colour with the colours off: {}",
                row_text(&cells[row])
            );
        }
    }

    // The picker, with its colours on: its rows name CDSs, and 8m16 §4 rule 1
    // leaves them uncoloured.
    let mut picker: ViewState = ViewState::new(pbr(), W, H);
    assert!(picker.colours(), "the picker opens with its colours on");
    assert_eq!(
        picker.handle(Key::Char('c')),
        Action::Continue,
        "c opens the CDS picker"
    );
    let cells: Vec<Vec<Cell>> = drawn_cells(&picker);
    assert!(
        row_text(&cells[LIST_ROW]).contains("tet"),
        "the picker's first row is the tet CDS: {}",
        row_text(&cells[LIST_ROW])
    );
    for row in ROWS {
        for column in COLUMNS.0..=COLUMNS.1 {
            assert_eq!(
                fg(&cells[row], column),
                Color::Reset,
                "row {row} column {column} of the picker has no colour",
            );
        }
    }
}

/// §6 test 3 (8m16 §4 rule 3): the feature list's filter reaches the key, with
/// the same matcher it uses for names, and a feature named by both its name and
/// its key still stands once.
#[test]
fn the_filter_matches_the_key() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert_eq!(
        state.handle(Key::Char('f')),
        Action::Continue,
        "f opens the feature list"
    );

    // `CDS` names three features, and nothing in the file is named by a word
    // starting `cds` (contract §5), so the rows are exactly those three features:
    // 10 `tet`, 26 the ROP CDS, 38 `bla` (pbr322-features.tsv).
    type_into(&mut state, "CDS");
    assert_eq!(
        state.list().expect("open").rows(),
        [10, 26, 38],
        "the three CDS features, each once, in file order"
    );
    let cells: Vec<Vec<Cell>> = drawn_cells(&state);
    assert_eq!(
        row_text(&cells[LIST_ROW]),
        "tet CDS 86..1276 (+)",
        "the first CDS feature's row"
    );
    assert_eq!(
        row_text(&cells[LIST_ROW + 1]),
        "ROP protein CDS 1915..2106 (+)",
        "the ROP CDS feature's row names its feature"
    );
    assert_eq!(
        row_text(&cells[LIST_ROW + 2]),
        "bla CDS 3293..4153 (-)",
        "the bla CDS feature's row is last"
    );
    assert_eq!(
        row_text(&cells[LIST_ROW + 3]),
        "",
        "and there are no rows past the three"
    );
    walk(&mut state, Key::Backspace, 3);

    // `gene` names the two `gene` features — 9 and 37 — and is not a name of
    // anything in the file (contract §5).
    type_into(&mut state, "gene");
    assert_eq!(
        state.list().expect("open").rows(),
        [9, 37],
        "the two gene features"
    );
    walk(&mut state, Key::Backspace, 4);

    // `tet` answers the features named by `/gene="tet"`, each once: 9 the gene,
    // 10 the CDS, and 12..16 — `misc_feature 146..147`, `misc_binding 411..414`,
    // `misc_difference 426`, `misc_binding 469..472`, `old_sequence 526..528` —
    // the same rows as before this contract.
    type_into(&mut state, "tet");
    assert_eq!(
        state.list().expect("open").rows(),
        [9, 10, 12, 13, 14, 15, 16],
        "every feature named tet, and only once"
    );
    walk(&mut state, Key::Backspace, 3);

    // `bind` reaches every `misc_binding` key — the 15 rows of
    // pbr322-features.tsv — through the matcher's word start, a prefix no name
    // reaches (8m16 §1).
    type_into(&mut state, "bind");
    assert_eq!(
        state.list().expect("open").rows(),
        [2, 4, 6, 7, 8, 13, 15, 30, 43, 44, 45, 46, 47, 48, 49],
        "every misc_binding feature"
    );
}

/// §6 test 4: go-to is untouched — it matches names, and the keys are not names.
#[test]
fn go_to_still_matches_names_only() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert_eq!(
        state.handle(Key::Char('g')),
        Action::Continue,
        "g opens the prompt"
    );
    type_into(&mut state, "gene");
    assert_eq!(
        state.handle(Key::Enter),
        Action::Continue,
        "enter acts on the line"
    );

    assert!(
        state.list().is_none(),
        "no list opens for `gene`, which names no feature"
    );
    assert_eq!(state.cursor(), 0, "and nothing is jumped to");
    let words: &str = state.message().expect("the refusal is said");
    assert!(
        words.contains("no feature is named gene"),
        "the status line says what go-to says for a name with no match: {words}"
    );
}
