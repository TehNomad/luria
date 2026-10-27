//! Go-to as the view holds it and draws it (contract 8k2 §5; plan §5.3, D055,
//! D056, D057). Every record is a fixture read by `open_record`, every size is
//! 80 × 24, and every number comes off the fixture or off the table that records
//! it: pBR322 (`tests/fixtures/genbank/raw/J01749.1.gb`) is 4361 bp, its
//! `mobile_element` feature is 3148..4361 and its ROP CDS is 1915..2106 and is
//! refused for having no id, and its four `bla` features are rows 37 to 40 in
//! `tests/fixtures/genbank/pbr322-features.tsv`; `tests/fixtures/genbank/
//! synthetic-linear.gb` has one usable CDS, `synA`, whose residues M1, A42 and W87
//! sit on codons 42..45, 165..168 and 300..303 as
//! `tests/fixtures/genbank/synthetic-linear-residues.tsv` gives them (zero-based
//! half-open); and `tests/fixtures/genbank/synthetic.gb` feature 2 is
//! `ignored-gene` with `/locus_tag="m1"` at `complement(13..24)` and a usable CDS,
//! while features 17 and 18 are the `fuzzy` and `ord` CDSs that the display parser
//! refuses, as `tests/fixtures/genbank/synthetic-unused.tsv` records.
//!
//! Go-to is driven the way the terminal drives it, through `handle` and its `Key`s
//! — `g`, the text typed one character per key, then `enter` — and what the screen
//! shows is read back off a `ratatui` test backend the way `tests/sequence_view.rs`
//! and `tests/sequence_view_list.rs` read the screen. `parse_target` itself is
//! tested in `crates/cli/src/sequence_view/goto.rs`.

use luria_cli::sequence_view::layout::Layout;
use luria_cli::sequence_view::list::ListKind;
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

/// Every test draws at this size: 80 columns and 24 rows, which leaves the body
/// 20 rows (`height - 4`) and the status line row 21 (8k1 §4 rule 1).
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

/// pBR322: 4361 bp, circular, 50 features and three CDSs of which two are usable
/// — so M8-11 selects none on open.
fn pbr() -> ViewRecord {
    record("../../tests/fixtures/genbank/raw/J01749.1.gb", "SYNPBR322")
}

/// synthetic-linear: linear, one usable CDS `synA`, which M8-11 therefore selects
/// on open.
fn lin() -> ViewRecord {
    record("../../tests/fixtures/genbank/synthetic-linear.gb", "SYNLIN")
}

/// synthetic: 60 bp, circular, 27 features and 24 CDSs, several of them usable, so
/// again nothing is selected on open.
fn syn() -> ViewRecord {
    record("../../tests/fixtures/genbank/synthetic.gb", "synthetic")
}

/// `text` typed one character per key (8k2 §4 rule 2).
fn type_into(state: &mut ViewState, text: &str) {
    for ch in text.chars() {
        assert_eq!(
            state.handle(Key::Char(ch)),
            Action::Continue,
            "typing {ch:?} does not leave"
        );
    }
}

/// "`g`, this text, `enter`" (8k2 §4 rule 2), with the text typed as the terminal
/// would type it.
fn go_to(state: &mut ViewState, text: &str) {
    assert_eq!(
        state.handle(Key::Char('g')),
        Action::Continue,
        "g opens the prompt"
    );
    type_into(state, text);
    assert_eq!(
        state.handle(Key::Enter),
        Action::Continue,
        "enter acts and the view goes on"
    );
}

/// The block holding the cursor's base is among the blocks drawn: go-to scrolls
/// to what it moved the cursor to (8k1 §4 rule 6, 8k2 §4 rule 3).
fn cursor_block_is_drawn(state: &ViewState) {
    let built: &Layout = state.layout().expect("80 x 24 lays the record out");
    let block: usize = built
        .blocks
        .iter()
        .position(|(first, last)| *first <= state.cursor() && state.cursor() < *last)
        .expect("the cursor's base lies in some block");
    let drawn: std::ops::Range<usize> = state.visible();
    assert!(
        drawn.contains(&block),
        "the cursor's block {block} is not drawn in {drawn:?}"
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

/// §5.3 and D057: digits alone are the 1-based base, whatever CDS is selected
/// (8k2 §4 rule 3).
#[test]
fn a_number_goes_to_that_base() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);

    go_to(&mut state, "1210");
    assert_eq!(state.cursor(), 1209, "base 1210 is zero-based 1209");
    assert_eq!(state.prompt(), None, "enter closed the prompt");
    cursor_block_is_drawn(&state);

    go_to(&mut state, "4361");
    assert_eq!(state.cursor(), 4360, "the record's last base");
    cursor_block_is_drawn(&state);

    go_to(&mut state, "0");
    assert_eq!(
        state.cursor(),
        4360,
        "base 0 is no base, so the cursor stays"
    );
    assert!(state.message().is_some(), "and the range is said");

    go_to(&mut state, "4362");
    assert_eq!(state.cursor(), 4360, "one past the last base stays too");
    assert!(state.message().is_some(), "and the range is said again");

    // A blank line is nothing: it closes the prompt and changes no cursor and no
    // message (8k2 §4 rule 3).
    assert_eq!(state.handle(Key::Char('g')), Action::Continue);
    assert_eq!(state.handle(Key::Enter), Action::Continue);
    assert_eq!(state.cursor(), 4360, "a blank line moves nothing");
    assert_eq!(state.message(), None, "and refuses nothing");

    // A selected CDS does not make a bare number a residue (D057).
    let mut selected: ViewState = ViewState::new(lin(), W, H);
    assert_eq!(
        selected.selected(),
        Some(0),
        "synA is the only CDS, so it opens selected"
    );
    go_to(&mut selected, "100");
    assert_eq!(selected.cursor(), 99, "100 is a base, not residue 100");
}

