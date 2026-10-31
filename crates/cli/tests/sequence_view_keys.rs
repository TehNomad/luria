//! `F` cycles the feature lanes through their levels — all, CDSs only, none
//! (8m6 §4 rule 5) — and `?` lists every key the open view has (contract 8m3 §5,
//! M8-23, M8-25, M8-26).
//!
//! `pbr` is `tests/fixtures/genbank/raw/J01749.1.gb`, whose `CIRCULUS` line states
//! a circular topology, and `w16` is the FASTA `>w` / `TTCAAAAAAAAAAGAA`, whose
//! topology the view has to assume. Both draw at 80 x 24, which leaves 20 body
//! rows plus the overview strip's two when the strip is drawn: `pbr` opens with
//! the strip on, so its body is screen rows 3-20, and a list is drawn with no
//! strip, so its rows are read from screen rows 1-20.
//!
//! `tet` is the lane the fixture's `gene="tet"` names: the file puts that gene at
//! one-based base 86, which is zero-based 85 and falls in the second block of the
//! 72-column blocks pBR322 lays out at this size — the block drawn below the first
//! at this size, where the strip's two rows put the first block's lines at rows
//! 3-7. That the lane is drawn, and where it is drawn, is read from the fixture's
//! own feature table, not from what the screen draws.

use luria_cli::sequence_view::layout::FeatureLevel;
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

/// Every test draws at this size: 80 columns and 24 rows, whose frame has 78
/// inner columns, whose body is 20 rows plus the strip's two when drawn, whose
/// status line is row 21 and whose keys are row 23.
const W: u16 = 80;
const H: u16 = 24;

/// The body rows while the overview strip is drawn, which is how `pbr` opens
/// (8l2 §4 rule 1): the frame's inner rows less the strip's two and the status
/// line's one.
const BODY_WITH_STRIP: std::ops::Range<usize> = 3..21;

/// The body rows while the strip is not drawn, which is how a list is drawn
/// (8m3 §4 rule 2): the frame's inner rows less the status line's one.
const BODY_WITHOUT: std::ops::Range<usize> = 1..21;

/// The keys of the open view, in the order the key list has them (9m §4 rule 1),
/// as each row's own leading text: the five section headings and the keys under
/// them. `T` is not one of them here because pBR322 states its own topology, and
/// `ctrl-d`, `ctrl-u` and `esc` are the two half-screen keys, the key that clears
/// the hits and the key that shows the open reading frames (8m15 §3 item 2, M8-38,
/// M8-37, M8-40).
const KEYS: [&str; 39] = [
    "Moving",
    "hjkl",
    "PgUp PgDn",
    "ctrl-d ctrl-u",
    "Home End",
    "w b",
    "g",
    "Viewing",
    "t",
    "F",
    "C",
    "f",
    "c",
    "o",
    "O",
    "Searching",
    "/",
    "n N",
    "esc",
    "Editing",
    "v",
    "i a",
    "R",
    "r",
    "backspace",
    "x",
    "d",
    "y",
    "p P",
    "u ctrl-r",
    "Saving and quitting",
    ":w",
    ":w NAME",
    ":q :q!",
    ":wq :x ZZ",
    "ZQ",
    ":N",
    "q",
    "?",
];

/// The section headings, which are rows of their own (9m §4 rule 1): a heading's
/// drawn row is the heading word alone, where every other row is its key text, two
/// spaces and a description.
const HEADINGS: [&str; 5] = [
    "Moving",
    "Viewing",
    "Searching",
    "Editing",
    "Saving and quitting",
];

