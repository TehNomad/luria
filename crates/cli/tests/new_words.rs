//! Milestone 7m3's proof (contract 7m3 §3): `luria new`'s one `Mutations` box
//! read in words (M7-47). A line luria cannot read at all makes the whole box
//! words, which `[ Review ]` sends through `interpret` when a backend was given,
//! and is a blocker naming `--backend` when none was; a line luria reads and
//! refuses stays a red error and is never sent to the model. Words need a GenBank
//! template whose CDS list the model is shown, and lines that all read keep the
//! shorthand path of 7k3 as it was. The form is driven with `Key` values as
//! `crates/cli/tests/new_form.rs` drives it, and the files are read from the
//! fixtures at test time and given to the form through `NewForm::loaded`, which
//! is the loop's answer to the `FormAction::Load` the form asks for; the form
//! reads no file, so the words typed are the case's own text.
//!
//! One test runs the case through `interpret` with `luria_nl::backend::ScriptedBackend`
//! and the case's reply, exactly as `crates/cli/tests/review_proposal.rs` and
//! `crates/cli/tests/review_approve.rs` run it, then approves the document: that
//! pins that `luria new` reaches the same `review::approve` path 7f2 uses. With
//! no `--design` profile, the `Interpret` profile is the form's own values (M7-41,
//! M7-42), so the case's reply is read with the prefilled overlap (M7-45), the
//! default codon policy (M7-46) and counting from the CDS start. No test makes a
//! network call, and the terminal loop itself is the owner's hand test.
//!
//! Every expected value is one the corpus holds on its own: the row is
//! `tests/fixtures/nl/cases.tsv`'s own columns, recorded in
//! `tests/fixtures/nl/README.md`; the words typed are the case's text file; the
//! request written is compared with `tests/fixtures/nl/request/`. Nothing is
//! copied from running the code.

use luria_cli::interpret::document::InterpretationDocument;
use luria_cli::interpret::run::interpret;
use luria_cli::interpret::run::interpret_with_profile;
use luria_cli::interpret::run::InterpretInput;
use luria_cli::interpret::run::TextSource;
use luria_cli::new::form::{Field, FormAction, NewForm};
use luria_cli::request::parse_profile;
use luria_cli::request::CodonSpec;
use luria_cli::request::DesignSpec;
use luria_cli::request::OperationSpec;
use luria_cli::request::ProfileDocument;
use luria_cli::review::app::Key;
use luria_cli::review::approve;
use luria_cli::review::Approved;
use luria_cli::shorthand::Refusal;
use luria_nl::backend::ScriptedBackend;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

const NL: &str = "../../tests/fixtures/nl/";
const GENBANK: &str = "../../tests/fixtures/genbank/";
const FIXTURES: &str = "../../tests/fixtures/";
const SID1: &str = "example-construct/input/sid-1_cDNA.fa";
const CASES_HEADER: &str = "case_id\ttemplate\ttext_file\treply_file\tprofile\tstate\tcodes\trequest_file\tdesign_exit\tnote";

/// The blocker `[ Review ]` gives for words without a GenBank record (§2.2).
const GENBANK_MESSAGE: &str = "words need a GenBank template, whose CDS list the model is shown";

/// The blocker `[ Review ]` gives for words with no backend, on the first line
/// that does not read (§2.2).
fn no_backend_message(number: usize) -> String {
    format!(
        "line {} is not shorthand; start luria new with --backend to read the mutations in words",
        number
    )
}

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
/// profile, copied in under their own names, as `review_proposal.rs` does.
fn workspace(test: &str, row: &[String]) -> PathBuf {
    let dir: PathBuf =
        PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("new-words-{}-{}", test, row[0]));
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

/// One fixture as the JSON value it holds (key order free).
fn fixture(relative: &str) -> Value {
    serde_json::from_str(&read(&format!("{}{}", FIXTURES, relative))).expect("the fixture is JSON")
}

/// Types `text` one character at a time, as a user presses the keys, `Enter` for
/// a newline in a multi-line field.
fn type_text(form: &mut NewForm, text: &str) {
    for ch in text.chars() {
        if ch == '\n' {
            form.handle(Key::Enter);
        } else {
            form.handle(Key::Char(ch));
        }
    }
}

/// `Tab` forward until focus reaches `wanted`, as `new_form.rs` does.
fn tab_to(form: &mut NewForm, wanted: Field) {
    for _ in 0..30 {
        if form.focus() == wanted {
            return;
        }
        form.handle(Key::Tab);
    }
    panic!("focus never reached {:?}", wanted);
}

