//! Milestone 7k5's proof (contract 7k5 §3): the `luria new` confirmation, from
//! the form of 7k3 (whose helpers are copied here) to the request file the
//! approval writes and the design it runs. Each test works in a fresh directory
//! under `CARGO_TARGET_TMPDIR` with a fixture template copied into it, so the
//! files written are the ones the user's own directory holds. The request shown
//! is compared with `interpret::run::request_text`, the text the file holds, and
//! as JSON with `tests/fixtures/new/request-<case>.json`; the run's state and
//! member come from `luria-core` through `review::Approved`.
//!
//! The rules proved: a file is never replaced (§3.1 rule 1, test
//! `an_existing_file_is_never_replaced`), `y` only after `a` (§3.1 rule 2, test
//! `esc_returns_to_the_form`), and the request is written beside the template,
//! not in the current directory (§3.1 rule 3, test
//! `approving_writes_the_request_beside_the_template_and_runs_it`: the template
//! is copied into a subdirectory, so a request written in `cwd` is not found by
//! the run and not where the test reads it).

use luria_cli::interpret::run::request_text;
use luria_cli::new::confirm::{
    approve_new, confirm_lines, plan, ConfirmAction, ConfirmApp, ConfirmScreen, Planned,
};
use luria_cli::new::form::{Field, FormAction, NewForm};
use luria_cli::new::Built;
use luria_cli::result::request_echo;
use luria_cli::review::app::Key;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

const FIXTURES: &str = "../../tests/fixtures/";
const SID1: &str = "example-construct/input/sid-1_cDNA.fa";

/// `requests.py`'s `SYN`: the 100 bases of the pasted sequence.
const SYN: &str = "ATGGCTTGGAATCAGCAGCAGCCGAGCGGCTCGCGGACGCATGGCTTGGAATCAGCAGCA\
                   GCCGAGCGGCTCGCGGACGCATGGCTTGGAATCAGCAGCA";

/// One fixture template, read at test time (contract §3).
fn read_file(relative: &str) -> String {
    let path: String = format!("{}{}", FIXTURES, relative);
    fs::read_to_string(&path).unwrap_or_else(|problem| panic!("cannot read {}: {}", path, problem))
}

/// One fixture as the JSON value it holds (key order free, contract §3).
fn fixture(relative: &str) -> Value {
    serde_json::from_str(&read_file(relative)).expect("the fixture is JSON")
}

/// A fresh directory under `CARGO_TARGET_TMPDIR`, cleared of anything an
/// earlier run left, so the files the approval writes are the only ones there.
fn fresh(name: &str) -> PathBuf {
    let dir: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    if dir.exists() {
        fs::remove_dir_all(&dir).expect("the test directory can be cleared");
    }
    fs::create_dir_all(&dir).expect("the test directory can be created");
    dir
}

/// The `sid-1` FASTA copied into `dir`, as the user's own template file, giving
/// the directory it is in.
fn copy_sid1(dir: &Path) -> PathBuf {
    let source: PathBuf = Path::new(FIXTURES).join(SID1);
    let to: PathBuf = dir.join("sid-1_cDNA.fa");
    fs::copy(&source, &to)
        .unwrap_or_else(|problem| panic!("cannot copy {}: {}", source.display(), problem));
    to
}

/// The files one approval writes, as the form of the case built them:
/// `template_dir` is the directory the request is written beside, and `cwd` the
/// current directory, which for a paste is where both files go.
fn planned_of(built: Built, template_dir: Option<&Path>, cwd: &Path, save_as: &str) -> Planned {
    plan(built, template_dir, cwd, save_as)
}

/// Types `text` one character at a time, as a user presses the keys.
fn type_text(form: &mut NewForm, text: &str) {
    for ch in text.chars() {
        form.handle(Key::Char(ch));
    }
}

/// `Tab` forward until focus reaches `wanted`, as 7k3's test drives the form.
fn tab_to(form: &mut NewForm, wanted: Field) {
    for _ in 0..30 {
        if form.focus() == wanted {
            return;
        }
        form.handle(Key::Tab);
    }
    panic!("focus never reached {:?}", wanted);
}

