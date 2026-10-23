//! Milestone 7k3's proof (contract 7k3 §4): the `luria new` form driven with
//! `Key` values and `paste`, as a user drives it, builds the request documents
//! `tests/fixtures/new/request-*.json` holds. Templates are read from
//! `tests/fixtures/` at test time and given to the form through
//! `NewForm::loaded`, which is the loop's answer to the `FormAction::Load` the
//! form asks for; the form reads no file and draws nothing, so the form's text
//! is the file's text, not a copy. The form inputs are `requests.py`'s `BUILT`
//! cases, written out in Rust as the keys a user presses. The request is
//! rendered with `result::request_echo` and compared as a `serde_json::Value`
//! with the fixture.

use luria_cli::new::form::Field;
use luria_cli::new::form::FormAction;
use luria_cli::new::form::NewForm;
use luria_cli::new::form::Popup;
use luria_cli::new::form::Preview;
use luria_cli::new::picker::Entry;
use luria_cli::new::Built;
use luria_cli::request::parse_profile;
use luria_cli::request::CodonSpec;
use luria_cli::request::EditSpec;
use luria_cli::request::OperationSpec;
use luria_cli::request::ProfileDocument;
use luria_cli::request::RequestDocument;
use luria_cli::result::request_echo;
use luria_cli::result::RequestEcho;
use luria_cli::review::app::Key;
use luria_cli::shorthand::Reading;
use luria_cli::shorthand::Refusal;
use serde_json::Value;
use std::fs;

const FIXTURES: &str = "../../tests/fixtures/";
const SID1: &str = "example-construct/input/sid-1_cDNA.fa";

/// `requests.py`'s `SYN`: the 100 bases of the pasted sequence.
const SYN: &str = "ATGGCTTGGAATCAGCAGCAGCCGAGCGGCTCGCGGACGCATGGCTTGGAATCAGCAGCA\
                   GCCGAGCGGCTCGCGGACGCATGGCTTGGAATCAGCAGCA";

/// One fixture template, read at test time (contract §4).
fn read_file(relative: &str) -> String {
    let path: String = format!("{}{}", FIXTURES, relative);
    fs::read_to_string(&path).unwrap_or_else(|problem| panic!("cannot read {}: {}", path, problem))
}

/// One fixture as the JSON value it holds (key order free, contract §4).
fn fixture(relative: &str) -> Value {
    serde_json::from_str(&read_file(relative)).expect("the fixture is JSON")
}

/// The request the form built, as the JSON `request_echo` writes it.
fn rendered(request: &RequestDocument) -> Value {
    let echo: RequestEcho = request_echo(request);
    serde_json::to_value(&echo).expect("the echo is JSON")
}

/// Types `text` one character at a time, as a user presses the keys.
fn type_text(form: &mut NewForm, text: &str) {
    for ch in text.chars() {
        form.handle(Key::Char(ch));
    }
}

/// `Tab` forward until focus reaches `wanted`. The actions the form gives on the
/// way are the form's own, and the tests answer the `Load` of `File` where they
/// need to, before calling this.
fn tab_to(form: &mut NewForm, wanted: Field) {
    for _ in 0..30 {
        if form.focus() == wanted {
            return;
        }
        form.handle(Key::Tab);
    }
    panic!("focus never reached {:?}", wanted);
}

/// The `tab_to` cap is well past the fields, so every field is reached: a field
/// focus never reaches is left out, which is what the skipping tests look for.
fn focus_sequence(form: &mut NewForm, steps: usize) -> Vec<Field> {
    let mut seen: Vec<Field> = vec![form.focus()];
    for _ in 0..steps {
        form.handle(Key::Tab);
        seen.push(form.focus());
    }
    seen
}

/// The pasted text of `requests.py`'s `syn_paste`: `SYN[:50]`, a newline and
/// two spaces, then `SYN[50:]` lower case.
fn typed_paste() -> String {
    format!("{}\n  {}", &SYN[..50], SYN[50..].to_lowercase())
}

