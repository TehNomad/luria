//! The feature list and the CDS picker as the view holds them and draws them
//! (contract 8k3 §5). Every record is a fixture read by `open_record`, every size
//! 80 × 24, and every number comes off the fixture or off the tables that record
//! it: pBR322 has 50 features and three CDSs — `tet` (feature 9, the gene at
//! 86..1276 and feature 10, the CDS at the same place), the ROP CDS (feature 26,
//! `CDS 1915..2106`, refused for having no id) and `bla` (feature 37 the gene,
//! 38 the CDS, 39 the sig_peptide and 40 the mat_peptide, all `/gene="bla"`) — as
//! `tests/fixtures/genbank/pbr322-features.tsv` and
//! `tests/fixtures/genbank/pbr322-cds.tsv` give them. synthetic has 27 features,
//! 24 of them CDSs, and its refusals are in
//! `tests/fixtures/genbank/synthetic-unused.tsv`.
//!
//! The list is driven the way the terminal drives it, through `handle` and its
//! `Key`s, and what the screen shows is read back off a `ratatui` test backend
//! the way `tests/sequence_view.rs` reads the review and the blocks: rows of
//! symbols, and the modifiers of every cell for the highlight and the greyed rows
//! (§4 rule 7).

use luria_cli::sequence_view::layout::LaneMode;
use luria_cli::sequence_view::list::ListKind;
use luria_cli::sequence_view::screen::draw;
use luria_cli::sequence_view::state::Action;
use luria_cli::sequence_view::state::Key;
use luria_cli::sequence_view::state::ViewState;
use luria_core::identity::Identifier;
use luria_core::view_record::open_record;
use luria_core::view_record::ViewRecord;
use ratatui::backend::TestBackend;
use ratatui::style::Modifier;
use ratatui::Terminal;
use std::fs;
use std::path::Path;

/// Every test draws at this size: 80 columns and 24 rows, which leaves the body
/// 20 rows (`height - 4`) for the blocks or for the list's rows.
const W: u16 = 80;
const H: u16 = 24;

/// One fixture record, named as the CLI names it: the content decides the format,
/// never the name (D049), and `open_record` is what 8h calls.
fn record(path: &str, name: &str) -> ViewRecord {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text: String =
        fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"));
    open_record(Identifier::create(name.to_string()).unwrap(), text)
        .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
}

/// pBR322: 4361 bp, circular, 50 features, and three CDSs of which two are
/// usable — so M8-11 selects none on open.
fn pbr() -> ViewRecord {
    record("../../tests/fixtures/genbank/raw/J01749.1.gb", "SYNPBR322")
}

/// synthetic: 60 bp, circular, 27 features and 24 CDSs, several of them usable,
/// so again nothing is selected on open, and its refusals are the ones the
/// fixtures record.
fn syn() -> ViewRecord {
    record("../../tests/fixtures/genbank/synthetic.gb", "synthetic")
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

/// Every cell's modifiers on the same screen, in screen order.
fn modifiers(state: &ViewState) -> Vec<Vec<Modifier>> {
    let mut terminal = Terminal::new(TestBackend::new(W, H)).unwrap();
    terminal.draw(|frame| draw(frame, state)).unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = usize::from(buffer.area.width);
    buffer
        .content
        .chunks(width)
        .map(|cells| cells.iter().map(|cell| cell.modifier).collect())
        .collect()
}

/// The rows holding at least one cell with `want` set. With a list open the only
/// reversed cells on screen are the highlighted row's: the blocks, and with them
/// the cursor's cell, are not drawn (§4 rule 7).
fn marked_rows(state: &ViewState, want: Modifier) -> Vec<usize> {
    modifiers(state)
        .iter()
        .enumerate()
        .filter(|(_row, cells)| cells.iter().any(|cell| cell.contains(want)))
        .map(|(row, _cells)| row)
        .collect()
}

/// The column of the first `want` in a drawn row, counted in cells: the frame's
/// border is one cell but several bytes, so a byte index is not a column
/// (`tests/sequence_view.rs`'s `symbol` counts characters for the same reason).
fn column_of(drawn: &[String], row: usize, want: &str) -> usize {
    let line: &str = drawn
        .get(row)
        .unwrap_or_else(|| panic!("no row {row} on a {W}x{H} screen"));
    let at: usize = line
        .find(want)
        .unwrap_or_else(|| panic!("no {want:?} in row {row}: {line}"));
    line[..at].chars().count()
}

/// True when `line` holds a run of `want` or more DNA letters, which is what a
/// block's bases row is: the list's rows name features, and none of them can hold
/// such a run.
fn holds_base_run(line: &str, want: usize) -> bool {
    let mut run: usize = 0;
    for ch in line.chars() {
        run = if matches!(ch, 'A' | 'C' | 'G' | 'T') {
            run + 1
        } else {
            0
        };
        if run >= want {
            return true;
        }
    }
    false
}

/// The block of the laid-out record that holds a base.
fn block_holding(state: &ViewState, base: u64) -> usize {
    let built = state.layout().expect("a layout at this size");
    built
        .blocks
        .iter()
        .position(|(first, last)| *first <= base && base < *last)
        .unwrap_or_else(|| panic!("no block holds base {base}"))
}

/// §4 rules 1 and 3: `f` opens the feature list on every feature, in file order,
/// with nothing typed and the first row highlighted, and `esc` closes it without
/// touching anything else.
#[test]
fn f_lists_every_feature() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    let mode: LaneMode = *state.mode();

    assert_eq!(state.handle(Key::Char('f')), Action::Continue);
    let list = state.list().expect("f opens a list");
    assert_eq!(list.kind(), ListKind::Features, "the features");
    assert_eq!(
        list.rows(),
        (0..50).collect::<Vec<usize>>(),
        "every feature"
    );
    assert_eq!(list.at(), Some(0), "the first row highlighted");
    assert_eq!(list.filter(), "", "with nothing typed");

    assert_eq!(state.handle(Key::Esc), Action::Continue);
    assert!(state.list().is_none(), "esc closes it");
    assert_eq!(state.cursor(), 0, "and moves nothing");
    assert_eq!(state.mode(), &mode, "changes no lanes");
    assert_eq!(state.message(), None, "and says nothing");
}

