//! What the `luria new` form holds (milestone 7k3, contract 7k3): the text and
//! choices the user has typed, where focus is, the previews of each mutation
//! line, what `[ Save and run ]` builds, and the pop-up it writes from. Pure: no
//! terminal, no file I/O, no model (plan §5.1's rule for the screens holds here
//! too).
//!
//! The form is the state; its loop is 7k5. Reading a file is the loop's job: the
//! form asks for it with `FormAction::Load(path)` when the user leaves `File`
//! with a path, and is told the outcome with `NewForm::loaded`. `[ Save and run
//! ]` builds, `new::build`'s answer (contract 7k2), opens the pop-up that names
//! the file, and `Esc` gives `FormAction::Quit`. Drawing is 7k4, which reads the
//! accessors below; nothing is drawn, written or run here.
//!
//! # The rules this module applies (contract §3.1, M7-31 to M7-39)
//!
//! - focus: `Tab`/`Down` to the next visible field, `BackTab`/`Up` to the
//!   previous, wrapping; a field that does not apply is skipped, so `Paste` is
//!   skipped with Source = file, `File` with Source = paste, `Name`, `Shape` and
//!   `Strand` with a GenBank file, and `First`/`Last` unless the CDS is typed
//!   bases;
//! - inside `Paste` and `Mutations`, `Up`/`Down` move between lines and focus
//!   leaves the field only on `Tab`/`BackTab`; `PageDown`/`PageUp` move the
//!   cursor 10 lines in those two boxes, and put focus on the first or the last
//!   field anywhere else (M7-51);
//! - `Source`, `Shape`, `Cds` and `Strand` are choices: `Left`, `Right` or space
//!   gives the other value, and on a GenBank record `Cds` walks the catalogue's
//!   usable entries only, since an unusable CDS cannot be picked (plan rule 4,
//!   6c);
//! - `File`'s file list (M7-50): with the choices the loop listed for the
//!   directory typed in `File`, `Up`/`Down` highlight one and `Enter` picks it, a
//!   folder being typed into the path with focus kept, and a file completing the
//!   path and being left exactly as `Tab` leaves it, so the loop reads it; with no
//!   choices `Up`/`Down` move focus as before, and any edit of `File` clears the
//!   highlight;
//! - text: printable characters insert at the cursor, `Backspace`, `Delete`,
//!   `Left`, `Right` edit, `Enter` starts a line in a multi-line field and moves
//!   to the next field in a single-line one, and `Overlap`, `First` and `Last`
//!   take digits only;
//! - the previews follow every edit: each mutation line's 1-based number with
//!   what `shorthand::parse_line` says about it, and nothing for a blank line,
//!   so the screen shows what each line means (plan §2 rule 1);
//! - `[ Save and run ]` (the button, whose field keeps the name `Review`) checks
//!   the form's own blockers first, in order, moving focus to the field it names
//!   (plan §2 rule 2), then a mutation line luria reads but refuses, then the box
//!   read in words, then `new::build`, whose refusal names the field it concerns;
//!   the answer is given at once, `Interpret` for words and `Save` for the built
//!   request, so the first `Enter` is the one that writes and runs (M7-62, M7-63);
//! - one `Mutations` box (M7-47): a line luria cannot read at all, refusal
//!   `unreadable`, makes the whole box words (`reads_as_words`). With a backend
//!   `[ Save and run ]` asks the loop to send the box through `interpret`, which needs
//!   a GenBank template whose CDS list the model is shown (M7-41, M7-42), and
//!   gives `Interpret` with the path as typed in `File`, building nothing; without
//!   one the box is a blocker that names `--backend`. A line luria reads but
//!   refuses stays a red error and is never sent to the model;
//! - `Overlap` opens at 15 nt, which a `--design` profile's value replaces
//!   (M7-45), and the codon a line types is used, so `new::build` is always given
//!   the default codon policy (M7-46);
//! - `SaveAs` is filled from the name, and the name the user typed is never
//!   replaced (M7-37, M7-63); an empty name, and a name with a path separator, are
//!   blockers on `SaveAs`, checked before any words are sent, so a bad name never
//!   calls the model (M7-63); the action `[ Save and run ]` gave is kept, so when
//!   a file luria would write already exists `ask_replace` names it and only `y`
//!   repeats that action with `replace` true, while any other key says nothing was
//!   written (M7-49); a paste's FASTA is `<name>.fa`.

use crate::interpret::catalogue::catalogue;
use crate::interpret::catalogue::CatalogueEntry;
use crate::interpret::catalogue::Usability;
use crate::new::build;
use crate::new::picker::Entry;
use crate::new::BuildRefusal;
use crate::new::Built;
use crate::new::CdsChoice;
use crate::new::CodonChoice;
use crate::new::TemplateInput;
use crate::request::BufferSpec;
use crate::request::CodonSpec;
use crate::request::DesignSpec;
use crate::request::NumberingSpec;
use crate::request::PlacementSpec;
use crate::request::PolicySpec;
use crate::request::ProfileDocument;
use crate::request::StrandSpec;
use crate::request::StyleSpec;
use crate::request::TopologySpec;
use crate::review::app::Key;
use crate::shorthand::parse_line;
use crate::shorthand::Reading;
use crate::shorthand::Refusal;

use luria_core::genbank::read_genbank;
use luria_core::identity::Identifier;

/// One field of the form. The order of the variants is the order the screen
/// shows them, and so the order `Tab` moves through.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Source,    // file or paste
    File,      // the path typed (Source = file)
    Paste,     // the pasted text (Source = paste), multi-line
    Name,      // the template's name (FASTA and paste only)
    Shape,     // linear or circular (FASTA and paste only)
    Cds,       // whole, or bases (FASTA and paste); the CDS list (GenBank)
    First,     // 1-based CDS start (CdsChoice::Bases only)
    Last,      // 1-based CDS end (CdsChoice::Bases only)
    Strand,    // forward or reverse (FASTA and paste only)
    Mutations, // multi-line
    Overlap,   // digits, 15 nt at opening (M7-45)
    SaveAs,    // the request file's name (M7-63)
    Review,    // `[ Save and run ]`
}

/// The fields in focus order: the order the screen's rows are in (M7-45), the
/// request's name being a field again (M7-63).
const ORDER: [Field; 13] = [
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
    Field::SaveAs,
    Field::Review,
];

/// The pop-up the form opens over itself (M7-49). The request's name is a field
/// again (M7-63), so the only pop-up left asks before replacing a file.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Popup {
    /// No pop-up: the form takes its own keys.
    Closed,
    /// The files luria would write that already exist, named in order (M7-49):
    /// `y` replaces them, any other key writes nothing.
    Replace(Vec<String>),
}

