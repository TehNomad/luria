//! The overview strip as the view holds it and draws it (contract 8l2 §5; plan
//! §4.6 and proposal §2.1 of `docs/tasks/milestone-8-view.md`; D054).
//!
//! `pbr` is `tests/fixtures/genbank/raw/J01749.1.gb` — 4361 bp, circular by its own
//! declaration, 50 features and two usable CDSs of the three, so M8-11 selects none
//! — `f62` is `tests/fixtures/view/frames-62.gb`, 62 bp with the CDSs `fP`
//! (`join(56..62,1..11)`) and `fM` (`complement(join(53..62,1..8))`), 18 bases each
//! and both crossing the origin, and `w16` is the 16 bp FASTA `>w` /
//! `TTCAAAAAAAAAAGAA`. Every test draws at 80 × 24, whose frame has 78 inner
//! columns: `pbr`'s strip has 78 − 2 − 5 = 71 columns, and the 62 bp record gets 62
//! columns of one base each and a strip that much shorter, because a strip is never
//! wider than the record has bases (D054).
//!
//! The columns asserted below are the ones the planner derived with D054's floor
//! formula `⌊((p + 1)·w − 1) / L⌋`, written as the buffer's own `x` — 0 is the
//! frame's left border and 79 its right — which is how the contract's §5 numbers its
//! screen columns: `pbr`'s strip cells are x 3 to 73 with `4361`
//! at 75 to 78, and `f62`'s are 3 to 64 with `62` at 66 to 67. The hit starts are
//! the rows of `tests/fixtures/view/search-dna-hits.tsv` — `AATT` at 58, 251, 1319,
//! 1333, 3233, 3539, 3794 and 4359 — and the feature spans are the fixture's own
//! feature table: `mobile_element 3148..4361` and the `tet` gene at `86..1276`. The
//! rows they sit on are the ones `tests/fixtures/view/README.md` records.
//!
//! The screen is read back off a `ratatui` test backend the way
//! `tests/sequence_view_search.rs` reads it: row 1 is the strip's bar line and row
//! 2 its marks, both between the frame's borders at x 1 to 78.

use luria_cli::sequence_view::screen::draw;
use luria_cli::sequence_view::state::Action;
use luria_cli::sequence_view::state::Key;
use luria_cli::sequence_view::state::ViewState;
use luria_core::identity::Identifier;
use luria_core::view_record::open_record;
use luria_core::view_record::ViewRecord;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::fs;
use std::path::Path;

/// Every test draws at this size: 80 columns and 24 rows, so the frame's inner area
/// is 78 columns wide, its first two rows are the strip while it is drawn, and the
/// blocks below it have 18 rows — or 20 with the strip hidden (8l2 §4 rule 2).
const W: u16 = 80;
const H: u16 = 24;

/// The screen row the strip's bar line is drawn on, and the one under it.
const BAR_ROW: usize = 1;
const MARK_ROW: usize = 2;

/// The first and last inner column of the frame, as the buffer's own `x`: the
/// borders are at x 0 and x 79, and the strip's cells begin at x 3 after the `1 `
/// that names the line (8k1 §4 rule 1, 8l2 §4 rule 3).
const INSIDE: (usize, usize) = (1, 78);

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

/// The 62 bp record of the view fixtures: two 18 bp CDSs crossing the origin.
fn f62() -> ViewRecord {
    record("../../tests/fixtures/view/frames-62.gb", "frames62")
}

/// The 16 bp FASTA of contract §5: no features, no CDSs, an assumed topology.
fn w16() -> ViewRecord {
    open_record(
        Identifier::create("w".to_string()).unwrap(),
        ">w\nTTCAAAAAAAAAAGAA\n".to_string(),
    )
    .expect("the 16 bp FASTA record")
}

/// `key` pressed once, with a failure saying it leaves.
fn press(state: &mut ViewState, key: Key) {
    assert_eq!(
        state.handle(key),
        Action::Continue,
        "{key:?} does not leave"
    );
}

/// The screen drawn on a test backend: every row as the symbols of its cells.
fn rows(state: &ViewState) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(W, H)).unwrap();
    terminal.draw(|frame| draw(frame, state)).unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = usize::from(buffer.area.width);
    buffer
        .content
        .chunks(width)
        .map(|cells| cells.iter().map(|cell| cell.symbol()).collect())
        .collect()
}

