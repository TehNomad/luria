//! M8-35 (contract 8m13): a residue typed at go-to, and every `w` and `b`, land
//! on the first base of the codon in reading direction — the codon's highest
//! base on the minus strand — while the residue letter still stands over the
//! codon's middle base (D055).
//!
//! `lin` is `tests/fixtures/genbank/synthetic-linear.gb`, whose one usable CDS
//! `synA` (selected on open by M8-11) starts at zero-based 42 on the plus
//! strand, so residue `n`'s first base is `42 + 3(n − 1)`: residue 42's codon is
//! 165, 166, 167 and residue 87's is 300, 301, 302, as
//! `tests/fixtures/genbank/synthetic-linear-residues.tsv` gives them.
//!
//! `pbr` is `tests/fixtures/genbank/raw/J01749.1.gb`, whose `bla` is
//! `complement(3293..4153)` — zero-based 3292 to 4152 — so its residue 1 reads
//! 4152, 4151, 4150 downwards and residue `n`'s first base is
//! `4152 − 3(n − 1)`: 4152, 4149, 4146, … The file's own top-strand letters at
//! one-based 4151, 4152, 4153 are `cat`, drawn upper-cased (D008), which is
//! `ATG` read on the bottom strand, and its `/translation=` gives `bla` residue
//! 1 = `M` and residue 2 = `S`.
//!
//! The state is driven the way the terminal drives it: through `handle` and its
//! `Key`s, never by setting a field, and what the screen shows in the last test
//! is read back off a `ratatui` test backend, cell by cell, as
//! `tests/sequence_view_cursor_codon.rs` reads it.

use luria_cli::sequence_view::layout::LaneMode;
use luria_cli::sequence_view::screen::draw;
use luria_cli::sequence_view::state::Action;
use luria_cli::sequence_view::state::Key;
use luria_cli::sequence_view::state::ViewState;
use luria_core::identity::Identifier;
use luria_core::view_record::open_record;
use luria_core::view_record::ViewRecord;
use ratatui::backend::TestBackend;
use ratatui::buffer::Cell;
use ratatui::style::Modifier;
use ratatui::Terminal;
use std::fs;
use std::path::Path;

/// Every test draws at this size: 80 columns and 24 rows, as every other
/// sequence-view test does.
const W: u16 = 80;
const H: u16 = 24;

fn record(name: &str, path: &str) -> ViewRecord {
    let file: std::path::PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let text: String =
        fs::read_to_string(&file).unwrap_or_else(|error| panic!("read {path}: {error}"));
    open_record(Identifier::create(name.to_string()).unwrap(), text)
        .unwrap_or_else(|error| panic!("open {path}: {error:?}"))
}

fn pbr() -> ViewRecord {
    record("SYNPBR322", "../../tests/fixtures/genbank/raw/J01749.1.gb")
}

fn lin() -> ViewRecord {
    record("SYNLIN", "../../tests/fixtures/genbank/synthetic-linear.gb")
}

/// The keys, pressed one after the other, none of which leaves the view.
fn press(state: &mut ViewState, keys: &[Key]) {
    for key in keys {
        assert_eq!(
            state.handle(*key),
            Action::Continue,
            "key {key:?} does not leave"
        );
    }
}

/// `text` typed one character per key, then `enter` (8k2 §4 rule 2).
fn go_to(state: &mut ViewState, text: &str) {
    press(state, &[Key::Char('g')]);
    for ch in text.chars() {
        press(state, &[Key::Char(ch)]);
    }
    press(state, &[Key::Enter]);
}

/// `key` pressed `n` times.
fn walk(state: &mut ViewState, key: Key, n: usize) {
    for _ in 0..n {
        press(state, &[key]);
    }
}

/// The cells of the drawn screen, row by row, as the test backend holds them.
fn drawn(state: &ViewState) -> Vec<Vec<Cell>> {
    let mut terminal = Terminal::new(TestBackend::new(W, H)).unwrap();
    terminal.draw(|frame| draw(frame, state)).unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = usize::from(buffer.area.width);
    buffer
        .content()
        .chunks(width)
        .map(|row: &[Cell]| row.to_vec())
        .collect()
}

/// The symbol in one drawn cell.
fn symbol(rows: &[Vec<Cell>], row: usize, column: usize) -> String {
    rows[row][column].symbol().to_string()
}

/// Every cell of the drawn screen that is reversed and bold — the cursor's codon
/// on the residue line (M8-32, 8m8).
fn marked(rows: &[Vec<Cell>]) -> Vec<(usize, usize)> {
    rows.iter()
        .enumerate()
        .flat_map(|(row, cells)| {
            cells
                .iter()
                .enumerate()
                .filter(|(_column, cell)| {
                    cell.modifier.contains(Modifier::REVERSED)
                        && cell.modifier.contains(Modifier::BOLD)
                })
                .map(move |(column, _cell)| (row, column))
        })
        .collect()
}

/// The one cell that is reversed but not bold: the cursor's own base, which
/// 8m1 §4 rule 12 draws reversed alone.
fn cursor_cell(rows: &[Vec<Cell>]) -> (usize, usize) {
    let found: Vec<(usize, usize)> = rows
        .iter()
        .enumerate()
        .flat_map(|(row, cells)| {
            cells
                .iter()
                .enumerate()
                .filter(|(_column, cell)| {
                    cell.modifier.contains(Modifier::REVERSED)
                        && !cell.modifier.contains(Modifier::BOLD)
                })
                .map(move |(column, _cell)| (row, column))
        })
        .collect();
    assert_eq!(found.len(), 1, "one cursor cell on screen: {found:?}");
    found[0]
}

