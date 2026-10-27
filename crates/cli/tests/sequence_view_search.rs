//! Search as the view holds it and draws it (contract 8l1 §5; plan §5.4,
//! proposal §4, D048 M8-12, D049 M8-3, D052, D054).
//!
//! `pbr` is `tests/fixtures/genbank/raw/J01749.1.gb` — 4361 bp, circular by its
//! own declaration, 50 features and three CDSs of which two are usable, so
//! M8-11 selects none and an AA search reads six frames; `w16` is the 16 bp
//! FASTA `>w` / `TTCAAAAAAAAAAGAA`, whose topology the view assumed and `T`
//! switches (D049 M8-3). Every hit value asserted is a row of
//! `tests/fixtures/view/search-dna-hits.tsv` or `tests/fixtures/view/
//! search-protein-hits.tsv`, zero-based as those tables write it: `AATT` at 58,
//! 251, 1319, 1333, 3233, 3539, 3794 and 4359 (the last wrapping the origin,
//! strand `both`, length 4), `GAATTC` once at 4358 with length 6 and wrapping,
//! `N` at every base of the record — 4361 hits of length 1 — `pbr322 frames
//! MKSNNA` once at 85 on strand `+` frame 2 with length 18, and `pbr322 bla M`
//! ten times, first at 3349 and last at 4150. The columns those rows sit on are
//! the ones `tests/fixtures/view/README.md` records.
//!
//! The prompt is driven the way the terminal drives it — `/`, one `Key` per
//! character, `tab`, `enter` — and the screen is read back off a `ratatui` test
//! backend, the status line from row 21 as `tests/sequence_view_goto.rs` reads
//! it and a hit's yellow from the background of the cells it paints.

use luria_cli::sequence_view::layout::Layout;
use luria_cli::sequence_view::list::ListKind;
use luria_cli::sequence_view::screen::draw;
use luria_cli::sequence_view::search::Hits;
use luria_cli::sequence_view::search::SearchMode;
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

/// Every test draws at this size: 80 columns and 24 rows, which leaves the body
/// 20 rows and puts the status line on row 21 and the keys on row 23.
const W: u16 = 80;
const H: u16 = 24;

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

