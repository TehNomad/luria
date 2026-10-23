//! The state of the review screen (milestone 7b, 7c3): the result document,
//! which screen is open, which member is highlighted, how far the open screen is
//! scrolled, and what each key does. No I/O
//! and no terminal types are here, so the rule is testable without a terminal
//! (plan §5.1); `terminal` maps crossterm's events to `Key` and carries out the
//! `Action`. The screens themselves are drawn by `view` (7c, 7d), which reads
//! `screen()`, `selected()`, `scroll()` and `expanded()`. Nothing scientific is
//! computed: the app reads only how many entries `results` has, whether `error`
//! is `Some`, whether `clarifications` is empty, and, for the `i` key, how many
//! bases a member's mutation writes (plan §3 rule 1).

use crate::result::MutationView;
use crate::result::ResultDocument;
use crate::review::export::ExportPlan;
use crate::review::text::MEMBER_INLINE_LIMIT;
use crate::review::MemberFrame;

/// A key, as the app sees it. `terminal` maps crossterm's events to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    Enter,
    Backspace,
    Delete,
    Tab,
    BackTab,
    Left,
    Right,
    Esc,
    /// `ctrl-c` (M7-43): `terminal` maps `c` with the control modifier to it.
    CtrlC,
    Other,
}

/// What the app asks its shell to do after a key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Continue,
    Quit,
    Export,
    /// The write asked for on the `Replace` prompt, `y` answered (M7-49):
    /// `terminal` calls the export closure to replace the names it listed.
    ExportReplace,
}

/// Which screen is open. A member index is an index into
/// `document.results`, always in range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Overview,
    /// The sequence screen (7i4, plan §5.3a): `member_sequence`'s block with the
    /// short primer lines under it, and the mutant protein between the strands.
    Member(usize),
    /// The technical lines (7c2's member text, with the provenance under it, as
    /// M7-52 puts them) behind `d` (7i4, M7-30): the details the result records
    /// for one variant.
    Details(usize),
    Problem,
    Export,
    /// The prompt over the export screen when a name it would write exists
    /// (M7-49): it names those files, and `y` replaces them.
    Replace,
}

/// The screen's state: the result document to show, the open screen, the member
/// highlighted on the overview, how far the open screen is scrolled, the
/// template's bases and every member's mutant protein as core read it. The
/// frames are set once when the screen opens, as the template's bases are.
#[derive(Clone, Debug, PartialEq)]
pub struct App {
    document: ResultDocument,
    screen: Screen,
    selected: usize,
    scroll: usize,
    template: Option<Vec<char>>,
    frames: Vec<MemberFrame>,
    export: Option<ExportPlan>,
    export_message: Option<String>,
    /// The plan's names that exist in the export directory, as the shell set
    /// them before the last key on `Export` (M7-49). `Replace` lists them.
    taken: Vec<String>,
    /// The model's reading of the words typed, as `text::proposal`'s lines, shown
    /// at the bottom of each variant's Details screen (M7-64). `luria review` never
    /// sets it, so its keys are as they were.
    reading: Option<Vec<String>>,
    expanded: bool,
    help: bool,
}

impl App {
    /// The screen opens on the overview, except when the document has an error
    /// or clarifications: a design failure, a clarification or an error is
    /// never hidden (plan §3 rule 3).
    pub fn new(document: ResultDocument) -> App {
        App {
            screen: if document.error.is_some() || !document.clarifications.is_empty() {
                Screen::Problem
            } else {
                Screen::Overview
            },
            selected: 0,
            scroll: 0,
            document,
            template: None,
            frames: Vec::new(),
            export: None,
            export_message: None,
            taken: Vec::new(),
            // `luria review` has no reading, so Details ends as it did.
            reading: None,
            // A long insertion starts collapsed every time the screen opens (7i4).
            expanded: false,
            // The help legend is closed when the screen opens (M7-27).
            help: false,
        }
    }

    /// Every member's mutant protein, in `document.results` order, set once
    /// when the screen opens: core read each one, and a member whose frame
    /// cannot be read carries the reason (7i2, plan §5.3a).
    pub fn with_frames(self, frames: Vec<MemberFrame>) -> App {
        App { frames, ..self }
    }

    /// One member's frame, or `None` when there is no such member. Nothing is
    /// read from the frame here: `view` draws what it holds (7i3).
    pub fn frame(&self, index: usize) -> Option<&MemberFrame> {
        self.frames.get(index)
    }

