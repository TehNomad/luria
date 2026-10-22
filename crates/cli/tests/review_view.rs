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
use luria_cli::prepare::prepare;
use luria_cli::result::DesignView;
use luria_cli::result::MutationView;
use luria_cli::result::PrimerView;
use luria_cli::result::PrimerWarningView;
use luria_cli::result::ResultDocument;
use luria_cli::review::app::{App, Key, Screen};
use luria_cli::review::matching_template;
use luria_cli::review::member_frames;
use luria_cli::review::proposal::{ProposalApp, ProposalScreen};
use luria_cli::review::text;
use luria_cli::review::view;
use luria_cli::review::MemberFrame;
use luria_cli::run::run;
use luria_cli::run::run_with_constructs;
use luria_nl::backend::ScriptedBackend;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
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
        "q quit  enter open  j/k move  p provenance  x export  ? help"
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
            "details",
            opened(&document, &[Key::Enter, Key::Char('d')]),
            Screen::Details(0),
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
    let app: App = opened(&document, &[Key::Enter, Key::Char('d'), Key::PageDown]);
    assert_eq!(app.screen(), Screen::Details(0));
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
        key_line.starts_with("q quit  esc back  j/k scroll"),
        "the technical screen's keys are on the key line: {:?}",
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
    app.handle(Key::Char('d'));
    for _ in 0..100 {
        app.handle(Key::PageDown);
    }
    assert_eq!(app.screen(), Screen::Details(0));
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
            "q quit  esc back  j/k scroll  ? help  lines {}-{} of {}",
            first + 1,
            first + BODY_ROWS,
            n
        ),
        "the key line says the last page: {:?}",
        rows[23]
    );
}

// The redesigned member screen of 7i4 (plan §5.3a): `enter` opens the sequence
// screen, `t` the technical lines, `i` writes a long insertion out, and the
// overview names the template, the target and the members only. The request
// builders below are `tests/review_member.rs`'s, copied rather than imported, so
// the designs the screens show are the ones `tests/fixtures/q5-primer/` pins, and
// every value a test expects is read from the document, from a fixture table, or
// from a `tests/fixtures/review/` file, never pasted.

/// The SID-1 cDNA's digest, from `tests/fixtures/fasta/expected.tsv`.
const SID1_DIGEST: &str = "54704740a9f04b79d17147ef95788ad9f0f11e5263843a7ed490924d43b55e3a";

/// A fresh directory for one test, as `tests/review_member.rs`'s `workspace` is.
fn member_dir(test: &str) -> PathBuf {
    let dir =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("review-view-member-{}", test));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// A fresh SID-1 workspace, as `tests/review_member.rs`'s `sid1_workspace` is.
fn sid1_dir(test: &str) -> PathBuf {
    let dir = member_dir(test);
    fs::copy(
        "../../tests/fixtures/example-construct/input/sid-1_cDNA.fa",
        dir.join("sid-1_cDNA.fa"),
    )
    .unwrap();
    dir
}

/// One `tests/fixtures/review/` file's whole text, as `tests/review_member.rs`'s
/// `fixture` reads it.
fn fixture(name: &str) -> String {
    fs::read_to_string(PathBuf::from("../../tests/fixtures/review").join(name)).unwrap()
}

/// One `tests/fixtures/review/` file's lines, in order.
fn fixture_lines(name: &str) -> Vec<String> {
    fixture(name).lines().map(|line| line.to_string()).collect()
}

/// The design block `tests/end_to_end.rs`'s `design()` states: the same buffer,
/// so the bound is the only number a test changes.
fn design() -> Value {
    json!({
        "policy": "luriaNative",
        "style": "q5_polymerase",
        "minAnnealingOverlap": 15,
        "placement": "inline",
        "buffer": "q5_reaction"
    })
}