/// The 16 bp FASTA of contract §5: linear until `T` says circular, with no
/// features and no CDSs to select.
fn w16() -> ViewRecord {
    open_record(
        Identifier::create("w".to_string()).unwrap(),
        ">w\nTTCAAAAAAAAAAGAA\n".to_string(),
    )
    .expect("the 16 bp FASTA record")
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

/// The open prompt's text, or a failure saying no prompt is open.
fn typed(state: &ViewState) -> String {
    state
        .search()
        .expect("the search prompt is open")
        .text()
        .to_string()
}

/// The open prompt's mode.
fn mode(state: &ViewState) -> SearchMode {
    state.search().expect("the search prompt is open").mode()
}

/// "`/`, this text, `enter`" (8l1 §4 rule 1 and 3), with the text typed as the
/// terminal would type it.
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

/// "`/`, `tab`, this text, `enter`": AA whatever the text says (8l1 §4 rule 2).
fn search_as_aa(state: &mut ViewState, text: &str) {
    assert_eq!(
        state.handle(Key::Char('/')),
        Action::Continue,
        "/ opens the prompt"
    );
    assert_eq!(
        state.handle(Key::Tab),
        Action::Continue,
        "tab flips the mode"
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

/// The block holding the cursor's base is among the blocks drawn: the scroll
/// follows the hit the cursor was parked on (8k1 §4 rule 6, 8l1 §4 rule 4).
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

/// The status line as drawn: row 21 of the screen (8k1 §4 rule 1).
fn status_row(state: &ViewState) -> String {
    rows(state)[21].clone()
}

/// Where on screen a base is marked yellow: `(row, column)` of every cell whose
/// background is `Color::Yellow`, in screen order (8l1 §4 rule 8).
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

/// The columns of the yellow cells, all of which have to sit on one row: the
/// bases row of the block they belong to.
fn yellow_columns(state: &ViewState) -> Vec<usize> {
    let cells: Vec<(usize, usize)> = yellow(state);
    let row: usize = cells.first().expect("at least one cell is marked").0;
    assert!(
        cells.iter().all(|(cell_row, _column)| *cell_row == row),
        "the marks sit on the one row that carries the bases: {cells:?}"
    );
    cells.iter().map(|(_row, column)| *column).collect()
}

/// §5.1: `/` opens an empty DNA prompt; `esc` closes it and leaves nothing marked,
/// and nothing opens below 80 × 24 or inside another prompt.
#[test]
fn slash_opens_a_dna_prompt() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);

    assert_eq!(
        state.handle(Key::Char('/')),
        Action::Continue,
        "/ opens the prompt"
    );
    assert_eq!(typed(&state), "", "with nothing typed");
    assert_eq!(mode(&state), SearchMode::Dna, "and no letters to read");

    assert_eq!(state.handle(Key::Esc), Action::Continue, "esc closes it");
    assert!(state.search().is_none(), "and the prompt is gone");
    assert!(state.hits().is_none(), "and nothing was searched");
    assert_eq!(state.cursor(), 0, "and nothing moved");

    // With hits marked, esc closes the prompt and leaves the marks, the cursor and
    // the selection where they were (8l1 §4 rule 1).
    search(&mut state, "AATT");
    state.handle(Key::Char('/'));
    state.handle(Key::Esc);
    assert!(state.search().is_none(), "esc closes the prompt");
    assert_eq!(hits(&state).total(), 8, "and leaves the hits marked");
    assert_eq!(state.cursor(), 58, "and the cursor where it stood");

    // Below 80 x 24 the view draws nothing and no key reaches it (8k1 §4 rule 10).
    state.resize(79, H);
    state.handle(Key::Char('/'));
    assert!(state.search().is_none(), "no prompt at 79 columns");
    state.resize(W, H);

    // In the go-to prompt and in an open list, `/` is text (8l1 §4 rule 1).
    state.handle(Key::Char('g'));
    state.handle(Key::Char('/'));
    assert_eq!(state.prompt(), Some("/"), "/ is text at go-to");
    state.handle(Key::Esc);
    state.handle(Key::Char('f'));
    state.handle(Key::Char('/'));
    assert_eq!(
        state.list().expect("the feature list is open").filter(),
        "/",
        "/ is filter text"
    );
    assert_eq!(state.list().expect("still open").kind(), ListKind::Features);
}

/// §5.2: the mode follows the text until `tab` is pressed, `backspace` takes the
/// switch back with it, and a pattern of IUPAC letters only stays DNA.
#[test]
fn the_mode_follows_the_text_until_tab() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);

    state.handle(Key::Char('/'));
    type_into(&mut state, "GA");
    assert_eq!(mode(&state), SearchMode::Dna, "both letters are IUPAC");
    assert!(
        status_row(&state).contains("DNA"),
        "the badge says DNA: {}",
        status_row(&state)
    );

    type_into(&mut state, "E");
    assert_eq!(mode(&state), SearchMode::Aa, "E is no nucleotide");
    assert!(
        status_row(&state).contains("AA"),
        "and the badge on row 21 says so: {}",
        status_row(&state)
    );

    state.handle(Key::Backspace);
    assert_eq!(
        mode(&state),
        SearchMode::Dna,
        "deleting the E returns the badge to DNA"
    );

    state.handle(Key::Tab);
    assert_eq!(mode(&state), SearchMode::Aa, "tab switches by hand");
    type_into(&mut state, "C");
    assert_eq!(mode(&state), SearchMode::Aa, "and the mode stops following");

    press(&mut state, Key::Backspace, 3);
    assert_eq!(typed(&state), "", "the text is empty again");
    assert_eq!(mode(&state), SearchMode::Aa, "and still AA");

    state.handle(Key::Esc);
    assert!(state.search().is_none(), "esc closes the prompt");
    state.handle(Key::Char('/'));
    assert_eq!(mode(&state), SearchMode::Dna, "a new prompt starts at DNA");

    type_into(&mut state, "MKSNNA");
    assert_eq!(
        mode(&state),
        SearchMode::Dna,
        "every letter of MKSNNA is an IUPAC code, so DNA"
    );
}