    /// The template's bases the sequence-context screen draws, set once when the
    /// screen opens: `None` when the template could not be read again, or its
    /// digest is not the one the document reports (7d, D005).
    pub fn with_template(self, letters: Option<Vec<char>>) -> App {
        App {
            template: letters,
            ..self
        }
    }

    /// The template's bases, or `None` when there are none to draw.
    pub fn template(&self) -> Option<&[char]> {
        self.template.as_deref()
    }

    /// The model's reading, shown at the bottom of each variant's Details, as
    /// `text::proposal` gives it, set once when the screen opens (M7-64).
    pub fn with_reading(self, lines: Vec<String>) -> App {
        App {
            reading: Some(lines),
            ..self
        }
    }

    /// The reading's lines, or `None` when there is no reading to show.
    pub fn reading(&self) -> Option<&[String]> {
        self.reading.as_deref()
    }

    /// Whether a long insertion on the sequence screen is written out (`true`) or
    /// carried as its marker (`false`). `i` on a sequence screen switches it, and
    /// every screen change starts it collapsed, so leaving to `d` and coming back
    /// shows the marker again (7i4).
    pub fn expanded(&self) -> bool {
        self.expanded
    }

    /// Whether the `?` help legend is shown in place of the open screen's lines
    /// (M7-27, M7-53): `?` opens it on every screen, and every screen
    /// change closes it.
    pub fn help(&self) -> bool {
        self.help
    }

    /// The export the screen offers: the plan `main` built from the `--export`
    /// directory the command line gave, `None` when it gave none, in which case
    /// the screen says how to enable it (7g1).
    pub fn with_export(self, plan: Option<ExportPlan>) -> App {
        App {
            export: plan,
            ..self
        }
    }

    /// The plan, or `None` when there is none, so `y` has nothing to write.
    pub fn export_plan(&self) -> Option<&ExportPlan> {
        self.export.as_ref()
    }

    /// The names that exist in the export directory, set by the shell with
    /// `taken` of the plan before each key on `Export` (M7-49): the app stays
    /// pure, so the shell looks at the file system and the app only carries what
    /// it is given, which `Replace` draws and `w` asks about.
    pub fn set_taken(&mut self, names: Vec<String>) {
        self.taken = names;
    }

    /// The names the shell said exist, in the plan's order.
    pub fn taken(&self) -> &[String] {
        self.taken.as_slice()
    }

    /// The one line the last export gave, kept on the export screen.
    pub fn export_message(&self) -> Option<&str> {
        self.export_message.as_deref()
    }

    /// The outcome of the write `y` asked for: the names written in the order
    /// written, or the reason nothing was. The message is the screen's own line
    /// and the screen stays on `Export`, so what was written is said, and the
    /// user goes back by choice (7g1).
    pub fn export_done(&mut self, outcome: Result<Vec<String>, String>) {
        self.export_message = Some(match outcome {
            Ok(names) => {
                let directory: String = self
                    .export
                    .as_ref()
                    .map(|plan| plan.directory.display().to_string())
                    .unwrap_or_default();
                format!(
                    "exported {} files to {}: {}",
                    names.len(),
                    directory,
                    names.join(", ")
                )
            }
            Err(message) => format!("export failed: {}", message),
        });
        self.screen = Screen::Export;
    }

    pub fn document(&self) -> &ResultDocument {
        &self.document
    }

    pub fn screen(&self) -> Screen {
        self.screen
    }

    /// The member highlighted on the overview; 0 when there are none.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// How far the open screen's body is scrolled. The app does not know how
    /// many lines a screen has, so `terminal` clamps it to `view::max_scroll`
    /// after every key (M7-56); `view` clamps what it draws to the same value.
    pub fn scroll(&self) -> usize {
        self.scroll
    }

    /// The scroll, clamped to `max`, the largest the open screen's body can start
    /// at (M7-56): `scroll` never goes past it, so `k` moves the body at once
    /// after scrolling past the bottom.
    pub fn clamp_scroll(&mut self, max: usize) {
        self.scroll = self.scroll.min(max);
    }

