//! The sequence view's screen as a state and a drawing (contract 8k1 §5). Every
//! record is a fixture read by `open_record`, every size 80 × 24 unless the test
//! says otherwise, and every number below was read off the fixture or worked out
//! from 8j's layout rule — the blocks are 66 bases wide at this width for pBR322
//! (`tests/fixtures/view/README.md`, "Block layout"), which is what the cursor
//! and block numbers in these tests count.
//!
//! The state is driven the way the terminal drives it: through `handle` and its
//! `Key`s, never by setting a field. What the screen shows is read back off a
//! `ratatui` test backend, cell by cell, the way `tests/review_view.rs` reads the
//! review screen: rows of symbols for the words, and the modifiers of every cell
//! for the cursor and its codon (§4 rule 12).
//!
//! Rows 13 to 15 come from the planner's review of the first round rather than
//! from §5's table, with their values derived by hand from the fixtures: the
//! status span of a codon across the origin, and the two counts that decide which
//! blocks are drawn.

use luria_cli::sequence_view::layout::LaneMode;
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

/// One fixture record, named as the CLI names it: the content decides the format,
/// never the name (D049), and `open_record` is what 8h will call.
fn record(path: &str, name: &str) -> ViewRecord {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text: String =
        fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"));
    open_record(Identifier::create(name.to_string()).unwrap(), text)
        .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
}

/// pBR322: 4361 bp, circular as its LOCUS line states, with `tet` (0), the ROP
/// CDS (1, unusable) and `bla` (2) — two usable CDSs, so M8-11 selects none.
fn pbr() -> ViewRecord {
    record("../../tests/fixtures/genbank/raw/J01749.1.gb", "SYNPBR322")
}

/// The hand-written linear record: 393 bp, one CDS `synA` over zero-based
/// 42..345 on the plus strand, so its one usable CDS is selected on open.
fn lin() -> ViewRecord {
    record("../../tests/fixtures/genbank/synthetic-linear.gb", "synlin")
}

/// The 16 bp FASTA of contract §5: no features, no CDSs, and a topology the file
/// does not state, which core assumes is linear (D049).
fn w16() -> ViewRecord {
    open_record(
        Identifier::create("w".to_string()).unwrap(),
        ">w\nTTCAAAAAAAAAAGAA\n".to_string(),
    )
    .expect("the 16 bp FASTA record")
}

/// The 62 bp circular record of the view fixtures: `fP` over `join(56..62,1..11)`
/// (CDS 0, plus strand) and `fM` over `complement(join(53..62,1..8))` (CDS 1,
/// minus strand), both of them across the origin and both usable
/// (`tests/fixtures/view/frames-62.gb`, `tests/fixtures/view/README.md`).
fn f62() -> ViewRecord {
    record("../../tests/fixtures/view/frames-62.gb", "frames62")
}

/// `key` pressed `times` times, which is how the cursor is walked to the base a
/// test starts from: a walk is a row of presses, not an assignment.
fn walk(state: &mut ViewState, key: Key, times: usize) {
    for _ in 0..times {
        assert_eq!(
            state.handle(key),
            Action::Continue,
            "{key:?} does not leave"
        );
    }
}

/// The screen drawn on a test backend: every row as the symbols of its cells.
fn rows(state: &ViewState, width: u16, height: u16) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
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
fn modifiers(state: &ViewState, width: u16, height: u16) -> Vec<Vec<Modifier>> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| draw(frame, state)).unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = usize::from(buffer.area.width);
    buffer
        .content
        .chunks(width)
        .map(|cells| cells.iter().map(|cell| cell.modifier).collect())
        .collect()
}

/// Where on screen a modifier appears: `(row, column)` of every cell carrying it.
fn marked(state: &ViewState, width: u16, height: u16, want: Modifier) -> Vec<(usize, usize)> {
    let mut found: Vec<(usize, usize)> = Vec::new();
    for (row, cells) in modifiers(state, width, height).iter().enumerate() {
        for (column, cell) in cells.iter().enumerate() {
            if cell.contains(want) {
                found.push((row, column));
            }
        }
    }
    found
}