/// pBR322: 4361 bp, circular by its own declaration, three CDSs of which two are
/// usable — `tet` on the plus strand and `bla` on the minus strand — and no CDS
/// selected on open, which is why a block holds the lanes, the two strands and
/// nothing else.
fn pbr() -> ViewRecord {
    let path: &str = "../../tests/fixtures/genbank/raw/J01749.1.gb";
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text: String =
        fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"));
    open_record(Identifier::create("SYNPBR322".to_string()).unwrap(), text)
        .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
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

/// The cells between the frame's borders on one screen row, trimmed at both ends:
/// what §5 calls one body row.
fn body_row(state: &ViewState, row: usize) -> String {
    let line: Vec<char> = rows(state)[row].chars().collect();
    let right: usize = usize::from(W) - 1;
    line[1..right].iter().collect::<String>().trim().to_string()
}

/// Every non-blank body row in `given`, in screen order: what the body holds, and
/// nothing else. A row of a list is one of these (§5), and the rows read for the
/// blocks are the same rows.
fn body_rows(state: &ViewState, given: std::ops::Range<usize>) -> Vec<String> {
    given
        .map(|row| body_row(state, row))
        .filter(|row| !row.is_empty())
        .collect()
}

/// Whether any body row in `given` names the `tet` lane, which only a block
/// drawn on screen can do.
fn tet_drawn(state: &ViewState, given: std::ops::Range<usize>) -> bool {
    body_rows(state, given)
        .iter()
        .any(|row| row.contains("tet"))
}

/// `F` cycles the feature lanes through their three levels (M8-30, 8m6 §4 rule
/// 5: all, CDSs only, none), and the rows the levels leave are the rows the
/// blocks have. `tet` is a `gene` and a `CDS` at 86..1276, so its lane is drawn
/// at the first two levels and not at the last.
#[test]
fn f_cycles_the_feature_levels() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert_eq!(
        state.features(),
        FeatureLevel::All,
        "the levels open at all the features"
    );
    assert!(state.lanes_shown(), "the lanes are drawn at open");
    assert!(
        state.overview(),
        "pBR322 opens with the strip on, which is what §5 takes its body rows from"
    );
    assert!(
        tet_drawn(&state, BODY_WITH_STRIP),
        "the `tet` lane of block 1 is drawn on open"
    );
    let drawn: usize = state.visible().len();

    press(&mut state, Key::Char('F'));
    assert_eq!(
        state.features(),
        FeatureLevel::Cdss,
        "the first `F` moves the levels to the CDSs only"
    );
    assert!(state.lanes_shown(), "the CDSs still have their lanes");
    assert!(
        tet_drawn(&state, BODY_WITH_STRIP),
        "the `tet` lane, a `CDS` at 86..1276, is still drawn"
    );

    press(&mut state, Key::Char('F'));
    assert_eq!(
        state.features(),
        FeatureLevel::None,
        "the second `F` moves the levels to no features"
    );
    assert!(!state.lanes_shown(), "no features means no lanes");
    assert!(
        !tet_drawn(&state, BODY_WITH_STRIP),
        "no lane line is drawn while the levels are at none"
    );
    let hidden: usize = state.visible().len();
    assert!(
        hidden > drawn,
        "the rows the lanes took are rows the blocks have: {hidden} blocks drawn \
         against {drawn} before"
    );

    press(&mut state, Key::Char('F'));
    assert_eq!(
        state.features(),
        FeatureLevel::All,
        "the third `F` comes back round to all the features"
    );
    assert!(state.lanes_shown(), "the lanes are drawn again");
    assert!(
        tet_drawn(&state, BODY_WITH_STRIP),
        "the `tet` lane is drawn again"
    );
}

/// `T` has a row only for a record whose topology the view assumed (M8-25, §5
/// test 2), and pBR322 states its own.
#[test]
fn t_is_listed_only_for_an_assumed_topology() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    press(&mut state, Key::Char('?'));
    assert!(state.help(), "? opens the key list");
    let listed: Vec<String> = body_rows(&state, BODY_WITHOUT)
        .into_iter()
        .filter(|row| row.starts_with("T "))
        .collect();
    assert!(
        listed.is_empty(),
        "pBR322 states its own topology, so no row starts with `T `: {listed:?}"
    );

    let mut fasta: ViewState = ViewState::new(w16(), W, H);
    press(&mut fasta, Key::Char('?'));
    assert!(fasta.help(), "? opens the key list");
    let listed: Vec<String> = body_rows(&fasta, BODY_WITHOUT)
        .into_iter()
        .filter(|row| row.starts_with("T "))
        .collect();
    assert_eq!(
        listed.len(),
        1,
        "one row starts with `T ` for a topology the view assumed: {listed:?}"
    );
    assert!(
        listed[0].chars().count() > "T ".chars().count(),
        "the `T` row says what the key does: {:?}",
        listed[0]
    );
}

/// Whether the drawn rows are `entries`, one row per entry and in order (9m §4
/// rule 1): a heading's row is the heading word alone, and every other row is its
/// key text, two spaces and a description that is not empty.
fn rows_match(drawn: &[String], entries: &[&str]) {
    assert_eq!(
        drawn.len(),
        entries.len(),
        "one row per entry and nothing else: {drawn:?}"
    );
    for (row, entry) in drawn.iter().zip(entries) {
        if HEADINGS.contains(entry) {
            assert_eq!(row, entry, "a heading row is the heading word alone: {row}");
        } else {
            let prefix: String = format!("{entry}  ");
            assert!(
                row.starts_with(&prefix) && row.chars().count() > prefix.chars().count(),
                "the row for {entry} is its key text, two spaces and a description: {row}"
            );
        }
    }
}

