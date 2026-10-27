//! Which row of a block a hit's yellow marks: the strand the hit was found on
//! (contract 8m2 §5, M8-24, changing 8l1 §4 rule 8).
//!
//! `pbr` is `tests/fixtures/genbank/raw/J01749.1.gb` at 80 x 24, which opens with
//! the overview strip on so that block 0's top-strand row is screen row 8 and its
//! bottom-strand row is row 9, and a base `p` of block 0 sits at column `6 + p`
//! as `tests/sequence_view_search.rs::every_shown_hit_is_marked` reads them.
//!
//! The pattern the first two tests search is the run of eight bases at 10:
//! zero-based bases `[10, 18)` of the fixture read `TGACAGCT` on the top strand,
//! that pattern occurs nowhere else on either strand, and its reverse complement
//! `AGCTGTCA` occurs on the top strand nowhere. The planner read those eight
//! bases out of the file with an independent script and derived the two searches
//! from them; the hit's expected start and length are the run's own, 10 with
//! length 8, and its strand is what a pattern that only matches the complement
//! can be found on.
//!
//! The protein test's values come the same way: `MSIQ` is `bla`'s first four
//! residues from its `/translation`, read on the minus strand, so its twelve
//! bases end where `bla`'s codons do — the same run of bases as the fixture row
//! `pbr322 bla MSIQHF 4135 18` in `tests/fixtures/view/search-protein-hits.tsv`,
//! cut short at its left-hand end by six bases — and `MKSNNAL` is `tet`'s first
//! seven residues, read on the plus strand from base 85, where `tet` starts.

use luria_cli::sequence_view::screen::draw;
use luria_cli::sequence_view::search::Hits;
use luria_cli::sequence_view::search::SearchMode;
use luria_cli::sequence_view::state::Action;
use luria_cli::sequence_view::state::Key;
use luria_cli::sequence_view::state::ViewState;
use luria_core::identity::Identifier;
use luria_core::search::HitStrand;
use luria_core::view_record::open_record;
use luria_core::view_record::ViewRecord;
use ratatui::backend::TestBackend;
use ratatui::style::Color;
use ratatui::Terminal;
use std::fs;
use std::path::Path;

/// Every test draws at this size, as `tests/sequence_view_search.rs` does: 80
/// columns and 24 rows, which leaves the body 20 rows and puts the status line on
/// row 21 and the keys on row 23.
const W: u16 = 80;
const H: u16 = 24;

/// pBR322: 4361 bp, circular by its own declaration, three CDSs of which two are
/// usable — `tet` on the plus strand and `bla` on the minus strand — and no CDS
/// selected on open.
fn pbr() -> ViewRecord {
    let path: &str = "../../tests/fixtures/genbank/raw/J01749.1.gb";
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text: String =
        fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"));
    open_record(Identifier::create("SYNPBR322".to_string()).unwrap(), text)
        .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
}

/// `text` typed one character per key (8l1 §4 rule 2).
fn type_into(state: &mut ViewState, text: &str) {
    for ch in text.chars() {
        assert_eq!(
            state.handle(Key::Char(ch)),
            Action::Continue,
            "typing {ch:?} does not leave"
        );
    }
}

/// "`/`, this text, `enter`", with the text typed as the terminal would type it.
fn search(state: &mut ViewState, text: &str) {
    assert_eq!(
        state.handle(Key::Char('/')),
        Action::Continue,
        "/ opens the prompt"
    );
    type_into(state, text);
    assert_eq!(
        state.handle(Key::Enter),
        Action::Continue,
        "enter searches and the view goes on"
    );
}

/// `key` pressed `times` times.
fn press(state: &mut ViewState, key: Key, times: usize) {
    for _ in 0..times {
        assert_eq!(
            state.handle(key),
            Action::Continue,
            "{key:?} does not leave"
        );
    }
}

/// The hits of the search now on screen, or a failure saying there are none.
fn hits(state: &ViewState) -> &Hits {
    state.hits().expect("a search marked hits")
}

/// "`/`, this text, `enter`" in the mode the letters ask for: the prompt is read
/// as protein with no `tab`, because `text` holds a letter that cannot be a
/// nucleotide letter (8l1 §4 rule 2, contract §5).
fn search_as_protein(state: &mut ViewState, text: &str) {
    assert_eq!(
        state.handle(Key::Char('/')),
        Action::Continue,
        "/ opens the prompt"
    );
    type_into(state, text);
    assert_eq!(
        state.search().expect("the prompt is open").mode(),
        SearchMode::Aa,
        "{text} cannot be a nucleotide pattern"
    );
    assert_eq!(
        state.handle(Key::Enter),
        Action::Continue,
        "enter searches and the view goes on"
    );
}