/// The symbol in one cell of the screen.
fn symbol(state: &ViewState, width: u16, height: u16, row: usize, column: usize) -> String {
    rows(state, width, height)
        .get(row)
        .and_then(|line| line.chars().nth(column))
        .unwrap_or_else(|| panic!("no cell at row {row} column {column}"))
        .to_string()
}

/// The block of the laid-out record that holds a base. `blocks` are half-open and
/// tile the record, so exactly one does.
fn block_holding(state: &ViewState, base: u64) -> usize {
    let built = state.layout().expect("a layout at this size");
    built
        .blocks
        .iter()
        .position(|(first, last)| *first <= base && base < *last)
        .unwrap_or_else(|| panic!("no block holds base {base}"))
}

/// M8-11 (D048, §4 rule 2): one usable CDS is selected on open and its own lane
/// is drawn; several usable CDSs select none and the lanes are off.
#[test]
fn the_cds_on_open_follows_m8_11() {
    // `lin` has one usable CDS, so it is the one selected.
    let state: ViewState = ViewState::new(lin(), 80, 24);
    assert_eq!(state.selected(), Some(0), "synA is the only usable CDS");
    assert_eq!(state.mode(), &LaneMode::Cds(0), "and its lane is drawn");

    // pBR322 has two usable CDSs — `tet` and `bla`, the ROP CDS is unusable —
    // so none is selected, and a record with no CDS at all is the same.
    let state: ViewState = ViewState::new(pbr(), 80, 24);
    assert_eq!(state.selected(), None, "two usable CDSs select none");
    assert_eq!(state.mode(), &LaneMode::Off);

    let state: ViewState = ViewState::new(w16(), 80, 24);
    assert_eq!(state.selected(), None, "no CDS to select");
    assert_eq!(state.mode(), &LaneMode::Off);
}

// §4 rules 3, 4 and 8: the cursor wraps only on a topology the view moves on as
// circular, and `T` changes that only where the file left it assumed.
#[test]
fn base_moves_wrap_only_when_circular() {
    // pBR322's LOCUS line states circular, so the ends are joined from the open.
    let mut state: ViewState = ViewState::new(pbr(), 80, 24);
    assert!(state.circular(), "the file states circular");
    assert_eq!(state.handle(Key::Char('h')), Action::Continue);
    assert_eq!(state.cursor(), 4360, "one left of base 1 is the last base");
    assert_eq!(state.handle(Key::Char('l')), Action::Continue);
    assert_eq!(state.cursor(), 0, "and back again");

    // `T` on that record changes nothing about the topology and says so.
    assert_eq!(state.handle(Key::Char('T')), Action::Continue);
    assert!(state.circular(), "a stated topology never changes");
    assert!(
        state.message().is_some(),
        "and the reason is said: {:?}",
        state.message()
    );

    // The FASTA's topology was assumed linear: its ends are apart until `T`.
    let mut state: ViewState = ViewState::new(w16(), 80, 24);
    assert!(!state.circular(), "linear as assumed");
    assert_eq!(state.handle(Key::Char('h')), Action::Continue);
    assert_eq!(
        state.cursor(),
        0,
        "past the first base of a linear record: stay"
    );
    assert_eq!(state.handle(Key::End), Action::Continue);
    assert_eq!(state.cursor(), 15, "the last base of a 16 bp record");
    assert_eq!(state.handle(Key::Char('l')), Action::Continue);
    assert_eq!(state.cursor(), 15, "and past the last: stay");

    assert_eq!(state.handle(Key::Char('T')), Action::Continue);
    assert!(state.circular(), "the session's assumption is set aside");
    assert_eq!(state.handle(Key::Char('l')), Action::Continue);
    assert_eq!(state.cursor(), 0, "so the ends are joined");
    assert_eq!(state.handle(Key::Char('T')), Action::Continue);
    assert!(!state.circular(), "and `T` again puts the assumption back");
    assert_eq!(state.handle(Key::Char('h')), Action::Continue);
    assert_eq!(
        state.cursor(),
        0,
        "with the ends apart again there is nothing left of the first base"
    );
}

