//! Where the cursor lands when a pick or a go-to line finishes (contract 8m1
//! §5, M8-21, M8-22): a CDS picked from the `c` list goes to its start codon,
//! `f` + `enter` goes to a feature's five prime end, and a residue typed with no
//! CDS selected opens the CDS picker and then goes to that residue.
//!
//! Every record is a fixture read by `open_record` at 80 x 24 with no CDS selected
//! — pBR322 (`tests/fixtures/genbank/raw/J01749.1.gb`) has two usable CDSs, so
//! M8-11 selects none (M8-21 and M8-22 say what its rows mean: `tet` is CDS 0 at
//! `86..1276`, ROP is CDS 1 and is refused for having no id, and `bla` is CDS 2 at
//! `complement(3293..4153)`), and `tests/fixtures/view/frames-62.gb` has the
//! origin-crossing CDSs `join(56..62,1..11)` and `complement(join(53..62,1..8))`.
//! Every number comes off the fixture or off the `/translation` qualifier: they are
//! zero-based (D006) and, for a minus strand CDS, in reading direction, so the
//! start codon of `bla` is base 4153 as the record writes it. The 30 bp record of
//! test 3 is written in the test, because only its written ends matter.
//!
//! Keys are driven the way the terminal drives them, through `handle`: `Home` and
//! `Down` move the list's highlight, so nothing depends on where a list opens.

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

/// Every test draws at this size: 80 columns and 24 rows (8k1 §4 rule 1).
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

/// pBR322: 4361 bp, circular, three CDSs of which two are usable, so M8-11 selects
/// none on open and its picker rows are `[0, 1, 2]`.
fn pbr() -> ViewRecord {
    record("../../tests/fixtures/genbank/raw/J01749.1.gb", "SYNPBR322")
}

/// The 62 bp circular record: `fP` is `join(56..62,1..11)` and `fM` is
/// `complement(join(53..62,1..8))`, both usable and both crossing the origin.
fn f62() -> ViewRecord {
    record("../../tests/fixtures/view/frames-62.gb", "SYNFRAMES62")
}

/// The 30 bp linear record of test 3: one minus strand `misc_feature` whose three
/// prime end is written open, and one whose five prime end is. Both are usable as
/// drawn shapes, so the list offers both rows.
fn open_ends() -> ViewRecord {
    let text: &str = r#"LOCUS       openends                  30 bp    DNA     linear   SYN 27-OCT-2026
FEATURES             Location/Qualifiers
     source          1..30
                     /organism="synthetic construct"
                     /mol_type="other DNA"
     misc_feature    complement(<5..10)
                     /gene="a"
     misc_feature    complement(5..>20)
                     /gene="b"
ORIGIN
        1 acgtacgtac gtacgtacgt acgtacgtac
//
"#;
    open_record(
        Identifier::create("openends".to_string()).unwrap(),
        text.to_string(),
    )
    .unwrap_or_else(|error| panic!("open the 30 bp record: {error:?}"))
}

/// The 16 bp FASTA of 8h §5: no features and no CDSs at all.
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
        "key {key:?} does not leave"
    );
}

/// `text` typed one character per key (8k2 §4 rule 2).
fn type_into(state: &mut ViewState, text: &str) {
    for ch in text.chars() {
        press(state, Key::Char(ch));
    }
}

/// "`g`, this text, `enter`" (8k2 §4 rule 2).
fn go_to(state: &mut ViewState, text: &str) {
    press(state, Key::Char('g'));
    type_into(state, text);
    press(state, Key::Enter);
}

/// `key` pressed `n` times.
fn walk(state: &mut ViewState, key: Key, n: usize) {
    for _ in 0..n {
        press(state, key);
    }
}

/// The block holding `base`, which is the block `visible` has to draw after a jump.
fn block_holding(state: &ViewState, base: u64) -> usize {
    let built = state.layout().expect("80 x 24 lays the record out");
    built
        .blocks
        .iter()
        .position(|(first, last)| *first <= base && base < *last)
        .unwrap_or_else(|| panic!("no block holds base {base}"))
}