/// A form with a backend: the case's profile read with `parse_profile`, the
/// template typed in `File` and loaded as the loop answers `Load`, and the
/// case's words typed into `Mutations`, the one box M7-47 gives, whose lines
/// luria cannot read at all, so the box reads as words. The profile prefills
/// `Overlap` with its own value (M7-45). `[ Review ]` is left on the button, so
/// the test says what it gives.
fn words_form(row: &[String]) -> NewForm {
    let profile: ProfileDocument = parse_profile(&read(&format!("{}profile/{}", NL, row[4])))
        .expect("the profile is a profile");
    let mut form = NewForm::new(Some(&profile), true);
    tab_to(&mut form, Field::File);
    type_text(&mut form, &row[1]);
    assert_eq!(
        form.handle(Key::Tab),
        FormAction::Load(row[1].clone()),
        "leaving File with a path asks the loop to read it"
    );
    form.loaded(&row[1], Ok(read(&format!("{}{}", GENBANK, row[1]))));
    tab_to(&mut form, Field::Mutations);
    type_text(&mut form, &text_of(row));
    tab_to(&mut form, Field::Review);
    form
}

/// A form with **no** `--design` profile (M7-41): the case's template typed in
/// `File` and loaded as the loop answers `Load`, and the case's words typed into
/// `Mutations`. `Overlap` opens prefilled with 15 (M7-45), the codon is the
/// default policy (M7-46), so every value the `Interpret` profile carries is one
/// the form shows. `[ Review ]` is left on the button.
fn words_form_without_profile(row: &[String]) -> NewForm {
    let mut form = NewForm::new(None, true);
    tab_to(&mut form, Field::File);
    type_text(&mut form, &row[1]);
    assert_eq!(
        form.handle(Key::Tab),
        FormAction::Load(row[1].clone()),
        "leaving File with a path asks the loop to read it"
    );
    form.loaded(&row[1], Ok(read(&format!("{}{}", GENBANK, row[1]))));
    tab_to(&mut form, Field::Mutations);
    type_text(&mut form, &text_of(row));
    // The prefilled 15 is the overlap the words are read with, so nothing is
    // typed on `Overlap`.
    tab_to(&mut form, Field::Review);
    form
}

/// A form with **no** backend (M7-47): the case's template typed in `File` and
/// loaded as the loop answers `Load`, and the case's words typed into
/// `Mutations`, so the box reads as words with no model to read it. `[ Review ]`
/// is left on the button.
fn words_form_without_backend(row: &[String]) -> NewForm {
    let mut form = NewForm::new(None, false);
    tab_to(&mut form, Field::File);
    type_text(&mut form, &row[1]);
    assert_eq!(
        form.handle(Key::Tab),
        FormAction::Load(row[1].clone()),
        "leaving File with a path asks the loop to read it"
    );
    form.loaded(&row[1], Ok(read(&format!("{}{}", GENBANK, row[1]))));
    tab_to(&mut form, Field::Mutations);
    type_text(&mut form, &text_of(row));
    tab_to(&mut form, Field::Review);
    form
}