/// The form with the `sid-1` file loaded as the loop answers `Load`: the path
/// typed in `File`, the name `sid-1` typed over the stem the load filled, and
/// `lines` typed into the mutation box, each line's `Enter` starting the next.
/// `overlap` is typed last, over the 15 nt M7-45 prefills `Overlap` with, so a
/// case can leave it empty (contract §4.5, §3).
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

#[test]
fn typing_the_sid1_set_form_builds_its_request() {
    let mut form = sid1(
        &[
            "D404N",
            "del 1210-1212",
            "",
            "ins GAT after 1209",
            "1210 G>A",
            "1210-1212 GAT>AAC",
        ],
        "15",
    );
    // The CDS stays the whole sequence, forward, and the codon the default
    // policy, so nothing is toggled: the form's own starting values.
    let built: Built = review(&mut form);
    assert_eq!(
        rendered(&built.request),
        fixture("new/request-sid1-set.json"),
        "the request the sid1-set form builds"
    );
    assert_eq!(built.paste, None, "a file template writes no paste");
}

#[test]
fn a_pasted_template_builds_its_request() {
    let mut form = NewForm::new(None, false);
    // Source = paste: the `Source` field's toggle, so `File` is skipped.
    form.handle(Key::Right);
    assert_eq!(form.focus(), Field::Source);
    tab_to(&mut form, Field::Paste);
    form.paste(&typed_paste());
    tab_to(&mut form, Field::Name);
    type_text(&mut form, "syn");
    // The CDS is the form's own bases: the `Cds` field toggles whole to bases.
    tab_to(&mut form, Field::Cds);
    form.handle(Key::Right);
    tab_to(&mut form, Field::First);
    type_text(&mut form, "1");
    tab_to(&mut form, Field::Last);
    type_text(&mut form, "99");
    // The strand stays forward, so nothing is toggled on `Strand`.
    tab_to(&mut form, Field::Mutations);
    type_text(&mut form, "del 46");
    form.handle(Key::Enter);
    type_text(&mut form, "51-53 ATC>TT");
    // M7-46: no codon field, so the default policy is the form's own, and M7-45
    // prefills `Overlap` with 15, so nothing is typed there.
    let built: Built = review(&mut form);
    assert_eq!(
        rendered(&built.request),
        fixture("new/request-syn-paste.json"),
        "the request the syn-paste form builds"
    );
    assert_eq!(
        built.paste,
        Some(("syn.fa".to_string(), read_file("new/paste-syn-paste.fa"))),
        "a paste's FASTA is the file 7k5 writes beside the request"
    );
}

// §3.1 rule 3 (contract §2.2): many terminals send a bracketed paste's line
// breaks as `\r`, so a lone `\r` is one line break, and `\r\n` is one, each read
// exactly as `\n` is: the pasted lines are the same lines.
#[test]
fn a_paste_with_carriage_returns_reads_as_lines() {
    let typed: String = typed_paste();
    let mut form = NewForm::new(None, false);
    // Source = paste, as the `Source` field's toggle gives it.
    form.handle(Key::Right);
    tab_to(&mut form, Field::Paste);
    form.paste(&typed);
    let with_breaks: String = form.paste_text();
    assert_eq!(
        with_breaks.lines().count(),
        2,
        "the `\\n` paste reads as the two lines it typed"
    );

    // The same text with every `\n` sent as a lone `\r`.
    let mut form = NewForm::new(None, false);
    form.handle(Key::Right);
    tab_to(&mut form, Field::Paste);
    form.paste(&typed.replace('\n', "\r"));
    assert_eq!(
        form.paste_text(),
        with_breaks,
        "a lone `\\r` is one line break"
    );

    // The same text with every `\n` sent as `\r\n`.
    let mut form = NewForm::new(None, false);
    form.handle(Key::Right);
    tab_to(&mut form, Field::Paste);
    form.paste(&typed.replace("\n", "\r\n"));
    assert_eq!(
        form.paste_text(),
        with_breaks,
        "`\\r\\n` is one line break, not two"
    );
}