/// §5.3: the current hit is the first at or after the cursor, and `n` and `N`
/// walk the shown hits, wrapping at the ends.
#[test]
fn the_first_hit_at_or_after_the_cursor() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);

    search(&mut state, "AATT");
    assert_eq!(
        hits(&state).total(),
        8,
        "search-dna-hits.tsv has eight rows"
    );
    assert_eq!(hits(&state).current(), 0, "58 is the first at or after 0");
    assert_eq!(state.cursor(), 58, "and the cursor goes to its start");
    cursor_block_is_drawn(&state);
    // The rows of search-dna-hits.tsv, in D054's order.
    let starts: Vec<u64> = hits(&state).shown().iter().map(|hit| hit.start).collect();
    assert_eq!(starts, vec![58, 251, 1319, 1333, 3233, 3539, 3794, 4359]);
    assert!(
        hits(&state)
            .shown()
            .iter()
            .all(|hit| hit.length == 4 && hit.label == "both"),
        "AATT is palindromic, so one hit of 4 bases per start, strand both"
    );

    state.handle(Key::Char('n'));
    assert_eq!(state.cursor(), 251, "n goes to the next shown hit");
    assert_eq!(hits(&state).current(), 1);
    state.handle(Key::Char('N'));
    assert_eq!(state.cursor(), 58, "N goes back");
    state.handle(Key::Char('N'));
    assert_eq!(hits(&state).current(), 7, "and wraps to the last one shown");
    assert_eq!(state.cursor(), 4359);
    state.handle(Key::Char('n'));
    assert_eq!(hits(&state).current(), 0, "and forward wraps to the first");
    assert_eq!(state.cursor(), 58);

    press(&mut state, Key::Char('n'), 2);
    assert_eq!(state.cursor(), 1319, "two hits on");
    search(&mut state, "AATT");
    assert_eq!(
        hits(&state).current(),
        2,
        "searching again starts from where the cursor stands"
    );
    assert_eq!(state.cursor(), 1319, "which is itself a hit");

    state.handle(Key::Char('l'));
    search(&mut state, "AATT");
    assert_eq!(
        state.cursor(),
        1333,
        "one base past 1319 reaches the next hit"
    );

    state.handle(Key::End);
    search(&mut state, "AATT");
    assert_eq!(hits(&state).current(), 0, "nothing lies at or after 4360");
    assert_eq!(state.cursor(), 58, "so the first hit is current");
}

/// §5.4: a hit whose run crosses the origin is marked on both of its runs, and
/// only what the screen draws gets cells. pBR322's EcoRI site sits at 4358, 6
/// bases, wrapping (search-dna-hits.tsv); the block on top starts at 4320 with a
/// gutter 4 wide, so its bases run from column `1 + 4 + 1 + (4358 - 4320)`.
#[test]
fn a_hit_across_the_origin_is_marked() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);

    search(&mut state, "GAATTC");
    assert_eq!(state.cursor(), 4358, "the one hit of one row");
    assert_eq!(hits(&state).shown()[0].length, 6);
    let columns: Vec<usize> = yellow_columns(&state);
    assert_eq!(
        columns,
        vec![44, 45, 46],
        "the three bases this block holds; the other three sit in block 0, which \
         the scroll left off screen"
    );

    // The same hit seen from the other end of the record: block 0 is drawn now and
    // carries the run the site wrapped on to (8l1 §4 rule 8, D052).
    state.handle(Key::Home);
    let columns: Vec<usize> = yellow_columns(&state);
    assert_eq!(
        columns,
        vec![6, 7, 8],
        "the wrapped run, at gutter 4 from block 0's first base"
    );
}

/// §5.5: AA reads the selected CDS, or the six frames when none is selected
/// (M8-11), and the cap and count the status line gives.
#[test]
fn aa_searches_the_cds_or_six_frames() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert_eq!(state.selected(), None, "pBR322 opens with no CDS selected");

    search_as_aa(&mut state, "MKSNNA");
    assert_eq!(
        hits(&state).total(),
        1,
        "search-protein-hits.tsv: frames, one row"
    );
    assert_eq!(state.cursor(), 85, "its start");
    let hit = &hits(&state).shown()[0];
    assert_eq!(hit.length, 18, "six residues is eighteen bases");
    assert_eq!(hit.label, "+2", "the frame it was read in");

    // `bla` is cds 2, and its own translation holds `M` ten times, the first at
    // base 3349 (search-protein-hits.tsv).
    assert!(state.select_cds(Some(2)), "bla is usable");
    state.handle(Key::Home);
    search_as_aa(&mut state, "M");
    assert_eq!(hits(&state).total(), 10, "ten rows in the CDS");
    assert_eq!(state.cursor(), 3349, "and D054 puts the leftmost first");
    assert_eq!(
        hits(&state).shown()[0].length,
        3,
        "one residue is one codon"
    );
    // The row is `pbr322 bla M` hit 1: start 3349, first residue 268 and last
    // residue 268 — `bla` is read on the minus strand, so its 268th residue sits at
    // reference base 3350 (D054 puts this leftmost of the ten).
    assert_eq!(hits(&state).shown()[0].label, "268", "the residue numbers");
    let line: String = status_row(&state);
    assert!(
        line.contains("of 10"),
        "the status line counts them: {line}"
    );
}

