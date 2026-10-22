//! The `luria new` form's text (milestone 7k4, contract 7k4 §4.1): the lines
//! `new::text::form_lines` builds are read as the screen shows them, with the
//! form driven by keys and `loaded` exactly as `tests/new_form.rs` drives it, so
//! every line is the state 7k3 holds, not a copy typed into the test. The
//! 1-based coordinates are M7-24's, taken from `review::text`'s own
//! `bases_at` and `insertion_place`, and the labels are padded to 11 characters
//! as contract §2.1 lays them out.

use luria_cli::new::form::Field;
use luria_cli::new::form::FormAction;
use luria_cli::new::form::NewForm;
use luria_cli::new::text::form_lines;
use luria_cli::new::text::reading_words;
use luria_cli::new::text::FormText;
use luria_cli::new::BuildRefusal;
use luria_cli::review::app::Key;
use luria_cli::shorthand::parse_line;
use luria_cli::shorthand::Reading;
use luria_cli::shorthand::Refusal;
use std::fs;

const FIXTURES: &str = "../../tests/fixtures/";
const SID1: &str = "example-construct/input/sid-1_cDNA.fa";

/// One fixture template, read at test time, as `tests/new_form.rs` does.
fn read_file(relative: &str) -> String {
    let path: String = format!("{}{}", FIXTURES, relative);
    fs::read_to_string(&path).unwrap_or_else(|problem| panic!("cannot read {}: {}", path, problem))
}

/// Types `text` one character at a time, as a user presses the keys.
fn type_text(form: &mut NewForm, text: &str) {
    for ch in text.chars() {
        form.handle(Key::Char(ch));
    }
}

/// `Tab` forward until focus reaches `wanted`, as `tests/new_form.rs` does.
fn tab_to(form: &mut NewForm, wanted: Field) {
    for _ in 0..30 {
        if form.focus() == wanted {
            return;
        }
        form.handle(Key::Tab);
    }
    panic!("focus never reached {:?}", wanted);
}

/// The form with the `sid-1` file loaded as the loop answers `Load`: the path
/// typed in `File`, the name `sid-1` typed over the stem the load filled, and
/// `lines` typed into the mutation box, each line's `Enter` starting the next,
/// so the box ends with an empty line. `overlap` is typed last.
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

/// The form with the GenBank fixture loaded as the loop answers `Load`.
fn genbank() -> NewForm {
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
    form
}

#[test]
fn the_sid1_form_reads_as_drawn() {
    let form: NewForm = sid1(
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
    let body: FormText = form_lines(&form);
    let expected: [&str; 28] = [
        "Template   ◉ file  ○ paste",
        "  File     sid-1_cDNA.fa",
        "           sid-1_cDNA.fa, FASTA",
        "  Name     sid-1",
        "  Shape    ◉ linear  ○ circular",
        "  CDS      ◉ whole sequence  ○ bases",
        "  Strand   ◉ forward  ○ reverse",
        "",
        "Mutations  D404N",
        "           del 1210-1212",
        "",
        "           ins GAT after 1209",
        "           1210 G>A",
        "           1210-1212 GAT>AAC",
        "",
        "           ✓ 1  residue 404 D→N, codon by default policy",
        "           ✓ 2  delete bases 1210–1212",
        "           ✓ 4  insert GAT after 1209",
        "           ✓ 5  base 1210 G→A",
        "           ✓ 6  bases 1210–1212 GAT→AAC",
        "In words   needs --backend",
        "",
        "Primers    inline (confine to tail not supported yet)",
        "Overlap    15 nt",
        "Codon      ◉ default policy  ○ typed per mutation (D404N:AAC)",
        "Save as    sid-1.request.json",
        "",
        "           [ Review ]",
    ];
    assert_eq!(
        body.lines,
        expected.to_vec(),
        "the sid-1 form as it is drawn"
    );
}

#[test]
fn a_genbank_cds_reads_as_its_entry() {
    let form: NewForm = genbank();
    let body: FormText = form_lines(&form);
    let cds: &String = body
        .lines
        .iter()
        .find(|line| line.starts_with("  CDS"))
        .expect("the CDS row is there for a GenBank record");
    assert_eq!(
        cds, "  CDS      ◀ synA  synA  synthetic protein A ▶",
        "the CDS row is the catalogue's chosen entry, its gene and product"
    );
    for label in ["  Name", "  Shape", "  Strand"] {
        assert!(
            !body.lines.iter().any(|line| line.starts_with(label)),
            "the {} row is left out on a GenBank record",
            label
        );
    }
}

#[test]
fn a_refusal_is_shown_and_named() {
    let mut form: NewForm = sid1(&["D404N", "D404*"], "15");
    tab_to(&mut form, Field::Review);
    assert_eq!(
        form.handle(Key::Enter),
        FormAction::Continue,
        "a line luria cannot read does not build"
    );
    let body: FormText = form_lines(&form);
    let wanted: String = format!("           ✗ 2  {}", Refusal::Stop.message());
    assert!(
        body.lines.contains(&wanted),
        "the refused line is shown with its refusal's message, seen {:?} first",
        body.lines[9].clone()
    );
    let last: &String = body.lines.last().expect("the form has lines");
    assert_eq!(
        last,
        &format!("✗ line 2: {}", BuildRefusal::LineRefused.message()),
        "the last line names the line the blocker is about"
    );
}

#[test]
fn reading_words_are_one_based() {
    for (line, words) in [
        ("D404N", "residue 404 D→N, codon by default policy"),
        ("D404N:AAC", "residue 404 D→N, codon AAC"),
        ("1210 G>A", "base 1210 G→A"),
        ("1210-1212 GAT>TT", "bases 1210–1212 GAT→TT"),
        ("del 46", "delete base 46"),
        ("del 1210-1212", "delete bases 1210–1212"),
        ("ins GAT after 1209", "insert GAT after 1209"),
        ("ins GAT after 0", "insert GAT at the start"),
    ] {
        let reading: Reading = parse_line(line).expect("the line reads");
        assert_eq!(reading_words(&reading), words, "the words of {}", line);
    }
}

#[test]
fn the_focus_line_follows_tab() {
    let mut form: NewForm = NewForm::new(None, false);
    assert_eq!(form.focus(), Field::Source, "the form opens on Source");
    form.handle(Key::Tab);
    assert_eq!(form.focus(), Field::File, "one Tab moves to File");
    let body: FormText = form_lines(&form);
    let file_row: usize = body
        .lines
        .iter()
        .position(|line| line.starts_with("  File"))
        .expect("the File row is there");
    assert_eq!(body.focus_line, file_row, "the focus line is the File row");
    assert!(body.cursor.is_some(), "the cursor is in the text field");
}