/// What the form asks its loop to do after a key. `Load`, `Save` and
/// `SaveWords` are the only I/O the form asks for, and it starts none itself.
#[derive(Clone, Debug, PartialEq)]
pub enum FormAction {
    Continue,
    Quit,
    /// The user left `File` with a path: the loop reads it and calls
    /// `NewForm::loaded`.
    Load(String),
    /// `[ Save and run ]` with every field valid and the `SaveAs` name written:
    /// the built request, returned at once (M7-63), and `replace` true when the
    /// user answered `y` on the pop-up that named the files already there (M7-49).
    Save {
        built: Box<Built>,
        replace: bool,
    },
    /// The `y` on the pop-up that names a taken file, repeating the save of the
    /// words `[ Save and run ]` sent (M7-62): the loop writes the request at the
    /// path it builds from `save_as`. The form built nothing.
    SaveWords {
        replace: bool,
    },
    /// `[ Review ]` with the mutation box read in words (M7-47): the loop sends
    /// the whole box through `interpret` with the template at `template`, the path
    /// typed in `File`, and shows the proposal. `profile` is the design the form
    /// shows, which the words are read with (M7-41), and `text` is every mutation
    /// line joined with newlines. `cds` is the CDS picked on the form's `CDS`
    /// field, the id of `cds_list[cds_index]`, which the words name no CDS use
    /// (M7-59). The form built nothing.
    Interpret {
        template: String,
        text: String,
        profile: ProfileDocument,
        cds: String,
    },
}

/// The template's source, the choice on the `Source` field (M7-32).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    File,
    Paste,
}

/// How the CDS is stated: the form's own for FASTA and paste (M7-32), 6c's
/// catalogue for a GenBank record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CdsMode {
    Whole,
    Bases,
    Catalogue,
}

/// One mutation line's preview (plan §2 rule 1): the line's 1-based number, and
/// what the line says; `None` for a blank line, which has nothing to read.
#[derive(Clone, Debug, PartialEq)]
pub struct Preview {
    pub line: usize,
    pub reading: Option<Result<Reading, Refusal>>,
}

/// The file the loop read: the path the form typed, the file's name (no
/// directory), its text, and whether it is a GenBank record.
#[derive(Clone, Debug, PartialEq)]
struct Loaded {
    path: String,
    name: String,
    text: String,
    genbank: bool,
}

/// One text field: its lines, the line the cursor is on, and the column a typed
/// character goes before.
#[derive(Clone, Debug, PartialEq)]
struct Text {
    lines: Vec<String>,
    line: usize,
    col: usize,
}

impl Text {
    /// One empty field: a single empty line, the cursor at its start.
    fn empty() -> Text {
        Text {
            lines: vec![String::new()],
            line: 0,
            col: 0,
        }
    }

    /// The field's text: its lines joined with newlines, the text a paste
    /// typed.
    fn text(&self) -> String {
        self.lines.join("\n")
    }

    /// The field, replaced by `value` (a name the loop set from a file's stem):
    /// the cursor goes to the end of the last line.
    fn set(&mut self, value: String) {
        self.lines = value.split('\n').map(|row| row.to_string()).collect();
        self.line = self.lines.len() - 1;
        self.col = self.lines[self.line].chars().count();
    }

    /// One character inserted at the cursor.
    fn insert(&mut self, ch: char) {
        let mut chars: Vec<char> = self.lines[self.line].chars().collect();
        self.col = self.col.min(chars.len());
        chars.insert(self.col, ch);
        self.lines[self.line] = chars.into_iter().collect();
        self.col += 1;
    }

    /// `Backspace`: the character before the cursor goes, and at the start of a
    /// line nothing does, so a line the user typed is not swallowed.
    fn backspace(&mut self) {
        if self.col == 0 {
            return;
        }
        let mut chars: Vec<char> = self.lines[self.line].chars().collect();
        chars.remove(self.col - 1);
        self.lines[self.line] = chars.into_iter().collect();
        self.col -= 1;
    }

    /// `Delete`: the character at the cursor goes.
    fn delete(&mut self) {
        let mut chars: Vec<char> = self.lines[self.line].chars().collect();
        if self.col < chars.len() {
            chars.remove(self.col);
            self.lines[self.line] = chars.into_iter().collect();
        }
    }

    /// `Left`: the cursor moves back within the line.
    fn left(&mut self) {
        self.col = self.col.saturating_sub(1);
    }

    /// `Right`: the cursor moves forward within the line.
    fn right(&mut self) {
        let length: usize = self.lines[self.line].chars().count();
        self.col = (self.col + 1).min(length);
    }

    /// `Enter` in a multi-line field: the line splits at the cursor, so the
    /// text after it starts a new line, and the cursor goes to the new line's
    /// start. At the end of a line this starts an empty one.
    fn enter(&mut self) {
        let chars: Vec<char> = self.lines[self.line].chars().collect();
        let tail: String = chars[self.col..].iter().collect();
        self.lines[self.line] = chars[..self.col].iter().collect();
        self.lines.insert(self.line + 1, tail);
        self.line += 1;
        self.col = 0;
    }

    /// `Up`: the cursor moves to the previous line, keeping its column where
    /// that line has room for it.
    fn up(&mut self) {
        self.set_line(self.line.saturating_sub(1));
    }

    /// `PageUp`: the cursor moves `steps` lines up, clamped to the first line,
    /// the column clamped to that line's length (M7-51).
    fn up_by(&mut self, steps: usize) {
        self.set_line(self.line.saturating_sub(steps));
    }

    /// `Down`: the cursor moves to the next line, the last line being its own.
    fn down(&mut self) {
        let last: usize = self.lines.len() - 1;
        self.set_line((self.line + 1).min(last));
    }

    /// `PageDown`: the cursor moves `steps` lines down, clamped to the last line
    /// (M7-51).
    fn down_by(&mut self, steps: usize) {
        let last: usize = self.lines.len() - 1;
        self.set_line((self.line + steps).min(last));
    }

    /// Moves the cursor to line `to`, keeping its column where the line has room.
    fn set_line(&mut self, to: usize) {
        self.line = to;
        let length: usize = self.lines[to].chars().count();
        self.col = self.col.min(length);
    }
}

