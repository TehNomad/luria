//! The `luria new` form as the lines its screen shows (milestone 7k4, contract
//! 7k4): plain text lines, as every review screen is built (plan §5.1), read
//! from `NewForm`'s accessors only. `view` draws these lines and adds the frame,
//! the styles, the scroll and the cursor; no character of a line is changed by
//! the drawing, so a test can read the form's text without a terminal.
//!
//! # The rules this module applies (contract §2)
//!
//! - one row per field, in `Field`'s order, with the label padded to 11
//!   characters; the template's sub-fields are indented by two spaces;
//! - a row whose field is hidden by 7k3's `is_visible` is left out, so a GenBank
//!   record has no `Name`, `Shape` or `Strand` row and `First`/`Last` appear
//!   only as the `Bases` row when the CDS is typed bases;
//! - `◉` marks the chosen option and `○` the other on every choice row;
//! - under the `File` row, and before its `<name>, FASTA` note, the file list
//!   (M7-50): one continued row per choice, at most 8, `> name` for the
//!   highlighted one and `  name` for the rest, a folder's name ending with `/`,
//!   and a 9th row `  … <n> more` when there are more than 8, the window moved so
//!   the highlighted choice is shown; `picker_rows` says where they are drawn;
//! - each mutation line is shown as typed, then one preview line per non-blank
//!   line, in order: `✓ <n>  <words>` from `reading_words`, `✗ <n>  <message>`
//!   from the refusal's own `message()`, and `… <n>  read in words by the model at
//!   [ Review ]` for a line luria cannot read at all when the box reads as words
//!   and a backend was given, or `✗ <n>  not shorthand; …` without one (plan §2
//!   rule 1, M7-47); with a backend, words, and no loaded GenBank record, one
//!   more row follows the previews, `needs a GenBank template`;
//! - a mutation line's 1-based number and the `Bases` row use the screens'
//!   1-based inclusive coordinates (M7-24, D006), so `reading_words` reuses
//!   `review::text`'s `bases_at`, `insertion_place` and `bases_text` with no
//!   second conversion;
//! - every line has its trailing spaces cut, so an empty text field shows
//!   nothing after its label and an empty mutation line is an empty string;
//! - the button reads `[ Save and run ]`, the `Save as` row is gone, and the
//!   paste's `template saved as` row is in the pop-up, whose lines
//!   `popup_lines` writes (M7-48, M7-49); `cursor` is `None` while a pop-up is
//!   open, because the cursor is the pop-up's.

use crate::new::form::CdsMode;
use crate::new::form::Field;
use crate::new::form::NewForm;
use crate::new::form::Popup;
use crate::new::form::Source;
use crate::new::picker::Entry;
use crate::request::StrandSpec;
use crate::request::TopologySpec;
use crate::review::text::bases_at;
use crate::review::text::bases_text;
use crate::review::text::insertion_place;
use crate::shorthand::Reading;
use crate::shorthand::Refusal;

/// The form as lines, the index of the focused field's first line, and the cursor's
/// column in that line when the focus is a text field (contract §2).
pub struct FormText {
    pub lines: Vec<String>,
    pub focus_line: usize,
    pub cursor: Option<usize>,
    /// The file list's rows (M7-50): the first line's index and how many rows
    /// are drawn, the `… <n> more` row counted with them, or `None` when the
    /// picker shows nothing. `view` styles these rows by position.
    pub picker_rows: Option<(usize, usize)>,
}

/// The label column every row is padded to (contract §2.1).
const LABEL: usize = 11;

/// The choice rows the picker draws under `File` at most (M7-50, contract §2.3).
const PICKER_WINDOW: usize = 8;

/// The fields in the order the screen shows them (contract §2.1), which is
/// 7k3's focus order, with the primers' own row and the box's `needs a GenBank
/// template` row written where §2.3 puts them. The request's name is not a row:
/// it is typed in the pop-up (M7-48).
const ROWS: [Field; 12] = [
    Field::Source,
    Field::File,
    Field::Paste,
    Field::Name,
    Field::Shape,
    Field::Cds,
    Field::First,
    Field::Last,
    Field::Strand,
    Field::Mutations,
    Field::Overlap,
    Field::Review,
];