/// §4 rule 4: `j` and `k` land at the same offset in the next or previous block,
/// and the only clamping is to the block's own last base.
#[test]
fn line_moves_keep_the_offset() {
    let mut state: ViewState = ViewState::new(pbr(), 80, 24);
    let built = state.layout().expect("a layout at 80 wide");
    // 78 columns for the lines (80 less the frame's border): 78 - (2 * 4 + 2)
    // = 68 free columns, and 68 / 3 = 22 whole steps of 3, so 66 bases.
    assert_eq!(built.line_width, 66, "pBR322's blocks hold 66 bases here");

    walk(&mut state, Key::Char('l'), 5);
    assert_eq!(state.cursor(), 5);
    assert_eq!(state.handle(Key::Char('j')), Action::Continue);
    assert_eq!(
        state.cursor(),
        71,
        "the same offset in the next block: 66 + 5"
    );
    assert_eq!(state.handle(Key::Char('k')), Action::Continue);
    assert_eq!(state.cursor(), 5, "and back");
    assert_eq!(state.handle(Key::Char('k')), Action::Continue);
    assert_eq!(state.cursor(), 5, "past the first block: stay");

    // pBR322's last block holds 5 bases (4361 = 66 * 66 + 5), so the line from
    // the block before it ends at the last base of the record rather than past
    // it.
    walk(&mut state, Key::Char('l'), 4305);
    assert_eq!(state.cursor(), 4310);
    let blocks: &Vec<(u64, u64)> = &state.layout().expect("a layout").blocks;
    assert_eq!(blocks.len(), 67, "4361 bases in blocks of 66");
    assert_eq!(blocks[66], (4356, 4361), "the last block holds 5 bases");
    assert_eq!(state.handle(Key::Char('j')), Action::Continue);
    assert_eq!(state.cursor(), 4360, "clamped to that block's last base");
    assert_eq!(state.handle(Key::Char('j')), Action::Continue);
    assert_eq!(state.cursor(), 4360, "past the last block: stay");
}

/// §4 rule 5: `w` and `b` step to the nearest codon's middle base of the selected
/// CDS, at most three bases away, and stay when there is none in that direction.
#[test]
fn codon_moves_step_codon_middles() {
    // `synA` starts at zero-based 42, so its codon middles are 43, 46, 49, …
    let mut state: ViewState = ViewState::new(lin(), 80, 24);
    walk(&mut state, Key::Char('l'), 42);
    assert_eq!(state.cursor(), 42);
    assert_eq!(state.handle(Key::Char('w')), Action::Continue);
    assert_eq!(state.cursor(), 43, "the next codon's middle base");
    assert_eq!(state.handle(Key::Char('w')), Action::Continue);
    assert_eq!(state.cursor(), 46, "and the next");
    assert_eq!(state.handle(Key::Char('b')), Action::Continue);
    assert_eq!(state.cursor(), 43, "back to a middle base");
    assert_eq!(state.handle(Key::Char('b')), Action::Continue);
    assert_eq!(
        state.cursor(),
        43,
        "the first codon has no middle base left of it"
    );

    // With no CDS selected there is no codon to step between.
    let mut state: ViewState = ViewState::new(pbr(), 80, 24);
    walk(&mut state, Key::Char('l'), 100);
    assert_eq!(state.selected(), None);
    assert_eq!(state.handle(Key::Char('w')), Action::Continue);
    assert_eq!(state.cursor(), 100, "no CDS selected, no codon move");

    // `bla` is on the minus strand and ends at zero-based 4153, so its codons run
    // downwards: 4152, 4151, 4150 are its first codon, and 4151 — a middle base —
    // has nothing within three bases above it.
    let mut state: ViewState = ViewState::new(pbr(), 80, 24);
    assert!(state.select_cds(Some(2)), "bla is usable");
    walk(&mut state, Key::Char('l'), 4151);
    assert_eq!(state.cursor(), 4151);
    assert_eq!(state.handle(Key::Char('w')), Action::Continue);
    assert_eq!(state.cursor(), 4151, "the CDS ends above the cursor");
    assert_eq!(state.handle(Key::Char('b')), Action::Continue);
    assert_eq!(state.cursor(), 4148, "the next codon down reads at 4148");
}