/// The form's state: what the user has typed and chosen, where focus is, the
/// file the loop read, the previews of the mutation lines, and the last refusal.
pub struct NewForm {
    source: Source,
    file: Text,
    paste: Text,
    name: Text,
    shape: TopologySpec,
    strand: StrandSpec,
    cds_mode: CdsMode,
    /// The index in `cds_list`, the usable entries of 6c's catalogue.
    cds_index: usize,
    cds_list: Vec<CatalogueEntry>,
    first: Text,
    last: Text,
    mutations: Text,
    overlap: Text,
    save_as: Text,
    /// Whether the user has typed in `save_as`, after which the form fills
    /// nothing there (M7-37: a name the user wrote is never replaced).
    save_touched: bool,
    /// The pop-up over the form (M7-49).
    popup: Popup,
    /// The action `[ Save and run ]` gave, kept so the `Replace` pop-up's `y`
    /// gives the same one, with `replace` true (M7-49, M7-62).
    pending: Option<FormAction>,
    focus: Field,
    backend: bool,
    loaded: Option<Loaded>,
    locus: Option<String>,
    /// The directory `listing` was read from, relative to the loop's directory
    /// (7m5), so the loop lists a directory only when `file_dir()` changes.
    listing_dir: Option<String>,
    /// `listing_dir`'s folders and template files, as `picker::list` gave them.
    listing: Vec<Entry>,
    /// The highlighted choice's position in `choices()`, `None` until `Up` or
    /// `Down` picks one (M7-50).
    chosen: Option<usize>,
    /// The `--design` profile's placement, which the form shows as `inline`
    /// (M7-39), and whose value fills `Overlap` (plan §2 rule 2).
    profile_placement: Option<PlacementSpec>,
    blocker: Option<(Field, Option<usize>, String)>,
}

impl NewForm {
    /// The form opens on `Source` with nothing typed. `Overlap` opens at 15 nt
    /// (M7-45), which a `--design` profile's `min_annealing_overlap` replaces; the
    /// profile's placement is the one shown as `inline` (M7-39). `backend` says
    /// whether `--backend` was given, which is whether words can be read at all
    /// (M7-47).
    pub fn new(profile: Option<&ProfileDocument>, backend: bool) -> NewForm {
        let mut form = NewForm {
            source: Source::File,
            file: Text::empty(),
            paste: Text::empty(),
            name: Text::empty(),
            shape: TopologySpec::Linear,
            strand: StrandSpec::Plus,
            cds_mode: CdsMode::Whole,
            cds_index: 0,
            cds_list: Vec::new(),
            first: Text::empty(),
            last: Text::empty(),
            mutations: Text::empty(),
            overlap: Text::empty(),
            save_as: Text::empty(),
            save_touched: false,
            popup: Popup::Closed,
            pending: None,
            focus: Field::Source,
            backend,
            loaded: None,
            locus: None,
            listing_dir: None,
            listing: Vec::new(),
            chosen: None,
            profile_placement: None,
            blocker: None,
        };
        // M7-45: the prefilled overlap, so the value is shown and editable.
        form.overlap.set("15".to_string());
        if let Some(chosen) = profile {
            if let Some(design) = &chosen.design {
                form.overlap.set(design.min_annealing_overlap.to_string());
                form.profile_placement = Some(design.placement.clone());
            }
        }
        form
    }

    /// One key. While a pop-up is open every key goes to it (M7-48), `ctrl-c`
    /// quitting first. `Esc` quits from any field (`esc quit` on the form's
    /// legend), and so does `ctrl-c`, which types nothing (M7-43);
    /// `PageDown`/`PageUp` move 10 lines in the paste and mutation boxes, and to
    /// the ends elsewhere (M7-51); the rest move focus, edit text, toggle a
    /// choice, and `Enter` on `[ Save and run ]` builds and opens the pop-up.
    pub fn handle(&mut self, key: Key) -> FormAction {
        // M7-43: `ctrl-c` quits from any field, typing nothing, as `Esc` does.
        if key == Key::CtrlC {
            return FormAction::Quit;
        }
        // M7-48: the pop-up is the form's own key rule, over every field.
        if self.popup != Popup::Closed {
            return self.popup_key(key);
        }
        if key == Key::Esc {
            return FormAction::Quit;
        }
        let from: Field = self.focus;
        // M7-51: the page keys, on every field, read one rule: the cursor in the
        // two multi-line boxes, focus at the ends everywhere else.
        if key == Key::PageUp || key == Key::PageDown {
            return self.paged(key, from);
        }
        if from == Field::Review {
            if key == Key::Enter {
                return self.save_and_run();
            }
            return self.moved(key, from);
        }
        if is_choice(from) {
            match key {
                Key::Char(' ') => {
                    self.cycle(1);
                    return FormAction::Continue;
                }
                Key::Right => {
                    self.cycle(1);
                    return FormAction::Continue;
                }
                Key::Left => {
                    self.cycle(-1);
                    return FormAction::Continue;
                }
                _ => {}
            }
            return self.moved(key, from);
        }
        // A text field.
        let multi: bool = is_multi(from);
        match key {
            Key::Char(ch) => {
                self.insert_char(ch);
                return FormAction::Continue;
            }
            Key::Backspace | Key::Delete => {
                if let Some(field) = self.text_mut(from) {
                    if key == Key::Backspace {
                        field.backspace();
                    } else {
                        field.delete();
                    }
                }
                self.after_edit(from);
                return FormAction::Continue;
            }
            Key::Left => {
                if let Some(field) = self.text_mut(from) {
                    field.left();
                }
                return FormAction::Continue;
            }
            Key::Right => {
                if let Some(field) = self.text_mut(from) {
                    field.right();
                }
                return FormAction::Continue;
            }
            Key::Enter => {
                if multi {
                    if let Some(field) = self.text_mut(from) {
                        field.enter();
                    }
                    return FormAction::Continue;
                }
                // M7-50: `enter` with a highlighted choice picks it; with none,
                // `Enter` moves focus on, as it does today.
                if from == Field::File {
                    if let Some(action) = self.pick_enter() {
                        return action;
                    }
                }
                return self.moved(key, from);
            }
            Key::Up | Key::Down => {
                if multi {
                    if let Some(field) = self.text_mut(from) {
                        if key == Key::Up {
                            field.up();
                        } else {
                            field.down();
                        }
                    }
                    return FormAction::Continue;
                }
                // M7-50: while `File` has choices, `Up`/`Down` highlight one; with
                // none they move focus, as they do today.
                if from == Field::File && !self.choices().is_empty() {
                    self.pick(key);
                    return FormAction::Continue;
                }
                return self.moved(key, from);
            }
            Key::Tab | Key::BackTab => return self.moved(key, from),
            _ => {}
        }
        FormAction::Continue
    }

    /// Bracketed paste: the text goes into the focused text field as typed
    /// characters would, newlines kept in multi-line fields and dropped in
    /// single-line ones. Many terminals send a paste's line breaks as `\r`, so a
    /// lone `\r` is one line break, and `\r\n` is one break, read exactly as `\n`
    /// is (M7-42): a FASTA pasted from such a terminal keeps its lines.
    pub fn paste(&mut self, text: &str) {
        let multi: bool = is_multi(self.focus);
        let mut chars: std::iter::Peekable<std::str::Chars> = text.chars().peekable();
        while let Some(ch) = chars.next() {
            let brake: bool = if ch == '\r' {
                // `\r\n` is one break, so the `\n` of the pair goes with it.
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                true
            } else {
                ch == '\n'
            };
            if brake {
                if multi {
                    if let Some(field) = self.text_mut(self.focus) {
                        field.enter();
                    }
                }
                continue;
            }
            self.insert_char(ch);
        }
    }