    /// `q` quits from every screen, and so does `ctrl-c`, which writes nothing
    /// (M7-43). Every other key follows the table of §3.3:
    /// `Esc` and `Backspace` go back, `Enter` opens the selected member, `j` `k`
    /// `e` `x` move, `d` opens the details of the open member, `?` opens the
    /// help legend on every screen (M7-53), `w` `o` on the export screen ask for
    /// the write when a plan exists and `esc` goes back (M7-58), and any other
    /// key changes nothing. When a name the export would write exists, as the
    /// shell set with `set_taken`, `w` `o` open the `Replace` prompt, where `y`
    /// asks for the replace and any other key goes back saying nothing was
    /// written (M7-49). While help is open, `esc` and `backspace` only close
    /// it, `q` quits and every other key does nothing (M7-27). The selection does
    /// not wrap, and `Q` is not quit. On the overview `Up` `Down` `j` `k` move
    /// the selection, so only `PageUp` `PageDown` `Home` scroll there; on every
    /// screen below it they scroll the body, and a screen change puts the new
    /// screen at its top (7c3 §3.1, 7g1), and `terminal` clamps the scroll to the
    /// body's length (M7-56). A reading changes nothing here: with one, every key
    /// behaves as without one (M7-64).
    pub fn handle(&mut self, key: Key) -> Action {
        // M7-43: `ctrl-c` quits from every screen, help open or not, and writes
        // nothing, exactly as `q` does.
        if key == Key::CtrlC {
            return Action::Quit;
        }
        if key == Key::Char('q') {
            return Action::Quit;
        }

        // M7-27: with the help legend open, `?` closes it, `esc` and `backspace`
        // close it, and nothing else moves the screen.
        if self.help {
            if matches!(key, Key::Char('?') | Key::Esc | Key::Backspace) {
                self.help = false;
            }
            return Action::Continue;
        }
        // `?` opens the help legend on every screen, Export included (M7-53).
        if key == Key::Char('?') {
            self.help = true;
            return Action::Continue;
        }

        // The only facts the app uses about the document (plan §3 rule 1).
        let n: usize = self.document.results.len();
        let has_problem: bool =
            self.document.error.is_some() || !self.document.clarifications.is_empty();
        let opened: Screen = self.screen;

        match (self.screen, key) {
            (Screen::Overview, Key::Esc) => return Action::Quit,
            (Screen::Overview, Key::Up | Key::Char('k')) => {
                self.selected = self.selected.saturating_sub(1);
            }
            (Screen::Overview, Key::Down | Key::Char('j')) => {
                if n > 0 {
                    self.selected = (self.selected + 1).min(n - 1);
                }
            }
            (Screen::Overview, Key::Enter) if n > 0 => self.screen = Screen::Member(self.selected),
            // 7g1: `x` opens the export screen, which says how to enable an
            // export when the command line gave no `--export` directory.
            (Screen::Overview, Key::Char('x')) => self.screen = Screen::Export,
            (Screen::Overview, Key::Char('e')) if has_problem => self.screen = Screen::Problem,
            // `PageUp` `PageDown` `Home` scroll on every screen, the overview
            // included (7c3 §3.1 rule 3).
            (Screen::Overview, Key::PageUp | Key::PageDown | Key::Home) => self.page(key),
            // Below the overview there is no selection to move, so `j` `k` and
            // the arrows scroll as well.
            (
                Screen::Member(_) | Screen::Details(_) | Screen::Problem,
                Key::PageUp | Key::PageDown | Key::Home,
            ) => self.page(key),
            (
                Screen::Member(_) | Screen::Details(_) | Screen::Problem,
                Key::Down | Key::Char('j'),
            ) => {
                self.scroll += 1;
            }
            (
                Screen::Member(_) | Screen::Details(_) | Screen::Problem,
                Key::Up | Key::Char('k'),
            ) => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            // M7-30: `d` opens the details of the open member; `t` does nothing.
            (Screen::Member(i), Key::Char('d')) => self.screen = Screen::Details(i),
            // `i` expands a long insertion, and only one that can be: on a short
            // edit, and on a member with no design, `i` does nothing (7i4).
            (Screen::Member(i), Key::Char('i')) => {
                if expandable(&self.document, i) {
                    self.expanded = !self.expanded;
                }
            }
            (Screen::Member(i), Key::Esc | Key::Backspace) => {
                self.screen = Screen::Overview;
                self.selected = i;
            }
            (Screen::Details(i), Key::Esc | Key::Backspace) => self.screen = Screen::Member(i),
            (Screen::Problem, Key::Esc | Key::Backspace) => self.screen = Screen::Overview,
            // M7-58: `w` and `o` write, and only when there is a plan to write
            // with, so `w` with no `--export` directory does nothing (no silent
            // action), and `y` and `n` do nothing at all. M7-49: when a name the
            // export would write exists, `w` opens the prompt that names it and
            // asks, and writes nothing until the prompt is answered.
            (Screen::Export, Key::Char('w') | Key::Char('o')) => {
                if self.export.is_some() {
                    if self.taken.is_empty() {
                        return Action::Export;
                    }
                    self.screen = Screen::Replace;
                }
            }
            (Screen::Export, Key::Esc | Key::Backspace) => {
                self.screen = Screen::Overview;
            }
            // M7-49: on the prompt, `y` replaces the names listed, and any
            // other key goes back to the export screen with the line saying
            // nothing was written. `ctrl-c` quits, as on every screen (M7-43).
            (Screen::Replace, Key::Char('y')) => return Action::ExportReplace,
            (Screen::Replace, _) => {
                self.screen = Screen::Export;
                self.export_message = Some("nothing was written".to_string());
            }
            _ => {}
        }
        // A screen change starts at the top of the new screen (7c3 §3.1 rule 4),
        // collapses an expanded insertion, so `d` and back both start the screen
        // with its marker (7i4), and closes the help legend (M7-27).
        if self.screen != opened {
            self.scroll = 0;
            self.expanded = false;
            self.help = false;
        }
        Action::Continue
    }