/// The SID-1 cDNA as a FASTA template, as `tests/end_to_end.rs`'s
/// `sid1_template()` declares it: the same id, the same digest, the same CDS.
fn sid1_template() -> Value {
    json!({
        "id": "sid-1",
        "fasta": "sid-1_cDNA.fa",
        "topology": "linear",
        "sha256": SID1_DIGEST,
        "cds": [{
            "id": "sid-1",
            "intervals": [[0, 3153]],
            "strand": "+",
            "completeness": "complete",
            "geneticCode": "standard",
            "numbering": "from-cds-start"
        }]
    })
}

/// The synthetic template as `tests/review_frames.rs` declares the rows' `seq:`
/// templates: id `syn`, file `syn.fa`, one CDS over its whole 100 bases.
fn syn_template() -> Value {
    json!({
        "id": "syn",
        "fasta": "syn.fa",
        "topology": "linear",
        "cds": [{
            "id": "syn",
            "intervals": [[0, 99]],
            "strand": "+",
            "completeness": "complete",
            "geneticCode": "standard",
            "numbering": "from-cds-start"
        }]
    })
}

/// A one-edit request on SID-1 at `bound`, as `tests/end_to_end.rs`'s `request`
/// writes it: `edit` is the one kind, with `name` added to it.
fn sid1_request(name: &str, edit: Value, bound: u64) -> Value {
    let mut spec = design();
    spec["minAnnealingOverlap"] = json!(bound);
    let mut edit_object = json!({ "name": name });
    for (key, value) in edit.as_object().expect("an edit object") {
        edit_object[key] = value.clone();
    }
    json!({
        "schema": "luria.q5-request/1",
        "coordinates": "zero-based-half-open",
        "template": sid1_template(),
        "scope": "sid-1",
        "edit": edit_object,
        "design": spec
    })
}

/// A one-edit request on the synthetic template, as `tests/review_frames.rs`
/// writes its `seq:` rows: the same design, with `name` added to the edit.
fn syn_request(name: &str, edit: Value) -> Value {
    let mut edit_object = json!({ "name": name });
    for (key, value) in edit.as_object().expect("an edit object") {
        edit_object[key] = value.clone();
    }
    json!({
        "schema": "luria.q5-request/1",
        "coordinates": "zero-based-half-open",
        "template": syn_template(),
        "edit": edit_object,
        "design": design()
    })
}

/// One column of one row of one `tests/fixtures/q5-primer/` table, found by its
/// `case_id` column and read at test time.
fn column_of(table: &str, case_id: &str, column: &str) -> String {
    let path = format!("../../tests/fixtures/q5-primer/{}", table);
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("cannot read {}: {}", path, error));
    let lines: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    let header: Vec<String> = lines[0]
        .split('\t')
        .map(|field| field.to_string())
        .collect();
    for line in lines[1..].iter() {
        let fields: Vec<String> = line.split('\t').map(|field| field.to_string()).collect();
        if fields[0] == case_id {
            for (index, name) in header.iter().enumerate() {
                if name == column {
                    return fields[index].clone();
                }
            }
            panic!("no column {} in {}", column, table);
        }
    }
    panic!("no row {} in {}", case_id, table)
}

/// The replacement `native.tsv` row `native-01` records, from its `edit` column
/// (`s:e:from:to`).
fn native_edit() -> Value {
    let field: String = column_of("native.tsv", "native-01", "edit");
    let parts: Vec<&str> = field.split(':').collect();
    assert!(parts.len() == 4, "edit {:?} is not s:e:from:to", field);
    json!({ "replacement": {
        "interval": [parts[0].parse::<u64>().unwrap(), parts[1].parse::<u64>().unwrap()],
        "from": parts[2],
        "to": parts[3]
    } })
}

/// An insertion as one `native-insertion.tsv` row states it: its `point` and
/// `inserted` columns, with `before` and `after` the template's own bases at
/// `point - 1` and `point`.
fn row_insertion(row: &str, template: &str) -> Value {
    let point: u64 = column_of("native-insertion.tsv", row, "point")
        .parse::<u64>()
        .unwrap();
    json!({ "insertion": {
        "point": point,
        "before": template[(point - 1) as usize..point as usize].to_string(),
        "after": template[point as usize..point as usize + 1].to_string(),
        "insert": column_of("native-insertion.tsv", row, "inserted")
    } })
}

