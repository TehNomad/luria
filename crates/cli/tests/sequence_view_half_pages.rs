//! `ctrl-d` and `ctrl-u` move the cursor half a screen of blocks down and up,
//! half a page of rows in an open list, and `esc` clears the hits the screen
//! marks when no list, prompt or key list is open (contract 8m15 §4 rules 1 to
//! 5 and §6; M8-37, M8-38).
//!
//! `pbr` is `tests/fixtures/genbank/raw/J01749.1.gb`, named `SYNPBR322` as the
//! view names it: 4361 bp, circular by its own `CIRCULUS` line, and at the size
//! the two half-screen tests draw at it lays out in 67 blocks of 66 bases,
//! whose last block holds zero-based 4356..4361 and is 5 bases long. That is
//! what the clamps land on. Every cursor and top value asserted is the planner's
//! hand value of contract §5, and the hit `esc` clears is the fixture's own:
//! `AATT` first at zero-based 58, 8 hits in all, as
//! `tests/fixtures/view/search-dna-hits.tsv` records it.
//!
//! The keys are driven as the terminal drives them — one `Key` per press — and
//! the screen is read back off a `ratatui` test backend: the body rows for the
//! key list, the yellow background of a marked base for the hits.

use luria_cli::sequence_view::layout::FeatureLevel;
use luria_cli::sequence_view::screen::draw;
use luria_cli::sequence_view::state::Action;
use luria_cli::sequence_view::state::Key;
use luria_cli::sequence_view::state::ViewState;
use luria_core::identity::Identifier;
use luria_core::view_record::open_record;
use luria_core::view_record::ViewRecord;
use ratatui::backend::TestBackend;
use ratatui::style::Color;
use ratatui::Terminal;
use std::fs;
use std::path::Path;

/// The size the two half-screen tests draw at: 80 columns and 60 rows, whose
/// frame has 78 inner columns, whose body is 56 rows less the strip's three, and
/// which lays `pbr` out in 67 blocks of 66 bases (§5).
const BIG_W: u16 = 80;
const BIG_H: u16 = 60;

/// The size the list, the hits and the key list are read at: 80 columns and 24
/// rows, which leaves the body 20 rows and puts the status line on row 21.
const W: u16 = 80;
const H: u16 = 24;

/// The body rows while a list is drawn, which is with no strip (8k3 §4 rule 1):
/// the frame's inner rows less the status line's one, at 80 x 24.
const BODY_WITHOUT: std::ops::Range<usize> = 1..21;

/// pBR322: 4361 bp, circular by its own declaration, and no CDS selected.
fn pbr() -> ViewRecord {
    let path: &str = "../../tests/fixtures/genbank/raw/J01749.1.gb";
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text: String =
        fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"));
    open_record(Identifier::create("SYNPBR322".to_string()).unwrap(), text)
        .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
}

/// `key` pressed once, with a failure saying it leaves.
fn press(state: &mut ViewState, key: Key) {
    assert_eq!(
        state.handle(key),
        Action::Continue,
        "{key:?} does not leave"
    );
}

/// `key` pressed `times` times.
fn press_times(state: &mut ViewState, key: Key, times: usize) {
    for _ in 0..times {
        press(state, key);
    }
}

/// "`g`, this line, `enter`": go-to as the terminal drives it, one `Key` per
/// character of the line (8k2 §4 rule 2).
fn go_to(state: &mut ViewState, line: &str) {
    press(state, Key::Char('g'));
    for ch in line.chars() {
        press(state, Key::Char(ch));
    }
    press(state, Key::Enter);
}

/// "`/`, this text, `enter`" (8l1 §4 rule 1 and 3).
fn search(state: &mut ViewState, text: &str) {
    press(state, Key::Char('/'));
    for ch in text.chars() {
        press(state, Key::Char(ch));
    }
    press(state, Key::Enter);
}