/// §5.6: at most 200 hits are shown and the real count is said (M8-12, D048); `N`
/// matches every base of the record once, strand both (D052).
#[test]
fn two_hundred_are_shown() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);

    search(&mut state, "N");
    assert_eq!(hits(&state).total(), 4361, "one hit per base of pBR322");
    assert_eq!(hits(&state).shown().len(), 200, "and 200 of them are shown");
    assert_eq!(hits(&state).current(), 0);
    assert_eq!(state.cursor(), 0, "the first hit is at the cursor");
    assert_eq!(hits(&state).shown()[199].start, 199, "core's order, D054");
    let line: String = status_row(&state);
    assert!(
        line.contains("200") && line.contains("4361"),
        "the line says what is shown and what there really is: {line}"
    );

    state.handle(Key::Char('N'));
    assert_eq!(
        hits(&state).current(),
        199,
        "N wraps back inside what is shown"
    );
    assert_eq!(state.cursor(), 199, "and never past it");
}

/// §5.7: no hits and a refusal both clear what was marked, leave the cursor where
/// it was and leave a message; an empty pattern changes nothing at all.
#[test]
fn no_hits_and_refusals_leave_the_cursor() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);

    search(&mut state, "AATT");
    assert_eq!(state.cursor(), 58);

    search(&mut state, "GAATTT");
    assert_eq!(state.cursor(), 58, "the cursor stays");
    assert!(state.hits().is_none(), "and what was marked is gone");
    assert!(state.message().is_some(), "and the line says so");

    // `A1` reads as AA, and `1` is no residue: core's own refusal.
    state.handle(Key::Char('/'));
    type_into(&mut state, "A1");
    assert_eq!(mode(&state), SearchMode::Aa, "1 is no nucleotide letter");
    state.handle(Key::Enter);
    assert_eq!(state.cursor(), 58, "a refusal moves nothing");
    assert!(state.hits().is_none(), "and marks nothing");
    assert!(state.message().is_some(), "with core's message");

    search(&mut state, "AATT");
    assert_eq!(hits(&state).total(), 8);
    state.handle(Key::Char('/'));
    state.handle(Key::Enter);
    assert!(state.search().is_none(), "enter closes an empty prompt");
    assert_eq!(hits(&state).total(), 8, "and searches nothing");
    assert_eq!(state.cursor(), 58, "which changes nothing");
}

/// §5.8: while the prompt is open the keys are text, `ctrl-c` leaves from
/// anywhere, and below 80 × 24 `q` leaves.
#[test]
fn keys_go_to_the_search_prompt() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);

    state.handle(Key::Char('/'));
    type_into(&mut state, "qn");
    assert_eq!(typed(&state), "qn", "q and n are pattern letters here");
    assert_eq!(state.cursor(), 0, "and move nothing");
    assert_eq!(
        state.handle(Key::CtrlC),
        Action::Quit,
        "ctrl-c quits from an open prompt"
    );

    let mut state: ViewState = ViewState::new(pbr(), W, H);
    state.handle(Key::Char('/'));
    state.resize(79, H);
    assert_eq!(
        state.handle(Key::Char('q')),
        Action::Quit,
        "below 80 x 24 the prompt draws nothing, so q quits"
    );
}

/// §5.9: after `T` a FASTA is searched as circular, as `luria find --circular`
/// searches it (D049 M8-3, find.rs §4.4); `T` clears the hits found on the other
/// topology (8l1 §4 rule 6).
#[test]
fn t_searches_a_fasta_as_circular() {
    let mut state: ViewState = ViewState::new(w16(), W, H);
    assert!(!state.circular(), "a FASTA opens linear (assumed)");

    search(&mut state, "GAATTC");
    assert!(state.hits().is_none(), "nothing matches a linear 16-mer");
    assert_eq!(state.cursor(), 0);

    state.handle(Key::Char('T'));
    assert!(state.circular(), "T switches it for the session");
    search(&mut state, "GAATTC");
    assert_eq!(hits(&state).total(), 1, "the site across the origin");
    assert_eq!(hits(&state).shown()[0].start, 13, "GAA at 13 and TTC at 0");
    assert_eq!(state.cursor(), 13);

    state.handle(Key::Char('T'));
    assert!(!state.circular(), "and back");
    assert!(
        state.hits().is_none(),
        "the hits of the other topology are gone"
    );
}