/// The `template` column of one row of one `tests/fixtures/q5-primer/` table,
/// without its `seq:` prefix.
fn synthetic(table: &str, row: &str) -> String {
    let column: String = column_of(table, row, "template");
    assert!(
        column.starts_with("seq:"),
        "row {} of {} gives a template file, not a sequence",
        row,
        table
    );
    column["seq:".len()..].to_string()
}

/// One run: the document it gave, the template's bases as `matching_template`
/// accepts them, and every member's frame as `member_frames` gives it, the way
/// `main` gives them to the app (7i2).
struct Run {
    document: ResultDocument,
    template: Vec<char>,
    frames: Vec<MemberFrame>,
}

/// Run one request file through `run_with_constructs` and prepare it, so the
/// bases are the template that run read and the frames are read on the constructs
/// that run made. The run is checked before the screen is built.
fn run_at(path: &Path, test: &str) -> Run {
    let (outcome, constructs) = run_with_constructs(path);
    let prepared = prepare(path).expect("the request prepares");
    assert_eq!(
        outcome.document.state, "verified",
        "{}: the run is verified",
        test
    );
    assert_eq!(outcome.document.results.len(), 1, "{}: one member", test);
    let template: Vec<char> = matching_template(&outcome.document, &prepared.template)
        .expect("the template is the one the run used");
    let frames: Vec<MemberFrame> = member_frames(
        &outcome.document,
        &prepared.template,
        &prepared.annotations,
        &constructs,
    );
    Run {
        document: outcome.document,
        template,
        frames,
    }
}

/// The app of one run, with the template's bases and the frames set the way
/// `main` sets them when the screen opens.
fn app_of(run: &Run) -> App {
    App::new(run.document.clone())
        .with_template(Some(run.template.clone()))
        .with_frames(run.frames.clone())
}

/// The app of the `native-01` row, run on the SID-1 cDNA at bound 15.
fn native01(test: &str) -> Run {
    let dir = sid1_dir(test);
    let path = dir.join("request.json");
    fs::write(
        &path,
        sid1_request("native-01", native_edit(), 15).to_string(),
    )
    .unwrap();
    run_at(&path, test)
}

/// The app of the `ins-13` row: 210 bases inserted on the synthetic FASTA
/// template written beside the request, as `tests/review_member.rs` does.
fn ins13(test: &str) -> Run {
    let template: String = synthetic("native-insertion.tsv", "ins-13");
    let dir = member_dir(test);
    fs::write(dir.join("syn.fa"), format!(">syn\n{}\n", template)).unwrap();
    let path = dir.join("request.json");
    fs::write(
        &path,
        syn_request("ins-13", row_insertion("ins-13", &template)).to_string(),
    )
    .unwrap();
    run_at(&path, test)
}

/// `app` with `keys` handled, the way the terminal reaches a screen.
fn opened_app(app: App, keys: &[Key]) -> App {
    let mut app: App = app;
    for key in keys {
        app.handle(*key);
    }
    app
}

/// The body's rows at 80x24: rows 3 to `3 + BODY_ROWS - 1`.
fn body(app: &App) -> Vec<String> {
    let rows: Vec<String> = screen(app, 80, 24);
    rows[3..3 + BODY_ROWS].to_vec()
}

/// The key line at 80x24.
fn keys(app: &App) -> String {
    screen(app, 80, 24)[23].clone()
}

/// The first `BODY_ROWS` rows of `app`' body as the screen shows the fixture's
/// lines at width 80, with the rows past the screen's last line blank.
fn shown_rows(lines: &[String]) -> Vec<String> {
    (0..BODY_ROWS)
        .map(|row| {
            lines
                .get(row)
                .map(|line| shown(line, 80))
                .unwrap_or_default()
        })
        .collect()
}

/// The `code` a primer warning carries, as the document writes it.
fn warning_code(warning: &PrimerWarningView) -> String {
    let value: Value = serde_json::to_value(warning).expect("a warning serializes");
    value["code"]
        .as_str()
        .expect("a warning names its code")
        .to_string()
}