/// `pbr` at 80 x 60 with the feature levels at none, so a block is three lines
/// — ruler, top strand, bottom strand — and the 53 body rows hold 13 blocks and
/// their gaps, 4 rows each bar the last block's gap (§5).
fn big() -> ViewState {
    let mut state: ViewState = ViewState::new(pbr(), BIG_W, BIG_H);
    press_times(&mut state, Key::Char('F'), 2);
    assert_eq!(state.features(), FeatureLevel::None, "no lanes");
    assert_eq!(
        state.visible(),
        0..13,
        "13 blocks of three lines fill the body, so a half is 6",
    );
    state
}

/// The block of the laid-out record that holds `base`, read off the layout and
/// not off the screen.
fn block_of(state: &ViewState, base: u64) -> usize {
    let built = state.layout().expect("80 columns lays the record out");
    built
        .blocks
        .iter()
        .position(|(first, last)| *first <= base && base < *last)
        .expect("the base lies in some block")
}

/// The screen drawn on a test backend at `size`: every cell's background, in
/// screen order.
fn backgrounds(state: &ViewState, size: (u16, u16)) -> Vec<Color> {
    let (width, height): (u16, u16) = size;
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| draw(frame, state)).unwrap();
    terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(|cell| cell.bg)
        .collect()
}

/// Whether any cell on the screen is painted yellow: the marks the hits make
/// (8l1 §4 rule 8).
fn any_yellow(state: &ViewState, size: (u16, u16)) -> bool {
    backgrounds(state, size).contains(&Color::Yellow)
}

/// The non-blank body rows of `given`, in screen order: what the body holds. A
/// row of the key list is one of these.
fn body_rows(state: &ViewState, size: (u16, u16), given: std::ops::Range<usize>) -> Vec<String> {
    let (width, height): (u16, u16) = size;
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| draw(frame, state)).unwrap();
    let buffer = terminal.backend().buffer();
    let columns: usize = usize::from(width);
    given
        .map(|row| {
            buffer.content[row * columns..(row + 1) * columns]
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>()
        })
        .map(|line| {
            line.chars()
                .skip(1)
                .take(columns - 2)
                .collect::<String>()
                .trim()
                .to_string()
        })
        .filter(|row| !row.is_empty())
        .collect()
}

/// The cursor and the top block after `ctrl-d` and `ctrl-u`, which move half a
/// screen of blocks and take the top with them (§4 rule 2, M8-38).
#[test]
fn ctrl_d_and_ctrl_u_move_half_a_screen() {
    let mut state: ViewState = big();
    press(&mut state, Key::Home);
    press_times(&mut state, Key::Char('l'), 10);
    assert_eq!(
        state.cursor(),
        10,
        "the cursor stands ten bases into the record"
    );

    press(&mut state, Key::CtrlD);
    assert_eq!(
        state.cursor(),
        406,
        "six blocks down from block 0, whose offset is kept: block 6 starts at \
         396"
    );
    assert_eq!(state.visible().start, 6, "the top moves by half");

    press(&mut state, Key::CtrlD);
    assert_eq!(state.cursor(), 802, "another six blocks down, block 12");
    assert_eq!(state.visible().start, 12, "and the top again");

    press(&mut state, Key::CtrlU);
    assert_eq!(state.cursor(), 406, "half a screen back up");
    assert_eq!(state.visible().start, 6, "and the top back with the cursor");
    assert_eq!(
        state.visible().len(),
        13,
        "the same 13 blocks are drawn, so the half is still 6"
    );
}