#[test]
fn a_genbank_file_offers_its_usable_cds() {
    let mut form = NewForm::new(None, false);
    tab_to(&mut form, Field::File);
    type_text(&mut form, "synthetic-linear.gb");
    assert_eq!(
        form.handle(Key::Tab),
        FormAction::Load("synthetic-linear.gb".to_string()),
        "leaving File with a path asks the loop to read it"
    );
    form.loaded(
        "synthetic-linear.gb",
        Ok(read_file("genbank/synthetic-linear.gb")),
    );
    // The CDS list is 6c's catalogue, usable entries only, and the form's
    // choice starts at its first entry.
    let list: Vec<String> = form
        .cds_list()
        .iter()
        .map(|entry| entry.id.clone())
        .collect();
    assert_eq!(
        list,
        vec!["synA".to_string()],
        "the usable CDSs of the record"
    );
    assert_eq!(
        form.cds_index(),
        0,
        "the CDS list starts at its first entry"
    );
    // `Tab` skips `Name`, `Shape` and `Strand` for a GenBank record (rule 4).
    let seen: Vec<Field> = focus_sequence(&mut form, 20);
    for field in [Field::Name, Field::Shape, Field::Strand] {
        assert!(
            !seen.contains(&field),
            "focus reached {:?} on a GenBank record",
            field
        );
    }
    tab_to(&mut form, Field::Mutations);
    type_text(&mut form, "A42G");
    form.handle(Key::Enter);
    type_text(&mut form, "W87A");
    tab_to(&mut form, Field::Overlap);
    // M7-45: 20 typed over the prefilled 15.
    let prefilled: usize = form.overlap().chars().count();
    for _ in 0..prefilled {
        form.handle(Key::Backspace);
    }
    type_text(&mut form, "20");
    let built: Built = review(&mut form);
    assert_eq!(
        rendered(&built.request),
        fixture("new/request-synlin-genbank.json"),
        "the request the synlin-genbank form builds, whose id is the LOCUS name"
    );
}

#[test]
fn previews_follow_every_edit() {
    let mut form = NewForm::new(None, false);
    tab_to(&mut form, Field::Mutations);
    // A line that is not one luria reads is shown as its refusal.
    type_text(&mut form, "D404");
    assert_eq!(
        form.previews(),
        vec![Preview {
            line: 1,
            reading: Some(Err(Refusal::Unreadable)),
        }],
        "the preview of a line that does not read"
    );
    // The residue's target makes the line read.
    type_text(&mut form, "N");
    assert_eq!(
        form.previews(),
        vec![Preview {
            line: 1,
            reading: Some(Ok(Reading::Residue {
                number: 404,
                from: 'D',
                to: 'N',
                codon: None,
            })),
        }],
        "the preview of the residue line"
    );
    // `Backspace` takes the target away again, so the preview is recomputed on
    // the edit, not only on what is typed.
    form.handle(Key::Backspace);
    assert_eq!(
        form.previews(),
        vec![Preview {
            line: 1,
            reading: Some(Err(Refusal::Unreadable)),
        }],
        "the preview after Backspace"
    );
}

#[test]
fn review_without_an_overlap_names_the_field() {
    let mut form = sid1(&["D404N"], "");
    tab_to(&mut form, Field::Review);
    assert_eq!(
        form.handle(Key::Enter),
        FormAction::Continue,
        "an empty overlap does not build"
    );
    let blocker = form.blocker().expect("the refusal names a field");
    assert_eq!(
        blocker.0,
        Field::Overlap,
        "the overlap is the missing value"
    );
    assert_eq!(blocker.1, None, "the refusal is not about a mutation line");
    assert!(!blocker.2.is_empty(), "the refusal says the reason");
    assert_eq!(
        form.focus(),
        Field::Overlap,
        "focus moves to the field the refusal names"
    );
}