/// The `message` a warning carries, as the document writes it.
fn warning_message(warning: &PrimerWarningView) -> String {
    let value: Value = serde_json::to_value(warning).expect("a warning serializes");
    value["message"]
        .as_str()
        .expect("a warning names its message")
        .to_string()
}

/// The character column `needle` ends at in `line`, counting characters, so two
/// columns of the primer table can be shown to line up (M7-28).
fn end_column(line: &str, needle: &str) -> usize {
    let chars: Vec<char> = line.chars().collect();
    let target: Vec<char> = needle.chars().collect();
    let start: usize = chars
        .windows(target.len())
        .position(|window| window == target.as_slice())
        .unwrap_or_else(|| panic!("{:?} holds {}", line, needle));
    start + target.len()
}

/// One primer's row of the sequence screen's table, from the document's own
/// length, Tm and sequence, laid out as M7-28 states: the label padded to 10, the
/// `<n> nt` right-aligned in 6, the `{:.1} °C` right-aligned in 9, three spaces,
/// the sequence.
fn primer_row(label: &str, primer: &PrimerView) -> String {
    format!(
        "{:width$}{:>6}  {:>9}   {}",
        label,
        format!("{} nt", primer.length),
        format!("{:.1} °C", primer.tm.celsius),
        primer.sequence,
        width = 10
    )
}

/// The lines `member_screen` writes under 7i3's block for `native-01`'s design:
/// the primer table with the annealing temperature, then every warning by the
/// document's own message, in the order the document holds them (M7-28, M7-29).
/// Every value is `design`' own field, so only the layout is typed here.
fn native01_tail(design: &DesignView) -> Vec<String> {
    let mut lines: Vec<String> = vec![
        String::new(),
        format!(
            "{}{:>6}  {:>9}   sequence 5'→3'",
            " ".repeat(10),
            "length",
            "Tm"
        ),
        primer_row("forward", &design.forward),
        primer_row("reverse", &design.reverse),
        format!("annealing temperature {} °C", design.annealing_temperature),
    ];
    let messages: Vec<String> = design
        .forward
        .warnings
        .iter()
        .map(|warning| format!("  forward: {}", warning_message(warning)))
        .chain(
            design
                .reverse
                .warnings
                .iter()
                .map(|warning| format!("  reverse: {}", warning_message(warning))),
        )
        .collect();
    assert!(
        !messages.is_empty(),
        "native-01's design has warnings, so the screen lists them"
    );
    lines.push(String::new());
    lines.push("Warnings".to_string());
    lines.extend(messages);
    lines
}

#[test]
fn enter_opens_the_sequence() {
    let run: Run = native01("enter_sequence");
    let app: App = opened_app(app_of(&run), &[Key::Enter]);
    assert_eq!(app.screen(), Screen::Member(0), "`enter` opens the member");
    let first: String = fixture_lines("member-native-01.txt")[0].clone();
    assert_eq!(
        body(&app)[0],
        shown(&first, 80),
        "the first body row is the first line of the sequence block: {:?}",
        body(&app)[0]
    );
    assert_eq!(
        keys(&app),
        "q quit  esc back  d details  j/k scroll  ? help",
        "the sequence screen's keys, with no `i` on a short edit: {:?}",
        keys(&app)
    );
}

#[test]
fn d_opens_the_details_and_esc_returns() {
    let run: Run = native01("details_lines");
    let app: App = opened_app(app_of(&run), &[Key::Enter, Key::Char('d')]);
    assert_eq!(app.screen(), Screen::Details(0), "`d` opens the details");
    let first: String = text::member(&run.document, 0)[0].clone();
    let rows: Vec<String> = screen(&app, 80, 24);
    assert_eq!(
        rows[3],
        shown(&first, 80),
        "the details screen shows 7c2's member text: {:?}",
        rows[3]
    );
    assert_eq!(
        rows[23],
        format!(
            "q quit  esc back  j/k scroll  ? help  lines 1-{} of {}",
            BODY_ROWS,
            text::member(&run.document, 0).len()
        ),
        "the details screen's keys, with the page it shows: {:?}",
        rows[23]
    );
    let app: App = opened_app(app_of(&run), &[Key::Enter, Key::Char('d'), Key::Esc]);
    assert_eq!(
        app.screen(),
        Screen::Member(0),
        "`esc` goes back to the sequence screen"
    );
    let first: String = fixture_lines("member-native-01.txt")[0].clone();
    assert_eq!(
        body(&app)[0],
        shown(&first, 80),
        "and the sequence block is there again: {:?}",
        body(&app)[0]
    );
}

