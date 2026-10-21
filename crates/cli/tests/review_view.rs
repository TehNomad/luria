//! `luria review`'s screen, drawn on a `ratatui` test backend and read back as
//! rows (milestone 7a, 7c3). Every row the screen asserts is the document's own
//! text: the state and the experimental validation come from the result the CLI
//! runs, never from a literal typed into the view (plan §3 rule 1, §3 rule 3),
//! and the size rule is §3.2's. The body rows come from `text`'s own functions
//! applied to the same document, so a row is checked against the line the screen
//! must show, not against what running the screen gave.
//!
//! The result documents are `luria_cli::run::run` on `edit-a42g.json` against
//! `synthetic-linear.gb`, in a directory under `CARGO_TARGET_TMPDIR` as
//! `tests/end_to_end.rs`'s `workspace` does. `state` `verified` and
//! `experimental_validation` `not_performed` are pinned by `tests/interpret_end_to_end.rs` and
//! `tests/fixtures/nl/README.md` (residue 42 of `synthetic-linear.gb` is Ala,
//! codon `GCA`), and `state` `failed` by §3.1: a request file that is not there
//! could not be prepared.

use luria_cli::result::ResultDocument;
use luria_cli::review::app::{App, Key, Screen};
use luria_cli::review::text;
use luria_cli::review::view;
use luria_cli::run::run;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use std::fs;
use std::path::PathBuf;

/// A fresh directory for one test, holding the fixture template and the request
/// that names it.
fn workspace(test: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("review-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs::copy(
        "../../tests/fixtures/genbank/synthetic-linear.gb",
        dir.join("synthetic-linear.gb"),
    )
    .unwrap();
    fs::copy(
        "../../tests/fixtures/nl/request/edit-a42g.json",
        dir.join("request.json"),
    )
    .unwrap();
    dir
}

/// The document `edit-a42g.json` runs to: `verified`, and designed. Pinned
/// here first, because the screen must show what it says.
fn verified(test: &str) -> ResultDocument {
    let outcome = run(&workspace(test).join("request.json"));
    assert_eq!(outcome.document.state, "verified");
    assert_eq!(outcome.document.experimental_validation, "not_performed");
    outcome.document
}

/// The document of a request file that is not in the workspace: `failed`, with
/// no member and no design to show.
fn failed(test: &str) -> ResultDocument {
    let outcome = run(&workspace(test).join("no-such-request.json"));
    assert_eq!(outcome.document.state, "failed");
    outcome.document
}

/// Draw `app` on a `width` by `height` test backend, and return every row as
/// the symbols of its cells, in screen order.
fn rows(app: &App, width: u16, height: u16) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| view::draw(frame, app)).unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = buffer.area.width as usize;
    buffer
        .content
        .chunks(width)
        .map(|cells| cells.iter().map(|cell| cell.symbol()).collect())
        .collect()
}

/// The same rows with the trailing spaces the screen pads with removed, so a row
/// can be compared with the text it shows.
fn screen(app: &App, width: u16, height: u16) -> Vec<String> {
    rows(app, width, height)
        .iter()
        .map(|row| row.trim_end().to_string())
        .collect()
}

/// The two-variant set, as `tests/review_text.rs`'s `set` builds it:
/// `variants-separate.json` beside a copy of `synthetic-linear.gb`.
fn set(test: &str) -> ResultDocument {
    let dir = workspace(test);
    fs::copy(
        "../../tests/fixtures/nl/request/variants-separate.json",
        dir.join("request.json"),
    )
    .unwrap();
    let document: ResultDocument = run(&dir.join("request.json")).document;
    assert_eq!(document.state, "verified");
    assert_eq!(document.results.len(), 2);
    document
}

/// `app` with `keys` handled, so each screen is reached the way the terminal
/// reaches it, through keys.
fn opened(document: &ResultDocument, keys: &[Key]) -> App {
    let mut app: App = App::new(document.clone());
    for key in keys {
        app.handle(*key);
    }
    app
}

/// A line as the screen shows it: cut at `width` characters, not wrapped, with
/// the padding the test backend pads the row with removed.
fn shown(line: &str, width: usize) -> String {
    let cut: String = line.chars().take(width).collect::<String>();
    cut.trim_end().to_string()
}

/// The body's rows at 80x24: rows 3 to `height - 2`, so 20 of the 24 rows.
const BODY_ROWS: usize = 20;

#[test]
fn the_state_and_validation_are_on_screen() {
    let app = App::new(verified("on_screen"));
    let screen = screen(&app, 80, 24);
    assert_eq!(
        screen
            .iter()
            .filter(|row| **row == "state: verified")
            .count(),
        1
    );
    assert_eq!(
        screen
            .iter()
            .filter(|row| **row == "experimental validation: not_performed")
            .count(),
        1
    );
    assert_eq!(
        screen.last().unwrap(),
        "q quit  enter open  j/k move  p provenance"
    );
}

#[test]
fn a_failed_result_shows_its_state() {
    let screen = screen(&App::new(failed("failed_state")), 80, 24);
    assert_eq!(
        screen.iter().filter(|row| **row == "state: failed").count(),
        1
    );
    assert!(
        screen.iter().all(|row| !row.contains("verified")),
        "a failed document's screen says nothing of a verified run: {:?}",
        screen
    );
}

#[test]
fn a_small_screen_says_so_and_shows_nothing_else() {
    let app = App::new(verified("small"));
    for (width, height) in [(79u16, 24u16), (80u16, 23u16)] {
        let screen = screen(&app, width, height);
        assert_eq!(
            screen[0],
            format!(
                "the screen is {}x{}; luria review needs at least 80x24",
                width, height
            ),
            "{}x{}",
            width,
            height
        );
        assert!(
            screen.iter().all(|row| !row.contains("state:")),
            "{}x{} shows the banner only: {:?}",
            width,
            height,
            screen
        );
    }
}