/// D055 and 8k2 §4 rule 4: a residue stands over its codon's middle base, in the
/// reading order of the strand its CDS is on (8k1 §4 rule 5).
#[test]
fn a_residue_goes_to_its_codon_middle() {
    let mut state: ViewState = ViewState::new(lin(), W, H);

    go_to(&mut state, "A42");
    assert_eq!(
        state.cursor(),
        166,
        "residue 42's codon is 165..168, so it stands over 166"
    );
    go_to(&mut state, "W87");
    assert_eq!(state.cursor(), 301, "residue 87's codon is 300..303");
    go_to(&mut state, "M1");
    assert_eq!(state.cursor(), 43, "residue 1's codon is 42..45");

    // bla is a minus-strand CDS: M1's codon is read 4152, 4151, 4150, and the
    // middle base of that reading order is 4151.
    let mut selected: ViewState = ViewState::new(pbr(), W, H);
    assert!(selected.select_cds(Some(2)), "bla's CDS is usable");
    go_to(&mut selected, "M1");
    assert_eq!(selected.cursor(), 4151, "the middle of 4152, 4151, 4150");
    go_to(&mut selected, "S2");
    assert_eq!(selected.cursor(), 4148, "and of 4149, 4148, 4147");
}

/// D057: the letter catches a number written for the wrong residue, and the
/// refusal names the residue that is actually there (8k2 §4 rule 4).
#[test]
fn a_wrong_letter_is_refused_with_the_actual_residue() {
    let mut state: ViewState = ViewState::new(lin(), W, H);
    assert_eq!(state.cursor(), 0, "the view opens on base 1");

    go_to(&mut state, "D42");
    assert_eq!(
        state.cursor(),
        0,
        "a residue letter that does not match moves nothing"
    );
    let words: &str = state.message().expect("the refusal is said");
    assert!(
        words.contains('4') && words.contains("42") && words.contains('A') && words.contains('D'),
        "the status line says which residue it is, what it is and what was typed: {words}"
    );
}

/// 8k2 §4 rule 4: a residue needs a selected CDS and a number with a residue of
/// its own, and the terminal stop has a codon and no residue (D010).
#[test]
fn a_residue_needs_a_cds_and_a_number_in_range() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert_eq!(
        state.selected(),
        None,
        "pBR322 has two usable CDSs, so it opens with none"
    );

    go_to(&mut state, "M1");
    assert_eq!(
        state.cursor(),
        0,
        "with no CDS selected a residue is nowhere"
    );
    assert!(state.message().is_some(), "and the status line says so");

    assert!(
        state.select_cds(Some(2)),
        "bla's CDS is usable, so it selects"
    );
    go_to(&mut state, "X287");
    assert_eq!(
        state.cursor(),
        0,
        "bla has 286 residues: 287 is the terminal stop, which has no residue"
    );
    assert!(state.message().is_some(), "and the range is said");

    go_to(&mut state, "X0");
    assert_eq!(state.cursor(), 0, "residue 0 is not a residue");
    assert!(state.message().is_some(), "and the range is said");
}

/// §5.3 and 8k2 §4 rule 5: a name that matches one feature goes to it exactly as
/// the feature list's `enter` does (8k3 §4 rule 4), taking its usable CDS with it
/// and refusing its unusable one (D042, D052).
#[test]
fn a_name_goes_to_its_feature() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);

    go_to(&mut state, "mobile");
    assert_eq!(
        state.cursor(),
        3147,
        "mobile_element is 3148..4361, so its first base is zero-based 3147"
    );
    cursor_block_is_drawn(&state);

    go_to(&mut state, "rop");
    assert_eq!(state.cursor(), 1914, "the ROP CDS feature is 1915..2106");
    assert_eq!(
        state.selected(),
        None,
        "and its CDS is refused, so nothing is selected"
    );
    assert!(
        state.message().is_some_and(|words| words.contains("no_id")),
        "with the code that refuses it: {:?}",
        state.message()
    );

    // A `/locus_tag` reaches a feature's item through core's matcher (D056), and a
    // usable CDS is selected with the jump (M8-11).
    let mut other: ViewState = ViewState::new(syn(), W, H);
    assert_eq!(other.selected(), None, "synthetic has several usable CDSs");
    go_to(&mut other, "m1");
    assert_eq!(
        other.cursor(),
        12,
        "complement(13..24) starts at zero-based 12"
    );
    assert_eq!(
        other.selected(),
        Some(1),
        "and feature 2's CDS is cdss entry 1"
    );
}

