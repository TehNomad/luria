//! `luria review`'s screen, drawn on a `ratatui` test backend and read back as
//! rows (milestone 7a). Every row the screen asserts is the document's own
//! text: the state and the experimental validation come from the result the CLI
//! runs, never from a literal typed into the view (plan §3 rule 1, §3 rule 3),
//! and the size rule is §3.2's.
//!
//! The result documents are `luria_cli::run::run` on `edit-a42g.json` against
//! `synthetic-linear.gb`, in a directory under `CARGO_TARGET_TMPDIR` as
//! `tests/end_to_end.rs`'s `workspace` does. `state` `verified` and
//! `experimental_validation` `not_performed` are pinned by `tests/interpret_end_to_end.rs` and
//! `tests/fixtures/nl/README.md` (residue 42 of `synthetic-linear.gb` is Ala,
//! codon `GCA`), and `state` `failed` by §3.1: a request file that is not there
//! could not be prepared.

use luria_cli::result::ResultDocument;
use luria_cli::review::app::App;
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
    assert_eq!(screen.last().unwrap(), "q quit");
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