    /// The loop's answer to `Load`: the file's name (no directory) and its
    /// text, or the operating system's reason it could not be read. Text
    /// opening with `LOCUS` is GenBank: its CDS list is the catalogue's usable
    /// entries and the request's `id` is the LOCUS name. Text opening with `>`
    /// is FASTA, whose `Name` is the file's stem when the field is empty.
    /// Anything else, and `Err`, is a blocker on `File` naming the reason.
    pub fn loaded(&mut self, file_name: &str, outcome: Result<String, String>) {
        match outcome {
            Err(reason) => {
                self.loaded = None;
                self.refuse(
                    Field::File,
                    None,
                    format!("{} cannot be read: {}", file_name, reason),
                );
            }
            Ok(text) => {
                let opened: &str = text.trim_start();
                if opened.starts_with("LOCUS") {
                    self.load_genbank(file_name, text);
                } else if opened.starts_with('>') {
                    self.loaded = Some(Loaded {
                        path: file_name.to_string(),
                        name: file_name.to_string(),
                        text,
                        genbank: false,
                    });
                    self.locus = None;
                    if self.name.text().is_empty() {
                        let stem: String = stem(file_name);
                        self.name.set(stem);
                    }
                    self.sync_mode();
                    self.sync_save_as();
                    self.blocker = None;
                } else {
                    self.loaded = None;
                    self.refuse(
                        Field::File,
                        None,
                        format!(
                            "{} is neither FASTA nor GenBank; write a FASTA file, a GenBank record, or paste the sequence",
                            file_name
                        ),
                    );
                }
            }
        }
    }

    /// The field focus is on.
    pub fn focus(&self) -> Field {
        self.focus
    }

    /// The line the cursor is on inside the focused field, 0-based; 0 for a
    /// field with one line.
    pub fn focus_line(&self) -> usize {
        match self.text_of(self.focus) {
            Some(field) => field.line,
            None => 0,
        }
    }

    /// The cursor's column in the focused field, counted in characters.
    pub fn cursor(&self) -> usize {
        match self.text_of(self.focus) {
            Some(field) => field.col,
            None => 0,
        }
    }

    /// The fields focus moves through, in order, with the ones that do not
    /// apply left out (7k4 draws these rows).
    pub fn visible(&self) -> Vec<Field> {
        let mut fields: Vec<Field> = Vec::new();
        for field in ORDER.iter() {
            if self.field_visible(*field) {
                fields.push(*field);
            }
        }
        fields
    }

    /// Whether one field applies, so focus reaches it and the screen shows it.
    pub fn is_visible(&self, field: Field) -> bool {
        self.field_visible(field)
    }

    /// The template's source, as the `Source` field holds it.
    pub fn source(&self) -> Source {
        self.source
    }

    /// The path typed in `File`.
    pub fn file(&self) -> String {
        self.file.text()
    }

    /// The typed `File` text up to and including its last `/`, which is the
    /// directory the loop lists, relative to `cwd`; `""` when it has none
    /// (contract §2.2).
    pub fn file_dir(&self) -> String {
        let text: String = self.file.text();
        match text.rfind('/') {
            Some(at) => text[..=at].to_string(),
            None => String::new(),
        }
    }

    /// The listing of `dir`, which the loop gives once per directory. The
    /// highlight is cleared with it, so a new directory starts unpicked.
    pub fn set_listing(&mut self, dir: String, entries: Vec<Entry>) {
        self.listing_dir = Some(dir);
        self.listing = entries;
        self.chosen = None;
    }

    /// The directory the stored listing was read from, `None` when the loop has
    /// listed nothing, so the loop lists a directory only when `file_dir()`
    /// changes.
    pub fn listing_dir(&self) -> Option<String> {
        self.listing_dir.clone()
    }

    /// The stored listing's entries whose name starts with the text typed after
    /// `file_dir()`, compared case-insensitively, in listing order. Empty when
    /// focus is not `File`, or when the stored listing is for another directory
    /// (contract §2.2).
    pub fn choices(&self) -> Vec<Entry> {
        if self.focus != Field::File {
            return Vec::new();
        }
        let dir: String = self.file_dir();
        if self.listing_dir.as_deref() != Some(dir.as_str()) {
            return Vec::new();
        }
        let typed: String = self.file.text()[dir.len()..].to_lowercase();
        self.listing
            .iter()
            .filter(|entry| entry.name.to_lowercase().starts_with(&typed))
            .cloned()
            .collect()
    }

    /// The highlighted choice, `None` until `Up` or `Down` picks one (M7-50).
    pub fn chosen(&self) -> Option<usize> {
        self.chosen
    }

    /// The pasted text.
    pub fn paste_text(&self) -> String {
        self.paste.text()
    }

    /// The FASTA file a paste is saved as: `<name>.fa`, shown read-only beside
    /// the paste (M7-37).
    pub fn paste_file(&self) -> String {
        format!("{}.fa", self.name.text())
    }

    /// The template's name.
    pub fn name(&self) -> String {
        self.name.text()
    }

    /// The template's shape.
    pub fn shape(&self) -> TopologySpec {
        self.shape.clone()
    }

    /// The CDS's strand.
    pub fn strand(&self) -> StrandSpec {
        self.strand.clone()
    }

    /// How the CDS is stated.
    pub fn cds_mode(&self) -> CdsMode {
        self.cds_mode
    }

    /// The CDS the form states for a FASTA or pasted template, `None` when it
    /// is picked from the catalogue.
    pub fn cds_choice(&self) -> Option<CdsChoice> {
        if self.cds_mode == CdsMode::Catalogue {
            return None;
        }
        if self.cds_mode == CdsMode::Bases {
            Some(CdsChoice::Bases {
                first: self.first.text().parse::<i64>().unwrap_or(i64::MAX),
                last: self.last.text().parse::<i64>().unwrap_or(i64::MAX),
                strand: self.strand.clone(),
            })
        } else {
            Some(CdsChoice::Whole(self.strand.clone()))
        }
    }

    /// A GenBank record's CDS list: 6c's catalogue, usable entries only, since
    /// an unusable CDS cannot be picked (plan rule 4).
    pub fn cds_list(&self) -> Vec<CatalogueEntry> {
        self.cds_list.clone()
    }

    /// The chosen entry of `cds_list`.
    pub fn cds_index(&self) -> usize {
        self.cds_index
    }

    /// The typed CDS start, as written.
    pub fn first(&self) -> String {
        self.first.text()
    }

    /// The typed CDS end, as written.
    pub fn last(&self) -> String {
        self.last.text()
    }

    /// The mutation lines as typed, in order.
    pub fn mutations(&self) -> Vec<String> {
        self.mutations.lines.clone()
    }

