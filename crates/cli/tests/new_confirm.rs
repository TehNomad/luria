//! Milestone 7m4's proof (contract 7m4 §2.2, §3): the writing behind `luria new`'s
//! pop-up, from the form of 7m3 (whose helpers are copied here) to the request
//! file the approval writes and the design it runs. Each test works in a fresh
//! directory under `CARGO_TARGET_TMPDIR` with a fixture template copied into it,
//! so the files written are the ones the user's own directory holds. The request
//! written is compared with `interpret::run::request_text`, the text the file
//! holds, and as JSON with `tests/fixtures/new/request-<case>.json`; the run's
//! state and member come from `luria-core` through `review::Approved`.
//!
//! The rules proved: the request is written beside the template, not in the
//! current directory (§3.1 rule 3, test
//! `approving_writes_the_request_beside_the_template_and_runs_it`: the template
//! is copied into a subdirectory, so a request written in `cwd` is not found by
//! the run and not where the test reads it), a file is replaced only on the `y`
//! the user answered on the pop-up that named it (§3.1 rule 1, test
//! `approve_new_replaces_only_when_asked`), and a `y` never writes through a link
//! (§3.1 rule 2, test `replace_never_writes_through_a_link`).

use luria_cli::interpret::run::request_text;
use luria_cli::new::confirm::{approve_new, plan, Planned};
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
/// `overlap` is typed last, over the 15 nt M7-45 prefills `Overlap` with (7k3
/// §4.5, 7m3 §3).
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

/// `[ Save and run ]`, then the pop-up's `enter`: the built request, insisting it
/// is the action the contract names, and returning what was built.
fn review(form: &mut NewForm) -> Built {
    tab_to(form, Field::Review);
    form.handle(Key::Enter);
    match form.handle(Key::Enter) {
        FormAction::Save { built, replace } => {
            assert!(!replace, "the pop-up's enter asks for no replace");
            *built
        }
        other => panic!("the pop-up gave {:?} instead of the built request", other),
    }
}

/// The `sid1-codon` form, as `requests.py` writes it: the whole sequence, the
/// codon `D404N:AAC` typed in its line, and overlap 15 (M7-46: no codon field).
fn sid1_codon() -> Built {
    let mut form = sid1(&["D404N:AAC"], "15");
    review(&mut form)
}

/// The `syn-paste` form, as `requests.py` writes it: a paste, its CDS typed as
/// bases 1 to 99, two mutations, and overlap 15 (M7-46: no codon field).
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
    // M7-45 prefills `Overlap` with 15, so nothing is typed there.
    review(&mut form)
}

/// The request text the approval writes, the text `request_text` writes.
fn shown(planned: &Planned) -> String {
    request_text(&request_echo(&planned.built.request))
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
    let approved =
        approve_new(&planned, false).expect("the approval writes the request and runs it");

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
    let approved = approve_new(&planned, false).expect("the approval writes both files and runs");

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
    let refusal: String = approve_new(&planned, false).expect_err("a taken name is refused");
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

// §3.1 rule 1 (M7-49): `approve_new` replaces only when it is asked to, so
// `replace` false refuses a name that is there and leaves the file as the user
// wrote it, and `replace` true, the `y` the user answered on the pop-up that
// named it, writes the request over it.
#[test]
fn approve_new_replaces_only_when_asked() {
    let dir: PathBuf = fresh("confirm-replace");
    copy_sid1(&dir);
    let taken: PathBuf = dir.join("sid-1.request.json");
    fs::write(&taken, "old\n").expect("the taken name is filled");

    let built: Built = sid1_codon();
    let planned: Planned = planned_of(built, None, &dir, "sid-1.request.json");

    let refusal: String = approve_new(&planned, false)
        .expect_err("a name that is there is refused when nothing was asked");
    assert!(
        refusal.contains("sid-1.request.json"),
        "the refusal names the file that is there: {}",
        refusal
    );
    assert_eq!(
        fs::read_to_string(&taken).expect("the file is still readable"),
        "old\n",
        "a write nobody asked for leaves the file as the user wrote it"
    );

    let approved = approve_new(&planned, true).expect("the user's y replaces the file it named");
    assert_eq!(
        fs::read_to_string(&taken).expect("the request is written over the old file"),
        shown(&planned),
        "the file holds the request, the text the pop-up named"
    );
    assert_eq!(
        approved.document.state, "verified",
        "the design ran from the file the replace wrote"
    );
}

// §3.1 rule 2 (M7-49, C-1): `y` never writes through a link, so a request path
// that is a link to a file outside the directory is refused, and the file the
// link points at stands as the user wrote it.
#[cfg(unix)]
#[test]
fn replace_never_writes_through_a_link() {
    use std::os::unix::fs::symlink;

    let dir: PathBuf = fresh("confirm-link");
    // The link's target is outside the directory, so the test reads it back.
    let outside: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR")).join("confirm-link-target");
    fs::write(&outside, "not a request\n").expect("the link's target is written");
    copy_sid1(&dir);
    let taken: PathBuf = dir.join("sid-1.request.json");
    symlink(&outside, &taken).expect("the request path is a link to the outside file");

    let built: Built = sid1_codon();
    let planned: Planned = planned_of(built, None, &dir, "sid-1.request.json");

    let refusal: String =
        approve_new(&planned, true).expect_err("a link is written through by nobody, y included");
    assert!(
        refusal.contains("sid-1.request.json"),
        "the refusal names the link: {}",
        refusal
    );
    assert_eq!(
        fs::read_to_string(&outside).expect("the link's target is readable"),
        "not a request\n",
        "the file the link points at is unchanged (C-1)"
    );
    assert_eq!(
        fs::read_to_string(&taken).expect("the link stands"),
        "not a request\n",
        "reading the path gives its target, which was not written"
    );
}
