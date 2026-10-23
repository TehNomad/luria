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
use luria_cli::new::picker::Entry;
use luria_cli::new::text::form_lines;
use luria_cli::new::text::popup_lines;
use luria_cli::new::text::reading_words;
use luria_cli::new::text::FormText;
use luria_cli::review::app::Key;
use luria_cli::shorthand::parse_line;
use luria_cli::shorthand::Reading;
use luria_cli::shorthand::Refusal;
use std::fs;

const FIXTURES: &str = "../../tests/fixtures/";
const SID1: &str = "example-construct/input/sid-1_cDNA.fa";

/// `requests.py`'s `SYN`: the 100 bases of the pasted sequence.
const SYN: &str = "ATGGCTTGGAATCAGCAGCAGCCGAGCGGCTCGCGGACGCATGGCTTGGAATCAGCAGCA\
                   GCCGAGCGGCTCGCGGACGCATGGCTTGGAATCAGCAGCA";

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
/// so the box ends with an empty line. `overlap` is typed last, over the 15 nt
/// M7-45 prefills `Overlap` with.
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

/// The pasted text of `requests.py`'s `syn_paste`: `SYN[:50]`, a newline and
/// two spaces, then `SYN[50:]` lower case.
fn typed_paste() -> String {
    format!("{}\n  {}", &SYN[..50], SYN[50..].to_lowercase())
}

/// The `syn-paste` form, as `tests/new_confirm.rs` builds it: a paste, its CDS
/// typed as bases 1 to 99, the name `syn`, two mutations, and `Overlap` at its
/// prefilled 15, so `[ Save and run ]` builds and the pop-up opens.
fn syn_paste() -> NewForm {
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
    let expected: [&str; 24] = [
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
        "",
        "Primers    inline (confine to tail not supported yet)",
        "Overlap    15 nt",
        "           [ Save and run ]",
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
    // §2.2 step 1: the blocker on `Mutations` carries the refusal's own message,
    // so the last line names the second line's stop refusal, not `line_refused`.
    assert_eq!(
        last,
        &format!("✗ line 2: {}", Refusal::Stop.message()),
        "the last line names the line the blocker is about, with the refusal's own message"
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

// M7-48 (contract §3 test 7): the pop-up says what it writes, where it goes, the
// failure it is holding, and the keys that answer it, and the pop-up that names
// the files already there names them in order (M7-49). With the pop-up open, the
// form's own body has no cursor, since the name typed carries it.
#[test]
fn the_popup_says_what_it_writes() {
    // The `sid-1` file form, with the pop-up `[ Save and run ]` opened.
    let mut form = sid1(&["D404N"], "15");
    tab_to(&mut form, Field::Review);
    form.handle(Key::Enter);
    let expected: [&str; 4] = [
        "Request    sid-1.request.json",
        "           beside sid-1_cDNA.fa",
        "",
        "enter save and run  esc back",
    ];
    assert_eq!(
        popup_lines(&form),
        expected.to_vec(),
        "the file form says the name, and the template file it goes beside"
    );
    assert!(
        form_lines(&form).cursor.is_none(),
        "the form's cursor is the pop-up's, not the field's"
    );

    // The paste form: the request and the pasted FASTA, both in the current
    // directory, with the failure the write gave.
    let mut paste = syn_paste();
    tab_to(&mut paste, Field::Review);
    paste.handle(Key::Enter);
    paste.save_failed("nothing was written".to_string());
    let expected: [&str; 6] = [
        "Request    syn.request.json",
        "           in the current directory",
        "Template   syn.fa, from the paste",
        "✗ nothing was written",
        "",
        "enter save and run  esc back",
    ];
    assert_eq!(
        popup_lines(&paste),
        expected.to_vec(),
        "the paste form says both files, and the failure line is its own"
    );

    // The pop-up that names the files luria would write that are there (M7-49).
    paste.ask_replace(vec!["syn.fa".to_string(), "syn.request.json".to_string()]);
    let expected: [&str; 5] = [
        "These files exist:",
        "  syn.fa",
        "  syn.request.json",
        "",
        "y replace them  any other key: back",
    ];
    assert_eq!(
        popup_lines(&paste),
        expected.to_vec(),
        "the paste is named first, then the request, and only y replaces"
    );
}

// M7-50 (contract §3 test 7): the file list is drawn under the `File` row,
// before its note, as one continued row per choice, `> name` for the highlighted
// one and `  name` for the rest, a folder's name ending with `/`, 8 rows at most
// and the `  … <n> more` row for the rest, the window moved so the highlighted
// choice is drawn.
/// The listing of ten entries the loop gives: the two folders `picker::list`
/// puts first, then eight files in name order.
fn ten() -> Vec<Entry> {
    [
        ("genbank", true),
        ("primers", true),
        ("a.fa", false),
        ("b.fa", false),
        ("c.fa", false),
        ("d.fa", false),
        ("e.fa", false),
        ("f.fa", false),
        ("g.fa", false),
        ("h.fa", false),
    ]
    .iter()
    .map(|(name, folder)| Entry {
        name: (*name).to_string(),
        folder: *folder,
    })
    .collect()
}

#[test]
fn the_choices_are_drawn_under_file() {
    let mut form = NewForm::new(None, false);
    tab_to(&mut form, Field::File);
    form.set_listing(String::new(), ten());
    let body: FormText = form_lines(&form);
    // `Template`'s row, then `File`'s, then the list, and no note row, as
    // nothing has been loaded.
    // Every row is written at the label column, so its two-character marker is
    // the part after the 11 label spaces.
    let expected: [&str; 9] = [
        "             genbank/",
        "             primers/",
        "             a.fa",
        "             b.fa",
        "             c.fa",
        "             d.fa",
        "             e.fa",
        "             f.fa",
        "             … 2 more",
    ];
    assert_eq!(
        body.picker_rows,
        Some((2, 9)),
        "the list starts under the File row, and the more row is counted"
    );
    assert_eq!(
        body.lines[2..11].to_vec(),
        expected.to_vec(),
        "8 rows, and the ninth names the 2 choices not shown"
    );

    // The highlighted row carries the `> ` marker, and the view styles it.
    form.handle(Key::Down);
    let body: FormText = form_lines(&form);
    assert_eq!(
        body.lines[2], "           > genbank/",
        "the highlighted folder is marked with `> ` and its name ends with `/`"
    );

    // With the highlight past the window, the window moves so it is drawn.
    for _ in 0..10 {
        form.handle(Key::Down);
    }
    let body: FormText = form_lines(&form);
    assert_eq!(body.picker_rows, Some((2, 9)), "the list keeps its rows");
    assert_eq!(
        body.lines[9], "           > h.fa",
        "the window ends at the highlighted choice, so the last entry is shown"
    );
    assert!(
        body.lines[2..10]
            .iter()
            .any(|line| line.trim_start().starts_with("> ")),
        "the highlighted row is inside the rows drawn"
    );

    // With no choices, the picker writes no rows.
    let mut empty = NewForm::new(None, false);
    tab_to(&mut empty, Field::File);
    let body: FormText = form_lines(&empty);
    assert_eq!(body.picker_rows, None, "no listing gives no rows");
}