    /// One page key's effect on the scroll: `PageDown` adds 10, `PageUp`
    /// subtracts 10 and never goes below 0, `Home` is 0 (7c3 §3.1 rule 3).
    fn page(&mut self, key: Key) {
        self.scroll = match key {
            Key::PageDown => self.scroll.saturating_add(10),
            Key::PageUp => self.scroll.saturating_sub(10),
            Key::Home => 0,
            _ => self.scroll,
        };
    }
}

/// Whether the sequence screen of member `index` can be asked to write its
/// insertion out: the member exists, has a design, and its mutation is an
/// insertion of more than `MEMBER_INLINE_LIMIT` written bases, the length 7i3's
/// block collapses to a marker (7i4, plan §5.3a M7-19). Nothing is computed: only
/// the count of the bases the document names is read (plan §3 rule 1).
pub fn expandable(document: &ResultDocument, index: usize) -> bool {
    match document.results.get(index) {
        Some(member) => {
            member.design.is_some()
                && matches!(
                    &member.mutation,
                    MutationView::Insertion { written, .. }
                        if written.chars().count() > MEMBER_INLINE_LIMIT
                )
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::{Details, EditedView, ErrorView, MemberView, MutationView};

    /// A document, for the key rule to be tested on: the screen shows its
    /// state, and the key rule never reads it. Nothing scientific is here; the
    /// fields `run` fills from a prepared request are left empty.
    fn document() -> ResultDocument {
        ResultDocument {
            schema: crate::RESULT_SCHEMA,
            state: "verified",
            experimental_validation: "not_performed",
            provenance: crate::result::Provenance {
                package: crate::result::PackageView {
                    name: env!("CARGO_PKG_NAME"),
                    version: env!("CARGO_PKG_VERSION"),
                },
                algorithms: Vec::new(),
                request_sha256: None,
                operation: None,
                template: None,
                buffer: None,
            },
            request: None,
            target: None,
            clarifications: Vec::new(),
            candidates: Vec::new(),
            warnings: Vec::new(),
            results: Vec::new(),
            error: None,
        }
    }

    /// The document with `n` entries in `results`. The app never reads a
    /// member's content, so nothing in a member here is a scientific value.
    fn document_with(n: usize) -> ResultDocument {
        let mut document: ResultDocument = document();
        document.results = (0..n)
            .map(|_| MemberView {
                name: String::new(),
                mutation: MutationView::Substitution {
                    state: String::new(),
                    interval: (0, 0),
                    original: String::new(),
                    written: String::new(),
                    coordinates: Vec::new(),
                    edited: EditedView {
                        id: String::new(),
                        sha256: String::new(),
                        length: 0,
                    },
                },
                warnings: Vec::new(),
                design: None,
                design_failure: None,
            })
            .collect();
        document
    }

    /// The document with `error: Some(..)`, so the screen opens on the problem.
    fn document_with_error() -> ResultDocument {
        let mut document: ResultDocument = document();
        document.error = Some(ErrorView {
            code: "request_invalid",
            rule: None,
            category: "malformed_input",
            message: "an error, for the key rule to be tested on".to_string(),
            path: None,
            details: Details(vec![]),
        });
        document
    }

    /// The document with one clarification, so the screen opens on the problem.
    fn document_with_clarification() -> ResultDocument {
        let mut document: ResultDocument = document();
        document
            .clarifications
            .push("which CDS is meant".to_string());
        document
    }

    use crate::review::export::{export_plan, ExportPlan};
    use std::path::Path;

    /// The plan `main` builds from `--export out` for the request `set.json`,
    /// built by `export_plan`, which is pure and writes nothing.
    fn plan_of(document: &ResultDocument) -> ExportPlan {
        export_plan(document, Path::new("set.json"), Path::new("out"))
    }

    /// The app on the export screen, as the keys reach it, with a plan built
    /// from `document`.
    fn export_app(document: ResultDocument) -> App {
        let plan: ExportPlan = plan_of(&document);
        let mut app: App = App::new(document).with_export(Some(plan));
        app.handle(Key::Char('x'));
        app
    }

    #[test]
    fn q_and_esc_quit() {
        let mut app: App = App::new(document());
        assert_eq!(app.handle(Key::Char('q')), Action::Quit);
        let mut app: App = App::new(document());
        assert_eq!(app.handle(Key::Esc), Action::Quit);
    }

    #[test]
    fn other_keys_continue() {
        let mut app: App = App::new(document());
        assert_eq!(app.handle(Key::Char('x')), Action::Continue);
        assert_eq!(app.handle(Key::Char('Q')), Action::Continue);
        assert_eq!(app.handle(Key::Other), Action::Continue);
    }

    #[test]
    fn the_first_screen_is_the_overview_without_a_problem() {
        let app: App = App::new(document_with(2));
        assert_eq!(app.screen(), Screen::Overview);
        assert_eq!(app.selected(), 0);
    }

    #[test]
    fn an_error_opens_on_the_problem_screen() {
        let app: App = App::new(document_with_error());
        assert_eq!(app.screen(), Screen::Problem);
    }

    #[test]
    fn a_clarification_opens_on_the_problem_screen() {
        let app: App = App::new(document_with_clarification());
        assert_eq!(app.screen(), Screen::Problem);
    }

    #[test]
    fn selection_moves_and_stops_at_the_ends() {
        let mut app: App = App::new(document_with(3));
        assert_eq!(app.handle(Key::Up), Action::Continue);
        assert_eq!(app.selected(), 0);
        for _ in 0..5 {
            app.handle(Key::Down);
        }
        assert_eq!(app.selected(), 2);
        app.handle(Key::Char('k'));
        assert_eq!(app.selected(), 1);
        app.handle(Key::Char('j'));
        assert_eq!(app.selected(), 2);
    }

    #[test]
    fn selection_does_nothing_without_members() {
        let mut app: App = App::new(document_with(0));
        assert_eq!(app.handle(Key::Down), Action::Continue);
        assert_eq!(app.handle(Key::Enter), Action::Continue);
        assert_eq!(app.selected(), 0);
        assert_eq!(app.screen(), Screen::Overview);
    }

    #[test]
    fn enter_opens_the_selected_member_and_back_keeps_it() {
        let mut app: App = App::new(document_with(3));
        app.handle(Key::Down);
        assert_eq!(app.handle(Key::Enter), Action::Continue);
        assert_eq!(app.screen(), Screen::Member(1));
        assert_eq!(app.handle(Key::Char('d')), Action::Continue);
        assert_eq!(app.screen(), Screen::Details(1));
        assert_eq!(app.handle(Key::Char('t')), Action::Continue);
        assert_eq!(app.screen(), Screen::Details(1), "`t` does nothing (M7-30)");
        assert_eq!(app.handle(Key::Esc), Action::Continue);
        assert_eq!(app.screen(), Screen::Member(1));
        assert_eq!(app.handle(Key::Backspace), Action::Continue);
        assert_eq!(app.screen(), Screen::Overview);
        assert_eq!(app.selected(), 1);
    }

    #[test]
    fn e_opens_the_problem_screen_only_when_there_is_one() {
        let mut app: App = App::new(document_with(2));
        assert_eq!(app.handle(Key::Char('e')), Action::Continue);
        assert_eq!(app.screen(), Screen::Overview);

        let mut app: App = App::new(document_with_error());
        assert_eq!(app.handle(Key::Esc), Action::Continue);
        assert_eq!(app.screen(), Screen::Overview);
        assert_eq!(app.handle(Key::Char('e')), Action::Continue);
        assert_eq!(app.screen(), Screen::Problem);
    }

    // M7-43: `ctrl-c` quits from every screen the app has, help open or not, and
    // writes nothing.
    #[test]
    fn ctrl_c_quits_from_every_screen() {
        let screens: Vec<(ResultDocument, Vec<Key>)> = vec![
            (document_with(2), Vec::new()),
            (document_with(2), vec![Key::Enter]),
            (document_with(2), vec![Key::Enter, Key::Char('d')]),
            (document_with(2), vec![Key::Char('x')]),
            (document_with(2), vec![Key::Enter, Key::Char('?')]),
            (document_with_error(), Vec::new()),
            (document_with_error(), vec![Key::Esc, Key::Char('e')]),
        ];
        for (document, keys) in screens {
            let mut app: App = App::new(document);
            for key in keys {
                app.handle(key);
            }
            assert_eq!(
                app.handle(Key::CtrlC),
                Action::Quit,
                "`ctrl-c` quits from {:?}",
                app.screen()
            );
        }
    }

    #[test]
    fn q_quits_from_every_screen() {
        let screens: Vec<(ResultDocument, Vec<Key>)> = vec![
            (document_with(2), Vec::new()),
            (document_with(2), vec![Key::Enter]),
            (document_with(2), vec![Key::Enter, Key::Char('d')]),
            (document_with(2), vec![Key::Enter, Key::Char('?')]),
            (document_with_error(), Vec::new()),
            (document_with_error(), vec![Key::Esc, Key::Char('e')]),
        ];
        for (document, keys) in screens {
            let mut app: App = App::new(document);
            for key in keys {
                app.handle(key);
            }
            assert_eq!(app.handle(Key::Char('q')), Action::Quit);
        }
    }

    #[test]
    fn esc_goes_back_rather_than_quitting_below_the_overview() {
        let mut app: App = App::new(document_with(2));
        app.handle(Key::Enter);
        assert_eq!(app.screen(), Screen::Member(0));
        assert_eq!(app.handle(Key::Esc), Action::Continue);

        let mut app: App = App::new(document_with(2));
        app.handle(Key::Enter);
        app.handle(Key::Char('d'));
        assert_eq!(app.screen(), Screen::Details(0));
        assert_eq!(app.handle(Key::Esc), Action::Continue);

        let mut app: App = App::new(document_with_error());
        assert_eq!(app.screen(), Screen::Problem);
        assert_eq!(app.handle(Key::Esc), Action::Continue);
    }

    #[test]
    fn scrolling_moves_and_stops_at_zero() {
        // M7-52: the provenance screen is gone, so the scrolling is proved on
        // the details, the screen `d` opens.
        let mut app: App = App::new(document_with(2));
        app.handle(Key::Enter);
        assert_eq!(app.handle(Key::Char('d')), Action::Continue);
        assert_eq!(app.screen(), Screen::Details(0));
        assert_eq!(app.scroll(), 0);
        for _ in 0..3 {
            app.handle(Key::Down);
        }
        assert_eq!(app.scroll(), 3, "`Down` moves one line at a time");
        for _ in 0..5 {
            app.handle(Key::Up);
        }
        assert_eq!(app.scroll(), 0, "`Up` never takes the scroll below 0");
        app.handle(Key::PageUp);
        assert_eq!(app.scroll(), 0, "`PageUp` never goes below 0 either");
        app.handle(Key::PageDown);
        assert_eq!(app.scroll(), 10);
        app.handle(Key::Home);
        assert_eq!(app.scroll(), 0, "`Home` is the top");
        app.handle(Key::PageDown);
        app.handle(Key::PageDown);
        app.handle(Key::PageUp);
        assert_eq!(app.scroll(), 10, "`PageUp` subtracts 10");
    }

    #[test]
    fn changing_screen_resets_the_scroll() {
        let mut app: App = App::new(document_with(2));
        app.handle(Key::Enter);
        assert_eq!(app.screen(), Screen::Member(0));
        for _ in 0..3 {
            app.handle(Key::Char('j'));
        }
        assert_eq!(app.scroll(), 3);
        assert_eq!(app.handle(Key::Char('d')), Action::Continue);
        assert_eq!(app.screen(), Screen::Details(0));
        assert_eq!(app.scroll(), 0, "the new screen opens at its top");
        app.handle(Key::PageDown);
        assert_eq!(app.handle(Key::Esc), Action::Continue);
        assert_eq!(app.screen(), Screen::Member(0));
        assert_eq!(app.scroll(), 0, "going back opens the screen at its top");
    }

    #[test]
    fn overview_keys_still_move_the_selection() {
        // 7b's rule, proved with the scroll keys added: on the overview `j` `k`
        // and the arrows move the selection, and only the page keys scroll.
        let mut app: App = App::new(document_with(3));
        for key in [Key::Down, Key::Char('j'), Key::Char('k'), Key::Up] {
            app.handle(key);
        }
        assert_eq!(app.selected(), 0);
        assert_eq!(app.scroll(), 0, "the arrows do not scroll the overview");
        app.handle(Key::Down);
        app.handle(Key::Down);
        assert_eq!(app.selected(), 2);
        assert_eq!(app.scroll(), 0);
        app.handle(Key::PageDown);
        assert_eq!(app.scroll(), 10, "the page keys scroll the overview");
        assert_eq!(app.selected(), 2, "and do not move the selection");
    }

    #[test]
    fn x_opens_export_and_w_o_ask_only_with_a_plan() {
        // `x` opens the export screen whether or not a plan exists: the screen
        // explains how to enable an export when the command line gave none.
        let mut app: App = App::new(document_with(2));
        app.handle(Key::PageDown);
        assert_eq!(app.handle(Key::Char('x')), Action::Continue);
        assert_eq!(app.screen(), Screen::Export);
        assert!(
            app.export_plan().is_none(),
            "no --export directory, so no plan"
        );
        assert_eq!(app.scroll(), 0, "the new screen opens at its top");
        // M7-58: `w` and `o` with no plan ask for nothing, so nothing is written,
        // and `y` `n` do nothing at all, plan or not.
        for key in [
            Key::Char('w'),
            Key::Char('o'),
            Key::Char('y'),
            Key::Char('n'),
        ] {
            assert_eq!(app.handle(key), Action::Continue, "{:?}", key);
            assert_eq!(app.screen(), Screen::Export, "{:?}", key);
        }

        // With a plan, `w` and `o` ask for the write, and the screen stays for the
        // report (M7-58, 7g1).
        for key in [Key::Char('w'), Key::Char('o')] {
            let mut app: App = export_app(document_with(2));
            assert_eq!(app.screen(), Screen::Export);
            assert_eq!(app.handle(key), Action::Export, "`{:?}` writes", key);
            assert_eq!(
                app.screen(),
                Screen::Export,
                "the screen stays for the report"
            );
        }
        for key in [Key::Char('y'), Key::Char('n')] {
            let mut app: App = export_app(document_with(2));
            assert_eq!(
                app.handle(key),
                Action::Continue,
                "`{:?}` does nothing",
                key
            );
            assert_eq!(app.screen(), Screen::Export, "{:?}", key);
        }

        // `Esc` and `Backspace` go back the same way, and `q` quits.
        for key in [Key::Esc, Key::Backspace] {
            let mut app: App = export_app(document_with(2));
            assert_eq!(app.handle(key), Action::Continue);
            assert_eq!(app.screen(), Screen::Overview, "{:?}", key);
        }
        let mut app: App = export_app(document_with(2));
        assert_eq!(app.handle(Key::Char('q')), Action::Quit);
    }

    // M7-49: a name the export would write that exists opens the prompt naming
    // it, where `y` replaces them and any other key writes nothing. The names are
    // what the shell set with `set_taken`, so the app stays pure.
    #[test]
    fn a_taken_name_opens_the_replace_prompt() {
        // Nothing exists, so `w` and `o` write, as M7-58 says.
        for key in [Key::Char('w'), Key::Char('o')] {
            let mut app: App = export_app(document_with(2));
            assert!(app.taken().is_empty(), "the shell said nothing exists");
            assert_eq!(app.handle(key), Action::Export, "`{:?}` writes", key);
            assert_eq!(
                app.screen(),
                Screen::Export,
                "`{:?}` stays for the report",
                key
            );
        }

        // A name taken, so `w` and `o` open the prompt, which writes nothing
        // until it is answered.
        for key in [Key::Char('w'), Key::Char('o')] {
            let mut app: App = export_app(document_with(2));
            app.set_taken(vec!["sid-1_del-11.fa".to_string()]);
            assert_eq!(
                app.handle(key),
                Action::Continue,
                "`{:?}` asks, not writes",
                key
            );
            assert_eq!(
                app.screen(),
                Screen::Replace,
                "`{:?}` opens the prompt",
                key
            );
            assert_eq!(app.scroll(), 0, "the new screen opens at its top");
        }

        // `y` answers the prompt, asking for the replace of the names listed.
        let mut app: App = export_app(document_with(2));
        app.set_taken(vec!["sid-1_del-11.fa".to_string()]);
        app.handle(Key::Char('w'));
        assert_eq!(app.handle(Key::Char('y')), Action::ExportReplace);
        assert_eq!(
            app.screen(),
            Screen::Replace,
            "the prompt stays for the shell to write and report"
        );

        // Any other key goes back to the export screen, writing nothing.
        for key in [
            Key::Char('n'),
            Key::Esc,
            Key::Backspace,
            Key::Char('x'),
            Key::Other,
        ] {
            let mut app: App = export_app(document_with(2));
            app.set_taken(vec!["sid-1_del-11.fa".to_string()]);
            app.handle(Key::Char('w'));
            assert_eq!(
                app.handle(key),
                Action::Continue,
                "`{:?}` writes nothing",
                key
            );
            assert_eq!(app.screen(), Screen::Export, "`{:?}` goes back", key);
            assert_eq!(
                app.export_message(),
                Some("nothing was written"),
                "`{:?}` says nothing was written",
                key
            );
        }

        // `q` and `ctrl-c` quit from the prompt as on every screen, M7-43.
        for key in [Key::Char('q'), Key::CtrlC] {
            let mut app: App = export_app(document_with(2));
            app.set_taken(vec!["sid-1_del-11.fa".to_string()]);
            app.handle(Key::Char('w'));
            assert_eq!(app.handle(key), Action::Quit, "`{:?}` quits", key);
        }
    }

    #[test]
    fn export_done_reports_and_stays() {
        let mut app: App = export_app(document_with(2));
        app.export_done(Ok(vec![
            "del-11.fa".to_string(),
            "set.result.json".to_string(),
        ]));
        assert_eq!(
            app.screen(),
            Screen::Export,
            "the screen stays on the export"
        );
        assert_eq!(
            app.export_message(),
            Some("exported 2 files to out: del-11.fa, set.result.json"),
            "the names written, in the order written"
        );

        app.export_done(Err(
            "set.result.json already exists; luria never replaces a file; nothing was written"
                .to_string(),
        ));
        assert_eq!(app.screen(), Screen::Export);
        assert_eq!(
            app.export_message(),
            Some(
                "export failed: set.result.json already exists; luria never replaces a file; nothing was written"
            ),
            "the refusal's own line, on the screen that listed the files"
        );

        // Going back to the overview keeps the message; a screen change clears
        // the scroll and nothing else.
        assert_eq!(app.handle(Key::Esc), Action::Continue);
        assert_eq!(app.screen(), Screen::Overview);
        assert!(app.export_message().is_some());
    }

    #[test]
    fn help_opens_and_closes() {
        let mut app: App = App::new(document_with(2));
        app.handle(Key::Enter);
        assert_eq!(app.screen(), Screen::Member(0));
        assert!(!app.help(), "the screen opens with the legend closed");

        app.handle(Key::Char('?'));
        assert!(app.help(), "`?` opens the legend");
        app.handle(Key::Char('j'));
        app.handle(Key::PageDown);
        assert_eq!(app.scroll(), 0, "no key scrolls while help is open");
        app.handle(Key::Char('d'));
        assert_eq!(
            app.screen(),
            Screen::Member(0),
            "and no key moves the screen while help is open"
        );
        assert!(app.help(), "help stays open");

        assert_eq!(app.handle(Key::Esc), Action::Continue);
        assert!(!app.help(), "`esc` closes help");
        assert_eq!(
            app.screen(),
            Screen::Member(0),
            "`esc` closes help and stays on the screen, not back"
        );

        // `q` quits with help open, on the overview and below it.
        for keys in [vec![Key::Char('?')], vec![Key::Enter, Key::Char('?')]] {
            let mut app: App = App::new(document_with(2));
            for key in keys {
                app.handle(key);
            }
            assert!(app.help(), "with help open");
            assert_eq!(app.handle(Key::Char('q')), Action::Quit, "`q` quits");
        }

        // M7-53: `?` opens help on the export screen too, `esc` and `?` close it,
        // and a screen change keeps help closed.
        let mut app: App = App::new(document_with(2));
        app.handle(Key::Char('x'));
        app.handle(Key::Char('?'));
        assert!(app.help(), "`?` opens help on the export screen");
        app.handle(Key::Char('w'));
        assert!(
            app.help() && app.screen() == Screen::Export,
            "no key moves the screen while help is open"
        );
        app.handle(Key::Esc);
        assert!(!app.help(), "`esc` closes help");
        assert_eq!(
            app.screen(),
            Screen::Export,
            "`esc` closes help and stays on the screen"
        );
        app.handle(Key::Char('?'));
        app.handle(Key::Char('?'));
        assert!(!app.help(), "`?` closes help as well");
        app.handle(Key::Esc);
        assert_eq!(app.screen(), Screen::Overview, "then `esc` goes back");
        app.handle(Key::Char('?'));
        app.handle(Key::Char('x'));
        assert!(app.help(), "no key moves the screen while help is open");
        app.handle(Key::Esc);
        app.handle(Key::Char('x'));
        assert_eq!(
            app.screen(),
            Screen::Export,
            "a screen change moves the screen"
        );
        assert!(!app.help(), "and keeps help closed");
    }
}