/// §5.10: with nothing marked, `n` and `N` do nothing at all (8l1 §4 rule 5).
#[test]
fn n_without_hits_does_nothing() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);

    assert_eq!(state.handle(Key::Char('n')), Action::Continue);
    assert_eq!(state.handle(Key::Char('N')), Action::Continue);
    assert_eq!(state.cursor(), 0, "no hits, nowhere to go");
    assert!(state.hits().is_none());
}

/// §7: the wrapping hit's last base is numbered again from the first, the same
/// span `luria find` prints (8i2, `find::last_base`). pBR322's EcoRI site starts
/// at 4358 and runs 6 bases of a 4361 bp record, so the status line says
/// `4359..3`: bases 4362 to 4364 do not exist (search-dna-hits.tsv).
#[test]
fn a_wrapping_hit_names_its_last_base() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    search(&mut state, "GAATTC");
    assert_eq!(hits(&state).total(), 1, "one EcoRI site, and it wraps");
    assert_eq!(state.cursor(), 4358, "which is where the cursor goes");
    let line: String = status_row(&state);
    assert!(line.contains("4359..3"), "the span the line gives: {line}");
    assert!(
        !line.contains("4364"),
        "no base past the record's last: {line}"
    );
}

/// §7: every shown hit is marked, not only the current one. pBR322 `/RATY`: hits
/// 1, 2 and 52 of search-dna-hits.tsv start at 58, 89 and 4359, 4 bases each, and
/// the cursor at 58 draws blocks 0 and 1 — their bases rows are 6 and 13, and a
/// base sits at column `1 + 4 + 1 + (base - block start)`. The hit at 4359 wraps
/// the origin, so its second run, bases 1 and 2, is marked on block 0 as much as
/// its first run is (D052, 8l1 §4 rule 8); the hit at 58 straddles the two blocks.
#[test]
fn every_shown_hit_is_marked() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    search(&mut state, "RATY");
    assert_eq!(
        hits(&state).total(),
        52,
        "search-dna-hits.tsv has 52 RATY rows"
    );
    assert_eq!(
        hits(&state).shown()[0].start,
        58,
        "the first hit starts at 58"
    );
    assert_eq!(hits(&state).shown()[1].start, 89, "the second at 89");
    assert_eq!(
        hits(&state).shown()[51].start,
        4359,
        "and the last wraps the origin"
    );
    assert_eq!(state.cursor(), 58, "and the cursor goes to the first");
    let mut cells: Vec<(usize, usize)> = [6usize, 7, 64, 65]
        .into_iter()
        .map(|column| (6, column))
        .collect();
    cells.extend(
        [6usize, 7, 35, 36, 37, 38]
            .into_iter()
            .map(|column| (13, column)),
    );
    assert_eq!(
        yellow(&state),
        cells,
        "10 cells: 2 and 2 on block 0's row (the 58 hit's tail of it, and the 4359 hit's wrapped run), 2 and 4 on block 1's"
    );
}

/// §7: the whole key line is visible on an 80 column screen, `q quit` included,
/// with room left over for the key 8l2 adds (8l1 §4 rule 9).
#[test]
fn the_key_line_fits_eighty_columns() {
    let state: ViewState = ViewState::new(pbr(), W, H);
    let line: String = rows(&state)[23].trim_end().to_string();
    assert!(line.contains("q quit"), "the row as drawn: {line}");
    assert!(
        line.chars().count() <= 76,
        "{} columns of key line drawn on 80: {line}",
        line.chars().count()
    );
}

/// The other three key lines have to fit the same 80 columns (8l1 §4 rule 9):
/// the features list, the go-to prompt and the search prompt.
#[test]
fn the_other_key_lines_fit() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    for (what, keys) in [
        ("the view", None),
        ("the features list", Some(Key::Char('f'))),
        ("go-to", Some(Key::Char('g'))),
        ("the search prompt", Some(Key::Char('/'))),
    ] {
        if let Some(key) = keys {
            assert_eq!(state.handle(key), Action::Continue, "{key:?}");
        }
        let line: String = rows(&state)[23].trim_end().to_string();
        assert!(
            line.chars().count() <= 76,
            "{} key line is {} columns on 80: {line}",
            what,
            line.chars().count()
        );
        if keys.is_some() {
            assert_eq!(state.handle(Key::Esc), Action::Continue, "esc");
        }
    }
}