#[test]
fn the_screen_shows_the_document_not_a_literal() {
    let mut document = verified("literal");
    document.state = "needs_clarification";
    document.experimental_validation = "performed elsewhere";
    let screen = screen(&App::new(document), 80, 24);
    assert!(
        screen.contains(&"state: needs_clarification".to_string()),
        "{:?}",
        screen
    );
    assert!(
        screen.contains(&"experimental validation: performed elsewhere".to_string()),
        "{:?}",
        screen
    );
}

#[test]
fn every_screen_keeps_the_banner() {
    let document: ResultDocument = set("banner");
    let banner: Vec<String> = vec![
        format!("state: {}", document.state),
        format!(
            "experimental validation: {}",
            document.experimental_validation
        ),
    ];
    for (name, app, expected) in [
        ("overview", opened(&document, &[]), Screen::Overview),
        (
            "member",
            opened(&document, &[Key::Enter]),
            Screen::Member(0),
        ),
        (
            "context",
            opened(&document, &[Key::Enter, Key::Char('c')]),
            Screen::Context(0),
        ),
        (
            "provenance",
            opened(&document, &[Key::Char('p')]),
            Screen::Provenance,
        ),
    ] {
        assert_eq!(app.screen(), expected, "the {} screen is open", name);
        let rows: Vec<String> = screen(&app, 80, 24);
        assert_eq!(
            &rows[0..2],
            &banner[..],
            "the banner is rows 0 and 1 of the {} screen: {:?}",
            name,
            &rows[0..2]
        );
    }
}

#[test]
fn the_body_is_the_text_of_the_screen() {
    let document: ResultDocument = set("body_text");
    let lines: Vec<String> = text::provenance(&document);
    let app: App = opened(&document, &[Key::Char('p')]);
    assert_eq!(app.screen(), Screen::Provenance);
    let rows: Vec<String> = screen(&app, 80, 24);
    assert_eq!(rows[2], "", "row 2 is blank");
    for row in 0..BODY_ROWS {
        let expected: String = lines
            .get(row)
            .map(|line| shown(line, 80))
            .unwrap_or_default();
        assert_eq!(
            rows[3 + row],
            expected,
            "body row {} shows the screen's line {}",
            3 + row,
            row
        );
    }
}

#[test]
fn a_long_screen_scrolls_and_says_where() {
    let document: ResultDocument = set("long_scroll");
    let lines: Vec<String> = text::member(&document, 0);
    assert!(
        lines.len() > BODY_ROWS,
        "the member screen must be longer than the {} body rows: {} lines",
        BODY_ROWS,
        lines.len()
    );
    let n: usize = lines.len();
    let app: App = opened(&document, &[Key::Enter, Key::PageDown]);
    assert_eq!(app.screen(), Screen::Member(0));
    let rows: Vec<String> = screen(&app, 80, 24);
    let first: usize = 10.min(n - BODY_ROWS);
    assert_eq!(
        rows[3],
        shown(&lines[first], 80),
        "the first body row is line {} of the screen: {:?}",
        first,
        rows[3]
    );
    let key_line: &String = &rows[23];
    assert!(
        key_line.ends_with(&format!(
            "  lines {}-{} of {}",
            first + 1,
            first + BODY_ROWS,
            n
        )),
        "the key line says where the body is: {:?}",
        key_line
    );
    assert!(
        key_line.starts_with("q quit  esc back  c context  j/k scroll"),
        "the member's keys are on the key line: {:?}",
        key_line
    );
}

#[test]
fn scrolling_past_the_end_stops_at_the_last_page() {
    let document: ResultDocument = set("last_page");
    let lines: Vec<String> = text::member(&document, 0);
    let n: usize = lines.len();
    assert!(
        n > BODY_ROWS,
        "the screen is longer than the body: {} lines",
        n
    );
    let mut app: App = App::new(document);
    app.handle(Key::Enter);
    for _ in 0..100 {
        app.handle(Key::PageDown);
    }
    assert_eq!(app.screen(), Screen::Member(0));
    let rows: Vec<String> = screen(&app, 80, 24);
    let first: usize = n - BODY_ROWS;
    assert_eq!(
        rows[3 + BODY_ROWS - 1],
        shown(&lines[n - 1], 80),
        "the last body row is the screen's last line: {:?}",
        rows[3 + BODY_ROWS - 1]
    );
    assert_eq!(
        rows[23],
        format!(
            "q quit  esc back  c context  j/k scroll  lines {}-{} of {}",
            first + 1,
            first + BODY_ROWS,
            n
        ),
        "the key line says the last page: {:?}",
        rows[23]
    );
}

#[test]
fn the_key_line_offers_problem_only_with_a_problem() {
    // The set: `verified`, with no error and no clarification, so the overview
    // offers no problem screen.
    let rows: Vec<String> = screen(&App::new(set("key_line_set")), 80, 24);
    assert_eq!(
        rows[23], "q quit  enter open  j/k move  p provenance",
        "{:?}",
        rows[23]
    );

    // A `failed` result opens on the problem screen; from the overview, the key
    // that goes back to it is offered, because a problem is never hidden.
    let app: App = opened(&failed("key_line_failed"), &[Key::Esc]);
    assert_eq!(app.screen(), Screen::Overview);
    let rows: Vec<String> = screen(&app, 80, 24);
    assert_eq!(
        rows[23], "q quit  enter open  j/k move  p provenance  e problem",
        "{:?}",
        rows[23]
    );
}