/// §4 rule 2: the filter is core's shared matcher, so a prefix reaches a name at
/// a word start and no further, and `esc` starts a list afresh (D056).
#[test]
fn the_filter_is_the_shared_matcher() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    state.handle(Key::Char('f'));

    type_into(&mut state, "bla");
    assert_eq!(
        state.list().expect("open").rows(),
        [37, 38, 39, 40],
        "the gene, the CDS and the two peptides named bla"
    );
    assert_eq!(state.list().expect("open").at(), Some(0));

    walk(&mut state, Key::Backspace, 3);
    assert_eq!(
        state.list().expect("open").filter(),
        "",
        "three characters go"
    );
    assert_eq!(
        state.list().expect("open").rows().len(),
        50,
        "and every feature is back"
    );

    type_into(&mut state, "prot");
    assert_eq!(
        state.list().expect("open").rows(),
        [26],
        "prot reaches ROP protein"
    );

    assert_eq!(state.handle(Key::Esc), Action::Continue);
    state.handle(Key::Char('f'));
    assert_eq!(state.list().expect("open").filter(), "", "a fresh list");
    assert_eq!(state.list().expect("open").rows().len(), 50);
    type_into(&mut state, "rotein");
    assert!(
        state.list().expect("open").rows().is_empty(),
        "and rotein reaches nothing, because a prefix starts at a word"
    );
    assert_eq!(
        state.list().expect("open").at(),
        None,
        "so nothing is highlighted"
    );
}

