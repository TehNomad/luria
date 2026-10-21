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

use luria_cli::interpret::document::InterpretationDocument;
use luria_cli::interpret::run::interpret;
use luria_cli::interpret::run::InterpretInput;
use luria_cli::interpret::run::InterpretOutcome;
use luria_cli::interpret::run::TextSource;
use luria_cli::result::ResultDocument;
use luria_cli::review::app::{App, Key, Screen};
use luria_cli::review::proposal::{ProposalApp, ProposalScreen};
use luria_cli::review::text;
use luria_cli::review::view;
use luria_cli::run::run;
use luria_nl::backend::ScriptedBackend;
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
        "q quit  enter open  j/k move  p provenance  x export"
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
        rows[23], "q quit  enter open  j/k move  p provenance  x export",
        "{:?}",
        rows[23]
    );

    // A `failed` result opens on the problem screen; from the overview, the key
    // that goes back to it is offered, because a problem is never hidden.
    let app: App = opened(&failed("key_line_failed"), &[Key::Esc]);
    assert_eq!(app.screen(), Screen::Overview);
    let rows: Vec<String> = screen(&app, 80, 24);
    assert_eq!(
        rows[23], "q quit  enter open  j/k move  p provenance  x export  e problem",
        "{:?}",
        rows[23]
    );
}

// The proposal screens of 7f2: the documents are corpus cases run through the
// scripted backend as `tests/review_proposal.rs` runs them, with
// `write_request: None`, so the screen's content is the corpus's own answer, and
// the request path is the one `--request` names. The banner, the confirmation's
// line and the key line are checked against the document's own state and the
// path as given, not against what running the screen gave.
const NL: &str = "../../tests/fixtures/nl/";
const GENBANK: &str = "../../tests/fixtures/genbank/";
const CASES_HEADER: &str = "case_id\ttemplate\ttext_file\treply_file\tprofile\tstate\tcodes\trequest_file\tdesign_exit\tnote";

fn nl_read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("cannot read {}: {}", path, error))
}

fn row_of(case: &str) -> Vec<String> {
    let text: String = nl_read(&format!("{}cases.tsv", NL));
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], CASES_HEADER);
    lines[1..]
        .iter()
        .map(|line| line.split('\t').map(|field| field.to_string()).collect())
        .find(|row: &Vec<String>| row[0] == case)
        .expect("the corpus has this case")
}

/// A fresh directory for one corpus case, with its template copied in under its
/// own name, as `tests/review_proposal.rs` does.
fn proposal_workspace(test: &str, row: &[String]) -> PathBuf {
    let dir: PathBuf =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("review-view-{}-{}", test, row[0]));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a workspace");
    fs::copy(format!("{}{}", GENBANK, row[1]), dir.join(&row[1])).expect("the template is copied");
    if row[4] != "-" {
        fs::copy(format!("{}profile/{}", NL, row[4]), dir.join(&row[4]))
            .expect("the profile is copied");
    }
    dir
}

/// The case, run through `interpret` with the scripted backend and
/// `write_request: None`: the document the proposal screen shows, with the
/// request path `--request` names beside the template. The state is checked
/// against the corpus's own column first, so the banner is the corpus's state.
fn proposal_of(test: &str, case: &str) -> (PathBuf, InterpretationDocument, PathBuf) {
    let row: Vec<String> = row_of(case);
    let dir: PathBuf = proposal_workspace(test, &row);
    let reply: String = nl_read(&format!("{}reply/{}", NL, row[3]));
    let mut backend: ScriptedBackend = ScriptedBackend::new("scripted-corpus", vec![Ok(reply)]);
    let text: String = nl_read(&format!("{}text/{}", NL, row[2]));
    let input: InterpretInput = InterpretInput {
        template: dir.join(&row[1]),
        text: TextSource::Inline(text.strip_suffix('\n').unwrap_or(&text).to_string()),
        profile: if row[4] == "-" {
            None
        } else {
            Some(dir.join(&row[4]))
        },
        write_request: None,
    };
    let outcome: InterpretOutcome = interpret(&input, &mut backend);
    assert_eq!(
        outcome.document.state, row[5],
        "{}: the corpus's own state column",
        row[0]
    );
    (
        dir.clone(),
        outcome.document.clone(),
        dir.join("proposal.json"),
    )
}