    /// The free text in `Mutations` read in words, which is what `Interpret`
    /// sends to the model (M7-47): true when some non-blank mutation line is one
    /// luria cannot read at all, refusal `unreadable`. A line luria reads but
    /// refuses does not make the box words.
    pub fn reads_as_words(&self) -> bool {
        self.mutations.lines.iter().any(|line| {
            !line.trim().is_empty() && matches!(parse_line(line), Err(Refusal::Unreadable))
        })
    }

    /// Whether `--backend` was given, which is whether words can be read at all
    /// (M7-47).
    pub fn backend(&self) -> bool {
        self.backend
    }

    /// The design the words are read with (M7-41, M7-42): the overlap as typed,
    /// with placement `inline`, the only one `luria new` supports (D029), and
    /// `q5 new`'s own policy, style and buffer; the codon `default` (M7-46, since
    /// the form has no codon field, so a line that states one carries it in the
    /// text); and residues counted from the CDS's first codon, as the shorthand
    /// path counts them (docs/specs/cli.md §12.2). `[ Review ]` reaches `Interpret` only
    /// after `form_blocker`, so `Overlap` is written, holds digits only, and is one
    /// `i64` can read (M7-44).
    pub fn words_profile(&self) -> ProfileDocument {
        ProfileDocument {
            design: Some(DesignSpec {
                policy: PolicySpec::LuriaNative,
                style: StyleSpec::Q5Polymerase,
                min_annealing_overlap: self
                    .overlap
                    .text()
                    .parse::<i64>()
                    .expect("the overlap is digits"),
                placement: PlacementSpec::Inline,
                buffer: BufferSpec::Q5Reaction,
            }),
            codon: Some(CodonSpec::Default),
            numbering: Some(NumberingSpec::FromCdsStart),
        }
    }

    /// The overlap as typed, digits only.
    pub fn overlap(&self) -> String {
        self.overlap.text()
    }

    /// The request file's name.
    pub fn save_as(&self) -> String {
        self.save_as.text()
    }

    /// Which pop-up is open over the form (M7-49).
    pub fn popup(&self) -> Popup {
        self.popup.clone()
    }

    /// The files luria would write that already exist, named in the order the
    /// pop-up shows them (M7-49): it asks, `y` replaces, any other key writes
    /// nothing.
    pub fn ask_replace(&mut self, names: Vec<String>) {
        self.popup = Popup::Replace(names);
    }

    /// A write that refused: the pop-up closes, and the message becomes the
    /// blocker on `SaveAs`, whose name is what the user changes (M7-63).
    pub fn save_failed(&mut self, message: String) {
        self.popup = Popup::Closed;
        self.refuse(Field::SaveAs, None, message);
    }

    /// A GenBank record's LOCUS name, which is the request's `id` (M7-32).
    pub fn locus(&self) -> Option<String> {
        self.locus.clone()
    }

    /// The file the loop read, its name alone, `None` before one has been.
    pub fn template_file(&self) -> Option<String> {
        self.loaded.as_ref().map(|found| found.name.clone())
    }

    /// Whether the words in `Mutations` ask for a GenBank record the form has not
    /// loaded (M7-47): the box reads as words, a backend was given, and the file
    /// read is not a GenBank record, whose CDS list is the model is shown. `text`
    /// draws the row this says.
    pub fn words_need_genbank(&self) -> bool {
        self.backend
            && self.reads_as_words()
            && !matches!(&self.loaded, Some(found) if found.genbank)
    }

    /// What each mutation line means, in order: recomputed here from the lines
    /// as they now are, so every edit, `Backspace` included, changes what the
    /// screen shows (plan §2 rule 1).
    pub fn previews(&self) -> Vec<Preview> {
        let mut previews: Vec<Preview> = Vec::new();
        for (index, line) in self.mutations.lines.iter().enumerate() {
            let reading: Option<Result<Reading, Refusal>> = if line.trim().is_empty() {
                None
            } else {
                Some(parse_line(line))
            };
            previews.push(Preview {
                line: index + 1,
                reading,
            });
        }
        previews
    }

    /// The last refusal: the field it names, the mutation line it is about when
    /// it is about one, and the message. Focus is on that field.
    pub fn blocker(&self) -> Option<(Field, Option<usize>, String)> {
        self.blocker.clone()
    }

    /// `[ Save and run ]`: `build_action`, returned at once, so the first `Enter`
    /// is the one that writes and runs (M7-62, M7-63). The action it gave is
    /// kept, so the `Replace` pop-up's `y` gives the same one with `replace` true
    /// (M7-49).
    fn save_and_run(&mut self) -> FormAction {
        let action: FormAction = self.build_action();
        match &action {
            FormAction::Save { .. } => self.pending = Some(action.clone()),
            FormAction::Interpret { .. } => {
                self.pending = Some(FormAction::SaveWords { replace: false });
            }
            _ => {}
        }
        action
    }

    /// One key with a pop-up open (M7-49, M7-62). `y` replaces the names listed,
    /// giving the action `[ Save and run ]` kept, with `replace` true; any other
    /// key closes the pop-up and sets the blocker saying nothing was written.
    /// `ctrl-c` quits first, as `handle` puts it before the pop-up's keys.
    fn popup_key(&mut self, key: Key) -> FormAction {
        match self.popup.clone() {
            Popup::Closed => FormAction::Continue,
            Popup::Replace(_names) => {
                // M7-49: `y` is the only key that replaces, and it gives the
                // action `[ Save and run ]` gave, with `replace` true.
                if key == Key::Char('y') {
                    return match self.pending.take() {
                        Some(FormAction::Save { built, .. }) => FormAction::Save {
                            built,
                            replace: true,
                        },
                        Some(FormAction::SaveWords { .. }) => {
                            FormAction::SaveWords { replace: true }
                        }
                        _ => FormAction::Continue,
                    };
                }
                self.popup = Popup::Closed;
                self.refuse(Field::SaveAs, None, "nothing was written".to_string());
                FormAction::Continue
            }
        }
    }

