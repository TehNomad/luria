//! The words preview on the variant screens (contract 7m8b, M7-60, M7-61): when
//! the model's reading can be approved, `luria new` runs the design in memory and
//! opens on the result screens as a preview, with nothing saved, and the reading
//! itself at the bottom of each variant's Details screen.
//!
//! The preview is built as `7m8a`'s `a_preview_is_the_saved_run` builds it: the
//! corpus case `two-separate` through `interpret` with the scripted backend, then
//! `review::preview`, so the document, the template letters and the frames are the
//! saved run's, and the reading is `text::proposal` of the interpretation. Nothing
//! is written: the request path the preview names stays empty, as
//! `tests/review_approve.rs` proves.
//!
//! The screen is drawn on a `ratatui` test backend of 120 by 40 and read back as
//! rows, as `tests/review_view.rs` reads `luria review`'s screen: row 0 is the
//! frame's title, the body is rows 1 to `height - 3` (37 rows), and row
//! `height - 1` is the key line. Every expected value is the document's own, the
//! corpus's own, or `text`'s own lines, never a value copied from running the
//! screen.

use luria_cli::interpret::document::InterpretationDocument;
use luria_cli::interpret::run::interpret;
use luria_cli::interpret::run::InterpretInput;
use luria_cli::interpret::run::InterpretOutcome;
use luria_cli::interpret::run::TextSource;
use luria_cli::review::app::{Action, App, Key, Screen};
use luria_cli::review::preview;
use luria_cli::review::text;
use luria_cli::review::view;
use luria_cli::review::Approved;
use luria_nl::backend::ScriptedBackend;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::Terminal;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

const NL: &str = "../../tests/fixtures/nl/";
const GENBANK: &str = "../../tests/fixtures/genbank/";
const CASES_HEADER: &str = "case_id\ttemplate\ttext_file\treply_file\tprofile\tstate\tcodes\trequest_file\tdesign_exit\tnote";

/// The screen the preview is drawn on: 120 columns by 40 rows, at or above the
/// `MIN_WIDTH` by `MIN_HEIGHT` the review needs.
const WIDTH: u16 = 120;
const HEIGHT: u16 = 40;
/// The body's rows at that size: `height - 3`, the frame's inner area, so rows 1
/// to 37, and row 37 is the last body row.
const BODY_ROWS: usize = HEIGHT as usize - 3;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("cannot read {}: {}", path, error))
}

fn cases() -> Vec<Vec<String>> {
    let text: String = read(&format!("{}cases.tsv", NL));
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines[0], CASES_HEADER);
    lines[1..]
        .iter()
        .map(|line| line.split('\t').map(|field| field.to_string()).collect())
        .collect()
}

/// The corpus's own row for one case.
fn row_of(case: &str) -> Vec<String> {
    cases()
        .into_iter()
        .find(|row| row[0] == case)
        .expect("the corpus has this case")
}

/// A fresh directory for one case: its template and, when it has one, its
/// profile, copied in under their own names, as `tests/review_approve.rs` does.
fn workspace(test: &str, row: &[String]) -> PathBuf {
    let dir: PathBuf = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("review-preview-{}-{}", test, row[0]));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("a workspace");
    fs::copy(format!("{}{}", GENBANK, row[1]), dir.join(&row[1])).expect("the template is copied");
    if row[4] != "-" {
        fs::copy(format!("{}profile/{}", NL, row[4]), dir.join(&row[4]))
            .expect("the profile is copied");
    }
    dir
}

/// The case's text, without its file's final newline.
fn text_of(row: &[String]) -> String {
    let text: String = read(&format!("{}text/{}", NL, row[2]));
    text.strip_suffix('\n').unwrap_or(&text).to_string()
}

/// The case, interpreted in its own directory with `write_request: None`, as
/// `tests/review_approve.rs` runs it: the run writes nothing.
fn interpret_case(dir: &Path, row: &[String]) -> InterpretOutcome {
    let reply: String = read(&format!("{}reply/{}", NL, row[3]));
    let mut backend: ScriptedBackend = ScriptedBackend::new("scripted-corpus", vec![Ok(reply)]);
    let input: InterpretInput = InterpretInput {
        template: dir.join(&row[1]),
        text: TextSource::Inline(text_of(row)),
        profile: if row[4] == "-" {
            None
        } else {
            Some(dir.join(&row[4]))
        },
        write_request: None,
    };
    interpret(&input, &mut backend)
}