/// One screen row as its cells. Every glyph on the strip is one cell wide, so the
/// index of a cell is the screen column minus one.
fn cells(state: &ViewState, row: usize) -> Vec<char> {
    rows(state)[row].chars().collect()
}

/// The strip's bar line as one string.
fn bar_line(state: &ViewState) -> String {
    rows(state)[BAR_ROW].clone()
}

/// One stretch of a row, `first` to `last` at both ends, as a string: what a bar run
/// shows, spelled out for the message of the check that reads it.
fn run(state: &ViewState, row: usize, first: usize, last: usize) -> String {
    cells(state, row)[first..=last].iter().collect()
}

/// Every non-space cell between the frame's borders, as `(x, glyph)` in order:
/// what the marks line holds, and nothing else (8l2 §4 rule 5).
fn marks(state: &ViewState, row: usize) -> Vec<(usize, char)> {
    let line: Vec<char> = cells(state, row);
    (INSIDE.0..=INSIDE.1)
        .filter_map(|column| {
            let glyph: char = line[column];
            match glyph {
                ' ' => None,
                _ => Some((column, glyph)),
            }
        })
        .collect()
}

/// §5.1: the strip opens drawn on a record too long for the body and not on a short
/// one. pBR322's 67 blocks — `tests/sequence_view.rs` counts them, 4361 bases in
/// blocks of 66 — are more than twice the three the body holds with the strip off
/// (8l2 §4 rule 1), and its bar line is `1 `, the strip and ` 4361` between
/// the borders; the 16 bp FASTA and the 62 bp record hold the body at open, so they
/// open without a strip.
#[test]
fn the_strip_opens_on_a_long_record() {
    let state: ViewState = ViewState::new(pbr(), W, H);
    assert!(state.overview(), "4361 bp on 18 body rows needs a strip");
    let line: String = bar_line(&state);
    assert!(line.starts_with("│1 "), "row 1 starts: {line}");
    assert!(line.ends_with("4361│"), "row 1 ends: {line}");

    for (what, short) in [("the 16 bp FASTA", w16()), ("the 62 bp record", f62())] {
        let small: ViewState = ViewState::new(short, W, H);
        assert!(
            !small.overview(),
            "{what} fits the body, so it opens without a strip"
        );
    }
}

/// §5.2: `o` flips the strip, and while it is drawn the blocks have two rows less.
/// pBR322 with `t` on its three frames puts 10 lines in block 0 and 9 in block 1:
/// 10 alone fits the 18 rows the strip leaves, and 10 + the gap + 9 fits only the
/// 20 rows the body has without it (8l2 §4 rule 2).
#[test]
fn o_flips_it_and_it_takes_two_rows() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    press(&mut state, Key::Char('t'));
    assert!(state.overview(), "`t` does not hide the strip");
    assert_eq!(
        state.visible(),
        0..1,
        "block 0 and block 1 do not fit under the strip"
    );

    press(&mut state, Key::Char('o'));
    assert!(!state.overview(), "o hides the strip");
    assert_eq!(
        state.visible(),
        0..2,
        "the two blocks fit once the strip's two rows are back"
    );

    press(&mut state, Key::Char('o'));
    assert!(state.overview(), "o brings it back");
}

/// §5.3: the marks line holds the part on screen and the cursor, and nothing else.
/// pBR322 at open draws blocks 0 and 1, bases 0 to 131 (66 + 66), and of D054's
/// columns column 0 (x 3, where `▲` stands on base 1), column 1 (x 4) and column 2
/// (x 5, since base 131 is in `⌊((131 + 1)·71 − 1)/4361⌋ = 2`) hold a base on
/// screen. `End` moves the cursor to the record's last base, which is drawn in the
/// strip's last column, x 73.
#[test]
fn line_two_marks_the_screen_and_the_cursor() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert_eq!(
        marks(&state, MARK_ROW),
        vec![(3, '▲'), (4, '▀'), (5, '▀')],
        "the cursor's column, and the two columns the rest of the screen falls in"
    );
    let line: Vec<char> = cells(&state, BAR_ROW);
    assert_eq!(line[1], '1', "the bar line names the record's first base");

    press(&mut state, Key::End);
    let line: Vec<char> = cells(&state, MARK_ROW);
    assert_eq!(line[73], '▲', "the last base is drawn in the last column");
}