/// The pasted text of `requests.py`'s `syn_paste`: `SYN[:50]`, a newline and
/// two spaces, then `SYN[50:]` lower case.
fn typed_paste() -> String {
    format!("{}\n  {}", &SYN[..50], SYN[50..].to_lowercase())
}

/// The form with the `sid-1` file loaded as the loop answers `Load`: the path
/// typed in `File`, the name `sid-1` typed over the stem the load filled, and
/// `lines` typed into the mutation box, each line's `Enter` starting the next.
/// `overlap` is typed last, so a case can leave it empty (7k3 §4.5).
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
    type_text(&mut form, overlap);
    form
}

/// `[ Review ]`: the built request, insisting it is the action the contract
/// names, and returning what was built.
fn review(form: &mut NewForm) -> Built {
    tab_to(form, Field::Review);
    match form.handle(Key::Enter) {
        FormAction::Review(placed) => *placed,
        other => panic!("Review gave {:?} instead of the built request", other),
    }
}

/// The `sid1-codon` form, as `requests.py` writes it: the whole sequence, the
/// codon typed per mutation, `D404N:AAC` and overlap 15.
fn sid1_codon() -> Built {
    let mut form = sid1(&["D404N:AAC"], "15");
    tab_to(&mut form, Field::Codon);
    form.handle(Key::Right);
    review(&mut form)
}

/// The `syn-paste` form, as `requests.py` writes it: a paste, its CDS typed as
/// bases 1 to 99, two mutations, a codon typed per mutation and overlap 15.
fn syn_paste() -> Built {
    let mut form = NewForm::new(None, false);
    form.handle(Key::Right);
    tab_to(&mut form, Field::Paste);
    form.paste(&typed_paste());
    tab_to(&mut form, Field::Name);
    type_text(&mut form, "syn");
    tab_to(&mut form, Field::Cds);
    form.handle(Key::Right);
    tab_to(&mut form, Field::First);
    type_text(&mut form, "1");
    tab_to(&mut form, Field::Last);
    type_text(&mut form, "99");
    tab_to(&mut form, Field::Mutations);
    type_text(&mut form, "del 46");
    form.handle(Key::Enter);
    type_text(&mut form, "51-53 ATC>TT");
    tab_to(&mut form, Field::Codon);
    form.handle(Key::Right);
    tab_to(&mut form, Field::Overlap);
    type_text(&mut form, "15");
    review(&mut form)
}

/// The confirmation of a built form, at its top, with the request file named by
/// `save_as`.
fn confirm_of(built: Built, template_dir: Option<&Path>, cwd: &Path, save_as: &str) -> ConfirmApp {
    ConfirmApp::new(planned_of(built, template_dir, cwd, save_as))
}

/// The request text the confirmation shows, the text `request_text` writes.
fn shown(planned: &Planned) -> String {
    request_text(&request_echo(&planned.built.request))
}

#[test]
fn the_confirmation_shows_the_request_as_written() {
    let dir: PathBuf = fresh("confirm-shown");
    let built: Built = sid1_codon();
    let app: ConfirmApp = confirm_of(built, Some(&dir), &dir, "sid-1.request.json");
    let lines: Vec<String> = confirm_lines(&app);

    assert_eq!(
        lines[0], "This request will be written:",
        "the screen says what the request will be, before it is written"
    );
    assert_eq!(
        lines[1],
        app.planned().request.display().to_string(),
        "the file's path, as the approval will write it"
    );
    assert_eq!(lines[2], "", "a blank line before the request");

    // The request exactly as `request_text` writes it (plan §3 rule 2), and the
    // text parsed as JSON is the fixture the form is expected to build.
    let text: String = shown(app.planned());
    let written: Vec<String> = text.lines().map(|line| line.to_string()).collect();
    assert_eq!(
        lines[3..3 + written.len()].to_vec(),
        written,
        "the request, byte for byte as the file holds it"
    );
    assert_eq!(
        lines.len(),
        3 + written.len(),
        "nothing of the confirmation's reading screen follows the request"
    );
    assert_eq!(
        serde_json::from_str::<Value>(&text).expect("the shown request is JSON"),
        fixture("new/request-sid1-codon.json"),
        "the request shown is the fixture the sid1-codon form builds"
    );
    assert_eq!(app.screen(), ConfirmScreen::Reading);
    assert_eq!(app.failure(), None);
}