/// `?` opens the key list, which takes the body and keeps the status line and the
/// key line (9m §5 test 1): the list's rows, grouped under their headings, show a
/// window at a time from the top, `j` `k` `PgUp` `PgDn` `Home` `End` scroll that
/// window and are clamped at both ends, and the status line says which rows show
/// and how many the list has. `q` closes the list and does not quit, and the key
/// line then reads as §4 rule 3.
#[test]
fn question_mark_lists_every_key() {
    /// The rows the body has at this size, which is the window the list shows
    /// (9m §4 rule 2).
    const WINDOW: usize = 20;
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    press(&mut state, Key::Char('?'));
    assert!(state.help(), "? opens the key list at the top");
    assert_eq!(
        body_row(&state, 21),
        format!("keys 1-{WINDOW} of {}", KEYS.len()),
        "the status line names the rows drawn and the whole list (9m §4 rule 4)"
    );
    let top: Vec<String> = body_rows(&state, BODY_WITHOUT.clone());
    assert_eq!(
        top.len(),
        WINDOW,
        "the body holds the window and nothing else: {top:?}"
    );
    rows_match(&top, &KEYS[..WINDOW]);
    assert!(
        !tet_drawn(&state, BODY_WITHOUT.clone()),
        "the list takes the body, so no block is drawn and no lane line is"
    );
    assert!(
        rows(&state)[23].contains("jk PgUp PgDn Home End  scroll  esc ? q  close"),
        "the key line names what scrolls the list and what closes it: {}",
        rows(&state)[23].trim_end()
    );

    // `End` is the last full window, whose last row is the list's last row, and
    // every clamp that follows keeps it there (9m §4 rule 3).
    press(&mut state, Key::End);
    assert!(state.help(), "`End` scrolls the list, it does not close it");
    let end: Vec<String> = body_rows(&state, BODY_WITHOUT.clone());
    assert_eq!(end.len(), WINDOW, "the last window is a full one: {end:?}");
    rows_match(&end, &KEYS[KEYS.len() - WINDOW..]);
    assert_eq!(
        body_row(&state, 21),
        format!(
            "keys {}-{} of {}",
            KEYS.len() - WINDOW + 1,
            KEYS.len(),
            KEYS.len()
        ),
        "`End` shows the last full window, where the list's last row is the body's"
    );
    press(&mut state, Key::Char('j'));
    assert_eq!(
        body_rows(&state, BODY_WITHOUT.clone()),
        end,
        "`j` at the end of the list scrolls nothing"
    );
    assert_eq!(
        body_row(&state, 21),
        format!(
            "keys {}-{} of {}",
            KEYS.len() - WINDOW + 1,
            KEYS.len(),
            KEYS.len()
        ),
        "and the status line stays with it"
    );
    press(&mut state, Key::Home);
    assert_eq!(
        body_rows(&state, BODY_WITHOUT.clone()),
        top,
        "`Home` goes back to the top"
    );
    assert_eq!(
        body_row(&state, 21),
        format!("keys 1-{WINDOW} of {}", KEYS.len()),
        "and the status line goes with the rows it shows"
    );

    // `?` closes the list, and opens it at the top again even though it closed at
    // the last window (9m §4 rules 2 and 3).
    press(&mut state, Key::End);
    press(&mut state, Key::Char('?'));
    assert!(!state.help(), "`?` closes the list");
    press(&mut state, Key::Char('?'));
    assert!(state.help(), "`?` opens it again");
    assert_eq!(
        body_row(&state, 21),
        format!("keys 1-{WINDOW} of {}", KEYS.len()),
        "and opens it at the top, though it closed at the end (9m §4 rule 2)"
    );
    assert_eq!(
        state.handle(Key::Char('q')),
        Action::Continue,
        "q closes the list and does not quit"
    );
    assert!(!state.help(), "q closed the list");
    let keys: String = rows(&state)[23].trim_end().to_string();
    assert_eq!(
        keys, "hjkl PgUp PgDn Home End  t frames  F lanes  f c g  / n N  ? keys  q quit",
        "the key line keeps the most-used keys and offers the list (M8-26)"
    );
    assert_eq!(keys.chars().count(), 72, "the line §4 rule 3 spells");
}