/// The same keys stop at the record's last block instead of wrapping round, even
/// on a circular record, clamping the cursor to that block's last base, and stop
/// at the first block above the record's first base (§4 rule 2, §5).
#[test]
fn half_pages_stop_at_the_ends() {
    let mut state: ViewState = big();
    go_to(&mut state, "4169");
    assert_eq!(state.cursor(), 4168, "go-to 4169 names");
    let block: usize = block_of(&state, 4168);
    assert_eq!(block, 63, "and it stands ten bases into block 63");
    assert_eq!(
        state.cursor() - state.layout().expect("a layout").blocks[block].0,
        10,
        "the offset the cursor keeps"
    );
    let drawn: std::ops::Range<usize> = state.visible();
    assert!(
        drawn.contains(&block),
        "the cursor's block is drawn in {drawn:?}"
    );

    press(&mut state, Key::CtrlD);
    assert_eq!(
        state.cursor(),
        4360,
        "six blocks down from block 63 runs past the last block, so the cursor \
         lands on block 66's last base and not on the offset it carries"
    );
    assert_eq!(block_of(&state, 4360), 66, "block 66 holds 4356..4360");
    assert_eq!(
        state.cursor(),
        state.record().sequence.length() - 1,
        "the record's last base, and no wrap past it on a circular record"
    );
    assert!(
        state.visible().contains(&66),
        "and the last block is drawn in {:?}",
        state.visible()
    );
    press(&mut state, Key::CtrlD);
    assert_eq!(state.cursor(), 4360, "`ctrl-d` again does not move it");

    go_to(&mut state, "11");
    assert_eq!(state.cursor(), 10, "go-to 11 names");
    assert_eq!(state.visible(), 0..13, "and the scroll has come back up");
    press(&mut state, Key::CtrlU);
    assert_eq!(
        state.cursor(),
        10,
        "half a screen up from block 0 stops at the first block"
    );
    assert_eq!(state.visible().start, 0, "and the top stays there");
    press(&mut state, Key::CtrlU);
    assert_eq!(state.cursor(), 10, "`ctrl-u` again does not move it");
    assert_eq!(state.visible().start, 0, "and the top neither");
}

/// In an open list the two keys move the highlight half a page of the rows the
/// list draws, and at a prompt they are not text and move nothing (§4 rules 1
/// and 3).
#[test]
fn half_pages_in_a_list() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    press(&mut state, Key::Char('j'));
    let cursor: u64 = state.cursor();
    assert_ne!(cursor, 0, "the cursor stands somewhere in the record");

    press(&mut state, Key::Char('f'));
    let rows: usize = state.list().expect("`f` opens the list").rows().len();
    assert!(
        rows > 21,
        "pBR322's feature rows are more than the 21 rows at hand here: {rows}"
    );
    press(&mut state, Key::Home);
    assert_eq!(
        state.list().expect("the list is open").at(),
        Some(0),
        "`home` highlights the first row"
    );

    press(&mut state, Key::CtrlD);
    assert_eq!(
        state.list().expect("the list is open").at(),
        Some(10),
        "`ctrl-d` moves the highlight half the 20 body rows a list draws"
    );
    press(&mut state, Key::CtrlD);
    assert_eq!(
        state.list().expect("the list is open").at(),
        Some(20),
        "and again"
    );
    press(&mut state, Key::CtrlU);
    assert_eq!(
        state.list().expect("the list is open").at(),
        Some(10),
        "`ctrl-u` moves it back"
    );
    assert_eq!(state.cursor(), cursor, "and the cursor never moved");

    press(&mut state, Key::Esc);
    assert!(state.list().is_none(), "`esc` closes the list");
    assert_eq!(state.cursor(), cursor, "and moves nothing");

    go_to(&mut state, "12");
    assert_eq!(state.cursor(), 11, "the 1-based 12 go-to gives");

    press(&mut state, Key::Char('g'));
    press(&mut state, Key::Char('1'));
    press(&mut state, Key::Char('2'));
    press(&mut state, Key::CtrlD);
    assert_eq!(
        state.prompt(),
        Some("12"),
        "`ctrl-d` at a prompt is not text and adds nothing to the line"
    );
    assert_eq!(state.cursor(), 11, "and it moves nothing there");
    press(&mut state, Key::CtrlU);
    assert_eq!(state.prompt(), Some("12"), "as `ctrl-u` also does nothing");
    assert_eq!(state.cursor(), 11, "and nothing there either");
    press(&mut state, Key::Enter);
    assert!(state.prompt().is_none(), "`enter` acts on the line");
}