// §3.1 rule 3: writing the request in `cwd`, next to the template's directory,
// fails this test, because the request is read beside the template, where
// `prepare` finds the template the request names (plan §3.4).
#[test]
fn approving_writes_the_request_beside_the_template_and_runs_it() {
    let dir: PathBuf = fresh("confirm-approve");
    let templates: PathBuf = dir.join("templates");
    fs::create_dir_all(&templates).expect("the template's directory can be created");
    copy_sid1(&templates);

    let built: Built = sid1_codon();
    let planned: Planned = planned_of(built, Some(&templates), &dir, "sid-1.request.json");
    let approved = approve_new(&planned).expect("the approval writes the request and runs it");

    let written: String = fs::read_to_string(templates.join("sid-1.request.json"))
        .expect("the request is written beside the template");
    assert_eq!(
        written,
        shown(&planned),
        "the file holds exactly the text the confirmation showed"
    );
    assert!(
        !dir.join("sid-1.request.json").exists(),
        "the request is not in the current directory, so a write there fails this rule"
    );
    assert_eq!(
        approved.document.state, "verified",
        "the design ran from the file the approval wrote"
    );
    assert_eq!(approved.document.results.len(), 1);
    assert_eq!(
        approved.document.results[0].name, "D404N",
        "the one member the form built, designed"
    );
}

#[test]
fn approving_a_paste_writes_both_files() {
    let dir: PathBuf = fresh("confirm-paste");
    let built: Built = syn_paste();
    assert_eq!(
        built.paste.as_ref().map(|(name, _)| name.clone()),
        Some("syn.fa".to_string()),
        "the paste's FASTA is named by the form's name (M7-37)"
    );
    let planned: Planned = planned_of(built, None, &dir, "syn.request.json");
    let approved = approve_new(&planned).expect("the approval writes both files and runs");

    assert_eq!(
        fs::read_to_string(dir.join("syn.fa")).expect("the pasted FASTA is written"),
        read_file("new/paste-syn-paste.fa"),
        "the paste is written as the FASTA `render_fasta` gives"
    );
    assert_eq!(
        serde_json::from_str::<Value>(
            &fs::read_to_string(dir.join("syn.request.json"))
                .expect("the request is written in the current directory, beside the paste")
        )
        .expect("the written request is JSON"),
        fixture("new/request-syn-paste.json"),
        "the request the paste form builds"
    );
    assert_eq!(
        approved.document.state, "verified",
        "the design ran on the pasted template"
    );
    assert_eq!(
        dir.join("syn.fa").parent(),
        dir.join("syn.request.json").parent(),
        "a paste's FASTA and request share the current directory"
    );
}

// §3.1 rule 1: `approve_new` writes a new file only, so a name already taken
// refuses the approval, and the file stands as the user wrote it.
#[test]
fn an_existing_file_is_never_replaced() {
    let dir: PathBuf = fresh("confirm-existing");
    let templates: PathBuf = dir.join("templates");
    fs::create_dir_all(&templates).expect("the template's directory can be created");
    copy_sid1(&templates);
    let taken: PathBuf = templates.join("sid-1.request.json");
    fs::write(&taken, "not a request\n").expect("the taken name is filled");

    let built: Built = sid1_codon();
    let planned: Planned = planned_of(built, Some(&templates), &dir, "sid-1.request.json");
    let refusal: String = approve_new(&planned).expect_err("a taken name is refused");
    assert!(
        refusal.contains("sid-1.request.json"),
        "the refusal names the file that is there: {}",
        refusal
    );
    assert_eq!(
        fs::read_to_string(&taken).expect("the file is still readable"),
        "not a request\n",
        "luria never replaces a file (C-1)"
    );
}