/// §5.4: a search dots the column of every shown hit's start. `AATT` gives the
/// eight rows of `search-dna-hits.tsv`: 58, 251, 1319, 1333, 3233, 3539, 3794 and
/// 4359, which under `⌊((p + 1)·71 − 1)/4361⌋` come to x 3, 7, 24, 24, 55, 60, 64
/// and 73 — 1319 and 1333 share a column, and 58 is the cursor's, where `▲` wins the
/// cell (8l2 §4 rule 5). The screen holds blocks 0 and 1, bases 0 to 131, so its
/// columns 0, 1 and 2 are `▀`: the hit at 58 gives column 0 its `▲`, and the hits
/// in columns 1 and 2 are none.
#[test]
fn hits_are_dotted() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    press(&mut state, Key::Char('/'));
    for ch in "AATT".chars() {
        press(&mut state, Key::Char(ch));
    }
    press(&mut state, Key::Enter);
    assert_eq!(state.cursor(), 58, "the cursor goes to the first hit");
    assert_eq!(
        marks(&state, MARK_ROW),
        vec![
            (3, '▲'),
            (4, '▀'),
            (5, '▀'),
            (7, '·'),
            (24, '·'),
            (55, '·'),
            (60, '·'),
            (64, '·'),
            (73, '·'),
        ],
        "two hits share column 24 and the hit at 58 is under the cursor"
    );
}

/// §5.5: the longest features are bars, longest first by bases covered. pBR322's
/// `mobile_element 3148..4361` (1214 bases) is drawn before its `tet` gene
/// (`gene 86..1276`, 1191 bases), which is drawn before anything shorter; the two
/// spans come to D054's columns 51 to 70 and 1 to 20, which are x 54 to 73 and 4 to
/// 23, 20 cells each: the name, the space and the arrow §4 rule 4 asks for, then `═`
/// (8l2 §4 rule 4).
///
/// §5's row for this test says every cell of the two spans is "neither `─` nor a
/// space", which cannot hold next to §4 rule 4's label: the space between a name and
/// its arrow sits in the span. The check here pins the whole run, which is what §5
/// describes apart from that space, and the difference is reported to the planner.
#[test]
fn the_longest_features_are_bars() {
    let state: ViewState = ViewState::new(pbr(), W, H);
    let line: String = bar_line(&state);
    assert_eq!(
        run(&state, BAR_ROW, 54, 73),
        format!("mobile_element ▸{}", "═".repeat(20 - 16)),
        "the longest bar, 20 cells of which its name and arrow take 16: {line}"
    );
    assert_eq!(
        run(&state, BAR_ROW, 4, 23),
        format!("tet ▸{}", "═".repeat(20 - 5)),
        "the next bar, 20 cells of which its name and arrow take 5: {line}"
    );
}

/// §5.6: while the `f` list is open the strip is not drawn and the list has the
/// body as 8k3 gives it, so its first row is the frame's first inner row (8l2 §4
/// rule 2). `overview()` stays where `o` left it: closing the list brings the strip
/// back to the same place.
#[test]
fn a_list_hides_the_strip() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert!(state.overview(), "pbr opens with the strip");
    press(&mut state, Key::Char('f'));
    let line: String = bar_line(&state);
    assert!(line.contains("source"), "the first list row: {line}");
    assert!(
        !line.starts_with("│1 "),
        "the strip is not drawn over the list: {line}"
    );
    assert!(state.overview(), "the flag is only hidden, not reset");
}

/// §5.7: a feature crossing the origin is drawn at both ends of the strip. `f62`
/// opens without a strip — two blocks fit the body — and `o` brings it: its 62 bases
/// on 73 spare columns give 62 columns of one base each (D054), so the strip runs to
/// x 64 and `62` ends at 67. `fP` (`join(56..62,1..11)`) is drawn from
/// its two intervals, bases `[55, 62)` at columns 58 to 64 and `[0, 11)` at columns
/// 3 to 13, leaving columns 14 and 57 `─`; `fM` covers 18 bases too, so file order
/// draws `fP` first and skips `fM` for every column it needs (8l2 §4 rule 4). Each
/// run again holds §4 rule 4's label space, as §5.5 notes.
#[test]
fn a_feature_across_the_origin_has_two_runs() {
    let mut state: ViewState = ViewState::new(f62(), W, H);
    assert!(!state.overview(), "62 bp on 20 body rows needs no strip");
    press(&mut state, Key::Char('o'));
    assert!(state.overview(), "o draws it anyway");
    let cells: Vec<char> = cells(&state, BAR_ROW);
    let left: String = run(&state, BAR_ROW, 3, 13);
    assert_eq!(
        left,
        format!("fP ▸{}", "═".repeat(11 - 4)),
        "fP's 11-cell run at the strip's left end"
    );
    let right: String = run(&state, BAR_ROW, 58, 64);
    assert_eq!(
        right,
        format!("fP ▸{}", "═".repeat(7 - 4)),
        "and its 7-cell run at the strip's right end"
    );
    assert_eq!(cells[14], '─', "the gap between the two runs");
    assert_eq!(cells[57], '─', "and the gap on their other side");
    assert_eq!(cells[64], '═', "the strip ends at column 64");
    assert_eq!(cells[65], ' ', "and the length follows it");
    let label: String = cells[66..=67].iter().collect();
    assert_eq!(label, "62", "62 bp ends at column 67");
}

