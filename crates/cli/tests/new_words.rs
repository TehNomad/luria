//! Milestone 7k7's proof (contract 7k7 §3): `luria new`'s `In words` box with a
//! backend. Words and shorthand are either one, not both, words need a GenBank
//! template whose CDS list the model is shown (M7-40), and blank words leave the
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
//! M7-42), so the case's reply is read with the typed overlap, the codon choice
//! and counting from the CDS start. No test makes a network call, and the terminal
//! loop itself is the owner's hand test.
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
use luria_cli::new::CodonChoice;
use luria_cli::request::parse_profile;
use luria_cli::request::DesignSpec;
use luria_cli::request::OperationSpec;
use luria_cli::request::ProfileDocument;
use luria_cli::review::app::Key;
use luria_cli::review::approve;
use luria_cli::review::Approved;
use luria_nl::backend::ScriptedBackend;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

const NL: &str = "../../tests/fixtures/nl/";
const GENBANK: &str = "../../tests/fixtures/genbank/";
const FIXTURES: &str = "../../tests/fixtures/";
const SID1: &str = "example-construct/input/sid-1_cDNA.fa";
const CASES_HEADER: &str = "case_id\ttemplate\ttext_file\treply_file\tprofile\tstate\tcodes\trequest_file\tdesign_exit\tnote";

/// The blocker `[ Review ]` gives for words and lines together (contract §2.2).
const BOTH_MESSAGE: &str = "write the mutations either in words or as lines, not both";

/// The blocker `[ Review ]` gives for words without a GenBank record (§2.2).
const GENBANK_MESSAGE: &str = "words need a GenBank template, whose CDS list the model is shown";

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
/// case's words typed into `Words`. `[ Review ]` is left on the button, so the
/// test says what it gives.
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
    tab_to(&mut form, Field::Words);
    type_text(&mut form, &text_of(row));
    tab_to(&mut form, Field::Review);
    form
}

/// A form with **no** `--design` profile (M7-41): the case's template typed in
/// `File` and loaded as the loop answers `Load`, the case's words typed into
/// `Words`, `15` typed in Overlap, and the codon left at "default policy", so
/// every value the `Interpret` profile carries is one the form shows. `[ Review ]`
/// is left on the button.
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
    tab_to(&mut form, Field::Words);
    type_text(&mut form, &text_of(row));
    tab_to(&mut form, Field::Overlap);
    type_text(&mut form, "15");
    // "default policy" is the form's starting choice, so nothing is toggled.
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
    type_text(&mut form, overlap);
    form
}

// §3.1 rule 1 and 2: `[ Review ]` with words typed asks the loop to send them
// through `interpret`, with the template as it was typed in `File`, the words
// as they were typed, and the design the form shows, which for this case is the
// `--design` profile it opened with (M7-41).
#[test]
fn words_with_a_genbank_file_ask_to_interpret() {
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

// §3.1 rule 2 (contract §2.2, M7-41, M7-42): with no `--design`, the form's own
// values are the design the words are read with, so the case that asks about
// every unstated value is proposed, with no questions, and gives the request the
// corpus states for this edit under overlap 15, codon default and numbering from
// the CDS start.
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
// overlap, over the value `--design` opened the form with.
#[test]
fn a_typed_overlap_wins_over_the_profile() {
    let row: Vec<String> = row_of("two-separate");
    let mut form = words_form(&row);
    // `--design` filled Overlap with 15, and the user types 20 over it.
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

// §3.1 rule 2's codon half (contract §2.2): "typed per mutation" leaves the codon
// unstated, so the text must state a codon or the model path asks.
#[test]
fn typed_codons_leave_the_codon_unstated() {
    let row: Vec<String> = row_of("two-separate");
    let mut form = words_form(&row);
    tab_to(&mut form, Field::Codon);
    form.handle(Key::Char(' '));
    assert_eq!(
        form.codon(),
        CodonChoice::TypedPerMutation,
        "the choice is toggled on the Codon field"
    );
    tab_to(&mut form, Field::Review);
    match form.handle(Key::Enter) {
        FormAction::Interpret { profile, .. } => {
            assert_eq!(
                profile.codon, None,
                "codons typed per mutation leave the codon unstated"
            );
        }
        other => panic!("the words form gave {:?} instead of the Interpret", other),
    }
}

// §3.1 rule 1: words and mutation lines together are refused, with the blocker on
// `Words` and no call to the model, so `[ Review ]` gives `Continue`.
#[test]
fn words_and_lines_together_are_refused() {
    let row: Vec<String> = row_of("two-separate");
    let mut form = words_form(&row);
    // One shorthand line typed, so both boxes hold the mutations.
    tab_to(&mut form, Field::Mutations);
    type_text(&mut form, "A42G");
    tab_to(&mut form, Field::Review);
    assert_eq!(
        form.handle(Key::Enter),
        FormAction::Continue,
        "words with a line build nothing and call no model"
    );
    assert_eq!(
        form.blocker(),
        Some((Field::Words, None, BOTH_MESSAGE.to_string())),
        "words and lines are either one, not both"
    );
}

// §3.1 rule 2: words need a GenBank record, because the model is shown the
// record's CDS list (M6-7), so a FASTA file is refused with the blocker on `Words`.
#[test]
fn words_with_a_fasta_file_are_refused() {
    let row: Vec<String> = row_of("two-separate");
    // The FASTA template, with no words typed, is 7k3's form as it was.
    let mut form = sid1(&[], "15");
    tab_to(&mut form, Field::Words);
    type_text(&mut form, &text_of(&row));
    tab_to(&mut form, Field::Review);
    assert_eq!(
        form.handle(Key::Enter),
        FormAction::Continue,
        "words on a FASTA template call no model"
    );
    assert_eq!(
        form.blocker(),
        Some((Field::Words, None, GENBANK_MESSAGE.to_string())),
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

// §3.1 rule 3: words of only spaces are blank, so the shorthand path is 7k3's,
// `[ Review ]` building the request, with a FASTA template, as it was.
#[test]
fn blank_words_keep_the_shorthand_path() {
    let mut form = sid1(&["A42G", "W87A"], "15");
    tab_to(&mut form, Field::Words);
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