#[test]
fn a_refused_line_is_named() {
    let mut form = sid1(&["D404N", "D404*"], "15");
    tab_to(&mut form, Field::Review);
    assert_eq!(
        form.handle(Key::Enter),
        FormAction::Continue,
        "a line luria cannot read does not build"
    );
    let blocker = form.blocker().expect("the refusal names a field");
    assert_eq!(blocker.0, Field::Mutations, "the refusal is on the box");
    assert_eq!(blocker.1, Some(2), "the refusal names the second line");
    assert_eq!(form.focus(), Field::Mutations, "focus moves to the box");
}

#[test]
fn escape_quits() {
    let mut form = NewForm::new(None, false);
    tab_to(&mut form, Field::Mutations);
    assert_eq!(form.handle(Key::Esc), FormAction::Quit, "esc quits");
}

// M7-43: `ctrl-c` quits from a text field as `esc` does, and types nothing: on
// a field it is not the `c` a field takes as input.
#[test]
fn ctrl_c_quits_from_a_text_field() {
    let mut form = sid1(&["D404N"], "15");
    tab_to(&mut form, Field::Mutations);
    let typed: Vec<Preview> = form.previews();
    assert_eq!(
        form.handle(Key::CtrlC),
        FormAction::Quit,
        "`ctrl-c` quits from the mutation box"
    );
    assert_eq!(
        form.previews(),
        typed,
        "`ctrl-c` types nothing in the field it quits from"
    );
    assert_eq!(form.overlap(), "15", "the rest of the form is as it was");
}

// M7-44: `Overlap` takes any number of digits, so a length past `i64` is too
// large to read. `[ Review ]` reads that blocker first, so the shorthand does
// not build, the words are not sent to the model, and the value is never read as
// a number at all: the refusal names `Overlap` with the message, on both paths.
#[test]
fn an_overlap_too_large_is_a_blocker() {
    const TOO_LARGE: &str = "99999999999999999999";
    const MESSAGE: &str =
        "the minimum annealing overlap is too large; write a length in nucleotides";

    let mut form = sid1(&["D404N"], TOO_LARGE);
    tab_to(&mut form, Field::Review);
    assert_eq!(
        form.handle(Key::Enter),
        FormAction::Continue,
        "an overlap past `i64` does not build"
    );
    let blocker = form.blocker().expect("the refusal names a field");
    assert_eq!(blocker.0, Field::Overlap, "the refusal is on the overlap");
    assert_eq!(blocker.1, None, "the refusal is not about a mutation line");
    assert_eq!(
        blocker.2, MESSAGE,
        "the refusal says the value is too large"
    );
    assert_eq!(
        form.focus(),
        Field::Overlap,
        "focus moves to the field the refusal names"
    );

    // The same with words typed on a GenBank template, as `new_words.rs` builds
    // it: `[ Review ]` refuses before the words' design is built, so the message
    // is the same, and `words_profile` never panics on the value.
    let mut form = NewForm::new(None, true);
    tab_to(&mut form, Field::File);
    type_text(&mut form, "synthetic-linear.gb");
    assert_eq!(
        form.handle(Key::Tab),
        FormAction::Load("synthetic-linear.gb".to_string()),
        "leaving File with a path asks the loop to read it"
    );
    form.loaded(
        "synthetic-linear.gb",
        Ok(read_file("genbank/synthetic-linear.gb")),
    );
    tab_to(&mut form, Field::Mutations);
    type_text(&mut form, "delete base 42");
    tab_to(&mut form, Field::Overlap);
    type_text(&mut form, TOO_LARGE);
    tab_to(&mut form, Field::Review);
    assert_eq!(
        form.handle(Key::Enter),
        FormAction::Continue,
        "an overlap past `i64` does not send the words to the model"
    );
    let blocker = form.blocker().expect("the refusal names a field");
    assert_eq!(
        blocker.0,
        Field::Overlap,
        "the words path reads the same blocker"
    );
    assert_eq!(
        blocker.2, MESSAGE,
        "the same message, with no panic in the words' design"
    );
}