#[test]
fn i_expands_a_long_insertion_and_reopening_collapses_it() {
    let run: Run = ins13("expand");
    let collapsed: Vec<String> = fixture_lines("member-ins-13.txt");
    let expanded: Vec<String> = fixture_lines("member-ins-13-expanded.txt");
    // The screen's own lines, so the key line's page count is the screen's own
    // length, and the block the body shows is the fixture's lines.
    let collapsed_screen: Vec<String> = text::member_screen(
        &run.document,
        0,
        Some(&run.template),
        run.frames.first(),
        false,
    );
    let expanded_screen: Vec<String> = text::member_screen(
        &run.document,
        0,
        Some(&run.template),
        run.frames.first(),
        true,
    );
    assert_eq!(
        &collapsed_screen[..collapsed.len()],
        &collapsed[..],
        "the sequence screen is 7i3's collapsed block, then the short lines"
    );
    assert_eq!(
        &expanded_screen[..expanded.len()],
        &expanded[..],
        "expanded, it is 7i3's expanded block, then the short lines"
    );

    // The page the 20 body rows show, as `view` adds it when the screen is longer
    // than the body.
    let page = |lines: &[String]| {
        if lines.len() > BODY_ROWS {
            format!("  lines 1-{} of {}", BODY_ROWS, lines.len())
        } else {
            String::new()
        }
    };

    // The screen opens collapsed, and `i` is offered: 210 bases are inserted.
    let app: App = opened_app(app_of(&run), &[Key::Enter]);
    assert_eq!(app.screen(), Screen::Member(0));
    assert!(!app.expanded(), "the screen starts collapsed");
    // The fixture holds the block, so its rows are the body's first rows; under
    // them the body continues with M7-28's primer table (`the_primer_columns_line_up`).
    assert_eq!(
        body(&app)[..collapsed.len()],
        shown_rows(&collapsed)[..collapsed.len()],
        "collapsed, the body is the marker block the fixture holds: {:?}",
        body(&app)
    );
    assert_eq!(
        keys(&app),
        format!(
            "q quit  esc back  d details  j/k scroll  i expand  ? help{}",
            page(&collapsed_screen)
        ),
        "`i` is offered, and the page is named: {:?}",
        keys(&app)
    );

    // `i` writes the insertion out: all 210 bases make the screen longer.
    let app: App = opened_app(app_of(&run), &[Key::Enter, Key::Char('i')]);
    assert!(app.expanded(), "`i` expands a long insertion");
    assert_eq!(
        body(&app),
        shown_rows(&expanded),
        "the 20 body rows are the fixture's first 20 expanded lines"
    );
    assert_eq!(
        keys(&app),
        format!(
            "q quit  esc back  d details  j/k scroll  i collapse  ? help{}",
            page(&expanded_screen)
        ),
        "expanded, `i` collapses again, and the page is named: {:?}",
        keys(&app)
    );

    // `esc` back to the overview and `enter` again start collapsed.
    let app: App = opened_app(
        app_of(&run),
        &[Key::Enter, Key::Char('i'), Key::Esc, Key::Enter],
    );
    assert_eq!(app.screen(), Screen::Member(0), "`enter` opens it again");
    assert!(!app.expanded(), "reopening starts collapsed");
    assert_eq!(
        body(&app)[..collapsed.len()],
        shown_rows(&collapsed)[..collapsed.len()],
        "the marker block is back: {:?}",
        body(&app)
    );
    assert_eq!(
        keys(&app),
        format!(
            "q quit  esc back  d details  j/k scroll  i expand  ? help{}",
            page(&collapsed_screen)
        ),
        "and `i` offers to expand again: {:?}",
        keys(&app)
    );
}