/// The cursor's block is among the blocks drawn: the scroll follows every jump
/// (8k1 §4 rule 6, 8m1 §4 rule 5).
fn cursor_block_is_drawn(state: &ViewState) {
    let block: usize = block_holding(state, state.cursor());
    let drawn: std::ops::Range<usize> = state.visible();
    assert!(
        drawn.contains(&block),
        "the cursor's block {block} is not drawn in {drawn:?}"
    );
}

/// The screen drawn on a test backend, as `tests/sequence_view.rs` reads it. The
/// status line is what the refusals are shown on, so the rows hold it too.
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

/// M8-21 and 8m1 §4 rule 1: `enter` on a usable CDS row selects it, closes the
/// list, and puts the cursor on the first base of its start codon, in reading
/// direction; `tet` is on the plus strand, so that is its lowest base.
#[test]
fn a_cds_pick_goes_to_its_start_codon() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert_eq!(
        state.selected(),
        None,
        "two usable CDSs, so M8-11 selects none"
    );

    press(&mut state, Key::Char('c'));
    assert_eq!(
        state.list().expect("the CDS list opens").rows(),
        [0, 1, 2],
        "every CDS the record has"
    );
    press(&mut state, Key::Home);
    walk(&mut state, Key::Down, 2);
    assert_eq!(
        state.list().expect("still open").at(),
        Some(2),
        "the bla CDS row"
    );
    press(&mut state, Key::Enter);
    assert_eq!(state.selected(), Some(2), "the pick selects bla");
    assert!(state.list().is_none(), "and closes the list");
    assert_eq!(
        state.cursor(),
        4152,
        "complement(3293..4153) starts at its highest base, zero-based 4152"
    );
    cursor_block_is_drawn(&state);

    press(&mut state, Key::Char('c'));
    press(&mut state, Key::Home);
    press(&mut state, Key::Enter);
    assert_eq!(state.selected(), Some(0), "tet is the file's first CDS");
    assert_eq!(
        state.cursor(),
        85,
        "tet is 86..1276 as the file writes it, so its codon starts at 85",
    );
    cursor_block_is_drawn(&state);
}

/// M8-21 and 8m1 §4 rule 2: `enter` on a feature row goes to its five prime end —
/// the first base of its first interval on the plus strand, the last base of its
/// last interval on the minus strand, the intervals in D036 order.
#[test]
fn a_minus_strand_feature_goes_to_its_five_prime_end() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    press(&mut state, Key::Char('f'));
    type_into(&mut state, "bla");
    walk(&mut state, Key::Down, 1);
    press(&mut state, Key::Enter);
    assert_eq!(
        state.cursor(),
        4152,
        "the bla CDS is complement(3293..4153), so its five prime end is 4152"
    );

    let mut frames: ViewState = ViewState::new(f62(), W, H);
    press(&mut frames, Key::Char('f'));
    type_into(&mut frames, "fM");
    press(&mut frames, Key::Enter);
    assert_eq!(
        frames.cursor(),
        7,
        "complement(join(53..62,1..8)) ends at its last written base, zero-based 7"
    );
    press(&mut frames, Key::Char('f'));
    type_into(&mut frames, "fP");
    press(&mut frames, Key::Enter);
    assert_eq!(
        frames.cursor(),
        55,
        "join(56..62,1..11) starts at its first written base, zero-based 55"
    );
}

/// M8-21 and 8m1 §4 rule 2: the open end of the written location is the unstated
/// five prime end, so `<` on the plus strand and `>` on the minus strand are
/// refused, and the other marker says nothing about this end.
#[test]
fn an_open_five_prime_end_is_not_jumped_to() {
    let mut state: ViewState = ViewState::new(open_ends(), W, H);
    assert_eq!(
        state.selected(),
        None,
        "a record of two misc_features has no CDS"
    );

    // `complement(<5..10)` writes its three prime end open: the five prime end,
    // base 10 as the record writes it, is stated and jumped to.
    press(&mut state, Key::Char('f'));
    type_into(&mut state, "a");
    press(&mut state, Key::Enter);
    assert_eq!(state.cursor(), 9, "complement(5..10) ends at zero-based 9");
    assert!(state.list().is_none(), "the jump closes the list");
    assert!(
        state.message().is_none(),
        "and a jump leaves no refusal: {:?}",
        state.message()
    );

    // `complement(5..>20)` writes its five prime end open: there is no first base
    // to stand on, so nothing moves and the list stays.
    press(&mut state, Key::Char('f'));
    type_into(&mut state, "b");
    assert_eq!(
        state.list().expect("the list is open").at(),
        Some(0),
        "the one row the filter leaves is highlighted"
    );
    press(&mut state, Key::Enter);
    assert_eq!(state.cursor(), 9, "an unstated start is not jumped to");
    assert!(state.list().is_some(), "and the list stays open");
    assert!(
        state.message().is_some(),
        "and the status line says why: {:?}",
        state.message()
    );
    let shown: Vec<String> = rows(&state);
    assert!(
        shown.iter().any(|line| line.contains("start")),
        "the refusal reaches the status line: {shown:?}"
    );
}