/// The `sid-1` form, as `new_form.rs` builds it, with a backend: the file loaded
/// as the loop answers `Load`, the name typed over the stem, `lines` typed into
/// the mutation box, and `overlap` typed last.
fn sid1(lines: &[&str], overlap: &str) -> NewForm {
    let mut form = NewForm::new(None, true);
    tab_to(&mut form, Field::File);
    type_text(&mut form, SID1);
    assert_eq!(
        form.handle(Key::Tab),
        FormAction::Load(SID1.to_string()),
        "leaving File with a path asks the loop to read it"
    );
    form.loaded(SID1, Ok(read(&format!("{}{}", FIXTURES, SID1))));
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

// M7-47 (contract §3 test 4): a line luria cannot read at all makes the whole
// `Mutations` box words, so `[ Review ]` asks the loop to send it through
// `interpret`, with the template as it was typed in `File`, the box as it was
// typed, and the design the form shows, which for this case is the `--design`
// profile it opened with (M7-41).
#[test]
fn unreadable_lines_go_to_the_model() {
    let row: Vec<String> = row_of("two-separate");
    let mut form = words_form(&row);
    let given: ProfileDocument =
        parse_profile(&read(&format!("{}profile/{}", NL, row[4]))).expect("a profile");
    assert_eq!(
        form.handle(Key::Enter),
        FormAction::Interpret {
            template: row[1].clone(),
            text: text_of(&row),
            profile: given,
        },
        "the words go to the model with the GenBank template as typed in File"
    );
}

// §3.1 rule 2's codon half is M7-46's: with no codon field, `words_profile` is
// given the default policy, so the text must state a codon, as the case's does.
#[test]
fn words_without_a_profile_use_the_forms_values() {
    let row: Vec<String> = row_of("no-profile");
    let dir: PathBuf = workspace("values", &row);
    let mut form = words_form_without_profile(&row);
    let (typed, words, profile): (String, String, ProfileDocument) = match form.handle(Key::Enter) {
        FormAction::Interpret {
            template,
            text,
            profile,
        } => (template, text, profile),
        other => panic!("the words form gave {:?} instead of the Interpret", other),
    };

    // The loop's closure, as `emit_new` builds it: the case's reply, with
    // `write_request: None`, so the run writes nothing.
    let reply: String = read(&format!("{}reply/{}", NL, row[3]));
    let mut backend: ScriptedBackend = ScriptedBackend::new("scripted-corpus", vec![Ok(reply)]);
    let input: InterpretInput = InterpretInput {
        template: dir.join(&typed),
        text: TextSource::Inline(words),
        profile: None,
        write_request: None,
    };
    let document: InterpretationDocument =
        interpret_with_profile(&input, profile, &mut backend).document;
    assert_eq!(
        document.state, "proposed",
        "the form's values answer what the text leaves unstated"
    );
    assert_eq!(document.questions, [].to_vec(), "nothing is asked");
    let request: Value =
        serde_json::to_value(document.request.as_ref().expect("a proposed request"))
            .expect("the request is JSON");
    assert_eq!(
        request,
        fixture("nl/request/edit-a42g.json"),
        "the request the corpus states for this edit under overlap 15, codon default and numbering from the CDS start"
    );
    let _ = fs::remove_dir_all(&dir);
}

// §3.1 rule 1 (contract §2.2): the overlap the user types is the profile's
// overlap, over the value M7-45 prefills `Overlap` with (contract §3 test 1).
#[test]
fn a_typed_overlap_wins_over_the_profile() {
    let row: Vec<String> = row_of("two-separate");
    let mut form = words_form(&row);
    // The `--design` profile prefills Overlap with its own value, and the user
    // types 20 over it.
    tab_to(&mut form, Field::Overlap);
    for _ in 0..form.overlap().chars().count() {
        form.handle(Key::Backspace);
    }
    type_text(&mut form, "20");
    tab_to(&mut form, Field::Review);
    match form.handle(Key::Enter) {
        FormAction::Interpret { profile, .. } => {
            let design: DesignSpec = profile.design.expect("the form states its design");
            assert_eq!(
                design.min_annealing_overlap, 20,
                "the typed overlap is the overlap the words are read with"
            );
        }
        other => panic!("the words form gave {:?} instead of the Interpret", other),
    }
}

// M7-46 (contract §2.1): the form has no codon field, so a line that states a
// codon carries it and the profile the words are read with states `default`; the
// codons the corpus takes from the case's text are recorded in the case's request.
#[test]
fn the_words_profile_states_the_default_codon() {
    let row: Vec<String> = row_of("codon-stated");
    let mut form = words_form(&row);
    match form.handle(Key::Enter) {
        FormAction::Interpret { profile, .. } => {
            assert_eq!(
                profile.codon,
                Some(CodonSpec::Default),
                "the form has no codon field, so the profile states the default policy"
            );
        }
        other => panic!("the words form gave {:?} instead of the Interpret", other),
    }
}

// M7-47 (contract §2.2 step 1, §3.1 rule 1): a line luria reads and refuses, a
// stop included, is a red error naming its line, and the box is never sent to the
// model for it, so the stop refusal on line 1 is the blocker, not `Interpret`.
#[test]
fn a_refused_line_is_never_words() {
    let row: Vec<String> = row_of("two-separate");
    let mut form = NewForm::new(None, true);
    tab_to(&mut form, Field::File);
    type_text(&mut form, &row[1]);
    assert_eq!(
        form.handle(Key::Tab),
        FormAction::Load(row[1].clone()),
        "leaving File with a path asks the loop to read it"
    );
    form.loaded(&row[1], Ok(read(&format!("{}{}", GENBANK, row[1]))));
    tab_to(&mut form, Field::Mutations);
    type_text(&mut form, "D404*");
    form.handle(Key::Enter);
    type_text(&mut form, "change alanine 42 to glycine");
    tab_to(&mut form, Field::Review);
    assert_eq!(
        form.handle(Key::Enter),
        FormAction::Continue,
        "a line luria reads and refuses calls no model"
    );
    assert_eq!(
        form.blocker(),
        Some((
            Field::Mutations,
            Some(1),
            Refusal::Stop.message().to_string()
        )),
        "the stop refusal on the first line is the blocker, with its own message"
    );
}

// M7-47 (contract §3 test 5): without a backend the box cannot be read at all, so
// `[ Review ]` is a blocker on `Mutations` that names `--backend`, on the first
// line that does not read.
#[test]
fn unreadable_lines_without_a_backend_name_it() {
    let row: Vec<String> = row_of("two-separate");
    let mut form = words_form_without_backend(&row);
    assert_eq!(
        form.handle(Key::Enter),
        FormAction::Continue,
        "words with no backend call no model and build nothing"
    );
    assert_eq!(
        form.blocker(),
        Some((Field::Mutations, Some(1), no_backend_message(1))),
        "the blocker names --backend on the first line that does not read"
    );
}

// §3.1 rule 2: words need a GenBank record, because the model is shown the
// record's CDS list (M6-7), so a FASTA file is refused with the blocker on the
// `Mutations` box.
#[test]
fn words_with_a_fasta_file_are_refused() {
    let row: Vec<String> = row_of("two-separate");
    // The FASTA template, with no words typed, is 7k3's form as it was.
    let mut form = sid1(&[], "15");
    tab_to(&mut form, Field::Mutations);
    type_text(&mut form, &text_of(&row));
    tab_to(&mut form, Field::Review);
    assert_eq!(
        form.handle(Key::Enter),
        FormAction::Continue,
        "words on a FASTA template call no model"
    );
    assert_eq!(
        form.blocker(),
        Some((Field::Mutations, None, GENBANK_MESSAGE.to_string())),
        "words need the record whose CDS list the model is shown"
    );
}

// §3.1 rule 1 and the approve path: the words form's `Interpret` run through
// `interpret` with the case's reply, then `review::approve`, gives the request
// the corpus states for the case, and the run is `verified`.
#[test]
fn the_interpreted_request_is_the_corpus_request() {
    let row: Vec<String> = row_of("two-separate");
    let dir: PathBuf = workspace("request", &row);
    let mut form = words_form(&row);
    let (typed, words): (String, String) = match form.handle(Key::Enter) {
        FormAction::Interpret {
            template,
            text,
            profile: _,
        } => (template, text),
        other => panic!("the words form gave {:?} instead of the Interpret", other),
    };

    // The loop's closure, as `emit_new` builds it: the case's reply, with
    // `write_request: None`, so the run writes nothing and approving does.
    let reply: String = read(&format!("{}reply/{}", NL, row[3]));
    let mut backend: ScriptedBackend = ScriptedBackend::new("scripted-corpus", vec![Ok(reply)]);
    let input: InterpretInput = InterpretInput {
        template: dir.join(&typed),
        text: TextSource::Inline(words),
        profile: Some(dir.join(&row[4])),
        write_request: None,
    };
    let document: InterpretationDocument = interpret(&input, &mut backend).document;
    assert_eq!(document.state, row[5], "the corpus's own state column");

    // The request path is the one `confirm::plan` gives for the form's `SaveAs`,
    // beside the template file the form typed (M7-37).
    let template: PathBuf = dir.join(&typed);
    let request: PathBuf = dir.join(form.save_as());
    let approved: Approved = match approve(&document, &template, &request) {
        Ok(approved) => approved,
        Err(problem) => panic!("the approval of a proposal must run: {}", problem),
    };

    let written: Value =
        serde_json::from_str(&fs::read_to_string(&request).expect("the request was written"))
            .expect("the written file is JSON");
    assert_eq!(
        written,
        fixture("nl/request/variants-separate.json"),
        "the request the corpus states for the case"
    );
    assert_eq!(
        approved.document.state, "verified",
        "the run of the written file"
    );
    assert!(
        approved.template.is_some(),
        "the template beside the request is the one the result names (D005)"
    );
    let _ = fs::remove_dir_all(&dir);
}

// §3.1 rule 3: a box whose lines all read is not words, so the shorthand path is
// 7k3's, `[ Review ]` building the request, with a FASTA template, as it was.
#[test]
fn blank_words_keep_the_shorthand_path() {
    let mut form = sid1(&["A42G", "W87A"], "15");
    tab_to(&mut form, Field::Mutations);
    type_text(&mut form, "   ");
    tab_to(&mut form, Field::Review);
    match form.handle(Key::Enter) {
        FormAction::Review(built) => {
            let members: usize = match &built.request.operation {
                OperationSpec::Variants { members, .. } => members.len(),
                other => panic!("the two lines give a set of variants, gave {:?}", other),
            };
            assert_eq!(members, 2, "the shorthand lines are the form's own request");
        }
        other => panic!("blank words gave {:?} instead of the built request", other),
    }
}