/// §4 rule 7: `t` runs the lanes through the selected CDS, three frames, six
/// frames and off, skipping the CDS lane when none is selected, and the cursor
/// stays where it was.
#[test]
fn t_cycles_the_lanes() {
    let mut state: ViewState = ViewState::new(lin(), 80, 24);
    walk(&mut state, Key::Char('l'), 100);
    let seen: Vec<LaneMode> = vec![
        *state.mode(),
        {
            state.handle(Key::Char('t'));
            *state.mode()
        },
        {
            state.handle(Key::Char('t'));
            *state.mode()
        },
        {
            state.handle(Key::Char('t'));
            *state.mode()
        },
        {
            state.handle(Key::Char('t'));
            *state.mode()
        },
    ];
    assert_eq!(
        seen,
        vec![
            LaneMode::Cds(0),
            LaneMode::ThreeFrames,
            LaneMode::SixFrames,
            LaneMode::Off,
            LaneMode::Cds(0),
        ],
        "the cycle with a CDS selected"
    );

    // pBR322 opens with nothing selected, so its cycle has no CDS lane in it.
    let mut state: ViewState = ViewState::new(pbr(), 80, 24);
    walk(&mut state, Key::Char('l'), 100);
    let seen: Vec<LaneMode> = vec![
        *state.mode(),
        {
            state.handle(Key::Char('t'));
            *state.mode()
        },
        {
            state.handle(Key::Char('t'));
            *state.mode()
        },
        {
            state.handle(Key::Char('t'));
            *state.mode()
        },
    ];
    assert_eq!(
        seen,
        vec![
            LaneMode::Off,
            LaneMode::ThreeFrames,
            LaneMode::SixFrames,
            LaneMode::Off
        ],
        "the cycle with no CDS selected"
    );
    assert_eq!(state.cursor(), 100, "the cursor stayed throughout");
}

/// §4 rules 4 and 6: the drawn run of blocks always holds the cursor's block,
/// whether the cursor moved by a key, a page or a new terminal size.
#[test]
fn the_cursor_block_stays_visible() {
    let mut state: ViewState = ViewState::new(pbr(), 80, 24);
    assert_eq!(
        state.visible().start,
        0,
        "the first block is on top at open"
    );
    assert!(
        !state.visible().is_empty(),
        "and at least one block is drawn"
    );

    // The last block: `End` puts the cursor in it, and it is drawn.
    assert_eq!(state.handle(Key::End), Action::Continue);
    let last: usize = state.layout().expect("a layout").blocks.len() - 1;
    assert_eq!(last, 66, "4361 bases in blocks of 66, zero-based 0..=66");
    assert!(
        state.visible().contains(&block_holding(&state, 4360)),
        "the last block is drawn"
    );

    // Back to the first base and the run starts again at the first block.
    assert_eq!(state.handle(Key::Home), Action::Continue);
    assert_eq!(state.visible().start, 0);

    // A page moves the cursor a whole run of blocks and puts its block on top.
    walk(&mut state, Key::Char('l'), 5);
    let screen: usize = state.visible().len();
    assert_eq!(state.handle(Key::PageDown), Action::Continue);
    assert_eq!(
        state.cursor(),
        5 + 66 * u64::try_from(screen).unwrap(),
        "a page of blocks, same offset"
    );
    assert_eq!(state.visible().start, screen, "and its block on top");
    assert!(state
        .visible()
        .contains(&block_holding(&state, state.cursor())));

    // A wider terminal lays the record out again: the cursor's block is still
    // drawn, at the new block size.
    state.resize(120, 40);
    // 118 columns for the lines at width 120: 118 - (2 * 4 + 2) = 108 free
    // columns, exactly 36 steps of 3.
    assert_eq!(
        state.layout().expect("a layout").line_width,
        108,
        "120 wide fits 108 bases"
    );
    assert!(
        state
            .visible()
            .contains(&block_holding(&state, state.cursor())),
        "cursor {} still drawn",
        state.cursor()
    );
}