/// §4 rule 4: `enter` on a feature jumps to its first base and closes the list;
/// its CDS is selected when one is usable and refused by its code when it is not;
/// a feature that is not a CDS feature selects nothing.
#[test]
fn enter_jumps_to_a_feature() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert_eq!(
        state.selected(),
        None,
        "two usable CDSs, so M8-11 selects none"
    );

    // The ROP feature: drawn, with its CDS refused for having no id. The feature
    // is jumped to, and the CDS is not selected (D042, D052).
    state.handle(Key::Char('f'));
    type_into(&mut state, "rop");
    assert_eq!(
        state.list().expect("open").rows(),
        [26],
        "the ROP CDS feature"
    );
    assert_eq!(state.handle(Key::Enter), Action::Continue);
    assert_eq!(state.cursor(), 1914, "CDS 1915..2106, zero-based (D006)");
    assert_eq!(state.selected(), None, "its CDS cannot be selected");
    assert!(
        state.message().is_some_and(|words| words.contains("no_id")),
        "message: {:?}",
        state.message()
    );
    assert!(state.list().is_none(), "and the list closed");

    // The bla CDS, second of the four features named bla: usable, so it is
    // selected, its lane goes on, and the scroll follows the cursor.
    state.handle(Key::Char('f'));
    type_into(&mut state, "bla");
    assert_eq!(
        state.list().expect("open").rows(),
        [37, 38, 39, 40],
        "four features named bla"
    );
    walk(&mut state, Key::Down, 1);
    assert_eq!(state.list().expect("open").at(), Some(1));
    assert_eq!(state.handle(Key::Enter), Action::Continue);
    assert_eq!(
        state.cursor(),
        4152,
        "complement(3293..4153) goes to its five prime end, the last base it writes (M8-21)"
    );
    assert_eq!(state.selected(), Some(2), "the bla CDS");
    assert_eq!(state.mode(), &LaneMode::Cds(2), "and its own lane");
    assert!(state.list().is_none(), "the list closed");
    let block: usize = block_holding(&state, 4152);
    assert!(
        state.visible().contains(&block),
        "the scroll followed the cursor to block {block}: {:?} visible",
        state.visible()
    );

    // The tet gene: not a CDS feature, so the selection stays where it was. Seven
    // features carry `/gene="tet"` — the gene at 86..1276 (line 259), the CDS there
    // (line 261) and the five small features at 146..147, 411..414, 426, 469..472
    // and 526..528 (lines 278, 282, 285, 290, 293) — and the gene is the first.
    state.handle(Key::Char('f'));
    type_into(&mut state, "tet");
    assert_eq!(
        state.list().expect("open").rows(),
        [9, 10, 12, 13, 14, 15, 16],
        "every feature named tet, in file order"
    );
    assert_eq!(state.handle(Key::Enter), Action::Continue);
    assert_eq!(state.cursor(), 85, "86..1276, zero-based");
    assert_eq!(
        state.selected(),
        Some(2),
        "a feature that is no CDS selects nothing"
    );
    assert_eq!(state.mode(), &LaneMode::Cds(2), "and changes no lanes");
}

/// §4 rule 4 and plan §5.3: a feature that is not drawn, and one whose start is
/// not stated, are not jumped to; the list stays open and says why. A feature
/// with a stated start is.
#[test]
fn a_feature_without_a_stated_start_does_not_move() {
    let mut state: ViewState = ViewState::new(syn(), W, H);
    walk(&mut state, Key::Char('l'), 5);
    assert_eq!(state.cursor(), 5, "the cursor walked there by keys");

    // `order(1..6,7..12)` is refused `order` by the display parser (D053), so it
    // is in the list with its reason and has no base to jump to.
    state.handle(Key::Char('f'));
    type_into(&mut state, "ord");
    assert_eq!(
        state.list().expect("open").rows(),
        [18],
        "the ord CDS feature"
    );
    assert_eq!(state.handle(Key::Enter), Action::Continue);
    assert_eq!(
        state.cursor(),
        5,
        "a feature that is not drawn does not move"
    );
    assert!(state.list().is_some(), "and the list stays open");
    assert!(
        state.message().is_some_and(|words| words.contains("order")),
        "message: {:?}",
        state.message()
    );

    // `<1..12` is drawn, with a start that is not stated (D053): there is no first
    // base to stand on, so nothing moves (plan §5.3).
    assert_eq!(state.handle(Key::Esc), Action::Continue);
    state.handle(Key::Char('f'));
    type_into(&mut state, "fuzzy");
    assert_eq!(
        state.list().expect("open").rows(),
        [17],
        "the fuzzy CDS feature"
    );
    assert_eq!(state.handle(Key::Enter), Action::Continue);
    assert_eq!(state.cursor(), 5, "an open start has no base to jump to");
    assert!(state.list().is_some(), "and the list stays open");
    assert!(
        state.message().is_some(),
        "with the reason: {:?}",
        state.message()
    );

    // `filler`, the misc_feature at 49..54, is drawn with a stated start.
    assert_eq!(state.handle(Key::Esc), Action::Continue);
    state.handle(Key::Char('f'));
    type_into(&mut state, "filler");
    assert_eq!(
        state.list().expect("open").rows(),
        [25],
        "the filler misc_feature"
    );
    assert_eq!(state.handle(Key::Enter), Action::Continue);
    assert_eq!(state.cursor(), 48, "49..54, zero-based");
    assert!(state.list().is_none(), "and the list closed");
    assert_eq!(state.message(), None, "with nothing refused");
}