/// The form as lines, the index of the focused field's first line, and the
/// cursor's column in that line when the focus is a text field.
pub fn form_lines(form: &NewForm) -> FormText {
    let mut lines: Vec<String> = Vec::new();
    // The row each field's first line is drawn on, so `focus_line` and the
    // cursor can be found from the focused field.
    let mut rows: Vec<(Field, usize)> = Vec::new();
    let mut picker: Option<(usize, usize)> = None;
    for field in ROWS.iter() {
        let row: usize = lines.len();
        match *field {
            Field::Source => {
                rows.push((*field, row));
                lines.push(row_line(
                    "Template",
                    &choice(&["file", "paste"], form.source() == Source::File),
                ));
            }
            Field::File => {
                if !form.is_visible(*field) {
                    continue;
                }
                rows.push((*field, row));
                lines.push(row_line("  File", form.file().as_str()));
                // M7-50: the file list, under the row and before its note: one
                // continued row per choice, `> ` on the highlighted one, and the
                // `… <n> more` row when the listing is longer than the window.
                let picked: Option<(usize, usize)> = picker_rows(form, &mut lines);
                if let Some(name) = form.template_file() {
                    let note: String = match form.locus() {
                        Some(locus) => format!("{}, GenBank {}", name, locus),
                        None => format!("{}, FASTA", name),
                    };
                    lines.push(continued(note.as_str()));
                }
                picker = picked;
            }
            Field::Paste => {
                if !form.is_visible(*field) {
                    continue;
                }
                rows.push((*field, row));
                for (index, one) in form.paste_text().split('\n').enumerate() {
                    if index == 0 {
                        lines.push(row_line("  Paste", one));
                    } else {
                        lines.push(continued(one));
                    }
                }
            }
            Field::Name => {
                if !form.is_visible(*field) {
                    continue;
                }
                rows.push((*field, row));
                lines.push(row_line("  Name", form.name().as_str()));
            }
            Field::Shape => {
                if !form.is_visible(*field) {
                    continue;
                }
                rows.push((*field, row));
                lines.push(row_line(
                    "  Shape",
                    &choice(
                        &["linear", "circular"],
                        form.shape() == TopologySpec::Linear,
                    ),
                ));
            }
            Field::Cds => {
                rows.push((*field, row));
                if form.cds_mode() == CdsMode::Catalogue {
                    let entry = form.cds_list()[form.cds_index()].clone();
                    let chosen: String = format!(
                        "◀ {}  {}  {} ▶",
                        entry.id,
                        entry.gene.unwrap_or_else(|| "-".to_string()),
                        entry.product.unwrap_or_else(|| "-".to_string())
                    );
                    lines.push(row_line("  CDS", chosen.as_str()));
                } else {
                    lines.push(row_line(
                        "  CDS",
                        &choice(
                            &["whole sequence", "bases"],
                            form.cds_mode() == CdsMode::Whole,
                        ),
                    ));
                }
            }
            Field::First => {
                if !form.is_visible(*field) {
                    continue;
                }
                // `First` and `Last` share the one `Bases` row (contract §2.1).
                rows.push((Field::First, row));
                rows.push((Field::Last, row));
                lines.push(row_line(
                    "  Bases",
                    &format!("{}–{}", form.first(), form.last()),
                ));
            }
            Field::Last => {
                // Its row is the `Bases` row, written at `First`.
                continue;
            }
            Field::Strand => {
                if !form.is_visible(*field) {
                    continue;
                }
                rows.push((*field, row));
                lines.push(row_line(
                    "  Strand",
                    &choice(&["forward", "reverse"], form.strand() == StrandSpec::Plus),
                ));
                // The blank row §2.1 puts between the template and the box.
                lines.push(String::new());
            }
            Field::Mutations => {
                rows.push((*field, row));
                for (index, one) in form.mutations().iter().enumerate() {
                    if index == 0 {
                        lines.push(row_line("Mutations", one));
                    } else {
                        lines.push(continued(one));
                    }
                }
                // M7-47: a line luria cannot read at all makes the whole box
                // words, so its preview says the model reads it, and names
                // `--backend` when none was given.
                let as_words: bool = form.reads_as_words();
                for preview in form.previews().iter() {
                    match &preview.reading {
                        None => {}
                        Some(Ok(reading)) => lines.push(continued(&format!(
                            "✓ {}  {}",
                            preview.line,
                            reading_words(reading)
                        ))),
                        Some(Err(refusal)) => {
                            if as_words && *refusal == Refusal::Unreadable {
                                if form.backend() {
                                    lines.push(continued(&format!(
                                        "… {}  read in words by the model at [ Review ]",
                                        preview.line
                                    )));
                                } else {
                                    lines.push(continued(&format!(
                                        "✗ {}  not shorthand; start luria new with --backend to read it in words",
                                        preview.line
                                    )));
                                }
                            } else {
                                lines.push(continued(&format!(
                                    "✗ {}  {}",
                                    preview.line,
                                    refusal.message()
                                )));
                            }
                        }
                    }
                }
                // M7-47: the words need a record the form has not loaded, so the
                // box says what it needs, under its previews.
                if form.words_need_genbank() {
                    lines.push(continued("needs a GenBank template"));
                }
            }
            Field::Overlap => {
                // The blank row §2.3 puts under the box, then the primers' own,
                // which is the plan's rule (M7-39, D029), not a form field.
                lines.push(String::new());
                lines.push(row_line(
                    "Primers",
                    "inline (confine to tail not supported yet)",
                ));
                // `Overlap`'s own row is the one after them, so the cursor and
                // the focus mark sit on the value typed.
                rows.push((*field, lines.len()));
                lines.push(row_line("Overlap", &format!("{} nt", form.overlap())));
            }
            Field::Review => {
                rows.push((*field, row));
                lines.push(continued("[ Save and run ]"));
            }
        }
    }
    if let Some((_, _, message)) = form.blocker() {
        lines.push(String::new());
        let mut row: String = String::from("✗ ");
        if let Some(line) = form.blocker().unwrap().1 {
            row.push_str(&format!("line {}: ", line));
        }
        row.push_str(&message);
        lines.push(row);
    }
    let mut focus: usize = 0;
    for (field, row) in rows.iter() {
        if *field == form.focus() {
            focus = row + form.focus_line();
            break;
        }
    }
    FormText {
        lines: lines
            .iter()
            .map(|line| line.trim_end().to_string())
            .collect(),
        focus_line: focus,
        cursor: typed_cursor(form),
        picker_rows: picker,
    }
}