/// §4 rule 9 and D041: the picker takes a usable CDS and nothing else, and
/// dropping the selection takes its lane off with it.
#[test]
fn select_cds_takes_only_a_usable_one() {
    let mut state: ViewState = ViewState::new(pbr(), 80, 24);

    // The ROP CDS is unusable, so it is refused and nothing changes.
    assert!(!state.select_cds(Some(1)), "the ROP CDS is not usable");
    assert_eq!(state.selected(), None);
    assert_eq!(state.mode(), &LaneMode::Off);

    assert!(state.select_cds(Some(2)), "bla is usable");
    assert_eq!(state.selected(), Some(2));
    assert_eq!(state.mode(), &LaneMode::Cds(2));

    assert!(
        !state.select_cds(Some(3)),
        "pBR322 has three CDSs, indexed 0 to 2"
    );
    assert_eq!(state.selected(), Some(2), "and the refusal changes nothing");

    assert!(state.select_cds(None), "and none is a choice");
    assert_eq!(state.selected(), None);
    assert_eq!(state.mode(), &LaneMode::Off, "with the CDS lane off too");
}

/// §4 rule 10: `q` and `ctrl-c` leave, and a key with no rule does nothing.
#[test]
fn q_and_ctrl_c_quit() {
    let mut state: ViewState = ViewState::new(pbr(), 80, 24);
    walk(&mut state, Key::Char('l'), 7);
    let mode: LaneMode = *state.mode();

    assert_eq!(state.handle(Key::Char('x')), Action::Continue);
    assert_eq!(state.cursor(), 7, "an unbound key moves nothing");
    assert_eq!(state.mode(), &mode, "and changes no lanes");
    assert_eq!(state.handle(Key::Other), Action::Continue);
    assert_eq!(state.cursor(), 7, "nor does a key this view does not name");

    assert_eq!(state.handle(Key::Char('q')), Action::Quit);
    assert_eq!(state.handle(Key::CtrlC), Action::Quit);
}

/// §4 rules 1, 11 and 12: the title in the frame's top border, the cursor's cell
/// reversed on the top strand's row, its codon underlined, and the status line
/// under the blocks.
#[test]
fn the_screen_shows_cursor_codon_and_status() {
    let mut state: ViewState = ViewState::new(lin(), 80, 24);
    walk(&mut state, Key::Char('l'), 43);
    assert_eq!(state.cursor(), 43, "the cursor walked there by keys");

    // One reversed cell, and it is the cursor's base: the gutter is 3 columns for
    // a 393 bp record, so base 43 of the first block stands at screen column
    // 1 (the border) + 3 + 1 + 43.
    let cursor_cells: Vec<(usize, usize)> = marked(&state, 80, 24, Modifier::REVERSED);
    assert_eq!(
        cursor_cells.len(),
        1,
        "one cursor cell on screen: {cursor_cells:?}"
    );
    let (row, column): (usize, usize) = cursor_cells[0];
    assert_eq!(column, 48, "the base column the layout's rule gives");
    assert_eq!(
        symbol(&state, 80, 24, row, column),
        "T",
        "the base the cursor stands on"
    );

    // Its codon is the whole codon: three underlined cells on the same row.
    let codon_cells: Vec<(usize, usize)> = marked(&state, 80, 24, Modifier::UNDERLINED);
    assert_eq!(
        codon_cells,
        vec![(row, 47), (row, 48), (row, 49)],
        "the codon the cursor stands in, on screen"
    );

    let drawn: Vec<String> = rows(&state, 80, 24);
    assert!(drawn[0].contains("synlin"), "title: {}", drawn[0]);
    assert!(drawn[0].contains("393 bp"), "title: {}", drawn[0]);
    assert!(drawn[0].contains("linear"), "title: {}", drawn[0]);
    assert!(drawn[0].contains("synA"), "title: {}", drawn[0]);

    // The status line is the frame's last inner row, at 24 rows tall that is row
    // 21, the row above the keys.
    let status: &String = &drawn[21];
    assert!(status.contains("44"), "status: {status}");
    assert!(status.contains("M1"), "status: {status}");
    assert!(status.contains("ATG"), "status: {status}");
    assert!(status.contains("43"), "status: {status}");
    assert!(status.contains("45"), "status: {status}");

    // pBR322 has usable CDSs and M8-11 selected none, and its topology is stated.
    let pbr_state: ViewState = ViewState::new(pbr(), 80, 24);
    let pbr_rows: Vec<String> = rows(&pbr_state, 80, 24);
    assert!(
        pbr_rows[0].contains("no CDS selected"),
        "title: {}",
        pbr_rows[0]
    );
    assert!(!pbr_rows[0].contains("(assumed)"), "title: {}", pbr_rows[0]);
}