    /// `[ Save and run ]`'s build: the form's own blockers first, in order (plan
    /// §2 rule 2), then the `SaveAs` name, checked before any words are sent, so
    /// a bad name never calls the model (M7-63), then a mutation line luria reads
    /// but refuses, which is a red
    /// error naming its line and is never sent to the model (M7-47), then the box
    /// read in words (M7-47), whose own blockers are `--backend` and a GenBank
    /// template, then `new::build`, whose refusal becomes a blocker on the field
    /// it concerns. Success gives `Save` with the built request, and clears the
    /// blocker. With words the form builds nothing: the whole box goes to the
    /// model, which needs a GenBank record whose CDS list it is shown, and
    /// `Interpret` names the template as it was typed in `File` (contract §2.2).
    fn build_action(&mut self) -> FormAction {
        if let Some((field, line, message)) = self.form_blocker() {
            self.refuse(field, line, message);
            return FormAction::Continue;
        }
        // M7-63: the `Save as` name is checked here, before words are sent.
        if self.save_as.text().is_empty() {
            self.refuse(
                Field::SaveAs,
                None,
                "write the request's file name".to_string(),
            );
            return FormAction::Continue;
        }
        if self.save_as.text().contains('/') || self.save_as.text().contains('\\') {
            self.refuse(
                Field::SaveAs,
                None,
                "write a file name, not a path".to_string(),
            );
            return FormAction::Continue;
        }
        // §2.2 step 1: a line luria reads and refuses, a stop or wrong bases
        // included, stays a red error, so it is refused before the box is read as
        // words, and the whole box is never sent to the model for it.
        for (index, one) in self.mutations.lines.iter().enumerate() {
            if one.trim().is_empty() {
                continue;
            }
            if let Err(refusal) = parse_line(one) {
                if refusal != Refusal::Unreadable {
                    self.refuse(
                        Field::Mutations,
                        Some(index + 1),
                        refusal.message().to_string(),
                    );
                    return FormAction::Continue;
                }
            }
        }
        if self.reads_as_words() {
            // §2.2 step 2: without a backend the box cannot be read, so the
            // blocker names `--backend` on the first line that does not read.
            if !self.backend {
                let number: usize = self
                    .mutations
                    .lines
                    .iter()
                    .enumerate()
                    .find(|(_, one)| {
                        !one.trim().is_empty()
                            && matches!(parse_line(one), Err(Refusal::Unreadable))
                    })
                    .map(|(index, _)| index + 1)
                    .expect("a line reads as words");
                self.refuse(
                    Field::Mutations,
                    Some(number),
                    format!(
                        "line {} is not shorthand; start luria new with --backend to read the mutations in words",
                        number
                    ),
                );
                return FormAction::Continue;
            }
            // The model is shown the record's CDS list, so it needs a record.
            if !matches!(&self.loaded, Some(found) if found.genbank) {
                self.refuse(
                    Field::Mutations,
                    None,
                    "words need a GenBank template, whose CDS list the model is shown".to_string(),
                );
                return FormAction::Continue;
            }
            return FormAction::Interpret {
                template: self.file.text(),
                text: self.mutations.lines.join("\n"),
                profile: self.words_profile(),
                // M7-59: the CDS picked on the form, which words that name no
                // CDS use. A GenBank record under `File` is the catalogue mode,
                // and `form_blocker` above stopped the form when the catalogue
                // has no usable CDS, so there is one to name.
                cds: self.cds_list[self.cds_index].id.clone(),
            };
        }
        match build(&self.build_form()) {
            Ok(built) => {
                self.blocker = None;
                FormAction::Save {
                    built: Box::new(built),
                    replace: false,
                }
            }
            Err((refusal, line)) => {
                let field: Field = self.field_of(refusal, line);
                self.refuse(field, line, refusal.message().to_string());
                FormAction::Continue
            }
        }
    }

    /// The form's blockers, in contract §3.1's order: a template that is not
    /// there, a name that is, a CDS whose bases are, the overlap, and a
    /// `--design` profile whose placement is not `inline` (M7-39). The request's
    /// file name is not a blocker here: the `SaveAs` blockers in `build_action`
    /// run after these (M7-63).
    /// A GenBank record whose catalogue has no usable CDS is a template the
    /// form cannot build, since one cannot be picked (plan rule 4).
    fn form_blocker(&self) -> Option<(Field, Option<usize>, String)> {
        if self.source == Source::File && self.loaded.is_none() {
            return Some((
                Field::File,
                None,
                "no template has been read; type a file's name, or choose paste".to_string(),
            ));
        }
        if self.source == Source::Paste && self.paste.text().trim().is_empty() {
            return Some((
                Field::Paste,
                None,
                "no sequence has been pasted; paste one, or write its bases".to_string(),
            ));
        }
        if self.cds_mode == CdsMode::Catalogue && self.cds_list.is_empty() {
            return Some((
                Field::Cds,
                None,
                "the record has no CDS luria can use; pick a record whose CDS imports".to_string(),
            ));
        }
        if self.cds_mode != CdsMode::Catalogue && self.name.text().is_empty() {
            return Some((
                Field::Name,
                None,
                "the template has no name; write the name to give it".to_string(),
            ));
        }
        if self.cds_mode == CdsMode::Bases && self.first.text().is_empty() {
            return Some((
                Field::First,
                None,
                "the CDS start is empty; write its first base's position, counting from 1"
                    .to_string(),
            ));
        }
        if self.cds_mode == CdsMode::Bases && self.last.text().is_empty() {
            return Some((
                Field::Last,
                None,
                "the CDS end is empty; write its last base's position, counting from 1".to_string(),
            ));
        }
        if self.overlap.text().is_empty() {
            return Some((
                Field::Overlap,
                None,
                "the minimum annealing overlap is empty; write a length in nucleotides".to_string(),
            ));
        }
        // M7-44: `Overlap` takes digits, any number of them, so a length past
        // `i64` is too large to read. This comes first, so every value read
        // below fits.
        if self.overlap.text().parse::<i64>().is_err() {
            return Some((
                Field::Overlap,
                None,
                "the minimum annealing overlap is too large; write a length in nucleotides"
                    .to_string(),
            ));
        }
        if let Some(placement) = &self.profile_placement {
            if *placement != PlacementSpec::Inline {
                return Some((
                    Field::Overlap,
                    None,
                    "`q5 design` supports inline placement only, so the profile's placement cannot be used; run without the profile, or write one whose placement is inline".to_string(),
                ));
            }
        }
        None
    }

    /// The filled fields as `new::build`'s input (contract 7k2): the blockers
    /// above guarantee the name and the numbers are written.
    fn build_form(&self) -> crate::new::Form {
        let template: TemplateInput = if self.cds_mode == CdsMode::Catalogue {
            let read: &Loaded = self
                .loaded
                .as_ref()
                .expect("a GenBank template is loaded before its CDS is picked");
            TemplateInput::GenBank {
                file_name: read.name.clone(),
                text: read.text.clone(),
                cds: self.cds_list[self.cds_index].id.clone(),
            }
        } else if self.source == Source::File {
            let read: &Loaded = self
                .loaded
                .as_ref()
                .expect("a file template is loaded before the form builds");
            TemplateInput::Fasta {
                file_name: read.name.clone(),
                text: read.text.clone(),
                name: self.name.text(),
                topology: self.shape.clone(),
                cds: self
                    .cds_choice()
                    .expect("a FASTA template states its own CDS"),
            }
        } else {
            let name: String = self.name.text();
            TemplateInput::Paste {
                text: self.paste.text(),
                name: name.clone(),
                save_as: self.paste_file(),
                topology: self.shape.clone(),
                cds: self.cds_choice().expect("a paste states its own CDS"),
            }
        };
        crate::new::Form {
            template,
            lines: self.mutations.lines.clone(),
            codon: CodonChoice::DefaultPolicy,
            overlap: self.overlap.text().parse::<u64>().ok(),
        }
    }