// M7-45 (contract §3 test 1): `Overlap` opens at 15 nt, shown and editable, and
// a `--design` profile's `minAnnealingOverlap` replaces it, so the value the user
// sees is there before anything is typed.
#[test]
fn overlap_opens_at_15() {
    let form: NewForm = NewForm::new(None, false);
    assert_eq!(
        form.overlap(),
        "15",
        "the form opens with the overlap shown"
    );

    // The profile's own value, read from the fixture, not typed into the test.
    let profile: ProfileDocument =
        parse_profile(&read_file("nl/profile/full.json")).expect("the profile is a profile");
    let given: i64 = fixture("nl/profile/full.json")["design"]["minAnnealingOverlap"]
        .as_i64()
        .expect("the fixture states its own overlap");
    let form: NewForm = NewForm::new(Some(&profile), false);
    assert_eq!(
        form.overlap(),
        given.to_string(),
        "the profile's value replaces the prefilled 15"
    );
}

// M7-45 (contract §2.1), M7-48: `Overlap`'s place in the focus order is its place on
// screen, so `Tab` from `Mutations` reaches it, then the button. The request's
// name is typed in the pop-up, so it is not a field to tab to.
#[test]
fn focus_order_is_screen_order() {
    let mut form = sid1(&["D404N"], "15");
    tab_to(&mut form, Field::Mutations);
    let seen: Vec<Field> = focus_sequence(&mut form, 2);
    assert_eq!(
        seen[1..],
        [Field::Overlap, Field::Review],
        "tabbing from Mutations reaches Overlap, then the button"
    );
}

// M7-46 (contract §2.1): with no codon field, a line that states a codon carries
// it, and a line that states none takes the default policy.
#[test]
fn a_typed_codon_is_used_and_none_is_the_default() {
    let mut form = sid1(&["D404N:AAC", "D405A"], "15");
    let built: Built = review(&mut form);
    let codons: Vec<CodonSpec> = match &built.request.operation {
        OperationSpec::Variants { members, .. } => members
            .iter()
            .map(|member| match &member.edit {
                EditSpec::Residue { codon, .. } => codon.clone(),
                other => panic!("the lines give residue edits, gave {:?}", other),
            })
            .collect(),
        other => panic!("the lines give a set of variants, gave {:?}", other),
    };
    assert_eq!(
        codons,
        vec![CodonSpec::Supplied("AAC".to_string()), CodonSpec::Default],
        "the typed codon is used and a line stating none takes the default policy"
    );
}