// §3.1 rule 2, and 7f2's rule: quitting at any point before `y` writes nothing.
#[test]
fn quitting_writes_nothing() {
    let dir: PathBuf = fresh("confirm-quit");
    copy_sid1(&dir);
    let built: Built = sid1_codon();
    let mut app: ConfirmApp = confirm_of(built.clone(), Some(&dir), &dir, "sid-1.request.json");

    assert_eq!(app.handle(Key::Char('q')), ConfirmAction::Quit);
    let mut back: ConfirmApp = confirm_of(built.clone(), Some(&dir), &dir, "sid-1.request.json");
    back.handle(Key::Char('a'));
    assert_eq!(back.handle(Key::Char('q')), ConfirmAction::Quit);

    let names: Vec<String> = fs::read_dir(&dir)
        .expect("the directory is readable")
        .map(|entry| {
            entry
                .expect("an entry")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(names, vec!["sid-1_cDNA.fa".to_string()]);
}

// §3.1 rule 2: `y` is the approval key on the confirmation only, and the
// confirmation is reached by `a` alone, so `y` on reading approves nothing.
#[test]
fn esc_returns_to_the_form() {
    let dir: PathBuf = fresh("confirm-keys");
    let built: Built = sid1_codon();

    let mut app: ConfirmApp = confirm_of(built.clone(), Some(&dir), &dir, "sid-1.request.json");
    assert_eq!(app.screen(), ConfirmScreen::Reading);
    assert_eq!(app.handle(Key::Esc), ConfirmAction::Back);

    // `a` opens the confirmation; `n` goes back to reading; a second `a` opens
    // it again, and only `y` there asks for the approval.
    let mut again: ConfirmApp = confirm_of(built.clone(), Some(&dir), &dir, "sid-1.request.json");
    assert_eq!(again.handle(Key::Char('y')), ConfirmAction::Continue);
    assert_eq!(again.screen(), ConfirmScreen::Reading);
    assert_eq!(again.handle(Key::Char('a')), ConfirmAction::Continue);
    assert_eq!(again.screen(), ConfirmScreen::Confirm);
    assert_eq!(again.handle(Key::Char('n')), ConfirmAction::Continue);
    assert_eq!(again.screen(), ConfirmScreen::Reading);
    assert_eq!(again.handle(Key::Char('y')), ConfirmAction::Continue);
    assert_eq!(again.handle(Key::Char('a')), ConfirmAction::Continue);
    assert_eq!(again.handle(Key::Char('y')), ConfirmAction::Approve);

    // The scroll rule of 7c3, and `q` on the confirmation.
    let mut scrolled: ConfirmApp =
        confirm_of(built.clone(), Some(&dir), &dir, "sid-1.request.json");
    for _ in 0..3 {
        scrolled.handle(Key::Char('j'));
    }
    assert_eq!(scrolled.scroll(), 3);
    assert_eq!(scrolled.handle(Key::PageDown), ConfirmAction::Continue);
    assert_eq!(scrolled.scroll(), 13);
    assert_eq!(scrolled.handle(Key::PageUp), ConfirmAction::Continue);
    assert_eq!(scrolled.scroll(), 3);
    assert_eq!(scrolled.handle(Key::Char('a')), ConfirmAction::Continue);
    assert_eq!(
        scrolled.scroll(),
        0,
        "a screen change puts the new screen at its top"
    );
    assert_eq!(scrolled.handle(Key::Esc), ConfirmAction::Continue);
    assert_eq!(scrolled.screen(), ConfirmScreen::Reading);
    assert_eq!(scrolled.handle(Key::Char('q')), ConfirmAction::Quit);

    assert_eq!(
        fs::read_dir(&dir)
            .expect("the directory is readable")
            .count(),
        0,
        "the keys wrote nothing"
    );
}