/// §4 rule 5 and D052: `c` lists every CDS, usable or not, and `enter` selects
/// only a usable one. An unusable one leaves the list open with its code.
#[test]
fn c_selects_only_a_usable_cds() {
    let mut state: ViewState = ViewState::new(syn(), W, H);
    assert_eq!(
        state.selected(),
        None,
        "several usable CDSs, so M8-11 selects none"
    );

    state.handle(Key::Char('c'));
    let list = state.list().expect("c opens a list");
    assert_eq!(list.kind(), ListKind::Cdss, "the CDSs");
    assert_eq!(list.rows(), (0..24).collect::<Vec<usize>>(), "every CDS");
    assert_eq!(list.filter(), "");

    assert_eq!(state.handle(Key::Enter), Action::Continue);
    assert_eq!(state.selected(), Some(0), "the first CDS, p1");
    assert_eq!(state.mode(), &LaneMode::Cds(0), "and its own lane");
    assert!(state.list().is_none(), "the list closed");
    assert_eq!(state.cursor(), 0, "and the cursor did not move");

    // `ps` is refused `pseudo` (tests/fixtures/genbank/synthetic-unused.tsv), so
    // it is listed, and choosing it changes nothing but the message.
    state.handle(Key::Char('c'));
    type_into(&mut state, "ps");
    assert_eq!(
        state.list().expect("open").rows(),
        [9],
        "the ps CDS, listed though it cannot be chosen"
    );
    assert_eq!(state.handle(Key::Enter), Action::Continue);
    assert_eq!(
        state.selected(),
        Some(0),
        "an unusable CDS is never selected"
    );
    assert!(state.list().is_some(), "and the list stays open");
    assert!(
        state
            .message()
            .is_some_and(|words| words.contains("pseudo")),
        "message: {:?}",
        state.message()
    );

    // pBR322's ROP CDS: the second row of its picker.
    let mut pbr_state: ViewState = ViewState::new(pbr(), W, H);
    pbr_state.handle(Key::Char('c'));
    assert_eq!(pbr_state.list().expect("open").rows(), [0, 1, 2]);
    walk(&mut pbr_state, Key::Down, 1);
    assert_eq!(pbr_state.list().expect("open").at(), Some(1));
    assert_eq!(pbr_state.handle(Key::Enter), Action::Continue);
    assert_eq!(pbr_state.selected(), None, "the ROP CDS has no id");
    assert!(
        pbr_state
            .message()
            .is_some_and(|words| words.contains("no_id")),
        "message: {:?}",
        pbr_state.message()
    );
    assert!(pbr_state.list().is_some(), "and its row stays on screen");
}

/// §4 rules 6 and 7: the rows read with the name, the span and the refusal that
/// keeps a CDS unusable; the highlighted row is reversed, a refused row is DIM,
/// and the blocks are not drawn behind them.
#[test]
fn the_list_is_drawn_in_the_body() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    state.handle(Key::Char('c'));

    let drawn: Vec<String> = rows(&state);
    // The body starts at the frame's inner row 1, and the picker's three rows
    // come from there: tet, ROP and bla, in file order.
    assert!(drawn[1].contains("tet"), "row 1: {}", drawn[1]);
    assert!(drawn[1].contains("86..1276"), "row 1: {}", drawn[1]);
    assert!(
        drawn[2].contains("ROP protein"),
        "row 2 names the ROP CDS by its feature: {}",
        drawn[2]
    );
    assert!(
        drawn[2].contains("no_id"),
        "row 2 gives the reason it cannot be chosen: {}",
        drawn[2]
    );
    assert!(drawn[3].contains("bla"), "row 3: {}", drawn[3]);
    assert!(drawn[3].contains("3293..4153"), "row 3: {}", drawn[3]);

    let mods: Vec<Vec<Modifier>> = modifiers(&state);
    let highlight: usize = column_of(&drawn, 1, "tet");
    assert!(
        mods[1][highlight].contains(Modifier::REVERSED),
        "the highlighted row is reversed"
    );
    let refused: usize = column_of(&drawn, 2, "ROP");
    assert!(
        mods[2][refused].contains(Modifier::DIM),
        "an unusable CDS's row is greyed (D052)"
    );
    assert!(
        !mods[2][refused].contains(Modifier::REVERSED),
        "and it is not the highlight"
    );

    // The blocks are not drawn while the list is: no body row carries a run of
    // bases. pBR322's blocks are 60 bases wide at this width, so a drawn block
    // would show one at once.
    for row in 1..=(H - 4) {
        assert!(
            !holds_base_run(&drawn[usize::from(row)], 8),
            "row {row} shows no bases: {}",
            drawn[usize::from(row)]
        );
    }
}