#[test]
fn i_does_nothing_on_a_short_edit() {
    let run: Run = native01("short_edit");
    let document: &ResultDocument = &run.document;
    let app: App = opened_app(app_of(&run), &[Key::Enter, Key::Char('i')]);
    assert_eq!(app.screen(), Screen::Member(0), "`i` does not move");
    assert!(
        !app.expanded(),
        "`i` expands nothing on a three-base replacement"
    );
    let design = document.results[0]
        .design
        .as_ref()
        .expect("native-01 is designed");
    let expected: Vec<String> =
        shown_rows(&[fixture_lines("member-native-01.txt"), native01_tail(design)].concat());
    assert_eq!(
        body(&app),
        expected,
        "the sequence screen is unchanged by `i`: {:?}",
        body(&app)
    );
    assert_eq!(
        keys(&app),
        "q quit  esc back  d details  j/k scroll  ? help",
        "so `i` is not offered, because `i` has nothing to expand: {:?}",
        keys(&app)
    );
}

#[test]
fn the_short_primer_lines_are_the_documents() {
    let run: Run = native01("primer_lines");
    let document: &ResultDocument = &run.document;
    let design = document.results[0]
        .design
        .as_ref()
        .expect("native-01 is designed");
    let block: Vec<String> =
        text::member_sequence(document, 0, Some(&run.template), run.frames.first(), false);
    let lines: Vec<String> =
        text::member_screen(document, 0, Some(&run.template), run.frames.first(), false);
    assert_eq!(
        &lines[..block.len()],
        &block[..],
        "the sequence screen starts with 7i3's block"
    );
    let tail: Vec<String> = lines[block.len()..].to_vec();
    assert_eq!(
        tail,
        native01_tail(design),
        "every value of the short lines is the document's own field: {:?}",
        tail
    );
}

#[test]
fn the_primer_columns_line_up() {
    let run: Run = native01("primer_columns");
    let design = run.document.results[0]
        .design
        .as_ref()
        .expect("native-01 is designed");
    let lines: Vec<String> = text::member_screen(
        &run.document,
        0,
        Some(&run.template),
        run.frames.first(),
        false,
    );
    let header: usize = lines
        .iter()
        .position(|line| line.ends_with("sequence 5'→3'"))
        .expect("the primer table names its columns");
    assert!(
        lines.len() > header + 2,
        "the header is followed by two rows"
    );
    for (primer, offset) in [(&design.forward, 1usize), (&design.reverse, 2)] {
        let row: &String = &lines[header + offset];
        assert_eq!(
            end_column(row, "nt"),
            end_column(&lines[header], "length"),
            "the length ends in the header's column: {:?}",
            lines[header..header + 3].to_vec()
        );
        assert_eq!(
            end_column(row, "°C"),
            end_column(&lines[header], "Tm"),
            "the Tm ends in the header's column: {:?}",
            lines[header..header + 3].to_vec()
        );
        assert!(
            row.contains(&format!("{} nt", primer.length)),
            "the length is the document's own: {:?}",
            row
        );
        assert!(
            row.contains(&format!("{:.1} °C", primer.tm.celsius)),
            "the Tm is the document's own, shown to one decimal: {:?}",
            row
        );
        assert!(
            row.ends_with(&primer.sequence),
            "the row ends with the primer's own sequence: {:?}",
            row
        );
    }
}

