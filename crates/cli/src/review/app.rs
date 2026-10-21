//! The state of the review screen (milestone 7b, 7c3): the result document,
//! which screen is open, which member is highlighted, how far the open screen is
//! scrolled, and what each key does. No I/O
//! and no terminal types are here, so the rule is testable without a terminal
//! (plan §5.1); `terminal` maps crossterm's events to `Key` and carries out the
//! `Action`. The screens themselves are drawn by `view` (7c, 7d), which reads
//! `screen()`, `selected()` and `scroll()`. Nothing scientific is computed: the app reads
//! only how many entries `results` has, whether `error` is `Some`, and whether
//! `clarifications` is empty (plan §3 rule 1).

use crate::result::ResultDocument;

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
    Esc,
    Other,
}

/// What the app asks its shell to do after a key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Continue,
    Quit,
}

/// Which screen is open. A member index is an index into
/// `document.results`, always in range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Overview,
    Member(usize),
    Context(usize),
    Provenance,
    Problem,
}

/// The screen's state: the result document to show, the open screen, the member
/// highlighted on the overview, and how far the open screen is scrolled.
#[derive(Clone, Debug, PartialEq)]
pub struct App {
    document: ResultDocument,
    screen: Screen,
    selected: usize,
    scroll: usize,
    template: Option<Vec<char>>,
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
        }
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
    /// many lines a screen has, so this has no upper bound: `view` clamps it
    /// to the screen's own lines (7c3 §3.1 rule 5).
    pub fn scroll(&self) -> usize {
        self.scroll
    }

    /// `q` quits from every screen. Every other key follows the table of §3.3:
    /// `Esc` and `Backspace` go back, `Enter` opens the selected member, `j`
    /// `k` `c` `p` `e` move, and any other key changes nothing. The selection
    /// does not wrap, and `Q` is not quit. On the overview `Up` `Down` `j` `k`
    /// move the selection, so only `PageUp` `PageDown` `Home` scroll there; on
    /// every screen below it they scroll the body, and a screen change puts the
    /// new screen at its top (7c3 §3.1).
    pub fn handle(&mut self, key: Key) -> Action {
        if key == Key::Char('q') {
            return Action::Quit;
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
            (Screen::Overview, Key::Char('p')) => self.screen = Screen::Provenance,
            (Screen::Overview, Key::Char('e')) if has_problem => self.screen = Screen::Problem,
            // `PageUp` `PageDown` `Home` scroll on every screen, the overview
            // included (7c3 §3.1 rule 3).
            (Screen::Overview, Key::PageUp | Key::PageDown | Key::Home) => self.page(key),
            // Below the overview there is no selection to move, so `j` `k` and
            // the arrows scroll as well.
            (
                Screen::Member(_) | Screen::Context(_) | Screen::Provenance | Screen::Problem,
                Key::PageUp | Key::PageDown | Key::Home,
            ) => self.page(key),
            (
                Screen::Member(_) | Screen::Context(_) | Screen::Provenance | Screen::Problem,
                Key::Down | Key::Char('j'),
            ) => {
                self.scroll += 1;
            }
            (
                Screen::Member(_) | Screen::Context(_) | Screen::Provenance | Screen::Problem,
                Key::Up | Key::Char('k'),
            ) => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            (Screen::Member(i), Key::Char('c')) => self.screen = Screen::Context(i),
            (Screen::Member(i), Key::Esc | Key::Backspace) => {
                self.screen = Screen::Overview;
                self.selected = i;
            }
            (Screen::Context(i), Key::Esc | Key::Backspace) => self.screen = Screen::Member(i),
            (Screen::Provenance, Key::Esc | Key::Backspace) => self.screen = Screen::Overview,
            (Screen::Problem, Key::Esc | Key::Backspace) => self.screen = Screen::Overview,
            (Screen::Problem, Key::Char('p')) => self.screen = Screen::Provenance,
            _ => {}
        }
        // A screen change starts at the top of the new screen (7c3 §3.1 rule 4).
        if self.screen != opened {
            self.scroll = 0;
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
        assert_eq!(app.handle(Key::Char('c')), Action::Continue);
        assert_eq!(app.screen(), Screen::Context(1));
        assert_eq!(app.handle(Key::Esc), Action::Continue);
        assert_eq!(app.screen(), Screen::Member(1));
        assert_eq!(app.handle(Key::Backspace), Action::Continue);
        assert_eq!(app.screen(), Screen::Overview);
        assert_eq!(app.selected(), 1);
    }

    #[test]
    fn provenance_opens_and_closes() {
        let mut app: App = App::new(document_with(2));
        assert_eq!(app.handle(Key::Char('p')), Action::Continue);
        assert_eq!(app.screen(), Screen::Provenance);
        assert_eq!(app.handle(Key::Esc), Action::Continue);
        assert_eq!(app.screen(), Screen::Overview);

        let mut app: App = App::new(document_with_error());
        assert_eq!(app.screen(), Screen::Problem);
        assert_eq!(app.handle(Key::Char('p')), Action::Continue);
        assert_eq!(app.screen(), Screen::Provenance);
        assert_eq!(app.handle(Key::Esc), Action::Continue);
        assert_eq!(app.screen(), Screen::Overview);
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

    #[test]
    fn q_quits_from_every_screen() {
        let screens: Vec<(ResultDocument, Vec<Key>)> = vec![
            (document_with(2), Vec::new()),
            (document_with(2), vec![Key::Enter]),
            (document_with(2), vec![Key::Enter, Key::Char('c')]),
            (document_with(2), vec![Key::Char('p')]),
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
        app.handle(Key::Char('c'));
        assert_eq!(app.screen(), Screen::Context(0));
        assert_eq!(app.handle(Key::Esc), Action::Continue);

        let mut app: App = App::new(document_with(2));
        app.handle(Key::Char('p'));
        assert_eq!(app.screen(), Screen::Provenance);
        assert_eq!(app.handle(Key::Esc), Action::Continue);

        let mut app: App = App::new(document_with_error());
        assert_eq!(app.screen(), Screen::Problem);
        assert_eq!(app.handle(Key::Esc), Action::Continue);
    }

    #[test]
    fn scrolling_moves_and_stops_at_zero() {
        let mut app: App = App::new(document_with(2));
        assert_eq!(app.handle(Key::Char('p')), Action::Continue);
        assert_eq!(app.screen(), Screen::Provenance);
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
        assert_eq!(app.handle(Key::Char('c')), Action::Continue);
        assert_eq!(app.screen(), Screen::Context(0));
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
}