    /// The field `new::build`'s refusal concerns: `Overlap` for the missing
    /// overlap, `File`/`Paste` for a template core cannot read, `First` for a
    /// CDS outside the template, which is the form's bases and not a line, and
    /// `Mutations` for every refusal about a line.
    fn field_of(&self, refusal: BuildRefusal, line: Option<usize>) -> Field {
        match refusal {
            BuildRefusal::MissingOverlap => Field::Overlap,
            BuildRefusal::TemplateUnreadable => {
                if self.source == Source::Paste {
                    Field::Paste
                } else {
                    Field::File
                }
            }
            BuildRefusal::OutsideTemplate if line.is_none() => Field::First,
            _ => Field::Mutations,
        }
    }

    /// A blocker: the field, the mutation line it is about when it is about
    /// one, and the message. Focus moves to the field, so the reason is shown
    /// where the value is missing (plan §2 rule 2).
    fn refuse(&mut self, field: Field, line: Option<usize>, message: String) {
        self.focus = field;
        self.blocker = Some((field, line, message));
    }

    /// A GenBank record read by core once: its LOCUS name is the request's
    /// `id` (M7-32), and its CDS list is 6c's catalogue, usable entries only.
    /// `new::build` reads the record again under the LOCUS name, as it states.
    fn load_genbank(&mut self, file_name: &str, text: String) {
        let placeholder: Identifier = match Identifier::create("luria-new".to_string()) {
            Ok(found) => found,
            Err(_) => {
                self.refuse(
                    Field::File,
                    None,
                    "luria-new is not a name luria can read".to_string(),
                );
                return;
            }
        };
        let record = match read_genbank(placeholder, text.clone()) {
            Ok(found) => found,
            Err(_) => {
                self.refuse(
                    Field::File,
                    None,
                    format!(
                        "{} is not a GenBank record core can read; check its LOCUS line",
                        file_name
                    ),
                );
                return;
            }
        };
        let entries: Vec<CatalogueEntry> = match catalogue(&record) {
            Ok(found) => found,
            Err(_) => {
                self.refuse(
                    Field::File,
                    None,
                    format!(
                        "{} is not a GenBank record core can read; check its LOCUS line",
                        file_name
                    ),
                );
                return;
            }
        };
        let mut usable: Vec<CatalogueEntry> = Vec::new();
        for entry in entries.into_iter() {
            if matches!(entry.usability, Usability::Usable { .. }) {
                usable.push(entry);
            }
        }
        self.locus = Some(record.locus_name());
        self.loaded = Some(Loaded {
            path: file_name.to_string(),
            name: file_name.to_string(),
            text,
            genbank: true,
        });
        self.cds_list = usable;
        self.cds_index = 0;
        self.sync_mode();
        self.sync_save_as();
        self.blocker = None;
    }

    /// The CDS's mode, as the source and the file read say: a GenBank record's
    /// CDS is the catalogue, and a paste's is the form's own bases, since a
    /// paste has no record to pick from.
    fn sync_mode(&mut self) {
        let genbank: bool = match &self.loaded {
            Some(found) => found.genbank,
            None => false,
        };
        if self.source == Source::File && genbank {
            self.cds_mode = CdsMode::Catalogue;
            if self.cds_index >= self.cds_list.len() {
                self.cds_index = 0;
            }
        } else if self.cds_mode == CdsMode::Catalogue {
            self.cds_mode = CdsMode::Whole;
            self.cds_index = 0;
        }
        self.settle_focus();
    }

    /// Focus never rests on a field that does not apply: when a change hides
    /// the focused field, focus moves to the next visible one, wrapping, as
    /// `Tab` would (§3.1).
    fn settle_focus(&mut self) {
        if self.field_visible(self.focus) {
            return;
        }
        let mut start: usize = 0;
        for (index, field) in ORDER.iter().enumerate() {
            if *field == self.focus {
                start = index;
            }
        }
        for step in 1..ORDER.len() {
            let next: Field = ORDER[(start + step) % ORDER.len()];
            if self.field_visible(next) {
                self.focus = next;
                return;
            }
        }
    }

    /// The pop-up's name field follows the name, which for a GenBank record is
    /// the LOCUS name, and never replaces what the user typed in the pop-up
    /// (M7-37, M7-48).
    fn sync_save_as(&mut self) {
        if self.save_touched {
            return;
        }
        let named: String = match &self.locus {
            Some(found) => found.clone(),
            None => self.name.text(),
        };
        if named.is_empty() {
            self.save_as.set(String::new());
            return;
        }
        self.save_as.set(format!("{}.request.json", named));
    }

    /// `Up`/`Down` on `File` with the list showing: `Down` highlights the next
    /// choice, from `None` the first, `Up` the previous, from the first back to
    /// `None`, with no wrapping, so `Down` at the last choice keeps it (contract
    /// §2.2).
    fn pick(&mut self, key: Key) {
        let count: usize = self.choices().len();
        if count == 0 {
            return;
        }
        match key {
            Key::Down => {
                self.chosen = Some(self.chosen.map_or(0, |at| (at + 1).min(count - 1)));
            }
            Key::Up => {
                self.chosen = match self.chosen {
                    Some(0) | None => None,
                    Some(at) => Some(at - 1),
                };
            }
            _ => {}
        }
    }

    /// `Enter` on `File` with a highlighted choice: a folder is typed into the
    /// path, focus stays on `File`, the highlight is cleared, and nothing is
    /// loaded; a file completes the path and is then left exactly as `Tab` leaves
    /// it, so the loop is asked to read it (contract §2.2). `None` when nothing
    /// is highlighted, which is `Enter` as it is today.
    fn pick_enter(&mut self) -> Option<FormAction> {
        let at: usize = self.chosen?;
        let choices: Vec<Entry> = self.choices();
        if at >= choices.len() {
            return None;
        }
        let entry: Entry = choices[at].clone();
        let mut path: String = self.file_dir();
        path.push_str(&entry.name);
        if entry.folder {
            path.push('/');
            self.file.set(path);
            self.chosen = None;
            return Some(FormAction::Continue);
        }
        self.file.set(path);
        self.chosen = None;
        Some(self.moved(Key::Tab, Field::File))
    }

    /// A typed character: digits only in `Overlap`, `First` and `Last`, then the
    /// rule `Name` keeps.
    fn insert_char(&mut self, ch: char) {
        let field: Field = self.focus;
        if matches!(field, Field::Overlap | Field::First | Field::Last) && !ch.is_ascii_digit() {
            return;
        }
        if let Some(text) = self.text_mut(field) {
            text.insert(ch);
        }
        self.after_edit(field);
    }