/// §5.3 and 8k2 §4 rule 5: several matches open the feature list filtered to the
/// text typed, and the cursor stays where it was.
#[test]
fn several_names_open_the_list() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);

    go_to(&mut state, "bla");
    let list = state.list().expect("four bla features, so the list opens");
    assert_eq!(list.kind(), ListKind::Features, "the feature list");
    assert_eq!(list.filter(), "bla", "filtered to what was typed");
    assert_eq!(
        list.rows(),
        [37, 38, 39, 40],
        "the gene, the CDS, the sig_peptide and the mat_peptide"
    );
    assert_eq!(
        state.cursor(),
        0,
        "and the cursor stays for the list's own keys"
    );
    assert_eq!(state.prompt(), None, "the prompt closed");
}

/// §5.3: a name that matches nothing leaves the cursor and says so, and a feature
/// whose location was not drawn is never jumped to (M8-1, D053).
#[test]
fn an_unknown_or_undrawn_name_does_not_move() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);

    go_to(&mut state, "zzz");
    assert_eq!(state.cursor(), 0, "no feature is named zzz");
    assert!(state.message().is_some(), "and the status line says so");
    assert!(state.list().is_none(), "and no list opens");

    // `order(1..6,7..12)` and the `>5` end are refused by the display parser, so
    // they are listed but have no first base to stand on.
    let mut other: ViewState = ViewState::new(syn(), W, H);
    go_to(&mut other, "ord");
    assert_eq!(other.cursor(), 0, "an undrawn feature is not jumped to");
    assert!(
        other.message().is_some_and(|words| words.contains("order")),
        "with the code that keeps it off the screen: {:?}",
        other.message()
    );
    go_to(&mut other, "fuzzy");
    assert_eq!(other.cursor(), 0, "and the same for a fuzzy end");
    assert!(other.message().is_some(), "said in the same way");
    assert!(other.list().is_none(), "and still no list");
}

/// 8k2 §4 rule 2: the prompt takes every printable key as text, `backspace`
/// shortens it, `esc` closes it and changes nothing else, and `ctrl-c` leaves.
/// Below 80 × 24 nothing is drawn and `q` quits, as 8k3 round 2 made it for a list.
#[test]
fn keys_go_to_the_prompt() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);

    assert_eq!(state.handle(Key::Char('g')), Action::Continue);
    assert_eq!(
        state.prompt(),
        Some(""),
        "g opens the prompt with nothing typed"
    );
    assert_eq!(
        state.handle(Key::Char('j')),
        Action::Continue,
        "hjkl are text here, not movement"
    );
    assert_eq!(
        state.handle(Key::Char('q')),
        Action::Continue,
        "q is text here"
    );
    assert_eq!(state.prompt(), Some("jq"), "every character is kept");
    assert_eq!(state.cursor(), 0, "and none of them moves the cursor");

    assert_eq!(state.handle(Key::Backspace), Action::Continue);
    assert_eq!(
        state.prompt(),
        Some("j"),
        "backspace drops the last character"
    );

    assert_eq!(state.handle(Key::Esc), Action::Continue);
    assert_eq!(state.prompt(), None, "esc closes the prompt");
    assert_eq!(state.cursor(), 0, "and changes nothing else");
    assert_eq!(state.message(), None, "not even the status line");

    assert_eq!(state.handle(Key::Char('g')), Action::Continue);
    assert_eq!(
        state.handle(Key::CtrlC),
        Action::Quit,
        "ctrl-c quits from anywhere"
    );

    // One column narrower than the minimum: the screen shows the size notice and
    // nothing of the prompt, which is still open behind it, so `q` is not text.
    let mut small: ViewState = ViewState::new(pbr(), W, H);
    assert_eq!(small.handle(Key::Char('g')), Action::Continue);
    assert_eq!(small.handle(Key::Char('1')), Action::Continue);
    small.resize(W - 1, H);
    assert_eq!(
        small.handle(Key::Char('q')),
        Action::Quit,
        "q quits below the size"
    );
}

/// 8k2 §4 rule 2: while the prompt is open the status line shows the text typed,
/// on the row the status line holds at this size (8k1 §4 rule 1).
#[test]
fn the_status_line_shows_the_prompt() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    state.handle(Key::Char('g'));
    type_into(&mut state, "12");

    // The frame takes rows 0 to height - 2 and the key line the last row, so the
    // status line is the frame's inner last row: 21 at 80 × 24.
    let drawn: Vec<String> = rows(&state);
    assert!(drawn[21].contains("12"), "status row: {}", drawn[21]);
}