/// The case, run in a fresh directory, with the request path the save names beside
/// the template: its workspace, the interpretation, and that path.
fn proposal_of(test: &str, case: &str) -> (PathBuf, InterpretationDocument, PathBuf) {
    let row: Vec<String> = row_of(case);
    let dir: PathBuf = workspace(test, &row);
    let outcome: InterpretOutcome = interpret_case(&dir, &row);
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

/// The preview as `luria new` builds it (contract 7m8b §2.4): the case's proposal,
/// run in memory with nothing written, in an app with the template letters, the
/// frames and the model's reading. The request path is named so a test can say no
/// file was written.
fn preview_of(test: &str, case: &str) -> (PathBuf, App, Vec<String>, PathBuf) {
    let (dir, document, request) = proposal_of(test, case);
    assert_eq!(document.state, "proposed", "the case proposes a request");
    let found: Approved = match preview(&document, &request) {
        Ok(approved) => approved,
        Err(problem) => panic!("a proposed request has a preview: {}", problem),
    };
    assert_eq!(
        found.document.state, "verified",
        "the preview is the design run"
    );
    assert_eq!(found.document.experimental_validation, "not_performed");
    let reading: Vec<String> = text::proposal(&document);
    let app: App = App::new(found.document)
        .with_template(found.template)
        .with_frames(found.frames)
        .with_reading(reading.clone());
    (dir, app, reading, request)
}

/// A body row as the text it shows: the frame's right border and the padding the
/// test backend pads the row with removed, so a row can be compared with the line
/// drawn in it.
fn body_row(row: &str) -> String {
    row.strip_suffix('│')
        .expect("the frame's right border")
        .trim_end()
        .to_string()
}

/// `app` drawn on a `WIDTH` by `HEIGHT` test backend, with every row as the
/// symbols of its cells, the trailing padding removed.
fn screen(app: &App, width: u16, height: u16) -> Vec<String> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| view::draw(frame, app)).unwrap();
    let buffer = terminal.backend().buffer();
    let width: usize = buffer.area.width as usize;
    buffer
        .content
        .chunks(width)
        .map(|cells| cells.iter().map(|cell| cell.symbol()).collect::<String>())
        .map(|row| row.trim_end().to_string())
        .collect()
}

/// The key line, the row below the frame's bottom border.
fn key_line(app: &App) -> String {
    screen(app, WIDTH, HEIGHT)[HEIGHT as usize - 1].clone()
}

/// `app` with `keys` handled, so each screen is reached the way the terminal
/// reaches it, through keys.
fn opened(app: &mut App, keys: &[Key]) {
    for key in keys {
        app.handle(*key);
    }
}

/// The area a `WIDTH` by `HEIGHT` terminal gives `max_scroll`, as `terminal`
/// passes it after every key (M7-56).
fn area() -> Rect {
    Rect::new(0, 0, WIDTH, HEIGHT)
}

/// The reading as `preview_details` draws it: every line indented by two spaces,
/// an empty line kept empty.
fn indented(reading: &[String]) -> Vec<String> {
    reading
        .iter()
        .map(|line| {
            if line.is_empty() {
                String::new()
            } else {
                format!("  {}", line)
            }
        })
        .collect()
}

// M7-60: the title says the screen is a preview, and the state is still on it
// (D020). The experimental validation words are gone.
#[test]
fn a_preview_says_nothing_is_saved() {
    let (dir, app, _reading, request) = preview_of("title", "two-separate");
    assert!(
        !request.exists(),
        "the preview wrote no file at {:?}",
        request
    );

    let rows: Vec<String> = screen(&app, WIDTH, HEIGHT);
    let title: &String = &rows[0];
    assert!(
        title.contains("✓ verified"),
        "the state badge stays on the title row: {}",
        title
    );
    assert!(
        title.contains("preview, nothing is saved yet"),
        "the title names the preview: {}",
        title
    );
    assert!(
        !rows
            .iter()
            .any(|row| row.contains("not tested at the bench")),
        "the validation words are not on a preview: {:?}",
        rows[0]
    );
    let _ = fs::remove_dir_all(&dir);
}

// M7-60: `a` asks for the save on every variant screen of a preview, and on no
// other screen, so `a` on the overview of a result asks for nothing.
#[test]
fn a_saves_from_the_variant_screens() {
    let (dir, mut app, _reading, request) = preview_of("save", "two-separate");
    assert_eq!(
        app.screen(),
        Screen::Overview,
        "the preview opens on the overview"
    );
    assert_eq!(
        app.handle(Key::Char('a')),
        Action::Approve,
        "`a` on the overview of a preview asks for the save"
    );
    assert_eq!(
        app.screen(),
        Screen::Overview,
        "and the screen stays for the pop-up to sit over"
    );

    opened(&mut app, &[Key::Enter]);
    assert_eq!(app.screen(), Screen::Member(0), "`enter` opens the member");
    assert_eq!(
        app.handle(Key::Char('a')),
        Action::Approve,
        "`a` on a sequence screen asks for the save"
    );
    assert_eq!(app.screen(), Screen::Member(0));

    opened(&mut app, &[Key::Char('d')]);
    assert_eq!(app.screen(), Screen::Details(0), "`d` opens its details");
    assert_eq!(
        app.handle(Key::Char('a')),
        Action::Approve,
        "`a` on a details screen asks for the save"
    );
    assert_eq!(app.screen(), Screen::Details(0));

    // Without a reading, `a` behaves as it always did: it asks for nothing.
    let mut plain: App = App::new(app.document().clone());
    assert_eq!(plain.reading(), None, "a result has no reading");
    assert_eq!(
        plain.handle(Key::Char('a')),
        Action::Continue,
        "`a` on the overview of a result asks for nothing"
    );
    assert_eq!(plain.screen(), Screen::Overview);
    assert!(
        !request.exists(),
        "asking for the save writes nothing here: no file at {:?}",
        request
    );
    let _ = fs::remove_dir_all(&dir);
}