// M7-51 (contract §2.4): in the paste and mutation boxes `PgDn`/`PgUp` move the
// cursor 10 lines, clamped to the first and the last line and to the line's
// length; on every other field focus jumps to the ends of the form, and leaving
// `File` this way asks the loop to read the path, as `tab` does.
#[test]
fn page_keys_move_ten_lines_or_to_the_ends() {
    // Twelve mutation lines, so the box holds 13 lines, the last one the empty
    // line the final `Enter` starts.
    let typed: Vec<String> = (1..=12).map(|n| format!("A{}G", n)).collect();
    let lines: Vec<&str> = typed.iter().map(|one| one.as_str()).collect();
    let mut form = sid1(&lines, "15");
    tab_to(&mut form, Field::Mutations);
    assert_eq!(
        form.focus_line(),
        12,
        "the cursor starts on the last line typed"
    );

    form.handle(Key::PageUp);
    assert_eq!(form.focus_line(), 2, "PgUp moves the cursor 10 lines up");
    for _ in 0..3 {
        form.handle(Key::Right);
    }
    assert_eq!(form.cursor(), 3, "the cursor sits at the line's end");
    form.handle(Key::PageDown);
    assert_eq!(
        form.focus_line(),
        12,
        "PgDn moves 10 lines down, clamped to the last line"
    );
    assert_eq!(
        form.cursor(),
        0,
        "the column is clamped to the length of the line it lands on"
    );
    form.handle(Key::PageUp);
    form.handle(Key::PageUp);
    assert_eq!(
        form.focus_line(),
        0,
        "PgUp is clamped to the first line, not past it"
    );
    form.handle(Key::PageDown);
    assert_eq!(
        form.focus_line(),
        10,
        "and 10 lines down from the first line"
    );

    // Off the boxes, focus goes to the ends of the form.
    tab_to(&mut form, Field::Overlap);
    assert_eq!(
        form.handle(Key::PageDown),
        FormAction::Continue,
        "PgDn on a single-line field asks nothing of the loop"
    );
    assert_eq!(form.focus(), Field::Review, "PgDn goes to [ Review ]");
    form.handle(Key::PageUp);
    assert_eq!(
        form.focus(),
        Field::Source,
        "PgUp goes to the first visible field"
    );

    // Leaving `File` this way asks the loop to read the path, as `tab` does.
    let mut form = NewForm::new(None, false);
    tab_to(&mut form, Field::File);
    type_text(&mut form, "sid-1_cDNA.fa");
    assert_eq!(
        form.handle(Key::PageDown),
        FormAction::Load("sid-1_cDNA.fa".to_string()),
        "PgDn leaving File asks the loop to read it, as Tab does"
    );
    assert_eq!(form.focus(), Field::Review, "and focus goes to the button");
}

// M7-48 (contract §3 test 1): `[ Save and run ]` opens the pop-up that names the
// file, holding the name it suggested from the template's name, builds the
// request, and asks the loop for nothing, so nothing is written.
#[test]
fn save_opens_the_popup_with_the_suggested_name() {
    let mut form = sid1(&["D404N"], "15");
    tab_to(&mut form, Field::Review);
    assert_eq!(
        form.handle(Key::Enter),
        FormAction::Continue,
        "the button opens the pop-up, so the loop is asked for nothing"
    );
    assert_eq!(form.popup(), Popup::Naming, "the pop-up is open, naming");
    assert_eq!(
        form.save_as(),
        "sid-1.request.json",
        "the pop-up's name field holds the name suggested from the template's name"
    );
    assert_eq!(
        form.popup_failure(),
        None,
        "the pop-up opens with no failure"
    );
}

// M7-48 (contract §3 test 2): the pop-up's name is edited with the keys a text
// field takes, and the two names that cannot be written are its own failures, so
// the loop is asked for nothing, and a name that is a name builds as the button
// did.
#[test]
fn the_popups_name_is_edited_and_checked() {
    let mut form = sid1(&["D404N"], "15");
    tab_to(&mut form, Field::Review);
    form.handle(Key::Enter);
    for _ in 0..form.save_as().chars().count() {
        form.handle(Key::Backspace);
    }
    assert_eq!(form.save_as(), "", "Backspace clears the suggested name");
    assert_eq!(
        form.handle(Key::Enter),
        FormAction::Continue,
        "an empty name asks the loop for nothing"
    );
    assert_eq!(
        form.popup_failure(),
        Some("write the request's file name".to_string()),
        "the pop-up names the missing name"
    );
    assert_eq!(form.popup(), Popup::Naming, "the pop-up stays open");

    type_text(&mut form, "sid-1/a.json");
    assert_eq!(
        form.handle(Key::Enter),
        FormAction::Continue,
        "a path asks the loop for nothing"
    );
    assert_eq!(
        form.popup_failure(),
        Some("write a file name, not a path".to_string()),
        "the pop-up says a path is no name"
    );

    // A name that is a name: `enter` builds exactly as `[ Save and run ]` did.
    for _ in 0..form.save_as().chars().count() {
        form.handle(Key::Backspace);
    }
    type_text(&mut form, "x.json");
    match form.handle(Key::Enter) {
        FormAction::Save { built, replace } => {
            let members: usize = match &built.request.operation {
                OperationSpec::Edit(_) => 1,
                OperationSpec::Variants { members, .. } => members.len(),
                other => panic!("the line gives a variant set, gave {:?}", other),
            };
            assert_eq!(members, 1, "the built request is the one the button built");
            assert!(!replace, "enter asks for no replace");
            assert_eq!(form.save_as(), "x.json", "the typed name is the name");
        }
        other => panic!("the pop-up gave {:?} instead of the built request", other),
    }
}