/// Test 1: go-to puts the cursor on the codon's first base in reading order, on
/// both strands — the highest base of the codon on the minus strand.
#[test]
fn go_to_lands_on_the_first_base() {
    // `synA` starts at zero-based 42, so residue 42's codon is 165, 166, 167.
    let mut state: ViewState = ViewState::new(lin(), W, H);
    assert_eq!(state.selected(), Some(0), "synA is the only CDS");
    go_to(&mut state, "A42");
    assert_eq!(state.cursor(), 165, "the first base of 165, 166, 167");

    // `bla` reads downwards: residue 1's codon is 4152, 4151, 4150 and residue
    // 2's is 4149, 4148, 4147.
    let mut selected: ViewState = ViewState::new(pbr(), W, H);
    assert!(selected.select_cds(Some(2)), "bla is usable");
    go_to(&mut selected, "M1");
    assert_eq!(selected.cursor(), 4152, "the highest base of its codon");
    go_to(&mut selected, "S2");
    assert_eq!(selected.cursor(), 4149, "and of the codon below it");
}

/// Test 2: `w` and `b` step from first base to first base on the plus strand,
/// starting from a base that is neither the first nor the last of its codon.
#[test]
fn w_and_b_step_first_bases_on_the_plus_strand() {
    let mut state: ViewState = ViewState::new(lin(), W, H);
    assert_eq!(state.selected(), Some(0), "synA is the only CDS");
    walk(&mut state, Key::Char('l'), 43);
    assert_eq!(state.cursor(), 43, "the middle base of codon 1");
    assert_eq!(state.handle(Key::Char('w')), Action::Continue);
    assert_eq!(state.cursor(), 45, "codon 2's first base");
    assert_eq!(state.handle(Key::Char('w')), Action::Continue);
    assert_eq!(state.cursor(), 48, "codon 3's first base");
    assert_eq!(state.handle(Key::Char('b')), Action::Continue);
    assert_eq!(state.cursor(), 45, "back to codon 2's first base");
}

/// Test 3: the same steps on the minus strand, where the codons run downwards:
/// from residue 1's third base, `b` goes to the codon below and `w` back up.
#[test]
fn w_and_b_step_first_bases_on_the_minus_strand() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert!(state.select_cds(Some(2)), "bla is usable");
    assert_eq!(state.mode(), &LaneMode::Cds(2), "so its own lane is drawn");
    walk(&mut state, Key::Char('l'), 4150);
    assert_eq!(state.cursor(), 4150, "residue 1's third base");
    assert_eq!(state.handle(Key::Char('b')), Action::Continue);
    assert_eq!(state.cursor(), 4149, "residue 2's first base");
    assert_eq!(state.handle(Key::Char('b')), Action::Continue);
    assert_eq!(state.cursor(), 4146, "residue 3's first base");
    assert_eq!(state.handle(Key::Char('w')), Action::Continue);
    assert_eq!(state.cursor(), 4149, "back up to residue 2's");
    assert_eq!(state.handle(Key::Char('w')), Action::Continue);
    assert_eq!(state.cursor(), 4152, "and up to residue 1's");
}

/// Test 4: the residue letter is still drawn over the codon's middle base
/// (D055) when go-to puts the cursor on the codon's first base — the column of
/// base 4151, not that of 4152.
#[test]
fn the_letter_stays_over_the_middle_base() {
    let mut state: ViewState = ViewState::new(pbr(), W, H);
    assert!(state.select_cds(Some(2)), "bla is usable");
    assert_eq!(
        state.mode(),
        &LaneMode::Cds(2),
        "and its residue line is drawn"
    );
    go_to(&mut state, "M1");
    assert_eq!(state.cursor(), 4152, "the codon's first base");

    // The cursor's own cell stands on base 4152, and the two cells to its left
    // hold the file's letters one-based 4151 and 4152, `c` and `a` — so those
    // three columns are the codon's three columns, reference order left to
    // right: zero-based 4150, 4151 and 4152.
    let rows: Vec<Vec<Cell>> = drawn(&state);
    let (row, column): (usize, usize) = cursor_cell(&rows);
    assert_eq!(
        symbol(&rows, row, column),
        "T",
        "the base the cursor stands on, the file's `t` at 4153"
    );
    assert_eq!(
        symbol(&rows, row, column - 1),
        "A",
        "and 4151's base below it"
    );
    assert_eq!(
        symbol(&rows, row, column - 2),
        "C",
        "and 4150's base below it"
    );

    // The cursor's codon on the residue line is those same three columns, on a
    // row other than the bases' (8m8), and its letter stands over the column of
    // 4151, the codon's middle base, leaving a space over the column of 4152,
    // which the cursor now stands on (D055).
    let marks: Vec<(usize, usize)> = marked(&rows);
    assert_eq!(marks.len(), 3, "the codon's three cells: {marks:?}");
    let mut lines: Vec<usize> = marks.iter().map(|(line, _column)| *line).collect();
    lines.dedup();
    assert_eq!(lines.len(), 1, "all three on one residue line: {marks:?}");
    let residues: usize = lines[0];
    assert_ne!(residues, row, "and that line is not one of the bases");
    assert_eq!(
        marks
            .iter()
            .map(|(_line, column)| *column)
            .collect::<Vec<usize>>(),
        vec![column - 2, column - 1, column],
        "the codon's own columns"
    );
    assert_eq!(
        symbol(&rows, residues, column - 1),
        "M",
        "the letter over its middle base 4151"
    );
    assert_eq!(
        symbol(&rows, residues, column),
        " ",
        "not over 4152, where the cursor stands"
    );
    assert_eq!(
        symbol(&rows, residues, column - 2),
        " ",
        "and nothing over 4150"
    );
}