// M7-60: a preview has no export, so `x` on its overview asks for nothing, and
// its key line names `a save` and not `x export`.
#[test]
fn a_preview_has_no_export() {
    let (dir, mut app, _reading, _request) = preview_of("no-export", "two-separate");
    assert_eq!(
        app.handle(Key::Char('x')),
        Action::Continue,
        "`x` on a preview's overview asks for nothing"
    );
    assert_eq!(
        app.screen(),
        Screen::Overview,
        "and stays on the overview, so it leaves the screen"
    );

    let keys: String = key_line(&app);
    assert!(
        keys.contains("a save"),
        "the overview's key line names the save key: {}",
        keys
    );
    assert!(!keys.contains("x export"), "and names no export: {}", keys);

    // A result's overview keeps `x export`, and names no save.
    let plain: App = App::new(app.document().clone());
    let keys: String = key_line(&plain);
    assert!(keys.contains("x export"), "a result's key line: {}", keys);
    assert!(!keys.contains("a save"), "and no save key: {}", keys);
    let _ = fs::remove_dir_all(&dir);
}

// M7-61: a preview's details are `details`' own lines, then the heading, then the
// reading indented by two spaces, and the screen `d` opens shows them at its
// bottom, where `PgDn` reaches.
#[test]
fn details_end_with_the_models_reading() {
    let (dir, document, request) = proposal_of("details", "two-separate");
    let found: Approved = match preview(&document, &request) {
        Ok(approved) => approved,
        Err(problem) => panic!("a proposed request has a preview: {}", problem),
    };
    let reading: Vec<String> = text::proposal(&document);
    assert!(
        !reading.is_empty(),
        "the reading has lines to show at the bottom of Details"
    );

    let lines: Vec<String> = text::preview_details(&found.document, 0, &reading);
    let plain: Vec<String> = text::details(&found.document, 0);
    assert_eq!(&lines[..plain.len()], &plain[..], "its start is `details`");
    assert_eq!(lines[plain.len()], String::new(), "then one empty line");
    assert_eq!(
        lines[plain.len() + 1],
        "the model's reading of your words:",
        "then the heading"
    );
    let shown: Vec<String> = indented(&reading);
    assert_eq!(
        &lines[plain.len() + 2..],
        &shown[..],
        "then the reading, indented by two spaces, empty line kept empty"
    );

    // The drawn rule (contract §3.1): the reading is on the screen, at its
    // bottom. `End` is not a key, so `PgDn` is pressed until the scroll stops, as
    // `terminal` clamps it after every key (M7-56).
    let mut app: App = App::new(found.document.clone())
        .with_template(found.template)
        .with_frames(found.frames)
        .with_reading(reading.clone());
    opened(&mut app, &[Key::Enter, Key::Char('d')]);
    assert_eq!(app.screen(), Screen::Details(0));
    loop {
        let before: usize = app.scroll();
        app.handle(Key::PageDown);
        app.clamp_scroll(view::max_scroll(&app, area()));
        if app.scroll() == before {
            break;
        }
    }
    assert!(
        app.scroll() > 0,
        "the details of a preview are longer than the body, so `PgDn` scrolls"
    );
    let rows: Vec<String> = screen(&app, WIDTH, HEIGHT);
    let last_line: &String = shown.last().expect("the reading's last line");
    assert_eq!(
        body_row(&rows[BODY_ROWS]),
        format!("│{}", last_line.trim_end()),
        "the last body row is the reading's last line"
    );
    assert!(
        !body_row(&rows[BODY_ROWS]).contains("not tested at the bench"),
        "and the reading's last line is the reading's own: {:?}",
        rows[BODY_ROWS]
    );
    let _ = fs::remove_dir_all(&dir);
}

// M7-60: `?` on a preview's variant screen names the key those screens add, and
// the plain legend does not name it.
#[test]
fn the_help_names_the_save_key() {
    let save: &str = "  a          save and run";
    let previewed: Vec<String> = text::preview_help(Screen::Member(0), false, false);
    let plain: Vec<String> = text::help(Screen::Member(0), false, false);
    assert!(
        previewed.iter().any(|line| line == save),
        "the preview's legend names the save key: {:?}",
        previewed
    );
    assert!(
        !plain.iter().any(|line| line == save),
        "`help` itself names no save key: {:?}",
        plain
    );
    // `a` is named just before `q`, the key that follows it in the legend.
    let at: usize = previewed
        .iter()
        .position(|line| line == save)
        .expect("the save line is in the legend");
    assert_eq!(
        previewed[at + 1],
        "  q          quit",
        "`a` is the line before `q`, whose key is padded to 11 as `help` pads it"
    );
    assert_eq!(
        previewed.last(),
        plain.last(),
        "and `ctrl-c` stays the last line"
    );
    assert_eq!(
        plain.len() + 1,
        previewed.len(),
        "the legend is `help`'s lines plus one"
    );
}