// M7-48 (contract §3 test 3): `esc` on the pop-up closes it and leaves the form,
// so `esc` is not the form's quit, and the form's values, its name included, are
// kept.
#[test]
fn esc_closes_the_popup_and_keeps_the_form() {
    let mut form = sid1(&["D404N"], "15");
    tab_to(&mut form, Field::Review);
    form.handle(Key::Enter);
    type_text(&mut form, "-a");
    assert_eq!(
        form.handle(Key::Esc),
        FormAction::Continue,
        "esc closes the pop-up, so the loop is asked for nothing"
    );
    assert_eq!(form.popup(), Popup::Closed, "the pop-up is closed");
    assert_eq!(form.focus(), Field::Review, "focus stays on the button");
    assert_eq!(
        form.save_as(),
        "sid-1.request.json-a",
        "the name typed in the pop-up is kept"
    );
    // The form takes its own keys again, `esc` included, which is its quit.
    assert_eq!(
        form.handle(Key::Esc),
        FormAction::Quit,
        "esc quits from the form"
    );
}

// M7-49 (contract §3 test 4): the pop-up that names the files already there
// replaces only on `y`, which gives the action `enter` gave with `replace` true,
// and any other key writes nothing and says so.
#[test]
fn a_taken_name_asks_and_only_y_replaces() {
    let mut form = sid1(&["D404N"], "15");
    tab_to(&mut form, Field::Review);
    form.handle(Key::Enter);
    // `enter` on the name is the `Save` the loop answers by asking, since the
    // name is there, so the action `enter` gave is kept for `y` to answer.
    let first: Built = match form.handle(Key::Enter) {
        FormAction::Save { built, replace } => {
            assert!(!replace, "enter asks for no replace");
            *built
        }
        other => panic!("the pop-up gave {:?} instead of the built request", other),
    };
    form.ask_replace(vec!["sid-1.request.json".to_string()]);
    assert_eq!(
        form.popup(),
        Popup::Replace(vec!["sid-1.request.json".to_string()]),
        "the pop-up names the file that is there"
    );
    assert_eq!(
        form.handle(Key::Char('n')),
        FormAction::Continue,
        "a key that is not `y` asks the loop for nothing"
    );
    assert_eq!(form.popup(), Popup::Naming, "the name pop-up is back");
    assert_eq!(
        form.popup_failure(),
        Some("nothing was written".to_string()),
        "the pop-up says nothing was written"
    );
    // The loop asks again with the same name, and `y` answers it.
    form.ask_replace(vec!["sid-1.request.json".to_string()]);
    match form.handle(Key::Char('y')) {
        FormAction::Save { built, replace } => {
            assert!(replace, "y is the only key that replaces");
            assert_eq!(*built, first, "the request `enter` built is the one y runs");
        }
        other => panic!("the pop-up gave {:?} instead of the built request", other),
    }
}

// M7-50 (contract §3 tests 3 to 6): the file list. The listings are given to the
// form through `set_listing`, exactly as the loop gives them, so the form reads
// no directory, and the entries are in `picker::list`'s order: folders first,
// then the template files in name order.
/// The listing of the current directory the loop gives: the folder `genbank`,
/// then `sid-1_cDNA.fa` and `syn.fa`, as `picker::list` sorts them.
fn listed(form: &mut NewForm) {
    form.set_listing(
        String::new(),
        vec![
            Entry {
                name: "genbank".to_string(),
                folder: true,
            },
            Entry {
                name: "sid-1_cDNA.fa".to_string(),
                folder: false,
            },
            Entry {
                name: "syn.fa".to_string(),
                folder: false,
            },
        ],
    );
}