    /// What an edit of one field means for the others: a typed name carries its
    /// request name with it, and a typed `SaveAs` means the form writes nothing
    /// there again (M7-37, M7-63).
    fn after_edit(&mut self, field: Field) {
        // M7-50: any edit of `File` clears the highlighted choice, so what is
        // typed is what is picked.
        if field == Field::File {
            self.chosen = None;
        }
        if field == Field::SaveAs {
            self.save_touched = true;
        }
        if field == Field::Name {
            self.sync_save_as();
        }
    }

    /// Focus moves: `Tab` `Enter` `Down` to the next visible field, `BackTab`
    /// `Up` to the previous, wrapping. Leaving `File` asks the loop to read the
    /// path, when it is written and is not the file last loaded.
    fn moved(&mut self, key: Key, from: Field) -> FormAction {
        if !matches!(
            key,
            Key::Tab | Key::BackTab | Key::Enter | Key::Up | Key::Down
        ) {
            return FormAction::Continue;
        }
        let forward: bool = matches!(key, Key::Tab | Key::Enter | Key::Down);
        let fields: Vec<Field> = self.visible();
        let mut at: usize = 0;
        for (index, field) in fields.iter().enumerate() {
            if *field == from {
                at = index;
            }
        }
        let next: usize = if forward {
            (at + 1) % fields.len()
        } else {
            (at + fields.len() - 1) % fields.len()
        };
        self.focus = fields[next];
        self.left_file(from)
    }

    /// `PageDown`/`PageUp` (M7-51): in `Paste` and `Mutations` the cursor moves 10
    /// lines, clamped to the first and the last line, its column clamped to that
    /// line's length; anywhere else focus goes to the first visible field
    /// (`PageUp`) or to `[ Review ]` (`PageDown`). Leaving `File` this way asks the
    /// loop to read the path, as `Tab` does.
    fn paged(&mut self, key: Key, from: Field) -> FormAction {
        if is_multi(from) {
            if let Some(field) = self.text_mut(from) {
                if key == Key::PageDown {
                    field.down_by(10);
                } else {
                    field.up_by(10);
                }
            }
            return FormAction::Continue;
        }
        let fields: Vec<Field> = self.visible();
        self.focus = if key == Key::PageDown {
            Field::Review
        } else {
            fields[0]
        };
        self.left_file(from)
    }

    /// The path typed in `File`, when leaving it asks the loop to read it:
    /// non-empty, and not the file the loop has already read.
    fn left_file(&mut self, from: Field) -> FormAction {
        if from != Field::File {
            return FormAction::Continue;
        }
        let path: String = self.file.text();
        if path.is_empty() {
            return FormAction::Continue;
        }
        match &self.loaded {
            Some(found) if found.path == path => FormAction::Continue,
            _ => FormAction::Load(path),
        }
    }

    /// A choice field's `Left`/`Right`/space: `Source`, `Shape` for a FASTA and
    /// `Strand` have two values, so each key gives the other; on a GenBank
    /// record `Cds` walks the catalogue's usable entries, wrapping, and `step`
    /// says which way.
    fn cycle(&mut self, step: i64) {
        match self.focus {
            Field::Source => {
                self.source = other_source(self.source);
                self.sync_mode();
            }
            Field::Shape => self.shape = other_shape(self.shape.clone()),
            Field::Strand => self.strand = other_strand(self.strand.clone()),
            Field::Cds => {
                if self.cds_mode == CdsMode::Catalogue {
                    let count: usize = self.cds_list.len();
                    if count > 0 {
                        self.cds_index = if step > 0 {
                            (self.cds_index + 1) % count
                        } else {
                            (self.cds_index + count - 1) % count
                        };
                    }
                } else {
                    self.cds_mode = if self.cds_mode == CdsMode::Whole {
                        CdsMode::Bases
                    } else {
                        CdsMode::Whole
                    };
                }
            }
            _ => {}
        }
    }

    /// A field's text, `None` for a choice field and for `[ Review ]`.
    fn text_of(&self, field: Field) -> Option<&Text> {
        match field {
            Field::File => Some(&self.file),
            Field::Paste => Some(&self.paste),
            Field::Name => Some(&self.name),
            Field::First => Some(&self.first),
            Field::Last => Some(&self.last),
            Field::Mutations => Some(&self.mutations),
            Field::Overlap => Some(&self.overlap),
            Field::SaveAs => Some(&self.save_as),
            _ => None,
        }
    }

    /// A field's text, to be edited.
    fn text_mut(&mut self, field: Field) -> Option<&mut Text> {
        match field {
            Field::File => Some(&mut self.file),
            Field::Paste => Some(&mut self.paste),
            Field::Name => Some(&mut self.name),
            Field::First => Some(&mut self.first),
            Field::Last => Some(&mut self.last),
            Field::Mutations => Some(&mut self.mutations),
            Field::Overlap => Some(&mut self.overlap),
            Field::SaveAs => Some(&mut self.save_as),
            _ => None,
        }
    }

    /// Whether focus reaches one field: the fields that do not apply are left
    /// out (contract §3.1).
    fn field_visible(&self, field: Field) -> bool {
        match field {
            Field::Source
            | Field::Cds
            | Field::Mutations
            | Field::Overlap
            | Field::SaveAs
            | Field::Review => true,
            Field::File => self.source == Source::File,
            Field::Paste => self.source == Source::Paste,
            Field::Name | Field::Shape | Field::Strand => self.cds_mode != CdsMode::Catalogue,
            Field::First | Field::Last => self.cds_mode == CdsMode::Bases,
        }
    }
}

/// A field whose value is chosen with `Left`/`Right`/space, not typed.
fn is_choice(field: Field) -> bool {
    matches!(
        field,
        Field::Source | Field::Shape | Field::Cds | Field::Strand
    )
}

/// A field with more than one line: `Up`/`Down` move between lines, `Enter`
/// starts one, and focus leaves it only on `Tab`/`BackTab`.
fn is_multi(field: Field) -> bool {
    matches!(field, Field::Paste | Field::Mutations)
}

/// The other source.
fn other_source(source: Source) -> Source {
    if source == Source::File {
        Source::Paste
    } else {
        Source::File
    }
}

/// The other shape.
fn other_shape(shape: TopologySpec) -> TopologySpec {
    match shape {
        TopologySpec::Linear => TopologySpec::Circular,
        TopologySpec::Circular => TopologySpec::Linear,
    }
}

/// The other strand.
fn other_strand(strand: StrandSpec) -> StrandSpec {
    match strand {
        StrandSpec::Plus => StrandSpec::Minus,
        StrandSpec::Minus => StrandSpec::Plus,
    }
}

/// A file name without its extension, the name to give a FASTA template when
/// `Name` is empty (contract §3.1).
fn stem(file_name: &str) -> String {
    match file_name.rfind('.') {
        Some(at) => file_name[..at].to_string(),
        None => file_name.to_string(),
    }
}
