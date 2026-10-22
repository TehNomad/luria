//! The `luria new` form's screen (milestone 7k4, contract 7k4 §4.2): drawn on a
//! 80x24 `ratatui` test backend and read back as rows, colours and the cursor,
//! as `tests/review_view.rs` reads the review's screen. The form is built with
//! keys and `loaded`, as `tests/new_form.rs` builds it, so every row on the
//! screen is the state 7k3 holds (plan §3 rule 1).

use luria_cli::new::form::Field;
use luria_cli::new::form::FormAction;
use luria_cli::new::form::NewForm;
use luria_cli::new::view;
use luria_cli::review::app::Key;
use ratatui::backend::TestBackend;
use ratatui::layout::Position;
use ratatui::style::{Color, Modifier, Style};
use ratatui::Terminal;
use std::fs;

const FIXTURES: &str = "../../tests/fixtures/";
const SID1: &str = "example-construct/input/sid-1_cDNA.fa";

/// One fixture template, read at test time, as `tests/new_form.rs` does.
fn read_file(relative: &str) -> String {
    let path: String = format!("{}{}", FIXTURES, relative);
    fs::read_to_string(&path).unwrap_or_else(|problem| panic!("cannot read {}: {}", path, problem))
}

/// Types `text` one character at a time, as a user presses the keys.
fn type_text(form: &mut NewForm, text: &str) {
    for ch in text.chars() {
        form.handle(Key::Char(ch));
    }
}

/// `Tab` forward until focus reaches `wanted`, as `tests/new_form.rs` does.
fn tab_to(form: &mut NewForm, wanted: Field) {
    for _ in 0..30 {
        if form.focus() == wanted {
            return;
        }
        form.handle(Key::Tab);
    }
    panic!("focus never reached {:?}", wanted);
}

/// The form with the `sid-1` file loaded as the loop answers `Load`, as
/// `tests/new_form.rs` builds it.
fn sid1(lines: &[&str], overlap: &str) -> NewForm {
    let mut form = NewForm::new(None, false);
    tab_to(&mut form, Field::File);
    type_text(&mut form, "sid-1_cDNA.fa");
    assert_eq!(
        form.handle(Key::Tab),
        FormAction::Load("sid-1_cDNA.fa".to_string()),
        "leaving File with a path asks the loop to read it"
    );
    form.loaded("sid-1_cDNA.fa", Ok(read_file(SID1)));
    tab_to(&mut form, Field::Name);
    let filled: usize = form.name().chars().count();
    for _ in 0..filled {
        form.handle(Key::Backspace);
    }
    type_text(&mut form, "sid-1");
    tab_to(&mut form, Field::Mutations);
    for line in lines.iter() {
        type_text(&mut form, line);
        form.handle(Key::Enter);
    }
    tab_to(&mut form, Field::Overlap);
    // M7-45: the prefilled 15 is typed over, so `overlap` is the value the case
    // states, empty included.
    let prefilled: usize = form.overlap().chars().count();
    for _ in 0..prefilled {
        form.handle(Key::Backspace);
    }
    type_text(&mut form, overlap);
    form
}

/// Draw `form` on a `width` by `height` test backend, and return every row as
/// the symbols of its cells, in screen order.
fn rows(form: &NewForm, width: u16, height: u16) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| view::draw_form(frame, form)).unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = buffer.area.width as usize;
    buffer
        .content
        .chunks(width)
        .map(|cells| cells.iter().map(|cell| cell.symbol()).collect())
        .collect()
}

/// `form` drawn on a `width` by `height` test backend, with the style of every
/// cell in screen order, as `tests/review_view.rs` reads styles.
fn styles(form: &NewForm, width: u16, height: u16) -> Vec<Vec<Style>> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| view::draw_form(frame, form)).unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = buffer.area.width as usize;
    buffer
        .content
        .chunks(width)
        .map(|cells| cells.iter().map(|cell| cell.style()).collect())
        .collect()
}

/// `form` drawn on a `width` by `height` test backend, with the cursor the
/// backend is left at.
fn cursor(form: &NewForm, width: u16, height: u16) -> Position {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| view::draw_form(frame, form)).unwrap();
    terminal.backend().cursor_position()
}

/// `form` drawn by `draw_waiting` on a `width` by `height` test backend, read
/// back as rows, as `rows` reads `draw_form`'s.
fn waiting_rows(form: &NewForm, width: u16, height: u16) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| view::draw_waiting(frame, form))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = buffer.area.width as usize;
    buffer
        .content
        .chunks(width)
        .map(|cells| cells.iter().map(|cell| cell.symbol()).collect())
        .collect()
}

#[test]
fn the_form_has_its_badge() {
    let form: NewForm = sid1(&["D404N"], "15");
    let screen: Vec<String> = rows(&form, 80, 24);
    let title: &str = &screen[0];
    assert!(
        title.contains("NEW DESIGN"),
        "the badge is the form's own: {}",
        title
    );
    assert!(
        title.contains("nothing is written until you approve"),
        "the title says nothing is written: {}",
        title
    );
    assert!(
        !title.contains("verified"),
        "a form has no result state to badge: {}",
        title
    );
}