/// The `choices()` names, as the picker shows them.
fn names(form: &NewForm) -> Vec<String> {
    form.choices()
        .iter()
        .map(|entry| entry.name.clone())
        .collect()
}

#[test]
fn typing_filters_the_choices() {
    let mut form = NewForm::new(None, false);
    tab_to(&mut form, Field::File);
    listed(&mut form);
    assert_eq!(
        names(&form),
        vec!["genbank", "sid-1_cDNA.fa", "syn.fa"],
        "the whole listing is shown while `File` is empty"
    );
    type_text(&mut form, "s");
    assert_eq!(
        names(&form),
        vec!["sid-1_cDNA.fa", "syn.fa"],
        "typing `s` keeps the two files, and the folder whose name does not start with it"
    );
    type_text(&mut form, "i");
    assert_eq!(
        names(&form),
        vec!["sid-1_cDNA.fa"],
        "typing `si` keeps the one file it names"
    );
    for _ in 0..2 {
        form.handle(Key::Backspace);
    }
    type_text(&mut form, "x");
    assert!(form.choices().is_empty(), "typing `x` keeps none");
}

#[test]
fn down_and_enter_choose_a_file_and_load_it() {
    let mut form = NewForm::new(None, false);
    tab_to(&mut form, Field::File);
    listed(&mut form);
    // The file is sorted after the folder, so its own position is picked.
    let at: usize = form
        .choices()
        .iter()
        .position(|entry| !entry.folder && entry.name == "sid-1_cDNA.fa")
        .expect("the file is a choice");
    for _ in 0..at + 1 {
        form.handle(Key::Down);
    }
    assert_eq!(
        form.chosen(),
        Some(at),
        "`Down` highlights the entry at its own position in `choices()`"
    );
    assert_eq!(
        form.handle(Key::Enter),
        FormAction::Load("sid-1_cDNA.fa".to_string()),
        "`enter` on a file leaves `File` exactly as `tab` does, so the loop reads it"
    );
    assert_eq!(
        form.file(),
        "sid-1_cDNA.fa",
        "the chosen name completes the typed path"
    );
    assert_eq!(
        form.chosen(),
        None,
        "the highlight is cleared with the choice"
    );
}

#[test]
fn a_folder_is_entered_not_loaded() {
    let mut form = NewForm::new(None, false);
    tab_to(&mut form, Field::File);
    listed(&mut form);
    let at: usize = form
        .choices()
        .iter()
        .position(|entry| entry.folder && entry.name == "genbank")
        .expect("the folder is a choice");
    for _ in 0..at + 1 {
        form.handle(Key::Down);
    }
    assert_eq!(
        form.handle(Key::Enter),
        FormAction::Continue,
        "`enter` on a folder asks the loop for nothing, so nothing is loaded"
    );
    assert_eq!(form.focus(), Field::File, "focus stays on `File`");
    assert_eq!(
        form.file(),
        "genbank/",
        "the folder's name is typed with its slash"
    );
    assert_eq!(
        form.file_dir(),
        "genbank/",
        "the typed path is now the directory the loop lists"
    );
    assert_eq!(
        form.chosen(),
        None,
        "the highlight is cleared with the choice"
    );
}

#[test]
fn up_and_down_move_focus_without_choices() {
    let mut form = NewForm::new(None, false);
    tab_to(&mut form, Field::File);
    assert!(form.choices().is_empty(), "no listing set gives no choices");
    form.handle(Key::Down);
    // With Source = file `Paste` does not apply, so the next visible field after
    // `File` is `Name`, as 7k3's skipping rule states.
    assert_eq!(
        form.focus(),
        Field::Name,
        "with no choices `Down` moves focus to the next visible field, as it does today"
    );
    form.handle(Key::Up);
    assert_eq!(
        form.focus(),
        Field::File,
        "with no choices `Up` moves focus back, as it does today"
    );
}