/// The `File` row's file list, written after the row: `> name` for the
/// highlighted choice and `  name` for the rest, a folder's name ending with `/`,
/// at most `PICKER_WINDOW` rows, and the `… <n> more` row when the listing has
/// more than that, with the window moved so the highlighted choice is shown
/// (M7-50, contract §2.3). The first row written and how many, or `None` with no
/// choices, which writes no rows.
fn picker_rows(form: &NewForm, lines: &mut Vec<String>) -> Option<(usize, usize)> {
    let choices: Vec<Entry> = form.choices();
    if choices.is_empty() {
        return None;
    }
    let first: usize = lines.len();
    let start: usize = match form.chosen() {
        Some(at) if choices.len() > PICKER_WINDOW => at
            .saturating_sub(PICKER_WINDOW - 1)
            .min(choices.len() - PICKER_WINDOW),
        _ => 0,
    };
    let chosen: Option<usize> = form.chosen();
    let shown: usize = PICKER_WINDOW.min(choices.len() - start);
    for (index, entry) in choices.iter().enumerate().skip(start).take(shown) {
        let name: String = if entry.folder {
            format!("{}/", entry.name)
        } else {
            entry.name.clone()
        };
        let row: String = if chosen == Some(index) {
            format!("> {}", name)
        } else {
            format!("  {}", name)
        };
        lines.push(continued(row.as_str()));
    }
    let mut count: usize = lines.len() - first;
    if choices.len() > PICKER_WINDOW {
        lines.push(continued(&format!(
            "  … {} more",
            choices.len() - PICKER_WINDOW
        )));
        count += 1;
    }
    Some((first, count))
}