#[test]
fn the_cursor_sits_in_the_focused_field() {
    let mut form: NewForm = NewForm::new(None, false);
    tab_to(&mut form, Field::File);
    type_text(&mut form, "sid");
    let at: Position = cursor(&form, 80, 24);
    // The `File` row: the frame's one border column, the label padded to 11
    // characters, then the 3 characters typed (measured 2026-10-22).
    assert_eq!(
        at.x,
        1 + 11 + 3,
        "the cursor sits after `sid` on the File row"
    );
    let screen: Vec<String> = rows(&form, 80, 24);
    let file_row: usize = screen
        .iter()
        .position(|row| row.contains("  File"))
        .expect("the File row is on the screen");
    assert_eq!(at.y, file_row as u16, "the cursor is on the File row");
}

#[test]
fn a_refused_preview_is_red() {
    let mut form: NewForm = sid1(&["D404N", "D404*"], "15");
    tab_to(&mut form, Field::Review);
    form.handle(Key::Enter);
    let screen: Vec<String> = rows(&form, 80, 24);
    let row: usize = screen
        .iter()
        .position(|line| line.contains("✗ 2  "))
        .expect("the refused line's preview is on the screen");
    let column: usize = screen[row].find('✗').expect("the mark is on its row");
    assert_eq!(
        styles(&form, 80, 24)[row][column].fg,
        Some(Color::Red),
        "a refused preview is red"
    );
}

/// The `sid-1` form with a backend (M7-47): the file loaded as the loop answers
/// `Load`, the name typed over the stem, `Overlap` at its prefilled 15 (M7-45),
/// and `words` typed in the `Mutations` box, whose lines luria cannot read.
fn words_form(words: &str) -> NewForm {
    let mut form = NewForm::new(None, true);
    tab_to(&mut form, Field::File);
    type_text(&mut form, "sid-1_cDNA.fa");
    assert_eq!(
        form.handle(Key::Tab),
        FormAction::Load("sid-1_cDNA.fa".to_string()),
        "leaving File with a path asks the loop to read it"
    );
    form.loaded("sid-1_cDNA.fa", Ok(read_file(SID1)));
    tab_to(&mut form, Field::Name);
    let filled: usize = form.name().chars().count();
    for _ in 0..filled {
        form.handle(Key::Backspace);
    }
    type_text(&mut form, "sid-1");
    tab_to(&mut form, Field::Overlap);
    // M7-45: `Overlap` opens prefilled with 15, so nothing is typed there.
    tab_to(&mut form, Field::Mutations);
    type_text(&mut form, words);
    form
}

// M7-47 (contract §2.2): words with no loaded GenBank record say the template is
// needed, dim in the view, as the box's own hint, so the typed words read as the
// user's own text.
#[test]
fn a_genbank_need_is_dim() {
    let form: NewForm = words_form("A42G is a substitution");
    let screen: Vec<String> = rows(&form, 80, 24);
    let row: usize = screen
        .iter()
        .position(|line| line.contains("needs a GenBank template"))
        .expect("the words box says a GenBank template is needed");
    // The cell under the `n`, counted in characters: the frame's left border and
    // the 11-column label field.
    let column: usize = screen[row].chars().take_while(|ch| *ch != 'n').count();
    assert_eq!(
        column, 12,
        "the row is written at the label column, inside the frame"
    );
    assert!(
        styles(&form, 80, 24)[row][column].has_modifier(Modifier::DIM),
        "the hint is dim in the view, as the box's preview the model reads is"
    );
}

// 7k9 §2.4: the screen the loop shows while it waits for the model names the
// call on the key line, and is `draw_form`'s screen in every other row, so the
// user sees the form they typed while the call runs.
#[test]
fn waiting_names_the_model_call() {
    const KEYS: &str = "asking the model; this screen answers when it replies";
    let form: NewForm = sid1(&["D404N"], "15");
    let drawn: Vec<String> = rows(&form, 80, 24);
    let waiting: Vec<String> = waiting_rows(&form, 80, 24);
    assert_eq!(
        waiting[23].trim_end(),
        KEYS,
        "the key line says the model is being asked: {:?}",
        waiting[23]
    );
    assert_eq!(
        waiting[..23],
        drawn[..23],
        "every row but the key line is the form's own screen"
    );
    assert_ne!(
        waiting[23], drawn[23],
        "the key line is the only row that differs"
    );
}

// 7m1c: `option_runs` takes the first character's style from the character
// itself, so a choice row written at the value column, whose text starts with
// `◉`, draws that mark green as a `◉` in the middle of a row does.
#[test]
fn a_leading_choice_mark_is_green() {
    let form: NewForm = sid1(&["D404N"], "15");
    let screen: Vec<String> = rows(&form, 80, 24);
    // A choice row's text starts right after the 11-column label, so its first
    // character sits at column 12, inside the frame's left border column.
    let row: usize = screen
        .iter()
        .position(|line| line.chars().nth(12) == Some('◉'))
        .expect("a choice row whose text starts with ◉");
    let styles: Vec<Vec<Style>> = styles(&form, 80, 24);
    assert_eq!(
        styles[row][12].fg,
        Some(Color::Green),
        "the leading ◉ on the row {:?} is green: {:?}",
        styles[row][12],
        screen[row]
    );
}