/// M8-22 and 8m1 §4 rule 3: a residue typed with no CDS selected opens the CDS
/// picker, and `enter` on a usable row picks it and goes to the residue as 8k2 does.
#[test]
fn a_residue_with_no_cds_opens_the_picker() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    go_to(&mut state, "M1");
    assert_eq!(
        state.cursor(),
        0,
        "the residue that opens a picker moves nothing"
    );
    assert_eq!(state.selected(), None, "and selects nothing yet");
    let list = state.list().expect("go-to opens the picker");
    assert_eq!(list.kind(), ListKind::Cdss, "the CDS picker");
    assert_eq!(list.filter(), "", "with no filter");
    assert!(state.prompt().is_none(), "and the go-to prompt has closed");

    press(&mut state, Key::Home);
    walk(&mut state, Key::Down, 2);
    press(&mut state, Key::Enter);
    assert_eq!(state.selected(), Some(2), "the pick selects bla");
    assert!(state.list().is_none(), "and closes the list");
    assert_eq!(
        state.cursor(),
        4152,
        "residue 1 of complement(3293..4153) goes to its codon's first base"
    );
    cursor_block_is_drawn(&state);
}

/// M8-22 and 8m1 §4 rule 3: a residue this CDS refuses still selects it, still
/// closes the list, and leaves the cursor on the start codon with 8k2's refusal in
/// the status line; and `esc` drops the residue, so a later pick follows M8-21.
#[test]
fn a_refused_residue_still_picks() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    go_to(&mut state, "S1");
    assert_eq!(state.selected(), None, "the residue is waiting for a CDS");
    press(&mut state, Key::Home);
    press(&mut state, Key::Enter);
    assert_eq!(state.selected(), Some(0), "the pick selects tet");
    assert!(state.list().is_none(), "and closes the list");
    assert_eq!(
        state.cursor(),
        85,
        "the refused residue leaves the cursor on tet's start codon"
    );
    assert!(
        state.message().is_some_and(|words| words.contains('M')),
        "tet's residue 1 is M, and the refusal says so: {:?}",
        state.message()
    );

    // The residue is forgotten with the list, so the next pick follows M8-21.
    let mut fresh: ViewState = ViewState::new(pbr(), W, H);
    go_to(&mut fresh, "M1");
    assert!(fresh.list().is_some(), "the picker is open");
    assert_eq!(fresh.handle(Key::Esc), Action::Continue);
    assert!(fresh.list().is_none(), "esc closes it");
    assert_eq!(fresh.selected(), None, "and selects nothing");
    assert_eq!(fresh.cursor(), 0, "and moves nothing");

    press(&mut fresh, Key::Char('c'));
    press(&mut fresh, Key::Home);
    walk(&mut fresh, Key::Down, 2);
    press(&mut fresh, Key::Enter);
    assert_eq!(
        fresh.cursor(),
        4152,
        "the pick goes to bla's start codon, not to residue 1",
    );
}

/// M8-22 and 8m1 §4 rule 4: a record with no CDS at all opens nothing, moves
/// nothing, and says the record has no CDS.
#[test]
fn a_residue_in_a_record_with_no_cds() {
    let mut state: ViewState = ViewState::new(w16(), W, H);
    assert_eq!(state.selected(), None, "a FASTA record has no CDS");
    go_to(&mut state, "M1");
    assert!(state.list().is_none(), "and no list opens");
    assert_eq!(state.cursor(), 0, "nothing moves");
    assert!(
        state.message().is_some(),
        "and the status line says the record has no CDS: {:?}",
        state.message()
    );
}