/// `esc` clears the hits the screen marks, leaving the cursor, the scroll and
/// the status line where they were, and leaves `n` and `N` with nothing to step;
/// at a prompt it closes the prompt and leaves the hits, as before (§4 rule 4,
/// M8-37).
#[test]
fn esc_clears_the_hits() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    search(&mut state, "AATT");
    assert_eq!(state.cursor(), 58, "the fixture's first hit for `AATT`");
    assert_eq!(
        state.hits().expect("the search marked hits").total(),
        8,
        "the eight hits the fixture records"
    );
    let drawn: std::ops::Range<usize> = state.visible();
    assert!(
        any_yellow(&state, (W, H)),
        "the marked bases are drawn yellow"
    );

    // The prompt's own rule, unchanged: `esc` closes it and leaves the hits, the
    // cursor and the scroll where they were (8l1 §4 rule 1).
    press(&mut state, Key::Char('/'));
    assert!(state.search().is_some(), "`/` opens the prompt again");
    press(&mut state, Key::Esc);
    assert!(state.search().is_none(), "`esc` closes the prompt");
    assert_eq!(
        state.hits().expect("the hits are still there").total(),
        8,
        "and leaves them marked"
    );
    assert_eq!(state.cursor(), 58, "and moves nothing");
    assert!(
        any_yellow(&state, (W, H)),
        "and draws the marks on the bases still"
    );

    press(&mut state, Key::Esc);
    assert!(
        state.hits().is_none(),
        "`esc` with no list, prompt or key list open clears the hits"
    );
    assert_eq!(state.cursor(), 58, "and leaves the cursor where it stood");
    assert_eq!(state.visible(), drawn, "and the scroll where it was");
    assert!(
        !any_yellow(&state, (W, H)),
        "no cell is marked yellow any more"
    );
    assert!(
        state.message().is_none(),
        "and the status line says nothing new"
    );

    press(&mut state, Key::Char('n'));
    assert_eq!(state.cursor(), 58, "`n` with no hits moves nothing");
    press(&mut state, Key::Char('N'));
    assert_eq!(state.cursor(), 58, "as `N` either");

    press(&mut state, Key::Esc);
    assert!(
        state.hits().is_none(),
        "`esc` with no hits to clear changes nothing"
    );
    assert_eq!(state.cursor(), 58, "and it moves nothing at all");
    assert_eq!(state.visible(), drawn, "and the scroll either");
}

/// The key list has a row for each of the two new keys, next to the rows the
/// keys go with, and the key line stays as it was (§4 rule 5).
#[test]
fn the_key_list_has_the_new_keys() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    press(&mut state, Key::Char('?'));
    assert!(state.help(), "`?` opens the key list");
    let listed: Vec<String> = body_rows(&state, (W, H), BODY_WITHOUT);
    let page: usize = listed
        .iter()
        .position(|row| row.starts_with("PgUp PgDn  "))
        .expect("the key list has a `PgUp PgDn` row");
    let half: &str = &listed[page + 1];
    assert!(
        half.starts_with("ctrl-d ctrl-u  ")
            && half.chars().count() > "ctrl-d ctrl-u  ".chars().count(),
        "the row after `PgUp PgDn` names the two keys and says what they do: {half}"
    );

    let steps: usize = listed
        .iter()
        .position(|row| row.starts_with("n N  "))
        .expect("the key list has an `n N` row");
    let clears: &str = &listed[steps + 1];
    assert!(
        clears.starts_with("esc  ") && clears.chars().count() > "esc  ".chars().count(),
        "the row after `n N` names `esc` and says what it does: {clears}"
    );
}