/// Draw `app`' proposal on a `width` by `height` test backend, with the padding
/// the backend pads each row with removed.
fn proposal_screen(app: &ProposalApp, width: u16, height: u16) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal
        .draw(|frame| view::draw_proposal(frame, app))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = buffer.area.width as usize;
    buffer
        .content
        .chunks(width)
        .map(|cells| {
            cells
                .iter()
                .map(|cell| cell.symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

/// `app` with `keys` handled, the way the terminal reaches a screen.
fn opened_proposal(
    document: &InterpretationDocument,
    request: PathBuf,
    keys: &[Key],
) -> ProposalApp {
    let mut app: ProposalApp = ProposalApp::new(document.clone(), request);
    for key in keys {
        app.handle(*key);
    }
    app
}

#[test]
fn a_proposal_says_it_is_not_a_result() {
    for case in ["two-separate", "composition-unstated"] {
        let (dir, document, request) = proposal_of("banner", case);
        let rows: Vec<String> =
            proposal_screen(&ProposalApp::new(document.clone(), request), 80, 24);
        assert_eq!(
            rows[0],
            format!("proposal: {}", document.state),
            "row 0 is the banner built from the document's own state: {:?}",
            rows[0]
        );
        assert_eq!(
            rows[1], "nothing has run yet: this is a proposal, not a result",
            "row 1 says nothing has run: a proposal is never shown as a result (plan §3 rule 3)"
        );
        assert!(
            !rows[0].starts_with("state:") && !rows[1].contains("experimental validation"),
            "the result's banner is not on a proposal's screen: {:?}",
            &rows[0..2]
        );
        assert_eq!(rows[2], "", "row 2 is blank, as every result screen's is");
        let _ = fs::remove_dir_all(&dir);
    }
}

#[test]
fn the_confirmation_names_the_file() {
    let (dir, document, _) = proposal_of("confirm", "two-separate");
    // The path as `--request` gives it, relative to the directory the command
    // runs in, so the whole line fits on the screen.
    let request: PathBuf = PathBuf::from("proposal.json");
    let app: ProposalApp = opened_proposal(&document, request.clone(), &[Key::Char('a')]);
    assert_eq!(app.screen(), ProposalScreen::Confirm);
    let rows: Vec<String> = proposal_screen(&app, 80, 24);
    assert_eq!(
        rows[3],
        format!(
            "write the request to {} and run luria q5 design on it?",
            request.display()
        ),
        "the confirmation names the file the approval writes (plan §3 rule 4): {:?}",
        rows[3]
    );
    assert_eq!(rows[4], "", "a blank line between the file and the keys");
    assert_eq!(rows[5], "y yes  n no");
    assert_eq!(rows[23], "q quit  y yes  n no");
    assert!(
        rows[3..BODY_ROWS + 3]
            .iter()
            .all(|row| !row.contains("proposal:")),
        "the confirmation shows the file, not the proposal: {:?}",
        rows
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn approve_is_offered_only_for_a_proposal() {
    // The proposal: `text::proposal`' own lines (7e), so the key line's page
    // count is the screen's own length, and `a` is offered because the document
    // is `proposed` and names its request.
    let (proposed_dir, proposed, _) = proposal_of("key_line_proposed", "two-separate");
    let lines: Vec<String> = text::proposal(&proposed);
    let n: usize = lines.len();
    assert!(
        n > BODY_ROWS,
        "the proposal is longer than the body: {} lines",
        n
    );
    let app: ProposalApp = ProposalApp::new(proposed.clone(), PathBuf::from("proposal.json"));
    let rows: Vec<String> = proposal_screen(&app, 80, 24);
    assert_eq!(
        rows[23],
        format!(
            "q quit  j/k scroll  a approve  lines 1-{} of {}",
            BODY_ROWS, n
        ),
        "a proposal that names its request offers `a`, and says where its body sits: {:?}",
        rows[23]
    );
    for row in 0..BODY_ROWS {
        assert_eq!(
            rows[3 + row],
            shown(&lines[row], 80),
            "body row {} shows the proposal's line {}",
            3 + row,
            row
        );
    }

    // A run that asks proposes nothing, so `a` is not on its key line, and the
    // document's `request` is null.
    let (asked_dir, asked, _) = proposal_of("key_line_asked", "composition-unstated");
    assert!(asked.request.is_none(), "a run that asks names no request");
    let lines: Vec<String> = text::proposal(&asked);
    let n: usize = lines.len();
    let rows: Vec<String> = proposal_screen(
        &ProposalApp::new(asked, PathBuf::from("proposal.json")),
        80,
        24,
    );
    assert!(
        !rows[23].contains("a approve"),
        "nothing to approve, so `a` is not offered: {:?}",
        rows[23]
    );
    assert!(
        rows[23].starts_with("q quit  j/k scroll"),
        "the reading keys are the same: {:?}",
        rows[23]
    );
    assert_eq!(
        rows[23],
        format!("q quit  j/k scroll  lines 1-{} of {}", BODY_ROWS, n),
        "and the page count is the asked screen's own length: {:?}",
        rows[23]
    );
    let _ = fs::remove_dir_all(&proposed_dir);
    let _ = fs::remove_dir_all(&asked_dir);
}