/// §4 rule 11 and D049 M8-3: a FASTA says its topology was assumed, and says it
/// is set once `T` has set it for the session.
#[test]
fn a_fasta_title_says_assumed() {
    let mut state: ViewState = ViewState::new(w16(), 80, 24);
    let title: Vec<String> = rows(&state, 80, 24);
    assert!(title[0].contains("linear (assumed)"), "title: {}", title[0]);

    assert_eq!(state.handle(Key::Char('T')), Action::Continue);
    let title: Vec<String> = rows(&state, 80, 24);
    assert!(
        title[0].contains("circular (set with T)"),
        "title: {}",
        title[0]
    );
}

/// §4 rules 1 and 10 and §2: below 80 × 24 the screen is a notice about the size
/// and nothing else, and the only keys doing anything are the two that leave.
#[test]
fn a_small_screen_draws_only_a_notice() {
    let mut state: ViewState = ViewState::new(pbr(), 79, 24);
    assert!(state.layout().is_none(), "no layout below the minimum");
    assert_eq!(state.visible(), 0..0, "and no block drawn");

    let drawn: Vec<String> = rows(&state, 79, 24);
    assert!(drawn[0].contains("79x24"), "row 0: {}", drawn[0]);
    assert!(drawn[0].contains("80x24"), "row 0: {}", drawn[0]);
    for (row, line) in drawn.iter().enumerate().skip(1) {
        assert!(line.trim_end().is_empty(), "row {row} is blank: {line:?}");
    }

    assert_eq!(state.handle(Key::Char('l')), Action::Continue);
    assert_eq!(state.cursor(), 0, "a cursor move does nothing at this size");
    assert_eq!(state.handle(Key::Char('q')), Action::Quit);
}

/// §4 rule 13 and D040: the codon the status line names is read on its own
/// strand, and its span is given in reference order.
#[test]
fn a_minus_strand_codon_is_read_on_its_strand() {
    let mut state: ViewState = ViewState::new(pbr(), 80, 24);
    assert!(state.select_cds(Some(2)), "bla is usable");
    walk(&mut state, Key::Char('l'), 4151);
    assert_eq!(state.cursor(), 4151, "zero-based 4151");

    // `bla`'s first codon reads the top strand's CAT backwards: the codon is ATG,
    // and its bases are the reference span 4151 to 4153, one-based.
    let status: String = rows(&state, 80, 24)[21].trim_end().to_string();
    assert!(status.contains("M1"), "status: {status}");
    assert!(status.contains("ATG"), "status: {status}");
    assert!(status.contains("4151"), "status: {status}");
    assert!(status.contains("4153"), "status: {status}");

    // One codon further down the same CDS: AGT, Ser, the second residue.
    assert_eq!(state.handle(Key::Char('b')), Action::Continue);
    let status: String = rows(&state, 80, 24)[21].trim_end().to_string();
    assert!(status.contains("S2"), "status: {status}");
    assert!(status.contains("AGT"), "status: {status}");
}