/// §4 rules 3, 7 and 8: while a list is open every character is filter text, `q`
/// included, and only `ctrl-c` leaves.
#[test]
fn keys_go_to_the_open_list() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    state.handle(Key::Char('f'));

    assert_eq!(state.handle(Key::Char('j')), Action::Continue);
    assert_eq!(
        state.list().expect("open").filter(),
        "j",
        "a movement key is a typed character here"
    );
    assert_eq!(state.cursor(), 0, "and it moves nothing");
    assert_eq!(state.mode(), &LaneMode::Off, "and changes no lanes");

    assert_eq!(
        state.handle(Key::Char('q')),
        Action::Continue,
        "q types while a list is open"
    );
    assert_eq!(state.list().expect("open").filter(), "jq");
    assert_eq!(state.handle(Key::CtrlC), Action::Quit, "ctrl-c quits");
}

/// §4 rules 3 and 7: `home`, `end` and the page keys move the highlight by the
/// body's rows, and the rows drawn are the ones holding it.
#[test]
fn the_highlight_stays_on_screen() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    state.handle(Key::Char('f'));

    assert_eq!(state.handle(Key::End), Action::Continue);
    assert_eq!(state.list().expect("open").at(), Some(49), "the last row");
    let rows_with: Vec<usize> = marked_rows(&state, Modifier::REVERSED);
    assert_eq!(rows_with.len(), 1, "one highlighted row: {rows_with:?}");
    let drawn: Vec<String> = rows(&state);
    let bottom: usize = rows_with[0];
    assert!(
        drawn[bottom].contains("4331..4334"),
        "row {bottom} is the last feature: {}",
        drawn[bottom]
    );

    assert_eq!(state.handle(Key::Home), Action::Continue);
    assert_eq!(state.list().expect("open").at(), Some(0), "the first row");
    assert_eq!(state.handle(Key::PageDown), Action::Continue);
    assert_eq!(
        state.list().expect("open").at(),
        Some(20),
        "a page is the 20 body rows of this size"
    );
    let rows_with: Vec<usize> = marked_rows(&state, Modifier::REVERSED);
    assert_eq!(rows_with.len(), 1, "one highlighted row: {rows_with:?}");
    let drawn: Vec<String> = rows(&state);
    assert!(
        drawn[rows_with[0]].contains("1891..1892"),
        "row {} is the twenty-first feature: {}",
        rows_with[0],
        drawn[rows_with[0]]
    );
}

/// §4 rule 2: the highlight goes to the first row whenever the rows change, and
/// where it was before is not kept.
#[test]
fn the_highlight_resets_when_the_rows_change() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    state.handle(Key::Char('f'));
    walk(&mut state, Key::Down, 2);
    assert_eq!(state.list().expect("open").at(), Some(2), "the third row");

    type_into(&mut state, "bla");
    assert_eq!(
        state.list().expect("open").at(),
        Some(0),
        "the first of the four rows bla leaves"
    );
}

/// §4 rule 3: `up` and `down` stay at the ends; they do not wrap.
#[test]
fn the_highlight_stops_at_the_ends() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    state.handle(Key::Char('f'));

    assert_eq!(state.handle(Key::Up), Action::Continue);
    assert_eq!(
        state.list().expect("open").at(),
        Some(0),
        "up at the top stays"
    );

    state.handle(Key::End);
    walk(&mut state, Key::Down, 1);
    assert_eq!(
        state.list().expect("open").at(),
        Some(49),
        "down at the last row stays"
    );
}

/// §4 rule 7: what is typed into the filter is what the status line says, on the
/// row the status line holds at this size (8k1 §4 rule 1).
#[test]
fn the_status_line_shows_the_filter() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    state.handle(Key::Char('f'));
    type_into(&mut state, "tet");

    // The frame takes rows 0 to height - 2 and the key line the last row, so the
    // status line is the frame's inner last row: 21 at 80 × 24.
    let drawn: Vec<String> = rows(&state);
    assert!(drawn[21].contains("tet"), "status row: {}", drawn[21]);
}

/// 8k1 §4 rule 10, read with 8k3 §4 rule 3: below 80 × 24 the list draws nothing
/// and no key reaches it, so `q` is not filter text there — it quits.
#[test]
fn q_quits_under_the_size_notice() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    state.handle(Key::Char('f'));
    assert_eq!(state.list().expect("f opens a list").filter(), "");

    // One column narrower than the minimum: the screen shows the size notice and
    // nothing of the list, which is still open behind it.
    state.resize(W - 1, H);
    assert_eq!(
        state.handle(Key::Char('j')),
        Action::Continue,
        "no key but q and ctrl-c does anything here"
    );
    assert_eq!(
        state.list().expect("still open").filter(),
        "",
        "and it changes nothing"
    );
    assert_eq!(state.handle(Key::Char('q')), Action::Quit, "q quits");
}