#[test]
fn warnings_are_descriptive() {
    let run: Run = native01("descriptive_warnings");
    let design = run.document.results[0]
        .design
        .as_ref()
        .expect("native-01 is designed");
    let lines: Vec<String> = text::member_screen(
        &run.document,
        0,
        Some(&run.template),
        run.frames.first(),
        false,
    );
    for (side, primer) in [("forward", &design.forward), ("reverse", &design.reverse)] {
        assert!(
            !primer.warnings.is_empty(),
            "native-01's {} primer has warnings to list",
            side
        );
        for warning in &primer.warnings {
            let expected: String = format!("  {}: {}", side, warning_message(warning));
            assert_eq!(
                lines.iter().filter(|line| **line == expected).count(),
                1,
                "the warning is the document's own message, once: {:?}",
                lines
            );
            let code: String = warning_code(warning);
            assert!(
                lines.iter().all(|line| !line.contains(&code)),
                "no warning code is on the screen: {:?}",
                lines
            );
        }
    }
    assert_eq!(
        lines.iter().filter(|line| **line == "Warnings").count(),
        1,
        "the warnings are under one heading: {:?}",
        lines
    );
}

#[test]
fn the_overview_is_members_only() {
    let document: ResultDocument = set("members_only");
    let template = document
        .provenance
        .template
        .as_ref()
        .expect("the set names its template");
    let target = document.target.as_ref().expect("the set has a target");
    assert_eq!(target.strand, "+", "the set's target is on the plus strand");
    // Every expected value is the document's own field, so only the words and the
    // padding of the new form are typed here (M7-25, M7-26).
    let summaries: Vec<String> = document
        .results
        .iter()
        .map(|member| {
            let (original_residue, residue, written_residue, original_codon, written_codon) =
                match &member.mutation {
                    MutationView::Residue {
                        original_residue,
                        residue,
                        written_residue,
                        original_codon,
                        written_codon,
                        ..
                    } => (
                        original_residue,
                        residue,
                        written_residue,
                        original_codon,
                        written_codon,
                    ),
                    other => panic!("the set's members are residue edits: {:?}", other),
                };
            format!(
                "{}{}{} ({}→{})",
                original_residue, residue, written_residue, original_codon, written_codon
            )
        })
        .collect();
    let id_width: usize = template.id.chars().count().max(target.id.chars().count());
    let name_width: usize = document
        .results
        .iter()
        .map(|member| member.name.chars().count())
        .max()
        .expect("the set has a member");
    let summary_width: usize = summaries
        .iter()
        .map(|summary| summary.chars().count())
        .max()
        .expect("the set has a member");
    let expected: Vec<String> = [
        vec![
            format!(
                "Template  {:id$}  {}, {} bp",
                template.id,
                template.topology,
                template.length,
                id = id_width
            ),
            // The interval `[start, end)` as 1-based inclusive bases (M7-24).
            format!(
                "Target    {:id$}  forward strand, bases {}–{}",
                target.id,
                target.intervals[0].0 + 1,
                target.intervals[0].1,
                id = id_width
            ),
            String::new(),
            format!("Variants ({})", document.results.len()),
        ],
        document
            .results
            .iter()
            .enumerate()
            .map(|(index, member)| {
                format!(
                    "{}{:name$}  {:sum$}  designed",
                    if index == 0 { "▶ " } else { "  " },
                    member.name,
                    summaries[index],
                    name = name_width,
                    sum = summary_width
                )
            })
            .collect(),
    ]
    .concat();
    let lines: Vec<String> = text::overview(&document, 0);
    assert_eq!(
        lines, expected,
        "the overview is the template line, the target line, `Variants` and one row per member"
    );
    assert_eq!(lines.len(), 4 + document.results.len());
}

#[test]
fn the_key_line_offers_problem_only_with_a_problem() {
    // The set: `verified`, with no error and no clarification, so the overview
    // offers no problem screen.
    let rows: Vec<String> = screen(&App::new(set("key_line_set")), 80, 24);
    assert_eq!(
        rows[23], "q quit  enter open  j/k move  p provenance  x export  ? help",
        "{:?}",
        rows[23]
    );

    // A `failed` result opens on the problem screen; from the overview, the key
    // that goes back to it is offered, because a problem is never hidden.
    let app: App = opened(&failed("key_line_failed"), &[Key::Esc]);
    assert_eq!(app.screen(), Screen::Overview);
    let rows: Vec<String> = screen(&app, 80, 24);
    assert_eq!(
        rows[23], "q quit  enter open  j/k move  p provenance  x export  e problem  ? help",
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