/// Where on screen a base is marked yellow: `(row, column)` of every cell whose
/// background is `Color::Yellow`, in screen order.
fn yellow(state: &ViewState) -> Vec<(usize, usize)> {
    let mut terminal = Terminal::new(TestBackend::new(W, H)).unwrap();
    terminal.draw(|frame| draw(frame, state)).unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = usize::from(buffer.area.width);
    let mut found: Vec<(usize, usize)> = Vec::new();
    for (row, cells) in buffer.content.chunks(width).enumerate() {
        for (column, cell) in cells.iter().enumerate() {
            if cell.bg == Color::Yellow {
                found.push((row, column));
            }
        }
    }
    found
}

/// The yellow cells of the eight bases at 10, on the given row: block 0 holds
/// them, and base `10 + i` sits at column `6 + 10 + i`.
fn cells_of(row: usize) -> Vec<(usize, usize)> {
    (16..=23).map(|column| (row, column)).collect()
}

/// §5.1: a pattern that matches only the complement is found on the minus strand,
/// and its bases go yellow on the bottom row of the block, never the top row.
#[test]
fn a_minus_hit_is_marked_on_the_bottom_strand() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    search(&mut state, "AGCTGTCA");
    assert_eq!(
        hits(&state).shown().len(),
        1,
        "AGCTGTCA occurs on the top strand nowhere"
    );
    assert_eq!(hits(&state).shown()[0].start, 10, "the run at 10");
    assert_eq!(hits(&state).shown()[0].length, 8, "eight bases");
    assert_eq!(
        hits(&state).shown()[0].strand,
        HitStrand::Minus,
        "found on the complement"
    );
    assert_eq!(state.cursor(), 10, "and the cursor is the run's first base");
    assert_eq!(
        yellow(&state),
        cells_of(9),
        "the eight bases on block 0's bottom-strand row, and nowhere else"
    );
}

/// §5.2: the same run seen from the top strand is a plus hit, and goes on the top
/// row exactly as 8l1 drew it.
#[test]
fn a_plus_hit_is_marked_on_the_top_strand() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    search(&mut state, "TGACAGCT");
    assert_eq!(
        hits(&state).shown().len(),
        1,
        "TGACAGCT occurs once on the top strand"
    );
    assert_eq!(hits(&state).shown()[0].start, 10, "the run at 10");
    assert_eq!(hits(&state).shown()[0].length, 8, "eight bases");
    assert_eq!(
        hits(&state).shown()[0].strand,
        HitStrand::Plus,
        "found on the top strand"
    );
    assert_eq!(state.cursor(), 10, "and the cursor is the run's first base");
    assert_eq!(
        yellow(&state),
        cells_of(8),
        "the eight bases on block 0's top-strand row, and nowhere else"
    );
}

/// §5.3: a protein hit is marked on the strand its CDS stands on, and carries
/// that strand as its own — `bla`'s residues minus and `tet`'s plus. The patterns
/// hold a letter that cannot be a nucleotide letter, so the prompt's mode is AA
/// without `tab` (8l1 §4 rule 2).
#[test]
fn a_protein_hit_takes_its_cds_strand() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert_eq!(state.handle(Key::Char('c')), Action::Continue, "c");
    press(&mut state, Key::Home, 1);
    press(&mut state, Key::Down, 2);
    assert_eq!(state.handle(Key::Enter), Action::Continue, "enter");
    assert_eq!(
        state.selected(),
        Some(2),
        "bla, the third CDS in file order"
    );
    search_as_protein(&mut state, "MSIQ");
    let found: &Hits = hits(&state);
    assert_eq!(found.shown().len(), 1, "bla's first four residues, once");
    assert_eq!(found.shown()[0].start, 4141, "the twelve bases of MSI");
    assert_eq!(found.shown()[0].length, 12, "four codons");
    assert_eq!(
        found.shown()[0].strand,
        HitStrand::Minus,
        "bla is on the minus strand"
    );

    // The same search in the other usable CDS of the same record: `tet`, whose
    // residues run the other way.
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert_eq!(state.handle(Key::Char('c')), Action::Continue, "c");
    press(&mut state, Key::Home, 1);
    assert_eq!(state.handle(Key::Enter), Action::Continue, "enter");
    assert_eq!(state.selected(), Some(0), "tet, the first CDS");
    search_as_protein(&mut state, "MKSNNAL");
    let found: &Hits = hits(&state);
    assert_eq!(found.shown().len(), 1, "tet's first seven residues, once");
    assert_eq!(
        found.shown()[0].start,
        85,
        "the 21 bases of the run from tet's first codon"
    );
    assert_eq!(found.shown()[0].length, 21, "seven codons");
    assert_eq!(
        found.shown()[0].strand,
        HitStrand::Plus,
        "tet is on the plus strand"
    );
}