/// The cursor's column in the drawn line, when the focus is a text field: the
/// label's 11 characters, then the cursor's column in the field's line. While a
/// pop-up is open the cursor belongs to the pop-up, so the form's screen has
/// none (M7-48).
fn typed_cursor(form: &NewForm) -> Option<usize> {
    if form.popup() != Popup::Closed {
        return None;
    }
    match form.focus() {
        Field::File
        | Field::Paste
        | Field::Name
        | Field::First
        | Field::Last
        | Field::Mutations
        | Field::Overlap => Some(LABEL + form.cursor()),
        _ => None,
    }
}

/// The pop-up's lines as they are drawn (M7-48, M7-49, contract §2.4). On
/// `Naming`, the request's file name, where it goes, for a paste the FASTA the
/// form writes first, the failure line when there is one, then the keys. On
/// `Replace`, the files that exist, named in the order given, then the keys.
pub fn popup_lines(form: &NewForm) -> Vec<String> {
    match form.popup() {
        Popup::Closed => Vec::new(),
        Popup::Naming => {
            let mut lines: Vec<String> = Vec::new();
            lines.push(row_line("Request", form.save_as().as_str()));
            // Where the request goes: beside the template file, as M7-37
            // requires, and in the current directory for a paste.
            if form.source() == Source::Paste {
                lines.push(continued("in the current directory"));
                lines.push(row_line(
                    "Template",
                    &format!("{}, from the paste", form.paste_file()),
                ));
            } else {
                lines.push(continued(&format!(
                    "beside {}",
                    form.template_file().unwrap_or_default()
                )));
            }
            if let Some(failure) = form.popup_failure() {
                lines.push(format!("✗ {}", failure));
            }
            lines.push(String::new());
            lines.push("enter save and run  esc back".to_string());
            lines
        }
        Popup::Replace(names) => {
            let mut lines: Vec<String> = Vec::new();
            lines.push("These files exist:".to_string());
            for name in names.iter() {
                lines.push(format!("  {}", name));
            }
            lines.push(String::new());
            lines.push("y replace them  any other key: back".to_string());
            lines
        }
    }
}

/// One row: `label` padded to `LABEL`, then `value`; trailing spaces cut.
fn row_line(label: &str, value: &str) -> String {
    format!("{:<width$}{}", label, value, width = LABEL)
        .trim_end()
        .to_string()
}

/// A row that continues its field: `LABEL` spaces, then `text`; trailing
/// spaces cut, so an empty line is an empty string.
fn continued(text: &str) -> String {
    format!("{:<width$}{}", "", text, width = LABEL)
        .trim_end()
        .to_string()
}

/// A choice row: `◉` on the chosen option, `○` on the other, the two options
/// split by the two spaces the form's rows use.
fn choice(options: &[&str; 2], chosen_first: bool) -> String {
    let (first, second) = if chosen_first {
        ('◉', '○')
    } else {
        ('○', '◉')
    };
    format!("{} {}  {} {}", first, options[0], second, options[1])
}

/// A preview in words, 1-based (M7-24): `bases_at` and `insertion_place` give
/// the coordinates, `bases_text` a base text over 30 as `<N> bp`; no second
/// conversion (contract §2.2).
pub fn reading_words(reading: &Reading) -> String {
    match reading {
        Reading::Residue {
            number,
            from,
            to,
            codon,
        } => match codon {
            Some(given) => format!("residue {} {}→{}, codon {}", number, from, to, given),
            None => format!(
                "residue {} {}→{}, codon by default policy",
                number, from, to
            ),
        },
        Reading::Substitution {
            start,
            end,
            from,
            to,
        } => format!(
            "base {} {}→{}",
            bases_at(*start as u64, *end as u64),
            bases_text(from),
            bases_text(to)
        ),
        Reading::Replacement {
            start,
            end,
            from,
            to,
        } => format!(
            "bases {} {}→{}",
            bases_at(*start as u64, *end as u64),
            bases_text(from),
            bases_text(to)
        ),
        Reading::Deletion { start, end } => {
            let one: bool = end - start == 1;
            format!(
                "delete {} {}",
                if one { "base" } else { "bases" },
                bases_at(*start as u64, *end as u64)
            )
        }
        Reading::Insertion { point, insert } => format!(
            "insert {} {}",
            bases_text(insert),
            insertion_place(*point as u64)
        ),
    }
}