/// D036 and §4 rule 13: the span the status line gives a codon is in reference
/// order, which is the order `cds_span` prints a joined location's parts in
/// (`fP` is `56..11`). A codon running across the origin of a circular record
/// therefore has a span whose first number is the larger one: `frames-62.gb`'
/// `fP` = `join(56..62,1..11)` and `fM` = `complement(join(53..62,1..8))` each
/// hold a codon over the 1-based bases 62, 1 and 2, and its span is `62..2`.
#[test]
fn a_codon_across_the_origin_spans_it() {
    let mut state: ViewState = ViewState::new(f62(), 80, 24);
    assert!(state.select_cds(Some(0)), "fP is usable");
    assert_eq!(state.cursor(), 0, "the first base of the record");

    // `fP`'s third codon reads the 1-based bases 62, 1, 2 - ACA, Thr, the third
    // residue of MRTDH - and the cursor stands on its middle base.
    let status: String = rows(&state, 80, 24)[21].trim_end().to_string();
    assert!(status.contains("T3"), "status: {status}");
    assert!(status.contains("ACA"), "status: {status}");
    assert!(status.contains("62..2"), "status: {status}");

    // The codon above it lies wholly in the CDS's first location part: its middle
    // base is zero-based 59, one base left of the cursor across the join. From
    // there, `w` steps the three bases of one codon back over the join.
    assert_eq!(state.handle(Key::Char('b')), Action::Continue);
    assert_eq!(state.cursor(), 59, "the codon above the join");
    assert_eq!(state.handle(Key::Char('w')), Action::Continue);
    assert_eq!(state.cursor(), 0, "and back across it");

    // The same three bases are `fM`'s third codon, read downwards and complemented
    // - TGT, Cys, the third residue of MVCAH - and its span is reference order too.
    assert!(state.select_cds(Some(1)), "fM is usable");
    assert_eq!(
        state.cursor(),
        0,
        "selecting a CDS does not move the cursor"
    );
    let status: String = rows(&state, 80, 24)[21].trim_end().to_string();
    assert!(status.contains("C3"), "status: {status}");
    assert!(status.contains("TGT"), "status: {status}");
    assert!(status.contains("62..2"), "status: {status}");

    assert_eq!(state.handle(Key::Char('w')), Action::Continue);
    assert_eq!(
        state.cursor(),
        3,
        "the middle base of the codon below the join"
    );
    assert_eq!(state.handle(Key::Char('b')), Action::Continue);
    assert_eq!(state.cursor(), 0, "and back");
}

/// §4 rule 1: `visible()` is the longest run of whole blocks whose lines *and the
/// gaps between them* fit in `height - 4` rows. pBR322 with the lanes off: the
/// body is 20 rows, block 0 is 7 lines (4 feature lanes, the `more` marker, top
/// and bottom), block 1 is 6 and block 2 is 6, so 7 + 1 + 6 = 14 fits and
/// 7 + 1 + 6 + 1 + 6 = 21 does not.
#[test]
fn visible_counts_the_gaps() {
    let state: ViewState = ViewState::new(pbr(), 80, 24);
    assert_eq!(
        state.mode(),
        &LaneMode::Off,
        "the plain lanes, as M8-11 opens it"
    );
    assert_eq!(
        state.visible(),
        0..2,
        "the run stops at two blocks, gaps and all"
    );
}

/// §4 rule 6: when the cursor's block is below the drawn run, the top becomes the
/// smallest block from which it is visible, and the run is then counted again from
/// there. pBR322 with the lanes off, from the open: two `j` presses put the cursor
/// at zero-based 132 (66 * 2), which is block 2, and the top becomes 1 rather
/// than 0 - from 1 the run is 6 + 1 + 6 = 13 rows, and from 0 it would be 21 - so
/// blocks 1, 2 and 3 are drawn, block 3 being 4 lines: 6 + 1 + 6 + 1 + 4 = 18.
#[test]
fn the_scroll_moves_as_little_as_it_can() {
    let mut state: ViewState = ViewState::new(pbr(), 80, 24);
    assert_eq!(state.visible(), 0..2, "the run at the open");

    assert_eq!(state.handle(Key::Char('j')), Action::Continue);
    assert_eq!(
        state.cursor(),
        66,
        "the same offset in block 1, which is drawn"
    );
    assert_eq!(state.visible(), 0..2, "so nothing has scrolled yet");
    assert_eq!(state.handle(Key::Char('j')), Action::Continue);
    assert_eq!(state.cursor(), 132, "block 2, below the run");
    assert_eq!(
        state.visible(),
        1..4,
        "the least scroll that brings block 2 in"
    );
}