/// §5.8 (round 2 §4): showing the strip takes two rows from the blocks, so the
/// scroll follows the cursor as 8k1 §4 rule 6 has it — the cursor's block stays
/// drawn and the top moves as little as the rows allow. `pbr` with `t` on and the
/// strip hidden holds block 0 (10 lines) and block 1 (9) in the 20 rows the body
/// has without it; base 67 as the record numbers it, the first base of block 1 on
/// the 66-base line, is drawn. With the strip back the same two blocks need 20 rows
/// again, 2 more than the 18 left, so the top moves to block 1 and no further.
#[test]
fn o_keeps_the_cursor_on_screen() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    press(&mut state, Key::Char('t'));
    press(&mut state, Key::Char('o'));
    assert!(!state.overview(), "o hides the strip that opens drawn");
    for key in [Key::Char('g'), Key::Char('6'), Key::Char('7'), Key::Enter] {
        press(&mut state, key);
    }
    assert_eq!(
        state.cursor(),
        66,
        "base 67 as the record numbers it (D006), the first base of block 1"
    );
    assert_eq!(
        state.visible(),
        0..2,
        "both blocks fit the 20 rows the body holds without the strip"
    );
    press(&mut state, Key::Char('o'));
    assert!(state.overview(), "o draws it again");
    assert_eq!(
        state.visible(),
        1..2,
        "block 1 is still drawn, and the top has moved as little as follow allows"
    );
}

/// §5.9 (round 2 §4): a pick from the CDS list closes the list, and the run of
/// blocks the pick measured has to be the one the screen then draws. `pbr` opens
/// with the strip on; base 302 sits in block 4 (zero-based 301, and 4·66 = 264 ≤
/// 301 < 5·66 = 330) and `c` lists the CDSs with the one
/// holding it highlighted. `enter` selects `tet` (CDS 0, `86..1276` in the file's
/// numbering), closes the list and moves the cursor to tet's start codon, base 86
/// as the file numbers it (M8-21); the block holding it is drawn whatever the
/// list's own rows were while it was open. The run that follows the cursor starts
/// at block 2, where 6 + 1 + 4 + 1 + 4 = 16 rows cover blocks 2, 3 and 4 of the 18
/// the strip leaves.
#[test]
fn picking_a_cds_keeps_the_cursor_on_screen() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert!(state.overview(), "4361 bp on 18 body rows needs a strip");
    for key in [
        Key::Char('g'),
        Key::Char('3'),
        Key::Char('0'),
        Key::Char('2'),
        Key::Enter,
    ] {
        press(&mut state, key);
    }
    assert_eq!(state.cursor(), 301, "base 302 as the record numbers it");
    let before: std::ops::Range<usize> = state.visible();
    assert_eq!(before, 2..5, "and the run drawn reaches block 4");
    press(&mut state, Key::Char('c'));
    assert!(state.list().is_some(), "the CDS list opens over the blocks");
    press(&mut state, Key::Enter);
    assert_eq!(state.selected(), Some(0), "tet, the file's first CDS");
    assert!(state.list().is_none(), "and the list has closed");
    assert_eq!(
        state.cursor(),
        85,
        "the pick moves the cursor to tet's start codon, base 86 as the file numbers it (M8-21)"
    );
    let block: usize = state
        .layout()
        .expect("80 x 24 lays the record out")
        .blocks
        .iter()
        .position(|(first, last)| *first <= 85 && 85 < *last)
        .expect("some block holds base 85");
    let after: std::ops::Range<usize> = state.visible();
    assert!(
        after.contains(&block),
        "the run drawn holds the block the cursor moved to: {before:?} then {after:?}"
    );
}
